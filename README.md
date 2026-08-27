![Build](https://github.com/hats-np/avis-imgv/actions/workflows/rust.yml/badge.svg)

---

# avis-imgv

A GPU accelerated image viewer for people with more RAM than patience.

Open a folder and avis-imgv starts decoding **all** of it on background threads,
keeps as much as your budget allows resident in RAM as ready-to-upload pixels,
and holds the images around the one you are looking at on the **GPU** as live
textures. Moving to the next photograph is then a texture swap: no file read, no
decode, no upload. Zoom and pan cost nothing beyond four numbers handed to the
GPU, because they are a UV rectangle rather than a resample.

Built with Rust and [egui](https://github.com/emilk/egui)/wgpu.

[Changelog](docs/changelog.md)

## How it works

Four tiers, from cheap to instant:

| Tier | Holds | Bounded by |
|------|-------|------------|
| Disk | every path the crawler found | the folder |
| Preview | metadata and the camera's thumbnail, read from the first 512 KB | `cache.previews_resident` |
| RAM | decoded, oriented, colour converted RGBA8 | `cache.ram_budget_mb` |
| GPU | textures ready to draw | `gpu_resident_images` / `gpu_resident_thumbnails` |

The preview tier is what puts something on screen immediately. A dedicated
thread reads the front of each file near the cursor, which takes a couple of
milliseconds and yields both the metadata for the side panel and the thumbnail
the camera stored. The thumbnail is uploaded reporting the size of the image it
stands for, so the layout is already right and nothing moves when the real
decode lands.

What goes to the GPU is a copy no larger than the screen, made on the decode
worker. A 24 megapixel photograph is 96 MB of RGBA8 and a 1080p monitor can
show three of those megapixels: uploading all of it costs fifteen milliseconds
of the UI thread to no visible effect. The full resolution stays in RAM and is
uploaded the moment you zoom in far enough to tell the difference.

A pool of decode workers pulls from a **priority queue** ordered by distance
from the image on screen, so the one you are about to reach is always decoded
first — and requests you have navigated away from are dropped before they cost
anything. Both caches evict the image furthest from the cursor, wrapping around
the ends of the collection.

The preload radius is trimmed automatically to what the RAM budget can hold. A
folder of 60 megapixel raws will not be decoded only to be evicted; the viewer
simply keeps a smaller window resident.

Nothing in the draw path waits on I/O or on a decoder.

### Metadata

EXIF, ICC profiles, XMP packets and raw previews are read **in process**, from
the same buffer the file was read into for decoding — no `exiftool`, no
subprocess, no second read of the file. Reading the metadata of a JPEG takes
tens of microseconds.

Supported containers: JPEG (APP1/APP2), PNG (`eXIf`, `iCCP`), WebP (RIFF `EXIF`,
`ICCP`), TIFF, TIFF derived raws (DNG, NEF, CR2, ARW, ORF, RW2, PEF, …), Fuji
RAF, and Canon CR3 (ISO base media). Tag names follow exiftool's, so existing
`metadata_tags` and `name_format` settings keep working.

## Ratings and tags

Press `K` for the rating and tagging panel: a resizable, hideable side panel
holding the stars for the open image, the tags on it, the tags you used most
recently, and the catalog you configured — searchable by tag name or by the
name of the category a tag is filed under.

- **Stars.** `0` to `5` set the rating from the keyboard, with or without the
  panel open; the status bar shows it either way. Clicking the star a rating
  already ends on clears it.
- **Recent tags.** Whatever you applied last, one click away, remembered
  between sessions.
- **Your catalog.** `tags.categories` in the configuration is the list you
  always want to hand. Searching matches a category name too, so typing
  "places" offers everything filed under Places.
- **Anything already used.** Tags found on the other images of the folder are
  offered as well, so a tag typed once does not have to be configured.
- Typing something new offers to create it.

### Where they are stored

In XMP sidecars beside the image — `DSC001.cr2.xmp` — as `xmp:Rating` and
`dc:subject`, which is what every raw converter reads. Adobe's `DSC001.xmp` is
read too, and whichever sidecar is already there is the one edited: a develop
history written by another tool survives untouched.

Ratings and keywords **inside** the image are read as well (JPEG `APP1`, PNG
`iTXt`, WebP, TIFF, and Windows Explorer's EXIF rating), so an image rated
elsewhere shows up already rated. Nothing is ever written back into the image
file itself.

Saves happen on a thread of their own, so rating a photograph never waits on
the disk.

## Dependencies

- A C toolchain for `lcms2` (colour management)
- [LibRaw](https://www.libraw.org/) for the `libraw` feature: `libraw-dev` on
  Debian and Ubuntu, `libraw` on Arch and Homebrew, `vcpkg install libraw` on
  Windows
- `cmake` and a C++ toolchain for the `jxl` feature
- coreutils, for `install.sh`

No runtime dependency on exiftool.

## Build

With Rust [installed](https://rustup.rs/):

```sh
cargo build --release
```

Developing raw files rather than showing their embedded preview links against
LibRaw, so it is a feature you turn on:

```sh
cargo build --release --features libraw
```

build.rs looks for LibRaw with pkg-config, then vcpkg, then `LIBRAW_LIB_DIR` if
you point it somewhere yourself. On Windows that means setting `VCPKG_ROOT`:

```
set VCPKG_ROOT=C:\path\to\vcpkg
set VCPKGRS_TRIPLET=x64-windows-static-md
cargo build --release --features libraw
```

JPEG XL support builds libjxl from source and is therefore off by default too:

```sh
cargo build --release --features jxl
```

It needs `cmake` and a C++ compiler. On Windows `jpegxl-src` asks MSBuild for
the `ClangCL` platform toolset specifically, which is a Visual Studio component
rather than part of the default C++ workload; without it the build stops at
`error MSB8020`. Add it from an elevated prompt:

```
"C:\Program Files (x86)\Microsoft Visual Studio\Installer\setup.exe" modify ^
  --installPath "C:\Program Files (x86)\Microsoft Visual Studio2\BuildTools" ^
  --add Microsoft.VisualStudio.Component.VC.Llvm.ClangToolset --passive
```

## Install

`install.sh` builds and installs to `~/.local/bin` and creates a `.desktop`
entry. Linux only, and still rudimentary.

`cargo install avis-imgv` also works but does not integrate with your desktop.

## Usage

```
avis-imgv [OPTIONS] [PATH]

  PATH          An image to open, or a directory to open the first image of.
                Defaults to the working directory.

  --slideshow   Start in slideshow mode. Useful as a photo frame.
  --fullscreen  Start fullscreen.
  --benchmark   Walk the folder as fast as it will go, report how many images
                a second that was, and quit.
  --help        Show this message.
```

### How fast is it

`--benchmark` walks a folder one image per frame and reports what it managed,
which folds in decoding, waiting, uploading and drawing. On a folder of 120
24-megapixel JPEGs, larger than the cache, on a 24 core Ryzen with a 1080p
monitor:

```
Benchmark: 501 images in 13.96s — 35.9 images/s, median frame 0.87ms
```

The frame time is what the viewer costs you; the images a second is what the
decoders can supply. Past about eight workers the decoders stop scaling — a 24
megapixel image is a hundred megabytes of output and they saturate memory
bandwidth long before they run out of cores — which is why `decode_threads`
defaults to eight rather than to a core count.

The numbers are there to be checked on your own machine and your own files.
`cargo run --release --example bench_decode -- <files>` breaks a single image
down stage by stage.

## Supported image formats

JPEG, PNG, WebP, GIF, BMP and TIFF, through the
[image](https://github.com/image-rs/image) crate's default features. JPEG XL
through libjxl behind the `jxl` feature.

### Raw files

A raw file holds two pictures: the JPEG preview the camera embedded, and the
sensor data it was made from. `raw.source` decides which one you get.

**`"preview"`** (the default) extracts the embedded JPEG, which is what the
camera showed you on its own screen and costs almost nothing to decode.
Extraction is native: TIFF derived raws are read through their IFD chain, Fuji
RAF through its header, Canon CR3 through its box tree, and anything
unrecognised falls back to a scan for the largest embedded JPEG stream. The
catch is resolution — a CR3 preview is 1620x1080, which is all Canon stores.

**`"develop"`** demosaics the sensor data with [LibRaw](https://www.libraw.org/),
giving the full resolution and the full dynamic range. The same CR3 comes out at
6022x4024. It costs about a second per image, so it needs the `libraw` feature
and it is off by default.

The bindings are hand written against LibRaw's C API (`src/decoder/raw/ffi.rs`),
which keeps `libraw_data_t` opaque: its layout changes between releases while
the setters and getters do not.

Two things worth knowing about developing:

- Thumbnails always use the preview. Developing a whole folder to fill a
  contact sheet would take minutes.
- It is memory hungry. A 24 megapixel raw needs a few hundred megabytes while
  it is being developed, times however many `cache.decode_threads` are running.
  Lower that number if the machine starts swapping.
- If LibRaw cannot read a particular file, the viewer falls back to its preview
  rather than showing nothing.

## Colour management

Done with `lcms2`. The input profile is the one embedded in the file when there
is one; otherwise the closest of the three bundled profiles (sRGB, Adobe RGB,
Display P3) is matched by name against the `Profile Description` tag. The match
is deliberately lax, so `RT_sRGB` resolves to sRGB. Images already in the output
profile skip conversion entirely.

The output profile is sRGB by default and must be one of the bundled ones. To
add more, edit `src/metadata/icc.rs`, or open a PR.

sRGB and Adobe RGB (ClayRGB) come from
[elles_icc_profiles](https://github.com/ellelstone/elles_icc_profiles).

## Configuration

`~/.config/avis-imgv/config.json`, created with the defaults on first run. A
fully populated example is in `examples/config.json`; valid key and modifier
names are in `examples/keys.txt`.

### Cache

These are the knobs that decide how far ahead of you the viewer runs.

| Key | Meaning | Default |
|-----|---------|---------|
| `ram_budget_mb` | Ceiling on decoded pixels held in RAM, shared by both views. An eighth goes to thumbnails. | 4096 |
| `decode_threads` | Decode workers. `0` picks one per core, less one for the UI, capped at 8 — past which they saturate memory bandwidth rather than adding throughput. | 0 |
| `previews_resident` | Camera thumbnails kept on the GPU to stand in for images still decoding, and how far either side of the cursor their files are read. `0` turns the preview tier off. | 16 |
| `upload_budget_ms` | How long a frame may spend moving decoded images onto the GPU. | 8 |

A decoded image costs `width × height × 4` bytes: about 96 MB for a 24
megapixel photograph, plus the screen sized copy that is what actually gets
uploaded. The default budget therefore holds roughly 30 of them.

### General

| Key | Meaning | Default |
|-----|---------|---------|
| `output_icc_profile` | Display profile to convert into | `srgb` |
| `text_scaling` | Interface text scale | 1.25 |
| `metadata_tags` | Tags shown in the side panel, in order | File Name, Date/Time Original, Camera Model Name, Lens Model, Focal Length, Aperture, Shutter Speed, ISO, Image Size, File Size, Color Space, Directory |

### Image view

| Key | Meaning | Default |
|-----|---------|---------|
| `nr_loaded_images` | Images decoded either side of the one on screen | 64 |
| `gpu_resident_images` | Images kept as GPU textures | 8 |
| `max_image_edge` | Cap on the longest edge of a decoded image. `0` means as large as the GPU allows. Unrelated to the screen sized copy, which is worked out from your monitor and needs no setting. | 0 |
| `nr_images_shown` | Images displayed side by side | 1 |
| `should_wait` | Wait for the next image to finish decoding before advancing to it | true |
| `frame_size_relative_to_image` | White frame width, as a fraction of the shortest side | 0.2 |
| `scroll_navigation` | Use the scroll wheel to change image | true |
| `name_format` | Status bar name. `$(...#Tag#...)` fragments disappear when the tag is missing. Ex: `$(#File Name#)$( • ƒ#Aperture#)$( • #Shutter Speed#)$( • #ISO# ISO)` → `DSCF6114.JPG • ƒ5.6 • 1/500 • 200 ISO` | as above |

### Grid view

| Key | Meaning | Default |
|-----|---------|---------|
| `images_per_row` | Thumbnails per row | 5 |
| `preloaded_rows` | Off-screen rows decoded in each direction | 1 |
| `thumbnail_resolution` | Longest edge of a decoded thumbnail | 512 |
| `gpu_resident_thumbnails` | Thumbnails kept as GPU textures | 256 |

### Raw

| Key | Meaning | Default |
|-----|---------|---------|
| `source` | `"preview"` shows the JPEG the camera embedded; `"develop"` demosaics the sensor data with LibRaw | `preview` |
| `quality` | Demosaic effort: `"fast"` is bilinear, `"balanced"` is PPG, `"best"` is AHD | `balanced` |
| `camera_white_balance` | Use the white balance the camera recorded. Without it colours come out noticeably wrong. | true |
| `auto_brighten` | Stretch the histogram to use the whole range | true |
| `highlight_mode` | 0 clips blown highlights, 1 leaves them unclipped, 2 blends, 3 and up rebuild | 0 |

`source: "develop"` needs a build with `--features libraw`; without it the
viewer logs that it is showing previews instead.

### Tags

| Key | Meaning | Default |
|-----|---------|---------|
| `categories` | The tags you always want to hand, grouped. Searching matches category names as well as tag names. | Status, Subject |
| `recent_tags` | How many recently used tags to remember | 12 |
| `panel_width` | Starting width of the panel, in points | 260 |
| `sc_toggle_tag_panel` | Shortcut that opens the panel | `K` |
| `sc_rating` | Shortcuts that set a rating, listed from no stars upwards | `0` – `5` |

### Slideshow

| Key | Meaning | Default |
|-----|---------|---------|
| `seconds_per_image` | How long each image is held | 15 |
| `percent_zoom` | How far it drifts in over that time. `0` disables the movement. | 25 |
| `start_with_frame_enabled` | Start with the white frame on | false |
| `image_frame_background_color_override` | Hex backdrop while in slideshow mode | null |

## Default shortcuts

### General

| Key | Action |
|-----|--------|
| Backspace | Toggle between image view and grid view |
| Alt + Q | Exit |
| F1 | Toggle the menu |
| Ctrl + L | Navigation bar |
| T | Directory tree |
| Ctrl + F | Flatten (read files from all sub directories) |
| Ctrl + W | Watch the directory for new and changed files |
| I | Toggle the side panel: metadata and cache occupancy |
| K | Toggle the rating and tagging panel |
| 0 – 5 | Set the star rating of the open image |
| F10 | Toggle frame timings |

### Image view

| Key | Action |
|-----|--------|
| Arrow keys / Scroll | Next or previous |
| F | Fit the image to the screen |
| M | Fill the screen |
| Ctrl + M | Toggle: keep filling the screen while navigating |
| H / V | Fit horizontal / vertical |
| Alt + 1 | 100% magnification |
| Space | Zoom step |
| Ctrl + Scroll | Zoom |
| Drag | Pan |
| G | Toggle the white frame |
| + / - | More or fewer images side by side |

### Grid view

| Key | Action |
|-----|--------|
| Space | Scroll down |
| Click | Open that image in the image view |
| Ctrl + Scroll | More or fewer thumbnails per row |
| + / - | More or fewer thumbnails per row |

## User actions and context menu

Actions are external commands, bound either to a shortcut or to a right-click
menu entry. Supported placeholders:

- `{}` full path
- `{.}` path without extension
- `{/}` file name
- `{/.}` file stem
- `{//}` parent directory
- `{.//}` grandparent directory

Keep the commands simple; for anything involved, call a script and pass it the
path.

Examples:

- `gimp {}` — open the file in GIMP
- `darktable {.}.RAF` — open the adjacent Fujifilm raw in darktable
- `rate.sh {.}.RAF 5` — write a base XMP with a rating (see `examples/rate.sh`)

### Callbacks

A user action or menu entry may name a callback to run after the command
succeeds:

- `Pop` — remove the image from the collection
- `Reload` — decode the image again
- `ReloadAll` — reopen the whole directory
- `Advance` — move to the next image

## Font

The viewer ships with `Atkinson Hyperlegible Next`. Remove `custom_font` from
the default features to use the system font, or edit `src/ui/theme.rs` to point
at another one.

## Tools

`cargo run --example dump_metadata -- <path>...` prints everything the metadata
reader finds in a file, which is the quickest way to compare against exiftool
when adding tags.

`cargo run --features libraw --example develop_raw -- <path>...` develops a raw
at each quality setting and reports the size and the time, which is how to see
what the setting is worth on your own files and your own machine.
