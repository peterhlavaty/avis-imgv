//! What a photograph is drawn at on the frame it first appears.
//!
//! Fitted, filled, as wide or as tall as the window, or at a percentage of its
//! own pixels. The choice divides the ways a shoot is actually gone through —
//! judging a composition wants the whole of the picture, judging focus wants a
//! hundred per cent of it and nothing else, and a folder of panoramas wants the
//! width — so it is a setting rather than a number chosen here on somebody's
//! behalf.
//!
//! It cannot be decided anywhere but the canvas. The fitted size is a function
//! of the window and of whichever panels are open in it, so what a hundred per
//! cent is worth as a magnification is not known until the frame is being
//! measured. This says what to aim for; `canvas::draw` applies it, once per
//! photograph.

use eframe::epaint::Vec2;

use crate::config::kinds::Opening;

use super::canvas;
use super::zoom;

/// What a photograph opens at, with the magnification the one answer that
/// needs one.
///
/// The two are one decision and are held apart only in the file, where a
/// number and a choice cannot be the same key. Everything that asks — the
/// canvas, the status bar — asks this rather than the two halves, so there is
/// nowhere for them to be read in disagreement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opens {
    pub at: Opening,
    /// Per cent of the photograph's own pixels, read when `at` is
    /// [`Opening::Percent`].
    pub percent: f32,
}

impl Default for Opens {
    fn default() -> Self {
        Opens {
            at: Opening::default(),
            percent: 100.0,
        }
    }
}

impl Opens {
    /// The word the status bar wears while this is what a photograph opens at.
    ///
    /// Fitting says nothing, being what the viewer has always done and what
    /// most people mean by opening a photograph. The bar is a summary of what
    /// is unusual about the moment, not a list of every setting.
    pub fn word(&self) -> Option<String> {
        match self.at {
            Opening::Fit => None,
            Opening::Fill => Some("Filling".to_string()),
            Opening::Width => Some("Full width".to_string()),
            Opening::Height => Some("Full height".to_string()),
            // A whole number is written as one: "250%", not "250.0%", which
            // in a bar of short readings reads as a precision nobody asked
            // for.
            Opening::Percent if self.percent.fract() == 0.0 => {
                Some(format!("{:.0}%", self.percent))
            }
            Opening::Percent => Some(format!("{:.1}%", self.percent)),
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
        &self,
        image_size: Vec2,
        fit_size: Vec2,
        available: Vec2,
        pixels_per_point: f32,
    ) -> f32 {
        match self.at {
            Opening::Fit => zoom::FITTED,
            Opening::Fill => canvas::fill_zoom(fit_size, available),
            Opening::Width => zoom::ratio(available.x, fit_size.x),
            Opening::Height => zoom::ratio(available.y, fit_size.y),
            Opening::Percent => {
                zoom::at_percent(image_size, fit_size, pixels_per_point, self.percent)
            }
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

    fn opens(at: Opening) -> Opens {
        Opens { at, percent: 100.0 }
    }

    #[test]
    fn every_opening_survives_the_file() {
        for opening in Opening::ALL {
            assert_eq!(Opening::of(opening.value()), Some(*opening));
        }

        assert_eq!(Opening::of("halfway"), None);
    }

    #[test]
    fn fitting_is_the_magnification_everything_else_is_measured_against() {
        assert_eq!(opens(Opening::Fit).zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 1.0);
    }

    /// The fitted photograph touches the left and right of the panel and
    /// leaves half of it empty, so filling it means twice the magnification.
    #[test]
    fn filling_covers_the_longer_side() {
        assert_eq!(opens(Opening::Fill).zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 2.0);
    }

    /// This photograph is fitted by its width already, so asking for the width
    /// asks for nothing; asking for the height is what fills the panel.
    #[test]
    fn a_width_and_a_height_are_the_two_sides_of_the_panel() {
        assert_eq!(opens(Opening::Width).zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 1.0);
        assert_eq!(
            opens(Opening::Height).zoom(IMAGE, FIT_SIZE, PANEL, 1.0),
            2.0
        );
    }

    /// A portrait photograph in the same panel is fitted by its height, and
    /// the two answers swap over.
    #[test]
    fn a_tall_photograph_swaps_the_two_over() {
        let fitted = Vec2::new(400.0, 800.0);

        assert_eq!(opens(Opening::Width).zoom(IMAGE, fitted, PANEL, 1.0), 2.0);
        assert_eq!(opens(Opening::Height).zoom(IMAGE, fitted, PANEL, 1.0), 1.0);
    }

    /// Four thousand pixels drawn across eight hundred points is five times
    /// the fitted magnification.
    #[test]
    fn a_hundred_per_cent_is_the_photographs_pixels_against_the_screens() {
        assert_eq!(
            opens(Opening::Percent).zoom(IMAGE, FIT_SIZE, PANEL, 1.0),
            5.0
        );
    }

    /// The magnification is whatever was asked for, not only a hundred.
    #[test]
    fn any_percentage_can_be_asked_for() {
        let half = Opens {
            at: Opening::Percent,
            percent: 50.0,
        };
        let quadruple = Opens {
            at: Opening::Percent,
            percent: 400.0,
        };

        assert_eq!(half.zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 2.5);
        assert_eq!(quadruple.zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 20.0);
    }

    /// The same photograph on a screen at 125%: a point is a pixel and a
    /// quarter, so one for one is four fifths of what it would otherwise be.
    #[test]
    fn a_percentage_is_the_screens_pixels_rather_than_its_points() {
        assert_eq!(
            opens(Opening::Percent).zoom(IMAGE, FIT_SIZE, PANEL, 1.25),
            4.0
        );
    }

    /// The magnification is read by one of the five and carried by all of
    /// them, so switching away and back does not lose it.
    #[test]
    fn only_one_of_them_reads_the_magnification() {
        assert!(Opening::Percent.reads_the_percentage());
        assert!(!Opening::Fit.reads_the_percentage());

        let asked = Opens {
            at: Opening::Fill,
            percent: 250.0,
        };

        assert_eq!(asked.zoom(IMAGE, FIT_SIZE, PANEL, 1.0), 2.0);
        assert_eq!(asked.percent, 250.0);
    }

    #[test]
    fn the_key_goes_round_them_all_and_back() {
        let mut at = Opening::Fit;

        for expected in &Opening::ALL[1..] {
            at = at.next();
            assert_eq!(at, *expected);
        }

        assert_eq!(at.next(), Opening::Fit);
    }

    /// Fitting is what the viewer has always done, so the bar says nothing
    /// about it and something about each of the others — including which
    /// magnification was asked for, since "at a percentage" without the
    /// number is not a reading.
    #[test]
    fn only_the_unusual_openings_reach_the_status_bar() {
        assert_eq!(opens(Opening::Fit).word(), None);

        for at in &Opening::ALL[1..] {
            assert!(opens(*at).word().is_some(), "{at:?} says nothing");
        }

        assert_eq!(
            Opens {
                at: Opening::Percent,
                percent: 250.0,
            }
            .word()
            .as_deref(),
            Some("250%")
        );
        assert_eq!(
            Opens {
                at: Opening::Percent,
                percent: 12.5,
            }
            .word()
            .as_deref(),
            Some("12.5%")
        );
    }
}
