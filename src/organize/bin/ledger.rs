//! Where each thing in the bin came from.
//!
//! A folder is a list of files and nothing else, so a bin that is only a
//! folder cannot say where anything in it belongs. This is the one thing
//! written beside them that a plain folder does not carry, and it is kept as
//! small as it can be: a name and a path, no dates, no sizes, nothing the
//! files themselves already say.
//!
//! It lives *inside* the bin, so emptying the bin forgets it in the same
//! `remove_dir_all` and two machines sharing a bin folder share its memory
//! too. It is never removed a row at a time: a row whose file has gone is
//! invisible to [`super::holds`] and live again the moment the file comes
//! back, which is what lets undo, redo and "put back" agree without any of
//! them telling the others anything.

use std::path::{Path, PathBuf};

/// What the note is called. Hidden by the leading dot where that means
/// anything, and skipped everywhere else because it is not a photograph.
pub const NAME: &str = ".avis-bin.json";

/// One thing the bin has held, and where it came from.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// What it is called inside the bin, which is not always what it was
    /// called outside it: two folders may both have had a `DSC0001.jpg`.
    pub name: String,
    /// Where it was when it was thrown out.
    pub from: PathBuf,
}

/// Reads the note, if there is one.
///
/// A bin with no note is a bin that has never held anything, which is an empty
/// list rather than a failure. A note that cannot be *parsed* is a failure, and
/// deliberately so: the same rule as a configuration section that was only
/// partly understood, for the same reason. Everything upstream of this refuses
/// to write over what it could not read, so a note damaged by something else
/// costs the origins it holds only if somebody empties the bin.
pub fn read(root: &Path) -> std::io::Result<Vec<Entry>> {
    let path = root.join(NAME);

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };

    serde_json::from_str(&text).map_err(|e| {
        std::io::Error::other(format!(
            "{} says where everything in the bin came from and could not be read: {e}",
            path.display()
        ))
    })
}

/// Writes the note, whole or not at all.
pub fn write(root: &Path, entries: &[Entry]) -> std::io::Result<()> {
    let text = serde_json::to_vec_pretty(entries).map_err(std::io::Error::other)?;

    crate::atomic::replace(&root.join(NAME), &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-ledger-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_bin_that_has_never_held_anything_reads_as_empty() {
        let dir = temp_dir("fresh");

        assert_eq!(read(&dir).unwrap(), Vec::new());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn what_is_written_is_what_is_read_back() {
        let dir = temp_dir("round-trip");
        let entries = vec![Entry {
            name: "DSC0001.jpg".to_string(),
            from: PathBuf::from("/photos/holiday/DSC0001.jpg"),
        }];

        write(&dir, &entries).unwrap();

        assert_eq!(read(&dir).unwrap(), entries);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Losing where a photograph came from is losing the only way back, so a
    /// note that cannot be read is a failure rather than an empty list — an
    /// empty list would be written over on the very next deletion.
    #[test]
    fn a_note_that_cannot_be_read_is_reported_rather_than_ignored() {
        let dir = temp_dir("damaged");
        std::fs::write(dir.join(NAME), b"{ not json").unwrap();

        assert!(read(&dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
