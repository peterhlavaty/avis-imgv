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

/// How a photograph is presented, as opposed to which one.
#[derive(Debug, Clone)]
pub struct Style {
    /// What to say over the photograph, and where. Empty text draws nothing.
    pub overlay: Overlay,
    /// A clipping or focus mask to paint over the leading pane, if any.
    pub mask: Option<egui::TextureId>,
    pub frame: FrameStyle,
    /// Whether a photograph smaller than the panel is enlarged to fill it.
    ///
    /// The one that needs it is a raw file's embedded copy: some DNGs carry a
    /// 256 pixel preview and nothing else.
    pub enlarge: bool,
}

/// What is written over the photograph, already expanded.
///
/// Expanded by the caller rather than here: the canvas draws one pane and does
/// not know which photograph it is, and rendering a template per pane per
/// frame would be work to arrive at the same sentence.
#[derive(Debug, Clone, Default)]
pub struct Overlay {
    pub corner: super::overlay::Corner,
    pub lines: String,
    pub size: f32,
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
#[derive(Debug, Clone, Copy)]
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
    /// Screen pixels to a logical point, as the window is scaled right now.
    ///
    /// A texture is measured in pixels and a layout in points, so anything
    /// that compares the two — what "100%" means, above all — needs this and
    /// is wrong without it.
    pub pixels_per_point: f32,
    /// Where on screen the image ended up.
    ///
    /// Needed to zoom about the pointer: the panel is not the picture, and a
    /// letterboxed photograph sits somewhere inside it.
    pub rect: Rect,
    /// Which part of the texture that rectangle showed.
    ///
    /// What the clipping and focus masks are drawn through, so they follow the
    /// zoom and the pan without knowing anything about either.
    pub uv: Rect,
}

/// Nothing drawn yet, which every zoom command has to be safe against.
///
/// Written out rather than derived because a rectangle has no default: an
/// empty one at the origin is the honest answer here, and `Rect::ZERO` is it.
impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            image_size: Vec2::ZERO,
            available_size: Vec2::ZERO,
            fit_size: Vec2::ZERO,
            percentage_zoom: 0.0,
            drawn_width: 0.0,
            pixels_per_point: 1.0,
            rect: Rect::ZERO,
            uv: Rect::ZERO,
        }
    }
}

impl Metrics {
    /// Size the whole image takes at the current zoom, in points.
    pub fn scaled(&self, zoom: f32) -> Vec2 {
        self.fit_size * zoom
    }

    /// The window onto it, which is the smaller of the image and the panel.
    pub fn display(&self, zoom: f32) -> Vec2 {
        let scaled = self.scaled(zoom);

        Vec2::new(
            scaled.x.min(self.available_size.x),
            scaled.y.min(self.available_size.y),
        )
    }
}

/// Draws `texture` into the current `ui`, returning the geometry it used.
/// `leading` marks the one pane that owns the viewport.
///
/// Every pane shares one zoom and one pan, which is what makes a comparison a
/// comparison. Only one of them may move it: the drag used to be applied once
/// per pane, so two side by side panned twice as fast as one and four four
/// times, and the last pane drawn clamped the pan against *its* picture rather
/// than against the one being looked at.
pub fn draw(
    ui: &mut egui::Ui,
    texture: &GpuTexture,
    viewport: &mut Viewport,
    style: &Style,
    leading: bool,
) -> Metrics {
    let available = ui.available_size();
    let fit_size = if style.enlarge {
        fill(texture.size, available)
    } else {
        fit(texture.size, available)
    };

    if leading && viewport.maximize && !viewport.maximized {
        viewport.maximized = true;
        viewport.zoom = fill_zoom(fit_size, available);
    }

    let scaled = fit_size * viewport.zoom;
    let display_size = Vec2::new(scaled.x.min(available.x), scaled.y.min(available.y));

    let mut pan = viewport.pan;
    let uv = uv_rect(
        scaled,
        display_size,
        &mut pan,
        if leading {
            viewport.scroll_delta
        } else {
            Vec2::ZERO
        },
    );

    if leading {
        viewport.pan = pan;
    }

    let pixels_per_point = ui.ctx().pixels_per_point();
    let drawn_width = scaled.x * pixels_per_point;

    let framed = if style.frame.enabled {
        paint_frame(ui, display_size, style.frame.relative_size)
    } else {
        display_size
    };

    let (rect, _) = ui.allocate_exact_size(framed, Sense::hover());
    texture::draw(ui, rect, texture, uv);

    // Through the photograph's own texture coordinates, so a mask follows the
    // zoom and the pan for nothing — and a quarter turn turns it too, because
    // `texture::draw` has already mapped the corners.
    if leading {
        if let Some(mask) = style.mask {
            let mut mesh = eframe::epaint::Mesh::with_texture(mask);
            let turned = Rect::from_min_max(
                texture::to_texture(texture.orientation, uv.min),
                texture::to_texture(texture.orientation, uv.max),
            );

            mesh.add_rect_with_uv(rect, turned, eframe::epaint::Color32::WHITE);
            ui.painter().add(mesh);
        }
    }

    // Over the picture rather than over the panel, so a letterboxed
    // photograph gets its caption on the photograph.
    super::overlay::show(
        ui,
        rect,
        style.overlay.corner,
        &style.overlay.lines,
        style.overlay.size,
    );

    Metrics {
        image_size: texture.size,
        available_size: available,
        fit_size,
        // Against the pixels the screen actually has, not the points the
        // layout is measured in. They are the same thing only at 100% window
        // scaling, and everywhere else this used to report a photograph drawn
        // one pixel for one pixel as 80%.
        percentage_zoom: if texture.size.x > 0.0 {
            drawn_width * 100.0 / texture.size.x
        } else {
            0.0
        },
        drawn_width,
        pixels_per_point,
        rect,
        uv,
    }
}

/// Where the view sits `progress` of the way across a picture larger than the
/// panel.
///
/// Nought puts the near edge of the picture against the near edge of the
/// screen and one puts the far edge against the far one, so a picture that
/// overflows is entirely seen by the time its turn is up. The axis that fits
/// exactly has no slack and does not move.
pub fn travelled(scaled: Vec2, available: Vec2, progress: f32) -> Vec2 {
    let slack = scaled - available;
    let along = progress.clamp(0.0, 1.0) - 0.5;

    Vec2::new(slack.x.max(0.0) * along, slack.y.max(0.0) * along)
}

/// The largest size with the image's shape that the panel holds, enlarging a
/// small one to reach it.
///
/// What a raw file's embedded copy needs: some DNGs carry a 256 pixel preview
/// and nothing else, and drawn at its own size it is a postage stamp in the
/// middle of a 4K screen.
fn fill(image: Vec2, available: Vec2) -> Vec2 {
    if image.x <= 0.0 || image.y <= 0.0 {
        return Vec2::ZERO;
    }

    let scale = (available.x / image.x).min(available.y / image.y);
    image * scale
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
    fn travelling_starts_at_one_edge_and_ends_at_the_other() {
        // A picture filling an 800x600 panel and overflowing by 400 across.
        let scaled = Vec2::new(1200.0, 600.0);
        let panel = Vec2::new(800.0, 600.0);

        assert_eq!(travelled(scaled, panel, 0.0), Vec2::new(-200.0, 0.0));
        assert_eq!(travelled(scaled, panel, 0.5), Vec2::ZERO);
        assert_eq!(travelled(scaled, panel, 1.0), Vec2::new(200.0, 0.0));
    }

    #[test]
    fn a_picture_that_fits_exactly_does_not_travel() {
        let exact = Vec2::new(800.0, 600.0);

        assert_eq!(travelled(exact, exact, 0.0), Vec2::ZERO);
        assert_eq!(travelled(exact, exact, 1.0), Vec2::ZERO);
    }

    #[test]
    fn travelling_past_the_end_stops_at_the_end() {
        let scaled = Vec2::new(1200.0, 600.0);
        let panel = Vec2::new(800.0, 600.0);

        assert_eq!(travelled(scaled, panel, 3.0), travelled(scaled, panel, 1.0));
        assert_eq!(
            travelled(scaled, panel, -1.0),
            travelled(scaled, panel, 0.0)
        );
    }

    /// A raw file's embedded copy can be 256 pixels across; drawn at its own
    /// size it is a postage stamp in the middle of a 4K screen.
    #[test]
    fn filling_enlarges_a_small_photograph() {
        let filled = fill(Vec2::new(256.0, 171.0), Vec2::new(1920.0, 1080.0));

        assert!((filled.x - 1616.8422).abs() < 0.1, "{filled:?}");
        assert!((filled.y - 1080.0).abs() < 0.1, "{filled:?}");
    }

    #[test]
    fn filling_shrinks_a_large_one_the_same_way_fitting_does() {
        let size = Vec2::new(6000.0, 4000.0);
        let panel = Vec2::new(1000.0, 1000.0);

        assert_eq!(fill(size, panel), fit(size, panel));
    }

    #[test]
    fn filling_a_degenerate_image_produces_nothing() {
        assert_eq!(fill(Vec2::ZERO, Vec2::new(100.0, 100.0)), Vec2::ZERO);
    }

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
