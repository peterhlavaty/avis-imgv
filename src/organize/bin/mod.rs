//! The viewer's own bin: a folder rather than the platform's.
//!
//! The platform's bin stays the default, because Delete meaning what it means
//! in every other program is what nearly everybody expects. It has two costs,
//! though, and both of them land on somebody culling a shoot: it does not
//! reach a memory card or a share over the network at all, and it cannot be
//! *looked in* — the question after an hour of culling being "did I throw out
//! anything I meant to keep".
//!
//! This is the other answer, and it is deliberately nothing clever. A folder.
//! It opens like any other folder in this viewer, the frames in it are
//! browsed, compared and zoomed like any others, and emptying it is one
//! `remove_dir_all`. The only thing a folder cannot do by itself is say where
//! a file belongs, so that is written inside it, in [`ledger`].

pub mod ledger;

use std::path::{Path, PathBuf};

pub use ledger::Entry;

use crate::{APPLICATION, ORGANIZATION, QUALIFIER};

/// The most files that may share one name before the bin gives up.
///
/// A number rather than an unbounded loop: every attempt is a `stat`, and a
/// folder that somehow held ten thousand `DSC0001.jpg` would be a folder this
/// spends a second in every time anything is thrown out.
const CROWD: usize = 999;

/// Where the bin is when the configuration names no folder.
///
/// The *local* data directory rather than the roaming one: what goes in here
/// is photographs, and sixty megabytes a frame is not something to synchronise
/// onto somebody's other machine behind their back.
pub fn default_root() -> Option<PathBuf> {
    directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.data_local_dir().join("bin"))
}

/// Where the bin is, given what the configuration says.
///
/// Only an absolute path is taken. A relative one would be a different folder
/// in every shoot, which is not a bin: the question asked on the way out would
/// be about whichever one happened to be open, and putting a photograph back
/// would depend on where the viewer was standing. It falls back to the
/// viewer's own folder and [`crate::config::Config::check`] says so at load
/// rather than leaving the setting looking as though it did something.
pub fn root_from(named: Option<&str>) -> Option<PathBuf> {
    let path = PathBuf::from(named.unwrap_or_default().trim());

    match path.is_absolute() {
        true => Some(path),
        false => default_root(),
    }
}

/// Whether `path` is the bin, or something inside it.
///
/// Case-folded on Windows for the same reason [`crate::organize::same_file`]
/// is: the navigator will happily be given `c:\users` for a folder the program
/// knows as `C:\Users`, and a bin that stops being the bin when it is typed in
/// lower case is worse than no bin at all.
pub fn is_inside(root: &Path, path: &Path) -> bool {
    if cfg!(windows) {
        let fold = |path: &Path| path.as_os_str().to_string_lossy().to_lowercase();

        PathBuf::from(fold(path)).starts_with(PathBuf::from(fold(root)))
    } else {
        path.starts_with(root)
    }
}

/// A free name for `image` inside the bin, making the bin if it is not there.
///
/// Two folders on one card both hold a `DSC0001.jpg` and both of them may be
/// thrown out, so the name is only the photograph's where the photograph's is
/// free. A name the note has ever mentioned counts as taken even when nothing
/// is there under it any more: a row whose file has gone is how the bin
/// remembers a photograph that undo took back out, and reusing the name would
/// put a different picture behind that memory.
pub fn room_for(root: &Path, image: &Path) -> std::io::Result<PathBuf> {
    let held = ledger::read(root)?;
    std::fs::create_dir_all(root)?;

    let Some(name) = image.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} has no name to file it under", image.display()),
        ));
    };

    let free = |candidate: &Path| {
        let name = candidate.file_name().unwrap_or_default().to_string_lossy();

        !candidate.exists() && !held.iter().any(|entry| entry.name == name)
    };

    let plain = root.join(name);
    if free(&plain) {
        return Ok(plain);
    }

    let stem = image.file_stem().unwrap_or_default().to_string_lossy();
    let suffix = image
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();

    for nth in 2..=CROWD {
        let candidate = root.join(format!("{stem} ({nth}){suffix}"));

        if free(&candidate) {
            return Ok(candidate);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!(
            "the bin already holds {CROWD} things called {}",
            name.to_string_lossy()
        ),
    ))
}

/// Notes where the things that have just arrived came from.
///
/// `arrivals` is `(where it came from, where it landed)`, which is the shape
/// the move that put them there already reports. A name the note has heard of
/// is written over rather than added twice: the same photograph coming back
/// after being taken out is the ordinary case, and it comes back from the same
/// place it went.
pub fn note(root: &Path, arrivals: &[(PathBuf, PathBuf)]) -> std::io::Result<()> {
    if arrivals.is_empty() {
        return Ok(());
    }

    let mut held = ledger::read(root)?;

    for (from, landed) in arrivals {
        let Some(name) = landed.file_name() else {
            continue;
        };
        let name = name.to_string_lossy().into_owned();

        match held.iter_mut().find(|entry| entry.name == name) {
            Some(entry) => entry.from = from.clone(),
            None => held.push(Entry {
                name,
                from: from.clone(),
            }),
        }
    }

    ledger::write(root, &held)
}

/// What the bin holds, and where each of them came from.
///
/// Only the rows whose file is actually there. A note that cannot be read
/// leaves this empty rather than failing, because everything that asks is
/// asking in order to draw something.
pub fn holds(root: &Path) -> Vec<Entry> {
    ledger::read(root)
        .unwrap_or_default()
        .into_iter()
        .filter(|entry| root.join(&entry.name).exists())
        .collect()
}

/// Where the thing at `path` came from, if the bin put it there.
pub fn came_from(root: &Path, path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?.to_string_lossy().into_owned();

    ledger::read(root)
        .ok()?
        .into_iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.from)
}

/// How many photographs are in the bin.
///
/// The folder rather than the note, because the question is about the folder:
/// something dragged in by hand is still something in the bin, and something
/// the note remembers but the folder no longer has is not.
pub fn count(root: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(root) else {
        return 0;
    };

    entries
        .flatten()
        .filter(|entry| crate::formats::is_supported(&entry.path()))
        .count()
}

/// Empties the bin, and reports how many photographs went.
///
/// `remove_dir_all` against a path that came out of a text box is the most
/// dangerous line in this program, so it is only ever run where the note the
/// bin keeps about itself is. A folder this viewer never filled is left
/// exactly as it is and said so, which covers both of the ways a bin setting
/// ends up pointing at somebody's photographs: a typo, and a path pasted into
/// the wrong row.
pub fn empty(root: &Path) -> std::io::Result<usize> {
    if !root.join(ledger::NAME).is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "{} is not a bin this viewer filled, so nothing in it was touched",
                root.display()
            ),
        ));
    }

    let held = count(root);

    std::fs::remove_dir_all(root)?;
    // Made again rather than left missing: it may be the folder on screen, and
    // a bin that only comes back when the next photograph is thrown out is a
    // bin nobody can open and look in.
    std::fs::create_dir_all(root)?;

    Ok(held)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-bin-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// A whole deletion, the way `carry_out` does it: find room, move, note.
    fn throw_out(root: &Path, image: &Path) -> PathBuf {
        let inside = room_for(root, image).unwrap();
        crate::organize::files::move_file(image, &inside).unwrap();
        note(root, &[(image.to_path_buf(), inside.clone())]).unwrap();

        inside
    }

    #[test]
    fn a_photograph_keeps_its_name_where_the_name_is_free() {
        let dir = temp_dir("plain");
        let bin = dir.join("bin");
        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();

        let inside = throw_out(&bin, &dir.join("a.jpg"));

        assert_eq!(inside, bin.join("a.jpg"));
        assert_eq!(came_from(&bin, &inside), Some(dir.join("a.jpg")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Two folders on one card both hold a `DSC0001.jpg`, and both of them can
    /// be thrown out in the same session.
    #[test]
    fn a_second_photograph_of_the_same_name_gets_a_number() {
        let dir = temp_dir("clash");
        let bin = dir.join("bin");

        for folder in ["one", "two"] {
            std::fs::create_dir_all(dir.join(folder)).unwrap();
            std::fs::write(dir.join(folder).join("DSC0001.jpg"), folder).unwrap();
        }

        let first = throw_out(&bin, &dir.join("one").join("DSC0001.jpg"));
        let second = throw_out(&bin, &dir.join("two").join("DSC0001.jpg"));

        assert_eq!(first, bin.join("DSC0001.jpg"));
        assert_eq!(second, bin.join("DSC0001 (2).jpg"));
        assert_eq!(
            came_from(&bin, &second),
            Some(dir.join("two").join("DSC0001.jpg")),
            "each of them remembers its own folder"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The sidecar goes in with the photograph and comes back out with it.
    #[test]
    fn the_sidecar_follows_the_photograph_in_and_out() {
        let dir = temp_dir("sidecar");
        let bin = dir.join("bin");
        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();
        std::fs::write(dir.join("a.jpg.xmp"), b"<x:xmpmeta/>").unwrap();

        let inside = throw_out(&bin, &dir.join("a.jpg"));

        assert!(bin.join("a.jpg.xmp").exists());
        assert!(!dir.join("a.jpg.xmp").exists());

        let home = came_from(&bin, &inside).unwrap();
        crate::organize::files::move_file(&inside, &home).unwrap();

        assert!(dir.join("a.jpg").exists());
        assert!(dir.join("a.jpg.xmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A row whose file has gone is how the bin remembers something undo took
    /// back out: invisible while the file is away, live again the moment it
    /// returns, and nothing had to be told either time.
    #[test]
    fn a_photograph_taken_back_out_leaves_its_name_reserved() {
        let dir = temp_dir("reserved");
        let bin = dir.join("bin");
        std::fs::write(dir.join("a.jpg"), b"first").unwrap();
        std::fs::create_dir_all(dir.join("other")).unwrap();
        std::fs::write(dir.join("other").join("a.jpg"), b"second").unwrap();

        let inside = throw_out(&bin, &dir.join("a.jpg"));

        // Undo: back out of the bin, and the note left alone.
        crate::organize::files::move_file(&inside, &dir.join("a.jpg")).unwrap();
        assert!(holds(&bin).is_empty(), "nothing is in it");

        // Something else called the same thing must not take the name.
        let other = throw_out(&bin, &dir.join("other").join("a.jpg"));
        assert_eq!(other, bin.join("a (2).jpg"));

        // Redo: back in, and it is itself again with no bookkeeping.
        crate::organize::files::move_file(&dir.join("a.jpg"), &inside).unwrap();
        assert_eq!(came_from(&bin, &inside), Some(dir.join("a.jpg")));
        assert_eq!(holds(&bin).len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn emptying_takes_everything_and_says_how_much() {
        let dir = temp_dir("empty");
        let bin = dir.join("bin");
        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();
        throw_out(&bin, &dir.join("a.jpg"));

        assert_eq!(empty(&bin).unwrap(), 1);
        assert!(bin.is_dir(), "the folder is still there to be looked in");
        assert!(holds(&bin).is_empty());
        assert!(!bin.join(ledger::NAME).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The guard on the most dangerous line in the program: a folder full of
    /// somebody's photographs is not a bin, whatever a setting says.
    #[test]
    fn a_folder_this_viewer_never_filled_is_not_emptied() {
        let dir = temp_dir("not-a-bin");
        std::fs::write(dir.join("a.jpg"), b"somebody's work").unwrap();

        assert!(empty(&dir).is_err());
        assert!(dir.join("a.jpg").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A note that could not be read must not be written over: the origins in
    /// it are the only way back for everything already in the bin.
    #[test]
    fn a_bin_whose_note_cannot_be_read_takes_nothing_new() {
        let dir = temp_dir("damaged");
        let bin = dir.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join(ledger::NAME), b"{ not json").unwrap();

        assert!(room_for(&bin, Path::new("/photos/a.jpg")).is_err());
        assert!(note(&bin, &[(PathBuf::from("/photos/a.jpg"), bin.join("a.jpg"))]).is_err());
        assert_eq!(
            std::fs::read(bin.join(ledger::NAME)).unwrap(),
            b"{ not json"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A relative path is not a bin, and a bin nobody named is the viewer's
    /// own.
    #[test]
    fn only_an_absolute_path_is_taken_from_the_configuration() {
        let mine = if cfg!(windows) {
            r"D:\Deleted"
        } else {
            "/mnt/deleted"
        };

        assert_eq!(root_from(Some(mine)), Some(PathBuf::from(mine)));
        assert_eq!(root_from(Some(" ")), default_root());
        assert_eq!(root_from(None), default_root());
        assert_eq!(
            root_from(Some("Deleted")),
            default_root(),
            "a relative path would be a different bin in every folder"
        );
    }

    #[test]
    fn the_bin_knows_what_is_inside_it() {
        let (root, elsewhere) = if cfg!(windows) {
            (r"C:\Users\x\bin", r"C:\Users\x\photos\a.jpg")
        } else {
            ("/home/x/bin", "/home/x/photos/a.jpg")
        };
        let root = Path::new(root);

        assert!(is_inside(root, root));
        assert!(is_inside(root, &root.join("a.jpg")));
        assert!(!is_inside(root, Path::new(elsewhere)));
    }

    /// Windows hands the same folder back in whatever case it was typed in,
    /// and a bin that stops being the bin in lower case is a bin that deletes
    /// out of itself into itself.
    #[cfg(windows)]
    #[test]
    fn the_case_of_a_windows_path_does_not_decide_it() {
        assert!(is_inside(
            Path::new(r"C:\Users\x\Bin"),
            Path::new(r"c:\users\x\bin\a.jpg")
        ));
    }
}
