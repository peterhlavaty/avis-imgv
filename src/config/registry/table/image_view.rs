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
            whole!(u32, 0, 32768, " px", false, image_view.max_image_edge),
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
            decimal!(0.0, 0.5, "", true, image_view.frame_size_relative_to_image),
        ),
        row!(
            KeysAndMouse / Mouse,
            "image_view.scroll_navigation",
            "The wheel moves through the folder",
            "A boolean that cannot answer \"what should the wheel do instead\", which \
             is the question people actually have. It becomes one row of the mouse \
             settings.",
            ["wheel", "scroll", "mouse", "zoom with wheel"],
            Live,
            None,
            boolean!(image_view.scroll_navigation),
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
        row!(KeysAndMouse / Keys, "image_view.sc_latch_fit_maximize", "Keep filling",
            "Carry on filling the window as you move through the folder.",
            ["latch", "fill"], Live, ImageView, key!(image_view.sc_latch_fit_maximize)),
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
            "Pin this picture and the next side by side, sharing one zoom and one pan. Tab moves which one the keys are about, / drops it, Escape leaves.",
            ["compare", "side by side"], Live, ImageView, key!(image_view.sc_compare)),
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
    ]
}
