//! What the second button offers on a photograph, a cell and a thumbnail.
//!
//! One list of rows, drawn the same way in both views, with whatever the user
//! configured appended under a separator in their own order. A fresh install
//! used to answer a right-click with nothing at all: the default entry list is
//! empty and the menu returns before registering anything when it is.
//!
//! One level, with three exceptions. A submenu is placed against the right edge
//! of the row that opens it and folds back to the left when the screen has no
//! room — and every panel in this program sits against an edge, so a second
//! level is worth the risk only where the rows behind it are variations on one
//! another and would otherwise take several of the twelve rows a menu may
//! carry. The turns are that, and so are the three zooms; the panels are the
//! third, and the one that is not a verb. Five, three and eight rows folded
//! into three. The submenu is kept narrower than its parent so that there is
//! somewhere for it to go.

use eframe::egui;

use crate::config::ContextMenuEntry;
use crate::metadata::Orientation;

/// A verb the built-in menu offers.
///
/// Done by whoever is able to: the view answers for what it draws, and the
/// rest goes up to the application, which is the only thing that knows about
/// raw and JPEG pairs, the journal, and what a selection is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    /// Show this photograph on its own. Cells only.
    Open,
    /// Fit the whole photograph in the panel.
    Fit,
    /// One screen pixel per image pixel.
    ActualPixels,
    /// Fill the panel, cropping the overflowing side.
    Fill,
    /// Pin photographs side by side: the ones picked out, or this one and
    /// its neighbours when nothing is.
    Compare,
    /// Keep this one, or take the mark off if it already carries it.
    ///
    /// The two verbs a cull is made of, and the only ones here that are about
    /// the *photograph* rather than the file or the view. They are on the
    /// photograph's own menu because that is what a pane carries, and a
    /// comparison of four is exactly where "which of these" has to be
    /// answerable without counting keys.
    Keep,
    /// Throw this one out, or take the mark off again.
    Reject,
    /// Put every picked-out photograph back, leaving the set empty.
    ///
    /// The one verb here that is about the set rather than about a file, and
    /// the one that needs a way out that is not a key: a set built by twenty
    /// clicks is undone by one, and until now the only way to reach that was
    /// `Escape` in the contact sheet — which is not where a set built on the
    /// strip is being looked at.
    PickNone,
    /// The turns, written to the sidecar rather than to the file. All five
    /// live behind one word; [`Verb::TURNS`] is what that word opens.
    TurnRight,
    TurnLeft,
    TurnHalf,
    MirrorHorizontally,
    MirrorVertically,
    /// Send it to the bin.
    Bin,
    /// Move it somewhere, through the panel that asks where.
    ///
    /// What a cut would be, if a clipboard could hold "these files, to be
    /// moved". None of the three platforms agrees on how to say that and the
    /// crate this program uses for the clipboard carries text and pixels and
    /// nothing else — so the destination is asked for instead, which is one
    /// gesture rather than two and does not leave a cut hanging when the paste
    /// never comes.
    MoveTo,
    /// Copy it somewhere, through the same panel.
    CopyTo,
    /// Take it out of the viewer's own bin and put it back where it came from.
    /// Offered only inside the bin, where [`Verb::Bin`] means nothing.
    PutBack,
    /// Off the disk. Offered only inside the bin, which is the one place
    /// deleting for good is the verb that applies.
    DeleteForGood,
    CopyPath,
    /// The pixels themselves, on the clipboard.
    CopyPicture,
    ShowInFolder,
}

impl Verb {
    /// The turns, in the order they are drawn.
    ///
    /// The two quarters first because they are what people open the word for,
    /// then the half turn, then the two mirrors — which are a different thing
    /// from a turn and are last so that a slip of the pointer does not land on
    /// one.
    pub const TURNS: &'static [Verb] = &[
        Verb::TurnRight,
        Verb::TurnLeft,
        Verb::TurnHalf,
        Verb::MirrorHorizontally,
        Verb::MirrorVertically,
    ];

    /// The three ways of saying how large, in the order they are drawn.
    ///
    /// One decision with three answers, which is the turns' shape, and they
    /// fold behind a word for the turns' reason: the photograph's menu is the
    /// longest in the program and the panels wanted a row on it. Fitting
    /// first, because it is what the zoom already floors at and what most
    /// presses of the word are looking for.
    pub const ZOOMS: &'static [Verb] = &[Verb::Fit, Verb::ActualPixels, Verb::Fill];

    /// What this verb does to the orientation, for the five that are turns.
    ///
    /// An orientation rather than a number of quarters, which is what lets the
    /// mirrors in at all: composed with whatever is already there
    /// ([`Orientation::then`]) any of these is one of the same eight values, so
    /// nothing below this line learns that a photograph can now be flipped.
    pub fn turn(self) -> Option<Orientation> {
        match self {
            Verb::TurnRight => Some(Orientation::quarter(true)),
            Verb::TurnLeft => Some(Orientation::quarter(false)),
            Verb::TurnHalf => Some(Orientation::Rotate180),
            Verb::MirrorHorizontally => Some(Orientation::MirrorHorizontal),
            Verb::MirrorVertically => Some(Orientation::MirrorVertical),
            _ => None,
        }
    }

    /// What the row says, for `count` photographs.
    ///
    /// The count goes in the label rather than being left to be guessed:
    /// "Move 24 photographs to the bin" is a different sentence from "Move 1
    /// photograph to the bin", and the second button is where somebody finds
    /// out how big their selection got. The turns say nothing about it because
    /// the word that opens them already has.
    pub fn label(self, count: usize) -> String {
        let these = |singular: &str, plural: &str| {
            if count == 1 {
                singular.to_string()
            } else {
                format!("{count} {plural}")
            }
        };

        match self {
            Verb::Open => "Open".to_string(),
            Verb::Fit => "Fit in the window".to_string(),
            Verb::ActualPixels => "Actual pixels".to_string(),
            Verb::Fill => "Fill the window".to_string(),
            Verb::Compare if count == 1 => "Compare".to_string(),
            Verb::Compare => format!("Compare {count} photographs side by side"),
            Verb::Keep => "Keep".to_string(),
            Verb::Reject => "Throw out".to_string(),
            Verb::PickNone => format!("Put all {count} back"),
            Verb::TurnRight => "Clockwise".to_string(),
            Verb::TurnLeft => "Anticlockwise".to_string(),
            Verb::TurnHalf => "Upside down".to_string(),
            Verb::MirrorHorizontally => "Mirror left to right".to_string(),
            Verb::MirrorVertically => "Mirror top to bottom".to_string(),
            Verb::Bin => format!(
                "Move {} to the bin",
                these("this photograph", "photographs")
            ),
            Verb::PutBack => format!(
                "Put {} back where {} came from",
                these("this photograph", "photographs"),
                if count == 1 { "it" } else { "they" }
            ),
            Verb::DeleteForGood => format!(
                "Delete {} for good",
                these("this photograph", "photographs")
            ),
            Verb::MoveTo => format!("Move {}…", these("this photograph", "photographs")),
            Verb::CopyTo => format!("Copy {} to…", these("this photograph", "photographs")),
            Verb::CopyPath => format!("Copy the {}", these("path", "paths")),
            Verb::CopyPicture => "Copy the picture".to_string(),
            Verb::ShowInFolder => "Show it in the file manager".to_string(),
        }
    }

    /// The sentence under the pointer.
    ///
    /// The mirrors are named twice — "left to right" on the row and
    /// "horizontally" here — because the two wordings are used
    /// interchangeably by other programs and neither of them is obvious.
    pub fn hint(self) -> &'static str {
        match self {
            Verb::Open => "Show this photograph on its own",
            Verb::Keep => {
                "Mark it kept, or take the mark off if it is already kept. The                  same as the keep key, about the photograph under the pointer"
            }
            Verb::Reject => {
                "Mark it rejected, or take the mark off again. A rejected                  photograph leaves a comparison of the ones picked out"
            }
            Verb::Fit => "The whole photograph, as large as the window allows",
            Verb::ActualPixels => "One screen pixel per pixel of the photograph",
            Verb::Fill => "Fill the window, cropping whichever side is longer",
            Verb::Compare => {
                "Side by side and pinned there. The photographs picked out, or this \n                 one and the ones beside it when none are"
            }
            Verb::TurnRight => {
                "A quarter turn clockwise, written to the sidecar. The photograph \
                 itself is never touched"
            }
            Verb::TurnLeft => {
                "A quarter turn anticlockwise, written to the sidecar. The \
                 photograph itself is never touched"
            }
            Verb::TurnHalf => "Half a turn, for one that came in upside down",
            Verb::MirrorHorizontally => {
                "Mirrored horizontally, left for right — a negative scanned the \
                 wrong way round"
            }
            Verb::MirrorVertically => "Mirrored vertically, top for bottom",
            Verb::Bin => {
                "To the bin — the platform's, unless the viewer has been given a \
                 folder of its own, which is what reaches a memory card"
            }
            Verb::PutBack => {
                "Out of the bin and back into the folder it was thrown out of, \
                 which the bin wrote down when it took it"
            }
            Verb::DeleteForGood => "Off the disk. Nothing can take this one back",
            Verb::PickNone => {
                "Put every picked-out photograph back, so the next command is \
                 about the one being looked at again"
            }
            Verb::MoveTo => {
                "Off to one of the numbered destinations, which the panel asks for. \
                 The nearest thing to a cut: the file goes, in one gesture"
            }
            Verb::CopyTo => {
                "A copy to one of the numbered destinations, leaving the original \
                 where it is"
            }
            Verb::CopyPath => "The whole path, for pasting into something else",
            Verb::CopyPicture => "The pixels themselves, decoded at full size and turned upright",
            Verb::ShowInFolder => "Open the folder it is in, with it picked out",
        }
    }

    /// Whether choosing this closes the menu.
    ///
    /// The zoom verbs do not: somebody trying "fit" then "fill" is comparing
    /// them, and closing the menu between the two makes that four gestures
    /// rather than two.
    fn closes(self) -> bool {
        !matches!(self, Verb::Fit | Verb::ActualPixels | Verb::Fill)
    }

    /// Which flag this verb sets, for the two that are marks.
    pub fn flag(self) -> Option<crate::metadata::xmp::Flag> {
        match self {
            Verb::Keep => Some(crate::metadata::xmp::Flag::Picked),
            Verb::Reject => Some(crate::metadata::xmp::Flag::Rejected),
            _ => None,
        }
    }

    /// Where the key that also does this lives in the registry.
    ///
    /// A menu row names its key beside it, and the row has to say which key
    /// that is: there is no table anywhere mapping a command to the binding
    /// that raises it — `input::collect` pairs a shortcut *field* with a
    /// `Command`, and half of these verbs are not commands at all. So it is
    /// said here, once, and the test at the foot of this file holds it to the
    /// registry.
    ///
    /// Empty where the verb is this menu's alone: the two mirrors and the half
    /// turn are the three of the five turns nobody has bound, the two paths
    /// are copied from nowhere else, and showing a file in the file manager
    /// has never had a key. `ui::keys::of` answers an empty path with nothing,
    /// which is what a row with no key to name draws.
    pub fn key(self) -> &'static str {
        match self {
            Verb::Open => "fixed.grid_open",
            Verb::Fit => "image_view.sc_fit",
            Verb::ActualPixels => "image_view.sc_one_to_one",
            Verb::Fill => "image_view.sc_fit_maximize",
            Verb::Compare => "image_view.sc_compare",
            Verb::Keep => "tags.sc_pick",
            Verb::Reject => "tags.sc_reject",
            Verb::PickNone => "grid_view.sc_select_none",
            Verb::TurnRight => "general.sc_turn_right",
            Verb::TurnLeft => "general.sc_turn_left",
            Verb::Bin => "general.sc_delete",
            Verb::MoveTo => "cull.sc_move",
            Verb::CopyTo => "cull.sc_copy",
            Verb::PutBack => "cull.sc_put_back",
            Verb::DeleteForGood => "general.sc_delete_permanently",
            // The chord the toolkit turns into a copy *event* before this
            // program sees it, which is why it is a fixed row rather than a
            // binding.
            Verb::CopyPicture => "fixed.copy_area",
            Verb::TurnHalf
            | Verb::MirrorHorizontally
            | Verb::MirrorVertically
            | Verb::CopyPath
            | Verb::ShowInFolder => "",
        }
    }

    /// Whether this verb means anything in a menu about `count` photographs.
    ///
    /// One rule and one verb so far, and it belongs here rather than in a
    /// second pair of row lists: putting them all back means nothing where
    /// there is nothing to put back, and a menu about one photograph is a menu
    /// where nothing has been picked out — or where what was picked out is
    /// somewhere else, and offering to throw it away from here would be a
    /// surprise. A row that would do nothing is worse than a row that is not
    /// drawn.
    pub fn applies(self, count: usize) -> bool {
        match self {
            Verb::PickNone => count > 1,
            _ => true,
        }
    }
}

/// A row of the menu: a verb, or a word that opens a few of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Verb(Verb),
    /// The word, and the verbs behind it. One of the two second levels in the
    /// program; the note at the top of this file says why there are any.
    Group(&'static str, &'static [Verb]),
    /// The panels, and which of them are showing.
    ///
    /// The other second level, and the only row here that is not about the
    /// photograph: the rows behind it are [`crate::ui::panel::show_and_hide`]'s,
    /// one per panel with a tick against the ones on screen. It is behind a
    /// word for the same reason the turns are — eight rows inline would be two
    /// thirds of a menu that already carries eleven — and it is on the
    /// photograph because the photograph is the whole window when the panels
    /// are away, and so the only surface left to ask.
    Panels,
}

impl Row {
    /// The rows a photograph carries, in the order they are drawn.
    ///
    /// Verbs first and most used first, then copy and show because the object
    /// is a file on disk, and last the panels, which are about the window
    /// rather than about the photograph.
    pub const ON_A_PHOTOGRAPH: &'static [Row] = &[
        Row::Verb(Verb::Keep),
        Row::Verb(Verb::Reject),
        Row::Group("Zoom", Verb::ZOOMS),
        Row::Verb(Verb::Compare),
        Row::Group("Turn", Verb::TURNS),
        Row::Verb(Verb::Bin),
        Row::Verb(Verb::CopyPath),
        Row::Verb(Verb::CopyPicture),
        Row::Verb(Verb::ShowInFolder),
        Row::Panels,
    ];

    /// The same, standing inside the viewer's own bin.
    ///
    /// A menu carries the verbs that apply to what it was drawn over, and
    /// "move this to the bin" applies to nothing that is already in one. The
    /// two verbs that do apply take its place, in its position, so the row
    /// muscle memory reaches for is the one that means something here.
    pub const ON_A_PHOTOGRAPH_IN_THE_BIN: &'static [Row] = &[
        Row::Group("Zoom", Verb::ZOOMS),
        Row::Verb(Verb::Compare),
        Row::Group("Turn", Verb::TURNS),
        Row::Verb(Verb::PutBack),
        Row::Verb(Verb::DeleteForGood),
        Row::Verb(Verb::CopyPath),
        Row::Verb(Verb::CopyPicture),
        Row::Verb(Verb::ShowInFolder),
        Row::Panels,
    ];

    /// The rows a cell carries. `Open` leads, because that is what a cell is
    /// for, and the zoom verbs are not about anything the sheet draws.
    ///
    /// The two destinations are here and on the strip but not on the
    /// photograph, whose list is at ten of the eleven rows that fit. They are
    /// verbs about a *file*, which is what a cell and a thumbnail are; the
    /// photograph's own menu leads with what it is showing. Both have a key
    /// each, so nothing is only reachable here.
    pub const ON_A_CELL: &'static [Row] = &[
        Row::Verb(Verb::Open),
        Row::Verb(Verb::Compare),
        Row::Verb(Verb::PickNone),
        Row::Group("Turn", Verb::TURNS),
        Row::Verb(Verb::MoveTo),
        Row::Verb(Verb::CopyTo),
        Row::Verb(Verb::Bin),
        Row::Verb(Verb::CopyPath),
        Row::Verb(Verb::CopyPicture),
        Row::Verb(Verb::ShowInFolder),
    ];

    /// The same, standing inside the viewer's own bin.
    pub const ON_A_CELL_IN_THE_BIN: &'static [Row] = &[
        Row::Verb(Verb::Open),
        Row::Verb(Verb::Compare),
        Row::Verb(Verb::PickNone),
        Row::Group("Turn", Verb::TURNS),
        Row::Verb(Verb::MoveTo),
        Row::Verb(Verb::CopyTo),
        Row::Verb(Verb::PutBack),
        Row::Verb(Verb::DeleteForGood),
        Row::Verb(Verb::CopyPath),
        Row::Verb(Verb::CopyPicture),
        Row::Verb(Verb::ShowInFolder),
    ];

    /// The rows a thumbnail on the strip carries.
    ///
    /// A cell's list, and for the same reasons — a thumbnail is a file, and
    /// `Open` is what one is for. It is its own list rather than the cell's
    /// because the two surfaces will diverge and sharing a name for two
    /// meanings is how they diverge quietly.
    pub const ON_THE_STRIP: &'static [Row] = Row::ON_A_CELL;

    /// The same, standing inside the viewer's own bin.
    pub const ON_THE_STRIP_IN_THE_BIN: &'static [Row] = Row::ON_A_CELL_IN_THE_BIN;

    /// Which of the two lists a photograph carries.
    ///
    /// Asked in one place, so the two views cannot answer it differently and a
    /// verb added to one list cannot go quietly missing from the other.
    pub fn on_a_photograph(in_the_bin: bool) -> &'static [Row] {
        match in_the_bin {
            true => Row::ON_A_PHOTOGRAPH_IN_THE_BIN,
            false => Row::ON_A_PHOTOGRAPH,
        }
    }

    /// Which of the two lists a cell carries.
    pub fn on_a_cell(in_the_bin: bool) -> &'static [Row] {
        match in_the_bin {
            true => Row::ON_A_CELL_IN_THE_BIN,
            false => Row::ON_A_CELL,
        }
    }

    /// Which of the two lists a thumbnail on the strip carries.
    pub fn on_the_strip(in_the_bin: bool) -> &'static [Row] {
        match in_the_bin {
            true => Row::ON_THE_STRIP_IN_THE_BIN,
            false => Row::ON_THE_STRIP,
        }
    }

    /// Every verb this row can reach, whether or not it is behind a word.
    pub fn verbs(self) -> impl Iterator<Item = Verb> {
        let (alone, behind): (Option<Verb>, &'static [Verb]) = match self {
            Row::Verb(verb) => (Some(verb), &[]),
            Row::Group(_, verbs) => (None, verbs),
            Row::Panels => (None, &[]),
        };

        alone.into_iter().chain(behind.iter().copied())
    }

    /// What the row says, for `count` photographs.
    pub fn label(self, count: usize) -> String {
        match self {
            Row::Verb(verb) => verb.label(count),
            Row::Group(word, _) if count == 1 => word.to_string(),
            Row::Group(word, _) => format!("{word} {count} photographs"),
            // No count: the panels are not the photographs the rest of the
            // menu is about, and a selection changes nothing about them.
            Row::Panels => "Show".to_string(),
        }
    }
}

/// What a menu was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    Verb(Verb),
    /// The user's own entry at this position in the configuration.
    Entry(usize),
}

/// One verb, drawn. Whether it was clicked.
///
/// `ui.close()` inside a submenu closes the submenu and the menu that opened
/// it, which is what a menu is expected to do and is why nothing here has to
/// know which of the two levels it is on.
fn verb_row(ui: &mut egui::Ui, verb: Verb, count: usize) -> bool {
    let clicked = crate::ui::keys::button(ui, verb.label(count), verb.key())
        .on_hover_text(verb.hint())
        .clicked();

    if clicked && verb.closes() {
        ui.close();
    }

    clicked
}

/// Draws the built-in rows, then whatever the user configured.
///
/// The user's entries are appended in their own order under a separator, and
/// are never reordered, renamed or removed: they are the one part of this menu
/// somebody has already decided about.
pub fn rows(
    ui: &mut egui::Ui,
    rows: &[Row],
    entries: &[ContextMenuEntry],
    count: usize,
) -> Option<Chosen> {
    let mut chosen = None;

    ui.set_max_width(crate::ui::surface::WIDEST);

    for row in rows {
        // A row that would do nothing where it stands is not drawn. The lists
        // themselves stay static, which is what keeps them readable and free
        // of an allocation per cell per frame. The panels are not a verb and
        // apply wherever they are listed, selection or no selection.
        if !matches!(row, Row::Panels) && !row.verbs().any(|verb| verb.applies(count)) {
            continue;
        }

        match *row {
            Row::Verb(verb) => {
                if verb_row(ui, verb, count) {
                    chosen = Some(Chosen::Verb(verb));
                }
            }
            Row::Group(_, verbs) => {
                // Narrower than its parent, and with no hover text of its own:
                // three words to a row is a submenu that still has room to
                // open against the edge of a screen, and a tooltip on the word
                // would be drawn over what the word had just opened.
                ui.menu_button(row.label(count), |ui| {
                    ui.set_max_width(220.);

                    for verb in verbs {
                        if verb_row(ui, *verb, count) {
                            chosen = Some(Chosen::Verb(*verb));
                        }
                    }
                });
            }
            Row::Panels => {
                // Set apart, because everything above it is about the
                // photograph and this is about the window around it. What was
                // ticked goes to `ui::panel`'s own mailbox rather than back
                // through `Chosen`: the verbs here are the panels', not this
                // menu's, and the same rows are drawn on the bar.
                ui.separator();

                ui.menu_button(row.label(count), |ui| {
                    crate::ui::panel::show_and_hide(ui);
                });
            }
        }
    }

    if !entries.is_empty() {
        ui.separator();

        for (i, entry) in entries.iter().enumerate() {
            if ui.button(&entry.description).clicked() {
                chosen = Some(Chosen::Entry(i));
                ui.close();
            }
        }
    }

    chosen
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The count goes in the label, and the singular reads as a sentence.
    #[test]
    fn a_selection_says_how_big_it_is() {
        assert_eq!(Verb::Bin.label(1), "Move this photograph to the bin");
        assert_eq!(Verb::Bin.label(24), "Move 24 photographs to the bin");
        assert_eq!(Verb::CopyPath.label(1), "Copy the path");
        assert_eq!(Verb::CopyPath.label(3), "Copy the 3 paths");
    }

    /// The turns are the one row whose count is on the word above them.
    ///
    /// It used to be on each of them, through the same helper as the bin —
    /// which put the number where the pronoun goes and said "Turn 24 them
    /// clockwise" to anybody who right-clicked a selection in the sheet.
    #[test]
    fn the_word_above_the_turns_carries_the_count() {
        let turns = Row::Group("Turn", Verb::TURNS);

        assert_eq!(turns.label(1), "Turn");
        assert_eq!(turns.label(24), "Turn 24 photographs");
        assert_eq!(Verb::TurnRight.label(24), "Clockwise");
    }

    /// Twelve rows including the last is the ceiling, and the user's own
    /// entries are appended to whatever is here.
    ///
    /// The five turns take one row between them, and so do the three zooms and
    /// the eight panels: that is what the second level bought and what keeps
    /// every one of these under it. The photograph's list is the longest at
    /// ten, which with the settings row is eleven. The zooms folded on the day
    /// the panels wanted a row, because it was already at the twelve.
    #[test]
    fn no_built_in_list_uses_up_the_menu() {
        // The list, plus the settings row that closes every menu.
        for list in [
            Row::ON_A_PHOTOGRAPH,
            Row::ON_A_PHOTOGRAPH_IN_THE_BIN,
            Row::ON_A_CELL,
            Row::ON_A_CELL_IN_THE_BIN,
            Row::ON_THE_STRIP,
            Row::ON_THE_STRIP_IN_THE_BIN,
        ] {
            assert!(list.len() < 12, "{} rows", list.len());
        }
    }

    /// The panels are on the photograph and nowhere else, because the
    /// photograph is the whole window once the panels are away.
    #[test]
    fn the_panels_are_listed_on_the_photograph() {
        for list in [Row::ON_A_PHOTOGRAPH, Row::ON_A_PHOTOGRAPH_IN_THE_BIN] {
            assert!(
                list.contains(&Row::Panels),
                "the photograph lists the panels"
            );
        }

        for list in [Row::ON_A_CELL, Row::ON_A_CELL_IN_THE_BIN, Row::ON_THE_STRIP] {
            assert!(
                !list.contains(&Row::Panels),
                "a cell is a file, not a window"
            );
        }
    }

    /// One word, and the same word whatever is picked out: a selection changes
    /// nothing about which panels are up.
    #[test]
    fn the_word_above_the_panels_says_nothing_about_the_count() {
        assert_eq!(Row::Panels.label(1), "Show");
        assert_eq!(Row::Panels.label(24), "Show");
    }

    /// It carries no verb, so the rule that drops a row nothing applies to
    /// must not drop it.
    #[test]
    fn the_panels_are_not_a_verb_and_are_drawn_all_the_same() {
        assert_eq!(Row::Panels.verbs().count(), 0);

        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rows(ui, &[Row::Panels], &[], 24);
            });
        });

        let drawn: Vec<String> = output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_string()),
                _ => None,
            })
            .collect();

        assert!(drawn.iter().any(|text| text == "Show"), "{drawn:?}");
    }

    /// The two verbs a cull is made of are on the photograph, which is what a
    /// pane carries — and not inside the bin, where a frame has already been
    /// thrown out and keeping it means nothing.
    #[test]
    fn the_marks_are_on_the_photograph_and_not_in_the_bin() {
        let on_a_photograph: Vec<Verb> = Row::ON_A_PHOTOGRAPH
            .iter()
            .flat_map(|row| row.verbs())
            .collect();

        assert!(on_a_photograph.contains(&Verb::Keep));
        assert!(on_a_photograph.contains(&Verb::Reject));

        let in_the_bin: Vec<Verb> = Row::ON_A_PHOTOGRAPH_IN_THE_BIN
            .iter()
            .flat_map(|row| row.verbs())
            .collect();

        assert!(!in_the_bin.contains(&Verb::Keep));
        assert!(!in_the_bin.contains(&Verb::Reject));
    }

    /// Which flag each of the two sets, so the view does not have to know.
    #[test]
    fn the_two_marks_say_which_flag_they_are() {
        use crate::metadata::xmp::Flag;

        assert_eq!(Verb::Keep.flag(), Some(Flag::Picked));
        assert_eq!(Verb::Reject.flag(), Some(Flag::Rejected));
        assert_eq!(Verb::Bin.flag(), None);
    }

    /// Standing in the bin, the two verbs that mean nothing there are gone and
    /// the two that only mean something there have taken their place.
    #[test]
    fn the_bin_swaps_the_verbs_that_apply() {
        for list in [Row::on_a_cell(true), Row::on_the_strip(true)] {
            let verbs: Vec<Verb> = list.iter().flat_map(|row| row.verbs()).collect();

            assert!(verbs.contains(&Verb::PutBack));
            assert!(verbs.contains(&Verb::DeleteForGood));
            assert!(!verbs.contains(&Verb::Bin));
        }

        for list in [Row::on_a_cell(false), Row::on_the_strip(false)] {
            let verbs: Vec<Verb> = list.iter().flat_map(|row| row.verbs()).collect();

            assert!(verbs.contains(&Verb::Bin));
            assert!(!verbs.contains(&Verb::PutBack));
        }
    }

    /// A thumbnail on the strip is a file, so the verbs that move one are
    /// there. This is the nearest thing the program has to a cut.
    #[test]
    fn a_thumbnail_can_be_sent_somewhere() {
        let verbs: Vec<Verb> = Row::ON_THE_STRIP
            .iter()
            .flat_map(|row| row.verbs())
            .collect();

        assert!(verbs.contains(&Verb::MoveTo));
        assert!(verbs.contains(&Verb::CopyTo));
        assert!(verbs.contains(&Verb::Bin));
        assert!(verbs.contains(&Verb::CopyPicture));
        assert!(verbs.contains(&Verb::Compare));
    }

    /// Asked about a set, comparing says so: it is a different thing from
    /// pinning this photograph and the ones beside it.
    #[test]
    fn comparing_a_set_says_how_many() {
        assert_eq!(Verb::Compare.label(1), "Compare");
        assert_eq!(Verb::Compare.label(4), "Compare 4 photographs side by side");
    }

    /// Putting them all back is drawn where there is a set to put back and
    /// nowhere else: a row that would do nothing is worse than one that is not
    /// there.
    #[test]
    fn putting_them_all_back_is_offered_only_where_there_is_a_set() {
        assert!(!Verb::PickNone.applies(1));
        assert!(Verb::PickNone.applies(2));
        assert!(Verb::PickNone.applies(200));

        assert_eq!(Verb::PickNone.label(4), "Put all 4 back");
    }

    /// And every other verb applies whatever the menu is about, so the rule
    /// stays one rule about one verb.
    #[test]
    fn every_other_verb_applies_to_one_photograph() {
        for row in Row::ON_THE_STRIP {
            for verb in row.verbs() {
                if verb == Verb::PickNone {
                    continue;
                }

                assert!(verb.applies(1), "{verb:?}");
                assert!(verb.applies(9), "{verb:?}");
            }
        }
    }

    /// The strip is where a set is built, so it is where the way out of one
    /// has to be.
    #[test]
    fn the_strip_offers_a_way_out_of_a_set() {
        for list in [Row::on_the_strip(false), Row::on_the_strip(true)] {
            let verbs: Vec<Verb> = list.iter().flat_map(|row| row.verbs()).collect();

            assert!(verbs.contains(&Verb::PickNone));
        }
    }

    #[test]
    fn sending_photographs_somewhere_says_how_many() {
        assert_eq!(Verb::MoveTo.label(1), "Move this photograph…");
        assert_eq!(Verb::MoveTo.label(12), "Move 12 photographs…");
        assert_eq!(Verb::CopyTo.label(1), "Copy this photograph to…");
        assert_eq!(Verb::CopyTo.label(12), "Copy 12 photographs to…");
    }

    /// Comparing two fits means two clicks, not four.
    #[test]
    fn the_zoom_verbs_leave_the_menu_up() {
        assert!(!Verb::Fit.closes());
        assert!(!Verb::Fill.closes());
        assert!(Verb::Bin.closes());
        assert!(Verb::Open.closes());
        assert!(Verb::MirrorVertically.closes());
    }

    #[test]
    fn every_verb_says_what_it_does() {
        for row in Row::ON_A_PHOTOGRAPH.iter().chain(Row::ON_A_CELL) {
            assert!(!row.label(1).is_empty());

            for verb in row.verbs() {
                assert!(!verb.label(1).is_empty());
                assert!(!verb.hint().is_empty());
            }
        }
    }

    /// A verb behind the word is still reachable from the row that holds it,
    /// which is what the shadow check and this file's own tests walk.
    #[test]
    fn a_row_reaches_what_is_behind_it() {
        assert_eq!(
            Row::Verb(Verb::Bin).verbs().collect::<Vec<_>>(),
            vec![Verb::Bin]
        );
        assert_eq!(
            Row::Group("Turn", Verb::TURNS).verbs().collect::<Vec<_>>(),
            Verb::TURNS.to_vec()
        );
        assert!(Row::ON_A_PHOTOGRAPH
            .iter()
            .flat_map(|row| row.verbs())
            .any(|verb| verb == Verb::MirrorHorizontally));
    }

    /// The five that are turns, and nothing else, say what they turn by.
    #[test]
    fn only_the_turns_are_turns() {
        for verb in Verb::TURNS {
            assert!(verb.turn().is_some(), "{verb:?} is in TURNS");
        }

        for verb in [Verb::Fit, Verb::Bin, Verb::CopyPath, Verb::Open] {
            assert!(verb.turn().is_none());
        }
    }

    /// What each of them does to an upright photograph, which is the whole of
    /// the difference between them.
    #[test]
    fn each_turn_is_a_different_orientation() {
        let upright = Orientation::Normal;

        assert_eq!(
            upright.then(Verb::TurnRight.turn().unwrap()),
            Orientation::Rotate90Cw
        );
        assert_eq!(
            upright.then(Verb::TurnLeft.turn().unwrap()),
            Orientation::Rotate270Cw
        );
        assert_eq!(
            upright.then(Verb::TurnHalf.turn().unwrap()),
            Orientation::Rotate180
        );
        assert_eq!(
            upright.then(Verb::MirrorHorizontally.turn().unwrap()),
            Orientation::MirrorHorizontal
        );
        assert_eq!(
            upright.then(Verb::MirrorVertically.turn().unwrap()),
            Orientation::MirrorVertical
        );
    }

    /// Two quarters the same way are the half turn, and either mirror twice is
    /// nothing at all — which is the property that says these five compose
    /// with what is already there rather than replacing it.
    #[test]
    fn the_turns_compose() {
        let quarter = Verb::TurnRight.turn().unwrap();
        assert_eq!(
            Orientation::Normal.then(quarter).then(quarter),
            Verb::TurnHalf.turn().unwrap()
        );

        for verb in [Verb::MirrorHorizontally, Verb::MirrorVertically] {
            let mirror = verb.turn().unwrap();
            assert_eq!(
                Orientation::Normal.then(mirror).then(mirror),
                Orientation::Normal,
                "{verb:?} twice"
            );
        }

        // And a mirror on a photograph the camera had already turned is still
        // one of the eight, which is what keeps this out of the decoder.
        assert_eq!(
            Orientation::Rotate90Cw.then(Verb::MirrorHorizontally.turn().unwrap()),
            Orientation::MirrorHorizontalRotate90Cw
        );
    }

    /// Every verb in the program, so that one added without a line in
    /// [`Verb::key`] fails the build rather than drawing a row that names no
    /// key however the person bound it.
    const EVERY_VERB: &[Verb] = &[
        Verb::Open,
        Verb::Fit,
        Verb::ActualPixels,
        Verb::Fill,
        Verb::Compare,
        Verb::Keep,
        Verb::Reject,
        Verb::PickNone,
        Verb::TurnRight,
        Verb::TurnLeft,
        Verb::TurnHalf,
        Verb::MirrorHorizontally,
        Verb::MirrorVertically,
        Verb::Bin,
        Verb::MoveTo,
        Verb::CopyTo,
        Verb::PutBack,
        Verb::DeleteForGood,
        Verb::CopyPath,
        Verb::CopyPicture,
        Verb::ShowInFolder,
    ];

    /// The key a row names is a key the registry has. A renamed binding
    /// otherwise leaves the row naming nothing, silently, for ever.
    #[test]
    fn every_key_a_verb_names_is_one_the_registry_has() {
        for verb in EVERY_VERB {
            let path = verb.key();

            if path.is_empty() {
                continue;
            }

            assert!(
                crate::config::bindings::is_a_key(path),
                "{verb:?} names {path}, which is not a key"
            );
        }
    }

    /// And every verb a menu can draw is in the list above, so that the test
    /// before this one keeps covering all of them.
    #[test]
    fn every_verb_a_menu_draws_is_one_of_them() {
        let lists = [
            Row::ON_A_PHOTOGRAPH,
            Row::ON_A_PHOTOGRAPH_IN_THE_BIN,
            Row::ON_A_CELL,
            Row::ON_A_CELL_IN_THE_BIN,
            Row::ON_THE_STRIP,
            Row::ON_THE_STRIP_IN_THE_BIN,
        ];

        for row in lists.iter().flat_map(|list| list.iter()) {
            for verb in row.verbs() {
                assert!(EVERY_VERB.contains(&verb), "{verb:?}");
            }
        }
    }
}
