//! The two checks on the configuration that need the window's own words.
//!
//! Seven of the nine live in `config::registry::check`, where they belong: a
//! budget out of range or a keyword file that cannot be read are questions
//! about the file. These two are not. One asks whether a menu row the user
//! wrote now sits below a built-in one saying nearly the same thing, and the
//! other whether the fine modifier collides with a binding — and both need to
//! know what a menu row *says* and what a chord *reads as*, which the drawing
//! layer owns.
//!
//! They were in `config`, which meant the configuration reached up into the
//! window to ask. Here instead, chained onto the rest by the settings window,
//! which is the one place that wants all nine.

use crate::config::registry::{rows, Complaint, Scope};
use crate::config::shortcut::Chord;
use crate::config::Config;

/// The two checks the configuration cannot make for itself.
pub fn about_the_window(config: &Config) -> Vec<Complaint> {
    let mut found = Vec::new();

    actions(config, &mut found);
    the_fine_pan(config, &mut found);

    found
}

/// An action with no command has a key that does nothing.
fn actions(config: &Config, found: &mut Vec<Complaint>) {
    for (at, action) in config.image_view.user_actions.iter().enumerate() {
        if action.exec.trim().is_empty() {
            found.push(Complaint {
                path: "image_view.user_actions",
                says: format!("Action {} has no command.", at + 1),
                instead: "Its key does nothing.".to_string(),
            });
        }
    }

    // A configured menu row whose words now sit two rows below a built-in one
    // saying nearly the same thing. It is reported and nothing is done about
    // it: the entry is the user's own, and the viewer does not rename or remove
    // what somebody wrote.
    for (where_it_is, entries) in [
        ("image_view.context_menu", &config.image_view.context_menu),
        ("grid_view.context_menu", &config.grid_view.context_menu),
    ] {
        for entry in entries.iter() {
            let words = entry.description.to_lowercase();
            let shadows = crate::ui::menus::Row::ON_A_PHOTOGRAPH
                .iter()
                .chain(crate::ui::menus::Row::ON_A_CELL)
                .chain(crate::ui::menus::Row::ON_A_PHOTOGRAPH_IN_THE_BIN)
                .chain(crate::ui::menus::Row::ON_A_CELL_IN_THE_BIN)
                .flat_map(|row| row.verbs())
                .any(|verb| verb.label(1).to_lowercase().contains(&words) && words.len() > 3);

            if !shadows {
                continue;
            }

            found.push(Complaint {
                path: where_it_is,
                says: format!(
                    "Your menu row \"{}\" now sits below a built-in one saying nearly \n                     the same thing.",
                    entry.description
                ),
                instead: "Nothing was changed: the row is yours. Rename it, take it off, \n                          or leave it."
                    .to_string(),
            });
        }
    }
}

/// A binding sitting on the chord a fine pan is asked for with.
///
/// The fine pan is a modifier and the four keys that already pan, so no row in
/// the registry holds it and the clash check cannot see it — but a binding on
/// the same chord is read on the same frame, and the platform repeats it for
/// as long as the key is held. `Ctrl + W` was that case on the day the
/// modifier arrived — the folder watcher, sharing a letter with pan up — and
/// is why it is now `Ctrl + Shift + W`.
fn the_fine_pan(config: &Config, found: &mut Vec<Complaint>) {
    let fine = config.image_view.fine_modifier;

    let ways = [
        ("up", &config.image_view.sc_pan_up),
        ("down", &config.image_view.sc_pan_down),
        ("left", &config.image_view.sc_pan_left),
        ("right", &config.image_view.sc_pan_right),
    ];

    for (way, pan) in ways {
        // Every key that pans that way, not only the first: a second one is
        // as much a held key as the first, and the modifier is held with
        // whichever the finger is on.
        for chord in pan.chords() {
            let fined = Chord::new(
                &crate::config::shortcut::capitalize_first_char(&chord.key),
                &[fine.value()],
            );

            for row in rows() {
                // Only where the photograph is: a key the contact sheet reads
                // is never read on a frame this one is.
                if !row.scope.overlaps(Scope::ImageView) {
                    continue;
                }

                let Some(bound) = row.access.shortcut(config) else {
                    continue;
                };

                if !bound.holds(&fined) {
                    continue;
                }

                found.push(Complaint {
                    path: "image_view.fine_modifier",
                    says: format!(
                        "{} is both \"{}\" and the fine pan {way}.",
                        crate::ui::keys::chord(&fined),
                        row.label
                    ),
                    instead: "Both happen on every press, and the platform repeats the key \
                              for as long as it is held. Another modifier here, or another \
                              key for the command, settles it."
                        .to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ContextMenuEntry;

    /// Neither of these two checks had a test while it lived in `config`,
    /// which is most of the argument for moving it somewhere it can have one:
    /// a check nothing exercises is a check that is right until somebody
    /// changes the words it compares.
    #[test]
    fn a_fresh_configuration_has_nothing_to_complain_about() {
        assert!(about_the_window(&Config::default()).is_empty());
    }

    /// A menu row the user wrote that now sits below a built-in one saying
    /// nearly the same thing. Reported and nothing done about it: the entry is
    /// the user's own, and the viewer does not rename or remove what somebody
    /// wrote.
    #[test]
    fn a_menu_row_shadowing_a_built_in_one_is_reported() {
        let shadowing = crate::ui::menus::Row::ON_A_PHOTOGRAPH
            .iter()
            .flat_map(|row| row.verbs())
            .map(|verb| verb.label(1))
            .find(|label| label.len() > 3)
            .expect("the photograph's menu has verbs");

        let mut config = Config::default();
        config.image_view.context_menu = vec![ContextMenuEntry {
            description: shadowing.to_string(),
            exec: "true".to_string(),
            callback: None,
        }];

        let found = about_the_window(&config);

        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].path, "image_view.context_menu");
    }

    /// A row the user wrote that says something of its own is left alone.
    #[test]
    fn a_menu_row_of_its_own_is_not_complained_about() {
        let mut config = Config::default();
        config.image_view.context_menu = vec![ContextMenuEntry {
            description: "Send to the retoucher".to_string(),
            exec: "true".to_string(),
            callback: None,
        }];

        assert!(about_the_window(&config).is_empty());
    }

    /// Very short words are not compared, or every row would shadow every
    /// other one.
    #[test]
    fn a_very_short_row_is_not_matched_against_everything() {
        let mut config = Config::default();
        config.image_view.context_menu = vec![ContextMenuEntry {
            description: "Go".to_string(),
            exec: "true".to_string(),
            callback: None,
        }];

        assert!(about_the_window(&config).is_empty());
    }
}
