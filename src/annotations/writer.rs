//! The thread that persists annotations.
//!
//! Rating an image is a keystroke; it must not wait on a disk that happens to
//! be a network share. Saves are queued instead, and a run of changes to the
//! same image collapses into one write.
//!
//! The queue, the thread and the shutdown are [`crate::work`]; what is left
//! here is the two things about *this* queue — that it coalesces by file, and
//! that what it holds is finished rather than dropped when the program closes,
//! because a queued save is somebody's keywords and not a photograph nobody is
//! waiting for.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::metadata::xmp::Xmp;
use crate::work::{Coalescing, OnShutdown, Pool};

use super::sidecar;

/// Queues annotation saves onto a background thread.
pub struct Writer {
    pool: Pool<Coalescing<PathBuf, Xmp>>,
    /// Saves that failed, waiting to be told to the user.
    ///
    /// A rating is a keystroke and the disk it lands on may be a read-only
    /// card. Losing that quietly is worse than the write failing.
    problems: Arc<Mutex<Vec<String>>>,
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer {
    pub fn new() -> Writer {
        let problems: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let collected = Arc::clone(&problems);

        let pool = Pool::new(
            "avis-annotations",
            1,
            OnShutdown::Finish,
            move |(image, annotations): (PathBuf, Xmp)| {
                let Err(e) = sidecar::write(&image, &annotations) else {
                    return;
                };

                tracing::error!(
                    "Failure writing {} -> {e}",
                    sidecar::path_for(&image).display()
                );

                let name = image
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| image.display().to_string());

                if let Ok(mut problems) = collected.lock() {
                    problems.push(format!("Could not save {name}: {e}"));
                }
            },
        );

        Writer { pool, problems }
    }

    /// Queues `annotations` to be written beside `image`.
    pub fn save(&self, image: PathBuf, annotations: Xmp) {
        self.pool.submit((image, annotations));
    }

    /// Takes whatever failed since this was last asked, for the user to see.
    pub fn problems(&self) -> Vec<String> {
        match self.problems.lock() {
            Ok(mut problems) => std::mem::take(&mut problems),
            Err(_) => Vec::new(),
        }
    }

    /// Blocks until the queue is empty and no save is in flight.
    pub fn flush(&self) {
        self.pool.flush();
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
                ..Xmp::default()
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
                    ..Xmp::default()
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
                    ..Xmp::default()
                },
            );
            writer.flush();
        }

        assert_eq!(sidecar::read(&image).unwrap().rating, 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
