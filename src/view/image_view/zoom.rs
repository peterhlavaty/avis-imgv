//! The zoom commands, expressed against the geometry of the last frame.
//!
//! Every one of them is a pure function of the viewport and what the canvas
//! measured, which keeps the arithmetic out of the view and under test.

use super::canvas::{self, Metrics, Viewport};

/// Highest magnification the zoom step reaches before wrapping back to fitted.
const MAX_STEP: f32 = 8.0;

/// Shows the whole image, fitted to the panel.
pub fn fit(viewport: &mut Viewport) {
    viewport.zoom = 1.0;
}

/// Fills the panel, cropping whichever side overflows.
pub fn fill(viewport: &mut Viewport, metrics: &Metrics) {
    set(
        viewport,
        canvas::fill_zoom(metrics.fit_size, metrics.available_size),
    );
}

/// Makes the image exactly as wide as the panel.
pub fn fit_horizontal(viewport: &mut Viewport, metrics: &Metrics) {
    set(
        viewport,
        ratio(metrics.available_size.x, metrics.fit_size.x),
    );
}

/// Makes the image exactly as tall as the panel.
pub fn fit_vertical(viewport: &mut Viewport, metrics: &Metrics) {
    set(
        viewport,
        ratio(metrics.available_size.y, metrics.fit_size.y),
    );
}

/// Doubles the magnification, returning to fitted once it goes far enough.
pub fn step(viewport: &mut Viewport) {
    viewport.zoom = if viewport.zoom < MAX_STEP {
        viewport.zoom * 2.0
    } else {
        1.0
    };
}

/// Draws the image at `percent` of its own pixels: 100 is one for one.
pub fn to_percent(viewport: &mut Viewport, metrics: &Metrics, percent: f32) {
    let drawn_width = metrics.image_size.x * percent / 100.0;
    set(viewport, ratio(drawn_width, metrics.fit_size.x));
}

/// Multiplies the magnification, for pinch and scroll gestures.
pub fn by(viewport: &mut Viewport, factor: f32) {
    set(viewport, viewport.zoom * factor);
}

/// Applies a zoom only when it is a usable number.
///
/// Before the first frame is drawn the metrics are all zero, so the commands
/// must be safe to issue against them.
fn set(viewport: &mut Viewport, zoom: f32) {
    if zoom.is_finite() && zoom > 0.0 {
        viewport.zoom = zoom;
    }
}

fn ratio(numerator: f32, denominator: f32) -> f32 {
    if denominator == 0.0 {
        f32::NAN
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::epaint::Vec2;

    /// A 4000x2000 image fitted into an 800x800 panel.
    fn metrics() -> Metrics {
        Metrics {
            image_size: Vec2::new(4000.0, 2000.0),
            available_size: Vec2::new(800.0, 800.0),
            fit_size: Vec2::new(800.0, 400.0),
            percentage_zoom: 20.0,
            drawn_width: 800.0,
        }
    }

    fn viewport() -> Viewport {
        Viewport::default()
    }

    #[test]
    fn fitting_is_the_baseline() {
        let mut viewport = viewport();
        viewport.zoom = 4.0;
        fit(&mut viewport);

        assert_eq!(viewport.zoom, 1.0);
    }

    #[test]
    fn filling_covers_the_shorter_side() {
        let mut viewport = viewport();
        fill(&mut viewport, &metrics());

        // The fitted image is 400 tall in an 800 tall panel.
        assert_eq!(viewport.zoom, 2.0);
    }

    #[test]
    fn fitting_to_an_edge_matches_the_panel() {
        let mut viewport = viewport();
        fit_horizontal(&mut viewport, &metrics());
        assert_eq!(viewport.zoom, 1.0);

        fit_vertical(&mut viewport, &metrics());
        assert_eq!(viewport.zoom, 2.0);
    }

    #[test]
    fn one_to_one_draws_a_pixel_per_pixel() {
        let mut viewport = viewport();
        to_percent(&mut viewport, &metrics(), 100.0);

        // 4000 image pixels across a fitted width of 800.
        assert_eq!(viewport.zoom, 5.0);

        to_percent(&mut viewport, &metrics(), 50.0);
        assert_eq!(viewport.zoom, 2.5);
    }

    #[test]
    fn stepping_doubles_then_wraps() {
        let mut viewport = viewport();

        for expected in [2.0, 4.0, 8.0] {
            step(&mut viewport);
            assert_eq!(viewport.zoom, expected);
        }

        step(&mut viewport);
        assert_eq!(viewport.zoom, 1.0);
    }

    #[test]
    fn multiplying_scales_the_current_zoom() {
        let mut viewport = viewport();
        by(&mut viewport, 1.5);
        assert_eq!(viewport.zoom, 1.5);
    }

    #[test]
    fn commands_issued_before_the_first_frame_are_ignored() {
        let empty = Metrics::default();

        for command in [
            fill as fn(&mut Viewport, &Metrics),
            fit_horizontal,
            fit_vertical,
        ] {
            let mut viewport = viewport();
            command(&mut viewport, &empty);
            assert_eq!(viewport.zoom, 1.0);
        }

        let mut viewport = viewport();
        to_percent(&mut viewport, &empty, 100.0);
        assert_eq!(viewport.zoom, 1.0);
    }

    #[test]
    fn a_nonsense_factor_is_ignored() {
        let mut viewport = viewport();
        by(&mut viewport, 0.0);
        assert_eq!(viewport.zoom, 1.0);

        by(&mut viewport, f32::INFINITY);
        assert_eq!(viewport.zoom, 1.0);
    }
}
