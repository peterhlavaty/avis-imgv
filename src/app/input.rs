//! Application wide shortcuts, and the overlays that swallow them.

use eframe::egui;

use crate::config::GeneralConfig;
use crate::utils;

/// Something the application can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Exit,
    ToggleGrid,
    ToggleMenu,
    ToggleSidePanel,
    ToggleMetrics,
    ToggleFlatten,
    ToggleWatcher,
}

/// Overlays that take over the keyboard while open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Navigator,
    DirectoryTree,
}

/// Reads this frame's input and returns the commands it maps to.
pub fn collect(ctx: &egui::Context, config: &GeneralConfig) -> Vec<Command> {
    let mut commands = Vec::new();

    // Quitting must work even while typing in the navigator.
    if ctx.input_mut(|i| i.consume_shortcut(&config.sc_exit.kbd_shortcut)) {
        commands.push(Command::Exit);
    }

    if utils::are_inputs_muted(ctx) {
        return commands;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::F10)) {
        commands.push(Command::ToggleMetrics);
    }

    let bindings = [
        (&config.sc_toggle_gallery, Command::ToggleGrid),
        (&config.sc_menu, Command::ToggleMenu),
        (&config.sc_toggle_side_panel, Command::ToggleSidePanel),
        (&config.sc_flatten_dir, Command::ToggleFlatten),
        (&config.sc_watch_directory, Command::ToggleWatcher),
    ];

    ctx.input_mut(|input| {
        commands.extend(
            bindings
                .iter()
                .filter(|(shortcut, _)| input.consume_shortcut(&shortcut.kbd_shortcut))
                .map(|(_, command)| *command),
        );
    });

    commands
}

/// Opens and closes the overlays, keeping the input mute flag in step.
///
/// While one is open every other shortcut is muted so that typing a path
/// cannot trigger an action; Escape always closes.
pub fn update_overlay(ctx: &egui::Context, open: &mut Option<Overlay>, config: &GeneralConfig) {
    let toggles = [
        (Overlay::Navigator, &config.sc_navigator),
        (Overlay::DirectoryTree, &config.sc_dir_tree),
    ];

    let escaped = open.is_some() && ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if escaped {
        close(ctx, open);
        return;
    }

    for (overlay, shortcut) in toggles {
        // The overlay that is open owns its own shortcut even while muted.
        let mine = *open == Some(overlay);
        if !mine && utils::are_inputs_muted(ctx) {
            continue;
        }

        if ctx.input_mut(|i| i.consume_shortcut(&shortcut.kbd_shortcut)) {
            if mine {
                close(ctx, open);
            } else {
                *open = Some(overlay);
                utils::set_mute_state(ctx, true);
            }
            return;
        }
    }
}

/// Closes whatever overlay is open and gives the keyboard back.
pub fn close(ctx: &egui::Context, open: &mut Option<Overlay>) {
    *open = None;
    utils::set_mute_state(ctx, false);
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
    fn maps_the_default_shortcuts() {
        let config = GeneralConfig::default();
        let ctx = context_with(vec![key_press(Key::Backspace, Modifiers::NONE)]);

        assert_eq!(collect(&ctx, &config), vec![Command::ToggleGrid]);
    }

    #[test]
    fn quitting_works_even_while_typing() {
        let config = GeneralConfig::default();
        let ctx = context_with(vec![key_press(Key::Q, Modifiers::ALT)]);
        utils::set_mute_state(&ctx, true);

        assert_eq!(collect(&ctx, &config), vec![Command::Exit]);
    }

    #[test]
    fn other_shortcuts_are_muted_while_typing() {
        let config = GeneralConfig::default();
        let ctx = context_with(vec![key_press(Key::Backspace, Modifiers::NONE)]);
        utils::set_mute_state(&ctx, true);

        assert!(collect(&ctx, &config).is_empty());
    }

    #[test]
    fn an_overlay_opens_mutes_and_closes() {
        let config = GeneralConfig::default();
        let mut open = None;

        let ctx = context_with(vec![key_press(Key::T, Modifiers::NONE)]);
        update_overlay(&ctx, &mut open, &config);
        assert_eq!(open, Some(Overlay::DirectoryTree));
        assert!(utils::are_inputs_muted(&ctx));

        let ctx = context_with(vec![key_press(Key::T, Modifiers::NONE)]);
        utils::set_mute_state(&ctx, true);
        update_overlay(&ctx, &mut open, &config);
        assert_eq!(open, None);
        assert!(!utils::are_inputs_muted(&ctx));
    }

    #[test]
    fn escape_closes_any_overlay() {
        let config = GeneralConfig::default();
        let mut open = Some(Overlay::Navigator);

        let ctx = context_with(vec![key_press(Key::Escape, Modifiers::NONE)]);
        utils::set_mute_state(&ctx, true);
        update_overlay(&ctx, &mut open, &config);

        assert_eq!(open, None);
        assert!(!utils::are_inputs_muted(&ctx));
    }
}
