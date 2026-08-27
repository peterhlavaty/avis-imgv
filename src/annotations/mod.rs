//! Star ratings and keywords: what the user puts on an image rather than what
//! the camera did.
//!
//! Both live in XMP sidecars, so they survive this viewer and are understood
//! by every raw converter. Reading is lazy — a sidecar is only looked for when
//! an image is actually shown — and writing happens on a thread of its own so
//! a slow disk cannot stutter the interface.

pub mod catalog;
pub mod recent;
pub mod sidecar;
pub mod writer;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::metadata::xmp::Xmp;

pub use catalog::Catalog;
pub use recent::RecentTags;

/// Annotations for the images seen so far, and the thread that persists them.
pub struct AnnotationStore {
    entries: HashMap<PathBuf, Xmp>,
    writer: writer::Writer,
}

impl Default for AnnotationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AnnotationStore {
    pub fn new() -> AnnotationStore {
        AnnotationStore {
            entries: HashMap::new(),
            writer: writer::Writer::new(),
        }
    }

    /// The annotations for `image`, loading them the first time it is asked
    /// for.
    ///
    /// `embedded` is what the image file itself carries, used when no sidecar
    /// exists yet: a rating set in Lightroom or Explorer shows up rather than
    /// being silently ignored.
    pub fn get(&mut self, image: &Path, embedded: Option<&Xmp>) -> &Xmp {
        if !self.entries.contains_key(image) {
            let loaded = sidecar::read(image)
                .or_else(|| embedded.cloned())
                .unwrap_or_default();

            self.entries.insert(image.to_path_buf(), loaded);
        }

        // The entry was just inserted if it was missing.
        self.entries.get(image).expect("entry present")
    }

    /// What is already known about `image`, without touching the disk.
    pub fn peek(&self, image: &Path) -> Option<&Xmp> {
        self.entries.get(image)
    }

    /// Sets the star rating, saving when it actually changes.
    pub fn set_rating(&mut self, image: &Path, rating: u8) -> bool {
        self.edit(image, |annotations| {
            let rating = rating.min(crate::metadata::xmp::MAX_RATING);
            let changed = annotations.rating != rating;
            annotations.rating = rating;

            changed
        })
    }

    /// Adds a keyword. Returns whether it was not already there.
    pub fn add_tag(&mut self, image: &Path, tag: &str) -> bool {
        let tag = tag.trim().to_string();
        if tag.is_empty() {
            return false;
        }

        self.edit(image, |annotations| {
            if annotations.keywords.contains(&tag) {
                return false;
            }

            annotations.keywords.push(tag.clone());
            annotations.keywords.sort();
            true
        })
    }

    /// Removes a keyword. Returns whether it was there.
    pub fn remove_tag(&mut self, image: &Path, tag: &str) -> bool {
        self.edit(image, |annotations| {
            let before = annotations.keywords.len();
            annotations.keywords.retain(|existing| existing != tag);

            annotations.keywords.len() != before
        })
    }

    /// Adds a keyword, or removes it when the image already has it.
    pub fn toggle_tag(&mut self, image: &Path, tag: &str) -> bool {
        let present = self
            .peek(image)
            .is_some_and(|annotations| annotations.keywords.iter().any(|k| k == tag));

        if present {
            self.remove_tag(image, tag);
            false
        } else {
            self.add_tag(image, tag);
            true
        }
    }

    /// Forgets what is cached for `image`, so it is read again.
    ///
    /// Used when the file changes underneath us.
    pub fn forget(&mut self, image: &Path) {
        self.entries.remove(image);
    }

    /// Forgets everything, for when the whole folder has moved underneath us.
    ///
    /// A bulk rename leaves every entry keyed by a name nothing is called any
    /// more, and a stale rating shown against the wrong photograph would be
    /// worse than reading them all again.
    pub fn forget_all(&mut self) {
        self.entries.clear();
    }

    /// Every keyword seen on the images visited so far.
    pub fn known_tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = self
            .entries
            .values()
            .flat_map(|annotations| annotations.keywords.iter().map(String::as_str))
            .collect();

        tags.sort_unstable();
        tags.dedup();
        tags
    }

    /// Blocks until everything queued has been written.
    pub fn flush(&self) {
        self.writer.flush();
    }

    /// Applies `change`, queueing a save when it reports something changed.
    fn edit(&mut self, image: &Path, change: impl FnOnce(&mut Xmp) -> bool) -> bool {
        let annotations = self.entries.entry(image.to_path_buf()).or_default();

        if !change(annotations) {
            return false;
        }

        self.writer.save(image.to_path_buf(), annotations.clone());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> AnnotationStore {
        AnnotationStore::new()
    }

    fn image() -> PathBuf {
        // Never written to: every test here edits entries that were seeded in
        // memory, and the writer only saves paths it is handed.
        std::env::temp_dir().join("avis-annotations-test/photo.jpg")
    }

    /// Seeds an entry without touching the disk.
    fn seed(store: &mut AnnotationStore, path: &Path, annotations: Xmp) {
        store.entries.insert(path.to_path_buf(), annotations);
    }

    #[test]
    fn an_unseen_image_starts_unrated_and_untagged() {
        let mut store = store();
        let annotations = store.get(&image(), None).clone();

        assert_eq!(annotations, Xmp::default());
    }

    #[test]
    fn what_the_file_carries_is_used_when_there_is_no_sidecar() {
        let mut store = store();
        let embedded = Xmp {
            rating: 3,
            keywords: vec!["FromTheFile".to_string()],
        };

        assert_eq!(store.get(&image(), Some(&embedded)), &embedded);
    }

    #[test]
    fn ratings_are_clamped() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        store.set_rating(&path, 99);

        assert_eq!(store.peek(&path).unwrap().rating, 5);
    }

    #[test]
    fn setting_the_same_rating_changes_nothing() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(store.set_rating(&path, 4));
        assert!(!store.set_rating(&path, 4));
    }

    #[test]
    fn tags_are_kept_sorted_and_unique() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(store.add_tag(&path, "Tatras"));
        assert!(store.add_tag(&path, "Autumn"));
        assert!(!store.add_tag(&path, "Tatras"));

        assert_eq!(
            store.peek(&path).unwrap().keywords,
            vec!["Autumn", "Tatras"]
        );
    }

    #[test]
    fn a_blank_tag_is_not_added() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(!store.add_tag(&path, "   "));
        assert!(store.peek(&path).unwrap().keywords.is_empty());
    }

    #[test]
    fn toggling_adds_then_removes() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(store.toggle_tag(&path, "Keeper"));
        assert_eq!(store.peek(&path).unwrap().keywords, vec!["Keeper"]);

        assert!(!store.toggle_tag(&path, "Keeper"));
        assert!(store.peek(&path).unwrap().keywords.is_empty());
    }

    #[test]
    fn removing_a_tag_that_is_not_there_changes_nothing() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(!store.remove_tag(&path, "Nothing"));
    }

    #[test]
    fn every_tag_seen_is_offered_back() {
        let mut store = store();
        seed(
            &mut store,
            Path::new("/a.jpg"),
            Xmp {
                rating: 0,
                keywords: vec!["Shared".to_string(), "One".to_string()],
            },
        );
        seed(
            &mut store,
            Path::new("/b.jpg"),
            Xmp {
                rating: 0,
                keywords: vec!["Shared".to_string()],
            },
        );

        assert_eq!(store.known_tags(), vec!["One", "Shared"]);
    }

    #[test]
    fn forgetting_an_image_drops_what_was_cached() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        store.forget(&path);

        assert!(store.peek(&path).is_none());
    }
}
