//! Central knowledge about the file formats the viewer understands.
//!
//! Everything that needs to reason about "is this a raw file", "can we open
//! this", ... goes through here so the extension tables live in exactly one
//! place.

/// Container of a file, derived from its extension.
///
/// The decoder uses this to pick a code path without re-inspecting the
/// extension string in several places.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Jpeg,
    Png,
    Webp,
    Gif,
    Bmp,
    Tiff,
    JpegXl,
    /// Camera raw. Most are TIFF derivatives, a few are not; the decoder
    /// extracts the embedded preview either way.
    Raw,
}

impl Format {
    /// Whether a file of this format can carry pixels that are not opaque.
    ///
    /// Resampling has to be done on premultiplied values wherever the alpha
    /// varies, and finding out whether it varies costs a pass over every pixel
    /// — measured at five per cent of the viewer's throughput on a folder of
    /// photographs, every one of which is opaque. A JPEG and a raw file cannot
    /// be anything else, so for the formats people actually photograph in the
    /// question is answered here for nothing.
    pub fn may_have_alpha(self) -> bool {
        match self {
            Format::Jpeg | Format::Raw => false,
            Format::Png | Format::Webp | Format::Gif | Format::Bmp | Format::Tiff => true,
            // Supports alpha, and is rare enough that the pass costs nothing
            // anybody will notice.
            Format::JpegXl => true,
        }
    }
}

/// Extensions of camera raw formats. TIFF is deliberately absent: it is a
/// first class image format that merely doubles as a raw container.
pub const RAW_EXTENSIONS: &[&str] = &[
    "3fr", "ari", "arq", "arw", "cam", "cr2", "cr3", "crw", "dcr", "dng", "erf", "fff", "gpr",
    "iiq", "kdc", "lri", "mdc", "mef", "mos", "mrw", "nef", "nrw", "orf", "ori", "pef", "raf",
    "raw", "rw2", "rwl", "sr2", "srf", "srw", "sti", "x3f",
];

const JPEG_EXTENSIONS: &[&str] = &["jpg", "jpeg", "jpe", "jfif"];
const TIFF_EXTENSIONS: &[&str] = &["tif", "tiff"];

impl Format {
    /// Classifies a lowercase extension. Returns `None` for anything we cannot
    /// open.
    pub fn from_extension(ext: &str) -> Option<Format> {
        if JPEG_EXTENSIONS.contains(&ext) {
            return Some(Format::Jpeg);
        }
        if TIFF_EXTENSIONS.contains(&ext) {
            return Some(Format::Tiff);
        }
        if RAW_EXTENSIONS.contains(&ext) {
            return Some(Format::Raw);
        }

        match ext {
            "png" => Some(Format::Png),
            "webp" => Some(Format::Webp),
            "gif" => Some(Format::Gif),
            "bmp" => Some(Format::Bmp),
            "jxl" => Some(Format::JpegXl),
            _ => None,
        }
    }

    /// Classifies a path by its extension.
    pub fn from_path(path: &std::path::Path) -> Option<Format> {
        Format::from_extension(&extension_of(path))
    }

    /// Whether the pixels of this format already come out in their final
    /// orientation, making the EXIF orientation tag redundant.
    pub fn ignores_exif_orientation(self) -> bool {
        matches!(self, Format::JpegXl)
    }
}

/// Lowercased extension of `path`, or an empty string when it has none.
pub fn extension_of(path: &std::path::Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_lowercase()
}

/// Whether the viewer can open this path, judging by its extension only.
pub fn is_supported(path: &std::path::Path) -> bool {
    Format::from_path(path).is_some()
}

/// Every extension we accept, for file dialog filters.
pub fn supported_extensions() -> Vec<&'static str> {
    let mut exts: Vec<&'static str> = JPEG_EXTENSIONS
        .iter()
        .chain(TIFF_EXTENSIONS)
        .chain(RAW_EXTENSIONS)
        .chain(["png", "webp", "gif", "bmp", "jxl"].iter())
        .copied()
        .collect();
    exts.sort_unstable();
    exts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn classifies_by_extension() {
        assert_eq!(Format::from_extension("jpg"), Some(Format::Jpeg));
        assert_eq!(Format::from_extension("jpeg"), Some(Format::Jpeg));
        assert_eq!(Format::from_extension("tif"), Some(Format::Tiff));
        assert_eq!(Format::from_extension("nef"), Some(Format::Raw));
        assert_eq!(Format::from_extension("jxl"), Some(Format::JpegXl));
        assert_eq!(Format::from_extension("txt"), None);
    }

    #[test]
    fn extension_is_lowercased() {
        assert_eq!(extension_of(Path::new("/a/B.JPG")), "jpg");
        assert_eq!(extension_of(Path::new("/a/noext")), "");
        assert!(is_supported(Path::new("/a/B.CR2")));
        assert!(!is_supported(Path::new("/a/notes.md")));
    }

    #[test]
    fn tiff_is_not_treated_as_raw() {
        assert!(!RAW_EXTENSIONS.contains(&"tif"));
        assert_eq!(Format::from_extension("tif"), Some(Format::Tiff));
    }

    #[test]
    fn supported_extensions_are_unique_and_sorted() {
        let exts = supported_extensions();
        let mut deduped = exts.clone();
        deduped.dedup();
        assert_eq!(exts, deduped);
        assert!(exts.windows(2).all(|w| w[0] <= w[1]));
    }
}
