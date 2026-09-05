//! What a slideshow answers the second button with.
//!
//! Everything drawn answers the second button with the verbs that apply to it
//! and the settings that govern it, and in a slideshow neither of those is
//! what the photograph's own menu carries. Keeping, throwing out, comparing
//! and zooming are the verbs of somebody culling a shoot; a picture that will
//! be gone in five seconds is not being decided about. What the person in
//! front of a slideshow wants is the slideshow itself: how long each picture
//! stays up, whether and how it moves while it is there, and the way out.
//!
//! The way out matters most. A fullscreen mode has put every panel away, so
//! there is no bar to reach for and this menu is the only surface on the
//! screen — which is why the row that leaves is first and names the key that
//! does the same thing, and why nothing here is reachable *only* from here.
//!
//! The two rows that do survive from the photograph's list are in
//! [`Row::IN_A_SLIDESHOW`], and are drawn from it rather than written out
//! again.

use eframe::egui::{self, Response};

use crate::config::registry::Page;
use crate::config::{Motion, SlideshowConfig};
use crate::ui::menus::{self, Chosen, Row, Verb};
use crate::ui::surface::{self, Subject};
use crate::view::image_view::bottom_bar::BarAction;
use crate::view::image_view::ImageView;

/// The durations the menu offers without opening the settings window.
///
/// A photo frame's answer at one end and a look through a shoot at the other,
/// with the ordinary answers between. Anything else is a number to be typed,
/// which is what the settings row is for — and the configured value joins the
/// list wherever it falls, so exactly one row is always ticked.
const HELD_FOR: &[u64] = &[2, 3, 5, 10, 15, 30, 60];

/// Where the settings for a slideshow live, for the rows that lead to them.
const PAGE: Page = Page::Slideshow;

/// What the slideshow's menu asked for.
///
/// Never carried out where it is drawn: the duration and the motion are lines
/// of the configuration file, leaving is a mode, and a turn is written to a
/// sidecar. Every one of them goes up through the outbox the view already has.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Asked {
    /// Stop the slideshow and go back to the photograph.
    Leave,
    /// Hold each picture for this many seconds.
    Seconds(u64),
    /// Move the picture this way while it is up.
    Moves(Motion),
    /// Open the settings window at this row.
    Settings(&'static str),
    /// Arm the keyboard editor on the row that binds this.
    BindKey(&'static str),
    /// One of the shared rows: a turn. The panels answer for themselves,
    /// through `ui::panel`'s own mailbox.
    Verb(Verb),
}

/// The durations a menu offers, given the one in force.
///
/// The configured value is in the list wherever it belongs, so a slideshow set
/// to seven seconds from the settings window shows seven seconds ticked rather
/// than nothing ticked at all — a menu where none of the answers is the
/// current one reads as a menu that has lost track of the state.
fn offered(seconds: u64) -> Vec<u64> {
    let mut held: Vec<u64> = HELD_FOR.to_vec();

    if let Err(at) = held.binary_search(&seconds) {
        held.insert(at, seconds);
    }

    held
}

impl ImageView {
    /// Draws the slideshow's menu and posts whatever it asked for.
    ///
    /// Here rather than beside the photograph's own menu in `interaction.rs`,
    /// because the rows and what they mean are one subject and the file that
    /// holds one should hold the other.
    pub(in crate::view::image_view) fn handle_slideshow_menu(
        &mut self,
        ctx: &egui::Context,
        response: &Response,
    ) {
        let ui = egui::Ui::new(
            ctx.clone(),
            egui::Id::new("slideshow menu"),
            egui::UiBuilder::new().max_rect(response.rect),
        );

        let Some(asked) = show(&ui, response, &self.slideshow_config) else {
            return;
        };

        match asked {
            Asked::Leave => self
                .bar_actions
                .push(BarAction::Mode(crate::mode::Mode::Image)),
            Asked::Seconds(seconds) => self
                .bar_actions
                .push(BarAction::SetSlideshowSeconds(seconds)),
            Asked::Moves(motion) => self.bar_actions.push(BarAction::SetSlideshowMotion(motion)),
            Asked::Settings(path) => self.bar_actions.push(BarAction::Settings(path)),
            Asked::BindKey(path) => self.bar_actions.push(BarAction::BindKey(path)),
            // A turn, which is about the photograph rather than about the
            // slideshow: one picture is on screen in a slideshow, so the
            // cursor is the one the button came down on.
            Asked::Verb(verb) => {
                if let Some(path) = self.path_at_cursor() {
                    self.verb = Some((verb, path));
                }
            }
        }
    }
}

/// Draws the menu when the second button asks for it, and says what it was
/// asked for.
fn show(ui: &egui::Ui, response: &Response, config: &SlideshowConfig) -> Option<Asked> {
    surface::named_menu(
        ui,
        response,
        "slideshow",
        Subject::the("The slideshow"),
        |ui| rows(ui, config),
    )
    .flatten()
}

/// The rows themselves, apart from the popup they are drawn in, so that a test
/// can read them without opening one.
fn rows(ui: &mut egui::Ui, config: &SlideshowConfig) -> Option<Asked> {
    let mut asked = None;

    ui.set_max_width(surface::WIDEST);

    // First, because with every panel put away this menu is the only way out
    // that is on the screen. The key named beside it is the one that does the
    // same thing: the slideshow is the last mode round, so the next one is the
    // photograph.
    if crate::ui::keys::button(ui, "Leave the slideshow", "general.sc_next_mode")
        .on_hover_text("Back to the photograph, with the panels as they were.")
        .clicked()
    {
        asked = Some(Asked::Leave);
        ui.close();
    }

    // Beside the row it is about, the way a panel's menu puts the keys for
    // showing it under the row that hides it.
    if surface::bind_a_key(ui, "leaving the slideshow") {
        asked = Some(Asked::BindKey("general.sc_next_mode"));
        ui.close();
    }

    ui.separator();

    asked = how_long(ui, config.seconds_per_image).or(asked);
    asked = while_it_is_up(ui, config.motion).or(asked);

    // The turns and the panels, from the one list, so a verb added there is a
    // verb here. The panels set themselves apart with a rule of their own,
    // being the one row that is about the window rather than about what is on
    // the screen.
    if let Some(Chosen::Verb(verb)) = menus::rows(ui, Row::IN_A_SLIDESHOW, &[], 1) {
        asked = Some(Asked::Verb(verb));
    }

    // Last, and drawing the separator above itself, as it does everywhere.
    if surface::more_settings(ui, PAGE) {
        asked = Some(Asked::Settings("slideshow.seconds_per_image"));
        ui.close();
    }

    asked
}

/// How long each picture stays up: one decision with several answers, which is
/// what buys a second level.
fn how_long(ui: &mut egui::Ui, seconds: u64) -> Option<Asked> {
    let mut asked = None;

    ui.menu_button("Hold each picture for", |ui| {
        ui.set_max_width(220.);

        for offer in offered(seconds) {
            if crate::ui::keys::radio(ui, offer == seconds, format!("{offer} s"), "").clicked() {
                asked = Some(Asked::Seconds(offer));
                ui.close();
            }
        }

        if surface::more_settings(ui, PAGE) {
            asked = Some(Asked::Settings("slideshow.seconds_per_image"));
            ui.close();
        }
    });

    asked
}

/// Whether the picture moves while it is on screen, and how.
///
/// The three answers and their sentences are the enum's own, so this menu and
/// the settings page say the same words about them.
fn while_it_is_up(ui: &mut egui::Ui, motion: Motion) -> Option<Asked> {
    let mut asked = None;

    ui.menu_button("While it is up", |ui| {
        ui.set_max_width(260.);

        for offer in Motion::ALL {
            if crate::ui::keys::radio(ui, *offer == motion, offer.label(), "")
                .on_hover_text(offer.description())
                .clicked()
            {
                asked = Some(Asked::Moves(*offer));
                ui.close();
            }
        }

        if surface::more_settings(ui, PAGE) {
            asked = Some(Asked::Settings("slideshow.motion"));
            ui.close();
        }
    });

    asked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::drawn;

    fn config() -> SlideshowConfig {
        SlideshowConfig {
            seconds_per_image: 5,
            percent_zoom: 25.0,
            motion: Motion::Zoom,
            start_with_frame_enabled: false,
            image_frame_background_color_override: None,
        }
    }

    /// What the menu paints, with the submenus left folded.
    fn said() -> String {
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rows(ui, &config());
            });
        });

        drawn::text(&output).join(" | ")
    }

    /// One row is ticked whatever the file says, which is the whole reason the
    /// list is built rather than written out.
    #[test]
    fn the_configured_duration_is_always_one_of_the_answers() {
        assert!(offered(7).contains(&7));
        assert!(offered(3600).contains(&3600));
        assert_eq!(
            offered(5),
            HELD_FOR.to_vec(),
            "one already in it is kept once"
        );
    }

    /// In order, so the list reads as a scale rather than as a scale with one
    /// number stuck on the end.
    #[test]
    fn the_durations_are_offered_in_order() {
        let held = offered(7);

        assert!(held.windows(2).all(|pair| pair[0] < pair[1]), "{held:?}");
        assert_eq!(held.iter().filter(|it| **it == 7).count(), 1);
    }

    /// The way out is on the menu, and it is the first thing on it: with every
    /// panel put away there is nothing else on the screen to ask.
    #[test]
    fn the_way_out_is_the_first_row() {
        let words = {
            let ctx = egui::Context::default();
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    rows(ui, &config());
                });
            });

            drawn::text(&output)
        };

        let leaves = words
            .iter()
            .position(|said| said.contains("Leave the slideshow"));

        assert_eq!(leaves, Some(0), "{words:?}");
    }

    /// The slideshow's own two decisions are on it, and the culling verbs are
    /// not: a picture that will be gone in five seconds is not one anybody is
    /// deciding about.
    #[test]
    fn it_offers_the_slideshow_rather_than_the_culling() {
        let said = said();

        assert!(said.contains("Hold each picture for"), "{said}");
        assert!(said.contains("While it is up"), "{said}");
        assert!(said.contains("Turn"), "{said}");
        assert!(said.contains("Show"), "{said}");
        assert!(!said.contains("Keep"), "{said}");
        assert!(!said.contains("bin"), "{said}");
        assert!(!said.contains("Compare"), "{said}");
    }

    /// The last row of every menu in the program, and this one is no
    /// exception: nothing here is reachable only by right-click.
    #[test]
    fn it_ends_on_the_page_that_owns_it() {
        assert!(said().contains(Page::Slideshow.label()), "{}", said());
    }
}
