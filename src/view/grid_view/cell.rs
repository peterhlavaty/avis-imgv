//! What a contact sheet cell says about its photograph.
//!
//! A grid of bare thumbnails is a browser; a grid that shows the marks is a
//! triage surface. Marks you cannot see while scanning a sheet are marks you
//! do not trust, and the whole point of putting stars and flags and labels on
//! a folder is being able to look at the folder and see them.

use eframe::egui::{self, Align2, Color32, FontId, Rect};
use eframe::epaint::{pos2, Stroke, Vec2};

use crate::metadata::xmp::Flag;
use crate::view::image_view::bottom_bar::Marks;

/// How much of the cell the marks strip takes, and how it is drawn.
pub const CAPTION_HEIGHT: f32 = 20.0;

/// What is drawn under each thumbnail.
///
/// Cycled with one key rather than settled in the configuration, because how
/// much a person wants to see changes with what they are doing: everything
/// while triaging, nothing while looking.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Badges {
    /// The picture alone.
    None,
    /// Stars, flag and colour label.
    #[default]
    Marks,
    /// Those and the file name.
    Full,
}

impl Badges {
    pub const ALL: &'static [Badges] = &[Badges::None, Badges::Marks, Badges::Full];

    pub fn next(self) -> Badges {
        match self {
            Badges::None => Badges::Marks,
            Badges::Marks => Badges::Full,
            Badges::Full => Badges::None,
        }
    }

    /// How tall the strip under the picture has to be.
    pub fn caption_height(self) -> f32 {
        match self {
            Badges::None => 0.0,
            Badges::Marks | Badges::Full => CAPTION_HEIGHT,
        }
    }

    fn shows_marks(self) -> bool {
        self != Badges::None
    }

    fn shows_name(self) -> bool {
        self == Badges::Full
    }
}

/// Colour of the border round the photograph the image view is on.
const CURRENT: Color32 = Color32::from_rgb(232, 232, 232);

/// Colour of the border round the one the keyboard is on.
const CURSOR: Color32 = Color32::from_rgb(126, 168, 224);

const STAR: &str = "★";
const REJECTED_TINT: Color32 = Color32::from_rgba_premultiplied(28, 6, 6, 150);

/// Draws the strip under a thumbnail.
///
/// `marks` is what the photograph carries; a cell the folder scan has not
/// reached yet passes `None` and draws nothing, because an unread photograph
/// showing nought stars is a lie.
pub fn caption(ui: &egui::Ui, strip: Rect, badges: Badges, marks: Option<&Marks>, name: &str) {
    if !badges.shows_marks() || strip.height() <= 0.0 {
        return;
    }

    let painter = ui.painter();
    let colour = ui.visuals().weak_text_color();
    let font = FontId::proportional((strip.height() * 0.6).min(13.0));
    let inset = 4.0;

    if let Some(marks) = marks {
        let mut left = strip.left() + inset;

        if let Some(label) = marks.label {
            let (r, g, b) = label.colour();
            let swatch =
                Rect::from_min_size(pos2(left, strip.center().y - 4.0), Vec2::new(8.0, 8.0));
            painter.rect_filled(swatch, 1.0, Color32::from_rgb(r, g, b));
            left += 12.0;
        }

        if marks.flag != Flag::Unflagged {
            let tint = match marks.flag {
                Flag::Rejected => Color32::from_rgb(219, 96, 96),
                _ => colour,
            };

            let drawn = painter.text(
                pos2(left, strip.center().y),
                Align2::LEFT_CENTER,
                marks.flag.glyph(),
                font.clone(),
                tint,
            );
            left = drawn.right() + 4.0;
        }

        if marks.stars > 0 {
            painter.text(
                pos2(left, strip.center().y),
                Align2::LEFT_CENTER,
                STAR.repeat(marks.stars as usize),
                font.clone(),
                colour,
            );
        }
    }

    if badges.shows_name() {
        painter.text(
            pos2(strip.right() - inset, strip.center().y),
            Align2::RIGHT_CENTER,
            elided(painter, name, strip.width() * 0.6, &font),
            font,
            colour,
        );
    }
}

/// Dims a rejected photograph, so a sheet of them reads at a glance.
pub fn dim_if_rejected(ui: &egui::Ui, picture: Rect, marks: Option<&Marks>) {
    if marks.is_some_and(|marks| marks.flag == Flag::Rejected) {
        ui.painter().rect_filled(picture, 0.0, REJECTED_TINT);
    }
}

/// Outlines the cell the image view is on, and the one the keyboard is on.
///
/// Two different things, and the sheet used to mark neither: opening the
/// gallery told you nothing about which photograph you had come from.
pub fn outline(ui: &egui::Ui, rect: Rect, is_current: bool, has_cursor: bool) {
    let (colour, width) = match (has_cursor, is_current) {
        (true, _) => (CURSOR, 2.0),
        (false, true) => (CURRENT, 1.5),
        (false, false) => return,
    };

    ui.painter().rect_stroke(
        rect.shrink(width / 2.0),
        0.0,
        Stroke::new(width, colour),
        egui::StrokeKind::Inside,
    );
}

/// A name cut to fit, with the *end* kept.
///
/// The other way round from the usual: a folder off a camera is a hundred
/// names sharing a prefix, and what tells two of them apart is the frame
/// number and the extension at the end.
fn elided(painter: &egui::Painter, name: &str, width: f32, font: &FontId) -> String {
    let measure = |text: &str| {
        painter
            .layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE)
            .rect
            .width()
    };

    if width <= 0.0 || measure(name) <= width {
        return name.to_string();
    }

    let mut kept: String = name.to_string();
    while !kept.is_empty() && measure(&format!("…{kept}")) > width {
        kept.remove(0);
    }

    format!("…{kept}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_badge_sets_cycle_back_round() {
        let mut badges = Badges::default();
        let mut seen = vec![badges];

        for _ in 0..Badges::ALL.len() {
            badges = badges.next();
            if !seen.contains(&badges) {
                seen.push(badges);
            }
        }

        assert_eq!(seen.len(), Badges::ALL.len());
        assert_eq!(badges, Badges::default());
    }

    #[test]
    fn only_the_sets_that_draw_something_reserve_room() {
        assert_eq!(Badges::None.caption_height(), 0.0);
        assert!(Badges::Marks.caption_height() > 0.0);
        assert!(Badges::Full.caption_height() > 0.0);
    }

    #[test]
    fn the_name_is_only_drawn_by_the_full_set() {
        assert!(!Badges::None.shows_name());
        assert!(!Badges::Marks.shows_name());
        assert!(Badges::Full.shows_name());

        assert!(!Badges::None.shows_marks());
        assert!(Badges::Marks.shows_marks());
    }
}
