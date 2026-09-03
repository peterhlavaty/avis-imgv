//! Saying that a comparison is up, and offering the way out of it.
//!
//! A comparison is the one thing this view does that a person can be in
//! without being able to see that they are in it. Four photographs side by
//! side look exactly like four photographs side by side: `nr_images_shown` set
//! to four is the ordinary view, and a pinned set of four is a state the
//! arrow keys behave differently in and that nothing on screen mentioned. The
//! only way out was a key, or a row in a menu on a figure in the status bar —
//! neither of which is where somebody looks when they are wondering what has
//! happened to their viewer.
//!
//! So the panel is outlined and named. The outline is the state made visible
//! at the size of the whole window, the name says which of the two
//! comparisons it is and how many photographs are in it, the hover says what
//! that means and what will change it, and the cross beside it is the way
//! back. A state you can see is a state you can leave.

use eframe::egui::{self, Align2, Color32, FontId, Rect, Sense, Stroke, Vec2};
use eframe::epaint::pos2;

/// What the outline and the plate are drawn in, before the configuration is
/// read, and what a value the configuration cannot spell falls back to.
///
/// Warm rather than blue: the blue is what is picked out, and a comparison of
/// two neighbours has nothing picked out in it at all — a border in the
/// selection's colour round a panel showing no selection would be a sentence
/// that is sometimes false. The same reasoning as the stack plate's.
pub const DEFAULT: Color32 = Color32::from_rgb(226, 186, 120);

/// What is being compared, for the banner to say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Banner {
    /// How many photographs are pinned.
    pub panes: usize,
    /// Whether they are the photographs picked out, which changes both the
    /// name and what the hover promises about it.
    pub from_selection: bool,
}

impl Banner {
    /// The name, short enough to sit in a corner over a photograph.
    pub fn name(self) -> String {
        match self.from_selection {
            true => format!("Comparing {} picked out", self.panes),
            false => format!("Comparing {}", self.panes),
        }
    }

    /// The sentence under the pointer, which is where the state is explained
    /// rather than merely announced.
    pub fn said(self) -> String {
        let panes = self.panes;

        match self.from_selection {
            true => format!(
                "The {panes} photographs you picked out, pinned side by side. Picking \
                 another out or putting one back changes what is shown, and putting \
                 them all back ends it. The cross goes back to the ordinary view."
            ),
            false => format!(
                "{panes} photographs pinned side by side. They stay as they are while \
                 you look between them; the arrow keys try a different photograph \
                 against the one in front. The cross goes back to the ordinary view."
            ),
        }
    }
}

/// What the banner was clicked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// Go back to the ordinary view.
    Stop,
    /// Open the page of settings the comparison is governed from.
    Settings,
}

/// How far in from the panel's top right corner the banner sits.
const INSET: f32 = 10.0;

/// The banner's own padding, and the side of the cross.
const PAD: f32 = 7.0;
const CROSS: f32 = 16.0;

/// The cross itself, which is the one the rest of the program already uses —
/// the rejected flag's, and the "take it off" buttons in the settings lists.
///
/// Not `✕`, U+2715, which this was written with: that one is in neither the
/// typeface the program ships nor the emoji font egui falls through to, so
/// what was drawn was an empty box. A glyph is only as available as the fonts
/// actually loaded, and the way to know is to look at it.
const CROSS_GLYPH: &str = "✖";

/// How thick the outline round the panel is.
///
/// Thicker than the two points that mark the leading pane, and a different
/// colour: one says which of these photographs the keys are about and the
/// other says that all of them together are a state.
const OUTLINE: f32 = 3.0;

/// Outlines the panel, which is the state made visible at the size of the
/// whole window.
///
/// The panel rather than the panes, because that is what the state is about —
/// outlining the photographs would say something narrower and wrong: they are
/// not pinned, the *view* is. Paint alone, drawn inside the panel; the plate
/// is a layer of its own, and [`banner`] says why.
pub fn outline(ui: &egui::Ui, panel: Rect, colour: Color32) {
    if panel.width() <= 0.0 || panel.height() <= 0.0 {
        return;
    }

    ui.painter().rect_stroke(
        panel.shrink(OUTLINE / 2.0),
        0.0,
        Stroke::new(OUTLINE, colour),
        egui::StrokeKind::Inside,
    );
}

/// Draws the name and the cross in the panel's top right corner, and answers
/// with what was clicked.
///
/// In a layer of its own rather than inside the panel, because the panel is
/// registered as one click-sensing widget covering the whole of itself *after*
/// its contents are drawn — that is how this view reads a drag over the
/// photograph — and egui hands a press to the last such widget registered
/// under it. A cross drawn inside the panel is therefore a cross the panel
/// swallows every press aimed at, which is what happened: the banner appeared,
/// said the right thing, and could not be clicked.
pub fn banner(ctx: &egui::Context, panel: Rect, banner: Banner, colour: Color32) -> Option<Asked> {
    if panel.width() <= 0.0 || panel.height() <= 0.0 {
        return None;
    }

    let mut asked = None;

    egui::Area::new(egui::Id::new("comparison-banner"))
        .order(egui::Order::Middle)
        .fixed_pos(pos2(panel.left(), panel.top()))
        .show(ctx, |ui| {
            asked = plate(ui, panel, banner, colour);
        });

    asked
}

/// The plate itself, once it has a layer to be drawn in.
fn plate(ui: &egui::Ui, panel: Rect, banner: Banner, colour: Color32) -> Option<Asked> {
    let name = banner.name();
    let font = FontId::proportional(13.0);
    let text = ui
        .painter()
        .layout_no_wrap(name.clone(), font.clone(), Color32::WHITE);

    let width = text.rect.width() + CROSS + PAD * 3.0;
    let height = text.rect.height().max(CROSS) + PAD * 1.5;
    let top_right = pos2(panel.right() - INSET, panel.top() + INSET);
    let plate = Rect::from_min_size(top_right - Vec2::new(width, 0.0), Vec2::new(width, height));

    let cross = Rect::from_center_size(
        pos2(plate.right() - PAD - CROSS / 2.0, plate.center().y),
        Vec2::splat(CROSS),
    );

    // The plate first and the cross over it. egui hands a press to the last
    // click-sensing widget registered at that point, so a plate registered
    // after the cross it contains would swallow every press aimed at the one
    // control the banner exists to offer.
    let plated = ui.interact(plate, ui.id().with("comparison banner"), Sense::click());
    let crossed = ui.interact(cross, ui.id().with("stop comparing"), Sense::click());
    let bright = crossed.hovered();

    // Over the photograph, so it needs a ground of its own to be read against
    // — a white sky and a white word are the same thing.
    ui.painter().rect_filled(plate, 4.0, plate_colour(colour));
    ui.painter().rect_stroke(
        plate,
        4.0,
        Stroke::new(1.0_f32, colour),
        egui::StrokeKind::Inside,
    );

    ui.painter().galley(
        pos2(
            plate.left() + PAD,
            plate.center().y - text.rect.height() / 2.0,
        ),
        text,
        Color32::WHITE,
    );

    if bright {
        ui.painter().rect_filled(cross, 3.0, colour);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    ui.painter().text(
        cross.center(),
        Align2::CENTER_CENTER,
        CROSS_GLYPH,
        FontId::proportional(CROSS * 0.8),
        match bright {
            true => Color32::from_rgb(16, 16, 16),
            false => Color32::WHITE,
        },
    );

    let mut asked = crossed
        .on_hover_text("Stop comparing and go back to the ordinary view")
        .clicked()
        .then_some(Asked::Stop);

    // The plate answers the second button like everything else drawn here, and
    // ends on the page the comparison is governed from.
    let chosen = crate::ui::surface::with_menu(
        ui,
        &plated,
        crate::ui::surface::Subject::of("Comparison", &banner.name()),
        &banner.said(),
        |ui| {
            let mut chosen = None;

            if crate::ui::keys::button(ui, "Stop comparing", "image_view.sc_compare").clicked() {
                chosen = Some(Asked::Stop);
                ui.close();
            }

            if crate::ui::surface::more_settings(ui, crate::config::registry::Page::ThePhotograph) {
                chosen = Some(Asked::Settings);
            }

            chosen
        },
    );

    if let Some(Some(chosen)) = chosen {
        asked = Some(chosen);
    }

    asked
}

/// The ground the name is drawn on: the colour, darkened and made mostly
/// opaque, so the word is legible over whatever is behind it without the plate
/// becoming a second bright thing competing with the photographs.
fn plate_colour(colour: Color32) -> Color32 {
    let quarter = |channel: u8| ((channel as u32 * 55) / (4 * 255)) as u8;

    Color32::from_rgba_premultiplied(
        quarter(colour.r()),
        quarter(colour.g()),
        quarter(colour.b()),
        215,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name says which of the two comparisons it is, because the way out
    /// of them is the same and what changes them is not.
    #[test]
    fn the_two_comparisons_are_named_apart() {
        let picked = Banner {
            panes: 4,
            from_selection: true,
        };
        let neighbours = Banner {
            panes: 2,
            from_selection: false,
        };

        assert_eq!(picked.name(), "Comparing 4 picked out");
        assert_eq!(neighbours.name(), "Comparing 2");
        assert_ne!(picked.said(), neighbours.said());
    }

    /// The count is in the name, so a set that was trimmed to the eight panes
    /// the view holds says eight rather than the forty that were asked for.
    #[test]
    fn the_name_counts_the_panes_rather_than_the_wish() {
        let banner = Banner {
            panes: 8,
            from_selection: true,
        };

        assert!(banner.name().contains('8'), "{}", banner.name());
    }

    /// Both sentences say what the cross does, because that is the question
    /// somebody hovering a state they did not mean to enter is asking.
    #[test]
    fn both_sentences_say_where_the_way_out_is() {
        for from_selection in [true, false] {
            let banner = Banner {
                panes: 3,
                from_selection,
            };

            assert!(banner.said().contains("cross"), "{}", banner.said());
        }
    }

    /// The plate has to be opaque enough to read a word on and dark enough not
    /// to be a second bright thing beside the photographs.
    #[test]
    fn the_plate_is_dark_and_nearly_opaque() {
        for colour in [
            Color32::WHITE,
            Color32::from_rgb(226, 186, 120),
            Color32::from_rgb(0, 0, 255),
        ] {
            let plate = plate_colour(colour);

            assert!(plate.a() > 200, "{colour:?} gave {plate:?}");
            for channel in [plate.r(), plate.g(), plate.b()] {
                assert!(channel <= plate.a() / 3, "{colour:?} gave {plate:?}");
            }
        }
    }
}
