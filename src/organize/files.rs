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
/// The sidecar is a convenience, so failing to move it is worth a line in the
/// log and not worth undoing the move over; failing to move the photograph is
/// reported.
pub fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::rename(from, to)?;

    for candidate in sidecar::candidates(from) {
        if !candidate.exists() {
            continue;
        }

        let wanted = sidecar_beside(&candidate, from, to);
        if let Err(e) = std::fs::rename(&candidate, &wanted) {
            tracing::warn!("Could not move {}: {e}", candidate.display());
        }
    }

    Ok(())
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

    #[test]
    fn a_photograph_that_is_not_there_is_reported() {
        let dir = temp_dir("missing");

        assert!(move_file(&dir.join("gone.jpg"), &dir.join("b.jpg")).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
