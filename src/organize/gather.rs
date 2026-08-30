//! Tidying confirmed groups into folders of their own.
//!
//! `hdr1`, `hdr2`, `stack1`, `timelapse1`, `series1` — one folder per group,
//! numbered per kind in the order they were taken. A number already taken on
//! disk is skipped rather than merged into, so running this twice on a folder
//! that has grown does not tip new frames in among the old ones.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use super::files;
use super::group::{Group, Kind};

/// The highest number tried before giving up on a kind.
///
/// A folder holding ten thousand separate brackets is not a folder anyone is
/// tidying by hand, and an unbounded search would hang on a filesystem that
/// answered every question with yes.
const MAX_FOLDERS: usize = 10_000;

/// One group and where its frames are going.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    /// Which group this came from, so the interface can line the two up.
    pub group: usize,
    pub kind: Kind,
    /// The folder to make, named as it will appear.
    pub folder: PathBuf,
    /// Each frame and where it lands.
    pub moves: Vec<(PathBuf, PathBuf)>,
}

impl Planned {
    /// The folder's name alone, which is what the header shows.
    pub fn name(&self) -> String {
        self.folder
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
}

/// Works out where every confirmed group goes.
///
/// `into` is the folder the pictures are in now, which is where the new
/// folders are made. Groups that were emptied are skipped.
pub fn plan(groups: &[Group], into: &Path) -> Vec<Planned> {
    let mut used: BTreeMap<Kind, usize> = BTreeMap::new();
    let mut claimed: Vec<PathBuf> = Vec::new();
    let mut planned = Vec::new();

    for (index, group) in groups.iter().enumerate() {
        if group.is_empty() {
            continue;
        }

        let counter = used.entry(group.kind).or_insert(1);
        let Some(folder) = free_folder(into, group.kind, counter, &claimed) else {
            continue;
        };

        let moves = group
            .members
            .iter()
            .map(|entry| {
                let name = entry.path.file_name().unwrap_or_default();
                (entry.path.clone(), folder.join(name))
            })
            .collect();

        claimed.push(folder.clone());
        planned.push(Planned {
            group: index,
            kind: group.kind,
            folder,
            moves,
        });
    }

    planned
}

/// Carries a plan out, making each folder and moving its frames into it.
///
/// A folder that cannot be made is reported and its frames left where they
/// are; there is nothing to undo, because nothing was moved.
///
/// A group whose frames would land on one another — two frames of the same
/// name, which is what flattening a tree produces — is refused whole rather
/// than half moved, so nobody has to work out afterwards which half went.
pub fn apply(planned: &[Planned]) -> Outcome {
    let mut outcome = Outcome::default();

    for plan in planned {
        if let Some(problem) = collisions_within(plan) {
            outcome.failed.push((plan.folder.clone(), problem));
            continue;
        }

        if let Err(e) = std::fs::create_dir_all(&plan.folder) {
            outcome
                .failed
                .push((plan.folder.clone(), format!("could not be made: {e}")));
            continue;
        }

        for (from, to) in &plan.moves {
            match files::move_file(from, to) {
                Ok(()) => outcome.moved += 1,
                Err(e) => outcome.failed.push((from.clone(), e.to_string())),
            }
        }

        outcome.folders += 1;
    }

    outcome
}

/// Why a group cannot be tidied as a whole, if it cannot.
fn collisions_within(plan: &Planned) -> Option<String> {
    let mut taken: HashSet<String> = HashSet::new();

    for (_, to) in &plan.moves {
        let name = to.file_name().unwrap_or_default().to_string_lossy();
        let key = if cfg!(windows) {
            name.to_lowercase()
        } else {
            name.into_owned()
        };

        if !taken.insert(key) {
            return Some(format!(
                "two frames would both be called {}",
                to.file_name().unwrap_or_default().to_string_lossy()
            ));
        }

        if to.exists() {
            return Some(format!(
                "{} is already there",
                to.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    None
}

/// What an applied plan did.
#[derive(Debug, Default)]
pub struct Outcome {
    pub folders: usize,
    pub moved: usize,
    pub failed: Vec<(PathBuf, String)>,
}

impl Outcome {
    pub fn summary(&self) -> String {
        match (self.moved, self.failed.len()) {
            (0, 0) => "Nothing to tidy".to_string(),
            (moved, 0) => format!("Moved {moved} file(s) into {} folder(s)", self.folders),
            (0, failed) => format!("{failed} file(s) could not be moved"),
            (moved, failed) => format!("Moved {moved}, {failed} could not be"),
        }
    }
}

/// The next folder of this kind that nothing is using, advancing `counter`
/// past it.
fn free_folder(
    into: &Path,
    kind: Kind,
    counter: &mut usize,
    claimed: &[PathBuf],
) -> Option<PathBuf> {
    while *counter <= MAX_FOLDERS {
        let folder = into.join(format!("{}{counter}", kind.folder()));
        *counter += 1;

        if !folder.exists() && !claimed.contains(&folder) {
            return Some(folder);
        }
    }

    tracing::warn!("Ran out of folder names for {}", kind.label());
    None
}

#[cfg(test)]
mod tests {
    use super::super::group::test_support::frame;
    use super::super::Entry;
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-gather-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// A group of files that exist on disk.
    fn group(dir: &Path, kind: Kind, names: &[&str]) -> Group {
        let members: Vec<Entry> = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let path = dir.join(name);
                std::fs::write(&path, name.as_bytes()).unwrap();

                let mut entry = frame(name, index as i64, 1);
                entry.path = path;

                entry
            })
            .collect();

        Group::new(kind, members)
    }

    fn listing(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();

        names.sort();
        names
    }

    #[test]
    fn each_group_gets_a_folder_named_after_its_kind() {
        let dir = temp_dir("names");
        let groups = vec![
            group(&dir, Kind::Hdr, &["a.jpg", "b.jpg"]),
            group(&dir, Kind::Hdr, &["c.jpg", "d.jpg"]),
            group(&dir, Kind::Timelapse, &["e.jpg", "f.jpg"]),
        ];

        let planned = plan(&groups, &dir);
        let names: Vec<String> = planned.iter().map(Planned::name).collect();

        assert_eq!(names, vec!["hdr1", "hdr2", "timelapse1"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn applying_makes_the_folders_and_moves_the_frames() {
        let dir = temp_dir("apply");
        let groups = vec![group(&dir, Kind::Hdr, &["a.jpg", "b.jpg", "c.jpg"])];

        let outcome = apply(&plan(&groups, &dir));

        assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
        assert_eq!((outcome.folders, outcome.moved), (1, 3));
        assert_eq!(listing(&dir), vec!["hdr1"]);
        assert_eq!(listing(&dir.join("hdr1")), vec!["a.jpg", "b.jpg", "c.jpg"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_contents_arrive_with_the_names() {
        let dir = temp_dir("contents");
        let groups = vec![group(&dir, Kind::Series, &["a.jpg"])];

        apply(&plan(&groups, &dir));

        assert_eq!(std::fs::read(dir.join("series1/a.jpg")).unwrap(), b"a.jpg");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_folder_already_there_is_stepped_over_rather_than_tipped_into() {
        let dir = temp_dir("occupied");
        std::fs::create_dir_all(dir.join("hdr1")).unwrap();
        std::fs::write(dir.join("hdr1/old.jpg"), b"from last time").unwrap();

        let groups = vec![group(&dir, Kind::Hdr, &["a.jpg"])];
        let planned = plan(&groups, &dir);

        assert_eq!(planned[0].name(), "hdr2");

        apply(&planned);
        assert_eq!(listing(&dir.join("hdr1")), vec!["old.jpg"]);
        assert_eq!(listing(&dir.join("hdr2")), vec!["a.jpg"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_emptied_group_is_skipped() {
        let dir = temp_dir("emptied");
        let mut groups = vec![
            group(&dir, Kind::Hdr, &["a.jpg"]),
            group(&dir, Kind::Hdr, &["b.jpg"]),
        ];
        groups[0].members.clear();

        let planned = plan(&groups, &dir);

        assert_eq!(planned.len(), 1);
        assert_eq!(planned[0].name(), "hdr1", "the number is not wasted");
        assert_eq!(planned[0].group, 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_sidecar_goes_into_the_folder_with_its_photograph() {
        let dir = temp_dir("sidecar");
        let groups = vec![group(&dir, Kind::Hdr, &["a.jpg"])];
        std::fs::write(dir.join("a.jpg.xmp"), b"<x:xmpmeta/>").unwrap();

        apply(&plan(&groups, &dir));

        assert_eq!(listing(&dir.join("hdr1")), vec!["a.jpg", "a.jpg.xmp"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_frame_that_is_no_longer_there_is_reported_and_the_rest_still_move() {
        let dir = temp_dir("missing");
        let groups = vec![group(&dir, Kind::Hdr, &["a.jpg", "b.jpg"])];
        std::fs::remove_file(dir.join("a.jpg")).unwrap();

        let outcome = apply(&plan(&groups, &dir));

        assert_eq!(outcome.moved, 1);
        assert_eq!(outcome.failed.len(), 1);
        assert_eq!(listing(&dir.join("hdr1")), vec!["b.jpg"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nothing_planned_is_nothing_done() {
        let outcome = apply(&[]);

        assert_eq!(outcome.summary(), "Nothing to tidy");
        assert!(outcome.failed.is_empty());
    }

    #[test]
    fn an_outcome_says_what_happened() {
        let outcome = Outcome {
            folders: 2,
            moved: 7,
            failed: Vec::new(),
        };

        assert_eq!(outcome.summary(), "Moved 7 file(s) into 2 folder(s)");
    }
}
