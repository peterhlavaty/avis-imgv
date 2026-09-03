//! Reading the key somebody just pressed, so it can be written down.

use eframe::egui::{self, Key, Modifiers};

use crate::config::shortcut::{MOD_ALT, MOD_CTRL, MOD_MAC_CMD, MOD_SHIFT};
use crate::config::Chord;

/// What a window waiting for a key saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Pressed(Chord),
    /// Escape: leave the binding as it was.
    Cancelled,
}

/// The key pressed this frame, as a chord.
///
/// Delete and Backspace are ordinary keys here. They used to mean "no key at
/// all", which was the only way to unbind a command and cost the two of them
/// being unbindable — including Delete, which is what sends a photograph to
/// the bin and is therefore the one key somebody rearranging their keyboard is
/// most likely to want to move. A command with no keys is now the list with
/// nothing left in it, which is a state the window can both show and reach.
pub fn captured(ctx: &egui::Context) -> Option<Captured> {
    let pressed = ctx.input(|input| {
        input.events.iter().find_map(|event| match event {
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => Some((*key, *modifiers)),
            _ => None,
        })
    });

    let (key, modifiers) = pressed?;

    if key == Key::Escape && modifiers.is_none() {
        return Some(Captured::Cancelled);
    }

    Some(Captured::Pressed(Chord::new(
        &canonical(key),
        &modifier_names(modifiers),
    )))
}

/// The modifiers held, in the words the file uses.
///
/// `cmd` is emitted on macOS rather than folding Command into Ctrl, because on
/// that platform they are two different keys and a file that says `ctrl` asks
/// for the one nobody presses.
fn modifier_names(modifiers: Modifiers) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();

    if modifiers.ctrl {
        names.push(MOD_CTRL);
    }
    if modifiers.mac_cmd || (cfg!(target_os = "macos") && modifiers.command && !modifiers.ctrl) {
        names.push(MOD_MAC_CMD);
    }
    if modifiers.alt {
        names.push(MOD_ALT);
    }
    if modifiers.shift {
        names.push(MOD_SHIFT);
    }

    names
}

/// The spelling of a key that reads back as the same key.
///
/// `Key::name` gives "PageUp"; `capitalize_first_char` turns whatever is
/// written into "Pageup", which `Key::from_name` rejects — so those two of the
/// eighty names became the unreachable sentinel on the way back in. Written
/// canonically here, and both spellings accepted on read.
fn canonical(key: Key) -> String {
    key.name().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every key name the editor writes has to read back as the same key, or a
    /// rebind is a command made unreachable.
    #[test]
    fn every_captured_key_name_reads_back() {
        for key in Key::ALL {
            let written = canonical(*key);
            assert!(
                crate::config::shortcut::names_a_key(&written),
                "{written} does not read back as a key"
            );
        }
    }

    #[test]
    fn escape_leaves_the_binding_alone() {
        assert_eq!(pressed(Key::Escape, Modifiers::NONE), Captured::Cancelled);
    }

    /// The bin is on Delete, so it is the key somebody rearranging their
    /// keyboard is most likely to want to move — and it could not be bound at
    /// all while a bare press of it meant "no key".
    #[test]
    fn delete_and_backspace_are_keys_like_any_other() {
        for key in [Key::Delete, Key::Backspace, Key::Escape] {
            assert!(
                matches!(pressed(key, Modifiers::CTRL), Captured::Pressed(_)),
                "Ctrl + {key:?} was not taken as a key"
            );
        }

        assert!(matches!(
            pressed(Key::Delete, Modifiers::NONE),
            Captured::Pressed(_)
        ));
    }

    #[test]
    fn the_modifiers_held_are_written_down() {
        let Captured::Pressed(chord) = pressed(Key::F, Modifiers::CTRL | Modifiers::SHIFT) else {
            panic!("a key was pressed");
        };

        assert_eq!(chord, Chord::new("F", &[MOD_CTRL, MOD_SHIFT]));
    }

    fn pressed(key: Key, modifiers: Modifiers) -> Captured {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            }],
            ..Default::default()
        });

        captured(&ctx).expect("a key was pressed")
    }
}
