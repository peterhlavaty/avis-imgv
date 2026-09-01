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

/// digiKam's own namespace, which is where the one mark nobody standardised —
/// picked, as opposed to rejected — is written.
pub const NS_DIGIKAM: &str = "http://www.digikam.org/ns/1.0/";
/// Lightroom's namespace, which is where `hierarchicalSubject` lives — and
/// where darktable, digiKam, Bridge and exiftool all look for it.
pub const NS_LIGHTROOM: &str = "http://ns.adobe.com/lightroom/1.0/";

/// What separates the levels of a hierarchical keyword.
///
/// A vertical bar, which is what every program that writes these uses. Not
/// configurable: the whole value of the field is that other programs read it.
pub const HIERARCHY_SEPARATOR: char = '|';

/// The namespaces this reader cares about, reduced to a value that does not
/// borrow the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Xmp,
    Dc,
    Rdf,
    DigiKam,
    /// Lightroom's, which is where hierarchical keywords live.
    Lightroom,
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
        NS_DIGIKAM => Namespace::DigiKam,
        NS_LIGHTROOM => Namespace::Lightroom,
        _ => Namespace::Other,
    }
}

/// Highest star rating XMP defines.
pub const MAX_RATING: i8 = 5;

/// The rating XMP reserves for "I have looked at this, and the answer is no".
///
/// Not a separate field: Adobe, Bridge, Lightroom, FastRawViewer and darktable
/// all put a rejection here, so a rejected frame has no stars and a rated one
/// is not rejected.
pub const REJECTED: i8 = -1;

/// digiKam's value for a frame the photographer kept.
const PICKED: i32 = 3;

/// Where a frame stands in a first pass: kept, thrown out, or not looked at.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Flag {
    #[default]
    Unflagged,
    Picked,
    Rejected,
}

impl Flag {
    /// A single character for the status bar and the contact sheet.
    pub fn glyph(self) -> &'static str {
        match self {
            Flag::Unflagged => "",
            Flag::Picked => "⚑",
            Flag::Rejected => "✖",
        }
    }
}

/// The colour labels every other program writes, in their conventional order.
///
/// The value on disk is a free string, and Adobe writes the name in whatever
/// language the interface is in — which is why Capture One has a standing bug
/// about not reading Lightroom's labels. What the viewer writes is always the
/// English name; what it reads is matched against the names other programs are
/// known to use, and anything unrecognised is kept as it is rather than thrown
/// away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
}

impl Label {
    pub const CHOICES: &'static [Label] = &[
        Label::Red,
        Label::Yellow,
        Label::Green,
        Label::Blue,
        Label::Purple,
    ];

    /// The name written to disk.
    pub fn name(self) -> &'static str {
        match self {
            Label::Red => "Red",
            Label::Yellow => "Yellow",
            Label::Green => "Green",
            Label::Blue => "Blue",
            Label::Purple => "Purple",
        }
    }

    /// The colour drawn for it, chosen to stay legible on a dark background.
    pub fn colour(self) -> (u8, u8, u8) {
        match self {
            Label::Red => (219, 74, 74),
            Label::Yellow => (214, 184, 62),
            Label::Green => (92, 179, 92),
            Label::Blue => (72, 132, 214),
            Label::Purple => (156, 96, 200),
        }
    }

    /// Recognises a label written by this viewer or by another program.
    ///
    /// Bridge ships a second set of names for the same five colours, and this
    /// viewer adds three workflow words of its own; all of them are matched, so
    /// a frame labelled elsewhere shows its colour here. The first match wins,
    /// so no name may stand against two colours.
    pub fn of(text: &str) -> Option<Label> {
        let text = text.trim();

        Label::CHOICES.iter().copied().find(|label| {
            label.name().eq_ignore_ascii_case(text)
                || label
                    .aliases()
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(text))
        })
    }

    /// What Bridge calls the same colour, and the workflow words beside them.
    ///
    /// "To Do" is Bridge's purple. It was listed against red as well, which
    /// made purple's entry unreachable — `of` returns the first match — so a
    /// frame labelled "To Do" by Bridge drew red here. Red keeps the name
    /// Bridge actually gives it, "Select". The three workflow words are this
    /// viewer's own; no source attributes them to Lightroom, whose default
    /// label set is the five colour names themselves.
    fn aliases(self) -> &'static [&'static str] {
        match self {
            Label::Red => &["Select"],
            Label::Yellow => &["Second", "In Progress"],
            Label::Green => &["Approved", "Done"],
            Label::Blue => &["Review"],
            Label::Purple => &["To Do", "On Hold"],
        }
    }
}

/// Whether one keyword answers to what somebody typed.
///
/// One predicate, so "rename everything I tagged Slovakia" means the same thing
/// in the browsing bar and in the folder jobs. The bar took a case-insensitive
/// substring of the whole hierarchical keyword and the organiser took equality
/// on the stored one, so typing the word twice gave two different answers.
///
/// A substring, and over the whole path: `Places|Slovakia|Tatras` answers to
/// `slovakia`, because a keyword filed under levels is still that keyword.
pub fn keyword_matches(keyword: &str, wanted: &str) -> bool {
    let wanted = wanted.trim();
    if wanted.is_empty() {
        return true;
    }

    keyword.to_lowercase().contains(&wanted.to_lowercase())
}

/// What the viewer reads out of, and writes back into, an XMP document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Xmp {
    /// Stars, 0 to [`MAX_RATING`], or [`REJECTED`].
    pub rating: i8,
    /// Whether the frame was picked. Rejection is not here: it lives in the
    /// rating, which is where every other program looks for it.
    pub picked: bool,
    /// The colour label, kept as the string the document carries so one the
    /// viewer does not offer survives a round trip.
    pub label: Option<String>,
    /// Keywords, in the order the document lists them.
    ///
    /// The leaves, which is what `dc:subject` holds and what every program
    /// that does not know about hierarchies reads.
    pub keywords: Vec<String>,
    /// The same keywords with their paths, as `Places|Slovakia|Tatras`.
    ///
    /// Kept beside the flat list rather than instead of it, because that is
    /// what Lightroom, darktable and digiKam all do: a program that
    /// understands hierarchies reads this one, and a program that does not
    /// still finds the keyword in `dc:subject`. Writing only the paths would
    /// leave the second kind seeing nothing.
    pub hierarchy: Vec<String>,
}

/// The last part of a hierarchical keyword, which is the keyword itself.
///
/// `Places|Slovakia|Tatras` is the keyword `Tatras`, filed under two levels.
pub fn leaf_of(path: &str) -> &str {
    path.rsplit(HIERARCHY_SEPARATOR)
        .next()
        .unwrap_or(path)
        .trim()
}

/// The levels of a hierarchical keyword, outermost first.
pub fn levels_of(path: &str) -> Vec<&str> {
    path.split(HIERARCHY_SEPARATOR)
        .map(str::trim)
        .filter(|level| !level.is_empty())
        .collect()
}

impl Xmp {
    pub fn is_empty(&self) -> bool {
        self.rating == 0 && !self.picked && self.label.is_none() && self.keywords.is_empty()
    }

    /// Stars only: a rejected frame has none.
    pub fn stars(&self) -> u8 {
        self.rating.max(0) as u8
    }

    pub fn flag(&self) -> Flag {
        if self.rating == REJECTED {
            Flag::Rejected
        } else if self.picked {
            Flag::Picked
        } else {
            Flag::Unflagged
        }
    }

    /// Sets the flag, keeping the two axes consistent.
    ///
    /// Rejecting clears the stars because the rating is where the rejection
    /// goes, and picking a rejected frame un-rejects it. Returns whether
    /// anything moved.
    pub fn set_flag(&mut self, flag: Flag) -> bool {
        let before = (self.rating, self.picked);

        match flag {
            Flag::Rejected => {
                self.rating = REJECTED;
                self.picked = false;
            }
            Flag::Picked => {
                if self.rating == REJECTED {
                    self.rating = 0;
                }
                self.picked = true;
            }
            Flag::Unflagged => {
                if self.rating == REJECTED {
                    self.rating = 0;
                }
                self.picked = false;
            }
        }

        before != (self.rating, self.picked)
    }

    /// Sets the stars, which is also how a rejection is undone.
    pub fn set_rating(&mut self, stars: u8) -> bool {
        let rating = (stars as i8).clamp(0, MAX_RATING);
        let changed = self.rating != rating;
        self.rating = rating;

        changed
    }

    /// The label, when it is one the viewer knows how to draw.
    pub fn known_label(&self) -> Option<Label> {
        self.label.as_deref().and_then(Label::of)
    }
}

/// Parses a rating, clamping anything out of range.
pub fn parse_rating(text: &str) -> Option<i8> {
    let rating: f32 = text.trim().parse().ok()?;

    Some(rating.round().clamp(REJECTED as f32, MAX_RATING as f32) as i8)
}

/// Parses digiKam's pick label, which is 0 none, 1 rejected, 2 pending, 3
/// accepted.
pub fn parse_pick(text: &str) -> Option<bool> {
    let pick: i32 = text.trim().parse().ok()?;

    Some(pick == PICKED)
}

#[cfg(test)]
mod hierarchy_tests {
    use super::*;

    #[test]
    fn the_keyword_is_the_last_level() {
        assert_eq!(leaf_of("Places|Slovakia|Tatras"), "Tatras");
        assert_eq!(leaf_of("Tatras"), "Tatras");
        assert_eq!(leaf_of(""), "");
    }

    /// Written by hand in a text file, so the spacing round the bars is
    /// whatever somebody typed.
    #[test]
    fn the_levels_are_trimmed_and_the_empty_ones_dropped() {
        assert_eq!(
            levels_of(" Places | Slovakia | Tatras "),
            vec!["Places", "Slovakia", "Tatras"]
        );
        assert_eq!(levels_of("Places||Tatras"), vec!["Places", "Tatras"]);
        assert_eq!(levels_of("|"), Vec::<&str>::new());
    }

    #[test]
    fn a_flat_keyword_is_one_level() {
        assert_eq!(levels_of("Autumn"), vec!["Autumn"]);
        assert_eq!(leaf_of("Autumn"), "Autumn");
    }
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
        assert_eq!(parse_rating("-4"), Some(REJECTED));
    }

    #[test]
    fn a_rejection_is_kept_as_one() {
        assert_eq!(parse_rating("-1"), Some(REJECTED));

        let rejected = Xmp {
            rating: REJECTED,
            ..Xmp::default()
        };

        assert_eq!(rejected.flag(), Flag::Rejected);
        assert_eq!(rejected.stars(), 0);
        assert!(!rejected.is_empty());
    }

    #[test]
    fn rejecting_and_rating_are_the_same_field() {
        let mut xmp = Xmp {
            rating: 4,
            ..Xmp::default()
        };

        assert!(xmp.set_flag(Flag::Rejected));
        assert_eq!(xmp.rating, REJECTED);
        assert_eq!(xmp.stars(), 0);

        assert!(xmp.set_rating(2));
        assert_eq!(xmp.flag(), Flag::Unflagged);
        assert_eq!(xmp.stars(), 2);
    }

    #[test]
    fn picking_a_rejected_frame_un_rejects_it() {
        let mut xmp = Xmp {
            rating: REJECTED,
            ..Xmp::default()
        };

        assert!(xmp.set_flag(Flag::Picked));
        assert_eq!(xmp.flag(), Flag::Picked);
        assert_eq!(xmp.rating, 0);
    }

    #[test]
    fn setting_the_flag_it_already_has_changes_nothing() {
        let mut xmp = Xmp::default();

        assert!(!xmp.set_flag(Flag::Unflagged));
        assert!(xmp.set_flag(Flag::Picked));
        assert!(!xmp.set_flag(Flag::Picked));
    }

    #[test]
    fn labels_are_recognised_whoever_wrote_them() {
        assert_eq!(Label::of("Red"), Some(Label::Red));
        assert_eq!(Label::of(" green "), Some(Label::Green));
        assert_eq!(Label::of("Approved"), Some(Label::Green));
        assert_eq!(Label::of("Review"), Some(Label::Blue));
        assert_eq!(Label::of("Chartreuse"), None);
    }

    #[test]
    fn an_unknown_label_is_kept_rather_than_thrown_away() {
        let xmp = Xmp {
            label: Some("Chartreuse".to_string()),
            ..Xmp::default()
        };

        assert!(!xmp.is_empty());
        assert_eq!(xmp.known_label(), None);
        assert_eq!(xmp.label.as_deref(), Some("Chartreuse"));
    }

    #[test]
    fn digikam_picks_are_read() {
        assert_eq!(parse_pick("3"), Some(true));
        assert_eq!(parse_pick("0"), Some(false));
        assert_eq!(parse_pick("1"), Some(false));
        assert_eq!(parse_pick("what"), None);
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
            ..Xmp::default()
        }
        .is_empty());
        assert!(!Xmp {
            keywords: vec!["a".into()],
            ..Xmp::default()
        }
        .is_empty());
        assert!(!Xmp {
            picked: true,
            ..Xmp::default()
        }
        .is_empty());
    }

    /// The alias table had "To Do" against two colours and `of` returns the
    /// first match, so purple was unreachable.
    /// The two surfaces that filter by keyword agree now. They did not: the
    /// bar took a substring of the whole path and the organiser took equality
    /// on the leaf, so the same word typed twice gave two different answers.
    #[test]
    fn a_keyword_answers_to_a_word_inside_it() {
        assert!(keyword_matches("Places|Slovakia|Tatras", "slovakia"));
        assert!(keyword_matches("Slovakia", "SLOVAKIA"));
        assert!(keyword_matches("Tatras", "tat"));
        assert!(!keyword_matches("Austria", "slovakia"));
    }

    /// An empty query narrows nothing, which is what makes it safe to call
    /// unconditionally.
    #[test]
    fn an_empty_word_matches_everything() {
        assert!(keyword_matches("anything", ""));
        assert!(keyword_matches("anything", "   "));
    }

    #[test]
    fn no_label_name_stands_against_two_colours() {
        let mut seen: Vec<String> = Vec::new();

        for label in Label::CHOICES {
            for name in std::iter::once(label.name()).chain(label.aliases().iter().copied()) {
                let name = name.to_ascii_lowercase();
                assert!(
                    !seen.contains(&name),
                    "{name} is listed against two colours"
                );
                seen.push(name);
            }
        }
    }

    #[test]
    fn a_bridge_to_do_label_is_purple() {
        assert_eq!(Label::of("To Do"), Some(Label::Purple));
        assert_eq!(Label::of("Select"), Some(Label::Red));
    }
}
