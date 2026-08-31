//! XMP sidecar files: where they live, and reading and writing them.
//!
//! Ratings and keywords go beside the image rather than into it. Rewriting a
//! photograph to change a star is both slow and risky, and every raw converter
//! already looks for a sidecar.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::atomic::replace;
use crate::metadata::xmp::{self, Xmp};

/// Where a sidecar for `image` is written.
///
/// The whole file name is kept, so `DSC001.jpg.xmp` and `DSC001.cr2.xmp` stay
/// apart — a raw and a JPEG of the same frame are different images with
/// possibly different keywords.
pub fn path_for(image: &Path) -> PathBuf {
    let mut name = image.file_name().unwrap_or_default().to_os_string();
    name.push(".xmp");

    image.with_file_name(name)
}

/// Sidecars to look for, most specific first.
///
/// Adobe writes `DSC001.xmp` next to a raw file; darktable and exiftool write
/// `DSC001.cr2.xmp`. Both are read, and the more specific one wins.
pub fn candidates(image: &Path) -> Vec<PathBuf> {
    let mut paths = vec![path_for(image)];

    if let Some(stem) = image.file_stem() {
        let adobe = image.with_file_name(stem).with_extension("xmp");

        // The image is never its own sidecar, which it would be if it were
        // itself an .xmp file.
        if adobe != paths[0] && adobe != image {
            paths.push(adobe);
        }
    }

    paths
}

/// Which naming a new sidecar gets, for the whole process.
///
/// A static rather than a parameter threaded through every caller: the writing
/// happens on a background thread with an image path and nothing else, and this
/// is one value the whole program agrees about. Both forms are still *read*,
/// most specific first, and a sidecar that already exists is edited rather than
/// joined by a second — so changing this changes only what gets created for a
/// photograph that has none.
static ADOBE_NAMING: AtomicBool = AtomicBool::new(false);

/// Sets it from the configuration.
pub fn name_like_adobe(on: bool) {
    ADOBE_NAMING.store(on, Ordering::Relaxed);
}

/// Where a sidecar this viewer creates goes.
pub fn new_path_for(image: &Path) -> PathBuf {
    if !ADOBE_NAMING.load(Ordering::Relaxed) {
        return path_for(image);
    }

    match image.file_stem() {
        Some(stem) => {
            let adobe = image.with_file_name(stem).with_extension("xmp");
            // Never the photograph itself, which it would be for an .xmp.
            if adobe == image {
                path_for(image)
            } else {
                adobe
            }
        }
        None => path_for(image),
    }
}

/// Reads the first sidecar that exists and parses.
pub fn read(image: &Path) -> Option<Xmp> {
    candidates(image)
        .into_iter()
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .find_map(|document| xmp::read(&document))
}

/// Writes `annotations` to the sidecar, preserving whatever else it holds.
///
/// A sidecar that cannot be read, or that the writer cannot rewrite without
/// losing what is in it, is reported rather than replaced: it may be holding a
/// raw converter's entire develop history.
pub fn write(image: &Path, annotations: &Xmp) -> std::io::Result<()> {
    // Edit whichever sidecar is already there rather than adding a second one
    // beside it.
    let target = candidates(image)
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| new_path_for(image));

    let existing = match std::fs::read_to_string(&target) {
        Ok(document) => Some(document),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };

    let Some(document) = xmp::update(existing.as_deref(), annotations) else {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "the sidecar could not be rewritten without losing what it holds",
        ));
    };

    replace(&target, document.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique directory for one test, cleaned up by the caller.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("avis-sidecar-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn annotations(rating: i8, keywords: &[&str]) -> Xmp {
        Xmp {
            rating,
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            ..Xmp::default()
        }
    }

    /// A path built with the separator of the host platform.
    fn photos(name: &str) -> PathBuf {
        PathBuf::from("photos").join(name)
    }

    #[test]
    fn the_sidecar_keeps_the_whole_file_name() {
        assert_eq!(path_for(&photos("DSC001.cr2")), photos("DSC001.cr2.xmp"));
    }

    #[test]
    fn both_conventions_are_looked_for() {
        let paths = candidates(&photos("DSC001.cr2"));

        assert_eq!(paths, vec![photos("DSC001.cr2.xmp"), photos("DSC001.xmp")]);
    }

    #[test]
    fn a_sidecar_is_never_its_own_sidecar() {
        let paths = candidates(&photos("DSC001.xmp"));

        assert_eq!(paths, vec![photos("DSC001.xmp.xmp")]);
    }

    #[test]
    fn writing_then_reading_round_trips() {
        let dir = temp_dir("round-trip");
        let image = dir.join("photo.jpg");
        std::fs::write(&image, b"pretend this is a photograph").unwrap();

        write(&image, &annotations(4, &["Slovakia"])).unwrap();
        let back = read(&image).expect("reads back");

        assert_eq!(back.rating, 4);
        assert_eq!(back.keywords, vec!["Slovakia"]);
        assert!(dir.join("photo.jpg.xmp").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_image_without_a_sidecar_has_nothing_to_read() {
        let dir = temp_dir("missing");
        let image = dir.join("photo.jpg");

        assert!(read(&image).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_existing_adobe_sidecar_is_edited_rather_than_duplicated() {
        let dir = temp_dir("adobe");
        let image = dir.join("photo.cr2");
        let adobe = dir.join("photo.xmp");
        std::fs::write(
            &adobe,
            xmp::update(None, &annotations(1, &["Old"])).unwrap(),
        )
        .unwrap();

        write(&image, &annotations(5, &["New"])).unwrap();

        assert!(!dir.join("photo.cr2.xmp").exists());
        assert_eq!(read(&image).unwrap().rating, 5);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_more_specific_sidecar_wins() {
        let dir = temp_dir("both");
        let image = dir.join("photo.cr2");
        std::fs::write(
            dir.join("photo.xmp"),
            xmp::update(None, &annotations(1, &[])).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dir.join("photo.cr2.xmp"),
            xmp::update(None, &annotations(5, &[])).unwrap(),
        )
        .unwrap();

        assert_eq!(read(&image).unwrap().rating, 5);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_sidecar_that_cannot_be_rewritten_is_left_alone() {
        let dir = temp_dir("unreadable");
        let image = dir.join("photo.cr2");
        let sidecar = dir.join("photo.cr2.xmp");

        // XML the writer cannot make sense of, standing in for a document
        // holding somebody else's work.
        let original = b"<not-xmp>a develop history lives here</not-xmp>";
        std::fs::write(&sidecar, original).unwrap();

        assert!(write(&image, &annotations(5, &["New"])).is_err());
        assert_eq!(std::fs::read(&sidecar).unwrap(), original);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_blank_sidecar_is_filled_in() {
        let dir = temp_dir("blank");
        let image = dir.join("photo.jpg");
        std::fs::write(dir.join("photo.jpg.xmp"), b"   \n").unwrap();

        write(&image, &annotations(2, &[])).unwrap();

        assert_eq!(read(&image).unwrap().rating, 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn writing_leaves_no_temporary_behind() {
        let dir = temp_dir("temporary");
        let image = dir.join("photo.jpg");

        write(&image, &annotations(3, &["One"])).unwrap();

        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".tmp"))
            .collect();

        assert!(leftovers.is_empty(), "{leftovers:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
