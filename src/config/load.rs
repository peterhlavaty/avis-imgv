//! Finding, reading and creating the configuration file.

use std::{fs, io::ErrorKind, path::PathBuf, sync::Mutex, time::SystemTime};

use super::{migrate, Config};
use crate::atomic;
use crate::{APPLICATION, ORGANIZATION, QUALIFIER};

/// When the configuration file was last written or read by this process.
///
/// A save that would write over somebody's hand edit is refused, and the only
/// way to know an edit happened is to have looked at the time before.
static SEEN: Mutex<Option<SystemTime>> = Mutex::new(None);

/// What a save did, when it did not fail outright.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Save {
    /// The file was written.
    Written,
    /// The file on disk had moved since it was read, so nothing was written.
    Refused,
}

/// The file's modification time, or `None` when there is no file to ask about.
fn modified() -> Option<SystemTime> {
    let path = Config::path()?;
    fs::metadata(path).ok()?.modified().ok()
}

/// Records the file as this process last saw it.
///
/// Read back from the path rather than assumed, because the write is a rename
/// over the original and so produces a different file with a different time.
pub fn remember_on_disk() {
    if let Ok(mut seen) = SEEN.lock() {
        *seen = modified();
    }
}

/// Whether the file has been edited since this process last looked.
pub fn moved_on_disk() -> bool {
    let Ok(seen) = SEEN.lock() else {
        return false;
    };

    match (*seen, modified()) {
        // Nothing was ever recorded, so there is nothing to have moved: this
        // is a run that never managed to read a file.
        (None, _) => false,
        (Some(_), None) => true,
        (Some(a), Some(b)) => a != b,
    }
}

impl Config {
    pub fn new() -> Config {
        Self::fetch_cfg()
    }

    /// Where the configuration file lives, whether or not it exists yet.
    pub fn path() -> Option<PathBuf> {
        directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.config_dir().join("config.json"))
    }

    /// Writes the configuration back out, unless the file has been edited.
    ///
    /// The file is read once at startup and the viewer holds a copy for the
    /// rest of the run, so an in-app save writes over whatever was hand-edited
    /// meanwhile. A save that would do that is refused, and the caller offers
    /// to read the file again or to keep what is on screen.
    pub fn save(&self) -> std::io::Result<Save> {
        if moved_on_disk() {
            return Ok(Save::Refused);
        }

        self.save_over()?;

        Ok(Save::Written)
    }

    /// Writes it whatever the file on disk says.
    ///
    /// The answer to "keep what is on screen" after a refusal, and the way the
    /// program's own writes are made — the fresh install and the brought
    /// forward file — which happen before any interface exists and so have
    /// nobody to ask.
    pub fn save_over(&self) -> std::io::Result<()> {
        // What was not understood on the way in is not there to write back
        // out, and writing anyway would make the loss permanent.
        if self.partial {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "part of the configuration file could not be read, so it is not being written over",
            ));
        }

        let path = Self::path().ok_or_else(|| {
            std::io::Error::new(ErrorKind::NotFound, "no configuration directory")
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.merged_document())
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

        atomic::replace(&path, json.as_bytes())?;
        remember_on_disk();
        tracing::info!("Wrote config -> {}", path.display());

        Ok(())
    }

    /// This configuration laid over the document it was read from.
    ///
    /// Serialising the struct and writing that is how a key a newer build
    /// wrote is lost when an older one saves — Geeqie's defect, whose reporter
    /// diagnosed it himself as a consequence of regenerating the file from
    /// scratch each time. What was read is kept and the struct is merged into
    /// it, recursing one level, so an unknown key inside a known section
    /// survives as well as an unknown section.
    pub fn merged_document(&self) -> serde_json::Value {
        let mine = serde_json::to_value(self).unwrap_or(serde_json::Value::Null);

        let (Some(mut base), serde_json::Value::Object(mine)) = (self.document.clone(), mine)
        else {
            return serde_json::to_value(self).unwrap_or(serde_json::Value::Null);
        };

        for (key, value) in mine {
            match (base.get_mut(&key), value) {
                (Some(serde_json::Value::Object(kept)), serde_json::Value::Object(fresh)) => {
                    for (inner, value) in fresh {
                        kept.insert(inner, value);
                    }
                }
                (_, value) => {
                    base.insert(key, value);
                }
            }
        }

        serde_json::Value::Object(base)
    }

    /// Puts the interface text back to its normal size, on disk.
    ///
    /// For `--reset-text-size`, whose whole point is that the interface cannot
    /// be read well enough to find the control — so it goes through the same
    /// merge-and-atomic-write every other save does, and takes effect on the
    /// launch that follows.
    pub fn reset_text_size() -> std::io::Result<()> {
        let mut config = Config::new();
        config.general.text_scaling = crate::config::default_text_scaling();

        config.save_over()
    }

    pub fn fetch_cfg() -> Config {
        let config_dir = match directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        {
            Some(dirs) => dirs.config_dir().to_owned(),
            None => return Config::default(),
        };

        let cfg_path = config_dir.join(PathBuf::from("config.json"));
        tracing::info!("Reading config -> {}", cfg_path.display());

        let config_json = match fs::read_to_string(&cfg_path) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!("Failure reading config file -> {e}");
                let default_cfg = Config::default();

                if e.kind() == ErrorKind::NotFound {
                    tracing::info!("Config file does not exist -> creating default config");

                    if !config_dir.exists() {
                        tracing::info!("Config directory does not exist, creating");
                        if let Err(e) = fs::create_dir_all(&config_dir) {
                            tracing::error!("Failure creating config directory {:?}", e);
                        }
                    }

                    // Pretty printed, like every other write. It used to be
                    // one long line, which is the file the README tells people
                    // to open in an editor.
                    if let Err(e) = default_cfg.save_over() {
                        tracing::error!("Failure writing default config file -> {e}");
                    }
                }
                return default_cfg;
            }
        };

        remember_on_disk();

        let mut cfg = Self::from_json(&config_json);

        // Brought up to date on the way in, and written back out so it is only
        // done once. A file that was only partly understood is not written
        // over — the same rule as everywhere else — so it is migrated in
        // memory and left alone on disk.
        let brought_forward = migrate::apply(&mut cfg);
        cfg.migrated.extend(brought_forward);
        if !cfg.migrated.is_empty() && !cfg.partial {
            // `save_over` rather than `save`: this is the program's own write
            // and there is nobody to ask about it yet, so it must not be
            // refused by the guard it then re-arms.
            if let Err(e) = cfg.save_over() {
                tracing::error!("Could not write the brought-forward config: {e}");
            }
        }

        // The whole configuration is one long line, so it stays out of the
        // way unless something needs explaining.
        tracing::debug!(
            "Using config: {}",
            serde_json::to_string(&cfg).unwrap_or_default()
        );

        cfg
    }

    /// Reads a configuration document one section at a time.
    ///
    /// Section by section rather than in one go, because one section the
    /// viewer cannot make sense of — a key renamed between versions, a number
    /// where a string belongs, a hand edit with a comma missing — used to
    /// discard the whole file and hand back the defaults for everything. A
    /// section that is simply absent is not a problem: it is what a file
    /// written by an older build looks like, and the defaults are the right
    /// answer.
    pub fn from_json(document: &str) -> Config {
        // Notepad, and every other Windows editor with a "UTF-8" option that
        // means "UTF-8 with a byte order mark", writes three bytes in front of
        // the opening brace. JSON has no place for them, so a file somebody had
        // merely opened and saved parsed as nothing at all and silently handed
        // back the defaults for everything.
        let document = document.trim_start_matches('\u{feff}');
        let document = strip_comments(document);

        let mut map: serde_json::Map<String, serde_json::Value> =
            match serde_json::from_str(&document) {
                Ok(map) => map,
                Err(e) => {
                    tracing::error!("The configuration file could not be read at all: {e}");
                    return Config {
                        partial: true,
                        ..Config::default()
                    };
                }
            };

        // Before the sections are built, because a key that changes section
        // or type is gone once `serde` has finished with it.
        let migrated = migrate::document(&mut map);

        let mut partial = false;

        Config {
            version: map
                .get("version")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as u32,
            image_view: section(&map, "image_view", &mut partial),
            grid_view: section(&map, "grid_view", &mut partial),
            general: section(&map, "general", &mut partial),
            cache: section(&map, "cache", &mut partial),
            slideshow: section(&map, "slideshow", &mut partial),
            tags: section(&map, "tags", &mut partial),
            raw: section(&map, "raw", &mut partial),
            cull: section(&map, "cull", &mut partial),
            browsing: section(&map, "browsing", &mut partial),
            group: section(&map, "group", &mut partial),
            menus: section(&map, "menus", &mut partial),
            mouse: section(&map, "mouse", &mut partial),
            history: section(&map, "history", &mut partial),
            partial,
            migrated,
            document: Some(map),
        }
    }
}

/// Takes `//` and `/* */` out of a document before it is parsed.
///
/// JSON has no comments and `serde_json` says so by refusing the whole
/// document, which sets `partial`, blocks every save for the session and hands
/// back the defaults for everything — while the README promises the opposite.
/// The same shape and the same place as the byte order mark strip above it.
/// What is stripped is not written back: a save writes JSON.
fn strip_comments(document: &str) -> String {
    let mut out = String::with_capacity(document.len());
    let mut chars = document.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut previous = '\0';
                for c in chars.by_ref() {
                    // Newlines are kept so a parse failure further down still
                    // names the line the person is looking at.
                    if c == '\n' {
                        out.push('\n');
                    }
                    if previous == '*' && c == '/' {
                        break;
                    }
                    previous = c;
                }
            }
            _ => out.push(c),
        }
    }

    out
}

/// One section of the document, or its defaults when it cannot be read.
fn section<T: serde::de::DeserializeOwned + Default>(
    map: &serde_json::Map<String, serde_json::Value>,
    name: &str,
    partial: &mut bool,
) -> T {
    let Some(value) = map.get(name) else {
        return T::default();
    };

    match serde_json::from_value(value.clone()) {
        Ok(parsed) => parsed,
        Err(e) => {
            tracing::error!("Ignoring the \"{name}\" section of the configuration: {e}");
            *partial = true;
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::GridViewConfig;
    use super::*;

    #[test]
    fn a_complete_file_is_read_whole() {
        let json = serde_json::to_string(&Config::default()).unwrap();
        let cfg = Config::from_json(&json);

        assert!(!cfg.partial);
    }

    /// A second key reaches the program from the file, and both keys press
    /// the command.
    ///
    /// The whole chain in one test: the `also` list off the disk, through
    /// `Config`, into `shortcut::consume`. The pieces are covered where they
    /// live; this is the one that fails if any of the joins between them come
    /// apart.
    #[test]
    fn a_second_key_read_from_the_file_presses_the_command() {
        let cfg = Config::from_json(
            r#"{"general": {"sc_exit": {"key": "F13", "modifiers": [],
                 "also": [{"key": "F14", "modifiers": ["ctrl"]}]}}}"#,
        );

        assert!(!cfg.partial);
        assert_eq!(cfg.general.sc_exit.len(), 2);

        for (key, modifiers) in [
            (eframe::egui::Key::F13, eframe::egui::Modifiers::NONE),
            (eframe::egui::Key::F14, eframe::egui::Modifiers::CTRL),
        ] {
            let ctx = eframe::egui::Context::default();
            ctx.begin_pass(eframe::egui::RawInput {
                events: vec![eframe::egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }],
                ..Default::default()
            });

            let pressed = ctx
                .input_mut(|input| crate::config::shortcut::consume(input, &cfg.general.sc_exit));
            assert!(pressed, "{key:?} did not press it");
        }
    }

    /// And writing it out again keeps both, in the shape an older build reads
    /// the first of.
    #[test]
    fn a_second_key_survives_being_written_out() {
        let mut cfg = Config::default();
        cfg.general
            .sc_exit
            .add(crate::config::Chord::new("F14", &["ctrl"]));

        let json = serde_json::to_string(&cfg).expect("writes");
        let read = Config::from_json(&json);

        assert_eq!(read.general.sc_exit, cfg.general.sc_exit);
        assert!(json.contains(r#""also":[{"key":"F14","modifiers":["ctrl"]}]"#));
    }

    /// A file written by an older build has sections the newer one added.
    #[test]
    fn a_missing_section_costs_nothing() {
        let cfg = Config::from_json(r#"{"general": {}}"#);

        assert!(!cfg.partial);
        assert_eq!(
            cfg.grid_view.images_per_row,
            GridViewConfig::default().images_per_row
        );
    }

    /// The case that used to discard the file: one section the viewer cannot
    /// make sense of.
    #[test]
    fn one_bad_section_costs_only_that_section() {
        let cfg = Config::from_json(
            r#"{"grid_view": "not an object", "general": {"text_scaling": 2.5}}"#,
        );

        assert!(cfg.partial);
        assert_eq!(cfg.general.text_scaling, 2.5);
        assert_eq!(
            cfg.grid_view.images_per_row,
            GridViewConfig::default().images_per_row
        );
    }

    #[test]
    fn a_file_that_was_not_understood_is_never_written_back() {
        let cfg = Config {
            partial: true,
            ..Config::default()
        };

        assert!(cfg.save_over().is_err());
    }

    /// A file saved by a Windows editor keeps its settings.
    #[test]
    fn a_byte_order_mark_does_not_cost_the_file() {
        let json = serde_json::to_string(&Config::default()).unwrap();
        let cfg = Config::from_json(&format!("\u{feff}{json}"));

        assert!(!cfg.partial);
        assert_eq!(cfg.version, migrate::CURRENT);
    }

    #[test]
    fn a_document_that_is_not_json_at_all_is_partial() {
        assert!(Config::from_json("not json").partial);
    }

    /// The README says the file may be annotated. It used to cost the file.
    #[test]
    fn a_line_comment_costs_nothing() {
        let cfg = Config::from_json(
            r#"{
            // how big the text is
            "general": {"text_scaling": 2.0}
        }"#,
        );

        assert!(!cfg.partial);
        assert_eq!(cfg.general.text_scaling, 2.0);
    }

    #[test]
    fn a_block_comment_costs_nothing() {
        let cfg = Config::from_json(
            r#"{ /* two
            lines */ "general": {"text_scaling": 2.0} }"#,
        );

        assert!(!cfg.partial);
        assert_eq!(cfg.general.text_scaling, 2.0);
    }

    /// A path with two slashes in it is not a comment.
    #[test]
    fn a_slash_inside_a_string_survives() {
        let cfg = Config::from_json(r#"{"tags": {"catalog_file": "//server/share/tags.txt"}}"#);

        assert!(!cfg.partial);
        assert_eq!(
            cfg.tags.catalog_file,
            Some("//server/share/tags.txt".into())
        );
    }

    #[test]
    fn an_escaped_quote_does_not_end_the_string() {
        assert_eq!(strip_comments(r#"{"a": "b\"//c"}"#), r#"{"a": "b\"//c"}"#);
    }

    /// Geeqie's defect: a key this build does not know is dropped on the way
    /// out, so the newer build's settings are lost when the older one saves.
    #[test]
    fn an_unknown_key_survives_a_save() {
        let cfg = Config::from_json(r#"{"tomorrow": {"a": 1}, "general": {"whatsit": 7}}"#);
        let out = cfg.merged_document();

        assert_eq!(out["tomorrow"]["a"], 1);
        assert_eq!(out["general"]["whatsit"], 7);
        // And what this build does know is still written.
        assert!(out["general"]["text_scaling"].is_number());
    }

    /// A configuration nobody read from a file still writes the whole struct.
    #[test]
    fn a_configuration_with_no_document_writes_itself() {
        let out = Config::default().merged_document();

        assert!(out["general"]["text_scaling"].is_number());
        assert!(out["version"].is_number());
    }
}
