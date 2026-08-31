//! Showing a file in the platform's file manager.
//!
//! The same machinery a user action uses — a program spawned directly with an
//! argument vector, no shell — because the path is a file name off a card and
//! is not something to trust.

use std::path::Path;
use std::process::Command;

/// Opens the file manager with `path` picked out, where the platform can, and
/// on its folder where it cannot.
///
/// Reports whether the program was started, not whether it did anything: a
/// file manager that opens and shows the wrong thing is not something this can
/// find out.
pub fn in_file_manager(path: &Path) -> bool {
    let spawned = if cfg!(target_os = "windows") {
        // The comma is part of the flag rather than a separator, and Explorer
        // wants the whole thing as one argument.
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg("-R").arg(path).spawn()
    } else {
        // No portable way to pick a file out on Linux, so the folder is opened
        // and the file is left to be found. Better than nothing happening.
        let folder = path.parent().unwrap_or(path);
        Command::new("xdg-open").arg(folder).spawn()
    };

    match spawned {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Could not show {} in the file manager: {e}", path.display());
            false
        }
    }
}

/// Opens a file with whatever the platform thinks owns it.
///
/// Used for the configuration file and the log, which are text and have no
/// business being opened by this program.
pub fn with_the_system(path: &Path) -> bool {
    let spawned = if cfg!(target_os = "windows") {
        // `start` is a shell builtin, so the shell is unavoidable here; the
        // empty string is the window title `start` insists on consuming, and
        // the path goes in its own argument so a space in it is not a split.
        Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()
    } else {
        Command::new("xdg-open").arg(path).spawn()
    };

    match spawned {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Could not open {}: {e}", path.display());
            false
        }
    }
}

/// Opens a folder in the file manager.
pub fn folder(path: &Path) -> bool {
    with_the_system(path)
}
