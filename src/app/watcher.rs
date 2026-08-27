//! Watching the open directory for files appearing or changing.
//!
//! Useful when tethered shooting or a background export drops new images into
//! the folder that is already open.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::formats;

/// What changed on disk since the last frame.
#[derive(Debug, Default, PartialEq)]
pub struct Changes {
    /// Files that appeared and should join the collection.
    pub added: Vec<PathBuf>,
    /// Files already in the collection whose contents changed.
    pub modified: Vec<PathBuf>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty()
    }
}

/// A filesystem watcher that can be toggled on and off.
#[derive(Default)]
pub struct DirectoryWatcher {
    watcher: Option<RecommendedWatcher>,
    events: Arc<Mutex<Vec<Event>>>,
}

impl DirectoryWatcher {
    pub fn is_active(&self) -> bool {
        self.watcher.is_some()
    }

    /// Starts or stops watching `path`. Returns the new state.
    pub fn toggle(&mut self, path: &Path, recursive: bool) -> bool {
        if self.watcher.is_some() {
            self.stop();
            return false;
        }

        self.start(path, recursive);
        self.watcher.is_some()
    }

    /// Restarts the watcher, used when flattening changes the recursion mode.
    pub fn restart(&mut self, path: &Path, recursive: bool) {
        if self.watcher.is_some() {
            self.stop();
            self.start(path, recursive);
        }
    }

    pub fn stop(&mut self) {
        if self.watcher.take().is_some() {
            tracing::info!("Stopped watching for changes");
        }
    }

    fn start(&mut self, path: &Path, recursive: bool) {
        let events = Arc::clone(&self.events);

        let mut watcher = match notify::recommended_watcher(move |result| match result {
            Ok(event) => {
                if let Ok(mut queue) = events.lock() {
                    queue.push(event);
                }
            }
            Err(e) => tracing::error!("Error watching directory: {e}"),
        }) {
            Ok(watcher) => watcher,
            Err(e) => {
                tracing::error!("Failure creating directory watcher -> {e}");
                return;
            }
        };

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        match watcher.watch(path, mode) {
            Ok(()) => {
                tracing::info!("Watching {} for changes", path.display());
                self.watcher = Some(watcher);
            }
            Err(e) => tracing::error!("Failure watching {} -> {e}", path.display()),
        }
    }

    /// Takes the changes seen since the last call.
    ///
    /// `known` decides whether a path is an addition or a modification.
    pub fn take_changes(&mut self, known: impl Fn(&Path) -> bool) -> Changes {
        let Ok(mut queue) = self.events.try_lock() else {
            // Contended for one frame at most; the events stay queued.
            return Changes::default();
        };

        let events = std::mem::take(&mut *queue);
        drop(queue);

        classify(events, known)
    }
}

/// Sorts watcher events into additions and modifications.
fn classify(events: Vec<Event>, known: impl Fn(&Path) -> bool) -> Changes {
    let mut changes = Changes::default();

    for event in events {
        if !(event.kind.is_create() || event.kind.is_modify()) {
            continue;
        }

        for path in event.paths {
            if !formats::is_supported(&path) {
                continue;
            }

            let target = if known(&path) {
                &mut changes.modified
            } else {
                &mut changes.added
            };

            if !target.contains(&path) {
                target.push(path);
            }
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind};

    fn event(kind: EventKind, paths: &[&str]) -> Event {
        Event {
            kind,
            paths: paths.iter().map(PathBuf::from).collect(),
            attrs: Default::default(),
        }
    }

    #[test]
    fn splits_new_files_from_changed_ones() {
        let events = vec![
            event(EventKind::Create(CreateKind::File), &["/photos/new.jpg"]),
            event(
                EventKind::Modify(ModifyKind::Any),
                &["/photos/existing.jpg"],
            ),
        ];

        let changes = classify(events, |path| path.ends_with("existing.jpg"));

        assert_eq!(changes.added, vec![PathBuf::from("/photos/new.jpg")]);
        assert_eq!(
            changes.modified,
            vec![PathBuf::from("/photos/existing.jpg")]
        );
    }

    #[test]
    fn ignores_files_we_cannot_open() {
        let events = vec![event(
            EventKind::Create(CreateKind::File),
            &["/photos/notes.txt", "/photos/new.jpg"],
        )];

        let changes = classify(events, |_| false);
        assert_eq!(changes.added, vec![PathBuf::from("/photos/new.jpg")]);
    }

    #[test]
    fn ignores_removals() {
        let events = vec![event(
            EventKind::Remove(RemoveKind::File),
            &["/photos/gone.jpg"],
        )];

        assert!(classify(events, |_| false).is_empty());
    }

    #[test]
    fn a_path_is_reported_once_per_batch() {
        let events = vec![
            event(EventKind::Create(CreateKind::File), &["/photos/new.jpg"]),
            event(EventKind::Create(CreateKind::File), &["/photos/new.jpg"]),
        ];

        assert_eq!(classify(events, |_| false).added.len(), 1);
    }

    #[test]
    fn an_idle_watcher_reports_nothing() {
        let mut watcher = DirectoryWatcher::default();

        assert!(!watcher.is_active());
        assert!(watcher.take_changes(|_| false).is_empty());
    }
}
