//! What each card holds, and what a card that changed something sets off.
//!
//! The other half of `app::cards`: that file says which cards there are and
//! draws whichever is up, and this one is the thirteen bodies. Everything
//! outside is drawn by the module that owns it — the settings by
//! `ui::settings`, the keys by `ui::keys`, the legend by `ui::legend` — so
//! what is here is the wiring each of them needs and nothing else.

use eframe::egui;

use crate::ui::{cheat_sheet, destinations, keys, legend, notice, placeholders, settings};

use super::super::{about, conflict, App};
use super::Card;

impl App {
    /// What one card holds, and what it did.
    pub(super) fn contents(&mut self, ui: &mut egui::Ui, card: Card) {
        match card {
            Card::Settings => self.settings_card(ui),
            Card::Keyboard => self.keyboard_card(ui),
            Card::OneKey => self.one_key_card(ui),
            Card::CheatSheet => self.cheat_sheet_card(ui),
            Card::Legend => legend::contents(ui),
            Card::Placeholders => placeholders::contents(ui),
            Card::Messages => self.messages_card(ui),
            Card::About => about::contents(ui, &self.about),
            Card::Deleting => self.ask_about_deleting(ui),
            Card::Destinations => self.ask_about_destinations(ui),
            Card::TheBin => self.ask_about_leaving(ui),
            Card::TheHistory => self.ask_about_history(ui),
            Card::TheConfiguration => self.ask_about_the_configuration(ui),
        }
    }

    /// The settings, and everything a changed row sets off.
    ///
    /// One commit model across every row: no OK, no Cancel, no Apply. The one
    /// qualification is arithmetic — the stores are pure functions of the
    /// configuration, so a rail on true per-frame apply would rebuild the
    /// cache sixty times a second, and a row whose effect is a rebuild waits
    /// for the gesture to end.
    fn settings_card(&mut self, ui: &mut egui::Ui) {
        // The one field in the whole card that cannot take effect while it is
        // open. The pool is spawned once, each thread is expected to exist,
        // and one loader is shared by both views; draining a running pool
        // mid-session is a larger job than this deserves, and pretending
        // otherwise would be worse than saying so.
        self.settings_state.waiting_on_a_restart = (self.settings.cache.decode_threads
            != self.threads_at_start)
            .then_some("decode threads");

        let outcome = settings::contents(ui, &mut self.settings_state, &mut self.settings);

        if let Some(page) = self.settings_state.page {
            // Through the registry into `config.json`, not into the session
            // file: `on_exit` returns early when `restore_session` is off, so a
            // preference kept there is one some people silently do not have.
            let remembered = format!("{page:?}");
            if self.settings.general.last_settings_page != remembered {
                self.settings.general.last_settings_page = remembered;
            }
        }

        if let Some(path) = self.settings_state.arm_key.take() {
            self.arm_key(path);
        }

        if let Some(run) = outcome.run {
            let ctx = ui.ctx().clone();
            self.run_settings_button(run, &ctx);
        }

        if outcome.committed {
            self.rebuild_stores();
        }

        if !outcome.changed {
            return;
        }

        self.apply_settings();

        // Written on every gesture, which is what makes the card the thing
        // that decides rather than a form to be submitted.
        self.save_settings();
    }

    /// The list of every command, with a sentence against each.
    ///
    /// The whole card takes the keyboard, not only an armed row: a key pressed
    /// here is a key being bound, and one the viewer also read would both
    /// rebind and fire.
    fn keyboard_card(&mut self, ui: &mut egui::Ui) {
        let (outcome, armed) = keys::list(ui, &mut self.keys, &mut self.settings);

        // A row asked for one command's keys. The card goes on top of this
        // one, so the list is the way back.
        if armed == keys::Armed(true) {
            self.deck.go(Card::OneKey);
        }

        if outcome.is_none() {
            return;
        }

        self.apply_settings();
        self.save_settings();
    }

    /// Every key bound to one command, and the way to add or take one away.
    fn one_key_card(&mut self, ui: &mut egui::Ui) {
        let (changed, asked) = keys::one::card(ui, &mut self.keys, &mut self.settings);

        if changed {
            self.apply_settings();
            self.save_settings();
        }

        // The way out to the index, which is where this card ends for the same
        // reason every menu ends on the settings page that owns it.
        if asked == Some(keys::one::Asked::EveryKey) {
            self.keys.close_one();
            self.deck.go(Card::Keyboard);
        }
    }

    fn cheat_sheet_card(&mut self, ui: &mut egui::Ui) {
        let just_opened = std::mem::take(&mut self.cheat_sheet_opened);
        let mut change = None;

        let stays = cheat_sheet::contents(
            ui,
            &self.settings,
            self.mode,
            just_opened,
            &mut self.cheat_sheet_query,
            &mut change,
        );

        if !stays {
            self.deck.close(Card::CheatSheet);
        }

        // The route out. A row opens the keys of the command it names; the
        // footer opens the list of every one of them.
        let Some(path) = change else {
            return;
        };

        // A key row opens that command's keys; a gesture row opens the page
        // that owns it. The sheet lists both now, and the row itself says
        // which it is.
        if path.is_empty() {
            self.deck.go(Card::Keyboard);
        } else if crate::config::bindings::is_a_key(path) {
            self.arm_key(path);
        } else {
            self.open_settings_at(path);
        }
    }

    fn messages_card(&mut self, ui: &mut egui::Ui) {
        match notice::contents(ui, &mut self.notices) {
            Some(notice::Asked::OpenLog) => self.open_named_file(crate::logging::path(), "log"),
            Some(notice::Asked::Keys) => self.deck.go(Card::Keyboard),
            None => {}
        }
    }

    /// Where a move or a copy is going, and what was picked.
    fn ask_about_destinations(&mut self, ui: &mut egui::Ui) {
        let Some(asking) = self.asking.clone() else {
            return;
        };

        let Some(answer) = destinations::contents(ui, &asking) else {
            return;
        };

        self.carry_destination(&asking, answer);
    }

    /// The question about a file edited underneath, and what it was told.
    fn ask_about_the_configuration(&mut self, ui: &mut egui::Ui) {
        match conflict::contents(ui) {
            conflict::Answer::Waiting => return,
            conflict::Answer::Reread => {
                self.settings = crate::config::Config::new();
                self.commit_settings();
                self.notices.say("Read the configuration file again.");
            }
            conflict::Answer::Overwrite => {
                if let Err(e) = self.settings.save_over() {
                    tracing::error!("Could not write the configuration: {e}");
                    self.notices
                        .say(format!("Could not write the configuration: {e}"));
                } else {
                    self.notices.say("Wrote over the configuration file.");
                }
            }
        }

        self.conflict_visible = false;
    }
}
