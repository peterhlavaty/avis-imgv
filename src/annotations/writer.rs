//! The thread that persists annotations.
//!
//! Rating an image is a keystroke; it must not wait on a disk that happens to
//! be a network share. Saves are queued instead, and a run of changes to the
//! same image collapses into one write.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use crate::metadata::xmp::Xmp;

use super::sidecar;

/// A queue of pending saves, keyed so repeated edits to one image coalesce.
#[derive(Default)]
struct Queue {
    pending: HashMap<PathBuf, Xmp>,
    /// Number of saves started but not yet finished, so a flush knows to wait.
    in_flight: usize,
    shutdown: bool,
    /// Saves that failed, waiting to be told to the user.
    ///
    /// A rating is a keystroke and the disk it lands on may be a read-only
    /// card. Losing that quietly is worse than the write failing.
    problems: Vec<String>,
}

struct Shared {
    queue: Mutex<Queue>,
    /// Signalled when work arrives, and again when the queue drains.
    changed: Condvar,
}

/// Queues annotation saves onto a background thread.
pub struct Writer {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer {
    pub fn new() -> Writer {
        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue::default()),
            changed: Condvar::new(),
        });

        let worker = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("avis-annotations".to_string())
            .spawn(move || run(&worker))
            .ok();

        Writer { shared, thread }
    }

    /// Queues `annotations` to be written beside `image`.
    pub fn save(&self, image: PathBuf, annotations: Xmp) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.pending.insert(image, annotations);
            self.shared.changed.notify_all();
        }
    }

    /// Takes whatever failed since this was last asked, for the user to see.
    pub fn problems(&self) -> Vec<String> {
        match self.shared.queue.lock() {
            Ok(mut queue) => std::mem::take(&mut queue.problems),
            Err(_) => Vec::new(),
        }
    }

    /// Blocks until the queue is empty and no save is in flight.
    pub fn flush(&self) {
        let Ok(mut queue) = self.shared.queue.lock() else {
            return;
        };

        while !queue.pending.is_empty() || queue.in_flight > 0 {
            let Ok(waited) = self.shared.changed.wait(queue) else {
                return;
            };
            queue = waited;
        }
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        if let Ok(mut queue) = self.shared.queue.lock() {
            queue.shutdown = true;
        }
        self.shared.changed.notify_all();

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(shared: &Shared) {
    while let Some((image, annotations)) = next(shared) {
        let outcome = sidecar::write(&image, &annotations);

        if let Err(e) = &outcome {
            tracing::error!(
                "Failure writing {} -> {e}",
                sidecar::path_for(&image).display()
            );
        }

        if let Ok(mut queue) = shared.queue.lock() {
            queue.in_flight -= 1;

            if let Err(e) = outcome {
                let name = image
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| image.display().to_string());

                queue.problems.push(format!("Could not save {name}: {e}"));
            }

            shared.changed.notify_all();
        }
    }
}

/// Takes the next save, blocking until there is one or the writer shuts down.
fn next(shared: &Shared) -> Option<(PathBuf, Xmp)> {
    let mut queue = shared.queue.lock().ok()?;

    loop {
        if let Some(image) = queue.pending.keys().next().cloned() {
            let annotations = queue.pending.remove(&image)?;
            queue.in_flight += 1;

            return Some((image, annotations));
        }

        if queue.shutdown {
            return None;
        }

        queue = shared.changed.wait(queue).ok()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-writer-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_queued_save_reaches_the_disk() {
        let dir = temp_dir("save");
        let image = dir.join("photo.jpg");

        let writer = Writer::new();
        writer.save(
            image.clone(),
            Xmp {
                rating: 4,
                keywords: vec!["Keeper".to_string()],
            },
        );
        writer.flush();

        let back = sidecar::read(&image).expect("sidecar written");
        assert_eq!(back.rating, 4);
        assert_eq!(back.keywords, vec!["Keeper"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn repeated_edits_leave_the_last_one() {
        let dir = temp_dir("coalesce");
        let image = dir.join("photo.jpg");

        let writer = Writer::new();
        for rating in 1..=5 {
            writer.save(
                image.clone(),
                Xmp {
                    rating,
                    keywords: vec![],
                },
            );
        }
        writer.flush();

        assert_eq!(sidecar::read(&image).unwrap().rating, 5);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn flushing_an_idle_writer_returns_at_once() {
        let writer = Writer::new();

        writer.flush();
    }

    #[test]
    fn dropping_the_writer_stops_the_thread() {
        let dir = temp_dir("drop");
        let image = dir.join("photo.jpg");

        {
            let writer = Writer::new();
            writer.save(
                image.clone(),
                Xmp {
                    rating: 2,
                    keywords: vec![],
                },
            );
            writer.flush();
        }

        assert_eq!(sidecar::read(&image).unwrap().rating, 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
