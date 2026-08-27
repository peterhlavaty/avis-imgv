//! The rating and tagging panel.
//!
//! A resizable side panel holding the star rating for the open image, the tags
//! on it, the tags used most recently, and the configured catalog — searchable
//! by tag name or by category.

pub mod model;

use eframe::egui::{self, RichText};

use crate::metadata::xmp::MAX_RATING;

pub use model::{sections, Sections, Source};

/// A filled and an empty star, drawn side by side to make a rating.
const FILLED: &str = "★";
const EMPTY: &str = "☆";

/// Shown on a tag that clicking would remove. A multiplication sign rather
/// than a cross, because the bundled font has it.
const REMOVE: &str = "×";

/// What the user asked for by clicking in the panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    SetRating(u8),
    AddTag(String),
    RemoveTag(String),
}

/// The panel's own state, which the application owns between frames.
#[derive(Debug, Default)]
pub struct State {
    /// Contents of the search box.
    pub search: String,
}

/// Draws the panel and reports what was clicked.
///
/// Nothing is drawn while `visible` is false, and the panel animates in and
/// out with it.
pub fn ui(
    ctx: &egui::Context,
    visible: bool,
    width: f32,
    state: &mut State,
    source: &Source<'_>,
) -> Vec<Action> {
    let mut actions = Vec::new();

    egui::SidePanel::left("tag_panel")
        .resizable(true)
        .show_separator_line(false)
        .default_width(width)
        .min_width(180.)
        .show_animated(ctx, visible, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(20.);
                ui.label(RichText::new("Rating & Tags").heading());
                ui.add_space(10.);

                actions.extend(stars(ui, source.annotations.rating));
                ui.add_space(10.);

                let sections = sections(source, &state.search);
                actions.extend(on_image(ui, &sections));

                ui.add_space(10.);
                ui.separator();
                search_box(ui, state);

                actions.extend(offered(ui, &sections));
            });
        });

    actions
}

/// The star row. Clicking the star already at the end of the rating clears it,
/// which is how every photo application behaves.
fn stars(ui: &mut egui::Ui, rating: u8) -> Vec<Action> {
    let mut actions = Vec::new();

    ui.horizontal(|ui| {
        for star in 1..=MAX_RATING {
            let filled = star <= rating;
            let label = RichText::new(if filled { FILLED } else { EMPTY }).size(22.);

            if ui
                .add(egui::Button::new(label).frame(false))
                .on_hover_text(format!("{star} star(s)"))
                .clicked()
            {
                let wanted = if star == rating { 0 } else { star };
                actions.push(Action::SetRating(wanted));
            }
        }

        if rating > 0 {
            ui.label(format!("{rating}/{MAX_RATING}"));
        }
    });

    actions
}

/// The tags already on the image, each removable.
fn on_image(ui: &mut egui::Ui, sections: &Sections) -> Vec<Action> {
    let mut actions = Vec::new();

    if sections.on_image.is_empty() {
        ui.label(RichText::new("No tags on this image").weak());
        return actions;
    }

    ui.horizontal_wrapped(|ui| {
        for tag in &sections.on_image {
            if ui
                .selectable_label(true, format!("{tag} {REMOVE}"))
                .on_hover_text("Remove")
                .clicked()
            {
                actions.push(Action::RemoveTag(tag.clone()));
            }
        }
    });

    actions
}

fn search_box(ui: &mut egui::Ui, state: &mut State) {
    ui.add_space(6.);
    ui.add(
        egui::TextEdit::singleline(&mut state.search)
            .hint_text("Search tags or categories")
            .desired_width(ui.available_width()),
    );
    ui.add_space(6.);
}

/// Everything that can be added: the search text as a new tag, the recently
/// used list, the catalog, and anything seen elsewhere in the folder.
fn offered(ui: &mut egui::Ui, sections: &Sections) -> Vec<Action> {
    let mut actions = Vec::new();

    if let Some(new_tag) = &sections.create {
        if ui.button(format!("+ Add \"{new_tag}\"")).clicked() {
            actions.push(Action::AddTag(new_tag.clone()));
        }
    }

    actions.extend(chips(ui, "Recent", &sections.recent));

    for group in &sections.categories {
        actions.extend(chips(ui, &group.category, &group.tags));
    }

    actions.extend(chips(ui, "Seen in this folder", &sections.seen));

    if sections.is_empty() && sections.create.is_none() {
        ui.add_space(6.);
        ui.label(RichText::new("Nothing matches").weak());
    }

    actions
}

/// One titled row of clickable tags.
fn chips(ui: &mut egui::Ui, title: &str, tags: &[String]) -> Vec<Action> {
    let mut actions = Vec::new();

    if tags.is_empty() {
        return actions;
    }

    ui.add_space(8.);
    ui.label(RichText::new(title).strong());

    ui.horizontal_wrapped(|ui| {
        for tag in tags {
            if ui.selectable_label(false, tag).clicked() {
                actions.push(Action::AddTag(tag.clone()));
            }
        }
    });

    actions
}
