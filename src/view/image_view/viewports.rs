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

// `Place` is `crate::collection::place`: where a photograph was left is
// something the photograph carries, not something the view owns, and the
// history had to name this drawing module to read it.
pub use crate::collection::place::Place;

use crate::collection::place::Pan;

/// Where a viewport currently is.
///
/// The one constructor that needs a `Viewport`, which is a drawing type — so
/// it is here rather than on `Place`, and `Place` itself names nothing from
/// the toolkit.
pub fn place_of(viewport: &Viewport) -> Place {
    Place {
        zoom: viewport.zoom,
        pan: Pan(viewport.pan.x, viewport.pan.y),
    }
}

/// Whether the magnification and the corner travel with the viewer from one
/// photograph to the next.
///
/// Two toggles in the status bar rather than a page in the settings window,
/// because this is turned on for ten minutes and off again: going through a
/// burst at a hundred per cent, looking at the same eye in every frame. What a
/// photograph *opens* at is the other half of the question and is a
/// preference; this is a way of working, and it overrides the preference for
/// as long as it is on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Keep {
    /// The magnification carries over.
    pub zoom: bool,
    /// Where in the photograph the view is carries over.
    pub pan: bool,
}

/// Where the user got to in each image they zoomed.
#[derive(Debug, Default)]
pub struct Viewports {
    places: HashMap<PathBuf, Place>,
}

impl Viewports {
    /// Records where `path` was left, or forgets it if it was left untouched.
    pub fn save(&mut self, path: &Path, viewport: &Viewport) {
        let place = place_of(viewport);

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

    /// Puts the viewport onto the photograph being arrived at.
    ///
    /// Three answers, in the order they win: what the two toggles carry over
    /// from the picture just left, then where this photograph was itself left,
    /// and last the opening the settings give a photograph nobody has been
    /// into — which is the canvas's to apply, once it has measured the frame,
    /// and so is left to it by leaving `opened` alone.
    ///
    /// Carrying a magnification over beats putting one back, which sounds like
    /// the wrong way round until the toggle is on: somebody walking a burst at
    /// a hundred per cent asked for every frame at a hundred per cent, and a
    /// frame in the middle of it that was once looked at closely is not a
    /// reason to break the walk. Turning the toggle off puts every remembered
    /// place back in charge, having lost none of them.
    pub fn arrive(
        &self,
        path: Option<&Path>,
        viewport: &mut Viewport,
        keep: Keep,
        previous: Place,
    ) {
        let remembered = path.and_then(|path| self.places.get(path).copied());

        Self::put(viewport, remembered.unwrap_or(Place::UNTOUCHED));
        viewport.opened = remembered.is_some();

        if keep.zoom {
            viewport.zoom = previous.zoom;
            viewport.opened = true;
        }

        if keep.pan {
            viewport.pan = Vec2::new(previous.pan.x(), previous.pan.y());
        }
    }

    /// Moves a viewport to a place, wherever the place came from.
    pub fn put(viewport: &mut Viewport, place: Place) {
        viewport.zoom = place.zoom;
        viewport.pan = Vec2::new(place.pan.x(), place.pan.y());
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

    /// Arriving at a photograph nobody has kept anything for and nobody has
    /// been into: what it opens at is the canvas's to say, so it is left
    /// unopened.
    fn arrive_at(viewports: &Viewports, path: &str, viewport: &mut Viewport, keep: Keep) {
        viewports.arrive(Some(Path::new(path)), viewport, keep, Place::UNTOUCHED);
    }

    #[test]
    fn coming_back_finds_the_same_place() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(4.0, Vec2::new(120.0, -30.0)));

        let mut viewport = Viewport::default();
        arrive_at(&viewports, "a.jpg", &mut viewport, Keep::default());

        assert_eq!(viewport.zoom, 4.0);
        assert_eq!(viewport.pan, Vec2::new(120.0, -30.0));
        assert!(
            viewport.opened,
            "and the opening does not then draw over what was put back"
        );
    }

    #[test]
    fn another_image_is_shown_whole() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(4.0, Vec2::new(10.0, 10.0)));

        let mut viewport = zoomed(4.0, Vec2::new(10.0, 10.0));
        arrive_at(&viewports, "b.jpg", &mut viewport, Keep::default());

        assert_eq!(viewport.zoom, 1.0);
        assert_eq!(viewport.pan, Vec2::ZERO);
        assert!(
            !viewport.opened,
            "and what it opens at is left to the canvas, which has the window"
        );
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
        arrive_at(&viewports, "a.jpg", &mut viewport, Keep::default());

        assert_eq!(viewport.scroll_delta, Vec2::ZERO);
    }

    #[test]
    fn a_place_can_be_put_on_a_viewport_from_anywhere() {
        // What repeating the last view does: the place of the picture just
        // left, applied to this one.
        let mut viewport = Viewport::default();
        Viewports::put(&mut viewport, place_of(&zoomed(3.0, Vec2::new(8.0, 9.0))));

        assert_eq!(viewport.zoom, 3.0);
        assert_eq!(viewport.pan, Vec2::new(8.0, 9.0));
    }

    /// The zoom carries over to a photograph that has none of its own, and
    /// the opening is left alone: that is the whole of what the toggle is for.
    #[test]
    fn keeping_the_zoom_carries_it_to_the_next_photograph() {
        let viewports = Viewports::default();
        let mut viewport = Viewport::default();

        viewports.arrive(
            Some(Path::new("b.jpg")),
            &mut viewport,
            Keep {
                zoom: true,
                pan: false,
            },
            place_of(&zoomed(4.0, Vec2::new(80.0, 20.0))),
        );

        assert_eq!(viewport.zoom, 4.0);
        assert_eq!(viewport.pan, Vec2::ZERO, "the corner was not asked for");
        assert!(viewport.opened, "and the opening does not undo it");
    }

    /// A frame in the middle of a burst that was once looked at closely is
    /// not a reason to break a walk at one magnification.
    #[test]
    fn a_kept_zoom_beats_what_the_photograph_was_left_at() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("b.jpg"), &zoomed(9.0, Vec2::new(5.0, 5.0)));

        let mut viewport = Viewport::default();
        viewports.arrive(
            Some(Path::new("b.jpg")),
            &mut viewport,
            Keep {
                zoom: true,
                pan: false,
            },
            place_of(&zoomed(4.0, Vec2::ZERO)),
        );

        assert_eq!(viewport.zoom, 4.0);
        assert_eq!(
            viewports.place(Path::new("b.jpg")).zoom,
            9.0,
            "and where it was left is still remembered for when the toggle goes off"
        );
    }

    /// The corner without the magnification: what it opens at is still the
    /// canvas's to say, because the zoom was not decided here.
    #[test]
    fn keeping_the_pan_alone_leaves_the_opening_to_the_canvas() {
        let viewports = Viewports::default();
        let mut viewport = Viewport::default();

        viewports.arrive(
            Some(Path::new("b.jpg")),
            &mut viewport,
            Keep {
                zoom: false,
                pan: true,
            },
            place_of(&zoomed(4.0, Vec2::new(80.0, 20.0))),
        );

        assert_eq!(viewport.pan, Vec2::new(80.0, 20.0));
        assert!(!viewport.opened);
    }

    #[test]
    fn a_removed_image_is_forgotten() {
        let mut viewports = Viewports::default();
        viewports.save(Path::new("a.jpg"), &zoomed(3.0, Vec2::ZERO));
        viewports.forget(Path::new("a.jpg"));

        assert_eq!(viewports.len(), 0);
    }
}
