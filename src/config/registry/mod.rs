//! One table the file and the window both read.
//!
//! The configuration is a hundred and forty fields spread over a dozen structs,
//! which is the right shape for reading it and the wrong shape for showing it
//! to somebody. This is the other view of the same thing: one row per field,
//! carrying where it is drawn, what it is called, what it means, what kind of
//! control it wants, when a change takes effect, where it is read, and the pair
//! of accessors that reach it.
//!
//! It is not a new idea in this codebase. `bindings` has been exactly this for
//! the sixty shortcut fields since the keyboard editor was written; this is the
//! same idea applied to the other eighty. Everything from here on — the pages,
//! the search, the changed-from-default marker, the per-field reset, the
//! restart footer, the load-time check, the export and the cheat sheet — is a
//! view over this table rather than a second copy of the truth.

pub mod access;
pub mod check;
pub mod effect;
pub mod page;
pub mod search;
mod table;

pub use access::{Access, Choice, Run};
pub use check::Complaint;
pub use effect::{Effect, Scope};
pub use page::{Group, Page};

use super::Config;

/// One field of the configuration, as everything but the file sees it.
pub struct Row {
    /// Where it is drawn.
    pub page: Page,
    /// Which block of the page it belongs to.
    pub group: Group,
    /// What it is called, in the words a person would use.
    pub label: &'static str,
    /// What it means, in a sentence. Becomes the help text under the control
    /// and a search term, which is why the doc comments had to be written.
    pub sentence: &'static str,
    /// Its path in the configuration file, which is also its identity: the
    /// registry is keyed on this and never on the label. nomacs stored
    /// shortcuts under their *translated* names and broke every one of them
    /// when the interface language changed.
    pub path: &'static str,
    /// Other programs' words for the same thing, and the complaint rather than
    /// the noun, in their spelling.
    pub aliases: &'static [&'static str],
    /// How to reach the value.
    pub access: Access,
    /// When a change takes effect.
    pub effect: Effect,
    /// Where the value is read, which for a shortcut is where it can clash.
    pub scope: Scope,
    /// Why this row has no control of its own, where it has none.
    ///
    /// Six controls come off the pages under the rule that a field is only a
    /// setting when two reasonable people would choose differently. Four of
    /// the six end with no control anywhere — the two GPU counts, the
    /// loaded-image radius and the upload budget — and each keeps a line on
    /// the page that used to hold it, saying where its value now comes from.
    /// The answer to "where did that setting go" belongs on the page rather
    /// than in a document nobody has.
    pub explained: Option<&'static str>,
}

impl Row {
    /// Whether this row differs from what a fresh configuration would hold.
    ///
    /// Computed against `Config::default()` rather than recorded in a second
    /// file. darktable keeps an origin map and its own bug is the warning: only
    /// *generated* preferences show its marker, so the marker cannot be
    /// trusted.
    pub fn changed(&self, config: &Config) -> bool {
        let fresh = Config::default();
        self.access.differs(config, &fresh)
    }

    /// The section of the file this row lives in: `cache` for
    /// `cache.ram_budget_mb`.
    pub fn section(&self) -> &'static str {
        match self.path.split_once('.') {
            Some((section, _)) => section,
            None => self.path,
        }
    }

    /// The key within its section.
    pub fn key(&self) -> &'static str {
        match self.path.split_once('.') {
            Some((_, key)) => key,
            None => self.path,
        }
    }
}

/// Every field, in the order the pages draw them.
pub fn rows() -> &'static [Row] {
    table::rows()
}

/// The row with this path, if the registry has one.
pub fn row(path: &str) -> Option<&'static Row> {
    rows().iter().find(|row| row.path == path)
}

/// Every row drawn on one page, block by block in reading order.
pub fn on_page(page: Page) -> impl Iterator<Item = &'static Row> {
    let mut found: Vec<&'static Row> = rows().iter().filter(|row| row.page == page).collect();

    // A stable sort, so within a block the rows keep the order the table
    // declares them in — which is the order somebody reading the file sees.
    found.sort_by_key(|row| row.group.order());

    found.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// The test the whole plan rests on: adding a field to `Config` without
    /// adding a row fails the build, and so does a row naming a key the file
    /// does not carry. It is the generalisation of the count assertion the
    /// keyboard list has always had.
    #[test]
    fn every_field_is_in_the_index() {
        let document = serde_json::to_value(Config::default()).expect("the defaults serialise");
        let object = document.as_object().expect("a configuration is an object");

        let mut in_file: HashSet<String> = HashSet::new();
        for (section, value) in object {
            match value.as_object() {
                Some(fields) => {
                    for key in fields.keys() {
                        in_file.insert(format!("{section}.{key}"));
                    }
                }
                None => {
                    in_file.insert(section.clone());
                }
            }
        }

        let in_registry: HashSet<String> = rows()
            .iter()
            // A bracket says the row is one element of a list rather than a
            // key of its own; the list itself has a row.
            .filter(|row| !row.path.contains('['))
            .filter(|row| !matches!(row.access, Access::Fixed(_) | Access::Run(_)))
            .map(|row| row.path.to_string())
            .collect();

        let missing: Vec<&String> = in_file.difference(&in_registry).collect();
        assert!(
            missing.is_empty(),
            "the file has keys the registry has never heard of: {missing:?}"
        );

        let invented: Vec<&String> = in_registry.difference(&in_file).collect();
        assert!(
            invented.is_empty(),
            "the registry names keys the file does not carry: {invented:?}"
        );
    }

    /// A path is an identity, so two rows may not share one.
    #[test]
    fn no_path_is_used_twice() {
        let mut seen: HashSet<&str> = HashSet::new();

        for row in rows() {
            assert!(seen.insert(row.path), "{} appears twice", row.path);
        }
    }

    /// Every row can be found, said and explained.
    #[test]
    fn every_row_is_complete() {
        for row in rows() {
            assert!(!row.label.is_empty(), "{} has no label", row.path);
            assert!(!row.sentence.is_empty(), "{} has no sentence", row.path);
            assert!(
                row.sentence.len() > 12,
                "{} has a sentence too short to help: {:?}",
                row.path,
                row.sentence
            );
        }
    }

    /// A fresh configuration has changed nothing, by definition.
    #[test]
    fn nothing_differs_from_the_defaults_on_a_fresh_configuration() {
        let fresh = Config::default();

        for row in rows() {
            assert!(!row.changed(&fresh), "{} says it was changed", row.path);
        }
    }

    /// And a field that was changed says so.
    #[test]
    fn a_changed_field_says_so() {
        let mut config = Config::default();
        config.cache.ram_budget_mb = 8192;

        let row = row("cache.ram_budget_mb").expect("the registry has it");
        assert!(row.changed(&config));

        let other = row_or_fail("cache.gpu_budget_mb");
        assert!(!other.changed(&config));
    }

    fn row_or_fail(path: &str) -> &'static Row {
        row(path).unwrap_or_else(|| panic!("{path} is not in the registry"))
    }

    #[test]
    fn a_path_splits_into_a_section_and_a_key() {
        let row = row_or_fail("cache.ram_budget_mb");

        assert_eq!(row.section(), "cache");
        assert_eq!(row.key(), "ram_budget_mb");
    }
}
