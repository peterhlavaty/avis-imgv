//! Application wide shortcuts, and the overlays that swallow them.

use eframe::egui;

use crate::app::mode::Mode;
use crate::config::{shortcut, GeneralConfig, TagConfig};
use crate::metadata::xmp::{Flag, Label};
use crate::utils;

/// Something the application can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Exit,
    ToggleGrid,
    /// Move to the next mode round.
    NextMode,
    /// Go straight to one, as the menu does.
    SetMode(Mode),
    ToggleMenu,
    ToggleSidePanel,
    ToggleMetrics,
    ToggleFlatten,
    ToggleWatcher,
    ToggleTagPanel,
    /// Put this many stars on the image on screen.
    SetRating(u8),
    /// Keep it, throw it out, or take the mark back off.
    SetFlag(Flag),
    /// Put this colour label on it, by its position in [`Label::CHOICES`].
    SetLabel(usize),
    /// Move to the next photograph after every mark, or stop doing that.
    ToggleAdvance,
    /// Send the picture on screen to the platform's bin.
    Delete,
    /// Delete it outright, which is asked about first.
    DeletePermanently,
    /// Fill the screen, or give it back.
    ToggleFullscreen,
}

impl Command {
    /// Whether this is a mark, and so whether it may advance to the next
    /// photograph once it has been applied.
    fn is_a_mark(self) -> bool {
        matches!(
            self,
            Command::SetRating(_) | Command::SetFlag(_) | Command::SetLabel(_)
        )
    }
}

/// Overlays that take over the keyboard while open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    Navigator,
    DirectoryTree,
}

/// Reads this frame's input and returns the commands it maps to.
pub fn collect(ctx: &egui::Context, config: &GeneralConfig, tags: &TagConfig) -> Vec<Command> {
    let mut commands = Vec::new();

    // Quitting must work even while typing in the navigator.
    if ctx.input_mut(|i| shortcut::consume(i, &config.sc_exit)) {
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
        (&config.sc_next_mode, Command::NextMode),
        (&config.sc_menu, Command::ToggleMenu),
        (&config.sc_toggle_side_panel, Command::ToggleSidePanel),
        (&config.sc_flatten_dir, Command::ToggleFlatten),
        (&config.sc_watch_directory, Command::ToggleWatcher),
        (&config.sc_delete, Command::Delete),
        (&config.sc_delete_permanently, Command::DeletePermanently),
        (&config.sc_fullscreen, Command::ToggleFullscreen),
    ];

    ctx.input_mut(|input| {
        commands.extend(
            bindings
                .iter()
                .filter(|(binding, _)| shortcut::consume(input, binding))
                .map(|(_, command)| *command),
        );

        if shortcut::consume(input, &tags.sc_toggle_tag_panel) {
            commands.push(Command::ToggleTagPanel);
        }

        if shortcut::consume(input, &tags.sc_toggle_advance) {
            commands.push(Command::ToggleAdvance);
        }

        // The rating shortcuts are listed from no stars upwards, so a
        // shortcut's position is the rating it applies.
        commands.extend(
            tags.sc_rating
                .iter()
                .enumerate()
                .filter(|(_, binding)| shortcut::consume(input, binding))
                .map(|(stars, _)| Command::SetRating(stars as u8)),
        );

        for (shortcut, flag) in [
            (&tags.sc_pick, Flag::Picked),
            (&tags.sc_reject, Flag::Rejected),
            (&tags.sc_unflag, Flag::Unflagged),
        ] {
            if shortcut::consume(input, shortcut) {
                commands.push(Command::SetFlag(flag));
            }
        }

        // As with the ratings, a shortcut's position is the label it applies.
        commands.extend(
            tags.sc_label
                .iter()
                .enumerate()
                .take(Label::CHOICES.len())
                .filter(|(_, binding)| shortcut::consume(input, binding))
                .map(|(index, _)| Command::SetLabel(index)),
        );
    });

    commands
}

/// Whether a mark should be followed by a move to the next photograph.
///
/// A mode rather than a modifier, the way Lightroom does it. A modifier would
/// have been cheaper, and it does not work: on a Slovak or German keyboard the
/// digits are the shifted characters of the top row, so every rating would
/// arrive with shift held and every rating would advance.
pub fn advances(command: Command, advancing: bool) -> bool {
    advancing && command.is_a_mark()
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

        if ctx.input_mut(|i| shortcut::consume(i, shortcut)) {
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
        let ctx = context_with(vec![key_press(Key::Backspace, Modifiers::NONE)]);

        assert_eq!(collected(&ctx), vec![Command::ToggleGrid]);
    }

    /// Collects with the default configuration.
    fn collected(ctx: &egui::Context) -> Vec<Command> {
        collect(ctx, &GeneralConfig::default(), &TagConfig::default())
    }

    #[test]
    fn a_digit_key_sets_that_many_stars() {
        let ctx = context_with(vec![key_press(Key::Num4, Modifiers::NONE)]);
        assert_eq!(collected(&ctx), vec![Command::SetRating(4)]);

        let ctx = context_with(vec![key_press(Key::Num0, Modifiers::NONE)]);
        assert_eq!(collected(&ctx), vec![Command::SetRating(0)]);
    }

    #[test]
    fn the_tag_panel_has_a_shortcut_of_its_own() {
        let ctx = context_with(vec![key_press(Key::K, Modifiers::NONE)]);

        assert_eq!(collected(&ctx), vec![Command::ToggleTagPanel]);
    }

    #[test]
    fn quitting_works_even_while_typing() {
        let ctx = context_with(vec![key_press(Key::Q, Modifiers::ALT)]);
        utils::set_mute_state(&ctx, true);

        assert_eq!(collected(&ctx), vec![Command::Exit]);
    }

    #[test]
    fn other_shortcuts_are_muted_while_typing() {
        let ctx = context_with(vec![key_press(Key::Backspace, Modifiers::NONE)]);
        utils::set_mute_state(&ctx, true);

        assert!(collected(&ctx).is_empty());
    }

    #[test]
    fn typing_a_digit_in_the_search_box_does_not_rate_the_image() {
        let ctx = context_with(vec![key_press(Key::Num3, Modifiers::NONE)]);
        utils::set_mute_state(&ctx, true);

        assert!(collected(&ctx).is_empty());
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
    fn the_flag_keys_are_lightrooms() {
        for (key, flag) in [
            (Key::P, Flag::Picked),
            (Key::X, Flag::Rejected),
            (Key::U, Flag::Unflagged),
        ] {
            let ctx = context_with(vec![key_press(key, Modifiers::NONE)]);
            assert_eq!(collected(&ctx), vec![Command::SetFlag(flag)], "{key:?}");
        }
    }

    #[test]
    fn the_digits_above_the_ratings_are_the_colour_labels() {
        let ctx = context_with(vec![key_press(Key::Num6, Modifiers::NONE)]);
        assert_eq!(collected(&ctx), vec![Command::SetLabel(0)]);

        let ctx = context_with(vec![key_press(Key::Num9, Modifiers::NONE)]);
        assert_eq!(collected(&ctx), vec![Command::SetLabel(3)]);
    }

    #[test]
    fn a_mark_advances_only_when_it_is_meant_to() {
        assert!(!advances(Command::SetRating(3), false));
        assert!(advances(Command::SetRating(3), true));
        assert!(advances(Command::SetFlag(Flag::Rejected), true));
        assert!(advances(Command::SetLabel(0), true));

        // Nothing that is not a mark ever advances.
        assert!(!advances(Command::ToggleGrid, true));
        assert!(!advances(Command::ToggleAdvance, true));
    }

    /// Shift is exclusive here because the two are one key apart, and getting
    /// it wrong means deleting a photograph nobody can get back.
    #[test]
    fn the_two_deletes_are_told_apart() {
        let ctx = context_with(vec![key_press(Key::Delete, Modifiers::NONE)]);
        assert_eq!(collected(&ctx), vec![Command::Delete]);

        let ctx = context_with(vec![key_press(Key::Delete, Modifiers::SHIFT)]);
        assert_eq!(collected(&ctx), vec![Command::DeletePermanently]);
    }

    #[test]
    fn advancing_is_a_mode_with_a_key_of_its_own() {
        let ctx = context_with(vec![key_press(
            Key::A,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Modifiers::NONE
            },
        )]);

        assert_eq!(collected(&ctx), vec![Command::ToggleAdvance]);
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
