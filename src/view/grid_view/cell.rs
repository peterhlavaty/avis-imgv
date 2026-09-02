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

/// How much of the cell the marks strip takes.
///
/// A fraction rather than a flat twenty points: at sixteen columns a flat
/// strip is proportionally enormous, and at one column the stars are a sliver
/// in a wall. The font inside it already scales this way and only its cap had
/// to move.
pub const CAPTION_FRACTION: f32 = 0.11;

/// The strip never goes below this, whatever the cell measures: a strip too
/// short to hold a star is worse than one slightly out of proportion.
pub const CAPTION_FLOOR: f32 = 14.0;

/// And never above this, so a contact sheet of one column is not half caption.
pub const CAPTION_CEILING: f32 = 34.0;

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

    /// The word the file holds.
    pub fn value(self) -> &'static str {
        match self {
            Badges::None => "none",
            Badges::Marks => "marks",
            Badges::Full => "full",
        }
    }

    /// What a stored word means. Anything unrecognised is the useful middle.
    pub fn of(value: &str) -> Badges {
        Badges::ALL
            .iter()
            .copied()
            .find(|it| it.value() == value)
            .unwrap_or(Badges::Marks)
    }

    pub fn next(self) -> Badges {
        match self {
            Badges::None => Badges::Marks,
            Badges::Marks => Badges::Full,
            Badges::Full => Badges::None,
        }
    }

    /// How tall the strip under the picture has to be, for a cell this wide.
    pub fn caption_height(self, cell: f32) -> f32 {
        match self {
            Badges::None => 0.0,
            Badges::Marks | Badges::Full => {
                (cell * CAPTION_FRACTION).clamp(CAPTION_FLOOR, CAPTION_CEILING)
            }
        }
    }

    fn shows_marks(self) -> bool {
        self != Badges::None
    }

    pub fn shows_name(self) -> bool {
        self == Badges::Full
    }
}

/// Colour of the border round the photograph the image view is on.
const CURRENT: Color32 = Color32::from_rgb(232, 232, 232);

/// Colour of the border round the one the keyboard is on.
const CURSOR: Color32 = Color32::from_rgb(126, 168, 224);

/// Wash over a cell that has been picked out, and the colour of its tick.
///
/// A wash rather than a border, because the borders are spoken for: one says
/// where the keyboard is and one says which photograph the other view is on,
/// and a selection has to be legible at the same time as both.
///
/// The colour itself is `grid_view.selection_colour`; this is what it falls
/// back to when the configuration holds something that is not a colour.
pub const SELECTED: Color32 = Color32::from_rgb(126, 168, 224);

/// How much of a picked-out cell's wash is the selection colour.
///
/// A quarter of it, and darkened first: the wash goes over the photograph and
/// has to leave it readable, which is the whole difference between marking a
/// frame and covering it.
const WASH: u8 = 90;

/// The wash a selection colour makes.
///
/// A third of the colour, so a light blue and a strong red both come out as
/// something the photograph can still be seen through.
pub fn wash(colour: Color32) -> Color32 {
    let third = |channel: u8| ((channel as u32 * WASH as u32) / (3 * 255)) as u8;

    Color32::from_rgba_premultiplied(
        third(colour.r()),
        third(colour.g()),
        third(colour.b()),
        WASH,
    )
}

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

/// Marks a cell that has been picked out, in `colour`.
///
/// A wash over the whole picture and a tick in the corner: the wash is what
/// makes a selection countable at a glance from across a sheet, and the tick
/// is what makes a single selected cell unmistakable when the wash could be
/// taken for a dark photograph.
///
/// The strip under the photograph draws the same two things for the same
/// reason, so this is where both of them are: one mark, one meaning, and the
/// tick shrinks to fit a cell a fifth the size.
pub fn picked(ui: &egui::Ui, picture: Rect, selected: bool, colour: Color32) {
    if !selected || picture.width() <= 0.0 {
        return;
    }

    let painter = ui.painter();
    painter.rect_filled(picture, 0.0, wash(colour));

    let side = (picture.width() * 0.18).clamp(10.0, 22.0);
    let badge = Rect::from_min_size(picture.min + Vec2::splat(3.0), Vec2::splat(side));

    painter.rect_filled(badge, 3.0, colour);
    painter.text(
        badge.center(),
        Align2::CENTER_CENTER,
        "✔",
        FontId::proportional(side * 0.7),
        Color32::from_rgb(16, 24, 36),
    );
}

/// The plate a closed stack wears, and the bar down the side of an open one.
///
/// Warm rather than blue: the blue is the selection and the cursor, and a
/// stack is a fact about the folder rather than something the user has picked.
const STACK: Color32 = Color32::from_rgb(226, 186, 120);
const STACK_PLATE: Color32 = Color32::from_rgba_premultiplied(30, 24, 14, 190);

/// Says that a cell stands for a run of frames rather than for one.
///
/// A closed stack gets a plate in the corner with its count and a glyph for
/// what kind of run it is — the number is the important half, because "this is
/// one of seventeen" is the thing that changes what you do next. An open stack
/// gets a bar down its left edge instead, so a run that has been opened still
/// reads as one block rather than as loose frames that happen to be adjacent.
pub fn stack(ui: &egui::Ui, picture: Rect, glyph: &str, frames: usize, collapsed: bool) {
    if picture.width() <= 0.0 {
        return;
    }

    let painter = ui.painter();

    if !collapsed {
        let bar = Rect::from_min_max(picture.min, pos2(picture.left() + 3.0, picture.bottom()));

        painter.rect_filled(bar, 0.0, STACK);
        return;
    }

    let height = (picture.height() * 0.16).clamp(14.0, 24.0);
    let text = format!("{glyph} {frames}");
    let font = FontId::proportional(height * 0.62);

    // Measured rather than guessed: the count is one character for a bracket
    // and three for a timelapse, and a plate cut to fit the first hides the
    // second.
    let galley = painter.layout_no_wrap(text.clone(), font.clone(), STACK);
    let plate = Rect::from_min_size(
        pos2(
            picture.right() - galley.size().x - 12.0,
            picture.top() + 3.0,
        ),
        Vec2::new(galley.size().x + 9.0, height),
    );

    painter.rect_filled(plate, 3.0, STACK_PLATE);
    painter.rect_stroke(
        plate,
        3.0,
        Stroke::new(1.0_f32, STACK),
        egui::StrokeKind::Inside,
    );
    painter.text(plate.center(), Align2::CENTER_CENTER, text, font, STACK);
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

    /// The wash goes over the photograph, so whatever colour is chosen it has
    /// to leave the photograph visible through it.
    #[test]
    fn a_wash_is_faint_whatever_colour_it_is_made_of() {
        for colour in [
            SELECTED,
            Color32::WHITE,
            Color32::from_rgb(255, 0, 0),
            Color32::BLACK,
        ] {
            let washed = wash(colour);

            assert_eq!(washed.a(), WASH, "{colour:?}");
            // Premultiplied, so no channel may be brighter than the alpha or
            // the colour is not one egui can draw.
            for channel in [washed.r(), washed.g(), washed.b()] {
                assert!(channel <= washed.a(), "{colour:?} gave {washed:?}");
            }
        }
    }

    /// The default is still the blue the sheet has always used, so a
    /// configuration that says nothing looks as it did.
    #[test]
    fn the_default_selection_colour_is_the_one_the_sheet_had() {
        let configured =
            crate::ui::theme::colour(&crate::config::default_selection_colour(), Color32::BLACK);

        assert_eq!(configured, SELECTED);
    }

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
        assert_eq!(Badges::None.caption_height(160.0), 0.0);
        assert!(Badges::Marks.caption_height(160.0) > 0.0);
        assert!(Badges::Full.caption_height(160.0) > 0.0);

        // A narrow cell keeps a strip tall enough to hold a star, and a wide
        // one does not turn into half caption.
        assert_eq!(Badges::Marks.caption_height(40.0), CAPTION_FLOOR);
        assert_eq!(Badges::Marks.caption_height(2000.0), CAPTION_CEILING);
        assert!(
            Badges::Marks.caption_height(400.0) > Badges::Marks.caption_height(120.0),
            "a bigger cell gets a bigger strip"
        );
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
