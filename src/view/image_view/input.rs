//! Turning key presses into view commands.
//!
//! Collecting commands rather than mutating the view directly keeps the
//! keyboard map in one readable place and lets the status bar issue the same
//! commands as the keyboard.

use eframe::egui;

use crate::config::ImageViewConfig;
use crate::utils;

/// Something the image view can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Next,
    Previous,
    /// Fit the whole image in the panel.
    Fit,
    /// Fill the panel, cropping the overflowing side.
    Fill,
    /// Keep filling the panel as the user navigates.
    ToggleFillLatch,
    FitHorizontal,
    FitVertical,
    /// Double the magnification, wrapping back to fitted.
    ZoomStep,
    /// Magnify to a percentage of the image's own pixels.
    ZoomToPercent(f32),
    ToggleFrame,
    ShowMoreImages,
    ShowFewerImages,
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

    let bindings = [
        (&config.sc_next, Command::Next),
        (&config.sc_prev, Command::Previous),
        (&config.sc_fit, Command::Fit),
        (&config.sc_fit_maximize, Command::Fill),
        (&config.sc_latch_fit_maximize, Command::ToggleFillLatch),
        (&config.sc_fit_horizontal, Command::FitHorizontal),
        (&config.sc_fit_vertical, Command::FitVertical),
        (&config.sc_zoom, Command::ZoomStep),
        (&config.sc_one_to_one, Command::ZoomToPercent(100.0)),
        (&config.sc_frame, Command::ToggleFrame),
        (&config.sc_more_images_shown, Command::ShowMoreImages),
        (&config.sc_less_images_shown, Command::ShowFewerImages),
    ];

    let mut commands: Vec<Command> = ctx.input_mut(|input| {
        bindings
            .iter()
            .filter(|(shortcut, _)| input.consume_shortcut(&shortcut.kbd_shortcut))
            .map(|(_, command)| *command)
            .collect()
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
            .filter(|(_, action)| input.consume_shortcut(&action.shortcut.kbd_shortcut))
            .map(|(i, _)| Command::UserAction(i))
            .collect()
    })
}

/// Navigation driven by the scroll wheel, which only applies while the pointer
/// is over the image and is not being used to zoom.
pub fn scroll_navigation(ctx: &egui::Context, hovered: bool) -> Option<Command> {
    if !hovered || ctx.input(|i| i.zoom_delta()) != 1.0 {
        return None;
    }

    match ctx.input(|i| i.raw_scroll_delta.y) {
        delta if delta > 0.0 => Some(Command::Next),
        delta if delta < 0.0 => Some(Command::Previous),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn scroll_navigation_needs_the_pointer_over_the_image() {
        let ctx = context_with(vec![]);
        assert_eq!(scroll_navigation(&ctx, false), None);
        assert_eq!(scroll_navigation(&ctx, true), None);
    }
}
