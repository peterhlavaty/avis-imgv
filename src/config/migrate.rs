//! Bringing a configuration file written by an older build up to date.
//!
//! Defaults change. Most of the time that costs nothing — somebody who never
//! touched a setting simply gets the new one, and somebody who did keeps what
//! they wrote. The exception is a default that *moves*: when a key stops
//! meaning one thing and starts meaning another, a file still holding the old
//! binding leaves two commands fighting over it, and the one that loses does
//! nothing at all with no explanation.
//!
//! So the file carries a version, and each change that needs a hand gets a
//! step here. A step is only ever applied to a file old enough to need it, so
//! somebody who has deliberately bound a key back to what it used to be keeps
//! it.
//!
//! What a step must not do is overwrite a choice. Every one of them checks
//! that what it finds is the *old default* before touching it: a setting the
//! user has actually changed is theirs, and a migration that flattens it is
//! worse than the clash it was avoiding.

use super::{defaults, Config, Shortcut};

/// What this build writes.
///
/// Bumped whenever a step is added below, and never otherwise.
pub const CURRENT: u32 = 1;

/// One thing that has to be put right in an older file.
struct Step {
    /// The version the file has to be *below* for this to apply.
    until: u32,
    /// What to say to the user, if it changes anything.
    said: &'static str,
    apply: fn(&mut Config) -> bool,
}

const STEPS: &[Step] = &[
    Step {
        until: 1,
        said: "Scrolling the contact sheet moved from Space to PageDown, because \
               Space now picks a photograph out",
        apply: scroll_off_the_space_bar,
    },
    Step {
        until: 1,
        said: "Showing more or fewer images side by side moved to Ctrl with \
               Plus and Minus, which is where it stopped fighting with zoom",
        apply: side_by_side_onto_ctrl,
    },
];

/// Brings `config` up to [`CURRENT`], returning what was changed.
///
/// Nothing is reported for a file that was already current, which is the
/// ordinary case and should be silent.
pub fn apply(config: &mut Config) -> Vec<&'static str> {
    let from = config.version;
    let mut changed = Vec::new();

    // A file that has never been written carries no version and needs every
    // step; one from the future is left alone entirely, because this build
    // cannot know what a later one meant.
    if from >= CURRENT {
        config.version = config.version.max(CURRENT);
        return changed;
    }

    for step in STEPS {
        if from < step.until && (step.apply)(config) {
            changed.push(step.said);
        }
    }

    config.version = CURRENT;
    changed
}

/// Space was "scroll down half a row" and is now "pick this one out".
///
/// Both would match the same key, and the selection is claimed first, so an
/// untouched older file would find its scroll key had quietly stopped working.
fn scroll_off_the_space_bar(config: &mut Config) -> bool {
    let was = Shortcut::new("Space", &[]);
    if config.grid_view.sc_scroll != was {
        return false;
    }

    config.grid_view.sc_scroll = defaults::default_sc_scroll();
    true
}

/// Plus and Minus were both "more images side by side" and "zoom in".
///
/// The default moved to Ctrl with them, but a file written before that keeps
/// the old binding for ever — `serde` fills in the keys that are missing, not
/// the ones that have since moved. Both commands then matched the same key,
/// zoom won, and the side-by-side view was simply unreachable with no hint as
/// to why. This is the case the startup clash warning was written for, found
/// on a real configuration; the warning says so, and now the file is put right
/// as well.
fn side_by_side_onto_ctrl(config: &mut Config) -> bool {
    let mut moved = false;

    for (setting, was, now) in [
        (
            &mut config.image_view.sc_more_images_shown,
            "Plus",
            defaults::default_sc_more_images_shown(),
        ),
        (
            &mut config.image_view.sc_less_images_shown,
            "Minus",
            defaults::default_sc_less_images_shown(),
        ),
    ] {
        if *setting == Shortcut::new(was, &[]) {
            *setting = now;
            moved = true;
        }
    }

    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file from before versions existed gets every step.
    fn ancient() -> Config {
        Config {
            version: 0,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("Space", &[]),
                ..Default::default()
            },
            ..Config::default()
        }
    }

    #[test]
    fn an_old_file_has_its_scroll_key_moved() {
        let mut config = ancient();
        let said = apply(&mut config);

        assert_eq!(said.len(), 1);
        assert_eq!(config.grid_view.sc_scroll, defaults::default_sc_scroll());
        assert_eq!(config.version, CURRENT);
    }

    /// The clash that made the side-by-side view unreachable: both commands
    /// on a bare Plus, with zoom winning.
    #[test]
    fn an_old_file_has_its_side_by_side_keys_moved() {
        let mut config = Config {
            version: 0,
            image_view: super::super::ImageViewConfig {
                sc_more_images_shown: Shortcut::new("Plus", &[]),
                sc_less_images_shown: Shortcut::new("Minus", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        let said = apply(&mut config);

        assert_eq!(said.len(), 1);
        assert_eq!(
            config.image_view.sc_more_images_shown,
            defaults::default_sc_more_images_shown()
        );
        assert_eq!(
            config.image_view.sc_less_images_shown,
            defaults::default_sc_less_images_shown()
        );

        // And the clash it was for is gone.
        assert!(crate::ui::keys::clashes(&config).is_empty());
    }

    /// The rule that makes migrations safe: a key the user chose is theirs.
    #[test]
    fn a_key_the_user_chose_is_left_alone() {
        let mut config = Config {
            version: 0,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("j", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        let said = apply(&mut config);

        assert!(said.is_empty());
        assert_eq!(config.grid_view.sc_scroll, Shortcut::new("j", &[]));
        assert_eq!(config.version, CURRENT);
    }

    #[test]
    fn a_current_file_is_not_touched_and_says_nothing() {
        let mut config = Config {
            version: CURRENT,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("Space", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        assert!(apply(&mut config).is_empty());
        assert_eq!(config.grid_view.sc_scroll, Shortcut::new("Space", &[]));
    }

    /// A file from a build newer than this one is not "migrated" backwards.
    #[test]
    fn a_file_from_the_future_is_left_as_it_is() {
        let mut config = Config {
            version: CURRENT + 5,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("Space", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        assert!(apply(&mut config).is_empty());
        assert_eq!(config.version, CURRENT + 5);
        assert_eq!(config.grid_view.sc_scroll, Shortcut::new("Space", &[]));
    }

    /// Migrating twice does nothing the second time.
    #[test]
    fn migrating_is_idempotent() {
        let mut config = ancient();

        assert_eq!(apply(&mut config).len(), 1);
        assert!(apply(&mut config).is_empty());
    }

    /// A default freshly built by this version is already current, so nobody
    /// starting today is told anything.
    #[test]
    fn a_new_configuration_needs_no_migration() {
        let mut config = Config::default();

        assert_eq!(config.version, CURRENT);
        assert!(apply(&mut config).is_empty());
    }

    /// Every step has to be reachable, and to say something.
    #[test]
    fn every_step_is_within_the_current_version() {
        for step in STEPS {
            assert!(step.until <= CURRENT, "{} is unreachable", step.said);
            assert!(!step.said.is_empty());
        }
    }
}
