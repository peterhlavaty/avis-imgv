# Change Log

## 2026-08-30

- **The previews are colour managed too.** The camera's thumbnail — what the
  contact sheet draws, and what stands in for a photograph while it decodes —
  was drawn without any conversion at all. A camera set to Adobe RGB writes
  its preview in Adobe RGB, so a whole sheet was flat and undersaturated and
  every image visibly shifted colour the moment the real decode landed under
  it. It goes through the same conversion the photograph does, into the same
  configured display profile, and the measured throughput does not move.
- Fixed: the group panel drew every portrait frame on its side. The viewer
  turns its images with texture coordinates, which costs nothing and which
  egui's own image widget cannot do — so there the pixels are turned instead,
  once per file, into what is cached.
- Fixed: downscaling ignored the alpha channel, so the colour hiding under a
  transparent pixel — arbitrary, and in a lot of PNGs black — bled into its
  neighbours and left a dark halo around every soft edge. It resamples on
  premultiplied values now wherever the alpha is not uniform. Whether it is
  uniform is answered by the format where the format can answer it: a JPEG and
  a raw cannot be transparent, and asking the pixels instead costs a pass over
  all of them, measured at five per cent of the viewer's throughput.
- Fixed: the side panel reported the size of the part of the file that had
  been read rather than the size of the file. Only half a megabyte is ever
  read to get the metadata, so every raw file claimed to be 512 kB.
- **A folder that links back to itself is crawled once.** Testing whether
  something is a directory follows links, so a symbolic link or a Windows
  junction pointing at one of its own ancestors sent a flattened crawl round
  for ever, collecting the same photographs again at a longer path each time
  until the memory ran out. Somebody's `Pictures/latest -> .` was all it took.
- **Hidden entries are left alone.** `.thumbnails`, `.DS_Store`, and above all
  the `._IMG_1234.JPG` resource forks macOS writes beside photographs on
  non-native volumes — named like the photograph, not photographs, and opened
  as a black frame between every pair of real ones on a card that had been
  through a Mac.
- Fixed: opening a folder with no photographs in it put the viewer in the
  *home* directory, because the folder was worked out from the first
  photograph and there wasn't one. Asking to flatten an empty folder then
  crawled everything the user owns — six thousand files and a gigabyte of
  memory, here. The folder that was opened is remembered rather than derived.
- **A user action can no longer be steered by what a file is called.** The
  placeholders were substituted into the command line as a whole and the
  result split afterwards, so the file name decided how many arguments the
  program received: `holiday 1.jpg` arrived as two, and a name containing an
  apostrophe opened or closed a quoted run and could add arguments of its own
  — `a' --delete 'b.jpg` passing `--delete` to whatever was being run. Names
  come off cards, downloads and shared drives. The template is split into
  arguments first now and the placeholders filled inside each one, so a
  substituted path is exactly one argument whatever is in it.
- Fixed: a run of spaces in a configured command produced empty arguments,
  which some programs read as a file name of no characters.
- **The watcher updates the folder instead of reopening it.** A photograph
  appearing in a watched folder is inserted at its sorted position and nothing
  else moves: what is on screen stays on screen, at the zoom it was at, and
  every decoded photograph and thumbnail in the folder stays decoded. It used
  to read the folder again and hand both views a new collection — throwing all
  of that away, jumping to the newcomer and clearing the selection, once per
  frame during a tethered shoot.
- The watcher now notices files that have *gone*, too. A folder open here and
  tidied up in a file manager used to keep drawing photographs that were no
  longer there, and opening one failed with no explanation.
- Fixed: the watcher stayed on the folder it was started on. Walking away from
  a watched folder left it reporting arrivals where nobody was looking and
  nothing about the folder on screen, while the status bar said "Watching".
  It follows the open folder now, and drops the events queued for the one that
  was left.
- Fixed: losing a photograph from *below* the one being looked at stepped the
  viewer forward a frame. Losing the one being looked at still keeps the
  position and shows what is now next, which is what culling wants; losing any
  other keeps the photograph.
- Fixed a crash: taking a photograph out of the collection from the image view
  before the contact sheet had ever been opened panicked, because the marks
  the sheet draws are only read when the sheet needs them and the list was
  still empty. Both the arrival and the departure paths ask before they touch
  it now.
- **Zoom that holds its point.** Magnifying used to keep the middle of the
  *panel*, so zooming in on something near an edge pushed it further out of
  sight with every step — the one thing zoom is for. The point under the
  pointer stays under the pointer now, for the keys, the wheel and a pinch
  alike; fit, fill and the two fit-to-an-edge commands hold the middle, being
  about the panel rather than about a point in the picture.
- Fixed: `100%` was counted in layout points rather than in the pixels the
  screen has, so on a window at 125% scaling a photograph drawn one pixel for
  one pixel reported itself as 80% — and asking for 100% drew it a quarter
  larger than it claimed.
- Fixed: the zoom slider ran from a tenth to ten times the *fitted* size. On a
  twenty-four megapixel photograph in a normal window fitted is about a
  twelfth of actual size, so the slider could not reach one-for-one at all. It
  runs from 1% to 1600% of the photograph's own pixels now, logarithmically,
  in the same percentages the readout beside it shows.
- **A selection, on `Space`.** The contact sheet picks photographs out:
  `Space` toggles the one under the cursor, `Shift` with the arrow keys picks
  out everything walked over, `Ctrl + A` takes everything on show, `Ctrl` and
  `Shift` do the same with the mouse, and `Escape` puts it all down. The cells
  carry a blue wash and a tick, and the corner says how many.

  Whatever is picked out is what the next command is about — a rating, a flag,
  a colour label, a keyword clicked in the tag panel, a move, a copy, a
  deletion — so tagging two hundred frames is one keystroke rather than two
  hundred. The set is held as positions in the folder rather than as names, so
  narrowing the folder down does not throw it away.

  What a mark ends up as is decided by the first photograph in the set and
  then applied to all of them: a toggle applied one at a time would leave half
  the set flagged and half not, which is never what pressing one key over two
  hundred frames meant. Undo takes the whole thing back in one press, however
  many photographs it touched, and marking a selection never auto-advances,
  because there is nothing for "the next one" to mean.
- Deleting a selection asks first even when it is only going to the bin: the
  cost of a wrong keystroke there is a folder rather than a frame.
- **The questions can be answered from the keyboard.** The delete confirmation
  took the keyboard and then offered only buttons; `Enter` or `Y` answers it
  now and `Escape` or `N` leaves things alone, and a permanent deletion takes
  `Y` alone, because `Enter` should not be one tap from something nobody can
  undo.
- Fixed: answering one of those windows handed the keyboard back in the middle
  of the frame, and the views draw after they do, so the same key went on to
  mean whatever it means the rest of the time — pressing `Enter` to empty the
  bin also opened the photograph under the cursor. The windows consume the
  keys they answer with now.
- Changed: scrolling the contact sheet half a row moved from `Space` to
  `PageDown` (`grid_view.sc_scroll`), because `Space` is what every program
  with a contact sheet uses to pick a photograph out, and the arrows, the
  wheel and the scrollbar all scroll it already.
- **A comparison, on `N`.** Two photographs side by side sharing one zoom and
  one pan: 100% on an eye in one pane puts the same eye at the same
  magnification in the other. `Tab` moves which pane the keys are about and
  draws a border round it, the arrow keys try a different photograph against
  the one that is staying, `Ctrl + Plus` widens it to eight, `/` drops the
  focused pane and the survivors re-tile, and `Escape` leaves. Every marking
  key applies to the focused pane and to nothing else.
- Fixed: panning was applied once per pane, so two images side by side moved
  twice as fast as one and four four times — and the last pane drawn clamped
  the pan against *its* picture rather than against the one being looked at.
  One pane owns the viewport now.
- Fixed: `Tab` also moved egui's own keyboard focus into the status bar's
  "go to" field, and a text field with focus mutes every shortcut in the
  viewer. That field is reachable by clicking and by nothing else now, and
  `Escape` takes the keyboard back from wherever it has gone.
- **Somewhere else, rather than nowhere.** `Alt + M` moves the photograph and
  `Alt + C` copies it, to a panel of numbered folders the digits pick from;
  `Enter` repeats the last, and the same key twice in a row skips the panel
  entirely. `Shift + X` moves it into `_Rejected` beside it, which is what a
  card or a network share has instead of a bin. A configured destination may be
  a relative path, so `Selects` follows the shoot rather than naming one.
- **`Ctrl + Z`**, which is what makes it reasonable to bind any of that to one
  key. It covers moving, copying, sending to the bin and every mark — a rating
  pressed by mistake is one keystroke to undo — keeps the last two hundred, and
  says what it is about to do first, because a silent bulk undo is as
  frightening as none. Undoing a copy sends the copies to the bin rather than
  deleting them.
- Fixed: a window asking a question did not own the keyboard, so the key that
  answered it also did whatever it does the rest of the time.
- **Filtering and ordering where the photographs are**, on `F3`. Stars, flag,
  colour label, name, keyword and file type, and an order by name, stars,
  label or flag. `organize::Filter` has done this since the folder modes were
  written and was sealed inside three modes that draw no photographs, so
  "show me the three stars and better" meant leaving the picture behind.
  - It re-evaluates as marks change, so rejecting a frame with "Not rejected"
    on takes it out of the strip at once, and the cursor lands on its
    neighbour rather than back at the beginning.
  - `\` sets the rules aside without forgetting them.
  - The status bar says `2/27 (+2)`: where you are, how many are on show, and
    how many are held back.
  - Nothing is re-decoded: the caches keep the whole folder and the filter is
    a list of positions into it.
- New: `Home`, `End`, `Page Up` and `Page Down` in the image view.
- Fixed: a DNG opened as a postage stamp. Some of them — anything written by
  Camera Raw — embed no JPEG at all, only a 256 pixel copy stored as plain
  pixels, so the scan for embedded JPEGs found nothing and the file fell
  through to the TIFF decoder, which reads the *first* directory. That
  directory is the small copy, and the viewer then reported it as the
  photograph: `Image Size: 256x171` for a forty-five megapixel frame, drawn at
  "100%" in the middle of the screen. The uncompressed copy is read properly
  now, the size comes from the main sub-directory, a `Preview Size` line says
  what is actually on screen, and `image_view.enlarge_to_fit` fills the window
  with it rather than leaving it a stamp.
- Fixed: the metadata panel had no maximum width, so egui sized it to its
  widest line — the directory — and a deep path took sixty per cent of the
  window and squeezed the photograph into eleven per cent of it.
- Fixed: the browsing order compared bytes, so `IMG_10` came before `IMG_9` and
  the folder modes disagreed with the views people actually browse in. Both
  read names the way a person does now, and a flattened tree stays grouped by
  folder.
- New: `F11` fills the screen and gives it back. Fullscreen used to be
  reachable only by starting with `--fullscreen` or by entering the slideshow.
- The contact sheet now says what it knows. Stars, flag and colour label under
  every thumbnail, a red tint over the rejected ones, the file name when asked
  for (`Ctrl + I` cycles the three), a white outline on the photograph the
  image view is on and a blue one on the photograph the keyboard is on. The
  arrow keys walk it, `Home` and `End` jump to the ends, `Enter` opens what is
  under the cursor — and every marking key now applies to *that* photograph
  rather than to whatever the other view was last left on.
- Cells take the shape of the photographs in them rather than being square.
  A folder of landscape frames used to leave about forty-four per cent of the
  sheet drawn in grey; `grid_view.cell_aspect` sets it, and 1.0 brings the
  squares back.
- **Delete**, which a viewer for choosing photographs had no answer to at all.
  `Delete` sends the picture on screen to the platform's bin — the freedesktop
  specification on Linux, the Recycle Bin on Windows — with its sidecar, as one
  unit; `Shift + Delete` deletes outright and asks first; and
  **File → Send rejected to the bin…** collects every picture in the folder
  marked with `X` and puts the lot there behind one question. The cursor stays
  where it is rather than following the picture that has gone, so what it lands
  on is the next one.

- Three ways of saying something about a photograph rather than one, because a
  cull needs three different answers and every other program keeps them apart.
  - **Keep and reject** on `P` and `X`, `U` to take either back off, and
    pressing the key of the mark a picture already carries is the same as `U`.
    A rejection is written as `xmp:Rating="-1"`, which is what Adobe reserves
    for it and what Bridge, Lightroom, FastRawViewer and darktable all read;
    a keep is `digiKam:PickLabel`. Rejecting clears the stars and rating clears
    the rejection, because they are the same field.
  - **Colour labels** on `6` to `9` and `Ctrl + 9`, written as `xmp:Label` and
    always in English whatever the interface ever says. Read back against the
    names Bridge and Lightroom use as well, and a label from somewhere else is
    kept as it is rather than thrown away.
  - The panel shows all three, and so does the status bar, so a key pressed
    with the panel shut is not a keystroke that appears to do nothing.
- **Advance after marking**, on `Ctrl + Shift + A` or in the configuration:
  rating, flagging or labelling moves to the next picture by itself, which is
  what turns a cull into one keystroke a frame. A mode rather than a held
  modifier, because on a Slovak or German keyboard the digits are the shifted
  characters of the top row and every rating would have arrived with shift.
- Fixed: a shortcut fired whether or not the modifiers it did *not* ask for
  were held, so `Alt + 1` both magnified to 100% and put one star on the
  photograph. Alt is exclusive now, and so is shift everywhere it can be —
  everywhere but the digits and the arithmetic keys, which on a Slovak or
  German layout need shift to be typed at all.
- The viewer says so at startup when two keys clash. A configuration written by
  an older build keeps whatever it said for ever, because serde only fills in
  the keys that are *missing*: one on the author's machine had zoom-in and
  show-more-images both on plain `Plus`, which made the side-by-side view
  unreachable and said nothing about it.
- Fixed: taking a photograph out of the collection left every decode already in
  flight pointing one place along, so one could be drawn under its neighbour's
  name, metadata and rating. Deleting made that easy to reach.

## 2026-08-27

- Fixed: an image drawn smaller than the copy on the GPU went invisible, which
  showed as an empty grey window. The two resolutions the viewer keeps were
  being swapped for one another every frame for any picture no larger than the
  screen — including the previews inside raw files — and each swap freed the
  texture the frame had already been drawn with. Now it only swaps when there
  are genuinely two copies to choose between.
- Fixed: the gallery would not scroll. It was being told to jump to the open
  image on every frame rather than only when it opened, so it snapped back the
  instant it was scrolled.
- New: `R` puts the picture on screen at the zoom and position the last one was
  left at, for comparing two frames of the same thing.
- New: **Settings → Keyboard…** lists every shortcut with a sentence saying
  what it does, and lets any of them be changed. Click a key, press the one you
  want; it is written to the configuration file at once. Keys already spoken
  for within the same section are pointed out rather than refused.
- New: a **Slideshow** mode, which fills the screen and runs its own clock
  while the arrow keys still work. **Settings → Slideshow…** sets the interval
  and what happens while a picture is up: hold still, drift inwards, or travel
  across — the last fills the screen at the picture's own shape and moves along
  it, so a panorama is seen whole rather than shrunk into a strip.
- The group panel now shows the pictures, at a size set at the top of it. They
  are the thumbnails the folder sweep already decoded to compare frames by, so
  showing them costs nothing new.

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
