//! What is under the separator: the files, the version, what has been changed,
//! and the buttons.
//!
//! Below a separator so it does not read as a twelfth page. Every one of these
//! is a registry `Run` row, so they are searchable like everything else —
//! "config file" has to be a query that lands somewhere.

use eframe::egui::{self, RichText};

use crate::actions::reveal;
use crate::config::registry::{self, Row};
use crate::config::Config;

use super::{shown, Reset, State};

/// A button the footer was asked to press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    OpenConfigFile,
    ShowConfigFolder,
    OpenLogFile,
    Restart,
    ExportChanges,
    ImportChanges,
}

/// What the footer did.
#[derive(Debug, Default)]
pub struct Outcome {
    pub run: Option<Run>,
    pub changed: bool,
    pub committed: bool,
}

pub fn ui(ui: &mut egui::Ui, state: &mut State, config: &mut Config) -> Outcome {
    let mut outcome = Outcome::default();

    ui.add_space(6.0);

    file_row(
        ui,
        "Settings",
        Config::path(),
        Run::OpenConfigFile,
        Run::ShowConfigFolder,
        &mut outcome,
    );
    file_row(
        ui,
        "Log",
        crate::logging::path(),
        Run::OpenLogFile,
        Run::OpenLogFile,
        &mut outcome,
    );

    ui.horizontal(|ui| {
        ui.label("File version:");
        ui.weak(config.version.to_string())
            .on_hover_text("The one key here nobody should change by hand");
    });

    ui.add_space(8.0);
    changed_list(ui, state, config, &mut outcome);

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui
            .button("Save what I have changed…")
            .on_hover_text(
                "Writes only the fields that differ from the defaults: a small file that \
                 goes into version control and onto the other two machines. A bundle \
                 written by an older build stays valid, because the fields it does not \
                 know about are the fields it does not name.",
            )
            .clicked()
        {
            outcome.run = Some(Run::ExportChanges);
        }

        if ui
            .button("Load a saved file…")
            .on_hover_text("Says what it would change before changing anything")
            .clicked()
        {
            outcome.run = Some(Run::ImportChanges);
        }

        if ui
            .button("Restart now")
            .on_hover_text("Saves where you are and starts the viewer again")
            .clicked()
        {
            outcome.run = Some(Run::Restart);
        }
    });

    outcome
}

/// One path, selectable, with the two ways of getting at it.
fn file_row(
    ui: &mut egui::Ui,
    name: &str,
    path: Option<std::path::PathBuf>,
    open: Run,
    show: Run,
    outcome: &mut Outcome,
) {
    ui.horizontal(|ui| {
        ui.label(format!("{name}:"));

        let Some(path) = path else {
            ui.weak("no configuration directory on this system");
            return;
        };

        let text = path.display().to_string();
        ui.add(egui::Label::new(RichText::new(&text).monospace()).wrap());

        if ui.small_button("Copy").clicked() {
            ui.ctx().copy_text(text);
        }
        if ui.small_button("Open").clicked() {
            outcome.run = Some(open);
        }
        if ui.small_button("Show me the folder").clicked() {
            outcome.run = Some(show);
        }
    });
}

/// Everything that differs from a fresh configuration, with its page.
///
/// The answer to "I changed something and I do not remember where", which
/// otherwise costs eleven pages.
fn changed_list(ui: &mut egui::Ui, state: &mut State, config: &mut Config, outcome: &mut Outcome) {
    let changed: Vec<&'static Row> = registry::rows()
        .iter()
        .filter(|row| row.access.is_writable())
        .filter(|row| row.changed(config))
        .collect();

    egui::CollapsingHeader::new(format!("What I have changed ({})", changed.len()))
        .default_open(false)
        .show(ui, |ui| {
            if changed.is_empty() {
                ui.weak("Nothing. This is a fresh configuration.");
                return;
            }

            for row in &changed {
                ui.horizontal(|ui| {
                    ui.weak(RichText::new(row.page.label()).small());
                    ui.label(row.label);
                    ui.weak(RichText::new(shown(row, config)).monospace().small());

                    if ui.small_button("Go to it").clicked() {
                        state.page = Some(row.page);
                        state.query.clear();
                        state.reveal = Some(row.path);
                    }
                });
            }

            ui.add_space(8.0);
            everything(ui, state, config, changed.len(), outcome);
        });
}

/// The global reset, which writes a backup first.
fn everything(
    ui: &mut egui::Ui,
    state: &mut State,
    config: &mut Config,
    count: usize,
    outcome: &mut Outcome,
) {
    if state.confirming == Some(Reset::Everything) {
        ui.label(
            RichText::new(format!("Put all {count} back to their defaults?"))
                .color(ui.visuals().warn_fg_color),
        );
        ui.weak("The file is copied to config.json.bak first.");

        ui.horizontal(|ui| {
            if ui.button("Yes, all of them").clicked() {
                back_up();

                for row in registry::rows() {
                    if row.access.is_writable() {
                        row.access.reset(config);
                    }
                }

                state.confirming = None;
                outcome.changed = true;
                outcome.committed = true;
            }

            if ui.button("Leave them").clicked() {
                state.confirming = None;
            }
        });

        return;
    }

    if ui
        .button("Put everything back to the defaults")
        .on_hover_text("Copies the file to config.json.bak first")
        .clicked()
    {
        state.confirming = Some(Reset::Everything);
    }
}

/// Copies the configuration beside itself before it is written over.
///
/// Two XnView users asked for exactly this when the same feature was requested
/// there, and the author's answer was "It's the same as deleting xnview.ini, is
/// it really needed?" — which is true only for somebody who knows where
/// xnview.ini is.
fn back_up() {
    let Some(path) = Config::path() else {
        return;
    };

    let backup = path.with_extension("json.bak");
    if let Err(e) = std::fs::copy(&path, &backup) {
        tracing::warn!("Could not back the configuration up: {e}");
    }
}

/// Does what a footer button asked for.
pub fn carry_out(run: Run, config: &Config) -> Result<String, String> {
    match run {
        Run::OpenConfigFile => open(Config::path(), "settings"),
        Run::ShowConfigFolder => match Config::path() {
            Some(path) => {
                reveal::in_file_manager(&path);
                Ok(String::new())
            }
            None => Err("There is no configuration directory on this system.".to_string()),
        },
        Run::OpenLogFile => open(crate::logging::path(), "log"),
        Run::ExportChanges => export(config),
        Run::ImportChanges => Err(String::new()),
        Run::Restart => Ok(String::new()),
    }
}

fn open(path: Option<std::path::PathBuf>, name: &str) -> Result<String, String> {
    let Some(path) = path else {
        return Err(format!("There is no {name} file on this system."));
    };

    if reveal::with_the_system(&path) {
        Ok(String::new())
    } else {
        Err(format!("Could not open {}.", path.display()))
    }
}

/// Writes out only what differs from the defaults.
///
/// A patch rather than a snapshot, so a bundle written by an older build stays
/// valid: the fields it does not know about are the fields it does not name.
/// Key bindings are opted into and never included by default — a shared file
/// that silently rebinds `x` is the complaint about every settings-sharing
/// feature in miniature — and machine-specific paths are left out for the same
/// reason.
pub fn changes_of(config: &Config, include_keys: bool, include_paths: bool) -> serde_json::Value {
    let mut document = serde_json::Map::new();

    let whole = serde_json::to_value(config).unwrap_or(serde_json::Value::Null);
    let fresh = serde_json::to_value(Config::default()).unwrap_or(serde_json::Value::Null);

    for row in registry::rows() {
        if !row.access.is_writable() || row.path.contains('[') {
            continue;
        }

        if row.access.is_a_key() && !include_keys {
            continue;
        }

        if !include_paths && MACHINE_SPECIFIC.contains(&row.path) {
            continue;
        }

        if !row.changed(config) {
            continue;
        }

        let (Some(section), key) = (row.path.split_once('.'), row.key()) else {
            continue;
        };

        let Some(value) = whole.get(section.0).and_then(|s| s.get(key)) else {
            continue;
        };

        // Belt and braces: a row that says it changed but whose serialised
        // value equals the default is not written, so an export is a patch and
        // nothing else.
        if fresh.get(section.0).and_then(|s| s.get(key)) == Some(value) {
            continue;
        }

        document
            .entry(section.0.to_string())
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .expect("just inserted an object")
            .insert(key.to_string(), value.clone());
    }

    serde_json::Value::Object(document)
}

/// Fields whose value is about this machine and not about this person.
const MACHINE_SPECIFIC: &[&str] = &[
    "cull.destinations",
    "cull.rejected_folder",
    "tags.catalog_file",
    "general.start_folder",
    "image_view.user_actions",
    "image_view.context_menu",
    "grid_view.context_menu",
];

fn export(config: &Config) -> Result<String, String> {
    let Some(directory) = Config::path().and_then(|path| path.parent().map(|p| p.join("bundles")))
    else {
        return Err("There is no configuration directory on this system.".to_string());
    };

    if let Err(e) = std::fs::create_dir_all(&directory) {
        return Err(format!("Could not make {}: {e}", directory.display()));
    }

    let Some(path) = rfd::FileDialog::new()
        .set_directory(&directory)
        .set_file_name("my-settings.json")
        .add_filter("settings", &["json"])
        .save_file()
    else {
        return Ok(String::new());
    };

    let document = changes_of(config, false, false);
    let json = serde_json::to_string_pretty(&document)
        .map_err(|e| format!("Could not build the file: {e}"))?;

    match crate::atomic::replace(&path, json.as_bytes()) {
        Ok(()) => Ok(format!("Wrote {}.", path.display())),
        Err(e) => Err(format!("Could not write {}: {e}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh configuration exports nothing, which is what makes the file a
    /// patch rather than a snapshot.
    #[test]
    fn a_fresh_configuration_has_nothing_to_export() {
        let document = changes_of(&Config::default(), true, true);

        assert_eq!(document.as_object().map(|o| o.len()), Some(0));
    }

    #[test]
    fn only_what_changed_is_written() {
        let mut config = Config::default();
        config.cache.ram_budget_mb = 8192;
        config.grid_view.cell_aspect = 1.0;

        let document = changes_of(&config, false, false);
        let object = document.as_object().expect("an object");

        assert_eq!(object.len(), 2);
        assert_eq!(document["cache"]["ram_budget_mb"], 8192);
        assert_eq!(document["grid_view"]["cell_aspect"], 1.0);
    }

    /// A shared file that silently rebinds a key is the complaint about every
    /// settings-sharing feature there is, so keys are opted into.
    #[test]
    fn keys_are_left_out_unless_they_are_asked_for() {
        let mut config = Config::default();
        config.general.sc_exit = crate::config::Shortcut::new("F9", &[]);

        assert!(changes_of(&config, false, false)
            .as_object()
            .expect("an object")
            .is_empty());

        assert!(!changes_of(&config, true, false)
            .as_object()
            .expect("an object")
            .is_empty());
    }

    /// And so are the paths, which are about a machine rather than a person.
    #[test]
    fn machine_paths_are_left_out_unless_they_are_asked_for() {
        let mut config = Config::default();
        config.tags.catalog_file = Some("/home/me/keywords.txt".to_string());

        assert!(changes_of(&config, false, false)
            .as_object()
            .expect("an object")
            .is_empty());
        assert!(changes_of(&config, false, true)["tags"]["catalog_file"].is_string());
    }
}
