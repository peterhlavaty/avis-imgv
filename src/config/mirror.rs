//! A value the program holds *and* the file holds, kept in step both ways.
//!
//! Some settings are also live state. Whether marking advances to the next
//! photograph is a key, a menu row and a tick in the settings window; whether
//! the strip of thumbnails is up is a key and a tick. The program keeps its own
//! copy so a key can change it without going through the file, and the file
//! keeps one so the next launch opens as the last one closed.
//!
//! Two copies need two walks, and the rule is that a value with only one of
//! them is broken:
//!
//! - **program to file** — a key press has to survive the next launch, and the
//!   next save from the settings window must not snap the view back to whatever
//!   the file still said;
//! - **file to program** — without it, ticking the box sets the file and the
//!   next frame overwrites it with the live value it was just asked to change.
//!   The tick does nothing at all, and it does nothing *silently*.
//!
//! That second half was missing twice. "Show the strip" had it missing until
//! somebody noticed the checkbox did nothing, and the fix was written up in
//! `CLAUDE.md` with the reason beside it — and then `tags.advance_after_marking`
//! was written the same way and had the same fault, found the same way, three
//! years later.
//!
//! # Why this is a type and not a rule
//!
//! Because the rule was written down, carefully, with its reasoning, and broken
//! again anyway. A [`Reflect`] carries *both* accessors in one variant, so a
//! one-way mirror is a struct literal short of a field and the compiler refuses
//! it. There is nothing to remember.
//!
//! # Why it is generic
//!
//! `App` cannot be constructed without a GPU, so a table of `fn(&App)` is a
//! table no test can walk. `Mirror<L>` is generic over whatever holds the live
//! copy, and the tests below walk a `Fake` — which is what makes the mirroring
//! itself testable rather than only its call sites.

use super::Config;

/// The two halves of one mirrored value.
///
/// Every variant carries four functions: read and write on the live side, read
/// and write on the file's. Missing one is a compile error, which is the whole
/// point of the type.
pub enum Reflect<L> {
    Flag {
        /// What the program currently thinks.
        live: fn(&L) -> bool,
        /// Put the file's answer into the program.
        into_live: fn(&mut L, bool),
        /// What the file currently says.
        file: fn(&Config) -> bool,
        /// Put the program's answer into the file.
        into_file: fn(&mut Config, bool),
    },
}

/// One value the program and the file both hold.
pub struct Mirror<L: 'static> {
    /// Its registry path, which is its name everywhere else.
    pub path: &'static str,
    pub reflect: Reflect<L>,
}

impl<L> Mirror<L> {
    /// Writes the file from the program, for a value a key has nudged.
    ///
    /// Returns whether anything moved, so the caller saves once for the batch
    /// rather than once per value.
    pub fn remember(&self, live: &L, config: &mut Config) -> bool {
        match &self.reflect {
            Reflect::Flag {
                live: read,
                file,
                into_file,
                ..
            } => {
                let now = read(live);

                if file(config) == now {
                    return false;
                }

                into_file(config, now);
                true
            }
        }
    }

    /// Writes the program from the file, for a value the settings window moved.
    pub fn apply(&self, config: &Config, live: &mut L) {
        match &self.reflect {
            Reflect::Flag {
                file, into_live, ..
            } => into_live(live, file(config)),
        }
    }
}

/// Walks a table both ways.
///
/// Free functions rather than methods on a collection, because the table is a
/// `const` slice and the two walks are what the caller is actually asking for.
pub fn remember_all<L>(table: &[Mirror<L>], live: &L, config: &mut Config) -> bool {
    let mut moved = false;

    for mirror in table {
        moved |= mirror.remember(live, config);
    }

    moved
}

pub fn apply_all<L>(table: &[Mirror<L>], config: &Config, live: &mut L) {
    for mirror in table {
        mirror.apply(config, live);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Something that holds live copies, which `App` cannot be in a test
    /// because it needs a GPU. This is the reason `Mirror` is generic.
    #[derive(Default, PartialEq, Debug)]
    struct Fake {
        advancing: bool,
        filmstrip: bool,
    }

    const TABLE: &[Mirror<Fake>] = &[
        Mirror {
            path: "tags.advance_after_marking",
            reflect: Reflect::Flag {
                live: |fake| fake.advancing,
                into_live: |fake, on| fake.advancing = on,
                file: |config| config.tags.advance_after_marking,
                into_file: |config, on| config.tags.advance_after_marking = on,
            },
        },
        Mirror {
            path: "grid_view.filmstrip_visible",
            reflect: Reflect::Flag {
                live: |fake| fake.filmstrip,
                into_live: |fake, on| fake.filmstrip = on,
                file: |config| config.grid_view.filmstrip_visible,
                into_file: |config, on| config.grid_view.filmstrip_visible = on,
            },
        },
    ];

    #[test]
    fn a_key_press_reaches_the_file() {
        let live = Fake {
            advancing: true,
            filmstrip: false,
        };
        let mut config = Config::default();
        config.tags.advance_after_marking = false;

        assert!(remember_all(TABLE, &live, &mut config));
        assert!(config.tags.advance_after_marking);
    }

    /// The half that was missing twice. Without it, ticking the box sets the
    /// file and the next frame overwrites it with the value it was just asked
    /// to change.
    #[test]
    fn the_settings_window_reaches_the_program() {
        let mut live = Fake::default();
        let mut config = Config::default();
        config.tags.advance_after_marking = true;

        apply_all(TABLE, &config, &mut live);

        assert!(live.advancing);
    }

    /// The exact failure, played out: the window sets the file, the program is
    /// told, and the next frame's write-back finds nothing to do. Before, the
    /// write-back found a difference and undid the tick.
    #[test]
    fn a_tick_survives_the_frame_that_follows_it() {
        let mut live = Fake::default();
        let mut config = Config::default();

        // The settings window sets the file.
        config.tags.advance_after_marking = true;

        // The frame applies it, then writes back as it does every frame.
        apply_all(TABLE, &config, &mut live);
        let moved = remember_all(TABLE, &live, &mut config);

        assert!(!moved, "there is nothing left to write");
        assert!(
            config.tags.advance_after_marking,
            "and the tick is still ticked"
        );
    }

    #[test]
    fn nothing_moving_is_not_a_save() {
        let live = Fake::default();
        let mut config = Config::default();
        config.tags.advance_after_marking = false;
        config.grid_view.filmstrip_visible = false;

        assert!(!remember_all(TABLE, &live, &mut config));
    }

    #[test]
    fn one_value_moving_is_one_save_for_the_batch() {
        let live = Fake {
            advancing: true,
            filmstrip: true,
        };
        let mut config = Config::default();
        config.tags.advance_after_marking = false;
        config.grid_view.filmstrip_visible = false;

        assert!(remember_all(TABLE, &live, &mut config));
        assert!(config.tags.advance_after_marking);
        assert!(config.grid_view.filmstrip_visible);
    }

    /// Both directions compose: whatever the file says ends up in the program,
    /// and writing back changes nothing.
    #[test]
    fn the_two_walks_settle() {
        for (advancing, filmstrip) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut live = Fake::default();
            let mut config = Config::default();
            config.tags.advance_after_marking = advancing;
            config.grid_view.filmstrip_visible = filmstrip;

            apply_all(TABLE, &config, &mut live);

            assert_eq!(live.advancing, advancing);
            assert_eq!(live.filmstrip, filmstrip);
            assert!(!remember_all(TABLE, &live, &mut config));
        }
    }

    /// Every row names a registry path, because that is its name in the
    /// settings window, the search and the keyboard editor.
    #[test]
    fn every_row_names_a_row_the_registry_has() {
        for mirror in TABLE {
            assert!(
                crate::config::registry::row(mirror.path).is_some(),
                "{} is mirrored and is not a setting",
                mirror.path
            );
        }
    }
}
