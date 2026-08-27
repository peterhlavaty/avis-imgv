//! Drawing one image: fitting, zooming, panning and the optional frame.
//!
//! The texture is drawn once per frame with a UV rectangle, so zoom and pan
//! cost nothing beyond the four numbers handed to the GPU — no pixels are
//! touched on the CPU.

use eframe::egui::{self, Rect, Sense};
use eframe::epaint::{Color32, Pos2, Vec2};

use crate::cache::gpu::GpuTexture;
use crate::view::texture;

/// The white border some users like around a displayed photo.
#[derive(Debug, Clone, Copy)]
pub struct FrameStyle {
    pub enabled: bool,
    /// Stroke width as a fraction of the image's shortest side.
    pub relative_size: f32,
}

/// Zoom and pan, owned by the view and mutated as the user navigates.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// 1.0 means "fitted to the panel"; larger crops into the image.
    pub zoom: f32,
    /// Accumulated pan, in screen pixels.
    pub pan: Vec2,
    /// Scroll or drag applied this frame.
    pub scroll_delta: Vec2,
    /// Whether newly shown images should fill the panel rather than fit in it.
    pub maximize: bool,
    /// Set once the current image has been maximised, so it happens exactly
    /// once per image and never mid-interaction.
    pub maximized: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            zoom: 1.0,
            pan: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            maximize: false,
            maximized: false,
        }
    }
}

impl Viewport {
    /// Called when a different image is shown.
    pub fn reset_for_new_image(&mut self) {
        self.pan = Vec2::ZERO;
        self.maximized = false;
    }
}

/// What the last draw worked out, needed by the zoom commands and the status
/// bar.
#[derive(Debug, Clone, Copy, Default)]
pub struct Metrics {
    /// Full resolution of the texture.
    pub image_size: Vec2,
    /// Space the image was drawn into.
    pub available_size: Vec2,
    /// Size the image takes at zoom 1, fitted to the panel.
    pub fit_size: Vec2,
    /// Magnification relative to the image's own pixels.
    pub percentage_zoom: f32,
    /// Width the whole image would take at the current zoom, before it is
    /// clipped to the panel, in the pixels the screen actually has.
    ///
    /// What the uploaded resolution has to cover, so it is measured in the
    /// same physical pixels a texture is.
    pub drawn_width: f32,
}

/// Draws `texture` into the current `ui`, returning the geometry it used.
pub fn draw(
    ui: &mut egui::Ui,
    texture: &GpuTexture,
    viewport: &mut Viewport,
    frame: &FrameStyle,
) -> Metrics {
    let available = ui.available_size();
    let fit_size = fit(texture.size, available);

    if viewport.maximize && !viewport.maximized {
        viewport.maximized = true;
        viewport.zoom = fill_zoom(fit_size, available);
    }

    let scaled = fit_size * viewport.zoom;
    let display_size = Vec2::new(scaled.x.min(available.x), scaled.y.min(available.y));

    let uv = uv_rect(
        scaled,
        display_size,
        &mut viewport.pan,
        viewport.scroll_delta,
    );

    let metrics = Metrics {
        image_size: texture.size,
        available_size: available,
        fit_size,
        percentage_zoom: if texture.size.x > 0.0 {
            scaled.x * 100.0 / texture.size.x
        } else {
            0.0
        },
        drawn_width: scaled.x * ui.ctx().pixels_per_point(),
    };

    let display_size = if frame.enabled {
        paint_frame(ui, display_size, frame.relative_size)
    } else {
        display_size
    };

    let (rect, _) = ui.allocate_exact_size(display_size, Sense::hover());
    texture::draw(ui, rect, texture, uv);

    metrics
}

/// Largest size with the image's aspect ratio that fits inside `available`.
///
/// Never enlarges: a small image is shown at its own size until zoomed.
fn fit(image: Vec2, available: Vec2) -> Vec2 {
    if image.x <= 0.0 || image.y <= 0.0 {
        return Vec2::ZERO;
    }

    let aspect = image.x / image.y;
    let mut size = image;

    if available.x < size.x {
        size = Vec2::new(available.x, available.x / aspect);
    }

    if available.y < size.y {
        size = Vec2::new(available.y * aspect, available.y);
    }

    size
}

/// Zoom that makes the fitted image cover the whole panel, cropping the
/// overflowing side.
///
/// The fitted image already touches one edge of the panel, so covering both
/// means scaling by whichever ratio is larger.
pub fn fill_zoom(fit_size: Vec2, available: Vec2) -> f32 {
    if fit_size.x <= 0.0 || fit_size.y <= 0.0 {
        return 1.0;
    }

    (available.x / fit_size.x).max(available.y / fit_size.y)
}

/// The visible part of the image, in normalised texture coordinates.
///
/// `pan` is clamped to the slack between the zoomed image and the panel, so
/// the image can never be dragged off screen.
fn uv_rect(scaled: Vec2, display: Vec2, pan: &mut Vec2, scroll_delta: Vec2) -> Rect {
    if scaled.x <= 0.0 || scaled.y <= 0.0 {
        return Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    }

    let slack = scaled - display;

    // Scrolling down should move the image up, hence the negated delta.
    *pan = Vec2::new(
        clamp_pan(pan.x - scroll_delta.x, slack.x),
        clamp_pan(pan.y - scroll_delta.y, slack.y),
    );

    let min = (slack / 2.0 + *pan) / scaled;
    let max = min + display / scaled;

    Rect::from_min_max(min.to_pos2(), max.to_pos2())
}

fn clamp_pan(value: f32, slack: f32) -> f32 {
    if slack <= 0.0 {
        0.0
    } else {
        value.clamp(-slack / 2.0, slack / 2.0)
    }
}

/// Paints the border behind the image and returns the size left for it.
fn paint_frame(ui: &mut egui::Ui, display_size: Vec2, relative_size: f32) -> Vec2 {
    let stroke = display_size.x.min(display_size.y) * relative_size;
    let aspect = if display_size.y > 0.0 {
        display_size.x / display_size.y
    } else {
        1.0
    };

    let inner = Vec2::new(
        (display_size.x - stroke).max(1.0),
        (display_size.y - stroke / aspect).max(1.0),
    );

    let available = ui.available_rect_before_wrap();
    let outer = inner + Vec2::new(stroke, stroke / aspect);
    let top_left = available.center() - outer / 2.0;

    ui.painter()
        .rect_filled(Rect::from_min_size(top_left, outer), 1.0, Color32::WHITE);

    inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitting_preserves_the_aspect_ratio() {
        let fitted = fit(Vec2::new(4000.0, 2000.0), Vec2::new(1000.0, 1000.0));
        assert_eq!(fitted, Vec2::new(1000.0, 500.0));

        let fitted = fit(Vec2::new(2000.0, 4000.0), Vec2::new(1000.0, 1000.0));
        assert_eq!(fitted, Vec2::new(500.0, 1000.0));
    }

    #[test]
    fn fitting_never_enlarges() {
        let fitted = fit(Vec2::new(100.0, 50.0), Vec2::new(1000.0, 1000.0));
        assert_eq!(fitted, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn a_degenerate_image_fits_to_nothing() {
        assert_eq!(fit(Vec2::ZERO, Vec2::new(10.0, 10.0)), Vec2::ZERO);
    }

    #[test]
    fn filling_crops_the_longer_side() {
        // A wide image in a square panel must grow until its height covers it.
        let zoom = fill_zoom(Vec2::new(1000.0, 500.0), Vec2::new(1000.0, 1000.0));
        assert_eq!(zoom, 2.0);

        let zoom = fill_zoom(Vec2::new(500.0, 1000.0), Vec2::new(1000.0, 1000.0));
        assert_eq!(zoom, 2.0);
    }

    #[test]
    fn an_unzoomed_image_shows_all_of_itself() {
        let mut pan = Vec2::ZERO;
        let uv = uv_rect(
            Vec2::new(800.0, 600.0),
            Vec2::new(800.0, 600.0),
            &mut pan,
            Vec2::ZERO,
        );

        assert_eq!(uv.min, Pos2::ZERO);
        assert_eq!(uv.max, Pos2::new(1.0, 1.0));
    }

    #[test]
    fn a_zoomed_image_shows_its_centre() {
        let mut pan = Vec2::ZERO;
        let uv = uv_rect(
            Vec2::new(1600.0, 1200.0),
            Vec2::new(800.0, 600.0),
            &mut pan,
            Vec2::ZERO,
        );

        assert_eq!(uv.min, Pos2::new(0.25, 0.25));
        assert_eq!(uv.max, Pos2::new(0.75, 0.75));
    }

    #[test]
    fn panning_stops_at_the_edges() {
        let mut pan = Vec2::ZERO;
        // Far more scroll than there is slack.
        let uv = uv_rect(
            Vec2::new(1600.0, 1200.0),
            Vec2::new(800.0, 600.0),
            &mut pan,
            Vec2::new(0.0, -10_000.0),
        );

        assert_eq!(pan.y, 300.0);
        assert_eq!(uv.max.y, 1.0);
        assert_eq!(uv.min.y, 0.5);
    }

    #[test]
    fn panning_is_ignored_without_slack() {
        let mut pan = Vec2::new(50.0, 50.0);
        uv_rect(
            Vec2::new(800.0, 600.0),
            Vec2::new(800.0, 600.0),
            &mut pan,
            Vec2::new(100.0, 100.0),
        );

        assert_eq!(pan, Vec2::ZERO);
    }

    #[test]
    fn a_new_image_starts_centred() {
        let mut viewport = Viewport {
            pan: Vec2::new(10.0, 10.0),
            maximized: true,
            ..Default::default()
        };
        viewport.reset_for_new_image();

        assert_eq!(viewport.pan, Vec2::ZERO);
        assert!(!viewport.maximized);
    }
}
