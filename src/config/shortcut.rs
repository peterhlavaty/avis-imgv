//! Keyboard shortcuts as they appear in the configuration file.

use eframe::egui::{Key, KeyboardShortcut, Modifiers};
use serde::{Deserialize, Serialize};

use crate::utils;

pub const MOD_ALT: &str = "alt";
pub const MOD_SHIFT: &str = "shift";
pub const MOD_CTRL: &str = "ctrl";
pub const MOD_MAC_CMD: &str = "mac_cmd";
pub const MOD_CMD: &str = "cmd";

#[derive(Deserialize, Serialize, Clone)]
#[serde(from = "ShortcutData")]
pub struct Shortcut {
    pub key: String,
    pub modifiers: Vec<String>,
    #[serde(skip)]
    #[serde(default = "default_shortcut")]
    pub kbd_shortcut: KeyboardShortcut,
}

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

    #[test]
    fn deserialised_shortcuts_are_ready_to_use() {
        let shortcut: Shortcut =
            serde_json::from_str(r#"{"key":"q","modifiers":["alt"]}"#).expect("parses");

        assert_eq!(shortcut.kbd_shortcut.logical_key, Key::Q);
        assert!(shortcut.kbd_shortcut.modifiers.alt);
    }
}
