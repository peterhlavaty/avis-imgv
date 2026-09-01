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
file that holds it. There are around 114 such modules, and they are the reason
changes to XMP writing and cache eviction can be made at all.

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

**Every change ends with a release build.** `cargo build --release`, every time
the code has been touched, and the viewer run from that build when the change
is visible. Debug builds optimise this crate at level 1: too slow to judge
anything about speed by, and a different program where it matters —
`debug_assert!` is compiled out and integer overflow wraps rather than
panicking. A change that has only ever been built in debug has not been built.

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
| `src/organize/` | work on the folder rather than the image: renaming, timeshift, grouping |
| `src/config/` | the configuration file, its defaults, migrations between versions, and `registry/` — one row per field, which the settings window, the search and the key editor are all views over |
| `src/session.rs` | what is remembered between runs: window, folder, position |
| `src/ui/` | shared widgets, the notice bar, the key binding clash check, `settings/`, `surface.rs` for the menus, `slider/` for the rails, `progress.rs` |

**Keep files short.** Aim for 300 lines; the median here is 264. Past that,
split along the seam the file already has — a `mod.rs` plus siblings — rather
than growing it. Sixty files are over that today and are candidates for
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
- **Writing a sidecar** is atomic: temporary file in the same directory, then
  rename over the original. A document the writer did not finish is never
  written, and a document the reader could not parse is never replaced.
- **A configuration that was only partly understood** is never written back —
  see `Config::partial`. Sections load independently, so one bad section costs
  one section rather than the file.
- **A new key binding** must pass `ui::keys::clashes`, which warns at startup.
  Check what the key already does before taking it.
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
- **Every menu opens on the press**, through `ui::surface`, and ends with the
  settings page that owns it. `Response::context_menu` opens on the release and
  loses the menu to a six-point drag, so it is not used.
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
- **One menu has a second level, and it is the turns.** A menu is a list of
  `ui::menus::Row`; `Row::Group` is the only one, holding the five ways of
  saying one verb that would otherwise take five of the twelve rows a menu may
  carry. A second level for five *different* decisions is still wrong and is
  drawn as an inline row. egui 0.33 folds a submenu that will not fit to the far
  side of its parent rather than over it, which is what makes even the one
  affordable against a panel on the screen edge.
- **A gesture is a second route, never the only one.** The mouse is eight fields
  in `mouse`; the ones with a single meaning hold the name of a command from
  `config::mouse::VERBS`, and a test asserts every one of them also has a key.
  The wheel is read off the `MouseWheel` event (`view::wheel`) rather than off
  `raw_scroll_delta`, because Shift and Alt are spent by egui before this crate
  sees a delta.
- **A window in front owns the mouse and the keyboard.** Whether one is up is
  decided once a frame, in `App::a_window_is_in_front`, and written to the
  context with `utils::set_window_in_front`; a window that sets and clears a
  flag of its own is a window that clears it while another still needs it.
  `are_inputs_muted` is that flag *or* a focused text field, and gates the keys;
  the pointer takes two more things. Every window calls `utils::in_front`, which
  is egui's modal layer and stops the scroll areas and the focus behind it — and
  the few places that read the pointer for themselves ask
  `utils::is_a_window_in_front`, because `Response::contains_pointer` comes from
  a hit test that knows nothing about modal layers and is true wherever the
  window is not actually drawn. `Escape` shuts the window in front, and
  `App::was_typing` is why it takes two presses to do it from a search box:
  egui clears the focus itself before the program is called, so "was anything
  being typed into" has to be remembered from the frame before.
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
