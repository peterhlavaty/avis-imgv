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

use super::{input, pan, ImageView};

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

    /// The pan asked for by the keys this frame: one step for a press, and a
    /// glide for a key held longer than the delay. [`pan`] decides both.
    fn keyboard_panning(&mut self, ctx: &egui::Context) -> Vec2 {
        // egui runs the frame again whenever something in it asks for another
        // look, and the second pass arrives with no events but with a clock
        // that has moved on — so the keys would be paid twice for one frame,
        // and `canvas::metrics` applies the delta once a pass. The pan is
        // decided on the first pass and carried by the ones after it.
        if ctx.current_pass_index() > 0 || self.metrics.available_size == Vec2::ZERO {
            return Vec2::ZERO;
        }

        let keys = pan::asked(ctx, &self.config);

        if keys.anything() {
            // Held keys produce no events, so nothing else would ask for the
            // next frame: the image would move one step and stop, and a glide
            // waiting on the delay would never start at all.
            ctx.request_repaint();
        }

        let pace = pan::Pace::of(&self.config, keys.fine);
        let seconds = ctx.input(|input| input.stable_dt);

        self.glide
            .moved(keys, pace, self.metrics.available_size, seconds)
    }

    /// Which pane the pointer is over, where the pointer is anywhere at all.
    ///
    /// `None` with one pane as readily as with none: a single photograph needs
    /// no telling apart, and every caller falls back to the cursor, which is
    /// what it always was.
    pub(super) fn pane_under_pointer(&self, ctx: &egui::Context) -> Option<usize> {
        if self.drawn_panes.len() < 2 {
            return None;
        }

        let at = ctx.input(|i| i.pointer.interact_pos())?;

        super::panes::at(&self.pane_layout(), at)
    }

    /// Makes the pane that was clicked the one the keys are about.
    ///
    /// A plain click on the photograph did nothing at all until now, which is
    /// what makes this affordable: it takes a gesture that was going spare
    /// rather than one that meant something else. A drag still pans or marks
    /// an area — `clicked` is a press and release in the same place — and with
    /// one photograph on screen there is nothing to move the focus to.
    pub(super) fn handle_pane_click(&mut self, ctx: &egui::Context, response: &Response) {
        if !response.clicked() || self.drawn_panes.len() < 2 {
            return;
        }

        if crate::utils::is_a_window_in_front(ctx) || self.pointer_is_on_the_marking(ctx) {
            return;
        }

        if let Some(index) = self.pane_under_pointer(ctx) {
            self.select(index);
        }
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

        // The photograph under the *button*, not the one the keys are about.
        // With four side by side those are different photographs three times
        // out of four, and a menu that names one of them while sitting over
        // another is a menu that will throw the wrong file away.
        let index = self.pane_under_pointer(ctx).unwrap_or(self.cursor);
        let Some(path) = self.store.path(index).map(std::path::Path::to_path_buf) else {
            return;
        };

        let chosen = actions::show_context_menu(
            &egui::Ui::new(
                ctx.clone(),
                egui::Id::new("photograph menu"),
                egui::UiBuilder::new().max_rect(response.rect),
            ),
            "photograph",
            Row::on_a_photograph(self.in_the_bin),
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
            // About the pane the button came down on rather than the photograph
            // the keys are about, which is the whole reason the menu asks
            // which pane it is over.
            Some(Chosen::Verb(verb @ (Verb::Keep | Verb::Reject))) => {
                let flag = match verb {
                    Verb::Keep => crate::metadata::xmp::Flag::Picked,
                    _ => crate::metadata::xmp::Flag::Rejected,
                };

                self.bar_actions
                    .push(crate::view::image_view::bottom_bar::BarAction::FlagOne(
                        index, flag,
                    ));
            }
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
