# Plan

A review of avis-imgv against what it says it is for — browsing, selecting,
sorting and tagging photographs, quickly — and what to do about the distance
between the two.

## How this was arrived at

The whole of `src/` was read, subsystem by subsystem, and every finding was put
to a second reader whose job was to refute it: 122 findings survived the first
pass and 117 the second. Alongside that, the release build was run on Windows
against four folders — thirty-eight mixed files including a CR3 and four DNGs,
two thousand and thirty images, a folder of deliberately broken files, and a
folder shot RAW+JPEG — and every mode was opened and used. Photo Mechanic,
FastRawViewer, Lightroom Classic, Capture One, digiKam, darktable, XnView MP,
FastStone, IrfanView, nomacs, qimgv, Geeqie, nsxiv, Aftershoot, Narrative
Select, Optyx and Excire were read up on from their own documentation and
shortcut references, and the recurring requests were taken from Reddit,
DPReview, pixls.us, the Camera Bits forum, the Lightroom Queen forum and the
XnView newsgroup.

Where a claim below names a file and a line, it was read. Where it names a
number, it was measured.

## What has been done since

The review below is kept as it was written, because it is the argument for the
work rather than a description of it. What has since shipped, in the order it
was done — each of these was verified by running the viewer, not only by its
tests:

| | |
|---|---|
| **Stage 0** | Done. The six ways a keystroke destroyed another program's keywords, both panics, the hang on a crafted raw, the seventeen-gigabyte allocation, the CMYK garbage, the worker a malformed file could kill, the moves that overwrote their destinations, and the configuration file one bad section discarded. |
| **Stage 1** | Done. Reject and pick flags, colour labels, advance-after-marking, delete to the platform's bin, "send rejected to the bin", the contact sheet rebuilt as a triage surface, filtering and sorting in the browsing views, move and copy to destination slots, the undo journal, the compare view, and a selection every command reads. |
| **Stage 2** | Exact modifiers, natural browsing order, the raw-preview postage stamp, the side panel's runaway width, a fullscreen key, and the generation bug removal exposed. |
| **Stage 2** | Done. Exact modifiers, natural order, the raw-preview postage stamp, the side panel, zoom that holds its point, the watcher updating instead of reopening, the generation bug, a configuration version and migration, user actions taking an argument vector, RAW+JPEG pairing, and the rest of the correctness list. |
| **Still to do** | All of stages 3 and 4. |

The test suite went from 568 to 744. `--benchmark` on the same folder and the
same machine reports 57.9 images a second against the 54.6 it did before, so
none of it cost anything.

## 1. Where the viewer stands

The engine is the best thing about it, and it does what the README says. On a
folder of thirty-eight photographs on a 24-core Ryzen, `--benchmark` reports

```
Benchmark: 500 images in 9.16s — 54.6 images/s, median frame 3.73ms, slowest 93.55ms
```

and sixty arrow presses thirty milliseconds apart landed on image 61 of 2030
with nothing dropped. The metadata reader parses a JPEG in a hundred
microseconds and a raw in a hundred and forty, in process, with no subprocess
and no second read of the file. The pure parts of the code — the wrapping cache
policy, the zoom arithmetic, the rename template, the dHash similarity, the EXIF
timestamp shift, the orientation matrix — are correct and carry 568 passing
tests.

What it is not yet is a tool for *choosing* photographs. Take the four verbs in
turn.

| | Where it stands |
|---|---|
| **Browsing** | Does what it claims. Navigation is a texture swap, zoom is a UV rectangle, and a folder of two thousand keeps up. |
| **Selecting** | Barely begun. There are stars, and nothing else: no reject, no colour label, no selection set, no multi-select, no delete, no move-to-folder, and no way to see only the photographs you kept without leaving the picture behind. |
| **Sorting** | Strong — six sort keys, seven filter rules, natural ordering, bulk rename, capture-time shift, automatic grouping of bursts and brackets — but sealed inside three modes that draw no photographs. The browsing views cannot sort or filter at all, and they browse in byte order rather than the natural order those modes use. |
| **Tagging** | Works, for one photograph at a time, and has three verified ways of destroying keywords another program wrote. |

The gap is not in the engine. It is that the viewer can put a photographer's
shoot in front of them faster than anything else on the machine, and then gives
them almost nothing to *do* with it.

## 2. What is broken

### 2.1 Six ways a photograph's metadata is lost

These come first because they are silent, they are not hypothetical, and what
they destroy is the user's own work.

| # | What happens | Where |
|---|---|---|
| 1 | **A rating pressed before the decode lands erases every keyword on the file.** `AnnotationStore::edit` calls `entries.entry(path).or_default()`, so an image nobody has read yet is edited as though it were unrated and untagged. The queued save then rewrites the sidecar with the new star and no `dc:subject` at all. The fabricated entry is cached, so the panel goes on reporting "No tags on this image" for the rest of the session and the loss is never visible. For a file the decoder cannot read this is not a race but a certainty. | `annotations/mod.rs:155`, `app/tagging.rs:22,80` |
| 2 | **A sidecar larger than about a megabyte is written back truncated.** `xmp::edit` gives up after `MAX_EVENTS` (100 000) XML events, and the partial buffer is returned as though it were the finished document — `written` is still true — and written over the original. | `metadata/xmp/write.rs:48` |
| 3 | **A sidecar that fails to parse is replaced wholesale.** Anything the reader cannot make sense of — a develop history, another tool's namespaces, a file that is not UTF-8 — is discarded and a minimal document written in its place. | `metadata/xmp/write.rs:28` |
| 4 | **Every sidecar write is a plain `fs::write` over the original.** No temporary file, no rename. A crash, a full disk or a card pulled mid-write leaves a truncated file where a rating and a develop history used to be. `organize/timeshift.rs` already does this properly; the annotation writer does not. | `annotations/sidecar.rs:63` |
| 5 | **Renaming a JPEG steals the raw's sidecar.** `sidecar::candidates` returns both `IMG_1.jpg.xmp` and Adobe's `IMG_1.xmp`, and `move_file` moves every candidate that exists without asking whether another image in the folder shares the stem. In a folder holding `IMG_1.cr2`, `IMG_1.jpg` and one `IMG_1.xmp`, renaming the JPEG walks off with the raw's ratings. The destination is never checked either, so an existing sidecar there is overwritten and only a `tracing::warn!` records it. | `organize/files.rs:21` |
| 6 | **The second pass of a rename moves files over their destinations.** `files::move_file` calls `fs::rename` unconditionally, and `fs::rename` replaces. A file that failed or was skipped in the first pass keeps its name occupied, and the next file is renamed on top of it. A group tidy with two frames of the same name does the same thing. | `organize/rename/apply.rs:35`, `organize/gather.rs:100` |

And once anything does go wrong on disk, nobody is told: `writer::run` logs
`tracing::error!` and carries on, so a whole culling session on a read-only card
disappears while the stars stay lit on screen (`annotations/writer.rs:98`).

### 2.2 Two crashes and a hang

- **The directory tree panics.** `Tree::close_at` walks forward with an
  unbounded `loop { let next = &mut self.entries[i]; … }`. Collapsing the last
  entry in the list indexes past the end. `ui()` also computes
  `tree.entries.len() - 1`, which underflows when the list is empty, and indexes
  `tree.entries[tree.selected_index]` directly. `ui/tree.rs:100,272,298`
- **The navigator underflows** when Down is pressed and nothing matches what was
  typed. `ui/navigator.rs:64`
- **A 54 KB crafted raw hangs the preview thread for seven seconds** and
  allocates gigabytes doing it. `raw::directories` follows a file-controlled
  `SubIFDs` offset list with neither deduplication nor a cap, while every other
  walk in the metadata reader is bounded. `metadata/containers/raw.rs:92`
- **A JPEG header of about a kilobyte can demand a 17 GB allocation** and abort
  the process, because the fast path raises zune-jpeg's dimension limits without
  putting a pixel-count limit in their place. `decoder/codec.rs:52`
- **A panicking decode kills a worker for good.** The pool has no catch, so one
  malformed file removes a thread permanently and leaves its image on a spinner
  for ever. `cache/loader.rs:184`

There is a minidump on the development machine from 2026-08-28 with exception
code `0xC0000005`, and nothing to attribute it to, because the viewer writes no
log file and installs no panic hook.

### 2.3 Things that are simply wrong

| What | Where |
|---|---|
| **Alt+1 rates the photograph one star** as well as zooming to 100%. egui's `matches_logically` only checks that a shortcut's *required* modifiers are held, not that no others are, so every unmodified binding also fires with Alt or Shift down. | `config/shortcut.rs`, `app/input.rs:75` |
| **Browsing order is byte order.** `paths.sort()` gives `IMG_10, IMG_100, IMG_11, IMG_2, IMG_9`. `organize/sort.rs::natural` exists and is used by the folder modes, so the browsing order and the sorting order disagree, and the README's claim that names sort the way a person reads them is true only in the modes that draw no photographs. | `app/mod.rs:107,175`, `app/chrome.rs:41` |
| **A DNG opens as a 256×171 postage stamp at "100%".** `jpeg_candidates` only accepts blobs that begin with a JPEG start marker, so a DNG whose preview is an uncompressed RGB strip yields no preview at all, and the fallback decodes IFD0 — which is the thumbnail. The side panel reports the thumbnail's dimensions as the image size. Measured on a Canon R5 DNG whose main sub-directory is 8192×5464. | `metadata/containers/raw.rs:104`, `decoder/preview.rs:110` |
| **CMYK and YCCK JPEGs decode to garbage.** The length check meant to send them to the slow path cannot catch them: zune sizes its buffer by the *output* colour space, so `w*h*4` always matches and `RgbaImage::from_raw` always succeeds. | `decoder/codec.rs:62` |
| **"100% magnification" is wrong on every HiDPI screen**, as is the percentage readout, because neither accounts for `pixels_per_point`. | `view/image_view/canvas.rs:106` |
| **Zooming does not hold the point you were looking at.** Pan is stored in screen pixels and never rescaled when the zoom changes. | `view/image_view/canvas.rs:192` |
| **Panning is applied once per visible pane**, so with two images side by side a drag moves twice as fast and clamps against the wrong picture. | `view/image_view/canvas.rs:95` |
| **The zoom slider clamps magnification to 10× fitted**, writing its clamped value back into the viewport every frame. | `view/image_view/bottom_bar.rs` |
| **One new file in a watched folder throws away the whole decoded folder** — every texture, every remembered zoom, the scroll position and the cursor — because `handle_watcher` calls `set_images` with a freshly sorted list. | `app/chrome.rs:38` |
| **The watcher keeps watching the folder you left**, and pours its files into the new one. | `app/mod.rs:173`, `app/watcher.rs:59` |
| **`ImageStore::remove` does not bump the generation**, so decodes already in flight land one position off and a photograph is drawn under its neighbour's name, metadata and rating. | `cache/store/mod.rs:296` |
| **One missing section discards the whole configuration file**, and the settings editor then writes the defaults over it. There is no version, no migration and no warning; a config written by an older build keeps a broken keymap for ever. Observed on this machine: a config in which `sc_zoom_in` and `sc_more_images_shown` are both plain `Plus`, which makes the side-by-side view unreachable from the keyboard. | `config/mod.rs:15`, `config/load.rs:88` |
| **A user action builds its shell command by string concatenation**, so a filename can inject arguments and any path containing a space breaks. | `actions/user_action.rs:7,78` |
| **The XMP reader's keyword state machine leaks between properties**: a rating bleeds into the first keyword (`["3Macro"]`), and an empty `<dc:subject/>` makes every later `rdf:li` in the document a keyword, including Lightroom's hierarchical subjects. | `metadata/xmp/read.rs:56,71` |
| **Only the first `rdf:Description`'s rating is stripped on write**, so a multi-Description sidecar ends up with two `xmp:Rating` values and the stale one is read back. | `metadata/xmp/write.rs:116` |
| **Group-shot thumbnails ignore EXIF orientation**, so portrait frames lie on their side in the one panel that shows several at once. | `view/organize/thumbnails.rs:64` |
| **Filter rules pass files the scan has not read yet**, so a destructive job can include files the user believes they excluded. | `organize/filter.rs:87` |
| **A rendered name containing a dot is truncated at its last dot.** | `organize/rename/mod.rs:206` |
| **On Windows a case-only rename is silently dropped**, so folding an extension's case does nothing. | `organize/rename/mod.rs:113` |
| **The time shift previews from the first 512 KB but writes from the whole file**, and silently skips files whose dates it has already changed. | `organize/timeshift.rs:155` |
| **The preview tier is never colour managed**, so a wide-gamut photograph flashes over-saturated until its real decode lands. | `decoder/preview.rs:62` |
| **Downscaling ignores alpha premultiplication**, producing halos on exactly the transparent formats the viewer advertises. | `decoder/resize.rs:48` |
| **`File Size` reads 512.0 KiB for every photograph** until its decode lands, because the preview reports the size of the head read. | `decoder/preview.rs:70` |
| **Flattening follows directory symlinks with no cycle guard**, on the UI thread, and descends into hidden directories. | `crawler.rs:71` |
| **A stale EXIF rating overrides an XMP packet that explicitly says unrated.** | `metadata/mod.rs:112` |
| **Fuji RAF timestamps are shown in the panel but invisible to the capture-time shift**, so a Fuji shoot cannot be corrected. | `metadata/dates.rs:58` |
| **The metadata side panel has no maximum width.** egui sizes it to its widest child, which is the `Directory:` line; a deep path takes sixty per cent of the window and squeezes the photograph to eleven per cent. | `app/chrome.rs:52` |
| **Grid cells are square.** Rows advance by the cell width, so a 3:2 photograph leaves a band of background above and below every row — about forty-four per cent of the contact sheet is empty. | `view/grid_view/layout.rs:24` |
| **`fit` never enlarges**, so a 1620×1080 raw preview is drawn postage-stamp-small beside its 6000×4000 JPEG twin, and the status bar reads 100% for one and 41.5% for the other, with no setting to change it. | `view/image_view/canvas.rs:143` |
| **The benchmark reports twice**, once at the image limit and once on the following frame. | `app/benchmark.rs` |

### 2.4 Where the time goes

The viewer's whole argument is speed, so places where it spends time carelessly
matter more here than they would elsewhere.

| What | When it costs | Where |
|---|---|---|
| The preload window is rebuilt every frame for both stores, with an O(radius²) duplicate check and a fresh allocation. At the default radius that is a quarter of a million comparisons a frame. | every frame, always | `cache/policy.rs:50` |
| The folder-job panels deep-clone, filter and re-sort every entry in the folder every frame while the scan runs. | every frame, in a mode | `view/organize/mod.rs:184` |
| The rename preview stats the whole folder and does an O(n²) collision scan every frame. | every frame, in a mode | `organize/rename/mod.rs:188` |
| Every keyword on every visited image is collected and sorted every frame, whether or not the tag panel is open. | every frame, always | `app/tagging.rs:37` |
| The watcher clones the whole path list every frame and scans it linearly per event. | every frame, when watching | `app/chrome.rs:23` |
| The scanned-metadata map is never cleared and sits outside every budget. | grows without bound | `cache/store/mod.rs:72` |
| Full-resolution copies are decoded for images that were never reduced, at a priority that outranks the preload. | wasted decodes | `cache/store/detail.rs:153` |
| GPU residency is bounded by texture count, not by bytes, so eight full-resolution textures can exhaust a small card. | VRAM | `cache/gpu.rs:210` |

Two measurements worth recording. Opening the folder of 2 030 images settled at
1 722 MB resident and stopped; after navigating it reached 3 033 MB while the
panel reported "1350 MiB of 4096 MiB budget". The budget covers the decoded
pixel caches and nothing else — not the preview tier, not the mip chains, not
the staging buffers, not allocator slack — so a person who sets 4096 should
expect something closer to eight gigabytes.

### 2.5 What the tests do not cover

568 tests pass, and the pure functions are genuinely well covered. What is not:
`GpuCache` accounting and the store orchestrator, which are the crate's headline
subsystem, have no tests at all; 345 lines of unsafe LibRaw FFI are never linted
and their tests never see a real raw file; the XMP writer is only ever validated
against its own reader, with no golden file; there are no fuzz or property tests
for the metadata parsers; `--no-default-features` fails the project's own clippy
gate; CI is Ubuntu-only, so the documented Windows and macOS paths are never
built; and CI runs on `master` and pull requests only, so the branch the work is
happening on is never checked. `install.sh` reports failure and then reports
success, exiting 0 either way.

## 3. What is missing

### 3.1 The six verbs every other program has

**Reject, as distinct from nought stars.** Lightroom has `X`, Photo Mechanic a
tag, Bridge a label, FastRawViewer `X`, digiKam a pick label, darktable `R`. All
of them write `xmp:Rating = -1`, which is what Adobe's XMP Basic namespace
reserves for it. avis-imgv stores its rating as a `u8`, so it cannot represent
the value at all. Nought stars means "I have not looked at this yet"; a reject
means "I have looked and the answer is no", and a culling pass needs both.

**Colour labels.** `xmp:Label` is a free string with a conventional set of names
— Red, Yellow, Green, Blue, Purple — and every program in the comparison writes
it. It is the axis photographers use for *what happens next* (to retouch, to
send, sent) while stars carry *how good it is*. A Photo Mechanic reviewer
describes the motion as "keeping the finger on the keyboard for adding colour
flags".

**Delete.** There is no delete anywhere in the product. `grep -rin
"remove_file\|trash\|recycle"` over `src/` finds three hits, all of them test
cleanup. The only way to remove a photograph is to configure an external
command. Every viewer in the comparison deletes to the platform's trash on
`Delete`, and Lightroom's "Delete Rejected Photos" is the single most-cited step
of a cull.

**Move and copy to a folder on one key.** FastStone's `C`/`M` with a sticky
destination, XnView's `Alt+C`/`Alt+M`, qimgv's nine-slot panel, FastRawViewer's
`_Rejected` folder — which exists precisely because the system trash is
unavailable on cards and network shares. This is how a shoot is physically
sorted, and it costs one keystroke everywhere else.

**Auto-advance.** Rate and move on in one press. Lightroom binds it to Caps Lock
so the keyboard's own light shows the mode; Capture One added "Select Next
When → Star Rated / Color Tagged"; Photo Mechanic advances on tag by default. A
darktable user wrote a Lua script for nothing but "a shortcut that rejects the
current photo and moves to the next". Without it every frame costs two
keystrokes instead of one, which on fifteen hundred frames is fifteen hundred
extra presses. The most-complained-about detail is the corollary: when a frame
leaves the collection the cursor must stay where it is and show what is now
next, not jump to the beginning.

**Filtering while browsing.** "Show me the three stars and better" is the second
pass of every cull. avis-imgv can filter — seven rules, including by rating and
keyword — but only inside the three modes that draw no photographs.

### 3.2 What photographers ask for

- **"Just show me the picture, now."** The most common complaint about Lightroom
  is the wait between frames. This is exactly what avis-imgv already wins, and it
  is the reason the rest of this list is worth doing: the audience is people who
  have already given up on the big programs for being slow.
- **Compare two frames properly.** Linked zoom and pan across panes is the
  single most-requested missing feature in the FastRawViewer forum — "a real
  showstopper" — and its author refused for years on the grounds that the program
  "is built around single main window and simultaneous several images display
  requires deep program internals re-work". avis-imgv already draws *n* images
  side by side, already expresses zoom and pan as a UV rectangle, and already
  holds the neighbours as live textures. Linked zoom is one `Viewport` shared
  between the panes. This is the place where its architecture beats every one of
  them at their own headline feature.
- **Instruments, not verdicts.** Every AI culler draws its worst reviews for
  opaque scoring — Aftershoot's blur detection is "hit-and-miss", Adobe's
  assisted culling rejects "very sharp images (Nikon Z8)". What photographers
  trust is a measurement they can see: focus peaking, clipping blinkies, a real
  histogram, a sharpness number they can sort by. None of those need a model, and
  because avis-imgv decodes the *whole folder* rather than one file at a time, it
  can do what FastRawViewer structurally cannot: make edge energy and clipping
  percentage folder-wide sort and filter keys.
- **The RAW+JPEG pair problem.** Universally complained about, universally
  solved by treating the pair as one frame. avis-imgv shows every frame twice.
- **Sidecars, not a database.** The one thing forum users consistently praise
  about darktable and FastRawViewer and distrust about Lightroom is that the
  metadata is beside the file. avis-imgv is already on the right side of this;
  the remaining interoperability details are a day or two each and buy the
  digiKam and darktable audience outright.
- **Grouping should be a view before it is a filesystem change.** avis-imgv has
  the better half of what Lightroom's stacks and Capture One's Cull window offer
  — a detector that tells a bracket from a stack from a timelapse — but resolves
  a group by *moving files*. Photographers are wary of anything that reorganises
  originals mid-cull; the valuable version is a collapsed cell with a count badge
  and a live tolerance slider, with the folder move demoted to the explicit
  action it already is.
- **"Where did I stop?"** Nothing about a session is remembered — not the window
  size, not the folder, not the image. Every program in the comparison remembers
  at least the window.
- **A cheat sheet.** darktable pays this debt with one key; avis-imgv already has
  the prerequisite that most programs lack — every binding in one table with a
  sentence of prose in `config/bindings.rs` — and none of the discoverability
  that table makes free.

### 3.3 How it compares

"◕" means present but weaker than the others.

| | avis-imgv | Photo Mechanic | FastRawViewer | Lightroom | darktable | XnView MP | nomacs / qimgv |
|---|---|---|---|---|---|---|---|
| Instant next frame | ● | ● | ● | ○ | ○ | ◕ | ◕ |
| Whole folder resident | ● | ○ | ○ | ○ | ○ | ○ | ○ |
| Star rating | ● | ● | ● | ● | ● | ● | ○ |
| Reject flag | ○ | ● | ● | ● | ● | ○ | ○ |
| Colour labels | ○ | ● | ● | ● | ● | ● | ○ |
| Keywords | ● | ● | ◕ | ● | ● | ● | ○ |
| Hierarchical keywords | ○ | ● | ○ | ● | ● | ◕ | ○ |
| Auto-advance | ○ | ● | ● | ● | ● | ○ | ○ |
| Filter while browsing | ○ | ● | ● | ● | ● | ● | ○ |
| Multi-select and bulk apply | ○ | ● | ● | ● | ● | ● | ○ |
| Delete to trash | ○ | ● | ● | ● | ● | ● | ● |
| Move / copy to folder on a key | ○ | ● | ● | ● | ○ | ● | ◕ |
| Compare with locked zoom | ○ | ● | ● | ● | ● | ○ | ○ |
| Histogram | ○ | ● | ● | ● | ● | ● | ◕ |
| Clipping warning | ○ | ● | ● | ● | ● | ○ | ○ |
| Focus aid | ○ | ○ | ● | ○ | ● | ○ | ○ |
| Burst / bracket grouping | ● | ○ | ○ | ◕ | ◕ | ○ | ○ |
| Bulk rename with templates | ● | ● | ○ | ● | ● | ● | ○ |
| Capture-time shift | ● | ● | ○ | ● | ● | ● | ○ |
| Undo of file operations | ○ | ◕ | ● | ◕ | ◕ | ◕ | ○ |
| Session restore | ○ | ● | ● | ● | ● | ● | ● |
| Raw development | ◕ | ○ | ◕ | ● | ● | ○ | ○ |
| Free and local | ● | ○ | ○ | ○ | ● | ● | ● |

The shape of the table is the argument. avis-imgv already wins the two rows
nobody else wins, and loses almost every row in the middle — and the middle rows
are the cull.

## 4. The plan

Five stages, in the order they should be done. Each is a commit or a small run
of them, each leaves the viewer working, and each says what finishes it.

### Stage 0 — Stop losing photographs

Nothing else matters while a keystroke can destroy a file's keywords. All of
this is repair; none of it changes what the viewer looks like.

1. **`AnnotationStore::edit` must never invent an entry.** Seed from
   `sidecar::read` on a miss instead of `or_default()`, so an edit is never built
   on an `Xmp` that never saw the disk.
2. **Never write a document the writer did not finish.** `xmp::edit` returns
   nothing when it exhausts its event budget or when the reader errors, and
   `update` then leaves the file alone and reports it.
3. **Never replace a sidecar the reader could not parse.** Same route: an
   unparseable or non-UTF-8 document is left where it is and the failure is
   reported.
4. **Write sidecars atomically** — temporary file in the same directory, then
   rename over the original, the way `organize/timeshift.rs` already does.
5. **Strip the rating and the subject from every `rdf:Description`**, not only
   from the first.
6. **Fix the XMP reader's state machine** so a rating cannot bleed into a keyword
   and an empty `dc:subject` cannot swallow the rest of the document.
7. **No move may clobber.** `organize::files::move_file` refuses a destination
   that exists, for the photograph and for its sidecar, and returns the problem
   rather than logging it. The rename's collision pass counts only the plans that
   will actually move; the group tidy checks a whole group before moving any of
   it.
8. **A sidecar in the Adobe form is only followed when no other file shares its
   stem**, so renaming a JPEG cannot take the raw's ratings with it.
9. **Report failures.** The writer counts what failed and the status bar says so.
10. **Fix the two panics** — the directory tree's unbounded walk and its
    unchecked indexing, and the navigator's underflow.
11. **Bound the raw directory walk**: deduplicate `SubIFDs` offsets, skip ones
    already seen, and cap the total.
12. **Bound the fast JPEG path** by pixel count, and send anything that is not
    YCbCr or RGB to the slow path so CMYK decodes correctly.
13. **Catch a panicking decode** so a worker survives a malformed file.
14. **Load the configuration section by section**, so one bad section costs one
    section and not the whole file, and never overwrite a file that failed to
    parse.

*Finished when:* rating an image the decoder cannot read leaves its keywords
alone; a two-megabyte sidecar round-trips; a rename onto an occupied name is
refused and reported; the tree can be collapsed at its last row; and there is a
test for each of those.

### Stage 1 — Make it a culling tool

This is the stage that changes what the program is.

1. **Three axes instead of one.**
   - *Reject and pick.* `X` rejects, `P` picks, `U` unflags, and pressing the key
     of the state already set returns to unflagged. Reject is written as
     `xmp:Rating = -1`, which is what Bridge, Lightroom, FastRawViewer and
     darktable all read; setting a star clears a reject and rejecting clears the
     stars. Pick is mirrored into `digiKam:PickLabel` so digiKam round-trips.
     `Xmp::rating` becomes an `i8`.
   - *Colour label.* One label per image, mutually exclusive with itself and
     orthogonal to the other two. `6` red, `7` yellow, `8` green, `9` blue,
     `Alt+9` purple, and the key of the label already set clears it. Written as
     `xmp:Label` holding the canonical English colour name whatever the interface
     ever says, and read case-insensitively against the Lightroom, Bridge and
     Review-Status name sets, mapping anything unrecognised to a neutral chip
     rather than throwing it away.
   - Both become sort keys, filter rules and badges.
2. **Auto-advance.** Applying a rating, a flag, a label or a tag optionally moves
   to the next photograph in the current order. A persistent toggle with its
   state named in the status bar, `Shift` with any marking key for a one-shot
   advance whatever the toggle says, and a per-binding flag in the configuration
   so stars can advance while tags do not. And the corollary, which matters as
   much: when a photograph leaves the collection the cursor stays at the same
   index and shows what is now next.
3. **A selection.** The grid gets a keyboard cursor, arrow keys, `Enter` to open,
   `Space` to toggle selection, `Shift`+arrows to extend, `Ctrl+A`. Every
   annotation command applies to the selection when there is one and to the
   current photograph when there is not, so tagging two hundred frames is one
   operation.
4. **Triage badges.** Stars along the bottom edge of each grid cell, the colour
   label as a corner swatch, the pick or reject glyph opposite it with rejected
   cells dimmed and struck, the file name under it, and a border on the current
   cell. `Ctrl+I` cycles the badge set from none to full. Cells the scan has not
   reached draw nothing rather than drawing nought stars.
5. **Delete, properly.** `Delete` sends the photograph — or the selection — to
   the platform's trash together with its sidecar and its raw or JPEG partner, as
   one transaction. `Shift+Delete` deletes outright behind a confirmation that
   names the count. Never `fs::remove_file` on a photograph. A menu entry
   collects every rejected file in the folder and hands the lot to it.
6. **Move and copy to destination slots.** `Alt+M` and `Alt+C` open a small panel
   of nine configurable destinations plus the last one used; the digit keys act
   immediately, `Enter` repeats the last, pressing the key twice repeats without
   showing the panel. `Shift+X` moves to a `_Rejected` subfolder of the current
   folder, created if absent — the idiom that exists because a card or a network
   share has no trash. Sidecars and partners move as one unit, the collection is
   updated in place, and an inverse is pushed onto the journal.
7. **An undo journal.** One bounded journal of inverse operations covering the
   bulk rename, the group tidy, the capture-time shift, move, copy, trash and
   every annotation change. `Ctrl+Z` undoes and says what it is about to do
   first, because a silent bulk undo is as frightening as none. This is the
   safety net that makes it reasonable to bind a destructive operation to one
   key at all, and it is the reason it sits in this stage rather than a later
   one.
8. **Filter and sort where the photographs are.** The `Filter` and `Sort` already
   in `organize/` are lifted out of the folder modes into a bar the image and
   grid views share. It re-evaluates as marks are applied, so rejecting a frame
   with "hide rejected" on removes it at once with the cursor staying put; `\`
   suspends and restores it without discarding it; named presets are recalled
   from a key, with `Unrated`, `Picks`, `Two stars and up` and `Rejected` shipped
   to teach the pattern. The rating rule gains comparison operators. This is the
   largest single change in the plan and the one that makes the sorting work
   already done reachable.
9. **A compare view.** `N` opens the selection — or, with nothing selected and
   the cursor inside a detected group, that group — as two or four panes sharing
   one viewport: 100% on an eye in one pane puts the same eye at the same
   magnification in all of them. One pane holds the keyboard focus and is drawn
   unmistakably; `Tab` moves it; every marking key acts on the focused pane only.
   `/` drops the focused pane and the survivors re-tile larger, `Enter` promotes
   it and leaves, `Esc` leaves without promoting. Holding `Shift` moves one pane
   alone. This replaces `nr_images_shown` as what those keys are for.

*Finished when:* a folder of fifteen hundred can be culled to a keep list, the
rejects sent to the trash and the keepers moved to a folder, without leaving the
keyboard or the photograph — and when the second pass, "show me the three stars
and better", is one keystroke and undoing a wrong one is another.

### Stage 2 — Make it correct

1. **Exact modifiers.** A binding without Alt does not fire with Alt held. Shift
   stays permissive, because `+` needs it on some layouts.
2. **Natural order everywhere.** The browsing collection sorts with
   `organize::sort::natural`, which is what the folder modes already use.
3. **RAW+JPEG pairs are one frame.** A setting decides which of the two is shown;
   ratings, tags, renames, moves and deletes act on both.
4. **Raw previews.** Accept uncompressed RGB previews as well as JPEG ones, take
   the image size from the main sub-directory rather than from IFD0, and say on
   the status bar when what is on screen is a preview smaller than the
   photograph. A DNG stops being a postage stamp.
5. **Zoom that holds its point**, magnification that is right on a HiDPI screen,
   a slider that does not clamp, and panning that does not multiply by the number
   of panes. A setting for whether a photograph smaller than the window is
   enlarged to fit.
6. **The side panel gets a maximum width** and its long values elide.
7. **The contact sheet stops wasting half the screen**: cells take the
   photograph's own aspect ratio.
8. **The watcher updates instead of reopening.** A new file is inserted at its
   sorted position, a deleted one removed, and nothing else disturbed. It stops
   watching the folder that was left and drops the events queued for it.
9. **`ImageStore::remove` bumps the generation** so in-flight decodes cannot land
   one position off.
10. **A version and a migration for the configuration**, and a warning at startup
    naming any two bindings that share a key.
11. **User actions take an argument vector**, not a concatenated string.
12. **Orientation in the group panel**, colour management in the preview tier,
    premultiplied downscaling, the real file size in the side panel, a cycle
    guard and a hidden-file rule in the crawler, and the Fuji dates fed to the
    capture-time shift.
13. **Fullscreen on `F11`**, `Home` and `End`, `Page Up` and `Page Down` by a
    screenful, and a "go to image" box.

### Stage 3 — Make it as fast as it says it is

1. **Memoise the preload window.** Recompute only when the cursor, the count or
   the radius changes, and replace the O(radius²) duplicate check with the
   arithmetic that makes it unnecessary.
2. **Cache the folder-job plan.** Recompute when the entries, the filter, the
   sort or the template change, not every frame.
3. **Stop cloning the path list every frame** in the watcher, and index it by
   path.
4. **Compute the known tags when the tag panel opens**, not every frame.
5. **Bound the scanned-metadata map** and put it inside the budget.
6. **Bound GPU residency by bytes**, not by texture count.
7. **Only decode a full-resolution copy for photographs that were actually
   reduced**, and behind the preload rather than in front of it.
8. **Report honest memory**: the readout adds the preview tier and the mip chains
   and says what the process is actually holding.

### Stage 4 — The instruments, and the rest of what is expected

1. **A histogram and clipping statistics**, accumulated on the decode workers
   that already touch every pixel of every file in the folder — which makes
   "percentage of blown highlights" a folder-wide sort and filter key, something
   no single-file viewer can offer.
2. **A sharpness score** — Tenengrad over the thumbnail the folder scan already
   decodes — shown in the grid, sortable, and used to rank the frames inside a
   group so the sharpest of a burst is offered first.
3. **Focus peaking and clipping overlays**, as a paint callback over the texture
   already resident.
4. **Virtual stacks.** The group detector runs over the open folder as a view:
   one cell with a count badge and a kind glyph, `S` to expand or collapse, `,`
   and `.` to change which frame it stands for, a live tolerance slider, and
   `Ctrl`+arrows to step group to group with the status bar always saying
   `series3 · frame 4 of 17 · group 6 of 41`. Nothing is written; the folder move
   stays the explicit action it already is.
5. **A cheat sheet** on `?`, generated from `config/bindings.rs`, filtered to the
   current mode, showing the user's own keys rather than the documentation's.
6. **An information overlay** on the photograph itself, and caption lines under
   the thumbnails, both from the same template grammar the status bar uses.
7. **One template resolver** shared by the bulk rename, the captions, the
   overlay, the destination paths and a sort key, with the EXIF and date
   vocabulary widened to what Photo Mechanic's variables cover.
8. **Hierarchical keywords**: `lr:hierarchicalSubject` in an `rdf:Bag` beside
   `dc:subject`, a tag tree in the panel, and the catalog readable from a text
   file.
9. **Session state**: window geometry, the last folder, the photograph that was
   open, and the per-folder position.
10. **A filmstrip** along the bottom of the image view, from the thumbnail store
    that already exists.
11. **Log to a file** beside the configuration, and a panic hook that writes to
    it, so a crash leaves something behind.
12. **CI on Windows and macOS**, on every branch, with clippy over
    `--no-default-features` and `--features libraw`, and a golden-file test for
    the XMP writer.

## 5. Deliberately not doing

- **Raw development as an editing feature.** The `develop` path exists for people
  who want the sensor data; competing with darktable is not what this is.
- **A database-backed catalogue.** The sidecar is the point, and it is what the
  audience says it wants. Camera Bits withdrew Photo Mechanic Plus from sale
  because its catalogue rested on a dependency that stopped being supported;
  that is the risk in miniature. A cross-folder index is worth having eventually,
  but as a cache that can be deleted, never as the truth.
- **Face detection, recognition, and AI aesthetic or expression scoring.** They
  need a model, and the transparent approximations — sharpness, exposure
  statistics, similarity — are what photographers say they actually trust.
- **Cloud sync, FTP, an upload queue, print layouts, a map view.** Somebody
  else's program.
- **Destructive editing.** Crop, straighten and resize belong in the editor the
  user action already opens.
- **Archive browsing, tabs for several folders, and drag-out to other
  applications.** Each collides with something the viewer is built on — the
  folder-centric collection, the single residency budget, and winit's lack of a
  drag source respectively.
