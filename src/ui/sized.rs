//! A resizable panel's size, and the whole of the rule for keeping it.
//!
//! egui records a panel's size as the rectangle its *contents* came to, not the
//! one the drag asked for, and it honours a `default_*` only while it has no
//! size of its own for that panel — which it does from the second frame on. So
//! a panel that wants to hold a configured width has four things to do, and
//! doing three of them leaves a fault that looks like something else:
//!
//! 1. offer the configured size as the default, for the first frame;
//! 2. state it *exactly* for one frame whenever it changes by any route that
//!    is not a drag, or the change never arrives;
//! 3. fill the size it is given, or the contents report their own width back
//!    and the edge springs out of the hand;
//! 4. read the dragged size back only when the button comes up, or an
//!    animation writes a half-open panel's width into the file.
//!
//! Four panels have a size. Between them they implemented the rule zero times:
//! the filmstrip had (2) and (3), the keyword panel (2), and the metadata and
//! history panels neither — so a width typed into the settings window reached
//! two of them at the next launch and not before. All four had (4), because
//! that one is `ui::dragged` and was already shared.
//!
//! This is the other three, in one type. A panel gets its size from a [`Sized`]
//! and cannot get part of the rule.
//!
//! # Taken by value
//!
//! [`Sized::asked_for`] consumes and hands back, rather than taking `&mut`.
//! Three of the four call sites draw inside a closure that calls a method on
//! the thing holding the size — `App`, or the panel's own state — so a mutable
//! borrow held across the closure is `E0499`. `Sized` is three words of `Copy`
//! data; passing it through costs nothing and compiles everywhere.

/// A panel's size, and whether it has just been changed by something other
/// than a drag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sized {
    size: f32,
    /// Set when the size moved by any route but a drag, cleared by the one
    /// frame that states it exactly.
    forced: bool,
}

impl Sized {
    /// A size as the configuration holds it.
    pub const fn of(size: f32) -> Sized {
        Sized {
            size,
            forced: false,
        }
    }

    /// What the panel should be drawn at.
    pub fn size(self) -> f32 {
        self.size
    }

    /// Notes that the size has moved by some route that is not a drag — the
    /// settings window, the history putting a panel back, a key that nudges
    /// it — so the next frame states it rather than suggesting it.
    ///
    /// Nothing happens where the size has not actually changed: a settings
    /// save that moved something else must not cost every panel an exact
    /// frame, because an exact frame is one the user cannot drag.
    pub fn moved_to(self, size: f32) -> Sized {
        if (self.size - size).abs() <= UNCHANGED {
            return self;
        }

        Sized { size, forced: true }
    }

    /// What a drag left it at.
    ///
    /// Never forces: the panel is already the size the hand put it, and
    /// stating it back would fight the next frame of the same drag.
    pub fn dragged_to(self, size: f32) -> Sized {
        Sized {
            size,
            forced: false,
        }
    }

    /// The size to draw at, and whether this is the frame that must state it.
    ///
    /// Takes the flag, so the exact frame happens once. Consumes and hands
    /// back — see the note at the top of this file about `E0499`.
    pub fn asked_for(self) -> (Sized, f32, bool) {
        (
            Sized {
                forced: false,
                ..self
            },
            self.size,
            self.forced,
        )
    }

    /// Whether the next frame will state the size exactly.
    pub fn is_forced(self) -> bool {
        self.forced
    }
}

/// How far two sizes must differ to count as a change.
///
/// Half a point. Below that it is a layout pass rounding, and treating it as a
/// change costs an exact frame — which is a frame the user cannot drag in.
const UNCHANGED: f32 = 0.5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_starts_unforced() {
        let held = Sized::of(320.0);

        assert_eq!(held.size(), 320.0);
        assert!(!held.is_forced());
    }

    /// The half of the rule two panels were missing: without it, egui keeps
    /// its own width from the second frame on and the settings window reaches
    /// the panel at the next launch.
    #[test]
    fn a_size_changed_from_elsewhere_is_stated_once() {
        let held = Sized::of(320.0).moved_to(480.0);
        assert!(held.is_forced());

        let (held, size, forced) = held.asked_for();
        assert_eq!(size, 480.0);
        assert!(forced, "the one frame that states it");

        let (_, _, forced) = held.asked_for();
        assert!(!forced, "and only that frame");
    }

    /// A settings save that moved something else must not cost every panel an
    /// exact frame, because an exact frame is one the hand cannot drag.
    #[test]
    fn a_size_that_did_not_move_is_not_forced() {
        let held = Sized::of(320.0).moved_to(320.0);

        assert!(!held.is_forced());
    }

    #[test]
    fn a_rounding_of_less_than_half_a_point_is_not_a_change() {
        let held = Sized::of(320.0).moved_to(320.3);

        assert!(!held.is_forced());
        assert_eq!(held.size(), 320.0, "and the old value is kept");
    }

    /// The panel is already the size the hand put it, and stating it back
    /// would fight the next frame of the same drag.
    #[test]
    fn a_drag_never_forces() {
        let held = Sized::of(320.0).dragged_to(500.0);

        assert_eq!(held.size(), 500.0);
        assert!(!held.is_forced());
    }

    /// A drag while a forced frame is pending takes the flag with it: the
    /// panel is where the hand left it, not where the file said.
    #[test]
    fn a_drag_settles_a_pending_force() {
        let held = Sized::of(320.0).moved_to(480.0).dragged_to(500.0);

        assert_eq!(held.size(), 500.0);
        assert!(!held.is_forced());
    }

    #[test]
    fn two_changes_before_a_frame_are_one_forced_frame() {
        let held = Sized::of(320.0).moved_to(400.0).moved_to(480.0);

        let (held, size, forced) = held.asked_for();
        assert_eq!(size, 480.0);
        assert!(forced);
        assert!(!held.is_forced());
    }
}
