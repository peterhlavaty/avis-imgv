//! The modifier held with a pan or zoom key to ask for a smaller step.

use eframe::egui::Modifiers;
use serde::{Deserialize, Serialize};

use super::{MOD_ALT, MOD_CTRL, MOD_SHIFT};

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
    fn a_modifier_is_named_the_way_the_file_names_it() {
        assert_eq!(FineModifier::Alt.value(), MOD_ALT);
        assert_eq!(FineModifier::of(MOD_CTRL), Some(FineModifier::Ctrl));
    }

    /// A name this build has never heard of leaves the caller's default in
    /// place rather than turning the fine pan off altogether.
    #[test]
    fn a_name_it_cannot_read_is_nothing_rather_than_a_default() {
        assert_eq!(FineModifier::of("hyper"), None);
    }

    #[test]
    fn it_reads_whether_it_is_held() {
        assert!(FineModifier::Alt.held(&Modifiers::ALT));
        assert!(!FineModifier::Alt.held(&Modifiers::SHIFT));
        assert!(FineModifier::Ctrl.held(&Modifiers::COMMAND));
    }
}
