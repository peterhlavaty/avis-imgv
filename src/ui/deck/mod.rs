//! A deck of cards: one page of chrome at a time, and the way back to the rest.
//!
//! Nothing here knows what this program's cards are. A deck is a stack of
//! whatever identifier the caller uses — an enum, an index, a string — and the
//! rules it keeps are three:
//!
//! - one card is on screen, and it is the last one put down;
//! - a card opened while it is already open is *raised* rather than opened
//!   twice, so the same page never appears at two depths and the way back
//!   cannot run in a circle;
//! - taking a card off takes everything that was put on top of it as well,
//!   because the cards above it were reached *through* it.
//!
//! That is the whole of the state. [`draw`] is the other half: an opaque page
//! over the window with a bar saying where you are, or a plate over the rest
//! dimmed, for the cards that are a question about what is behind them.

mod draw;

pub use draw::{show, Ask, Face, Spread};

/// The cards that are open, the last of them on screen.
#[derive(Debug, Clone)]
pub struct Deck<C> {
    stack: Vec<C>,
}

impl<C> Default for Deck<C> {
    fn default() -> Self {
        Self { stack: Vec::new() }
    }
}

impl<C: Copy + PartialEq> Deck<C> {
    /// The card on screen, if any.
    pub fn showing(&self) -> Option<C> {
        self.stack.last().copied()
    }

    /// Whether anything at all is up.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// How deep the card on screen is, counting from one.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Whether this card is open, on screen or under something.
    pub fn holds(&self, card: C) -> bool {
        self.stack.contains(&card)
    }

    /// The cards under the one on screen, oldest first: the way back.
    pub fn crumbs(&self) -> &[C] {
        let depth = self.stack.len();
        &self.stack[..depth.saturating_sub(1)]
    }

    /// Puts a card on screen.
    ///
    /// A card already in the deck is raised to the top rather than put down a
    /// second time, and everything that was above it comes off with the move:
    /// those cards were reached through this one, and leaving them under it
    /// would make the way back a circle.
    pub fn go(&mut self, card: C) {
        if let Some(depth) = self.stack.iter().position(|open| *open == card) {
            self.stack.truncate(depth + 1);
            return;
        }

        self.stack.push(card);
    }

    /// Takes the card on screen off, and says which it was.
    pub fn back(&mut self) -> Option<C> {
        self.stack.pop()
    }

    /// Back to the card at that depth, counting from nought.
    ///
    /// A depth the deck does not have is left alone rather than clamped: the
    /// ask came from a bar drawn a frame ago, and obeying a stale one by
    /// guessing which card was meant is worse than ignoring it.
    pub fn back_to(&mut self, depth: usize) {
        if depth < self.stack.len() {
            self.stack.truncate(depth + 1);
        }
    }

    /// Takes one card off wherever it is, and everything above it.
    pub fn close(&mut self, card: C) {
        if let Some(depth) = self.stack.iter().position(|open| *open == card) {
            self.stack.truncate(depth);
        }
    }

    /// Takes every card off.
    pub fn shut(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Page {
        A,
        B,
        C,
    }

    #[test]
    fn an_empty_deck_shows_nothing_and_has_nowhere_to_go_back_to() {
        let mut deck: Deck<Page> = Deck::default();

        assert!(deck.is_empty());
        assert_eq!(deck.showing(), None);
        assert_eq!(deck.back(), None);
        assert!(deck.crumbs().is_empty());
    }

    #[test]
    fn the_card_on_screen_is_the_last_one_put_down() {
        let mut deck = Deck::default();
        deck.go(Page::A);
        deck.go(Page::B);

        assert_eq!(deck.showing(), Some(Page::B));
        assert_eq!(deck.crumbs(), [Page::A]);
        assert_eq!(deck.depth(), 2);
    }

    /// The rule that keeps the way back from running in a circle.
    #[test]
    fn a_card_already_open_is_raised_rather_than_opened_twice() {
        let mut deck = Deck::default();
        deck.go(Page::A);
        deck.go(Page::B);
        deck.go(Page::C);

        deck.go(Page::A);

        assert_eq!(deck.showing(), Some(Page::A));
        assert_eq!(deck.depth(), 1);
        assert!(!deck.holds(Page::B));
        assert!(!deck.holds(Page::C));
    }

    #[test]
    fn going_back_returns_the_card_that_came_off() {
        let mut deck = Deck::default();
        deck.go(Page::A);
        deck.go(Page::B);

        assert_eq!(deck.back(), Some(Page::B));
        assert_eq!(deck.showing(), Some(Page::A));
    }

    #[test]
    fn a_crumb_takes_everything_above_it_off() {
        let mut deck = Deck::default();
        deck.go(Page::A);
        deck.go(Page::B);
        deck.go(Page::C);

        deck.back_to(0);

        assert_eq!(deck.showing(), Some(Page::A));
        assert_eq!(deck.depth(), 1);
    }

    /// The ask came from a bar drawn before the deck changed under it.
    #[test]
    fn a_depth_the_deck_does_not_have_is_ignored() {
        let mut deck = Deck::default();
        deck.go(Page::A);

        deck.back_to(7);

        assert_eq!(deck.showing(), Some(Page::A));
    }

    #[test]
    fn closing_a_card_underneath_takes_what_was_above_it_too() {
        let mut deck = Deck::default();
        deck.go(Page::A);
        deck.go(Page::B);
        deck.go(Page::C);

        deck.close(Page::B);

        assert_eq!(deck.showing(), Some(Page::A));
        assert!(!deck.holds(Page::C));
    }

    #[test]
    fn closing_a_card_that_is_not_open_changes_nothing() {
        let mut deck = Deck::default();
        deck.go(Page::A);

        deck.close(Page::C);

        assert_eq!(deck.showing(), Some(Page::A));
    }

    #[test]
    fn shutting_takes_every_card_off() {
        let mut deck = Deck::default();
        deck.go(Page::A);
        deck.go(Page::B);

        deck.shut();

        assert!(deck.is_empty());
        assert_eq!(deck.showing(), None);
    }
}
