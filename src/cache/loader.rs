//! A priority aware background decode pool.
//!
//! Which image gets decoded next matters more than raw throughput: the one the
//! user is about to look at must jump the queue ahead of the twenty that were
//! requested a moment earlier. Workers therefore pull from a priority queue and
//! drop requests that the viewer has since navigated away from.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crate::decoder::{self, DecodeError, DecodeOptions, DecodedImage};

pub use super::focus::Focus;

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

/// One image to decode, and everything needed to decide whether it still
/// should be by the time a worker reaches it.
pub struct Job {
    pub key: ImageKey,
    /// A rank; lower runs first.
    pub priority: usize,
    /// How far the viewer may move before this is no longer worth decoding.
    /// `None` follows the store's preload window.
    pub radius: Option<usize>,
    pub path: PathBuf,
    pub options: DecodeOptions,
    /// The store's view of what is still worth decoding. Held per job so
    /// several stores can share one pool.
    pub focus: Arc<Focus>,
    pub responder: Sender<LoadResult>,
}

/// One unit of work, as the queue holds it.
struct Request {
    job: Job,
}

// The heap orders by priority alone; ties are broken arbitrarily, which is
// fine because equally distant images are equally urgent.
impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        self.job.priority == other.job.priority
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
        self.job.priority.cmp(&other.job.priority)
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

    /// Queues an image for decoding.
    pub fn submit(&self, job: Job) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.pending.push(Reverse(Request { job }));
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

        let job = request.job;

        // The viewer may have moved on while this job sat in the queue.
        let outcome = if job.focus.accepts(job.key, job.radius) {
            match decode_without_dying(&job.path, &job.options) {
                Ok(image) => Loaded::Decoded(image),
                Err(e) => {
                    tracing::warn!("{} {e}", job.path.display());
                    Loaded::Failed(e)
                }
            }
        } else {
            tracing::trace!("Abandoning {}", job.path.display());
            Loaded::Abandoned
        };

        // A closed channel just means the store was dropped or replaced.
        let _ = job.responder.send(LoadResult {
            key: job.key,
            outcome,
        });
    }
}

/// Decodes, turning a panic into a failed image rather than a lost worker.
///
/// Decoders are handed bytes off a disk, and a malformed file that panics one
/// of them used to take the whole thread with it: the pool shrank by one for
/// the rest of the session, and the image sat on a spinner for ever because
/// nothing was ever sent back for it.
fn decode_without_dying(
    path: &std::path::Path,
    options: &DecodeOptions,
) -> Result<DecodedImage, DecodeError> {
    let decoded = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        decoder::load(path, options)
    }));

    match decoded {
        Ok(result) => result,
        Err(_) => Err(DecodeError::Unsupported(
            "the decoder gave up on this file".to_string(),
        )),
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
/// Decoding a photograph is not compute bound past a handful of threads: a 24
/// megapixel image is a hundred megabytes of output, and the decoders saturate
/// memory bandwidth long before they run out of cores. Measured on a 24 core
/// machine, eight workers sustained 42 images a second and twelve sustained
/// 39, while each worker holding a whole decoded image cost another 130MB of
/// peak memory. `decode_threads` overrides this either way.
const MAX_DEFAULT_WORKERS: usize = 8;

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

    fn key(generation: u64, index: usize) -> ImageKey {
        ImageKey { generation, index }
    }
    use std::sync::mpsc::channel;

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
        let wanted = key(1, 0);
        loader.submit(Job {
            key: wanted,
            priority: 0,
            radius: None,
            path: path.clone(),
            options: DecodeOptions::new(Arc::from("srgb")),
            focus,
            responder: tx,
        });

        let result = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("worker produced a result");

        assert_eq!(result.key, wanted);
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
        loader.submit(Job {
            key: key(1, 0),
            priority: 0,
            radius: None,
            path: PathBuf::from("never-read.png"),
            options: DecodeOptions::new(Arc::from("srgb")),
            focus,
            responder: tx,
        });

        // Abandoned rather than silently dropped, so the store learns it may
        // ask again.
        let result = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("worker reported back");

        assert!(matches!(result.outcome, Loaded::Abandoned));
    }
}
