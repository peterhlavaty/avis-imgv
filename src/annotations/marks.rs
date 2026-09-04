//! What the user has said about a photograph, as the rest of the program
//! reads it.
//!
//! One per photograph in the open collection, built once when the collection
//! changes rather than asked per cell per frame: the contact sheet needs all of
//! it at once, and reading a folder's sidecars is a few milliseconds one time.
//!
//! It lived in the status bar, which drew it — and so `view::narrow`, which
//! filters on it, imported a struct out of a eleven-hundred-line drawing file,
//! and through that one edge `metadata`, `annotations`, `organize` and
//! `decoder` all depended transitively on the toolkit. Nothing here draws;
//! this is what a mark *is*, and the bar is one of the things that shows it.

use crate::metadata::xmp::{leaf_of, Flag, Label, Xmp};

/// What the user has said about the photograph on screen.
///
/// Drawn in the bar so that rating, flagging or labelling with the panel shut
/// is not a keystroke that appears to do nothing.
#[derive(Debug, Clone, Default)]
pub struct Marks {
    pub stars: u8,
    pub flag: Flag,
    pub label: Option<Label>,
    /// Kept here as well as in the annotation store, because the filter asks
    /// about every photograph in the folder at once and a lookup per file per
    /// keystroke is the thing this list exists to avoid.
    ///
    /// With their levels where the sidecar records them, so narrowing by
    /// `Slovakia` finds everything filed underneath it and not only what is
    /// tagged with the word itself.
    pub keywords: Vec<String>,
}

impl Marks {
    pub fn of(annotations: &Xmp) -> Marks {
        Marks {
            stars: annotations.stars(),
            flag: annotations.flag(),
            label: annotations.known_label(),
            keywords: annotations
                .keywords
                .iter()
                .map(|keyword| {
                    annotations
                        .hierarchy
                        .iter()
                        .find(|path| leaf_of(path) == keyword)
                        .unwrap_or(keyword)
                        .clone()
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mark_carries_what_the_sidecar_says() {
        let annotations = Xmp {
            rating: 3,
            ..Xmp::default()
        };

        let marks = Marks::of(&annotations);

        assert_eq!(marks.stars, 3);
        assert!(marks.keywords.is_empty());
    }

    /// The keyword is kept with its levels, so narrowing by `Slovakia` finds
    /// everything filed underneath it and not only what carries the word.
    #[test]
    fn a_keyword_is_kept_at_the_depth_the_sidecar_files_it() {
        let annotations = Xmp {
            keywords: vec!["Tatras".to_string()],
            hierarchy: vec!["Europe|Slovakia|Tatras".to_string()],
            ..Xmp::default()
        };

        let marks = Marks::of(&annotations);

        assert_eq!(marks.keywords, vec!["Europe|Slovakia|Tatras".to_string()]);
    }

    /// A keyword the hierarchy says nothing about is kept as it stands rather
    /// than dropped.
    #[test]
    fn a_keyword_with_no_hierarchy_is_kept_as_it_is() {
        let annotations = Xmp {
            keywords: vec!["Untidy".to_string()],
            ..Xmp::default()
        };

        let marks = Marks::of(&annotations);

        assert_eq!(marks.keywords, vec!["Untidy".to_string()]);
    }
}
