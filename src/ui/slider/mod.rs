//! A rail whose handle need not keep up with the pointer.
//!
//! Every slider in the program is one of these. It is drawn exactly as the
//! toolkit's own is drawn — [`paint`] is the toolkit's code, from the same
//! style — and differs in one thing: the toolkit reads the value off wherever
//! the pointer *is*, and this reads it off how far the pointer has *moved*,
//! divided by `mouse.slider_travel`. A press still puts the handle under the
//! pointer, so the far end of a long range is one gesture away; the drag after
//! it is finer than a hand.
//!
//! That is why the widget is written out rather than wrapped round the
//! toolkit's. `Slider` sets its value from `interact_pointer_pos` before it
//! paints, with no way in for a caller to say where the pointer should be taken
//! to be, and no rect it can be given that is not also the rect it draws itself
//! into. The interaction is the part being changed, so the interaction is the
//! part written here.
//!
//! The arithmetic lives in [`drag`], which knows nothing about egui and is
//! tested without a window.

pub mod drag;
mod paint;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use eframe::egui::{
    self, emath::Numeric, EventFilter, Key, NumExt as _, PointerButton, Rangef, Response, RichText,
    Sense, TextStyle, Ui, Vec2, ViewportCommand, Widget, WidgetInfo,
};

use crate::config::registry::Page;
use crate::ui::surface::{self, Subject};

/// How far the pointer travels to cross a rail, as everything but the
/// configuration file sees it.
///
/// A process-wide value rather than a parameter threaded through five call
/// sites in four subsystems, for the same reason `surface::show_settings_rows`
/// is one: it is a single decision the whole program agrees about, and the
/// places that draw a rail have no configuration in hand. Written from
/// `App::apply_settings`, where every other copy of a setting is handed out.
static TRAVEL: AtomicU32 = AtomicU32::new(drag::SHIPS_AS.to_bits());

/// What a rail's menu asked the program for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ask {
    /// A travel to write to `mouse.slider_travel`.
    Travel(f32),
    /// The settings window, on the row that owns it.
    Settings,
}

/// The ask, waiting to be taken.
///
/// The same shape as the keyboard's ask in `surface`, and for the same reason:
/// a menu drawn from inside a widget has no route back to `Config`, and a
/// mailbox the program empties once a frame is shorter than a return value
/// threaded up through every caller of every rail.
static ASKED: Mutex<Option<Ask>> = Mutex::new(None);

/// Hands the travel to the rails.
pub fn travels(travel: f32) {
    TRAVEL.store(travel.to_bits(), Ordering::Relaxed);
}

/// What the rails are using.
pub fn travel() -> f32 {
    f32::from_bits(TRAVEL.load(Ordering::Relaxed))
}

/// What a rail's menu asked for, if anything, taking the ask.
pub fn asked() -> Option<Ask> {
    ASKED.lock().ok().and_then(|mut asked| asked.take())
}

/// How near two travels have to be to be the same one.
///
/// Not `f32::EPSILON`: that is smaller than the gap between representable
/// numbers up at three, so a value that had been through the file and back
/// could tick nothing at all. A tenth is the finest the row is worth setting.
const CLOSE_ENOUGH: f32 = 0.01;

fn ask(what: Ask) {
    if let Ok(mut asked) = ASKED.lock() {
        *asked = Some(what);
    }
}

/// Where a drag has got to, kept between frames.
///
/// Two numbers: where on the rail the value is being read from, which is no
/// longer where the pointer is, and where the pointer was last seen, which is
/// what the next frame's movement is measured from.
#[derive(Clone, Copy, Debug, Default)]
struct At {
    on_rail: f32,
    pointer: f32,
}

/// A slider.
pub struct Fine<'a> {
    get_set: Box<dyn 'a + FnMut(Option<f64>) -> f64>,
    range: (f64, f64),
    integral: bool,
    logarithmic: bool,
    show_value: bool,
    suffix: String,
    text: String,
    /// What the rail sets, for the heading of its menu.
    name: &'a str,
    /// The sentence under the pointer, without the words `surface` adds.
    hint: &'a str,
}

impl<'a> Fine<'a> {
    pub fn new<Num: Numeric>(value: &'a mut Num, range: std::ops::RangeInclusive<Num>) -> Self {
        let (min, max) = (range.start().to_f64(), range.end().to_f64());

        Fine {
            get_set: Box::new(move |v: Option<f64>| {
                if let Some(v) = v {
                    *value = Num::from_f64(v);
                }
                value.to_f64()
            }),
            range: if min <= max { (min, max) } else { (max, min) },
            integral: Num::INTEGRAL,
            logarithmic: false,
            show_value: true,
            suffix: String::new(),
            text: String::new(),
            name: "",
            hint: "",
        }
    }

    pub fn logarithmic(mut self, on: bool) -> Self {
        self.logarithmic = on;
        self
    }

    pub fn show_value(mut self, on: bool) -> Self {
        self.show_value = on;
        self
    }

    pub fn suffix(mut self, suffix: impl ToString) -> Self {
        self.suffix = suffix.to_string();
        self
    }

    pub fn text(mut self, text: impl ToString) -> Self {
        self.text = text.to_string();
        self
    }

    /// What this rail sets, which is what its menu says it is about.
    pub fn about(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    /// The sentence under the pointer.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    fn get(&mut self) -> f64 {
        (self.get_set)(None)
    }

    fn set(&mut self, value: f64) {
        let mut value = value.clamp(self.range.0, self.range.1);

        if self.integral {
            value = value.round();
        }

        (self.get_set)(Some(value));
    }

    fn value_at(&self, position: f32, rail: Rangef) -> f64 {
        drag::value_at(position, rail, self.range, self.logarithmic)
    }

    fn position_of(&self, value: f64, rail: Rangef) -> f32 {
        drag::position_of(value, rail, self.range, self.logarithmic)
    }

    /// The roundest value within a hand's width of a point on the rail.
    ///
    /// The toolkit's aim radius is how precisely a hand can place a pointer; a
    /// fine drag places it that many times more precisely, so the radius is
    /// divided by the travel along with everything else. Without that the
    /// values a drag could reach would be exactly the ones a bound drag
    /// reaches, and the whole thing would buy nothing on a long range.
    fn aimed(&mut self, at: f32, rail: Rangef, aim: f32) -> f64 {
        egui::emath::smart_aim::best_in_range_f64(
            self.value_at(at - aim, rail),
            self.value_at(at + aim, rail),
        )
    }

    /// How much the value changes per point of rail, for the box beside it.
    fn gradient(&mut self, rail: Rangef) -> f64 {
        let value = self.get();
        let at = self.position_of(value, rail);
        self.value_at(at + 0.5, rail) - self.value_at(at - 0.5, rail)
    }

    fn add_contents(&mut self, ui: &mut Ui) -> Response {
        let thickness = ui
            .text_style_height(&TextStyle::Body)
            .at_least(ui.spacing().interact_size.y);

        let was = self.get();
        let mut response = ui.allocate_response(
            Vec2::new(ui.spacing().slider_width, thickness),
            Sense::click_and_drag(),
        );

        let shape = ui.style().visuals.handle_shape;
        let rail = paint::rail(response.rect, shape);

        self.dragged(ui, &response, rail);
        self.keyed(ui, &response, rail);

        let value = self.get();
        paint::draw(ui, &response, self.position_of(value, rail), shape);

        if value != was {
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::slider(ui.is_enabled(), value, self.name));

        self.menu(ui, &response);

        if self.show_value {
            response = response.union(self.value_ui(ui, rail));
        }

        if !self.text.is_empty() {
            ui.label(self.text.clone());
        }

        response
    }

    /// The whole of what is different about this slider.
    fn dragged(&mut self, ui: &Ui, response: &Response, rail: Rangef) {
        let id = response.id;

        // The second button is how a menu is asked for, and a menu asked for
        // over a rail must not also move it. The toolkit's slider takes any
        // button, which is a good part of why it never carried one.
        let held = ui.input(|i| i.pointer.button_down(PointerButton::Primary));
        let Some(pointer) = response.interact_pointer_pos().filter(|_| held) else {
            ui.data_mut(|data| data.remove_temp::<At>(id));
            return;
        };

        let travel = travel();
        let gain = drag::gain(travel);
        let window = ui.ctx().viewport_rect().x_range();
        let before = ui.data_mut(|data| data.get_temp::<At>(id));

        let at = match before {
            // The press itself: the handle goes where it was pressed, so the
            // far end of a long range is still one gesture away.
            None => rail.clamp(pointer.x),
            Some(before) => drag::along(
                before.on_rail,
                drag::moved(pointer.x, before.pointer, window),
                gain,
                rail,
            ),
        };

        ui.data_mut(|data| {
            data.insert_temp(
                id,
                At {
                    on_rail: at,
                    pointer: pointer.x,
                },
            );
        });

        // Only while there is rail left to cover. At either end, and whenever
        // the handle is under the pointer anyway, the pointer running out of
        // window means the drag is finished rather than that it wants more
        // room.
        if travel > drag::BOUND && rail.min < at && at < rail.max {
            if let Some(put_back) = drag::wrap(pointer.x, window) {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::CursorPosition(egui::pos2(
                        put_back, pointer.y,
                    )));
            }
        }

        let aim = ui.input(|i| i.aim_radius()) * gain;
        let value = self.aimed(at, rail, aim);
        self.set(value);
    }

    /// The arrows, which move the handle a point of rail at a time.
    fn keyed(&mut self, ui: &Ui, response: &Response, rail: Rangef) {
        if !response.has_focus() {
            return;
        }

        ui.ctx().memory_mut(|memory| {
            memory.set_focus_lock_filter(
                response.id,
                EventFilter {
                    horizontal_arrows: true,
                    ..Default::default()
                },
            );
        });

        let step = ui.input(|i| {
            i.num_presses(Key::ArrowRight) as f32 - i.num_presses(Key::ArrowLeft) as f32
        });

        if step == 0.0 {
            return;
        }

        let value = self.get();

        // A whole number moves by one. A point of rail on a range of four
        // thousand is a fifth of a photograph, and a key that rounds back to
        // where it started is a key that does nothing.
        let wanted = if self.integral {
            value + f64::from(step)
        } else {
            self.aimed(self.position_of(value, rail) + step, rail, 0.49)
        };

        self.set(wanted);
    }

    /// The box beside the rail, which is the toolkit's number box.
    fn value_ui(&mut self, ui: &mut Ui, rail: Rangef) -> Response {
        let speed = self.gradient(rail);
        let was = self.get();
        let mut value = was;

        let response = ui.add({
            let mut shown = egui::DragValue::new(&mut value)
                .speed(speed)
                .suffix(self.suffix.clone())
                .range(self.range.0..=self.range.1)
                .clamp_existing_to_range(false);

            if self.integral {
                shown = shown.max_decimals(0);
            }

            shown
        });

        // Only when it moved. A value the file holds outside the range is left
        // exactly as it was written, and writing it back every frame would
        // clamp it on the first unrelated save.
        if value != was {
            self.set(value);
        }

        response
    }

    /// The second button on a rail, which is about the drag rather than about
    /// the value: *Slider* **Across**, and the five distances the pointer can
    /// be asked to travel.
    ///
    /// About the drag because the value already has a menu wherever it has one
    /// worth having — the zoom rail sits beside a reading that carries fit and
    /// fill — and two menus a few points apart both headed *Zoom* would be two
    /// menus nobody could predict.
    fn menu(&self, ui: &Ui, response: &Response) {
        let travel = travel();

        surface::with_menu(
            ui,
            response,
            Subject::of("Slider", self.name),
            self.hint,
            |ui| {
                ui.label(
                    RichText::new("How far the pointer moves to cross it")
                        .weak()
                        .small(),
                );

                for (distance, label) in drag::CHOICES {
                    let chosen = (travel - distance).abs() < CLOSE_ENOUGH;
                    if ui
                        .selectable_label(chosen, format!("{label}  ({distance:.0}×)"))
                        .clicked()
                    {
                        ask(Ask::Travel(*distance));
                        ui.close();
                    }
                }

                if surface::more_settings(ui, Page::KeysAndMouse) {
                    ask(Ask::Settings);
                    ui.close();
                }
            },
        );
    }
}

impl Widget for Fine<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let inner = ui.horizontal(|ui| self.add_contents(ui));
        inner.inner | inner.response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, Pos2, RawInput, Rect};

    /// The travel and the mailbox are one per program, which is what makes them
    /// reachable from a widget; two tests changing them at once would each see
    /// the other's. Every test here takes this first.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

    fn moved_to(at: Pos2) -> Vec<Event> {
        vec![Event::PointerMoved(at)]
    }

    fn button(at: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        }
    }

    /// Presses a rail in the middle, drags it by each step in turn, and says
    /// where the value ended up.
    ///
    /// A widget's interaction is decided from where it was the frame before, so
    /// the press cannot land on the frame it is sent: the empty frame after it
    /// is the one the rail first hears about it, and the steps follow that.
    fn dragged_from_the_middle(travel: f32, steps: &[f32]) -> f64 {
        travels(travel);

        let ctx = egui::Context::default();
        let mut value = 50.0_f64;
        let mut rect = Rect::ZERO;
        let mut at = Pos2::ZERO;

        let mut input = RawInput::default();

        for pass in 0..steps.len() + 4 {
            let _ = ctx.run(input.clone(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let response = ui.add(Fine::new(&mut value, 0.0..=100.0).show_value(false));
                    rect = response.rect;
                });
            });

            let rail = paint::rail(rect, ctx.style().visuals.handle_shape);

            input = RawInput::default();
            input.events = match pass {
                0 => {
                    at = Pos2::new(rail.center(), rect.center().y);
                    vec![Event::PointerMoved(at), button(at, true)]
                }
                1 => Vec::new(),
                pass if pass < steps.len() + 2 => {
                    at.x += steps[pass - 2];
                    moved_to(at)
                }
                _ => vec![button(at, false)],
            };
        }

        value
    }

    /// The complaint and the answer to it in one assertion: the same movement
    /// of the hand moves the value a third as far.
    #[test]
    fn a_longer_travel_moves_the_value_less() {
        let _one_at_a_time = ONE_AT_A_TIME.lock();

        let bound = dragged_from_the_middle(1.0, &[30.0]);
        let fine = dragged_from_the_middle(3.0, &[30.0]);

        assert!(bound > 50.0, "the bound drag moved it: {bound}");
        assert!(fine > 50.0, "and so did the fine one: {fine}");
        assert!(
            fine - 50.0 < (bound - 50.0) / 2.0,
            "three times the travel should be well under half the movement, \
             but it went to {fine} against {bound}"
        );
    }

    /// A drag is the sum of what the hand did, however many frames it took, and
    /// back is back.
    #[test]
    fn the_drag_adds_up_over_the_frames_it_took() {
        let _one_at_a_time = ONE_AT_A_TIME.lock();

        let one_go = dragged_from_the_middle(2.0, &[40.0]);
        let in_pieces = dragged_from_the_middle(2.0, &[10.0, 10.0, 10.0, 10.0]);
        assert!(
            (one_go - in_pieces).abs() < 1.0,
            "{one_go} in one go against {in_pieces} in four"
        );

        let there_and_back = dragged_from_the_middle(2.0, &[40.0, -40.0]);
        assert!(
            (there_and_back - 50.0).abs() < 1.0,
            "it should have come back to where it started, not {there_and_back}"
        );
    }

    /// The press is still a jump, which is what keeps the far end of a long
    /// range one gesture away rather than a journey.
    #[test]
    fn a_press_puts_the_handle_under_the_pointer() {
        let _one_at_a_time = ONE_AT_A_TIME.lock();

        let pressed = dragged_from_the_middle(10.0, &[]);
        assert!(
            (pressed - 50.0).abs() < 2.0,
            "the middle of the rail is the middle of the range, not {pressed}"
        );
    }

    /// The mailbox holds one ask and empties when it is read, so a row pressed
    /// once is not acted on twice.
    #[test]
    fn an_ask_is_taken_once() {
        let _one_at_a_time = ONE_AT_A_TIME.lock();
        let _ = asked();

        ask(Ask::Travel(5.0));
        assert_eq!(asked(), Some(Ask::Travel(5.0)));
        assert_eq!(asked(), None);

        ask(Ask::Settings);
        assert_eq!(asked(), Some(Ask::Settings));
        assert_eq!(asked(), None);
    }

    /// What the program hands out is what the rails read.
    #[test]
    fn the_travel_is_the_one_the_program_handed_out() {
        let _one_at_a_time = ONE_AT_A_TIME.lock();

        travels(4.5);
        assert_eq!(travel(), 4.5);
        travels(drag::SHIPS_AS);
        assert_eq!(travel(), drag::SHIPS_AS);
    }
}
