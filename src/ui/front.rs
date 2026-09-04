//! Who owns the mouse and the keyboard.
//!
//! A card in front takes both, and the answer is decided once a frame in
//! `App::something_is_in_front` and written here — a card that sets and clears
//! a flag of its own is a card that clears it while another still needs it.
//!
//! In egui's memory rather than in a static of this program's, deliberately:
//! the question is asked per context, and one window's answer is not another's.

use eframe::egui::{self, Id, Response};

pub fn textedit_move_cursor_to_end(resp: &Response, ui: &mut egui::Ui, len: usize) {
    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
        let ccursor = egui::text::CCursor::new(len);
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
        state.store(ui.ctx(), resp.id);
        resp.request_focus();
        ui.ctx().memory_mut(|m| m.request_focus(resp.id))
    }
}

/// Says that something of the viewer's own is in front of the photograph, and
/// so owns the keyboard: a card of the deck, or one of the two overlays.
///
/// Written once a frame by [`crate::app::App`] rather than by each of them,
/// because the answer is "is any of them up" and a flag that several owners
/// set and clear is a flag one of them clears while another still needs it.
pub fn set_in_front(ctx: &egui::Context, in_front: bool) {
    ctx.memory_mut(|mem| {
        mem.data.insert_temp::<bool>(in_front_id(), in_front);
    })
}

/// Whether something of the viewer's own is in front of the photograph.
///
/// Not the same question as [`are_inputs_muted`], which a focused text field
/// also answers yes to: typing in the filter bar takes the keys and leaves the
/// mouse alone, while a card in front takes both.
pub fn is_in_front(ctx: &egui::Context) -> bool {
    ctx.memory_mut(|mem| mem.data.get_temp::<bool>(in_front_id()).unwrap_or(false))
}

/// Puts a layer in front of the viewer: while it is drawn, everything behind
/// it stops answering the pointer.
///
/// `Memory::set_modal_layer` is egui's own way of saying it, and it reaches
/// two things this program needs: `Context::rect_contains_pointer` answers
/// `false` for every layer below, which is what every scroll area in the
/// viewer asks before it reads the wheel, and no widget below can take the
/// keyboard focus.
///
/// It does not reach `Response::contains_pointer`, which is decided by a hit
/// test that knows nothing about modal layers, so the few places that read the
/// pointer for themselves ask [`is_in_front`] instead — the wheel and
/// the drag on the photograph, a cell in the contact sheet, the strip.
///
/// Called on every frame the layer is drawn, because the flag egui keeps is
/// this frame's and is promoted at the end of it.
///
/// The deck has its own copy of this that raises the layer as well —
/// `ui::deck::draw` — because two cards can be on screen at once and the one
/// drawn last is the one being answered. This one is what the overlays use.
pub fn in_front<R>(ctx: &egui::Context, shown: Option<&egui::InnerResponse<R>>) {
    let Some(shown) = shown else {
        return;
    };

    ctx.memory_mut(|memory| memory.set_modal_layer(shown.response.layer_id));
}

/// Takes the keyboard back from whatever widget has it.
///
/// egui hands focus to the next widget on Tab and keeps it there, and a text
/// field with focus mutes every shortcut in the viewer. That is right while
/// somebody is typing a path and wrong the instant they are not: Tab means
/// "the next pane" here, and Escape means "give me the keyboard back".
pub fn surrender_focus(ctx: &eframe::egui::Context) {
    ctx.memory_mut(|memory| {
        if let Some(id) = memory.focused() {
            memory.surrender_focus(id);
        }
    });
}

/// Whether the viewer should keep its keys to itself this frame.
///
/// Two reasons, and they are different sizes: a card in front owns the mouse
/// and the keyboard both, while a focused text field owns only the keyboard.
pub fn are_inputs_muted(ctx: &egui::Context) -> bool {
    is_in_front(ctx) || ctx.memory(|mem| mem.focused().is_some())
}

fn in_front_id() -> Id {
    Id::new("something is in front")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One frame with a layer over part of the screen and the viewer's own
    /// panel under all of it, and the pointer beside that layer rather than on
    /// it.
    ///
    /// A `Window` here because it is the shortest thing to draw that covers
    /// part of the screen; the program's own cards cover all of it, and what
    /// is being tested is what `in_front` does to the layer beneath either.
    /// Returns whether the layer the photograph is drawn in still answers the
    /// pointer, which is the question every scroll area and every hover in the
    /// program below it asks.
    fn the_layer_behind_answers(ctx: &egui::Context, claim: bool) -> bool {
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 800.0));
        let input = egui::RawInput {
            screen_rect: Some(screen),
            events: vec![egui::Event::PointerMoved(egui::pos2(900.0, 700.0))],
            ..Default::default()
        };

        let mut answered = false;
        let _ = ctx.run(input, |ctx| {
            let shown = egui::Window::new("Settings")
                .fixed_pos([100.0, 100.0])
                .fixed_size([600.0, 500.0])
                .show(ctx, |ui| {
                    ui.label("a setting");
                });

            if claim {
                in_front(ctx, shown.as_ref());
            }

            answered = ctx.rect_contains_pointer(egui::LayerId::background(), screen);
        });

        answered
    }

    /// The fault this is about. A wheel notch spent on the settings used to
    /// reach whatever was behind it, because what was in front covered part of
    /// the screen and the pointer is often on the other part.
    #[test]
    fn a_card_in_front_takes_the_pointer_from_the_layer_behind_it() {
        let ctx = egui::Context::default();

        // Twice: egui promotes the flag at the end of the frame it is written
        // in, so the second frame is the one that has to be quiet.
        the_layer_behind_answers(&ctx, true);
        assert!(!the_layer_behind_answers(&ctx, true));
    }

    /// And with nothing in front, the photograph is a surface again.
    #[test]
    fn nothing_in_front_leaves_the_pointer_alone() {
        let ctx = egui::Context::default();

        the_layer_behind_answers(&ctx, false);
        assert!(the_layer_behind_answers(&ctx, false));
    }

    /// Something in front mutes the keys; nothing in front does not.
    #[test]
    fn a_card_in_front_mutes_the_keys() {
        let ctx = egui::Context::default();
        assert!(!are_inputs_muted(&ctx));

        set_in_front(&ctx, true);
        assert!(is_in_front(&ctx));
        assert!(are_inputs_muted(&ctx));

        set_in_front(&ctx, false);
        assert!(!are_inputs_muted(&ctx));
    }
}
