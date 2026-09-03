//! The settings window.
//!
//! `App::show_keyboard` with a longer body: a window with the configuration in
//! hand, a fan-out when something changed, and a save. No new file format, no
//! second viewport, no shadow state.
//!
//! Every page is a filter over the registry, so nothing here decides what a
//! field is called, what it means, what control it wants or when it takes
//! effect — the table does, and this draws it.

pub mod footer;
mod lists;
mod problems;
pub mod widgets;

use eframe::egui::{self, RichText};

use crate::config::registry::{self, Complaint, Page, Row};
use crate::config::Config;

pub use footer::Run;

/// The window's own state.
#[derive(Debug, Default)]
pub struct State {
    /// Which page is on screen.
    pub page: Option<Page>,
    /// What is being searched for.
    pub query: String,
    /// Set on the frame the window opens, so the search box takes the cursor.
    pub just_opened: bool,
    /// A row to scroll to and flash, from a **[Fix]** button or a link.
    pub reveal: Option<&'static str>,
    /// What is wrong with the file, read once at load.
    pub problems: Vec<Complaint>,
    /// What was said at startup and has since faded: the migration report and
    /// the key clashes. Six seconds is not long enough to read a warning that
    /// two commands are on one key, and it could not be recovered at all.
    pub at_startup: Vec<String>,
    /// Which reset scope has been asked for and not confirmed.
    pub confirming: Option<Reset>,
    /// A key the window was asked to change, from a row's own menu.
    pub arm_key: Option<&'static str>,
    /// What has been changed that is waiting for a restart, if anything.
    ///
    /// Named rather than counted: "one change needs a restart" without saying
    /// which is a sentence that cannot be acted on. The application fills this
    /// in, because only it knows what the running program was started with.
    pub waiting_on_a_restart: Option<&'static str>,
}

/// How much a reset covers. Always stated, never implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reset {
    Page(Page),
    Everything,
}

/// What the window did this frame.
#[derive(Debug, Default)]
pub struct Outcome {
    /// A value moved.
    pub changed: bool,
    /// A gesture ended, so a rebuild-bound change may be carried out.
    pub committed: bool,
    /// A button was pressed.
    pub run: Option<Run>,
}

/// Draws the window.
pub fn show(
    ctx: &egui::Context,
    open: &mut bool,
    state: &mut State,
    config: &mut Config,
) -> Outcome {
    let mut outcome = Outcome::default();

    if !*open {
        state.just_opened = true;
        return outcome;
    }

    // 900 by 600 comfortable and 720 by 480 the floor, which fits the 1092 by
    // 614 logical space of a 1366 by 768 laptop at 125 per cent. darktable's
    // preferences window put its Close button below the bottom of a 14-inch
    // screen, where it could not be reached at all.
    let shown = egui::Window::new("Settings")
        .open(open)
        .default_size([900.0, 600.0])
        .min_size([720.0, 480.0])
        .resizable(true)
        .show(ctx, |ui| {
            outcome = contents(ui, state, config);
        });

    // Nothing behind it is clicked, dragged or scrolled while it is up.
    crate::utils::in_front(ctx, shown.as_ref());

    outcome
}

/// The band that says something is waiting for a restart, and offers one.
///
/// Persistent while it is true rather than a message that fades: the change has
/// *not* taken effect, and a toast that has gone leaves somebody looking at a
/// number that is not the number in force. darktable's own fix for this
/// complaint was a toast on closing the dialogue; a button that does the thing
/// is strictly better.
fn restart_band(ui: &mut egui::Ui, state: &State) -> Option<Run> {
    let waiting = state.waiting_on_a_restart?;
    let mut asked = None;

    egui::Frame::group(ui.style())
        .fill(ui.visuals().warn_fg_color.gamma_multiply(0.12))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} One change needs a restart: {waiting}.",
                        crate::config::registry::Effect::BADGE
                    ))
                    .color(ui.visuals().warn_fg_color),
                );

                if ui
                    .button("Restart now")
                    .on_hover_text("Saves where you are and starts the viewer again")
                    .clicked()
                {
                    asked = Some(Run::Restart);
                }
            });
        });

    ui.add_space(6.0);
    asked
}

fn contents(ui: &mut egui::Ui, state: &mut State, config: &mut Config) -> Outcome {
    let mut outcome = Outcome::default();

    if let Some(run) = restart_band(ui, state) {
        outcome.run = Some(run);
    }

    if let Some(run) = problems::band(ui, state, config) {
        match run {
            problems::Asked::Fix(path) => {
                state.reveal = Some(path);
                if let Some(row) = registry::row(path) {
                    state.page = Some(row.page);
                    state.query.clear();
                }
            }
            problems::Asked::Dismiss => state.problems.clear(),
        }
    }

    ui.horizontal_top(|ui| {
        navigation(ui, state, config);
        ui.separator();

        // A vertical child first: a `ScrollArea` inherits the layout it is
        // given, so one drawn straight into a horizontal row lays its contents
        // out left to right — and `indent` refuses that outright.
        ui.vertical(|ui| {
            // Disabled rather than hidden while the file is only partly read:
            // nothing can be saved for the rest of the session, and greying the
            // whole window out would force somebody to look on every other page
            // to find out why. Microsoft's rule, and its reason.
            ui.add_enabled_ui(!config.partial, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let touched = if state.query.trim().is_empty() {
                            page(ui, state, config)
                        } else {
                            results(ui, state, config)
                        };

                        outcome.changed |= touched.changed;
                        outcome.committed |= touched.committed;

                        ui.add_space(16.0);
                        ui.separator();
                        let footer = footer::ui(ui, state, config);
                        outcome.run = footer.run;
                        outcome.changed |= footer.changed;
                        outcome.committed |= footer.committed;
                    });
            });
        });
    });

    outcome
}

/// The list of pages down the left, with the search box above it.
fn navigation(ui: &mut egui::Ui, state: &mut State, config: &Config) {
    ui.vertical(|ui| {
        ui.set_width(NAVIGATION_WIDTH);

        let field = ui.add(
            egui::TextEdit::singleline(&mut state.query)
                .hint_text("Search every setting")
                .desired_width(f32::INFINITY),
        );

        // The cursor goes here when the window opens: somebody who knows what
        // they want should not have to find the page first.
        if std::mem::take(&mut state.just_opened) {
            field.request_focus();
        }

        ui.add_space(8.0);

        // Read once: the list is a column of one width, and a row that asked
        // for its own would be a row narrower than the one above it.
        let width = ui.available_width();

        for wanted in Page::ALL {
            let on = state.page == Some(*wanted) && state.query.trim().is_empty();
            let changed = registry::on_page(*wanted)
                .filter(|row| row.access.is_writable())
                .filter(|row| row.changed(config))
                .count();

            let label = if changed > 0 {
                format!("{}  ({changed})", wanted.label())
            } else {
                wanted.label().to_string()
            };

            let picked = ui
                .add(navigation_row(on, label, width))
                .on_hover_text(if changed > 0 {
                    format!(
                        "{}  Â·  {changed} changed from the default",
                        wanted.sentence()
                    )
                } else {
                    wanted.sentence().to_string()
                });

            if picked.clicked() {
                state.page = Some(*wanted);
                state.query.clear();
            }
        }
    });
}

const NAVIGATION_WIDTH: f32 = 210.0;

/// One row of the navigation list, the whole width of the list.
///
/// A category is a place to go, and the row is what is pointed at: a press an
/// inch to the right of a short name landing on nothing is a miss nobody
/// attributes to their aim, and the hover then lights only the words rather
/// than the row, which says the words are the button. `min_size` rather than a
/// justified layout, because a button takes its alignment from the layout it
/// is in and a justified one would centre every name.
fn navigation_row(on: bool, label: String, width: f32) -> egui::Button<'static> {
    egui::Button::selectable(on, label).min_size(egui::vec2(width, 0.0))
}

/// One page.
fn page(ui: &mut egui::Ui, state: &mut State, config: &mut Config) -> widgets::Touched {
    let page = state.page.unwrap_or(Page::OpeningAFolder);
    let mut touched = widgets::Touched::default();

    ui.heading(page.label());
    ui.weak(page.sentence());
    ui.add_space(10.0);

    let rows: Vec<&'static Row> = registry::on_page(page).collect();
    let mut group = None;

    for row in rows {
        if Some(row.group) != group {
            group = Some(row.group);

            if let Some(heading) = row.group.label() {
                ui.add_space(8.0);
                ui.label(RichText::new(heading).strong());
                ui.add_space(4.0);
            }
        }

        let scrolled = state.reveal == Some(row.path);
        let response = ui.scope(|ui| {
            if scrolled {
                ui.visuals_mut().override_text_color = Some(ui.visuals().warn_fg_color);
            }
            widgets::row(ui, row, config)
        });

        if scrolled {
            response.response.scroll_to_me(Some(egui::Align::Center));
            state.reveal = None;
        }

        let (found, key) = response.inner;
        if let Some(path) = key {
            // "Change this key…" on a row of the keyboard page: the control is
            // the route to its own key.
            state.arm_key = Some(path);
        }

        touched = widgets::Touched {
            changed: touched.changed || found.changed,
            committed: touched.committed || found.committed,
        };
    }

    ui.add_space(10.0);
    reset_scope(ui, state, config, page, &mut touched);

    touched
}

/// The rows a query found, drawn the same way so a value changes in the result.
fn results(ui: &mut egui::Ui, state: &mut State, config: &mut Config) -> widgets::Touched {
    let hits = registry::search::find(&state.query);
    let mut touched = widgets::Touched::default();

    if hits.is_empty() {
        ui.heading("Nothing matched");
        ui.add_space(6.0);
        ui.weak(
            "No setting matches that. The whole file is JSON and can be opened from the \
             footer below.",
        );
        return touched;
    }

    // Never an empty result: a failed AND is re-run as an OR under a line
    // saying so, because "no results" is the one answer a search box may not
    // give.
    if !hits[0].matched_everything {
        ui.heading("Nothing matched all of that");
        ui.weak("The closest:");
    } else {
        ui.heading(format!("{} settings", hits.len()));
    }

    ui.add_space(10.0);

    for hit in hits {
        ui.weak(RichText::new(hit.row.page.label()).small());
        let (found, key) = widgets::row(ui, hit.row, config);

        if let Some(path) = key {
            state.arm_key = Some(path);
        }

        touched = widgets::Touched {
            changed: touched.changed || found.changed,
            committed: touched.committed || found.committed,
        };
    }

    touched
}

/// The reset buttons under a page.
///
/// Always with a scope: this setting (the bullet beside each row), this page,
/// or everything. The page and everything scopes say what they would change
/// before writing anything.
fn reset_scope(
    ui: &mut egui::Ui,
    state: &mut State,
    config: &mut Config,
    page: Page,
    touched: &mut widgets::Touched,
) {
    let changed: Vec<&'static Row> = registry::on_page(page)
        .filter(|row| row.access.is_writable())
        .filter(|row| row.changed(config))
        .collect();

    if changed.is_empty() {
        return;
    }

    ui.separator();

    if state.confirming == Some(Reset::Page(page)) {
        ui.label(
            RichText::new(format!(
                "Put these {} back to their defaults?",
                changed.len()
            ))
            .color(ui.visuals().warn_fg_color),
        );

        // Named by its sentence rather than its path, and with what it would
        // become: a reset that does not say what it is resetting to is a leap.
        let fresh = Config::default();
        ui.indent("reset preview", |ui| {
            for row in &changed {
                ui.weak(RichText::new(format!("{} → {}", row.label, shown(row, &fresh))).small());
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Yes, put them back").clicked() {
                for row in &changed {
                    row.access.reset(config);
                }
                state.confirming = None;
                touched.changed = true;
                touched.committed = true;
            }
            if ui.button("Leave them").clicked() {
                state.confirming = None;
            }
        });

        return;
    }

    if ui
        .button(format!("Put this page's {} back", changed.len()))
        .on_hover_text("Says what it would change before it changes anything")
        .clicked()
    {
        state.confirming = Some(Reset::Page(page));
    }
}

/// A row's value as a short string, for a from→to preview.
pub fn shown(row: &Row, config: &Config) -> String {
    if let Some(value) = row.access.boolean(config) {
        return if value { "on" } else { "off" }.to_string();
    }
    if let Some(value) = row.access.int(config) {
        return value.to_string();
    }
    if let Some(value) = row.access.float(config) {
        return format!("{value}");
    }
    if let Some(value) = row.access.choice(config) {
        return value.to_string();
    }
    if let Some(value) = row.access.text(config) {
        return if value.is_empty() {
            "nothing".to_string()
        } else {
            value
        };
    }
    if let Some(shortcut) = row.access.shortcut(config) {
        return crate::ui::keys::describe(shortcut);
    }

    "—".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every page has rows on it, or the navigation list leads somewhere empty.
    #[test]
    fn no_page_is_empty() {
        for page in Page::ALL {
            let rows = registry::on_page(*page).count();
            assert!(rows > 0, "{} has no rows", page.label());
        }
    }

    /// And every row is drawn somewhere.
    #[test]
    fn every_row_is_on_a_page() {
        let drawn: usize = Page::ALL
            .iter()
            .map(|page| registry::on_page(*page).count())
            .sum();

        assert_eq!(drawn, registry::rows().len());
    }

    /// The whole row is the category, not the words on it — and it stops at
    /// the edge of the list.
    #[test]
    fn a_category_answers_a_press_beside_its_name() {
        assert!(pressed_at(8.0), "the name itself");
        assert!(pressed_at(NAVIGATION_WIDTH - 8.0), "the far end of the row");
        assert!(!pressed_at(NAVIGATION_WIDTH + 40.0), "past the list");
    }

    /// Draws one navigation row in a list `NAVIGATION_WIDTH` wide inside a
    /// wider window, and answers whether a press `x` from the row's left edge
    /// was a press on it.
    fn pressed_at(x: f32) -> bool {
        let ctx = egui::Context::default();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0));

        let at = std::cell::Cell::new(egui::Pos2::ZERO);
        let clicked = std::cell::Cell::new(false);

        let draw = |ctx: &egui::Context| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.set_width(NAVIGATION_WIDTH);
                    let width = ui.available_width();
                    let row = ui.add(navigation_row(false, "Caching".to_string(), width));
                    at.set(egui::pos2(row.rect.left() + x, row.rect.center().y));
                    clicked.set(clicked.get() || row.clicked());
                });
            });
        };

        let input = |events: Vec<egui::Event>| egui::RawInput {
            screen_rect: Some(screen),
            events,
            ..Default::default()
        };

        // Twice, so the hit test the press is decided by has a frame of
        // rectangles behind it — and so `at` is read off a row that was drawn.
        for _ in 0..2 {
            let _ = ctx.run(input(Vec::new()), draw);
        }

        let button = |pressed| egui::Event::PointerButton {
            pos: at.get(),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        };

        let _ = ctx.run(
            input(vec![egui::Event::PointerMoved(at.get()), button(true)]),
            draw,
        );
        let _ = ctx.run(input(vec![button(false)]), draw);

        clicked.get()
    }

    #[test]
    fn a_value_reads_back_as_something_worth_showing() {
        let config = Config::default();

        for row in registry::rows() {
            let said = shown(row, &config);
            assert!(!said.is_empty(), "{} shows nothing", row.path);
        }
    }
}
