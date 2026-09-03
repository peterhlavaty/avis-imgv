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
- **Tags with levels.** A keyword written `Places|Slovakia|Tatras` is filed
  under its levels: the panel draws it as a tree, the sidecar records the path
  where Lightroom, darktable, digiKam and exiftool read it, and the keyword
  itself still goes into `dc:subject` so a program that has never heard of
  hierarchies finds it anyway. Narrowing by `Slovakia` then finds everything
  below it, and taking the keyword off takes its path with it.

  `tags.catalog_file` points at a keyword list exported from another program —
  one tag a line, indentation making the hierarchy — so years of keywords built
  up in Lightroom or digiKam do not have to be typed again:

  ```text
  # Where I shoot
  Places
      Slovakia
          Tatras
      Austria
  Subjects
      Portrait
  ```

  A relative name is taken against the configuration file, the outermost level
  becomes a category in the panel, and a list that cannot be read is a warning
  in the log rather than a refusal to start.

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

### A bin of the viewer's own

The platform's bin is the default and stays it, because Delete meaning what it
means in every other program is what nearly everybody expects. It has two costs
and both land on a photographer: it does not reach a memory card or a share
over the network at all, and it cannot be *looked in* — the question after an
hour of culling being "did I throw out anything I meant to keep".

Setting **What the delete key means** (`cull.bin`) to *A folder of the viewer's
own* answers both. It is deliberately nothing clever: a folder, at
`cull.bin_folder` or beside the viewer's own files, that photographs are
*moved* into. Because it is only a folder it opens like any other —
**File → Open the bin** — so what an hour of culling threw away can be walked
through, compared and zoomed before any of it is really gone.

- **It remembers where everything came from.** A `.avis-bin.json` inside the
  bin holds a name and a path per photograph, so `Ctrl + B`, or **Put it back
  where it came from** on the second button, returns it to the folder it was
  thrown out of — making that folder again if it has since been tidied away.
  Two folders both holding a `DSC0001.jpg` is the ordinary case, so the second
  one in is filed as `DSC0001 (2).jpg` and its origin recorded under that name.
- **Emptying it deletes the folder**, and is always asked about. A folder with
  no `.avis-bin.json` in it is refused: `remove_dir_all` against a path that
  came out of a text box is the most dangerous line in the program, and it only
  ever runs where the note the bin keeps about itself is.
- **Standing in the bin, `Delete` means for good**, as it does in every file
  manager's own bin, and asks; `Shift + Delete` is the same thing without the
  detour. Both take one photograph or a whole selection, which is how a bin is
  emptied in part.
- **A bin left with something in it is asked about on the way out** — empty it
  and close, keep it and close, or do not close. Turn that off with
  `cull.ask_to_empty_the_bin` if the bin is being used as a holding folder.
- Undo covers all of it. Sending to the viewer's own bin is one step in the
  history and comes straight back out, which is not true of the platform's on
  macOS.

Moving onto a different drive from the card is a copy rather than a rename, so
a folder of sixty megapixel raws takes as long to throw out as it takes to
write; the same is now true of a destination on another drive, which used to
fail outright.

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

`Ctrl + Z` takes back the last thing done and `Ctrl + Y` does it again. It
covers everything: moving, copying, sending to the bin and every mark — a
rating pressed by mistake is one keystroke to undo — and also the mode, the
panels, the photograph you were on, the zoom and pan, the number of columns,
what the folder is narrowed to, what is picked out, and every setting. It keeps
all of them, unless `history.remember` is set to a number.

Nothing has to be told about it. The history watches what the program *looks
like* once at the foot of each frame rather than being called from the places
that carry out commands, so a key, a menu, the second button, the bottom bar
and a mouse gesture are all covered by the same few lines, and so is anything
added later.

A gesture is one line rather than sixty. Nothing is recorded while the button
is down, so a zoom dragged out is one entry — where it started and where it was
let go; and a wheel turned twice or an arrow held down folds into one entry as
long as the notches land within `history.merge_within_ms` of each other.

Going back never throws anything away. Having undone four things, all four are
still there and `Ctrl + Y` walks forward through them one at a time; and going
back and then doing something *different* keeps both, so the four that were
undone are still reachable rather than overwritten. What that makes is a tree
rather than a list.

`Ctrl + H` shows and hides the panel down the right-hand side listing
everything done this run, in the order it happened. The key is
`history.sc_panel` and can be rebound like any other; the second button
anywhere in the panel — the heading and the blank below the list included —
offers *Hide this panel*, a key to bind to it and a route to its settings, and
*Show the history panel* in the settings does the same. However it is changed
it is written down, so the next launch opens with it as it was left. The row you are on is picked out; anything
taken back, or on a branch you left, is still in the list and drawn in italics
rather than removed. Every row says what it was rather than what kind of thing
it was: *Gave DSC0142.jpg 3 stars*, *Zoomed in to 250%*, *Moved 12 files to
Selects*, *Sorted by capture time, backwards*, *Turned on show the filmstrip*.
A row is cut off with an ellipsis when the panel is too narrow for it, with the
whole of it on the hover. Clicking a row
takes the viewer back — or forward — to just after it. The second button on a row offers *Do only this again*, which
carries that one thing out where you are now and files it as the latest thing
done, rather than jumping to it.

One press of undo walks back until it has taken back something worth stopping
on, and `history.undoes` says what counts. All four kinds are ticked to start
with. Unticking *Where you were* does not stop the viewer remembering where you
were — it stays in the panel and can still be clicked — it only stops `Ctrl + Z`
coming to rest there, so that one press after twenty photographs walked past
still lands on the rating. Where you were goes back with it, because all of it
did happen.

The history is kept between runs, and used only when it still describes what is
on disk. Written beside the session file on the way out, it carries a signature
of every photograph and every sidecar any of its rows mentions — the size and
the time each was last written, or the fact that it is not there — together
with the configuration file. If any of that has moved by the next run, the
history is discarded and the viewer says so, because an undo against a file
something else has edited is the one thing this program is most careful never
to do. Rows about where you were name no files, so a run spent looking around
never invalidates anything. It follows `general.restore_session` like the rest
of what is remembered between runs, and the last five hundred rows are kept.

Anything that would touch more than one file says what it is about to do and
waits — that is `cull.confirm.undo_several`, and until now it was a setting
nothing read.

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
| Tags with levels | `lr:hierarchicalSubject`, as an `rdf:Bag` of `Places|Slovakia|Tatras` paths, beside the flat keywords rather than instead of them |
| A turn | `tiff:Orientation`, the same eight values EXIF uses, composed with the camera's own before anything is drawn |

Rejecting clears the stars and rating clears the rejection, because they are the
same field — which is the convention rather than a limitation.

### Turning a photograph

`[` and `]` turn it a quarter. **Turn** on the second button, on the photograph
or on a cell, offers the same two and three more: upside down, mirror left to
right, mirror top to bottom. The turn is written to the sidecar and **the
photograph itself is never touched**: a raw file cannot be rewritten without
losing something, and a JPEG re-encoded is a JPEG made worse. It is composed
with the camera's own orientation, so what is drawn is one turn however many
went into it — a mirror on a frame the camera had already turned is still one of
the eight values EXIF defines — and it survives a restart because the decoder
reads the sidecar. `Ctrl + Z` puts both the sidecar and the picture back.

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
  --installPath "C:\Program Files (x86)\Microsoft Visual StudioÂ2\BuildTools" ^
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
  --reset-text-size
                Put the interface text back to its normal size and write that
                to the settings, for when it has been made unreadable.
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

### Templates

One grammar, used by the bulk rename, the status bar line and anywhere else the
viewer builds a sentence about a photograph: literal text, with `{...}` for the
parts that differ.

| Placeholder | Becomes |
|-------------|---------|
| `{name}` `{ext}` `{folder}` | the parts of where it is |
| `{counter}` | the number, padded to the digits you set (rename only) |
| `{date}` `{time}` `{datetime}` | the capture time, as `2024-11-06`, `22-07-19`, or both |
| `{year}` `{month}` `{day}` | parts of the capture date |
| `{hour}` `{minute}` `{second}` | parts of the capture time |
| `{iso}` `{aperture}` `{shutter}` | how it was exposed |
| `{focal}` `{lens}` `{camera}` | what it was taken with |
| `{dimensions}` `{size}` | how big it is |
| `{stars}` `{rating}` `{flag}` `{label}` `{keywords}` | what you put on it |
| `{tag:Name}` | any metadata tag, such as `{tag:ISO}` |
| `$( … )` | kept only when what is inside it resolves |
| `{{` `}}` | a literal brace |

Anything a photograph cannot answer expands to nothing, so one template serves a
folder where only some of the pictures carry a lens name.

`$( … )` is what makes that bearable in a line rather than a file name. A
separator you cannot suppress leaves ` • •  • ` on a photograph that answers
nothing, so the literal text inside a group goes with the value it was
decorating: `{name}$( • {iso} ISO)` gives `IMG_1234 • 400 ISO` where there is an
ISO and `IMG_1234` where there is not.

The older `#Tag#` spelling still works and means `{tag:Tag}`, so a
`name_format` written before this still says what it said.

#### Bulk rename

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
a whole run back among the loose frames, `Ã` takes one frame out, and anything
loose can be put into any group from the bottom of the list — where it lands in
the order it was taken, not at the end. Three numbers at the top decide how the
folder is read at all: the gap that ends a run, how alike two frames have to
be, and the fewest frames worth calling a group.

## Marking out part of a photograph

Drag on a photograph that fits the window and a rectangle follows the pointer.
Let go and it stays: the rest of the picture darkens behind it, its four sides
and four corners can be taken hold of and dragged, and the pointer says which
by turning into the arrows for that side.

| Gesture | What it does |
|---------|--------------|
| Left drag on the photograph | Marks out a rectangle |
| Drag a side or a corner | Moves that side, never past the one opposite |
| Click inside it | Magnifies until the marked area fills the window |
| Click outside it, or `Escape` | Forgets it |
| `Enter` | The same as clicking inside it |
| `Ctrl + C` | Its pixels on the clipboard, decoded at full size (⌘C on macOS) |
| Right button inside it | *Zoom to it*, *Copy the marked area*, *Clear the marking* |

The rectangle is held against the photograph rather than against the screen,
so it stays on the same eyelash while you zoom, pan and turn the frame. It
belongs to the photograph on screen and goes when that changes: a rectangle
drawn over one frame means nothing over the next, and a marking left behind is
a marking somebody copies by accident.

**Copying it decodes the file again at full size** and turns it the right way
up before cutting the marked part out, on a thread of its own, so what reaches
the clipboard is the camera's pixels rather than the screen sized copy that was
being drawn. With nothing marked, `Ctrl + C` copies the whole photograph — the
first key that verb has had. Either way the viewer says how many pixels went.

`Ctrl + C` is one of the handful of keys that cannot be rebound, and for a
reason worth knowing: the toolkit turns the copy chord into a copy *event* and
never reports the key, so a row in the editor for it would be a row that did
nothing. Read as the event it is, it is right on every platform for nothing —
⌘C on a Mac, and the dedicated Copy key where a keyboard has one.

**It never takes the drag away from moving the photograph.** With the whole
picture in the window there is no slack to pan into and a left drag already did
nothing at all, which is the gesture the marking is given.  The moment there is
somewhere to pan to, the left button goes back to panning; `mouse.mark_area` is
`always` for whoever would rather mark a magnified photograph and pan with the
wheel pressed, and `never` for whoever wants the dead gesture back.

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
| `Escape`, `N` | Leave it |

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

## Stacks

`Ctrl + G` shows the folder stacked: every burst, bracket, focus stack and
timelapse becomes one cell with a count on it and a glyph for what kind of run
it is. Thirteen frames become six cells, and the six are six different
photographs rather than three photographs and ten near-copies.

Nothing is written. Lightroom keeps its stacks in a catalogue and Bridge in a
hidden file beside the pictures; this works them out from what the files
already say — the clock and what the frames look like — every time it is asked.
Turning stacks off leaves nothing behind to clean up, and no other program has
to know anything happened.

| Key | Action |
|-----|--------|
| `Ctrl + G` | Stack the folder, or put every frame back |
| `E` | Open the run under the cursor, or fold it up again |
| `,` `.` | Walk the frames of a folded run without opening it |
| `Ctrl + ←` `Ctrl + →` | Step to the run before or after this one, over a burst rather than through it |

The bar on `F3` carries the rest: how many runs were found, "fold all" and
"open all", the longest pause that is still one run, and how alike two frames
have to be to belong together. That last one is a slider because it is a
judgement rather than a number — drag it and watch the runs join up or come
apart.

The frame that stands for a folded run is the sharpest one that could be
measured, which is usually the question a burst is asking. `,` and `.` change
it, and the status bar says where you are the whole time:
`Series 2 Â· frame 4 of 17 Â· stack 6 of 41`, in amber while the run is folded.
The first word is the kind of run, and **Help → What the marks mean** is the
legend for the four glyphs.

It is the same mechanism as the filter — a list of positions into the folder —
so stacking composes with narrowing and ordering, and nothing is decoded twice
for it. A rule that hides the frame standing for a run leaves the run standing
on the next frame that survived, rather than taking the whole burst out of the
folder.

Reading a folder for stacks means reading what every file says about itself, so
it happens on a scan of its own the first time it is asked for; the sheet folds
up as the reading reaches each run. It is never done for a folder nobody asked
to stack.

## Slideshow

A mode of its own: the window goes fullscreen, the status bar goes away, and
the pictures change themselves. The arrow keys still work — moving by hand just
restarts the clock — and leaving the mode puts the window back the way it was.

The **Slideshow** page of the settings window sets how long each picture is held
and what happens while it is up:

| | |
|---|---|
| Hold still | The whole picture, fitted to the screen, not moving. |
| Drift inwards | Fills the screen and creeps closer while it is up. |
| Travel across | Fills the screen at its own shape and travels across it, so the whole picture has been seen by the time the next one comes up. |

The last is for pictures that do not match the shape of the screen: rather than
letterboxing a panorama into a strip down the middle, it fills the screen at
the picture's own proportions and moves along the overflowing side, arriving at
the far edge exactly as the picture's turn ends.

## Settings

`Ctrl + ,` opens the settings window on the page it was last left on;
**Settings → All settings…** does the same. Eleven pages, named for what you are
doing rather than for what the field is made of:

| | |
|---|---|
| **Opening a folder** | What a launch starts with, and what a folder opens as: its order, its filter, whether its bursts are folded, and what counts as one run of frames. |
| **The photograph** | The overlay, the frame, the ground behind it, and how far one press of a movement key goes. |
| **The contact sheet** | The cells and the strip of thumbnails. |
| **Stars, flags and labels** | The marks, and what a new sidecar is called. |
| **Keywords** | The keyword list, a keyword file, and the panel. |
| **Moving and deleting** | Where photographs go, and which of the reversible things ask first. |
| **Raw files** | Whether a raw file shows the camera's preview or the developed sensor data, and what pairs with what. |
| **Slideshow** | How long each picture is held, and whether it moves. |
| **Speed and memory** | What is held in RAM and on the graphics card, and how hard the viewer works. |
| **The window** | Light or dark, text size, the panels, and the file paths. |
| **Keys and mouse** | Every key the viewer reads, including the ones it reads for itself. |

A page in that list is a row rather than a name: the whole width of the list
answers the press, wherever along it you land.

The search box holds the cursor when the window opens. It is over the name, the
sentence, other programs' words for the same thing, and the path — so "blurry
thumbnails" finds the thumbnail resolution, "where do rejects go" finds the
rejects folder, and `cache.ram_budget_mb` pasted from a forum post lands on the
control. It never comes back empty.

Every change is written as it is made. A bullet beside a row that differs from
its default puts it back when clicked; a count beside each page in the list says
how many of its rows were changed. Reset always says how much it covers — this
setting, this page, or everything — and the wider two show what they would change
before changing it.

**Save what I have changed…** writes out only the fields that differ from the
defaults: a small file that goes into version control and onto another machine.
Key bindings and machine-specific paths are left out unless asked for.

If something in the configuration file cannot be acted on — a screen profile
that matches nothing, a keyword file that is not there, a rejects folder with no
name — the window says so across the top, with a button that goes to the
control. A value outside what a control can produce is shown, marked, and left
exactly as it was written: hand-editing always wins.

### While it is working

A folder is read a few milliseconds at a time, between frames, so the window
keeps drawing while a deep tree or a share over the network is walked — and a
strip at the foot of the window says what is happening and how many photographs
have been found so far. It used to be three synchronous lines: on a slow tree
the window stopped repainting entirely, with nothing on screen to say why,
which is the one state that draws nothing because the program is not drawing.

The strip appears for three things and says which: reading a folder, reading
the times a stack is grouped by, and decoding — the last only once it has been
going for half a second, because something is nearly always decoding and a mark
that is always on says nothing.

A percentage only where there is an honest total. The stack read counts what it
is working through, so it gets a bar; a folder walk does not know how many
photographs it will find until it has found them, so it gets a spinner and a
count. The cache readout cannot give one at all: what is in RAM is the length
of a list trimmed to a budget, so it is permanently less than the folder and a
bar driven by it would never fill.

### While a window is open

A window the viewer opens over itself owns the mouse and the keyboard while it
is up. The wheel scrolls the page of settings rather than walking the folder
behind it, a click lands on the window rather than on a thumbnail behind it,
and the keys mean what the window says they mean — a digit typed into the
search box is a digit and not a star. The bar at the top, the bar at the
bottom, the filter bar and the keyword panel grey out to say so; the
information panel stays readable, because reading it is not doing anything.

`Escape` shuts the window in front. The first press leaves whatever field has
the cursor and the second shuts the window, so a search half typed is not lost
to a key pressed once. The questions — the two deletions, the bulk undo, and
where photographs are being sent — answer `Escape` themselves, where it means
"leave them alone".

The same holds for the keyboard editor, the sheet of keys, the four windows the
Help menu opens, the "go to" bar and the directory tree.

### When a change takes effect

Everything in the settings window applies while the window is open, with one
exception. Most of it is the next frame; the seventeen fields the caches are
built from — the two budgets, the preload radii, the decode ceiling, the
thumbnail resolution, the camera-thumbnail count, the five raw settings and the
screen profile — apply when you let go of the control, because the way to apply
them is to build the caches again and a slider on true per-frame apply would do
that sixty times a second.

**`cache.decode_threads` is the exception**, and the only row in the whole
window that carries the `↻` badge. The decode pool is spawned once and shared
by both views; draining a running pool mid-session is a larger job than it is
worth. While a change to it is waiting, the window says so in a band across its
top and offers to restart.

A setting about the *next* launch — which mode it opens in, which folder, which
panels are up, whether the session is restored — is not a restart and carries no
badge: the change has taken effect and there is nothing on screen for it to
change. A badge means *your change has not taken effect*, and using it for
changes that have is what teaches people to ignore it.

Six values a key nudges are written back to the file: the overlay's corner, how
many photographs are side by side, how many thumbnails are across, what is drawn
under them, whether marking advances, and whether the strip of thumbnails is up.
Where the window is left is where the next launch starts.

## Changing the keys

A command has as many keys as you give it. The arrow keys and `WASD` can both
walk the folder; the key another viewer used can sit beside the one this one
chose.

**Settings → Keyboard…** lists every key the viewer listens for, grouped by
where it applies and each with a sentence saying what it does. Click one and a
window opens holding that command alone: every key bound to it, a cross beside
each, and **Add a key…**, which takes the next key pressed, modifiers and all.
Escape leaves it alone. Changes are written to the configuration file straight
away.

The same window is reached from the key on the settings page, from a row in the
sheet of keys, and from **Keys for…** in the right-click menu of the thing
itself — the panel, the badge in the bottom bar, the mask, the star. A person
who can see a control changes its keys from where they are standing.

Taking the last key away leaves the command with no key, which is a state the
list draws as *no key* and nothing else can reach it by. There is no key that
means "unbind" any more, which is what makes `Delete` and `Backspace` bindable
like anything else — `Delete` being the key that sends a photograph to the bin,
and so the one somebody rearranging a keyboard most often wants to move.

Two things on one key are not refused — sometimes that is what a person means —
but they are pointed out, against the key rather than against the command: a
command can be clear on its first key and taken on its second, and saying which
is which is the difference between a warning that helps and one that does not.

In the configuration file the first key stays where it always was and the rest
go under `also`, so a file nobody has added a key to is unchanged:

```json
"sc_next_image": {
  "key": "d",
  "modifiers": [],
  "also": [{ "key": "ArrowRight", "modifiers": [] }]
}
```

## Help

`F1` shows the menu bar, which starts up on a first run and thereafter is
wherever it was left.

**View** is the list of panels, with a tick against the ones on screen and the
key that shows and hides each beside it — the menu bar, the performance
readout, the filter bar, the metadata panel, the history, the stars and
keywords panel and the strip of thumbnails. The same list is behind **Show** at
the foot of the photograph's own menu, because the photograph is the whole
window once the panels are away and so the only surface left to ask. The status
bar is not in either: it cannot be put away, and a tick nothing can clear is
worse than no row at all.

**Help** carries the things a person needs when the program has stopped
explaining itself:

| | |
|---|---|
| **Keys… `?`** | Every key bound in the mode on screen, with what it does and a box to search them. |
| **Keyboard…** | Change what a key does. |
| **What the marks mean** | The legend: the four stack glyphs, the three states of the strip under a cell, the overlay colours, the border round a pinned pane. |
| **Template placeholders…** | Everything that may go in a name template, an overlay line or a cell caption. Click one to copy it. |
| **Recent messages…** | The last hundred things the viewer said, whether or not they were read. The band across the top holds four for six seconds and drops the rest; this does not. |
| **Open the configuration file** Â· **Open the log file** | With whatever the system uses for them. |
| **About** | The version, the graphics adapter being drawn on, whether this build can develop a raw file, and both file paths with a button that copies them. |

## The right button

About thirty surfaces answer the second button, and each of them says so: the
same small chevron on hover, and the same four words at the end of its hover
text. Every menu opens on the *press* rather than on the release, and the last
row of every one of them is the settings page that holds the same decision —
nothing here is reachable only by right-click. `Shift + F10` opens the menu of
whatever has the keyboard, which is the only keyboard route there is: egui
cannot read the dedicated Menu key at all.

**A row that a key also does names the key**, on the right of it and in the
weak colour, the way a desktop menu has always done — *Keep* `p`, *Move this
photograph to the bin* `delete`, *Copy the picture* `Ctrl + C`. The name is
read off the binding rather than written into the row, so a key you rebind is
the key the menu names from that moment on, and a command you have given two
keys names both. A row whose key is not read where you are standing names
none: *Open* says `Enter` in the contact sheet, where `Enter` opens the cell
under the cursor, and says nothing on the strip beside a photograph, where it
does not.

Every menu also opens by saying what was clicked: the kind of thing and which
one of them — *Keyword* **Tatras**, *Rating* **3/5**, *Setting* **Thumbnails
per row**, *Photograph* **DSC0142.jpg**, or *24 photographs* where a selection
is what the verbs are about. The menu is drawn over the thing it belongs to,
often a glyph a few pixels wide with a neighbour that looks much like it, and
*Show only these* means something different on each of the three badges in the
status bar.

**A panel answers it anywhere in itself.** The menu bar, the filter bar, the
metadata panel, the stars and keywords panel, the history, the strip of
thumbnails, the status bar and the performance readout each carry one menu over
the whole of themselves: it says which panel was clicked, offers *Hide this
panel* and *Bind a key to showing and hiding it*, and ends on the settings page
that governs it. Anything in the panel with a menu of its own answers first — a
history row still offers *Do only this again*, a keyword still offers *Show
only this* — and the panel's menu is what the heading, the separators, the gap
beside a short row and the empty half below the last one answer with. Which is
what changed: a button that works in some of a panel and not the rest is worse
than one that never works, because it teaches you the panel has no menu. The
status bar is the one panel with no *Hide this panel*, deliberately: it is
where the photograph's name, its marks and the magnification are.

`menus.settings_rows` turns the settings rows off, which leaves the verbs, your
own entries and the copy group. It is the whole of the configurability offered
for the built-in rows, and the reason there is no menu editor.

Right-clicking a photograph, a cell or a thumbnail on the strip offers what can
be done to it: zoom, compare, turn, move to the bin, copy the path, copy the
picture, and show it in the file manager. **Zoom** holds fit, actual pixels and
fill, which are three ways of saying one thing and take one row between them —
the shape the turns already had, and what made room for **Show** at the foot of
the list. **Copy the picture** puts the file's own pixels
on the clipboard, decoded at full size and turned the right way up, on a thread
of its own so a sixty megapixel raw does not stop the window. With part of the
photograph marked out it copies that part, cut from the same full size decode.

A cell and a thumbnail also offer **Move…** and **Copy to…**, which open the
panel of numbered destinations — the nearest thing here to a cut, in one
gesture rather than two, and without a cut left hanging when the paste never
comes. There is no clipboard cut: none of the three platforms agrees on how to
say "these files, to be moved", and the clipboard this program uses carries
text and pixels and nothing else. The photograph's own menu does not carry the
two, because its list is full; both have a key.

**Compare** asked about a set is *Compare 4 photographs side by side*, and pins
exactly those — which is how a run picked out on the strip is looked at
together. Asked about one photograph it means what it always did: this one and
the ones beside it. The panel holds eight, and a larger set is trimmed to the
first eight rather than refused.

**Each pane carries keep and throw out.** Photographs side by side are a
question about which of them, so every pane has the two verbs over the picture:
green for kept, red for thrown out, filled when the mark is on and clicked
again to take it off. A left click on a pane makes it the one being looked at,
and the second button offers the same two — about the pane the button came down
on, not about whichever pane the keys were on.

**A comparison of the picked-out photographs is a place to work.** While one is
up, a rating, a flag, a label and a keyword from the panel are about the pane
being looked at rather than about the whole set, which is what makes it possible
to tag one of five and throw out another without closing the comparison. The
arrow keys move between the panes and stop at either end — going further means
putting the set down first — while a comparison pinned from this photograph and
its neighbours still tries the folder against the frame in front. A photograph
thrown out leaves the set and so the panel, whichever way it was thrown out,
and the focus moves on to the next pane first. `Ctrl` and a click on the one
being looked at takes it out of the set as readily as any other, and then
nothing is current: no pane is outlined and a mark has nothing to land on until
a click or an arrow says which one.

**A comparison says that it is one.** The whole panel is outlined in
`image_view.comparison_colour` and named in its top right corner — *Comparing 4
picked out*, or *Comparing 2* for the kind the compare key pins — with the
sentence explaining it on the hover and a cross beside it that goes back to the
ordinary view. Four photographs side by side otherwise look exactly like four
photographs side by side, and the only way out used to be a key. A comparison
made from the picked-out photographs follows them: picking another out or
putting one back changes what is shown, and putting them all back ends it. One
pinned from this photograph and its neighbours stays as it was pinned.

**Put them all back** is on a cell's menu and a thumbnail's whenever there is a
set to put back, and on `Ctrl + D` wherever the photographs are — the other
half of `Ctrl + A`, and reachable from the strip rather than from the contact
sheet alone.

**A marked area answers the second button for itself**, because a menu is drawn
over the thing it belongs to and inside a marking that thing is the marking:
*Zoom to it*, *Copy the marked area*, *Clear the marking*.

**Turn** is the one row that opens a second level, and the only one in the
program: clockwise, anticlockwise, upside down, mirror left to right, mirror top
to bottom. Five ways of saying one verb, which as five rows would have taken
nearly half the menu. Against the right edge of the screen the second level
opens on the other side of the first rather than over it.

Whatever `image_view.context_menu` and `grid_view.context_menu` hold is appended
under a separator, in the order it is written, unchanged.

The words in the status bar are doors as well. **Flattened**, **Watching**,
**Filling**, **Advancing**, **Comparing** and **RAW+JPEG** each say what they
mean and carry the verb that turns them off — and **Advancing** and **RAW+JPEG**
are the only place in the running program those two settings are visible at all.
**Show the photograph as it is**, on the word for whichever mask is painted,
takes the mask off; it used to cycle, which from the clipping mask meant
turning focus peaking on instead.

**So are the two figures under the histogram.** *Blown 3.4%* and *Crushed 0.2%*
are the numbers a screen cannot show you — a monitor renders 250 and 255 as the
same white — and clicking either paints the mask that marks exactly those
pixels, red at the top of the range and blue at the bottom. One mask rather
than a half each: a photograph is looked at from both ends at once. The second
button carries the same verb written out, worded for the state the mask is in,
and the keys behind it.

## The mouse

Ten settings, in the `mouse` section of the configuration and on the *Keys
and mouse* page. The number of gestures is fixed and small; what opens up is
what each one means.

| Gesture | The photograph | The contact sheet |
|---------|----------------|-------------------|
| Wheel | One photograph, or zoom, or move it, or nothing — `mouse.wheel` | Scrolls the sheet |
| Shift + wheel | Ten photographs | Ten rows |
| Ctrl + wheel | `mouse.ctrl_wheel`, which ships as zoom, by `zoom_step` a notch | Thumbnails per row |
| Alt + wheel | Moves the photograph sideways, or magnifies by `zoom_fine_step` where the notch already magnifies — `image_view.fine_modifier` | — |
| One click | Zooms to a marked area, or clears it; nothing otherwise | Picks the photograph out |
| Two clicks | `mouse.double_click`, which ships as fit ↔ actual pixels | Opens the photograph |
| Left drag | Moves the photograph, or marks part of it out where there is nowhere to move it to — `mouse.mark_area` | Picks out everything it crosses |
| Alt + drag | Moves it a quarter as far, for placing a detail — `image_view.pan_fine_drag` | — |
| Middle drag | Moves the photograph, always | Scrolls the sheet |
| Right button | The menu, on the press | The menu, on the press |
| Thumb buttons | Previous and next — `mouse.back`, `mouse.forward` | Previous and next |
| A file dropped on the window | Opens its folder, on that file | The same |

**Wheel down is forward**, in both views. It used to be wheel up in the image
view and wheel down in the contact sheet, so the same movement of the wrist
meant "later" in one and "earlier" in the other. `mouse.wheel_reversed` turns
both round.

**One notch does one job.** A notch used to move to the next photograph *and*
shove the one that had just arrived, which showed whenever the arriving
photograph had any slack.

**Which button moves the photograph** is `mouse.drag`, and it ships as the left
one. It used to be every button, so whether a right drag opened the menu or
moved the picture was decided by six points of travel and eight tenths of a
second, neither of which is on screen anywhere. `any` puts the old behaviour
back. Whatever it says, the wheel pressed and dragged always moves the
photograph, so a photograph that fits the window is not a dead surface.

**Where a left drag would move nothing, it marks part of the photograph out**
instead — see [marking out part of a photograph](#marking-out-part-of-a-photograph).
`mouse.mark_area` decides when: `when_it_fits`, `always` or `never`.

**The double click, the middle button and the thumb buttons** hold the name of
a command — `next`, `fullscreen`, `exit`, `delete` and about thirty others,
listed in the control. `nothing` is a legal value and is what the middle button
ships as. The thumb buttons fire when the button goes down and have no
double-click meaning: a viewer that waits to see whether a side-button click is
a double makes walking a folder feel slow and still moves one frame.

**A slider's handle does not have to keep up with the pointer.** Two hundred
points of rail carrying four thousand values gives twenty of them to every
point the pointer moves, and no hand places a pointer to the point.
`mouse.slider_travel` says how far the pointer goes to cross the whole rail, as
a multiple of the rail's own length: at `1` the handle is under the pointer, as
it always used to be, and at the `3` it ships as the hand moves three times as
far and places the handle three times as precisely. Taking hold of the
handle moves nothing — the drag sets out from the value it is already on — while
pressing the rail somewhere else still puts the handle there, so the far end of
a range stays one gesture away whatever the setting says. When the pointer runs out of screen with rail still to cover it is
put back on the other side and carries on, so a long drag is not cut short by
the edge of the monitor. Every slider in the program reads the one value, and
the second button on any of them offers the five distances and the page that
owns them.

**A click in the contact sheet picks a photograph out; two clicks open it.** A
plain click used to leave the sheet altogether, which contradicted the cursor,
the selection, `Ctrl`-click, `Shift`-click, `Space` and `Enter` all at once, and
the only way back was `Backspace`. `grid_view.click_opens` puts the old
behaviour back. Dragging across the sheet picks out everything the band
crosses, and the middle button drags the sheet about.

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

Created with the defaults on first run, at:

| Platform | Where |
|---|---|
| Linux | `~/.config/avis-imgv/config.json` |
| Windows | `%APPDATA%\avis-imgv\avis-imgv\config\config.json` |
| macOS | `~/Library/Application Support/com.avis-imgv.avis-imgv/config.json` |

**Help ▸ Open the configuration file** opens it, and **Help ▸ About** shows the
path on this machine with a button that copies it. A fully populated example is
in `examples/config.json`, generated from the defaults; valid key and modifier
names are in `examples/keys.txt`.

A section the viewer cannot make sense of costs that section and nothing else,
and a file that was only partly understood is never written back over — so one
misplaced comma cannot quietly replace everything you had configured with the
defaults.

You may annotate it. `//` to the end of a line and `/* */` are taken out before
the document is parsed. They are not kept: a save writes JSON, so a comment
survives until the viewer next writes the file and no longer.

A save is a merge, not a replacement. A key this build has never heard of is
kept and written back where it was, so a file shared between two builds does not
lose the newer one's settings to the older one. And a save that would write over
a file edited since the viewer read it is refused: it says so, and offers to read
the file again or to keep what is on screen.

The file is written beside itself and renamed over the original, so an
interrupted write leaves the old one intact rather than half of a new one.

The file carries a `version`. When a *default* moves — a key that used to mean
one thing and now means another — a file still holding the old binding would
leave two commands fighting over it, with the loser doing nothing and saying
nothing. So the viewer brings such a file forward on the way in, says in the
corner what it moved, and writes it back once. It only ever touches a setting
that still holds the old default: anything you have actually chosen is yours,
including choosing the old binding back.

### Cache

These are the knobs that decide how far ahead of you the viewer runs.

| Key | Meaning | Default |
|-----|---------|---------|
| `ram_budget_mb` | Ceiling on decoded pixels held in RAM, shared by both views. An eighth goes to thumbnails. | 4096 |
| `gpu_budget_mb` | Ceiling on what the two caches hold on the adapter, mip chains included. The counts beside it bound how *many* textures stay resident, which is not the same thing: two hundred thumbnails and two hundred sixty-megapixel photographs are the same number and a thousandfold difference. | 1024 |
| `decode_threads` | Decode workers. `0` picks one per core, less one for the UI, capped at 8 — past which they saturate memory bandwidth rather than adding throughput. | 0 |
| `previews_resident` | Camera thumbnails kept on the GPU to stand in for images still decoding, and how far either side of the cursor their files are read. `0` turns the preview tier off. | 16 |
| `full_resolution_neighbours` | How far either side of the image on screen to also decode at full resolution, ready to be zoomed into. Each one is a whole decoded photograph in memory. `0` turns that off. | 1 |
| `upload_budget_ms` | How long a frame may spend moving decoded images onto the GPU. | 8 |

A screen sized copy costs `width Ã height Ã 4` bytes at the size of your
monitor: about 11 MB for a 24 megapixel photograph on a 1080p screen, so the
default budget holds a couple of hundred of them. A quarter of the budget is
set aside for the full resolution copies, which are 96 MB each.

### General

| Key | Meaning | Default |
|-----|---------|---------|
| `output_icc_profile` | Display profile to convert into | `srgb` |
| `restore_session` | Open where the last run left off: the window's size and place, the folder that was open, and which photograph was being looked at in each folder visited lately. A path named on the command line always wins. | true |
| `text_scaling` | Interface text scale | 1.25 |
| `metadata_tags` | Tags shown in the side panel, in order | File Name, Date/Time Original, Camera Model Name, Lens Model, Focal Length, Aperture, Shutter Speed, ISO, Image Size, File Size, Color Space, Directory |
| `sc_delete` | Send the picture on screen to the bin | `Delete` |
| `sc_delete_permanently` | Delete it outright, after asking | `Shift + Delete` |
| `sc_fullscreen` | Fill the screen, and give it back | `F11` |
| `sc_filter` | Show or hide the filter bar | `F3` |
| `sc_suspend_filter` | Set the rules aside without forgetting them | `\` |
| `sc_stacks` | Stack the folder into its runs of frames, or put every frame back | `Ctrl + G` |
| `sc_turn_left`, `sc_turn_right` | Turn the photograph a quarter, written to the sidecar and never to the file | `[`, `]` |
| `sc_toggle_stack` | Open the run under the cursor, or fold it up again | `E` |
| `sc_standing_back` `sc_standing_forward` | Walk the frames of a folded run without opening it | `,` `.` |
| `sc_previous_stack` `sc_next_stack` | Step to the run before or after this one | `Ctrl + ←` `Ctrl + →` |

### Image view

| Key | Meaning | Default |
|-----|---------|---------|
| `nr_loaded_images` | Images decoded either side of the one on screen. Trimmed to what the RAM budget can hold, so the default is deliberately more than any budget will grant. | 512 |
| `gpu_resident_images` | Images kept as GPU textures | 8 |
| `max_image_edge` | Cap on the longest edge of a decoded image. `0` means as large as the GPU allows. Unrelated to the screen sized copy, which is worked out from your monitor and needs no setting. | 0 |
| `nr_images_shown` | Images displayed side by side | 1 |
| `comparison_colour` | The colour a pinned comparison outlines the panel and names itself in | `#E2BA78` |
| `sc_compare` | Pin this picture and the next side by side, sharing one zoom and one pan | `N` |
| `sc_drop_pane` | Take the focused photograph out of a comparison. A binding rather than a bare key, because `/` on the Slovak, German and French layouts is Shift and a digit | `/` |
| `sc_go_to` | Put the cursor in the "go to" box, which could be reached by clicking and by nothing else | `Ctrl + J` |
| `sc_zoom_to_area` | Magnify until the marked area fills the window | `Enter` |
| `marked_area_dim` | How far the rest of the photograph is darkened while part of it is marked out, out of a hundred. `0` leaves it alone, which is what somebody judging an exposure against its surroundings wants. | 45 |
| `should_wait` | Wait for the next image to finish decoding before advancing to it | true |
| `frame_size_relative_to_image` | White frame width, as a fraction of the shortest side | 0.2 |
| `keep_zoom` | Carry the magnification to the next photograph, whatever it opens at and whatever it was left at. The green and red magnifying glass in the status bar is the same switch. | false |
| `keep_pan` | Carry where in the photograph you are looking to the next one, so the same corner of every frame comes up. The hand beside the magnifying glass. | false |
| `opening` | What a photograph is drawn at on the frame it first appears: `fit`, `fill`, `width`, `height` or `percent`. Whatever the photograph was last left at wins over it, and `Ctrl + M` moves it round the five. | `fit` |
| `opening_percent` | What `percent` above means, against the photograph's own pixels. Kept whatever the choice beside it says. | 100 |
| `enlarge_to_fit` | Enlarge a photograph smaller than the window to fill it. What needs it is a raw file's embedded copy: some DNGs carry a 256 pixel preview and nothing else. | true |
| `zoom_out_past_fit` | Let the zoom go out past fitting the window, leaving a border on all four sides | false |
| `zoom_step` | How much one press of the zoom keys, or one notch of a wheel that zooms, magnifies by | 1.25 |
| `zoom_fine_step` | The same with the fine modifier held | 1.05 |
| `pan_step` | How far one press of a pan key moves, in screen pixels | 40 |
| `pan_speed` | How fast a held pan key moves, in screenfuls a second | 1.5 |
| `pan_glide_delay` | How long a pan key is held before it starts to glide, in seconds. `0` glides from the first frame | 0.25 |
| `fine_modifier` | The modifier that means finer, on the keys and on the mouse alike: `alt`, `ctrl` or `shift`. Written `pan_fine_modifier` in files from before it reached the mouse, and still read under that name | `alt` |
| `pan_fine_step` | How far one press moves with that modifier held, in screen pixels | 1 |
| `pan_fine_speed` | How fast a held pan key moves with it held, in screenfuls a second | 0.15 |
| `pan_fine_drag` | How much of the pointer's travel a drag moves the photograph by with it held. The drag without it is one for one | 0.25 |
| `name_format` | Status bar name. `$(...#Tag#...)` fragments disappear when the tag is missing. Ex: `$(#File Name#)$( • Æ#Aperture#)$( • #Shutter Speed#)$( • #ISO# ISO)` → `DSCF6114.JPG • Æ5.6 • 1/500 • 200 ISO` | as above |

### Grid view

| Key | Meaning | Default |
|-----|---------|---------|
| `images_per_row` | Thumbnails per row | 5 |
| `cell_aspect` | How wide a cell's picture is against its height. 1.5 is the three-to-two most cameras shoot; 1.0 brings back the square cells, which for a folder of landscape photographs left about forty-four per cent of the sheet drawn in grey. | 1.5 |
| `preloaded_rows` | Off-screen rows decoded in each direction | 1 |
| `thumbnail_resolution` | Longest edge of a decoded thumbnail | 512 |
| `gpu_resident_thumbnails` | Thumbnails kept as GPU textures | 256 |
| `sc_cycle_badges` | Cycles what is drawn under each thumbnail: nothing, the marks, or the marks and the name | `Ctrl + I` |
| `filmstrip_height` | How tall the strip of thumbnails under the image view is, in points. The thumbnails are as large as the strip allows, so this is how big they are as much as it is how tall it is; dragging the strip's top edge writes it here. | 96 |
| `selection_colour` | The colour a picked-out photograph is washed and ticked in, in the sheet and in the strip alike | `#7EA8E0` |
| `moving_on_clears_the_selection` | Whether going to a photograph that is not picked out lets go of everything that is | true |
| `sc_select` | Picks the photograph under the cursor out, or puts it back | `Space` |
| `sc_select_all` | Picks out everything on show, or puts it all back | `Ctrl + A` |
| `sc_select_none` | Puts every picked-out photograph back. Read wherever the photographs are, not in the contact sheet alone | `Ctrl + D` |

### Mouse

| Key | Meaning | Default |
|-----|---------|---------|
| `wheel` | What one notch over the photograph does: `next_or_previous`, `zoom`, `pan` or `nothing` | `next_or_previous` |
| `wheel_reversed` | Turns the wheel round in both views | false |
| `ctrl_wheel` | The same four, with Ctrl held | `zoom` |
| `drag` | Which button moves the photograph: `left`, `middle`, `right` or `any` | `left` |
| `mark_area` | When a left drag marks part of the photograph out instead: `when_it_fits`, `always` or `never` | `when_it_fits` |
| `double_click` | The command two clicks on the photograph run | `fit_or_actual` |
| `middle` | The command the wheel pressed runs | `nothing` |
| `back`, `forward` | The commands the thumb buttons run | `previous`, `next` |
| `slider_travel` | How far the pointer moves to cross a slider, as a multiple of the rail's length | 3 |

An older file with `image_view.scroll_navigation` in it has that key moved into
`mouse.wheel` on the first launch — `true` becomes `next_or_previous` and
`false` becomes `pan`, which is what the wheel was actually doing — and the
viewer says so when it does.

### Raw

| Key | Meaning | Default |
|-----|---------|---------|
| `source` | `"preview"` shows the JPEG the camera embedded; `"develop"` demosaics the sensor data with LibRaw | `preview` |
| `quality` | Demosaic effort: `"fast"` is bilinear, `"balanced"` is PPG, `"best"` is AHD | `balanced` |
| `camera_white_balance` | Use the white balance the camera recorded. Without it colours come out noticeably wrong. | true |
| `auto_brighten` | Stretch the histogram to use the whole range | true |
| `highlight_mode` | 0 clips blown highlights, 1 leaves them unclipped, 2 blends, 3 and up rebuild | 0 |
| `pair_with_jpeg` | Which half of a raw+JPEG pair is browsed: `"jpeg"`, `"raw"`, or `"off"` to browse both | `jpeg` |

`source: "develop"` needs a build with `--features libraw`; without it the
viewer logs that it is showing previews instead.

#### Raw and JPEG shot together

A camera set to raw+JPEG writes two files of the same frame. Browsing both
means walking the shoot twice, rating everything twice, and — worse — letting
the two copies disagree: reject the JPEG, keep the raw, and what survives the
cull is the opposite of what was decided.

So one of them is browsed and the other follows it. Everything else acts on
both: a rating, a flag, a colour label, a keyword, a move, a copy, a deletion.
Each keeps its own sidecar, so the marks are readable by whatever opens either
file next. The status bar says `RAW+JPEG` when the photograph on screen is a
pair, because what happens next is about to happen to two files.

Files are paired by the name the camera gave them — same folder, same stem, one
of them raw and one not — which is the convention every camera follows. Two
files that are both pictures, `a.jpg` beside `a.png`, are two photographs: a
group without a raw in it is not a pair, and hiding one of them would be a way
to lose a picture.

The folder jobs — bulk rename, capture-time shift, grouping — still see every
file, because a rename that renamed the JPEG and left the raw behind would
break the pairing it depends on.

### Cull

| Key | Meaning | Default |
|-----|---------|---------|
| `destinations` | Folders one keystroke can send a photograph to, in the order the digits reach them. A relative path is taken against the open folder. | Selects, To edit |
| `rejected_folder` | What the folder for the frames that are not staying is called | `_Rejected` |
| `bin` | What the delete key means: `system` for the platform's bin, `folder` for one of the viewer's own that can be opened and looked in | `system` |
| `bin_folder` | Where that folder is. An absolute path, or nothing for one beside the viewer's own files. One bin rather than one per shoot | null |
| `ask_to_empty_the_bin` | Whether closing the viewer with something still in that bin asks about emptying it first | `true` |
| `confirm` | Which of the reversible things ask first: `bin_several`, `empty_rejects`, `undo_several` | all on |
| `sc_move` `sc_copy` | Open the panel that asks where | `Alt + M` `Alt + C` |
| `sc_reject_folder` | Move into the rejected folder | `Shift + X` |
| `sc_put_back` | Take it out of the viewer's own bin and put it back where it came from | `Ctrl + B` |

### History

| Key | Meaning | Default |
|-----|---------|---------|
| `remember` | How many of the things you have done are kept, or nought for all of them | `0` |
| `undoes` | Which kinds one press of undo comes to rest on: `view`, `selection`, `settings`, `content` | all on |
| `merge_within_ms` | How close two nudges have to be to count as one line. Nought lists every notch | `500` |
| `panel_visible` | Whether the panel is up. Written whichever way it is changed | `false` |
| `panel_width` | How wide the history panel is | `260` |
| `sc_undo` | Take back the last thing done | `Ctrl + Z` |
| `sc_redo` | Do it again | `Ctrl + Y` |
| `sc_panel` | Show or hide the history panel | `Ctrl + H` |

### Tags

| Key | Meaning | Default |
|-----|---------|---------|
| `categories` | The tags you always want to hand, grouped. Searching matches category names as well as tag names. A tag may be a path: `Places|Slovakia|Tatras`. | Status, Subject |
| `catalog_file` | A keyword list exported from another program, read at startup and added to the categories above. Indentation makes the hierarchy; a relative name is taken against the configuration file. | null |
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
| Ctrl + Z | Take back the last thing done |
| Ctrl + Y | Do it again |
| Ctrl + H | The list of everything done this run |
| F3 | Show or hide the filter bar |
| \ | Show everything, without forgetting the rules |
| F11 | Fullscreen |
| Alt + Q | Exit |
| F1 | Toggle the menu |
| Ctrl + L | Navigation bar |
| T | Directory tree |
| Ctrl + F | Flatten (read files from all sub directories) |
| Ctrl + Shift + W | Watch the directory for files appearing, changing or going |
| ? | The keys, for whatever is on screen |
| Ctrl + T | The strip of thumbnails under the photograph |
| Ctrl + G | Stack the folder into its runs of frames |
| I | Toggle the side panel: metadata and cache occupancy |
| K | Toggle the rating and tagging panel |
| 0 – 5 | Set the star rating of the open image |
| P / X / U | Keep it, throw it out, or take either mark off |
| 6 – 9, Ctrl + 9 | Colour label: red, yellow, green, blue, purple |
| Ctrl + Shift + A | Move to the next picture after every mark |
| Delete | Send the picture on screen to the bin, sidecar and all. Standing in the viewer's own bin it means for good, and asks |
| Shift + Delete | Delete it outright, after asking |
| Ctrl + B | Put it back where it came from, out of the viewer's own bin
| F10 | Toggle frame timings |

### Image view

| Key | Action |
|-----|--------|
| Arrow keys / Scroll | Next or previous |
| Home / End | First or last picture on show |
| Page Up / Page Down | Ten at a time |
| F | Fit the image to the screen |
| M | Fill the screen |
| Ctrl + M | What every photograph opens at: fitted, filling, full width, full height, a magnification |
| Ctrl + R | Keep the magnification from one photograph to the next |
| Ctrl + Shift + R | Keep where in the photograph you are |
| H / V | Fit horizontal / vertical |
| Alt + 1 | 100% magnification |
| R | Put this picture where the last one was left |
| Space | Zoom step |
| + / - | Zoom in or out |
| Alt + + / Alt + - | The same, five per cent a press |
| Ctrl + Scroll | Zoom |
| W A S D | Pan: one step a press, gliding while the key is held |
| Alt + W A S D | The same, a pixel a press |
| Drag | Pan |
| G | Toggle the white frame |
| O | Move what it says about itself round the corners, and off |
| C | Mark what has clipped, then what is in focus, then nothing |
| Ctrl + / Ctrl - | More or fewer images side by side, or panes while comparing |
| [ / ] | Turn it a quarter, written to the sidecar and never to the file |
| N | Compare this picture with the next |
| Tab | Which pane the keys are about |
| / | Drop that pane; the survivors re-tile — `sc_drop_pane` |
| Escape | Leave the comparison |
| Ctrl + J | Put the cursor in the "go to" box in the status bar |

Panning is two gestures on one key. A press moves the picture a fixed step —
forty screen pixels, `pan_step` — and holding the key past a quarter of a
second glides it at `pan_speed` screenfuls a second, which is the shape of a
keyboard's own repeat and the reason the shortest press anybody can make is
worth exactly one step rather than however many frames a finger stayed down.
Alt held with a pan key swaps both figures for `pan_fine_step` and
`pan_fine_speed`: a pixel a press by default, for putting an eyelash in the
middle of the window at 400%. `fine_modifier` moves that to Ctrl or Shift,
and the viewer says so at startup if the modifier and a pan key are a binding
somewhere else — which is why watching the folder is `Ctrl + Shift + W` rather
than `Ctrl + W`. Alt with the zoom keys is the same idea: `Alt + +` and
`Alt + -` magnify by `zoom_fine_step`, five per cent against the ordinary
key's twenty-five, for arriving at a framing rather than crossing a range.

The mouse reads the same modifier. Held while the photograph is dragged, the
picture takes `pan_fine_drag` of what the pointer travelled — a quarter of it
by default, which is what makes a detail placeable by hand at 400%, where an
ordinary drag overshoots everything it aims at. Held with a wheel notch that
magnifies, the notch is worth `zoom_fine_step` rather than `zoom_step`: with
the wheel as it ships that gesture is `Ctrl + Alt` and the wheel. A bare Alt
and the wheel still pans sideways, as it always did, except where the wheel
itself has been set to zoom — there a finer version of the gesture is worth
more than a second way to pan.

A notch of the wheel magnifies by `zoom_step`, the same figure one press of
the zoom keys takes, whether the wheel is set to zoom or Ctrl is held with it.
A trackpad, which reports one stroke as a great many small movements, is
counted in notches rather than in movements, so the same stroke covers about
what the same movement of a wheel covers.

Zooming keeps the point under the pointer, so magnifying an eye near the edge
of the frame brings the eye closer rather than pushing it off screen. The keys
that are about the panel rather than about a point in the picture — fit, fill,
fit horizontal, fit vertical — hold the middle instead.

`100%` means one image pixel to one **screen** pixel, counted in the pixels the
screen actually has rather than in the points a window at 125% scaling is laid
out in. The readout beside the slider says the same number, and the slider runs
logarithmically up to 1600%: it used to run from a tenth to ten times the
*fitted* size, which on a twenty-four megapixel photograph could not reach
actual size at all.

**What a photograph opens at is a setting.** Fitted, which is what a viewer
usually does — or filling the window, or exactly as wide or as tall as it, or at
a magnification you name: `image_view.opening_percent` is a hundred by default,
which is the magnification focus is judged at and the one worth having when a
shoot is being gone through for sharpness, and anything from 1 to 1600 is a
number somebody's photographs might want. The width is what a folder of
panoramas wants; the height is what a folder of portraits wants.

`Ctrl + M` moves round the five and says which in the status bar, leaving the
photograph on screen exactly as it is — `F` and `M` are the two that mean *do it
to this one now*. A photograph that was zoomed and left comes back where it was
left rather than opening again.

**Two toggles in the status bar carry the view from one photograph to the
next.** A magnifying glass and a hand, green when they are on and red when they
are not, beside the zoom readout: with the first on, every photograph arrives at
the magnification the last one was at, and with the second on it arrives showing
the same part of itself. That is how a burst is gone through at a hundred per
cent — the same eye in every frame — and it is why they are a click away rather
than a page in the settings window. They override what a photograph opens at,
and where each photograph was last left, for as long as they are on; turning
them off puts every remembered place back in charge, none of them lost. `R`
still does the same thing once.

**Zooming out stops at fitting.** Past it the photograph has a border on all
four sides and there is nothing more to see, so every notch spent getting there
is a notch spent getting back. The rail ends where the zoom does — its left end
is whatever percentage this photograph fits the window at, not one per cent —
and `image_view.zoom_out_past_fit` is there for whoever wants the border.

**Magnifying holds whatever is under the pointer**, so an eye near the edge of
the frame stays where it was instead of being pushed off screen by the very
gesture aimed at it. Fitting and filling hold the middle of the panel instead,
because they are about the panel rather than about a point in the picture — and
so does the rail in the status bar, whose pointer is on the rail and not on the
photograph.

Zoom and pan belong to the image, not to the window: leaving a photograph
half way into a corner and coming back to it later finds it exactly there.

**The strip of thumbnails under the photograph says what is on screen and what
is picked out.** `Ctrl + T` puts it up; dragging its top edge makes it taller
and the thumbnails grow with it. A white border marks the photograph the keys
are about and a fainter one marks the others beside it, so a comparison of four
reads on the strip as four. Everything picked out is washed and ticked in
`grid_view.selection_colour`, the same mark the contact sheet draws, and the
white border sits over the wash so the one being looked at is never in doubt.

A plain click opens a photograph, `Ctrl` and a click picks one out or puts it
back, and `Shift` and a click picks out the run between what was clicked and
the nearest thing already picked — which is not quite the sheet's shift-click,
where the run comes from wherever the last one started. Neither modifier
changes the photograph on screen: they are for building a set. The photograph
being looked at is always part of that set, so picking out a second frame
brings the first with it, and unpicking the way back to one puts the set down
again. Going to a photograph that is not picked out lets the set go, the way a
plain click in any list of files does; going to one that *is* picked out keeps
it, which is what makes clicking through a picked-out run work.
`grid_view.moving_on_clears_the_selection` turns the first half of that off for
anyone who picks a set out in the sheet and then walks through it.

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
| Ctrl + I | Cycle what the cells say: nothing, the marks, the marks and the caption |
| Ctrl + G | Stack the folder into its runs of frames, or put every frame back |
| E | Open the run under the cursor, or fold it up again |
| , / . | Walk the frames of a folded run without opening it |
| Ctrl + ← / Ctrl + → | Step to the run before or after this one |
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

No shell is involved: the command line is split into arguments first — single
quotes group words, so `bash -c 'a && b'` is three arguments — and the
placeholders are then filled *inside* each argument. A substituted path is
therefore always exactly one argument, whatever the file is called. Keep the
commands simple; for anything involved, call a script and pass it the path.

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
