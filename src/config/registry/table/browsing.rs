//! `browsing.*` and `group.*`: what a folder opens as, and what counts as one
//! run of frames.

use super::*;

use crate::choices::Choices;
use crate::config::kinds::{FlagRule, SortBy};

/// The words themselves, from the set that declares them — not a second
/// copy of them. These five had drifted into two vocabularies.
const SORT: &[Choice] = <crate::config::kinds::SortBy as Choices>::ROWS;

/// The words themselves, from the set that declares them — not a second
/// copy of them. These five had drifted into two vocabularies.
const FLAGS: &[Choice] = <crate::config::kinds::FlagRule as Choices>::ROWS;

const LABELS: &[Choice] = &[
    Choice {
        value: "any",
        label: "Any colour",
        sentence: "",
    },
    Choice {
        value: "none",
        label: "No colour",
        sentence: "",
    },
    Choice {
        value: "red",
        label: "Red",
        sentence: "",
    },
    Choice {
        value: "yellow",
        label: "Yellow",
        sentence: "",
    },
    Choice {
        value: "green",
        label: "Green",
        sentence: "",
    },
    Choice {
        value: "blue",
        label: "Blue",
        sentence: "",
    },
    Choice {
        value: "purple",
        label: "Purple",
        sentence: "",
    },
];

pub fn rows() -> Vec<Row> {
    vec![
        row!(
            OpeningAFolder / Browsing,
            "browsing.sort",
            "Order a folder by",
            "What a folder is sorted by when it is opened. The filter bar changes it \
             for the session; this is what a launch starts with.",
            ["sort", "order", "arrange", "by name", "by date"],
            Reopen,
            None,
            Access::Enum {
                get: |c| c.browsing.sort.value(),
                set: |c, v| {
                    if let Some(sort) = SortBy::of(v) {
                        c.browsing.sort = sort;
                    }
                },
                choices: SORT,
            },
        ),
        row!(
            OpeningAFolder / Browsing,
            "browsing.descending",
            "Newest or highest first",
            "Turns that order round.",
            ["descending", "reverse", "backwards", "newest first"],
            Reopen,
            None,
            boolean!(browsing.descending),
        ),
        row!(
            OpeningAFolder / Browsing,
            "browsing.flag",
            "Show which flags",
            "Which flag a photograph has to carry to be shown when a folder opens.",
            ["filter", "flag", "reject", "pick", "keeper"],
            Reopen,
            None,
            Access::Enum {
                get: |c| c.browsing.flag.value(),
                set: |c, v| {
                    if let Some(flag) = FlagRule::of(v) {
                        c.browsing.flag = flag;
                    }
                },
                choices: FLAGS,
            },
        ),
        row!(
            OpeningAFolder / Browsing,
            "browsing.min_stars",
            "Fewest stars shown",
            "Photographs with fewer than this are held back when a folder opens.",
            ["stars", "rating", "filter", "3 stars and better"],
            Reopen,
            None,
            whole!(u8, 0, 5, "", true, browsing.min_stars),
        ),
        row!(
            OpeningAFolder / Browsing,
            "browsing.max_stars",
            "Most stars shown",
            "And with more than this.",
            ["stars", "rating", "filter"],
            Reopen,
            None,
            whole!(u8, 0, 5, "", true, browsing.max_stars),
        ),
        row!(
            OpeningAFolder / Browsing,
            "browsing.label",
            "Show which colour",
            "Which colour label a photograph has to carry to be shown when a folder \
             opens.",
            ["colour", "color", "label", "color class", "filter"],
            Reopen,
            None,
            Access::Enum {
                get: |c| {
                    LABELS
                        .iter()
                        .find(|choice| choice.value == c.browsing.label)
                        .map(|choice| choice.value)
                        .unwrap_or("any")
                },
                set: |c, v| c.browsing.label = v.to_string(),
                choices: LABELS,
            },
        ),
        row!(
            OpeningAFolder / Browsing,
            "browsing.filter_follows_folder",
            "Keep the filter when another folder is opened",
            "The rules survive today and nothing resets them, which is right for \
             somebody walking a card of shoots with \"three stars and better\" up, and \
             wrong for somebody who set a rule once and forgot.",
            ["filter", "sticky", "reset", "folder"],
            Live,
            None,
            boolean!(browsing.filter_follows_folder),
        ),
        row!(
            OpeningAFolder / Browsing,
            "browsing.stack_by_default",
            "Fold bursts into one cell",
            "Whether a folder opens with every burst, bracket and timelapse shown as \
             one cell standing for the run. Worked out from what the files say, every \
             time; nothing is written to disk for it.",
            ["stack", "burst", "group", "fold", "bracket"],
            Reopen,
            None,
            boolean!(browsing.stack_by_default),
        ),
        row!(
            OpeningAFolder / Grouping,
            "group.max_gap",
            "A run breaks after a gap of",
            "The longest pause between two frames that is still one run. Read by the \
             contact sheet's stacking and by Group shots, which used to hold two \
             separate values tuned by two control sets that did not span the same \
             ranges.",
            ["burst", "gap", "seconds", "interval", "stack"],
            Reopen,
            None,
            decimal!(1.0, 3600.0, " s", true, group.max_gap),
        ),
        row!(
            OpeningAFolder / Grouping,
            "group.tolerance",
            "How alike two frames must look",
            "Zero is identical; sixty-four accepts anything. A judgement rather than a \
             number, so it is dragged and watched.",
            ["burst", "similar", "alike", "tolerance", "stack"],
            Reopen,
            None,
            whole!(u32, 0, 64, "", true, group.tolerance),
        ),
        row!(
            OpeningAFolder / Grouping,
            "group.min_frames",
            "Fewest frames that make a run",
            "Below this the frames stay loose. Two is a pair of exposures; raising it \
             leaves the pairs alone and folds only the real bursts.",
            ["burst", "minimum", "frames", "stack"],
            Reopen,
            None,
            whole!(usize, 2, 50, " frames", true, group.min_frames),
        ),
    ]
}
