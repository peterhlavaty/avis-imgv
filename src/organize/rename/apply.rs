//! Carrying a rename out, once it has been looked at.
//!
//! Everything here touches the disk, which is why it is apart from the working
//! out: a plan can be built and shown a hundred times a second, and applied
//! exactly once.

use std::path::{Path, PathBuf};

use crate::annotations::sidecar;

use super::Planned;

/// Carries out a plan, skipping anything with a problem.
///
/// Renames go through a temporary name first. Without that, swapping two files
/// or shifting a numbered sequence down by one would have each rename land on
/// a file the next one still needs.
pub fn apply(planned: &[Planned]) -> Outcome {
    let wanted: Vec<&Planned> = planned.iter().filter(|plan| plan.changes()).collect();
    let mut outcome = Outcome::default();

    // Somewhere nothing else can be, so the first pass cannot collide with
    // anything, including with itself.
    let mut parked: Vec<(PathBuf, &Planned)> = Vec::with_capacity(wanted.len());

    for plan in wanted {
        let temporary = temporary_name(&plan.from, parked.len());

        match rename(&plan.from, &temporary) {
            Ok(()) => parked.push((temporary, plan)),
            Err(e) => outcome.failed.push((plan.from.clone(), e.to_string())),
        }
    }

    for (temporary, plan) in parked {
        match rename(&temporary, &plan.to) {
            Ok(()) => outcome.renamed.push((plan.from.clone(), plan.to.clone())),
            Err(e) => {
                // Put it back rather than leaving a file under a name nobody
                // asked for.
                let _ = std::fs::rename(&temporary, &plan.from);
                outcome.failed.push((plan.from.clone(), e.to_string()));
            }
        }
    }

    outcome
}

/// What an applied plan did.
#[derive(Debug, Default)]
pub struct Outcome {
    /// The files that moved, as `(before, after)`.
    pub renamed: Vec<(PathBuf, PathBuf)>,
    /// The ones that did not, and why.
    pub failed: Vec<(PathBuf, String)>,
}

impl Outcome {
    /// A sentence for the status bar.
    pub fn summary(&self) -> String {
        match (self.renamed.len(), self.failed.len()) {
            (0, 0) => "Nothing to rename".to_string(),
            (renamed, 0) => format!("Renamed {renamed} file(s)"),
            (0, failed) => format!("{failed} file(s) could not be renamed"),
            (renamed, failed) => format!("Renamed {renamed}, {failed} could not be"),
        }
    }
}

/// Moves a file and whatever sidecar belongs to it.
///
/// A rating left behind under the old name would be lost, which is worse than
/// the rename failing.
fn rename(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::rename(from, to)?;

    for candidate in sidecar::candidates(from) {
        if !candidate.exists() {
            continue;
        }

        // The sidecar is a convenience, so a failure to move it is worth a
        // line in the log and not worth undoing the rename over.
        let wanted = sidecar_beside(&candidate, from, to);
        if let Err(e) = std::fs::rename(&candidate, &wanted) {
            tracing::warn!("Could not move {}: {e}", candidate.display());
        }
    }

    Ok(())
}

/// Where a sidecar of `image` ends up when the image becomes `renamed`.
///
/// Sidecars are named either `photo.jpg.xmp` or `photo.xmp`, and which of the
/// two this one is decides how much of its name is the image's.
fn sidecar_beside(candidate: &Path, image: &Path, renamed: &Path) -> PathBuf {
    let suffix = candidate
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
            let image_name = image.file_name()?.to_str()?;
            name.strip_prefix(image_name)
        });

    match suffix {
        // `photo.jpg` + `.xmp`
        Some(suffix) => {
            let mut name = renamed.file_name().unwrap_or_default().to_os_string();
            name.push(suffix);
            renamed.with_file_name(name)
        }
        // `photo` + `.xmp`, which is what the extension replacing form gives.
        None => renamed.with_extension(
            candidate
                .extension()
                .map(|ext| ext.to_string_lossy().into_owned())
                .unwrap_or_else(|| "xmp".to_string()),
        ),
    }
}

/// A name nothing else could be using, in the same folder so the rename stays
/// a rename rather than becoming a copy across devices.
fn temporary_name(path: &Path, index: usize) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();

    path.with_file_name(format!(".avis-rename-{index}-{stem}.tmp"))
}

#[cfg(test)]
mod tests {
    use super::super::{plan, Options, Planned, Problem};
    use super::*;
    use crate::organize::Entry;

    fn options(template: &str) -> Options {
        Options {
            template: template.to_string(),
            ..Default::default()
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-rename-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// Creates files with the given names and returns entries for them, in the
    /// order they were named.
    fn folder(dir: &Path, names: &[&str]) -> Vec<Entry> {
        names
            .iter()
            .map(|name| {
                let path = dir.join(name);
                std::fs::write(&path, name.as_bytes()).unwrap();

                Entry::new(path)
            })
            .collect()
    }

    /// What the folder holds now, sorted so the assertion is stable.
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
    fn applying_renames_the_files_on_disk() {
        let dir = temp_dir("apply");
        let entries = folder(&dir, &["a.jpg", "b.jpg"]);

        let outcome = apply(&plan(&entries, &options("shot_{counter}")));

        assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
        assert_eq!(outcome.renamed.len(), 2);
        assert_eq!(listing(&dir), vec!["shot_0001.jpg", "shot_0002.jpg"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_contents_go_with_the_name() {
        let dir = temp_dir("contents");
        let entries = folder(&dir, &["first.jpg", "second.jpg"]);

        apply(&plan(&entries, &options("x{counter}")));

        assert_eq!(std::fs::read(dir.join("x0001.jpg")).unwrap(), b"first.jpg");
        assert_eq!(std::fs::read(dir.join("x0002.jpg")).unwrap(), b"second.jpg");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The case a one-at-a-time rename gets wrong: every file's new name is
    /// the next file's old one, so each rename lands on a file still needed.
    #[test]
    fn a_sequence_can_be_shifted_onto_itself() {
        let dir = temp_dir("shift");
        let entries = folder(&dir, &["2.jpg", "3.jpg", "4.jpg"]);

        let outcome = apply(&plan(
            &entries,
            &Options {
                template: "{counter}".into(),
                counter_digits: 1,
                ..Default::default()
            },
        ));

        assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
        assert_eq!(listing(&dir), vec!["1.jpg", "2.jpg", "3.jpg"]);
        assert_eq!(std::fs::read(dir.join("1.jpg")).unwrap(), b"2.jpg");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn two_files_can_swap_names() {
        let dir = temp_dir("swap");
        let mut entries = folder(&dir, &["a.jpg", "b.jpg"]);
        entries.reverse();

        let planned = vec![
            Planned {
                from: dir.join("a.jpg"),
                to: dir.join("b.jpg"),
                problem: None,
            },
            Planned {
                from: dir.join("b.jpg"),
                to: dir.join("a.jpg"),
                problem: None,
            },
        ];

        let outcome = apply(&planned);

        assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
        assert_eq!(std::fs::read(dir.join("a.jpg")).unwrap(), b"b.jpg");
        assert_eq!(std::fs::read(dir.join("b.jpg")).unwrap(), b"a.jpg");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_with_a_problem_is_left_where_it_is() {
        let dir = temp_dir("problem");
        let entries = folder(&dir, &["a.jpg", "b.jpg"]);

        // Both would be named the same, so neither should move.
        let outcome = apply(&plan(&entries, &options("same")));

        assert!(outcome.renamed.is_empty());
        assert_eq!(listing(&dir), vec!["a.jpg", "b.jpg"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_rating_follows_the_photograph_it_belongs_to() {
        let dir = temp_dir("sidecar");
        let entries = folder(&dir, &["a.jpg"]);
        std::fs::write(dir.join("a.jpg.xmp"), b"<x:xmpmeta/>").unwrap();

        apply(&plan(&entries, &options("renamed")));

        assert_eq!(listing(&dir), vec!["renamed.jpg", "renamed.jpg.xmp"]);
        assert_eq!(
            std::fs::read(dir.join("renamed.jpg.xmp")).unwrap(),
            b"<x:xmpmeta/>"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nothing_temporary_is_left_behind() {
        let dir = temp_dir("temporary");
        let entries = folder(&dir, &["a.jpg", "b.jpg", "c.jpg"]);

        apply(&plan(&entries, &options("x{counter}")));

        assert!(
            listing(&dir)
                .iter()
                .all(|name| !name.contains("avis-rename")),
            "{:?}",
            listing(&dir)
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_already_on_disk_and_not_part_of_the_job_is_in_the_way() {
        let dir = temp_dir("occupied");
        let entries = folder(&dir, &["a.jpg"]);
        std::fs::write(dir.join("taken.jpg"), b"someone else").unwrap();

        let planned = plan(&entries, &options("taken"));

        assert_eq!(planned[0].problem, Some(Problem::Exists));
        apply(&planned);
        assert_eq!(
            std::fs::read(dir.join("taken.jpg")).unwrap(),
            b"someone else"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_outcome_says_what_happened() {
        assert_eq!(Outcome::default().summary(), "Nothing to rename");
        assert_eq!(
            Outcome {
                renamed: vec![(PathBuf::from("a"), PathBuf::from("b"))],
                failed: Vec::new(),
            }
            .summary(),
            "Renamed 1 file(s)"
        );
    }

    #[test]
    fn a_sidecar_named_after_the_whole_file_follows_it() {
        let moved = sidecar_beside(
            Path::new("/photos/a.jpg.xmp"),
            Path::new("/photos/a.jpg"),
            Path::new("/photos/b.jpg"),
        );

        assert_eq!(moved, Path::new("/photos/b.jpg.xmp"));
    }

    #[test]
    fn a_sidecar_named_after_the_stem_follows_it_too() {
        let moved = sidecar_beside(
            Path::new("/photos/a.xmp"),
            Path::new("/photos/a.jpg"),
            Path::new("/photos/b.jpg"),
        );

        assert_eq!(moved, Path::new("/photos/b.xmp"));
    }
}
