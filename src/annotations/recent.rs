//! The tags most recently applied, remembered between sessions.
//!
//! A photographer working through a shoot reaches for the same handful of tags
//! over and over; having them one click away matters more than any catalog.

use std::collections::VecDeque;
use std::path::PathBuf;

use crate::{APPLICATION, ORGANIZATION, QUALIFIER};

const FILE_NAME: &str = "recent_tags.json";

/// A most-recently-used list of tags, capped so it stays a shortlist.
#[derive(Debug, Default)]
pub struct RecentTags {
    tags: VecDeque<String>,
    limit: usize,
    /// Set when the list changed and has not been saved yet.
    dirty: bool,
}

impl RecentTags {
    /// An empty list that is never read from disk.
    pub fn with_limit(limit: usize) -> RecentTags {
        RecentTags {
            tags: VecDeque::new(),
            limit: limit.max(1),
            dirty: false,
        }
    }

    /// Loads the list, or starts an empty one if there is nothing to load.
    pub fn load(limit: usize) -> RecentTags {
        let tags = path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
            .unwrap_or_default();

        let mut recent = RecentTags::with_limit(limit);

        // Rebuilt through `remember` so a hand-edited file cannot exceed the
        // limit or hold duplicates.
        for tag in tags.into_iter().rev() {
            recent.remember(tag);
        }
        recent.dirty = false;

        recent
    }

    /// Most recent first.
    /// Changes how many are kept, dropping any past the new limit.
    ///
    /// So that the setting takes effect while the window is open rather than
    /// at the next launch: the list is in memory and the number is the only
    /// thing that changed.
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = limit.max(1);
        self.tags.truncate(self.limit);
    }

    pub fn tags(&self) -> impl Iterator<Item = &str> {
        self.tags.iter().map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Moves `tag` to the front, dropping the oldest if the list is full.
    pub fn remember(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        let tag = tag.trim();

        if tag.is_empty() {
            return;
        }

        self.tags.retain(|existing| existing != tag);
        self.tags.push_front(tag.to_string());
        self.tags.truncate(self.limit);
        self.dirty = true;
    }

    /// Writes the list if it has changed since the last save.
    pub fn save_if_changed(&mut self) {
        if !self.dirty {
            return;
        }
        self.dirty = false;

        let Some(path) = path() else {
            return;
        };

        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::error!("Failure creating the configuration directory -> {e}");
                return;
            }
        }

        let tags: Vec<&str> = self.tags().collect();
        match serde_json::to_string_pretty(&tags) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    tracing::error!("Failure saving recent tags -> {e}");
                }
            }
            Err(e) => tracing::error!("Failure serialising recent tags -> {e}"),
        }
    }
}

/// Where the list is kept, beside the configuration.
fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.config_dir().join(FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A list that is never read from or written to disk.
    fn detached(limit: usize) -> RecentTags {
        RecentTags::with_limit(limit)
    }

    #[test]
    fn the_newest_tag_comes_first() {
        let mut recent = detached(8);
        recent.remember("Slovakia");
        recent.remember("Tatras");

        assert_eq!(
            recent.tags().collect::<Vec<_>>(),
            vec!["Tatras", "Slovakia"]
        );
    }

    #[test]
    fn reusing_a_tag_moves_it_back_to_the_front() {
        let mut recent = detached(8);
        recent.remember("A");
        recent.remember("B");
        recent.remember("A");

        assert_eq!(recent.tags().collect::<Vec<_>>(), vec!["A", "B"]);
    }

    #[test]
    fn the_list_stays_a_shortlist() {
        let mut recent = detached(3);
        for tag in ["A", "B", "C", "D"] {
            recent.remember(tag);
        }

        assert_eq!(recent.tags().collect::<Vec<_>>(), vec!["D", "C", "B"]);
    }

    #[test]
    fn blank_tags_are_not_remembered() {
        let mut recent = detached(8);
        recent.remember("");
        recent.remember("   ");

        assert!(recent.is_empty());
    }

    #[test]
    fn surrounding_space_is_trimmed() {
        let mut recent = detached(8);
        recent.remember("  Tatras  ");

        assert_eq!(recent.tags().collect::<Vec<_>>(), vec!["Tatras"]);
    }

    #[test]
    fn a_zero_limit_still_holds_one() {
        let mut recent = detached(0);
        recent.remember("A");

        assert_eq!(recent.tags().count(), 1);
    }

    #[test]
    fn saving_an_unchanged_list_does_nothing() {
        let mut recent = detached(8);

        // Would otherwise touch the real configuration directory.
        recent.save_if_changed();
        assert!(!recent.dirty);
    }
}
