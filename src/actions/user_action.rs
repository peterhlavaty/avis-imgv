use std::{path::Path, process::Command};

use eframe::egui::Response;

use crate::{actions::callback::Callback, config::ContextMenuEntry};

fn format_exec_string(exec: &str, path: &Path) -> Option<String> {
    let mut exec = exec.to_string();

    if exec.contains("{}") {
        exec = exec.replace("{}", path.to_str()?);
    }
    if exec.contains("{.}") {
        let parent = path.parent()?;
        let file_stem = path.file_stem()?;
        let file_path = parent.join(file_stem);
        exec = exec.replace("{.}", file_path.to_str()?);
    }
    if exec.contains("{//}") {
        let parent = path.parent()?;
        exec = exec.replace("{//}", parent.to_str()?);
    }
    if exec.contains("{/}") {
        exec = exec.replace("{/}", path.file_name()?.to_str()?);
    }
    if exec.contains("{/.}") {
        exec = exec.replace("{/.}", path.file_stem()?.to_str()?);
    }
    if exec.contains("{.//}") {
        let arg = path.ancestors().nth(2)?.to_str()?;
        exec = exec.replace("{.//}", arg);
    }

    Some(exec)
}

/// Executes command, returns false if command wasn't executed
/// or errored out
pub fn execute(exec: &str, path: &Path) -> bool {
    if exec.is_empty() {
        return true;
    }

    let exec = match format_exec_string(exec, path) {
        Some(exec) => exec,
        None => return false,
    };

    tracing::info!("exec -> {exec}");
    let exec_split = get_command_args(&exec);
    let mut exec_split = exec_split.iter();

    let cmd = match exec_split.next() {
        Some(cmd) => cmd,
        None => return false,
    };

    tracing::info!("cmd: {cmd}");
    let mut cmd = Command::new(cmd);

    for arg in exec_split {
        tracing::info!("arg: {arg}");
        cmd.arg(arg);
    }

    //Show toast with result?
    //We could return the error instead but we don't care much about it now
    //Provide the error to the user in the future
    match cmd.spawn() {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("{e}");
            false
        }
    }
}

pub fn get_command_args(cmd: &str) -> Vec<String> {
    let mut args: Vec<String> = vec![];
    let mut it = cmd.chars();
    let mut current = String::new();
    let mut in_string = false;
    loop {
        let next = it.next();

        if next.is_none() {
            if !current.is_empty() {
                args.push(current.to_string());
            }
            break;
        }

        let next = next.unwrap();

        if next == ' ' && !in_string {
            args.push(current.to_string());
            current = String::new();
            in_string = false;
            continue;
        }

        if next == '\'' {
            in_string = !in_string;
            continue;
        }

        current += &next.to_string();
    }

    args
}

pub fn show_context_menu(
    entries: &Vec<ContextMenuEntry>,
    response: &Response,
    path: &Path,
) -> Option<Callback> {
    if entries.is_empty() {
        return None;
    }

    let mut result: Option<Callback> = None;
    response.context_menu(|ui| {
        ui.set_max_width(300.);
        for entry in entries {
            let button_resp = ui.button(&entry.description);

            if button_resp.clicked() {
                if execute(&entry.exec, path) {
                    result = entry.callback.clone();
                }
                ui.close();
            }
        }
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

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
            let formatted = format_exec_string(template, path).unwrap();
            assert_eq!(
                formatted,
                format!("cmd {expected}"),
                "should replace with the {description}"
            );
        }
    }

    #[test]
    fn a_command_without_placeholders_is_left_alone() {
        let formatted = format_exec_string("gimp", Path::new("/tmp/a.jpg")).unwrap();
        assert_eq!(formatted, "gimp");
    }

    #[test]
    fn an_empty_command_counts_as_success() {
        assert!(execute("", Path::new("/tmp/a.jpg")));
    }
}
