//! Reading a press against what the configuration asked for.

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};

use super::Shortcut;
use crate::utils;

/// Takes this frame's press of `shortcut`, if it is there.
///
/// egui's own `consume_shortcut` asks only whether the modifiers a binding
/// *wants* are held, not whether any others are — so every unmodified binding
/// also fired with alt down, and `Alt + 1` both zoomed to 100% and put one
/// star on the photograph. Alt is required to match exactly here; control and
/// command go through egui's own reconciliation, which knows that a binding
/// asking for control means command on a Mac.
///
/// Shift is only exact for the keys where it can be: see [`shift_is_exact`].
///
/// A shortcut carrying several chords answers to any of them, and one carrying
/// none answers to nothing — the loop over an empty list, rather than a check
/// somebody has to remember to write.
pub fn consume(input: &mut egui::InputState, shortcut: &Shortcut) -> bool {
    let mut pressed = false;

    input.events.retain(|event| {
        let hit = matches!(
            event,
            egui::Event::Key {
                key,
                modifiers,
                pressed: true,
                ..
            } if shortcut.chords().iter().any(|chord| {
                *key == chord.kbd_shortcut.logical_key && matches(*modifiers, chord.kbd_shortcut)
            })
        );

        pressed |= hit;
        !hit
    });

    pressed
}

/// Whether the modifiers held match the ones a binding asked for.
pub(super) fn matches(held: Modifiers, wanted: KeyboardShortcut) -> bool {
    if held.alt != wanted.modifiers.alt {
        return false;
    }

    if shift_is_exact(wanted.logical_key) && held.shift != wanted.modifiers.shift {
        return false;
    }

    held.cmd_ctrl_matches(wanted.modifiers)
}

/// Whether shift has to match exactly for this key.
///
/// Not for the digits or the arithmetic keys. On a Slovak or German keyboard
/// the digits *are* the shifted characters of the top row, and `+` needs shift
/// on most layouts including the American one, so requiring shift to be absent
/// would leave those bindings unreachable for the people who have to press it.
/// Everywhere else it is exact, which is what keeps `Shift + Delete` from also
/// being `Delete`.
fn shift_is_exact(key: Key) -> bool {
    !matches!(
        key,
        Key::Num0
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Num5
            | Key::Num6
            | Key::Num7
            | Key::Num8
            | Key::Num9
            | Key::Plus
            | Key::Minus
            | Key::Equals
    )
}

/// Whether egui can read a key by this name.
///
/// An unknown name becomes the unreachable sentinel `Ctrl+Alt+Shift+Cmd+F20`,
/// so a typo makes a command permanently unpressable and the only record used
/// to be a log line. Both spellings are accepted, since `Key::from_name` takes
/// several for the same key.
pub fn names_a_key(name: &str) -> bool {
    Key::from_name(name).is_some() || Key::from_name(&utils::capitalize_first_char(name)).is_some()
}

pub fn default_shortcut() -> KeyboardShortcut {
    //Bogus shortcut as default so we don't have to use option
    //Easier when implementing the shortcuts
    //We use F20 as most users don't have it and all modifiers
    let modi = Modifiers {
        alt: true,
        ctrl: true,
        shift: true,
        command: true,
        mac_cmd: false,
    };

    KeyboardShortcut::new(modi, Key::F20)
}

pub fn build_keyboard_shortcut(mods: &[String], key: &str) -> KeyboardShortcut {
    let mut modifiers = Modifiers::default();
    for modi in mods {
        match modi.as_str() {
            super::MOD_ALT => modifiers.alt = true,
            super::MOD_CTRL => modifiers.ctrl = true,
            super::MOD_SHIFT => modifiers.shift = true,
            super::MOD_CMD => modifiers.command = true,
            super::MOD_MAC_CMD => modifiers.mac_cmd = true,
            _ => {
                tracing::error!("Invalid modifier({}) in configuration", modi.as_str())
            }
        }
    }

    match Key::from_name(&utils::capitalize_first_char(key)) {
        Some(key) => KeyboardShortcut {
            logical_key: key,
            modifiers,
        },
        None => {
            tracing::error!("Invalid shortcut key: {key}");
            default_shortcut()
        } //uses default unreachable shortcut
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::shortcut::{Chord, MOD_ALT, MOD_CMD, MOD_CTRL, MOD_SHIFT};

    fn built(key: &str, modifiers: &[&str]) -> KeyboardShortcut {
        Chord::new(key, modifiers).kbd_shortcut
    }

    #[test]
    fn key_names_are_case_insensitive() {
        assert_eq!(
            build_keyboard_shortcut(&[], "backspace").logical_key,
            Key::Backspace
        );
        assert_eq!(
            build_keyboard_shortcut(&[], "Backspace").logical_key,
            Key::Backspace
        );
    }

    #[test]
    fn an_invalid_key_falls_back_to_an_unreachable_shortcut() {
        let built = build_keyboard_shortcut(&[MOD_CTRL.to_string()], "not-a-key");
        assert_eq!(built, default_shortcut());
    }

    #[test]
    fn an_invalid_modifier_is_ignored() {
        let built = build_keyboard_shortcut(&["nonsense".to_string(), MOD_ALT.to_string()], "q");
        assert!(built.modifiers.alt);
        assert!(!built.modifiers.ctrl);
    }

    /// The bug this was written for: every unmodified binding also fired with
    /// alt down, so `Alt + 1` zoomed to 100% *and* put one star on the
    /// photograph.
    #[test]
    fn a_binding_without_alt_does_not_fire_with_alt_held() {
        assert!(matches(Modifiers::NONE, built("1", &[])));
        assert!(!matches(Modifiers::ALT, built("1", &[])));
        assert!(matches(Modifiers::ALT, built("1", &[MOD_ALT])));
        assert!(!matches(Modifiers::NONE, built("1", &[MOD_ALT])));
    }

    /// One key apart, and getting it wrong means a photograph nobody can get
    /// back.
    #[test]
    fn shift_is_exact_where_it_can_be() {
        assert!(matches(Modifiers::NONE, built("delete", &[])));
        assert!(!matches(Modifiers::SHIFT, built("delete", &[])));
        assert!(matches(Modifiers::SHIFT, built("delete", &[MOD_SHIFT])));
    }

    /// And loose where it cannot be: on a Slovak or German keyboard the digits
    /// are the shifted characters of the top row.
    #[test]
    fn shift_is_forgiven_on_the_keys_that_need_it_to_be_typed() {
        assert!(matches(Modifiers::SHIFT, built("3", &[])));
        assert!(matches(Modifiers::SHIFT, built("Plus", &[])));

        // Alt is still exact on those keys.
        assert!(!matches(Modifiers::ALT, built("3", &[])));
    }

    /// Control and command go through egui's own reconciliation rather than a
    /// plain comparison, because the two are one field on some platforms and
    /// two on others. The configuration keeps them apart with `ctrl` and
    /// `cmd`, and a binding on either fires for its own key alone.
    #[test]
    fn control_and_command_keep_their_own_meanings() {
        assert!(matches(Modifiers::CTRL, built("f", &[MOD_CTRL])));
        assert!(!matches(Modifiers::NONE, built("f", &[MOD_CTRL])));

        assert!(matches(Modifiers::COMMAND, built("f", &[MOD_CMD])));
        assert!(!matches(Modifiers::NONE, built("f", &[MOD_CMD])));
    }

    /// A binding with no control still refuses one that is held, which is what
    /// keeps `Ctrl + Backspace` out of the gallery toggle.
    #[test]
    fn control_is_exclusive_too() {
        assert!(matches(Modifiers::NONE, built("backspace", &[])));
        assert!(!matches(Modifiers::CTRL, built("backspace", &[])));
    }

    /// A press of the second key does the command, and is taken off the queue
    /// so nothing behind it sees the same press.
    #[test]
    fn any_chord_of_a_shortcut_takes_the_press() {
        let mut shortcut = Shortcut::new("d", &[]);
        shortcut.add(Chord::new("ArrowRight", &[]));

        for key in [Key::D, Key::ArrowRight] {
            let mut input = egui::InputState::default();
            input.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            });

            assert!(consume(&mut input, &shortcut), "{key:?} was not read");
            assert!(input.events.is_empty(), "{key:?} was left on the queue");
        }
    }

    /// And a key that is on neither chord is left where it was.
    #[test]
    fn a_key_on_no_chord_is_left_alone() {
        let mut shortcut = Shortcut::new("d", &[]);
        shortcut.add(Chord::new("ArrowRight", &[]));

        let mut input = egui::InputState::default();
        input.events.push(egui::Event::Key {
            key: Key::N,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        });

        assert!(!consume(&mut input, &shortcut));
        assert_eq!(input.events.len(), 1);
    }

    /// A command with no key answers to nothing, which is the loop over an
    /// empty list rather than a check anybody has to write.
    #[test]
    fn an_unbound_shortcut_reads_no_press_at_all() {
        let mut input = egui::InputState::default();
        input.events.push(egui::Event::Key {
            key: Key::D,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        });

        assert!(!consume(&mut input, &Shortcut::unbound()));
        assert_eq!(input.events.len(), 1);
    }
}
