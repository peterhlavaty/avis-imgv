//! Reading a resizable panel's size back, once the drag has finished.
//!
//! Shared because three panels want it and the naive version is wrong in a way
//! that is hard to see and easy to live with for a long time. Reading
//! `rect.width()` and writing it back whenever it differs from the configured
//! value looks right, and is a feedback loop: `show_animated` opens a panel by
//! growing it, so the first frames report a width part of the way there, that
//! gets written to the configuration, and the next frame's `default_width` is
//! the number the animation happened to be passing through. A panel set to 260
//! points came back at 298.66 after one run, and the history had two rows in it
//! saying the user had made the panel wider, before it had been touched.
//!
//! Waiting for the size to hold still is not enough either. An animation is
//! sampled at whatever rate the frames happen to run at, and a panel settles
//! more than once while a window is being laid out, so "it has not moved for a
//! few frames" catches both of those as well as a drag.
//!
//! What actually tells them apart is the pointer. A drag has a button held
//! down; an animation, a layout pass and a window being restored do not. So a
//! size is written back only when it moved while the button was down and has
//! since come to rest — which is also the convention the rest of the program
//! follows, a change landing on the frame the gesture ends.
//!
//! A width and a height are the same problem seen from two sides, which is why
//! this says size: the two side panels drag left and right and the filmstrip
//! drags up and down, and the reading is the same arithmetic on whichever of
//! the two numbers the panel's edge moves.

/// The size a panel was last seen at, and whether a hand moved it there.
#[derive(Debug, Default)]
pub struct Dragged {
    was: Option<f32>,
    /// Whether the size has moved while the pointer was down since it was
    /// last reported. Nothing is written back without this.
    by_hand: bool,
    /// Whether the size it came to rest at has already been reported.
    said: bool,
}

/// How far a size has to move to count as somebody having changed it.
///
/// Panels land on fractional sizes, and a point either way is not a decision.
const ENOUGH: f32 = 1.0;

impl Dragged {
    /// The size to write back, if a drag has finished at a new one.
    ///
    /// `None` while the button is still down, `None` when nothing but a layout
    /// or an animation moved it, `None` when where it came to rest is where it
    /// was told to be, and `None` on every frame after the one that reported it.
    pub fn settled(&mut self, size: f32, configured: f32, pointer_down: bool) -> Option<f32> {
        let moved = self
            .was
            .replace(size)
            .is_some_and(|was| (was - size).abs() >= f32::EPSILON);

        if moved && pointer_down {
            self.by_hand = true;
            self.said = false;
        }

        if pointer_down || !self.by_hand || self.said {
            return None;
        }

        if (size - configured).abs() <= ENOUGH {
            // It came back to where it was told to be. Nothing to write, and
            // nothing left to report about this drag.
            self.by_hand = false;
            return None;
        }

        self.said = true;
        self.by_hand = false;
        Some(size)
    }

    /// Forgets where the panel was, for when it has been shut.
    pub fn forget(&mut self) {
        self.was = None;
        self.by_hand = false;
        self.said = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drives a run of frames, each a size and whether the button was down,
    /// and answers with everything reported.
    fn frames(dragged: &mut Dragged, run: &[(f32, bool)], configured: f32) -> Vec<f32> {
        run.iter()
            .filter_map(|(size, down)| dragged.settled(*size, configured, *down))
            .collect()
    }

    /// A run of sizes with nobody touching anything.
    fn alone(sizes: &[f32]) -> Vec<(f32, bool)> {
        sizes.iter().map(|size| (*size, false)).collect()
    }

    /// The bug this exists for: a panel opening reports a different width on
    /// every frame, and none of those is anybody's decision.
    #[test]
    fn a_panel_being_opened_writes_nothing_back() {
        let mut dragged = Dragged::default();

        assert!(frames(&mut dragged, &alone(&[40.0, 120.0, 200.0, 245.0]), 260.0).is_empty());
    }

    /// The one that got through a check on stillness alone: a panel that comes
    /// to rest somewhere the configuration disagrees with, without anybody
    /// having touched it. A window being laid out does this more than once.
    #[test]
    fn a_panel_that_settles_by_itself_writes_nothing_back() {
        let mut dragged = Dragged::default();

        let said = frames(
            &mut dragged,
            &alone(&[40.0, 298.0, 298.0, 298.0, 298.0, 298.0, 298.0]),
            260.0,
        );

        assert!(said.is_empty(), "{said:?}");
    }

    /// And the one it exists to allow: moved with the button down, then let go.
    #[test]
    fn a_drag_is_written_back_once_the_button_comes_up() {
        let mut dragged = Dragged::default();

        let mut run = vec![(260.0, false), (280.0, true), (300.0, true), (320.0, true)];
        run.extend(alone(&[320.0, 320.0, 320.0]));

        assert_eq!(frames(&mut dragged, &run, 260.0), vec![320.0]);
    }

    #[test]
    fn nothing_is_written_while_the_button_is_still_down() {
        let mut dragged = Dragged::default();

        let run = [(260.0, false), (280.0, true), (300.0, true), (320.0, true)];

        assert!(frames(&mut dragged, &run, 260.0).is_empty());
    }

    /// A drag that ends back where it started is not a change.
    #[test]
    fn a_drag_that_comes_back_writes_nothing() {
        let mut dragged = Dragged::default();

        let mut run = vec![(260.0, false), (300.0, true), (260.0, true)];
        run.extend(alone(&[260.0, 260.0]));

        assert!(frames(&mut dragged, &run, 260.0).is_empty());
    }

    /// Nothing is written from a size first seen with the button already
    /// down: there is nothing to say it moved, and guessing would be a write
    /// nobody asked for.
    #[test]
    fn a_size_first_seen_mid_drag_writes_nothing() {
        let mut dragged = Dragged::default();

        let mut run = vec![(300.0, true)];
        run.extend(alone(&[300.0, 300.0, 300.0]));

        assert!(frames(&mut dragged, &run, 260.0).is_empty());
    }

    /// A second drag after the first is reported in its turn.
    #[test]
    fn moving_again_is_reported_again() {
        let mut dragged = Dragged::default();

        let mut first = vec![(260.0, false), (300.0, true)];
        first.extend(alone(&[300.0, 300.0]));
        assert_eq!(frames(&mut dragged, &first, 260.0), vec![300.0]);

        let mut again = vec![(340.0, true)];
        again.extend(alone(&[340.0, 340.0]));
        assert_eq!(frames(&mut dragged, &again, 300.0), vec![340.0]);
    }

    /// Sitting where it was dragged to writes once, not once a frame: the
    /// write is a row in the history, and a row a frame is what this is for.
    #[test]
    fn resting_after_a_drag_writes_once() {
        let mut dragged = Dragged::default();

        let mut run = vec![(260.0, false), (300.0, true)];
        run.extend(alone(&[300.0; 20]));

        assert_eq!(frames(&mut dragged, &run, 260.0), vec![300.0]);
    }

    #[test]
    fn a_shut_panel_is_forgotten_so_opening_it_starts_again() {
        let mut dragged = Dragged::default();

        let mut run = vec![(260.0, false), (300.0, true)];
        run.extend(alone(&[300.0, 300.0]));
        frames(&mut dragged, &run, 260.0);

        dragged.forget();

        assert!(frames(&mut dragged, &alone(&[300.0, 300.0]), 260.0).is_empty());
    }
}
