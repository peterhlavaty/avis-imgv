//! Putting a folder in an order.
//!
//! The order is not cosmetic: a rename that numbers files takes its numbers
//! from it, so "sorted by capture time, ascending" is the difference between a
//! sequence that reads as the shoot happened and one that reads as the camera
//! happened to name things.

use std::cmp::Ordering;

use super::Entry;

/// What to order by.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SortKey {
    /// The file name, compared the way a person reads it: the numbers in it as
    /// numbers, so `IMG_9` comes before `IMG_10`.
    #[default]
    Name,
    /// The extension, then the name, so a folder groups by type.
    Extension,
    Size,
    /// When the shutter opened, with unscanned and undated files last.
    Captured,
    Rating,
    /// How sharp it looked, sharpest first when descending.
    ///
    /// A ranking rather than a measurement: it tells the frames of one scene
    /// apart and says nothing useful about a wall against a portrait. See
    /// [`crate::organize::sharpness`].
    Sharpness,
    /// Any metadata tag, by its exiftool name.
    Metadata(String),
}

impl SortKey {
    /// The keys offered in a dropdown, in the order they appear there.
    pub const CHOICES: &'static [SortKey] = &[
        SortKey::Name,
        SortKey::Captured,
        SortKey::Extension,
        SortKey::Size,
        SortKey::Rating,
        SortKey::Sharpness,
    ];

    pub fn label(&self) -> &str {
        match self {
            SortKey::Name => "Name",
            SortKey::Extension => "Type",
            SortKey::Size => "Size",
            SortKey::Captured => "Capture time",
            SortKey::Rating => "Rating",
            SortKey::Sharpness => "Sharpness",
            SortKey::Metadata(tag) => tag,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Ascending,
    Descending,
}

impl Direction {
    pub fn label(self) -> &'static str {
        match self {
            Direction::Ascending => "Ascending",
            Direction::Descending => "Descending",
        }
    }

    pub fn flipped(self) -> Direction {
        match self {
            Direction::Ascending => Direction::Descending,
            Direction::Descending => Direction::Ascending,
        }
    }
}

/// Sorts `entries` in place.
///
/// The file name always breaks a tie, so the result does not depend on what
/// order the files were read from the disk in — two runs of the same rename
/// have to number things the same way.
///
/// A file that has nothing to sort by, because it has no capture time or
/// because the scan has not reached it yet, goes last whichever way round the
/// order is. Reversing it along with everything else would scatter the files
/// still being read through the top of the list.
pub fn sort(entries: &mut [Entry], key: &SortKey, direction: Direction) {
    entries.sort_by(|a, b| {
        match (has_value(a, key), has_value(b, key)) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            (false, false) => return natural(a.name(), b.name()),
            (true, true) => {}
        }

        let ordering = match direction {
            Direction::Ascending => compare(a, b, key),
            Direction::Descending => compare(a, b, key).reverse(),
        };

        ordering.then_with(|| natural(a.name(), b.name()))
    });
}

/// Whether there is anything to compare for this entry.
fn has_value(entry: &Entry, key: &SortKey) -> bool {
    match key {
        SortKey::Captured => entry.captured().is_some(),
        // A file the scan has not reached, or one with no thumbnail to
        // measure, has no sharpness — and sorting it in among the blurred
        // ones would be a lie about a photograph nobody has looked at.
        SortKey::Sharpness => entry.sharpness.is_some(),
        SortKey::Metadata(tag) => entry.tag(tag).is_some(),
        // Every file has a name, a type, a size and a rating.
        _ => true,
    }
}

/// Orders two entries that both have a value for `key`.
fn compare(a: &Entry, b: &Entry, key: &SortKey) -> Ordering {
    match key {
        SortKey::Name => natural(a.name(), b.name()),
        SortKey::Extension => a
            .extension()
            .cmp(&b.extension())
            .then_with(|| natural(a.name(), b.name())),
        SortKey::Size => a.size.cmp(&b.size),
        SortKey::Rating => a.rating().cmp(&b.rating()),
        SortKey::Captured => a.captured().cmp(&b.captured()),
        SortKey::Sharpness => a
            .sharpness
            .partial_cmp(&b.sharpness)
            .unwrap_or(Ordering::Equal),
        SortKey::Metadata(tag) => match (a.tag(tag), b.tag(tag)) {
            (Some(a), Some(b)) => values(a, b),
            _ => Ordering::Equal,
        },
    }
}

/// Compares two metadata values as numbers when both are numbers.
///
/// `ISO` and `Focal Length` are written as text but mean quantities, and
/// comparing them as text puts 1000 before 200.
fn values(a: &str, b: &str) -> Ordering {
    match (leading_number(a), leading_number(b)) {
        (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
        _ => natural(a, b),
    }
}

/// The number a value starts with, ignoring whatever unit follows it.
fn leading_number(text: &str) -> Option<f64> {
    let text = text.trim();
    let digits: String = text
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();

    // Only a value that is a number, not one that merely starts with digits:
    // `2024:11:06 22:07:19` must not be read as the number 2024.
    let rest = text[digits.len()..].trim_start();
    let unit_only = rest.chars().all(|c| !c.is_ascii_digit() && c != ':');

    (unit_only && !digits.is_empty())
        .then(|| digits.parse().ok())
        .flatten()
}

/// Compares two strings the way a person reads them, with runs of digits taken
/// as numbers.
///
/// Case insensitive, because a folder holding `IMG_1.JPG` and `img_2.jpg` is
/// one sequence and not two.
pub fn natural(a: &str, b: &str) -> Ordering {
    let mut left = a.chars().peekable();
    let mut right = b.chars().peekable();

    loop {
        match (left.peek().copied(), right.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let x = take_number(&mut left);
                let y = take_number(&mut right);

                match x.cmp(&y) {
                    Ordering::Equal => {}
                    ordering => return ordering,
                }
            }
            (Some(x), Some(y)) => {
                left.next();
                right.next();

                let (x, y) = (x.to_ascii_lowercase(), y.to_ascii_lowercase());
                match x.cmp(&y) {
                    Ordering::Equal => {}
                    ordering => return ordering,
                }
            }
        }
    }
}

/// Consumes a run of digits, as the number it spells.
///
/// Saturating rather than wrapping: a file named after a hundred digit number
/// is not a sequence, and comparing the run as text is close enough for it.
fn take_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> u128 {
    let mut number: u128 = 0;

    while let Some(digit) = chars.peek().and_then(|c| c.to_digit(10)) {
        number = number.saturating_mul(10).saturating_add(u128::from(digit));
        chars.next();
    }

    number
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{entry, rated};
    use super::super::CAPTURE_TAG;
    use super::*;

    fn names(entries: &[Entry]) -> Vec<&str> {
        entries.iter().map(Entry::name).collect()
    }

    fn sorted(mut entries: Vec<Entry>, key: SortKey, direction: Direction) -> Vec<Entry> {
        sort(&mut entries, &key, direction);
        entries
    }

    #[test]
    fn names_sort_the_way_a_person_reads_them() {
        let entries = sorted(
            vec![
                entry("IMG_10.jpg", 0, &[]),
                entry("IMG_9.jpg", 0, &[]),
                entry("IMG_100.jpg", 0, &[]),
            ],
            SortKey::Name,
            Direction::Ascending,
        );

        assert_eq!(
            names(&entries),
            vec!["IMG_9.jpg", "IMG_10.jpg", "IMG_100.jpg"]
        );
    }

    #[test]
    fn a_mixture_of_cases_is_still_one_sequence() {
        let entries = sorted(
            vec![entry("img_2.jpg", 0, &[]), entry("IMG_1.JPG", 0, &[])],
            SortKey::Name,
            Direction::Ascending,
        );

        assert_eq!(names(&entries), vec!["IMG_1.JPG", "img_2.jpg"]);
    }

    #[test]
    fn descending_is_the_reverse() {
        let entries = sorted(
            vec![
                entry("b.jpg", 0, &[]),
                entry("a.jpg", 0, &[]),
                entry("c.jpg", 0, &[]),
            ],
            SortKey::Name,
            Direction::Descending,
        );

        assert_eq!(names(&entries), vec!["c.jpg", "b.jpg", "a.jpg"]);
    }

    #[test]
    fn capture_time_orders_a_shoot_as_it_happened() {
        let entries = sorted(
            vec![
                entry("late.jpg", 0, &[(CAPTURE_TAG, "2024:11:06 22:07:19")]),
                entry("early.jpg", 0, &[(CAPTURE_TAG, "2024:11:06 06:00:00")]),
                entry("undated.jpg", 0, &[]),
            ],
            SortKey::Captured,
            Direction::Ascending,
        );

        assert_eq!(
            names(&entries),
            vec!["early.jpg", "late.jpg", "undated.jpg"]
        );
    }

    #[test]
    fn a_file_with_no_date_stays_last_even_reversed() {
        let entries = sorted(
            vec![
                entry("dated.jpg", 0, &[(CAPTURE_TAG, "2024:11:06 22:07:19")]),
                entry("undated.jpg", 0, &[]),
            ],
            SortKey::Captured,
            Direction::Descending,
        );

        assert_eq!(names(&entries), vec!["dated.jpg", "undated.jpg"]);
    }

    #[test]
    fn size_orders_by_the_number_of_bytes() {
        let entries = sorted(
            vec![
                entry("big.jpg", 9_000_000, &[]),
                entry("small.jpg", 100, &[]),
            ],
            SortKey::Size,
            Direction::Ascending,
        );

        assert_eq!(names(&entries), vec!["small.jpg", "big.jpg"]);
    }

    #[test]
    fn type_groups_the_folder_and_names_break_the_tie() {
        let entries = sorted(
            vec![
                entry("b.jpg", 0, &[]),
                entry("a.cr3", 0, &[]),
                entry("a.jpg", 0, &[]),
            ],
            SortKey::Extension,
            Direction::Ascending,
        );

        assert_eq!(names(&entries), vec!["a.cr3", "a.jpg", "b.jpg"]);
    }

    #[test]
    fn rating_orders_by_stars() {
        let entries = sorted(
            vec![rated("a.jpg", 1, &[]), rated("b.jpg", 5, &[])],
            SortKey::Rating,
            Direction::Descending,
        );

        assert_eq!(names(&entries), vec!["b.jpg", "a.jpg"]);
    }

    #[test]
    fn a_numeric_metadata_tag_sorts_as_a_number() {
        let entries = sorted(
            vec![
                entry("a.jpg", 0, &[("ISO", "1000")]),
                entry("b.jpg", 0, &[("ISO", "200")]),
            ],
            SortKey::Metadata("ISO".into()),
            Direction::Ascending,
        );

        assert_eq!(names(&entries), vec!["b.jpg", "a.jpg"]);
    }

    #[test]
    fn a_metadata_tag_that_is_not_a_number_sorts_as_text() {
        let entries = sorted(
            vec![
                entry("a.jpg", 0, &[("Camera Model Name", "Z 6")]),
                entry("b.jpg", 0, &[("Camera Model Name", "EOS R5")]),
            ],
            SortKey::Metadata("Camera Model Name".into()),
            Direction::Ascending,
        );

        assert_eq!(names(&entries), vec!["b.jpg", "a.jpg"]);
    }

    #[test]
    fn a_date_is_not_mistaken_for_a_number() {
        // Read as a number, `2024:...` would be 2024 for every photograph of
        // the year and the order would collapse to the file names.
        assert_eq!(leading_number("2024:11:06 22:07:19"), None);
        assert_eq!(leading_number("1/500"), None);
        assert_eq!(leading_number("200"), Some(200.0));
        assert_eq!(leading_number("5.6"), Some(5.6));
        assert_eq!(leading_number("35 mm"), Some(35.0));
    }

    #[test]
    fn a_missing_metadata_tag_sorts_last() {
        let entries = sorted(
            vec![
                entry("without.jpg", 0, &[]),
                entry("with.jpg", 0, &[("ISO", "100")]),
            ],
            SortKey::Metadata("ISO".into()),
            Direction::Ascending,
        );

        assert_eq!(names(&entries), vec!["with.jpg", "without.jpg"]);
    }

    #[test]
    fn an_order_does_not_depend_on_how_the_folder_was_read() {
        let one = sorted(
            vec![
                entry("b.jpg", 5, &[]),
                entry("a.jpg", 5, &[]),
                entry("c.jpg", 5, &[]),
            ],
            SortKey::Size,
            Direction::Ascending,
        );
        let other = sorted(
            vec![
                entry("c.jpg", 5, &[]),
                entry("b.jpg", 5, &[]),
                entry("a.jpg", 5, &[]),
            ],
            SortKey::Size,
            Direction::Ascending,
        );

        assert_eq!(names(&one), names(&other));
        assert_eq!(names(&one), vec!["a.jpg", "b.jpg", "c.jpg"]);
    }

    #[test]
    fn a_hundred_digit_number_does_not_overflow() {
        let long = "9".repeat(100);
        assert_eq!(
            natural(&format!("{long}.jpg"), &format!("{long}.jpg")),
            Ordering::Equal
        );
    }
}
