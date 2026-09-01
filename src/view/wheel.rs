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
    /// What was held while it turned.
    pub modifiers: egui::Modifiers,
}

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
        let mut modifiers = None;

        for event in &input.events {
            if let egui::Event::MouseWheel {
                delta,
                modifiers: held,
                ..
            } = event
            {
                // Both axes, added. A mouse reports the wheel on y and a
                // trackpad on either, and only the sign and the fact of a
                // notch are wanted here: how far a pan travels comes from
                // `smooth_scroll_delta`, which egui has already scaled from
                // lines into points.
                amount += delta.x + delta.y;
                modifiers = Some(*held);
            }
        }

        modifiers
            .filter(|_| amount != 0.0)
            .map(|modifiers| Notch { amount, modifiers })
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
    /// Magnify about the pointer.
    ZoomIn,
    ZoomOut,
    /// Move the photograph along the wheel's axis, or across it.
    Pan,
    PanSideways,
    /// Ctrl and the wheel, where Ctrl and the wheel still means zoom. egui's
    /// `zoom_modifier` is Ctrl, so it has already turned this notch into a
    /// zoom of its own; doing another here would count one notch twice.
    AlreadyZoomed,
    /// Nothing, deliberately.
    Nothing,
}

/// What the wheel was asked to do.
///
/// The modifiers come first and are not settings. They are the same step the
/// page keys take and the same axis the pan keys move on, and somebody who has
/// given the bare wheel another job has not thereby said anything about Shift.
pub fn decide(notch: Notch, mouse: &MouseConfig) -> Job {
    let modifiers = notch.modifiers;
    let only = |wanted: bool| wanted && !modifiers.command;

    if only(modifiers.shift && !modifiers.alt) {
        return if forward(notch, mouse) {
            Job::PageForward
        } else {
            Job::PageBack
        };
    }

    if only(modifiers.alt && !modifiers.shift) {
        return Job::PanSideways;
    }

    let job = if modifiers.command {
        mouse.ctrl_wheel
    } else {
        mouse.wheel
    };

    match job {
        WheelJob::NextOrPrevious => {
            if forward(notch, mouse) {
                Job::Forward
            } else {
                Job::Back
            }
        }
        WheelJob::Zoom if modifiers.command => Job::AlreadyZoomed,
        WheelJob::Zoom if notch.amount > 0.0 => Job::ZoomIn,
        WheelJob::Zoom => Job::ZoomOut,
        WheelJob::Pan => Job::Pan,
        WheelJob::Nothing => Job::Nothing,
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
        Notch { amount, modifiers }
    }

    fn notch(y: f32, modifiers: Modifiers) -> Event {
        Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta: vec2(0.0, y),
            modifiers,
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
            decide(turned(-1.0, Modifiers::NONE), &mouse()),
            Job::Forward
        );
        assert_eq!(decide(turned(1.0, Modifiers::NONE), &mouse()), Job::Back);
    }

    #[test]
    fn and_the_reverse_flag_turns_it_round() {
        let mouse = MouseConfig {
            wheel_reversed: true,
            ..MouseConfig::default()
        };

        assert_eq!(decide(turned(-1.0, Modifiers::NONE), &mouse), Job::Back);
        assert_eq!(decide(turned(1.0, Modifiers::NONE), &mouse), Job::Forward);
    }

    /// The whole point of the field: somebody who wants the wheel to zoom can
    /// have it, without the argument about the default being settled.
    #[test]
    fn the_wheel_can_be_made_to_zoom_instead() {
        let mouse = MouseConfig {
            wheel: WheelJob::Zoom,
            ..MouseConfig::default()
        };

        assert_eq!(decide(turned(1.0, Modifiers::NONE), &mouse), Job::ZoomIn);
        assert_eq!(decide(turned(-1.0, Modifiers::NONE), &mouse), Job::ZoomOut);
    }

    /// And the other way about, which is nomacs #237 in one line.
    #[test]
    fn ctrl_can_be_made_to_walk_the_folder() {
        let mouse = MouseConfig {
            ctrl_wheel: WheelJob::NextOrPrevious,
            ..MouseConfig::default()
        };

        assert_eq!(
            decide(turned(-1.0, Modifiers::COMMAND), &mouse),
            Job::Forward
        );
    }

    /// Shipped: Ctrl and the wheel zooms, and egui has already done it.
    #[test]
    fn ctrl_and_the_wheel_is_left_to_the_toolkit() {
        assert_eq!(
            decide(turned(1.0, Modifiers::COMMAND), &mouse()),
            Job::AlreadyZoomed
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
            decide(turned(-1.0, Modifiers::SHIFT), &mouse),
            Job::PageForward
        );
        assert_eq!(decide(turned(1.0, Modifiers::SHIFT), &mouse), Job::PageBack);
        assert_eq!(
            decide(turned(1.0, Modifiers::ALT), &mouse),
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
            assert_eq!(decide(turned(-1.0, Modifiers::NONE), &mouse), job);
        }
    }
}
