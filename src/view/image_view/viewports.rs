//! Remembering where the user was in each image.
//!
//! Zooming into a photograph and moving on to the next one should not throw
//! the zoom away: coming back has to show the same corner at the same
//! magnification. Only images that were actually moved are remembered, so
//! walking a folder of ten thousand pictures costs nothing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::epaint::Vec2;

use super::canvas::Viewport;

/// The part of a viewport that belongs to an image rather than to the view.
///
/// The latches — whether new images should fill the panel — are a preference
/// and stay where they are; what is remembered is where the user got to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub zoom: f32,
    pub pan: Vec2,
}

impl Place {
    /// The whole image, centred: what an image that was never touched shows.
    pub const UNTOUCHED: Place = Place {
        zoom: 1.0,
        pan: Vec2::ZERO,
    };

    /// Where a viewport currently is.
    pub fn of(viewport: &Viewport) -> Place {
        Place {
            zoom: viewport.zoom,
            pan: viewport.pan,
        }
    }

    /// Whether this is worth a map entry.
    fn is_worth_remembering(&self) -> bool {
        // The pan is meaningless at a zoom that shows the whole image, and is
        // clamped away on the next frame anyway.
        (self.zoom - 1.0).abs() > f32::EPSILON
    }
}

/// Where the user got to in each image they zoomed.
#[derive(Debug, Default)]
pub struct Viewports {
    places: HashMap<PathBuf, Place>,
}

impl Viewports {
    /// Records where `path` was left, or forgets it if it was left untouched.
    pub fn save(&mut self, path: &Path, viewport: &Viewport) {
        let place = Place::of(viewport);

        if place.is_worth_remembering() {
            self.places.insert(path.to_path_buf(), place);
        } else {
            self.places.remove(path);
        }
    }

    /// Where `path` was left, or the whole image if it was never zoomed.
    pub fn place(&self, path: &Path) -> Place {
        self.places.get(path).copied().unwrap_or(Place::UNTOUCHED)
    }

    /// Puts `viewport` back where `path` was left.
    pub fn restore(&self, path: &Path, viewport: &mut Viewport) {
        Self::put(viewport, self.place(path));
    }

    /// Moves a viewport to a place, wherever the place came from.
    pub fn put(viewport: &mut Viewport, place: Place) {
        viewport.zoom = place.zoom;
        viewport.pan = place.pan;
        viewport.scroll_delta = Vec2::ZERO;
    }

    /// Drops what was remembered about an image that has left the collection.
    pub fn forget(&mut self, path: &Path) {
        self.places.remove(path);
    }

    pub fn clear(&mut self) {
        self.places.clear();
    }

    /// How many images are being remembered.
    pub fn len(&self) -> usize {
        self.places.len()
    }

    pub fn is_empty(&self) -> bool {
        self.places.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zoomed(zoom: f32, pan: Vec2) -> Viewport {
        Viewport {
            zoom,
            pan,
            ..Default::default()
        }
    }

    #[test]
    fn an_image_that_was_never_zoomed_is_not_remembered() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(1.0, Vec2::ZERO));

        assert_eq!(viewports.len(), 0);
        assert_eq!(viewports.place(Path::new("a.jpg")), Place::UNTOUCHED);
    }

    #[test]
    fn coming_back_finds_the_same_place() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(4.0, Vec2::new(120.0, -30.0)));

        let mut viewport = Viewport::default();
        viewports.restore(Path::new("a.jpg"), &mut viewport);

        assert_eq!(viewport.zoom, 4.0);
        assert_eq!(viewport.pan, Vec2::new(120.0, -30.0));
    }

    #[test]
    fn another_image_is_shown_whole() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(4.0, Vec2::new(10.0, 10.0)));

        let mut viewport = zoomed(4.0, Vec2::new(10.0, 10.0));
        viewports.restore(Path::new("b.jpg"), &mut viewport);

        assert_eq!(viewport.zoom, 1.0);
        assert_eq!(viewport.pan, Vec2::ZERO);
    }

    #[test]
    fn zooming_back_out_stops_the_image_being_remembered() {
        let mut viewports = Viewports::default();
        let path = Path::new("a.jpg");

        viewports.save(path, &zoomed(4.0, Vec2::new(10.0, 10.0)));
        assert_eq!(viewports.len(), 1);

        viewports.save(path, &zoomed(1.0, Vec2::new(10.0, 10.0)));
        assert_eq!(viewports.len(), 0);
    }

    #[test]
    fn a_stale_scroll_does_not_travel_with_the_place() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(2.0, Vec2::ZERO));

        let mut viewport = Viewport {
            scroll_delta: Vec2::new(50.0, 50.0),
            ..Default::default()
        };
        viewports.restore(Path::new("a.jpg"), &mut viewport);

        assert_eq!(viewport.scroll_delta, Vec2::ZERO);
    }

    #[test]
    fn a_place_can_be_put_on_a_viewport_from_anywhere() {
        // What repeating the last view does: the place of the picture just
        // left, applied to this one.
        let mut viewport = Viewport::default();
        Viewports::put(&mut viewport, Place::of(&zoomed(3.0, Vec2::new(8.0, 9.0))));

        assert_eq!(viewport.zoom, 3.0);
        assert_eq!(viewport.pan, Vec2::new(8.0, 9.0));
    }

    #[test]
    fn a_removed_image_is_forgotten() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(3.0, Vec2::ZERO));
        viewports.forget(Path::new("a.jpg"));

        assert_eq!(viewports.len(), 0);
    }
}
