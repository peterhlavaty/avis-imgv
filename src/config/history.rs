//! What the history remembers, and the keys that walk it.
//!
//! `sc_undo` was `cull.sc_undo`, because the journal it walked covered nothing
//! but the things a cull does to files. It covers marks, settings and where
//! the program was pointed as well now, so the key belongs with the history
//! rather than with the deleting; the move is a document migration, and
//! whatever the key had been bound to is carried across.

use serde::{Deserialize, Serialize};

use super::defaults::{default_sc_redo, default_sc_undo};
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

    #[serde(default = "default_sc_undo")]
    pub sc_undo: Shortcut,
    #[serde(default = "default_sc_redo")]
    pub sc_redo: Shortcut,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        HistoryConfig {
            remember: 0,
            sc_undo: default_sc_undo(),
            sc_redo: default_sc_redo(),
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

    #[test]
    fn undo_and_redo_are_not_on_the_same_key() {
        let config = HistoryConfig::default();

        assert_ne!(config.sc_undo, config.sc_redo);
    }
}
