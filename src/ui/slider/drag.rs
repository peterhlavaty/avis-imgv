//! How far the pointer travels to move a handle, and what happens when it runs
//! out of screen.
//!
//! A slider is the one control in this program where the pointer and the value
//! are bound together, and the binding is what makes it hard to use. Two
//! hundred points of rail carrying four thousand values gives twenty of them to
//! every point the pointer moves, and no hand places a pointer to the point. So
//! the handle takes a *share* of what the pointer does: `mouse.slider_travel`
//! says how far the pointer goes to cross the whole rail, as a multiple of the
//! rail's own length, and one is the old behaviour of the two moving together.
//!
//! Which leaves the pointer free to run further than the rail is long, and so
//! to run out of window. A drag that reached the edge with rail still to cover
//! would simply stop, so the pointer is put back on the other side and the
//! movement across the gap is read as the small step it was rather than as the
//! width of the window. That reading is the whole of the arithmetic here:
//! [`moved`] is `now - was` unless that is more than half a window, in which
//! case a wrap happened and the width comes off it.
//!
//! Nothing records that a wrap was asked for, which is what makes this right on
//! a platform that ignores the ask. Wayland has no cursor warping; there the
//! jump never arrives, no correction is made for one, and the drag stops at the
//! edge exactly as it does today.

use eframe::egui::Rangef;

/// The travel at which the handle and the pointer move together.
pub const BOUND: f32 = 1.0;

/// What a fresh configuration asks for.
///
/// Three rather than two because two is not enough to be felt on the short
/// rails, and not five because a rail is also how somebody sweeps to the far
/// end of a range, and five makes that a journey.
pub const SHIPS_AS: f32 = 3.0;

/// The furthest the settings window will go.
pub const FURTHEST: f32 = 20.0;

/// What the menu on a rail offers: the same decision as the setting, with the
/// numbers somebody would actually pick.
pub const CHOICES: &[(f32, &str)] = &[
    (1.0, "Bound to the pointer"),
    (2.0, "Twice the distance"),
    (3.0, "Three times"),
    (5.0, "Five times"),
    (10.0, "Ten times"),
];

/// How close to the edge of the window the pointer has to get to be put back.
///
/// One point: the wrap is meant to happen as the pointer arrives at the edge
/// and not before, or a drag with plenty of screen left would teleport.
const AT_THE_EDGE: f32 = 1.0;

/// How far inside the far edge it lands.
///
/// Further in than [`AT_THE_EDGE`], and that is the whole reason there are two
/// numbers: landing inside the strip that triggers a wrap would wrap it
/// straight back, and the pointer would sit in the corner flickering between
/// the two sides of the screen.
const LANDS: f32 = 8.0;

/// A travel that can be divided by, whatever the file said.
///
/// The file is hand-editable and a number below one would make the handle
/// outrun the pointer, which is not a thing anybody asked for; nought would
/// divide by nought.
pub fn sane(travel: f32) -> f32 {
    if travel.is_finite() && travel > BOUND {
        travel
    } else {
        BOUND
    }
}

/// The share of the pointer's movement the handle takes.
pub fn gain(travel: f32) -> f32 {
    1.0 / sane(travel)
}

/// How far the pointer actually moved.
///
/// A jump of more than half the window is not a hand: it is the pointer having
/// been put back on the other side, and what it really did was carry on in the
/// direction it was already going.
pub fn moved(now: f32, was: f32, span: Rangef) -> f32 {
    let width = span.span();
    let step = now - was;

    if width <= 0.0 {
        return step;
    }

    if step > width / 2.0 {
        step - width
    } else if step < -width / 2.0 {
        step + width
    } else {
        step
    }
}

/// Where to put the pointer when it has run out of window, if it has.
pub fn wrap(now: f32, span: Rangef) -> Option<f32> {
    // A window too narrow to hold both margins has nowhere to put it that is
    // not immediately the other edge.
    if span.span() < LANDS * 4.0 {
        return None;
    }

    if now >= span.max - AT_THE_EDGE {
        Some(span.min + LANDS)
    } else if now <= span.min + AT_THE_EDGE {
        Some(span.max - LANDS)
    } else {
        None
    }
}

/// Where the drag has got to on the rail after this much pointer movement.
pub fn along(at: f32, moved: f32, gain: f32, rail: Rangef) -> f32 {
    rail.clamp(at + moved * gain)
}

/// Where on the rail a value sits: nought at the left end and one at the right.
fn normalised(value: f64, range: (f64, f64), logarithmic: bool) -> f64 {
    let (min, max) = range;

    if min >= max {
        return 0.0;
    }

    let value = value.clamp(min, max);

    if logarithmic && min > 0.0 {
        (value.log10() - min.log10()) / (max.log10() - min.log10())
    } else {
        (value - min) / (max - min)
    }
}

/// The value at a point between the ends of the rail.
fn valued(at: f64, range: (f64, f64), logarithmic: bool) -> f64 {
    let (min, max) = range;

    if min >= max {
        return min;
    }

    let at = at.clamp(0.0, 1.0);

    if logarithmic && min > 0.0 {
        10f64.powf(min.log10() + at * (max.log10() - min.log10()))
    } else {
        min + at * (max - min)
    }
}

/// The value a point on the rail stands for.
///
/// A logarithmic rail reaching nought or below is read as a linear one. There
/// is exactly one logarithmic rail in the program, it runs from one per cent,
/// and the cases the toolkit carries for ranges that straddle nought have no
/// caller here to keep them honest.
pub fn value_at(position: f32, rail: Rangef, range: (f64, f64), logarithmic: bool) -> f64 {
    let at = if rail.span() > 0.0 {
        f64::from((position - rail.min) / rail.span())
    } else {
        0.0
    };

    valued(at, range, logarithmic)
}

/// The point on the rail a value sits at.
pub fn position_of(value: f64, rail: Rangef, range: (f64, f64), logarithmic: bool) -> f32 {
    rail.min + rail.span() * normalised(value, range, logarithmic) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAIL: Rangef = Rangef {
        min: 100.0,
        max: 300.0,
    };
    const WINDOW: Rangef = Rangef {
        min: 0.0,
        max: 1000.0,
    };

    /// The whole of the change, in one line: three times the travel is a third
    /// of the movement.
    #[test]
    fn the_handle_takes_a_share_of_what_the_pointer_does() {
        assert_eq!(along(100.0, 60.0, gain(3.0), RAIL), 120.0);
        assert_eq!(along(100.0, 60.0, gain(1.0), RAIL), 160.0);
    }

    /// A file saying nought, or nothing, or something absurd cannot make the
    /// handle outrun the pointer.
    #[test]
    fn a_travel_below_one_is_read_as_one() {
        assert_eq!(sane(0.0), BOUND);
        assert_eq!(sane(-4.0), BOUND);
        assert_eq!(sane(f32::NAN), BOUND);
        assert_eq!(sane(0.5), BOUND);
        assert_eq!(sane(4.0), 4.0);
    }

    /// Both ends hold: a drag that runs on past the end of the rail stays at
    /// the end of the rail.
    #[test]
    fn the_drag_stops_at_the_ends_of_the_rail() {
        assert_eq!(along(290.0, 400.0, gain(1.0), RAIL), 300.0);
        assert_eq!(along(110.0, -400.0, gain(1.0), RAIL), 100.0);
    }

    /// An ordinary movement is itself.
    #[test]
    fn a_movement_that_did_not_cross_the_edge_is_left_alone() {
        assert_eq!(moved(520.0, 500.0, WINDOW), 20.0);
        assert_eq!(moved(480.0, 500.0, WINDOW), -20.0);
    }

    /// The pointer left the right edge and came back at the left: what it did
    /// was carry on to the right.
    #[test]
    fn a_wrap_is_read_as_the_small_step_it_was() {
        // Reached 999, was put back at 8, and moved three further meanwhile.
        assert_eq!(moved(11.0, 999.0, WINDOW), 12.0);
        // And the same going the other way.
        assert_eq!(moved(989.0, 1.0, WINDOW), -12.0);
    }

    /// The wrap is asked for at the edge and nowhere before it.
    #[test]
    fn the_pointer_is_put_back_only_at_the_edge() {
        assert_eq!(wrap(500.0, WINDOW), None);
        assert_eq!(wrap(1000.0, WINDOW), Some(LANDS));
        assert_eq!(wrap(0.0, WINDOW), Some(1000.0 - LANDS));
    }

    /// Where it lands is not itself the edge, or the drag would spend the rest
    /// of its life bouncing between the two sides of the screen.
    #[test]
    fn where_it_lands_does_not_wrap_it_straight_back() {
        let landed = wrap(1000.0, WINDOW).expect("it wrapped");
        assert_eq!(wrap(landed, WINDOW), None);

        let landed = wrap(0.0, WINDOW).expect("it wrapped");
        assert_eq!(wrap(landed, WINDOW), None);
    }

    /// A window with no room for the two margins is left alone rather than
    /// having the pointer thrown about inside it.
    #[test]
    fn a_window_too_narrow_to_wrap_in_is_not_wrapped() {
        let narrow = Rangef::new(0.0, 20.0);
        assert_eq!(wrap(20.0, narrow), None);
        assert_eq!(wrap(0.0, narrow), None);
    }

    /// The two halves of the mapping agree, at the ends and in between.
    #[test]
    fn a_position_and_a_value_are_the_same_thing_twice() {
        let range = (0.0, 100.0);
        for value in [0.0, 25.0, 50.0, 99.0, 100.0] {
            let position = position_of(value, RAIL, range, false);
            let back = value_at(position, RAIL, range, false);
            assert!((back - value).abs() < 1e-4, "{value} came back as {back}");
        }
    }

    /// And on a logarithmic rail, where the middle of the rail is the
    /// geometric middle of the range rather than the arithmetic one.
    #[test]
    fn a_logarithmic_rail_puts_the_middle_of_the_range_where_it_belongs() {
        let range = (1.0, 100.0);
        let middle = value_at(RAIL.center(), RAIL, range, true);
        assert!((middle - 10.0).abs() < 1e-4, "the middle is {middle}");

        for value in [1.0, 10.0, 63.0, 100.0] {
            let position = position_of(value, RAIL, range, true);
            let back = value_at(position, RAIL, range, true);
            assert!((back - value).abs() < 1e-3, "{value} came back as {back}");
        }
    }

    /// The ends are the ends, whatever is asked for beyond them.
    #[test]
    fn nothing_maps_outside_the_range() {
        let range = (2.0, 8.0);
        assert_eq!(value_at(0.0, RAIL, range, false), 2.0);
        assert_eq!(value_at(1000.0, RAIL, range, false), 8.0);
        assert_eq!(position_of(-50.0, RAIL, range, false), RAIL.min);
        assert_eq!(position_of(50.0, RAIL, range, false), RAIL.max);
    }

    /// An empty range is a rail with nothing on it, not a division by nought.
    #[test]
    fn an_empty_range_is_survivable() {
        let range = (5.0, 5.0);
        assert_eq!(value_at(200.0, RAIL, range, false), 5.0);
        assert_eq!(position_of(5.0, RAIL, range, false), RAIL.min);

        let no_rail = Rangef::new(100.0, 100.0);
        assert_eq!(value_at(200.0, no_rail, (0.0, 1.0), false), 0.0);
    }

    /// Every number the menu offers is one the settings window can also hold,
    /// or the two views over one setting would disagree.
    #[test]
    fn the_menu_offers_nothing_the_settings_window_refuses() {
        for (travel, label) in CHOICES {
            assert!(
                (BOUND..=FURTHEST).contains(travel),
                "{label} is outside what the row allows"
            );
        }

        assert!(CHOICES.iter().any(|(travel, _)| *travel == SHIPS_AS));
        assert!(CHOICES.iter().any(|(travel, _)| *travel == BOUND));
    }

    /// The rail in the settings window and the numbers here are two views over
    /// one setting, and they are held to the same ends and the same default.
    #[test]
    fn the_row_and_the_rails_agree_about_the_setting() {
        let row = crate::config::registry::row("mouse.slider_travel")
            .expect("the setting has a row of its own");

        let crate::config::registry::Access::Float { min, max, .. } = row.access else {
            panic!("it is a number with a rail");
        };

        assert_eq!(min, BOUND);
        assert_eq!(max, FURTHEST);
        assert_eq!(
            crate::config::Config::default().mouse.slider_travel,
            SHIPS_AS
        );
    }
}
