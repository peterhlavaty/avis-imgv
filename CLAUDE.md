# avis-imgv

A GPU accelerated image viewer for photographers culling a shoot. It decodes a
whole folder into RAM on background threads, keeps the images around the cursor
resident on the GPU, and draws a textured quad. Rust, egui/wgpu, no runtime
dependencies beyond the optional decoders.

## How to work here

**Work alone.** Do not ask which of two reasonable options to take — take the
better one and say which in the commit message. Ask only when proceeding either
way would destroy something the user cannot get back, or would make the work
useless if the guess were wrong. A question that costs a round trip to save
yourself a decision is not worth its price.

**Spend context carefully.** Read with `sed -n` and `grep` rather than pulling
whole files in; this repository is forty thousand lines of Rust and nearly a
megabyte of planning notes. Delegate deep reading to subagents that write their
findings to the scratchpad and return a summary. Compact when the session grows
long rather than letting it end mid-task.

**Commit in whole units.** A commit is one change that leaves the viewer
working: the code, its tests, the README section it changes and a changelog
entry, together. Roughly ten files and a few hundred lines is the shape of one
here. Push when the branch has something worth having; do not stack six
unpushed commits. Never commit to `master` — branch first.

**Verify before claiming.** Run the tests. If a change is visible, run the
viewer and look at it. A commit message that says something was checked means
somebody checked it.

## Speed is the feature

This is a viewer whose entire reason for existing is that the next photograph
is already there. Treat a regression in frame time or in images a second as a
bug of the same rank as a crash.

- Nothing in the draw path may wait on I/O, on a decoder, or on a lock a worker
  holds. If a frame can block, it will, on somebody's folder of 60 megapixel
  raws.
- Work belongs on a worker thread, ordered by distance from the cursor, and
  must be droppable when the user navigates away from it.
- Per-image cost is multiplied by the size of a folder. An allocation per frame
  is an allocation per photograph per frame.
- Measure rather than assert: `--benchmark` walks a folder and reports images a
  second; `cargo run --release --example bench_decode -- <path>...` times each
  stage of the decode pipeline, so an optimisation can be aimed at the stage
  that costs something.
- Debug builds optimise dependencies at level 3 and this crate at level 1 —
  decoding is unusable otherwise. Do not undo that.

## Test everything

Every behaviour gets a test, in a `#[cfg(test)] mod tests` at the foot of the
file that holds it. There are 180 such modules, and they are the reason changes
to XMP writing and cache eviction can be made at all.

- Anything that touches a file on disk gets a test that a failure leaves the
  original where it was. Losing a photograph's keywords is the worst thing this
  program can do.
- Arithmetic on indices gets its ends tested — wrapping round the collection,
  the empty collection, the single item.
- XMP round-trips against the golden files in `tests/golden/`.
- Pure logic is kept separate from egui so it can be tested without a window:
  `view/image_view/zoom.rs`, `cache/policy.rs` and `app/input.rs` are the
  pattern to follow.
- Fix a bug, write the test that would have caught it, in the same commit.

## The checks that must pass

CI runs these on Linux, Windows and macOS, and again without the default
features, with `libraw`, and with `jxl`. Run at least the first three before
committing:

```
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo clippy --all-targets --no-default-features -- -D warnings
cargo build --release
```

A clippy warning is a failure. There is no rustfmt configuration: default
style.

**A run of work ends with a release build, not every commit in it.** `cargo
build --release` takes three minutes, and paying it thirty times down a branch
buys nothing: the viewer is not being looked at between commits, because there
is still something in development behind it. So the release build and the run
belong at the *end of the job* — when the work is being handed over to somebody
who will actually open a folder in it. Debug builds optimise this crate at level
1: too slow to judge anything about speed by, and a different program where it
matters — `debug_assert!` is compiled out and integer overflow wraps rather than
panicking. A change that has only ever been built in debug has not been built,
and the last commit of a run is where that is settled.

`cargo test` and `cargo clippy --all-targets` still run at every commit. They
take seconds, and they are what keeps each commit a whole unit.

## The shape of the code

`src/lib.rs` is the map and is kept accurate. One module per concern, each a
directory with a `mod.rs` holding the type and its wiring, and siblings holding
one piece of the work each.

| Path | What lives there |
|------|------------------|
| `src/crawler.rs` | finding the images in a folder |
| `src/decoder/` | bytes to RGBA8: `codec`, `color` (lcms2), `resize`, `histogram`, `overlays`, `raw/` (LibRaw FFI) |
| `src/metadata/` | EXIF and XMP read from the same buffer the decode uses; `containers/` per file format, `xmp/` for sidecars |
| `src/cache/` | what lives in RAM and what lives on the GPU; `policy.rs` decides, `store/` holds, `loader.rs` and `preview.rs` are the threads |
| `src/view/` | drawing: `image_view/`, `grid_view/`, `stacks.rs`, `narrow.rs`, `organize/`, `wheel.rs` |
| `src/app/` | wiring, input, modes, panels, the file watcher, `--benchmark`, `gestures.rs` |
| `src/annotations/` | stars, flags, labels and tags, written to XMP sidecars |
| `src/organize/` | work on the folder rather than the image: renaming, timeshift, grouping, `bin/` for the viewer's own bin |
| `src/config/` | the configuration file, its defaults, migrations between versions, and `registry/` — one row per field, which the settings window, the search and the key editor are all views over |
| `src/session.rs` | what is remembered between runs: window, folder, position |
| `src/ui/` | shared widgets, the notice bar, the key binding clash check, `deck/` for the cards everything is drawn on, `settings/`, `surface.rs` for the menus, `panel.rs` for what every panel does about them, `slider/` for the rails, `progress.rs` |

**Keep files short.** Aim for 300 lines of code. The median file is 302 lines
in total and 193 of code, so the aim is being kept on the measure that matters
and missed on the raw one — count code, not tests, or the rule punishes the
files that are best covered. Past that, split along the seam the file already
has — a `mod.rs` plus siblings — rather than growing it. Fifty-five files are
over it on code alone, 112 counting their tests, and they are candidates for
splitting when they are next touched; they are not licence to write more.

**Keep folders small.** Around fifteen files to a directory. Past that the
directory is two concerns and should be two directories.

**Folders follow the functionality**, not the kind of file. A directory is
named for a job the program does — decoding, caching, organising the folder —
and holds the logic, the drawing and the tests for that job together. There is
no directory of every widget or every type: `src/ui/` is the one exception and
holds only what several concerns genuinely share. A file goes where its job
lives, even when it resembles a file somewhere else.

These rules give way to a real reason, and the reason goes in the commit
message.

**Write it once.** Before writing something that feels familiar, grep for it. If
it exists, call it; if it nearly exists, widen it until it serves both callers
rather than copying it and letting the two drift apart. Make a helper general
enough for the callers it has and no more — the registry, `ui::surface` and
`cache::policy` are each one rule with several views over it, and that is the
shape to aim for. Duplication that survives a review is a bug that will be
fixed in one place out of two.

**Refactor as part of the change, not instead of it.** When a change wants a
third copy, a fifth parameter or a flag that means "the other caller", the
shape is wrong: change the shape first, then make the change. Either in the
same commit or in the one before it, with the tests still passing at each step.
Leaving the seam for later means the next session pays for it with interest.

## Things that already have an answer

- **Deleting** goes through the `trash` crate to the platform's bin, never
  `fs::remove_file`. Culling is when people delete fastest and regret hardest.
- **The viewer's own bin is a folder and nothing else.** That is the whole
  argument for it: it opens like any other folder, so an hour of culling can be
  looked through before any of it is really gone, and it reaches a card the
  platform's bin cannot. The one thing a folder cannot do is say where a file
  belongs, so `organize/bin/ledger.rs` writes that inside it — and the note is
  *append-mostly*: a row whose file has gone is invisible to everything that
  reads it and live again the moment the file returns. That is what lets undo,
  redo and "put back" agree without one of them ever telling the others
  anything, and it is why `Step::Interred` writes the note going forward and
  leaves it alone going back, and why putting something back is recorded as a
  plain `Step::Moved`. The name a photograph is filed under is reserved for
  good, because reusing it would put a different picture behind a memory.
  `remove_dir_all` runs only against a folder holding that note.
- **Moving a photograph may leave the filesystem.** `files::move_file` answers
  `EXDEV`/`ERROR_NOT_SAME_DEVICE` with a copy and only lets go of the source
  once the copy has arrived. A card is not the drive the bin is on.
- **Writing a sidecar** is atomic: temporary file in the same directory, then
  rename over the original. A document the writer did not finish is never
  written, and a document the reader could not parse is never replaced.
- **A configuration that was only partly understood** is never written back —
  see `Config::partial`. Sections load independently, so one bad section costs
  one section rather than the file.
- **A new key binding** must pass `ui::keys::clashes`, which warns at startup.
  Check what the key already does before taking it.
- **A shortcut is a list of chords, and the list may be empty.** `Shortcut`
  holds `Vec<Chord>`; `Chord` is what the whole of it used to be. That is the
  one seam multiplicity was threaded through, and it is why the sixteen
  `shortcut::consume` sites, the registry's five key-shaped `Access` variants
  and every one of the ninety table rows were untouched: `consume` tries each
  chord. The file keeps the first chord where it always was and writes the rest
  under `also` only when there are any, so a configuration nobody has added a
  key to is byte-identical and an older build still reads it. A blank chord is
  dropped on the way in, so "no key" is the empty list rather than a blank
  string four readers each had to remember to check — and it is reachable, by
  taking the last key away, which is what let `Delete` and `Backspace` become
  ordinary bindable keys. Two chords are the same when they are the same
  *press*: compared on the built `KeyboardShortcut`, because `Esc` and `Escape`
  are one key and a comparison minding the spelling let a clash through, with
  the written name used only to tell two unreadable names apart, since every
  typo builds the same sentinel. `pan.rs` is the one reader that bypasses
  `consume` — it reads held keys — and it asks every chord.
- **The list of keys is an index; a window holds one command.**
  `ui::keys::list` is the ninety rows with a sentence each, and clicking a key
  opens `ui::keys::one` on that command alone: its keys, a cross beside each,
  and a button that takes the next press. Editing in the list does not work
  once a command can have several keys — a row is one button, so there is
  nothing for a second key to be added *to* — and the armed row was invisible
  whenever the list was scrolled elsewhere. Every route in is the same one:
  `App::arm_key`, from the eleven `surface::bind_a_key` rows, the settings
  page's key button, the settings row menu and the cheat sheet. The window is
  drawn *after* the list so `set_modal_layer` leaves it in front, and at
  `Order::Foreground` rather than among the windows — an `Area` keeps the place
  it had in its order and egui raises one only when it is *new*, so the second
  time it was opened it was drawn underneath the list, which with a list nine
  hundred pixels tall and opaque reads as the button having stopped working.
- **A clash is asked about a key, not about a command.** `keys::clash_on` takes
  a chord, and `clash` is the walk over a command's chords. A command clear on
  its first key and taken on its second collides on every press of the second,
  and a warning naming only the command leaves a person looking at two keys not
  knowing which to move. `clashes` says a pair once *per key they share*.
- **The direction the modules depend in is a test.** `tests/layers.rs` walks
  the module graph transitively and asserts that nothing in `DECIDES` reaches
  the toolkit — not directly, and not through somebody who does. Transitive
  because the two edges that spoiled it were invisible to a grep: `view::narrow`
  imported a struct out of a drawing file, and a test in `metadata::xmp` named a
  type in `view`. It is a test and says so: a row added to `DRAWS` in the same
  commit defeats it. What makes it worth having is that the list is short, is in
  the diff, and shrinks. A settings enum belongs in `config`, not in the view
  that happens to draw it — `Corner` and `Opening` are in `config::kinds` for
  that reason, with the painting that acts on them left behind.

- **Anything the user should know** goes to `notices.say`, not only to the log.
- **Keywords are hierarchical**: written to `lr:hierarchicalSubject` *and*
  `dc:subject`, never only one.
- **A setting is a row in the registry.** `src/config/registry/table/` holds one
  file per section of the configuration file, and every view over the settings —
  the window, the search, the key editor, the cheat sheet, the checks, the
  export — is a filter over that one table. A field added to `Config` without a
  row fails the build. The row carries the page it is drawn on, a sentence, the
  aliases somebody might search for, an `Access` reaching the field, an `Effect`
  saying when the change takes effect and a `Scope` saying where a key is read.
- **If two photographers would want two answers, it is a setting.** A delay, a
  threshold, a colour, a default, a count — a number chosen in the code is a
  decision taken on somebody's behalf, and the whole cost of not taking it is a
  row in `src/config/registry/table/` and a field with a sensible default. What
  stays a constant is what has one right answer.
- **A setting is reached from the thing it governs, and from every other place
  it makes sense.** The settings window is an index, not the door: the panel a
  width sizes, the badge a toggle draws, the bar a figure fills are each a route
  to that row — `ui::surface::with_menu` marks the surface, `more_settings`
  ends its menu on the page that owns it, and `bind_a_key` opens the keyboard
  editor with the row armed. A person who can see a value changes it from
  where they are standing.
- **The history watches the state; nothing tells it anything.** Undo, redo and
  the history panel are `src/history/`, and not one of the five dispatchers
  calls into it. `App::watch_history` asks once at the foot of the frame what
  the program looks like and compares that with the frame before — so every
  route in is covered by construction, including ones not written yet. Adding a
  sixth dispatcher needs no work here; adding a piece of *state* worth taking
  back means a field on `Watched`/`Snapshot`, a `Change` variant and an arm in
  `app/history/restore.rs`, and nothing else. The comparison must stay
  allocation-free: `Watched` borrows, `Snapshot` owns, and a clone happens only
  on the frames something moved. The configuration is not in the snapshot —
  only the registry can compare it, and that walk costs ten microseconds a
  frame, so it runs on the frames `save_settings` has marked.
- **A deed is recorded with both halves, never as an inverse.** That is what
  makes redo possible at all, and it is why the journal never had one. A
  restore sets an absolute value and never flips a flag: "make it what it was"
  and "flip it" are the same thing only until something else flips it.
- **Nothing in the history is ever overwritten.** Going back and doing
  something else branches rather than truncating, node indices are stable for
  the life of the tree, and a row that was undone is still drawn — in italics —
  rather than removed. A limit forgets the *oldest*, re-parenting what hung off
  it.
- **A gesture is one row.** Nothing is watched while the pointer is down, and
  nudges of the same kind inside `history.merge_within_ms` fold into one,
  keeping where the gesture began. The same rule governs reading a panel's
  dragged width back (`ui::width::Dragged`): a width that moved without the
  button down is an animation or a layout pass, and writing it back is a
  feedback loop that corrupts the setting.
- **A value that is both view state and a setting is watched once.** A key
  that nudges a panel writes it back, which leaves it visible to the snapshot
  *and* to the settings look, and one press then makes two rows that undo the
  same thing. `history::watch::ALSO_IN_THE_SNAPSHOT` names them and the
  settings look steps over them; the snapshot wins, because it is the half that
  puts the panel back rather than only the file. Mirroring is two-way or it is
  broken: `remember_runtime` writes the file from the program and
  `apply_settings` writes the program from the file. "Show the strip" had only
  the first, and so did nothing at all.
- **Anything kept between runs carries a signature of what it assumed.**
  `history::persist` stores one over every file its rows mention — sidecars
  included, because undoing a mark writes a sidecar and never touches the
  picture — and refuses to load when it no longer holds, saying so.
- **Everything applies while the window is open.** A setting the caches are
  built from is applied by building them again (`ImageStore::rebuild`), on the
  frame the gesture *ends*. Exactly one field in the whole program carries the
  restart badge — `cache.decode_threads` — and a test says so. A badge means
  *your change has not taken effect*: a setting about the next launch gets
  `Effect::NextLaunch` and no badge, because using it for changes that have
  taken effect is what teaches people to ignore it.
- **A key that nudges a value writes it back.** The overlay corner, the panes,
  the columns, the badges, advance-after-marking and the filmstrip all live in
  `config.json`; only *position* belongs in the session file.
- **Everything drawn answers the second button.** A surface worth pointing at
  carries a menu of the verbs that apply to it and the settings that govern
  it — the picture, the filmstrip, a star, a keyword, a panel heading, a
  figure in the bottom bar. A surface without one is a gap to be filled, not
  a decision: the chevron and the same four words (`ui::surface::SAYS`) are
  how a person learns the button is worth pressing anywhere, and it stops
  being worth pressing the moment only some things answer. `Shift + F10` is
  the keyboard's way in, which is why a menu it can reach is a `named_menu`.
  Nothing, though, is reachable *only* by right-click.
- **Which panels are on screen is asked once and drawn twice.**
  `ui::panel::show_and_hide` is the one list — a row per entry of `EVERY_PANEL`
  that can be put away, ticked where it is up, with the key beside it — and it
  is drawn at the top level of the bar's **View** menu and behind **Show** at
  the foot of the photograph's. The photograph, because with the bar itself put
  away it is the whole window and the only surface left to ask. What is on
  screen and what each key is are published once a frame by
  `App::publish_panels`, the shape `utils::set_in_front` already has and
  for the same reason: neither drawing site holds the program's fields or its
  configuration, and both would otherwise take fourteen arguments to say one
  thing. Both routes leave `panel::Ask::Toggle` in the mailbox
  `App::take_panel_ask` already empties, which puts them in the history for
  free. The status bar is in neither list: a tick nothing can clear is worse
  than no row.
- **A panel is a surface, and it answers anywhere in itself.** `ui::panel` is
  the one definition: what a panel says for itself is five things — what it is,
  what puts it away, the key row for that, and the settings page and row its
  menu ends on — and the rows, their wording and their order are not the
  panel's to choose. A `Chrome` per panel, `EVERY_PANEL` naming all eight, and
  a test that each settings row and key row named is a row that exists. Doing
  it panel by panel does not work: most of a panel is painted rather than
  sensed, so there is no response to hang a menu on, and a click-sensing
  rectangle over the whole of one is hidden by the scroll area's drag-to-scroll
  surface if it is registered first and swallows every button in the panel if
  it is registered last. So the panel reads the press itself —
  `rect_contains_pointer` on its own rectangle, which is the one reading that
  knows about layers and egui's modal one — and `surface::menu_when` takes the
  answer. It is drawn last, after everything the panel holds, and stands down
  when one of them has taken the press: `surface::taken`, a pass number in
  egui's memory rather than a static of this program's, because a pass number
  counts from nought in every context.
- **Every menu opens on the press**, through `ui::surface`, and ends with the
  settings page that owns it. `Response::context_menu` opens on the release and
  loses the menu to a six-point drag, so it is not used.
- **A menu opens on the surface the press landed on, not on what is hovered.**
  egui empties the hover set while anything at all is being dragged, and a
  scroll area whose content has outgrown it lays a drag-to-scroll rectangle
  over the whole of itself which senses drag alone — so it counts as dragged
  from the frame *any* button goes down, the second one included, and every
  menu inside a list stopped opening on the day the list grew long enough to
  scroll. A fault that arrives with use rather than with the code, and reads as
  the button having broken. `Response::is_pointer_button_down_on` is the
  top-most click-sensing widget under the press, which is the question a menu
  is asking, and it answers no for a disabled panel or a layer under a window
  in front exactly as `hovered` did: egui strikes the sense off a widget that
  is either.
- **A menu says what it was asked about.** Every one of them opens with the
  kind of thing in the weak colour and which one of them in the strong —
  *Keyword* **Tatras**, *Rating* **3/5**, *Photograph* **DSC0142.jpg** —
  because a menu is drawn over the very thing it belongs to and its verbs are
  worded for somebody who already knows: "Show only these" is three different
  sentences depending on which of the three badges in the bottom bar was under
  the pointer. It is a `surface::Subject` and an argument of `menu`,
  `named_menu` and `with_menu`, so a new surface does not compile until it says
  what it is about, and `surface` draws it rather than the caller so that none
  of the thirty can word it differently. Always first, the way
  `more_settings` is always last.
- **A row that offers a way out of a state carries out the way out.** The key
  that reaches a state often cycles — the mask goes clipping, focus peaking,
  nothing — and the row saying *Show the photograph as it is* used to send that
  key's command, which from the clipping mask turned the other mask on. So
  `Command::NoMarks` sits beside `CycleMarks` and `StopComparing` beside
  `Compare`: a command with no key of its own, which is what a menu row means.
  And a figure that states a fact carries the verb that acts on it, on the
  *left* button as well — `Blown 3.4%` and the two glyphs in the status bar are
  buttons, and the menu carries the same verb written out for whoever has not
  worked out that a number is one.

- **A row a key also does names that key.** On the right of the row, in the
  weak colour, through `ui::keys::button`, `checkbox` or `radio` and no other
  way — thirty surfaces each choosing their own punctuation is what the two
  rows that already named one had been doing, one with brackets and one with
  two spaces. The name is rendered from the binding rather than written into
  the label, so a rebind stays correct and a command with two keys names both.
  Which key is published once a frame by `App::publish_keys` into
  `ui::keys::of`, the shape `panel::showing` and `surface::more_settings`
  already have and for the same reason: no menu in the program holds the
  configuration. The mode goes with it, because a key is only a key where it
  is *read* — `scopes_for` is the one answer, shared with the cheat sheet —
  and a menu naming a key that does nothing there is worse than one naming
  none: `Enter` opens the cell under the cursor in the contact sheet and does
  nothing on the strip. A row says which key by naming its registry path, the
  way `bind_a_key` and `Chrome` already do; an empty path is a row with no key
  to name, and a path the registry has never heard of trips a `debug_assert`
  rather than drawing nothing for ever.

- **A second level is bought with a row, never with a decision.** A menu is a
  list of `ui::menus::Row`, and three things are folded: the five turns, the
  three zooms and the eight panels — five, three and eight rows into three,
  against the twelve a menu may carry. Each fold is *one* decision with several
  answers; a second level for five *different* decisions is still wrong and is
  drawn as an inline row. egui 0.33 folds a submenu that will not fit to the far
  side of its parent rather than over it, which is what makes any of them
  affordable against a panel on the screen edge.
- **A key that moves something is a step and a glide, not a stopwatch.** The
  pan keys were read as held keys and multiplied by the frame time, so the
  smallest movement anybody could ask for was however long a finger stays on a
  key — two or three frames, whatever care is taken. `view/image_view/pan.rs`
  pays a press exactly `pan_step` and starts the glide only once the key has
  been down longer than `pan_glide_delay`, which is the shape of a keyboard's
  own repeat; the press is read off the events with `repeat: false`, because
  the platform's repeat is not a press and `key_pressed` counts it as one. A
  modifier held with it swaps both figures for the fine pair — Alt by default,
  which is also where the fine zoom keys are. A held modifier is not a binding,
  so the clash check cannot see it: `check.rs` compares the chord against every
  binding read where the photograph is, which is what moved the folder watcher
  off `Ctrl + W`, a chord anybody choosing Ctrl would otherwise have to clear
  first.
- **Anything read per frame is read on the first pass.** egui runs the frame
  again whenever something in it calls `request_discard`, and the second pass
  arrives with no events but with a clock that has moved on
  (`predicted_dt` is added to `input.time` for it), so anything that
  accumulates over time is paid twice on those frames while anything read off
  an event is not paid at all. `ctx.current_pass_index() > 0` is the guard, and
  `interaction::keyboard_panning` is where it is.
- **A resizable panel must fill the size it is given, or it snaps back.** egui
  stores a panel's size as the rectangle its *contents* came to, not the one
  the drag asked for (`PanelState { rect }` where `rect` is the frame's
  response, `egui-0.33/src/containers/panel.rs`), so a panel whose content is
  sized from the configured number reports that number back however far the
  edge was pulled — and the next frame is drawn at it. `ui.set_min_height` (or
  width) of `ui.available_*` at the top of the closure makes the two
  rectangles one. The other two halves: the size is read back through
  `ui::dragged::Dragged`, which lands on the frame the button comes up rather
  than sixty times a drag, and a size changed by anything *but* a drag needs
  one `exact_*` frame to reach the panel, because `default_*` is dead from the
  second frame on — `forced_panel_width` and `forced_filmstrip_height` are
  that flag.
- **The set of picked-out photographs is empty, or it holds the one on
  screen.** Empty is what every command already reads as "the one being looked
  at" (`App::marked_paths`), so picking a second frame out has to bring the
  first with it (`Selection::start_at`) and unpicking back to one has to put
  the set down (`Selection::settle_on`), or a command meant for two would run
  on one. Moving to a photograph that is not in the set lets the set go and
  moving to one that is keeps it — watched once a frame in
  `App::follow_the_photograph` rather than told, so all dozen routes to a
  different photograph are covered by construction, and empty-to-empty is the
  common case so an ordinary walk through a folder records nothing.
- **The sheet extends a run from an anchor; the strip extends it from the
  nearest.** Two shift-clicks, deliberately, both in `view/selection.rs`. A
  list of files is walked one run at a time and remembers where the run began;
  the strip is read with a photograph on screen, where the run wanted is the
  gap between the set and the frame just pointed at, and after two runs the
  anchor is somewhere nobody can still see. Nearest is measured in the shown
  order, never wraps round the end of the collection, and settles a tie in
  favour of the earlier frame.
- **A state the window is in has to be visible from the window.** A pinned
  comparison and four panes of the ordinary view are the same pixels, and the
  only ways out of the first were a key and a row in a menu on a figure in the
  status bar. `view/image_view/comparison.rs` outlines the panel, names the
  state in the corner, explains it on the hover and puts a cross beside it.
  The plate is drawn in an `Area` of its own rather than inside the panel,
  because the panel registers itself as one click-sensing widget covering the
  whole of itself *after* its contents — that is how a drag over the photograph
  is read — and egui hands a press to the last such widget under it, so
  anything inside the panel that wants a click is a thing the panel swallows.
- **A pane is a photograph, and the panel is one rectangle.** `layout::show`
  returns where each pane was drawn (`Shown::panes`); the icons over them, the
  click that focuses one and the menu opened on one all hit-test that list
  rather than asking which photograph the keys are about. The menu was about
  the focused photograph whichever pane the button came down on, which with
  four side by side is the wrong one three times in four.
- **While the picked-out photographs are side by side, a command is about the
  one being looked at.** The inversion of `marked_paths`' rule, and the whole
  reason a comparison is usable: rating one of five, tagging one of five,
  throwing one of five out. The set is still on the strip and closing the
  comparison puts it back in charge. Nothing being *current* — no pane focused
  — is reachable only inside a comparison, and `ImageView::focused` says so by
  construction rather than by a flag somebody has to remember to clear; it
  reaches every reader through the `None` that `active_path` already returns.
- **A glyph is only as available as the fonts actually loaded.** The program
  ships Atkinson Hyperlegible Next and falls through to egui's emoji font;
  `✕` (U+2715) is in neither and drew an empty box. `✔`, `✖` and `★` are used
  elsewhere and are known to render. The way to find out is to look at it.
- **A gesture is a second route, never the only one.** The mouse is eight fields
  in `mouse`; the ones with a single meaning hold the name of a command from
  `config::mouse::VERBS`, and a test asserts every one of them also has a key.
  The wheel is read off the `MouseWheel` event (`view::wheel`) rather than off
  `raw_scroll_delta`, because Shift and Alt are spent by egui before this crate
  sees a delta.
- **"Finer" is one modifier, and every gesture with a size reads it.**
  `image_view.fine_modifier` — Alt, and named `pan_fine_modifier` while it
  governed only the keys — is the pan keys' smaller step and speed, a drag's
  `pan_fine_drag` share of the pointer's travel, and a notch's
  `zoom_fine_step`. `pan::Pace::of` chooses the pan pair and
  `input::zoom_steps` the zoom pair, so no gesture picks its own answer and no
  caller keeps its own fallback. On the wheel it refines a notch that
  *magnifies* and nothing else: Alt is already the axis across the wheel's
  own, and walking a folder has no smaller version of itself. What a notch is
  worth is counted in *notches* — `Notch::turns`, one to a line and fifty to a
  point — because a trackpad reports one stroke as a great many movements and
  a whole step for each of them crosses the range in a frame. A Ctrl notch
  reaches egui first, whose `zoom_modifier` is Ctrl and must not be moved: it
  folds the notch into `zoom_delta` and smooths it over the frames after,
  frames carrying no event to recognise it by. The viewer answers the notch
  itself, so `wheel::Tail` latches that magnification shut from the notch
  until the first frame it comes back to exactly one. A pinch arrives as
  `Event::Zoom`, is neither folded nor smoothed, and is what is left.
- **The program opens no windows of its own; everything is a card.**
  `ui::deck` is the whole of it, and it is written to be lifted out: `Deck<C>`
  is a stack of whatever identifier the caller uses and keeps three rules — one
  card on screen, a card opened twice is *raised* rather than stacked twice,
  and taking a card off takes what was put on top of it. `deck::draw` is the
  other half: an opaque page over `ctx.available_rect()` with a bar saying
  where you are, or a plate over the rest dimmed. `app/cards.rs` is this
  program's thirteen, and nothing else knows they exist.

  A **page** is about itself and takes the window; a **question** is about the
  photographs and is a plate over them dimmed, because "send these three to the
  bin" cannot be answered by somebody who can no longer see them. A question is
  never *put* on the deck — `App::asked` derives it once a frame from the state
  that asked it, so there is no flag about a question that could disagree with
  whether there is one — and it carries no cross, because its own answers are
  the way out. `App::showing` is the question or the deck's card, in that
  order, and the two are drawn in that order too: `deck::show` raises the card
  it draws, so the one drawn last is the one being answered.

  A card that carries state of its own is reconciled once at the foot of
  `show_deck`: the deck says what is on screen and `keys::State` follows it. A
  route that arms the editor without going through the deck — the row in the
  list of every key, which is four call frames from anything holding one — is
  noticed by `keys::list` comparing before with after, the same answer as the
  mailboxes in `ui::panel` and `ui::slider`.

- **A card in front owns the mouse and the keyboard.** Whether something is up
  is decided once a frame, in `App::something_is_in_front`, and written to the
  context with `utils::set_in_front`; a card that sets and clears a flag of its
  own is a card that clears it while another still needs it. `are_inputs_muted`
  is that flag *or* a focused text field, and gates the keys; the pointer takes
  two more things. `deck::draw` and the two overlays call egui's modal layer,
  which stops the scroll areas and the focus behind them — and the few places
  that read the pointer for themselves ask `utils::is_in_front`, because
  `Response::contains_pointer` comes from a hit test that knows nothing about
  modal layers and is true wherever the card is not actually drawn. `Escape`
  takes the card in front off, and `App::was_typing` is why it takes two
  presses to do it from a search box: egui clears the focus itself before the
  program is called, so "was anything being typed into" has to be remembered
  from the frame before.

- **The menu bar is the one thing a card does not take.** It answers the mouse
  from over one, which costs a second layer: a panel lives in
  `LayerId::background()` and cannot be moved out of it, and egui decides the
  modal layer by comparing `Order`, so everything in every panel is under every
  card there is. So `panels::top_menu` lays the bar out in its panel with
  `set_invisible` — for the height, the background and the animation, which are
  the panel's to give — and draws it again from the same `rows` in an `Area` at
  `Order::Foreground`, over the rectangle the panel's contents came to. The
  panel's own rectangle is the wrong one: it takes in the margin, and the bar
  would shift by it the moment a card opened. The copy is salted, because the
  first has taken every id. What the bar is asked for then decides the deck:
  `MenuAction::goes_back_to_the_photographs` is an exhaustive match, so an
  action added later does not compile until somebody has said which it is.
- **A turn is written to the sidecar and never to the photograph.** The eight
  orientations compose (`Orientation::then`), so the camera's and the user's are
  one orientation by the time anything draws. The same rule as the ratings: this
  program does not open a photograph for writing.
- **A marking on a photograph is held in the photograph's coordinates.**
  `view/image_view/area/` keeps the rectangle normalised nought to one against
  the picture *as displayed*, which is the space `Metrics::uv` is already in.
  That is what makes it survive being zoomed to — it stays on the same eyelash
  through a zoom, a pan and a quarter turn — and it is the same two corners
  that crop the full size decode the clipboard is given, after the turn rather
  than before it. It belongs to the photograph on screen and `select` clears
  it: a rectangle over one frame means nothing over the next.
- **A new gesture takes a drag that was already doing nothing.** The left
  button marks an area only where the photograph fits the window, because there
  the canvas clamps every pan to nothing and the drag was dead. One press is
  one gesture — `Area::is_dragging` is why `interaction::dragging` returns
  false — and where two readings are defensible it is a setting
  (`mouse.mark_area`), not a decision taken in the code.
- **A zoom says what it holds still.** `input::Anchor` is an argument of
  `ZoomToPercent` rather than a decision taken in `apply`, because the same
  magnification is asked for from the keyboard, from a menu over the photograph
  and from a rail in the status bar, and only the first two happen anywhere near
  the picture. Magnifying holds the pointer; fitting and filling hold the middle
  because they are about the panel; and so does anything asked for from the bar
  (`Anchor::FROM_THE_BAR`), whose pointer is on the control and may even be *put
  back* on the other side of the window mid-drag.
- **Zooming out stops at fitting.** One floor, `zoom::floored`, applied in
  `zooming` after the change rather than inside each command, so it covers the
  wheel, the keys, the rail and the presets, and a command added later cannot
  forget it. `image_view.zoom_out_past_fit` lifts it. The rail's left end is
  then the fitted percentage rather than one per cent: a stretch of rail asking
  for something the view will refuse is worse on a fine drag than it sounds,
  because the drag carries on past the end and has to be walked all the way back
  before the handle moves again.
- **A slider's handle takes a share of what the pointer does.** Every rail in
  the program is `ui::slider::Fine`, which is drawn exactly as egui's is —
  `slider/paint.rs` is the toolkit's own code from the same style — and differs
  only in the interaction: it reads the value off how far the pointer has
  *moved*, divided by `mouse.slider_travel`. egui's `Slider` sets its value from
  `interact_pointer_pos` before it paints, with no way in for a caller to say
  where the pointer should be taken to be and no rect it can be given that is
  not also the rect it draws into, which is why the widget is written out rather
  than wrapped. A press still jumps to where it landed, so the far end of a
  range is one gesture away; the aim radius is divided by the travel too, or the
  reachable values would be the ones a bound drag already reached.
- **Taking hold of a handle moves nothing, and the ends of a rail are the ends
  of its range.** A press within `grabbing` of the handle sets out from the
  value it is on; a press elsewhere still puts the handle there. The value is
  held until the handle has *moved*, not merely until the press frame is over —
  egui runs a pass twice whenever something in it asks for another look, and the
  second pass would read the value back off the handle and round it. And
  `aimed` returns the range's own ends at the rail's ends rather than the
  roundest number near them, which on the logarithmic zoom rail was a per cent
  above fitting: a photograph a shade too large for the window with slack to pan
  into.
- **The pointer is put back when it runs out of window, and nothing records
  that it was.** `slider/drag.rs` reads a jump of more than half a window as a
  wrap and takes the width off it — phase unwrapping, not a log of what was
  asked for — which is what makes it right where the ask is ignored. Wayland
  cannot warp a cursor; there the jump never arrives, no correction is made for
  one, and the drag stops at the edge as it did before. Only while there is rail
  left to cover: at either end the pointer meeting the edge means the drag is
  over.
- **A menu drawn from inside a widget leaves its ask in a mailbox.** A rail is
  drawn in four subsystems and none of them has the configuration in hand, so
  `ui::slider::asked` is emptied once a frame by `App::take_slider_ask` and goes
  through the same `apply_settings` and `save_settings` the settings window
  uses — which is what puts it in the history for free. The same shape as the
  keyboard's ask in `surface`, and for the same reason.
- **Nothing long-running may block a frame.** The folder crawl is a `Walk`
  stepped a few milliseconds at a time; anything that takes longer than half a
  second says so at the foot of the window, with a percentage only where an
  honest total exists.

## Prose

Long-form text here — commit messages, doc comments, README, changelog,
`plan.md` — is written in one voice: British spelling, plain declarative
sentences, understatement, no marketing language. Read a recent commit message
before writing one. Doc comments say why the code is the way it is, not what
the next line does. Tables in planning documents carry a "Where" column citing
`file:line`, and every claim about the program carries a file and a line.

A user-visible change gets a dated entry at the top of `docs/changelog.md` and
its section in `README.md` updated, in the same commit.

## The roadmap

`plan.md` is the plan for what the viewer does — staged, each stage saying what
finishes it. `plan2.md` is the same treatment of the interface. Both are long;
read the stage being worked on with `sed -n`, not the whole file. When an item
in a stage is finished, that is what the commit message is about.

## Keeping this file honest

When something is settled that the next session would otherwise have to work
out again — a convention, a constraint, a decision with a reason behind it —
add it here in the same commit, in the same voice, briefly. Do not ask first.
Do not record what the code already says plainly.
