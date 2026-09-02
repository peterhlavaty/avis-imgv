//! Keyboard shortcuts as they appear in the configuration file.

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use serde::{Deserialize, Serialize};

use crate::utils;

pub const MOD_ALT: &str = "alt";
pub const MOD_SHIFT: &str = "shift";
pub const MOD_CTRL: &str = "ctrl";
pub const MOD_MAC_CMD: &str = "mac_cmd";
pub const MOD_CMD: &str = "cmd";

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(from = "ShortcutData")]
pub struct Shortcut {
    pub key: String,
    pub modifiers: Vec<String>,
    #[serde(skip)]
    #[serde(default = "default_shortcut")]
    pub kbd_shortcut: KeyboardShortcut,
}

/// Two shortcuts are the same when they name the same key with the same
/// modifiers, whatever order those were written in.
///
/// Written out rather than derived because the modifiers are a list in a
/// configuration file and `["ctrl", "shift"]` is the same shortcut as
/// `["shift", "ctrl"]` — and because the built `KeyboardShortcut` is derived
/// from the other two and has nothing of its own to say.
impl PartialEq for Shortcut {
    fn eq(&self, other: &Self) -> bool {
        if self.key != other.key || self.modifiers.len() != other.modifiers.len() {
            return false;
        }

        let mut mine: Vec<&str> = self.modifiers.iter().map(String::as_str).collect();
        let mut theirs: Vec<&str> = other.modifiers.iter().map(String::as_str).collect();
        mine.sort_unstable();
        theirs.sort_unstable();

        mine == theirs
    }
}

impl Eq for Shortcut {}

#[derive(Deserialize, Serialize)]
pub struct ShortcutData {
    pub key: String,
    pub modifiers: Vec<String>,
}

impl Shortcut {
    /// Builds a shortcut from a key name and modifier names.
    pub fn new(key: &str, modifiers: &[&str]) -> Shortcut {
        let modifiers: Vec<String> = modifiers.iter().map(|x| x.to_string()).collect();
        Shortcut {
            kbd_shortcut: build_keyboard_shortcut(&modifiers, key),
            key: key.to_string(),
            modifiers,
        }
    }
}

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
pub fn consume(input: &mut egui::InputState, shortcut: &Shortcut) -> bool {
    let wanted = shortcut.kbd_shortcut;
    let mut pressed = false;

    input.events.retain(|event| {
        let hit = matches!(
            event,
            egui::Event::Key {
                key,
                modifiers,
                pressed: true,
                ..
            } if *key == wanted.logical_key && matches(*modifiers, wanted)
        );

        pressed |= hit;
        !hit
    });

    pressed
}

/// Whether the modifiers held match the ones a binding asked for.
fn matches(held: Modifiers, wanted: KeyboardShortcut) -> bool {
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

impl From<ShortcutData> for Shortcut {
    fn from(data: ShortcutData) -> Self {
        Shortcut {
            kbd_shortcut: build_keyboard_shortcut(&data.modifiers, &data.key),
            key: data.key,
            modifiers: data.modifiers,
        }
    }
}

pub fn build_keyboard_shortcut(mods: &[String], key: &str) -> KeyboardShortcut {
    let mut modifiers = Modifiers::default();
    for modi in mods {
        match modi.as_str() {
            MOD_ALT => modifiers.alt = true,
            MOD_CTRL => modifiers.ctrl = true,
            MOD_SHIFT => modifiers.shift = true,
            MOD_CMD => modifiers.command = true,
            MOD_MAC_CMD => modifiers.mac_cmd = true,
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

/// The modifier that means "finer", held while a pan key is down.
///
/// A held modifier rather than a binding of its own: it says how far the key
/// beside it moves, and there is nothing for it to do on its own. Alt by
/// default, because it is the one modifier no binding in the viewer uses with
/// a letter — Ctrl is legal and is what most programs use for the careful
/// version of a gesture, but Ctrl with a pan key is a chord a binding can
/// already be sitting on, which is what [`Config::check`] is for.
///
/// Ctrl is read as egui reads a binding asking for control, so it is Command
/// on a Mac.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FineModifier {
    Ctrl,
    Shift,
    #[default]
    Alt,
}

impl FineModifier {
    pub fn value(self) -> &'static str {
        match self {
            FineModifier::Ctrl => MOD_CTRL,
            FineModifier::Shift => MOD_SHIFT,
            FineModifier::Alt => MOD_ALT,
        }
    }

    /// The modifier of that name, or nothing where this build has never heard
    /// of it: a name it cannot read leaves the caller's default in place
    /// rather than turning the fine pan off altogether.
    pub fn of(name: &str) -> Option<FineModifier> {
        [FineModifier::Ctrl, FineModifier::Shift, FineModifier::Alt]
            .into_iter()
            .find(|modifier| modifier.value() == name)
    }

    /// Whether it is down.
    pub fn held(self, modifiers: &Modifiers) -> bool {
        match self {
            FineModifier::Ctrl => modifiers.command,
            FineModifier::Shift => modifiers.shift,
            FineModifier::Alt => modifiers.alt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_shortcut_from_its_parts() {
        let shortcut = Shortcut::new("f", &[MOD_CTRL, MOD_SHIFT]);

        assert_eq!(shortcut.kbd_shortcut.logical_key, Key::F);
        assert!(shortcut.kbd_shortcut.modifiers.ctrl);
        assert!(shortcut.kbd_shortcut.modifiers.shift);
        assert!(!shortcut.kbd_shortcut.modifiers.alt);
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
        let plain = Shortcut::new("1", &[]);
        let with_alt = Shortcut::new("1", &[MOD_ALT]);

        assert!(matches(Modifiers::NONE, plain.kbd_shortcut));
        assert!(!matches(Modifiers::ALT, plain.kbd_shortcut));
        assert!(matches(Modifiers::ALT, with_alt.kbd_shortcut));
        assert!(!matches(Modifiers::NONE, with_alt.kbd_shortcut));
    }

    /// One key apart, and getting it wrong means a photograph nobody can get
    /// back.
    #[test]
    fn shift_is_exact_where_it_can_be() {
        let plain = Shortcut::new("delete", &[]);
        let with_shift = Shortcut::new("delete", &[MOD_SHIFT]);

        assert!(matches(Modifiers::NONE, plain.kbd_shortcut));
        assert!(!matches(Modifiers::SHIFT, plain.kbd_shortcut));
        assert!(matches(Modifiers::SHIFT, with_shift.kbd_shortcut));
    }

    /// And loose where it cannot be: on a Slovak or German keyboard the digits
    /// are the shifted characters of the top row.
    #[test]
    fn shift_is_forgiven_on_the_keys_that_need_it_to_be_typed() {
        let three = Shortcut::new("3", &[]);
        let plus = Shortcut::new("Plus", &[]);

        assert!(matches(Modifiers::SHIFT, three.kbd_shortcut));
        assert!(matches(Modifiers::SHIFT, plus.kbd_shortcut));

        // Alt is still exact on those keys.
        assert!(!matches(Modifiers::ALT, three.kbd_shortcut));
    }

    /// Control and command go through egui's own reconciliation rather than a
    /// plain comparison, because the two are one field on some platforms and
    /// two on others. The configuration keeps them apart with `ctrl` and
    /// `cmd`, and a binding on either fires for its own key alone.
    #[test]
    fn control_and_command_keep_their_own_meanings() {
        let with_ctrl = Shortcut::new("f", &[MOD_CTRL]);
        let with_cmd = Shortcut::new("f", &[MOD_CMD]);

        assert!(matches(Modifiers::CTRL, with_ctrl.kbd_shortcut));
        assert!(!matches(Modifiers::NONE, with_ctrl.kbd_shortcut));

        assert!(matches(Modifiers::COMMAND, with_cmd.kbd_shortcut));
        assert!(!matches(Modifiers::NONE, with_cmd.kbd_shortcut));
    }

    /// A binding with no control still refuses one that is held, which is what
    /// keeps `Ctrl + Backspace` out of the gallery toggle.
    #[test]
    fn control_is_exclusive_too() {
        let plain = Shortcut::new("backspace", &[]);

        assert!(matches(Modifiers::NONE, plain.kbd_shortcut));
        assert!(!matches(Modifiers::CTRL, plain.kbd_shortcut));
    }

    #[test]
    fn deserialised_shortcuts_are_ready_to_use() {
        let shortcut: Shortcut =
            serde_json::from_str(r#"{"key":"q","modifiers":["alt"]}"#).expect("parses");

        assert_eq!(shortcut.kbd_shortcut.logical_key, Key::Q);
        assert!(shortcut.kbd_shortcut.modifiers.alt);
    }
}
