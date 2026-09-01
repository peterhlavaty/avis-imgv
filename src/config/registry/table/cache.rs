//! `cache.*`: what the viewer holds, and how hard it works to fill it.

use super::*;

pub fn rows() -> Vec<Row> {
    vec![
        row!(
            SpeedAndMemory / Memory,
            "cache.ram_budget_mb",
            "RAM for decoded photographs",
            "The ceiling on decoded pixels held in RAM, across both views. A screen \
             sized copy of a 24 megapixel photograph is about 33 MB, so four thousand \
             megabytes is a hundred of them. When it is full the photograph furthest \
             from the cursor is dropped.",
            ["memory", "ram", "resources", "cache size", "budget"],
            Rebuild,
            None,
            whole!(usize, 256, 65536, " MB", true, cache.ram_budget_mb),
        ),
        row!(
            SpeedAndMemory / Work,
            "cache.decode_threads",
            "Decode threads",
            "How many photographs are decoded at once. Zero picks one per core less \
             one, kept for drawing. Decoding is not compute bound past a handful: on a \
             24 core machine eight threads sustained 42 images a second and twelve \
             sustained 39, and each thread holding a whole decoded image costs another \
             130 MB.",
            ["threads", "cores", "cpu", "workers", "slow"],
            Restart,
            None,
            whole!(usize, 0, 64, " threads", true, cache.decode_threads),
        ),
        row!(
            SpeedAndMemory / Graphics,
            "cache.previews_resident",
            "Camera thumbnails kept ready",
            "How many of the camera's own embedded previews stay on the graphics card, \
             so a photograph still being decoded shows something rather than a spinner. \
             Zero turns that off.",
            ["preview", "spinner", "placeholder", "embedded"],
            Rebuild,
            None,
            whole!(usize, 0, 2048, "", true, cache.previews_resident),
        ),
        row!(
            SpeedAndMemory / Memory,
            "cache.full_resolution_neighbours",
            "Neighbours kept at full resolution",
            "Browsing keeps a copy no larger than the screen, because a monitor can \
             show three megapixels and the file has twenty-four. This is how many of \
             the photographs either side also keep their own pixels, so magnifying \
             them costs nothing. Each one is the whole file in memory.",
            ["zoom", "magnify", "full size", "1:1", "sharp"],
            Rebuild,
            None,
            whole!(usize, 0, 32, "", true, cache.full_resolution_neighbours),
        ),
        row!(
            SpeedAndMemory / Graphics,
            "cache.gpu_budget_mb",
            "Graphics card memory",
            "The ceiling on what the two caches may hold on the adapter. A texture is \
             the decoded pixels again plus a third for the mip chain. This is a memory \
             bound and the counts beside it are not: two hundred thumbnails and two \
             hundred 60 megapixel photographs are the same number and a thousandfold \
             difference in what the card is holding.",
            ["gpu", "graphics", "vram", "card", "adapter", "video memory"],
            Rebuild,
            None,
            whole!(usize, 128, 32768, " MB", true, cache.gpu_budget_mb),
        ),
        row!(
            SpeedAndMemory / Work,
            "cache.upload_budget_ms",
            "Time per frame spent uploading",
            "How long one frame may spend moving decoded photographs onto the graphics \
             card. A 24 megapixel texture takes about 12 ms, so this is the difference \
             between a smooth frame rate and a stuttering one while the cache fills. \
             The default is computed from the frame time actually being measured; a \
             written value wins.",
            ["stutter", "jitter", "frame time", "upload"],
            Rebuild,
            None,
            whole!(u64, 1, 100, " ms", false, cache.upload_budget_ms),
            explained: "No control: it is computed from the frame time the viewer is \n                        already measuring every frame. A value written by hand still \n                        wins, for whoever is chasing a stutter.",
        ),
    ]
}
