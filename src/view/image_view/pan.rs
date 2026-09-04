//! What the pan keys, and a drag, move the photograph by.
//!
//! A press and a hold are two gestures. The press moves the view exactly one
//! step, and the hold glides at so many screenfuls a second once it has been
//! down longer than the delay — the shape of a keyboard's own repeat, and the
//! reason a tap is now worth a fixed distance rather than however long a
//! finger stays on a key. Time alone used to decide both: the shortest press
//! anybody can make covers two or three frames, so the smallest movement the
//! keys could ask for was several times the smallest movement worth asking
//! for, and no setting could bring it down without making a held key useless
//! for travelling.
//!
//! Kept apart from egui so the arithmetic can be tested without a window: the
//! reader below is the only part that needs a context.

use eframe::egui;
use eframe::epaint::Vec2;

use crate::config::ImageViewConfig;
use crate::ui::front;

/// Which way the photograph moves for each of the four keys.
///
/// Reading the key as "which way the eye moves", so pressing right shows what
/// is further right — which is the photograph going left, and where the
/// negatives come from. Right, left, down, up, and every array here is in
/// that order.
const WAYS: [Vec2; 4] = [
    Vec2::new(-1.0, 0.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, -1.0),
    Vec2::new(0.0, 1.0),
];

/// What the four keys are doing this frame.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Keys {
    /// Which of them are down.
    pub down: [bool; 4],
    /// Which of them went down on this very frame.
    ///
    /// Read off the events rather than from the frame before, so a tap shorter
    /// than a frame still moves the view, and told apart from the platform's
    /// key repeat, which would otherwise step the picture at whatever rate the
    /// operating system is set to on top of the glide.
    pub pressed: [bool; 4],
    /// Whether the modifier that means "finer" is held.
    pub fine: bool,
}

impl Keys {
    /// Whether anything at all is down, which is what the view asks for
    /// another frame for: held keys produce no events, so nothing else would
    /// ask, and the glide would never arrive.
    pub fn anything(&self) -> bool {
        self.down.iter().any(|down| *down) || self.pressed.iter().any(|pressed| *pressed)
    }
}

/// How far a press, a hold and a drag move, as the configuration has them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pace {
    /// One press, in points.
    pub step: f32,
    /// A held key, in screenfuls a second.
    pub speed: f32,
    /// How long a key is held before the glide starts.
    pub delay: f32,
    /// The share of the pointer's travel a drag moves the photograph by.
    pub drag: f32,
}

impl Pace {
    /// The pace the fine modifier asks for, or the ordinary one.
    ///
    /// The delay is the same either way: it is about telling a tap from a
    /// hold, which is a property of the hand rather than of the gesture. So
    /// is the ordinary drag, which is one for one and not a setting — a
    /// photograph that does not follow the hand is not being dragged. What
    /// the modifier is for is the other answer: a quarter of the travel, so
    /// an eyelash can be put in the middle of the window at four hundred per
    /// cent with the same gesture that got there.
    pub fn of(config: &ImageViewConfig, fine: bool) -> Pace {
        let (step, speed, drag) = if fine {
            (
                config.pan_fine_step,
                config.pan_fine_speed,
                config.pan_fine_drag,
            )
        } else {
            (config.pan_step, config.pan_speed, 1.0)
        };

        Pace {
            step: step.max(0.0),
            speed: speed.max(0.0),
            delay: config.pan_glide_delay.max(0.0),
            drag: drag.max(0.0),
        }
    }
}

/// How long each of the four keys has been down.
///
/// The one thing panning remembers between frames. A key that is up is
/// `None`, and a key that went down this frame is `Some(0.0)`, so the glide is
/// always measured from the press that started it rather than from whenever
/// the view last happened to look.
#[derive(Debug, Default, Clone, Copy)]
pub struct Glide {
    held: [Option<f32>; 4],
}

impl Glide {
    /// How far to move the photograph this frame, in points.
    ///
    /// `panel` is the space the photograph is drawn into, so a held key covers
    /// the same share of the picture whatever the window size, and `seconds`
    /// is how long the last frame took, so it covers it at the same speed
    /// whatever the frame rate. The step is in points and is deliberately not
    /// scaled by either: it is the distance somebody asked for exactly.
    pub fn moved(&mut self, keys: Keys, pace: Pace, panel: Vec2, seconds: f32) -> Vec2 {
        // A frame that took a second — a folder opening, a window coming back
        // from sleep — is worth a frame's travel and not a second's.
        let seconds = seconds.clamp(0.0, 0.1);

        let mut pressed = Vec2::ZERO;
        let mut gliding = Vec2::ZERO;

        for (way, direction) in WAYS.iter().enumerate() {
            if keys.pressed[way] {
                self.held[way] = Some(0.0);
                pressed += *direction;
            } else if keys.down[way] {
                // A key that was already down when the view started looking —
                // held through a text field taking the keyboard, or through a
                // window in front — glides from here rather than never.
                let held = self.held[way].get_or_insert(0.0);
                *held += seconds;

                if *held >= pace.delay {
                    gliding += *direction;
                }
            } else {
                self.held[way] = None;
            }
        }

        let mut moved = Vec2::ZERO;

        // A diagonal should not be faster than a straight line, in either
        // gesture; the two are added because a second key pressed during a
        // glide is a press of its own.
        if pressed != Vec2::ZERO {
            moved += pressed.normalized() * pace.step;
        }

        if gliding != Vec2::ZERO {
            let share = pace.speed * seconds;
            moved += gliding.normalized() * Vec2::new(panel.x * share, panel.y * share);
        }

        moved
    }
}

/// What the keyboard is asking of the pan this frame.
///
/// Read as held keys and as presses rather than as shortcuts, because panning
/// is a movement as much as it is an event: the picture glides while the key
/// is down instead of stepping at whatever rate the operating system chooses
/// to repeat it.
pub fn asked(ctx: &egui::Context, config: &ImageViewConfig) -> Keys {
    if front::are_inputs_muted(ctx) {
        return Keys::default();
    }

    let bindings = [
        &config.sc_pan_right,
        &config.sc_pan_left,
        &config.sc_pan_down,
        &config.sc_pan_up,
    ];

    ctx.input(|input| {
        let mut keys = Keys {
            fine: config.fine_modifier.held(&input.modifiers),
            ..Keys::default()
        };

        for (way, binding) in bindings.iter().enumerate() {
            // Any of the keys bound to that direction, and the modifiers of
            // none of them: what is held with a pan key says how far it moves,
            // not whether it moves — see `Config::check`, which is what warns
            // when a command is sitting on the chord that results.
            for chord in binding.chords() {
                let key = chord.kbd_shortcut.logical_key;

                keys.down[way] |= input.key_down(key);
                keys.pressed[way] |= input.events.iter().any(|event| {
                    matches!(
                        event,
                        egui::Event::Key {
                            key: pressed,
                            pressed: true,
                            repeat: false,
                            ..
                        } if *pressed == key
                    )
                });
            }
        }

        keys
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Key, Modifiers, RawInput};

    const PANEL: Vec2 = Vec2::new(1000.0, 800.0);
    const FRAME: f32 = 1.0 / 60.0;

    const RIGHT: usize = 0;
    const LEFT: usize = 1;
    const DOWN: usize = 2;
    const UP: usize = 3;

    fn pace() -> Pace {
        Pace::of(&ImageViewConfig::default(), false)
    }

    fn down(way: usize) -> Keys {
        let mut down = [false; 4];
        down[way] = true;

        Keys {
            down,
            ..Keys::default()
        }
    }

    fn press(way: usize) -> Keys {
        let mut keys = down(way);
        keys.pressed[way] = true;
        keys
    }

    /// Holds `keys` for `seconds`, a frame at a time, and reports how far the
    /// photograph moved in all.
    fn holding(glide: &mut Glide, keys: Keys, pace: Pace, seconds: f32) -> Vec2 {
        let frames = (seconds / FRAME).round() as usize;

        (0..frames).fold(Vec2::ZERO, |moved, frame| {
            let keys = if frame == 0 { keys } else { held(keys) };
            moved + glide.moved(keys, pace, PANEL, FRAME)
        })
    }

    /// The same keys, a frame later: still down, no longer a press.
    fn held(mut keys: Keys) -> Keys {
        keys.pressed = [false; 4];
        keys
    }

    #[test]
    fn nothing_held_pans_nowhere() {
        let mut glide = Glide::default();

        assert_eq!(
            glide.moved(Keys::default(), pace(), PANEL, FRAME),
            Vec2::ZERO
        );
    }

    /// The fault this was written for: the shortest press anybody can make
    /// spans two or three frames, and every one of them used to be paid for.
    #[test]
    fn the_shortest_press_moves_exactly_one_step() {
        let pace = pace();

        for frames in 1..=4 {
            let mut glide = Glide::default();
            let moved = holding(&mut glide, press(RIGHT), pace, frames as f32 * FRAME);

            assert!(
                (moved.x.abs() - pace.step).abs() < 0.001,
                "{frames} frames moved {moved:?}, not one step of {}",
                pace.step
            );
        }
    }

    #[test]
    fn a_press_moves_the_same_distance_whatever_the_frame_rate() {
        let pace = pace();

        let mut quick = Glide::default();
        let mut slow = Glide::default();

        assert_eq!(
            quick.moved(press(RIGHT), pace, PANEL, FRAME),
            slow.moved(press(RIGHT), pace, PANEL, 0.1)
        );
    }

    #[test]
    fn holding_a_key_glides_once_the_delay_has_passed() {
        let pace = pace();
        let mut glide = Glide::default();

        let tapped = holding(&mut glide, press(RIGHT), pace, pace.delay / 2.0);
        let travelled = holding(&mut glide, held(down(RIGHT)), pace, 1.0);

        assert!((tapped.x.abs() - pace.step).abs() < 0.001, "{tapped:?}");
        assert!(
            travelled.x.abs() > PANEL.x * pace.speed * 0.5,
            "{travelled:?}"
        );
    }

    /// Nought means what it says: the keys as they were before the step
    /// existed, gliding from the first frame.
    #[test]
    fn no_delay_glides_at_once() {
        let pace = Pace {
            delay: 0.0,
            ..pace()
        };
        let mut glide = Glide::default();

        glide.moved(press(RIGHT), pace, PANEL, FRAME);
        let second = glide.moved(held(down(RIGHT)), pace, PANEL, FRAME);

        assert!(second.x.abs() > 0.0, "{second:?}");
    }

    #[test]
    fn looking_right_moves_the_image_left() {
        let mut glide = Glide::default();
        let moved = glide.moved(press(RIGHT), pace(), PANEL, FRAME);

        assert!(moved.x < 0.0, "{moved:?}");
        assert_eq!(moved.y, 0.0);
    }

    #[test]
    fn the_four_directions_are_opposites() {
        let pace = pace();
        let mut glide = Glide::default();

        let right = glide.moved(press(RIGHT), pace, PANEL, FRAME);
        let left = Glide::default().moved(press(LEFT), pace, PANEL, FRAME);
        let down_way = Glide::default().moved(press(DOWN), pace, PANEL, FRAME);
        let up = Glide::default().moved(press(UP), pace, PANEL, FRAME);

        assert_eq!(right, -left);
        assert_eq!(down_way, -up);
        assert!(up.y > 0.0);
    }

    #[test]
    fn a_diagonal_is_no_faster_than_a_straight_line() {
        let pace = pace();

        let mut keys = press(RIGHT);
        keys.down[DOWN] = true;
        keys.pressed[DOWN] = true;

        let diagonal = Glide::default().moved(keys, pace, PANEL, FRAME);
        let straight = Glide::default().moved(press(RIGHT), pace, PANEL, FRAME);

        assert!(
            (diagonal.length() - straight.length()).abs() < 0.001,
            "{diagonal:?}"
        );
    }

    /// The second key of a diagonal is a press of its own, whenever it lands.
    #[test]
    fn a_key_pressed_during_a_glide_steps_as_well() {
        let pace = pace();
        let mut glide = Glide::default();

        holding(&mut glide, press(RIGHT), pace, pace.delay + 0.2);

        let mut keys = held(down(RIGHT));
        keys.down[DOWN] = true;
        keys.pressed[DOWN] = true;

        let moved = glide.moved(keys, pace, PANEL, FRAME);

        assert!((moved.y.abs() - pace.step).abs() < 0.001, "{moved:?}");
        assert!(moved.x.abs() > 0.0, "the glide stopped: {moved:?}");
    }

    #[test]
    fn a_long_frame_does_not_send_the_image_flying() {
        let pace = pace();
        let mut glide = Glide::default();

        glide.moved(press(RIGHT), pace, PANEL, FRAME);
        let stutter = glide.moved(held(down(RIGHT)), pace, PANEL, 5.0);

        assert!(stutter.x.abs() < PANEL.x, "{stutter:?}");
    }

    #[test]
    fn letting_go_starts_the_delay_again() {
        let pace = pace();
        let mut glide = Glide::default();

        holding(&mut glide, press(RIGHT), pace, pace.delay + 0.2);
        glide.moved(Keys::default(), pace, PANEL, FRAME);

        let again = glide.moved(press(RIGHT), pace, PANEL, FRAME);
        assert!((again.x.abs() - pace.step).abs() < 0.001, "{again:?}");

        let next = glide.moved(held(down(RIGHT)), pace, PANEL, FRAME);
        assert_eq!(next, Vec2::ZERO, "the glide came back without the delay");
    }

    #[test]
    fn the_fine_modifier_moves_a_press_by_a_pixel() {
        let config = ImageViewConfig::default();
        let fine = Pace::of(&config, true);
        let ordinary = Pace::of(&config, false);

        assert!(fine.step < ordinary.step);
        assert!(fine.speed < ordinary.speed);
        assert!(fine.drag < ordinary.drag);
        assert_eq!(fine.delay, ordinary.delay);

        let moved = Glide::default().moved(press(RIGHT), fine, PANEL, FRAME);
        assert!((moved.x.abs() - fine.step).abs() < 0.001, "{moved:?}");
    }

    /// A drag follows the hand exactly, whatever else is configured. That is
    /// the one answer there is to it; the modifier is where the other one
    /// lives.
    #[test]
    fn an_ordinary_drag_is_one_for_one() {
        let config = ImageViewConfig {
            pan_fine_drag: 0.5,
            ..ImageViewConfig::default()
        };

        assert_eq!(Pace::of(&config, false).drag, 1.0);
        assert_eq!(Pace::of(&config, true).drag, 0.5);
    }

    /// A negative in the file is a value nobody can produce from the window,
    /// and it would pan the wrong way.
    #[test]
    fn a_negative_setting_stands_still_rather_than_reversing() {
        let config = ImageViewConfig {
            pan_step: -40.0,
            pan_speed: -1.5,
            pan_fine_drag: -0.5,
            ..ImageViewConfig::default()
        };

        assert_eq!(Pace::of(&config, true).drag, 0.0);

        let pace = Pace::of(&config, false);
        let mut glide = Glide::default();

        assert_eq!(glide.moved(press(RIGHT), pace, PANEL, FRAME), Vec2::ZERO);
        assert_eq!(
            holding(&mut glide, held(down(RIGHT)), pace, 1.0),
            Vec2::ZERO
        );
    }

    fn context_with(events: Vec<Event>) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.begin_pass(RawInput {
            modifiers: events
                .iter()
                .find_map(|event| match event {
                    Event::Key { modifiers, .. } => Some(*modifiers),
                    _ => None,
                })
                .unwrap_or(Modifiers::NONE),
            events,
            ..Default::default()
        });
        ctx
    }

    fn key_press(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    #[test]
    fn the_default_keys_are_read_as_the_four_ways() {
        let config = ImageViewConfig::default();

        let keys = asked(
            &context_with(vec![key_press(Key::D, Modifiers::NONE)]),
            &config,
        );

        assert!(keys.down[RIGHT] && keys.pressed[RIGHT]);
        assert!(!keys.fine);
        assert!(!keys.down[LEFT]);
    }

    /// The platform's own repeat is not a press: it would step the picture at
    /// whatever rate the keyboard is set to, on top of the glide.
    ///
    /// Two passes, because egui decides for itself whether an event is a
    /// repeat — it is one when the key was already down — and a key cannot
    /// have been already down on the frame it first arrives.
    #[test]
    fn a_key_repeat_is_not_a_press() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::D, Modifiers::NONE)]);
        let _ = ctx.end_pass();

        ctx.begin_pass(RawInput {
            events: vec![key_press(Key::D, Modifiers::NONE)],
            ..Default::default()
        });

        let keys = asked(&ctx, &config);

        assert!(keys.down[RIGHT], "the key is still down");
        assert!(!keys.pressed[RIGHT]);
    }

    #[test]
    fn the_modifier_is_read_alongside_the_key() {
        let config = ImageViewConfig::default();
        let keys = asked(
            &context_with(vec![key_press(Key::D, Modifiers::ALT)]),
            &config,
        );

        assert!(keys.pressed[RIGHT]);
        assert!(keys.fine);
    }

    /// Alt by default, and the other two are legal answers.
    #[test]
    fn the_modifier_is_the_one_the_configuration_names() {
        let config = ImageViewConfig {
            fine_modifier: crate::config::FineModifier::Ctrl,
            ..ImageViewConfig::default()
        };

        let alt = asked(
            &context_with(vec![key_press(Key::D, Modifiers::ALT)]),
            &config,
        );
        let ctrl = asked(
            &context_with(vec![key_press(Key::D, Modifiers::COMMAND)]),
            &config,
        );

        assert!(!alt.fine);
        assert!(ctrl.fine);
    }

    #[test]
    fn panning_is_silent_while_a_text_field_has_focus() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::D, Modifiers::NONE)]);
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new("a text field")));

        assert_eq!(asked(&ctx, &config), Keys::default());
    }
}
