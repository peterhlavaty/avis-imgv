//! How a binding reads where it is printed.

use crate::config::{Chord, Shortcut};

/// Every key that does one thing, as one phrase: `Ctrl + Plus or D`.
///
/// All of them rather than the first, everywhere it is printed. A menu naming
/// a key that does nothing is worse than one naming none, and a menu naming
/// one of the two a person bound is the same fault in a smaller size: they are
/// as likely to have forgotten the one it does not name. Nothing changes for
/// the ninety bindings carrying a single key, which is all of them until
/// somebody adds one.
pub fn describe(shortcut: &Shortcut) -> String {
    let mut said = String::new();
    describe_into(shortcut, &mut said);
    said
}

/// The same, written into a string that already exists.
///
/// The menus name a key beside their verbs now, which means every binding is
/// written out once a frame ([`crate::ui::keys::publish`]) rather than the
/// handful the editor has on screen. A phrase built afresh each time is four
/// allocations for a sentence that changes when somebody rebinds a key and
/// never otherwise, so the string is kept and written over.
pub fn describe_into(shortcut: &Shortcut, into: &mut String) {
    if shortcut.is_empty() {
        into.push_str("no key");
        return;
    }

    for (at, chord) in shortcut.chords().iter().enumerate() {
        if at > 0 {
            into.push_str(" or ");
        }

        chord_into(chord, into);
    }
}

/// One press, as it reads on a button: `Ctrl + Plus`.
pub fn chord(chord: &Chord) -> String {
    let mut said = String::new();
    chord_into(chord, &mut said);
    said
}

/// The same, written into a string that already exists.
pub fn chord_into(chord: &Chord, into: &mut String) {
    for modifier in &chord.modifiers {
        capitalised_into(modifier, into);
        into.push_str(" + ");
    }

    into.push_str(&chord.key);
}

fn capitalised_into(text: &str, into: &mut String) {
    let mut chars = text.chars();

    if let Some(first) = chars.next() {
        into.extend(first.to_uppercase());
        into.push_str(chars.as_str());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::shortcut::{MOD_CTRL, MOD_SHIFT};

    #[test]
    fn a_shortcut_reads_as_its_parts() {
        assert_eq!(describe(&Shortcut::new("Plus", &[MOD_CTRL])), "Ctrl + Plus");
        assert_eq!(describe(&Shortcut::new("f", &[])), "f");
        assert_eq!(
            describe(&Shortcut::new("q", &[MOD_CTRL, MOD_SHIFT])),
            "Ctrl + Shift + q"
        );
    }

    /// A row with no key says so rather than drawing an empty button.
    #[test]
    fn a_binding_with_no_key_says_so() {
        assert_eq!(describe(&Shortcut::unbound()), "no key");
    }

    /// Both keys, in the order they were added, so the one a person put on
    /// second is not the one the program keeps quiet about.
    #[test]
    fn a_shortcut_on_two_keys_names_both() {
        let mut shortcut = Shortcut::new("d", &[]);
        shortcut.add(Chord::new("ArrowRight", &[MOD_CTRL]));

        assert_eq!(describe(&shortcut), "d or Ctrl + ArrowRight");
    }

    /// The string is written over rather than added to, so the phrase a menu
    /// names is this frame's and not this frame's after last frame's.
    #[test]
    fn a_phrase_written_into_a_string_says_the_same_thing() {
        let shortcut = Shortcut::new("Plus", &[MOD_CTRL]);
        let mut said = String::from("whatever was there before");

        said.clear();
        describe_into(&shortcut, &mut said);

        assert_eq!(said, describe(&shortcut));
    }
}
