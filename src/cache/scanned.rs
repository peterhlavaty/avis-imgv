//! The metadata read from the front of each file, and a ceiling on it.
//!
//! Reading the first half megabyte of a file gives its tags long before a
//! decoder gets to the pixels, and that is what the side panel, the status bar
//! and the filter all read. Keeping it is the whole point — asking the disk
//! again every time the cursor passes a photograph would undo the reason for
//! reading ahead at all.
//!
//! But it was kept for ever. Every file the preview reader touched left a map
//! of its tags behind, and nothing ever took one out: walking a folder of ten
//! thousand photographs built ten thousand of them, each a tree of strings and
//! possibly an embedded colour profile, entirely outside the budget the rest
//! of the cache is held to. The readout said nothing about it either, so the
//! viewer reported less memory than it was using and the difference grew with
//! every folder opened.
//!
//! So it is bounded, by bytes rather than by count for the same reason
//! everything else here is: one photograph's tags may be a few hundred bytes
//! and another's a two-kilobyte colour profile. What goes first is what was
//! read longest ago, which on a folder walked in order is what is furthest
//! behind.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

use crate::metadata::Metadata;

/// Metadata for the files read so far, up to a budget.
pub struct Scanned {
    entries: HashMap<PathBuf, Metadata>,
    /// The order they were read in, oldest first, for deciding what goes.
    read_order: VecDeque<PathBuf>,
    resident_bytes: usize,
    budget_bytes: usize,
}

impl Scanned {
    pub fn new(budget_bytes: usize) -> Scanned {
        Scanned {
            entries: HashMap::new(),
            read_order: VecDeque::new(),
            resident_bytes: 0,
            // Never nought, or the entry just read would be evicted before it
            // could be looked at.
            budget_bytes: budget_bytes.max(1),
        }
    }

    pub fn get(&self, path: &Path) -> Option<&Metadata> {
        self.entries.get(path)
    }

    pub fn insert(&mut self, path: PathBuf, metadata: Metadata) {
        let bytes = byte_len(&metadata);

        if let Some(replaced) = self.entries.insert(path.clone(), metadata) {
            self.resident_bytes = self.resident_bytes.saturating_sub(byte_len(&replaced));
            self.read_order.retain(|held| *held != path);
        }

        self.resident_bytes += bytes;
        self.read_order.push_back(path);

        self.evict_until_within_budget();
    }

    pub fn remove(&mut self, path: &Path) {
        if let Some(gone) = self.entries.remove(path) {
            self.resident_bytes = self.resident_bytes.saturating_sub(byte_len(&gone));
            self.read_order.retain(|held| held != path);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.read_order.clear();
        self.resident_bytes = 0;
    }

    pub fn resident_bytes(&self) -> usize {
        self.resident_bytes
    }

    pub fn budget_bytes(&self) -> usize {
        self.budget_bytes
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drops what was read longest ago until the budget is met.
    ///
    /// Never the last entry: the one just read is the one being looked at, and
    /// a budget smaller than a single photograph's tags must still leave that
    /// one there.
    fn evict_until_within_budget(&mut self) {
        while self.resident_bytes > self.budget_bytes && self.entries.len() > 1 {
            let Some(oldest) = self.read_order.pop_front() else {
                return;
            };

            if let Some(gone) = self.entries.remove(&oldest) {
                self.resident_bytes = self.resident_bytes.saturating_sub(byte_len(&gone));
            }
        }
    }
}

/// Roughly what one photograph's metadata costs.
///
/// Roughly, because an exact answer would mean walking every allocation in a
/// tree of strings, and the number is wanted to bound a cache rather than to
/// balance a ledger. The parts that actually vary are counted: the tag names
/// and their values, the embedded colour profile, and the keywords.
fn byte_len(metadata: &Metadata) -> usize {
    /// What a map entry costs beyond its contents: the node, the two lengths,
    /// the capacities.
    const PER_ENTRY: usize = 64;

    let tags: usize = metadata
        .tags
        .iter()
        .map(|(name, value)| name.len() + value.len() + PER_ENTRY)
        .sum();

    let icc = metadata.icc.as_ref().map_or(0, Vec::len);
    let keywords: usize = metadata
        .xmp
        .keywords
        .iter()
        .map(|keyword| keyword.len() + PER_ENTRY)
        .sum();

    tags + icc + keywords + PER_ENTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(tags: &[(&str, &str)]) -> Metadata {
        let mut found = Metadata::default();
        for (name, value) in tags {
            found.tags.insert(name.to_string(), value.to_string());
        }

        found
    }

    fn path(name: &str) -> PathBuf {
        PathBuf::from("/photos").join(name)
    }

    #[test]
    fn what_was_put_in_comes_back_out() {
        let mut scanned = Scanned::new(1 << 20);
        scanned.insert(path("a.jpg"), metadata(&[("ISO", "100")]));

        assert_eq!(
            scanned.get(&path("a.jpg")).and_then(|m| m.tags.get("ISO")),
            Some(&"100".to_string())
        );
        assert!(scanned.get(&path("b.jpg")).is_none());
    }

    #[test]
    fn nothing_read_yet_costs_nothing() {
        let scanned = Scanned::new(1 << 20);

        assert!(scanned.is_empty());
        assert_eq!(scanned.resident_bytes(), 0);
    }

    /// The bug this exists for: a folder walk used to leave an entry behind
    /// for every file, for ever.
    #[test]
    fn a_long_walk_stays_within_the_budget() {
        let one = byte_len(&metadata(&[("ISO", "100"), ("Aperture", "2.8")]));
        let mut scanned = Scanned::new(one * 10);

        for index in 0..1_000 {
            scanned.insert(
                path(&format!("{index}.jpg")),
                metadata(&[("ISO", "100"), ("Aperture", "2.8")]),
            );
        }

        assert!(
            scanned.resident_bytes() <= scanned.budget_bytes(),
            "{} over {}",
            scanned.resident_bytes(),
            scanned.budget_bytes()
        );
        assert!(scanned.len() <= 10, "{} entries", scanned.len());
    }

    /// And what is kept is what was read most recently.
    #[test]
    fn the_oldest_reading_is_what_goes() {
        let one = byte_len(&metadata(&[("ISO", "100")]));
        let mut scanned = Scanned::new(one * 3);

        for name in ["a.jpg", "b.jpg", "c.jpg", "d.jpg"] {
            scanned.insert(path(name), metadata(&[("ISO", "100")]));
        }

        assert!(scanned.get(&path("a.jpg")).is_none(), "the oldest stayed");
        assert!(scanned.get(&path("d.jpg")).is_some(), "the newest went");
    }

    /// A budget smaller than a single photograph's tags still holds the one
    /// being looked at, or the side panel would be empty.
    #[test]
    fn the_last_entry_is_never_evicted() {
        let mut scanned = Scanned::new(1);
        scanned.insert(path("a.jpg"), metadata(&[("ISO", "100")]));
        scanned.insert(path("b.jpg"), metadata(&[("ISO", "200")]));

        assert_eq!(scanned.len(), 1);
        assert!(scanned.get(&path("b.jpg")).is_some());
    }

    /// Reading the same file again replaces what was there rather than
    /// counting it twice.
    #[test]
    fn reading_the_same_file_again_does_not_double_count() {
        let mut scanned = Scanned::new(1 << 20);

        scanned.insert(path("a.jpg"), metadata(&[("ISO", "100")]));
        let once = scanned.resident_bytes();

        scanned.insert(path("a.jpg"), metadata(&[("ISO", "100")]));

        assert_eq!(scanned.resident_bytes(), once);
        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned.read_order.len(), 1);
    }

    #[test]
    fn removing_gives_the_bytes_back() {
        let mut scanned = Scanned::new(1 << 20);
        scanned.insert(path("a.jpg"), metadata(&[("ISO", "100")]));
        scanned.remove(&path("a.jpg"));

        assert!(scanned.is_empty());
        assert_eq!(scanned.resident_bytes(), 0);
        assert!(scanned.read_order.is_empty());
    }

    #[test]
    fn clearing_gives_all_of_them_back() {
        let mut scanned = Scanned::new(1 << 20);
        for name in ["a.jpg", "b.jpg"] {
            scanned.insert(path(name), metadata(&[("ISO", "100")]));
        }

        scanned.clear();

        assert!(scanned.is_empty());
        assert_eq!(scanned.resident_bytes(), 0);
    }

    /// A colour profile is the part that actually varies, and it is counted.
    #[test]
    fn an_embedded_profile_is_counted() {
        let bare = metadata(&[("ISO", "100")]);
        let mut with_profile = bare.clone();
        with_profile.icc = Some(vec![0; 4096]);

        assert!(byte_len(&with_profile) >= byte_len(&bare) + 4096);
    }
}
