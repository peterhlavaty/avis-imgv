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

use crate::metadata::xmp::{self, Flag, Label, Xmp};

pub use catalog::Catalog;
pub use recent::RecentTags;

/// Annotations for the images seen so far, and the thread that persists them.
pub struct AnnotationStore {
    entries: HashMap<PathBuf, Xmp>,
    writer: writer::Writer,
    /// Bumped whenever the entries change in a way anything derived from them
    /// would notice.
    ///
    /// The keyword list the tag panel offers is a walk over every entry in the
    /// folder, sorted and deduplicated. It is wanted once a frame while the
    /// panel is open and changes only when somebody types a keyword, so the
    /// panel keeps its own copy and compares this rather than building it
    /// again on every frame of a folder with two thousand rated photographs
    /// in it.
    revision: u64,
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
            revision: 0,
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
            self.revision += 1;
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
        self.edit(image, |annotations| annotations.set_rating(rating))
    }

    /// Sets the pick or reject flag, saving when it actually changes.
    pub fn set_flag(&mut self, image: &Path, flag: Flag) -> bool {
        self.edit(image, |annotations| annotations.set_flag(flag))
    }

    /// Applies a flag, or clears it when the image already carries it.
    ///
    /// Pressing reject on something already rejected means "no, put that
    /// back", which is how every other program reads it.
    pub fn toggle_flag(&mut self, image: &Path, flag: Flag) -> bool {
        let wanted = if self.get(image, None).flag() == flag {
            Flag::Unflagged
        } else {
            flag
        };

        self.set_flag(image, wanted)
    }

    /// Sets the colour label, or clears it when the image already carries it.
    pub fn toggle_label(&mut self, image: &Path, label: Label) -> bool {
        let wanted =
            (self.get(image, None).known_label() != Some(label)).then(|| label.name().to_string());

        self.edit(image, |annotations| {
            let changed = annotations.label != wanted;
            annotations.label = wanted;

            changed
        })
    }

    /// Puts one particular colour label on, or takes it off.
    ///
    /// The state rather than the toggle, for when several photographs are
    /// being marked together and they have to end up the same as each other
    /// rather than each the opposite of what it was.
    pub fn set_label(&mut self, image: &Path, label: Option<Label>) -> bool {
        let wanted = label.map(|label| label.name().to_string());

        self.edit(image, |annotations| {
            let changed = annotations.label != wanted;
            annotations.label = wanted;

            changed
        })
    }

    /// Turns the photograph a quarter, and writes that to the sidecar.
    ///
    /// Never to the photograph. It is the most-expected verb after delete and
    /// the one most often implemented by rewriting the file — which loses a
    /// raw file outright and costs a JPEG a generation of quality, and which
    /// the user did not ask for and is not told about. What is written here is
    /// an orientation beside the rating, and the camera's own is left where it
    /// is.
    pub fn turn(&mut self, image: &Path, clockwise: bool) -> bool {
        self.edit(image, |annotations| {
            annotations.orientation = annotations.orientation.turned(clockwise);
            true
        })
    }

    /// Takes the colour label off, whatever it was.
    pub fn clear_label(&mut self, image: &Path) -> bool {
        self.edit(image, |annotations| annotations.label.take().is_some())
    }

    /// Adds a keyword. Returns whether it was not already there.
    ///
    /// A tag with bars in it is a path — `Places|Slovakia|Tatras` — and goes
    /// in twice: the whole path into the hierarchy, and its last level into
    /// the flat keywords. That is what Lightroom, darktable and digiKam all
    /// write, and it is why a keyword filed under two levels is still found by
    /// a program that has never heard of levels.
    pub fn add_tag(&mut self, image: &Path, tag: &str) -> bool {
        let path = tag.trim().to_string();
        if path.is_empty() {
            return false;
        }

        let levels = xmp::levels_of(&path);
        let Some(leaf) = levels.last().map(|leaf| (*leaf).to_string()) else {
            return false;
        };

        // Normalised, so `Places | Slovakia` and `Places|Slovakia` are one
        // keyword rather than two that look alike.
        let path = levels.join(&xmp::HIERARCHY_SEPARATOR.to_string());
        let filed = levels.len() > 1;

        self.edit(image, |annotations| {
            let mut changed = false;

            if !annotations.keywords.contains(&leaf) {
                annotations.keywords.push(leaf.clone());
                annotations.keywords.sort();
                changed = true;
            }

            if filed && !annotations.hierarchy.contains(&path) {
                annotations.hierarchy.push(path.clone());
                annotations.hierarchy.sort();
                changed = true;
            }

            changed
        })
    }

    /// Removes a keyword. Returns whether it was there.
    ///
    /// Takes the paths that end in it with it: leaving
    /// `Places|Slovakia|Tatras` behind after removing `Tatras` would leave the
    /// photograph tagged in Lightroom and untagged here.
    pub fn remove_tag(&mut self, image: &Path, tag: &str) -> bool {
        let leaf = xmp::leaf_of(tag).to_string();

        self.edit(image, |annotations| {
            let before = (annotations.keywords.len(), annotations.hierarchy.len());

            annotations.keywords.retain(|existing| *existing != leaf);
            annotations
                .hierarchy
                .retain(|path| xmp::leaf_of(path) != leaf);

            (annotations.keywords.len(), annotations.hierarchy.len()) != before
        })
    }

    /// Adds a keyword, or removes it when the image already has it.
    ///
    /// By the keyword rather than the whole path: a photograph tagged `Tatras`
    /// already has `Places|Slovakia|Tatras`, and a shortcut that added it a
    /// second time under its levels would never turn anything off.
    pub fn toggle_tag(&mut self, image: &Path, tag: &str) -> bool {
        let leaf = xmp::leaf_of(tag);
        let present = self
            .get(image, None)
            .keywords
            .iter()
            .any(|existing| existing == leaf);

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
        if self.entries.remove(image).is_some() {
            self.revision += 1;
        }
    }

    /// Forgets everything, for when the whole folder has moved underneath us.
    ///
    /// A bulk rename leaves every entry keyed by a name nothing is called any
    /// more, and a stale rating shown against the wrong photograph would be
    /// worse than reading them all again.
    pub fn forget_all(&mut self) {
        self.entries.clear();
        self.revision += 1;
    }

    /// A number that changes whenever the entries do.
    ///
    /// What anything caching a view of them compares, rather than rebuilding
    /// that view every frame.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Every keyword seen on the images visited so far.
    ///
    /// With its levels where the folder records them: a keyword the panel
    /// offers back should carry the same path it was read with, or applying it
    /// to the next photograph would quietly flatten it.
    pub fn known_tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = self
            .entries
            .values()
            .flat_map(|annotations| {
                annotations.keywords.iter().map(|keyword| {
                    annotations
                        .hierarchy
                        .iter()
                        .find(|path| xmp::leaf_of(path) == keyword)
                        .unwrap_or(keyword)
                        .as_str()
                })
            })
            .collect();

        // Sorted by the keyword itself, with the path breaking ties: two
        // photographs can file one keyword under different parents, and the
        // panel has room to offer it once. Ordering by the leaf puts those two
        // side by side, so the same one is dropped every time.
        tags.sort_unstable_by(|one, other| {
            (xmp::leaf_of(one), *one).cmp(&(xmp::leaf_of(other), *other))
        });
        tags.dedup_by(|one, other| xmp::leaf_of(one) == xmp::leaf_of(other));
        tags
    }

    /// Blocks until everything queued has been written.
    pub fn flush(&self) {
        self.writer.flush();
    }

    /// Saves that failed since this was last asked, for the user to be told.
    pub fn problems(&self) -> Vec<String> {
        self.writer.problems()
    }

    /// Applies `change`, queueing a save when it reports something changed.
    ///
    /// The entry is read from disk first when it is not already known. Editing
    /// an entry nobody has read is how keywords get lost: the save is built
    /// from whatever the entry holds, and a fabricated empty one writes a
    /// sidecar with no `dc:subject` at all. Rating an image the decoder cannot
    /// read used to do exactly that.
    fn edit(&mut self, image: &Path, change: impl FnOnce(&mut Xmp) -> bool) -> bool {
        if !self.entries.contains_key(image) {
            self.entries.insert(
                image.to_path_buf(),
                sidecar::read(image).unwrap_or_default(),
            );
        }

        let Some(annotations) = self.entries.get_mut(image) else {
            return false;
        };

        if !change(annotations) {
            return false;
        }

        let saved = annotations.clone();

        self.revision += 1;
        self.writer.save(image.to_path_buf(), saved);
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
            ..Xmp::default()
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
                ..Xmp::default()
            },
        );
        seed(
            &mut store,
            Path::new("/b.jpg"),
            Xmp {
                rating: 0,
                keywords: vec!["Shared".to_string()],
                ..Xmp::default()
            },
        );

        assert_eq!(store.known_tags(), vec!["One", "Shared"]);
    }

    /// What the tag panel compares instead of rebuilding its list.
    #[test]
    fn the_revision_moves_whenever_the_entries_do() {
        let dir = std::env::temp_dir().join("avis-annotations-revision");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let image = dir.join("a.jpg");

        let mut store = AnnotationStore::new();
        let start = store.revision();

        // Reading an image in is a change: its keywords join the folder's.
        store.get(&image, None);
        let after_read = store.revision();
        assert_ne!(after_read, start);

        // Reading it again is not.
        store.get(&image, None);
        assert_eq!(store.revision(), after_read);

        // A keyword is.
        assert!(store.add_tag(&image, "Tatras"));
        let after_tag = store.revision();
        assert_ne!(after_tag, after_read);

        // A keyword that was already there is not.
        assert!(!store.add_tag(&image, "Tatras"));
        assert_eq!(store.revision(), after_tag);

        // And forgetting is.
        store.forget(&image);
        assert_ne!(store.revision(), after_tag);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn forgetting_an_image_drops_what_was_cached() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        store.forget(&path);

        assert!(store.peek(&path).is_none());
    }

    #[test]
    fn a_tag_with_levels_is_filed_under_them_and_kept_flat_as_well() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(store.add_tag(&path, "Places|Slovakia|Tatras"));

        let annotations = store.peek(&path).unwrap();
        // The leaf is the keyword every reader understands...
        assert_eq!(annotations.keywords, vec!["Tatras"]);
        // ...and the path is there for the readers that understand levels.
        assert_eq!(annotations.hierarchy, vec!["Places|Slovakia|Tatras"]);
    }

    /// Typed with spaces round the bars, as somebody reading it aloud would.
    #[test]
    fn the_levels_of_a_tag_are_tidied_before_they_are_filed() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(store.add_tag(&path, " Places | Slovakia | Tatras "));
        assert_eq!(
            store.peek(&path).unwrap().hierarchy,
            vec!["Places|Slovakia|Tatras"]
        );

        // And so the same keyword written the other way is not filed twice.
        assert!(!store.add_tag(&path, "Places|Slovakia|Tatras"));
    }

    #[test]
    fn a_keyword_already_on_the_image_can_still_be_given_its_levels() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(store.add_tag(&path, "Tatras"));
        assert!(store.add_tag(&path, "Places|Slovakia|Tatras"));

        let annotations = store.peek(&path).unwrap();
        assert_eq!(annotations.keywords, vec!["Tatras"]);
        assert_eq!(annotations.hierarchy, vec!["Places|Slovakia|Tatras"]);
    }

    /// Removing the keyword takes its path with it: leaving the path behind
    /// would leave the photograph tagged in Lightroom and untagged here.
    #[test]
    fn removing_a_keyword_removes_the_paths_that_end_in_it() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        store.add_tag(&path, "Places|Slovakia|Tatras");
        store.add_tag(&path, "Places|Austria");

        assert!(store.remove_tag(&path, "Tatras"));

        let annotations = store.peek(&path).unwrap();
        assert_eq!(annotations.keywords, vec!["Austria"]);
        assert_eq!(annotations.hierarchy, vec!["Places|Austria"]);
    }

    #[test]
    fn a_shortcut_turns_a_filed_keyword_off_by_its_own_name() {
        let mut store = store();
        let path = image();
        seed(&mut store, &path, Xmp::default());

        assert!(store.toggle_tag(&path, "Places|Slovakia|Tatras"));
        // Off again, whether it is named by its path or by its leaf.
        assert!(!store.toggle_tag(&path, "Tatras"));

        let annotations = store.peek(&path).unwrap();
        assert!(annotations.keywords.is_empty());
        assert!(annotations.hierarchy.is_empty());
    }

    #[test]
    fn a_keyword_seen_with_levels_is_offered_back_with_them() {
        let mut store = store();
        let one = image();
        let other = one.with_file_name("other.jpg");

        seed(&mut store, &one, Xmp::default());
        seed(&mut store, &other, Xmp::default());
        store.add_tag(&one, "Places|Slovakia|Tatras");
        store.add_tag(&other, "Winter");

        assert_eq!(store.known_tags(), vec!["Places|Slovakia|Tatras", "Winter"]);
    }
}
