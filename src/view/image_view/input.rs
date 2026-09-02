//! Turning key presses into view commands.
//!
//! Collecting commands rather than mutating the view directly keeps the
//! keyboard map in one readable place and lets the status bar issue the same
//! commands as the keyboard.

use eframe::egui;

use crate::config::{shortcut, ImageViewConfig};
use crate::utils;

/// How much one press of the zoom keys changes the magnification.
///
/// A quarter each way: small enough to arrive at a particular framing, large
/// enough that crossing a useful range does not take twenty presses.
///
/// The default of `image_view.zoom_step`, kept here so the fallbacks below have
/// something to say when there is no configuration in hand.
const ZOOM_STEP: f32 = 1.25;

/// What a zoom holds still.
///
/// Magnifying is about a point in the picture, so it holds whatever is under
/// the pointer: an eye near the edge of the frame used to be pushed off screen
/// by the very gesture aimed at it. Fitting and filling are about the panel
/// rather than about a point in the picture, so they hold its middle — and so
/// does anything asked for by a control that is *not* the picture. The rail in
/// the status bar is the case that taught that last part: the pointer is on the
/// rail, and wherever it happens to sit over the photograph is an accident of
/// how the hand drifted while dragging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// The middle of the panel.
    Centre,
    /// Whatever is under the pointer, or the middle when it is elsewhere.
    Pointer,
}

impl Anchor {
    /// What a zoom asked for from the status bar holds: the middle, always.
    ///
    /// Named once and shared by the rail and the list of percentages beside
    /// it, because it is one decision about one surface. Magnifying towards the
    /// pointer would swing the picture about under a gesture that has nothing
    /// to do with any point in it — and while the rail is being dragged the
    /// pointer may be *put back* on the other side of the window, so where it
    /// sits over the photograph is not even an accident of the hand.
    pub const FROM_THE_BAR: Anchor = Anchor::Centre;
}

/// Something the image view can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Next,
    Previous,
    /// The first or the last photograph on show.
    First,
    Last,
    /// A screenful forward or back, for walking a long folder quickly.
    PageForward,
    PageBack,
    /// Fit the whole image in the panel.
    Fit,
    /// Fill the panel, cropping the overflowing side.
    Fill,
    /// Move what a new photograph opens at round the three answers.
    CycleOpening,
    /// Carry the magnification, or where in the photograph the view is, from
    /// one photograph to the next.
    ToggleKeepZoom,
    ToggleKeepPan,
    FitHorizontal,
    FitVertical,
    /// Double the magnification, wrapping back to fitted.
    ZoomStep,
    /// Multiply the magnification, for the zoom keys and the wheel.
    ZoomBy(f32),
    /// Move what the photograph says about itself to the next corner, or off.
    CycleOverlay,
    /// Mark what has clipped, then what is in focus, then nothing.
    CycleMarks,
    /// Magnify to a percentage of the image's own pixels, holding a point.
    ///
    /// The anchor is part of the command because the same magnification is
    /// asked for from the keyboard, from a menu over the photograph and from a
    /// rail in the status bar, and only the first two happen anywhere near the
    /// picture. A caller has to say which it means.
    ZoomToPercent(f32, Anchor),
    /// Fitted, or at one screen pixel per photograph pixel, whichever it is
    /// not already.
    ///
    /// Not on a key: it is what the double click ships bound to, and the two
    /// halves of it have keys of their own. It is one command rather than two
    /// because a gesture that is its own way back is worth more than a gesture
    /// that needs a different one.
    ToggleActualPixels,
    /// Magnify until the marked area of the photograph fills the panel.
    ZoomToArea,
    /// Put the marked area on the clipboard, or the whole photograph when
    /// nothing is marked.
    CopyArea,
    /// Forget the marking, which `Escape` and a click outside it also do.
    ClearArea,
    /// Put this image where the last one was left.
    RepeatPlace,
    ToggleFrame,
    ShowMoreImages,
    ShowFewerImages,
    /// Pin the photograph on screen and its neighbours as a comparison.
    Compare,
    /// Move the focus to the next pane of one.
    NextPane,
    /// Put the cursor in the "go to" box.
    GoTo,
    /// Drop the focused pane, leaving the survivors to re-tile.
    DropPane,
    /// Leave the comparison.
    StopComparing,
    /// The user action at this position in the configuration.
    UserAction(usize),
}

/// Reads this frame's input and returns the commands it maps to.
///
/// Returns nothing while a text field has focus, so typing a path never
/// triggers a shortcut.
pub fn collect(ctx: &egui::Context, config: &ImageViewConfig) -> Vec<Command> {
    if utils::are_inputs_muted(ctx) {
        return Vec::new();
    }

    // From the configuration rather than a constant: how far one press moves
    // is a judgement about the photographs somebody looks at.
    let step = if config.zoom_step > 1.0 {
        config.zoom_step
    } else {
        ZOOM_STEP
    };

    let bindings = [
        (&config.sc_next, Command::Next),
        (&config.sc_prev, Command::Previous),
        (&config.sc_fit, Command::Fit),
        (&config.sc_fit_maximize, Command::Fill),
        (&config.sc_cycle_opening, Command::CycleOpening),
        (&config.sc_keep_zoom, Command::ToggleKeepZoom),
        (&config.sc_keep_pan, Command::ToggleKeepPan),
        (&config.sc_fit_horizontal, Command::FitHorizontal),
        (&config.sc_fit_vertical, Command::FitVertical),
        (&config.sc_zoom, Command::ZoomStep),
        (&config.sc_zoom_in, Command::ZoomBy(step)),
        (&config.sc_zoom_out, Command::ZoomBy(1.0 / step)),
        (
            &config.sc_one_to_one,
            Command::ZoomToPercent(100.0, Anchor::Pointer),
        ),
        (&config.sc_zoom_to_area, Command::ZoomToArea),
        (&config.sc_repeat_place, Command::RepeatPlace),
        (&config.sc_frame, Command::ToggleFrame),
        (&config.sc_more_images_shown, Command::ShowMoreImages),
        (&config.sc_less_images_shown, Command::ShowFewerImages),
        (&config.sc_compare, Command::Compare),
        (&config.sc_drop_pane, Command::DropPane),
        (&config.sc_go_to, Command::GoTo),
        (&config.sc_overlay, Command::CycleOverlay),
        (&config.sc_marks, Command::CycleMarks),
    ];

    let mut commands: Vec<Command> = ctx.input_mut(|input| {
        let mut found: Vec<Command> = bindings
            .iter()
            .filter(|(binding, _)| shortcut::consume(input, binding))
            .map(|(_, command)| *command)
            .collect();

        // Not in the configurable map: these mean the same thing in every
        // list on every platform, and a folder is a list.
        for (key, command) in [
            (egui::Key::Home, Command::First),
            (egui::Key::End, Command::Last),
            (egui::Key::PageDown, Command::PageForward),
            (egui::Key::PageUp, Command::PageBack),
            // The two comparison keys that mean the same thing on every
            // layout. Dropping a pane used to be here as well, on a bare
            // `/`, which on the Slovak, German and French layouts is Shift
            // and a digit — unpressable, and unrebindable because a key read
            // here is a key the editor cannot see.
            (egui::Key::Tab, Command::NextPane),
            (egui::Key::Escape, Command::StopComparing),
        ] {
            if input.consume_key(egui::Modifiers::NONE, key) {
                found.push(command);
            }
        }

        // Escape is the key for "not that", and a person pressing it means
        // whichever of the two is up rather than one of them in particular.
        if found.contains(&Command::StopComparing) {
            found.push(Command::ClearArea);
        }

        // Not a binding, because it cannot be one: egui-winit turns the
        // platform's copy chord into `Event::Copy` and returns before it ever
        // emits a key, so `Ctrl + C` read as a shortcut is a row in the editor
        // that does nothing. Read as the event it actually is, it is right on
        // every platform for nothing — Command and C on a Mac, and the
        // dedicated Copy key where a keyboard has one.
        if input.events.iter().any(|e| matches!(e, egui::Event::Copy)) {
            found.push(Command::CopyArea);
        }

        found
    });

    commands.extend(user_actions(ctx, config));
    commands
}

fn user_actions(ctx: &egui::Context, config: &ImageViewConfig) -> Vec<Command> {
    ctx.input_mut(|input| {
        config
            .user_actions
            .iter()
            .enumerate()
            .filter(|(_, action)| shortcut::consume(input, &action.shortcut))
            .map(|(i, _)| Command::UserAction(i))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Shortcut;
    use eframe::egui::{Event, Key, Modifiers, RawInput};

    fn context_with(events: Vec<Event>) -> egui::Context {
        let ctx = egui::Context::default();
        ctx.begin_pass(RawInput {
            events,
            ..Default::default()
        });
        ctx
    }

    fn key_press(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn maps_arrow_keys_to_navigation() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::ArrowRight)]);

        assert_eq!(collect(&ctx, &config), vec![Command::Next]);
    }

    #[test]
    fn unbound_keys_produce_nothing() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::Z)]);

        assert!(collect(&ctx, &config).is_empty());
    }

    #[test]
    fn user_actions_are_reported_by_position() {
        let mut config = ImageViewConfig::default();
        config.user_actions = vec![crate::config::UserAction {
            shortcut: crate::config::Shortcut::new("z", &[]),
            exec: "true".to_string(),
            callback: None,
        }];

        let ctx = context_with(vec![key_press(Key::Z)]);
        assert_eq!(collect(&ctx, &config), vec![Command::UserAction(0)]);
    }

    /// The two keys this stage rescued: one that could not be pressed on half
    /// the keyboards in Europe, and one that could not be pressed at all.
    #[test]
    fn dropping_a_pane_is_a_binding_now() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::Slash)]);

        assert_eq!(collect(&ctx, &config), vec![Command::DropPane]);
    }

    #[test]
    fn the_slash_can_be_moved_off_a_key_nobody_can_reach() {
        let config = ImageViewConfig {
            sc_drop_pane: Shortcut::new("D", &["ctrl"]),
            ..ImageViewConfig::default()
        };

        let ctx = context_with(vec![Event::Key {
            key: Key::D,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }]);

        assert_eq!(collect(&ctx, &config), vec![Command::DropPane]);
        assert_eq!(
            collect(&context_with(vec![key_press(Key::Slash)]), &config),
            vec![]
        );
    }

    /// The "go to" box could be reached by clicking and by nothing else.
    ///
    /// The modifiers are the ones Windows actually sends — `command` is set
    /// alongside `ctrl` there — because a test that sends only `ctrl` passes
    /// against a build where the real key does nothing.
    #[test]
    fn the_go_to_box_has_a_key() {
        let config = ImageViewConfig::default();
        let held = Modifiers {
            alt: false,
            ctrl: true,
            shift: false,
            mac_cmd: false,
            command: true,
        };

        let ctx = context_with(vec![Event::Key {
            key: Key::J,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: held,
        }]);

        assert_eq!(collect(&ctx, &config), vec![Command::GoTo]);
    }

    /// The marking's two keys.
    ///
    /// Copying is read as the *event*, not as a chord. egui-winit turns
    /// `Ctrl + C` into `Event::Copy` and returns before it emits a key
    /// (`egui-winit/src/lib.rs:818`), so a build that read it as a shortcut
    /// passed a test which sent a key event by hand and did nothing at all
    /// when a person pressed the key. This sends what the platform sends.
    #[test]
    fn the_marking_has_its_two_keys() {
        let config = ImageViewConfig::default();

        assert_eq!(
            collect(&context_with(vec![Event::Copy]), &config),
            vec![Command::CopyArea]
        );
        assert_eq!(
            collect(&context_with(vec![key_press(Key::Enter)]), &config),
            vec![Command::ZoomToArea]
        );
        // And the chord that produces that event is not also a key, so plain
        // `c` goes on cycling the marks.
        assert_eq!(
            collect(&context_with(vec![key_press(Key::C)]), &config),
            vec![Command::CycleMarks]
        );
    }

    /// Escape means "not that", and a person pressing it means whichever of
    /// the two is up rather than one of them in particular.
    #[test]
    fn escape_both_leaves_a_comparison_and_clears_a_marking() {
        let config = ImageViewConfig::default();
        let found = collect(&context_with(vec![key_press(Key::Escape)]), &config);

        assert!(found.contains(&Command::StopComparing), "{found:?}");
        assert!(found.contains(&Command::ClearArea), "{found:?}");
    }

    #[test]
    fn r_repeats_the_last_view() {
        let config = ImageViewConfig::default();
        let ctx = context_with(vec![key_press(Key::R)]);

        assert_eq!(collect(&ctx, &config), vec![Command::RepeatPlace]);
    }

    #[test]
    fn plus_and_minus_zoom() {
        let config = ImageViewConfig::default();

        assert_eq!(
            collect(&context_with(vec![key_press(Key::Plus)]), &config),
            vec![Command::ZoomBy(ZOOM_STEP)]
        );
        assert_eq!(
            collect(&context_with(vec![key_press(Key::Minus)]), &config),
            vec![Command::ZoomBy(1.0 / ZOOM_STEP)]
        );
    }
}
