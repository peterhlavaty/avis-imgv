//! A priority aware background decode pool.
//!
//! Which image gets decoded next matters more than raw throughput: the one the
//! user is about to look at must jump the queue ahead of the twenty that were
//! requested a moment earlier. Workers therefore pull from a priority queue and
//! drop requests that the viewer has since navigated away from.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crate::decoder::{self, DecodeError, DecodeOptions, DecodedImage};

use super::policy::distance;

/// Identifies a request across the queue, the workers and the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageKey {
    /// Bumped whenever the open collection changes, invalidating in-flight
    /// work for the previous one.
    pub generation: u64,
    pub index: usize,
}

/// What became of one request.
pub enum Loaded {
    Decoded(DecodedImage),
    /// Tried and could not be. Not worth trying again until something changes.
    Failed(DecodeError),
    /// The viewer moved on before a worker picked it up, so nothing was
    /// decoded and nothing is wrong.
    ///
    /// This has to be reported rather than dropped: the store remembers what
    /// it has asked for so it does not ask twice, and an answer that never
    /// arrives would leave the image marked as loading for good.
    Abandoned,
}

/// What became of one request, and which one it was.
pub struct LoadResult {
    pub key: ImageKey,
    pub outcome: Loaded,
}

/// What the viewer currently cares about, shared with the workers so they can
/// abandon requests that went stale while queued.
#[derive(Debug, Default)]
pub struct Focus {
    generation: AtomicUsize,
    cursor: AtomicUsize,
    total: AtomicUsize,
    window: AtomicUsize,
}

impl Focus {
    /// Records the collection the viewer moved to.
    pub fn set_collection(&self, generation: u64, total: usize) {
        self.generation
            .store(generation as usize, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);
    }

    /// Records where the viewer is and how far around it work is still useful.
    pub fn set_position(&self, cursor: usize, window: usize) {
        self.cursor.store(cursor, Ordering::Relaxed);
        self.window.store(window, Ordering::Relaxed);
    }

    /// Whether a queued request is still worth decoding.
    pub fn accepts(&self, key: ImageKey) -> bool {
        if key.generation as usize != self.generation.load(Ordering::Relaxed) {
            return false;
        }

        let total = self.total.load(Ordering::Relaxed);
        let window = self.window.load(Ordering::Relaxed);

        window == 0
            || total == 0
            || distance(self.cursor.load(Ordering::Relaxed), key.index, total) <= window
    }
}

/// One unit of work.
struct Request {
    key: ImageKey,
    priority: usize,
    path: PathBuf,
    options: DecodeOptions,
    /// The store's view of what is still worth decoding. Held per request so
    /// several stores can share one pool.
    focus: Arc<Focus>,
    responder: Sender<LoadResult>,
}

// The heap orders by priority alone; ties are broken arbitrarily, which is
// fine because equally distant images are equally urgent.
impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
impl Eq for Request {}
impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Request {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

#[derive(Default)]
struct Queue {
    pending: BinaryHeap<Reverse<Request>>,
    shutdown: bool,
}

struct Shared {
    queue: Mutex<Queue>,
    ready: Condvar,
}

/// A pool of decode workers fed by a shared priority queue.
pub struct Loader {
    shared: Arc<Shared>,
    workers: Vec<JoinHandle<()>>,
}

impl Loader {
    /// Starts `worker_count` threads. Zero means "pick a sensible number".
    pub fn new(worker_count: usize) -> Loader {
        let worker_count = if worker_count == 0 {
            default_worker_count()
        } else {
            worker_count
        };

        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue::default()),
            ready: Condvar::new(),
        });

        tracing::info!("Starting {worker_count} decode workers");

        let workers = (0..worker_count)
            .map(|i| {
                let shared = Arc::clone(&shared);
                thread::Builder::new()
                    .name(format!("avis-decode-{i}"))
                    .spawn(move || worker_loop(&shared))
                    .expect("decode worker thread")
            })
            .collect();

        Loader { shared, workers }
    }

    /// Queues an image for decoding. `priority` is a rank; lower runs first.
    pub fn submit(
        &self,
        key: ImageKey,
        priority: usize,
        path: PathBuf,
        options: DecodeOptions,
        focus: Arc<Focus>,
        responder: Sender<LoadResult>,
    ) {
        let request = Request {
            key,
            priority,
            path,
            options,
            focus,
            responder,
        };

        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.pending.push(Reverse(request));
            self.shared.ready.notify_one();
        }
    }

    /// Drops every queued request, used when the open collection changes.
    pub fn clear(&self) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.pending.clear();
        }
    }

    /// Whether the queue has been drained. Workers may still be finishing the
    /// requests they already picked up.
    pub fn is_queue_empty(&self) -> bool {
        self.shared
            .queue
            .lock()
            .map(|queue| queue.pending.is_empty())
            .unwrap_or(true)
    }
}

impl Drop for Loader {
    fn drop(&mut self) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.shutdown = true;
            queue.pending.clear();
        }
        self.shared.ready.notify_all();

        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

fn worker_loop(shared: &Shared) {
    loop {
        let Some(request) = next_request(shared) else {
            return;
        };

        // The viewer may have moved on while this request sat in the queue.
        let outcome = if request.focus.accepts(request.key) {
            match decoder::load(&request.path, &request.options) {
                Ok(image) => Loaded::Decoded(image),
                Err(e) => {
                    tracing::warn!("{} {e}", request.path.display());
                    Loaded::Failed(e)
                }
            }
        } else {
            tracing::trace!("Abandoning {}", request.path.display());
            Loaded::Abandoned
        };

        // A closed channel just means the store was dropped or replaced.
        let _ = request.responder.send(LoadResult {
            key: request.key,
            outcome,
        });
    }
}

/// Blocks until there is work, or returns `None` once the pool shuts down.
fn next_request(shared: &Shared) -> Option<Request> {
    let mut queue = shared.queue.lock().ok()?;

    loop {
        if queue.shutdown {
            return None;
        }

        if let Some(Reverse(request)) = queue.pending.pop() {
            return Some(request);
        }

        queue = shared.ready.wait(queue).ok()?;
    }
}

/// Ceiling on the default worker count.
///
/// Every worker holds a whole decoded image while it works — about 130MB for a
/// 24 megapixel photograph — so on a 24 core machine an unbounded pool would
/// briefly need several gigabytes. Twelve is a compromise; `decode_threads`
/// overrides it either way.
const MAX_DEFAULT_WORKERS: usize = 12;

/// Leaves a core for the UI thread so navigation stays responsive while a
/// folder is being read.
fn default_worker_count() -> usize {
    thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).clamp(1, MAX_DEFAULT_WORKERS))
        .unwrap_or(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn focus_rejects_other_generations() {
        let focus = Focus::default();
        focus.set_collection(7, 100);
        focus.set_position(0, 10);

        assert!(focus.accepts(ImageKey {
            generation: 7,
            index: 3
        }));
        assert!(!focus.accepts(ImageKey {
            generation: 6,
            index: 3
        }));
    }

    #[test]
    fn focus_rejects_indices_outside_the_window() {
        let focus = Focus::default();
        focus.set_collection(1, 100);
        focus.set_position(50, 5);

        assert!(focus.accepts(ImageKey {
            generation: 1,
            index: 55
        }));
        assert!(!focus.accepts(ImageKey {
            generation: 1,
            index: 70
        }));
    }

    #[test]
    fn a_zero_window_accepts_everything() {
        let focus = Focus::default();
        focus.set_collection(1, 100);
        focus.set_position(0, 0);

        assert!(focus.accepts(ImageKey {
            generation: 1,
            index: 99
        }));
    }

    #[test]
    fn decodes_submitted_work() {
        use crate::decoder::test_support::encode;
        use image::ImageFormat;

        let dir = std::env::temp_dir().join("avis-loader-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("loader.png");
        std::fs::write(&path, encode(4, 4, [9, 9, 9, 255], ImageFormat::Png)).unwrap();

        let focus = Arc::new(Focus::default());
        focus.set_collection(1, 1);
        let loader = Loader::new(2);

        let (tx, rx) = channel();
        let key = ImageKey {
            generation: 1,
            index: 0,
        };
        loader.submit(
            key,
            0,
            path.clone(),
            DecodeOptions::new(Arc::from("srgb")),
            focus,
            tx,
        );

        let result = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("worker produced a result");

        assert_eq!(result.key, key);
        match result.outcome {
            Loaded::Decoded(image) => assert_eq!(image.size(), [4, 4]),
            _ => panic!("the image should have been decoded"),
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_stale_request_is_reported_rather_than_decoded() {
        let focus = Arc::new(Focus::default());
        focus.set_collection(2, 1);
        let loader = Loader::new(1);

        let (tx, rx) = channel();
        loader.submit(
            ImageKey {
                generation: 1,
                index: 0,
            },
            0,
            PathBuf::from("never-read.png"),
            DecodeOptions::new(Arc::from("srgb")),
            focus,
            tx,
        );

        // Abandoned rather than silently dropped, so the store learns it may
        // ask again.
        let result = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("worker reported back");

        assert!(matches!(result.outcome, Loaded::Abandoned));
    }
}
