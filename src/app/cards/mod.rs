//! Which cards this program has, and the drawing of whichever is up.
//!
//! Everything that used to open a window opens a card instead. A card is not
//! a window: it has no title bar to drag, no corner to pull, no position to
//! remember, and there is never a second one behind the first. One is on
//! screen, the bar at its top says which and how to get back, and `Escape`
//! takes it off — the same key, from every one of them.
//!
//! Thirteen cards, of two kinds, and the difference is what the card is
//! *about*.
//!
//! - A **page** is about itself: the settings, the keyboard, the sheet of
//!   keys, the legend, the placeholders, the messages, what this build is. It
//!   takes the whole window under the menu bar, because what is behind it is
//!   no part of the question and looking at it is a distraction. These are the
//!   ones a person opens, and they stack: opening the keys of one command from
//!   the keyboard list leaves the list as the way back.
//! - A **question** is about the photographs on screen: three of them going to
//!   the bin, a bin with something still in it, a run of the history, a
//!   configuration file edited underneath. It is a plate over the rest dimmed,
//!   because "send these three to the bin" cannot be answered by somebody who
//!   can no longer see them.
//!
//! A question is not opened and not stacked. It is *derived*, once a frame,
//! from the state that makes it — [`App::asked`] — so there is no second flag
//! saying whether a question is up that could disagree with whether there is
//! one. Answering it puts that state down and the card goes with it, and the
//! page the question was asked from is still underneath.
//!
//! What each of them *holds* is `contents`, next door: this file is which
//! cards there are and the drawing of whichever is up.

mod contents;

use eframe::egui;

use crate::ui::deck::{Ask, Face, Spread};
use crate::ui::{deck, keys};

use super::{cull, input, App};

/// One card of the deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Card {
    Settings,
    Keyboard,
    /// Every key bound to one command, which command being `keys::State`'s.
    OneKey,
    CheatSheet,
    Legend,
    Placeholders,
    Messages,
    About,
    /// Photographs on their way to the bin, or off the disk.
    Deleting,
    /// Where a move or a copy is going.
    Destinations,
    /// A bin with something still in it, on the way out of the program.
    TheBin,
    /// A run of the history that would touch files.
    TheHistory,
    /// A configuration file edited under the viewer.
    TheConfiguration,
}

impl Card {
    /// The cards a person opens deliberately, in the order the switcher lists
    /// them.
    ///
    /// Not [`Card::OneKey`], which is about a command and has to be told
    /// which; not the questions, which arrive rather than being asked for.
    pub const EVERY: [Card; 7] = [
        Card::Settings,
        Card::Keyboard,
        Card::CheatSheet,
        Card::Legend,
        Card::Placeholders,
        Card::Messages,
        Card::About,
    ];

    /// Whether this card is a question about what is behind it.
    ///
    /// A question is drawn as a plate rather than a page, carries no cross —
    /// its own answers are the way out — and is never put on the deck: it is
    /// derived from the state that asked it.
    pub fn asks(self) -> bool {
        matches!(
            self,
            Card::Deleting
                | Card::Destinations
                | Card::TheBin
                | Card::TheHistory
                | Card::TheConfiguration
        )
    }

    /// How much of the window it takes.
    ///
    /// The widths are the ones the windows these replaced had chosen, which
    /// were chosen against the length of the sentences in them.
    pub fn spread(self) -> Spread {
        match self {
            Card::Deleting | Card::TheHistory => Spread::Plate(520.0),
            Card::Destinations => Spread::Plate(420.0),
            Card::TheBin => Spread::Plate(560.0),
            Card::TheConfiguration => Spread::Plate(500.0),
            _ => Spread::Full,
        }
    }
}

impl App {
    /// The question outstanding, if there is one.
    ///
    /// Asked of the state rather than of a flag, so a question cannot be up
    /// without the thing it is about, or the thing without the question. The
    /// order is how urgent they are: a deletion waiting on an answer is what
    /// somebody is looking at, and a configuration file edited underneath has
    /// been waiting since whenever it was edited.
    pub(super) fn asked(&self) -> Option<Card> {
        if self.pending_delete.is_some() {
            return Some(Card::Deleting);
        }

        if self.asking.is_some() {
            return Some(Card::Destinations);
        }

        if matches!(self.leaving, cull::Leaving::Asking(..)) {
            return Some(Card::TheBin);
        }

        if self.pending_history.is_some() {
            return Some(Card::TheHistory);
        }

        if self.conflict_visible {
            return Some(Card::TheConfiguration);
        }

        None
    }

    /// What is on screen: the question if there is one, else the deck's card.
    pub(super) fn showing(&self) -> Option<Card> {
        self.asked().or_else(|| self.deck.showing())
    }

    /// What a card is called, as its bar says it: the kind, then which one.
    fn named(&self, card: Card) -> (Option<&'static str>, String) {
        match card {
            Card::Settings => (None, "Settings".to_string()),
            Card::Keyboard => (None, "Keyboard".to_string()),
            Card::OneKey => (
                Some("Keys for"),
                keys::one::about(&self.keys).unwrap_or_else(|| "a command".to_string()),
            ),
            Card::CheatSheet => (Some("Keys"), self.mode.label().to_string()),
            Card::Legend => (None, "What the marks mean".to_string()),
            Card::Placeholders => (None, "Template placeholders".to_string()),
            Card::Messages => (None, "Recent messages".to_string()),
            Card::About => (None, "About avis-imgv".to_string()),
            Card::Deleting => (
                None,
                self.pending_delete
                    .as_ref()
                    .map_or("Delete", |pending| pending.sends.title())
                    .to_string(),
            ),
            Card::Destinations => (
                None,
                self.asking.as_ref().map_or_else(
                    || "Send to…".to_string(),
                    |asking| format!("{} to…", asking.errand.verb()),
                ),
            ),
            Card::TheBin => (None, "The bin is not empty".to_string()),
            Card::TheHistory => (None, "Undo".to_string()),
            Card::TheConfiguration => (None, "The configuration file has changed".to_string()),
        }
    }

    /// Draws whatever is up: the page, and then the question over it.
    ///
    /// Two draws rather than one, because both can be true at once — writing a
    /// setting is what asks about a configuration file edited underneath, and
    /// the page it was asked from is the right thing to have behind it. The
    /// question is drawn second, which is what puts it in front: `deck::show`
    /// raises the card it draws.
    pub(super) fn show_deck(&mut self, ctx: &egui::Context) {
        if let Some(page) = self.deck.showing() {
            self.draw_card(ctx, page);
        }

        if let Some(question) = self.asked() {
            self.draw_card(ctx, question);
        }

        // The one card that carries something of its own. The deck says
        // whether it is up and the editor's state follows it, rather than the
        // two telling each other: a card put down by the bar, by Escape or by
        // the switcher would otherwise leave the editor armed.
        match (self.deck.holds(Card::OneKey), self.keys.editing_one()) {
            (true, false) => self.deck.close(Card::OneKey),
            (false, true) => self.keys.close_one(),
            _ => {}
        }
    }

    fn draw_card(&mut self, ctx: &egui::Context, card: Card) {
        // Read off the deck before it is handed to the closure that may change
        // it: the card's own contents can open another, and the bar drawn this
        // frame is the deck as it stood when the frame began.
        let crumbs: Vec<String> = if card.asks() {
            Vec::new()
        } else {
            self.deck
                .crumbs()
                .to_vec()
                .into_iter()
                .map(|under| self.named(under).1)
                .collect()
        };

        let mut open = [false; Card::EVERY.len()];
        for (spot, wanted) in Card::EVERY.iter().enumerate() {
            open[spot] = self.deck.holds(*wanted);
        }

        let (kind, title) = self.named(card);
        let face = Face {
            id: egui::Id::new(card),
            kind,
            title: &title,
            crumbs: &crumbs,
            spread: card.spread(),
            shut: !card.asks(),
        };

        let mut go = None;
        let ask = deck::show(
            ctx,
            &face,
            |ui| go = switcher(ui, &open, card),
            |ui| self.contents(ui, card),
        );

        if let Some(wanted) = go {
            self.deck.go(wanted);
        }

        match ask {
            Some(Ask::Crumb(depth)) => self.deck.back_to(depth),
            Some(Ask::Shut) => self.deck.shut(),
            None => {}
        }
    }

    /// Puts a card on screen. Everything that opens one comes through here.
    pub(super) fn open_card(&mut self, card: Card) {
        if card == Card::CheatSheet {
            self.cheat_sheet_opened = true;
        }

        self.deck.go(card);
    }

    /// `Escape` takes the card on screen off.
    ///
    /// Not a question's: `Escape` is an *answer* there — leave them alone,
    /// stay open, leave it — and the card that asked reads it itself. Nor an
    /// armed row's, where it means "leave the binding as it was" rather than
    /// "put the card down".
    pub(super) fn escape_goes_back(&mut self, ctx: &egui::Context) {
        if self.keys.is_listening() || self.asked().is_some() || self.deck.is_empty() {
            return;
        }

        if input::escape_takes_a_card_off(ctx, self.was_typing) {
            self.deck.back();
        }
    }
}

/// The way to any other card, on the right of the bar.
///
/// The deck is what makes this affordable: with windows there was no one place
/// that knew which of them were open, so the way from the settings to the keys
/// was out of one window and back in through a menu. A tick says which are
/// open, because a person who left the settings to look something up wants to
/// know they can get back to it.
fn switcher(ui: &mut egui::Ui, open: &[bool; Card::EVERY.len()], showing: Card) -> Option<Card> {
    let mut go = None;

    ui.menu_button("Go to…", |ui| {
        for (spot, card) in Card::EVERY.iter().enumerate() {
            let label = match card {
                Card::Settings => "Settings",
                Card::Keyboard => "Keyboard",
                Card::CheatSheet => "The keys, as configured",
                Card::Legend => "What the marks mean",
                Card::Placeholders => "Template placeholders",
                Card::Messages => "Recent messages",
                Card::About => "About avis-imgv",
                _ => continue,
            };

            let mut on = open[spot];
            if ui
                .checkbox(&mut on, label)
                .on_hover_text(if *card == showing {
                    "The card you are on"
                } else if open[spot] {
                    "Open, under this one"
                } else {
                    "Not open yet"
                })
                .clicked()
            {
                go = Some(*card);
                ui.close();
            }
        }
    });

    go
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A question is derived from the state that asked it, so it is never put
    /// on the deck — a stacked question would outlive its answer.
    #[test]
    fn no_card_a_person_opens_is_a_question() {
        for card in Card::EVERY {
            assert!(!card.asks(), "{card:?} is opened by hand and is a question");
        }
    }

    /// A question is a plate, so the photographs it is about stay visible; a
    /// page takes the window, because what is behind it is no part of it.
    #[test]
    fn a_question_is_a_plate_and_a_page_is_the_window() {
        for card in Card::EVERY {
            assert_eq!(card.spread(), Spread::Full, "{card:?}");
        }

        for card in [
            Card::Deleting,
            Card::Destinations,
            Card::TheBin,
            Card::TheHistory,
            Card::TheConfiguration,
        ] {
            assert!(card.asks(), "{card:?}");
            assert!(matches!(card.spread(), Spread::Plate(_)), "{card:?}");
        }
    }

    /// The switcher lists what a person can open with nothing else to say.
    ///
    /// Not the card for one command, which is about a command and has to be
    /// told which; not the questions, which arrive rather than being asked
    /// for and would be a row offering to delete something.
    #[test]
    fn the_switcher_lists_every_page_and_nothing_else() {
        assert_eq!(Card::EVERY.len(), 7);
        assert!(!Card::EVERY.contains(&Card::OneKey));

        // No card is listed twice: the switcher's ticks are read by index into
        // this, and a card at two indices is a tick that lies at one of them.
        for (spot, card) in Card::EVERY.iter().enumerate() {
            assert!(!Card::EVERY[spot + 1..].contains(card), "{card:?} twice");
        }
    }

    /// Only a page carries a cross. A question's answers are its way out, and
    /// a cross beside "Yes" and "Leave them alone" is a third answer nobody
    /// wrote.
    #[test]
    fn a_question_carries_no_cross() {
        for card in [Card::Deleting, Card::TheBin, Card::TheConfiguration] {
            assert!(card.asks());
        }

        for card in Card::EVERY.iter().chain(std::iter::once(&Card::OneKey)) {
            assert!(!card.asks(), "{card:?}");
        }
    }
}
