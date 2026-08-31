//! The panel that proposes groups and lets them be corrected.
//!
//! Everything the detector decided is a suggestion the user can overrule: the
//! kind of each group, whether it is a group at all, and which frames are in
//! it. A frame taken out of a group goes back to the loose pile at the bottom,
//! and can be put into any group from there.

use eframe::egui::{self, Color32, RichText};

use crate::organize::gather;
use crate::organize::group::{Group, Kind};

use super::thumbnails;
use super::{Done, OrganizeView};

mod edit;

use edit::{apply_change, Change};

pub(super) use edit::regroup as regrouped;

pub fn show(ui: &mut egui::Ui, view: &mut OrganizeView) -> Option<Done> {
    settings(ui, view);
    view.regroup_if_stale();

    let done = actions(ui, view);

    ui.add_space(6.0);

    let mut change = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for index in 0..view.groups.len() {
                if let Some(asked) = one_group(ui, view, index) {
                    change = Some(asked);
                }
            }

            if let Some(asked) = loose(ui, view) {
                change = Some(asked);
            }
        });

    if let Some(change) = change {
        apply_change(view, change);
    }

    done
}

fn settings(ui: &mut egui::Ui, view: &mut OrganizeView) {
    let before = view.grouping;

    ui.horizontal(|ui| {
        ui.label("A group breaks after a gap of:");
        ui.add(
            egui::DragValue::new(&mut view.grouping.max_gap)
                .range(1.0..=3600.0)
                .suffix(" s"),
        );

        ui.label("Same scene within:");
        ui.add(egui::DragValue::new(&mut view.grouping.tolerance).range(0..=64))
            .on_hover_text(
                "How different two thumbnails may be and still count as the same \
                 view. Zero is identical; sixty-four accepts anything.",
            );

        ui.label("At least:");
        ui.add(egui::DragValue::new(&mut view.grouping.min_frames).range(2..=50));
        ui.label("frames");

        ui.separator();
        ui.label("Thumbnails:");
        egui::ComboBox::from_id_salt("organize thumbnail size")
            .selected_text(thumbnails::label(view.thumbnail_height))
            .show_ui(ui, |ui| {
                for (label, height) in thumbnails::SIZES {
                    ui.selectable_value(&mut view.thumbnail_height, *height, *label);
                }
            });
    });

    if before != view.grouping {
        view.groups_stale = true;
    }
}

fn actions(ui: &mut egui::Ui, view: &mut OrganizeView) -> Option<Done> {
    let planned = gather::plan(&view.groups, &view.folder());
    let files: usize = planned.iter().map(|plan| plan.moves.len()).sum();

    let mut done = None;

    ui.horizontal(|ui| {
        let button = egui::Button::new(format!("Tidy {} group(s) into folders", planned.len()));

        if ui.add_enabled(!planned.is_empty(), button).clicked() {
            let outcome = gather::apply(&planned);

            view.status = outcome.summary();
            for (path, problem) in &outcome.failed {
                tracing::warn!("Could not move {}: {problem}", path.display());
            }

            done = Some(Done::Renamed);
        }

        if files > 0 {
            ui.weak(format!("{files} file(s)"));
        }

        if !view.status.is_empty() {
            ui.weak(&view.status);
        }
    });

    done
}

fn one_group(ui: &mut egui::Ui, view: &mut OrganizeView, index: usize) -> Option<Change> {
    let mut change = None;

    let heading = {
        let group = &view.groups[index];
        format!("{}  ·  {}", folder_name(view, index), group.describe())
    };

    egui::CollapsingHeader::new(heading)
        .id_salt(("organize group", index))
        .default_open(view.groups.len() <= 8)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("This is a:");

                let group = &mut view.groups[index];
                egui::ComboBox::from_id_salt(("organize kind", index))
                    .selected_text(group.kind.label())
                    .show_ui(ui, |ui| {
                        for kind in Kind::ALL {
                            ui.selectable_value(&mut group.kind, *kind, kind.label());
                        }
                    });

                if let Some(note) = retyped_note(group) {
                    ui.label(note);
                }

                if ui
                    .button("Not a group")
                    .on_hover_text("Break it up and put its frames back in the loose pile")
                    .clicked()
                {
                    change = Some(Change::Dissolve(index));
                }
            });

            let height = view.thumbnail_height;

            // The frames of a group are what the eye compares, so they go
            // side by side and wrap rather than down a list.
            ui.horizontal_wrapped(|ui| {
                for member in 0..view.groups[index].members.len() {
                    let entry = view.groups[index].members[member].clone();

                    ui.vertical(|ui| {
                        view.thumbnails.show(
                            ui,
                            &entry.path,
                            entry.thumbnail.as_ref(),
                            entry.orientation(),
                            height,
                        );

                        ui.horizontal(|ui| {
                            if ui.small_button("×").on_hover_text("Take out").clicked() {
                                change = Some(Change::Remove {
                                    group: index,
                                    member,
                                });
                            }

                            ui.label(entry.name()).on_hover_text(
                                entry
                                    .captured()
                                    .map(|taken| taken.to_exif())
                                    .unwrap_or_else(|| "no capture time".to_string()),
                            );
                        });
                    });
                }
            });
        });

    change
}

fn loose(ui: &mut egui::Ui, view: &mut OrganizeView) -> Option<Change> {
    if view.loose.is_empty() {
        return None;
    }

    let mut change = None;

    ui.add_space(8.0);
    egui::CollapsingHeader::new(format!("Not in any group  ·  {} frames", view.loose.len()))
        .id_salt("organize loose")
        .default_open(false)
        .show(ui, |ui| {
            let height = view.thumbnail_height;

            ui.horizontal_wrapped(|ui| {
                for index in 0..view.loose.len() {
                    let entry = view.loose[index].clone();

                    ui.vertical(|ui| {
                        view.thumbnails.show(
                            ui,
                            &entry.path,
                            entry.thumbnail.as_ref(),
                            entry.orientation(),
                            height,
                        );

                        ui.horizontal(|ui| {
                            ui.label(entry.name()).on_hover_text(
                                entry
                                    .captured()
                                    .map(|taken| taken.to_exif())
                                    .unwrap_or_else(|| "no capture time".to_string()),
                            );

                            if view.groups.is_empty() {
                                return;
                            }

                            ui.menu_button("Put into…", |ui| {
                                for group in 0..view.groups.len() {
                                    let label = format!(
                                        "{}  ·  {}",
                                        folder_name(view, group),
                                        view.groups[group].describe()
                                    );

                                    if ui.button(label).clicked() {
                                        change = Some(Change::Add {
                                            group,
                                            loose: index,
                                        });
                                        ui.close();
                                    }
                                }
                            });
                        });
                    });
                }
            });
        });

    change
}

/// The folder this group would be tidied into, as the header shows it.
///
/// Worked out from the whole list rather than from the group alone, because
/// the number depends on how many of its kind come before it.
fn folder_name(view: &OrganizeView, index: usize) -> String {
    gather::plan(&view.groups, &view.folder())
        .into_iter()
        .find(|plan| plan.group == index)
        .map(|plan| plan.name())
        .unwrap_or_else(|| view.groups[index].kind.folder().to_string())
}

/// The colour of the note saying the detector was overruled.
const RETYPED: Color32 = Color32::from_rgb(150, 190, 230);

/// The note itself, saying what the group was read as before.
fn retyped_note(group: &Group) -> Option<RichText> {
    group
        .was_retyped()
        .then(|| RichText::new(format!("(read as {})", group.detected.label())).color(RETYPED))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organize::group::test_support::frame;

    #[test]
    fn a_retyped_group_says_what_it_was_read_as() {
        let mut group = Group::new(Kind::Series, vec![frame("a.jpg", 0, 1)]);
        assert!(retyped_note(&group).is_none());

        group.kind = Kind::Hdr;
        assert!(retyped_note(&group).is_some());
    }
}
