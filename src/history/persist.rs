//! Keeping the history between runs, and knowing when not to use it.
//!
//! A history written on the way out and read on the way in is only worth
//! having if it still describes the folder it was written about. Between two
//! runs a photograph can be renamed by something else, a sidecar edited in
//! another program, the whole folder synchronised from somewhere; and "undo"
//! against a file that is no longer what it was is exactly the operation this
//! program exists not to perform.
//!
//! So what is written beside the history is a signature of everything it
//! depends on, and it is read back only when that signature still holds.
//! Otherwise it is discarded and the user is told, which is the honest answer:
//! the alternative is an undo that quietly does something else.
//!
//! What goes into the signature is every file any deed mentions — as its size
//! and the time it was last written, or the fact that it is not there, which
//! is what a deed that binned something expects — and the configuration file,
//! because a settings row carries a whole configuration and putting one back
//! over an edit made elsewhere would lose it. Rows about where the program was
//! pointed name no files and put nothing in.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{Entry, History, Tree};

/// What this build writes, and the only thing it reads.
///
/// A file from another shape is discarded rather than guessed at. There is
/// nothing in a history worth a migration: the worst that happens is one run
/// that cannot take back what the run before it did.
const VERSION: u32 = 1;

/// How many rows are kept on disk.
///
/// `history.remember` is nought by default and a session's worth of rows costs
/// nothing in memory, but a settings row carries two whole configurations and
/// writing thousands of them to disk on every exit would be a slow exit and a
/// large file. The most recent are the ones anybody wants back.
const SAVED_AT_MOST: usize = 500;

/// The history as it goes to disk.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Saved {
    version: u32,
    /// What everything the history depends on looked like when it was written.
    signature: u64,
    tree: Tree<Entry>,
}

/// Where the file lives: beside the session, which is the other thing kept
/// between runs.
pub fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from(crate::QUALIFIER, crate::ORGANIZATION, crate::APPLICATION)
        .map(|dirs| dirs.config_dir().join("history.json"))
}

/// Writes the history, quietly.
///
/// A history that cannot be saved costs the next run the ability to take back
/// what this one did, and nothing else, so it is logged rather than reported.
pub fn save(history: &History, config: Option<&Path>) {
    let Some(path) = path() else {
        return;
    };

    if history.is_empty() {
        // Nothing was done. Leaving yesterday's file would offer to take back
        // something this run never did.
        let _ = std::fs::remove_file(&path);
        return;
    }

    let mut tree = history.tree().clone();
    tree.trim(SAVED_AT_MOST);

    let saved = Saved {
        version: VERSION,
        signature: signature(&mentioned(&tree), config),
        tree,
    };

    let Ok(text) = serde_json::to_string_pretty(&saved) else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Through the same one-step write everything else in this program uses, so
    // an interrupted exit leaves the old history rather than half of a new one.
    if let Err(e) = crate::atomic::replace(&path, text.as_bytes()) {
        tracing::warn!("The history could not be saved: {e}");
    }
}

/// What reading the history back came to.
pub enum Read {
    /// There was none, which is the ordinary first run.
    Nothing,
    /// It was read and still describes what is on disk.
    Kept(History),
    /// It was read and does not. Discarded, and worth saying so.
    Stale,
}

/// Reads the history, if it still describes the files it was written about.
pub fn load(remember: usize, config: Option<&Path>) -> Read {
    let Some(path) = path() else {
        return Read::Nothing;
    };

    let Ok(text) = std::fs::read_to_string(&path) else {
        return Read::Nothing;
    };

    let saved: Saved = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(saved) => saved,
        Err(e) => {
            tracing::warn!("The history could not be read, starting fresh: {e}");
            return Read::Nothing;
        }
    };

    if saved.version != VERSION {
        return Read::Nothing;
    }

    if saved.signature != signature(&mentioned(&saved.tree), config) {
        return Read::Stale;
    }

    Read::Kept(History::of(saved.tree, remember))
}

/// Every file the history mentions, in a settled order.
///
/// Sorted and deduplicated, because the signature has to come out the same
/// whatever order the rows happen to be walked in.
fn mentioned(tree: &Tree<Entry>) -> BTreeSet<PathBuf> {
    tree.in_order()
        .flat_map(|(_, node)| node.value.deed.paths())
        .collect()
}

/// A number that changes when anything the history depends on does.
///
/// FNV-1a rather than the standard hasher, because this is written to disk and
/// has to mean the same thing next week: `DefaultHasher` promises nothing about
/// staying the same between versions of the compiler, and a signature that
/// changed on its own would throw away a history that was perfectly good.
fn signature(paths: &BTreeSet<PathBuf>, config: Option<&Path>) -> u64 {
    let mut hash = Fnv::new();

    let everything = paths.iter().map(PathBuf::as_path).chain(config);

    for path in everything {
        hash.eat(path.to_string_lossy().as_bytes());

        match std::fs::metadata(path) {
            Ok(about) => {
                hash.eat(&about.len().to_le_bytes());

                // Not every platform keeps one, and a file that cannot say
                // when it was written still counts by its size and its name.
                if let Ok(written) = about.modified() {
                    if let Ok(since) = written.duration_since(std::time::UNIX_EPOCH) {
                        hash.eat(&since.as_secs().to_le_bytes());
                    }
                }
            }
            // Missing is a state like any other, and the one a deed that sent
            // something to the bin expects to find.
            Err(_) => hash.eat(b"gone"),
        }
    }

    hash.done()
}

/// FNV-1a, 64 bit. Small, and the same everywhere for ever.
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }

    fn eat(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x1000_0000_01b3);
        }

        // A separator, so that two fields running together cannot be confused
        // with the same bytes split differently.
        self.0 ^= 0xff;
        self.0 = self.0.wrapping_mul(0x1000_0000_01b3);
    }

    fn done(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{Deed, Step};
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-persist-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn history_of(paths: &[PathBuf]) -> History {
        let mut history = History::new();
        history.record(Deed::Files(Step::Binned(paths.to_vec())));
        history
    }

    #[test]
    fn a_signature_is_the_same_twice_over_the_same_files() {
        let dir = temp_dir("same");
        let file = dir.join("a.jpg");
        fs::write(&file, "one").unwrap();

        let paths: BTreeSet<PathBuf> = [file].into_iter().collect();

        assert_eq!(signature(&paths, None), signature(&paths, None));
    }

    /// The whole point: a file edited between two runs is a file the history
    /// no longer describes.
    #[test]
    fn a_changed_file_changes_the_signature() {
        let dir = temp_dir("changed");
        let file = dir.join("a.jpg");
        fs::write(&file, "one").unwrap();

        let paths: BTreeSet<PathBuf> = [file.clone()].into_iter().collect();
        let before = signature(&paths, None);

        fs::write(&file, "quite a lot more than one").unwrap();

        assert_ne!(before, signature(&paths, None));
    }

    /// And so is a file that has gone.
    #[test]
    fn a_file_that_went_changes_the_signature() {
        let dir = temp_dir("gone");
        let file = dir.join("a.jpg");
        fs::write(&file, "one").unwrap();

        let paths: BTreeSet<PathBuf> = [file.clone()].into_iter().collect();
        let before = signature(&paths, None);

        fs::remove_file(&file).unwrap();

        assert_ne!(before, signature(&paths, None));
    }

    /// A file that was never there is a state too, and a settled one: it must
    /// not read as different from itself.
    #[test]
    fn a_file_that_was_never_there_is_settled() {
        let paths: BTreeSet<PathBuf> = [PathBuf::from("/nowhere/at/all.jpg")].into_iter().collect();

        assert_eq!(signature(&paths, None), signature(&paths, None));
    }

    /// Two different sets of files must not come out the same. The separator
    /// in `eat` is what stops `["ab", "c"]` and `["a", "bc"]` colliding.
    #[test]
    fn different_files_have_different_signatures() {
        let one: BTreeSet<PathBuf> = [PathBuf::from("/a/b"), PathBuf::from("/c")]
            .into_iter()
            .collect();
        let other: BTreeSet<PathBuf> = [PathBuf::from("/a"), PathBuf::from("/b/c")]
            .into_iter()
            .collect();

        assert_ne!(signature(&one, None), signature(&other, None));
    }

    /// The configuration counts, because a settings row carries a whole one.
    #[test]
    fn the_configuration_counts() {
        let dir = temp_dir("config");
        let config = dir.join("config.json");
        fs::write(&config, "{}").unwrap();

        let paths = BTreeSet::new();
        let before = signature(&paths, Some(&config));

        fs::write(&config, "{\"general\": {}}").unwrap();

        assert_ne!(before, signature(&paths, Some(&config)));
    }

    /// Rows about where the program was pointed name no files, so a run spent
    /// looking around leaves the signature alone.
    #[test]
    fn a_row_about_the_view_mentions_no_files() {
        let mut history = History::new();
        history.note(
            vec![crate::history::Change::Cursor {
                from: 0,
                to: 1,
                name: String::new(),
            }],
            std::time::Duration::ZERO,
        );

        assert!(mentioned(history.tree()).is_empty());
    }

    /// Both photographs, and the sidecars each of them could have: undoing a
    /// mark writes a sidecar, so a sidecar edited elsewhere is a history that
    /// no longer describes the disk.
    #[test]
    fn a_row_about_files_mentions_them_and_their_sidecars() {
        let history = history_of(&[PathBuf::from("/a.jpg"), PathBuf::from("/b.jpg")]);
        let mentioned = mentioned(history.tree());

        assert!(mentioned.contains(&PathBuf::from("/a.jpg")));
        assert!(mentioned.contains(&PathBuf::from("/a.jpg.xmp")));
        assert!(mentioned.contains(&PathBuf::from("/a.xmp")));
        assert_eq!(mentioned.len(), 6);
    }

    /// A history survives the round trip with its shape and its place in it.
    #[test]
    fn a_history_comes_back_as_it_went() {
        let mut history = History::new();
        let first = history.record(binned("/a.jpg")).unwrap();
        history.record(binned("/b.jpg"));
        history.arrive(first);

        let text = serde_json::to_string(&Saved {
            version: VERSION,
            signature: 0,
            tree: history.tree().clone(),
        })
        .unwrap();

        let back: Saved = serde_json::from_str(&text).unwrap();
        let back = History::of(back.tree, 0);

        assert_eq!(back.len(), 2, "both rows, including the one taken back");
        assert_eq!(back.cursor(), first);
        assert_eq!(back.entry(first).unwrap().label, "Sent a.jpg to the bin");
    }

    fn binned(name: &str) -> Deed {
        Deed::Files(Step::Binned(vec![PathBuf::from(name)]))
    }
}
