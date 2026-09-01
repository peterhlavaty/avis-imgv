//! What the left button does to the marking.
//!
//! Kept apart from egui so the whole of it can be tested without a window:
//! [`Pointing`] is what the frame saw and [`Answered`] is what the view has to
//! do about it. The view reads the one and carries out the other; nothing in
//! between needs a context.
//!
//! Five gestures, and they are the five a person would guess: drag on the
//! photograph to mark an area, drag a side or a corner of the marking to move
//! it, click inside to magnify until it fills the panel, click outside to
//! forget it, and the second button anywhere inside for the menu.

use eframe::egui::{CursorIcon, Rect};
use eframe::epaint::{Pos2, Vec2};

use crate::view::image_view::canvas::Metrics;

use super::grip::Grip;
use super::{Area, Doing, MEANT, REACH, SMALLEST};

/// What the pointer did this frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct Pointing {
    pub at: Pos2,
    /// The left button went down this frame.
    pub pressed: bool,
    /// It is still down.
    pub down: bool,
    /// It came up this frame.
    pub released: bool,
    /// egui has decided the press is a drag rather than a click.
    pub dragging: bool,
    /// The press ended without travelling far enough to be a drag.
    pub clicked: bool,
}

/// What the view has to do about it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Answered {
    /// Magnify until the marking fills the panel.
    pub zoom_to_it: bool,
    /// What the pointer should look like over the photograph.
    pub cursor: Option<CursorIcon>,
}

impl Area {
    /// Reads the frame and moves the marking on.
    ///
    /// `may_draw` is whether the left button is free to start a new marking.
    /// It is never free while the same drag would be moving the photograph
    /// instead — one press is one gesture — and the setting decides the rest.
    /// Taking hold of a marking that already exists, and the two clicks, do
    /// not ask: they are about something the user put there.
    pub fn look(&mut self, metrics: &Metrics, pointing: &Pointing, may_draw: bool) -> Answered {
        let mut answered = Answered::default();

        if metrics.rect.width() <= 0.0 || metrics.rect.height() <= 0.0 {
            self.doing = None;
            return answered;
        }

        let started_on = self
            .on_screen(metrics)
            .and_then(|rect| Grip::at(rect, pointing.at, REACH));

        if pointing.pressed {
            self.doing = match started_on {
                Some(grip) => Some(Doing::Resizing(grip)),
                None if may_draw && metrics.rect.contains(pointing.at) => {
                    super::to_image(metrics, pointing.at)
                        .map(super::inside_unit)
                        .map(Doing::Drawing)
                }
                None => None,
            };
        }

        if pointing.down && pointing.dragging {
            self.carry_on(metrics, pointing.at);
        }

        if pointing.released {
            self.finish(metrics, pointing, &mut answered);
        }

        answered.cursor = self.cursor(metrics, pointing, may_draw);
        answered
    }

    /// Moves what the press took hold of to where the pointer is now.
    fn carry_on(&mut self, metrics: &Metrics, at: Pos2) {
        let Some(at) = super::to_image(metrics, at).map(super::inside_unit) else {
            return;
        };

        self.marked = match (self.doing, self.marked) {
            (Some(Doing::Drawing(anchor)), _) => Some(Rect::from_two_pos(anchor, at)),
            (Some(Doing::Resizing(grip)), Some(marked)) => {
                Some(grip.moved(marked, at, Vec2::splat(SMALLEST)))
            }
            _ => return,
        };
    }

    /// The button came up: either the gesture ends, or it was a click and
    /// means one of the two things a click on a marking means.
    fn finish(&mut self, metrics: &Metrics, pointing: &Pointing, answered: &mut Answered) {
        let was = self.doing.take();

        if pointing.clicked {
            // Only a marking answers a click. Without one the click means
            // whatever else it meant, which is nothing here.
            match self.on_screen(metrics) {
                Some(rect) if rect.expand(REACH).contains(pointing.at) => {
                    answered.zoom_to_it = true
                }
                Some(_) => self.clear(),
                None => {}
            }

            return;
        }

        // A drag that drew almost nothing was a click that wandered, and a
        // rectangle four points across is not what anybody was pointing at.
        if matches!(was, Some(Doing::Drawing(_))) && self.too_small(metrics) {
            self.clear();
        }
    }

    /// Whether what was drawn is too small to have been meant.
    fn too_small(&self, metrics: &Metrics) -> bool {
        match self.on_screen(metrics) {
            Some(rect) => rect.width() < MEANT || rect.height() < MEANT,
            None => true,
        }
    }

    /// What the pointer should look like, which is the same question the hit
    /// test answers and so is asked of the same function.
    ///
    /// The cross is the whole of the answer to the one objection against
    /// giving the left drag a second meaning: a rule about the size of the
    /// photograph is an invisible mode unless something on screen says which
    /// of the two the button is about, and the pointer is that something —
    /// before the first rectangle exists, not only after it.
    fn cursor(&self, metrics: &Metrics, pointing: &Pointing, may_draw: bool) -> Option<CursorIcon> {
        match self.doing {
            Some(Doing::Resizing(grip)) => return Some(grip.cursor()),
            Some(Doing::Drawing(_)) => return Some(CursorIcon::Crosshair),
            None => {}
        }

        if let Some(rect) = self.on_screen(metrics) {
            if let Some(grip) = Grip::at(rect, pointing.at, REACH) {
                return Some(grip.cursor());
            }

            // A cross, because the next click magnifies about what it is over.
            if rect.contains(pointing.at) {
                return Some(CursorIcon::Crosshair);
            }
        }

        (may_draw && metrics.rect.contains(pointing.at)).then_some(CursorIcon::Crosshair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 4000x2000 photograph fitted into an 800x800 panel, the picture
    /// letterboxed 200 points down.
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

    fn press(at: Pos2) -> Pointing {
        Pointing {
            at,
            pressed: true,
            down: true,
            ..Default::default()
        }
    }

    fn drag(at: Pos2) -> Pointing {
        Pointing {
            at,
            down: true,
            dragging: true,
            ..Default::default()
        }
    }

    fn release(at: Pos2, clicked: bool) -> Pointing {
        Pointing {
            at,
            released: true,
            clicked,
            ..Default::default()
        }
    }

    /// Presses at one corner, drags to the other, lets go.
    fn drawn(area: &mut Area, from: Pos2, to: Pos2) {
        let metrics = fitted();

        area.look(&metrics, &press(from), true);
        area.look(&metrics, &drag(to), true);
        area.look(&metrics, &release(to, false), true);
    }

    #[test]
    fn a_drag_marks_out_what_it_crossed() {
        let mut area = Area::default();
        drawn(&mut area, Pos2::new(200.0, 300.0), Pos2::new(600.0, 500.0));

        let marked = area.marked().expect("a marking");
        assert!((marked.min.x - 0.25).abs() < 0.001, "{marked:?}");
        assert!((marked.max.x - 0.75).abs() < 0.001, "{marked:?}");
        assert!((marked.min.y - 0.25).abs() < 0.001, "{marked:?}");
        assert!((marked.max.y - 0.75).abs() < 0.001, "{marked:?}");
    }

    /// Backwards is the same rectangle: which corner the hand started at is
    /// not a fact about the area it drew.
    #[test]
    fn a_drag_the_other_way_marks_the_same_thing() {
        let (mut one, mut other) = (Area::default(), Area::default());

        drawn(&mut one, Pos2::new(200.0, 300.0), Pos2::new(600.0, 500.0));
        drawn(&mut other, Pos2::new(600.0, 500.0), Pos2::new(200.0, 300.0));

        assert_eq!(one.marked(), other.marked());
    }

    #[test]
    fn a_drag_off_the_photograph_marks_up_to_the_edge_of_it() {
        let mut area = Area::default();
        drawn(
            &mut area,
            Pos2::new(400.0, 400.0),
            Pos2::new(2000.0, 2000.0),
        );

        let marked = area.marked().expect("a marking");
        assert_eq!(marked.max, Pos2::new(1.0, 1.0));
    }

    /// The setting, and the rule that a drag which would move the photograph
    /// is not also a drag that marks it.
    #[test]
    fn nothing_is_drawn_when_the_drag_is_not_free() {
        let metrics = fitted();
        let mut area = Area::default();

        area.look(&metrics, &press(Pos2::new(200.0, 300.0)), false);
        area.look(&metrics, &drag(Pos2::new(600.0, 500.0)), false);

        assert_eq!(area.marked(), None);
        assert!(!area.is_dragging());
    }

    /// A press that wandered four points is a click, and a click does not
    /// leave a rectangle behind.
    #[test]
    fn a_drag_too_small_to_have_been_meant_marks_nothing() {
        let mut area = Area::default();
        drawn(&mut area, Pos2::new(400.0, 400.0), Pos2::new(402.0, 401.0));

        assert_eq!(area.marked(), None);
    }

    #[test]
    fn a_click_inside_it_asks_for_the_zoom() {
        let metrics = fitted();
        let mut area = Area::default();
        drawn(&mut area, Pos2::new(200.0, 300.0), Pos2::new(600.0, 500.0));

        let answered = area.look(&metrics, &release(Pos2::new(400.0, 400.0), true), true);

        assert!(answered.zoom_to_it);
        assert!(area.marked().is_some(), "the marking survives the zoom");
    }

    #[test]
    fn a_click_outside_it_forgets_it() {
        let metrics = fitted();
        let mut area = Area::default();
        drawn(&mut area, Pos2::new(200.0, 300.0), Pos2::new(600.0, 500.0));

        let answered = area.look(&metrics, &release(Pos2::new(700.0, 550.0), true), true);

        assert!(!answered.zoom_to_it);
        assert_eq!(area.marked(), None);
    }

    /// Taking hold of a side moves that side and leaves the other three.
    #[test]
    fn a_side_is_dragged_by_taking_hold_of_it() {
        let metrics = fitted();
        let mut area = Area::default();
        drawn(&mut area, Pos2::new(200.0, 300.0), Pos2::new(600.0, 500.0));

        let before = area.marked().expect("a marking");

        area.look(&metrics, &press(Pos2::new(200.0, 400.0)), true);
        assert!(area.is_dragging(), "the press took hold of the side");

        area.look(&metrics, &drag(Pos2::new(320.0, 400.0)), true);
        area.look(&metrics, &release(Pos2::new(320.0, 400.0), false), true);

        let after = area.marked().expect("a marking");
        assert!((after.min.x - 0.4).abs() < 0.001, "{after:?}");
        assert_eq!(after.max, before.max);
        assert_eq!(after.min.y, before.min.y);
    }

    /// And a side can be taken hold of whether or not a new marking could be
    /// drawn: it is about something the user already put there.
    #[test]
    fn a_side_can_be_dragged_even_when_a_new_one_could_not_be_drawn() {
        let metrics = fitted();
        let mut area = Area::default();
        drawn(&mut area, Pos2::new(200.0, 300.0), Pos2::new(600.0, 500.0));

        area.look(&metrics, &press(Pos2::new(200.0, 400.0)), false);

        assert!(area.is_dragging());
    }

    #[test]
    fn the_pointer_says_what_the_button_would_do() {
        let metrics = fitted();
        let mut area = Area::default();
        drawn(&mut area, Pos2::new(200.0, 300.0), Pos2::new(600.0, 500.0));

        let over = |at: Pos2| {
            area.clone()
                .look(
                    &metrics,
                    &Pointing {
                        at,
                        ..Default::default()
                    },
                    true,
                )
                .cursor
        };

        assert_eq!(over(Pos2::new(400.0, 400.0)), Some(CursorIcon::Crosshair));
        assert_eq!(over(Pos2::new(200.0, 400.0)), Some(Grip::Left.cursor()));
        assert_eq!(over(Pos2::new(200.0, 300.0)), Some(Grip::TopLeft.cursor()));
        // Elsewhere on the photograph a drag would draw another one, which the
        // same cross says.
        assert_eq!(over(Pos2::new(700.0, 550.0)), Some(CursorIcon::Crosshair));
        // And off the photograph it says nothing at all.
        assert_eq!(over(Pos2::new(400.0, 50.0)), None);
    }

    /// The answer to the one objection against giving the left drag a second
    /// meaning: with nothing marked yet, the pointer still says which of the
    /// two the button is about, so the rule is not an invisible mode.
    #[test]
    fn the_pointer_says_a_drag_would_mark_before_anything_is_marked() {
        let metrics = fitted();
        let mut area = Area::default();
        let resting = Pointing {
            at: Pos2::new(400.0, 400.0),
            ..Default::default()
        };

        assert_eq!(
            area.look(&metrics, &resting, true).cursor,
            Some(CursorIcon::Crosshair)
        );

        // And says nothing where the same drag would move the photograph.
        assert_eq!(area.look(&metrics, &resting, false).cursor, None);
    }

    /// Nothing is marked before a frame has been drawn to mark it on.
    #[test]
    fn a_press_before_the_first_frame_does_nothing() {
        let mut area = Area::default();
        let answered = area.look(&Metrics::default(), &press(Pos2::new(10.0, 10.0)), true);

        assert_eq!(answered, Answered::default());
        assert_eq!(area.marked(), None);
    }
}
