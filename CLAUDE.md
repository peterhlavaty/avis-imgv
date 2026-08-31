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
```

A clippy warning is a failure. There is no rustfmt configuration: default
style.

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
| `src/view/` | drawing: `image_view/`, `grid_view/`, `stacks.rs`, `narrow.rs`, `organize/` |
| `src/app/` | wiring, input, modes, panels, the file watcher, `--benchmark` |
| `src/annotations/` | stars, flags, labels and tags, written to XMP sidecars |
| `src/organize/` | work on the folder rather than the image: renaming, timeshift, grouping |
| `src/config/` | the configuration file, its defaults, migrations between versions, and `registry/` — one row per field, which the settings window, the search and the key editor are all views over |
| `src/session.rs` | what is remembered between runs: window, folder, position |
| `src/ui/` | shared widgets, the notice bar, the key binding clash check |

**Keep files short.** Aim for 300 lines; the median here is 264. Past that,
split along the seam the file already has — a `mod.rs` plus siblings — rather
than growing it. Sixty files are over that today and are candidates for
splitting when they are next touched; they are not licence to write more.

**Keep folders small.** Around fifteen files to a directory. Past that the
directory is two concerns and should be two directories.

Both rules give way to a real reason, and the reason goes in the commit
message.

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
