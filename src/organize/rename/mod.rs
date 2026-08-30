//! Renaming a folder full of photographs at once.
//!
//! A name is written as a template: literal text with `{...}` placeholders for
//! the parts that differ. What each file becomes is worked out first, in full,
//! and shown; nothing is touched until that is applied.
//!
//! ```text
//! {date}_{counter}          2024-11-06_0001.jpg
//! {name}_{tag:ISO}          DSCF0001_400.jpg
//! Holiday {counter} ({time})  Holiday 007 (22-07-19).jpg
//! ```

mod apply;
mod template;

pub use apply::{apply, Outcome};
pub use template::PLACEHOLDERS;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::{same_file, Entry};

use template::{render, sanitize};

/// What the extension of the renamed file should look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Extension {
    #[default]
    Keep,
    Lowercase,
    Uppercase,
}

impl Extension {
    pub const CHOICES: &'static [Extension] =
        &[Extension::Keep, Extension::Lowercase, Extension::Uppercase];

    pub fn label(self) -> &'static str {
        match self {
            Extension::Keep => "Keep as it is",
            Extension::Lowercase => "lowercase",
            Extension::Uppercase => "UPPERCASE",
        }
    }

    fn apply(self, extension: &str) -> String {
        match self {
            Extension::Keep => extension.to_string(),
            Extension::Lowercase => extension.to_lowercase(),
            Extension::Uppercase => extension.to_uppercase(),
        }
    }
}

/// How to build the new names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub template: String,
    /// What the first file is numbered.
    pub counter_start: usize,
    /// How much the number goes up by between files.
    pub counter_step: usize,
    /// How many digits the number is padded to. A number too long for the
    /// padding is written in full rather than truncated.
    pub counter_digits: usize,
    pub extension: Extension,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            template: "{name}".to_string(),
            counter_start: 1,
            counter_step: 1,
            counter_digits: 4,
            extension: Extension::Keep,
        }
    }
}

/// Why a planned name cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    /// The template produced nothing at all for this file.
    Empty,
    /// Two files in the job would end up with the same name.
    Collides,
    /// Something already on disk is in the way.
    Exists,
}

impl Problem {
    pub fn message(self) -> &'static str {
        match self {
            Problem::Empty => "the template leaves this one with no name",
            Problem::Collides => "two files would end up with this name",
            Problem::Exists => "a file of this name is already there",
        }
    }
}

/// One file and what it is to become.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub from: PathBuf,
    pub to: PathBuf,
    pub problem: Option<Problem>,
}

impl Planned {
    /// Whether this file would actually move.
    pub fn changes(&self) -> bool {
        self.problem.is_none() && !same_file(&self.from, &self.to)
    }

    pub fn new_name(&self) -> String {
        self.to
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
}

/// Works out what every entry becomes, in the order they are given.
///
/// The order is the whole reason sorting matters: the counter follows it.
pub fn plan(entries: &[Entry], options: &Options) -> Vec<Planned> {
    let mut planned = Vec::with_capacity(entries.len());
    let mut counter = options.counter_start;

    for entry in entries {
        let name = render(&options.template, entry, counter, options.counter_digits);
        let name = sanitize(&name);

        let to = match name.is_empty() {
            true => entry.path.clone(),
            false => with_name(&entry.path, &name, options.extension),
        };

        planned.push(Planned {
            from: entry.path.clone(),
            to,
            problem: name.is_empty().then_some(Problem::Empty),
        });

        counter = counter.saturating_add(options.counter_step);
    }

    mark_collisions(&mut planned);
    planned
}

/// Flags the plans that would land on the same name, or on a file that is
/// already there and is not part of the job.
fn mark_collisions(planned: &mut [Planned]) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut collided: Vec<usize> = Vec::new();

    for (index, plan) in planned.iter().enumerate() {
        let key = comparable(&plan.to);

        match seen.get(&key) {
            // Both of them are the problem; flagging only the second would
            // hide half of it.
            Some(&first) => collided.extend([first, index]),
            None => {
                seen.insert(key, index);
            }
        }
    }

    for index in collided {
        planned[index].problem.get_or_insert(Problem::Collides);
    }

    // A file already on disk is only in the way if it is not one of ours, and
    // one of ours only gets out of the way if it is actually going to move: a
    // plan with a problem, or one whose name does not change, keeps its name.
    //
    // Marking a plan blocked can block another, so this settles rather than
    // running once — otherwise a file could be renamed onto a name the job
    // had already decided not to vacate.
    loop {
        let vacated: HashSet<String> = planned
            .iter()
            .filter(|plan| plan.changes())
            .map(|plan| comparable(&plan.from))
            .collect();

        let mut marked = false;

        for plan in planned.iter_mut() {
            if !plan.changes() {
                continue;
            }

            if !vacated.contains(&comparable(&plan.to)) && plan.to.exists() {
                plan.problem = Some(Problem::Exists);
                marked = true;
            }
        }

        if !marked {
            break;
        }
    }
}

/// A path as a key, comparing the way the platform does.
fn comparable(path: &Path) -> String {
    let text = path.to_string_lossy().into_owned();

    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

/// Replaces the name of `path`, keeping its folder and extension.
fn with_name(path: &Path, name: &str, extension: Extension) -> PathBuf {
    let renamed = path.with_file_name(name);

    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => renamed.with_extension(extension.apply(ext)),
        None => renamed,
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::entry;
    use super::super::CAPTURE_TAG;
    use super::*;

    fn dated(name: &str) -> Entry {
        entry(
            name,
            0,
            &[(CAPTURE_TAG, "2024:11:06 22:07:19"), ("ISO", "400")],
        )
    }

    fn options(template: &str) -> Options {
        Options {
            template: template.to_string(),
            ..Default::default()
        }
    }

    fn names(planned: &[Planned]) -> Vec<String> {
        planned.iter().map(Planned::new_name).collect()
    }

    #[test]
    fn the_counter_follows_the_order_it_is_given() {
        let planned = plan(
            &[dated("b.jpg"), dated("a.jpg"), dated("c.jpg")],
            &options("{counter}"),
        );

        assert_eq!(names(&planned), vec!["0001.jpg", "0002.jpg", "0003.jpg"]);
    }

    #[test]
    fn the_counter_starts_and_steps_where_it_is_told() {
        let planned = plan(
            &[dated("a.jpg"), dated("b.jpg")],
            &Options {
                template: "{counter}".into(),
                counter_start: 10,
                counter_step: 5,
                counter_digits: 3,
                ..Default::default()
            },
        );

        assert_eq!(names(&planned), vec!["010.jpg", "015.jpg"]);
    }

    #[test]
    fn a_number_too_long_for_its_padding_is_written_in_full() {
        let planned = plan(
            &[dated("a.jpg")],
            &Options {
                template: "{counter}".into(),
                counter_start: 12_345,
                counter_digits: 2,
                ..Default::default()
            },
        );

        assert_eq!(names(&planned), vec!["12345.jpg"]);
    }

    #[test]
    fn a_template_that_leaves_nothing_is_a_problem_rather_than_a_rename() {
        let planned = plan(&[entry("a.jpg", 0, &[])], &options("{date}"));

        assert_eq!(planned[0].problem, Some(Problem::Empty));
        assert!(!planned[0].changes());
        assert_eq!(planned[0].to, planned[0].from, "it stays where it is");
    }

    #[test]
    fn two_files_that_would_share_a_name_are_both_flagged() {
        let planned = plan(&[dated("a.jpg"), dated("b.jpg")], &options("same"));

        assert_eq!(planned[0].problem, Some(Problem::Collides));
        assert_eq!(planned[1].problem, Some(Problem::Collides));
    }

    #[test]
    fn files_of_different_types_do_not_collide() {
        let planned = plan(&[dated("a.jpg"), dated("a.cr3")], &options("shot"));

        assert!(planned.iter().all(|plan| plan.problem.is_none()));
        assert_eq!(names(&planned), vec!["shot.jpg", "shot.cr3"]);
    }

    #[test]
    fn a_file_already_named_what_it_would_be_named_is_left_alone() {
        let planned = plan(&[dated("keep.jpg")], &options("{name}"));

        assert_eq!(planned[0].problem, None);
        assert!(!planned[0].changes(), "nothing to do");
    }

    #[test]
    fn the_extension_can_be_folded_to_one_case() {
        let planned = plan(
            &[entry("a.JPG", 0, &[])],
            &Options {
                template: "{name}x".into(),
                extension: Extension::Lowercase,
                ..Default::default()
            },
        );

        assert_eq!(names(&planned), vec!["ax.jpg"]);
    }
}
