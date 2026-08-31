//! Putting a file in place in one step.
//!
//! Written beside the target and renamed over it, so an interrupted write
//! leaves the old file intact rather than half of a new one. The sidecars have
//! always done this; the configuration and the session are the two other things
//! a person cannot rebuild by hand, and they used to be a plain `fs::write`.

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Puts `contents` at `path` in one step.
///
/// The temporary carries the process id and a counter, because two viewers may
/// be looking at the same folder.
pub fn replace(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let temporary = path.with_file_name(format!(
        ".{name}.{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));

    let written = (|| {
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()
    })();

    if let Err(e) = written {
        let _ = std::fs::remove_file(&temporary);
        return Err(e);
    }

    if let Err(e) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-atomic-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_write_lands_whole() {
        let dir = temp_dir("whole");
        let path = dir.join("a.json");

        replace(&path, b"{}").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The reason for the rename: the original is either the old one or the
    /// new one and never half of either.
    #[test]
    fn a_second_write_replaces_the_first() {
        let dir = temp_dir("replace");
        let path = dir.join("a.json");

        replace(&path, b"first").unwrap();
        replace(&path, b"second").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Nothing is left behind for the next reader to trip over.
    #[test]
    fn no_temporary_survives_a_write() {
        let dir = temp_dir("tidy");
        replace(&dir.join("a.json"), b"{}").unwrap();

        let left: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        assert_eq!(left, vec!["a.json".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A write into a directory that is not there fails rather than leaving a
    /// temporary somewhere unexpected.
    #[test]
    fn a_missing_directory_is_an_error() {
        let dir = temp_dir("missing");
        let path = dir.join("nowhere").join("a.json");

        assert!(replace(&path, b"{}").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
