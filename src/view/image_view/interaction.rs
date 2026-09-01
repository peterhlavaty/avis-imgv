//! What the pointer, the wheel and the held keys do to the image on screen.
//!
//! Kept apart from the view itself because it is all one subject: turning a
//! gesture into a movement of the viewport, and a click into whatever the user
//! configured it to run.

use std::path::PathBuf;

use eframe::egui::{self, PointerButton, Response};
use eframe::epaint::Vec2;

use crate::actions::{self, Callback};
use crate::config::{DragButton, WheelJob};
use crate::ui::menus::{Chosen, Row, Verb};
use crate::view::wheel::{self, Job, Notch};

use super::{input, ImageView};

impl ImageView {
    pub(super) fn handle_pointer(&mut self, ctx: &egui::Context, response: &Response) {
        // While a window of the viewer's own is up, the photograph is a
        // picture rather than a surface: no wheel, no drag, no pinch. The
        // wheel is the one that mattered — a notch spent scrolling a page of
        // settings walked the folder behind it, because `contains_pointer` is
        // decided by where the pointer is and the window covers only part of
        // the screen.
        //
        // Not `are_inputs_muted`, which a focused text field also answers yes
        // to: typing in the filter bar takes the keys, and the wheel over the
        // photograph goes on meaning what it means.
        if crate::utils::is_a_window_in_front(ctx) {
            self.viewport.scroll_delta = Vec2::ZERO;
            return;
        }

        let hovered = response.contains_pointer();
        let notch = if hovered { wheel::read(ctx) } else { None };

        // Through the same command as the keys, so a pinch holds the point
        // under the fingers rather than the middle of the panel.
        //
        // Ctrl and the wheel arrive here as well, because egui's
        // `zoom_modifier` is Ctrl and it has already turned them into a zoom
        // by this point. Where the user has given Ctrl and the wheel another
        // job, that zoom is not wanted and is dropped: it is the same notch,
        // counted twice.
        let ctrl_wheel = notch.is_some_and(|notch| notch.modifiers.command);
        let zoom_delta = ctx.input(|i| i.zoom_delta());
        if hovered && zoom_delta != 1.0 && (!ctrl_wheel || self.mouse.ctrl_wheel == WheelJob::Zoom)
        {
            self.apply(input::Command::ZoomBy(zoom_delta), ctx);
        }

        let keyboard = self.keyboard_panning(ctx);

        if !hovered {
            // Losing the pointer mid-scroll would otherwise leave the last
            // delta applied every frame.
            self.viewport.scroll_delta = keyboard;
            return;
        }

        // One job at a time. This used to be `smooth_scroll_delta` whatever
        // else the wheel had already done, so a notch that had just called
        // `Next` also shoved the photograph that had arrived because of it.
        let mut delta = notch.map_or(Vec2::ZERO, |notch| self.wheel(ctx, notch));

        if self.dragging(ctx, response) {
            delta += ctx.input(|i| i.pointer.delta()) * ctx.pixels_per_point();
        }

        self.viewport.scroll_delta = delta + keyboard;
    }

    /// Carries out what the wheel asked for, and reports how far it moved the
    /// photograph.
    ///
    /// The decision itself is [`wheel::decide`], which is pure and tested;
    /// this is only the half that needs a view to apply it to.
    fn wheel(&mut self, ctx: &egui::Context, notch: Notch) -> Vec2 {
        // How far a pan travels, as egui has already smoothed and scaled it.
        // Only the pans use it; nothing else here has a distance.
        let smooth = ctx.input(|i| i.smooth_scroll_delta);

        let command = match wheel::decide(notch, &self.mouse) {
            Job::Forward => input::Command::Next,
            Job::Back => input::Command::Previous,
            Job::PageForward => input::Command::PageForward,
            Job::PageBack => input::Command::PageBack,
            Job::ZoomIn => input::Command::ZoomBy(self.zoom_step()),
            Job::ZoomOut => input::Command::ZoomBy(1.0 / self.zoom_step()),
            Job::Pan => return smooth,
            // Alt folds the wheel onto the vertical axis before this crate
            // sees a delta, so the movement is read off y and spent on x.
            Job::PanSideways => return Vec2::new(smooth.y, 0.0),
            Job::AlreadyZoomed | Job::Nothing => return Vec2::ZERO,
        };

        self.apply(command, ctx);
        Vec2::ZERO
    }

    /// How much one notch magnifies by, which is what one press of the zoom
    /// keys does.
    fn zoom_step(&self) -> f32 {
        if self.config.zoom_step > 1.0 {
            self.config.zoom_step
        } else {
            1.25
        }
    }

    /// Whether the photograph is being dragged about this frame.
    ///
    /// Named buttons, because `is_decidedly_dragging` answers for every one of
    /// them: a right-button drag used to pan the photograph and then release
    /// into whatever menu was registered on the panel, with the boundary
    /// between the two drawn by a distance of six points and eight tenths of a
    /// second that nothing on screen mentions.
    ///
    /// The wheel pressed and dragged always pans, whatever `mouse.drag` says,
    /// so a fitted photograph is not a dead surface.
    ///
    /// And the drag has to have *started* here. This is gated on
    /// `contains_pointer`, which egui documents as true "even if some other
    /// widget is being dragged", so dragging the zoom slider and letting the
    /// pointer stray up over the photograph used to move it under a drag that
    /// was never about it.
    fn dragging(&self, ctx: &egui::Context, response: &Response) -> bool {
        // One press is one gesture. A drag that is marking out part of the
        // photograph, or moving a side of what is already marked, is not also
        // a drag that moves the picture underneath it.
        if self.area.is_dragging() {
            return false;
        }

        ctx.input(|i| {
            if !i.pointer.is_decidedly_dragging() {
                return false;
            }

            if !i
                .pointer
                .press_origin()
                .is_some_and(|from| response.rect.contains(from))
            {
                return false;
            }

            let named = match self.mouse.drag {
                DragButton::Left => i.pointer.button_down(PointerButton::Primary),
                DragButton::Middle => i.pointer.button_down(PointerButton::Middle),
                DragButton::Right => i.pointer.button_down(PointerButton::Secondary),
                DragButton::Any => true,
            };

            named || i.pointer.button_down(PointerButton::Middle)
        })
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

    pub(super) fn handle_context_menu(&mut self, ctx: &egui::Context, response: &Response) {
        // The photograph has no menu while a window is over it.
        if crate::utils::is_a_window_in_front(ctx) {
            return;
        }

        // Nor where something is marked out on it: a menu is drawn over the
        // very thing it belongs to, and inside a marking that thing is the
        // marking. Its own menu ends with the same route to the settings.
        if self.pointer_is_on_the_marking(ctx) {
            return;
        }

        let Some(path) = self.active_path() else {
            return;
        };

        let chosen = actions::show_context_menu(
            &egui::Ui::new(
                ctx.clone(),
                egui::Id::new("photograph menu"),
                egui::UiBuilder::new().max_rect(response.rect),
            ),
            "photograph",
            Row::ON_A_PHOTOGRAPH,
            &self.config.context_menu,
            response,
            &path,
            1,
        );

        match chosen {
            None => {}
            // The zoom verbs are about what this view draws and are done here;
            // the rest need the folder, the journal or a decode thread, and go
            // up to whoever has them.
            Some(Chosen::Verb(Verb::Fit)) => self.apply(input::Command::Fit, ctx),
            Some(Chosen::Verb(Verb::Fill)) => self.apply(input::Command::Fill, ctx),
            Some(Chosen::Verb(Verb::ActualPixels)) => self.apply(
                input::Command::ZoomToPercent(100.0, input::Anchor::Pointer),
                ctx,
            ),
            Some(Chosen::Verb(Verb::Compare)) => self.apply(input::Command::Compare, ctx),
            Some(Chosen::Verb(verb)) => self.verb = Some((verb, path)),
            Some(Chosen::Entry(i)) => {
                if let Some(callback) = self
                    .config
                    .context_menu
                    .get(i)
                    .and_then(|entry| entry.callback.clone())
                {
                    self.callback = Some(Callback::from_callback(callback, Some(path)));
                }
            }
        }
    }

    /// The verb the menu asked for that this view cannot carry out itself.
    pub fn take_verb(&mut self) -> Option<(Verb, PathBuf)> {
        self.verb.take()
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
