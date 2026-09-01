//! One thing that was done, and which class of thing it is.
//!
//! A deed is whatever a node of the history holds. It has to be runnable in
//! both directions and it has to be able to say what it was in a few words,
//! and that is the whole of the contract: nothing here knows whether the deed
//! arrived from a key, a menu, a gesture or the settings window.

use super::files::{Step, Way};
use super::snapshot::Change;

/// What kind of thing a deed is, so that undo can be told to step over some.
///
/// Recording everything and stepping over some of it is deliberately not the
/// same as not recording it. A class switched off is still in the panel and
/// still reachable by a click; `Ctrl + Z` simply does not stop on it. Somebody
/// who does not want a wheel notch between them and an undone rating gets that
/// without losing the record of where they had been looking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class {
    /// Where the program was pointed: the mode, the panels, the cursor, the
    /// zoom, the selection, what the folder is narrowed to.
    View,
    /// A field of the configuration.
    Settings,
    /// The photographs themselves: their marks, and where they are on disk.
    Content,
}

impl Class {
    /// Every class, in the order they are drawn.
    pub const ALL: &'static [Class] = &[Class::View, Class::Settings, Class::Content];

    /// The name the configuration knows this by.
    pub fn name(self) -> &'static str {
        match self {
            Class::View => "view",
            Class::Settings => "settings",
            Class::Content => "content",
        }
    }

    /// What to call it on screen.
    pub fn label(self) -> &'static str {
        match self {
            Class::View => "Where you were",
            Class::Settings => "Settings",
            Class::Content => "Photographs",
        }
    }
}

/// One thing that was done.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum Deed {
    /// The beginning, which is never run.
    ///
    /// A root that is a node like any other is what lets everything else
    /// assume it has a parent.
    Start,
    /// Something that touched files or marks.
    Files(Step),
    /// Something the program looked like that it no longer does.
    ///
    /// One row may hold several: two things that moved on the same frame moved
    /// together, and putting one back without the other would leave a state
    /// that never existed.
    Changed(Vec<Change>),
}

impl Deed {
    /// Which class this belongs to.
    pub fn class(&self) -> Class {
        match self {
            // The beginning is never stepped on, so its class never decides
            // anything; it is filed under the view because that is what the
            // state at the start of a run is.
            Deed::Start => Class::View,
            Deed::Files(_) => Class::Content,
            // A row that carries a settings change is a settings row even if
            // something else moved with it, because that is the half somebody
            // switching the class off is asking not to stop on.
            Deed::Changed(changes) => match changes.iter().any(Change::is_a_setting) {
                true => Class::Settings,
                false => Class::View,
            },
        }
    }

    /// What this was, in a few words, for the row that stands for it.
    pub fn label(&self) -> String {
        match self {
            Deed::Start => "Where this run started".to_string(),
            Deed::Files(step) => step.label(),
            Deed::Changed(changes) => match changes.split_first() {
                Some((first, [])) => first.label(),
                Some((first, rest)) => format!("{}, and {} more", first.label(), rest.len()),
                None => "Did nothing".to_string(),
            },
        }
    }

    /// What running it in this direction would do, for the sentence shown
    /// before anything happens.
    pub fn describe(&self, way: Way) -> String {
        match self {
            Deed::Start => "do nothing".to_string(),
            Deed::Files(step) => step.describe(way),
            Deed::Changed(_) => match way {
                Way::Back => format!("undo \"{}\"", self.label()),
                Way::Forward => format!("do \"{}\" again", self.label()),
            },
        }
    }

    /// How many files running this would touch.
    ///
    /// Nought for anything that does not reach the disk, which is what decides
    /// whether running it is worth asking about first.
    pub fn files(&self) -> usize {
        match self {
            Deed::Start => 0,
            Deed::Files(step) => step.files(),
            // Nothing here reaches the disk, so nothing here is ever asked
            // about before it runs.
            Deed::Changed(_) => 0,
        }
    }

    /// Every file this deed would touch.
    ///
    /// Empty for a row about where the program was pointed: nothing there
    /// reaches the disk, so a run spent looking around cannot make a history
    /// stale.
    pub fn paths(&self) -> Vec<std::path::PathBuf> {
        match self {
            Deed::Start | Deed::Changed(_) => Vec::new(),
            Deed::Files(step) => step.paths(),
        }
    }

    /// Whether this did nothing and so is not worth recording.
    pub fn is_empty(&self) -> bool {
        match self {
            Deed::Start => true,
            Deed::Files(step) => step.is_empty(),
            Deed::Changed(changes) => changes.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn a_file_deed_is_content_and_says_how_many_it_touches() {
        let deed = Deed::Files(Step::Binned(vec![PathBuf::from("a"), PathBuf::from("b")]));

        assert_eq!(deed.class(), Class::Content);
        assert_eq!(deed.files(), 2);
        assert!(!deed.is_empty());
    }

    /// The row in the panel is in the past tense; the sentence before an undo
    /// is about what is going to happen. They are not the same words.
    #[test]
    fn what_it_was_and_what_undoing_it_would_do_are_different_sentences() {
        let deed = Deed::Files(Step::Binned(vec![PathBuf::from("a")]));

        assert_eq!(deed.label(), "Sent a to the bin");
        assert_eq!(deed.describe(Way::Back), "bring 1 file back from the bin");
    }

    #[test]
    fn the_beginning_does_nothing_and_touches_nothing() {
        assert_eq!(Deed::Start.files(), 0);
        assert!(Deed::Start.is_empty());
        assert_eq!(Deed::Start.describe(Way::Back), "do nothing");
    }

    /// The names are what the configuration file carries, so they are part of
    /// the format and are not free to be reworded.
    #[test]
    fn every_class_has_a_name_and_a_label() {
        for class in Class::ALL {
            assert!(!class.name().is_empty());
            assert!(!class.label().is_empty());
        }

        assert_eq!(Class::ALL.len(), 3);
        assert_eq!(Class::View.name(), "view");
        assert_eq!(Class::Settings.name(), "settings");
        assert_eq!(Class::Content.name(), "content");
    }
}
