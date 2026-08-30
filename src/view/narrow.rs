//! Narrowing and ordering the open folder, where the photographs are.
//!
//! `organize::Filter` has done this since the folder modes were written, and
//! it is sealed inside three modes that draw no photographs: "show me the
//! three stars and better" meant leaving the picture behind. This is the same
//! idea over what is cheaply known about every file — the marks, which are
//! already read for the whole collection, and the path — so it can be applied
//! while browsing and re-applied on every keystroke without touching a disk.

use std::path::Path;

use crate::metadata::xmp::{Flag, Label, MAX_RATING};
use crate::view::image_view::bottom_bar::Marks;

use super::visible::Visible;

/// What a photograph has to be to stay on show.
///
/// Every rule is "anything" by default and they combine with "and", so a
/// filter nobody has touched is the whole folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    /// Stars, as a closed range.
    pub min_stars: u8,
    pub max_stars: u8,
    pub flag: FlagRule,
    pub label: LabelRule,
    /// Kept when the file name contains this, ignoring case.
    pub name_contains: String,
    /// Comma separated extensions, without dots. Empty means any.
    pub extensions: String,
    /// Kept when one of the photograph's keywords contains this.
    pub keyword: String,
}

impl Default for Rules {
    fn default() -> Rules {
        Rules {
            min_stars: 0,
            max_stars: MAX_RATING as u8,
            flag: FlagRule::Any,
            label: LabelRule::Any,
            name_contains: String::new(),
            extensions: String::new(),
            keyword: String::new(),
        }
    }
}

/// Which flag a photograph has to carry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlagRule {
    #[default]
    Any,
    Picked,
    Rejected,
    Unflagged,
    /// Everything except the rejects, which is the one people leave on.
    NotRejected,
}

impl FlagRule {
    pub const ALL: &'static [FlagRule] = &[
        FlagRule::Any,
        FlagRule::NotRejected,
        FlagRule::Picked,
        FlagRule::Rejected,
        FlagRule::Unflagged,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FlagRule::Any => "Any flag",
            FlagRule::NotRejected => "Not rejected",
            FlagRule::Picked => "Kept",
            FlagRule::Rejected => "Rejected",
            FlagRule::Unflagged => "Unflagged",
        }
    }

    fn matches(self, flag: Flag) -> bool {
        match self {
            FlagRule::Any => true,
            FlagRule::NotRejected => flag != Flag::Rejected,
            FlagRule::Picked => flag == Flag::Picked,
            FlagRule::Rejected => flag == Flag::Rejected,
            FlagRule::Unflagged => flag == Flag::Unflagged,
        }
    }
}

/// Which colour label a photograph has to carry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LabelRule {
    #[default]
    Any,
    None,
    /// One in particular, by its position in [`Label::CHOICES`].
    One(usize),
}

impl LabelRule {
    fn matches(self, label: Option<Label>) -> bool {
        match self {
            LabelRule::Any => true,
            LabelRule::None => label.is_none(),
            LabelRule::One(index) => label == Label::CHOICES.get(index).copied(),
        }
    }
}

/// What the collection is ordered by.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortBy {
    /// The order the crawler found them in, which is already natural by name.
    #[default]
    Name,
    Stars,
    Label,
    Flag,
}

impl SortBy {
    pub const ALL: &'static [SortBy] = &[SortBy::Name, SortBy::Stars, SortBy::Label, SortBy::Flag];

    pub fn label(self) -> &'static str {
        match self {
            SortBy::Name => "Name",
            SortBy::Stars => "Stars",
            SortBy::Label => "Colour label",
            SortBy::Flag => "Flag",
        }
    }
}

/// A filter and an order, as the views hold them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Narrowing {
    pub rules: Rules,
    pub sort: SortBy,
    pub descending: bool,
    /// Set aside without being forgotten, so "what did I hide?" is one key.
    pub suspended: bool,
}

impl Narrowing {
    /// Whether this would change anything about the collection.
    pub fn is_idle(&self) -> bool {
        self.suspended
            || (self.rules == Rules::default() && self.sort == SortBy::Name && !self.descending)
    }

    /// Whether anything is being held back, as opposed to merely reordered.
    pub fn hides_anything(&self) -> bool {
        !self.suspended && self.rules != Rules::default()
    }

    /// What to show, given every photograph and what it carries.
    ///
    /// `marks` is in the same order as `paths`; a photograph the marks have
    /// not reached is treated as unmarked, which is what it is.
    pub fn apply(&self, paths: &[std::path::PathBuf], marks: &[Marks]) -> Visible {
        if self.is_idle() {
            return Visible::everything(paths.len());
        }

        let unmarked = Marks::default();
        let mark_of = |index: usize| marks.get(index).unwrap_or(&unmarked);

        let mut order: Vec<usize> = (0..paths.len())
            .filter(|index| self.suspended || self.rules.matches(&paths[*index], mark_of(*index)))
            .collect();

        if self.sort != SortBy::Name || self.descending {
            order.sort_by(|a, b| {
                let ordering = match self.sort {
                    // The crawler already ordered them; keeping that as the
                    // tie-break is what makes every other key stable.
                    SortBy::Name => a.cmp(b),
                    SortBy::Stars => mark_of(*a).stars.cmp(&mark_of(*b).stars).then(a.cmp(b)),
                    SortBy::Label => label_key(mark_of(*a))
                        .cmp(&label_key(mark_of(*b)))
                        .then(a.cmp(b)),
                    SortBy::Flag => flag_key(mark_of(*a))
                        .cmp(&flag_key(mark_of(*b)))
                        .then(a.cmp(b)),
                };

                if self.descending {
                    ordering.reverse()
                } else {
                    ordering
                }
            });
        }

        Visible::of(order, paths.len())
    }
}

/// Unlabelled sorts after every label rather than before it, because "no
/// label" is the absence of an answer rather than the first one.
fn label_key(marks: &Marks) -> usize {
    marks
        .label
        .and_then(|label| Label::CHOICES.iter().position(|known| *known == label))
        .unwrap_or(Label::CHOICES.len())
}

/// Kept, then unflagged, then rejected: best first, which is what every other
/// ordering here does.
fn flag_key(marks: &Marks) -> usize {
    match marks.flag {
        Flag::Picked => 0,
        Flag::Unflagged => 1,
        Flag::Rejected => 2,
    }
}

impl Rules {
    /// Whether a photograph passes every rule.
    pub fn matches(&self, path: &Path, marks: &Marks) -> bool {
        self.matches_stars(marks)
            && self.flag.matches(marks.flag)
            && self.label.matches(marks.label)
            && self.matches_name(path)
            && self.matches_extension(path)
            && self.matches_keyword(marks)
    }

    fn matches_stars(&self, marks: &Marks) -> bool {
        let high = self.max_stars.max(self.min_stars);

        marks.stars >= self.min_stars && marks.stars <= high
    }

    fn matches_name(&self, path: &Path) -> bool {
        let wanted = self.name_contains.trim();
        if wanted.is_empty() {
            return true;
        }

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        name.contains(&wanted.to_lowercase())
    }

    fn matches_extension(&self, path: &Path) -> bool {
        let wanted = crate::organize::list(&self.extensions);
        if wanted.is_empty() {
            return true;
        }

        let extension = crate::formats::extension_of(path);
        wanted.iter().any(|allowed| {
            allowed
                .trim_start_matches('.')
                .eq_ignore_ascii_case(&extension)
        })
    }

    fn matches_keyword(&self, marks: &Marks) -> bool {
        let wanted = self.keyword.trim();
        if wanted.is_empty() {
            return true;
        }

        marks
            .keywords
            .iter()
            .any(|keyword| keyword.to_lowercase().contains(&wanted.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn marks(stars: u8, flag: Flag, label: Option<Label>, keywords: &[&str]) -> Marks {
        Marks {
            stars,
            flag,
            label,
            keywords: keywords.iter().map(|k| (*k).to_string()).collect(),
        }
    }

    fn folder() -> (Vec<PathBuf>, Vec<Marks>) {
        let paths = ["a.jpg", "b.cr2", "c.jpg", "d.png"]
            .iter()
            .map(|name| PathBuf::from("/photos").join(name))
            .collect();

        let marks = vec![
            marks(5, Flag::Picked, Some(Label::Green), &["Keeper"]),
            marks(0, Flag::Rejected, None, &[]),
            marks(3, Flag::Unflagged, Some(Label::Red), &["Portrait"]),
            marks(0, Flag::Unflagged, None, &[]),
        ];

        (paths, marks)
    }

    fn shown(narrowing: &Narrowing) -> Vec<usize> {
        let (paths, marks) = folder();
        narrowing.apply(&paths, &marks).iter().collect()
    }

    #[test]
    fn an_untouched_filter_is_the_whole_folder() {
        let narrowing = Narrowing::default();

        assert!(narrowing.is_idle());
        assert!(!narrowing.hides_anything());
        assert_eq!(shown(&narrowing), vec![0, 1, 2, 3]);
    }

    #[test]
    fn stars_narrow_to_a_range() {
        let narrowing = Narrowing {
            rules: Rules {
                min_stars: 3,
                ..Rules::default()
            },
            ..Narrowing::default()
        };

        assert_eq!(shown(&narrowing), vec![0, 2]);
    }

    /// The rule people leave on: everything except what they have said no to.
    #[test]
    fn the_rejects_can_be_put_out_of_sight() {
        let narrowing = Narrowing {
            rules: Rules {
                flag: FlagRule::NotRejected,
                ..Rules::default()
            },
            ..Narrowing::default()
        };

        assert_eq!(shown(&narrowing), vec![0, 2, 3]);
        assert!(narrowing.hides_anything());
    }

    #[test]
    fn a_colour_label_narrows_to_itself() {
        let red = Label::CHOICES
            .iter()
            .position(|l| *l == Label::Red)
            .unwrap();

        let narrowing = Narrowing {
            rules: Rules {
                label: LabelRule::One(red),
                ..Rules::default()
            },
            ..Narrowing::default()
        };

        assert_eq!(shown(&narrowing), vec![2]);
    }

    #[test]
    fn unlabelled_is_a_rule_of_its_own() {
        let narrowing = Narrowing {
            rules: Rules {
                label: LabelRule::None,
                ..Rules::default()
            },
            ..Narrowing::default()
        };

        assert_eq!(shown(&narrowing), vec![1, 3]);
    }

    #[test]
    fn the_type_and_the_name_narrow_too() {
        let by_type = Narrowing {
            rules: Rules {
                extensions: "jpg".to_string(),
                ..Rules::default()
            },
            ..Narrowing::default()
        };
        assert_eq!(shown(&by_type), vec![0, 2]);

        let by_name = Narrowing {
            rules: Rules {
                name_contains: "C".to_string(),
                ..Rules::default()
            },
            ..Narrowing::default()
        };
        assert_eq!(shown(&by_name), vec![1, 2]);
    }

    #[test]
    fn a_keyword_narrows_and_ignores_case() {
        let narrowing = Narrowing {
            rules: Rules {
                keyword: "keep".to_string(),
                ..Rules::default()
            },
            ..Narrowing::default()
        };

        assert_eq!(shown(&narrowing), vec![0]);
    }

    #[test]
    fn rules_combine_with_and() {
        let narrowing = Narrowing {
            rules: Rules {
                min_stars: 3,
                extensions: "jpg".to_string(),
                name_contains: "c".to_string(),
                ..Rules::default()
            },
            ..Narrowing::default()
        };

        assert_eq!(shown(&narrowing), vec![2]);
    }

    #[test]
    fn sorting_by_stars_puts_the_best_last_and_then_first() {
        let up = Narrowing {
            sort: SortBy::Stars,
            ..Narrowing::default()
        };
        assert_eq!(shown(&up), vec![1, 3, 2, 0]);

        let down = Narrowing {
            sort: SortBy::Stars,
            descending: true,
            ..Narrowing::default()
        };
        assert_eq!(shown(&down), vec![0, 2, 3, 1]);
    }

    #[test]
    fn sorting_puts_the_unlabelled_and_the_unflagged_where_they_belong() {
        let by_label = Narrowing {
            sort: SortBy::Label,
            ..Narrowing::default()
        };
        // Red, green, then the two with no label at all.
        assert_eq!(shown(&by_label), vec![2, 0, 1, 3]);

        let by_flag = Narrowing {
            sort: SortBy::Flag,
            ..Narrowing::default()
        };
        assert_eq!(shown(&by_flag), vec![0, 2, 3, 1]);
    }

    /// Suspending keeps the rules and shows everything, so "what did I hide?"
    /// costs one key and answering it costs nothing.
    #[test]
    fn suspending_shows_everything_without_forgetting_the_rules() {
        let narrowing = Narrowing {
            rules: Rules {
                min_stars: 5,
                ..Rules::default()
            },
            suspended: true,
            ..Narrowing::default()
        };

        assert_eq!(shown(&narrowing), vec![0, 1, 2, 3]);
        assert!(!narrowing.hides_anything());
        assert_ne!(narrowing.rules, Rules::default());
    }

    #[test]
    fn a_filter_that_matches_nothing_shows_nothing() {
        let narrowing = Narrowing {
            rules: Rules {
                keyword: "nothing has this".to_string(),
                ..Rules::default()
            },
            ..Narrowing::default()
        };

        assert!(shown(&narrowing).is_empty());
    }
}
