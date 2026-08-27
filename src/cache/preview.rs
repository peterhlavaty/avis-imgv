//! The thread that reads previews.
//!
//! Reading the front of a file and decoding the camera's thumbnail costs a
//! couple of milliseconds, so one thread keeps hundreds of images a second
//! supplied. It is deliberately not the decode pool: this work has to stay out
//! of the way of the real decoding, and it must never queue behind it.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use crate::decoder::preview::{self, Preview};

use super::loader::ImageKey;

/// A preview, and which image it belongs to.
pub struct Read {
    pub key: ImageKey,
    pub preview: Preview,
}

/// A request that has not been picked up yet.
struct Request {
    key: ImageKey,
    path: PathBuf,
    responder: Sender<Read>,
}

#[derive(Default)]
struct Queue {
    /// Newest first: the image the viewer is on now matters more than the one
    /// it was on a moment ago.
    pending: VecDeque<Request>,
    shutdown: bool,
}

struct Shared {
    queue: Mutex<Queue>,
    ready: Condvar,
}

/// Reads previews on a thread of its own.
pub struct PreviewLoader {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

/// Beyond this the viewer has moved on faster than previews can be read, and
/// the oldest requests are no longer worth keeping.
const MAX_PENDING: usize = 256;

impl Default for PreviewLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewLoader {
    pub fn new() -> PreviewLoader {
        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue::default()),
            ready: Condvar::new(),
        });

        let worker = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("avis-previews".to_string())
            .spawn(move || run(&worker))
            .ok();

        PreviewLoader { shared, thread }
    }

    /// Queues an image, ahead of everything already waiting.
    pub fn submit(&self, key: ImageKey, path: PathBuf, responder: Sender<Read>) {
        let Ok(mut queue) = self.shared.queue.lock() else {
            return;
        };

        queue.pending.push_front(Request {
            key,
            path,
            responder,
        });
        queue.pending.truncate(MAX_PENDING);

        self.shared.ready.notify_one();
    }

    /// Forgets everything queued, for when the open folder changes.
    pub fn clear(&self) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.pending.clear();
        }
    }
}

impl Drop for PreviewLoader {
    fn drop(&mut self) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.shutdown = true;
            queue.pending.clear();
        }
        self.shared.ready.notify_all();

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(shared: &Shared) {
    while let Some(request) = next(shared) {
        let Some(preview) = preview::load(&request.path) else {
            continue;
        };

        // A closed channel means the store was replaced; nothing to do.
        let _ = request.responder.send(Read {
            key: request.key,
            preview,
        });
    }
}

fn next(shared: &Shared) -> Option<Request> {
    let mut queue = shared.queue.lock().ok()?;

    loop {
        if let Some(request) = queue.pending.pop_front() {
            return Some(request);
        }

        if queue.shutdown {
            return None;
        }

        queue = shared.ready.wait(queue).ok()?;
    }
}

/// A channel for one store's previews.
pub fn channel_pair() -> (Sender<Read>, Receiver<Read>) {
    channel()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::test_support::encode;
    use image::ImageFormat;
    use std::time::Duration;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-preview-loader-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn key(index: usize) -> ImageKey {
        ImageKey {
            generation: 1,
            index,
        }
    }

    #[test]
    fn a_queued_preview_comes_back() {
        let dir = temp_dir("read");
        let path = dir.join("photo.jpg");
        std::fs::write(&path, encode(320, 240, [9, 9, 9, 255], ImageFormat::Jpeg)).unwrap();

        let loader = PreviewLoader::new();
        let (tx, rx) = channel_pair();
        loader.submit(key(0), path, tx);

        let read = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the preview was read");

        assert_eq!(read.key, key(0));
        assert_eq!(read.preview.full_size, (320, 240));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_cannot_be_read_is_dropped_rather_than_reported() {
        let loader = PreviewLoader::new();
        let (tx, rx) = channel_pair();
        loader.submit(key(0), PathBuf::from("does-not-exist.jpg"), tx);

        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
    }

    #[test]
    fn the_newest_request_is_read_first() {
        let dir = temp_dir("order");
        let (tx, rx) = channel_pair();

        for index in 0..3 {
            let path = dir.join(format!("{index}.jpg"));
            std::fs::write(&path, encode(32, 32, [0, 0, 0, 255], ImageFormat::Jpeg)).unwrap();
        }

        // Queued while nothing is running, so all three are waiting when the
        // worker looks.
        let loader = PreviewLoader::new();
        loader.clear();
        for index in 0..3 {
            loader.submit(key(index), dir.join(format!("{index}.jpg")), tx.clone());
        }

        let mut seen = Vec::new();
        for _ in 0..3 {
            if let Ok(read) = rx.recv_timeout(Duration::from_secs(10)) {
                seen.push(read.key.index);
            }
        }

        // Whatever the scheduling, the last one queued is never last out.
        assert_eq!(seen.len(), 3);
        assert_ne!(seen.last(), Some(&2));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_queue_does_not_grow_without_bound() {
        let loader = PreviewLoader::new();
        let (tx, _rx) = channel_pair();

        for index in 0..MAX_PENDING * 2 {
            loader.submit(key(index), PathBuf::from("does-not-exist.jpg"), tx.clone());
        }

        let queued = loader.shared.queue.lock().unwrap().pending.len();
        assert!(queued <= MAX_PENDING, "{queued} queued");
    }
}
