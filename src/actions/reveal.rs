//! Showing a file in the platform's file manager.
//!
//! The same machinery a user action uses — a program spawned directly with an
//! argument vector, no shell — because the path is a file name off a card and
//! is not something to trust.

use std::path::Path;
use std::process::Command;

/// The command line Explorer needs to pick a file out.
///
/// Explorer parses its own command line rather than taking the arguments the
/// runtime built, and it splits what it is given on commas. So a photograph
/// off a camera called `2024-11-06 22-07-19 (C,S4).jpg` was cut in half at its
/// own comma, and the half that was left was not a path — whereupon Explorer
/// gave up and opened Documents. The space did as much harm on its own:
/// `Command::arg` wraps an argument holding one in quotes, and quoting the
/// whole `/select,PATH` rather than the path is a form Explorer does not
/// understand either. Both are fixed by writing the line out: the flag bare,
/// the path in quotes of its own.
///
/// The path is made absolute, because Explorer is not standing in this
/// program's directory, and its separators turned round, because Explorer
/// takes a forward slash for the start of a switch. A Windows file name cannot
/// hold a quotation mark, so quoting cannot be escaped out of.
pub fn select_argument(path: &Path) -> String {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());

    explorer_line(&absolute.to_string_lossy())
}

/// The line itself, once the path is absolute.
///
/// Split out because it is the half that has nothing to do with the platform
/// it runs on, and so is the half that can be tested on all three of them.
fn explorer_line(path: &str) -> String {
    format!("/select,\"{}\"", path.replace('/', "\\"))
}

/// Starts the platform's file manager with `path` picked out.
#[cfg(target_os = "windows")]
fn spawn_file_manager(path: &Path) -> std::io::Result<std::process::Child> {
    use std::os::windows::process::CommandExt;

    Command::new("explorer")
        .raw_arg(select_argument(path))
        .spawn()
}

#[cfg(target_os = "macos")]
fn spawn_file_manager(path: &Path) -> std::io::Result<std::process::Child> {
    Command::new("open").arg("-R").arg(path).spawn()
}

/// No portable way to pick a file out on Linux, so the folder is opened and
/// the file is left to be found. Better than nothing happening.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn spawn_file_manager(path: &Path) -> std::io::Result<std::process::Child> {
    let folder = path.parent().unwrap_or(path);

    Command::new("xdg-open").arg(folder).spawn()
}

/// Opens the file manager with `path` picked out, where the platform can, and
/// on its folder where it cannot.
///
/// Reports whether the program was started, not whether it did anything: a
/// file manager that opens and shows the wrong thing is not something this can
/// find out.
pub fn in_file_manager(path: &Path) -> bool {
    match spawn_file_manager(path) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The failure this exists for: a photograph off a camera with a comma in
    /// its name. Explorer split its line there and opened Documents.
    #[test]
    fn a_comma_in_the_name_stays_inside_the_quotes() {
        let line = explorer_line(r"C:\Photographs\2024-11-06 22-07-19 (C,S4).jpg");

        assert_eq!(
            line,
            "/select,\"C:\\Photographs\\2024-11-06 22-07-19 (C,S4).jpg\""
        );
    }

    /// The flag is outside the quotes and the path is inside them. Quoting the
    /// whole of it — which is what `Command::arg` does to an argument with a
    /// space in it — is a form Explorer does not understand either.
    #[test]
    fn the_flag_is_bare_and_the_path_is_quoted() {
        let line = explorer_line(r"C:\A Folder\photograph.jpg");

        assert!(line.starts_with("/select,\""), "{line}");
        assert!(line.ends_with('"'), "{line}");
        assert_eq!(line.matches('"').count(), 2, "{line}");
    }

    /// A forward slash is where Explorer expects a switch to start, so a path
    /// spelled the other way round is turned round first. Paths reach this
    /// program from a command line as readily as from a file dialog.
    #[test]
    fn forward_slashes_are_turned_round() {
        let line = explorer_line("C:/Photographs/one.jpg");

        assert_eq!(line, "/select,\"C:\\Photographs\\one.jpg\"");
        assert!(!line["/select,".len()..].contains('/'), "{line}");
    }

    /// Explorer is not standing in this program's directory, so a path that
    /// was relative when it arrived is made absolute before it is handed over.
    ///
    /// Windows alone: what "absolute" means is the platform's answer, and the
    /// turning round of separators is wrong anywhere else — which is why
    /// nothing else calls this.
    #[cfg(target_os = "windows")]
    #[test]
    fn a_relative_path_is_made_absolute() {
        let argument = select_argument(Path::new("one.jpg"));
        let inside = argument
            .trim_start_matches("/select,\"")
            .trim_end_matches('"');

        assert!(
            Path::new(inside).is_absolute(),
            "{argument} is still relative"
        );
        assert!(inside.ends_with("one.jpg"), "{argument}");
    }
}
