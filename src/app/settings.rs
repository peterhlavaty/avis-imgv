//! The menu, and what a changed setting sets off.
//!
//! The menu opens cards — see `app::cards`, which holds the drawing of every
//! one of them. What is here is the other half: the fan-out when a value
//! moves. The views hold their own copies of the configuration, so a changed
//! setting has to be handed to each of them, and the file is written as soon
//! as anything in it moves, so a key changed in the middle of a session
//! survives the end of it.

use eframe::egui::{self, ViewportCommand};

use crate::actions::reveal;
use crate::config::load::Save;
use crate::config::mirror::{Mirror, Reflect};
use crate::config::Config;
use crate::formats;
use crate::ui::settings;

use super::cards::Card;
use super::panels::MenuAction;
use super::stores;
use super::App;

/// The three values the program holds a copy of and the file holds too.
///
/// A small `Copy` struct rather than `App` itself, for two reasons: `App`
/// cannot be borrowed immutably and mutably at once to walk a table against
/// its own `settings` field, and `App` cannot be built without a GPU, so a
/// table over it is a table no test can walk.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(super) struct Mirrored {
    advancing: bool,
    filmstrip_visible: bool,
    history_panel_visible: bool,
}

/// Both halves of each, in one struct literal each.
///
/// Adding a fourth means writing four accessors; leaving one out is a missing
/// field and the compiler says so. That is the whole mechanism, and it exists
/// because the rule it replaces was written down carefully and broken twice.
const MIRRORED: &[Mirror<Mirrored>] = &[
    Mirror {
        path: "tags.advance_after_marking",
        reflect: Reflect::Flag {
            live: |live| live.advancing,
            into_live: |live, on| live.advancing = on,
            file: |config| config.tags.advance_after_marking,
            into_file: |config, on| config.tags.advance_after_marking = on,
        },
    },
    Mirror {
        path: "grid_view.filmstrip_visible",
        reflect: Reflect::Flag {
            live: |live| live.filmstrip_visible,
            into_live: |live, on| live.filmstrip_visible = on,
            file: |config| config.grid_view.filmstrip_visible,
            into_file: |config, on| config.grid_view.filmstrip_visible = on,
        },
    },
    Mirror {
        path: "history.panel_visible",
        reflect: Reflect::Flag {
            live: |live| live.history_panel_visible,
            into_live: |live, on| live.history_panel_visible = on,
            file: |config| config.history.panel_visible,
            into_file: |config, on| config.history.panel_visible = on,
        },
    },
];

/// Where the manual lives. The README, which is what the program has.
const MANUAL: &str = "https://github.com/hats-np/avis-imgv#readme";

impl App {
    /// Carries out whatever the menu bar was asked for.
    pub(super) fn handle_menu(&mut self, action: MenuAction) {
        let dialog = rfd::FileDialog::new().set_directory(&self.base_path);

        // The bar answers the mouse from over a card, so a folder or a mode
        // asked for from there is asked for by somebody who wants to look at
        // photographs. Before it is carried out rather than after: opening a
        // folder is what puts a question about an empty bin up, and shutting
        // the deck afterwards would take nothing off but might have.
        if action.goes_back_to_the_photographs() {
            self.deck.shut();
        }

        match action {
            MenuAction::OpenFolder => {
                if let Some(folder) = dialog.pick_folder() {
                    self.open_directory(&folder, None);
                }
            }
            MenuAction::OpenFiles => {
                let picked = dialog
                    .add_filter("image", &formats::supported_extensions())
                    .pick_files();

                if let Some(files) = picked {
                    let first = files.first().cloned();
                    if let Some(parent) = first.as_ref().and_then(|f| f.parent()) {
                        self.open_directory(parent, first.as_deref());
                    }
                }
            }
            MenuAction::BinRejected => self.bin_rejected(),
            MenuAction::OpenBin => self.open_bin(),
            MenuAction::EmptyBin => self.ask_to_empty_the_bin(),
            MenuAction::Mode(mode) => self.set_mode(mode),
            MenuAction::AllSettings => self.open_settings(),
            MenuAction::Keyboard => self.open_card(Card::Keyboard),
            // A deep link to one of the eleven pages rather than a card of its
            // own. The window this was drew three of the five slideshow fields
            // and omitted the other two; the page draws all five.
            MenuAction::Slideshow => self.open_settings_at("slideshow.seconds_per_image"),
            MenuAction::CheatSheet => self.open_card(Card::CheatSheet),
            MenuAction::MarksLegend => self.open_card(Card::Legend),
            MenuAction::Placeholders => self.open_card(Card::Placeholders),
            MenuAction::Messages => self.open_card(Card::Messages),
            MenuAction::About => self.open_card(Card::About),
            MenuAction::OpenConfigFile => self.open_named_file(Config::path(), "configuration"),
            MenuAction::OpenLogFile => self.open_named_file(crate::logging::path(), "log"),
            MenuAction::OpenManual => {
                if !reveal::with_the_system(std::path::Path::new(MANUAL)) {
                    self.notices.fail("Could not open the manual.");
                }
            }
        }
    }

    /// Opens one of the viewer's own files with whatever the system uses.
    pub(super) fn open_named_file(&mut self, path: Option<std::path::PathBuf>, name: &str) {
        let Some(path) = path else {
            self.notices.fail(format!(
                "There is no configuration directory, so no {name} file."
            ));
            return;
        };

        if !reveal::with_the_system(&path) {
            self.notices.fail(format!(
                "Could not open the {name} file at {}.",
                path.display()
            ));
        }
    }

    /// Sends whatever fullscreen change a mode asked for.
    pub(super) fn apply_fullscreen(&mut self, ctx: &egui::Context) {
        if let Some(wanted) = self.pending_fullscreen.take() {
            ctx.send_viewport_cmd(ViewportCommand::Fullscreen(wanted));
        }
    }

    /// Writes the configuration out, telling the user when it could not be.
    ///
    /// The interesting failure is a file that was only partly understood on
    /// the way in: writing it back would replace whatever the viewer could not
    /// read with the defaults that stood in for it, so it is refused, and the
    /// person who just changed a key needs to know their change is not being
    /// kept.
    pub(super) fn save_settings(&mut self) {
        // Every deliberate change to the configuration ends here, which is what
        // makes this the honest place to tell the history to look. Comparing
        // the hundred and eighty registry rows on every frame instead cost ten
        // microseconds of every one of them, measured, for an answer that is
        // "nothing moved" all but a handful of times in a session.
        self.settings_touched = true;

        match self.settings.save() {
            Ok(Save::Written) => {}
            // Somebody has edited the file since it was read. Writing would
            // throw their edit away, so the question is asked instead.
            Ok(Save::Refused) => self.conflict_visible = true,
            Err(e) => {
                tracing::error!("Could not write the configuration: {e}");
                self.notices
                    .say(format!("Could not write the configuration: {e}"));
            }
        }
    }

    /// Writes back what the keys have nudged.
    ///
    /// Seven values a key changes for the session and the configuration also
    /// holds: the overlay's corner, how many photographs are side by side,
    /// how many thumbnails are across, what is drawn under them, whether
    /// marking advances, whether the strip of thumbnails is up, and whether
    /// the history panel is. Once the
    /// configuration is authoritative these have to be written, or the next
    /// save from the settings window snaps the view back to whatever the file
    /// still says — and the key's effect is lost at the next launch besides.
    ///
    /// Preferences go to the configuration file; only *position* stays in the
    /// session file, because that is what a session is for.
    pub(super) fn remember_runtime(&mut self) {
        let mut moved = false;

        let corner = self.image_view.overlay_corner();
        if self.settings.image_view.overlay_corner != corner {
            self.settings.image_view.overlay_corner = corner;
            moved = true;
        }

        let opens = self.image_view.opens();
        if self.settings.image_view.opening != opens.at {
            self.settings.image_view.opening = opens.at;
            moved = true;
        }

        let keeping = self.image_view.keeping();
        if self.settings.image_view.keep_zoom != keeping.zoom
            || self.settings.image_view.keep_pan != keeping.pan
        {
            self.settings.image_view.keep_zoom = keeping.zoom;
            self.settings.image_view.keep_pan = keeping.pan;
            moved = true;
        }

        let shown = self.image_view.images_shown();
        if self.settings.image_view.nr_images_shown != shown {
            self.settings.image_view.nr_images_shown = shown;
            moved = true;
        }

        let columns = self.grid_view.columns();
        if self.settings.grid_view.images_per_row != columns {
            self.settings.grid_view.images_per_row = columns;
            moved = true;
        }

        let badges = self.grid_view.badges();
        if self.settings.grid_view.badges != badges {
            self.settings.grid_view.badges = badges.to_string();
            moved = true;
        }

        // The three values the program and the file both hold. A table rather
        // than three hand-written comparisons, because two of the three had
        // only this half and the missing one was silent — see
        // `config::mirror`, where both halves live in one struct literal and a
        // one-way mirror does not compile.
        let live = self.mirrored();
        moved |= crate::config::mirror::remember_all(MIRRORED, &live, &mut self.settings);
        self.tag_config.advance_after_marking = self.settings.tags.advance_after_marking;

        if moved {
            self.save_settings();
        }
    }

    /// Hands the configuration to everything holding a copy of part of it,
    /// and builds the caches again where it has to.
    ///
    /// For the one-shot changes — a file re-read, an import, a reset — where
    /// there is no gesture to wait for the end of.
    pub(super) fn commit_settings(&mut self) {
        self.apply_settings();
        self.rebuild_stores();
    }

    /// Builds both stores again if anything they were built from has moved.
    ///
    /// Seventeen fields at once, and the reason none of them needs a restart:
    /// the two budgets and their two GPU halves, the preload radii, the decode
    /// ceiling, the thumbnail resolution, the camera-thumbnail count, all five
    /// raw settings and the screen profile are read exactly once, when a store
    /// is built. So the way to apply them is to build the store again.
    ///
    /// Called when a gesture *ends* rather than while it moves: a rail on true
    /// per-frame apply would empty and refill the cache sixty times a second.
    /// Both stores compare what they are running on against what the
    /// configuration now says, so a commit that changed something else costs
    /// two comparisons.
    pub(super) fn rebuild_stores(&mut self) {
        let profile: std::sync::Arc<str> =
            std::sync::Arc::from(self.settings.general.output_icc_profile.as_str());

        let images = stores::image_store(
            &self.settings.cache,
            &self.settings.image_view,
            &self.settings.raw,
        );
        let thumbnails = stores::thumbnail_store(&self.settings.cache, &self.settings.grid_view);

        let rebuilt = self
            .image_view
            .rebuild_store(images, std::sync::Arc::clone(&profile));
        let sheet = self.grid_view.rebuild_store(thumbnails, profile);

        if rebuilt || sheet {
            // Said out loud because it is work: everything decoded under the
            // old settings has just been thrown away, and a folder of raws
            // will take a moment to come back.
            self.notices
                .say("Filling the cache again on the new settings.");
        }
    }

    /// Hands the configuration to everything holding a copy of part of it.
    pub(super) fn apply_settings(&mut self) {
        self.pending_theme = Some(self.settings.general.theme == "light");

        // Applied on the next frame because it wants the context, and from
        // the base the styles were at when the viewer started, which is what
        // makes calling it again safe.
        self.pending_text_size = true;
        crate::annotations::sidecar::name_like_adobe(
            self.settings.tags.sidecar_naming == "replacing",
        );
        crate::ui::surface::show_settings_rows(self.settings.menus.settings_rows);
        crate::ui::slider::travels(self.settings.mouse.slider_travel);
        // Before the copies are replaced, because the question is whether the
        // window moved them rather than what they now say. `Sized` answers
        // "did it move, and by a route that was not a drag" for all four, so
        // a panel cannot be given three quarters of the rule.
        self.tag_panel_size = self.tag_panel_size.moved_to(self.settings.tags.panel_width);
        self.filmstrip_size = self
            .filmstrip_size
            .moved_to(self.settings.grid_view.filmstrip_height);
        self.side_panel_size = self
            .side_panel_size
            .moved_to(self.settings.general.side_panel_width);
        self.history_panel_size = self
            .history_panel_size
            .moved_to(self.settings.history.panel_width);

        self.config = self.settings.general.clone();
        self.tag_config = self.settings.tags.clone();

        // The keyword list, the file it is merged with and how many recent
        // keywords are offered were all read once at startup, so editing a
        // keyword list meant restarting the viewer to see it. The catalogue is
        // a pure function of the configuration; the recent list is in memory
        // and only its length changed, so it is trimmed rather than re-read.
        self.catalog = crate::annotations::Catalog::configured(&self.settings.tags);
        self.recent_tags.set_limit(self.settings.tags.recent_tags);
        self.image_view.set_config(self.settings.image_view.clone());
        self.image_view.set_mouse(self.settings.mouse.clone());
        self.image_view
            .set_backdrop(&self.settings.general.backdrop);
        self.grid_view.set_config(self.settings.grid_view.clone());
        self.grid_view.set_backdrop(&self.settings.general.backdrop);

        // Moving the bin can make the folder already on screen the bin, or
        // stop it being one, and the two menus read this rather than asking
        // per frame.
        let in_the_bin = self.in_the_bin();
        self.image_view.in_the_bin = in_the_bin;
        self.grid_view.in_the_bin = in_the_bin;
        // Shortening it takes effect on the frame it is shortened, rather than
        // at the next deed: a limit that has not bitten yet is a limit nobody
        // can tell is working.
        self.history.set_remember(self.settings.history.remember);

        // The other half of the two the panels need. `remember_runtime` writes
        // the live flag into the file so a key press survives the next launch;
        // without this, that was the only direction, and the tick in the
        // settings window was overwritten on the same frame by the flag it had
        // just been asked to change. "Show the strip" had never worked.
        // The other half of the three the panels and the marking need. Both
        // halves are one struct literal in `config::mirror`, so a value with
        // only one of them does not compile — which is what stops there being
        // a fourth after "show the strip" and "advance after marking".
        let mut live = self.mirrored();
        crate::config::mirror::apply_all(MIRRORED, &self.settings, &mut live);
        self.take_mirrored(live);
    }

    /// The three values the file also holds, as the mirror sees them.
    fn mirrored(&self) -> Mirrored {
        Mirrored {
            advancing: self.advancing,
            filmstrip_visible: self.filmstrip_visible,
            history_panel_visible: self.history_panel_visible,
        }
    }

    fn take_mirrored(&mut self, live: Mirrored) {
        self.advancing = live.advancing;
        self.filmstrip_visible = live.filmstrip_visible;
        self.history_panel_visible = live.history_panel_visible;
    }
}

impl App {
    /// What a slider's own menu asked for, taken on the frame it was asked.
    ///
    /// A rail is drawn in four subsystems and not one of them has the
    /// configuration in hand, so the menu leaves its ask in `ui::slider` and it
    /// is collected here. Through the same two calls the settings window makes,
    /// so a travel picked from a menu and a travel typed into the window are
    /// the same deed — including in the history, which watches the registry
    /// whenever `save_settings` says something moved.
    /// Carries out whatever a panel's own menu asked for.
    ///
    /// The mirror of [`Self::take_slider_ask`], and for the same reason: a
    /// panel is drawn from a subsystem that has neither the command dispatcher
    /// nor the configuration in hand, so it leaves its ask in a mailbox rather
    /// than growing a return value along four call chains. Everything it can
    /// ask for goes through a route that already exists, which is what puts a
    /// panel put away from its own menu in the history without a word here.
    pub(super) fn take_panel_ask(&mut self, ctx: &egui::Context) {
        match crate::ui::panel::asked() {
            Some(crate::ui::panel::Ask::Toggle(command)) => self.apply(command, ctx),
            Some(crate::ui::panel::Ask::Settings(path)) => self.open_settings_at(path),
            Some(crate::ui::panel::Ask::BindAKey(path)) => self.arm_key(path),
            None => {}
        }
    }

    pub(super) fn take_slider_ask(&mut self) {
        match crate::ui::slider::asked() {
            Some(crate::ui::slider::Ask::Travel(travel)) => {
                if (self.settings.mouse.slider_travel - travel).abs() < 0.01 {
                    return;
                }

                self.settings.mouse.slider_travel = travel;
                self.apply_settings();
                self.save_settings();

                // Said out loud because the change is only felt on the next
                // drag, and a menu that closes having apparently done nothing
                // is a menu nobody trusts again.
                self.notices.say(if travel <= crate::config::mouse::BOUND {
                    "Sliders follow the pointer.".to_string()
                } else {
                    format!("The pointer now moves {travel:.0}× the rail to cross a slider.")
                });
            }
            Some(crate::ui::slider::Ask::Settings) => {
                self.open_settings_at("mouse.slider_travel");
            }
            None => {}
        }
    }

    /// Opens the settings card on the page it was last left on.
    pub(super) fn open_settings(&mut self) {
        if self.settings_state.page.is_none() {
            self.settings_state.page = Some(page_named(&self.settings.general.last_settings_page));
        }

        // The nine checks: seven the configuration can make about itself,
        // and two that need the window's own words.
        self.settings_state.problems = self.settings.check();
        self.settings_state
            .problems
            .extend(crate::ui::checks::about_the_window(&self.settings));

        // The migration report and the key clashes had six seconds and no way
        // back; here they have a home that does not fade.
        self.settings_state.at_startup = self.startup_notices.clone();
        self.settings_state.just_opened = true;
        self.open_card(Card::Settings);
    }

    /// Opens it on a named row, from a link or a **[Fix]** button.
    pub(super) fn open_settings_at(&mut self, path: &'static str) {
        self.open_settings();

        if let Some(row) = crate::config::registry::row(path) {
            self.settings_state.page = Some(row.page);
            self.settings_state.query.clear();
            self.settings_state.reveal = Some(path);
        }
    }

    pub(super) fn run_settings_button(&mut self, run: settings::Run, ctx: &egui::Context) {
        if run == settings::Run::Restart {
            self.session.save();
            restart();
            return;
        }

        match crate::ui::settings::footer::carry_out(run, &self.settings) {
            Ok(said) if said.is_empty() => {}
            Ok(said) => self.notices.say(said),
            Err(problem) if problem.is_empty() => self.import_settings(ctx),
            Err(problem) => self.notices.fail(problem),
        }
    }

    /// Reads a saved settings file over this one.
    fn import_settings(&mut self, _ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("settings", &["json"])
            .pick_file()
        else {
            return;
        };

        let Ok(text) = std::fs::read_to_string(&path) else {
            self.notices
                .fail(format!("Could not read {}.", path.display()));
            return;
        };

        // Merged into what is held rather than replacing it: a bundle is a
        // patch, and the fields it does not name are the fields it does not
        // change.
        let merged = match merge_bundle(&self.settings, &text) {
            Ok(merged) => merged,
            Err(problem) => {
                self.notices.fail(problem);
                return;
            }
        };

        let named = merged.1;
        self.settings = merged.0;
        self.commit_settings();
        self.save_settings();
        self.notices
            .say(format!("Read {named} setting(s) from {}.", path.display()));
    }
}

/// Applies a bundle over a configuration, reporting how many fields it named.
fn merge_bundle(config: &Config, text: &str) -> Result<(Config, usize), String> {
    let bundle: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(text).map_err(|e| format!("That is not a settings file: {e}"))?;

    let mut document = serde_json::to_value(config)
        .map_err(|e| format!("Could not read what is held: {e}"))?
        .as_object()
        .cloned()
        .ok_or_else(|| "Could not read what is held.".to_string())?;

    let mut named = 0;

    for (section, values) in &bundle {
        let (Some(values), Some(into)) = (
            values.as_object(),
            document.get_mut(section).and_then(|s| s.as_object_mut()),
        ) else {
            continue;
        };

        for (key, value) in values {
            into.insert(key.clone(), value.clone());
            named += 1;
        }
    }

    let json = serde_json::Value::Object(document).to_string();
    let mut merged = Config::from_json(&json);

    // Whatever the bundle did not name stays as it was, including the document
    // this configuration was read from — an import is not a reason to drop
    // somebody's unknown keys.
    merged.document = config.document.clone();

    Ok((merged, named))
}

/// Which page a remembered name is.
fn page_named(name: &str) -> crate::config::registry::Page {
    crate::config::registry::Page::ALL
        .iter()
        .copied()
        .find(|page| format!("{page:?}") == name)
        .unwrap_or(crate::config::registry::Page::OpeningAFolder)
}

/// Saves the session and starts the viewer again.
///
/// A button that does the thing rather than a toast telling somebody to do it:
/// darktable's own fix for this complaint was a toast on closing the dialogue.
fn restart() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = std::process::Command::new(exe).args(args).spawn() {
        tracing::error!("Could not start the viewer again: {e}");
        return;
    }

    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This file used to hold a test that read its own source, found every
    /// `if self.settings.x != self.y` in `remember_runtime`, and asserted that
    /// `apply_settings` wrote each one back. It was written in the commit that
    /// fixed "advance after marking", because at that point the rule had no
    /// other enforcement.
    ///
    /// It is gone, and its going is the point: the three values are a table of
    /// `Mirror`s now, and each carries both halves in one struct literal, so a
    /// one-way mirror is a missing field rather than a passing test. A test
    /// that reads source is what you write when the type cannot say it.
    ///
    /// What is left to check here is that the table names real settings.
    #[test]
    fn every_mirrored_value_is_a_setting_the_registry_has() {
        assert_eq!(MIRRORED.len(), 3);

        for mirror in MIRRORED {
            assert!(
                crate::config::registry::row(mirror.path).is_some(),
                "{} is mirrored and is not a setting",
                mirror.path
            );
        }
    }

    /// The fault this whole arrangement exists to prevent, played out against
    /// the real table: the window sets the file, the frame applies it, and the
    /// write-back that follows finds nothing to undo.
    #[test]
    fn a_tick_in_the_settings_window_survives_the_next_frame() {
        for path in MIRRORED.iter().map(|mirror| mirror.path) {
            let mut config = Config::default();
            let mut live = Mirrored::default();

            // Whatever the default is, ask for the other one.
            crate::config::mirror::apply_all(MIRRORED, &config, &mut live);
            let before = live;

            for mirror in MIRRORED {
                if mirror.path == path {
                    let crate::config::mirror::Reflect::Flag {
                        file, into_file, ..
                    } = &mirror.reflect;
                    let flipped = !file(&config);
                    into_file(&mut config, flipped);
                }
            }

            crate::config::mirror::apply_all(MIRRORED, &config, &mut live);
            assert_ne!(live, before, "{path} did not reach the program");

            let moved = crate::config::mirror::remember_all(MIRRORED, &live, &mut config);
            assert!(!moved, "{path} was undone by the write-back");
        }
    }
}
