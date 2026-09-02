//! What a photograph is drawn at on the frame it first appears.
//!
//! Fitted, filled, or one for one against the screen's own pixels. The choice
//! divides two ways of working — judging a composition wants the whole of the
//! picture, and judging focus wants a hundred per cent of it and nothing else
//! — so it is a setting rather than a number chosen here on somebody's behalf.
//!
//! It cannot be decided anywhere but the canvas. The fitted size is a function
//! of the window and of whichever panels are open in it, so what a hundred per
//! cent is worth as a magnification is not known until the frame is being
//! measured. This says what to aim for; `canvas::draw` applies it, once per
//! photograph.

use eframe::epaint::Vec2;

use super::canvas;
use super::zoom;

/// What a newly shown photograph is drawn at.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Opening {
    /// The whole of it, fitted to the panel.
    #[default]
    Fit,
    /// Filling the panel, cropping whichever side is longer.
    Fill,
    /// One screen pixel to one of the photograph's own.
    Actual,
}

impl Opening {
    pub const ALL: &'static [Opening] = &[Opening::Fit, Opening::Fill, Opening::Actual];

    /// The word the file holds.
    ///
    /// The registry is keyed on what a forum answer quotes, which is what the
    /// document says rather than what the control says.
    pub fn value(self) -> &'static str {
        match self {
            Opening::Fit => "fit",
            Opening::Fill => "fill",
            Opening::Actual => "actual",
        }
    }

    /// The opening that word names, if it names one.
    pub fn of(value: &str) -> Option<Opening> {
        Opening::ALL
            .iter()
            .copied()
            .find(|opening| opening.value() == value)
    }

    pub fn label(self) -> &'static str {
        match self {
            Opening::Fit => "Fitted to the window",
            Opening::Fill => "Filling the window",
            Opening::Actual => "Its own size",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Opening::Fit => "The whole photograph, as large as the window will take it.",
            Opening::Fill => {
                "Covers the window, cropping whichever side is longer. Nothing is lost \
                 — the rest is a pan away."
            }
            Opening::Actual => {
                "A hundred per cent: one screen pixel to one of the photograph's own, \
                 which is the magnification focus is judged at."
            }
        }
    }

    /// The word the status bar wears while this is what a photograph opens at.
    ///
    /// Fitting says nothing, being what the viewer has always done and what
    /// most people mean by opening a photograph. The bar is a summary of what
    /// is unusual about the moment, not a list of every setting.
    pub fn word(self) -> Option<&'static str> {
        match self {
            Opening::Fit => None,
            Opening::Fill => Some("Filling"),
            Opening::Actual => Some("100%"),
        }
    }

    /// The next one round, for the key that cycles it.
    pub fn next(self) -> Opening {
        match self {
            Opening::Fit => Opening::Fill,
            Opening::Fill => Opening::Actual,
            Opening::Actual => Opening::Fit,
        }
    }

    /// The magnification a photograph opens at, against the frame the canvas
    /// has just measured.
    ///
    /// `image_size` is the photograph's own pixels and `fit_size` the points
    /// it takes at the fitted magnification, which is why the ratio between
    /// them needs `pixels_per_point`: on a screen at 125% a photograph drawn
    /// at what looked like one for one was a quarter too large.
    pub fn zoom(
        self,
        image_size: Vec2,
        fit_size: Vec2,
        available: Vec2,
        pixels_per_point: f32,
    ) -> f32 {
        match self {
            Opening::Fit => zoom::FITTED,
            Opening::Fill => canvas::fill_zoom(fit_size, available),
            Opening::Actual => zoom::at_percent(image_size, fit_size, pixels_per_point, 100.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 4000x2000 photograph fitted into a square 800 point panel: it comes
    /// out 800 points wide, which is a fifth of the pixels it has.
    const IMAGE: Vec2 = Vec2::new(4000.0, 2000.0);
    const FIT_SIZE: Vec2 = Vec2::new(800.0, 400.0);
    const PANEL: Vec2 = Vec2::new(800.0, 800.0);

    #[test]
    fn every_opening_survives_the_file() {
        for opening in Opening::ALL {
            assert_eq!(Opening::of(opening.value()), Some(*opening));
        }

        assert_eq!(Opening::of("halfway"), None);
    }

    #[test]
    fn fitting_is_the_magnification_everything_else_is_measured_against() {
        assert_eq!(Opening::Fit.zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 1.0);
    }

    /// The fitted photograph touches the left and right of the panel and
    /// leaves half of it empty, so filling it means twice the magnification.
    #[test]
    fn filling_covers_the_longer_side() {
        assert_eq!(Opening::Fill.zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 2.0);
    }

    /// Four thousand pixels drawn across eight hundred points is five times
    /// the fitted magnification.
    #[test]
    fn its_own_size_is_the_photographs_pixels_against_the_screens() {
        assert_eq!(Opening::Actual.zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 5.0);
    }

    /// The same photograph on a screen at 125%: a point is a pixel and a
    /// quarter, so one for one is four fifths of what it would otherwise be.
    #[test]
    fn its_own_size_is_the_screens_pixels_rather_than_its_points() {
        assert_eq!(Opening::Actual.zoom(IMAGE, FIT_SIZE, PANEL, 1.25), 4.0);
    }

    #[test]
    fn the_key_goes_round_the_three_and_back() {
        assert_eq!(Opening::Fit.next(), Opening::Fill);
        assert_eq!(Opening::Fill.next(), Opening::Actual);
        assert_eq!(Opening::Actual.next(), Opening::Fit);
    }

    /// Fitting is what the viewer has always done, so the bar says nothing
    /// about it and something about the other two.
    #[test]
    fn only_the_unusual_openings_reach_the_status_bar() {
        assert_eq!(Opening::Fit.word(), None);
        assert!(Opening::Fill.word().is_some());
        assert!(Opening::Actual.word().is_some());
    }
}
