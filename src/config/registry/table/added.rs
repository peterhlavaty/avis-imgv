//! The fields that did not exist until the window needed them.
//!
//! Mostly persistence rather than choice: a startup state the program already
//! decided by hand, a constant nobody could reach, a runtime toggle the program
//! kept and threw away on exit. They live here rather than in their section's
//! own file so that what was added is visible as a list.

use super::*;

const MODES: &[Choice] = &[
    Choice {
        value: "image",
        label: "The photograph",
        sentence: "One photograph at a time, which is what a viewer is.",
    },
    Choice {
        value: "gallery",
        label: "The contact sheet",
        sentence: "Every photograph in the folder as a thumbnail.",
    },
    Choice {
        value: "slideshow",
        label: "Slideshow",
        sentence: "Fullscreen, changing itself.",
    },
];

const THEMES: &[Choice] = &[
    Choice {
        value: "dark",
        label: "Dark",
        sentence: "What the viewer has always been.",
    },
    Choice {
        value: "light",
        label: "Light",
        sentence: "The interface only. What surrounds the photograph is the backdrop \
                   below, which this does not touch.",
    },
];

const BADGES: &[Choice] = &[
    Choice {
        value: "none",
        label: "Nothing",
        sentence: "The picture alone.",
    },
    Choice {
        value: "marks",
        label: "The marks",
        sentence: "Stars, flag and colour label.",
    },
    Choice {
        value: "full",
        label: "The marks and the name",
        sentence: "",
    },
];

const EDGES: &[Choice] = &[
    Choice {
        value: "bottom",
        label: "Below the photograph",
        sentence: "",
    },
    Choice {
        value: "top",
        label: "Above it",
        sentence: "",
    },
];

const SIDECARS: &[Choice] = &[
    Choice {
        value: "with_extension",
        label: "photo.cr2.xmp",
        sentence: "What darktable and exiftool write. The only one of the two that can \
                   tell a raw's keywords from its JPEG twin's, which is why it is the \
                   default.",
    },
    Choice {
        value: "replacing",
        label: "photo.xmp",
        sentence: "What Adobe writes. Lightroom will see the ratings; a raw and a JPEG \
                   of one frame will share them.",
    },
];

const PANELS: &[Choice] = &[
    Choice {
        value: "menu",
        label: "The menu bar",
        sentence: "",
    },
    Choice {
        value: "side_panel",
        label: "The metadata panel",
        sentence: "",
    },
    Choice {
        value: "tag_panel",
        label: "The stars and keywords panel",
        sentence: "",
    },
    Choice {
        value: "filmstrip",
        label: "The strip of thumbnails",
        sentence: "",
    },
];

const CONFIRMATIONS: &[Choice] = &[
    Choice {
        value: "bin_several",
        label: "Moving more than one photograph to the bin",
        sentence: "",
    },
    Choice {
        value: "empty_rejects",
        label: "Emptying the rejects",
        sentence: "",
    },
    Choice {
        value: "undo_several",
        label: "Undoing a step that touched more than one file",
        sentence: "",
    },
];

pub fn rows() -> Vec<Row> {
    vec![
        row!(
            OpeningAFolder / Starting,
            "general.start_in",
            "Start in",
            "Which mode a launch opens in. A path on the command line, then the \
             restored session, then the startup folder, then the working directory — \
             this decides only what is drawn, not what is opened.",
            ["startup", "start", "mode", "launch", "open in"],
            Restart,
            None,
            Access::Enum {
                get: |c| {
                    MODES
                        .iter()
                        .find(|choice| choice.value == c.general.start_in)
                        .map(|choice| choice.value)
                        .unwrap_or("image")
                },
                set: |c, v| c.general.start_in = v.to_string(),
                choices: MODES,
            },
        ),
        row!(
            OpeningAFolder / Starting,
            "general.start_fullscreen",
            "Start fullscreen",
            "`--fullscreen` says this for one launch; this says it for every launch.",
            ["fullscreen", "startup", "maximise", "screen"],
            Restart,
            None,
            boolean!(general.start_fullscreen),
        ),
        row!(
            OpeningAFolder / Starting,
            "general.start_folder",
            "Folder to open",
            "Reached only when no path was given and there is no session to restore. \
             Without it the viewer reads the working directory of whatever launched \
             it, which is nobody's choice.",
            ["startup", "folder", "home", "default folder"],
            Restart,
            None,
            optional_text!(general.start_folder),
        ),
        row!(
            TheWindow / Panels,
            "general.panels_at_start",
            "Panels a launch starts with",
            "Which panels are up when the window opens. Each of them has a key as \
             well, and what the key leaves them at is remembered from here on.",
            ["panels", "startup", "sidebar", "menu bar", "layout"],
            Restart,
            None,
            Access::Flags {
                get: |c, name| c.general.panels_at_start.get(name),
                set: |c, name, on| c.general.panels_at_start.set(name, on),
                options: PANELS,
            },
        ),
        row!(
            TheWindow / Panels,
            "general.side_panel_width",
            "How wide the metadata panel is",
            "Dragging its edge changes this too, and it survives the session now — \
             which is the thing none of the viewer's in-view controls used to do.",
            ["panel", "width", "sidebar", "metadata"],
            Live,
            None,
            decimal!(220.0, 900.0, " pt", true, general.side_panel_width),
        ),
        row!(
            TheWindow / Appearance,
            "general.theme",
            "Light or dark",
            "The interface: the menus, the panels, the windows. Not the ground behind \
             the photograph, which is the backdrop below — a theme setting that \
             changed the backdrop would be answering a question nobody asked.",
            ["theme", "light", "dark", "colour", "color", "appearance"],
            Live,
            None,
            Access::Enum {
                get: |c| {
                    THEMES
                        .iter()
                        .find(|choice| choice.value == c.general.theme)
                        .map(|choice| choice.value)
                        .unwrap_or("dark")
                },
                set: |c, v| c.general.theme = v.to_string(),
                choices: THEMES,
            },
        ),
        row!(
            TheWindow / Appearance,
            "general.backdrop",
            "The ground behind the photograph",
            "A middle grey, because it is neutral enough not to shift how a photograph \
             reads against it. The contact sheet's cells and the filmstrip derive \
             theirs from it.",
            [
                "background",
                "backdrop",
                "grey",
                "gray",
                "surround",
                "canvas"
            ],
            Live,
            None,
            Access::Colour(
                |c| Some(c.general.backdrop.clone()),
                |c, v| c.general.backdrop = v.unwrap_or_else(crate::config::default_backdrop),
            ),
        ),
        row!(
            TheWindow / Footer,
            "general.last_settings_page",
            "The page this window was left on",
            "Written by the window and set by nobody, which is why it has a row and no \
             control: a key the registry has not heard of fails the build.",
            ["settings", "page", "remembered"],
            None,
            None,
            Access::ReadOnly(|c| c.general.last_settings_page.clone()),
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.zoom_step",
            "How much one zoom key changes it",
            "A quarter more each press. Every movement key in the viewer was a \
             compile-time constant until now.",
            ["zoom", "step", "magnify", "increment"],
            Live,
            None,
            decimal!(1.01, 4.0, "×", true, image_view.zoom_step),
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.zoom_step_factor",
            "How much the step-zoom key changes it",
            "Doubling, by default: that key exists to get from fitted to something \
             worth judging in as few presses as possible.",
            ["zoom", "double", "step", "space"],
            Live,
            None,
            decimal!(1.1, 8.0, "×", true, image_view.zoom_step_factor),
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.zoom_step_max",
            "How far it goes before starting again",
            "The step-zoom key wraps back to fitted once it passes this, so one key \
             both magnifies and gets out.",
            ["zoom", "maximum", "wrap", "limit"],
            Live,
            None,
            decimal!(2.0, 64.0, "×", true, image_view.zoom_step_max),
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.pan_speed",
            "How fast a held pan key moves",
            "In screenfuls a second, so it is the same speed whatever the window is \
             and whatever the photograph is magnified to.",
            ["pan", "speed", "scroll", "arrows"],
            Live,
            None,
            decimal!(0.1, 8.0, " screens/s", true, image_view.pan_speed),
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.page",
            "How many a screenful is",
            "What PageUp and PageDown move by, for walking a long folder quickly.",
            ["page", "skip", "jump", "pageup", "pagedown"],
            Live,
            None,
            whole!(usize, 1, 500, "", true, image_view.page),
        ),
        row!(
            TheContactSheet / Cells,
            "grid_view.badges",
            "What is drawn under a cell",
            "What a folder opens showing. The key cycles the three for the session.",
            ["badges", "caption", "marks", "name", "under"],
            Live,
            None,
            Access::Enum {
                get: |c| {
                    BADGES
                        .iter()
                        .find(|choice| choice.value == c.grid_view.badges)
                        .map(|choice| choice.value)
                        .unwrap_or("marks")
                },
                set: |c, v| c.grid_view.badges = v.to_string(),
                choices: BADGES,
            },
        ),
        row!(
            TheContactSheet / Filmstrip,
            "grid_view.filmstrip_visible",
            "Show the strip",
            "Split out of the height, which stored a height and a visibility in one \
             number — which is why the key that shows the strip did nothing on a fresh \
             install: the default height is zero.",
            ["filmstrip", "strip", "thumbnails", "show"],
            Live,
            None,
            boolean!(grid_view.filmstrip_visible),
        ),
        row!(
            TheContactSheet / Filmstrip,
            "grid_view.filmstrip_edge",
            "Which edge it sits against",
            "Below the photograph, or above it.",
            ["filmstrip", "strip", "top", "bottom", "position"],
            Live,
            None,
            Access::Enum {
                get: |c| {
                    EDGES
                        .iter()
                        .find(|choice| choice.value == c.grid_view.filmstrip_edge)
                        .map(|choice| choice.value)
                        .unwrap_or("bottom")
                },
                set: |c, v| c.grid_view.filmstrip_edge = v.to_string(),
                choices: EDGES,
            },
        ),
        row!(
            TheContactSheet / Cells,
            "grid_view.click_opens",
            "A single click opens a photograph",
            "Off: a click picks out and a double click opens. A culling tool's contact \
             sheet is a surface you act *on*, and a plain click that closes it \
             contradicts the cursor, the selection, Ctrl-click, Shift-click, Space and \
             Enter all at once.",
            ["click", "double click", "open", "select", "mouse"],
            Live,
            None,
            boolean!(grid_view.click_opens),
        ),
        row!(
            Marks / Plain,
            "tags.sidecar_naming",
            "Name new sidecars",
            "Both forms are read, most specific first, and a sidecar that already \
             exists is edited rather than joined by a second. This is only what gets \
             created for a photograph that has none.",
            [
                "sidecar",
                "xmp",
                "lightroom",
                "adobe",
                "darktable",
                "naming"
            ],
            Live,
            None,
            Access::Enum {
                get: |c| {
                    SIDECARS
                        .iter()
                        .find(|choice| choice.value == c.tags.sidecar_naming)
                        .map(|choice| choice.value)
                        .unwrap_or("with_extension")
                },
                set: |c, v| c.tags.sidecar_naming = v.to_string(),
                choices: SIDECARS,
            },
        ),
        row!(
            KeysAndMouse / Menus,
            "menus.settings_rows",
            "Menus carry their settings rows",
            "Turning it off leaves the verbs, your own entries, the copy group and the \n             last row, so nothing becomes unreachable. It is the whole of the \n             configurability offered for the built-in rows, and the reason there is no \n             menu editor.",
            ["menu", "right click", "context menu", "rows", "settings"],
            Live,
            None,
            boolean!(menus.settings_rows),
        ),
        row!(
            MovingAndDeleting / Confirmations,
            "cull.confirm",
            "Ask before",
            "Only what the undo journal covers can be switched off. Deleting for good \
             and putting the keyboard back have no inverse anywhere, so both always \
             ask and neither is here: a confirmation is not a substitute for \
             reversibility, because people click through dialogues by reflex.",
            ["confirm", "ask", "dialog", "dialogue", "warning", "prompt"],
            Live,
            None,
            Access::Flags {
                get: |c, name| c.cull.confirm.get(name),
                set: |c, name, on| c.cull.confirm.set(name, on),
                options: CONFIRMATIONS,
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Confirmations, PanelsAtStart};

    /// The names the flag rows use have to be the ones the structs answer to,
    /// or a tick would set nothing and say nothing.
    #[test]
    fn every_flag_name_is_one_the_struct_knows() {
        for choice in PANELS {
            assert!(
                PanelsAtStart::NAMES.contains(&choice.value),
                "{} is not a panel",
                choice.value
            );
        }

        for choice in CONFIRMATIONS {
            assert!(
                Confirmations::NAMES.contains(&choice.value),
                "{} is not a confirmation",
                choice.value
            );
        }
    }

    /// And every part of each struct is on its row, or a decision would be
    /// unreachable.
    #[test]
    fn no_part_of_a_set_is_left_off_its_row() {
        for name in PanelsAtStart::NAMES {
            assert!(PANELS.iter().any(|choice| choice.value == *name), "{name}");
        }

        for name in Confirmations::NAMES {
            assert!(
                CONFIRMATIONS.iter().any(|choice| choice.value == *name),
                "{name}"
            );
        }
    }
}
