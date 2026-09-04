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
use crate::config::Config;
use crate::formats;
use crate::ui::settings;

use super::cards::Card;
use super::panels::MenuAction;
use super::stores;
use super::App;

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

        if self.settings.tags.advance_after_marking != self.advancing {
            self.settings.tags.advance_after_marking = self.advancing;
            self.tag_config.advance_after_marking = self.advancing;
            moved = true;
        }

        if self.settings.grid_view.filmstrip_visible != self.filmstrip_visible {
            self.settings.grid_view.filmstrip_visible = self.filmstrip_visible;
            moved = true;
        }

        if self.settings.history.panel_visible != self.history_panel_visible {
            self.settings.history.panel_visible = self.history_panel_visible;
            moved = true;
        }

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
        // window moved them rather than what they now say.
        self.forced_panel_width |=
            (self.tag_config.panel_width - self.settings.tags.panel_width).abs() > 0.5;
        self.forced_filmstrip_height |=
            (self.grid_view.filmstrip_height() - self.settings.grid_view.filmstrip_height).abs()
                > 0.5;
        // The other two resizable panels, which had none of this: they were
        // drawn with `default_width` alone, and egui keeps a width of its own
        // for a panel from the second frame on, so a width typed into the
        // settings window reached them at the next launch and not before.
        self.forced_side_panel_width |=
            (self.config.side_panel_width - self.settings.general.side_panel_width).abs() > 0.5;
        // The history panel keeps no live copy — it reads the file every
        // frame — so there is nothing to compare against and it takes its
        // exact frame whenever anything in the settings moved. That costs one
        // frame drawn at the width the user asked for, which is the width it
        // was going to be drawn at anyway.
        self.forced_history_panel_width = true;

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
        self.filmstrip_visible = self.settings.grid_view.filmstrip_visible;
        self.history_panel_visible = self.settings.history.panel_visible;
        // The third of them, and it had the same fault for the same reason:
        // `remember_runtime` wrote `advancing` into the file and nothing wrote
        // it back, so ticking "advance after marking" in the settings window
        // was undone on the next frame by the flag it had just been asked to
        // change. Two of these have now been found by hand. The type that
        // makes a one-way mirror fail to compile is what stops there being a
        // fourth.
        self.advancing = self.settings.tags.advance_after_marking;
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
    /// Every live field the program writes into the file is written back out
    /// of it.
    ///
    /// `remember_runtime` writes the file from the program and
    /// `apply_settings` writes the program from the file. A value with only
    /// the first half is a setting that cannot be changed from the settings
    /// window: the tick is overwritten on the next frame by the flag it was
    /// just asked to change. That has now happened twice — "show the strip",
    /// and "advance after marking" — and both times it was found by somebody
    /// noticing a checkbox did nothing.
    ///
    /// The two halves are matched on the *live* location rather than on the
    /// settings path, because the settings path is written back by the bulk
    /// handovers (`set_config`, and the section clones) whether or not the
    /// field beside it is. That is exactly how the second one hid: the file's
    /// `tags` section was being handed to `tag_config` in full, while
    /// `App::advancing` next to it was read by nobody.
    ///
    /// Only the fields sourced from a bare `App` field are checked. The ones
    /// read out of a view by a method — the corner, the opening, the columns —
    /// go back through `set_config` with the rest of their section, which is a
    /// different shape and one the compiler already keeps whole.
    #[test]
    fn a_setting_written_from_the_program_is_read_back_into_it() {
        let source = include_str!("settings.rs");
        // Cut the tests off, or this very comment is part of what is searched.
        let code = source
            .split_once("#[cfg(test)]")
            .map_or(source, |(code, _)| code);

        let body = |name: &str| {
            let from = code
                .find(&format!("fn {name}("))
                .unwrap_or_else(|| panic!("{name} is still called that"));
            let rest = &code[from..];
            // To the start of the next item at the same indentation.
            let to = rest[1..].find("\n    /// ").map_or(rest.len(), |at| at + 1);
            &rest[..to]
        };

        let remember = body("remember_runtime");
        let apply = body("apply_settings");

        let mut checked = 0;

        for line in remember.lines() {
            let Some((left, right)) = line.split_once(" != self.") else {
                continue;
            };

            if !left.trim_start().starts_with("if self.settings.") {
                continue;
            }

            let live = right.trim_end_matches(&[' ', '{', '\n'][..]).trim();

            // A method call is a view handing its state over, not a field.
            if live.contains('(') || live.is_empty() {
                continue;
            }

            assert!(
                apply.contains(&format!("self.{live} = self.settings.")),
                "`App::{live}` is written into the file by `remember_runtime` \
                 and never read back by `apply_settings`, so changing it in \
                 the settings window does nothing: the next frame overwrites \
                 it with the value it was asked to change. Add \
                 `self.{live} = self.settings.<section>.<field>;` to \
                 `apply_settings`."
            );

            checked += 1;
        }

        // The three that exist today. If the shape of `remember_runtime`
        // changes so that none is found, the test above passes by matching
        // nothing at all, which is worse than failing.
        assert!(
            checked >= 3,
            "only {checked} mirrored fields were found; the test has stopped \
             reading `remember_runtime` and is now asserting nothing"
        );
    }
}
