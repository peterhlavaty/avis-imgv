//! Bringing a configuration file written by an older build up to date.
//!
//! Defaults change. Most of the time that costs nothing — somebody who never
//! touched a setting simply gets the new one, and somebody who did keeps what
//! they wrote. The exception is a default that *moves*: when a key stops
//! meaning one thing and starts meaning another, a file still holding the old
//! binding leaves two commands fighting over it, and the one that loses does
//! nothing at all with no explanation.
//!
//! So the file carries a version, and each change that needs a hand gets a
//! step here. A step is only ever applied to a file old enough to need it, so
//! somebody who has deliberately bound a key back to what it used to be keeps
//! it.
//!
//! What a step must not do is overwrite a choice. Every one of them checks
//! that what it finds is the *old default* before touching it: a setting the
//! user has actually changed is theirs, and a migration that flattens it is
//! worse than the clash it was avoiding.

use serde_json::{Map, Value};

use super::{defaults, Config, Shortcut};

/// What this build writes.
///
/// Bumped whenever a step is added below, and never otherwise.
pub const CURRENT: u32 = 4;

/// One thing that has to be put right in an older file.
struct Step {
    /// The version the file has to be *below* for this to apply.
    until: u32,
    /// What to say to the user, if it changes anything.
    said: &'static str,
    apply: fn(&mut Config) -> bool,
}

const STEPS: &[Step] = &[
    Step {
        until: 1,
        said: "Scrolling the contact sheet moved from Space to PageDown, because \
               Space now picks a photograph out",
        apply: scroll_off_the_space_bar,
    },
    Step {
        until: 1,
        said: "Showing more or fewer images side by side moved to Ctrl with \
               Plus and Minus, which is where it stopped fighting with zoom",
        apply: side_by_side_onto_ctrl,
    },
];

/// One thing that has to be put right in the *document*, before the typed
/// sections are built from it.
///
/// A step above works on `Config` and can only move a value from one field to
/// another field of the same name and type. A key that changes section, or
/// name, or type — `image_view.scroll_navigation`, a boolean, becoming
/// `mouse.wheel`, a job — is gone by the time `serde` has finished, and the
/// old value with it. So those are done on the `serde_json::Map` on the way
/// in, which is also where the old key can be taken out so that the merge on
/// the way back out does not put it back.
struct DocumentStep {
    until: u32,
    said: &'static str,
    apply: fn(&mut Map<String, Value>) -> bool,
}

const DOCUMENT_STEPS: &[DocumentStep] = &[
    DocumentStep {
        until: 2,
        said: "The wheel moved from image_view.scroll_navigation to mouse.wheel, which can say what it should do rather than only whether it does anything",
        apply: wheel_into_the_mouse_section,
    },
    DocumentStep {
        until: 3,
        said: "Undo moved from cull.sc_undo to history.sc_undo, because it now takes back settings and where you were looking as well as what was done to a file",
        apply: undo_into_the_history_section,
    },
    DocumentStep {
        until: 4,
        said: "Keeping the window filled moved from image_view.sc_latch_fit_maximize to image_view.sc_cycle_opening, which now goes round all three answers to what a photograph opens at rather than only the one",
        apply: the_latch_becomes_the_opening,
    },
];

/// Brings a document up to [`CURRENT`], returning what was changed.
///
/// Called before the sections are read, so what it writes is what `serde`
/// sees. The version is read out of the document itself, because the `Config`
/// that would carry it does not exist yet.
pub fn document(map: &mut Map<String, Value>) -> Vec<&'static str> {
    let from = map.get("version").and_then(Value::as_u64).unwrap_or(0) as u32;

    let mut changed = Vec::new();

    if from >= CURRENT {
        return changed;
    }

    for step in DOCUMENT_STEPS {
        if from < step.until && (step.apply)(map) {
            changed.push(step.said);
        }
    }

    changed
}

/// `image_view.scroll_navigation` becomes `mouse.wheel`.
///
/// `true` was "one notch is one photograph" and becomes *next or previous*.
/// `false` becomes *pan* rather than *nothing*, because *nothing* is not what
/// the program did: the scroll delta reached the viewport whatever the flag
/// said, so with the flag off the wheel moved the photograph about. Carrying
/// it across as *nothing* would hand somebody a dead wheel and call it their
/// own setting — which is nomacs #1281, where a checkbox unticked left the
/// wheel doing nothing at all.
///
/// A file that already says something about `mouse.wheel` is left alone: the
/// newer key is the deliberate one.
fn wheel_into_the_mouse_section(map: &mut Map<String, Value>) -> bool {
    let was = map
        .get_mut("image_view")
        .and_then(Value::as_object_mut)
        .and_then(|section| section.remove("scroll_navigation"));

    let Some(Value::Bool(navigated)) = was else {
        return false;
    };

    let mouse = map
        .entry("mouse")
        .or_insert_with(|| Value::Object(Map::new()));

    let Some(mouse) = mouse.as_object_mut() else {
        return false;
    };

    if mouse.contains_key("wheel") {
        // The old key is still gone: it means nothing to this build, and
        // leaving it would put it back on the next save.
        return true;
    }

    let job = if navigated { "next_or_previous" } else { "pan" };
    mouse.insert("wheel".to_string(), Value::String(job.to_string()));
    true
}

/// `cull.sc_undo` becomes `history.sc_undo`.
///
/// Undo was filed under culling because the journal it walked covered nothing
/// but moving, copying and binning. It walks everything now, so the key moves
/// with it — and whatever the user had bound goes across, because a rebind
/// silently reverted to `Ctrl + Z` is the migration doing the one thing it
/// exists to prevent.
///
/// A file that already says something about `history.sc_undo` is left alone:
/// the newer key is the deliberate one.
fn undo_into_the_history_section(map: &mut Map<String, Value>) -> bool {
    let was = map
        .get_mut("cull")
        .and_then(Value::as_object_mut)
        .and_then(|section| section.remove("sc_undo"));

    let Some(bound) = was else {
        return false;
    };

    let history = map
        .entry("history")
        .or_insert_with(|| Value::Object(Map::new()));

    let Some(history) = history.as_object_mut() else {
        return false;
    };

    if history.contains_key("sc_undo") {
        // The old key is still gone: it means nothing to this build, and
        // leaving it would put it back on the next save.
        return true;
    }

    history.insert("sc_undo".to_string(), bound);
    true
}

/// The latch that kept the window filled is now one of three openings.
///
/// The key kept its binding and its place on the keyboard and lost only its
/// name, so the value is carried across rather than dropped: somebody who
/// moved it off `Ctrl + M` would otherwise find it back there.
///
/// A file that already says something about the new key is left alone, on the
/// same rule as every other step here: what the newer build wrote is the
/// deliberate one.
fn the_latch_becomes_the_opening(map: &mut Map<String, Value>) -> bool {
    let Some(section) = map.get_mut("image_view").and_then(Value::as_object_mut) else {
        return false;
    };

    let Some(bound) = section.remove("sc_latch_fit_maximize") else {
        return false;
    };

    if section.contains_key("sc_cycle_opening") {
        // The old key is still gone: it means nothing to this build, and
        // leaving it would put it back on the next save.
        return true;
    }

    section.insert("sc_cycle_opening".to_string(), bound);
    true
}

/// Brings `config` up to [`CURRENT`], returning what was changed.
///
/// Nothing is reported for a file that was already current, which is the
/// ordinary case and should be silent.
pub fn apply(config: &mut Config) -> Vec<&'static str> {
    let from = config.version;
    let mut changed = Vec::new();

    // A file that has never been written carries no version and needs every
    // step; one from the future is left alone entirely, because this build
    // cannot know what a later one meant.
    if from >= CURRENT {
        config.version = config.version.max(CURRENT);
        return changed;
    }

    for step in STEPS {
        if from < step.until && (step.apply)(config) {
            changed.push(step.said);
        }
    }

    config.version = CURRENT;
    changed
}

/// Space was "scroll down half a row" and is now "pick this one out".
///
/// Both would match the same key, and the selection is claimed first, so an
/// untouched older file would find its scroll key had quietly stopped working.
fn scroll_off_the_space_bar(config: &mut Config) -> bool {
    let was = Shortcut::new("Space", &[]);
    if config.grid_view.sc_scroll != was {
        return false;
    }

    config.grid_view.sc_scroll = defaults::default_sc_scroll();
    true
}

/// Plus and Minus were both "more images side by side" and "zoom in".
///
/// The default moved to Ctrl with them, but a file written before that keeps
/// the old binding for ever — `serde` fills in the keys that are missing, not
/// the ones that have since moved. Both commands then matched the same key,
/// zoom won, and the side-by-side view was simply unreachable with no hint as
/// to why. This is the case the startup clash warning was written for, found
/// on a real configuration; the warning says so, and now the file is put right
/// as well.
fn side_by_side_onto_ctrl(config: &mut Config) -> bool {
    let mut moved = false;

    for (setting, was, now) in [
        (
            &mut config.image_view.sc_more_images_shown,
            "Plus",
            defaults::default_sc_more_images_shown(),
        ),
        (
            &mut config.image_view.sc_less_images_shown,
            "Minus",
            defaults::default_sc_less_images_shown(),
        ),
    ] {
        if *setting == Shortcut::new(was, &[]) {
            *setting = now;
            moved = true;
        }
    }

    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one key this plan moves, and the shape of move a typed `Config`
    /// cannot make: a boolean in one section becomes a named job in another.
    #[test]
    fn the_wheel_moves_out_of_the_image_view() {
        let mut map = document(r#"{"image_view": {"scroll_navigation": true}}"#);
        let said = super::document(&mut map);

        assert_eq!(said.len(), 1);
        assert_eq!(map["mouse"]["wheel"], serde_json::json!("next_or_previous"));
        assert!(
            map["image_view"].get("scroll_navigation").is_none(),
            "and the old key is gone, or the next save would put it back"
        );
    }

    /// Off did not mean a dead wheel: the delta reached the viewport whatever
    /// the flag said, so the wheel moved the photograph about. That is what is
    /// carried across, because carrying `nothing` across would hand somebody a
    /// wheel that does nothing and call it their own setting.
    #[test]
    fn the_wheel_turned_off_becomes_what_it_actually_did() {
        let mut map = document(r#"{"image_view": {"scroll_navigation": false}}"#);
        super::document(&mut map);

        assert_eq!(map["mouse"]["wheel"], serde_json::json!("pan"));
    }

    /// The one thing this migration must not do: a rebind silently reverted
    /// to the default would be the migration causing what it exists to prevent.
    #[test]
    fn a_rebound_undo_key_survives_moving_section() {
        let mut map = document(r#"{"cull": {"sc_undo": {"key": "u", "modifiers": ["alt"]}}}"#);
        let said = super::document(&mut map);

        assert_eq!(said.len(), 1);
        assert_eq!(map["history"]["sc_undo"]["key"], serde_json::json!("u"));
        assert_eq!(
            map["history"]["sc_undo"]["modifiers"],
            serde_json::json!(["alt"])
        );
        assert!(
            map["cull"].get("sc_undo").is_none(),
            "and the old key is gone, or the next save would put it back"
        );
    }

    /// A file that never said anything about the key is not given one: the
    /// default arrives through `serde` like every other absent field.
    #[test]
    fn a_file_that_never_bound_undo_gains_nothing() {
        let mut map = document(r#"{"cull": {"rejected_folder": "_Rejected"}}"#);
        let said = super::document(&mut map);

        assert!(said.is_empty());
        assert!(map.get("history").is_none());
    }

    /// The same rule as the wheel: a newer key written deliberately wins.
    #[test]
    fn an_undo_key_already_in_the_history_section_is_left_alone() {
        let mut map = document(
            r#"{"cull": {"sc_undo": {"key": "u", "modifiers": []}},
                "history": {"sc_undo": {"key": "z", "modifiers": ["ctrl"]}}}"#,
        );
        super::document(&mut map);

        assert_eq!(map["history"]["sc_undo"]["key"], serde_json::json!("z"));
        assert!(map["cull"].get("sc_undo").is_none());
    }

    /// The key kept its binding and its place and lost only its name, so what
    /// somebody bound it to has to come across with it.
    #[test]
    fn a_rebound_filling_latch_becomes_the_opening_key() {
        let mut map = document(
            r#"{"image_view": {"sc_latch_fit_maximize": {"key": "k", "modifiers": ["alt"]}}}"#,
        );
        let said = super::document(&mut map);

        assert_eq!(said.len(), 1);
        assert_eq!(
            map["image_view"]["sc_cycle_opening"]["key"],
            serde_json::json!("k")
        );
        assert_eq!(
            map["image_view"]["sc_cycle_opening"]["modifiers"],
            serde_json::json!(["alt"])
        );
        assert!(
            map["image_view"].get("sc_latch_fit_maximize").is_none(),
            "and the old key is gone, or the next save would put it back"
        );
    }

    /// The same rule as the other two: what a newer build wrote wins.
    #[test]
    fn an_opening_key_already_written_is_left_alone() {
        let mut map = document(
            r#"{"image_view": {"sc_latch_fit_maximize": {"key": "k", "modifiers": []},
                               "sc_cycle_opening": {"key": "m", "modifiers": ["ctrl"]}}}"#,
        );
        super::document(&mut map);

        assert_eq!(
            map["image_view"]["sc_cycle_opening"]["key"],
            serde_json::json!("m")
        );
        assert!(map["image_view"].get("sc_latch_fit_maximize").is_none());
    }

    /// The rule that makes every migration safe: a newer key that has been
    /// written deliberately wins over an older one being brought forward.
    #[test]
    fn a_wheel_already_set_is_left_alone() {
        let mut map =
            document(r#"{"image_view": {"scroll_navigation": true}, "mouse": {"wheel": "zoom"}}"#);
        super::document(&mut map);

        assert_eq!(map["mouse"]["wheel"], serde_json::json!("zoom"));
        assert!(map["image_view"].get("scroll_navigation").is_none());
    }

    /// A current file is not touched at all, which is the ordinary case.
    #[test]
    fn a_current_file_is_left_alone() {
        let mut map = document(&format!(
            r#"{{"version": {CURRENT}, "image_view": {{"scroll_navigation": true}}}}"#
        ));

        assert!(super::document(&mut map).is_empty());
        assert!(map.get("mouse").is_none());
    }

    /// And the move is reported, so somebody whose wheel changes hands is
    /// told why rather than left to find out.
    #[test]
    fn the_move_reaches_the_reader_of_the_file() {
        let config = Config::from_json(r#"{"image_view": {"scroll_navigation": false}}"#);

        assert_eq!(config.mouse.wheel, crate::config::WheelJob::Pan);
        assert!(config
            .migrated
            .iter()
            .any(|said| said.contains("mouse.wheel")));
    }

    fn document(json: &str) -> Map<String, Value> {
        serde_json::from_str(json).expect("the test document parses")
    }

    /// A file from before versions existed gets every step.
    fn ancient() -> Config {
        Config {
            version: 0,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("Space", &[]),
                ..Default::default()
            },
            ..Config::default()
        }
    }

    #[test]
    fn an_old_file_has_its_scroll_key_moved() {
        let mut config = ancient();
        let said = apply(&mut config);

        assert_eq!(said.len(), 1);
        assert_eq!(config.grid_view.sc_scroll, defaults::default_sc_scroll());
        assert_eq!(config.version, CURRENT);
    }

    /// The clash that made the side-by-side view unreachable: both commands
    /// on a bare Plus, with zoom winning.
    #[test]
    fn an_old_file_has_its_side_by_side_keys_moved() {
        let mut config = Config {
            version: 0,
            image_view: super::super::ImageViewConfig {
                sc_more_images_shown: Shortcut::new("Plus", &[]),
                sc_less_images_shown: Shortcut::new("Minus", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        let said = apply(&mut config);

        assert_eq!(said.len(), 1);
        assert_eq!(
            config.image_view.sc_more_images_shown,
            defaults::default_sc_more_images_shown()
        );
        assert_eq!(
            config.image_view.sc_less_images_shown,
            defaults::default_sc_less_images_shown()
        );

        // And the clash it was for is gone.
        assert!(crate::ui::keys::clashes(&config).is_empty());
    }

    /// The rule that makes migrations safe: a key the user chose is theirs.
    #[test]
    fn a_key_the_user_chose_is_left_alone() {
        let mut config = Config {
            version: 0,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("j", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        let said = apply(&mut config);

        assert!(said.is_empty());
        assert_eq!(config.grid_view.sc_scroll, Shortcut::new("j", &[]));
        assert_eq!(config.version, CURRENT);
    }

    #[test]
    fn a_current_file_is_not_touched_and_says_nothing() {
        let mut config = Config {
            version: CURRENT,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("Space", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        assert!(apply(&mut config).is_empty());
        assert_eq!(config.grid_view.sc_scroll, Shortcut::new("Space", &[]));
    }

    /// A file from a build newer than this one is not "migrated" backwards.
    #[test]
    fn a_file_from_the_future_is_left_as_it_is() {
        let mut config = Config {
            version: CURRENT + 5,
            grid_view: super::super::GridViewConfig {
                sc_scroll: Shortcut::new("Space", &[]),
                ..Default::default()
            },
            ..Config::default()
        };

        assert!(apply(&mut config).is_empty());
        assert_eq!(config.version, CURRENT + 5);
        assert_eq!(config.grid_view.sc_scroll, Shortcut::new("Space", &[]));
    }

    /// Migrating twice does nothing the second time.
    #[test]
    fn migrating_is_idempotent() {
        let mut config = ancient();

        assert_eq!(apply(&mut config).len(), 1);
        assert!(apply(&mut config).is_empty());
    }

    /// A default freshly built by this version is already current, so nobody
    /// starting today is told anything.
    #[test]
    fn a_new_configuration_needs_no_migration() {
        let mut config = Config::default();

        assert_eq!(config.version, CURRENT);
        assert!(apply(&mut config).is_empty());
    }

    /// Every step has to be reachable, and to say something.
    #[test]
    fn every_step_is_within_the_current_version() {
        for step in STEPS {
            assert!(step.until <= CURRENT, "{} is unreachable", step.said);
            assert!(!step.said.is_empty());
        }
    }
}
