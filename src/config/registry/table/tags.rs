//! `tags.*`: the marks a photograph carries, and the panel that puts them on.

use super::*;

use crate::metadata::xmp::{Label, MAX_RATING};

pub fn rows() -> Vec<Row> {
    let mut rows = vec![
        row!(
            Keywords / Plain,
            "tags.categories",
            "The keyword list",
            "Keywords offered in the panel, grouped under headings. Hierarchical: a \
             keyword written Places|Slovakia|Tatras is filed under its levels, so \
             narrowing by Slovakia finds everything underneath it.",
            ["keywords", "tags", "vocabulary", "hierarchy", "categories"],
            Restart,
            None,
            Access::Records(List::Categories, |c| c.tags.categories.len()),
        ),
        row!(
            Keywords / Plain,
            "tags.catalog_file",
            "A keyword list from a file",
            "A Lightroom-style keyword file, read once at startup and added to the \
             list above. A relative path is taken against the configuration directory, \
             not the working one.",
            ["catalog", "catalogue", "keywords", "lightroom", "import"],
            Restart,
            None,
            optional_text!(tags.catalog_file),
        ),
        row!(
            Keywords / Plain,
            "tags.recent_tags",
            "Recently used keywords offered",
            "How many of the keywords you have typed lately are offered again at the \
             top of the panel. Zero turns that off.",
            ["recent", "history", "keywords"],
            Restart,
            None,
            whole!(usize, 0, 64, "", true, tags.recent_tags),
        ),
        row!(
            Keywords / Panels,
            "tags.panel_width",
            "How wide the panel is",
            "The panel of stars, flags and keywords down the left. Dragging its edge \
             changes this too.",
            ["panel", "width", "sidebar"],
            Live,
            None,
            decimal!(180.0, 900.0, " pt", true, tags.panel_width),
        ),
        row!(
            Marks / Plain,
            "tags.advance_after_marking",
            "Move on after marking",
            "Whether a star, a flag or a colour label moves to the next photograph by \
             itself. Marking a selection never advances: the mark went to two hundred \
             photographs rather than to the one on screen, so there is nothing for \
             \"the next one\" to mean.",
            ["auto advance", "next", "cull", "rating"],
            Live,
            None,
            boolean!(tags.advance_after_marking),
        ),
    ];

    rows.extend(keys());
    rows.extend(labels());
    rows.extend(ratings());
    rows
}

fn keys() -> Vec<Row> {
    vec![
        row!(KeysAndMouse / Keys, "tags.sc_toggle_tag_panel", "Tag panel",
            "Show or hide the panel for stars and keywords.",
            ["keywords", "panel", "tags"], Live, Everywhere, key!(tags.sc_toggle_tag_panel)),
        row!(KeysAndMouse / Keys, "tags.sc_pick", "Keep",
            "Mark the picture on screen as one to keep. Pressing it again takes the mark off.",
            ["flag", "pick", "keeper"], Live, Everywhere, key!(tags.sc_pick)),
        row!(KeysAndMouse / Keys, "tags.sc_reject", "Reject",
            "Mark it as one to throw out. Pressing it again puts it back.",
            ["flag", "reject", "x"], Live, Everywhere, key!(tags.sc_reject)),
        row!(KeysAndMouse / Keys, "tags.sc_unflag", "No flag",
            "Take whichever of those two marks it carries back off it.",
            ["unflag", "clear"], Live, Everywhere, key!(tags.sc_unflag)),
        row!(KeysAndMouse / Keys, "tags.sc_toggle_advance", "Advance after marking",
            "Turn on and off moving to the next picture as soon as one is rated, flagged or labelled.",
            ["auto advance", "next"], Live, Everywhere, key!(tags.sc_toggle_advance)),
        row!(
            Marks / Plain,
            "tags.sc_rating",
            "The rating keys",
            "One key per rating, from no stars up to five. They are edited on Keys and \
             mouse, a row each.",
            ["stars", "rating", "0", "5"],
            Live,
            None,
            Access::Records(List::RatingKeys, |c| c.tags.sc_rating.len()),
        ),
        row!(
            Marks / Plain,
            "tags.sc_label",
            "The colour label keys",
            "One key per colour, in the order red, yellow, green, blue, purple. They \
             are edited on Keys and mouse, a row each.",
            ["colour", "color", "label", "color class"],
            Live,
            None,
            Access::Records(List::LabelKeys, |c| c.tags.sc_label.len()),
        ),
    ]
}

/// One row per colour label, over the list the file holds.
fn labels() -> Vec<Row> {
    Label::CHOICES
        .iter()
        .enumerate()
        .map(|(index, label)| Row {
            page: Page::KeysAndMouse,
            group: Group::Keys,
            path: LABEL_PATHS[index],
            label: label.name(),
            sentence: LABEL_SENTENCES[index],
            aliases: &["colour", "color", "label", "color class"],
            access: Access::LabelKey(index),
            effect: Effect::Live,
            scope: Scope::Everywhere,
            explained: None,
        })
        .collect()
}

/// And one per rating.
fn ratings() -> Vec<Row> {
    (0..=MAX_RATING as usize)
        .map(|stars| Row {
            page: Page::KeysAndMouse,
            group: Group::Keys,
            path: RATING_PATHS[stars],
            label: RATING_NAMES[stars],
            sentence: RATING_SENTENCES[stars],
            aliases: &["stars", "rating"],
            access: Access::RatingKey(stars),
            effect: Effect::Live,
            scope: Scope::Everywhere,
            explained: None,
        })
        .collect()
}

/// Synthetic paths: a bracket says the row is one element of a list rather
/// than a key of its own, which is how the index test tells them apart.
const LABEL_PATHS: &[&str] = &[
    "tags.sc_label[0]",
    "tags.sc_label[1]",
    "tags.sc_label[2]",
    "tags.sc_label[3]",
    "tags.sc_label[4]",
];

const LABEL_SENTENCES: &[&str] = &[
    "Put the red label on the picture on screen. Pressing it again takes it off.",
    "Put the yellow label on it.",
    "Put the green label on it.",
    "Put the blue label on it.",
    "Put the purple label on it.",
];

const RATING_PATHS: &[&str] = &[
    "tags.sc_rating[0]",
    "tags.sc_rating[1]",
    "tags.sc_rating[2]",
    "tags.sc_rating[3]",
    "tags.sc_rating[4]",
    "tags.sc_rating[5]",
];

const RATING_NAMES: &[&str] = &[
    "No stars",
    "One star",
    "Two stars",
    "Three stars",
    "Four stars",
    "Five stars",
];

const RATING_SENTENCES: &[&str] = &[
    "Take the rating off the picture on screen.",
    "Put one star on the picture on screen.",
    "Put two stars on it.",
    "Put three stars on it.",
    "Put four stars on it.",
    "Put five stars on it.",
];
