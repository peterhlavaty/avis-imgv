//! Every key the viewer listens for, as a filtered view over the registry.
//!
//! This module used to *be* the table: a flat list of the sixty shortcut
//! fields, with a sentence each and a pair of accessors reaching the field
//! behind it. That idea was right and too narrow — the other eighty fields of
//! the configuration wanted exactly the same treatment — so the table moved to
//! [`registry`] and this became the view of it the keyboard editor and the
//! cheat sheet already had.
//!
//! What changed for them: the fixed keys are now in the list, drawn read-only,
//! so the clash checker can see them; the shortcut on a user action is
//! reachable, which it never was; and clashes are decided by where a binding is
//! *read* rather than by which heading it happened to be filed under.
//!
//! [`registry`]: super::registry

use super::registry::{self, Access, Row, Scope};
use super::{Config, Shortcut};

/// One thing a key can be bound to.
///
/// A borrowed view of a registry row rather than a copy of one, so there is
/// nowhere for the two to disagree.
#[derive(Clone, Copy)]
pub struct Binding {
    row: &'static Row,
}

impl Binding {
    /// Which part of the viewer it belongs to, for grouping the list.
    pub fn section(&self) -> &'static str {
        section_of(self.row.scope)
    }

    /// What it does, as a heading.
    pub fn name(&self) -> &'static str {
        self.row.label
    }

    /// What it does, in a sentence.
    pub fn description(&self) -> &'static str {
        self.row.sentence
    }

    /// Where the value is read, which is where it can clash.
    pub fn scope(&self) -> Scope {
        self.row.scope
    }

    /// Its path in the configuration file, which is its identity.
    pub fn path(&self) -> &'static str {
        self.row.path
    }

    /// Whether the interface may change it. A fixed key is drawn and not
    /// edited: it is in the list so the clash checker can see it.
    pub fn is_editable(&self) -> bool {
        self.row.access.is_writable()
    }

    /// The registry row behind it, for whatever needs more than this view.
    pub fn row(&self) -> &'static Row {
        self.row
    }

    /// The shortcut currently bound, if the configuration still has it.
    pub fn get<'a>(&self, config: &'a Config) -> Option<&'a Shortcut> {
        self.row.access.shortcut(config)
    }

    /// Whether this row exists for this configuration.
    ///
    /// A user action row is one of nine written into the table; a file with two
    /// actions in it has two, and the other seven are not rows at all.
    pub fn exists(&self, config: &Config) -> bool {
        self.fixed().is_some() || self.get(config).is_some()
    }

    /// A fixed key's name, for the rows nobody can change.
    pub fn fixed(&self) -> Option<&'static str> {
        match self.row.access {
            Access::Fixed(name) => Some(name),
            _ => None,
        }
    }

    /// Replaces what this binding is bound to.
    pub fn set(&self, config: &mut Config, shortcut: Shortcut) {
        self.row.access.set_shortcut(config, shortcut);
    }

    /// Puts it back to what a fresh configuration binds it to.
    pub fn reset(&self, config: &mut Config) {
        self.row.access.reset(config);
    }

    /// Whether it differs from that.
    pub fn changed(&self, config: &Config) -> bool {
        self.row.changed(config)
    }
}

/// Sections, in the order the editor lists them.
///
/// Now derived from the scope rather than written on each row, so the heading a
/// binding appears under and the rule deciding whether it can clash are the
/// same fact rather than two.
pub const SECTIONS: &[&str] = &[
    "General",
    "Image view",
    "Gallery",
    "Ratings and tags",
    "Fixed keys",
];

/// The heading a scope is listed under.
fn section_of(scope: Scope) -> &'static str {
    match scope {
        Scope::Everywhere => "General",
        Scope::ImageView => "Image view",
        Scope::Gallery => "Gallery",
        Scope::Overlay => "Fixed keys",
        Scope::None => "General",
    }
}

/// Every key the viewer listens for.
pub fn all() -> Vec<Binding> {
    registry::rows()
        .iter()
        .filter(|row| row.access.is_a_key())
        .map(|row| Binding { row })
        .collect()
}

/// Whether `path` names a key rather than some other kind of setting.
///
/// The cheat sheet lists gestures beside keys now, and a row has to know which
/// window it opens: a key arms the key editor, and a gesture opens the page
/// that owns it.
pub fn is_a_key(path: &str) -> bool {
    registry::rows()
        .iter()
        .any(|row| row.path == path && row.access.is_a_key())
}

/// The heading the marks are listed under, which is not a scope.
///
/// The ratings, the colour labels and the flags are read everywhere, so their
/// scope says "General" — but somebody looking for the key that puts three
/// stars on a photograph looks under the marks. The one place where the list a
/// person reads and the rule the checker applies are deliberately different,
/// and the reason is written here rather than left to be worked out.
pub fn heading(binding: &Binding) -> &'static str {
    if binding.path().starts_with("tags.") {
        return "Ratings and tags";
    }

    if binding.fixed().is_some() {
        return "Fixed keys";
    }

    binding.section()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_binding_has_a_section_the_editor_lists() {
        for binding in all() {
            assert!(
                SECTIONS.contains(&heading(&binding)),
                "{} is filed under {}, which the editor does not draw",
                binding.path(),
                heading(&binding)
            );
        }
    }

    #[test]
    fn a_binding_reads_and_writes_its_field() {
        let mut config = Config::default();
        let bindings = all();

        let three = bindings
            .iter()
            .find(|binding| binding.name() == "Three stars")
            .expect("the list has it");

        three.set(&mut config, Shortcut::new("F5", &[]));
        assert_eq!(config.tags.sc_rating[3].key, "F5");
    }

    /// The count is what stops a shortcut being added to the configuration and
    /// quietly left out of the editor. Sixty-five written fields, plus the six
    /// ratings and the five colour labels; the keys the program reads for
    /// itself are in the list too, and are not editable.
    #[test]
    fn every_shortcut_in_the_configuration_can_be_changed_from_the_list() {
        let fresh = Config::default();
        let editable = all()
            .iter()
            .filter(|b| b.is_editable() && b.exists(&fresh))
            .count();

        assert_eq!(
            editable, 77,
            "a shortcut was added to the configuration without a registry row"
        );
    }

    /// And the keys the program reads for itself are drawn without being
    /// editable, so the clash checker can see them.
    #[test]
    fn the_fixed_keys_are_in_the_list_and_are_not_editable() {
        let fixed: Vec<_> = all().into_iter().filter(|b| b.fixed().is_some()).collect();

        assert!(!fixed.is_empty());
        for binding in fixed {
            assert!(!binding.is_editable(), "{} is editable", binding.path());
        }
    }

    /// The one shortcut in the file the editor could not reach.
    #[test]
    fn a_user_action_gets_a_row_when_the_file_has_one() {
        let mut config = Config::default();
        config
            .image_view
            .user_actions
            .push(crate::config::UserAction {
                shortcut: Shortcut::new("e", &[]),
                exec: "gimp {}".to_string(),
                callback: None,
            });

        let bindings = all();
        let action = bindings
            .iter()
            .find(|b| b.path() == "image_view.user_actions[0].shortcut")
            .expect("the table has nine of them");

        assert!(action.exists(&config));
        assert_eq!(action.get(&config).map(|s| s.key.as_str()), Some("e"));

        // And the other eight are not rows for this file.
        let missing = bindings
            .iter()
            .find(|b| b.path() == "image_view.user_actions[1].shortcut")
            .expect("the table has nine of them");
        assert!(!missing.exists(&config));
    }

    /// A search for the key that shows the keys finds one.
    #[test]
    fn the_cheat_sheet_key_is_in_the_list() {
        let found = all()
            .into_iter()
            .find(|binding| binding.path() == "fixed.cheat_sheet")
            .expect("the fixed keys are in the list");

        assert_eq!(found.fixed(), Some("?"));
    }
}
