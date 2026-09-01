//! What the history remembers, and the keys that walk it.
//!
//! `sc_undo` was `cull.sc_undo`, because the journal it walked covered nothing
//! but the things a cull does to files. It covers marks, settings and where
//! the program was pointed as well now, so the key belongs with the history
//! rather than with the deleting; the move is a document migration, and
//! whatever the key had been bound to is carried across.

use serde::{Deserialize, Serialize};

use super::defaults::{
    default_history_panel_width, default_merge_within_ms, default_sc_history, default_sc_redo,
    default_sc_undo,
};
use super::shortcut::Shortcut;

/// How much of what was done is kept, and how to walk it.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct HistoryConfig {
    /// How many deeds are kept, or nought for all of them.
    ///
    /// All of them is the default because a deed is a handful of paths and a
    /// small document, never a photograph: a day's culling is measured in
    /// kilobytes. A limit is there for somebody who would rather the list
    /// stayed short enough to read.
    #[serde(default)]
    pub remember: usize,

    /// Which kinds of thing one press of undo comes to rest on.
    #[serde(default)]
    pub undoes: Undoes,
    /// How close together two nudges have to be to count as one.
    ///
    /// A wheel turned twice or an arrow held down arrives once a frame. Nought
    /// switches the folding off, for somebody who wants every notch listed.
    #[serde(default = "default_merge_within_ms")]
    pub merge_within_ms: u64,

    /// How wide the panel is, in points.
    #[serde(default = "default_history_panel_width")]
    pub panel_width: f32,

    #[serde(default = "default_sc_undo")]
    pub sc_undo: Shortcut,
    #[serde(default = "default_sc_redo")]
    pub sc_redo: Shortcut,
    #[serde(default = "default_sc_history")]
    pub sc_panel: Shortcut,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        HistoryConfig {
            remember: 0,
            undoes: Undoes::default(),
            merge_within_ms: default_merge_within_ms(),
            panel_width: default_history_panel_width(),
            sc_undo: default_sc_undo(),
            sc_redo: default_sc_redo(),
            sc_panel: default_sc_history(),
        }
    }
}

/// Which classes of deed one press of undo comes to rest on.
///
/// Everything is recorded whatever this says, and everything is in the panel
/// and reachable by a click. What is switched off here is only whether
/// `Ctrl + Z` *stops* on it: with the view switched off, one press still lands
/// on the rating rather than on the wheel notch in front of it, and the zoom
/// goes back with it because everything between here and there did happen.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Undoes {
    /// The mode, the panels, the cursor, the zoom, the narrowing.
    #[serde(default = "yes")]
    pub view: bool,
    /// A field of the configuration.
    #[serde(default = "yes")]
    pub settings: bool,
    /// The photographs: their marks, and where they are on disk.
    #[serde(default = "yes")]
    pub content: bool,
}

fn yes() -> bool {
    true
}

impl Default for Undoes {
    fn default() -> Self {
        Undoes {
            view: true,
            settings: true,
            content: true,
        }
    }
}

impl Undoes {
    /// The names the flag row uses, which are what [`crate::history::Class`]
    /// answers to and what the configuration file carries.
    pub const NAMES: &'static [&'static str] = &["view", "settings", "content"];

    pub fn get(&self, name: &str) -> bool {
        match name {
            "view" => self.view,
            "settings" => self.settings,
            "content" => self.content,
            _ => false,
        }
    }

    pub fn set(&mut self, name: &str, on: bool) {
        match name {
            "view" => self.view = on,
            "settings" => self.settings = on,
            "content" => self.content = on,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default is all of them, which is what "nought" means here. A cap
    /// arrived at by accident would throw away work.
    #[test]
    fn nothing_is_forgotten_by_default() {
        assert_eq!(HistoryConfig::default().remember, 0);
    }

    /// Everything is stopped on until somebody says otherwise: a press that
    /// silently skipped a class would be a press that did nothing anybody
    /// asked for.
    #[test]
    fn undo_stops_on_everything_by_default() {
        let undoes = Undoes::default();

        for name in Undoes::NAMES {
            assert!(undoes.get(name), "{name} is not stopped on by default");
        }
    }

    /// The names are the ones `history::Class` answers to; if they drift, a
    /// tick in the settings would set a flag nothing reads.
    #[test]
    fn the_names_are_the_ones_the_classes_use() {
        let names: Vec<&str> = crate::history::Class::ALL
            .iter()
            .map(|class| class.name())
            .collect();

        assert_eq!(names, Undoes::NAMES);
    }

    #[test]
    fn an_unknown_name_reads_false_and_sets_nothing() {
        let mut undoes = Undoes::default();
        undoes.set("nonsense", false);

        assert!(!undoes.get("nonsense"));
        assert!(undoes.view && undoes.settings && undoes.content);
    }

    #[test]
    fn the_three_keys_are_all_different() {
        let config = HistoryConfig::default();

        assert_ne!(config.sc_undo, config.sc_redo);
        assert_ne!(config.sc_undo, config.sc_panel);
        assert_ne!(config.sc_redo, config.sc_panel);
    }
}
