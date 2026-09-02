//! `grid_view.*`: the contact sheet and the strip under the photograph.

use super::*;

pub fn rows() -> Vec<Row> {
    let mut rows = vec![
        row!(
            TheContactSheet / Cells,
            "grid_view.images_per_row",
            "Thumbnails across",
            "How many cells fit in a row, which is what decides how large they are. \
             The keys change it for the session; this is what a launch starts with.",
            ["columns", "size", "grid", "zoom", "bigger thumbnails"],
            Live,
            None,
            whole!(usize, 1, 16, "", true, grid_view.images_per_row),
        ),
        row!(
            TheContactSheet / Cells,
            "grid_view.cell_aspect",
            "The shape of the cells",
            "How wide a cell's picture is against its height, read as 1 : this. 1.50 is \
             the three to two most cameras shoot; a square sheet of landscape \
             photographs draws about forty-four per cent of itself in grey.",
            ["aspect", "shape", "square", "ratio", "letterbox"],
            Live,
            None,
            decimal!(0.5, 3.0, "", true, grid_view.cell_aspect),
        ),
        row!(
            SpeedAndMemory / Memory,
            "grid_view.preloaded_rows",
            "Rows of thumbnails kept ready",
            "How many rows above and below what is on screen are decoded ahead, so \
             scrolling the sheet does not run into grey.",
            ["preload", "scroll", "ahead", "thumbnails"],
            Rebuild,
            None,
            whole!(usize, 0, 64, " rows", true, grid_view.preloaded_rows),
        ),
        row!(
            SpeedAndMemory / Memory,
            "grid_view.thumbnail_resolution",
            "Thumbnails decoded to",
            "The longest edge a thumbnail is decoded to. Below the width a cell is \
             actually drawn at, thumbnails look soft — which is the commonest complaint \
             about every viewer that gets this wrong.",
            [
                "thumbnail",
                "blurry thumbnails",
                "soft",
                "resolution",
                "quality"
            ],
            Rebuild,
            None,
            whole!(u32, 64, 4096, " px", true, grid_view.thumbnail_resolution),
        ),
        row!(
            SpeedAndMemory / Graphics,
            "grid_view.gpu_resident_thumbnails",
            "Thumbnails kept on the graphics card",
            "A count where the real bound is bytes, which is what the graphics budget \
             is for. It exists because a sheet scrolled quickly wants many small \
             textures rather than a few large ones.",
            ["gpu", "vram", "textures", "thumbnails"],
            Rebuild,
            None,
            whole!(usize, 1, 4096, "", false, grid_view.gpu_resident_thumbnails),
            explained: "No control, for the same reason as the count above it: the \n                        graphics card budget bounds both in bytes.",
        ),
        row!(
            KeysAndMouse / Menus,
            "grid_view.context_menu",
            "Your own menu entries on a cell",
            "The same idea as the one on the photograph, written twice. The window \
             draws them as one table with a column saying where each entry appears.",
            ["right click", "context menu", "menu", "cell"],
            Live,
            None,
            Access::Records(List::ContextMenu, |c| c.grid_view.context_menu.len()),
        ),
        row!(
            TheContactSheet / Filmstrip,
            "grid_view.filmstrip_height",
            "How tall the strip is",
            "The band of thumbnails under the photograph. The thumbnails are as \
             large as the strip allows, so this is how big they are as much as it is \
             how tall it is. Dragging the strip's top edge writes it here.",
            ["strip", "filmstrip", "thumbnails", "height", "size"],
            Live,
            None,
            decimal!(0.0, 400.0, " pt", true, grid_view.filmstrip_height),
        ),
        row!(
            TheContactSheet / Cells,
            "grid_view.caption_format",
            "What is written under a cell",
            "In the placeholder grammar. Drawn only while the strip under a cell is \
             showing the file name, which the key cycles.",
            ["caption", "label", "name", "template", "under"],
            Live,
            None,
            template!(grid_view.caption_format),
        ),
    ];

    rows.extend(keys());
    rows
}

fn keys() -> Vec<Row> {
    vec![
        row!(KeysAndMouse / Keys, "grid_view.sc_scroll", "Scroll down",
            "Move half a row down the contact sheet.",
            ["scroll", "page"], Live, Gallery, key!(grid_view.sc_scroll)),
        row!(KeysAndMouse / Keys, "grid_view.sc_more_per_row", "More per row",
            "Fit one more thumbnail across, making them smaller.",
            ["columns", "smaller", "zoom out"], Live, Gallery, key!(grid_view.sc_more_per_row)),
        row!(KeysAndMouse / Keys, "grid_view.sc_less_per_row", "Fewer per row",
            "Fit one fewer, making them larger.",
            ["columns", "bigger", "zoom in"], Live, Gallery, key!(grid_view.sc_less_per_row)),
        row!(KeysAndMouse / Keys, "grid_view.sc_cycle_badges", "What the cells say",
            "Cycle what is drawn under each thumbnail: nothing, the marks, or the marks and the file name.",
            ["badges", "caption", "labels", "under"], Live, Gallery, key!(grid_view.sc_cycle_badges)),
        row!(KeysAndMouse / Keys, "grid_view.sc_select", "Pick out",
            "Pick the photograph under the cursor out, or put it back. Everything picked out is what the next mark, move or deletion applies to.",
            ["select", "multi", "selection"], Live, Gallery, key!(grid_view.sc_select)),
        row!(KeysAndMouse / Keys, "grid_view.sc_select_all", "Pick out everything",
            "Pick out every photograph on show, or put them all back if they already are.",
            ["select all", "ctrl a"], Live, Gallery, key!(grid_view.sc_select_all)),
    ]
}
