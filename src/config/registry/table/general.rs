//! `general.*`: the colour pipeline, the interface itself, and the keys that
//! are read in every mode.

use super::*;

pub fn rows() -> Vec<Row> {
    let mut rows = vec![
        row!(
            TheWindow / Appearance,
            "general.output_icc_profile",
            "Screen profile",
            "The colour profile every photograph is converted into before it is drawn. \
             Matched by substring against the profiles the viewer ships, so \"srgb\" \
             and \"sRGB\" both find sRGB. A name that matches nothing leaves the \
             photograph unconverted, which is what makes a wide-gamut screen show \
             oversaturated colour.",
            [
                "colour",
                "color",
                "colour management",
                "color management",
                "icc",
                "profile",
                "srgb",
                "adobe rgb",
                "saturated"
            ],
            Rebuild,
            None,
            text!(general.output_icc_profile),
        ),
        row!(
            TheWindow / Appearance,
            "general.text_scaling",
            "Text size",
            "How large every word in the interface is drawn, against the size the \
             theme asked for. Everything scales together: the menus, the panels, the \
             status bar and the windows.",
            [
                "font",
                "text too small",
                "dpi",
                "hidpi",
                "scaling",
                "readable"
            ],
            Live,
            None,
            // Read as a percentage rather than a multiplier: 125 per cent is what
            // a person means, and 1.25 is what the file holds. The file keeps
            // its own notation, because that is what a forum answer quotes.
            Access::Float {
                get: |c| c.general.text_scaling * 100.0,
                set: |c, v| c.general.text_scaling = v / 100.0,
                min: 50.0,
                max: 300.0,
                unit: " %",
                rail: true,
            },
        ),
        row!(
            ThePhotograph / Panels,
            "general.metadata_tags",
            "What the side panel shows",
            "Which metadata tags are listed beside the photograph, in this order. A \
             tag the file does not carry is skipped rather than drawn empty.",
            ["exif", "metadata", "panel", "tags", "camera", "lens"],
            Live,
            None,
            Access::Records(List::MetadataTags, |c| c.general.metadata_tags.len()),
        ),
        row!(
            OpeningAFolder / Starting,
            "general.restore_session",
            "Open where the last run left off",
            "The window's size and place, the folder that was open, and — the one that \
             earns its keep — which photograph was being looked at in each folder \
             visited lately. A cull is rarely one sitting. A path on the command line \
             always wins over this.",
            ["session", "remember", "resume", "last folder", "reopen"],
            NextLaunch,
            None,
            boolean!(general.restore_session),
        ),
    ];

    rows.extend(keys());
    rows
}

/// The keys read in every mode.
///
/// Every one of them is `Scope::Everywhere`, which is the point: `input::collect`
/// runs unconditionally every frame, so one of these on the same key as an
/// image-view binding is the collision that actually bites.
fn keys() -> Vec<Row> {
    vec![
        row!(KeysAndMouse / Keys, "general.sc_next_mode", "Next mode",
            "Move round the modes: image, gallery, bulk rename, shift capture time, group shots, slideshow.",
            ["mode", "cycle"], Live, Everywhere, key!(general.sc_next_mode)),
        row!(KeysAndMouse / Keys, "general.sc_toggle_gallery", "Gallery",
            "Switch between the image and the contact sheet.",
            ["grid", "thumbnails", "contact sheet"], Live, Everywhere, key!(general.sc_toggle_gallery)),
        row!(KeysAndMouse / Keys, "general.sc_menu", "Menu",
            "Show or hide the menu bar.",
            ["menu bar", "toolbar"], Live, Everywhere, key!(general.sc_menu)),
        row!(KeysAndMouse / Keys, "general.sc_filmstrip", "Filmstrip",
            "Show or hide the strip of thumbnails under the photograph.",
            ["strip", "thumbnails"], Live, Everywhere, key!(general.sc_filmstrip)),
        row!(KeysAndMouse / Keys, "general.sc_stacks", "Stacks",
            "Show the folder stacked: every burst, bracket and timelapse as one cell.",
            ["burst", "group", "bracket", "timelapse"], Live, Everywhere, key!(general.sc_stacks)),
        row!(KeysAndMouse / Keys, "general.sc_toggle_stack", "Open or close a stack",
            "Show what is inside the run of frames the cursor is on, or fold it back up.",
            ["expand", "burst"], Live, Everywhere, key!(general.sc_toggle_stack)),
        row!(KeysAndMouse / Keys, "general.sc_standing_back", "Which frame shows the stack",
            "Walk the frames of a closed stack without opening it. Nobody searches for \"standing\".",
            ["cover", "key frame", "representative"], Live, Everywhere, key!(general.sc_standing_back)),
        row!(KeysAndMouse / Keys, "general.sc_standing_forward", "Show the next frame instead",
            "The same, forwards.",
            ["cover", "key frame"], Live, Everywhere, key!(general.sc_standing_forward)),
        row!(KeysAndMouse / Keys, "general.sc_previous_stack", "Previous stack",
            "Step to the run of frames before this one, over a burst rather than through it.",
            ["burst", "skip"], Live, Everywhere, key!(general.sc_previous_stack)),
        row!(KeysAndMouse / Keys, "general.sc_next_stack", "Next stack",
            "Step to the run of frames after this one.",
            ["burst", "skip"], Live, Everywhere, key!(general.sc_next_stack)),
        row!(KeysAndMouse / Keys, "general.sc_turn_left", "Turn anticlockwise",
            "Turns the photograph a quarter. The turn is written to the sidecar beside the rating; the photograph itself is never touched, which is what keeps a raw file a raw file.",
            ["rotate", "turn", "sideways", "orientation"], Live, Everywhere, key!(general.sc_turn_left)),
        row!(KeysAndMouse / Keys, "general.sc_turn_right", "Turn clockwise",
            "The same, the other way.",
            ["rotate", "turn", "sideways", "orientation"], Live, Everywhere, key!(general.sc_turn_right)),
        row!(KeysAndMouse / Keys, "general.sc_toggle_side_panel", "Side panel",
            "Show or hide the metadata and cache readout down the side.",
            ["exif", "panel", "metadata"], Live, Everywhere, key!(general.sc_toggle_side_panel)),
        row!(KeysAndMouse / Keys, "general.sc_navigator", "Navigation bar",
            "Type a path to open instead of picking one.",
            ["go to", "path", "open"], Live, Everywhere, key!(general.sc_navigator)),
        row!(KeysAndMouse / Keys, "general.sc_dir_tree", "Directory tree",
            "Open the folder tree beside the image.",
            ["folders", "browse"], Live, Everywhere, key!(general.sc_dir_tree)),
        row!(KeysAndMouse / Keys, "general.sc_flatten_dir", "Flatten folders",
            "Read the pictures out of every sub folder as though they were one.",
            ["recursive", "subfolders"], Live, Everywhere, key!(general.sc_flatten_dir)),
        row!(KeysAndMouse / Keys, "general.sc_watch_directory", "Watch the folder",
            "Pick up pictures that appear or change while the viewer is open.",
            ["tether", "tethering", "live", "monitor"], Live, Everywhere, key!(general.sc_watch_directory)),
        row!(KeysAndMouse / Keys, "general.sc_delete", "To the bin",
            "Send the picture on screen to the platform's bin, along with its sidecar.",
            ["delete", "trash", "remove"], Live, Everywhere, key!(general.sc_delete)),
        row!(KeysAndMouse / Keys, "general.sc_delete_permanently", "Delete for good",
            "Delete it outright, for the cards and shares that have no bin. Asked about first.",
            ["delete", "permanent", "shift delete"], Live, Everywhere, key!(general.sc_delete_permanently)),
        row!(KeysAndMouse / Keys, "general.sc_filter", "Filter",
            "Show or hide the bar that narrows and orders the folder.",
            ["narrow", "sort", "search"], Live, Everywhere, key!(general.sc_filter)),
        row!(KeysAndMouse / Keys, "general.sc_suspend_filter", "Show everything",
            "Set the filter aside without forgetting it, so what it is hiding can be looked at.",
            ["unfilter", "show all"], Live, Everywhere, key!(general.sc_suspend_filter)),
        row!(KeysAndMouse / Keys, "general.sc_settings", "Settings",
            "Opens the settings window on the page it was last left on.",
            ["settings", "preferences", "options", "configure"], Live, Everywhere, key!(general.sc_settings)),
        row!(KeysAndMouse / Keys, "general.sc_fullscreen", "Fullscreen",
            "Fill the screen, and give it back.",
            ["full screen", "maximise"], Live, Everywhere, key!(general.sc_fullscreen)),
        row!(KeysAndMouse / Keys, "general.sc_exit", "Quit",
            "Close the viewer.",
            ["exit", "close"], Live, Everywhere, key!(general.sc_exit)),
    ]
}
