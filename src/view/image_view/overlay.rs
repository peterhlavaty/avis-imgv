//! What the photograph says about itself, over the photograph.
//!
//! The status bar has the same information and is in the wrong place for it:
//! it is at the bottom of the window, in the chrome, and a viewer running
//! fullscreen for a slideshow or a client review has no chrome at all. Every
//! program a photographer compares this one to can put the exposure over the
//! frame, because that is where the eye already is.
//!
//! Drawn from the same template grammar as the status bar and the bulk rename,
//! so what it says is the user's own sentence rather than a fixed list — and
//! anything a photograph cannot answer takes its separator with it instead of
//! leaving a row of bullets.
//!
//! Over the picture, not over the panel: it follows the drawn rectangle, so a
//! letterboxed photograph gets its caption on the photograph rather than on
//! the grey beside it.

use eframe::egui::{self, Align2, Color32, FontId, Rect};
use eframe::epaint::Vec2;

/// Where on the photograph the overlay sits.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    /// Not drawn at all.
    #[default]
    Off,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const ALL: &'static [Corner] = &[
        Corner::Off,
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];

    /// The word the file holds for this corner.
    ///
    /// The registry is keyed on what a forum answer quotes, which is what the
    /// document says rather than what the control says.
    pub fn value(self) -> &'static str {
        match self {
            Corner::Off => "off",
            Corner::TopLeft => "top_left",
            Corner::TopRight => "top_right",
            Corner::BottomLeft => "bottom_left",
            Corner::BottomRight => "bottom_right",
        }
    }

    /// The corner that word names, if it names one.
    pub fn of(value: &str) -> Option<Corner> {
        Corner::ALL
            .iter()
            .copied()
            .find(|corner| corner.value() == value)
    }

    /// The next corner round, for the key that cycles it.
    ///
    /// Through the corners and then off, so one key both moves it out of the
    /// way of whatever it is covering and turns it off entirely.
    pub fn next(self) -> Corner {
        match self {
            Corner::Off => Corner::TopLeft,
            Corner::TopLeft => Corner::TopRight,
            Corner::TopRight => Corner::BottomRight,
            Corner::BottomRight => Corner::BottomLeft,
            Corner::BottomLeft => Corner::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Corner::Off => "Off",
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }

    /// How the text is anchored, and which way the inset runs.
    fn anchoring(self) -> Option<(Align2, Vec2)> {
        let anchor = match self {
            Corner::Off => return None,
            Corner::TopLeft => (Align2::LEFT_TOP, Vec2::new(1.0, 1.0)),
            Corner::TopRight => (Align2::RIGHT_TOP, Vec2::new(-1.0, 1.0)),
            Corner::BottomLeft => (Align2::LEFT_BOTTOM, Vec2::new(1.0, -1.0)),
            Corner::BottomRight => (Align2::RIGHT_BOTTOM, Vec2::new(-1.0, -1.0)),
        };

        Some(anchor)
    }
}

/// How far in from the edge of the photograph the text sits.
const INSET: f32 = 14.0;

/// The plate behind it, so light text stays readable over a bright frame.
const PLATE: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 150);
const TEXT: Color32 = Color32::from_rgb(238, 238, 238);

/// Draws `lines` in `corner` of `picture`.
///
/// Nothing at all when there is nothing to say, rather than an empty plate:
/// a template that resolves to nothing on this photograph should leave the
/// photograph alone.
pub fn show(ui: &egui::Ui, picture: Rect, corner: Corner, lines: &str, size: f32) {
    let Some((anchor, direction)) = corner.anchoring() else {
        return;
    };

    let lines: Vec<&str> = lines
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.is_empty() || picture.width() <= 0.0 {
        return;
    }

    let painter = ui.painter().with_clip_rect(picture);
    let font = FontId::proportional(size.max(8.0));

    // Laid out first so the plate can be drawn behind it: a caption over a
    // bright sky is unreadable without one, and a plate sized to the text
    // rather than to the corner does not cover more of the picture than it
    // has to.
    let laid: Vec<_> = lines
        .iter()
        .map(|line| painter.layout_no_wrap((*line).to_string(), font.clone(), TEXT))
        .collect();

    let width = laid
        .iter()
        .map(|line| line.rect.width())
        .fold(0.0, f32::max);
    let height: f32 = laid.iter().map(|line| line.rect.height()).sum();

    let at = anchor.pos_in_rect(&picture.shrink(INSET));
    let block = anchor.align_size_within_rect(
        Vec2::new(width, height),
        Rect::from_min_size(at, Vec2::ZERO),
    );

    painter.rect_filled(block.expand(6.0), 3.0, PLATE);

    // Always downwards from the top of the block, whichever corner it is
    // anchored in: a caption reads top to bottom.
    let mut y = block.top();
    for line in laid {
        let line_height = line.rect.height();
        let x = match direction.x > 0.0 {
            true => block.left(),
            false => block.right() - line.rect.width(),
        };

        painter.galley(eframe::epaint::pos2(x, y), line, TEXT);
        y += line_height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_corner_has_a_name() {
        for corner in Corner::ALL {
            assert!(!corner.label().is_empty(), "{corner:?}");
        }

        assert_eq!(Corner::ALL.len(), 5);
    }

    #[test]
    fn off_is_the_default_and_draws_nothing() {
        assert_eq!(Corner::default(), Corner::Off);
        assert!(Corner::Off.anchoring().is_none());
    }

    /// Every corner that draws has an anchoring, and the inset runs inwards
    /// from that corner rather than off the picture.
    #[test]
    fn each_corner_pulls_towards_the_middle() {
        for corner in Corner::ALL.iter().filter(|c| **c != Corner::Off) {
            let (_, direction) = corner.anchoring().expect("it draws");

            let pulls_right = matches!(corner, Corner::TopLeft | Corner::BottomLeft);
            let pulls_down = matches!(corner, Corner::TopLeft | Corner::TopRight);

            assert_eq!(direction.x > 0.0, pulls_right, "{corner:?}");
            assert_eq!(direction.y > 0.0, pulls_down, "{corner:?}");
        }
    }

    /// One key reaches every corner and then turns it off.
    #[test]
    fn the_corners_cycle_through_all_of_them() {
        let mut corner = Corner::default();
        let mut seen = vec![corner];

        for _ in 0..Corner::ALL.len() {
            corner = corner.next();
            if !seen.contains(&corner) {
                seen.push(corner);
            }
        }

        assert_eq!(seen.len(), Corner::ALL.len());
        assert_eq!(corner, Corner::default(), "it comes back round");
    }

    #[test]
    fn the_names_round_trip_through_the_configuration() {
        for corner in Corner::ALL {
            let json = serde_json::to_string(corner).unwrap();
            let read: Corner = serde_json::from_str(&json).unwrap();

            assert_eq!(read, *corner);
        }

        // And the spelling is the one a person would write.
        assert_eq!(
            serde_json::to_string(&Corner::TopLeft).unwrap(),
            "\"top_left\""
        );
    }
}
