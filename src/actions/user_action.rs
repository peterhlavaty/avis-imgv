//! Running the commands the user configured, on the photograph on screen.
//!
//! No shell is involved — the program is spawned directly with an argument
//! vector — and the substitution is deliberately arranged so that it stays
//! that way whatever a file happens to be called.

use std::{path::Path, process::Command};

use eframe::egui::{self, Response};

use crate::config::ContextMenuEntry;
use crate::ui::menus::{self, Chosen, Verb};

/// Fills the placeholders in one argument.
///
/// One *argument*, which is the whole point. This used to substitute into the
/// command line as a whole and split the result afterwards, so what a file was
/// called decided how many arguments the program received: `holiday 1.jpg`
/// arrived as two, and a name containing an apostrophe opened or closed a
/// quoted run and could put extra arguments of its own into the command —
/// `a' --delete 'b.jpg` passing `--delete` to whatever was being run. File
/// names come off cards, downloads and shared drives, so they are not
/// something to trust.
///
/// Splitting the template first and filling each argument afterwards means a
/// substituted value is exactly one argument, whatever is in it.
fn fill(argument: &str, path: &Path) -> Option<String> {
    let mut argument = argument.to_string();

    if argument.contains("{}") {
        argument = argument.replace("{}", path.to_str()?);
    }
    if argument.contains("{.}") {
        let parent = path.parent()?;
        let file_stem = path.file_stem()?;
        let file_path = parent.join(file_stem);
        argument = argument.replace("{.}", file_path.to_str()?);
    }
    if argument.contains("{//}") {
        let parent = path.parent()?;
        argument = argument.replace("{//}", parent.to_str()?);
    }
    if argument.contains("{/}") {
        argument = argument.replace("{/}", path.file_name()?.to_str()?);
    }
    if argument.contains("{/.}") {
        argument = argument.replace("{/.}", path.file_stem()?.to_str()?);
    }
    if argument.contains("{.//}") {
        let arg = path.ancestors().nth(2)?.to_str()?;
        argument = argument.replace("{.//}", arg);
    }

    Some(argument)
}

/// The program and its arguments, ready to spawn.
///
/// `None` when a placeholder cannot be answered for this path — a file at the
/// root has no grandparent — because running the command with the placeholder
/// still in it would be worse than not running it.
pub fn command_line(exec: &str, path: &Path) -> Option<Vec<String>> {
    let template = get_command_args(exec);
    if template.is_empty() {
        return None;
    }

    template
        .iter()
        .map(|argument| fill(argument, path))
        .collect()
}

/// Executes command, returns false if command wasn't executed
/// or errored out
pub fn execute(exec: &str, path: &Path) -> bool {
    if exec.is_empty() {
        return true;
    }

    let Some(line) = command_line(exec, path) else {
        return false;
    };

    let Some((program, arguments)) = line.split_first() else {
        return false;
    };

    tracing::info!("exec -> {program} {arguments:?}");

    //Show toast with result?
    //We could return the error instead but we don't care much about it now
    //Provide the error to the user in the future
    match Command::new(program).args(arguments).spawn() {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("{e}");
            false
        }
    }
}

/// Splits a configured command line into arguments.
///
/// Single quotes group words, so `bash -c 'a && b'` is three arguments. Runs
/// of spaces collapse rather than producing empty arguments: `cmd  x` used to
/// pass an empty string between the two, which some programs read as a file
/// name of no characters.
pub fn get_command_args(cmd: &str) -> Vec<String> {
    let mut args: Vec<String> = vec![];
    let mut current = String::new();
    let mut in_string = false;
    let mut quoted = false;

    for next in cmd.chars() {
        if next == ' ' && !in_string {
            if !current.is_empty() || quoted {
                args.push(std::mem::take(&mut current));
            }

            quoted = false;
            continue;
        }

        if next == '\'' {
            in_string = !in_string;
            // So that an empty quoted argument is still an argument: `cmd ''`
            // asks for one, and dropping it changes what the program is told.
            quoted = true;
            continue;
        }

        current.push(next);
    }

    if !current.is_empty() || quoted {
        args.push(current);
    }

    args
}

/// Draws the menu for one photograph and reports what was asked for.
///
/// The built-in verbs come first and the user's own entries under a separator;
/// an entry that runs is reported back as its callback, and a verb is reported
/// as itself for whoever can carry it out. The menu used to return before
/// registering anything when the entry list was empty — which it is on a fresh
/// install — so the second button did nothing at all.
pub fn show_context_menu(
    ui: &egui::Ui,
    surface: &'static str,
    verbs: &[Verb],
    entries: &[ContextMenuEntry],
    response: &Response,
    path: &Path,
    count: usize,
) -> Option<Chosen> {
    let mut result = None;

    // Through the shared helper: on the press rather than the release, with the
    // same chevron and the same four words on every surface, and reachable from
    // the keyboard by name.
    crate::ui::surface::named_menu(ui, response, surface, |ui| {
        let Some(chosen) = menus::rows(ui, verbs, entries, count) else {
            return;
        };

        match chosen {
            Chosen::Verb(verb) => result = Some(Chosen::Verb(verb)),
            Chosen::Entry(i) => {
                let Some(entry) = entries.get(i) else {
                    return;
                };

                if execute(&entry.exec, path) {
                    result = Some(Chosen::Entry(i));
                }
            }
        }
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// The whole point of substituting per argument: a name with a space in
    /// it is one argument, not two.
    #[test]
    fn a_file_name_with_a_space_stays_one_argument() {
        let path = PathBuf::from(native(&["/photos", "holiday 1.jpg"]));
        let line = command_line("gimp {}", &path).unwrap();

        assert_eq!(line, vec!["gimp".to_string(), path.display().to_string()]);
    }

    /// The one that mattered: a name can no longer put arguments of its own
    /// into somebody else's command line.
    #[test]
    fn a_file_name_cannot_smuggle_in_arguments() {
        let path = PathBuf::from("/photos/a' --delete 'b.jpg");
        let line = command_line("convert {} out.png", &path).unwrap();

        assert_eq!(
            line,
            vec![
                "convert".to_string(),
                "/photos/a' --delete 'b.jpg".to_string(),
                "out.png".to_string(),
            ]
        );
        assert!(!line.contains(&"--delete".to_string()));
    }

    /// Quoting in the *template* is the user's own, and still groups.
    #[test]
    fn quoting_in_the_template_still_groups_arguments() {
        let path = PathBuf::from(native(&["/photos", "a.jpg"]));
        let line = command_line("bash -c 'cp {} /backup'", &path).unwrap();

        assert_eq!(line.len(), 3);
        assert_eq!(line[0], "bash");
        assert_eq!(line[1], "-c");
        assert!(line[2].starts_with("cp "));
        assert!(line[2].ends_with(" /backup"));
    }

    /// A placeholder that cannot be answered stops the command rather than
    /// being passed through as itself.
    #[test]
    fn an_unanswerable_placeholder_runs_nothing() {
        assert!(command_line("cmd {.//}", Path::new("/a.jpg")).is_none());
        assert!(!execute("cmd {.//}", Path::new("/a.jpg")));
    }

    #[test]
    fn runs_of_spaces_do_not_make_empty_arguments() {
        assert_eq!(get_command_args("cmd   x"), vec!["cmd", "x"]);
        assert_eq!(get_command_args("  cmd x  "), vec!["cmd", "x"]);
    }

    /// An argument the user deliberately made empty is still an argument.
    #[test]
    fn an_empty_quoted_argument_survives() {
        assert_eq!(get_command_args("cmd '' x"), vec!["cmd", "", "x"]);
    }

    #[test]
    fn get_command_args_returns_correct_args() {
        let cmd = "mkdir /this/is/a/path";
        let args = get_command_args(cmd);
        assert_eq!(args, vec!["mkdir", "/this/is/a/path"])
    }

    #[test]
    fn get_command_args_with_string_returns_correct_args() {
        let cmd = "bash -c 'mkdir /this/is/a/path && cp file /this/is/a/path'";
        let args = get_command_args(cmd);
        assert_eq!(
            args,
            vec![
                "bash",
                "-c",
                "mkdir /this/is/a/path && cp file /this/is/a/path"
            ]
        )
    }

    /// Renders an expected path with the separator of the host platform, so
    /// the assertions hold on Windows as well as on Unix.
    fn native(parts: &[&str]) -> String {
        parts
            .iter()
            .fold(PathBuf::new(), |acc, part| acc.join(part))
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn format_exec_string_placeholders() {
        let root = native(&["/tmp", "foo", "bar.txt"]);
        let path = Path::new(&root);

        for (template, expected, description) in [
            ("cmd {}", native(&["/tmp", "foo", "bar.txt"]), "full path"),
            (
                "cmd {.}",
                native(&["/tmp", "foo", "bar"]),
                "path without extension",
            ),
            ("cmd {//}", native(&["/tmp", "foo"]), "parent directory"),
            ("cmd {/}", "bar.txt".to_string(), "file name"),
            ("cmd {/.}", "bar".to_string(), "file stem"),
            ("cmd {.//}", native(&["/tmp"]), "grandparent directory"),
        ] {
            let line = command_line(template, path).unwrap();
            assert_eq!(
                line,
                vec!["cmd".to_string(), expected],
                "should replace with the {description}"
            );
        }
    }

    #[test]
    fn a_command_without_placeholders_is_left_alone() {
        let line = command_line("gimp", Path::new("/tmp/a.jpg")).unwrap();
        assert_eq!(line, vec!["gimp".to_string()]);
    }

    #[test]
    fn an_empty_command_counts_as_success() {
        assert!(execute("", Path::new("/tmp/a.jpg")));
    }
}
