//! What the second button offers on a photograph and on a cell.
//!
//! One list of verbs, drawn the same way in both views, with whatever the user
//! configured appended under a separator in their own order. A fresh install
//! used to answer a right-click with nothing at all: the default entry list is
//! empty and the menu returns before registering anything when it is.
//!
//! Flat, one level, no submenus. egui places a submenu with its right edge
//! against the screen edge where long text can cover its parent, and every
//! panel in this program sits against an edge.

use eframe::egui;

use crate::config::ContextMenuEntry;

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
    /// Send it to the platform's bin.
    Bin,
    CopyPath,
    /// The pixels themselves, on the clipboard.
    CopyPicture,
    ShowInFolder,
}

impl Verb {
    /// The rows a photograph carries, in the order they are drawn.
    ///
    /// Verbs first and most used first, then copy and show because the object
    /// is a file on disk. Eight rows, inside the twelve a menu may carry.
    pub const ON_A_PHOTOGRAPH: &'static [Verb] = &[
        Verb::Fit,
        Verb::ActualPixels,
        Verb::Fill,
        Verb::Compare,
        Verb::Bin,
        Verb::CopyPath,
        Verb::CopyPicture,
        Verb::ShowInFolder,
    ];

    /// The rows a cell carries. `Open` leads, because that is what a cell is
    /// for, and the zoom verbs are not about anything the sheet draws.
    pub const ON_A_CELL: &'static [Verb] = &[
        Verb::Open,
        Verb::Compare,
        Verb::Bin,
        Verb::CopyPath,
        Verb::CopyPicture,
        Verb::ShowInFolder,
    ];

    /// What the row says, for `count` photographs.
    ///
    /// The count goes in the label rather than being left to be guessed:
    /// "Move 24 photographs to the bin" is a different sentence from "Move 1
    /// photograph to the bin", and the second button is where somebody finds
    /// out how big their selection got.
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
    pub fn hint(self) -> &'static str {
        match self {
            Verb::Open => "Show this photograph on its own",
            Verb::Fit => "The whole photograph, as large as the window allows",
            Verb::ActualPixels => "One screen pixel per pixel of the photograph",
            Verb::Fill => "Fill the window, cropping whichever side is longer",
            Verb::Compare => "Pin this photograph and the ones beside it side by side",
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

/// What a menu was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    Verb(Verb),
    /// The user's own entry at this position in the configuration.
    Entry(usize),
}

/// Draws the built-in verbs, then whatever the user configured.
///
/// The user's entries are appended in their own order under a separator, and
/// are never reordered, renamed or removed: they are the one part of this menu
/// somebody has already decided about.
pub fn rows(
    ui: &mut egui::Ui,
    verbs: &[Verb],
    entries: &[ContextMenuEntry],
    count: usize,
) -> Option<Chosen> {
    let mut chosen = None;

    ui.set_max_width(320.);

    for verb in verbs {
        if ui
            .button(verb.label(count))
            .on_hover_text(verb.hint())
            .clicked()
        {
            chosen = Some(Chosen::Verb(*verb));
            if verb.closes() {
                ui.close();
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

    /// Twelve rows including the last is the ceiling, and the user's own
    /// entries are appended to whatever is here.
    #[test]
    fn no_built_in_list_uses_up_the_menu() {
        assert!(Verb::ON_A_PHOTOGRAPH.len() <= 8);
        assert!(Verb::ON_A_CELL.len() <= 8);
    }

    /// Comparing two fits means two clicks, not four.
    #[test]
    fn the_zoom_verbs_leave_the_menu_up() {
        assert!(!Verb::Fit.closes());
        assert!(!Verb::Fill.closes());
        assert!(Verb::Bin.closes());
        assert!(Verb::Open.closes());
    }

    #[test]
    fn every_verb_says_what_it_does() {
        for verb in Verb::ON_A_PHOTOGRAPH.iter().chain(Verb::ON_A_CELL) {
            assert!(!verb.label(1).is_empty());
            assert!(!verb.hint().is_empty());
        }
    }
}
