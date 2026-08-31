//! What the pointer, the wheel and the held keys do to the image on screen.
//!
//! Kept apart from the view itself because it is all one subject: turning a
//! gesture into a movement of the viewport, and a click into whatever the user
//! configured it to run.

use eframe::egui::{self, PointerButton, Response};
use eframe::epaint::Vec2;

use crate::actions::{self, Callback};

use super::{input, ImageView};

impl ImageView {
    pub(super) fn handle_pointer(&mut self, ctx: &egui::Context, response: &Response) {
        let hovered = response.contains_pointer();

        if self.config.scroll_navigation {
            if let Some(command) = input::scroll_navigation(ctx, hovered) {
                self.apply(command, ctx);
            }
        }

        // Through the same command as the keys, so a pinch holds the point
        // under the fingers rather than the middle of the panel.
        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if zoom_delta != 1.0 {
            self.apply(input::Command::ZoomBy(zoom_delta), ctx);
        }

        let keyboard = self.keyboard_panning(ctx);

        if !hovered {
            // Losing the pointer mid-scroll would otherwise leave the last
            // delta applied every frame.
            self.viewport.scroll_delta = keyboard;
            return;
        }

        let mut delta = ctx.input(|i| i.smooth_scroll_delta);
        // Named, because `is_decidedly_dragging` answers for every button: a
        // right-button drag used to pan the photograph and then release into
        // whatever menu was registered on the panel.
        if ctx.input(|i| {
            i.pointer.is_decidedly_dragging() && i.pointer.button_down(PointerButton::Primary)
        }) {
            delta += ctx.input(|i| i.pointer.delta()) * ctx.pixels_per_point();
        }

        self.viewport.scroll_delta = delta + keyboard;
    }

    /// The pan asked for by the keys held this frame.
    ///
    /// Nothing is asked for while the whole image is on screen, so the keys
    /// stay free for whatever else they are bound to until there is somewhere
    /// to move to.
    fn keyboard_panning(&self, ctx: &egui::Context) -> Vec2 {
        if self.metrics.available_size == Vec2::ZERO {
            return Vec2::ZERO;
        }

        let seconds = ctx.input(|input| input.stable_dt);
        let pan = input::panning(ctx, &self.config, self.metrics.available_size, seconds);

        if pan != Vec2::ZERO {
            // Held keys produce no events, so nothing else would ask for the
            // next frame and the image would move one step and stop.
            ctx.request_repaint();
        }

        pan
    }

    pub(super) fn handle_context_menu(&mut self, response: &Response) {
        let Some(path) = self.active_path() else {
            return;
        };

        if let Some(callback) =
            actions::show_context_menu(&self.config.context_menu, response, &path)
        {
            self.callback = Some(Callback::from_callback(callback, Some(path)));
        }
    }

    pub(super) fn run_user_action(&mut self, index: usize, ctx: &egui::Context) {
        let (Some(action), Some(path)) = (self.config.user_actions.get(index), self.active_path())
        else {
            return;
        };

        if !actions::execute(&action.exec, &path) {
            return;
        }

        if let Some(callback) = action.callback.clone() {
            self.callback = Some(Callback::from_callback(callback, Some(path)));
        }

        ctx.request_repaint();
    }
}
