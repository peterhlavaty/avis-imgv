//! What is wrong with a configuration, said once at load.
//!
//! Every one of these failures reaches only a log file today, whose own path
//! the program never states. They are collected here instead, so the window can
//! draw a band across its top with a row per complaint and a button that goes
//! to the control.

use crate::config::Config;

/// One thing wrong with the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complaint {
    /// The path of the field it is about, so the **[Fix]** button knows where
    /// to go. Empty where nothing in the registry owns it.
    pub path: &'static str,
    /// What is wrong, naming the value that is wrong.
    pub says: String,
    /// What was used instead.
    pub instead: String,
}

impl Config {
    /// Everything wrong with this configuration, in the order the pages draw
    /// the fields.
    ///
    /// Run once at load and drawn by the settings window. Nothing here changes
    /// the configuration: an out-of-range value is shown, marked, and left
    /// exactly as written, because `save` writes the whole document and
    /// clamping on load would destroy somebody's deliberate 8,192 MB budget on
    /// the first save after any unrelated change.
    pub fn check(&self) -> Vec<Complaint> {
        let mut found = Vec::new();

        icc(self, &mut found);
        keyword_catalogue(self, &mut found);
        rejects(self, &mut found);
        destinations(self, &mut found);
        actions(self, &mut found);
        keys(self, &mut found);
        ranges(self, &mut found);

        found
    }
}

/// The screen profile is matched by substring against three shipped names, so
/// a typo silently leaves every photograph unconverted.
fn icc(config: &Config, found: &mut Vec<Complaint>) {
    let wanted = config.general.output_icc_profile.trim();
    if wanted.is_empty() || crate::metadata::icc::is_known(wanted) {
        return;
    }

    found.push(Complaint {
        path: "general.output_icc_profile",
        says: format!("No screen profile matches \"{wanted}\"."),
        instead: "Photographs are drawn without conversion, which on a wide gamut \
                  screen makes colour look oversaturated."
            .to_string(),
    });
}

fn keyword_catalogue(config: &Config, found: &mut Vec<Complaint>) {
    let Some(path) = &config.tags.catalog_file else {
        return;
    };

    if crate::annotations::catalog::resolve(path).is_some_and(|path| path.is_file()) {
        return;
    }

    found.push(Complaint {
        path: "tags.catalog_file",
        says: format!("There is no keyword file at \"{path}\"."),
        instead: "Only the keywords written in the configuration are offered. A \
                  relative path is taken against the configuration directory, not the \
                  working one."
            .to_string(),
    });
}

/// A blank rejects folder makes its key a no-op, silently.
fn rejects(config: &Config, found: &mut Vec<Complaint>) {
    if !config.cull.rejected_folder.trim().is_empty() {
        return;
    }

    found.push(Complaint {
        path: "cull.rejected_folder",
        says: "The rejects folder has no name.".to_string(),
        instead: "The key that sends a photograph there does nothing.".to_string(),
    });
}

/// An empty destination path is dropped without a word, and the tenth is
/// truncated because there are only nine digits.
fn destinations(config: &Config, found: &mut Vec<Complaint>) {
    for (at, destination) in config.cull.destinations.iter().enumerate() {
        if destination.path.trim().is_empty() {
            found.push(Complaint {
                path: "cull.destinations",
                says: format!(
                    "Destination {} (\"{}\") has no folder.",
                    at + 1,
                    destination.label
                ),
                instead: "It is left out of the panel.".to_string(),
            });
        }
    }

    if config.cull.destinations.len() > 9 {
        found.push(Complaint {
            path: "cull.destinations",
            says: format!(
                "There are {} destinations and nine digits.",
                config.cull.destinations.len()
            ),
            instead: "The first nine keep their digits; the rest are reached with the \
                      arrow keys."
                .to_string(),
        });
    }
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
            let shadows = crate::ui::menus::Verb::ON_A_PHOTOGRAPH
                .iter()
                .chain(crate::ui::menus::Verb::ON_A_CELL)
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

/// An unknown key name becomes the unreachable sentinel, so a typo makes a
/// command permanently unreachable and the only record is a log line.
fn keys(config: &Config, found: &mut Vec<Complaint>) {
    for row in super::rows() {
        let Some(shortcut) = row.access.shortcut(config) else {
            continue;
        };

        if crate::config::shortcut::names_a_key(&shortcut.key) {
            continue;
        }

        found.push(Complaint {
            path: row.path,
            says: format!(
                "\"{}\" is not a key name, so {} cannot be pressed.",
                shortcut.key, row.label
            ),
            instead: "The command is unreachable. examples/keys.txt lists every name \
                      the viewer accepts."
                .to_string(),
        });
    }
}

/// A number outside what its control can produce. Reported and left alone:
/// hand-editing always wins, including hand-editing to a value the window
/// cannot make.
fn ranges(config: &Config, found: &mut Vec<Complaint>) {
    for row in super::rows() {
        let says = match &row.access {
            super::Access::Int { get, min, max, .. } => {
                let value = get(config);
                (value < *min || value > *max).then(|| format!("{value} is outside {min} to {max}"))
            }
            super::Access::Float { get, min, max, .. } => {
                let value = get(config);
                (!value.is_finite() || value < *min || value > *max)
                    .then(|| format!("{value} is outside {min} to {max}"))
            }
            _ => None,
        };

        if let Some(says) = says {
            found.push(Complaint {
                path: row.path,
                says: format!("{}: {says}.", row.label),
                instead: "It is left exactly as written; the control shows it marked \
                          out of range."
                    .to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_configuration_has_nothing_wrong_with_it() {
        let complaints = Config::default().check();

        assert!(
            complaints.is_empty(),
            "the defaults complain about themselves: {complaints:?}"
        );
    }

    /// The worked example: a misspelled profile names the field, the complaint
    /// and what was used instead.
    #[test]
    fn a_misspelled_profile_names_itself() {
        let mut config = Config::default();
        config.general.output_icc_profile = "sRGV".to_string();

        let complaints = config.check();
        let found = complaints
            .iter()
            .find(|c| c.path == "general.output_icc_profile")
            .expect("it complained");

        assert!(found.says.contains("sRGV"));
        assert!(!found.instead.is_empty());
    }

    #[test]
    fn a_blank_rejects_folder_says_its_key_does_nothing() {
        let mut config = Config::default();
        config.cull.rejected_folder = "  ".to_string();

        assert!(config
            .check()
            .iter()
            .any(|c| c.path == "cull.rejected_folder"));
    }

    /// A typo in a key name makes a command permanently unreachable, and the
    /// only record used to be a log line.
    #[test]
    fn a_key_name_that_is_not_a_key_is_reported() {
        let mut config = Config::default();
        config.general.sc_exit = crate::config::Shortcut::new("Excape", &[]);

        let complaints = config.check();
        assert!(complaints.iter().any(|c| c.path == "general.sc_exit"));
    }

    /// And a value outside its range is reported without being changed.
    #[test]
    fn a_number_out_of_range_is_reported_and_left_alone() {
        let mut config = Config::default();
        config.image_view.frame_size_relative_to_image = 5.0;

        let complaints = config.check();
        assert!(complaints
            .iter()
            .any(|c| c.path == "image_view.frame_size_relative_to_image"));
        assert_eq!(config.image_view.frame_size_relative_to_image, 5.0);
    }
}
