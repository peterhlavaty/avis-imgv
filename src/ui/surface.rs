//! Making a thing on screen answer the second button.
//!
//! One helper, used by every surface that carries a menu, so that all of them
//! behave the same way and none of them can drift.
//!
//! Two things it does that `Response::context_menu` does not. It opens on the
//! *press* rather than on the release — Microsoft's toolbar guidance, which
//! removes the ambiguity with a drag at its source instead of tuning a
//! threshold; `Popup::context_menu` keys off `secondary_clicked()`, which is
//! reported on the release. And it draws the same small chevron on hover and
//! appends the same four words to the hover text, so that a surface either says
//! it has a menu or does not have one. NN/g's finding, applied to a second
//! affordance: because only *some* things carried tooltips, people stopped
//! expecting them and missed the ones that existed.
//!
//! And every menu it draws opens with a line naming what was clicked, because
//! the menu itself is drawn over that thing and the verbs in it are worded for
//! a reader who already knows: "Show only these" is three different sentences
//! depending on which badge in the bar was under the pointer.

use eframe::egui::{self, PointerButton, Response, RichText};

/// The words every surface with a menu ends its hover text with.
///
/// *More* rather than "set it": most of these menus carry verbs as well as
/// settings, and a promise of settings on a menu of verbs is a promise broken.
pub const SAYS: &str = "Right-click for more.";

/// The chevron drawn in the corner of a surface that has a menu.
const CHEVRON: &str = "⌄";

/// How wide a menu is allowed to be.
///
/// Here rather than at each menu because the heading is truncated against it:
/// a menu whose width came from the longest thing in it would be as wide as a
/// path, and the two places that already chose a width chose this one.
pub const WIDEST: f32 = 320.0;

/// What a menu is about: the kind of thing, and which one of them.
///
/// Two fields rather than one sentence, so that every heading in the program
/// is built the same way — the kind in the weak colour and which one in the
/// strong — and no caller invents its own punctuation between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subject<'a> {
    kind: &'a str,
    which: &'a str,
}

impl<'a> Subject<'a> {
    /// A thing of a kind, and which one it is: a keyword, and the word.
    pub fn of(kind: &'a str, which: &'a str) -> Self {
        Subject { kind, which }
    }

    /// A thing there is only one of on the screen: a panel, a heading, a word
    /// in the bar that is either itself or absent.
    pub fn the(kind: &'a str) -> Self {
        Subject { kind, which: "" }
    }

    /// The whole of it on one line, for the hover and for the tests.
    pub fn said(&self) -> String {
        if self.which.is_empty() {
            self.kind.to_string()
        } else {
            format!("{} — {}", self.kind, self.which)
        }
    }
}

/// The first row of every menu: what was clicked to open it.
///
/// A menu opens at the pointer and covers the thing it was asked about, and
/// several of the things worth asking about are a glyph a few pixels wide with
/// a neighbour that looks much like it — the flag, the colour and the rating
/// sit together in the bottom bar, the five swatches in the tag panel differ
/// only by colour, and a metadata row's menu offers to copy a value it never
/// names. Always first, never varying: the mirror of [`more_settings`], which
/// is always last, and drawn here rather than by the callers so that it cannot
/// be forgotten by one of them or worded differently by another.
fn about(ui: &mut egui::Ui, subject: Subject<'_>) {
    ui.scope(|ui| {
        // The width is bounded for the heading alone. A long name is then
        // truncated here rather than deciding how wide the whole menu is, and
        // nothing about the rows below this changes.
        ui.set_max_width(WIDEST);

        ui.horizontal(|ui| {
            // Lined up with the text of the rows, which carry a button's
            // padding and would otherwise start further in than the heading.
            ui.add_space(ui.spacing().button_padding.x);
            ui.label(RichText::new(subject.kind).weak().small());

            if !subject.which.is_empty() {
                ui.add(egui::Label::new(RichText::new(subject.which).strong().small()).truncate())
                    .on_hover_text(subject.which);
            }
        });
    });

    ui.separator();
}

/// Marks a response as carrying a menu, and draws the menu when it is asked for.
///
/// `hint` is the sentence under the pointer, without the trailing words: those
/// are added here so every surface says the same thing.
pub fn with_menu<R>(
    ui: &egui::Ui,
    response: &Response,
    subject: Subject<'_>,
    hint: &str,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    chevron(ui, response);

    let hint = if hint.is_empty() {
        SAYS.to_string()
    } else {
        format!("{hint} {SAYS}")
    };
    response.clone().on_hover_text(hint);

    menu(ui, response, subject, contents)
}

/// Which surface the keyboard asked for a menu on.
///
/// `Shift + F10` is the only keyboard route to a menu — egui cannot read the
/// dedicated Menu key at all, since its key list runs F1 to F35 and has no
/// entry for it. The popup's own id is derived from the response and cannot be
/// invented, so the ask is recorded by name and each surface claims it.
static KEYBOARD_ASKED: std::sync::Mutex<Option<&'static str>> = std::sync::Mutex::new(None);

/// Records that the keyboard asked for the menu of a named surface.
pub fn ask_for_menu(surface: &'static str) {
    if let Ok(mut asked) = KEYBOARD_ASKED.lock() {
        *asked = Some(surface);
    }
}

/// Whether this surface is the one that was asked for, taking the ask.
fn claimed(surface: &str) -> bool {
    let Ok(mut asked) = KEYBOARD_ASKED.lock() else {
        return false;
    };

    if *asked == Some(surface) {
        *asked = None;
        return true;
    }

    false
}

/// The menu alone, for a surface whose hover text is drawn elsewhere.
pub fn menu<R>(
    ui: &egui::Ui,
    response: &Response,
    subject: Subject<'_>,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    named_menu(ui, response, "", subject, contents)
}

/// The same, for a surface the keyboard can reach by name.
pub fn named_menu<R>(
    ui: &egui::Ui,
    response: &Response,
    surface: &'static str,
    subject: Subject<'_>,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<R> {
    // On the press, not the release. A drag that begins on this surface and a
    // menu that opens under the pointer are then two different gestures rather
    // than the same one told apart by how long it lasted.
    //
    // And where the press landed, rather than what is hovered. egui empties the
    // hover set while anything at all is being dragged, and a scroll area
    // whose content has outgrown it registers a drag-to-scroll surface over
    // the whole of itself, which senses drag alone and is therefore marked as
    // dragged on the frame *any* button goes down — the second one included.
    // So every menu inside a list stopped opening on the day the list grew
    // long enough to scroll, which is a fault that arrives with use rather
    // than with the code and reads as the button having broken.
    //
    // `is_pointer_button_down_on` is the top-most *click*-sensing widget the
    // press landed on, which is the question a menu is asking, and it answers
    // no for a disabled panel or a layer under a window in front exactly as
    // `hovered` does: egui strikes the sense off a widget that is either.
    let pressed = (response.is_pointer_button_down_on()
        && ui
            .ctx()
            .input(|i| i.pointer.button_pressed(PointerButton::Secondary)))
        || (!surface.is_empty() && claimed(surface));

    let open = if pressed {
        Some(egui::SetOpenCommand::Bool(true))
    } else if response.clicked() {
        // A left click closes it, or the menu would stay up over whatever the
        // click just did.
        Some(egui::SetOpenCommand::Bool(false))
    } else {
        None
    };

    // The other half of opening on the press: the *release* of that same
    // gesture is a click, and a menu whose default is "close on a click
    // outside" would shut itself the instant the button came up. So while the
    // second button is down — or has just come up — no menu here closes on a
    // click. A left click elsewhere still closes it on any later frame.
    let own_gesture = ui.ctx().input(|i| {
        i.pointer.button_down(PointerButton::Secondary)
            || i.pointer.button_released(PointerButton::Secondary)
    });

    let behaviour = if own_gesture {
        egui::PopupCloseBehavior::IgnoreClicks
    } else {
        egui::PopupCloseBehavior::CloseOnClickOutside
    };

    egui::Popup::menu(response)
        .open_memory(open)
        .close_behavior(behaviour)
        .at_pointer_fixed()
        .show(|ui| {
            about(ui, subject);
            contents(ui)
        })
        .map(|inner| inner.inner)
}

/// Draws the chevron in the corner of a surface that carries a menu.
///
/// Uniform or none: the same six points in the weak text colour on every one of
/// them.
fn chevron(ui: &egui::Ui, response: &Response) {
    if !response.hovered() {
        return;
    }

    let rect = response.rect;
    if rect.width() < 12.0 || rect.height() < 8.0 {
        return;
    }

    ui.painter().text(
        egui::pos2(rect.right() - 3.0, rect.bottom() - 3.0),
        egui::Align2::RIGHT_BOTTOM,
        CHEVRON,
        egui::FontId::proportional(6.0),
        ui.visuals().weak_text_color(),
    );
}

/// Whether the built-in menus draw their settings rows.
///
/// A process-wide flag rather than a parameter threaded through twenty menus:
/// it is one decision the whole program agrees about, and the surfaces that
/// draw menus are scattered across a dozen files that have no configuration in
/// hand.
static SETTINGS_ROWS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Sets it from the configuration.
pub fn show_settings_rows(on: bool) {
    SETTINGS_ROWS.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// The last row of every menu: where the settings for this object live.
///
/// Always last, never varying, never removed — unless `menus.settings_rows` is
/// off, which leaves the verbs and takes nothing away that is not reachable
/// elsewhere. It is what makes the menu a route to the page rather than a
/// second place the same decisions are made, and it is why nothing in the
/// program is reachable *only* by right-click.
pub fn more_settings(ui: &mut egui::Ui, page: crate::config::registry::Page) -> bool {
    if !SETTINGS_ROWS.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }

    ui.separator();

    ui.button(RichText::new(format!("More settings…  ({})", page.label())))
        .on_hover_text("Opens the settings window on that page")
        .clicked()
}

/// A row that binds a key to whatever this menu is about.
///
/// Makes the control the route to its own key, which closes the loop the
/// keyboard editor otherwise owns alone — the direct answer to the request to
/// "change keybinds by ctrl+right clicking on the menu and picking them on the
/// GUI instead of having to edit configs".
pub fn bind_a_key(ui: &mut egui::Ui, what: &str) -> bool {
    if !SETTINGS_ROWS.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }

    ui.button(format!("Bind a key to {what}…"))
        .on_hover_text("Opens the keyboard editor with that row armed")
        .clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The same four words on every surface, so a person can learn them once.
    #[test]
    fn the_hover_text_always_ends_the_same_way() {
        assert!(SAYS.ends_with("more."));
        assert!(SAYS.starts_with("Right-click"));
    }

    /// Both halves are said, in that order, with one punctuation between them.
    #[test]
    fn a_subject_says_the_kind_and_which_one() {
        assert_eq!(Subject::of("Keyword", "Tatras").said(), "Keyword — Tatras");
        assert_eq!(Subject::of("Rating", "3/5").said(), "Rating — 3/5");
    }

    /// A thing there is only one of says only what it is, with no dangling
    /// dash after it.
    #[test]
    fn a_subject_of_one_thing_says_only_what_it_is() {
        assert_eq!(Subject::the("The history").said(), "The history");
        assert!(!Subject::the("Watching").said().contains('—'));
    }

    /// An empty half is the same as not having one, which is what lets a
    /// caller pass a name it computed and may not have.
    #[test]
    fn an_empty_half_is_no_half() {
        assert_eq!(Subject::of("Flag", ""), Subject::the("Flag"));
        assert_eq!(Subject::of("Flag", "").said(), "Flag");
    }

    /// The heading is drawn, both halves of it, before whatever the menu
    /// carries — which is the whole of what this file promises about it.
    #[test]
    fn the_heading_is_drawn_above_the_rows() {
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                about(ui, Subject::of("Keyword", "Tatras"));
                let _ = ui.button("Show only this");
            });
        });

        let drawn = painted(&output);
        let heading = drawn
            .iter()
            .position(|text| text == "Keyword")
            .expect("the kind is drawn");
        let which = drawn
            .iter()
            .position(|text| text == "Tatras")
            .expect("which one it is, is drawn");
        let row = drawn
            .iter()
            .position(|text| text == "Show only this")
            .expect("the row is drawn");

        assert!(heading < row, "the heading comes first: {drawn:?}");
        assert!(which < row, "and so does which one it is: {drawn:?}");
    }

    /// The fault this was written for: a menu inside a list stopped opening on
    /// the day the list grew long enough to scroll.
    ///
    /// It reached the history panel first, because that list is the one that
    /// grows on its own — a hundred and sixty-eight rows after a fortnight —
    /// and the button had visibly worked the week before. The cause is a
    /// surface egui adds and nothing here asks for: a scroll area whose
    /// content has outgrown it registers a drag-to-scroll rectangle over the
    /// whole of itself, senses drag alone on it, and is therefore *dragged*
    /// from the frame any button goes down. While something is dragged egui
    /// hovers nothing else, so every row under the pointer answered no.
    #[test]
    fn a_menu_opens_in_a_list_long_enough_to_scroll() {
        assert!(
            menu_opened_on_a_right_click(40, true),
            "a list taller than the space it is drawn in"
        );
        assert!(
            menu_opened_on_a_right_click(3, true),
            "and one that fits, which is how it looked when it was written"
        );
    }

    /// The half of `hovered` worth keeping. A window in front owns the mouse,
    /// and a panel behind it draws itself disabled — from which no menu opens,
    /// however the press is read.
    #[test]
    fn a_disabled_surface_answers_nothing() {
        assert!(!menu_opened_on_a_right_click(40, false));
        assert!(!menu_opened_on_a_right_click(3, false));
    }

    /// Right-clicks the one surface in a list of `rows` rows, drawn in a panel
    /// two hundred points tall, and answers whether its menu opened.
    fn menu_opened_on_a_right_click(rows: usize, enabled: bool) -> bool {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0));
        let mut where_it_is = egui::Rect::ZERO;

        let draw = |ctx: &egui::Context, where_it_is: &mut egui::Rect| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if !enabled {
                    ui.disable();
                }

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for row in 0..rows {
                            let response = ui.selectable_label(false, format!("row {row}"));

                            if row == 0 {
                                *where_it_is = response.rect;
                                menu(ui, &response, Subject::the("A row"), |ui| {
                                    let _ = ui.button("Do the thing");
                                });
                            }
                        }
                    });
            });
        };

        let input = |events: Vec<egui::Event>| egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };

        // Twice, so that the hit test the press is decided by has a frame of
        // rectangles behind it.
        for _ in 0..2 {
            let _ = ctx.run(input(Vec::new()), |ctx| draw(ctx, &mut where_it_is));
        }

        let at = where_it_is.center();
        let moved = || vec![egui::Event::PointerMoved(at)];
        let _ = ctx.run(input(moved()), |ctx| draw(ctx, &mut where_it_is));

        let mut press = moved();
        press.push(egui::Event::PointerButton {
            pos: at,
            button: PointerButton::Secondary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        });
        let _ = ctx.run(input(press), |ctx| draw(ctx, &mut where_it_is));

        // The popup is laid out on the frame after the one that opened it.
        let output = ctx.run(input(moved()), |ctx| draw(ctx, &mut where_it_is));

        painted(&output).iter().any(|text| text == "Do the thing")
    }

    /// Every piece of text the frame painted, in the order it was painted.
    fn painted(output: &egui::FullOutput) -> Vec<String> {
        output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_string()),
                _ => None,
            })
            .collect()
    }
}
