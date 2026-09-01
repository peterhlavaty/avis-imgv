//! The rating and tagging panel.
//!
//! A resizable side panel holding the star rating for the open image, the tags
//! on it, the tags used most recently, and the configured catalog — searchable
//! by tag name or by category.

pub mod model;

use eframe::egui::{self, RichText};

use crate::metadata::xmp::{leaf_of, Flag, Label, MAX_RATING};

pub use model::{rows_under, sections, Row, Sections, Source};

/// A filled and an empty star, drawn side by side to make a rating.
const FILLED: &str = "★";
const EMPTY: &str = "☆";

/// Shown on a tag that clicking would remove. A multiplication sign rather
/// than a cross, because the bundled font has it.
const REMOVE: &str = "×";

/// The blue the contact sheet marks a selection with, so the panel saying
/// how many it will touch reads as part of the same thing.
const SELECTED: egui::Color32 = egui::Color32::from_rgb(126, 168, 224);

/// How far one level of a tag tree is indented, in points.
const LEVEL: f32 = 12.;

/// What the user asked for by clicking in the panel.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SetRating(u8),
    SetFlag(Flag),
    /// A colour label by its position in [`Label::CHOICES`], or none.
    SetLabel(Option<usize>),
    AddTag(String),
    RemoveTag(String),
    /// The panel was dragged to this width.
    ///
    /// Reported so it reaches the configuration field the settings window
    /// reads, rather than being a gesture the viewer forgets on the way out.
    PanelWidth(f32),
    /// Narrow the folder to the photographs carrying this mark.
    ///
    /// The panel draws every mark the program has, and none of them could be
    /// acted on beyond putting it on or taking it off. "Show me the ones I
    /// gave three stars" was a thing you could see and not ask for.
    ShowOnlyStars(u8),
    ShowOnlyFlag(Flag),
    ShowOnlyLabel(usize),
    ShowOnlyKeyword(String),
    /// Go to the settings row behind a mark.
    Settings(&'static str),
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
/// Draws the panel greyed, for when there is no photograph to talk about.
///
/// The panel used to return before drawing anything, so pressing its key on an
/// empty folder changed no pixel and looked like a key that did nothing.
pub fn nothing_open(ctx: &egui::Context, visible: bool, width: f32) {
    egui::SidePanel::left("tag_panel")
        .resizable(true)
        .show_separator_line(false)
        .default_width(width)
        .min_width(180.)
        .show_animated(ctx, visible, |ui| {
            ui.add_space(20.);
            ui.label(RichText::new("Rating & Tags").heading());
            ui.add_space(10.);
            ui.weak("No photograph open. Stars, flags, colours and keywords go on one.");
        });
}

pub fn ui(
    ctx: &egui::Context,
    visible: bool,
    width: f32,
    state: &mut State,
    source: &Source<'_>,
) -> Vec<Action> {
    let mut actions = Vec::new();

    let panel = egui::SidePanel::left("tag_panel")
        .resizable(true)
        .show_separator_line(false)
        .default_width(width)
        .min_width(180.)
        .show_animated(ctx, visible, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(20.);
                ui.label(RichText::new("Rating & Tags").heading());

                if source.applies_to > 1 {
                    ui.label(
                        RichText::new(format!("Applies to {} photographs", source.applies_to))
                            .color(SELECTED),
                    );
                }

                ui.add_space(10.);

                actions.extend(stars(ui, source.annotations.stars()));
                actions.extend(flags(ui, source.annotations.flag()));
                actions.extend(labels(ui, source.annotations.known_label()));
                ui.add_space(10.);

                let sections = sections(source, &state.search);
                actions.extend(on_image(ui, &sections));

                ui.add_space(10.);
                ui.separator();
                search_box(ui, state);

                actions.extend(offered(ui, &sections));
            });
        });

    // The dragged width, reported back so it can be written to the field the
    // settings window reads. It was a gesture the viewer forgot on the way out.
    if let Some(panel) = panel {
        let dragged = panel.response.rect.width();
        if (dragged - width).abs() > 1.0 {
            actions.push(Action::PanelWidth(dragged));
        }
    }

    actions
}

/// The star row. Clicking the star already at the end of the rating clears it,
/// which is how every photo application behaves.
fn stars(ui: &mut egui::Ui, rating: u8) -> Vec<Action> {
    let rating = rating as i8;
    let mut actions = Vec::new();

    ui.horizontal(|ui| {
        for star in 1..=MAX_RATING {
            let filled = star <= rating;
            let label = RichText::new(if filled { FILLED } else { EMPTY }).size(22.);

            let button = ui.add(egui::Button::new(label).frame(false));

            crate::ui::surface::with_menu(ui, &button, &format!("{star} star(s)."), |ui| {
                if ui
                    .button(format!("Show only {star} stars and better"))
                    .clicked()
                {
                    actions.push(Action::ShowOnlyStars(star.max(0) as u8));
                    ui.close();
                }
            });

            if button.clicked() {
                let wanted = if star == rating { 0 } else { star };
                actions.push(Action::SetRating(wanted.max(0) as u8));
            }
        }

        if rating > 0 {
            ui.label(format!("{rating}/{MAX_RATING}"));
        }
    });

    actions
}

/// Keep, throw out, or neither. Clicking the mark already on the image takes
/// it off, the same as pressing its key twice.
fn flags(ui: &mut egui::Ui, current: Flag) -> Vec<Action> {
    let mut actions = Vec::new();

    ui.horizontal(|ui| {
        for (flag, label, hint) in [
            (Flag::Picked, "⚑ Keep", "Mark this one as a keeper"),
            (Flag::Rejected, "✖ Reject", "Mark this one to be thrown out"),
        ] {
            let chosen = current == flag;
            let button = ui.selectable_label(chosen, label);

            crate::ui::surface::with_menu(ui, &button, hint, |ui| {
                if ui.button("Show only these").clicked() {
                    actions.push(Action::ShowOnlyFlag(flag));
                    ui.close();
                }
            });

            if button.clicked() {
                actions.push(Action::SetFlag(if chosen { Flag::Unflagged } else { flag }));
            }
        }
    });

    actions
}

/// The five colour labels, as swatches. Clicking the one already set clears it.
fn labels(ui: &mut egui::Ui, current: Option<Label>) -> Vec<Action> {
    let mut actions = Vec::new();

    ui.add_space(4.);
    ui.horizontal(|ui| {
        for (index, label) in Label::CHOICES.iter().enumerate() {
            let (r, g, b) = label.colour();
            let chosen = current == Some(*label);
            let glyph = if chosen { "■" } else { "□" };

            let swatch = RichText::new(glyph)
                .size(20.)
                .color(egui::Color32::from_rgb(r, g, b));

            let button = ui.add(egui::Button::new(swatch).frame(false));

            crate::ui::surface::with_menu(ui, &button, label.name(), |ui| {
                if ui.button("Show only these").clicked() {
                    actions.push(Action::ShowOnlyLabel(index));
                    ui.close();
                }
            });

            if button.clicked() {
                actions.push(Action::SetLabel((!chosen).then_some(index)));
            }
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
            // The leaf is the keyword; the path above it is context, so it is
            // there to be read on hover rather than taking up the whole panel.
            let response = ui.selectable_label(true, format!("{} {REMOVE}", leaf_of(tag)));

            crate::ui::surface::with_menu(ui, &response, &hover(tag, "Remove"), |ui| {
                if ui
                    .button(format!("Show only \"{}\"", leaf_of(tag)))
                    .clicked()
                {
                    actions.push(Action::ShowOnlyKeyword(leaf_of(tag).to_string()));
                    ui.close();
                }

                if crate::ui::surface::more_settings(ui, crate::config::registry::Page::Keywords) {
                    // Keywords have their own page; the chip is the shortest
                    // route to it.
                    actions.push(Action::Settings("tags.categories"));
                    ui.close();
                }
            });

            if response.clicked() {
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
///
/// Flat keywords wrap as chips, which fits a great many of them into a narrow
/// panel. Keywords with levels are drawn as a tree instead: forty of them
/// wrapped into a paragraph is a wall of words in which the same leaf appears
/// under two different parents with nothing to tell them apart.
fn chips(ui: &mut egui::Ui, title: &str, tags: &[String]) -> Vec<Action> {
    let mut actions = Vec::new();

    if tags.is_empty() {
        return actions;
    }

    ui.add_space(8.);
    ui.label(RichText::new(title).strong());

    if tags.iter().any(|tag| leaf_of(tag) != tag) {
        return tree(ui, title, tags);
    }

    ui.horizontal_wrapped(|ui| {
        for tag in tags {
            if ui.selectable_label(false, tag).clicked() {
                actions.push(Action::AddTag(tag.clone()));
            }
        }
    });

    actions
}

/// Tags with levels, one to a line and indented by depth.
fn tree(ui: &mut egui::Ui, title: &str, tags: &[String]) -> Vec<Action> {
    let mut actions = Vec::new();

    for row in rows_under(title, tags) {
        ui.horizontal(|ui| {
            ui.add_space(row.depth as f32 * LEVEL);

            let response = ui
                .selectable_label(false, &row.leaf)
                .on_hover_text(hover(&row.path, "Add"));

            if response.clicked() {
                actions.push(Action::AddTag(row.path.clone()));
            }
        });
    }

    actions
}

/// What to say when the pointer rests on a tag: the whole path, when there is
/// more to it than the word on screen.
fn hover(tag: &str, verb: &str) -> String {
    if leaf_of(tag) == tag {
        return verb.to_string();
    }

    format!("{verb}  ·  {tag}")
}
