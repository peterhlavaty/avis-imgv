//! The zoom commands, expressed against the geometry of the last frame.
//!
//! Every one of them is a pure function of the viewport and what the canvas
//! measured, which keeps the arithmetic out of the view and under test.

use eframe::epaint::Vec2;

use super::canvas::{self, Metrics, Viewport};

/// Highest magnification the zoom step reaches before wrapping back to fitted.
///
/// The default of `image_view.zoom_step_max`.
pub const MAX_STEP: f32 = 8.0;

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

/// Multiplies the magnification, returning to fitted once it goes far enough.
///
/// The factor and the ceiling arrive from the configuration rather than being
/// written here: one key both magnifies and gets out, and how far it goes
/// before it does is a judgement about the photographs somebody looks at.
pub fn step(viewport: &mut Viewport, factor: f32, ceiling: f32) {
    let factor = if factor > 1.0 { factor } else { 2.0 };
    let ceiling = if ceiling > 1.0 { ceiling } else { MAX_STEP };

    viewport.zoom = if viewport.zoom < ceiling {
        viewport.zoom * factor
    } else {
        1.0
    };
}

/// Draws the image at `percent` of its own pixels: 100 is one for one.
///
/// One for one against the pixels the screen has, which is what anybody means
/// by it: the fitted width is measured in points, so it has to be converted
/// before the two can be divided. On a screen at 125% this used to leave the
/// photograph a quarter larger than it claimed.
pub fn to_percent(viewport: &mut Viewport, metrics: &Metrics, percent: f32) {
    let wanted = metrics.image_size.x * percent / 100.0;
    let fitted = metrics.fit_size.x * metrics.pixels_per_point;

    set(viewport, ratio(wanted, fitted));
}

/// Where the pan has to be for the point under `anchor` to stay put.
///
/// `anchor` is where in the drawn image the point sits, nought to one on each
/// axis, so the middle is `(0.5, 0.5)`. Without this, zooming moves whatever
/// was being looked at away from where it was: magnifying an eye at the edge
/// of the frame used to leave the eye off screen, which is the opposite of
/// what magnifying it was for.
pub fn hold(metrics: &Metrics, pan: Vec2, from: f32, to: f32, anchor: Vec2) -> Vec2 {
    let (scaled, display) = (metrics.scaled(from), metrics.display(from));
    let (after, window) = (metrics.scaled(to), metrics.display(to));

    if scaled.x <= 0.0 || scaled.y <= 0.0 || after.x <= 0.0 || after.y <= 0.0 {
        return pan;
    }

    // Where the visible window starts, in the image's own drawn points, and
    // then which point of the image the anchor is over. The clamp is the one
    // the canvas applies when it draws, so this works from where the picture
    // actually is rather than from where the pan was asked to be.
    let held = Vec2::new(
        clamped(pan.x, scaled.x - display.x),
        clamped(pan.y, scaled.y - display.y),
    );

    let origin = (scaled - display) / 2.0 + held;
    let over = Vec2::new(
        (origin.x + anchor.x * display.x) / scaled.x,
        (origin.y + anchor.y * display.y) / scaled.y,
    );

    Vec2::new(over.x * after.x, over.y * after.y) - anchor * window - (after - window) / 2.0
}

/// The pan the canvas would settle on: half the slack either way, no further.
fn clamped(pan: f32, slack: f32) -> f32 {
    let limit = (slack / 2.0).max(0.0);

    pan.clamp(-limit, limit)
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

    /// A 4000x2000 image fitted into an 800x800 panel, on a screen whose
    /// points and pixels are the same thing.
    fn metrics() -> Metrics {
        Metrics {
            image_size: Vec2::new(4000.0, 2000.0),
            available_size: Vec2::new(800.0, 800.0),
            fit_size: Vec2::new(800.0, 400.0),
            percentage_zoom: 20.0,
            drawn_width: 800.0,
            pixels_per_point: 1.0,
            ..Metrics::default()
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
            step(&mut viewport, 2.0, MAX_STEP);
            assert_eq!(viewport.zoom, expected);
        }

        step(&mut viewport, 2.0, MAX_STEP);
        assert_eq!(viewport.zoom, 1.0);
    }

    /// The factor and the ceiling come from the configuration now, and a
    /// nonsense one falls back rather than freezing the zoom where it is.
    #[test]
    fn a_configured_factor_and_ceiling_are_used() {
        let mut viewport = viewport();

        step(&mut viewport, 1.5, 3.0);
        assert_eq!(viewport.zoom, 1.5);
        step(&mut viewport, 1.5, 3.0);
        assert_eq!(viewport.zoom, 2.25);
        step(&mut viewport, 1.5, 3.0);
        assert_eq!(viewport.zoom, 3.375);
        step(&mut viewport, 1.5, 3.0);
        assert_eq!(viewport.zoom, 1.0);
    }

    #[test]
    fn a_nonsense_factor_falls_back_rather_than_freezing() {
        let mut viewport = viewport();

        step(&mut viewport, 0.0, 0.0);
        assert_eq!(viewport.zoom, 2.0);
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

    /// The case the readout used to get wrong: at 125% window scaling, one
    /// image pixel per screen pixel needs the image drawn over fewer points.
    #[test]
    fn one_to_one_counts_the_pixels_the_screen_has() {
        let scaled = Metrics {
            pixels_per_point: 1.25,
            ..metrics()
        };

        let mut viewport = viewport();
        to_percent(&mut viewport, &scaled, 100.0);

        // 4000 image pixels across a fitted width of 800 points, each of which
        // is a pixel and a quarter.
        assert_eq!(viewport.zoom, 4.0);
    }

    /// Zooming about the middle of the panel keeps the middle of the panel.
    #[test]
    fn the_centre_stays_put_when_it_is_the_anchor() {
        let centre = Vec2::splat(0.5);
        let pan = hold(&metrics(), Vec2::ZERO, 1.0, 4.0, centre);

        assert_eq!(pan, Vec2::ZERO);
    }

    /// The point under the pointer is the point that stays: zooming in on the
    /// left edge keeps the left edge, which means panning right by the amount
    /// the image grew to that side.
    #[test]
    fn the_point_under_the_pointer_is_the_one_that_stays() {
        let metrics = metrics();
        let left = Vec2::new(0.0, 0.5);

        let pan = hold(&metrics, Vec2::ZERO, 1.0, 2.0, left);

        // Fitted, the image is 800x400 in an 800x800 panel: its left edge is
        // the panel's. Doubled it is 1600 wide, so holding the left edge means
        // showing the leftmost 800 of it — the near end of the slack.
        assert_eq!(pan.x, -400.0);
    }

    #[test]
    fn holding_a_point_is_reversible() {
        let metrics = metrics();
        let anchor = Vec2::new(0.25, 0.5);

        let there = hold(&metrics, Vec2::ZERO, 1.0, 6.0, anchor);
        let back = hold(&metrics, there, 6.0, 1.0, anchor);

        assert!(back.x.abs() < 0.01, "{back:?}");
        assert!(back.y.abs() < 0.01, "{back:?}");
    }

    /// Before the first frame there is no geometry to hold a point against.
    #[test]
    fn holding_a_point_needs_a_drawn_frame() {
        let pan = Vec2::new(3.0, 4.0);

        assert_eq!(
            hold(&Metrics::default(), pan, 1.0, 2.0, Vec2::splat(0.5)),
            pan
        );
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
