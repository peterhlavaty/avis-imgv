//! The part of the photograph the user has marked out.
//!
//! Held as a rectangle normalised against the photograph *as displayed*,
//! nought to one on each axis, which is the space [`Metrics::uv`] is already
//! in. That is what makes a marking stick to the picture rather than to the
//! screen: zoom, pan and a quarter turn each move where it is drawn and none
//! of them move what it is about, and the same two corners crop the full size
//! decode the clipboard is given.
//!
//! It belongs to the photograph on screen and goes when that changes, in
//! [`ImageView::select`](super::ImageView::select). A rectangle over one frame
//! means nothing over the next one, and a marking left behind is a marking
//! somebody copies by accident.

pub mod draw;
pub mod grip;
pub mod pointer;
pub mod view;

use eframe::egui::Rect;
use eframe::epaint::{Pos2, Vec2};

use super::canvas::Metrics;
use grip::Grip;

/// The smallest a marking may be, as a fraction of the photograph.
///
/// Not nought: a rectangle with no width cannot be zoomed to, cannot be
/// copied, and cannot be taken hold of by an edge to make it large again.
pub const SMALLEST: f32 = 0.002;

/// How close to a side counts as being on it, in points.
///
/// Eight rather than the two a line is wide, because the thing being aimed at
/// is a side of a rectangle and the thing doing the aiming is a hand.
pub const REACH: f32 = 8.0;

/// How far a drag has to have gone before what it drew was meant, in points.
const MEANT: f32 = 4.0;

/// The area marked on the photograph on screen, if any.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Area {
    marked: Option<Rect>,
    doing: Option<Doing>,
}

/// What the pointer is in the middle of doing to it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Doing {
    /// Drawing a new one, out from the corner the press landed on.
    Drawing(Pos2),
    /// Moving one side or one corner of the one already marked.
    Resizing(Grip),
}

impl Area {
    /// The marking, in the photograph's own coordinates.
    pub fn marked(&self) -> Option<Rect> {
        self.marked
    }

    /// Whether the pointer is in the middle of a gesture this owns.
    ///
    /// What stops the same drag from also moving the photograph underneath it:
    /// one press is one gesture.
    pub fn is_dragging(&self) -> bool {
        self.doing.is_some()
    }

    /// Forgets it, which is what a click outside and `Escape` both do.
    pub fn clear(&mut self) {
        self.marked = None;
        self.doing = None;
    }

    /// Where it is on screen, given what the last frame drew.
    pub fn on_screen(&self, metrics: &Metrics) -> Option<Rect> {
        to_screen(metrics, self.marked?)
    }
}

/// A point on the photograph, nought to one, from a point on the screen.
pub fn to_image(metrics: &Metrics, at: Pos2) -> Option<Pos2> {
    let (rect, uv) = (metrics.rect, metrics.uv);

    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }

    Some(Pos2::new(
        uv.min.x + (at.x - rect.min.x) / rect.width() * uv.width(),
        uv.min.y + (at.y - rect.min.y) / rect.height() * uv.height(),
    ))
}

/// And back again, which is only possible while some of the photograph is on
/// screen to map onto.
pub fn to_screen(metrics: &Metrics, area: Rect) -> Option<Rect> {
    let (rect, uv) = (metrics.rect, metrics.uv);

    if uv.width() <= 0.0 || uv.height() <= 0.0 {
        return None;
    }

    let point = |at: Pos2| {
        Pos2::new(
            rect.min.x + (at.x - uv.min.x) / uv.width() * rect.width(),
            rect.min.y + (at.y - uv.min.y) / uv.height() * rect.height(),
        )
    };

    Some(Rect::from_min_max(point(area.min), point(area.max)))
}

/// Keeps a point on the photograph, so a drag off the edge of it marks the
/// edge rather than something that is not there.
pub fn inside_unit(at: Pos2) -> Pos2 {
    Pos2::new(at.x.clamp(0.0, 1.0), at.y.clamp(0.0, 1.0))
}

/// The zoom and the pan that put `marked` on the screen and nothing else.
///
/// The pan is what the canvas would settle on unclamped; it clamps it again
/// when it draws, which is what keeps a marking against the edge of a
/// photograph from asking to see past it.
pub fn zoom_to(metrics: &Metrics, marked: Rect) -> Option<(f32, Vec2)> {
    let (across, down) = (marked.width(), marked.height());

    if across < SMALLEST || down < SMALLEST {
        return None;
    }

    if metrics.fit_size.x <= 0.0 || metrics.fit_size.y <= 0.0 {
        return None;
    }

    let zoom = (metrics.available_size.x / (metrics.fit_size.x * across))
        .min(metrics.available_size.y / (metrics.fit_size.y * down));

    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }

    // The canvas puts the middle of the window at `0.5 + pan / scaled`, so
    // putting the middle of the marking there is one subtraction.
    let scaled = metrics.fit_size * zoom;
    let centre = marked.center();

    Some((
        zoom,
        Vec2::new((centre.x - 0.5) * scaled.x, (centre.y - 0.5) * scaled.y),
    ))
}

/// The marking as pixels of an image that many across and that many down.
///
/// Clamped to the image and to at least one pixel, because the caller is about
/// to index into it.
pub fn in_pixels(marked: Rect, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    if width == 0 || height == 0 {
        return None;
    }

    let at = |value: f32, of: u32| (value.clamp(0.0, 1.0) * of as f32).round() as u32;

    let left = at(marked.min.x, width).min(width - 1);
    let top = at(marked.min.y, height).min(height - 1);
    let across = at(marked.max.x, width)
        .saturating_sub(left)
        .clamp(1, width - left);
    let down = at(marked.max.y, height)
        .saturating_sub(top)
        .clamp(1, height - top);

    Some((left, top, across, down))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 4000x2000 photograph fitted into an 800x800 panel: the whole of it is
    /// on screen, letterboxed 200 points down.
    fn fitted() -> Metrics {
        Metrics {
            image_size: Vec2::new(4000.0, 2000.0),
            available_size: Vec2::new(800.0, 800.0),
            fit_size: Vec2::new(800.0, 400.0),
            pixels_per_point: 1.0,
            rect: Rect::from_min_size(Pos2::new(0.0, 200.0), Vec2::new(800.0, 400.0)),
            uv: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            ..Metrics::default()
        }
    }

    #[test]
    fn the_two_mappings_are_each_other() {
        let metrics = fitted();
        let area = Rect::from_min_max(Pos2::new(0.25, 0.5), Pos2::new(0.75, 0.9));

        let on_screen = to_screen(&metrics, area).expect("a picture to map onto");
        let back = Rect::from_min_max(
            to_image(&metrics, on_screen.min).expect("a picture"),
            to_image(&metrics, on_screen.max).expect("a picture"),
        );

        assert!((back.min.x - area.min.x).abs() < 0.001, "{back:?}");
        assert!((back.max.y - area.max.y).abs() < 0.001, "{back:?}");
    }

    #[test]
    fn a_marking_lands_where_the_picture_is() {
        let on_screen = to_screen(
            &fitted(),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(0.5, 0.5)),
        )
        .expect("a picture");

        assert_eq!(on_screen.min, Pos2::new(0.0, 200.0));
        assert_eq!(on_screen.max, Pos2::new(400.0, 400.0));
    }

    /// Zoomed, the same rectangle is drawn somewhere else and is still about
    /// the same part of the photograph.
    #[test]
    fn zooming_moves_where_it_is_drawn_and_not_what_it_is_about() {
        let zoomed = Metrics {
            // The right hand half of the photograph, filling the panel.
            uv: Rect::from_min_max(Pos2::new(0.5, 0.0), Pos2::new(1.0, 1.0)),
            rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 800.0)),
            ..fitted()
        };

        let area = Rect::from_min_max(Pos2::new(0.5, 0.0), Pos2::new(1.0, 1.0));

        assert_eq!(to_screen(&zoomed, area), Some(zoomed.rect));
    }

    #[test]
    fn a_marking_that_fills_the_photograph_is_the_fitted_zoom() {
        let whole = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        let (zoom, pan) = zoom_to(&fitted(), whole).expect("a zoom");

        assert!((zoom - 1.0).abs() < 0.001, "{zoom}");
        assert_eq!(pan, Vec2::ZERO);
    }

    /// Half the width of a photograph wider than it is tall doubles the
    /// magnification, and the middle of the marking moves to the middle of the
    /// panel.
    #[test]
    fn a_marking_is_magnified_until_it_fills_the_panel() {
        let right = Rect::from_min_max(Pos2::new(0.5, 0.0), Pos2::new(1.0, 1.0));
        let (zoom, pan) = zoom_to(&fitted(), right).expect("a zoom");

        assert!((zoom - 2.0).abs() < 0.001, "{zoom}");
        // The middle of the marking is three quarters of the way across a
        // picture drawn 1600 points wide, which is 400 right of the middle.
        assert!((pan.x - 400.0).abs() < 0.001, "{pan:?}");
        assert_eq!(pan.y, 0.0);
    }

    /// The side that does not fit is the one that decides, so nothing marked
    /// is left off the screen by the zoom that was supposed to show it.
    #[test]
    fn the_tighter_of_the_two_ratios_wins() {
        let tall = Rect::from_min_max(Pos2::new(0.4, 0.0), Pos2::new(0.6, 1.0));
        let (zoom, _) = zoom_to(&fitted(), tall).expect("a zoom");

        // 800 across a fifth of 800 points is 5; 800 down all 400 points is 2.
        assert!((zoom - 2.0).abs() < 0.001, "{zoom}");
    }

    #[test]
    fn a_marking_with_no_width_is_not_zoomed_to() {
        let line = Rect::from_min_max(Pos2::new(0.5, 0.0), Pos2::new(0.5, 1.0));
        assert_eq!(zoom_to(&fitted(), line), None);
    }

    /// Before the first frame there is no geometry to work against.
    #[test]
    fn nothing_is_zoomed_to_before_a_frame_is_drawn() {
        let whole = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        assert_eq!(zoom_to(&Metrics::default(), whole), None);
    }

    #[test]
    fn a_marking_becomes_the_pixels_under_it() {
        let quarter = Rect::from_min_max(Pos2::new(0.25, 0.5), Pos2::new(0.5, 1.0));

        assert_eq!(
            in_pixels(quarter, 4000, 2000),
            Some((1000, 1000, 1000, 1000))
        );
    }

    /// Rounding must never walk off the end of the buffer the caller is about
    /// to index into.
    #[test]
    fn the_pixels_stay_inside_the_image() {
        let past = Rect::from_min_max(Pos2::new(0.999, 0.999), Pos2::new(2.0, 2.0));
        let (left, top, across, down) = in_pixels(past, 100, 100).expect("some pixels");

        assert!(left + across <= 100, "{left} + {across}");
        assert!(top + down <= 100, "{top} + {down}");
        assert!(across >= 1 && down >= 1);
    }

    #[test]
    fn an_image_with_no_pixels_has_nothing_to_crop() {
        let whole = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        assert_eq!(in_pixels(whole, 0, 10), None);
    }

    #[test]
    fn a_point_off_the_photograph_is_pulled_back_onto_it() {
        assert_eq!(inside_unit(Pos2::new(-0.5, 1.5)), Pos2::new(0.0, 1.0));
    }
}
