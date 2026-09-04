//! A priority aware background decode pool.
//!
//! Which image gets decoded next matters more than raw throughput: the one the
//! user is about to look at must jump the queue ahead of the twenty that were
//! requested a moment earlier. Workers therefore pull from a priority queue and
//! drop requests that the viewer has since navigated away from.

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::decoder::{self, DecodeError, DecodeOptions, DecodedImage};
use crate::work::{OnShutdown, Pool, Ranked};

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

// `Ranked` hands back the least first, and the job's order is its priority:
// distance from the cursor. Ties are broken arbitrarily, which is right —
// equally distant photographs are equally urgent.
impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
impl Eq for Job {}
impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Job {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// A pool of decode workers fed by a shared priority queue.
pub struct Loader {
    pool: Pool<Ranked<Job>>,
}

impl Loader {
    /// Starts `worker_count` threads. Zero means "pick a sensible number".
    ///
    /// The number arrives from the configuration file, so the cap is here
    /// rather than in `Config`: the file keeps what it says and the consumer
    /// refuses to act on the impossible part of it.
    pub fn new(worker_count: usize) -> Loader {
        let worker_count = worker_count.min(MAX_WORKERS);
        let worker_count = if worker_count == 0 {
            default_worker_count()
        } else {
            worker_count
        };

        tracing::info!("Starting {worker_count} decode workers");

        // Dropped rather than finished on the way out: a queued decode is a
        // photograph nobody is waiting for any more.
        let pool = Pool::new("avis-decode", worker_count, OnShutdown::Drop, decode);

        Loader { pool }
    }

    /// How many decode threads are actually running.
    pub fn worker_count(&self) -> usize {
        self.pool.workers()
    }

    /// Queues an image for decoding.
    pub fn submit(&self, job: Job) {
        self.pool.submit(job);
    }

    /// Drops every queued request, used when the open collection changes.
    pub fn clear(&self) {
        self.pool.clear();
    }
}

/// One photograph: decode it unless the viewer has moved on, and send the
/// answer back.
fn decode(job: Job) {
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
        Err(_) => Err(DecodeError::Panicked),
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

/// The most threads a configuration file can ask for.
///
/// Far past anything useful — the measurements above say eight — and low
/// enough that the pool starts. What it is really defending against is a typo.
pub const MAX_WORKERS: usize = 64;

/// Leaves a core for the UI thread so navigation stays responsive while a
/// folder is being read.
fn default_worker_count() -> usize {
    std::thread::available_parallelism()
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

    /// A typo in the configuration file used to be a panic with no message.
    #[test]
    fn a_thousand_threads_is_a_pool_that_starts() {
        let loader = Loader::new(1000);

        assert_eq!(loader.worker_count(), MAX_WORKERS);
    }

    #[test]
    fn zero_threads_picks_a_number() {
        let loader = Loader::new(0);

        assert!(loader.worker_count() >= 1);
        assert!(loader.worker_count() <= MAX_DEFAULT_WORKERS);
    }
}
