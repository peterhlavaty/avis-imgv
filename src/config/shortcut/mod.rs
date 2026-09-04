//! Keyboard shortcuts as they appear in the configuration file.
//!
//! A shortcut is a *list* of chords rather than one, because a photographer
//! coming from another viewer wants the key they already know beside the one
//! this program chose, and because the arrow keys and `WASD` are the same
//! command by any reasonable reading. The file keeps the first chord where it
//! always was — `{"key": "d", "modifiers": []}` — and writes the rest under
//! `also` only when there are any, so a configuration nobody has added a key
//! to is byte-identical to the one an older build wrote.
//!
//! An empty list is a command with no key. That was already sayable by writing
//! `"key": ""`, and was already read that way in four places; now it is one
//! state of one type rather than a blank string four readers have to remember
//! to check.

mod fine;
mod press;

use eframe::egui::KeyboardShortcut;
use serde::{Deserialize, Serialize};

pub use fine::FineModifier;
pub use press::{build_keyboard_shortcut, consume, default_shortcut, names_a_key};

pub const MOD_ALT: &str = "alt";
pub const MOD_SHIFT: &str = "shift";
pub const MOD_CTRL: &str = "ctrl";
pub const MOD_MAC_CMD: &str = "mac_cmd";
pub const MOD_CMD: &str = "cmd";

/// One key with the modifiers held down with it — one thing a finger does.
///
/// What the whole of `Shortcut` used to be, and still what the file holds:
/// splitting the name out is what let a command carry more than one of them
/// without changing how any of them is written down.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(from = "ChordData")]
pub struct Chord {
    pub key: String,
    pub modifiers: Vec<String>,
    #[serde(skip)]
    #[serde(default = "default_shortcut")]
    pub kbd_shortcut: KeyboardShortcut,
}

/// Two chords are the same when they are the same press.
///
/// Decided on the built `KeyboardShortcut` rather than on the two strings it
/// came from, because the strings say the same thing several ways and the
/// question every caller is asking is whether one press does both: the
/// modifiers are a list, so `["ctrl", "shift"]` is `["shift", "ctrl"]`; a
/// modifier this build cannot read is dropped, so `["nonsense", "alt"]` is
/// `["alt"]`; and `Key::from_name` takes "delete", "Delete" and "Esc" for
/// keys it also takes other names for, so a comparison minding the spelling
/// called two spellings of one chord two chords and let a clash through.
///
/// The exception is the two ends of that: a key name egui cannot read at all
/// becomes the unreachable sentinel, and every typo builds the *same*
/// sentinel. Those are compared by what was written instead, so two different
/// mistakes are two rows to be told about rather than one.
impl PartialEq for Chord {
    fn eq(&self, other: &Self) -> bool {
        if self.kbd_shortcut != other.kbd_shortcut {
            return false;
        }

        if self.kbd_shortcut != default_shortcut() {
            return true;
        }

        self.key.eq_ignore_ascii_case(&other.key) && self.same_modifiers(other)
    }
}

impl Eq for Chord {}

impl Chord {
    /// Builds a chord from a key name and modifier names.
    pub fn new(key: &str, modifiers: &[&str]) -> Chord {
        let modifiers: Vec<String> = modifiers.iter().map(|x| x.to_string()).collect();
        Chord {
            kbd_shortcut: build_keyboard_shortcut(&modifiers, key),
            key: key.to_string(),
            modifiers,
        }
    }

    /// Whether it names no key at all, which is how a command is left unbound.
    pub fn is_blank(&self) -> bool {
        self.key.trim().is_empty()
    }

    /// Whether egui can read the key it names.
    pub fn is_readable(&self) -> bool {
        !self.is_blank() && names_a_key(&self.key)
    }

    /// The same set of modifiers, whatever order they were written in.
    fn same_modifiers(&self, other: &Chord) -> bool {
        if self.modifiers.len() != other.modifiers.len() {
            return false;
        }

        let mut mine: Vec<&str> = self.modifiers.iter().map(String::as_str).collect();
        let mut theirs: Vec<&str> = other.modifiers.iter().map(String::as_str).collect();
        mine.sort_unstable();
        theirs.sort_unstable();

        mine == theirs
    }
}

/// Every key that does one thing.
///
/// Never holds a blank chord: one read from the file is dropped on the way in,
/// so "no key" is the empty list and nothing downstream has to know that a
/// blank key name once stood for it.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(from = "ShortcutData", into = "ShortcutData")]
pub struct Shortcut {
    chords: Vec<Chord>,
}

/// Two shortcuts are the same when they hold the same chords in the same
/// order. The order is not decoration: it decides which chord is the first,
/// and the first is the one every menu and tooltip in the program prints.
impl PartialEq for Shortcut {
    fn eq(&self, other: &Self) -> bool {
        self.chords == other.chords
    }
}

impl Eq for Shortcut {}

impl Shortcut {
    /// Builds a shortcut on one chord, which is what most of them are.
    pub fn new(key: &str, modifiers: &[&str]) -> Shortcut {
        Shortcut::of(vec![Chord::new(key, modifiers)])
    }

    /// Builds one on several, dropping any that name no key.
    pub fn of(chords: Vec<Chord>) -> Shortcut {
        let mut shortcut = Shortcut { chords };
        shortcut.chords.retain(|chord| !chord.is_blank());
        shortcut
    }

    /// A command with no key.
    pub fn unbound() -> Shortcut {
        Shortcut { chords: Vec::new() }
    }

    /// Every chord that presses it.
    pub fn chords(&self) -> &[Chord] {
        &self.chords
    }

    /// The one a menu prints beside the command's name.
    pub fn first(&self) -> Option<&Chord> {
        self.chords.first()
    }

    /// Whether nothing presses it.
    pub fn is_empty(&self) -> bool {
        self.chords.is_empty()
    }

    pub fn len(&self) -> usize {
        self.chords.len()
    }

    /// Whether this chord is already one of them.
    pub fn holds(&self, chord: &Chord) -> bool {
        self.chords.contains(chord)
    }

    /// Adds a chord, unless it is blank or already there.
    ///
    /// Returns whether the list grew, so the window can say why nothing
    /// happened rather than leaving the person to press the key again harder.
    pub fn add(&mut self, chord: Chord) -> bool {
        if chord.is_blank() || self.holds(&chord) {
            return false;
        }

        self.chords.push(chord);
        true
    }

    /// Takes one away. An index the list does not have is left alone.
    pub fn remove(&mut self, index: usize) {
        if index < self.chords.len() {
            self.chords.remove(index);
        }
    }

    /// Whether any one press would do both commands.
    ///
    /// The question a clash is asking, and it is *overlap* rather than
    /// equality now that a command can carry several chords: two commands
    /// sharing their second key fire together on every press of it while
    /// their first keys say nothing about each other.
    pub fn overlaps(&self, other: &Shortcut) -> bool {
        self.chords.iter().any(|chord| other.holds(chord))
    }
}

/// What the file holds. The first chord where it always was, the rest under
/// `also`, and `also` left out altogether when there are none.
#[derive(Deserialize, Serialize)]
pub struct ShortcutData {
    pub key: String,
    pub modifiers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also: Vec<ChordData>,
}

#[derive(Deserialize, Serialize)]
pub struct ChordData {
    pub key: String,
    pub modifiers: Vec<String>,
}

impl From<ChordData> for Chord {
    fn from(data: ChordData) -> Self {
        Chord {
            kbd_shortcut: build_keyboard_shortcut(&data.modifiers, &data.key),
            key: data.key,
            modifiers: data.modifiers,
        }
    }
}

impl From<Chord> for ChordData {
    fn from(chord: Chord) -> Self {
        ChordData {
            key: chord.key,
            modifiers: chord.modifiers,
        }
    }
}

impl From<ShortcutData> for Shortcut {
    fn from(data: ShortcutData) -> Self {
        let first = ChordData {
            key: data.key,
            modifiers: data.modifiers,
        };

        Shortcut::of(
            std::iter::once(first)
                .chain(data.also)
                .map(Chord::from)
                .collect(),
        )
    }
}

impl From<Shortcut> for ShortcutData {
    fn from(shortcut: Shortcut) -> Self {
        let mut chords = shortcut.chords.into_iter();
        let first = chords.next();

        ShortcutData {
            key: first
                .as_ref()
                .map(|chord| chord.key.clone())
                .unwrap_or_default(),
            modifiers: first.map(|chord| chord.modifiers).unwrap_or_default(),
            also: chords.map(ChordData::from).collect(),
        }
    }
}

pub fn capitalize_first_char(str: &str) -> String {
    let mut chars = str.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Key;

    #[test]
    fn builds_a_shortcut_from_its_parts() {
        let shortcut = Shortcut::new("f", &[MOD_CTRL, MOD_SHIFT]);
        let chord = shortcut.first().expect("one chord");

        assert_eq!(chord.kbd_shortcut.logical_key, Key::F);
        assert!(chord.kbd_shortcut.modifiers.ctrl);
        assert!(chord.kbd_shortcut.modifiers.shift);
        assert!(!chord.kbd_shortcut.modifiers.alt);
    }

    #[test]
    fn deserialised_shortcuts_are_ready_to_use() {
        let shortcut: Shortcut =
            serde_json::from_str(r#"{"key":"q","modifiers":["alt"]}"#).expect("parses");
        let chord = shortcut.first().expect("one chord");

        assert_eq!(chord.kbd_shortcut.logical_key, Key::Q);
        assert!(chord.kbd_shortcut.modifiers.alt);
    }

    /// The whole point of the change: a file written by an older build reads
    /// exactly as it did, and one with alternates reads them all.
    #[test]
    fn a_file_without_alternates_still_reads() {
        let shortcut: Shortcut =
            serde_json::from_str(r#"{"key":"d","modifiers":[]}"#).expect("parses");

        assert_eq!(shortcut.len(), 1);
        assert_eq!(shortcut.first().expect("one").key, "d");
    }

    #[test]
    fn alternates_are_read_in_the_order_they_were_written() {
        let shortcut: Shortcut = serde_json::from_str(
            r#"{"key":"d","modifiers":[],"also":[{"key":"ArrowRight","modifiers":[]},
                {"key":"space","modifiers":["ctrl"]}]}"#,
        )
        .expect("parses");

        let keys: Vec<&str> = shortcut
            .chords()
            .iter()
            .map(|chord| chord.key.as_str())
            .collect();

        assert_eq!(keys, ["d", "ArrowRight", "space"]);
        assert!(shortcut.chords()[2].kbd_shortcut.modifiers.ctrl);
    }

    /// A configuration nobody has added a key to writes what it always wrote,
    /// so upgrading and saving does not churn every one of the ninety rows.
    #[test]
    fn one_chord_writes_the_shape_the_older_build_wrote() {
        let written = serde_json::to_string(&Shortcut::new("d", &[])).expect("writes");
        assert_eq!(written, r#"{"key":"d","modifiers":[]}"#);
    }

    #[test]
    fn several_chords_write_the_rest_under_also() {
        let mut shortcut = Shortcut::new("d", &[]);
        shortcut.add(Chord::new("ArrowRight", &[]));

        let written = serde_json::to_string(&shortcut).expect("writes");
        assert_eq!(
            written,
            r#"{"key":"d","modifiers":[],"also":[{"key":"ArrowRight","modifiers":[]}]}"#
        );
    }

    /// A round trip through the file keeps every chord, which is what the
    /// window is for.
    #[test]
    fn a_shortcut_survives_being_written_and_read_back() {
        let mut shortcut = Shortcut::new("d", &[MOD_CTRL]);
        shortcut.add(Chord::new("ArrowRight", &[]));
        shortcut.add(Chord::new("space", &[MOD_ALT, MOD_SHIFT]));

        let written = serde_json::to_string(&shortcut).expect("writes");
        let read: Shortcut = serde_json::from_str(&written).expect("parses");

        assert_eq!(read, shortcut);
    }

    /// "No key" is the empty list however the file said it.
    #[test]
    fn a_blank_key_is_a_shortcut_with_no_chords() {
        let shortcut: Shortcut =
            serde_json::from_str(r#"{"key":"","modifiers":[]}"#).expect("parses");

        assert!(shortcut.is_empty());
        assert!(shortcut.first().is_none());
    }

    /// And an unbound command writes the blank an older build reads as one.
    #[test]
    fn an_unbound_shortcut_writes_a_blank_key() {
        let written = serde_json::to_string(&Shortcut::unbound()).expect("writes");
        assert_eq!(written, r#"{"key":"","modifiers":[]}"#);
    }

    /// A blank alternate is dropped rather than kept as a chord nothing can
    /// press, which would otherwise reach the window as an empty row.
    #[test]
    fn a_blank_alternate_is_dropped() {
        let shortcut: Shortcut = serde_json::from_str(
            r#"{"key":"d","modifiers":[],"also":[{"key":"  ","modifiers":[]}]}"#,
        )
        .expect("parses");

        assert_eq!(shortcut.len(), 1);
    }

    /// The first chord stays the first: a menu prints it, and adding a second
    /// key must not change what a menu says.
    #[test]
    fn adding_a_chord_leaves_the_first_where_it_was() {
        let mut shortcut = Shortcut::new("d", &[]);
        shortcut.add(Chord::new("ArrowRight", &[]));

        assert_eq!(shortcut.first().expect("one").key, "d");
    }

    #[test]
    fn a_chord_already_bound_is_not_added_twice() {
        let mut shortcut = Shortcut::new("d", &[]);

        assert!(!shortcut.add(Chord::new("D", &[])));
        assert!(!shortcut.add(Chord::new("", &[])));
        assert!(shortcut.add(Chord::new("d", &[MOD_CTRL])));
        assert_eq!(shortcut.len(), 2);
    }

    #[test]
    fn removing_the_last_chord_leaves_the_command_unbound() {
        let mut shortcut = Shortcut::new("d", &[]);
        shortcut.remove(0);

        assert!(shortcut.is_empty());
        assert_eq!(shortcut, Shortcut::unbound());
    }

    #[test]
    fn removing_a_chord_that_is_not_there_changes_nothing() {
        let mut shortcut = Shortcut::new("d", &[]);
        shortcut.remove(7);

        assert_eq!(shortcut.len(), 1);
    }

    /// Modifiers are a set and the key's case is not part of it.
    #[test]
    fn a_chord_is_the_same_whatever_order_and_case_it_was_written_in() {
        assert_eq!(
            Chord::new("delete", &[MOD_CTRL, MOD_SHIFT]),
            Chord::new("Delete", &[MOD_SHIFT, MOD_CTRL])
        );
        assert_ne!(Chord::new("d", &[MOD_CTRL]), Chord::new("d", &[]));
    }

    /// And two names egui reads as one key are one chord, which is what keeps
    /// a hand-edited "Esc" from slipping past the clash check.
    #[test]
    fn two_names_for_one_key_are_one_chord() {
        assert_eq!(Chord::new("Esc", &[]), Chord::new("Escape", &[]));
    }

    /// Two typos build the same unreachable sentinel and are still two
    /// mistakes: the checker has a row to say about each.
    #[test]
    fn two_key_names_nothing_can_read_are_not_the_same_chord() {
        assert_ne!(Chord::new("not-a-key", &[]), Chord::new("nor-this", &[]));
        assert_eq!(Chord::new("not-a-key", &[]), Chord::new("NOT-A-KEY", &[]));
        assert!(!Chord::new("not-a-key", &[]).is_readable());
    }

    /// A clash is an overlap now, not an equality: two commands sharing only
    /// their second key still fire together on every press of it.
    #[test]
    fn two_shortcuts_overlap_when_they_share_any_chord() {
        let mut mine = Shortcut::new("d", &[]);
        mine.add(Chord::new("ArrowRight", &[]));

        let mut theirs = Shortcut::new("n", &[]);
        assert!(!mine.overlaps(&theirs));

        theirs.add(Chord::new("ArrowRight", &[]));
        assert!(mine.overlaps(&theirs));
        assert!(theirs.overlaps(&mine));
    }

    /// Two commands with no key at all do not collide with each other, which
    /// is what the blank-key guard used to be for.
    #[test]
    fn nothing_overlaps_with_nothing() {
        assert!(!Shortcut::unbound().overlaps(&Shortcut::unbound()));
    }
}
