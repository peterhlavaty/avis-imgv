//! Moving a photograph without leaving anything of it behind.
//!
//! A picture on disk is often more than one file: the raw, the JPEG the camera
//! wrote beside it, and the sidecar holding the rating and keywords. Renaming
//! a shoot and tidying it into folders both move pictures, and both have to
//! take the sidecar with them — a rating left under a name nothing is called
//! any more is a rating lost.

use std::path::{Path, PathBuf};

use crate::annotations::sidecar;

/// Moves `from` to `to`, taking whatever sidecar belongs to it.
///
/// Nothing is ever moved onto something already there. `fs::rename` replaces
/// silently, which on a folder of photographs means one of them ceasing to
/// exist, so the destination is checked first and an occupied one is reported
/// rather than overwritten.
///
/// The sidecar is a convenience, so failing to move it is worth a line in the
/// log and not worth undoing the move over; failing to move the photograph is
/// reported.
pub fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    // A move onto the same file is the case that has to be allowed through:
    // it is how a case-only rename works on a case-insensitive filesystem.
    if to.exists() && !super::same_file(from, to) {
        return Err(occupied(to));
    }

    // The folder a photograph came from can be gone by the time it is put
    // back: taking a deletion back is exactly the case where the shoot was
    // tidied away afterwards, and a rename will not make a parent. The copy
    // has always done this.
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }

    carry(from, to)?;

    for candidate in sidecars_of(from) {
        let wanted = sidecar_beside(&candidate, from, to);

        if wanted.exists() && !super::same_file(&candidate, &wanted) {
            tracing::warn!(
                "Left {} where it is: {} is already there",
                candidate.display(),
                wanted.display()
            );
            continue;
        }

        if let Err(e) = carry(&candidate, &wanted) {
            tracing::warn!("Could not move {}: {e}", candidate.display());
        }
    }

    Ok(())
}

/// Moves one file, across filesystems where it has to be.
///
/// `fs::rename` cannot leave the device it started on, and leaving it is the
/// ordinary case here rather than the exotic one: the card a first pass is
/// done from is not the drive the bin folder is on, and neither is a share
/// over the network. So the failure that means exactly that is caught and
/// answered with a copy — and the source is only let go of once the copy has
/// arrived, so an interrupted move leaves the photograph where it was rather
/// than nowhere.
fn carry(from: &Path, to: &Path) -> std::io::Result<()> {
    match std::fs::rename(from, to) {
        Err(e) if crosses_devices(&e) => {}
        other => return other,
    }

    std::fs::copy(from, to)?;

    if let Err(e) = std::fs::remove_file(from) {
        // Half a move is worse than none: the photograph is now in two places
        // and the next thing to read the folder will find both.
        let _ = std::fs::remove_file(to);
        return Err(e);
    }

    Ok(())
}

/// Whether a rename failed because the two paths are on different filesystems.
///
/// By number rather than by [`std::io::ErrorKind`], which has no stable name
/// for this: `EXDEV` on every unix, `ERROR_NOT_SAME_DEVICE` on Windows.
fn crosses_devices(e: &std::io::Error) -> bool {
    let code = if cfg!(windows) { 17 } else { 18 };

    e.raw_os_error() == Some(code)
}

/// The sidecars that belong to `image` and to nothing else.
///
/// The extension-replacing form — Adobe's `DSC001.xmp` — belongs to the frame
/// rather than to one file of it, so it is only followed when no other image
/// in the folder shares the stem. Renaming the JPEG of a raw+JPEG pair used to
/// walk off with the raw's ratings.
pub fn sidecars_of(image: &Path) -> Vec<PathBuf> {
    let candidates = sidecar::candidates(image);
    let specific = candidates.first().cloned();

    candidates
        .into_iter()
        .filter(|candidate| candidate.exists())
        .filter(|candidate| Some(candidate) == specific.as_ref() || !stem_is_shared(image))
        .collect()
}

/// Whether another image in the folder has the same stem as `image`.
fn stem_is_shared(image: &Path) -> bool {
    let (Some(directory), Some(stem)) = (image.parent(), image.file_stem()) else {
        return false;
    };

    let Ok(entries) = std::fs::read_dir(directory) else {
        return false;
    };

    entries.flatten().any(|entry| {
        let path = entry.path();

        path != image && path.file_stem() == Some(stem) && crate::formats::is_supported(&path)
    })
}

/// Copies a photograph and its sidecars, as one unit.
///
/// The same rules as a move: nothing is written over, and the sidecar goes
/// with the photograph because a copy without its rating is a copy somebody
/// has to rate again.
pub fn copy_file(from: &Path, to: &Path) -> std::io::Result<Vec<PathBuf>> {
    if to.exists() {
        return Err(occupied(to));
    }

    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(from, to)?;
    let mut made = vec![to.to_path_buf()];

    for sidecar in sidecars_of(from) {
        let wanted = sidecar_beside(&sidecar, from, to);

        if wanted.exists() {
            tracing::warn!("Left {} alone: it is already there", wanted.display());
            continue;
        }

        match std::fs::copy(&sidecar, &wanted) {
            Ok(_) => made.push(wanted),
            Err(e) => tracing::warn!("Could not copy {}: {e}", sidecar.display()),
        }
    }

    Ok(made)
}

/// Sends a photograph and its sidecars to the platform's bin, as one unit.
///
/// Never `fs::remove_file`: culling is when people delete fastest and regret
/// hardest, and the bin is the only thing that makes the regret survivable.
/// The sidecar goes with it, because a rating left behind under a name nothing
/// is called any more will be read onto whatever takes that name next.
pub fn to_bin(image: &Path) -> std::io::Result<()> {
    let mut everything: Vec<PathBuf> = sidecars_of(image);
    everything.push(image.to_path_buf());

    trash::delete_all(&everything).map_err(|e| {
        std::io::Error::other(format!("{} could not go to the bin: {e}", image.display()))
    })
}

/// Deletes a photograph and its sidecars outright.
///
/// For the places the bin does not reach: a memory card, a network share, a
/// filesystem that has none. The caller is expected to have asked first.
pub fn delete(image: &Path) -> std::io::Result<()> {
    for sidecar in sidecars_of(image) {
        if let Err(e) = std::fs::remove_file(&sidecar) {
            tracing::warn!("Could not delete {}: {e}", sidecar.display());
        }
    }

    std::fs::remove_file(image)
}

fn occupied(path: &Path) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!("{} is already there", path.display()),
    )
}

/// Where a sidecar of `image` ends up when the image becomes `moved`.
///
/// Sidecars are named either `photo.jpg.xmp` or `photo.xmp`, and which of the
/// two this one is decides how much of its name is the image's.
pub fn sidecar_beside(candidate: &Path, image: &Path, moved: &Path) -> PathBuf {
    let suffix = candidate
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
            let image_name = image.file_name()?.to_str()?;
            name.strip_prefix(image_name)
        });

    match suffix {
        // `photo.jpg` + `.xmp`
        Some(suffix) => {
            let mut name = moved.file_name().unwrap_or_default().to_os_string();
            name.push(suffix);
            moved.with_file_name(name)
        }
        // `photo` + `.xmp`, which is what the extension replacing form gives.
        None => moved.with_extension(
            candidate
                .extension()
                .map(|ext| ext.to_string_lossy().into_owned())
                .unwrap_or_else(|| "xmp".to_string()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-files-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_sidecar_named_after_the_whole_file_follows_it() {
        let moved = sidecar_beside(
            Path::new("/photos/a.jpg.xmp"),
            Path::new("/photos/a.jpg"),
            Path::new("/photos/b.jpg"),
        );

        assert_eq!(moved, Path::new("/photos/b.jpg.xmp"));
    }

    #[test]
    fn a_sidecar_named_after_the_stem_follows_it_too() {
        let moved = sidecar_beside(
            Path::new("/photos/a.xmp"),
            Path::new("/photos/a.jpg"),
            Path::new("/photos/b.jpg"),
        );

        assert_eq!(moved, Path::new("/photos/b.xmp"));
    }

    #[test]
    fn a_sidecar_follows_a_photograph_into_another_folder() {
        let dir = temp_dir("into");
        let into = dir.join("hdr1");
        std::fs::create_dir_all(&into).unwrap();

        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();
        std::fs::write(dir.join("a.jpg.xmp"), b"<x:xmpmeta/>").unwrap();

        move_file(&dir.join("a.jpg"), &into.join("a.jpg")).unwrap();

        assert!(into.join("a.jpg").exists());
        assert!(into.join("a.jpg.xmp").exists());
        assert!(!dir.join("a.jpg").exists());
        assert!(!dir.join("a.jpg.xmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_photograph_with_no_sidecar_moves_just_the_same() {
        let dir = temp_dir("plain");
        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();

        move_file(&dir.join("a.jpg"), &dir.join("b.jpg")).unwrap();

        assert_eq!(std::fs::read(dir.join("b.jpg")).unwrap(), b"picture");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The folder a photograph came from can have been tidied away since, and
    /// putting it back has to make it again rather than fail.
    #[test]
    fn a_photograph_moves_into_a_folder_that_is_not_there_yet() {
        let dir = temp_dir("makes-the-folder");
        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();
        std::fs::write(dir.join("a.jpg.xmp"), b"<x:xmpmeta/>").unwrap();

        let into = dir.join("gone").join("deeper");
        move_file(&dir.join("a.jpg"), &into.join("a.jpg")).unwrap();

        assert!(into.join("a.jpg").exists());
        assert!(into.join("a.jpg.xmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_photograph_that_is_not_there_is_reported() {
        let dir = temp_dir("missing");

        assert!(move_file(&dir.join("gone.jpg"), &dir.join("b.jpg")).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `fs::rename` replaces, which on a folder of photographs means one of
    /// them ceasing to exist.
    #[test]
    fn a_photograph_is_never_moved_onto_another_one() {
        let dir = temp_dir("occupied");
        std::fs::write(dir.join("a.jpg"), b"the one being moved").unwrap();
        std::fs::write(dir.join("b.jpg"), b"the one already there").unwrap();

        assert!(move_file(&dir.join("a.jpg"), &dir.join("b.jpg")).is_err());
        assert_eq!(
            std::fs::read(dir.join("b.jpg")).unwrap(),
            b"the one already there"
        );
        assert!(dir.join("a.jpg").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Adobe's sidecar is named after the frame, not after one file of it, so
    /// renaming the JPEG of a raw+JPEG pair used to take the raw's ratings.
    #[test]
    fn a_shared_adobe_sidecar_stays_with_the_frame() {
        let dir = temp_dir("shared");
        std::fs::write(dir.join("IMG_1.jpg"), b"jpeg").unwrap();
        std::fs::write(dir.join("IMG_1.cr2"), b"raw").unwrap();
        std::fs::write(dir.join("IMG_1.xmp"), b"<x:xmpmeta/>").unwrap();

        move_file(&dir.join("IMG_1.jpg"), &dir.join("Holiday_1.jpg")).unwrap();

        assert!(dir.join("IMG_1.xmp").exists(), "the raw still has it");
        assert!(!dir.join("Holiday_1.xmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// With nothing else sharing the stem it is the frame's only sidecar and
    /// it does follow.
    #[test]
    fn an_unshared_adobe_sidecar_follows() {
        let dir = temp_dir("unshared");
        std::fs::write(dir.join("IMG_1.cr2"), b"raw").unwrap();
        std::fs::write(dir.join("IMG_1.xmp"), b"<x:xmpmeta/>").unwrap();

        move_file(&dir.join("IMG_1.cr2"), &dir.join("Holiday_1.cr2")).unwrap();

        assert!(dir.join("Holiday_1.xmp").exists());
        assert!(!dir.join("IMG_1.xmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn deleting_takes_the_sidecar_with_it() {
        let dir = temp_dir("delete");
        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();
        std::fs::write(dir.join("a.jpg.xmp"), b"<x:xmpmeta/>").unwrap();

        delete(&dir.join("a.jpg")).unwrap();

        assert!(!dir.join("a.jpg").exists());
        assert!(
            !dir.join("a.jpg.xmp").exists(),
            "a rating left behind is a rating waiting to land on the next \
             photograph to take that name"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The same rule as a move: Adobe's sidecar belongs to the frame, so it
    /// only goes when nothing else is using it.
    #[test]
    fn deleting_the_jpeg_of_a_pair_leaves_the_raws_sidecar() {
        let dir = temp_dir("delete-shared");
        std::fs::write(dir.join("IMG_1.jpg"), b"jpeg").unwrap();
        std::fs::write(dir.join("IMG_1.cr2"), b"raw").unwrap();
        std::fs::write(dir.join("IMG_1.xmp"), b"<x:xmpmeta/>").unwrap();

        delete(&dir.join("IMG_1.jpg")).unwrap();

        assert!(!dir.join("IMG_1.jpg").exists());
        assert!(dir.join("IMG_1.xmp").exists());
        assert!(dir.join("IMG_1.cr2").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn deleting_something_that_is_not_there_is_reported() {
        let dir = temp_dir("delete-missing");

        assert!(delete(&dir.join("gone.jpg")).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_sidecar_already_at_the_destination_is_left_alone() {
        let dir = temp_dir("sidecar-occupied");
        std::fs::write(dir.join("a.jpg"), b"picture").unwrap();
        std::fs::write(dir.join("a.jpg.xmp"), b"<x:xmpmeta/>").unwrap();
        std::fs::write(dir.join("b.jpg.xmp"), b"somebody else's work").unwrap();

        move_file(&dir.join("a.jpg"), &dir.join("b.jpg")).unwrap();

        assert_eq!(
            std::fs::read(dir.join("b.jpg.xmp")).unwrap(),
            b"somebody else's work"
        );
        assert!(dir.join("a.jpg.xmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
