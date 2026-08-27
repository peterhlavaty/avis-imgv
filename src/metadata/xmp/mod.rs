//! XMP: the star ratings and keywords photographers actually edit.
//!
//! XMP travels either inside the image (a JPEG `APP1` segment, a PNG `iTXt`
//! chunk, a TIFF tag) or beside it in a sidecar file. Both are the same RDF
//! document, so one reader serves both.
//!
//! Only the two properties the viewer edits are understood. Writing preserves
//! everything else in an existing document, because a sidecar is frequently
//! shared with a raw converter that keeps its whole develop history in there.

pub mod read;
pub mod write;

use quick_xml::name::ResolveResult;

pub use read::read;
pub use write::{update, MARKER};

/// The namespaces we resolve against. Prefixes are conventional but not
/// guaranteed, so everything is matched by namespace URI.
pub const NS_XMP: &str = "http://ns.adobe.com/xap/1.0/";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// The namespaces this reader cares about, reduced to a value that does not
/// borrow the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Xmp,
    Dc,
    Rdf,
    Other,
}

/// Classifies a resolved prefix.
pub fn namespace_of(resolved: &ResolveResult<'_>) -> Namespace {
    let ResolveResult::Bound(bound) = resolved else {
        return Namespace::Other;
    };

    match bound.as_ref() {
        NS_XMP => Namespace::Xmp,
        NS_DC => Namespace::Dc,
        NS_RDF => Namespace::Rdf,
        _ => Namespace::Other,
    }
}

/// Highest star rating XMP defines. `-1` means rejected, which the viewer
/// reads as unrated rather than offering a control for it.
pub const MAX_RATING: u8 = 5;

/// What the viewer reads out of, and writes back into, an XMP document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Xmp {
    /// Stars, 0 to [`MAX_RATING`].
    pub rating: u8,
    /// Keywords, in the order the document lists them.
    pub keywords: Vec<String>,
}

impl Xmp {
    pub fn is_empty(&self) -> bool {
        self.rating == 0 && self.keywords.is_empty()
    }
}

/// Parses a rating, clamping anything out of range.
///
/// `-1` is XMP's "rejected" flag; the viewer has no control for it, so it
/// reads as unrated.
pub fn parse_rating(text: &str) -> Option<u8> {
    let rating: f32 = text.trim().parse().ok()?;

    Some(rating.round().clamp(0.0, MAX_RATING as f32) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratings_are_clamped_to_the_defined_range() {
        assert_eq!(parse_rating("0"), Some(0));
        assert_eq!(parse_rating("5"), Some(5));
        assert_eq!(parse_rating(" 3 "), Some(3));
        assert_eq!(parse_rating("9"), Some(5));
    }

    #[test]
    fn a_rejection_reads_as_unrated() {
        assert_eq!(parse_rating("-1"), Some(0));
    }

    #[test]
    fn some_writers_use_decimals() {
        assert_eq!(parse_rating("4.0"), Some(4));
        assert_eq!(parse_rating("2.5"), Some(3));
    }

    #[test]
    fn nonsense_is_not_a_rating() {
        assert_eq!(parse_rating(""), None);
        assert_eq!(parse_rating("five"), None);
    }

    #[test]
    fn classifies_the_namespaces_it_knows() {
        use quick_xml::name::Namespace as Ns;

        let bound = |uri| namespace_of(&ResolveResult::Bound(Ns(uri)));

        assert_eq!(bound(NS_XMP), Namespace::Xmp);
        assert_eq!(bound(NS_DC), Namespace::Dc);
        assert_eq!(bound(NS_RDF), Namespace::Rdf);
        assert_eq!(bound("http://example.com/"), Namespace::Other);
        assert_eq!(namespace_of(&ResolveResult::Unbound), Namespace::Other);
    }

    #[test]
    fn emptiness_is_having_nothing_to_say() {
        assert!(Xmp::default().is_empty());
        assert!(!Xmp {
            rating: 1,
            keywords: vec![]
        }
        .is_empty());
        assert!(!Xmp {
            rating: 0,
            keywords: vec!["a".into()]
        }
        .is_empty());
    }
}
