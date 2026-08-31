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
    /// Files that are no longer there.
    ///
    /// Watched for as well as the arrivals, because a folder open in this
    /// viewer and tidied up in a file manager used to keep drawing the
    /// photographs that had gone until something else made it read the folder
    /// again — and opening one of them failed with no explanation.
    pub removed: Vec<PathBuf>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.removed.is_empty()
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

        // The queue belongs to the folder that was being watched. Left in
        // place it would be handed to the next one, which would then be told
        // about arrivals in a folder nobody has open — and, worse, about
        // removals it would apply to whatever happened to share a name.
        if let Ok(mut queue) = self.events.lock() {
            queue.clear();
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

    /// Takes the events seen since the last call.
    ///
    /// Handed out unclassified because deciding what is an arrival and what is
    /// a change needs to know what the collection holds, and building that
    /// answer is only worth doing when there is something to answer about — a
    /// watched folder is quiet almost every frame.
    pub fn take_events(&mut self) -> Vec<Event> {
        let Ok(mut queue) = self.events.try_lock() else {
            // Contended for one frame at most; the events stay queued.
            return Vec::new();
        };

        std::mem::take(&mut *queue)
    }
}

/// Sorts watcher events into arrivals, changes and departures.
///
/// `known` decides whether a path is already in the collection.
///
/// A rename arrives as a remove and a create, which is exactly right: the old
/// name leaves the collection and the new one joins it at its own sorted
/// position.
pub fn classify(events: Vec<Event>, known: impl Fn(&Path) -> bool) -> Changes {
    let mut changes = Changes::default();

    for event in events {
        let removing = event.kind.is_remove();
        if !(removing || event.kind.is_create() || event.kind.is_modify()) {
            continue;
        }

        for path in event.paths {
            if !formats::is_supported(&path) {
                continue;
            }

            let target = match (removing, known(&path)) {
                // Something that was never in the collection cannot leave it.
                (true, false) => continue,
                (true, true) => &mut changes.removed,
                (false, true) => &mut changes.modified,
                (false, false) => &mut changes.added,
            };

            if !target.contains(&path) {
                target.push(path);
            }
        }
    }

    // A file written and then deleted within one batch is not an arrival, and
    // one deleted and written again is not a departure: the last word in the
    // batch is the one the disk will agree with.
    changes.added.retain(|path| !changes.removed.contains(path));

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
    fn a_photograph_that_has_gone_is_reported() {
        let events = vec![event(
            EventKind::Remove(RemoveKind::File),
            &["/photos/gone.jpg"],
        )];

        let changes = classify(events, |_| true);
        assert_eq!(changes.removed, vec![PathBuf::from("/photos/gone.jpg")]);
        assert!(changes.added.is_empty());
    }

    /// A removal of something the collection never held says nothing about
    /// the collection.
    #[test]
    fn a_removal_of_something_unknown_is_ignored() {
        let events = vec![event(
            EventKind::Remove(RemoveKind::File),
            &["/photos/never-had-it.jpg"],
        )];

        assert!(classify(events, |_| false).is_empty());
    }

    /// A rename arrives as a pair, and means exactly what it looks like.
    #[test]
    fn a_rename_is_a_departure_and_an_arrival() {
        let events = vec![
            event(EventKind::Remove(RemoveKind::File), &["/photos/old.jpg"]),
            event(EventKind::Create(CreateKind::File), &["/photos/new.jpg"]),
        ];

        let changes = classify(events, |path| path.ends_with("old.jpg"));

        assert_eq!(changes.removed, vec![PathBuf::from("/photos/old.jpg")]);
        assert_eq!(changes.added, vec![PathBuf::from("/photos/new.jpg")]);
    }

    /// A temporary file an exporter writes and then tidies away should not
    /// join the collection for a frame on its way past.
    #[test]
    fn something_written_and_deleted_in_one_batch_never_arrives() {
        let events = vec![
            event(EventKind::Create(CreateKind::File), &["/photos/tmp.jpg"]),
            event(EventKind::Remove(RemoveKind::File), &["/photos/tmp.jpg"]),
        ];

        // Known by the time the removal is seen, which is how it would look
        // had the arrival already been applied.
        let changes = classify(events, |path| path.ends_with("tmp.jpg"));

        assert!(changes.added.is_empty(), "{changes:?}");
        assert_eq!(changes.removed, vec![PathBuf::from("/photos/tmp.jpg")]);
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
        assert!(watcher.take_events().is_empty());
    }
}
