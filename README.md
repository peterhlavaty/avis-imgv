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

Five tiers, from cheap to instant:

| Tier | Holds | Bounded by |
|------|-------|------------|
| Disk | every path the crawler found | the folder |
| Preview | metadata and the camera's thumbnail, read from the first 512 KB | `cache.previews_resident` |
| RAM | one screen sized copy of every image in the folder | `cache.ram_budget_mb` |
| RAM, full size | the images within reach, ready to be zoomed into | `cache.full_resolution_neighbours` |
| GPU | textures ready to draw, with their mip chains | `gpu_resident_images` / `gpu_resident_thumbnails` |

The preview tier is what puts something on screen immediately. A dedicated
thread reads the front of each file near the cursor, which takes a couple of
milliseconds and yields both the metadata for the side panel and the thumbnail
the camera stored. The thumbnail is uploaded reporting the size of the image it
stands for, so the layout is already right and nothing moves when the real
decode lands.

What is kept for every image in the folder is a copy no larger than the
screen, made on the decode worker. A 24 megapixel photograph is 96 MB of RGBA8
and a 1080p monitor can show three of those megapixels: holding all of it costs
a tenth of the memory budget per image and fifteen milliseconds of the UI
thread to upload, to no visible effect. Eleven megabytes each instead means a
folder of a few hundred photographs is resident all at once.

The image on screen and the two either side of it are *also* decoded at their
own resolution and kept ready, so zooming in shows the photograph rather than a
magnified copy of it. Which of the two is on the GPU follows how wide the image
is being drawn, and swaps back when you zoom out. Images you zoomed into keep
their full sized copy until something nearer needs the room, so walking back
through a folder finds them still sharp.

Every texture carries a mip chain, built on the GPU in one pass per level. A
photograph is nearly always drawn smaller than it is stored, and a plain
bilinear sampler reads four texels of every nine it should — which is what
makes fine detail sparkle and crawl as an image is panned.

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

## Marks: stars, flags, labels and tags

Three axes, because a cull needs three different answers, and every other
program keeps them apart for the same reason. Press `K` for the panel that
holds all of them, or use the keys with it shut — the status bar shows what a
photograph carries either way.

- **Stars.** `0` to `5`. Clicking the star a rating already ends on clears it.
  How good it is.
- **Keep and reject.** `P` keeps, `X` throws out, `U` takes either mark back
  off, and pressing the key of the mark a photograph already carries is the
  same as `U`. Whether it stays at all. Nought stars means "not looked at yet";
  a reject means "looked at, and no".
- **Colour labels.** `6` red, `7` yellow, `8` green, `9` blue, `Ctrl + 9`
  purple, and the key of the label already set clears it. What happens next —
  to retouch, to send, sent.
- **Tags.** The panel offers the tags you used most recently, the catalog you
  configured, and anything already on the other photographs of the folder;
  typing something new offers to create it. `tags.categories` is the list you
  always want to hand, and searching matches a category name too, so typing
  "places" offers everything filed under Places.

**Advance after marking.** `Ctrl + Shift + A`, or `tags.advance_after_marking`
in the configuration, makes a rating, a flag or a label move to the next
photograph by itself, which is what turns a cull into one keystroke a frame.
A mode rather than a held modifier, because on a Slovak or German keyboard the
digits *are* the shifted characters of the top row.

### Getting rid of them

`Delete` sends the photograph on screen to the platform's bin — the freedesktop
specification on Linux, the Recycle Bin on Windows — together with its sidecar,
as one unit. Nothing is asked, because the bin *is* the asking, and a dialogue
in the middle of a cull is what people complain about most in the tools that
have one. The cursor stays where it is rather than following the picture that
has gone, so what it lands on is the next one.

`Shift + Delete` deletes outright, for the cards and network shares that have no
bin, and asks first.

**File → Send rejected to the bin…** collects every photograph in the folder
marked with `X` and puts the lot in the bin behind one question, which is the
second half of a first pass: mark what is not staying, then be rid of it. The
sidecars are read for the whole folder rather than only for the frames already
looked at, which is a few milliseconds.

Nothing is ever removed with an unconditional delete: `fs::remove_file` appears
in this codebase only where the user has said "for good" and been asked twice.

### Somewhere else, rather than nowhere

`Alt + M` moves the photograph and `Alt + C` copies it, both to a small panel of
numbered folders: the digits pick one, `Enter` repeats the last, `Escape` leaves
them where they are, and pressing the same key twice in a row skips the panel
and repeats the last answer. A destination in the configuration may be a
relative path, in which case it follows the shoot rather than naming one —
a configured `Selects` means "beside these photographs" and works on every card
that is ever put in.

`Shift + X` moves the photograph into `_Rejected` beside it, which is what a
memory card or a network share has instead of a bin.

### Taking it back

`Ctrl + Z` puts back whatever the last thing that touched a file did, and says
what it is about to do first. It covers moving, copying, sending to the bin and
every mark — a rating pressed by mistake is one keystroke to undo — and it keeps
the last two hundred.

A copy is undone by sending the copies to the bin rather than deleting them,
because an undo should not itself be the thing nobody can take back. Coming back
out of the bin needs a platform that lets a program address what is in it:
Windows and the freedesktop specification both do, macOS does not, and there the
viewer says so rather than pretending.

### Where they are stored

In XMP sidecars beside the image — `DSC001.cr2.xmp` — which is what every raw
converter reads. Adobe's `DSC001.xmp` is read too, and whichever sidecar is
already there is the one edited: a develop history written by another tool
survives untouched, and a sidecar the reader cannot make sense of is left alone
and reported rather than replaced.

| What | Written as |
|------|------------|
| Stars | `xmp:Rating`, 0 to 5 |
| Reject | `xmp:Rating` = `-1`, which is what Adobe reserves for it and what Bridge, Lightroom, FastRawViewer and darktable all read |
| Keep | `digiKam:PickLabel` = `3` |
| Colour label | `xmp:Label`, always the English colour name, and read back against the names Bridge and Lightroom use as well |
| Tags | `dc:subject`, as an `rdf:Bag` |

Rejecting clears the stars and rating clears the rejection, because they are the
same field — which is the convention rather than a limitation.

Marks **inside** the image are read as well (JPEG `APP1`, PNG `iTXt`, WebP,
TIFF, and Windows Explorer's EXIF rating), so a photograph rated elsewhere shows
up already rated. Nothing is ever written back into the image file itself.

Saves happen on a thread of their own, so rating a photograph never waits on
the disk — and a save that fails says so on screen rather than in a log nobody
reads.

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
Benchmark: 501 images in 11.50s — 43.6 images/s, median frame 2.70ms
```

The frame time is what the viewer costs you; the images a second is what the
decoders can supply. Past about eight workers the decoders stop scaling — a 24
megapixel image is a hundred megabytes of output and they saturate memory
bandwidth long before they run out of cores — which is why `decode_threads`
defaults to eight rather than to a core count.

The numbers are there to be checked on your own machine and your own files.
`cargo run --release --example bench_decode -- <files>` breaks a single image
down stage by stage.

## Working on the whole folder

Three of the six modes act on the folder rather than on one picture. The menu
lists them under **Mode**, and `F2` cycles through them all.

Both start by reading the folder — the front of every file, for its metadata,
and its sidecar, for the rating and keywords — which takes a couple of
milliseconds a file across every core and does not stop you working while it
happens. Both then narrow it down, put it in an order, and show you exactly
what would happen to every file. Nothing is written until you say so.

**Sorting** is by name, capture time, type, size, rating, or any metadata tag.
Names sort the way a person reads them, so `IMG_9` comes before `IMG_10`, and
tags that mean numbers sort as numbers, so ISO 200 comes before ISO 1000. The
order matters: it is what the counter in a rename follows.

**Filtering** narrows by name, type, size, a metadata tag and what it says,
star rating, and keywords the file must or must not carry. Every rule is empty
by default, and rules combine with "and".

### Bulk rename

A name is a template: literal text, with `{...}` for the parts that differ.

| Placeholder | Becomes |
|-------------|---------|
| `{name}` | the name it has now, without the extension |
| `{counter}` | the number, padded to the digits you set |
| `{date}` `{time}` `{datetime}` | the capture time, as `2024-11-06`, `22-07-19`, or both |
| `{year}` `{month}` `{day}` | parts of the capture date |
| `{tag:Name}` | any metadata tag, such as `{tag:ISO}` |
| `{{` `}}` | a literal brace |

`holiday_{date}_{counter}` gives `holiday_2024-11-06_0001.jpg`. The counter's
first number, its step and its width are all set beside the template, and the
extension can be kept as it is or folded to one case.

Anything that cannot happen is shown in red and left alone: a template that
leaves a file with no name at all, two files that would end up with the same
one, or a name something else on disk already has. A rename that shifts a
numbered sequence onto itself works, and so does swapping two names, because
every file goes through a temporary name first. Sidecars follow the pictures
they belong to.

### Shift capture time

For a camera whose clock was wrong. Set an offset in days, hours, minutes and
seconds, forwards or backwards, and every selected photograph moves by it.

Which timestamps move is up to you: the boxes list the ones the selected files
actually carry — the moment the shutter opened, the moment the file was
written, and whatever else is there — and all of them are ticked to begin with.

The dates are rewritten where they already are rather than by rebuilding the
file. An EXIF timestamp is a fixed nineteen characters whatever it says, so a
shifted one takes exactly as much room as the one it replaces: no offset in the
file has to be recomputed, and the maker notes, the embedded preview and the
pixels are all left byte for byte as they were. Each file is written to a
temporary copy and renamed over the original, so an interrupted run cannot
leave half a photograph behind.

### Group shots

A photographer rarely takes one frame of anything worth taking. A bracket for a
high dynamic range merge is three or five frames of the same view at different
exposures; a focus stack is a dozen at different distances; a timelapse is
hundreds at a steady interval; and a burst is however many it took. They all
arrive in one folder, interleaved with the single frames.

This mode finds them and offers to tidy each one into a folder of its own —
`hdr1`, `hdr2`, `stack1`, `timelapse1`, `series1`. A number already taken on
disk is stepped over rather than tipped into, so running it again on a folder
that has grown does not mix new frames in among the old.

Frames belong to the same group when they were taken close enough together
**and** show the same thing. Both halves matter: the clock alone joins two
unrelated pictures taken a second apart, and the picture alone joins two visits
to the same view a week apart. What they look like comes from a sixty-four bit
summary of the camera's own thumbnail — shrink it to a grid of brightness and
record which cells are brighter than their neighbours — so a frame two stops
darker still matches the one before it, and a different scene does not.

What kind of group it is follows from what the photographer changed:

| Reading | What it looks like |
|---------|--------------------|
| HDR bracket | three or more frames within half a minute, the exposure moving and the aperture and focus not |
| Focus stack | three or more with the focus distance moving and the exposure not |
| Timelapse | eight or more at an interval steady enough to be a timer rather than a finger |
| Series | everything else: the same thing, more than once |

Every one of those is a proposal. The kind is a dropdown, **Not a group** puts
a whole run back among the loose frames, `×` takes one frame out, and anything
loose can be put into any group from the bottom of the list — where it lands in
the order it was taken, not at the end. Three numbers at the top decide how the
folder is read at all: the gap that ends a run, how alike two frames have to
be, and the fewest frames worth calling a group.

## Comparing two frames

`N` pins the photograph on screen and the next one side by side, sharing **one
zoom and one pan**: 100% on an eye in one pane puts the same eye at the same
magnification in the other, which is what choosing between two frames of the
same thing actually is.

| Key | What it does |
|-----|--------------|
| `Tab` | Which pane the keys are about, drawn with a border |
| `←` `→` | Try a different photograph against the ones that are staying |
| `Ctrl` `+` / `Ctrl` `-` | More or fewer panes, up to eight |
| `/` | Drop the focused pane; the survivors re-tile larger |
| `Enter`, `Escape`, `N` | Leave it |

Every marking key applies to the pane with the focus and to nothing else:
marking "everything displayed" is the one thing a comparison must never do.

The pinned set is what makes it a comparison rather than the side-by-side view
`nr_images_shown` gives: the panes stay where they are while the eye moves
between them, and the arrow keys change the *candidate* rather than the lot.

## Narrowing the folder down

`F3` opens a bar above the photograph: stars from and to, the flag, the colour
label, the name, a keyword, the file types, and what the folder is ordered by.
Every rule is "anything" until it is touched, and they combine with "and".

It applies to the photographs, not to a mode that draws none of them. `Left` and
`Right` walk what is left, the contact sheet shows what is left, the status bar
says `2/27 (+2)` — where you are, how many are on show, and how many are being
held back — and the whole thing re-evaluates the instant a mark changes, so
rejecting a frame with "Not rejected" on takes it out of the strip at once.

`\` sets the rules aside without forgetting them, which is how "what did I
hide?" costs one key and answering it costs nothing.

Nothing is re-read or re-decoded by any of this: the caches still hold the whole
folder and the filter is a list of positions into it, so a rule changed in the
middle of a cull costs a vector rather than a folder's worth of decoding.

## Slideshow

A mode of its own: the window goes fullscreen, the status bar goes away, and
the pictures change themselves. The arrow keys still work — moving by hand just
restarts the clock — and leaving the mode puts the window back the way it was.

**Settings → Slideshow…** sets how long each picture is held and what happens
while it is up:

| | |
|---|---|
| Hold still | The whole picture, fitted to the screen, not moving. |
| Drift inwards | Fills the screen and creeps closer while it is up. |
| Travel across | Fills the screen at its own shape and travels across it, so the whole picture has been seen by the time the next one comes up. |

The last is for pictures that do not match the shape of the screen: rather than
letterboxing a panorama into a strip down the middle, it fills the screen at
the picture's own proportions and moves along the overflowing side, arriving at
the far edge exactly as the picture's turn ends.

## Changing the keys

**Settings → Keyboard…** lists every key the viewer listens for, grouped by
where it applies and each with a sentence saying what it does. Click a key,
press the one you want, and it is written to the configuration file straight
away; escape leaves it alone. Two things on one key are not refused — sometimes
that is what a person means — but they are pointed out.

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
unrecognised falls back to a scan for the largest embedded JPEG stream. Where a
file embeds no JPEG at all — a DNG written by Camera Raw keeps its
reduced-resolution copy as plain pixels — that copy is read instead.

The catch is resolution: a CR3 preview is 1620x1080, which is all Canon stores,
and that Camera Raw DNG carries 256x171. The side panel reports the size of the
*photograph* rather than of the copy being shown, and adds a `Preview Size` line
when the two differ, so it is clear which you are looking at.

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
| `full_resolution_neighbours` | How far either side of the image on screen to also decode at full resolution, ready to be zoomed into. Each one is a whole decoded photograph in memory. `0` turns that off. | 1 |
| `upload_budget_ms` | How long a frame may spend moving decoded images onto the GPU. | 8 |

A screen sized copy costs `width × height × 4` bytes at the size of your
monitor: about 11 MB for a 24 megapixel photograph on a 1080p screen, so the
default budget holds a couple of hundred of them. A quarter of the budget is
set aside for the full resolution copies, which are 96 MB each.

### General

| Key | Meaning | Default |
|-----|---------|---------|
| `output_icc_profile` | Display profile to convert into | `srgb` |
| `text_scaling` | Interface text scale | 1.25 |
| `metadata_tags` | Tags shown in the side panel, in order | File Name, Date/Time Original, Camera Model Name, Lens Model, Focal Length, Aperture, Shutter Speed, ISO, Image Size, File Size, Color Space, Directory |
| `sc_delete` | Send the picture on screen to the bin | `Delete` |
| `sc_delete_permanently` | Delete it outright, after asking | `Shift + Delete` |
| `sc_fullscreen` | Fill the screen, and give it back | `F11` |
| `sc_filter` | Show or hide the filter bar | `F3` |
| `sc_suspend_filter` | Set the rules aside without forgetting them | `\` |

### Image view

| Key | Meaning | Default |
|-----|---------|---------|
| `nr_loaded_images` | Images decoded either side of the one on screen. Trimmed to what the RAM budget can hold, so the default is deliberately more than any budget will grant. | 512 |
| `gpu_resident_images` | Images kept as GPU textures | 8 |
| `max_image_edge` | Cap on the longest edge of a decoded image. `0` means as large as the GPU allows. Unrelated to the screen sized copy, which is worked out from your monitor and needs no setting. | 0 |
| `nr_images_shown` | Images displayed side by side | 1 |
| `sc_compare` | Pin this picture and the next side by side, sharing one zoom and one pan | `N` |
| `should_wait` | Wait for the next image to finish decoding before advancing to it | true |
| `frame_size_relative_to_image` | White frame width, as a fraction of the shortest side | 0.2 |
| `scroll_navigation` | Use the scroll wheel to change image | true |
| `enlarge_to_fit` | Enlarge a photograph smaller than the window to fill it. What needs it is a raw file's embedded copy: some DNGs carry a 256 pixel preview and nothing else. | true |
| `name_format` | Status bar name. `$(...#Tag#...)` fragments disappear when the tag is missing. Ex: `$(#File Name#)$( • ƒ#Aperture#)$( • #Shutter Speed#)$( • #ISO# ISO)` → `DSCF6114.JPG • ƒ5.6 • 1/500 • 200 ISO` | as above |

### Grid view

| Key | Meaning | Default |
|-----|---------|---------|
| `images_per_row` | Thumbnails per row | 5 |
| `cell_aspect` | How wide a cell's picture is against its height. 1.5 is the three-to-two most cameras shoot; 1.0 brings back the square cells, which for a folder of landscape photographs left about forty-four per cent of the sheet drawn in grey. | 1.5 |
| `preloaded_rows` | Off-screen rows decoded in each direction | 1 |
| `thumbnail_resolution` | Longest edge of a decoded thumbnail | 512 |
| `gpu_resident_thumbnails` | Thumbnails kept as GPU textures | 256 |
| `sc_cycle_badges` | Cycles what is drawn under each thumbnail: nothing, the marks, or the marks and the name | `Ctrl + I` |
| `sc_select` | Picks the photograph under the cursor out, or puts it back | `Space` |
| `sc_select_all` | Picks out everything on show, or puts it all back | `Ctrl + A` |

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

### Cull

| Key | Meaning | Default |
|-----|---------|---------|
| `destinations` | Folders one keystroke can send a photograph to, in the order the digits reach them. A relative path is taken against the open folder. | Selects, To edit |
| `rejected_folder` | What the folder for the frames that are not staying is called | `_Rejected` |
| `sc_move` `sc_copy` | Open the panel that asks where | `Alt + M` `Alt + C` |
| `sc_reject_folder` | Move into the rejected folder | `Shift + X` |
| `sc_undo` | Put back whatever the last thing did | `Ctrl + Z` |

### Tags

| Key | Meaning | Default |
|-----|---------|---------|
| `categories` | The tags you always want to hand, grouped. Searching matches category names as well as tag names. | Status, Subject |
| `recent_tags` | How many recently used tags to remember | 12 |
| `panel_width` | Starting width of the panel, in points | 260 |
| `sc_toggle_tag_panel` | Shortcut that opens the panel | `K` |
| `sc_rating` | Shortcuts that set a rating, listed from no stars upwards | `0` – `5` |
| `sc_pick` `sc_reject` `sc_unflag` | Keep, throw out, and take the mark off | `P` `X` `U` |
| `sc_label` | Shortcuts for the colour labels, in the order red, yellow, green, blue, purple | `6` – `9`, `Ctrl + 9` |
| `advance_after_marking` | Move to the next picture as soon as one is rated, flagged or labelled | false |
| `sc_toggle_advance` | Turns that on and off | `Ctrl + Shift + A` |

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
| F2 | Next mode: image, gallery, bulk rename, shift capture time, group shots, slideshow |
| Alt + M / Alt + C | Move or copy the picture to a folder |
| Shift + X | Move it into the rejected folder |
| Ctrl + Z | Put back whatever the last thing did |
| F3 | Show or hide the filter bar |
| \ | Show everything, without forgetting the rules |
| F11 | Fullscreen |
| Alt + Q | Exit |
| F1 | Toggle the menu |
| Ctrl + L | Navigation bar |
| T | Directory tree |
| Ctrl + F | Flatten (read files from all sub directories) |
| Ctrl + W | Watch the directory for files appearing, changing or going |
| I | Toggle the side panel: metadata and cache occupancy |
| K | Toggle the rating and tagging panel |
| 0 – 5 | Set the star rating of the open image |
| P / X / U | Keep it, throw it out, or take either mark off |
| 6 – 9, Ctrl + 9 | Colour label: red, yellow, green, blue, purple |
| Ctrl + Shift + A | Move to the next picture after every mark |
| Delete | Send the picture on screen to the bin, sidecar and all |
| Shift + Delete | Delete it outright, after asking
| F10 | Toggle frame timings |

### Image view

| Key | Action |
|-----|--------|
| Arrow keys / Scroll | Next or previous |
| Home / End | First or last picture on show |
| Page Up / Page Down | Ten at a time |
| F | Fit the image to the screen |
| M | Fill the screen |
| Ctrl + M | Toggle: keep filling the screen while navigating |
| H / V | Fit horizontal / vertical |
| Alt + 1 | 100% magnification |
| R | Put this picture where the last one was left |
| Space | Zoom step |
| + / - | Zoom in or out |
| Ctrl + Scroll | Zoom |
| W A S D | Pan, while the key is held |
| Drag | Pan |
| G | Toggle the white frame |
| Ctrl + / Ctrl - | More or fewer images side by side, or panes while comparing |
| N | Compare this picture with the next |
| Tab | Which pane the keys are about |
| / | Drop that pane; the survivors re-tile |
| Escape | Leave the comparison |

Zooming keeps the point under the pointer, so magnifying an eye near the edge
of the frame brings the eye closer rather than pushing it off screen. The keys
that are about the panel rather than about a point in the picture — fit, fill,
fit horizontal, fit vertical — hold the middle instead.

`100%` means one image pixel to one **screen** pixel, counted in the pixels the
screen actually has rather than in the points a window at 125% scaling is laid
out in. The readout beside the slider says the same number, and the slider runs
from 1% to 1600% logarithmically: it used to run from a tenth to ten times the
*fitted* size, which on a twenty-four megapixel photograph could not reach
actual size at all.

Zoom and pan belong to the image, not to the window: leaving a photograph
half way into a corner and coming back to it later finds it exactly there.

### Grid view

The contact sheet is where a first pass actually happens, so it says what it
knows: the stars, the flag and the colour label under every thumbnail, a red
tint over the rejected ones, the file name when asked for, a white outline on
the photograph the image view is on and a blue one on the photograph the
keyboard is on. Every marking key applies to the one under the cursor rather
than to whatever the other view was last left on.

| Key | Action |
|-----|--------|
| Arrow keys | Move the cursor about the sheet |
| Home / End | First and last picture |
| Enter | Open the one under the cursor |
| Ctrl + I | Cycle what the cells say: nothing, the marks, the marks and the name |
| Space | Pick the one under the cursor out, or put it back |
| Shift + arrows | Pick out everything walked over |
| Ctrl + A | Pick out everything on show, or put it all back |
| Escape | Put the selection down |
| Ctrl + Click | Pick that one out |
| Shift + Click | Pick out the run up to it |
| Click | Open that image in the image view |
| PageDown | Scroll down |
| Ctrl + Scroll | More or fewer thumbnails per row |
| + / - | More or fewer thumbnails per row |

#### Picking several out at once

Whatever is picked out is what the next command is about. A rating, a flag, a
colour label, a keyword clicked in the tag panel, a move, a copy, a deletion:
each of them applies to the selection when there is one and to the photograph
being looked at when there is not, so tagging two hundred frames is one
keystroke rather than two hundred.

The cells that are picked out carry a blue wash and a tick, the corner says how
many there are, and the tag panel says how many it is about to change. What the
mark ends up as is decided by the first photograph in the set and then applied
to all of them, so a set never ends up half flagged and half not.

Undo takes the whole thing back in one press, however many photographs it
touched. Deleting a selection asks first even when it is only going to the bin,
because the cost of a wrong keystroke there is a folder rather than a frame —
`Enter` or `Y` answers it, `Escape` leaves them alone.

The selection is held as positions in the folder rather than as file names, so
narrowing the folder down with the filter bar does not throw it away: pick
frames out, filter to the ones you kept, mark them, and the set is still the
set.

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
