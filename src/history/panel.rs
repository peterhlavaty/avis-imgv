//! The list of what was done, and the two ways of getting back into it.
//!
//! Drawn here rather than in `src/ui/` because a directory in this program is
//! named for a job it does and holds the logic, the drawing and the tests for
//! that job together; `src/ui/` is for what several concerns genuinely share.
//!
//! The list is in the order things happened, which is the order somebody
//! remembers doing them, rather than in the order of the tree. Where the tree
//! shows is the indent: a branch is one level in, and everything that follows
//! it stays at that level. Indenting by depth instead would push a day's work
//! off the side of the panel, every deed being a child of the one before it.

use eframe::egui::{self, RichText};

use super::{Entry, NodeId};

/// What a click in the panel asked for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// Take the program back — or forward — to this row.
    GoTo(NodeId),
    /// Do this one thing again, as a new row at the end.
    Repeat(NodeId),
    /// The panel was dragged to this width.
    ///
    /// Reported rather than written here, because what is written is the
    /// configuration file and this module does not have one.
    Width(f32),
}

/// What the panel remembers between frames.
#[derive(Debug, Default)]
pub struct State {
    /// Which row was where we were, last time it was drawn.
    ///
    /// Kept so the list scrolls to the cursor when the cursor moves, and only
    /// then: scrolling on every frame would fight anybody reading further up.
    showing: Option<NodeId>,
    /// How wide the panel was on the frame before, so a drag is read back once
    /// it has finished rather than while it is happening.
    width: crate::ui::dragged::Dragged,
}

/// Draws the panel and reports what was clicked.
pub fn ui(
    ctx: &egui::Context,
    visible: bool,
    width: f32,
    forced: bool,
    state: &mut State,
    history: &super::History,
) -> Vec<Action> {
    let mut actions = Vec::new();

    let mut panel = egui::SidePanel::right("history_panel")
        .resizable(true)
        .show_separator_line(false)
        .default_width(width)
        .min_width(180.0);

    // `default_width` is honoured only while egui has no width of its own for
    // this panel, which it does from the first frame on — so a width typed
    // into the settings window did nothing until the next launch. For the one
    // frame after it changes, the width is stated rather than suggested.
    if forced {
        panel = panel.exact_width(width);
    }

    let panel = panel.show_animated(ctx, visible, |ui| {
        // A panel reports the rectangle its contents came to, not the one
        // the drag asked for, so the scroll area has to be told to fill
        // what it was given or the edge springs back.
        ui.set_min_width(ui.available_width());

        // A window in front owns the pointer, and a hit test knows nothing
        // about modal layers.
        if crate::utils::is_in_front(ui.ctx()) {
            ui.disable();
        }

        ui.add_space(4.0);
        // The heading on a line of its own. Sharing one with a settings
        // route left "History" drawn as "His" against a narrow panel, the
        // right-to-left layout beside it having taken the whole width.
        ui.heading("History");
        ui.label(RichText::new(count(history)).weak());
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                rows(ui, state, history, &mut actions);
            });

        crate::ui::panel::menu(ui, &CHROME, |_| {});
    });

    // A dragged edge is a setting the user has just changed, and every other
    // one of those in this program is written back rather than lost on exit.
    // Only when it has actually moved: half a point of drift would be a write
    // of the configuration file on every frame the panel is up.
    match panel {
        Some(panel) => {
            let held = ctx.input(|i| i.pointer.any_down());

            if let Some(settled) = state
                .width
                .settled(panel.response.rect.width(), width, held)
            {
                actions.push(Action::Width(settled));
            }
        }
        // Shut. Opening it again animates from nothing, and comparing across
        // that would read the animation as a drag.
        None => state.width.forget(),
    }

    actions
}

/// The line under the heading: how much is behind and how much is in front.
fn count(history: &super::History) -> String {
    let done = history.tree().depth(history.cursor());
    let total = history.len();

    match (total, total - done.min(total)) {
        (0, _) => "Nothing yet".to_string(),
        (1, 0) => "1 thing done".to_string(),
        (total, 0) => format!("{total} things done"),
        (1, _) => "1 thing done · 1 taken back".to_string(),
        (total, ahead) => format!("{total} things done · {ahead} taken back"),
    }
}

/// Every row, in the order the things happened.
fn rows(ui: &mut egui::Ui, state: &mut State, history: &super::History, actions: &mut Vec<Action>) {
    let tree = history.tree();
    let cursor = history.cursor();
    let moved = state.showing != Some(cursor);

    for (id, node) in tree.in_order() {
        let here = id == cursor;
        // Everything from the beginning up to where we are has been done;
        // anything else is either taken back or on a branch not taken.
        let done = tree.is_ancestor(id, cursor);

        ui.horizontal(|ui| {
            ui.add_space(tree.branch_depth(id) as f32 * 12.0);
            row(ui, id, node, here, done, actions);
        });

        if here && moved {
            ui.scroll_to_cursor(Some(egui::Align::Center));
        }
    }

    state.showing = Some(cursor);
}

/// What this panel says for itself, for the menu every panel carries.
///
/// One rule rather than two, because a menu opened on a row and a menu opened
/// on the space beside it are the same menu as far as the panel is concerned,
/// and two copies would be two things to keep in step.
pub const CHROME: crate::ui::panel::Chrome<'static> = crate::ui::panel::Chrome {
    subject: crate::ui::surface::Subject::the("The history panel"),
    hide: Some(crate::app::input::Command::ToggleHistoryPanel),
    key: Some("history.sc_panel"),
    page: crate::config::registry::Page::History,
    setting: "history.panel_visible",
};

/// One row: what it was, whether it still stands, and its menu.
fn row(
    ui: &mut egui::Ui,
    id: NodeId,
    node: &super::Node<Entry>,
    here: bool,
    done: bool,
    actions: &mut Vec<Action>,
) {
    let mut text = RichText::new(&node.value.label);

    if !done {
        // Taken back, or on a branch that was left. Still there, still
        // clickable — that is the whole promise — but plainly not in force.
        text = text.weak().italics();
    }
    if here {
        text = text.strong();
    }

    // Truncated to whatever the panel is wide, with an ellipsis, rather than
    // wrapped: a row is one line, and a photograph's name is the half of it
    // most likely not to fit. The whole of it is on the hover, so nothing is
    // lost — which is why `Entry::label` keeps the full text and the drawing
    // is what decides how much of it there is room for.
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

    let label = ui.selectable_label(here, text);

    let label = label.on_hover_text(match here {
        true => format!("{}\nWhere you are now.", node.value.label),
        false => format!(
            "{}\nClick to go back to just after this.\n{}",
            node.value.label,
            when(node)
        ),
    });

    if label.clicked() && !here {
        actions.push(Action::GoTo(id));
    }

    // The row's own words at the top of its menu: the panel holds a hundred
    // of them, they are truncated to whatever it is wide, and the menu opens
    // over the one it was asked for.
    crate::ui::surface::menu(
        ui,
        &label,
        crate::ui::surface::Subject::of("What you did", &node.value.label),
        |ui| {
            ui.set_max_width(crate::ui::surface::WIDEST);

            if ui
                .add_enabled(!here, egui::Button::new("Go back to this"))
                .clicked()
            {
                actions.push(Action::GoTo(id));
                ui.close();
            }

            if ui
                .button("Do only this again")
                .on_hover_text("Carries this one out where you are now, and adds it to the end.")
                .clicked()
            {
                actions.push(Action::Repeat(id));
                ui.close();
            }

            ui.separator();
            crate::ui::panel::rows(ui, &CHROME);
        },
    );
}

/// How long ago a row happened, in the words somebody would use.
fn when(node: &super::Node<Entry>) -> String {
    let Ok(ago) = node.value.at.elapsed() else {
        return "just now".to_string();
    };

    match ago.as_secs() {
        0..=5 => "just now".to_string(),
        seconds @ 6..=59 => format!("{seconds} seconds ago"),
        seconds @ 60..=3599 => format!("{} minutes ago", seconds / 60),
        seconds => format!("{} hours ago", seconds / 3600),
    }
}

/// Drawn in place of the panel when there is nothing to show it in.
///
/// The toggle has to change a pixel whatever the state, or the key looks
/// broken on a fresh start.
pub fn nothing_yet(ctx: &egui::Context, visible: bool, width: f32) {
    egui::SidePanel::right("history_panel")
        .resizable(false)
        .show_separator_line(false)
        .default_width(width)
        .show_animated(ctx, visible, |ui| {
            if crate::utils::is_in_front(ui.ctx()) {
                ui.disable();
            }

            ui.add_space(8.0);
            ui.heading("History");
            ui.add_space(6.0);
            ui.label(RichText::new("Nothing has been done yet.").weak());

            crate::ui::panel::menu(ui, &CHROME, |_| {});
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{Deed, History, Step};
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    fn binned(name: &str) -> Deed {
        Deed::Files(Step::Binned(vec![PathBuf::from(name)]))
    }

    fn entry(at: SystemTime) -> super::super::Node<Entry> {
        super::super::Node {
            parent: None,
            children: Vec::new(),
            preferred: None,
            value: Entry {
                deed: Deed::Start,
                at,
                label: "x".to_string(),
            },
        }
    }

    #[test]
    fn an_empty_history_says_so() {
        assert_eq!(count(&History::new()), "Nothing yet");
    }

    /// One of a thing is one thing. "1 things done" was on screen.
    #[test]
    fn one_thing_is_not_things() {
        let mut history = History::new();
        history.record(binned("a.jpg"));

        assert_eq!(count(&history), "1 thing done");

        let route = history.plan_undo(|_| true);
        history.arrive(history.landing(&route).unwrap());

        assert_eq!(count(&history), "1 thing done · 1 taken back");
    }

    #[test]
    fn the_count_says_how_much_is_behind_and_how_much_in_front() {
        let mut history = History::new();
        history.record(binned("a.jpg"));
        history.record(binned("b.jpg"));

        assert_eq!(count(&history), "2 things done");

        // Having gone back one, one is in front of us again.
        let route = history.plan_undo(|_| true);
        history.arrive(history.landing(&route).unwrap());

        assert_eq!(count(&history), "2 things done · 1 taken back");
    }

    /// The list never loses a row: what was taken back is drawn differently
    /// but is still there to be clicked.
    #[test]
    fn what_was_taken_back_is_still_in_the_list() {
        let mut history = History::new();
        let a = history.record(binned("a.jpg")).unwrap();

        let route = history.plan_undo(|_| true);
        history.arrive(history.landing(&route).unwrap());

        assert!(history.entry(a).is_some());
        assert!(
            !history.tree().is_ancestor(a, history.cursor()),
            "and it is drawn as no longer standing"
        );
    }

    #[test]
    fn how_long_ago_is_said_in_the_right_unit() {
        let now = SystemTime::now();

        assert_eq!(when(&entry(now)), "just now");
        assert_eq!(
            when(&entry(now - Duration::from_secs(30))),
            "30 seconds ago"
        );
        assert_eq!(
            when(&entry(now - Duration::from_secs(120))),
            "2 minutes ago"
        );
        assert_eq!(when(&entry(now - Duration::from_secs(7200))), "2 hours ago");
    }

    /// A clock that has gone backwards since the row was made must not panic
    /// or say something absurd.
    #[test]
    fn a_row_from_the_future_says_just_now() {
        assert_eq!(
            when(&entry(SystemTime::now() + Duration::from_secs(600))),
            "just now"
        );
    }
}
