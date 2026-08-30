//! Putting back what the last thing did.
//!
//! The viewer shipped three folder-wide operations that could not be undone,
//! and is now growing keys that move and delete photographs one at a time.
//! Those two facts together are what make people work on duplicates and
//! distrust a tool, so every operation that touches a file records how to
//! reverse itself before it starts.
//!
//! What is recorded is the inverse rather than the operation: a move remembers
//! where each file came from, a copy remembers what it made, and a mark
//! remembers what it replaced. Undoing is then doing that, which needs no
//! knowledge of what the original operation meant.

use std::path::{Path, PathBuf};

use crate::metadata::xmp::Xmp;

use super::files;

/// How many steps are kept.
///
/// Deep enough to cover a session's worth of mistakes, shallow enough that the
/// list is never the thing using the memory.
const DEPTH: usize = 200;

/// One reversible thing that was done.
#[derive(Debug, Clone)]
pub enum Step {
    /// Files that moved, as `(where it is now, where it was)`.
    Moved(Vec<(PathBuf, PathBuf)>),
    /// Files that were made, so undoing means taking them away again.
    Copied(Vec<PathBuf>),
    /// Files sent to the platform's bin.
    Binned(Vec<PathBuf>),
    /// A photograph's marks, as they were before they were changed.
    Marked { image: PathBuf, before: Box<Xmp> },
}

impl Step {
    /// What undoing this would do, in a sentence, so nothing happens silently.
    pub fn describe(&self) -> String {
        match self {
            Step::Moved(moves) => format!("put {} file(s) back", moves.len()),
            Step::Copied(made) => format!("take away {} copied file(s)", made.len()),
            Step::Binned(binned) => format!("bring {} file(s) back from the bin", binned.len()),
            Step::Marked { image, .. } => format!(
                "put the marks on {} back",
                image.file_name().unwrap_or_default().to_string_lossy()
            ),
        }
    }

    /// Whether there is anything in it.
    fn is_empty(&self) -> bool {
        match self {
            Step::Moved(moves) => moves.is_empty(),
            Step::Copied(made) => made.is_empty(),
            Step::Binned(binned) => binned.is_empty(),
            Step::Marked { .. } => false,
        }
    }
}

/// What undoing a step actually did.
#[derive(Debug, Default)]
pub struct Undone {
    /// Photographs that are somewhere different now, as `(from, to)`.
    pub moved: Vec<(PathBuf, PathBuf)>,
    /// Photographs that are no longer there.
    pub removed: Vec<PathBuf>,
    /// Photographs whose marks need reading again.
    pub remarked: Vec<PathBuf>,
    /// What could not be put back, and why.
    pub failed: Vec<String>,
}

/// The steps that can still be undone, most recent last.
#[derive(Debug, Default)]
pub struct Journal {
    steps: Vec<Step>,
}

impl Journal {
    /// Records a step, unless it did nothing.
    pub fn record(&mut self, step: Step) {
        if step.is_empty() {
            return;
        }

        self.steps.push(step);

        while self.steps.len() > DEPTH {
            self.steps.remove(0);
        }
    }

    /// What the next undo would do, without doing it.
    pub fn peek(&self) -> Option<&Step> {
        self.steps.last()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Forgets everything, for when the folder has changed underneath it.
    pub fn clear(&mut self) {
        self.steps.clear();
    }

    /// Reverses the most recent step.
    pub fn undo(&mut self) -> Option<Undone> {
        let step = self.steps.pop()?;

        Some(match step {
            Step::Moved(moves) => put_back(&moves),
            Step::Copied(made) => take_away(&made),
            Step::Binned(binned) => bring_back(&binned),
            Step::Marked { image, before } => remark(&image, &before),
        })
    }
}

/// Moves each file back where it came from.
fn put_back(moves: &[(PathBuf, PathBuf)]) -> Undone {
    let mut undone = Undone::default();

    for (now, was) in moves {
        match files::move_file(now, was) {
            Ok(()) => undone.moved.push((now.clone(), was.clone())),
            Err(e) => undone.failed.push(format!("{}: {e}", now.display())),
        }
    }

    undone
}

/// Sends the copies to the bin rather than deleting them, because an undo
/// should not itself be the thing nobody can take back.
fn take_away(made: &[PathBuf]) -> Undone {
    let mut undone = Undone::default();

    for copy in made {
        match files::to_bin(copy) {
            Ok(()) => undone.removed.push(copy.clone()),
            Err(e) => undone.failed.push(format!("{}: {e}", copy.display())),
        }
    }

    undone
}

/// Puts marks back as they were, which is a save like any other.
fn remark(image: &Path, before: &Xmp) -> Undone {
    let mut undone = Undone::default();

    match crate::annotations::sidecar::write(image, before) {
        Ok(()) => undone.remarked.push(image.to_path_buf()),
        Err(e) => undone.failed.push(format!("{}: {e}", image.display())),
    }

    undone
}

/// Brings files back out of the platform's bin.
///
/// Only where the platform lets a program address what is in it: Windows and
/// the freedesktop specification both do, macOS does not, and there the honest
/// answer is to say so rather than to pretend.
#[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
fn bring_back(binned: &[PathBuf]) -> Undone {
    let mut undone = Undone::default();

    let Ok(inside) = trash::os_limited::list() else {
        undone.failed.push("the bin could not be read".to_string());
        return undone;
    };

    let wanted: Vec<trash::TrashItem> = inside
        .into_iter()
        .filter(|item| binned.contains(&item.original_path()))
        .collect();

    if wanted.is_empty() {
        undone
            .failed
            .push("nothing matching is in the bin any more".to_string());
        return undone;
    }

    undone.moved.extend(wanted.iter().map(|item| {
        let original = item.original_path();
        (original.clone(), original)
    }));

    if let Err(e) = trash::os_limited::restore_all(wanted) {
        undone.moved.clear();
        undone.failed.push(format!("{e}"));
    }

    undone
}

#[cfg(not(any(target_os = "windows", all(unix, not(target_os = "macos")))))]
fn bring_back(_binned: &[PathBuf]) -> Undone {
    Undone {
        failed: vec![
            "this platform does not let a program take things back out of the bin; \
             they are still in it"
                .to_string(),
        ],
        ..Undone::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-journal-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_step_that_did_nothing_is_not_recorded() {
        let mut journal = Journal::default();
        journal.record(Step::Moved(Vec::new()));
        journal.record(Step::Copied(Vec::new()));

        assert!(journal.is_empty());
        assert!(journal.peek().is_none());
    }

    #[test]
    fn the_journal_is_bounded() {
        let mut journal = Journal::default();

        for i in 0..DEPTH + 20 {
            journal.record(Step::Copied(vec![PathBuf::from(format!("{i}.jpg"))]));
        }

        assert_eq!(journal.len(), DEPTH);
        // The oldest went, not the newest.
        assert!(journal.peek().unwrap().describe().contains("1 copied file"));
    }

    #[test]
    fn a_move_goes_back_where_it_came_from() {
        let dir = temp_dir("move");
        let into = dir.join("keep");
        std::fs::create_dir_all(&into).unwrap();

        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();
        files::move_file(&dir.join("a.jpg"), &into.join("a.jpg")).unwrap();

        let mut journal = Journal::default();
        journal.record(Step::Moved(vec![(into.join("a.jpg"), dir.join("a.jpg"))]));

        let undone = journal.undo().expect("a step to undo");

        assert!(undone.failed.is_empty(), "{:?}", undone.failed);
        assert_eq!(undone.moved.len(), 1);
        assert!(dir.join("a.jpg").exists());
        assert!(!into.join("a.jpg").exists());
        assert!(journal.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A move back onto an occupied name is refused rather than clobbering,
    /// the same as any other move.
    #[test]
    fn a_move_that_cannot_go_back_is_reported() {
        let dir = temp_dir("blocked");
        let into = dir.join("keep");
        std::fs::create_dir_all(&into).unwrap();

        std::fs::write(into.join("a.jpg"), b"moved").unwrap();
        std::fs::write(dir.join("a.jpg"), b"something else took the name").unwrap();

        let mut journal = Journal::default();
        journal.record(Step::Moved(vec![(into.join("a.jpg"), dir.join("a.jpg"))]));

        let undone = journal.undo().expect("a step to undo");

        assert_eq!(undone.failed.len(), 1, "{undone:?}");
        assert_eq!(
            std::fs::read(dir.join("a.jpg")).unwrap(),
            b"something else took the name"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn marks_go_back_to_what_they_were() {
        let dir = temp_dir("marks");
        let image = dir.join("photo.jpg");

        let before = Xmp {
            rating: 2,
            keywords: vec!["Before".to_string()],
            ..Xmp::default()
        };

        crate::annotations::sidecar::write(
            &image,
            &Xmp {
                rating: 5,
                ..Xmp::default()
            },
        )
        .unwrap();

        let mut journal = Journal::default();
        journal.record(Step::Marked {
            image: image.clone(),
            before: Box::new(before.clone()),
        });

        let undone = journal.undo().expect("a step to undo");

        assert!(undone.failed.is_empty(), "{:?}", undone.failed);
        assert_eq!(crate::annotations::sidecar::read(&image), Some(before));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_step_says_what_undoing_it_would_do() {
        let steps = [
            Step::Moved(vec![(PathBuf::from("a"), PathBuf::from("b"))]),
            Step::Copied(vec![PathBuf::from("a")]),
            Step::Binned(vec![PathBuf::from("a")]),
            Step::Marked {
                image: PathBuf::from("/photos/a.jpg"),
                before: Box::new(Xmp::default()),
            },
        ];

        for step in steps {
            assert!(!step.describe().is_empty());
        }
    }

    #[test]
    fn clearing_forgets_everything() {
        let mut journal = Journal::default();
        journal.record(Step::Copied(vec![PathBuf::from("a.jpg")]));
        journal.clear();

        assert!(journal.is_empty());
    }
}
