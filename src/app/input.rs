//! Application wide shortcuts, and the overlays that swallow them.

use eframe::egui;

use crate::app::mode::Mode;
use crate::config::{shortcut, CullConfig, GeneralConfig, HistoryConfig, TagConfig};
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
    /// Turn it a quarter, clockwise or the other way.
    ///
    /// Written to the sidecar and never to the photograph: a raw file cannot
    /// be rewritten without losing something, and a JPEG re-encoded is a JPEG
    /// made worse. It is the most-expected verb after delete and the one most
    /// often implemented by quietly modifying the file.
    Turn(bool),
    /// Send the picture on screen to the platform's bin.
    Delete,
    /// Delete it outright, which is asked about first.
    DeletePermanently,
    /// Fill the screen, or give it back.
    ToggleFullscreen,
    /// Show or hide the bar that narrows and orders the folder.
    ToggleFilter,
    /// Set the rules aside without forgetting them, or put them back.
    SuspendFilter,
    /// Send the photograph somewhere else, or make a copy of it there.
    MoveTo,
    CopyTo,
    /// Move it into the folder for the frames that are not staying.
    ToRejectedFolder,
    /// Put back whatever the last thing did.
    /// Take back the last thing done.
    Undo,
    /// Do again the thing that was last taken back.
    Redo,
    /// Show or hide the list of what has been done.
    ToggleHistoryPanel,
    /// Show the keys, for the mode that is on screen.
    ShowKeys,
    /// Open the whole settings window.
    ShowSettings,
    /// Open the menu for whatever last had the keyboard.
    ///
    /// The keyboard route to the second button. egui cannot read the dedicated
    /// Menu key at all — its key list runs F1 to F35 and grepping it for `Menu`
    /// returns nothing — so this is the only route there is.
    ContextMenu,
    /// Show or hide the strip of thumbnails under the photograph.
    ToggleFilmstrip,
    /// Show the folder stacked, or put every frame back.
    ToggleStacking,
    /// Open or close the stack the cursor is in.
    ToggleStack,
    /// Change which frame stands for that stack.
    StandingBack,
    StandingForward,
    /// Step over a run of frames rather than through it.
    PreviousStack,
    NextStack,
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
pub fn collect(
    ctx: &egui::Context,
    config: &GeneralConfig,
    tags: &TagConfig,
    cull: &CullConfig,
    history: &HistoryConfig,
) -> Vec<Command> {
    let mut commands = Vec::new();

    // Quitting must work even while typing in the navigator.
    if ctx.input_mut(|i| shortcut::consume(i, &config.sc_exit)) {
        commands.push(Command::Exit);
    }

    if utils::are_inputs_muted(ctx) {
        // One exception. `are_inputs_muted` treats any focused widget as mute,
        // and the keyboard route to a menu is exactly what somebody who is
        // typing needs to be able to reach.
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::F10)) {
            commands.push(Command::ContextMenu);
        }

        return commands;
    }

    // Shift first, and consumed, because both of these are read with
    // `key_pressed`, which ignores modifiers entirely — so the more specific
    // shortcut has to be matched first or F10 would answer both.
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::F10)) {
        commands.push(Command::ContextMenu);
    } else if ctx.input(|i| i.key_pressed(egui::Key::F10)) {
        commands.push(Command::ToggleMetrics);
    }

    // Not a configurable binding: it is the key every program uses for this,
    // and somebody who cannot remember the keys cannot look up the key for
    // looking up the keys.
    //
    // Only the question mark. F1 was the obvious companion and is already the
    // menu — which is exactly the kind of thing the startup clash warning
    // exists to catch, and it caught this one.
    if ctx.input(|i| i.key_pressed(egui::Key::Questionmark)) {
        commands.push(Command::ShowKeys);
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
        (&config.sc_filter, Command::ToggleFilter),
        (&config.sc_settings, Command::ShowSettings),
        (&config.sc_suspend_filter, Command::SuspendFilter),
        (&cull.sc_move, Command::MoveTo),
        (&cull.sc_copy, Command::CopyTo),
        (&cull.sc_reject_folder, Command::ToRejectedFolder),
        (&history.sc_undo, Command::Undo),
        (&history.sc_redo, Command::Redo),
        (&history.sc_panel, Command::ToggleHistoryPanel),
        (&config.sc_filmstrip, Command::ToggleFilmstrip),
        (&config.sc_stacks, Command::ToggleStacking),
        (&config.sc_toggle_stack, Command::ToggleStack),
        (&config.sc_standing_back, Command::StandingBack),
        (&config.sc_standing_forward, Command::StandingForward),
        (&config.sc_previous_stack, Command::PreviousStack),
        (&config.sc_next_stack, Command::NextStack),
        (&config.sc_turn_left, Command::Turn(false)),
        (&config.sc_turn_right, Command::Turn(true)),
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

/// Whether Escape means "shut the window in front".
///
/// `typing` is whether a text field had the keyboard on the frame before this
/// one, which the application remembers rather than asks: egui clears the
/// focus itself in `Focus::begin_pass` the moment Escape is pressed, before
/// this program is called, so by the time the key can be read nothing is
/// focused and the question has no answer left in the context.
///
/// The first press leaves the field and the second shuts the window, which is
/// the two-step the rest of the program uses. Consumed rather than read, so
/// the window being shut does not also mean whatever else Escape means that
/// frame — leaving a comparison, or clearing the selection in the sheet.
pub fn escape_shuts_a_window(ctx: &egui::Context, typing: bool) -> bool {
    if typing {
        return false;
    }

    ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
}

/// Opens and closes the overlays.
///
/// While one is open every other shortcut is muted, so that typing a path
/// cannot trigger an action; the muting itself is decided once a frame by the
/// application, which counts an open overlay as a window in front. Escape
/// always closes.
pub fn update_overlay(ctx: &egui::Context, open: &mut Option<Overlay>, config: &GeneralConfig) {
    let toggles = [
        (Overlay::Navigator, &config.sc_navigator),
        (Overlay::DirectoryTree, &config.sc_dir_tree),
    ];

    let escaped = open.is_some() && ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if escaped {
        close(open);
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
                close(open);
            } else {
                *open = Some(overlay);
            }
            return;
        }
    }
}

/// Closes whatever overlay is open.
pub fn close(open: &mut Option<Overlay>) {
    *open = None;
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
        collect(
            ctx,
            &GeneralConfig::default(),
            &TagConfig::default(),
            &CullConfig::default(),
            &HistoryConfig::default(),
        )
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
    fn quitting_works_even_while_something_else_has_the_input() {
        let ctx = context_with(vec![key_press(Key::Q, Modifiers::ALT)]);
        utils::set_window_in_front(&ctx, true);

        assert_eq!(collected(&ctx), vec![Command::Exit]);
    }

    #[test]
    fn other_shortcuts_are_muted_while_something_else_has_the_input() {
        let ctx = context_with(vec![key_press(Key::Backspace, Modifiers::NONE)]);
        utils::set_window_in_front(&ctx, true);

        assert!(collected(&ctx).is_empty());
    }

    #[test]
    fn typing_a_digit_in_the_search_box_does_not_rate_the_image() {
        let ctx = context_with(vec![key_press(Key::Num3, Modifiers::NONE)]);
        utils::set_window_in_front(&ctx, true);

        assert!(collected(&ctx).is_empty());
    }

    /// The overlay's own key still reaches it while everything else is muted,
    /// which is what closes it. Whether the viewer is muted at all is the
    /// application's decision now, made once a frame from what is open.
    #[test]
    fn an_overlay_opens_and_closes_on_its_own_key() {
        let config = GeneralConfig::default();
        let mut open = None;

        let ctx = context_with(vec![key_press(Key::T, Modifiers::NONE)]);
        update_overlay(&ctx, &mut open, &config);
        assert_eq!(open, Some(Overlay::DirectoryTree));

        let ctx = context_with(vec![key_press(Key::T, Modifiers::NONE)]);
        utils::set_window_in_front(&ctx, true);
        update_overlay(&ctx, &mut open, &config);
        assert_eq!(open, None);
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
        utils::set_window_in_front(&ctx, true);
        update_overlay(&ctx, &mut open, &config);

        assert_eq!(open, None);
    }

    /// Escape shuts the window in front, but only once nothing is being typed
    /// into: the first press leaves the search box, the second shuts the
    /// window.
    #[test]
    fn escape_shuts_a_window_only_when_nothing_had_the_keyboard() {
        let ctx = context_with(vec![key_press(Key::Escape, Modifiers::NONE)]);

        assert!(!escape_shuts_a_window(&ctx, true));
        assert!(escape_shuts_a_window(&ctx, false));
    }

    /// And it takes the key with it, so shutting the window does not also
    /// leave the comparison behind it.
    #[test]
    fn shutting_a_window_spends_the_key() {
        let ctx = context_with(vec![key_press(Key::Escape, Modifiers::NONE)]);

        assert!(escape_shuts_a_window(&ctx, false));
        assert!(!escape_shuts_a_window(&ctx, false));
    }
}
