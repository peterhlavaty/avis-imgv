//! What the second button offers on a photograph and on a cell.
//!
//! One list of rows, drawn the same way in both views, with whatever the user
//! configured appended under a separator in their own order. A fresh install
//! used to answer a right-click with nothing at all: the default entry list is
//! empty and the menu returns before registering anything when it is.
//!
//! One level, with one exception. A submenu is placed against the right edge of
//! the row that opens it and folds back to the left when the screen has no room
//! — and every panel in this program sits against an edge, so a second level is
//! worth the risk only where the rows behind it are variations on one another
//! and would otherwise take five of the twelve rows a menu may carry. The turns
//! are that: five ways of saying one verb, behind the word. The submenu is kept
//! narrower than its parent so that there is somewhere for it to go.

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
    /// Pin this photograph and its neighbours side by side.
    Compare,
    /// The turns, written to the sidecar rather than to the file. All five
    /// live behind one word; [`Verb::TURNS`] is what that word opens.
    TurnRight,
    TurnLeft,
    TurnHalf,
    MirrorHorizontally,
    MirrorVertically,
    /// Send it to the platform's bin.
    Bin,
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
            Verb::Compare => "Compare".to_string(),
            Verb::TurnRight => "Clockwise".to_string(),
            Verb::TurnLeft => "Anticlockwise".to_string(),
            Verb::TurnHalf => "Upside down".to_string(),
            Verb::MirrorHorizontally => "Mirror left to right".to_string(),
            Verb::MirrorVertically => "Mirror top to bottom".to_string(),
            Verb::Bin => format!(
                "Move {} to the bin",
                these("this photograph", "photographs")
            ),
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
            Verb::Fit => "The whole photograph, as large as the window allows",
            Verb::ActualPixels => "One screen pixel per pixel of the photograph",
            Verb::Fill => "Fill the window, cropping whichever side is longer",
            Verb::Compare => "Pin this photograph and the ones beside it side by side",
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
                "To the platform's bin, which does not reach a memory card \
                          or a network share"
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
}

/// A row of the menu: a verb, or a word that opens a few of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Verb(Verb),
    /// The word, and the verbs behind it. The only second level in the
    /// program; the note at the top of this file says why there is one.
    Group(&'static str, &'static [Verb]),
}

impl Row {
    /// The rows a photograph carries, in the order they are drawn.
    ///
    /// Verbs first and most used first, then copy and show because the object
    /// is a file on disk.
    pub const ON_A_PHOTOGRAPH: &'static [Row] = &[
        Row::Verb(Verb::Fit),
        Row::Verb(Verb::ActualPixels),
        Row::Verb(Verb::Fill),
        Row::Verb(Verb::Compare),
        Row::Group("Turn", Verb::TURNS),
        Row::Verb(Verb::Bin),
        Row::Verb(Verb::CopyPath),
        Row::Verb(Verb::CopyPicture),
        Row::Verb(Verb::ShowInFolder),
    ];

    /// The rows a cell carries. `Open` leads, because that is what a cell is
    /// for, and the zoom verbs are not about anything the sheet draws.
    pub const ON_A_CELL: &'static [Row] = &[
        Row::Verb(Verb::Open),
        Row::Verb(Verb::Compare),
        Row::Group("Turn", Verb::TURNS),
        Row::Verb(Verb::Bin),
        Row::Verb(Verb::CopyPath),
        Row::Verb(Verb::CopyPicture),
        Row::Verb(Verb::ShowInFolder),
    ];

    /// Every verb this row can reach, whether or not it is behind a word.
    pub fn verbs(self) -> impl Iterator<Item = Verb> {
        let (alone, behind): (Option<Verb>, &'static [Verb]) = match self {
            Row::Verb(verb) => (Some(verb), &[]),
            Row::Group(_, verbs) => (None, verbs),
        };

        alone.into_iter().chain(behind.iter().copied())
    }

    /// What the row says, for `count` photographs.
    pub fn label(self, count: usize) -> String {
        match self {
            Row::Verb(verb) => verb.label(count),
            Row::Group(word, _) if count == 1 => word.to_string(),
            Row::Group(word, _) => format!("{word} {count} photographs"),
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
    let clicked = ui
        .button(verb.label(count))
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
    /// Nine on the photograph and one settings row is ten, which leaves two.
    /// The five turns take one of those rows between them, which is what the
    /// second level bought.
    #[test]
    fn no_built_in_list_uses_up_the_menu() {
        // The list, plus the settings row that closes every menu.
        assert!(Row::ON_A_PHOTOGRAPH.len() < 11);
        assert!(Row::ON_A_CELL.len() < 11);
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
}
