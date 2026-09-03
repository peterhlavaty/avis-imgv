//! Drawing the marking over the photograph, and the menu it answers with.
//!
//! A `Ui` of its own, made after the central panel has closed. Two things fall
//! out of that and both are wanted: what it paints goes over the photograph
//! rather than under it, and the menu hangs off a response of its own — hung
//! off the panel's, it would share a popup with the photograph's own menu and
//! one of the two would never open.

use eframe::egui::{self, Rect, Sense, UiBuilder};
use eframe::epaint::{Color32, Pos2, Stroke, Vec2};

use crate::config::registry::Page;
use crate::ui::surface::{self, Subject};

use super::REACH;

/// What the menu on the marking was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    /// Magnify until it fills the panel.
    ZoomToIt,
    /// Its pixels, decoded at full size, on the clipboard.
    Copy,
    Clear,
    /// Arm the keyboard editor on the row that binds this.
    BindKey(&'static str),
    /// Open the settings window on the page that governs it.
    Settings(&'static str),
}

/// How wide the grab marks are drawn, in points.
const KNOB: f32 = 7.0;

/// Below this the marking is too small to draw grab marks inside without them
/// meeting in the middle.
const ROOM: f32 = KNOB * 4.0;

/// Paints the marking and answers the second button.
///
/// `picture` is where the photograph itself is, which is what the dimming
/// covers and what everything is clipped to; `dim` is how far the rest of it
/// is darkened, nought for not at all.
pub fn show(ctx: &egui::Context, on_screen: Rect, picture: Rect, dim: u8) -> Option<Chosen> {
    // Two ids, not one. A `Ui` registers *itself* as a widget under the id it
    // is given and re-registers it with its own (empty) rectangle when it is
    // dropped, so sharing the id with the surface inside it leaves the surface
    // with no rectangle to be hovered over and a menu that never opens.
    let mut ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("the marked area"),
        UiBuilder::new().max_rect(picture),
    );

    // The grab marks sit a little outside the marking, and the marking can be
    // flush against the edge of the photograph.
    ui.set_clip_rect(picture.expand(KNOB));
    paint(&ui, on_screen, picture, dim);

    let response = ui.interact(on_screen, egui::Id::new("marked area"), Sense::click());
    let mut chosen = None;

    surface::named_menu(
        &ui,
        &response,
        "marked area",
        Subject::the("The marked area"),
        |ui| {
            ui.set_max_width(surface::WIDEST);

            if crate::ui::keys::button(ui, "Zoom to it", "image_view.sc_zoom_to_area")
                .on_hover_text("Magnifies until the marked area fills the panel")
                .clicked()
            {
                chosen = Some(Chosen::ZoomToIt);
                ui.close();
            }

            if crate::ui::keys::button(ui, "Copy the marked area", "fixed.copy_area")
                .on_hover_text("The pixels inside it, decoded at full size and turned upright")
                .clicked()
            {
                chosen = Some(Chosen::Copy);
                ui.close();
            }

            if crate::ui::keys::button(ui, "Clear the marking", "fixed.escape")
                .on_hover_text("The same as clicking outside it, or Escape")
                .clicked()
            {
                chosen = Some(Chosen::Clear);
                ui.close();
            }

            if surface::bind_a_key(ui, "zooming to the marked area") {
                chosen = Some(Chosen::BindKey("image_view.sc_zoom_to_area"));
                ui.close();
            }

            if surface::more_settings(ui, Page::ThePhotograph) {
                chosen = Some(Chosen::Settings("image_view.marked_area_dim"));
                ui.close();
            }
        },
    );

    chosen
}

fn paint(ui: &egui::Ui, area: Rect, picture: Rect, dim: u8) {
    let painter = ui.painter();

    if dim > 0 {
        let shade = Color32::from_black_alpha(dim);

        for rest in around(area, picture) {
            painter.rect_filled(rest, 0.0, shade);
        }
    }

    // Two lines rather than one: a white one is lost against a bright sky and
    // a dark one is lost in a shadow, and most photographs have both.
    painter.rect_stroke(
        area,
        0.0,
        Stroke::new(1.0_f32, Color32::from_black_alpha(160)),
        egui::StrokeKind::Outside,
    );
    painter.rect_stroke(
        area,
        0.0,
        Stroke::new(1.0_f32, Color32::WHITE),
        egui::StrokeKind::Inside,
    );

    if area.width() < ROOM || area.height() < ROOM {
        return;
    }

    for at in knobs(area) {
        let knob = Rect::from_center_size(at, Vec2::splat(KNOB));

        painter.rect_filled(knob, 1.0, Color32::WHITE);
        painter.rect_stroke(
            knob,
            1.0,
            Stroke::new(1.0_f32, Color32::from_black_alpha(160)),
            egui::StrokeKind::Outside,
        );
    }
}

/// The four rectangles of the photograph that the marking leaves out.
///
/// Four rather than a hole in one, because a painter fills rectangles and the
/// alternative is a mesh: above, below, and the two beside it, which between
/// them cover everything the marking does not and nothing it does.
fn around(area: Rect, picture: Rect) -> [Rect; 4] {
    let area = area.intersect(picture);

    [
        Rect::from_min_max(
            picture.left_top(),
            Pos2::new(picture.right(), area.top().max(picture.top())),
        ),
        Rect::from_min_max(
            Pos2::new(picture.left(), area.bottom().min(picture.bottom())),
            picture.right_bottom(),
        ),
        Rect::from_min_max(
            Pos2::new(picture.left(), area.top()),
            Pos2::new(area.left(), area.bottom()),
        ),
        Rect::from_min_max(
            Pos2::new(area.right(), area.top()),
            Pos2::new(picture.right(), area.bottom()),
        ),
    ]
}

/// Where the grab marks go: the four corners and the middle of the four sides.
fn knobs(area: Rect) -> [Pos2; 8] {
    let centre = area.center();

    [
        area.left_top(),
        Pos2::new(centre.x, area.top()),
        area.right_top(),
        Pos2::new(area.right(), centre.y),
        area.right_bottom(),
        Pos2::new(centre.x, area.bottom()),
        area.left_bottom(),
        Pos2::new(area.left(), centre.y),
    ]
}

/// Whether the pointer is over the marking, and so whether the menu it opens
/// is this one rather than the photograph's.
///
/// The same reach the grips use, so the sides of the marking belong to it.
pub fn covers(on_screen: Rect, pointer: Pos2) -> bool {
    on_screen.expand(REACH).contains(pointer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture() -> Rect {
        Rect::from_min_max(Pos2::new(0.0, 100.0), Pos2::new(800.0, 500.0))
    }

    /// Every point of the photograph is either inside the marking or in one of
    /// the four pieces drawn around it, and no point is in both.
    #[test]
    fn the_four_pieces_cover_everything_the_marking_does_not() {
        let area = Rect::from_min_max(Pos2::new(200.0, 200.0), Pos2::new(600.0, 400.0));
        let rest = around(area, picture());

        for x in [10.0, 199.0, 300.0, 601.0, 790.0] {
            for y in [110.0, 199.0, 300.0, 401.0, 490.0] {
                let at = Pos2::new(x, y);
                let covered = rest.iter().filter(|piece| piece.contains(at)).count();

                if area.contains(at) {
                    assert_eq!(covered, 0, "{at:?} is inside the marking");
                } else {
                    assert_eq!(covered, 1, "{at:?} is covered {covered} times");
                }
            }
        }
    }

    /// A marking that fills the photograph leaves nothing to dim, and none of
    /// the four is turned inside out saying so.
    #[test]
    fn a_marking_over_the_whole_photograph_leaves_nothing_around_it() {
        for piece in around(picture(), picture()) {
            assert!(
                piece.width() <= 0.001 || piece.height() <= 0.001,
                "{piece:?}"
            );
        }
    }

    /// A marking on a photograph that has since been zoomed can be drawn off
    /// the side of it; the pieces still stay on the photograph.
    #[test]
    fn the_pieces_stay_on_the_photograph() {
        let off = Rect::from_min_max(Pos2::new(-400.0, -50.0), Pos2::new(300.0, 300.0));

        for piece in around(off, picture()) {
            assert!(
                picture().contains_rect(piece.intersect(picture())),
                "{piece:?}"
            );
            assert!(piece.top() >= picture().top() - 0.001, "{piece:?}");
        }
    }

    #[test]
    fn the_grab_marks_are_the_corners_and_the_middles() {
        let area = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(300.0, 200.0));
        let marks = knobs(area);

        assert!(marks.contains(&area.left_top()));
        assert!(marks.contains(&area.right_bottom()));
        assert!(marks.contains(&Pos2::new(200.0, 100.0)));
        assert!(marks.contains(&Pos2::new(100.0, 150.0)));
    }

    /// The sides of a marking belong to it, so the second button on one opens
    /// its menu rather than the photograph's.
    #[test]
    fn the_marking_covers_its_own_sides() {
        let area = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(300.0, 200.0));

        assert!(covers(area, Pos2::new(200.0, 150.0)));
        assert!(covers(area, Pos2::new(97.0, 150.0)));
        assert!(!covers(area, Pos2::new(50.0, 150.0)));
    }
}
