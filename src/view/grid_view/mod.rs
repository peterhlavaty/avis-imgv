//! The contact sheet: every image in the folder as a thumbnail.
//!
//! Thumbnails come from a store of their own so the grid can hold hundreds of
//! small textures without competing for the budget the full size view needs.

pub mod cell;
pub mod filmstrip;
pub mod layout;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{
    self, scroll_area::ScrollSource, Color32, PointerButton, Pos2, Rect, Sense, UiBuilder,
};
use eframe::egui_wgpu::RenderState;
use eframe::epaint::Vec2;

use crate::actions::{self, Callback};
use crate::cache::loader::Loader;
use crate::cache::{ImageState, ImageStore, StoreConfig, StoreStats};
use crate::config::{shortcut, GridViewConfig};
use crate::ui::empty::{self, Asked, Nothing};
use crate::ui::menus::{Chosen, Verb};
use crate::utils;
use crate::view::texture;

use crate::view::image_view::bottom_bar::Marks;
use crate::view::selection::Selection;
use crate::view::stacks::{self, Stacks};
use crate::view::visible::Visible;

use cell::Badges;
use layout::Layout;

/// The default ground behind a thumbnail, used before a configuration is in
/// hand. What is actually drawn derives from `general.backdrop`.
const CELL_BACKGROUND: Color32 = Color32::from_rgb(119, 119, 119);
const CELL_BORDER: Color32 = Color32::from_rgb(48, 48, 48);

/// A thumbnail is never cropped, so it always shows all of itself.
const WHOLE_IMAGE: Rect = Rect {
    min: eframe::epaint::pos2(0.0, 0.0),
    max: eframe::epaint::pos2(1.0, 1.0),
};

/// Widest the grid will go before more images stop fitting usefully.
const MAX_COLUMNS: usize = 16;

/// How many rows Shift and the wheel move at once.
///
/// The same ten the image view moves photographs by, so one gesture means the
/// same amount of folder in both views.
const PAGE: f32 = 10.0;

pub struct GridView {
    store: ImageStore,
    config: GridViewConfig,
    columns: usize,
    /// Set when the user picks an image, consumed by the app.
    selected: Option<PathBuf>,
    callback: Option<Callback>,
    /// A verb from the context menu that the sheet cannot carry out itself.
    verb: Option<(Verb, PathBuf)>,
    /// What the screen with nothing on it was clicked to do.
    asked: Option<Asked>,
    /// Image to scroll to on the next frame.
    scroll_to: Option<usize>,
    /// Where a rubber band started, in screen coordinates.
    ///
    /// The band is read off the pointer rather than off a dragged widget: the
    /// cells carry a menu on the second button and egui has a reported quarrel
    /// with a widget that is both a drag source and a menu, so nothing here is
    /// made a drag source.
    band_from: Option<Pos2>,
    /// The band as it stands this frame, for the cells to test against.
    band: Option<Rect>,
    /// What was picked out before the band began.
    ///
    /// A band adds to the selection rather than replacing it, so dragging a
    /// second run does not throw away the first.
    band_base: Selection,
    /// Points to scroll by on the next frame, from the middle button.
    scroll_points: f32,
    /// Rows to scroll by on the next frame, from Shift and the wheel.
    ///
    /// Kept rather than acted on, because the scroll has to happen inside the
    /// `ScrollArea` and the wheel is read outside it — and reading it inside
    /// would put a per-frame walk of the event list into the row loop.
    scroll_rows: f32,
    /// Where the keyboard is, as a position in what is on show. Not where the
    /// image view is: moving about a contact sheet should not decode a full
    /// sized photograph at every step.
    cursor: usize,
    /// Which photograph the image view is on, as a store position, so the
    /// sheet can say so.
    current: usize,
    badges: Badges,
    /// Which of the store's photographs are shown, and in what order.
    visible: Visible,
    /// The photographs picked out for the next command to act on.
    selection: Selection,
    /// The ground behind a thumbnail, derived from `general.backdrop`.
    backdrop: Color32,
}

impl GridView {
    pub fn new(
        render_state: RenderState,
        loader: Arc<Loader>,
        store_config: StoreConfig,
        output_profile: Arc<str>,
        config: GridViewConfig,
    ) -> GridView {
        GridView {
            store: ImageStore::new(render_state, loader, store_config, output_profile),
            columns: config.images_per_row.max(1),
            badges: Badges::of(&config.badges),
            config,
            selected: None,
            scroll_rows: 0.0,
            scroll_points: 0.0,
            band_from: None,
            band: None,
            band_base: Selection::default(),
            callback: None,
            verb: None,
            asked: None,
            scroll_to: None,
            cursor: 0,
            current: 0,
            visible: Visible::default(),
            selection: Selection::default(),
            backdrop: CELL_BACKGROUND,
        }
    }

    pub fn set_images(&mut self, paths: Vec<PathBuf>) {
        // A different folder is a different set of photographs, and carrying
        // positions over into it would pick out whichever frames happened to
        // land on the same numbers.
        self.selection.clear();
        self.visible = Visible::everything(paths.len());
        self.store.set_paths(paths);
        self.scroll_to = Some(0);
        self.cursor = 0;
        self.current = 0;
    }

    /// Narrows or reorders the sheet, keeping the cursor where it can.
    pub fn set_visible(&mut self, visible: Visible) {
        let staying = self.visible.at(self.cursor);
        self.visible = visible;

        self.cursor = staying
            .and_then(|index| self.visible.nearest(index))
            .unwrap_or(0);
        self.scroll_to = Some(self.cursor);
    }

    pub fn stats(&self) -> StoreStats {
        self.store.stats()
    }

    /// The image the user picked, if any. Consumed on read.
    pub fn take_selected(&mut self) -> Option<PathBuf> {
        self.selected.take()
    }

    pub fn take_callback(&mut self) -> Option<Callback> {
        self.callback.take()
    }

    pub fn pop(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.store.remove(index);
            self.visible.remove_shifting(index);
            self.selection.remove_shifting(index);

            // Which photograph the image view is on, as a store position, so
            // it follows that photograph down rather than pointing at its
            // neighbour.
            if index < self.current {
                self.current -= 1;
            }

            self.cursor = self.cursor.min(self.visible.len().saturating_sub(1));
        }
    }

    /// The photographs picked out, as paths, in the collection's order.
    ///
    /// Empty when nothing is picked, which is the signal every command uses to
    /// mean "this one" instead of "these".
    pub fn selected_paths(&self) -> Vec<PathBuf> {
        self.selection
            .iter()
            .filter_map(|index| self.store.path(index))
            .map(Path::to_path_buf)
            .collect()
    }

    /// How many photographs are picked out.
    pub fn selected_count(&self) -> usize {
        self.selection.len()
    }

    /// Puts the selection down, for when a command has finished with it.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Takes a photograph that has appeared into the sheet, at `index`.
    ///
    /// The keyboard cursor is a position in what is on show rather than a
    /// store position, so the caller fixes it when it hands over the new
    /// order; what has to move here is the selection and the mark saying which
    /// photograph the image view is on.
    pub fn insert(&mut self, index: usize, path: PathBuf) {
        self.store.insert(index, path);
        self.selection.insert_shifting(index);

        if index <= self.current {
            self.current += 1;
        }
    }

    pub fn reload(&mut self, path: &Path) {
        if let Some(index) = self.store.index_of(path) {
            self.store.reload(index);
        }
    }

    /// Turns a photograph a quarter on the card, without decoding it again.
    pub fn turn(&mut self, path: &Path, clockwise: bool) {
        if let Some(index) = self.store.index_of(path) {
            self.store.turn(index, clockwise);
        }
    }

    /// Turns a photograph on the card by any orientation, without decoding it
    /// again. Undo takes the difference between the two orientations.
    pub fn turn_by(&mut self, path: &Path, extra: crate::metadata::Orientation) {
        if let Some(index) = self.store.index_of(path) {
            self.store.turn_by(index, extra);
        }
    }

    /// Services the caches without drawing, so opening the grid does not start
    /// from nothing.
    pub fn warm(&mut self, cursor: usize) -> bool {
        self.current = cursor;
        self.store.set_cursor(cursor);
        self.store.tick()
    }

    /// Takes a changed configuration, for when the keyboard map is edited.
    pub fn set_config(&mut self, config: GridViewConfig) {
        self.badges = Badges::of(&config.badges);
        self.config = config;
    }

    /// The ground behind a thumbnail, derived from the one backdrop field.
    pub fn set_backdrop(&mut self, hex: &str) {
        self.backdrop = crate::ui::theme::backdrop(hex);
    }

    /// Scrolls to `index` on the next frame drawn.
    ///
    /// Asked for when the gallery is opened, and only then: doing it every
    /// frame would drag the view back to the open image the instant the user
    /// scrolled away from it.
    pub fn focus_on(&mut self, index: usize) {
        self.current = index;
        self.cursor = self.visible.nearest(index).unwrap_or(0);
        self.scroll_to = Some(self.cursor);
    }

    /// The store position the keyboard is on, so the panels follow the sheet.
    pub fn cursor(&self) -> Option<usize> {
        self.visible.at(self.cursor)
    }

    pub fn cursor_path(&self) -> Option<PathBuf> {
        self.cursor()
            .and_then(|index| self.store.path(index))
            .map(Path::to_path_buf)
    }

    /// Draws the strip of thumbnails under the image view.
    ///
    /// From this store rather than a second one: the contact sheet's textures
    /// are resident whichever view is on screen, which is why the grid is
    /// warmed while the image view is up. A strip with a cache of its own
    /// would decode the same folder twice.
    pub fn filmstrip(
        &mut self,
        ctx: &egui::Context,
        cursor: usize,
        height: f32,
    ) -> (Option<PathBuf>, Option<f32>) {
        let picked = filmstrip::show(ctx, &mut self.store, &self.visible, cursor, height);

        let opened = picked
            .selected
            .and_then(|index| self.store.path(index))
            .map(Path::to_path_buf);

        (opened, picked.height)
    }

    /// Draws the grid.
    ///
    /// `marks` is what every photograph in the collection carries, in the same
    /// order, so the sheet can draw them without asking the disk per cell.
    pub fn ui(&mut self, ctx: &egui::Context, marks: &[Marks], stacks: &Stacks, nothing: &Nothing) {
        if self.store.tick() {
            ctx.request_repaint();
        }

        self.handle_input(ctx);
        self.handle_band(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let shown = self.visible.len();
            let layout = Layout::new(
                ui.available_width(),
                self.columns,
                shown,
                self.config.cell_aspect,
                self.badges,
            );

            if shown == 0 {
                self.asked = empty::ui(ui, nothing);
                return;
            }

            // Everything but the drag. Dragging the contents to scroll is
            // egui's default and it is the same gesture as the rubber band;
            // the wheel and the bar still scroll, and so does the middle
            // button, which is the gesture that has nothing else to do here.
            let mut scroll_area = egui::ScrollArea::vertical().scroll_source(ScrollSource {
                drag: false,
                ..ScrollSource::ALL
            });
            if let Some(index) = self.scroll_to.take() {
                scroll_area = scroll_area.vertical_scroll_offset(layout.scroll_offset_of(index));
            }

            scroll_area.show_rows(ui, layout.row, layout.rows, |ui, rows| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                // Caching centres on what is on screen, so scrolling pulls the
                // rows just past the fold in ahead of the user. The middle of
                // the fold is a position in what is shown; the store wants the
                // photograph that position stands for.
                let onscreen = layout.indices(rows.clone(), shown);
                let middle = (onscreen.start + onscreen.end) / 2;

                if let Some(index) = self.visible.at(middle) {
                    self.store.set_cursor(index);
                }

                for row in rows {
                    self.show_row(ui, &layout, row, marks, stacks);
                }

                if !utils::are_inputs_muted(ctx)
                    && ui.input_mut(|i| shortcut::consume(i, &self.config.sc_scroll))
                {
                    ui.scroll_with_delta(Vec2::new(0., -(layout.row * 0.5)));
                }

                if self.scroll_rows != 0.0 || self.scroll_points != 0.0 {
                    let by = self.scroll_points - layout.row * self.scroll_rows;
                    ui.scroll_with_delta(Vec2::new(0., by));
                    self.scroll_rows = 0.0;
                    self.scroll_points = 0.0;
                }
            });
        });

        self.show_selection_count(ctx);
        self.show_band(ctx);
    }

    /// Reads the left drag that picks out everything it crosses.
    ///
    /// The image view has nothing to rubber-band and the sheet has nothing to
    /// pan, so the two never have to share a button: a left drag is always a
    /// selection here and always a pan there. What is deliberately not copied
    /// is the size-dependent rule some viewers use — drag means pan when the
    /// picture is larger than the window and something else when it is not —
    /// which is a mode with nothing on screen to say which one you are in.
    fn handle_band(&mut self, ctx: &egui::Context) {
        let (position, pressed, down, dragging) = ctx.input(|i| {
            (
                i.pointer.interact_pos(),
                i.pointer.button_pressed(PointerButton::Primary),
                i.pointer.button_down(PointerButton::Primary),
                i.pointer.is_decidedly_dragging(),
            )
        });

        // The gesture the left button has just given up. Dragging the sheet
        // about is what a middle drag does everywhere it does anything, and it
        // is the one button here with nothing else to do.
        if ctx.input(|i| i.pointer.button_down(PointerButton::Middle)) {
            self.scroll_points += ctx.input(|i| i.pointer.delta().y);
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        }

        if !down {
            self.band_from = None;
            self.band = None;
            return;
        }

        // A band begins on a press *here*, and not merely on finding the
        // button already down: a file being dragged in from a file manager
        // arrives with the button held and the press belonging to somebody
        // else, and it would otherwise paint a selection on its way to the
        // drop. Whether the press is ours is decided once, so a drag that
        // wanders over the selection count on its way is not cut short.
        if self.band_from.is_none() {
            if !pressed || ctx.is_pointer_over_area() || utils::are_inputs_muted(ctx) {
                return;
            }

            self.band_from = position;
            self.band_base = self.selection.clone();
            return;
        }

        let (Some(from), Some(now)) = (self.band_from, position) else {
            return;
        };

        if !dragging {
            // Still inside the click threshold, so this may yet be a click.
            return;
        }

        self.band = Some(Rect::from_two_pos(from, now));
        self.selection = self.band_base.clone();
    }

    /// Draws the band itself, so a drag that is picking things out looks like
    /// one.
    fn show_band(&self, ctx: &egui::Context) {
        let Some(band) = self.band else {
            return;
        };

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("grid-band"),
        ));

        painter.rect_filled(band, 0.0, cell::SELECTED.gamma_multiply(0.25));
        painter.rect_stroke(
            band,
            0.0,
            egui::Stroke::new(1.0_f32, cell::SELECTED),
            egui::StrokeKind::Inside,
        );
    }

    /// Says how many photographs are picked out, and how to stop.
    ///
    /// The sheet has no status bar of its own, and a selection that is only
    /// visible as a wash on the cells that happen to be scrolled into view is
    /// a selection somebody will forget they are holding — and then rate two
    /// hundred photographs meaning to rate one.
    fn show_selection_count(&self, ctx: &egui::Context) {
        let picked = self.selection.len();
        if picked == 0 {
            return;
        }

        egui::Area::new(egui::Id::new("grid-selection-count"))
            .anchor(egui::Align2::RIGHT_BOTTOM, [-12.0, -12.0])
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    // One line: an area anchored to the corner is given the
                    // width it asks for, and a wrapped count reads as two
                    // separate things.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    ui.label(format!("{picked} selected · Escape to clear"));
                });
            });
    }

    fn show_row(
        &mut self,
        ui: &mut egui::Ui,
        layout: &Layout,
        row: usize,
        marks: &[Marks],
        stacks: &Stacks,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.add_space(layout.padding);

            for position in layout.indices(row..row + 1, self.visible.len()) {
                let Some(index) = self.visible.at(position) else {
                    continue;
                };

                self.show_cell(ui, position, index, layout, marks.get(index), stacks);
            }
        });
    }

    fn show_cell(
        &mut self,
        ui: &mut egui::Ui,
        position: usize,
        index: usize,
        layout: &Layout,
        marks: Option<&Marks>,
        stacks: &Stacks,
    ) {
        let (_, rect) = ui.allocate_space(Vec2::new(layout.cell, layout.row));
        let picture = Rect::from_min_size(rect.min, Vec2::new(layout.cell, layout.picture));
        let strip = Rect::from_min_max(
            eframe::epaint::pos2(rect.left(), picture.bottom()),
            rect.max,
        );

        ui.painter().rect_filled(picture, 0, self.backdrop);

        let name = self.file_name(index);
        let caption = self.caption(index, &name);
        let drawn = self.store.texture(index).is_some();

        let response = ui
            .scope_builder(UiBuilder::new().max_rect(picture), |ui| {
                ui.centered_and_justified(|ui| {
                    if !drawn {
                        show_placeholder(ui, self.store.state(index), layout.picture);
                        return None;
                    }

                    // Borrowed again inside, because the placeholder branch
                    // needs the store and the drawing branch needs the
                    // texture.
                    let texture = self.store.texture(index)?;
                    let size = fit_in_cell(texture.size, layout.cell, layout.picture);
                    let (drawn_rect, response) = ui.allocate_exact_size(size, Sense::click());
                    texture::draw(ui, drawn_rect, texture, WHOLE_IMAGE);

                    Some(response.on_hover_text_at_pointer(&name))
                })
                .inner
            })
            .inner;

        cell::picked(ui, picture, self.selection.contains(index));
        cell::dim_if_rejected(ui, picture, marks);

        if let Some(stack) = stacks.stack_of(index) {
            cell::stack(
                ui,
                picture,
                stacks::glyph(stack.kind),
                stack.len(),
                stack.collapsed,
            );

            // The glyph is a shape with no legend anywhere on screen, so it
            // says what it is where somebody is looking at it.
            if let Some(response) = &response {
                response.clone().on_hover_text(format!(
                    "{} of {} frames. {} {}",
                    if stack.collapsed { "One" } else { "Open:" },
                    stack.len(),
                    stack.kind.label(),
                    if stack.collapsed {
                        "— the key opens it."
                    } else {
                        "— the key folds it back up."
                    }
                ));
            }
        }

        cell::caption(ui, strip, self.badges, marks, &caption);

        ui.painter().rect_stroke(
            rect,
            0.,
            egui::Stroke::new(1.0_f32, CELL_BORDER),
            egui::StrokeKind::Outside,
        );

        cell::outline(ui, rect, index == self.current, position == self.cursor);

        if self.band.is_some_and(|band| band.intersects(rect)) {
            self.selection.add(index);
        }

        if let Some(response) = response {
            self.handle_cell_interaction(ui, position, index, &response);
        }
    }

    fn handle_cell_interaction(
        &mut self,
        ui: &egui::Ui,
        position: usize,
        index: usize,
        response: &egui::Response,
    ) {
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        let Some(path) = self.store.path(index).map(Path::to_path_buf) else {
            return;
        };

        if response.clicked() {
            let modifiers = ui.input(|i| i.modifiers);
            self.cursor = position;

            // The two modifiers every file manager uses, and for once the
            // layout quirk that bedevils the digit keys does not apply: nobody
            // reaches Ctrl or Shift by accident with a mouse in their hand.
            if modifiers.command {
                self.selection.toggle(index, position);
            } else if modifiers.shift {
                self.selection.extend_to(&self.visible, position);
            } else if self.config.click_opens {
                self.selected = Some(path.clone());
            } else {
                // A plain click picks this one out, and nothing else. It used
                // to leave the contact sheet altogether, which contradicted
                // the cursor, the selection, Ctrl-click, Shift-click, Space
                // and Enter all at once, and the only way back was Backspace.
                // A culling tool's sheet is a surface you act *on*.
                self.selection.only(index, position);
            }
        }

        // What the sheet says `Open` means on a cell's menu, and what every
        // list of files means by two clicks.
        if !self.config.click_opens && response.double_clicked() {
            self.selected = Some(path.clone());
        }

        // What the menu is about: the selection where the cell is in it, and
        // this one photograph where it is not. Somebody who has picked out
        // two hundred and right-clicks one of them means the two hundred.
        let count = if self.selection.contains(index) {
            self.selection.len().max(1)
        } else {
            1
        };

        let chosen = actions::show_context_menu(
            ui,
            "cell",
            Verb::ON_A_CELL,
            &self.config.context_menu,
            response,
            &path,
            count,
        );

        match chosen {
            None => {}
            Some(Chosen::Verb(Verb::Open)) => self.selected = Some(path),
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

    /// What the screen with nothing on it was clicked to do.
    pub fn take_asked(&mut self) -> Option<Asked> {
        self.asked.take()
    }

    /// The verb the menu asked for that the sheet cannot carry out itself.
    pub fn take_verb(&mut self) -> Option<(Verb, PathBuf)> {
        self.verb.take()
    }

    /// Whether the sheet has nothing to draw.
    pub fn shows_nothing(&self) -> bool {
        self.visible.is_empty()
    }

    /// Which photograph the sheet says is on show, and how many there are.
    pub fn position(&self) -> (usize, usize) {
        (self.cursor, self.visible.len())
    }

    /// The line under one thumbnail.
    ///
    /// Rendered per cell, and only for the cells actually drawn: a contact
    /// sheet shows a screenful at a time however long the folder is, so this
    /// is a few dozen expansions a frame rather than a few thousand.
    ///
    /// Falls back to the file name when the scan has not reached the file, so
    /// a caption asking for a shutter speed says the name until there is one
    /// rather than saying nothing at all.
    fn caption(&self, index: usize, name: &str) -> String {
        if !self.badges.shows_name() || self.config.caption_format.is_empty() {
            return name.to_string();
        }

        let Some(path) = self.store.path(index) else {
            return name.to_string();
        };

        let mut subject = crate::metadata::template::Subject::new(path);
        if let Some(metadata) = self.store.metadata(index) {
            subject = subject.with_metadata(metadata);
        }

        let rendered = crate::metadata::template::render(&self.config.caption_format, &subject);
        if rendered.trim().is_empty() {
            return name.to_string();
        }

        rendered
    }

    fn file_name(&self, index: usize) -> String {
        self.store
            .path(index)
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        if utils::are_inputs_muted(ctx) {
            return;
        }

        let zooming = ctx.input(|i| i.zoom_delta() != 1.0);
        let scroll = ctx.input(|i| i.raw_scroll_delta.y);

        // Ten rows, which is the same step the image view takes with Shift
        // and the same one PageUp and PageDown take. Read off the event
        // because Shift is egui's horizontal scroll modifier, so by the time
        // there is a delta the movement has been spent going sideways across
        // a sheet that has nowhere sideways to go.
        if let Some(notch) = crate::view::wheel::read(ctx) {
            let modifiers = notch.modifiers;
            if modifiers.shift && !modifiers.command && !modifiers.alt {
                self.scroll_rows = if notch.amount < 0.0 { PAGE } else { -PAGE };
            }
        }

        let wider = ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_more_per_row))
            || (zooming && scroll < 0.);
        let narrower = ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_less_per_row))
            || (zooming && scroll > 0.);

        if wider && self.columns < MAX_COLUMNS {
            self.set_columns(self.columns + 1);
        } else if narrower && self.columns > 1 {
            self.set_columns(self.columns - 1);
        }

        if ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_cycle_badges)) {
            self.badges = self.badges.next();
        }

        self.handle_selection(ctx);
        self.move_cursor(ctx);
    }

    /// Walks the sheet with the arrow keys, and opens with Enter.
    ///
    /// A contact sheet nobody can move about without a mouse is a contact
    /// sheet nobody can cull from: every mark is a keystroke, and reaching the
    /// next photograph should be one too.
    ///
    /// Holding shift while walking picks out everything walked over, which is
    /// the one motion that turns a sheet into a way of marking two hundred
    /// frames at once.
    fn move_cursor(&mut self, ctx: &egui::Context) {
        let total = self.visible.len();
        if total == 0 {
            return;
        }

        // Read before anything is consumed, because egui's own matching
        // ignores a shift it was not asked about: the arrow keys are claimed
        // with no modifiers and would swallow the shifted ones too, so whether
        // shift was down has to be asked separately.
        let extending = ctx.input(|i| i.modifiers.shift && !i.modifiers.command);

        let columns = self.columns.max(1);
        let steps = [
            (egui::Key::ArrowRight, 1_isize),
            (egui::Key::ArrowLeft, -1),
            (egui::Key::ArrowDown, columns as isize),
            (egui::Key::ArrowUp, -(columns as isize)),
        ];

        let mut moved = false;

        for (key, step) in steps {
            if !ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                continue;
            }

            let wanted = self.cursor as isize + step;
            // Clamped rather than wrapped: a sheet has edges, and walking off
            // one to land at the far end of another row is disorienting.
            self.cursor = wanted.clamp(0, total as isize - 1) as usize;
            moved = true;
        }

        for (key, index) in [
            (egui::Key::Home, 0usize),
            (egui::Key::End, total.saturating_sub(1)),
        ] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                self.cursor = index;
                moved = true;
            }
        }

        if moved {
            if extending {
                // The run starts where the cursor was standing when shift was
                // first held, so the photograph walked away from is in it.
                self.selection.extend_to(&self.visible, self.cursor);
            } else {
                // Letting go of shift ends the run rather than the selection:
                // the next one starts here.
                self.selection.anchor_at(self.cursor);
            }

            self.scroll_to_cursor();
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            self.selected = self.cursor_path();
        }
    }

    /// Picking photographs out, putting them back, and taking everything.
    fn handle_selection(&mut self, ctx: &egui::Context) {
        // Everything first, because it is the one with the modifier and
        // matching a shortcut only checks the modifiers it asked about.
        if ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_select_all)) {
            self.selection.all_or_none(&self.visible);
            return;
        }

        if ctx.input_mut(|i| shortcut::consume(i, &self.config.sc_select)) {
            if let Some(index) = self.visible.at(self.cursor) {
                self.selection.toggle(index, self.cursor);
            }
            return;
        }

        // Escape puts the selection down, and only that: it is the key people
        // press when they are not sure what they have done, and having it also
        // leave the sheet would take them somewhere they did not ask to go.
        if !self.selection.is_empty()
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            self.selection.clear();
        }
    }

    /// Brings the cursor into view, without dragging the sheet about when it
    /// is already there.
    fn scroll_to_cursor(&mut self) {
        self.scroll_to = Some(self.cursor);
    }

    /// Changes the column count, keeping the user roughly where they were.
    fn set_columns(&mut self, columns: usize) {
        self.scroll_to = Some(self.cursor);
        self.columns = columns;
    }
}

/// Largest size with the thumbnail's aspect ratio that fits the cell.
fn fit_in_cell(size: Vec2, width: f32, height: f32) -> Vec2 {
    if size.x <= 0.0 || size.y <= 0.0 {
        return Vec2::new(width, height);
    }

    let scale = (width / size.x).min(height / size.y);
    size * scale
}

fn show_placeholder(ui: &mut egui::Ui, state: ImageState, cell: f32) {
    if state == ImageState::Failed {
        ui.label("✖");
        return;
    }

    ui.add(egui::Spinner::new().size(cell / 3.));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnails_keep_their_aspect_ratio_in_a_square_cell() {
        assert_eq!(
            fit_in_cell(Vec2::new(200.0, 100.0), 100.0, 100.0),
            Vec2::new(100.0, 50.0)
        );
        assert_eq!(
            fit_in_cell(Vec2::new(100.0, 200.0), 100.0, 100.0),
            Vec2::new(50.0, 100.0)
        );
        assert_eq!(
            fit_in_cell(Vec2::new(100.0, 100.0), 100.0, 100.0),
            Vec2::new(100.0, 100.0)
        );
    }

    /// The cell a three-to-two photograph is drawn into is now three to two
    /// itself, so that photograph fills it and a portrait one is letterboxed.
    #[test]
    fn a_matching_photograph_fills_the_cell() {
        assert_eq!(
            fit_in_cell(Vec2::new(6000.0, 4000.0), 300.0, 200.0),
            Vec2::new(300.0, 200.0)
        );

        let portrait = fit_in_cell(Vec2::new(4000.0, 6000.0), 300.0, 200.0);
        assert!((portrait.y - 200.0).abs() < 0.01, "{portrait:?}");
        assert!((portrait.x - 400.0 / 3.0).abs() < 0.01, "{portrait:?}");
    }

    #[test]
    fn a_degenerate_thumbnail_fills_the_cell() {
        assert_eq!(fit_in_cell(Vec2::ZERO, 64.0, 48.0), Vec2::new(64.0, 48.0));
    }
}
