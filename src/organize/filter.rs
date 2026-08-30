//! Narrowing a folder down to the files a job applies to.
//!
//! Every rule is empty by default and an empty rule matches everything, so a
//! filter nobody has touched is the whole folder. Rules combine with "and":
//! turning one on can only ever remove files, which is what makes it safe to
//! apply a rename to what is left.

use super::{list, Entry};

/// What a file has to be for the job to apply to it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// Kept when the file name contains this, ignoring case.
    pub name_contains: String,
    /// Comma separated extensions, without dots. Empty means any.
    pub extensions: String,
    /// Bounds on the size in bytes.
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    /// A metadata tag that has to be present, and optionally what it has to
    /// contain.
    pub metadata_tag: String,
    pub metadata_contains: String,
    /// Stars, as a closed range.
    pub min_rating: i8,
    pub max_rating: i8,
    /// Comma separated keywords the file has to carry at least one of.
    pub with_any_tag: String,
    /// Comma separated keywords that exclude a file.
    pub without_tags: String,
}

/// The highest rating there is, which is what "no upper bound" means.
const MAX_RATING: i8 = crate::metadata::xmp::MAX_RATING;

impl Filter {
    pub fn new() -> Filter {
        Filter {
            max_rating: MAX_RATING,
            ..Default::default()
        }
    }

    /// Whether nothing has been narrowed down.
    pub fn is_empty(&self) -> bool {
        *self == Filter::new()
    }

    /// Whether `entry` passes every rule.
    ///
    /// A file the scan has not reached yet passes the rules that depend on
    /// metadata: it is not known to fail them, and dropping files out of the
    /// list as the scan runs past them would be worse than including them.
    pub fn matches(&self, entry: &Entry) -> bool {
        self.matches_name(entry)
            && self.matches_extension(entry)
            && self.matches_size(entry)
            && self.matches_metadata(entry)
            && self.matches_rating(entry)
            && self.matches_tags(entry)
    }

    /// Keeps only the entries that pass.
    pub fn apply(&self, entries: &mut Vec<Entry>) {
        entries.retain(|entry| self.matches(entry));
    }

    fn matches_name(&self, entry: &Entry) -> bool {
        let wanted = self.name_contains.trim();

        wanted.is_empty() || contains_ignoring_case(entry.name(), wanted)
    }

    fn matches_extension(&self, entry: &Entry) -> bool {
        let wanted = list(&self.extensions);
        if wanted.is_empty() {
            return true;
        }

        let extension = entry.extension();
        wanted.iter().any(|allowed| {
            allowed
                .trim_start_matches('.')
                .eq_ignore_ascii_case(&extension)
        })
    }

    fn matches_size(&self, entry: &Entry) -> bool {
        // Size comes from the directory listing rather than the scan, so it is
        // known for every file from the start.
        self.min_size.is_none_or(|min| entry.size >= min)
            && self.max_size.is_none_or(|max| entry.size <= max)
    }

    fn matches_metadata(&self, entry: &Entry) -> bool {
        let tag = self.metadata_tag.trim();
        if tag.is_empty() {
            return true;
        }

        if !entry.is_scanned() {
            return true;
        }

        let Some(value) = entry.tag(tag) else {
            return false;
        };

        let wanted = self.metadata_contains.trim();
        wanted.is_empty() || contains_ignoring_case(value, wanted)
    }

    fn matches_rating(&self, entry: &Entry) -> bool {
        let rating = entry.rating();

        rating >= self.min_rating && rating <= self.max_rating.max(self.min_rating)
    }

    fn matches_tags(&self, entry: &Entry) -> bool {
        let any = list(&self.with_any_tag);
        let none = list(&self.without_tags);

        let wanted = any.is_empty() || any.iter().any(|tag| entry.has_tag(tag));
        let unwanted = none.iter().any(|tag| entry.has_tag(tag));

        wanted && !unwanted
    }
}

fn contains_ignoring_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{entry, rated};
    use super::super::CAPTURE_TAG;
    use super::*;

    #[test]
    fn a_filter_nobody_touched_is_the_whole_folder() {
        let filter = Filter::new();

        assert!(filter.is_empty());
        assert!(filter.matches(&entry("anything.jpg", 0, &[])));
        assert!(filter.matches(&Entry::new("/photos/unscanned.jpg".into())));
    }

    #[test]
    fn a_name_is_matched_whatever_case_it_was_typed_in() {
        let filter = Filter {
            name_contains: "IMG".into(),
            ..Filter::new()
        };

        assert!(filter.matches(&entry("img_0001.jpg", 0, &[])));
        assert!(filter.matches(&entry("MY_IMG.jpg", 0, &[])));
        assert!(!filter.matches(&entry("DSCF0001.jpg", 0, &[])));
    }

    #[test]
    fn extensions_are_a_list_and_the_dot_is_optional() {
        let filter = Filter {
            extensions: "jpg, .CR3".into(),
            ..Filter::new()
        };

        assert!(filter.matches(&entry("a.JPG", 0, &[])));
        assert!(filter.matches(&entry("a.cr3", 0, &[])));
        assert!(!filter.matches(&entry("a.png", 0, &[])));
    }

    #[test]
    fn size_bounds_are_inclusive_and_independent() {
        let over = Filter {
            min_size: Some(1000),
            ..Filter::new()
        };
        assert!(over.matches(&entry("a.jpg", 1000, &[])));
        assert!(!over.matches(&entry("a.jpg", 999, &[])));

        let under = Filter {
            max_size: Some(1000),
            ..Filter::new()
        };
        assert!(under.matches(&entry("a.jpg", 1000, &[])));
        assert!(!under.matches(&entry("a.jpg", 1001, &[])));
    }

    #[test]
    fn a_metadata_rule_can_ask_only_that_the_tag_is_there() {
        let filter = Filter {
            metadata_tag: CAPTURE_TAG.into(),
            ..Filter::new()
        };

        assert!(filter.matches(&entry("a.jpg", 0, &[(CAPTURE_TAG, "2024:11:06 22:07:19")])));
        assert!(!filter.matches(&entry("b.jpg", 0, &[])));
    }

    #[test]
    fn a_metadata_rule_can_ask_what_it_says() {
        let filter = Filter {
            metadata_tag: "Camera Model Name".into(),
            metadata_contains: "eos".into(),
            ..Filter::new()
        };

        assert!(filter.matches(&entry("a.jpg", 0, &[("Camera Model Name", "Canon EOS R5")])));
        assert!(!filter.matches(&entry("b.jpg", 0, &[("Camera Model Name", "NIKON Z 6")])));
    }

    #[test]
    fn an_unscanned_file_is_not_dropped_by_a_metadata_rule() {
        let filter = Filter {
            metadata_tag: "ISO".into(),
            ..Filter::new()
        };

        // It might well carry the tag; the scan has simply not reached it.
        assert!(filter.matches(&Entry::new("/photos/unscanned.jpg".into())));
    }

    #[test]
    fn ratings_are_a_closed_range() {
        let filter = Filter {
            min_rating: 3,
            max_rating: 4,
            ..Filter::new()
        };

        assert!(!filter.matches(&rated("a.jpg", 2, &[])));
        assert!(filter.matches(&rated("b.jpg", 3, &[])));
        assert!(filter.matches(&rated("c.jpg", 4, &[])));
        assert!(!filter.matches(&rated("d.jpg", 5, &[])));
    }

    #[test]
    fn an_upper_bound_below_the_lower_one_is_not_an_empty_folder() {
        // The two are separate boxes, and dragging one past the other should
        // not silently exclude everything.
        let filter = Filter {
            min_rating: 4,
            max_rating: 1,
            ..Filter::new()
        };

        assert!(filter.matches(&rated("a.jpg", 4, &[])));
        assert!(!filter.matches(&rated("b.jpg", 2, &[])));
    }

    #[test]
    fn keywords_can_be_required_and_excluded() {
        let filter = Filter {
            with_any_tag: "keeper, portfolio".into(),
            without_tags: "reject".into(),
            ..Filter::new()
        };

        assert!(filter.matches(&rated("a.jpg", 0, &["Keeper"])));
        assert!(filter.matches(&rated("b.jpg", 0, &["portfolio", "landscape"])));
        assert!(!filter.matches(&rated("c.jpg", 0, &["landscape"])));
        assert!(!filter.matches(&rated("d.jpg", 0, &["keeper", "reject"])));
    }

    #[test]
    fn the_rules_combine_with_and() {
        let filter = Filter {
            name_contains: "img".into(),
            extensions: "jpg".into(),
            min_rating: 3,
            ..Filter::new()
        };

        let mut wanted = rated("img_1.jpg", 4, &[]);
        wanted.path = "/photos/img_1.jpg".into();
        assert!(filter.matches(&wanted));

        let mut wrong_type = rated("img_1.cr3", 4, &[]);
        wrong_type.path = "/photos/img_1.cr3".into();
        assert!(!filter.matches(&wrong_type));
    }

    #[test]
    fn applying_keeps_the_order_of_what_is_left() {
        let mut entries = vec![
            entry("a.jpg", 0, &[]),
            entry("b.png", 0, &[]),
            entry("c.jpg", 0, &[]),
        ];

        Filter {
            extensions: "jpg".into(),
            ..Filter::new()
        }
        .apply(&mut entries);

        let names: Vec<&str> = entries.iter().map(Entry::name).collect();
        assert_eq!(names, vec!["a.jpg", "c.jpg"]);
    }
}
