//! The steps that touched files, and how to run one either way.
//!
//! This was `organize::journal`, which recorded the inverse of an operation
//! and nothing else. That is enough to undo and is the reason there was no
//! redo: the forward half had been thrown away, so having gone back there was
//! nothing left to say what going forward again would mean.
//!
//! A step now records both halves and is run in a direction. A move already
//! held both — where a file is and where it came from — and only wanted a word
//! saying which way round to read it; a copy gained the sources it was made
//! from, and a mark the document it was changed *to*. The extra half is cheap:
//! an [`Xmp`] is a rating, a flag, a label and two lists of keywords.
//!
//! What is *not* recorded is the operation's meaning. Running a step needs no
//! knowledge of whether it came from a key, a menu or a gesture, which is what
//! lets one list hold everything.

use std::path::{Path, PathBuf};

use crate::metadata::xmp::Xmp;
use crate::organize::files;

/// Which way a step is being run.
///
/// The same recording serves both, so there is one description of what
/// happened rather than two that can drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// Put back what was done.
    Back,
    /// Do it again.
    Forward,
}

impl Way {
    /// The opposite direction.
    pub fn inverse(self) -> Way {
        match self {
            Way::Back => Way::Forward,
            Way::Forward => Way::Back,
        }
    }
}

/// One thing that was done to files, recorded so it can be run either way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Files that moved, as `(where it went, where it came from)`.
    ///
    /// Symmetric already: going back moves the first to the second, going
    /// forward moves the second to the first.
    Moved(Vec<(PathBuf, PathBuf)>),
    /// Files that were copied.
    ///
    /// `pairs` is the photographs asked for, as `(source, destination)`, and
    /// `made` is everything that actually appeared — the sidecars as well —
    /// which is what taking the copy away has to remove.
    Copied {
        pairs: Vec<(PathBuf, PathBuf)>,
        made: Vec<PathBuf>,
    },
    /// Files sent to the platform's bin.
    Binned(Vec<PathBuf>),
    /// A photograph's marks, as they were and as they were made.
    Marked {
        image: PathBuf,
        before: Box<Xmp>,
        after: Box<Xmp>,
    },
    /// Several things that were done as one, and come back as one.
    ///
    /// Marking a selection is one keystroke, so undoing it has to be one
    /// keystroke as well: an undo that gave back one photograph of the two
    /// hundred just rated would be worse than none.
    Many(Vec<Step>),
}

impl Step {
    /// How many files running this would touch.
    ///
    /// A bulk step is the frightening one: two hundred files move with no
    /// chance to look at the sentence saying so.
    pub fn files(&self) -> usize {
        match self {
            Step::Moved(moves) => moves.len(),
            Step::Copied { made, .. } => made.len(),
            Step::Binned(binned) => binned.len(),
            Step::Marked { .. } => 1,
            Step::Many(steps) => steps.iter().map(Step::files).sum(),
        }
    }

    /// Whether this step did nothing, and so is not worth recording.
    ///
    /// A mark is never empty: setting a rating back to what it already was is
    /// still a document written, and the caller decides whether that counts.
    pub fn is_empty(&self) -> bool {
        match self {
            Step::Moved(moves) => moves.is_empty(),
            Step::Copied { made, .. } => made.is_empty(),
            Step::Binned(binned) => binned.is_empty(),
            Step::Marked { .. } => false,
            Step::Many(steps) => steps.iter().all(Step::is_empty),
        }
    }

    /// What running this would do, in a sentence, so nothing happens silently.
    pub fn describe(&self, way: Way) -> String {
        match (self, way) {
            (Step::Moved(moves), Way::Back) => format!("put {} file(s) back", moves.len()),
            (Step::Moved(moves), Way::Forward) => format!("move {} file(s) again", moves.len()),
            (Step::Copied { made, .. }, Way::Back) => {
                format!("take away {} copied file(s)", made.len())
            }
            (Step::Copied { pairs, .. }, Way::Forward) => {
                format!("copy {} file(s) again", pairs.len())
            }
            (Step::Binned(binned), Way::Back) => {
                format!("bring {} file(s) back from the bin", binned.len())
            }
            (Step::Binned(binned), Way::Forward) => {
                format!("send {} file(s) to the bin again", binned.len())
            }
            (Step::Marked { image, .. }, way) => {
                let name = image.file_name().unwrap_or_default().to_string_lossy();
                match way {
                    Way::Back => format!("put the marks on {name} back"),
                    Way::Forward => format!("put the marks on {name} on again"),
                }
            }
            (Step::Many(steps), way) => match steps.first() {
                // Named after what it is made of, because "undo 200 steps"
                // says nothing about what is about to happen.
                Some(first) if steps.len() == 1 => first.describe(way),
                Some(first) => format!(
                    "{} — and {} more like it",
                    first.describe(way),
                    steps.len() - 1
                ),
                None => "do nothing".to_string(),
            },
        }
    }

    /// What this step *was*, for the row in the panel that stands for it.
    ///
    /// Not the same sentence as [`Step::describe`], which says what running it
    /// would do next. A list of things that happened is written in the past
    /// tense, and reading "put 3 file(s) back" against something that moved
    /// them would be a list of the wrong history.
    pub fn label(&self) -> String {
        match self {
            Step::Moved(moves) => format!("Moved {} file(s)", moves.len()),
            Step::Copied { pairs, .. } => format!("Copied {} file(s)", pairs.len()),
            Step::Binned(binned) => format!("Sent {} file(s) to the bin", binned.len()),
            Step::Marked { image, .. } => format!(
                "Marked {}",
                image.file_name().unwrap_or_default().to_string_lossy()
            ),
            Step::Many(steps) => match steps.first() {
                Some(first) if steps.len() == 1 => first.label(),
                Some(first) => format!("{} and {} more", first.label(), steps.len() - 1),
                None => "Did nothing".to_string(),
            },
        }
    }

    /// Runs the step in the given direction and reports what happened on disk.
    pub fn run(&self, way: Way) -> Done {
        match self {
            Step::Moved(moves) => match way {
                Way::Back => shift(moves.iter().map(|(now, was)| (now, was))),
                Way::Forward => shift(moves.iter().map(|(now, was)| (was, now))),
            },
            Step::Copied { pairs, made } => match way {
                Way::Back => take_away(made),
                Way::Forward => copy_again(pairs),
            },
            Step::Binned(binned) => match way {
                Way::Back => bring_back(binned),
                Way::Forward => bin_again(binned),
            },
            Step::Marked {
                image,
                before,
                after,
            } => match way {
                Way::Back => remark(image, before),
                Way::Forward => remark(image, after),
            },
            Step::Many(steps) => all_of(steps, way),
        }
    }
}

/// What running a step actually did, for the caller to report and act on.
#[derive(Debug, Default)]
pub struct Done {
    /// Files that changed place, as `(from, to)`.
    pub moved: Vec<(PathBuf, PathBuf)>,
    /// Files that are no longer where they were.
    pub removed: Vec<PathBuf>,
    /// Photographs whose sidecar was rewritten.
    pub remarked: Vec<PathBuf>,
    /// What could not be done, each in a sentence.
    pub failed: Vec<String>,
}

impl Done {
    /// Whether anything at all reached the disk.
    pub fn is_empty(&self) -> bool {
        self.moved.is_empty() && self.removed.is_empty() && self.remarked.is_empty()
    }

    /// Folds one part of a batch into the whole.
    fn absorb(&mut self, part: Done) {
        self.moved.extend(part.moved);
        self.removed.extend(part.removed);
        self.remarked.extend(part.remarked);
        self.failed.extend(part.failed);
    }
}

/// Runs a batch, and reports it as one.
///
/// Backwards it goes newest first, because the parts of a batch can depend on
/// each other — two photographs that swapped places, say — and the same rule
/// that orders the history orders what is inside a step of it. Forwards it goes
/// in the order the parts were done, for the same reason read the other way.
fn all_of(steps: &[Step], way: Way) -> Done {
    let mut done = Done::default();

    match way {
        Way::Back => {
            for step in steps.iter().rev() {
                done.absorb(step.run(way));
            }
        }
        Way::Forward => {
            for step in steps {
                done.absorb(step.run(way));
            }
        }
    }

    done
}

/// Moves each file from the first path to the second.
fn shift<'a>(moves: impl Iterator<Item = (&'a PathBuf, &'a PathBuf)>) -> Done {
    let mut done = Done::default();

    for (from, to) in moves {
        match files::move_file(from, to) {
            Ok(()) => done.moved.push((from.clone(), to.clone())),
            Err(e) => done.failed.push(format!("{}: {e}", from.display())),
        }
    }

    done
}

/// Sends the copies to the bin rather than deleting them, because an undo
/// should not itself be the thing nobody can take back.
fn take_away(made: &[PathBuf]) -> Done {
    let mut done = Done::default();

    for copy in made {
        match files::to_bin(copy) {
            Ok(()) => done.removed.push(copy.clone()),
            Err(e) => done.failed.push(format!("{}: {e}", copy.display())),
        }
    }

    done
}

/// Makes the copies again, for a redo.
fn copy_again(pairs: &[(PathBuf, PathBuf)]) -> Done {
    let mut done = Done::default();

    for (from, to) in pairs {
        match files::copy_file(from, to) {
            Ok(copies) => done
                .moved
                .extend(copies.into_iter().map(|c| (from.clone(), c))),
            Err(e) => done.failed.push(format!("{}: {e}", from.display())),
        }
    }

    done
}

/// Sends the files to the bin again, for a redo.
fn bin_again(binned: &[PathBuf]) -> Done {
    let mut done = Done::default();

    for path in binned {
        match files::to_bin(path) {
            Ok(()) => done.removed.push(path.clone()),
            Err(e) => done.failed.push(format!("{}: {e}", path.display())),
        }
    }

    done
}

/// Writes a photograph's marks, which is a save like any other.
fn remark(image: &Path, xmp: &Xmp) -> Done {
    let mut done = Done::default();

    match crate::annotations::sidecar::write(image, xmp) {
        Ok(()) => done.remarked.push(image.to_path_buf()),
        Err(e) => done.failed.push(format!("{}: {e}", image.display())),
    }

    done
}

/// Brings files back out of the platform's bin.
///
/// Only where the platform lets a program address what is in it: Windows and
/// the freedesktop specification both do, macOS does not, and there the honest
/// answer is to say so rather than to pretend.
#[cfg(any(target_os = "windows", all(unix, not(target_os = "macos"))))]
fn bring_back(binned: &[PathBuf]) -> Done {
    let mut done = Done::default();

    let Ok(inside) = trash::os_limited::list() else {
        done.failed.push("the bin could not be read".to_string());
        return done;
    };

    let wanted: Vec<trash::TrashItem> = inside
        .into_iter()
        .filter(|item| binned.contains(&item.original_path()))
        .collect();

    if wanted.is_empty() {
        done.failed
            .push("nothing matching is in the bin any more".to_string());
        return done;
    }

    done.moved.extend(wanted.iter().map(|item| {
        let original = item.original_path();
        (original.clone(), original)
    }));

    if let Err(e) = trash::os_limited::restore_all(wanted) {
        done.moved.clear();
        done.failed.push(format!("{e}"));
    }

    done
}

#[cfg(not(any(target_os = "windows", all(unix, not(target_os = "macos")))))]
fn bring_back(_binned: &[PathBuf]) -> Done {
    Done {
        failed: vec![
            "this platform does not let a program take things back out of the bin; \
             they are still in it"
                .to_string(),
        ],
        ..Done::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A directory of this test's own, removed by the operating system rather
    /// than by us, so a failing assertion leaves the evidence behind.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-history-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, contents: &str) {
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn a_step_that_did_nothing_is_empty() {
        assert!(Step::Moved(vec![]).is_empty());
        assert!(Step::Copied {
            pairs: vec![],
            made: vec![]
        }
        .is_empty());
        assert!(Step::Binned(vec![]).is_empty());
        assert!(Step::Many(vec![Step::Binned(vec![])]).is_empty());
    }

    /// Setting a rating to what it already was still wrote a document, so it
    /// is still a step.
    #[test]
    fn a_mark_is_never_empty() {
        let step = Step::Marked {
            image: PathBuf::from("a.jpg"),
            before: Box::new(Xmp::default()),
            after: Box::new(Xmp::default()),
        };

        assert!(!step.is_empty());
    }

    /// The move already held both halves; the direction is the only new thing.
    #[test]
    fn a_move_runs_both_ways() {
        let dir = temp_dir("move-both-ways");
        let was = dir.join("a.jpg");
        let now = dir.join("b.jpg");
        write(&was, "one");

        let step = Step::Moved(vec![(now.clone(), was.clone())]);

        // Forward is what the operation did: from where it came to where it went.
        let done = step.run(Way::Forward);
        assert!(done.failed.is_empty(), "{:?}", done.failed);
        assert!(now.exists() && !was.exists());

        // Back is the other way round, off the same recording.
        let done = step.run(Way::Back);
        assert!(done.failed.is_empty(), "{:?}", done.failed);
        assert!(was.exists() && !now.exists());
        assert_eq!(fs::read_to_string(&was).unwrap(), "one");
    }

    /// A move that cannot go back reports it and leaves the file that is in
    /// the way alone.
    #[test]
    fn a_move_that_cannot_go_back_is_reported() {
        let dir = temp_dir("move-blocked");
        let was = dir.join("a.jpg");
        let now = dir.join("b.jpg");
        write(&now, "moved");
        write(&was, "someone else");

        let done = Step::Moved(vec![(now.clone(), was.clone())]).run(Way::Back);

        assert_eq!(done.failed.len(), 1);
        assert_eq!(fs::read_to_string(&was).unwrap(), "someone else");
        assert_eq!(fs::read_to_string(&now).unwrap(), "moved");
    }

    /// The half a copy was missing: the sources, so it can be made again.
    #[test]
    fn a_copy_can_be_made_again() {
        let dir = temp_dir("copy-again");
        let from = dir.join("a.jpg");
        let to = dir.join("sub").join("a.jpg");
        fs::create_dir_all(dir.join("sub")).unwrap();
        write(&from, "one");

        let step = Step::Copied {
            pairs: vec![(from.clone(), to.clone())],
            made: vec![to.clone()],
        };

        let done = step.run(Way::Forward);
        assert!(done.failed.is_empty(), "{:?}", done.failed);
        assert!(to.exists());
        assert_eq!(fs::read_to_string(&to).unwrap(), "one");
        // The source is still there: a copy is not a move.
        assert!(from.exists());
    }

    /// Both directions get a sentence, and it says which way it is going.
    #[test]
    fn every_step_says_what_running_it_would_do() {
        let steps = [
            Step::Moved(vec![(PathBuf::from("a"), PathBuf::from("b"))]),
            Step::Copied {
                pairs: vec![(PathBuf::from("a"), PathBuf::from("b"))],
                made: vec![PathBuf::from("b")],
            },
            Step::Binned(vec![PathBuf::from("a")]),
            Step::Marked {
                image: PathBuf::from("a.jpg"),
                before: Box::new(Xmp::default()),
                after: Box::new(Xmp::default()),
            },
        ];

        for step in &steps {
            let back = step.describe(Way::Back);
            let forward = step.describe(Way::Forward);

            assert!(!back.is_empty(), "{step:?} says nothing going back");
            assert!(!forward.is_empty(), "{step:?} says nothing going forward");
            assert_ne!(back, forward, "{step:?} says the same thing both ways");
        }
    }

    /// The batch is undone newest first, because its parts can depend on each
    /// other, and done again oldest first for the same reason.
    #[test]
    fn a_batch_runs_in_opposite_orders() {
        let dir = temp_dir("batch-order");
        let a = dir.join("a.jpg");
        let b = dir.join("b.jpg");
        let c = dir.join("c.jpg");
        write(&a, "one");

        // a -> b, then b -> c. Undoing in the order recorded would try to move
        // b back to a while b does not exist yet.
        let step = Step::Many(vec![
            Step::Moved(vec![(b.clone(), a.clone())]),
            Step::Moved(vec![(c.clone(), b.clone())]),
        ]);

        let done = step.run(Way::Forward);
        assert!(done.failed.is_empty(), "{:?}", done.failed);
        assert!(c.exists() && !a.exists() && !b.exists());

        let done = step.run(Way::Back);
        assert!(done.failed.is_empty(), "{:?}", done.failed);
        assert!(a.exists() && !b.exists() && !c.exists());
        assert_eq!(fs::read_to_string(&a).unwrap(), "one");
    }

    /// A batch is one step however many files it touches.
    #[test]
    fn a_batch_counts_every_file_in_it() {
        let step = Step::Many(vec![
            Step::Binned(vec![PathBuf::from("a"), PathBuf::from("b")]),
            Step::Marked {
                image: PathBuf::from("c.jpg"),
                before: Box::new(Xmp::default()),
                after: Box::new(Xmp::default()),
            },
        ]);

        assert_eq!(step.files(), 3);
    }

    #[test]
    fn a_batch_describes_itself_by_what_is_in_it() {
        let step = Step::Many(vec![
            Step::Copied {
                pairs: vec![(PathBuf::from("a"), PathBuf::from("b"))],
                made: vec![PathBuf::from("b")],
            },
            Step::Binned(vec![PathBuf::from("c")]),
        ]);

        let said = step.describe(Way::Back);
        assert!(said.starts_with("take away 1 copied file(s)"), "{said}");
        assert!(said.contains("1 more"), "{said}");
    }

    #[test]
    fn a_direction_is_its_own_inverse_twice_over() {
        assert_eq!(Way::Back.inverse(), Way::Forward);
        assert_eq!(Way::Forward.inverse().inverse(), Way::Forward);
    }

    /// Marks go back to the document they were, and forward to the one they
    /// were made — off one recording.
    #[test]
    fn marks_run_both_ways() {
        let dir = temp_dir("marks-both-ways");
        let image = dir.join("a.jpg");
        write(&image, "not really a photograph");

        let before = Xmp {
            rating: 1,
            ..Xmp::default()
        };
        let after = Xmp {
            rating: 5,
            ..Xmp::default()
        };

        let step = Step::Marked {
            image: image.clone(),
            before: Box::new(before.clone()),
            after: Box::new(after.clone()),
        };

        let done = step.run(Way::Forward);
        assert!(done.failed.is_empty(), "{:?}", done.failed);
        assert_eq!(
            crate::annotations::sidecar::read(&image).map(|x| x.rating),
            Some(5)
        );

        let done = step.run(Way::Back);
        assert!(done.failed.is_empty(), "{:?}", done.failed);
        assert_eq!(
            crate::annotations::sidecar::read(&image).map(|x| x.rating),
            Some(1)
        );
    }
}
