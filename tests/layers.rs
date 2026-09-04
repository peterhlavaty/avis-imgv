//! The direction the modules depend in, checked rather than believed.
//!
//! The crate is already layered — the survey that started this branch found
//! that half of it by line count never mentions the toolkit — but nothing said
//! so, and the two edges that spoiled it were a struct in a drawing file and a
//! `use` inside a test. Both were invisible to inspection, because neither
//! module named egui itself: they reached it through somebody who did.
//!
//! So this walk is **transitive**, and that is the whole point. A grep for
//! `egui` in a file passes `view/narrow.rs`, which imported `Marks` out of a
//! file with thirty-four mentions of it. Only following the edges finds that.
//!
//! # What this is and is not
//!
//! It is a test. Somebody can add a row to [`DRAWS`] in the same commit that
//! breaks the rule, and nothing stops them. That is said plainly because the
//! alternative — pretending a convention is enforcement — is how a boundary
//! rots while everybody believes it holds. What makes it worth having anyway
//! is that the exemption list is short, is in the diff, and shrinks.
//!
//! The one boundary that is not a test is the Cargo feature gate, when it
//! lands: `cargo check --no-default-features` takes eframe out of the
//! dependency graph, and then an unresolved import is `rustc`'s answer rather
//! than this file's.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Modules that decide rather than draw, and must not reach the toolkit — not
/// directly, and not through anybody else.
///
/// This is the list the branch is making longer. Every name added here is a
/// piece of the program that can be tested without a window.
const DECIDES: &[&str] = &[
    "atomic",
    "board",
    "command",
    "crawler",
    "fault",
    "fit",
    "formats",
    "mode",
    "work",
    "annotations",
    "decoder",
    "metadata",
    "organize",
];

/// Modules that legitimately name the toolkit or the graphics API.
///
/// Two kinds, and the difference matters. Most are the drawing layer, where
/// naming egui is the job. A few are a *drawing file inside a job directory* —
/// `history::panel` is the history's own panel — which is the house rule
/// working as intended: folders follow the functionality, and the file that
/// draws sits beside the files that decide.
const DRAWS: &[&str] = &[
    // The drawing layer proper.
    "app",
    "ui",
    "view",
    // The drawing file inside a job's own directory.
    "history::panel",
    // The graphics API itself, which is what these are for.
    "cache::gpu",
    "cache::mipmap",
    "cache::store",
    // Not to be cut, and the reason is worth having written down. A chord is
    // compared on the `KeyboardShortcut` egui builds from it, because `Esc`
    // and `Escape` are one key and a comparison minding the spelling let a
    // clash through. Owning that vocabulary here would mean copying
    // `Key::from_name`'s hundred names and their aliases out of egui, where
    // they would drift on the next update — and the clash checker would then
    // be answering about keys the toolkit does not read. `config::shortcut`
    // is the adaptor between the file's words and the toolkit's, and an
    // adaptor naming both sides is what an adaptor is.
    "config::shortcut",
    "config::load",
    // `src/actions/` is the one folder of business logic that draws.
    "actions::user_action",
];

/// Everything the crate is made of, as `module::path` to file contents.
fn sources() -> HashMap<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = HashMap::new();
    let mut stack = vec![root.clone()];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|it| it == "rs") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    found.insert(module_of(&root, &path), text);
                }
            }
        }
    }

    found
}

/// `src/view/image_view/mod.rs` is `view::image_view`; `src/fit.rs` is `fit`.
fn module_of(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect();

    if let Some(last) = parts.last_mut() {
        *last = last.trim_end_matches(".rs").to_string();

        if last == "mod" || last == "lib" {
            parts.pop();
        }
    }

    parts.join("::")
}

/// Whether a file names the toolkit in code rather than in prose.
///
/// Comments are stripped first: four of the modules this branch cleaned still
/// *mention* wgpu in a doc comment explaining why a buffer is laid out as it
/// is, and a rule that counted those would be a rule nobody could satisfy.
fn names_the_toolkit(text: &str) -> bool {
    text.lines()
        .map(|line| line.trim_start())
        .filter(|line| !line.starts_with("//") && !line.starts_with("*"))
        .any(|line| {
            ["egui", "eframe", "epaint", "wgpu"]
                .iter()
                .any(|name| line.contains(name))
        })
}

/// Which modules a file names, resolved against the modules that exist.
fn edges(text: &str, modules: &HashSet<&String>) -> HashSet<String> {
    let mut found = HashSet::new();

    for piece in text.split("crate::").skip(1) {
        let path: String = piece
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
            .collect();

        // Longest prefix that is a module: `crate::view::narrow::FlagRule`
        // names `view::narrow`, not `view::narrow::FlagRule`.
        let parts: Vec<&str> = path.split("::").filter(|p| !p.is_empty()).collect();
        for take in (1..=parts.len()).rev() {
            let candidate = parts[..take].join("::");
            if modules.contains(&candidate) {
                found.insert(candidate);
                break;
            }
        }
    }

    found
}

/// Whether `module` is covered by a row in `list`, itself or as a parent.
fn listed(module: &str, list: &[&str]) -> bool {
    list.iter()
        .any(|row| module == *row || module.starts_with(&format!("{row}::")))
}

#[test]
fn a_module_that_decides_never_reaches_the_toolkit() {
    let sources = sources();
    let names: HashSet<&String> = sources.keys().collect();

    let reaches: HashMap<&String, HashSet<String>> = sources
        .iter()
        .map(|(module, text)| (module, edges(text, &names)))
        .collect();

    let mut faults = Vec::new();

    for module in sources.keys() {
        if !listed(module, DECIDES) {
            continue;
        }

        // Breadth-first from the module, following every edge, keeping the
        // route so a failure can say how it got there rather than only that
        // it did.
        let mut seen: HashSet<&String> = HashSet::new();
        let mut queue = vec![(module, vec![module.clone()])];

        while let Some((at, route)) = queue.pop() {
            if !seen.insert(at) {
                continue;
            }

            if sources.get(at).is_some_and(|text| names_the_toolkit(text)) && !listed(at, DECIDES) {
                faults.push(format!("  {}", route.join(" -> ")));
                continue;
            }

            for next in reaches.get(at).into_iter().flatten() {
                if let Some(name) = names.get(next) {
                    let mut route = route.clone();
                    route.push((*name).clone());
                    queue.push((name, route));
                }
            }
        }
    }

    assert!(
        faults.is_empty(),
        "a module that decides reaches the toolkit:\n{}\n\nEither the edge is \
         a mistake — the usual cause is a type living in a drawing file \
         because one of the things that draws it happens to be there — or the \
         module belongs in DRAWS rather than DECIDES.",
        faults.join("\n")
    );
}

/// Every name in either list is a module that exists.
///
/// A list of exemptions nobody prunes is how a boundary rots quietly: a row
/// naming a module that has been renamed exempts nothing and looks like it
/// exempts something.
#[test]
fn both_lists_name_modules_that_exist() {
    let sources = sources();
    let known: HashSet<&String> = sources.keys().collect();

    for row in DECIDES.iter().chain(DRAWS.iter()) {
        assert!(
            known.iter().any(|module| listed(module, &[row])),
            "`{row}` is named in this test and is not a module in the crate"
        );
    }
}

/// The modules in `DRAWS` that are only there until an edge is cut.
///
/// Not an assertion about the code — a note that reads back. When
/// `config::shortcut` stops naming egui, this test says so and the row can go.
#[test]
fn the_exemptions_still_earn_their_place() {
    let sources = sources();
    let mut idle = Vec::new();

    for row in DRAWS {
        let still_draws = sources
            .iter()
            .any(|(module, text)| listed(module, &[row]) && names_the_toolkit(text));

        if !still_draws {
            idle.push(*row);
        }
    }

    assert!(
        idle.is_empty(),
        "these no longer name the toolkit and should be moved from DRAWS to \
         DECIDES: {idle:?}"
    );
}

/// Where the crate keeps its own files, and what a temporary one is for.
#[test]
fn the_module_names_are_read_the_way_rust_reads_them() {
    let root = PathBuf::from("src");

    assert_eq!(module_of(&root, &root.join("fit.rs")), "fit");
    assert_eq!(
        module_of(&root, &root.join("view").join("narrow.rs")),
        "view::narrow"
    );
    assert_eq!(
        module_of(&root, &root.join("view").join("image_view").join("mod.rs")),
        "view::image_view"
    );
}

/// The trap the first version of this fell into: a doc comment saying why a
/// buffer is laid out the way wgpu wants is not a dependency on wgpu.
#[test]
fn prose_about_the_toolkit_is_not_a_dependency_on_it() {
    assert!(!names_the_toolkit(
        "//! stored as RGBA8 — the layout wgpu wants"
    ));
    assert!(!names_the_toolkit(
        "    /// not — egui's own `Image` widget —"
    ));
    assert!(names_the_toolkit("use eframe::egui;"));
    assert!(names_the_toolkit("    let ctx = egui::Context::default();"));
}
