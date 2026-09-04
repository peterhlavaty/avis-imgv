//! Chrome around the images: the cards, the panels, the overlays and the theme.
//!
//! [`deck`] is the one everything else is drawn on. The program opens no
//! windows of its own — a card fills the viewer, one is on screen at a time,
//! and `app::cards` says which this program has.

pub mod cheat_sheet;
pub mod checks;
pub mod deck;
pub mod destinations;
pub mod dragged;
#[cfg(test)]
pub mod drawn;
pub mod empty;
pub mod filter_bar;
pub mod front;
pub mod histogram;
pub mod keys;
pub mod legend;
pub mod menus;
pub mod navigator;
pub mod notice;
pub mod panel;
pub mod perf_metrics;
pub mod placeholders;
pub mod progress;
pub mod settings;
pub mod slider;
pub mod surface;
pub mod tag_panel;
pub mod theme;
pub mod tree;
