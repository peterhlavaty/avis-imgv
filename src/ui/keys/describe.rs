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
    if shortcut.is_empty() {
        return "no key".to_string();
    }

    shortcut
        .chords()
        .iter()
        .map(chord)
        .collect::<Vec<String>>()
        .join(" or ")
}

/// One press, as it reads on a button: `Ctrl + Plus`.
pub fn chord(chord: &Chord) -> String {
    let mut parts: Vec<String> = chord.modifiers.iter().map(|m| capitalised(m)).collect();

    parts.push(chord.key.clone());
    parts.join(" + ")
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();

    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
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
}
