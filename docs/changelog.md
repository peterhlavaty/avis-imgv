# Change Log

## 2026-08-27

- A fifth mode, **Group shots**, for the runs of frames that belong together:
  brackets, focus stacks, timelapses and bursts.
  - Frames group when they were taken close enough together and show the same
    thing. What they look like comes from a difference hash of the camera's own
    thumbnail, so a frame two stops darker still matches the one before it and
    a different scene taken a second later does not.
  - What kind of run it is follows from what changed across it: the exposure
    for a bracket, the focus distance for a stack, a steady interval and enough
    frames for a timelapse, and a series for everything else.
  - Every reading is a proposal. The kind is a dropdown, a run can be told it
    is not a group at all, single frames can be taken out, and loose frames can
    be put into any group — landing in the order they were taken.
  - Confirming tidies each group into `hdr1`, `stack1`, `timelapse1`,
    `series1` and so on, sidecars included. A number already on disk is stepped
    over rather than merged into.
  - The folder sweep now also summarises each thumbnail, which costs well under
    a millisecond a file and is what makes the similarity test affordable.

## 2026-08-27

- Two new modes that work on the folder rather than on one picture. The menu
  lists them under Mode, and `F2` cycles through all four.
  - **Bulk rename.** A name is a template — literal text with `{name}`,
    `{counter}`, `{date}`, `{tag:ISO}` and the rest — and what every file would
    become is shown before anything is written. The counter's start, step and
    width are all settable, and it follows the order the folder is sorted in.
    A rename that shifts a numbered sequence onto itself works, and so does
    swapping two names: every file goes through a temporary name first.
    Sidecars follow the pictures they belong to.
  - **Shift capture time**, for a camera whose clock was wrong. An offset in
    days, hours, minutes and seconds, forwards or backwards, applied to
    whichever timestamps the files carry and you tick. The dates are rewritten
    in place — an EXIF timestamp is a fixed nineteen characters whatever it
    says — so nothing else in the file moves and the maker notes, the preview
    and the pixels are untouched. Each file is written to a temporary copy and
    renamed over the original.
  - Both modes sort and filter the folder first: by name, capture time, type,
    size, rating or any metadata tag, and filtered on the same things plus
    keywords and stars. Names sort the way a person reads them, so `IMG_9`
    comes before `IMG_10`.
  - Opening either reads the folder in the background — the front of every file
    and its sidecar — and the list is usable from the first frame, filling in
    as it goes.
- New: `general.sc_next_mode`, `F2` by default.

## 2026-08-27

- Memory now holds a whole folder rather than a window of it, and zooming shows
  the photograph rather than a magnified copy of it.
  - What is kept for every image is the screen sized copy: eleven megabytes for
    a 24 megapixel photograph on a 1080p monitor instead of ninety-six, so a
    few hundred of them are resident at once instead of thirty.
  - The image on screen and the two either side are also decoded at full
    resolution and held ready. Which copy is on the GPU follows how wide the
    image is being drawn and swaps back when you zoom out, so a hundred
    megabyte texture is never held for something being shown small.
  - Zoom and pan belong to the image. Leaving a photograph half way into a
    corner and coming back to it finds it exactly there; images that were
    zoomed keep their full sized copy until something nearer needs the room.
  - `+` and `-` zoom, and `W` `A` `S` `D` pan for as long as they are held.
    Showing more or fewer images side by side moved to `Ctrl` `+` and `Ctrl`
    `-`.
  - Every texture now carries a mip chain, built on the GPU in one pass per
    level. A photograph is nearly always drawn smaller than it is stored, and a
    bilinear sampler reads four texels of every nine it should — which is what
    made fine detail sparkle and crawl as an image was panned.
  - Fixed a cache that had been thrashing since it was written: the upload
    window was as wide as the GPU cache could hold, so every upload evicted a
    texture the same window still wanted, and the next frame uploaded it again.
  - Sustained browsing of 24 megapixel JPEGs went from 36 images a second
    to 43.6.
- BREAKING: `+` and `-` in the image view now zoom. `Ctrl` `+` and `Ctrl` `-`
  show more or fewer images side by side.

## 2026-08-27

- An image now appears as soon as the file is opened rather than when it has
  been decoded, and the viewer's own cost per frame fell from 23.6ms to 0.9ms.
  - A preview tier: one thread reads the first 512 KB of each file near the
    cursor, which gives the metadata for the side panel in about two
    milliseconds and the thumbnail the camera embedded. The thumbnail is shown
    at the size of the image it stands for, so nothing moves when the real
    decode arrives. `cache.previews_resident` sizes it; `0` turns it off.
  - The side panel fills in from that read, so it is no longer blank while a
    photograph decodes. Ratings and tags still wait for the whole file, since a
    sidecar seeded from a truncated read would drop what it could not see.
  - What goes to the GPU is now a copy no larger than the monitor, made on the
    decode worker rather than the UI thread. A 24 megapixel texture is 96 MB
    and takes fifteen milliseconds to upload; the screen sized copy takes two,
    and nothing on screen can tell the difference. The full resolution stays in
    RAM and is uploaded as soon as you zoom in past what the copy holds.
  - `decode_threads` now defaults to eight rather than to a core count. Past
    that the decoders saturate memory bandwidth instead of adding throughput:
    measured on 24 cores, eight workers sustained 42 images a second and twelve
    sustained 39.
  - Sustained browsing of 24 megapixel JPEGs went from 33 to 36 images a
    second, and the frame times behind it from 23.63ms to 0.87ms.
  - Measured and rejected: a DCT scaled JPEG decode. `jpeg-decoder` at half
    size costs 102ms against zune-jpeg's 90ms at full size, so the saving is
    not there in pure Rust. libjpeg-turbo through FFI is the only route left
    and would be a build dependency for perhaps a factor of two.

## 2026-08-27

- Browsing a folder larger than the cache went from 3.2 images a second to 33,
  and the pipeline for a 24 megapixel JPEG from 258ms to 145ms.
  - Fixed the cache: a worker that abandoned a request the viewer had
    navigated away from reported nothing back, so the image stayed marked as
    loading for good and was never asked for again. This was most of it.
  - The preload window now leaves the RAM budget a quarter spare, rather than
    sitting exactly at the ceiling and evicting images it was about to want.
  - The camera's orientation is applied by the GPU, by sampling the texture in
    a different order, instead of copying ninety megabytes on the CPU. Saves
    86ms on every photograph taken sideways.
  - JPEGs decode straight to RGBA instead of decoding to RGB and then widening
    in a second pass over every pixel. Saves 23ms.
  - Uploads have a time budget per frame rather than a fixed count, because a
    24 megapixel texture takes 12ms and four of them made a fifty millisecond
    frame.
  - `--benchmark` measures all of this, and `cargo run --example bench_decode`
    breaks the pipeline down stage by stage.
- BREAKING: `cache.uploads_per_frame` is now `cache.upload_budget_ms`.

- Raw files can now be developed rather than only previewed. `raw.source` picks
  between the JPEG the camera embedded, which is free and low resolution, and
  the sensor data demosaiced with LibRaw, which is full resolution and costs
  about a second an image. The bindings are hand written against LibRaw's C
  API; the feature is `libraw` and it is off by default because it links
  against a system library.

- Added star ratings and tagging. `K` opens a resizable panel with the stars
  for the open image, the tags on it, the tags used most recently, and a
  configurable catalog organised into categories and searchable by either. The
  digit keys rate the open image with or without the panel open.
  - Both live in XMP sidecars beside the image, as `xmp:Rating` and
    `dc:subject`. An existing sidecar is edited rather than replaced, so a
    develop history written by another tool survives.
  - Ratings and keywords already inside the image are read too, from a JPEG
    APP1 segment, a PNG iTXt chunk, a WebP chunk, a TIFF tag, or Windows
    Explorer's EXIF rating.

- The viewer now decodes a whole folder into RAM on a pool of background
  workers and keeps a configurable number of images resident on the GPU, so
  navigation is a texture swap rather than a load. See `cache` in the
  configuration.
- BREAKING: metadata is read in process instead of through `exiftool`, which is
  no longer a dependency. EXIF, ICC profiles and raw previews are parsed from
  the same buffer the file was read into. Tag names still follow exiftool's.
- BREAKING: the SQLite metadata database and the exif filter panel are gone,
  along with the `--import` and `--clean` arguments. The side panel now shows
  metadata and cache occupancy.
- BREAKING: configuration changes. `general.limit_cached` and
  `grid_view.simultaneous_load` were removed; `cache`,
  `image_view.gpu_resident_images`, `image_view.max_image_edge`,
  `grid_view.thumbnail_resolution` and `grid_view.gpu_resident_thumbnails` were
  added. Let the application recreate the file and move your settings over.
- Canon CR3 previews and metadata are now supported.
- JPEG XL moved behind the optional `jxl` feature, so a default build no longer
  needs cmake and a C++ toolchain.
- The dark theme is now applied regardless of the desktop's preference.

## 2025-07-06

- BREAKING: Improved database operations and storage footprint by using JSONB column type
    - You will need to delete your old database. A new cli argument has been added to recursively scan a directory
      `avis-imgv --import <path>`
- Added a new panel to replace the old metadata one. This new panel allows you to filter and sort by exif tags and also
  displays the image metadata.
- BREAKING: the configuration has breaking changes, best to let the application re-create it and then move over your
  settings.

## 2024-09-03

- Configuration file now uses json. This allows us to drop our dependency on yaml serialziation.
- Upgraded most crates to their recent version.
- QOL improvements and small bugfixes.
- Added a "latch maximize" function(ctrl + m) that automatically extends images to their maximum size for the screen.
- Added a maximize shortcut (m).
- Improved shortcuts code wise, this will allow easier configurations in the future.

## 2023-11-25

- Added the ability to watch the opened directory for new files or file changes. Can work together with directory
  flattening to watch all sub directories recursively. Shortcut is `sc_watch_directory` under general.
- Added sort by file modification date.
- Upgraded egui to latest version.
- Upgraded zune jpeg to latest version.

## 2023-09-32

- Added the ability to flatten the open directory, reading all files from subdirectories. Shortcut `sc_flatten_dir`
  under general.
- Allow to sort by random.

## 2023-04-26

### Added

- Right click menu for image magnification. Shortcut to set magnification as one to one (100%). Shortcut `sc_one_to_one`
  under `gallery`.
- Right click menu on magnification now also has "fit to screen", "fit horizontal" and "fit vertical". Shortcut
  `sc_fit_horizontal` and `sc_fit_vertical` under `gallery`.

## 2023-03-29

All config keys in the root of the file will need to be put under a "general" config. Please check the example
configuration.
"sc_del" and "delete_cmd" configs were removed as it's prefered to do it using a delete command plus a callback.

### Added

- User actions and Context Menus now can have callbacks. Currently 3 were implemented: Pop, Reload and ReloadAll.

### Changed

- Various bugfixes and adjustments.
- qcms now pulls directly from its repo as the crate is outdated and requires rust bootstrap.

---

## 2023-03-19

Two new configuration entries were added, "sc_dir_tree".

### Added

- Added a directory tree pannel to quickly browser through directories.

### Changed

### Fixed

---

## 2023-03-18

Two new configuration entries were added, "sc_del" and "delete_cmd" both under gallery.

### Added

- Added a shortcut to delete/move files. It executes the configured command, removes the image from the current list and
  loads the next image.

### Changed

- Implemented the fast_image_resize crate and changed the resizing algorithm to Bilinear. This greatly improves multi
  gallery performance.

### Fixed
