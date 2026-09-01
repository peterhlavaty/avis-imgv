//! The eight places a marked area can be taken hold of.
//!
//! Four sides and four corners, because a corner is two sides and moving one
//! side at a time to arrive at it is two gestures for one intention. Which one
//! the pointer is on decides both what the drag does and what the cursor says
//! it will do, so the hit test and the cursor are one answer here rather than
//! two answers that can disagree.

use eframe::egui::CursorIcon;
use eframe::epaint::{Pos2, Rect, Vec2};

/// A side or a corner of the marking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grip {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Grip {
    /// Every one of them, corners first, which is the order the hit test wants
    /// them in.
    pub const ALL: [Grip; 8] = [
        Grip::TopLeft,
        Grip::TopRight,
        Grip::BottomLeft,
        Grip::BottomRight,
        Grip::Left,
        Grip::Right,
        Grip::Top,
        Grip::Bottom,
    ];

    /// Which one the pointer is on, if any.
    ///
    /// Corners before sides: near a corner both of its sides answer, and the
    /// corner is what the hand was aiming at.
    pub fn at(area: Rect, pointer: Pos2, reach: f32) -> Option<Grip> {
        if !area.expand(reach).contains(pointer) {
            return None;
        }

        let near = |edge: f32, at: f32| (edge - at).abs() <= reach;
        let left = near(area.min.x, pointer.x);
        let right = near(area.max.x, pointer.x);
        let top = near(area.min.y, pointer.y);
        let bottom = near(area.max.y, pointer.y);

        Some(match (left, right, top, bottom) {
            (true, _, true, _) => Grip::TopLeft,
            (_, true, true, _) => Grip::TopRight,
            (true, _, _, true) => Grip::BottomLeft,
            (_, true, _, true) => Grip::BottomRight,
            (true, ..) => Grip::Left,
            (_, true, ..) => Grip::Right,
            (_, _, true, _) => Grip::Top,
            (_, _, _, true) => Grip::Bottom,
            _ => return None,
        })
    }

    /// What the pointer says it will do.
    pub fn cursor(self) -> CursorIcon {
        match self {
            Grip::Left | Grip::Right => CursorIcon::ResizeHorizontal,
            Grip::Top | Grip::Bottom => CursorIcon::ResizeVertical,
            Grip::TopLeft | Grip::BottomRight => CursorIcon::ResizeNwSe,
            Grip::TopRight | Grip::BottomLeft => CursorIcon::ResizeNeSw,
        }
    }

    /// Which of the four sides this one moves.
    fn moves(self) -> (bool, bool, bool, bool) {
        match self {
            Grip::Left => (true, false, false, false),
            Grip::Right => (false, true, false, false),
            Grip::Top => (false, false, true, false),
            Grip::Bottom => (false, false, false, true),
            Grip::TopLeft => (true, false, true, false),
            Grip::TopRight => (false, true, true, false),
            Grip::BottomLeft => (true, false, false, true),
            Grip::BottomRight => (false, true, false, true),
        }
    }

    /// Puts the side or the corner this holds where the pointer is.
    ///
    /// Everything is in the photograph's own coordinates, so `smallest` is
    /// too. A side stops `smallest` short of the one opposite rather than
    /// crossing it: a rectangle turned inside out by a drag is a rectangle
    /// whose grips have quietly swapped places, and the hand holding it has
    /// not been told.
    pub fn moved(self, area: Rect, to: Pos2, smallest: Vec2) -> Rect {
        let (mut min, mut max) = (area.min, area.max);
        let (left, right, top, bottom) = self.moves();

        if left {
            min.x = to.x.clamp(0.0, (max.x - smallest.x).max(0.0));
        }
        if right {
            max.x = to.x.clamp((min.x + smallest.x).min(1.0), 1.0);
        }
        if top {
            min.y = to.y.clamp(0.0, (max.y - smallest.y).max(0.0));
        }
        if bottom {
            max.y = to.y.clamp((min.y + smallest.y).min(1.0), 1.0);
        }

        Rect::from_min_max(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(300.0, 200.0))
    }

    const SMALLEST: Vec2 = Vec2::splat(0.002);

    #[test]
    fn a_side_is_found_from_either_way_of_it() {
        let just_outside = Pos2::new(96.0, 150.0);
        let just_inside = Pos2::new(104.0, 150.0);

        assert_eq!(Grip::at(area(), just_outside, 8.0), Some(Grip::Left));
        assert_eq!(Grip::at(area(), just_inside, 8.0), Some(Grip::Left));
    }

    #[test]
    fn a_corner_wins_over_the_two_sides_that_meet_there() {
        assert_eq!(
            Grip::at(area(), Pos2::new(102.0, 102.0), 8.0),
            Some(Grip::TopLeft)
        );
        assert_eq!(
            Grip::at(area(), Pos2::new(298.0, 198.0), 8.0),
            Some(Grip::BottomRight)
        );
    }

    #[test]
    fn the_middle_and_the_outside_are_no_grip_at_all() {
        assert_eq!(Grip::at(area(), Pos2::new(200.0, 150.0), 8.0), None);
        assert_eq!(Grip::at(area(), Pos2::new(40.0, 150.0), 8.0), None);
    }

    /// The reach does not run along the side past the end of it, or the whole
    /// of a screen edge would answer for a marking in the corner of it.
    #[test]
    fn the_reach_stops_at_the_end_of_the_side() {
        assert_eq!(Grip::at(area(), Pos2::new(100.0, 400.0), 8.0), None);
    }

    #[test]
    fn one_side_moves_and_the_other_three_stay() {
        let marked = Rect::from_min_max(Pos2::new(0.2, 0.2), Pos2::new(0.8, 0.8));
        let moved = Grip::Left.moved(marked, Pos2::new(0.4, 0.5), SMALLEST);

        assert_eq!(moved, Rect::from_min_max(Pos2::new(0.4, 0.2), marked.max));
    }

    #[test]
    fn a_corner_moves_both_of_its_sides() {
        let marked = Rect::from_min_max(Pos2::new(0.2, 0.2), Pos2::new(0.8, 0.8));
        let moved = Grip::TopRight.moved(marked, Pos2::new(0.6, 0.4), SMALLEST);

        assert_eq!(
            moved,
            Rect::from_min_max(Pos2::new(0.2, 0.4), Pos2::new(0.6, 0.8))
        );
    }

    /// Dragged past the far side, a side stops rather than turning the
    /// rectangle inside out.
    #[test]
    fn a_side_never_crosses_the_one_opposite() {
        let marked = Rect::from_min_max(Pos2::new(0.2, 0.2), Pos2::new(0.8, 0.8));

        for grip in Grip::ALL {
            let far = grip.moved(marked, Pos2::new(5.0, 5.0), SMALLEST);
            let back = grip.moved(marked, Pos2::new(-5.0, -5.0), SMALLEST);

            for moved in [far, back] {
                assert!(moved.width() >= SMALLEST.x * 0.99, "{grip:?} {moved:?}");
                assert!(moved.height() >= SMALLEST.y * 0.99, "{grip:?} {moved:?}");
            }
        }
    }

    /// And never off the photograph, whichever way it is dragged.
    #[test]
    fn a_marking_stays_on_the_photograph() {
        let marked = Rect::from_min_max(Pos2::new(0.2, 0.2), Pos2::new(0.8, 0.8));

        for grip in Grip::ALL {
            for to in [Pos2::new(-3.0, -3.0), Pos2::new(4.0, 4.0)] {
                let moved = grip.moved(marked, to, SMALLEST);

                assert!(moved.min.x >= 0.0 && moved.min.y >= 0.0, "{moved:?}");
                assert!(moved.max.x <= 1.0 && moved.max.y <= 1.0, "{moved:?}");
            }
        }
    }

    /// A marking pressed flat against an edge still has a grip that can pull
    /// it back, rather than a clamp with its ends the wrong way round.
    #[test]
    fn a_marking_against_the_edge_does_not_panic() {
        let flat = Rect::from_min_max(Pos2::new(1.0, 1.0), Pos2::new(1.0, 1.0));

        for grip in Grip::ALL {
            let moved = grip.moved(flat, Pos2::new(0.5, 0.5), SMALLEST);
            assert!(moved.min.x <= moved.max.x, "{grip:?} {moved:?}");
        }
    }

    #[test]
    fn every_grip_says_which_way_it_moves() {
        for grip in Grip::ALL {
            let (left, right, top, bottom) = grip.moves();
            assert!(left || right || top || bottom, "{grip:?} moves nothing");
            assert!(!(left && right), "{grip:?} moves both sides");
            assert!(!(top && bottom), "{grip:?} moves both ends");
        }
    }
}
