//! Working on a folder rather than on one image.
//!
//! Renaming a shoot and correcting a camera clock are the same shape of job:
//! decide which files it applies to, decide what order they are in, work out
//! what each one becomes, look at that, and only then touch the disk. The
//! deciding and the working out are here, as plain functions over a list; the
//! touching is a separate step that can be refused.

pub mod files;
pub mod filter;
pub mod gather;
pub mod group;
pub mod journal;
pub mod pairs;
pub mod rename;
pub mod scan;
pub mod sharpness;
pub mod similarity;
pub mod sort;
pub mod timeshift;

use std::path::{Path, PathBuf};

use std::sync::Arc;

use image::RgbaImage;

use crate::metadata::dates::DateField;
use crate::metadata::datetime::Timestamp;
use crate::metadata::xmp::Xmp;
use crate::metadata::Metadata;
use similarity::Fingerprint;

pub use filter::Filter;
pub use scan::Scan;
pub use sort::{Direction, SortKey};

/// The metadata tag holding the moment the shutter opened, which is what
/// nearly every sort and every name template is really about.
pub const CAPTURE_TAG: &str = "Date/Time Original";

/// One file, with everything the organiser needs to decide about it.
#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub path: PathBuf,
    /// Size on disk. Zero until the scan reaches it.
    pub size: u64,
    /// What the front of the file says. Absent until the scan reaches it.
    pub metadata: Option<Metadata>,
    /// The rating and keywords, from the sidecar or from the file itself.
    pub annotations: Xmp,
    /// The timestamps the file carries, as the scan found them. Shown, and
    /// used to decide what can be shifted; the shift itself locates them again
    /// in the file it is about to write.
    pub dates: Vec<DateField>,
    /// A summary of what the picture looks like, from the camera's thumbnail.
    /// Absent for a file that embeds none.
    pub fingerprint: Option<Fingerprint>,
    /// The camera's thumbnail itself, for the panels that show it. Shared
    /// rather than copied: the entries are cloned into every filtered list.
    pub thumbnail: Option<Arc<RgbaImage>>,
    /// How sharp it looked, for ranking the frames of one scene against each
    /// other. Absent for a file with no thumbnail to measure.
    pub sharpness: Option<sharpness::Sharpness>,
}

impl Entry {
    pub fn new(path: PathBuf) -> Entry {
        Entry {
            path,
            ..Default::default()
        }
    }

    /// Which way up the camera was holding it.
    ///
    /// Upright when the scan has not reached the file yet, which is the only
    /// honest answer: guessing would turn a picture and then turn it back.
    pub fn orientation(&self) -> crate::metadata::Orientation {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.orientation)
            .unwrap_or_default()
    }

    /// The file name, extension and all.
    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    /// The file name without its extension, which is what a rename keeps.
    pub fn stem(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
    }

    /// The extension, lowercased, without the dot.
    pub fn extension(&self) -> String {
        crate::formats::extension_of(&self.path)
    }

    /// A metadata tag, if the file has been scanned and carries it.
    pub fn tag(&self, name: &str) -> Option<&str> {
        self.metadata.as_ref()?.tags.get(name).map(String::as_str)
    }

    /// When the photograph was taken.
    pub fn captured(&self) -> Option<Timestamp> {
        self.tag(CAPTURE_TAG).and_then(Timestamp::parse)
    }

    pub fn rating(&self) -> i8 {
        self.annotations.rating
    }

    /// Whether this file answers to a keyword somebody typed.
    ///
    /// The same predicate the browsing bar uses, over the hierarchy as well as
    /// the leaves: the two used to disagree, so the same word typed in the two
    /// places gave two different sets of files.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.annotations
            .keywords
            .iter()
            .chain(self.annotations.hierarchy.iter())
            .any(|keyword| crate::metadata::xmp::keyword_matches(keyword, tag))
    }

    /// Whether the scan has reached this file.
    pub fn is_scanned(&self) -> bool {
        self.metadata.is_some()
    }
}

/// Builds the entries for a collection, before anything is known about them.
pub fn entries(paths: &[PathBuf]) -> Vec<Entry> {
    paths.iter().cloned().map(Entry::new).collect()
}

/// Splits a comma separated list into trimmed, non-empty pieces.
///
/// The one parser for every "list of things" box in the organiser, so a
/// trailing comma or a stray space never changes what a filter means.
pub fn list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|piece| !piece.is_empty())
        .map(str::to_string)
        .collect()
}

/// Whether `path` is the same file as `other`, comparing the way the platform
/// does.
///
/// Windows file names differ only in case for display purposes, so a rename
/// from `IMG.JPG` to `img.jpg` is a rename and not a collision, while on Linux
/// the two are different files.
pub fn same_file(path: &Path, other: &Path) -> bool {
    if cfg!(windows) {
        path.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&other.as_os_str().to_string_lossy())
    } else {
        path == other
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use std::collections::BTreeMap;

    /// An entry as the scan would have left it.
    pub fn entry(name: &str, size: u64, tags: &[(&str, &str)]) -> Entry {
        let mut map = BTreeMap::new();
        for (key, value) in tags {
            map.insert(key.to_string(), value.to_string());
        }

        Entry {
            path: PathBuf::from("/photos").join(name),
            size,
            metadata: Some(Metadata {
                tags: map,
                ..Default::default()
            }),
            annotations: Xmp::default(),
            dates: Vec::new(),
            fingerprint: None,
            thumbnail: None,
            sharpness: None,
        }
    }

    /// The same, with a rating and keywords on it.
    pub fn rated(name: &str, rating: i8, keywords: &[&str]) -> Entry {
        let mut entry = entry(name, 0, &[]);
        entry.annotations = Xmp {
            rating,
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            ..Xmp::default()
        };

        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_comma_separated_box_forgives_spaces_and_trailing_commas() {
        assert_eq!(list(" keeper , portfolio ,"), vec!["keeper", "portfolio"]);
        assert_eq!(list(""), Vec::<String>::new());
        assert_eq!(list(" , , "), Vec::<String>::new());
    }

    #[test]
    fn an_entry_knows_its_parts() {
        let entry = Entry::new(PathBuf::from("/photos/DSCF0001.JPG"));

        assert_eq!(entry.name(), "DSCF0001.JPG");
        assert_eq!(entry.stem(), "DSCF0001");
        assert_eq!(entry.extension(), "jpg");
        assert!(!entry.is_scanned());
    }

    #[test]
    fn a_scanned_entry_answers_about_its_metadata() {
        let entry = test_support::entry("a.jpg", 100, &[(CAPTURE_TAG, "2024:11:06 22:07:19")]);

        assert!(entry.is_scanned());
        assert_eq!(entry.captured().unwrap().to_date(), "2024-11-06");
        assert_eq!(entry.tag("Nothing Here"), None);
    }

    #[test]
    fn keywords_match_whatever_case_they_were_written_in() {
        let entry = test_support::rated("a.jpg", 3, &["Keeper"]);

        assert!(entry.has_tag("keeper"));
        assert!(entry.has_tag("KEEPER"));
        assert!(!entry.has_tag("reject"));
        assert_eq!(entry.rating(), 3);
    }
}
