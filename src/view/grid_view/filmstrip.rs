//! A strip of thumbnails under the photograph.
//!
//! The contact sheet answers "what is in this folder" and the image view
//! answers "what is this frame", and between them there is a question neither
//! does well: what is either side of the one I am looking at. Culling is a
//! walk along a line of frames, and a viewer that shows one frame at a time
//! makes that walk blind — you cannot see the burst you are in the middle of
//! without leaving the picture.
//!
//! From the thumbnail store the contact sheet already fills, so it costs
//! nothing but the drawing: the textures are resident whichever view is on
//! screen, which is why the grid is warmed while the image view is up.
//!
//! It follows what is on show rather than the whole folder, so a filtered
//! collection has a filtered strip — the frames it skips past are the frames
//! it does not draw.

use eframe::egui::{self, Color32, Rect, Sense};
use eframe::epaint::Vec2;

use crate::cache::{ImageState, ImageStore};
use crate::ui::menus::Chosen;
use crate::view::selection::Selection;
use crate::view::texture;
use crate::view::visible::Visible;

use super::cell;

/// The thumbnail is never cropped, so it always shows all of itself.
const WHOLE_IMAGE: Rect = Rect {
    min: eframe::epaint::pos2(0.0, 0.0),
    max: eframe::epaint::pos2(1.0, 1.0),
};

const BACKGROUND: Color32 = Color32::from_rgb(38, 38, 38);
const CELL: Color32 = Color32::from_rgb(70, 70, 70);

/// The border round the photograph the viewer is on.
const CURRENT: Color32 = Color32::from_rgb(232, 232, 232);

/// The border round the other photographs on screen beside it.
///
/// The same white at less than half the strength and half the width. Showing
/// four photographs side by side means four frames on the strip are on screen
/// and one of them is the one the keys are about, and the difference between
/// those two has to be obvious without a second look — so it is the same
/// colour said quietly rather than a colour of its own, which would read as a
/// third kind of thing.
const ALSO_SHOWN: Color32 = Color32::from_rgba_premultiplied(104, 104, 104, 115);

/// How thick the border round the photograph on screen is, and round the
/// others beside it.
const CURRENT_STROKE: f32 = 3.0;
const ALSO_SHOWN_STROKE: f32 = 1.5;

/// Gap either side of a cell.
const GAP: f32 = 3.0;

/// The margin inside the panel, on each of the four sides.
const MARGIN: f32 = 4.0;

/// Room kept under the thumbnails for the horizontal scroll bar.
///
/// Reserved rather than overlaid: a bar drawn across the bottom of a row of
/// thumbnails hides the part of a photograph that a cull is often about.
const BAR: f32 = 14.0;

/// The shortest and the tallest the strip may be dragged to.
///
/// The top matches the range the registry row declares, so the number typed
/// into the settings window and the number the edge can reach are the same
/// number.
pub const SHORTEST: f32 = 48.0;
pub const TALLEST: f32 = 400.0;

/// The largest square cell a strip that many points tall can draw.
///
/// Split out and free of egui so the arithmetic that decides how big the
/// thumbnails are can be tested without a window. `inner` is what the panel
/// leaves after its own margins, which is what the strip actually has.
pub fn cell_side(inner: f32) -> f32 {
    (inner - BAR).max(16.0)
}

/// What the strip has to mark, and in what colour.
///
/// The three marks are three different questions and the strip answers all of
/// them at once: which photograph the keys are about, which are on screen
/// beside it — a comparison of four means four of these thumbnails are in
/// front of the person — and which have been picked out for a command.
pub struct OnScreen<'a> {
    /// The store position the keys and every command are about, where one is.
    ///
    /// None while a comparison has had its focused photograph taken out of the
    /// set: nothing on screen is the one being marked, so nothing on the strip
    /// wears the border that says so.
    pub cursor: Option<usize>,
    /// Every store position drawn in the panel, the cursor included.
    pub panes: &'a [usize],
    /// The photographs picked out.
    pub selection: &'a Selection,
    /// What a picked-out photograph is marked in.
    pub colour: Color32,
    /// Whether the folder on show is the viewer's own bin, which changes two
    /// rows of the menu a thumbnail carries.
    pub in_the_bin: bool,
    /// The user's own menu entries, appended to every one of them.
    pub entries: &'a [crate::config::ContextMenuEntry],
}

impl OnScreen<'_> {
    /// What one cell stands for.
    ///
    /// Linear in the panes, which is at most eight and usually one: a set
    /// would cost more to build every frame than it saves.
    fn state(&self, index: usize) -> State {
        State {
            current: Some(index) == self.cursor,
            shown: self.panes.contains(&index),
            picked: self.selection.contains(index),
        }
    }
}

/// What a click on the strip was asking for.
///
/// Reported rather than acted on, because the strip is handed the collection
/// and the set but not the right to change either: what a click means is the
/// contact sheet's to say, since it is the one that owns the set both views
/// draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Click {
    /// Show this one. Plain click.
    Open(usize),
    /// Pick this one out, or put it back, leaving the rest alone. Ctrl.
    Toggle(usize),
    /// Pick out the run between this one and the nearest already picked.
    /// Shift.
    Run(usize),
}

impl Click {
    /// The store position the click landed on, whatever it was asking for.
    pub fn index(self) -> usize {
        match self {
            Click::Open(index) | Click::Toggle(index) | Click::Run(index) => index,
        }
    }

    /// What the modifiers held at the time mean.
    ///
    /// The two every file manager uses, and for once the layout quirk that
    /// bedevils the digit keys does not apply: nobody reaches Ctrl or Shift by
    /// accident with a mouse in their hand.
    pub fn of(modifiers: egui::Modifiers, index: usize) -> Click {
        if modifiers.command {
            Click::Toggle(index)
        } else if modifiers.shift {
            Click::Run(index)
        } else {
            Click::Open(index)
        }
    }
}

/// What the strip reports back.
pub struct Picked {
    /// What was clicked, and what the click meant.
    pub click: Option<Click>,
    /// What a menu was asked for, and about which store position.
    pub chosen: Option<(Chosen, usize)>,
    /// How tall the panel is actually drawn, this frame.
    ///
    /// Reported every frame rather than only when it differs, because telling
    /// a drag from a layout pass is [`crate::ui::dragged::Dragged`]'s job and
    /// it needs to see the frames in between to do it.
    pub height: f32,
}

/// Draws the strip, `height` points tall.
///
/// `cursor` is the store position on screen, so the strip can mark it and
/// scroll to keep it in view. `forced` states the height rather than
/// suggesting it, for the one frame after something other than a drag has
/// changed it — the settings window, or the history putting it back.
///
/// The contents are made to fill the panel, and that is not decoration. egui
/// remembers a panel's size as the rectangle its *contents* came to, not the
/// rectangle the drag asked for, so a strip whose thumbnails were sized from
/// the configured height reported that height back however far the edge had
/// been pulled — and the panel returned to where it started on the very next
/// frame. Filling the height makes the two rectangles the same one.
pub fn show(
    ctx: &egui::Context,
    store: &mut ImageStore,
    visible: &Visible,
    on_screen: &OnScreen<'_>,
    height: f32,
    forced: bool,
) -> Picked {
    let cursor = on_screen.cursor;
    let mut picked = Picked {
        click: None,
        chosen: None,
        height,
    };

    if height <= 0.0 || visible.is_empty() {
        return picked;
    }

    let mut panel = egui::TopBottomPanel::bottom("filmstrip")
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(BACKGROUND).inner_margin(MARGIN))
        .resizable(true)
        .default_height(height)
        .min_height(SHORTEST)
        .max_height(TALLEST);

    if forced {
        panel = panel.exact_height(height);
    }

    let panel = panel.show(ctx, |ui| {
            // Whatever the drag has just asked for, rather than what the
            // configuration last said: the cells follow the edge in the same
            // frame it moves, and the panel keeps the size it was given.
            let inner = ui.available_height();
            ui.set_min_height(inner);

            ui.interact(
                ui.max_rect(),
                ui.id().with("strip hover"),
                egui::Sense::hover(),
            )
            .on_hover_text(
                "Every photograph in the folder, in order. Click one to open it; \n                 drag the top edge to make the strip taller.",
            );

            ui.spacing_mut().item_spacing = Vec2::new(GAP, 0.0);

            let cell = cell_side(inner);
            let colour = on_screen.colour;
            let at = cursor.and_then(|cursor| visible.position_of(cursor));

            let mut area =
                egui::ScrollArea::horizontal().scroll_source(egui::scroll_area::ScrollSource::ALL);

            // Kept in view as the cursor moves, which is the whole reason to
            // have it: a strip showing where you were is no use.
            if let Some(at) = at {
                let step = cell + GAP;
                let middle = ui.available_width() / 2.0;
                area = area.horizontal_scroll_offset((at as f32 * step - middle).max(0.0));
            }

            area.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(GAP, 0.0);

                    for index in visible.iter() {
                        let path = store.path(index).map(std::path::Path::to_path_buf);
                        let name = path
                            .as_deref()
                            .and_then(|path| path.file_name())
                            .map(|name| name.to_string_lossy().into_owned());

                        let state = on_screen.state(index);
                        let Some(response) =
                            draw_cell(ui, store, index, state, cell, colour, name.as_deref())
                        else {
                            continue;
                        };

                        if response.clicked() {
                            let modifiers = ui.input(|i| i.modifiers);
                            picked.click = Some(Click::of(modifiers, index));
                        }

                        // What the menu is about: the set where this thumbnail
                        // is in it, and this one photograph where it is not.
                        // The same rule the contact sheet's cells use, because
                        // it is the same set.
                        let Some(path) = path else { continue };
                        let count = match on_screen.selection.contains(index) {
                            true => on_screen.selection.len().max(1),
                            false => 1,
                        };

                        let chosen = crate::actions::show_context_menu(
                            ui,
                            "strip",
                            crate::ui::menus::Row::on_the_strip(on_screen.in_the_bin),
                            on_screen.entries,
                            &response,
                            &path,
                            count,
                        );

                        if let Some(chosen) = chosen {
                            picked.chosen = Some((chosen, index));
                        }
                    }
                });
            });
        });

    picked.height = panel.response.rect.height();

    picked
}

/// What one cell on the strip stands for, this frame.
///
/// Three states rather than one, and they are independent: the photograph the
/// keys are about, the ones beside it in the panel, and the ones that have
/// been picked out. A frame can be all three at once, which is the ordinary
/// case once anything is picked out at all.
#[derive(Clone, Copy)]
pub struct State {
    /// The photograph the keys and every command are about.
    pub current: bool,
    /// On screen in the panel, beside the current one.
    pub shown: bool,
    /// Picked out, so a command means it too.
    pub picked: bool,
}

/// One thumbnail. Answers with its response, where it is worth interacting
/// with — a cell scrolled out of sight, or one behind a window, is not.
fn draw_cell(
    ui: &mut egui::Ui,
    store: &mut ImageStore,
    index: usize,
    state: State,
    cell: f32,
    colour: Color32,
    name: Option<&str>,
) -> Option<egui::Response> {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(cell), Sense::click());

    // Nothing is drawn for a cell that has scrolled past: a strip over a folder
    // of ten thousand would otherwise ask the store about every one of them
    // every frame.
    if !ui.is_rect_visible(rect) {
        return None;
    }

    ui.painter().rect_filled(rect, 0.0, CELL);

    match store.texture(index) {
        Some(texture) => {
            let size = fit(texture.size, cell);
            let at = Rect::from_center_size(rect.center(), size);

            texture::draw(ui, at, texture, WHOLE_IMAGE);
        }
        None => {
            // A cell the decoders have not reached says so quietly rather than
            // spinning: a strip of forty spinners is a strobe.
            if store.state(index) == ImageState::Failed {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✖",
                    egui::FontId::proportional(cell * 0.3),
                    Color32::from_rgb(150, 150, 150),
                );
            }
        }
    }

    // The wash first and the borders over it, so a picked-out frame that is
    // also the one on screen still reads as the one on screen. The same wash
    // and the same tick the contact sheet draws: one mark, one meaning.
    cell::picked(ui, rect, state.picked, colour);

    // Both borders are drawn inside the cell and the current one is thicker,
    // which is the whole of "which of these four am I actually on".
    let border = match (state.current, state.shown) {
        (true, _) => Some((CURRENT, CURRENT_STROKE)),
        (false, true) => Some((ALSO_SHOWN, ALSO_SHOWN_STROKE)),
        (false, false) => None,
    };

    if let Some((colour, width)) = border {
        ui.painter().rect_stroke(
            rect.shrink(width / 2.0),
            0.0,
            egui::Stroke::new(width, colour),
            egui::StrokeKind::Inside,
        );
    }

    // A thumbnail behind a window is not a thumbnail: no cursor, no click, no
    // menu.
    if crate::utils::is_a_window_in_front(ui.ctx()) {
        return None;
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    // Named, because a strip of forty thumbnails forty pixels across is a
    // strip in which nothing can be read.
    if let Some(name) = name {
        response.clone().on_hover_text(name);
    }

    Some(response)
}

/// Largest size with the thumbnail's shape that fits a square cell.
fn fit(size: Vec2, cell: f32) -> Vec2 {
    if size.x <= 0.0 || size.y <= 0.0 {
        return Vec2::splat(cell);
    }

    size * (cell / size.x).min(cell / size.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compared with a tolerance: the scale is a division and a multiply, so
    /// a square cell can come back a few bits off sixty.
    fn close(found: Vec2, wanted: Vec2) -> bool {
        (found.x - wanted.x).abs() < 0.01 && (found.y - wanted.y).abs() < 0.01
    }

    #[test]
    fn a_thumbnail_keeps_its_shape_in_a_square_cell() {
        for (size, cell, wanted) in [
            (Vec2::new(200.0, 100.0), 100.0, Vec2::new(100.0, 50.0)),
            (Vec2::new(100.0, 200.0), 100.0, Vec2::new(50.0, 100.0)),
            (Vec2::new(100.0, 100.0), 60.0, Vec2::new(60.0, 60.0)),
            (Vec2::new(6000.0, 4000.0), 48.0, Vec2::new(48.0, 32.0)),
        ] {
            let found = fit(size, cell);
            assert!(close(found, wanted), "{size:?} in {cell}: {found:?}");
        }
    }

    #[test]
    fn a_thumbnail_of_no_size_fills_the_cell() {
        assert_eq!(fit(Vec2::ZERO, 48.0), Vec2::splat(48.0));
    }

    /// The point of dragging the strip taller: the thumbnails grow with it.
    /// They did not, because the cell was sized from the configured height
    /// rather than from the height the panel had actually been given.
    #[test]
    fn a_taller_strip_draws_larger_thumbnails() {
        let mut last = 0.0_f32;

        for inner in [SHORTEST, 96.0, 200.0, TALLEST] {
            let cell = cell_side(inner);
            assert!(cell > last, "{inner} gave {cell}, no larger than {last}");
            last = cell;
        }
    }

    /// The scroll bar gets room of its own rather than being drawn across the
    /// bottom of the photographs.
    #[test]
    fn the_scroll_bar_is_not_drawn_over_a_thumbnail() {
        assert!(cell_side(200.0) <= 200.0 - BAR);
    }

    /// The shortest the edge can be dragged to still draws something worth
    /// looking at rather than collapsing to the floor.
    #[test]
    fn the_shortest_strip_still_draws_a_thumbnail() {
        assert!(cell_side(SHORTEST) >= 16.0);
    }

    /// A height below the floor is still answered with a cell rather than a
    /// negative one: a hand-edited configuration reaches here.
    #[test]
    fn a_strip_of_no_height_still_answers_with_a_cell() {
        assert_eq!(cell_side(0.0), 16.0);
        assert_eq!(cell_side(-40.0), 16.0);
    }

    #[test]
    fn a_plain_click_opens_and_the_two_modifiers_pick_out() {
        assert_eq!(Click::of(egui::Modifiers::NONE, 4), Click::Open(4));
        assert_eq!(Click::of(egui::Modifiers::COMMAND, 4), Click::Toggle(4));
        assert_eq!(Click::of(egui::Modifiers::SHIFT, 4), Click::Run(4));
    }

    /// Both at once is one gesture and has to mean one thing. Ctrl wins: it is
    /// the one that says "this frame and nothing else about it".
    #[test]
    fn both_modifiers_together_mean_the_first_of_them() {
        let both = egui::Modifiers::COMMAND | egui::Modifiers::SHIFT;

        assert_eq!(Click::of(both, 4), Click::Toggle(4));
    }

    #[test]
    fn a_click_says_which_frame_it_landed_on_whatever_it_meant() {
        for click in [Click::Open(7), Click::Toggle(7), Click::Run(7)] {
            assert_eq!(click.index(), 7);
        }
    }

    fn on_screen<'a>(cursor: usize, panes: &'a [usize], selection: &'a Selection) -> OnScreen<'a> {
        OnScreen {
            cursor: Some(cursor),
            panes,
            selection,
            colour: Color32::WHITE,
            in_the_bin: false,
            entries: &[],
        }
    }

    /// The three marks are three questions, and a frame can answer all of them
    /// at once — which is the ordinary case once anything is picked out.
    #[test]
    fn the_photograph_on_screen_is_marked_as_all_three() {
        let mut selection = Selection::default();
        selection.add(2);
        selection.add(3);

        let panes = [2, 3, 4];
        let on_screen = on_screen(2, &panes, &selection);

        let state = on_screen.state(2);
        assert!(state.current && state.shown && state.picked);
    }

    /// A pane beside it is marked as on screen but not as the one the keys are
    /// about, which is the whole point of the two borders.
    #[test]
    fn the_other_panes_are_marked_apart_from_the_current_one() {
        let selection = Selection::default();
        let panes = [2, 3, 4];
        let on_screen = on_screen(2, &panes, &selection);

        let state = on_screen.state(3);
        assert!(!state.current && state.shown && !state.picked);
    }

    /// A frame picked out in the contact sheet and left behind is marked as
    /// picked and nothing else.
    #[test]
    fn a_frame_picked_out_elsewhere_is_marked_as_picked_alone() {
        let mut selection = Selection::default();
        selection.add(40);

        let panes = [2];
        let on_screen = on_screen(2, &panes, &selection);

        let state = on_screen.state(40);
        assert!(!state.current && !state.shown && state.picked);
    }

    #[test]
    fn a_frame_that_is_none_of_the_three_is_marked_as_none_of_them() {
        let selection = Selection::default();
        let panes = [2];
        let on_screen = on_screen(2, &panes, &selection);

        let state = on_screen.state(9);
        assert!(!state.current && !state.shown && !state.picked);
    }
}
