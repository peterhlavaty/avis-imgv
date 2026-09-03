//! What the view does with the marking: reads the pointer for it, draws it,
//! and carries out the two verbs it offers.
//!
//! The four files beside this one know nothing about a view — they are a
//! rectangle, eight grips, a state machine and a painter — and this is the one
//! place they meet the store, the viewport and the configuration.

use eframe::egui::{self, PointerButton, Rect};
use eframe::epaint::Pos2;

use crate::config::mouse::{DragButton, MarkArea};
use crate::ui::menus::Verb;

use super::super::bottom_bar::BarAction;
use super::super::ImageView;
use super::draw::{self, Chosen};
use super::pointer::Pointing;

impl ImageView {
    /// Reads the left button for the marking, draws it, and answers its menu.
    ///
    /// `panel` is the whole of the central panel rather than the picture in
    /// it: the grey beside a letterboxed photograph is still the photograph's
    /// surface, and a click there is a click outside the marking rather than a
    /// click on something else.
    pub(in crate::view::image_view) fn handle_area(&mut self, ctx: &egui::Context, panel: Rect) {
        // While a window of the viewer's own is up the photograph is a picture
        // rather than a surface, which is the same rule the wheel and the drag
        // already follow.
        if crate::utils::is_in_front(ctx) {
            return;
        }

        let pointing = read(ctx, panel);
        let may_mark = self.may_mark();
        let answered = self.area.look(&self.metrics, &pointing, may_mark);

        if answered.zoom_to_it {
            self.zoom_to_marked_area();
        }

        if let Some(cursor) = answered.cursor {
            ctx.set_cursor_icon(cursor);
        }

        let Some(on_screen) = self.area.on_screen(&self.metrics) else {
            return;
        };

        // Out of a hundred, because that is what a person setting it is
        // thinking in; the painter wants an alpha.
        let dim = (self.config.marked_area_dim.min(100) as f32 * 2.55).round() as u8;

        match draw::show(ctx, on_screen, self.metrics.rect, dim) {
            None => {}
            Some(Chosen::ZoomToIt) => self.zoom_to_marked_area(),
            Some(Chosen::Copy) => self.copy_the_marked_area(),
            Some(Chosen::Clear) => self.area.clear(),
            Some(Chosen::BindKey(path)) => self.bar_actions.push(BarAction::BindKey(path)),
            Some(Chosen::Settings(path)) => self.bar_actions.push(BarAction::Settings(path)),
        }
    }

    /// Magnifies until the marking fills the panel.
    ///
    /// Both halves of the viewport at once, because a zoom without the pan
    /// that goes with it shows the middle of the photograph rather than the
    /// part of it that was asked for.
    pub(in crate::view::image_view) fn zoom_to_marked_area(&mut self) {
        let Some(marked) = self.area.marked() else {
            return;
        };

        let Some((zoom, pan)) = super::zoom_to(&self.metrics, marked) else {
            return;
        };

        self.viewport.zoom = zoom;
        self.viewport.pan = pan;
        // What a photograph opens at is applied once per photograph and would
        // otherwise undo this on the very next frame.
        self.viewport.opened = true;
    }

    /// Asks whoever has a decoder to put it on the clipboard.
    ///
    /// The same verb the whole photograph goes by: it is the one that owns the
    /// full size decode, and it reads the marking back from here.
    pub(in crate::view::image_view) fn copy_the_marked_area(&mut self) {
        if let Some(path) = self.active_path() {
            self.verb = Some((Verb::CopyPicture, path));
        }
    }

    /// The marking, in the photograph's own coordinates, nought to one.
    pub fn marked_area(&self) -> Option<Rect> {
        self.area.marked()
    }

    /// Whether there is one, which is what decides whose menu `Shift + F10`
    /// asks for.
    pub fn has_marked_area(&self) -> bool {
        self.area.marked().is_some()
    }

    /// Whether the second button belongs to the marking rather than to the
    /// photograph underneath it.
    pub(in crate::view::image_view) fn pointer_is_on_the_marking(
        &self,
        ctx: &egui::Context,
    ) -> bool {
        let (Some(on_screen), Some(at)) = (
            self.area.on_screen(&self.metrics),
            ctx.input(|i| i.pointer.latest_pos()),
        ) else {
            return false;
        };

        draw::covers(on_screen, at)
    }

    /// Whether the left button is free to draw a new marking.
    ///
    /// One press is one gesture: it is never free while the same drag would be
    /// moving the photograph instead, and the setting decides the rest.
    fn may_mark(&self) -> bool {
        match self.mouse.mark_area {
            MarkArea::Never => false,
            MarkArea::Always => true,
            MarkArea::WhenItFits => !self.left_drag_pans(),
        }
    }

    /// Whether a left drag would move the photograph right now.
    ///
    /// Both halves matter: the button has to be the one panning, and there has
    /// to be slack for it to pan into. With the whole photograph on screen the
    /// canvas clamps every pan to nothing, so the gesture was doing nothing at
    /// all before this.
    fn left_drag_pans(&self) -> bool {
        if !matches!(self.mouse.drag, DragButton::Left | DragButton::Any) {
            return false;
        }

        let scaled = self.metrics.scaled(self.viewport.zoom);

        scaled.x > self.metrics.available_size.x + 0.5
            || scaled.y > self.metrics.available_size.y + 0.5
    }
}

/// What the pointer did this frame, as the state machine wants it.
fn read(ctx: &egui::Context, panel: Rect) -> Pointing {
    let mut pointing = ctx.input(|i| Pointing {
        at: i.pointer.latest_pos().unwrap_or(Pos2::ZERO),
        pressed: i.pointer.button_pressed(PointerButton::Primary),
        down: i.pointer.button_down(PointerButton::Primary),
        released: i.pointer.button_released(PointerButton::Primary),
        dragging: i.pointer.is_decidedly_dragging(),
        clicked: i.pointer.button_clicked(PointerButton::Primary),
    });

    // A press or a click that landed on a panel, on the filmstrip or on the
    // bottom bar belongs to whatever is drawn there, and so does one that
    // landed on a menu drawn over the photograph — including the marking's
    // own, whose rows would otherwise be clicked *and* fall through to the
    // marking underneath, so that copying it also magnified to it.
    //
    // A drag already under way is not asked: letting go of a side beyond the
    // edge of the panel is how a side gets dragged to the edge of the panel.
    if !panel.contains(pointing.at) || over_something_else(ctx, pointing.at) {
        pointing.pressed = false;
        pointing.clicked = false;
    }

    pointing
}

/// Whether something is drawn over the photograph where the pointer is.
///
/// Not `Context::is_pointer_over_area`, which is what the gestures use and is
/// no good here: it answers for the background layer by asking whether the
/// point is in the *unused* rectangle, and by the time the marking is read the
/// central panel has consumed all of it — so it is true everywhere, and every
/// press would be somebody else's. The honest question is whether any layer
/// above the background is under the pointer, which is a menu, a popup, a
/// window or a tooltip and never a panel.
fn over_something_else(ctx: &egui::Context, at: Pos2) -> bool {
    ctx.layer_id_at(at)
        .is_some_and(|layer| layer.order != egui::Order::Background)
}
