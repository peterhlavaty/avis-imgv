//! `image_view.*`: how one photograph is drawn, and the keys read while it is
//! on screen.

use super::*;

/// Where the photograph's own details are drawn, if at all.
const CORNERS: &[Choice] = &[
    Choice {
        value: "off",
        label: "Not drawn",
        sentence: "Nothing over the picture.",
    },
    Choice {
        value: "top_left",
        label: "Top left",
        sentence: "",
    },
    Choice {
        value: "top_right",
        label: "Top right",
        sentence: "",
    },
    Choice {
        value: "bottom_left",
        label: "Bottom left",
        sentence: "",
    },
    Choice {
        value: "bottom_right",
        label: "Bottom right",
        sentence: "",
    },
];

/// What a photograph is drawn at on the frame it first appears.
///
/// The words are `Opening`'s own, and a test at the foot of this file says so:
/// a choice worded twice is a choice that drifts.
const OPENINGS: &[Choice] = &[
    Choice {
        value: "fit",
        label: "Fitted to the window",
        sentence: "The whole photograph, as large as the window will take it.",
    },
    Choice {
        value: "fill",
        label: "Filling the window",
        sentence: "Covers the window, cropping whichever side is longer. Nothing is \
                   lost — the rest is a pan away.",
    },
    Choice {
        value: "width",
        label: "As wide as the window",
        sentence: "As wide as the window, with the top and the bottom cropped if the \
                   photograph is taller than it is wide.",
    },
    Choice {
        value: "height",
        label: "As tall as the window",
        sentence: "As tall as the window, with the sides cropped. What a folder of \
                   panoramas wants.",
    },
    Choice {
        value: "percent",
        label: "At a magnification you choose",
        sentence: "At the magnification below, against the photograph's own pixels: a \
                   hundred per cent is one screen pixel to one of theirs, which is what \
                   focus is judged at.",
    },
];

pub fn rows() -> Vec<Row> {
    let mut rows = vec![
        row!(
            ThePhotograph / Overlay,
            "image_view.overlay_corner",
            "Where the details are drawn",
            "The photograph's own details, in a corner of the picture itself rather \
             than beside it. The key moves them round the corners and off again.",
            ["overlay", "exif on image", "info", "corner"],
            Live,
            None,
            Access::Enum {
                get: |c| c.image_view.overlay_corner.value(),
                set: |c, v| {
                    if let Some(corner) = crate::view::image_view::overlay::Corner::of(v) {
                        c.image_view.overlay_corner = corner;
                    }
                },
                choices: CORNERS,
            },
        ),
        row!(
            ThePhotograph / Overlay,
            "image_view.overlay_format",
            "What the details say",
            "One line per line, in the placeholder grammar. A fragment inside $( ) \
             disappears whole when what is in it is missing, so a photograph with no \
             lens recorded does not read as a row of separators.",
            ["overlay", "format", "template", "exif on image"],
            Live,
            None,
            template!(image_view.overlay_format),
        ),
        row!(
            ThePhotograph / Overlay,
            "image_view.overlay_text_size",
            "How large those details are",
            "In points, against the photograph rather than the window: the line stays \
             the same size on screen whatever the picture is magnified to.",
            ["overlay", "font", "size"],
            Live,
            None,
            decimal!(6.0, 48.0, " pt", true, image_view.overlay_text_size),
        ),
        row!(
            SpeedAndMemory / Memory,
            "image_view.nr_loaded_images",
            "Photographs kept ready either side",
            "How far ahead and behind the cursor the viewer decodes. It is trimmed to \
             whatever the RAM budget actually holds, so the number in the file has \
             never been the number in force: at the default budget it is about 22 on a \
             4K monitor and about 90 on a 1080p one.",
            ["preload", "ahead", "radius", "cache"],
            Rebuild,
            None,
            whole!(usize, 1, 4096, "", false, image_view.nr_loaded_images),
            explained: "No control: the RAM budget above already decides this, and \n                        trims whatever is written here to what it holds. At the \n                        default budget it is about 22 on a 4K monitor and about 90 on \n                        a 1080p one. The readout beside the budget says the number in \n                        force.",
        ),
        row!(
            SpeedAndMemory / Graphics,
            "image_view.gpu_resident_images",
            "Photographs kept on the graphics card",
            "A count where the real bound is bytes, which is what the graphics budget \
             beside it is for: two hundred thumbnails and two hundred 60 megapixel \
             photographs are the same number and a thousandfold difference in what the \
             card is holding.",
            ["gpu", "vram", "textures", "resident"],
            Rebuild,
            None,
            whole!(usize, 1, 512, "", false, image_view.gpu_resident_images),
            explained: "No control: the graphics card budget bounds this in bytes, \n                        which is the honest bound. A count cannot tell two hundred \n                        thumbnails from two hundred sixty-megapixel photographs.",
        ),
        row!(
            SpeedAndMemory / Memory,
            "image_view.max_image_edge",
            "Longest edge decoded to",
            "The ceiling on how large a photograph is decoded, whatever the screen. \
             Zero means the screen decides, which is what it should normally do: a \
             monitor shows three megapixels and the file has twenty-four.",
            ["resolution", "downscale", "size", "edge", "quality"],
            Rebuild,
            None,
            whole!(u32, 0, 32768, " px", true, image_view.max_image_edge),
        ),
        row!(
            ThePhotograph / Framing,
            "image_view.nr_images_shown",
            "Photographs side by side",
            "How many are drawn at once, sharing one zoom and one pan. The keys change \
             it for the session; this is what a launch starts with.",
            ["compare", "side by side", "panes", "multiple"],
            Live,
            None,
            whole!(usize, 1, 8, "", true, image_view.nr_images_shown),
        ),
        row!(
            ThePhotograph / Framing,
            "image_view.marked_area_dim",
            "Darkening around a marked area",
            "How far the rest of the photograph is darkened while a part of it is \
             marked out. Nought leaves it alone, which is what somebody judging an \
             exposure against its surroundings wants.",
            ["crop", "marquee", "selection", "marked", "dim", "darken"],
            Live,
            None,
            whole!(u8, 0, 90, " %", true, image_view.marked_area_dim),
            explained: "The marking itself is drawn as a white line with a dark one \
                        outside it, so that it is visible against a bright sky and \
                        against a shadow without being a colour anybody chose.",
        ),
        row!(
            ThePhotograph / Framing,
            "image_view.should_wait",
            "Wait for the photograph before drawing it",
            "Holds the photograph that is on screen until the next one is decoded, \
             rather than showing an empty panel while it arrives. On a fast folder the \
             difference is invisible; on a slow share it is the difference between a \
             flicker and a pause.",
            ["flicker", "blank", "wait", "loading"],
            Live,
            None,
            boolean!(image_view.should_wait),
        ),
        row!(
            ThePhotograph / Framing,
            "image_view.frame_size_relative_to_image",
            "How wide the white frame is",
            "As a fraction of the photograph's shorter edge, so a frame round a \
             landscape and one round a portrait read the same. The key shows and hides \
             it.",
            ["border", "frame", "white", "matte"],
            Live,
            None,
            // Per cent of the shorter edge. The file holds 0.2; a person means
            // twenty per cent, and one of those is not a number anybody can
            // picture.
            Access::Float {
                get: |c| c.image_view.frame_size_relative_to_image * 100.0,
                set: |c, v| c.image_view.frame_size_relative_to_image = v / 100.0,
                min: 0.0,
                max: 50.0,
                unit: " %",
                rail: true,
            },
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.opening",
            "What a photograph opens at",
            "Fitted, which is what a viewer has always done — or filling the window, \
             or its own size, which is the magnification focus is judged at. \
             Whatever a photograph was last left at wins over this, and so do \
             the two toggles in the status bar.",
            [
                "opening",
                "default zoom",
                "starts at",
                "fit",
                "fill",
                "fit width",
                "fit height",
                "100%",
                "actual size"
            ],
            Live,
            None,
            Access::Enum {
                get: |c| c.image_view.opening.value(),
                set: |c, v| {
                    if let Some(opening) = crate::view::image_view::opening::Opening::of(v) {
                        c.image_view.opening = opening;
                    }
                },
                choices: OPENINGS,
            },
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.opening_percent",
            "The magnification it opens at",
            "What *at a magnification you choose* means, in per cent of the \
             photograph's own pixels. A hundred is one screen pixel to one of theirs. \
             Kept whatever the choice above says, so switching away and back does not \
             lose the number.",
            [
                "opening",
                "percent",
                "magnification",
                "100%",
                "default zoom"
            ],
            Live,
            None,
            decimal!(1.0, 1600.0, " %", true, image_view.opening_percent),
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.keep_zoom",
            "Keep the magnification from one photograph to the next",
            "Off. On, every photograph arrives at the magnification the last one was \
             at, whatever it opens at and whatever it was itself left at — which is \
             how a burst is gone through at a hundred per cent. The green and red \
             magnifying glass in the status bar is the same switch, where somebody \
             turning it on for ten minutes can reach it.",
            [
                "keep zoom",
                "lock zoom",
                "same magnification",
                "carry",
                "burst"
            ],
            Live,
            None,
            boolean!(image_view.keep_zoom),
        ),
        row!(
            ThePhotograph / Movement,
            "image_view.keep_pan",
            "Keep where you are in the photograph",
            "Off. On, the next photograph arrives showing the same part of itself, \
             so the same corner of every frame comes up. The other half of keeping \
             the magnification, and separate from it because a hand-held sequence \
             moves and following it is what panning is for.",
            ["keep pan", "lock pan", "same corner", "position", "carry"],
            Live,
            None,
            boolean!(image_view.keep_pan),
        ),
        row!(
            ThePhotograph / Framing,
            "image_view.enlarge_to_fit",
            "Enlarge a small photograph to fit",
            "Whether a picture smaller than the window is blown up to fill it, or left \
             at its own size in the middle. Off is what a photographer usually wants: \
             an enlarged thumbnail looks like a soft photograph.",
            ["upscale", "small", "fit", "blurry"],
            Live,
            None,
            boolean!(image_view.enlarge_to_fit),
        ),
        row!(
            ThePhotograph / Plain,
            "image_view.name_format",
            "What the status bar calls the photograph",
            "In the placeholder grammar, on one line. The default is the file name and \
             the exposure, each fragment disappearing when the photograph cannot answer \
             it.",
            ["status bar", "title", "name", "caption", "template"],
            Live,
            None,
            template!(image_view.name_format),
        ),
        row!(
            KeysAndMouse / Menus,
            "image_view.user_actions",
            "Commands bound to keys",
            "Programs run on the photograph on screen, each with a key. No shell is \
             involved: the command is split into arguments first and the file name goes \
             into one of them, so a name with a space or an apostrophe in it cannot add \
             arguments of its own.",
            ["exec", "external", "open with", "command", "script"],
            Live,
            None,
            Access::Records(List::UserActions, |c| c.image_view.user_actions.len()),
        ),
        row!(
            KeysAndMouse / Menus,
            "image_view.context_menu",
            "Your own menu entries",
            "Appended under a separator to the menu the second button opens, in this \
             order. The built-in verbs above them are not configurable beyond being \
             turned off as a group.",
            ["right click", "context menu", "menu"],
            Live,
            None,
            Access::Records(List::ContextMenu, |c| c.image_view.context_menu.len()),
        ),
    ];

    rows.extend(keys());
    rows.extend(action_keys());
    rows
}

/// The shortcut on each user action.
///
/// This is the one shortcut in the file the keyboard editor could not reach,
/// which is why the shipped example's trash action can never fire: it is bound
/// to plain `delete` alongside `sc_delete`, and `input::collect` consumes the
/// event first. A fixed number of rows, since a static table cannot know how
/// many the file holds; a row whose action does not exist reports no shortcut
/// and the editor leaves it out.
fn action_keys() -> Vec<Row> {
    (0..MOST_ACTIONS)
        .map(|index| Row {
            page: Page::KeysAndMouse,
            group: Group::Keys,
            path: ACTION_PATHS[index],
            label: ACTION_LABELS[index],
            sentence: "Runs the command you configured on the photograph on screen.",
            aliases: &["exec", "external", "open with", "command", "action"],
            access: Access::ActionKey(index),
            effect: Effect::Live,
            scope: Scope::ImageView,
            explained: None,
        })
        .collect()
}

/// How many user actions can have a row.
///
/// Nine, because the digits are the gesture everywhere else in this program and
/// a tenth row nobody can reach teaches nothing.
const MOST_ACTIONS: usize = 9;

const ACTION_PATHS: &[&str] = &[
    "image_view.user_actions[0].shortcut",
    "image_view.user_actions[1].shortcut",
    "image_view.user_actions[2].shortcut",
    "image_view.user_actions[3].shortcut",
    "image_view.user_actions[4].shortcut",
    "image_view.user_actions[5].shortcut",
    "image_view.user_actions[6].shortcut",
    "image_view.user_actions[7].shortcut",
    "image_view.user_actions[8].shortcut",
];

const ACTION_LABELS: &[&str] = &[
    "Your command 1",
    "Your command 2",
    "Your command 3",
    "Your command 4",
    "Your command 5",
    "Your command 6",
    "Your command 7",
    "Your command 8",
    "Your command 9",
];

fn keys() -> Vec<Row> {
    vec![
        row!(KeysAndMouse / Keys, "image_view.sc_next", "Next image",
            "Move to the next picture in the folder.",
            ["forward", "advance"], Live, ImageView, key!(image_view.sc_next)),
        row!(KeysAndMouse / Keys, "image_view.sc_prev", "Previous image",
            "Move to the one before it.",
            ["back", "previous"], Live, ImageView, key!(image_view.sc_prev)),
        row!(KeysAndMouse / Keys, "image_view.sc_fit", "Fit",
            "Show the whole picture, as large as the window allows.",
            ["fit", "whole"], Live, ImageView, key!(image_view.sc_fit)),
        row!(KeysAndMouse / Keys, "image_view.sc_fit_maximize", "Fill",
            "Fill the window, cropping whichever side overflows.",
            ["fill", "crop"], Live, ImageView, key!(image_view.sc_fit_maximize)),
        row!(KeysAndMouse / Keys, "image_view.sc_cycle_opening", "How a photograph opens",
            "Move what a new photograph is drawn at round the three answers: fitted, \
             filling the window, its own size.",
            ["latch", "fill", "opening", "keep filling", "100%"],
            Live, ImageView, key!(image_view.sc_cycle_opening)),
        row!(KeysAndMouse / Keys, "image_view.sc_keep_zoom", "Keep the magnification",
            "Carry the magnification from one photograph to the next, or stop.",
            ["keep zoom", "lock zoom", "burst"],
            Live, ImageView, key!(image_view.sc_keep_zoom)),
        row!(KeysAndMouse / Keys, "image_view.sc_keep_pan", "Keep where you are",
            "Carry where in the photograph you are looking to the next one, or stop.",
            ["keep pan", "lock pan", "position"],
            Live, ImageView, key!(image_view.sc_keep_pan)),
        row!(KeysAndMouse / Keys, "image_view.sc_fit_horizontal", "Fit width",
            "Make the picture exactly as wide as the window.",
            ["width"], Live, ImageView, key!(image_view.sc_fit_horizontal)),
        row!(KeysAndMouse / Keys, "image_view.sc_fit_vertical", "Fit height",
            "Make it exactly as tall.",
            ["height"], Live, ImageView, key!(image_view.sc_fit_vertical)),
        row!(KeysAndMouse / Keys, "image_view.sc_zoom", "Zoom step",
            "Double the magnification, returning to fitted once it goes far enough.",
            ["zoom", "magnify"], Live, ImageView, key!(image_view.sc_zoom)),
        row!(KeysAndMouse / Keys, "image_view.sc_zoom_in", "Zoom in",
            "Magnify a little more.",
            ["zoom", "closer"], Live, ImageView, key!(image_view.sc_zoom_in)),
        row!(KeysAndMouse / Keys, "image_view.sc_zoom_out", "Zoom out",
            "Magnify a little less.",
            ["zoom", "further"], Live, ImageView, key!(image_view.sc_zoom_out)),
        row!(KeysAndMouse / Keys, "image_view.sc_one_to_one", "Actual pixels",
            "One screen pixel for each pixel of the photograph.",
            ["1:1", "100%", "actual"], Live, ImageView, key!(image_view.sc_one_to_one)),
        row!(KeysAndMouse / Keys, "image_view.sc_repeat_place", "Repeat the last view",
            "Put this picture at the zoom and position the last one was left at, for comparing two frames of the same thing.",
            ["same place", "compare", "sync"], Live, ImageView, key!(image_view.sc_repeat_place)),
        row!(KeysAndMouse / Keys, "image_view.sc_compare", "Compare",
            "Pin this picture and the next side by side, sharing one zoom and one pan. Tab moves which one the keys are about, Escape leaves.",
            ["compare", "side by side"], Live, ImageView, key!(image_view.sc_compare)),
        row!(KeysAndMouse / Keys, "image_view.sc_drop_pane", "Drop this pane",
            "Takes the focused photograph out of a comparison, leaving the others to re-tile.",
            ["compare", "close", "remove", "slash"], Live, ImageView, key!(image_view.sc_drop_pane)),
        row!(KeysAndMouse / Keys, "image_view.sc_go_to", "Go to a photograph by number",
            "Puts the cursor in the box in the status bar. It could be reached by clicking and by nothing else, because the key that would have landed in it means \"the other pane\" while comparing.",
            ["go to", "jump", "number", "position"], Live, ImageView, key!(image_view.sc_go_to)),
        row!(KeysAndMouse / Keys, "image_view.sc_zoom_in_fine", "Zoom in finely",
            "Magnify by the smaller step, for arriving at a framing rather than crossing a range. Alt, which is what the pan keys with Alt already mean.",
            ["zoom", "fine", "precise", "alt"], Live, ImageView, key!(image_view.sc_zoom_in_fine)),
        row!(KeysAndMouse / Keys, "image_view.sc_zoom_out_fine", "Zoom out finely",
            "The same, the other way.",
            ["zoom", "fine", "precise", "alt"], Live, ImageView, key!(image_view.sc_zoom_out_fine)),
        row!(KeysAndMouse / Keys, "image_view.sc_pan_up", "Pan up",
            "Move the view up, for as long as the key is held.",
            ["pan", "scroll"], Live, ImageView, key!(image_view.sc_pan_up)),
        row!(KeysAndMouse / Keys, "image_view.sc_pan_down", "Pan down",
            "Move the view down.",
            ["pan", "scroll"], Live, ImageView, key!(image_view.sc_pan_down)),
        row!(KeysAndMouse / Keys, "image_view.sc_pan_left", "Pan left",
            "Move the view left.",
            ["pan", "scroll"], Live, ImageView, key!(image_view.sc_pan_left)),
        row!(KeysAndMouse / Keys, "image_view.sc_pan_right", "Pan right",
            "Move the view right.",
            ["pan", "scroll"], Live, ImageView, key!(image_view.sc_pan_right)),
        row!(KeysAndMouse / Keys, "image_view.sc_frame", "White frame",
            "Show or hide the white border around the photograph.",
            ["border", "frame", "matte"], Live, ImageView, key!(image_view.sc_frame)),
        row!(KeysAndMouse / Keys, "image_view.sc_more_images_shown", "More side by side",
            "Show one more picture beside the current one.",
            ["panes", "compare"], Live, ImageView, key!(image_view.sc_more_images_shown)),
        row!(KeysAndMouse / Keys, "image_view.sc_less_images_shown", "Fewer side by side",
            "Show one fewer.",
            ["panes", "compare"], Live, ImageView, key!(image_view.sc_less_images_shown)),
        row!(KeysAndMouse / Keys, "image_view.sc_marks", "Mark clipping and focus",
            "Mark what has clipped, then what is in focus, then nothing.",
            ["clipping", "blown", "peaking", "focus", "overlay"], Live, ImageView, key!(image_view.sc_marks)),
        row!(KeysAndMouse / Keys, "image_view.sc_overlay", "What it says about itself",
            "Move the photograph's own details round its corners, and off again.",
            ["overlay", "exif on image", "info"], Live, ImageView, key!(image_view.sc_overlay)),
        row!(KeysAndMouse / Keys, "image_view.sc_zoom_to_area", "Zoom to the marked area",
            "Magnify until the part of the photograph that is marked out fills the panel.",
            ["crop", "marquee", "selection", "marked", "zoom to selection"],
            Live, ImageView, key!(image_view.sc_zoom_to_area)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::image_view::opening::Opening;

    /// The window offers exactly the three the program has, in the words the
    /// enum uses for them.
    #[test]
    fn the_openings_offered_are_the_openings_there_are() {
        assert_eq!(OPENINGS.len(), Opening::ALL.len());

        for (choice, opening) in OPENINGS.iter().zip(Opening::ALL) {
            assert_eq!(choice.value, opening.value());
            assert_eq!(choice.label, opening.label());
            assert_eq!(choice.sentence, opening.description());
        }
    }
}
