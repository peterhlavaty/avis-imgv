//! Putting one watched thing back where it was — or forward where it went.
//!
//! Every arm reads the same way: take the half the direction asks for, and set
//! the program to it. Never "flip it", which is the same thing as "make it
//! what it was" only for as long as nothing else has flipped it in between,
//! and the whole point of a history is that things do.
//!
//! Each of these goes through the ordinary route rather than writing a field:
//! `set_mode` because arriving at a mode does more than name it, `go_to`
//! because both views follow the cursor, `apply_narrowing` because what is
//! shown is derived from the rules and has to be derived again.

use crate::history::{Change, Panels, Way};

use super::super::App;

impl App {
    /// Sets the program to one side of one change.
    pub(super) fn restore(&mut self, change: &Change, way: Way) {
        match change {
            Change::Folder(from, to) => {
                let target = pick(way, from, to).clone();
                self.watcher.restart(&target, self.flattened);
                self.open_directory(&target, None);
            }
            Change::Mode(from, to) => self.set_mode(*pick(way, from, to)),
            Change::Panels(from, to) => {
                // Taken apart field by field rather than read one at a time,
                // because a struct pattern with no `..` is exhaustive: adding a
                // panel to `Panels` now fails to compile until it is put back
                // here as well. The history panel itself was added to the
                // struct and to the half that *reads* it and not to this one,
                // so it was recorded faithfully and then never restored — the
                // panel could be opened and closed, and no route through the
                // history could shut it.
                let Panels {
                    menu,
                    side,
                    metrics,
                    tags,
                    filter,
                    filmstrip,
                    history,
                } = *pick(way, from, to);

                self.menu_visible = menu;
                self.side_panel_visible = side;
                self.metrics_visible = metrics;
                self.tag_panel_visible = tags;
                self.filter_visible = filter;
                self.filmstrip_visible = filmstrip;
                self.history_panel_visible = history;
            }
            Change::Cursor { from, to, .. } => self.go_to(*pick(way, from, to)),
            Change::Place { from, to, .. } => self.image_view.set_place(*pick(way, from, to)),
            Change::Columns(from, to) => self.grid_view.set_columns(*pick(way, from, to)),
            Change::Flattened(from, to) => self.set_flattened(*pick(way, from, to)),
            Change::Advancing(from, to) => self.advancing = *pick(way, from, to),
            Change::Selection(from, to) => {
                self.grid_view
                    .set_selection(pick(way, from, to).as_ref().clone());
            }
            Change::Narrowing(from, to) => {
                self.narrowing = pick(way, from, to).as_ref().clone();
                self.apply_narrowing();
            }
            Change::Settings(from, to) => {
                self.settings = pick(way, from, to).as_ref().clone();
                // Everything the configuration reaches is rebuilt from it, so
                // a setting put back takes effect on this frame exactly as it
                // would have done from the window.
                self.apply_settings();
                self.save_settings();
            }
        }
    }
}

/// The half of a change the direction asks for.
///
/// Going back wants where it came from, going forward wants where it went.
fn pick<'a, T>(way: Way, from: &'a T, to: &'a T) -> &'a T {
    match way {
        Way::Back => from,
        Way::Forward => to,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one rule the whole file is: back takes the first, forward the
    /// second. Every arm above depends on this and none of them repeats it.
    #[test]
    fn back_takes_where_it_came_from_and_forward_where_it_went() {
        assert_eq!(pick(Way::Back, &"was", &"is"), &"was");
        assert_eq!(pick(Way::Forward, &"was", &"is"), &"is");
    }
}
