//! The wheel, read before the toolkit has folded it.
//!
//! Both views want the same three answers out of a notch — which way it went,
//! what was held while it went, and how far — and neither can get them from
//! `raw_scroll_delta`, which is what is left after egui has spent Shift and
//! Alt on axes of its own choosing. So the event is read directly, in one
//! place, and what is done with it is each view's own business.

use eframe::egui;

use crate::config::{MouseConfig, WheelJob};

/// One frame's worth of the wheel, as this crate reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Notch {
    /// How far it turned. Positive is away from the hand, which everywhere
    /// else in the toolkit means "up" and here means "earlier".
    pub amount: f32,
    /// The same movement counted in notches of a wheel.
    ///
    /// The raw figure means a different thing on each device: a mouse reports
    /// the wheel in lines, one to a notch, and a trackpad reports a stroke in
    /// points, as a great many small movements. Anything that is a *distance*
    /// — how much a notch magnifies by — has to be told which it is reading,
    /// or a stroke of a trackpad crosses in one frame what a wheel crosses in
    /// twenty. Anything that only wants the direction reads `amount`.
    pub turns: f32,
    /// What was held while it turned.
    pub modifiers: egui::Modifiers,
}

/// How many points of scrolling stand for one notch of a wheel.
///
/// egui's own figure, read the other way about: a line is
/// `InputOptions::line_scroll_speed` points on the desktop, and a notch of a
/// wheel is a line. Taking the toolkit's number rather than choosing one
/// keeps a trackpad and a wheel agreeing about how far a stroke went.
const POINTS_A_NOTCH: f32 = 40.0;

/// The wheel, read off the events rather than off `raw_scroll_delta`.
///
/// Shift is egui's `horizontal_scroll_modifier` and Alt its
/// `vertical_scroll_modifier`, so by the time a delta has been accumulated a
/// Shift + wheel has been rewritten into a purely horizontal movement and
/// `raw_scroll_delta.y` is zero. That is not a decision anybody here made; it
/// is inherited from the toolkit, and a plan that wants a meaning for those
/// two modifiers has to claim them back.
///
/// There are two ways to claim them and this is the quieter one. Writing
/// `Options::input_options` at startup would take Shift away from every widget
/// in the program, against egui's own documentation, which says of
/// `horizontal_scroll_modifier` that "it is STRONGLY recommended to NOT change
/// this". Reading the event before the fold changes nothing for anybody else,
/// and it is the same decision either way: the modifiers are claimed here
/// deliberately rather than left to the toolkit by inheritance.
pub fn read(ctx: &egui::Context) -> Option<Notch> {
    ctx.input(|input| {
        let mut amount = 0.0;
        let mut turns = 0.0;
        let mut modifiers = None;

        for event in &input.events {
            if let egui::Event::MouseWheel {
                unit,
                delta,
                modifiers: held,
            } = event
            {
                // Both axes, added. A mouse reports the wheel on y and a
                // trackpad on either, and only the sign and the fact of a
                // notch are wanted here: how far a pan travels comes from
                // `smooth_scroll_delta`, which egui has already scaled from
                // lines into points.
                let along = delta.x + delta.y;

                amount += along;
                turns += match unit {
                    // A page is a movement of its own size and there is no
                    // sensible number of notches in one; it is counted as the
                    // one press it was.
                    egui::MouseWheelUnit::Line | egui::MouseWheelUnit::Page => along,
                    egui::MouseWheelUnit::Point => along / POINTS_A_NOTCH,
                };
                modifiers = Some(*held);
            }
        }

        modifiers.filter(|_| amount != 0.0).map(|modifiers| Notch {
            amount,
            turns,
            modifiers,
        })
    })
}

/// What one notch means, once the modifiers and the settings have been read.
///
/// One of these and no more, which is the whole of the first fault this stage
/// is about: a notch used to call `Next` *and* write its delta into the
/// arriving photograph's viewport, in that order, with nothing guarding the
/// second against the first. It showed whenever the photograph that arrived
/// had any slack to be shoved into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job {
    /// Further into the folder, or back out of it.
    Forward,
    Back,
    /// Ten at a time, which is what the page keys move.
    PageForward,
    PageBack,
    /// Magnify about the pointer, by the step the configuration names — the
    /// fine one where the modifier that means "finer" was held.
    ZoomIn {
        fine: bool,
    },
    ZoomOut {
        fine: bool,
    },
    /// Move the photograph along the wheel's axis, or across it.
    Pan,
    PanSideways,
    /// Nothing, deliberately.
    Nothing,
}

/// What the wheel was asked to do.
///
/// The modifiers come first and are not settings. They are the same step the
/// page keys take and the same axis the pan keys move on, and somebody who has
/// given the bare wheel another job has not thereby said anything about Shift.
///
/// `fine` is whether [`ImageViewConfig::fine_modifier`] was held, which the
/// caller decides because the modifier lives with the photograph rather than
/// with the mouse. It refines a notch that magnifies and nothing else: a
/// notch that walks the folder has no smaller version of itself, and Alt on
/// one that pans is already the axis across the wheel's own.
///
/// [`ImageViewConfig::fine_modifier`]: crate::config::ImageViewConfig::fine_modifier
pub fn decide(notch: Notch, mouse: &MouseConfig, fine: bool) -> Job {
    let modifiers = notch.modifiers;
    let only = |wanted: bool| wanted && !modifiers.command;

    if only(modifiers.shift && !modifiers.alt) {
        return if forward(notch, mouse) {
            Job::PageForward
        } else {
            Job::PageBack
        };
    }

    let job = if modifiers.command {
        mouse.ctrl_wheel
    } else {
        mouse.wheel
    };

    // Alt is the axis across the wheel's own, and stays that — except on a
    // notch that magnifies, where a finer version of the gesture is worth
    // more than a second way to pan. With the wheel as it ships that is Ctrl
    // and Alt together, and a bare Alt still goes sideways.
    if only(modifiers.alt && !modifiers.shift) && !(fine && job == WheelJob::Zoom) {
        return Job::PanSideways;
    }

    match job {
        WheelJob::NextOrPrevious => {
            if forward(notch, mouse) {
                Job::Forward
            } else {
                Job::Back
            }
        }
        WheelJob::Zoom if notch.amount > 0.0 => Job::ZoomIn { fine },
        WheelJob::Zoom => Job::ZoomOut { fine },
        WheelJob::Pan => Job::Pan,
        WheelJob::Nothing => Job::Nothing,
    }
}

/// Whether the magnification egui is reporting is a notch already spent.
///
/// egui's `zoom_modifier` is Ctrl and it is *strongly* recommended not to
/// move it, so a Ctrl notch is folded into `zoom_delta` before this crate
/// sees the event — and smoothed, which spreads one turn of the wheel over
/// the several frames after it, frames carrying no event at all. The viewer
/// answers such a notch with the step the configuration names, so it would
/// otherwise pay for the same turn twice: once as a step, and again as a tail
/// nothing can tell apart from a pinch.
///
/// So the tail is latched shut by the notch and opened by the first frame the
/// magnification comes back to exactly one, which is what egui reports the
/// moment it has nothing left to smooth. A pinch arrives as `Event::Zoom`,
/// which is not folded and not smoothed, and is what is left.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tail {
    running: bool,
}

impl Tail {
    /// Whether this frame's `zoom_delta` is egui finishing a notch.
    ///
    /// Asked once a pass and before anything can return early: a latch left
    /// shut swallows the next pinch instead.
    pub fn swallows(&mut self, notch: Option<Notch>, zoom_delta: f32) -> bool {
        if notch.is_some_and(|notch| notch.modifiers.command) {
            self.running = true;
        } else if zoom_delta == 1.0 {
            self.running = false;
        }

        self.running
    }
}

/// Whether this notch means "further into the folder".
///
/// Wheel *down*, because that is what it means in the contact sheet, which is
/// an ordinary scrolling list, and in every list widget the toolkit provides.
/// The image view used to disagree with it, so the same wrist movement meant
/// "later" in one view and "earlier" in the other, and one key switched
/// between the two.
fn forward(notch: Notch, mouse: &MouseConfig) -> bool {
    (notch.amount < 0.0) != mouse.wheel_reversed
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{vec2, Event, Modifiers, MouseWheelUnit, RawInput};

    fn context_with(events: Vec<Event>) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.begin_pass(RawInput {
            events,
            ..Default::default()
        });
        ctx
    }

    fn turned(amount: f32, modifiers: Modifiers) -> Notch {
        Notch {
            amount,
            turns: amount,
            modifiers,
        }
    }

    fn notch(y: f32, modifiers: Modifiers) -> Event {
        Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta: vec2(0.0, y),
            modifiers,
        }
    }

    fn stroke(y: f32) -> Event {
        Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: vec2(0.0, y),
            modifiers: Modifiers::NONE,
        }
    }

    /// No wheel event, no notch. The pan and the zoom both key off this, so a
    /// `Some` here would move the photograph on every frame.
    #[test]
    fn a_frame_with_no_wheel_in_it_reports_no_notch() {
        let ctx = context_with(vec![]);
        assert_eq!(read(&ctx), None);
    }

    /// A notch reaches this crate with the modifiers it was turned under,
    /// which is the whole reason for reading the event rather than the delta:
    /// with Shift held, `raw_scroll_delta.y` is zero by the time egui has
    /// finished folding it onto the horizontal axis.
    #[test]
    fn a_notch_keeps_its_modifiers_and_its_direction() {
        let ctx = context_with(vec![notch(-1.0, Modifiers::SHIFT)]);

        let read = read(&ctx).expect("the wheel turned");
        assert!(read.amount < 0.0, "downwards");
        assert!(read.modifiers.shift);
        assert_eq!(
            ctx.input(|i| i.raw_scroll_delta.y),
            0.0,
            "and the toolkit has spent it on the other axis"
        );
    }

    /// Two notches in one frame are one movement, not two.
    #[test]
    fn notches_in_the_same_frame_add_up() {
        let ctx = context_with(vec![
            notch(1.0, Modifiers::NONE),
            notch(1.0, Modifiers::NONE),
        ]);
        assert_eq!(read(&ctx).map(|n| n.amount), Some(2.0));
    }

    /// Alt is the other one egui spends, onto the vertical axis. It survives
    /// here too, which is what lets it mean something of its own.
    #[test]
    fn alt_survives_as_well() {
        let ctx = context_with(vec![notch(1.0, Modifiers::ALT)]);
        assert!(read(&ctx).expect("the wheel turned").modifiers.alt);
    }

    fn mouse() -> MouseConfig {
        MouseConfig::default()
    }

    /// The default, and the direction that changed: wheel down is forward.
    #[test]
    fn a_notch_down_moves_further_into_the_folder() {
        assert_eq!(
            decide(turned(-1.0, Modifiers::NONE), &mouse(), false),
            Job::Forward
        );
        assert_eq!(
            decide(turned(1.0, Modifiers::NONE), &mouse(), false),
            Job::Back
        );
    }

    #[test]
    fn and_the_reverse_flag_turns_it_round() {
        let mouse = MouseConfig {
            wheel_reversed: true,
            ..MouseConfig::default()
        };

        assert_eq!(
            decide(turned(-1.0, Modifiers::NONE), &mouse, false),
            Job::Back
        );
        assert_eq!(
            decide(turned(1.0, Modifiers::NONE), &mouse, false),
            Job::Forward
        );
    }

    /// The whole point of the field: somebody who wants the wheel to zoom can
    /// have it, without the argument about the default being settled.
    #[test]
    fn the_wheel_can_be_made_to_zoom_instead() {
        let mouse = MouseConfig {
            wheel: WheelJob::Zoom,
            ..MouseConfig::default()
        };

        assert_eq!(
            decide(turned(1.0, Modifiers::NONE), &mouse, false),
            Job::ZoomIn { fine: false }
        );
        assert_eq!(
            decide(turned(-1.0, Modifiers::NONE), &mouse, false),
            Job::ZoomOut { fine: false }
        );
    }

    /// And the other way about, which is nomacs #237 in one line.
    #[test]
    fn ctrl_can_be_made_to_walk_the_folder() {
        let mouse = MouseConfig {
            ctrl_wheel: WheelJob::NextOrPrevious,
            ..MouseConfig::default()
        };

        assert_eq!(
            decide(turned(-1.0, Modifiers::COMMAND), &mouse, false),
            Job::Forward
        );
    }

    /// Shipped: Ctrl and the wheel zooms, by the step the configuration
    /// names. egui folds the same notch into a magnification of its own,
    /// which [`Tail`] is what swallows.
    #[test]
    fn ctrl_and_the_wheel_zooms_by_the_configured_step() {
        assert_eq!(
            decide(turned(1.0, Modifiers::COMMAND), &mouse(), false),
            Job::ZoomIn { fine: false }
        );
        assert_eq!(
            decide(turned(-1.0, Modifiers::COMMAND), &mouse(), false),
            Job::ZoomOut { fine: false }
        );
    }

    /// The gesture this stage is about: the modifier that means "finer" on a
    /// notch that magnifies. With the wheel as it ships, Ctrl and Alt.
    #[test]
    fn the_fine_modifier_refines_a_notch_that_magnifies() {
        assert_eq!(
            decide(
                turned(1.0, Modifiers::COMMAND | Modifiers::ALT),
                &mouse(),
                true
            ),
            Job::ZoomIn { fine: true }
        );
    }

    /// And where the bare wheel has been set to zoom, Alt alone is the finer
    /// version of it rather than the sideways pan: sideways is a second way
    /// to pan a photograph the wheel is not panning.
    #[test]
    fn alt_is_finer_where_the_bare_wheel_zooms() {
        let mouse = MouseConfig {
            wheel: WheelJob::Zoom,
            ..MouseConfig::default()
        };

        assert_eq!(
            decide(turned(1.0, Modifiers::ALT), &mouse, true),
            Job::ZoomIn { fine: true }
        );

        // Where it is not the fine modifier — somebody who chose Ctrl or
        // Shift — Alt is what it always was.
        assert_eq!(
            decide(turned(1.0, Modifiers::ALT), &mouse, false),
            Job::PanSideways
        );
    }

    /// The modifiers are not settings, and outrank whatever the bare wheel
    /// has been told to do.
    #[test]
    fn shift_pages_and_alt_goes_sideways_whatever_the_wheel_is_set_to() {
        let mouse = MouseConfig {
            wheel: WheelJob::Nothing,
            ..MouseConfig::default()
        };

        assert_eq!(
            decide(turned(-1.0, Modifiers::SHIFT), &mouse, false),
            Job::PageForward
        );
        assert_eq!(
            decide(turned(1.0, Modifiers::SHIFT), &mouse, false),
            Job::PageBack
        );
        assert_eq!(
            decide(turned(1.0, Modifiers::ALT), &mouse, false),
            Job::PanSideways
        );
    }

    /// A notch does one thing. It used to do two, and the second one landed on
    /// the photograph that had just arrived because of the first.
    #[test]
    fn one_notch_is_one_job() {
        for (job, mouse) in [
            (Job::Forward, MouseConfig::default()),
            (
                Job::Pan,
                MouseConfig {
                    wheel: WheelJob::Pan,
                    ..MouseConfig::default()
                },
            ),
            (
                Job::Nothing,
                MouseConfig {
                    wheel: WheelJob::Nothing,
                    ..MouseConfig::default()
                },
            ),
        ] {
            assert_eq!(decide(turned(-1.0, Modifiers::NONE), &mouse, false), job);
        }
    }

    /// A mouse reports lines, and one line is one notch.
    #[test]
    fn a_line_is_a_notch() {
        let ctx = context_with(vec![notch(1.0, Modifiers::NONE)]);
        assert_eq!(read(&ctx).map(|n| n.turns), Some(1.0));
    }

    /// A trackpad reports points, and a stroke of it is one movement made of
    /// a great many. Counted in notches it is a fraction of one, which is the
    /// difference between a stroke that magnifies smoothly and one that
    /// crosses the whole range in a frame.
    #[test]
    fn a_stroke_of_a_trackpad_is_a_fraction_of_a_notch() {
        let ctx = context_with(vec![stroke(5.0)]);

        let read = read(&ctx).expect("the wheel turned");
        assert_eq!(read.amount, 5.0);
        assert!(read.turns < 0.2, "{read:?}");
    }

    /// egui smooths a Ctrl notch into a magnification of its own over the
    /// frames after it, and the viewer has already answered the notch with a
    /// step. The tail is swallowed until the magnification comes back to one.
    #[test]
    fn the_tail_of_a_ctrl_notch_is_swallowed() {
        let mut tail = Tail::default();

        assert!(tail.swallows(Some(turned(1.0, Modifiers::COMMAND)), 1.07));
        assert!(tail.swallows(None, 1.04), "the frames after it");
        assert!(tail.swallows(None, 1.01));
        assert!(!tail.swallows(None, 1.0), "and then it is over");
    }

    /// A pinch is not folded and not smoothed, so nothing swallows it.
    #[test]
    fn a_pinch_is_left_alone() {
        let mut tail = Tail::default();

        assert!(!tail.swallows(None, 1.2));
        assert!(!tail.swallows(Some(turned(1.0, Modifiers::NONE)), 1.2));
    }
}
