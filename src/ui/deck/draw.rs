//! Drawing one card: the bar that says where you are, and the room under it.
//!
//! Two shapes, and the difference is what the card is *about*. A page — the
//! settings, the keys, a reference — is about itself, so it takes the whole
//! window and paints over everything: what is behind it is no part of the
//! question and looking at it is a distraction. A question about the
//! photograph on screen is the other case: a plate over the rest dimmed,
//! because "send these three to the bin" cannot be answered by somebody who
//! can no longer see them.
//!
//! Neither is an `egui::Window`. There is no title bar to drag, no corner to
//! pull, no second one behind the first and no position to remember: one card
//! is on screen, and the bar says how to get back to the last.

use eframe::egui::{self, RichText};

/// How much of the window a card takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spread {
    /// All of it, under whatever panels sit above the deck.
    Full,
    /// A plate this wide, over the rest of the window dimmed.
    Plate(f32),
}

/// What a card says for itself, and what its bar carries.
pub struct Face<'a> {
    /// Its own place in egui's memory. Two cards can be on screen at once — a
    /// question over a page — and they must not share one.
    pub id: egui::Id,
    /// The kind of thing this card is about, in the weak colour: *Keys for*.
    ///
    /// The same rule as a menu's first row. A card is opened from a dozen
    /// surfaces and the answer is different from each, so it says what it was
    /// asked about rather than leaving it to be worked out.
    pub kind: Option<&'a str>,
    /// Which one of them, in the strong colour: **Zoom in**.
    pub title: &'a str,
    /// The cards under this one, oldest first: the way back, as it is written.
    pub crumbs: &'a [String],
    pub spread: Spread,
    /// Whether the bar carries a cross.
    ///
    /// A question does not: its own answers are the way out of it, and a cross
    /// beside "Yes" and "Leave them alone" is a third answer nobody wrote.
    pub shut: bool,
}

/// What the bar was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// Back to the crumb at this depth, counting from nought.
    Crumb(usize),
    /// Every card off; back to what the program is for.
    Shut,
}

/// The glyphs the bar draws.
///
/// A glyph is only as available as the fonts actually loaded, so these are
/// tested against the fonts this program ships rather than chosen by eye. The
/// arrows are the reason there is a test: `←` U+2190 is in none of the five
/// fonts loaded here — not the bundled typeface, not Ubuntu-Light, not either
/// emoji font — and drew an empty box, so the way back is the crumb it names
/// rather than a symbol.
const CHEVRON: &str = "›";
const CROSS: &str = "✖";

/// How dark the rest of the window goes behind a plate.
///
/// Dark enough that the plate is plainly the thing being answered, light
/// enough that the photograph the question is about can still be seen. Black
/// rather than a colour from the palette, so it dims under both themes rather
/// than brightening under one of them.
const SCRIM: egui::Color32 = egui::Color32::from_black_alpha(170);

/// The room left round the edge of a card.
const MARGIN: i8 = 12;

/// Draws the card, and returns whatever its bar asked for.
///
/// `bar` is the right-hand end of that bar, and it is the caller's: the deck
/// knows how many cards are open but not what else there is to go to, and a
/// switcher built in here would want the whole list handed to it.
pub fn show(
    ctx: &egui::Context,
    face: &Face<'_>,
    bar: impl FnOnce(&mut egui::Ui),
    add: impl FnOnce(&mut egui::Ui),
) -> Option<Ask> {
    match face.spread {
        Spread::Full => page(ctx, face, bar, add),
        Spread::Plate(width) => plate(ctx, face, width, add),
    }
}

/// A card over the whole of the room the deck was given.
fn page(
    ctx: &egui::Context,
    face: &Face<'_>,
    bar: impl FnOnce(&mut egui::Ui),
    add: impl FnOnce(&mut egui::Ui),
) -> Option<Ask> {
    let over = ctx.available_rect();
    let mut ask = None;

    let shown = egui::Area::new(face.id)
        .order(egui::Order::Middle)
        .fixed_pos(over.min)
        .constrain_to(over)
        .show(ctx, |ui| {
            ui.set_clip_rect(over);
            ui.set_min_size(over.size());
            ui.set_max_size(over.size());

            // Painted before anything is added, so it lands behind the card's
            // own contents rather than over them: within one layer egui paints
            // in the order it was told to.
            ui.painter().rect_filled(over, 0.0, ui.visuals().panel_fill);

            egui::Frame::new()
                .inner_margin(egui::Margin::same(MARGIN))
                .show(ui, |ui| {
                    ask = heading(ui, face, bar);
                    ui.separator();

                    // The full width of the card, not a centred measure. A
                    // measure was tried and taken out again: a row of this
                    // program's is a horizontal layout of labels that do not
                    // wrap, so bounding the width does not shorten the line —
                    // it only pushes the start of it inwards until the end
                    // falls off the right of the window.
                    add(ui);
                });
        });

    in_front(ctx, &shown);
    ask
}

/// A card as a plate, over the rest of the window dimmed.
fn plate(
    ctx: &egui::Context,
    face: &Face<'_>,
    width: f32,
    add: impl FnOnce(&mut egui::Ui),
) -> Option<Ask> {
    let over = ctx.available_rect();

    let shown = egui::Area::new(face.id)
        .order(egui::Order::Middle)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            // Painted from the plate's own layer rather than from an area of
            // its own: one area is one thing to raise and one thing to make
            // modal, and a scrim that could end up above the plate is a scrim
            // nobody can click through to answer.
            ui.painter()
                .with_clip_rect(over)
                .rect_filled(over, 0.0, SCRIM);

            egui::Frame::window(ui.style()).show(ui, |ui| {
                ui.set_max_width(width);

                if let Some(kind) = face.kind {
                    ui.label(RichText::new(kind).weak());
                }

                ui.label(RichText::new(face.title).heading());
                ui.add_space(8.0);
                add(ui);
            });
        });

    in_front(ctx, &shown);
    None
}

/// The bar: the way back, where you are, and whatever the caller put beside it.
fn heading(ui: &mut egui::Ui, face: &Face<'_>, bar: impl FnOnce(&mut egui::Ui)) -> Option<Ask> {
    let mut ask = None;

    ui.horizontal(|ui| {
        // Framed rather than written in the weak colour, because a crumb is
        // the way back and a word that looks like a caption is not read as
        // one. It is also the only way back the bar draws: the arrow that
        // would have said so has no glyph in any font the program loads.
        for (depth, crumb) in face.crumbs.iter().enumerate() {
            if ui
                .small_button(crumb)
                .on_hover_text("Back to this card")
                .clicked()
            {
                ask = Some(Ask::Crumb(depth));
            }

            ui.label(RichText::new(CHEVRON).weak());
        }

        if let Some(kind) = face.kind {
            ui.label(RichText::new(kind).weak());
        }

        ui.label(RichText::new(face.title).heading());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if face.shut
                && ui
                    .button(CROSS)
                    .on_hover_text("Back to the photographs")
                    .clicked()
            {
                ask = Some(Ask::Shut);
            }

            bar(ui);
        });
    });

    ask
}

/// Puts the card in front: nothing behind it is clicked, dragged or scrolled.
///
/// Raised as well as made modal, because two cards can be on screen at once —
/// a question over a page — and the one drawn last is the one being answered.
/// An `Area` otherwise keeps the place it had in its order, and egui raises one
/// only when it is *new*.
fn in_front<R>(ctx: &egui::Context, shown: &egui::InnerResponse<R>) {
    let layer = shown.response.layer_id;

    ctx.memory_mut(|memory| {
        memory.areas_mut().move_to_top(layer);
        memory.set_modal_layer(layer);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every glyph the bar draws, against the fonts this program actually
    /// loads. `✕` was in neither of them and drew an empty box.
    #[test]
    fn every_glyph_the_bar_draws_is_in_the_fonts() {
        let ctx = egui::Context::default();
        #[cfg(feature = "custom_font")]
        crate::ui::theme::apply_fonts(&ctx);

        ctx.begin_pass(egui::RawInput::default());

        let font = egui::FontId::proportional(14.0);
        for glyph in [CHEVRON, CROSS] {
            assert!(
                ctx.fonts_mut(|fonts| fonts.has_glyphs(&font, glyph)),
                "{glyph} is not in the fonts the program loads"
            );
        }

        let _ = ctx.end_pass();
    }

    /// A card with nothing under it offers no way back; one that was opened
    /// from another names the card it came from.
    #[test]
    fn the_way_back_is_drawn_only_when_there_is_one() {
        assert!(!drawn(&[]).contains(&CHEVRON.to_string()));

        let from_a_card = drawn(&["Keyboard".to_string()]);
        assert!(from_a_card.contains(&"Keyboard".to_string()));
        assert!(from_a_card.contains(&CHEVRON.to_string()));
    }

    /// A card says what it was asked about, in both halves.
    #[test]
    fn a_card_says_the_kind_and_which_one() {
        let said = drawn(&[]);

        assert!(said.contains(&"Keys for".to_string()));
        assert!(said.contains(&"Zoom in".to_string()));
        assert!(said.contains(&"what the card holds".to_string()));
    }

    /// Twice, and the second frame is the one read.
    ///
    /// An `Area` whose size egui does not yet know is laid out in a sizing
    /// pass that paints nothing, so the first frame of any card is empty. The
    /// program never sees it — a card is up for as long as somebody is reading
    /// it — but a test that ran one frame would find nothing at all drawn.
    fn drawn(crumbs: &[String]) -> Vec<String> {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 600.0),
            )),
            ..Default::default()
        };

        let frame = || {
            ctx.run(input.clone(), |ctx| {
                let face = Face {
                    id: egui::Id::new("a card under test"),
                    kind: Some("Keys for"),
                    title: "Zoom in",
                    crumbs,
                    spread: Spread::Full,
                    shut: true,
                };

                show(
                    ctx,
                    &face,
                    |_| {},
                    |ui| {
                        ui.label("what the card holds");
                    },
                );
            })
        };

        let _ = frame();
        crate::ui::drawn::text(&frame())
    }
}
