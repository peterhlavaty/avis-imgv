//! What a single pane carries when there are several of them.
//!
//! Side by side, the question stops being "what is this photograph" and
//! becomes "which of these". That question is answered one frame at a time and
//! the answer is nearly always keep or throw out — so each pane carries those
//! two, over the picture, where the eye already is. The keys have always been
//! able to mark, but they mark *the* photograph, and with four on screen the
//! first thing anybody asks is which one they would land on.
//!
//! Drawn in a layer above the central panel for the reason the comparison
//! banner is: the panel registers itself as one click-sensing widget covering
//! the whole of itself, after its contents, and egui hands a press to the last
//! such widget under the pointer. Anything inside the panel that wants a click
//! is a thing the panel swallows.

use eframe::egui::{self, Align2, Color32, FontId, Rect, Sense, Stroke, Vec2};
use eframe::epaint::pos2;

use crate::metadata::xmp::Flag;

/// One pane: which photograph, where it was drawn, and what it carries.
#[derive(Debug, Clone, Copy)]
pub struct Pane {
    /// The store position drawn in it.
    pub index: usize,
    /// Where it was drawn, which is what a press is tested against.
    pub rect: Rect,
    /// The flag the photograph carries, for the two icons to show.
    pub flag: Flag,
    /// Whether it is the pane the keys are about.
    pub focused: bool,
}

/// What a click on the panes was asking for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Make this store position the pane the keys are about.
    Focus(usize),
    /// Set this flag on this photograph, or take it off if it is already on.
    Flag(usize, Flag),
}

/// The two verbs a pane offers, in the order a cull uses them.
const OFFERED: [Flag; 2] = [Flag::Picked, Flag::Rejected];

/// How far up from the foot of the pane the icons sit, and how big they are.
const UP: f32 = 14.0;
const SIDE: f32 = 30.0;
const GAP: f32 = 6.0;

/// The ground the icons sit on, so a glyph over a white sky can still be read.
const PLATE: Color32 = Color32::from_rgba_premultiplied(14, 14, 14, 190);

/// What an icon is drawn in when the photograph does not carry that flag.
const UNSET: Color32 = Color32::from_rgb(168, 168, 168);

/// Draws the icons over every pane and answers with what was clicked.
///
/// Nothing is drawn for a single pane: there is only one photograph, the keys
/// are unambiguously about it, and the status bar already says what it
/// carries. The icons are the answer to a question that only several panes
/// ask.
pub fn show(ctx: &egui::Context, panel: Rect, panes: &[Pane]) -> Option<Asked> {
    if panes.len() < 2 {
        return None;
    }

    let mut asked = None;

    egui::Area::new(egui::Id::new("pane-controls"))
        .order(egui::Order::Middle)
        .fixed_pos(pos2(panel.left(), panel.top()))
        .show(ctx, |ui| {
            for pane in panes {
                if let Some(this) = one(ui, pane) {
                    asked = Some(this);
                }
            }
        });

    asked
}

/// One pane's icons. The pane itself is not made click-sensing here — a press
/// anywhere else on it is the panel's, and where it lands is worked out from
/// the rects this was given.
fn one(ui: &egui::Ui, pane: &Pane) -> Option<Asked> {
    if pane.rect.width() <= SIDE * 3.0 || pane.rect.height() <= SIDE * 3.0 {
        // Too small to put anything on top of without covering the very thing
        // being judged. Eight panes on a laptop reach this.
        return None;
    }

    let width = SIDE * OFFERED.len() as f32 + GAP * (OFFERED.len() as f32 + 1.0);
    let plate = Rect::from_center_size(
        pos2(pane.rect.center().x, pane.rect.bottom() - UP - SIDE / 2.0),
        Vec2::new(width, SIDE + GAP),
    );

    ui.painter().rect_filled(plate, 6.0, PLATE);

    let mut asked = None;

    for (at, flag) in OFFERED.iter().enumerate() {
        let centre = pos2(
            plate.left() + GAP + SIDE / 2.0 + at as f32 * (SIDE + GAP),
            plate.center().y,
        );
        let rect = Rect::from_center_size(centre, Vec2::splat(SIDE));

        if icon(ui, rect, *flag, pane.flag == *flag, pane.index) {
            asked = Some(Asked::Flag(pane.index, *flag));
        }
    }

    asked
}

/// One icon. Returns whether it was clicked.
fn icon(ui: &egui::Ui, rect: Rect, flag: Flag, carried: bool, index: usize) -> bool {
    let response = ui.interact(
        rect,
        ui.id().with(("pane flag", index, flag.name())),
        Sense::click(),
    );

    let colour = match flag.colour() {
        Some((r, g, b)) => Color32::from_rgb(r, g, b),
        None => UNSET,
    };

    // Filled when the photograph carries it, outlined when it does not: the
    // state has to be readable from across the window, and a glyph that only
    // changes colour is a glyph two people will read two ways.
    if carried {
        ui.painter().rect_filled(rect, 4.0, colour);
    } else if response.hovered() {
        ui.painter().rect_stroke(
            rect,
            4.0,
            Stroke::new(1.5_f32, colour),
            egui::StrokeKind::Inside,
        );
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        flag.glyph(),
        FontId::proportional(SIDE * 0.55),
        match carried {
            true => Color32::from_rgb(16, 16, 16),
            false => colour,
        },
    );

    response.on_hover_text(hint(flag, carried)).clicked()
}

/// What the icon says under the pointer.
///
/// It says what pressing it will do rather than what the photograph is,
/// because the drawing already says what the photograph is and the question a
/// hand on the mouse is asking is what happens next.
fn hint(flag: Flag, carried: bool) -> String {
    match (flag, carried) {
        (Flag::Picked, false) => "Keep this one".to_string(),
        (Flag::Picked, true) => "Kept. Click to take the mark off".to_string(),
        (Flag::Rejected, false) => "Throw this one out".to_string(),
        (Flag::Rejected, true) => "Rejected. Click to take the mark off".to_string(),
        (Flag::Unflagged, _) => String::new(),
    }
}

/// Which pane a press at `at` landed in.
///
/// The panes are a row of rectangles, so this is a search over at most eight;
/// doing it by arithmetic on the width would be quicker and would be wrong the
/// first time the layout stops being a plain row.
pub fn at(panes: &[Pane], at: egui::Pos2) -> Option<usize> {
    panes
        .iter()
        .find(|pane| pane.rect.contains(at))
        .map(|pane| pane.index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pane(index: usize, left: f32, right: f32) -> Pane {
        Pane {
            index,
            rect: Rect::from_min_max(pos2(left, 0.0), pos2(right, 500.0)),
            flag: Flag::Unflagged,
            focused: false,
        }
    }

    #[test]
    fn a_press_lands_in_the_pane_it_is_over() {
        let panes = [pane(7, 0.0, 100.0), pane(9, 100.0, 200.0)];

        assert_eq!(at(&panes, pos2(50.0, 250.0)), Some(7));
        assert_eq!(at(&panes, pos2(150.0, 250.0)), Some(9));
    }

    /// A press outside every pane belongs to nothing: the panel is wider than
    /// the pictures in it, and a click on the grey is not a click on a
    /// photograph.
    #[test]
    fn a_press_outside_the_panes_lands_nowhere() {
        let panes = [pane(7, 0.0, 100.0)];

        assert_eq!(at(&panes, pos2(150.0, 250.0)), None);
        assert_eq!(at(&panes, pos2(50.0, 900.0)), None);
    }

    #[test]
    fn nothing_lands_anywhere_when_there_are_no_panes() {
        assert_eq!(at(&[], pos2(50.0, 50.0)), None);
    }

    /// The hint says what the click will do, and says something different once
    /// the mark is on.
    #[test]
    fn an_icon_says_what_pressing_it_does() {
        assert_ne!(hint(Flag::Picked, false), hint(Flag::Picked, true));
        assert_ne!(hint(Flag::Rejected, false), hint(Flag::Rejected, true));
        assert!(hint(Flag::Rejected, true).contains("off"));
    }

    /// Keep and reject have to be told apart by colour as well as by glyph:
    /// four panes marked in the same grey are four panes nobody can read.
    #[test]
    fn the_two_verbs_are_different_colours() {
        assert_ne!(Flag::Picked.colour(), Flag::Rejected.colour());
        assert!(Flag::Picked.colour().is_some());
        assert!(Flag::Rejected.colour().is_some());
        assert!(Flag::Unflagged.colour().is_none());
    }
}
