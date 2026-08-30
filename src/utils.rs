//! Small helpers shared by the UI: input muting and path predicates.

use std::path::Path;

use eframe::egui::{self, Id, Response};

use crate::formats;

pub fn textedit_move_cursor_to_end(resp: &Response, ui: &mut egui::Ui, len: usize) {
    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
        let ccursor = egui::text::CCursor::new(len);
        state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
        state.store(ui.ctx(), resp.id);
        resp.request_focus();
        ui.ctx().memory_mut(|m| m.request_focus(resp.id))
    }
}

pub fn set_mute_state(ctx: &egui::Context, muted: bool) {
    ctx.memory_mut(|mem| {
        mem.data.insert_temp::<bool>(get_muted_data_id(), muted);
    })
}

/// Takes the keyboard back from whatever widget has it.
///
/// egui hands focus to the next widget on Tab and keeps it there, and a text
/// field with focus mutes every shortcut in the viewer. That is right while
/// somebody is typing a path and wrong the instant they are not: Tab means
/// "the next pane" here, and Escape means "give me the keyboard back".
pub fn surrender_focus(ctx: &eframe::egui::Context) {
    ctx.memory_mut(|memory| {
        if let Some(id) = memory.focused() {
            memory.surrender_focus(id);
        }
    });
}

pub fn are_inputs_muted(ctx: &egui::Context) -> bool {
    ctx.memory_mut(|mem| {
        mem.data
            .get_temp::<bool>(get_muted_data_id())
            .unwrap_or(false)
    }) || ctx.memory(|mem| mem.focused().is_some())
}

pub fn get_muted_data_id() -> Id {
    Id::new("muted")
}

/// Returns true if path contains any images we can open
pub fn is_valid_path(path: &Path) -> bool {
    let dir_info = match path.read_dir() {
        Ok(dir) => dir,
        Err(_) => return false,
    };

    for path in dir_info.flatten() {
        if formats::is_supported(&path.path()) {
            return true;
        }
    }

    false
}

/// True when a directory name starts with a dot.
pub fn is_dir_hidden(path: &Path) -> bool {
    path.file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .starts_with('.')
}

pub fn capitalize_first_char(str: &str) -> String {
    let mut chars = str.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
