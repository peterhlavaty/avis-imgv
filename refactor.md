# The refactor: what it should be

A proposal, for approval before any code is written. Nothing here has been
carried out.

It was arrived at by two rounds of survey and design — forty-eight agents, the
first sixteen reading one subsystem or sweeping one cross-cutting concern each,
then two critics, then twelve seam designs each adversarially verified, three
whole architectures and three judges. Every number below was measured rather
than taken from a report, and the claims that decide the plan were re-checked by
hand. Where a report was wrong, the correction is in the text.

The baseline the work must not breach: **1,454 tests pass, `cargo clippy
--all-targets` is clean.**

---

## 1. The finding that reframes the brief

The brief asks for total decoupling of the business logic from the GUI
framework, by traits, generics and dependency injection. Two measurements
change what that means here.

**The crate declares no traits at all.** Across 75,918 lines and 222 files there
is not one `trait` of this program's own. Generics appear as `Tree<T>`
(`src/history/tree.rs:48`), `Deck<C>` (`src/ui/deck/mod.rs:24`) and the
lifetime-parameterised parsers in `src/metadata/containers/`. `dyn` appears
twice, at `src/ui/slider/mod.rs:112` and `src/view/image_view/zoom.rs:268`.

**And yet the decoupling is already done.** Building the module graph from all
846 `crate::` references and walking it transitively: 38,668 of the crate's
lines already sit outside the drawing layer, and genuine toolkit contact there
is 3,064 lines in nine files — eight per cent, of which 370 (`history/panel.rs`)
is deliberate. `decoder`, `metadata`, `organize` and `annotations` import egui
nowhere.

The two facts together are the point. This codebase achieved its separation by
**dependency direction** — the logic modules simply do not import the toolkit —
rather than by inversion. That is a legitimate way to get there, it is cheaper
than traits at runtime, and it is why the pure half is so well tested.

So clause 2 of the brief is not a programme of work here. It is a finishing job
of about 1,100 lines, and it is eleven edges wide (§4). Spending the branch
introducing traits to reach a decoupling that already exists would be motion
without progress. The traits worth adding are the seven in §6, and each is
justified by a second implementor that exists, not by a hypothetical backend.

---

## 2. The thesis

The rules in `CLAUDE.md` are unusually good. They are also not holding, and four
live bugs prove it — each one a rule that was written down carefully, with its
reasoning, and then broken anyway.

| # | The rule, as recorded | What actually happens | Where |
|---|---|---|---|
| 1 | *"A photograph arriving or leaving"* — never stated, because nothing owns the collection | `Stack.members` holds **store positions**; `App::forget` removes from `self.paths`, shifting every later position, and no cull path mentions `stacking`. `poll` returns `false` once the scan finishes, and `detect` runs only on a folder change. Culling from a stacked folder makes `stacking.fold` hide and label the wrong frames until the folder is reopened. | `view/stacks.rs:33`, `app/cull/mod.rs:224-235`, `app/stacking.rs:105-121`, `app/mod.rs:676` |
| 2 | *"An allocation per frame is an allocation per photograph per frame."* | `surface.pixels.clone()` runs unconditionally, **before** `Marks::prepare` early-outs on `built_for`. With an overlay on that is a full RGBA copy of the decode every frame — roughly 240 MB per frame on a 60 MP raw. | `view/image_view/mod.rs:1002` against `marks.rs:47-50` |
| 3 | *"Mirroring is two-way or it is broken… 'Show the strip' had only the first, and so did nothing at all."* | `tags.advance_after_marking` is written to the config in `remember_runtime` and **never read back** in `apply_settings`. Ticking that box in the settings window is overwritten by the runtime value on the next frame. It silently reverts. | `app/settings.rs:148,190-192` against `:260` |
| 4 | The four-part resizable-panel rule, recorded in full | Implemented in full by **none of the four panels**. | table below |

On rule 4, measured:

| Panel | `set_min_*` (fill the size given) | `forced_*` + one `exact_*` frame |
|---|---|---|
| Filmstrip | ✓ `view/grid_view/filmstrip.rs:238` | ✓ `app/views.rs:198` |
| Tag panel | ✗ | ✓ `app/tagging.rs:227` |
| Side panel | ✗ | ✗ — `.default_width` only, `app/chrome.rs:174` |
| History panel | ✗ | ✗ — `.default_width` only, `history/panel.rs:57` |

`CLAUDE.md` says why that fails: *"`default_*` is dead from the second frame
on."* So changing `general.side_panel_width` or `history.panel_width` in the
settings window never reaches the panel until the next launch.

**None of the four is caught by any of the 1,454 tests**, and the reason is the
same for all four: they live in the layer that has no tests. `app/mod.rs` is
1,445 lines of code against 23 of test; `view/grid_view/mod.rs` 1,097 against
39; `view/image_view/bottom_bar.rs` 1,062 against 39. The pure-logic files run
about half tests. Untestability and toolkit coupling are not two problems here.
They are one.

So the thesis of this refactor is narrow and, I think, defensible:

> The rules are right. Prose does not hold them. Convert the rules that have
> already failed into types that cannot fail, and take the duplication out on
> the way past.

That is what makes this refactor different from a reorganisation: every
abstraction below is answerable to a rule that was broken, a duplicate that
drifted, or a test that cannot be written today.

---

## 3. What the survey found, in one table

129 findings, consolidated. The adversarial critic's central charge was that
sixteen agents proposed the same half-dozen abstractions between five and seven
times each in incompatible spellings, and that adopting two of any of them would
produce exactly the drift the *Write it once* rule exists to prevent. The
consolidation below is one design per seam.

| Seam | Written out today | Collapses to | Survived verification |
|---|---|---|---|
| 1 · Once-a-frame publish/mailbox | 7 statics in 5 storage shapes + 3 verbatim copies of `ONE_AT_A_TIME` | `src/board.rs` — `Published<T>`, `Mailbox<T>` | ✓ |
| 2 · Background queues | 3 hand-rolled `Arc<Mutex + Condvar>` pools | `src/work/` — `Pool<B: Backlog>` | ✓ |
| 3 · Errors | 2 error types; 301 `-> Option<`; ~30 hand-written sentences | `src/fault.rs` + 8 `thiserror` enums | ✓ |
| 4 · GPU / residency | 2 budgeted caches written method-for-method twice | `cache/textures.rs` + `cache/residency.rs` | ✓ |
| 5 · The open collection | 11 index-keyed structures told by hand at 6 disagreeing sites | `src/collection/` — `Collection`, `Changing` | ✗ amended |
| 6 · The settings mirror | 2 hand-written walks, one field falling between them | `config/mirror.rs` — `Mirror<L>` | ✓ |
| 7 · Closed sets of choices | each set written 3–5 times; 8 are bare `String` | `src/choices/` — `Choices` + one macro | ✗ amended |
| 8 · Menu tails / status bar | 17 hand-written readouts; 4+ enums carrying a `bool` home | `ui/surface/{tail,readout}.rs` | ✓ |
| 9 · Virtualised lists | 4 lists, 4 answers to virtualisation | `view/list/` | ✗ amended |
| 10 · Input port | 18 readers each taking an egui `Context` | `src/input/` + `src/keychord/` | ✗ amended |
| 11 · `App` | 71 fields, ~155 methods, 17 `impl` blocks, no constructor without a GPU | merged into seam 5 | ✗ amended |
| 12 · Panels and cards | panel set written in 7 places, 4 shapes | `Panel`/`Hideable`/`Panels`, `ui/sized.rs` | ✓ |

Five came back refuted on specifics and none may land as written. What is wrong
with each and the amendment adopted is in §9 — that section is not an appendix,
it is the part of this document most likely to matter.

---

## 4. The eleven edges

The whole of the GUI coupling outside `app`/`ui`/`view` is eleven imports. Three
verified by hand, because the plan rests on them:

- `src/view/narrow.rs:13` — `use crate::view::image_view::bottom_bar::Marks;`
  One import of a 43-line struct (`bottom_bar.rs:47`) out of a 1,101-line
  drawing file. This single edge is what makes `metadata`, `annotations`,
  `organize` and `decoder` transitively depend on the toolkit.
- `src/config/mod.rs:519,577` — two settings fields typed
  `crate::view::image_view::overlay::Corner` and `::opening::Opening`. The
  configuration depends on the drawing layer for two enums that are settings,
  not drawing.
- `src/metadata/xmp/mod.rs:418` — `use crate::view::narrow::FlagRule;` in a test.

The rest: `config::shortcut → eframe`; `cache::gpu → view::texture`;
`config::mouse → ui::slider::drag`; `config::registry::check → ui::{menus,keys}`;
`actions::user_action → ui::surface`; `config::load → eframe` (test);
`history::snapshot → view::image_view::viewports`; `utils → eframe`.

And the whole upward dependency on the shell is four names: `app::mode::Mode`
(14 references from `ui`, `view` and `history`), `app::input::Command` (8), and
two `Chrome` constants. `Mode` has no imports at all. Two file moves close four
cycles.

Cutting all eleven is about 1,100 lines, all of it moves.

---

## 5. The mini-libraries

What the brief means by "designed as if to be published as independent crates".
Each of these names no type of this program and would compile out of it
unchanged.

| Module | Lines | Generic over | Replaces |
|---|---|---|---|
| `src/board.rs` | ~120 | `Published<T>`, `Mailbox<T>` over the value | 7 statics, 5 storage shapes, 3 test serialisers |
| `src/work/` | ~420 | `Pool<B: Backlog>`, `Backlog::Item` | 3 worker pools, ~275 lines |
| `src/fault.rs` | ~90 | `Fault: Error` with `severity`/`subject` | ~30 hand-written failure sentences |
| `src/choices/` | ~300 | `Choices: Copy` with `EVERY`/`ROWS` consts | 22 hand-written `&[Choice]` tables |
| `src/fit.rs` | ~120 | `Edges`, three implementors | 5 copies of aspect-fit that disagree |
| `src/keychord/` | ~480 | — | `config/shortcut/` minus egui |
| `src/mode.rs`, `src/command.rs` | 135 + ~60 | — | the upward dependency on the shell |

Two notes on `board.rs`, because it is the clearest case of the thesis. All
seven of those statics are written by `App::update` and read from inside a draw
closure — the same thread, always. They are the draw thread's own frame state,
kept in a `static` because there was nowhere else to put it. Saying so in the
type removes every lock from the draw path, removes two per-frame allocations,
and deletes the three copies of `static ONE_AT_A_TIME: Mutex<()>`
(`ui/panel.rs:297`, `ui/keys/shown.rs:190`, `ui/slider/mod.rs:490`) taken at
twenty-two call sites — which exist only because process-wide state cannot be
held by two tests at once. A serialiser folded into the library is still one a
new test can forget to take; thread-local state is one nobody can forget.

The narrowing is real and goes into `CLAUDE.md` in one sentence: a value a
worker must see is not this. `annotations::sidecar::ADOBE_NAMING`
(`src/annotations/sidecar.rs:54`) genuinely crosses threads and stays an atomic
where it is, as the standing counterexample.

On `fit.rs`, one correction to the survey worth recording: it named four copies
of the aspect-fit formula, all in `src/view/`. There is a fifth in the decoder,
`resize::target_size` (`decoder/resize.rs:110-123`), and it is the only one with
a zero guard on the bound and a `.max(1)` floor, and the only one with tests.
The four float copies disagree about the degenerate case — `canvas::fit` returns
`Vec2::ZERO`, `grid_view::fit_in_cell` returns the whole cell. A sixth reader is
`cache/gpu.rs:146-148`. So the shared helper's home is **not** `src/view/`.

---

## 6. The seven traits

Every one has a second implementor that exists. None is on the draw path.

| Trait | Where | Second implementor |
|---|---|---|
| `Backlog` | `work/` | `Ranked`, `Newest`, `Coalescing` |
| `Fault` | `fault.rs` | the eight module error enums |
| `Choices` | `choices/` | the sixteen converted sets |
| `Edges` | `fit.rs` | `(u32,u32)`, `(f32,f32)`, `Vec2` |
| `Resident` | `cache/residency.rs` | the RAM and GPU tiers |
| `Follows`, `Reads` | `collection/` | both views, and a test double |
| `Launcher` | `actions/` | the real one, and a test double |

`Mirror<L>` is generic over the live object for a stated reason: `App` cannot be
constructed without a GPU, so a table of `fn(&App)` is a table no test can walk.
That is the brief's zero-cost abstraction argued rather than asserted.

Deliberately **not** a trait: the mailbox. A `Store`/`Home` trait abstracting
over "thread-local or shared" would have exactly one shared implementor, which
is a helper more general than its callers. `panel::Ask` and `slider::Ask` are
already the zero-cost version of dependency injection — a value crossing the
seam — and `dyn Answers<T>` on top of them would be dispatch for its own sake.

---

## 7. What is actually enforced

This is the section I would most want challenged, because it is where
refactoring proposals usually overclaim.

**By the compiler, and these are the only ones I will call enforced:**

1. ~~**The orphan rule.**~~ **Withdrawn — this was wrong.** The claim was that
   `impl fit::Edges for epaint::Vec2` could only be written in the drawing
   layer. It cannot be: within one crate the trait is local everywhere, so the
   orphan rule does not apply and only coherence (E0119) stops a *second*
   impl. Checked by trying it. The placement is a convention that keeps `fit`
   liftable, and becomes the compiler's rule only when it is its own crate.
   This was the proposal's one claimed compile-time GUI boundary, so the honest
   position is that the Cargo feature gate below is the only one.
2. **Exhaustive matches.** `Panel::chrome()` makes a panel without a menu a
   compile error. `MenuAction::goes_back_to_the_photographs` is the recorded
   precedent.
3. **Const assertions.** `const _: () = <T as Choices>::SOUND;` fails the build
   on an empty word, a repeated word or a repeated label. A real `E0080`.
4. **Struct literals with no `..`.** `Mirror { path, in_the_snapshot, reflect }`
   where every `Reflect` variant carries *both* accessors: a one-way mirror is a
   struct literal short of a field. **This is the mechanism that would have
   caught bug 3.**
5. **`#[must_use]` + `Drop`.** `Changing` settles in its destructor, so a route
   that removes a photograph cannot fail to tell the stacks. **This is the
   mechanism that would have caught bug 1.**

**By a Cargo feature, which is the one boundary that checks direction.** Within
one crate there is no compilation unit for the compiler to check separation
across, so changing a `use` line is cosmetic — an honest point the survey got
wrong twice. But a feature gate is not cosmetic:

```toml
[features]
default = ["gui", "custom_font"]
gui = ["dep:eframe", "dep:epaint", "dep:rfd"]
```

with `#[cfg(feature = "gui")] pub mod view;` and the same for `ui` and `app`.
Then `cargo check --no-default-features` removes eframe from the dependency
graph entirely and the boundary becomes an unresolved-import error raised by
rustc rather than a test somebody can delete. It costs about twenty lines and no
workspace. It only compiles *after* the eleven cuts, which is why it lands at
the end of stage 2 and not before. CI gains one line, because
`--no-default-features` then means two things at once.

**By a test, and named as such.** One `tests/layers.rs` — replacing the five
differently-named source-grep tests that five separate seams each invented, one
of which always fails because the file matches its own needle. It builds the
module graph from the 846 `crate::` references plus `mod` containment and
asserts transitively that no deciding file reaches a toolkit-naming one. It must
be transitive: a grep for `egui` in the file passes `view/narrow.rs`, which
imports `Marks` from a file with 34 mentions of egui. That is precisely the
coupling the critic used to refute the collection design.

**By convention, and said so.** File naming. A rule enforcing it by file name
fails on day one: `view/image_view/zoom.rs` names `epaint::Vec2` at lines 6 and
200, and `view/image_view/layout.rs` is a drawing file with a `CentralPanel` in
it.

---

## 8. The central tension, resolved

`CLAUDE.md`: *"Folders follow the functionality… A directory is named for a job
the program does and holds the logic, the drawing and the tests for that job
together. There is no directory of every widget or every type."*

The brief: business logic 100% agnostic of the GUI.

These pull against each other, and the resolution is per **file** inside each
job-directory, not per directory:

> A file that decides names no toolkit type but the `emath` geometry
> re-exports, and every `crate::` path it names is itself such a file. A file
> that draws sits beside it in the same directory and is named in the exemption
> table.

`history/panel.rs` stays in `src/history/` — the job keeps its drawing. What
changes is that the *deciding* files in that directory become checkably free of
it. This is the existing rule made mechanical, not a new one, and it is why
`src/collection/` is admissible: it is a job the program does — which
photographs are open, in what order, and what each carries — with the drawing
for that job staying in `src/view/`. It is not "the domain layer of the view",
which would be the directory-of-a-kind the rule forbids. `CLAUDE.md`'s path
table gains that row in the same commit.

---

## 9. The five designs that failed verification

Each was refuted on specifics by an adversarial verifier reading the code. None
lands as written. Recorded here because these are the amendments that decide
whether the branch compiles.

**Seams 5 and 11 are one seam.** Both designed `Collection`, both fixed bug 1,
both were refuted on the *same* fatal: `settle`'s `is_idle() && !is_on()`
early-out is sound only at `open_within` (`app/mod.rs:578`), where `set_images`
has just reset `Visible` (`navigate.rs:29`), and silently breaks suspending a
filter, clearing the last rule and turning stacking off. Amendment: `settle`
runs unconditionally and holds the last `Visible` to compare; `changed` splits
into `rewritten` (reloads the stores) and `remarked` (touches no store) — merging
those two was the second fatal, and it would evict the photograph on screen from
the GPU every time somebody presses `3`.

**Seam 7 (`choices`).** The `choices!` macro as written does not expand — the
variant matcher needs `$(#[$attr:meta])*`, and the design's own worked example
fails on it. `LabelRule` is dropped from the conversion: it carries data.
`Corner::next` stays inherent — it is a clockwise circuit and the modular
version jumps diagonally.

**Seam 9 (`view::list`).** `List::show` cannot take `&mut Follow` at the contact
sheet: the draw closure calls an `App` method and captures the whole receiver
(`E0499`). Amendment: `Follow` is taken by value and handed back in `Shown`.
The design also silently deleted three shipped relative-scroll gestures
(`grid_view/mod.rs:529-533,535-540,578-581`); `List::by(points)` carries them.

**Seam 10 (input).** One frame snapshot at the top of `update` cannot work:
three of the five readers run after the cards have consumed `Escape`, and
`app/cull/bin.rs:290-292` records why that matters. Amendment: scoped readings,
not one snapshot.

**Seam 12's fixed keys.** The five keys the registry cannot see — arrows, Home,
End, Enter, Escape — stay read as plain keys. Giving them registry rows
contradicts the design's own risk note.

Three further corrections the verifiers made to *surviving* designs, adopted:
`Pool` loses `flush`/`in_flight`/its second condvar (no production caller, and a
`notify_all` per job is a futex wake on the decode path); `xmp::Error` needs five
variants, not four, because four of the five `.ok()?` sites in `update` yield
`std::io::Error`; and one `Textures` is built in `App` so the shader compiles
once at launch rather than four times.

---

## 10. The plan

Six stages, thirty-six commits, five push points. Each commit is one whole unit
in the shape `CLAUDE.md` describes — code, tests, the README section it changes
and a changelog entry — and each leaves the viewer working.

**The ordering rule that matters:** a commit that deletes a duplicate has no
chain behind it and may be dropped; a commit that introduces an owner does not.
The branch can stop at any push point without stranding what came before.

### Stage 0 — the bugs, first (4 commits, ~600 lines, low risk)

The four verified bugs are fixed before any architecture, each with the test
that would have caught it. None needs the refactor.

1. The stacks are told when a photograph leaves — `Stacks::remove_shifting` /
   `insert_shifting` beside the `Visible::remove_shifting` at `view/visible.rs:99`
   they copy. Two toolkit-free files that already have test modules. ~200 lines.
2. The overlay clone moves behind the early-out — `view/image_view/mod.rs:1002`,
   with before-and-after frame times on a 60 MP raw. Five files. (A survey
   report called `filmstrip.rs:271` its twin; it is not — that is a `PathBuf`
   clone per visible cell per frame, a smaller and separate cost, and it belongs
   with the list work in stage 5.)
3. `advance_after_marking` is read back — nine files, ninety lines. `App::advancing`
   is a third copy beside two others; the commit removes the copy rather than
   adding a read.
4. The four-part panel rule for the side panel and the history panel.

Fixing these first is deliberate. They are what the branch is *for*, they are
cheap, and if the architecture is never approved the photographer is still
better off. **Push point 1.**

### Stage 1 — the order is written down, and the bottom layer (11 commits, ~5,400 lines)

`tests/layers.rs`; the three stale `CLAUDE.md` figures corrected; `Mode` and
`Command` stop pointing upward; `src/utils.rs` is deleted into the three jobs it
holds; then `fault`, `board`, `work`, `choices`, `fit`, `keychord` — the
mini-libraries, in dependency order (`fault` before `work`, `choices` before
`mirror`, `Marks` moves before the collection). **Push point 2.**

### Stage 2 — the domain owns itself (5 commits, ~2,600 lines)

The remaining sets sink out of the drawing layer; the cache stops being both a
maker of textures and a store of them; one residency map replaces two written
method-for-method; the `organize` and `xmp` error enums. Ends with **the Cargo
feature gate**, which is the commit that turns the boundary from a test into a
compiler error. **Push point 3.**

### Stage 3 — the collection is one thing (4 commits, ~2,200 lines)

`src/collection/` exists; the two views become one `Follows`; `Collection` and
`Changing` own the eleven structures; the fifteen hand-written re-narrows become
none. This is where bug 1 stops being *fixed* and starts being *impossible*.
**Push point 4.**

### Stage 4 — the shell (8 commits, ~4,500 lines)

`ui/sized.rs` and the panel rule as one type; `Panel`/`Hideable`/`Panels`;
`Mirror<L>`, which is where bug 3 stops being possible; the menu tails; the
status-bar readout. **Push point 5.**

### Stage 5 — ports and adapters (4 commits, ~2,800 lines)

The input port and the virtualised list. Scheduled last on purpose: they are the
two largest behaviour-carrying rewrites, the two whose verification failed on
architecture rather than detail, and the two with the least duplication behind
them relative to their size.

### Cost

About 18,000 lines touched across roughly 190 of 222 files, netting close to
zero production lines: **+2,200 new against −4,700 deleted**, and **+3,500 to
+4,500 test lines**, concentrated exactly where there are none today.

Confidence ±25% on the total, ±40% on any single commit. Two measured figures
set the pace, both measured on this machine: `cargo test` is 23 seconds end to
end, and `cargo build --release` is **2m57s** from a touched `lib.rs`.
`CLAUDE.md` requires a release build at every commit, so the thirty-six commits
cost about **105 minutes of building** on their own — which is the real argument
for the push-point structure rather than a long unbroken branch.

---

## 11. What I will not do

Judgement is mostly in the refusals.

- **A Cargo workspace.** Measured on this machine, warm: `cargo check` **2.0s**,
  `--all-targets` **3.1s**, `cargo test` 23s end to end with the 1,454 tests
  themselves running in 0.54s. There is no build-time win to buy — the inner
  loop is already two seconds. Against that, feature forwarding for `jxl`,
  `libraw` and
  `custom_font` across a six-job CI matrix on three platforms, where a missing
  `default-features = false` silently checks the wrong thing. The workspace is
  the *reward* for the eleven cuts, not the tool for making them. Trigger for
  reconsidering, written down so nobody relitigates it: a second front end, or
  publishing one of the mini-libraries.
- **A GPU port.** Three seams proposed three incompatible traits; the critic
  showed one cannot work at all, because `GpuTexture` holds the `RenderState` so
  its `Drop` can free the texture (`cache/gpu.rs:10-12,93-95`) — a trait
  returning a bare `GpuTexture` has removed nothing. All three are honest that
  what is bought is tests for a few hundred lines, not a second backend, and
  the claim that it lets `--benchmark` run without a window is false:
  `benchmark.rs` names neither `RenderState` nor `ImageStore`, and its own doc
  says the number folds in the upload and the drawing. Seam 4 buys the same
  tests by splitting the file instead.
- **`Registry<C>` — the settings engine made generic over the document.** The
  most tempting move in the crate: the engine genuinely is generic. Refused for
  the same reason as the workspace. There is exactly one `Config` and there will
  be one until there are two crates.
- **A `Batch<O>` folder-job engine.** Five seams spelled it. The four bulk jobs
  neither stream nor may be cancelled; the crawl is cooperative; `Scan` streams.
  Three shapes whose shared abstraction would have one caller each. And the
  version that moves the cull to a worker must be refused outright: the bin's
  note is written once for the batch, outside the walk (`app/cull/mod.rs:180-182`),
  which is what lets undo, redo and put-back agree.
- **Genericising `zoom.rs` or `pan.rs` over a scalar.** One scalar, one caller
  apiece.
- **Adding `emath` to `Cargo.toml` and calling it decoupling.** `epaint` is
  already a direct dependency and re-exports `emath` and its types
  (`epaint-0.33.0/src/lib.rs:76,82`). Changing which path a name is written as
  changes nothing.
- **Splitting `src/ui/` into ten job-directories.** Correct — ten of its
  twenty-four entries have one caller — but a pure move with no bug behind it,
  and it collides head-on with the two seams that rewrite six of the ten. First
  thing to do *after* this branch.
- **Deleting `config/defaults.rs`** (148 functions, 827 lines). The single
  largest deletion available, and correct, but `#[serde(default)]` interacts
  with `Config::partial`'s per-section loading, whose failure mode is a
  photographer's configuration silently reset. Its own branch. Note also that
  five migration steps call it (`migrate.rs:291,310,327,347,352`) and each needs
  the default *as of that version*.
- **Threading the folder listing off the draw path.** `ui/tree.rs` and
  `ui/navigator.rs` read the disk inside a paint closure, breaking *"Nothing in
  the draw path may wait on I/O"*. It is a threading design, not a chrome one,
  and smuggling it into a commit about cards is how a branch stops being
  reviewable. It should be its own commit — flagged, not scheduled here.

---

## 12. Where this proposal is weak

Three concessions, unsoftened.

**The layering would not have prevented one of the four bugs.** Every verified
fault is temporal, not structural: the stacks are not told when a photograph
leaves; a clone happens before an early-out; a mirror is written one way; a
panel is given a width on a frame that no longer reads it. A dependency order
catches none of that. What catches them is §7's items 4 and 5 — the struct
literal with no `..`, and the `Drop` guard — and those are worth more than the
whole of the module graph. The graph is the *ordering constraint* on the work,
not the deliverable.

**Five of twelve designs failed verification.** They are amended in §9 rather
than dropped, and the two largest — the input port and the list — are scheduled
last precisely because their failures were architectural rather than
detail-level. If either turns out worse than costed, stopping after stage 4
loses nothing already landed.

**18,000 lines is a lot of change to a program that works.** The honest
mitigation is the push-point structure: the value is front-loaded, stage 0 needs
no architecture at all, and every stage boundary is a place to stop. A
reasonable outcome of this proposal is approval of stages 0–2 and a decision to
review again before stage 3.

---

## 13. Corrections to `CLAUDE.md`

Three of its own measurements have drifted, and rules rest on them. Per
*"Keeping this file honest"*, they are corrected in commit 1:

| Recorded | Measured | Note |
|---|---|---|
| "around 114" test modules | **180** | understates the crate's strongest asset by a third |
| "Sixty files are over" 300 lines | **112** total; **55** on non-test lines | the crate grew its *tests*, not its code — the rule is being kept better than the raw count suggests |
| "the median here is 264" | **302** total, 193 of code | the median file has crossed the aim on the raw measure |

Two figures checked and found correct, so they should not be touched: the
"sixteen `shortcut::consume` sites" is exactly sixteen, and "the registry's five
key-shaped `Access` variants" is exactly five.

`CLAUDE.md` also states that adding a piece of watched state means *"a field on
`Watched`/`Snapshot`, a `Change` variant and an arm in `app/history/restore.rs`,
and nothing else."* It is seven places, across 886 lines
(`history/snapshot.rs:111,125,168,225,270,309,542`). That sentence should be
corrected whether or not the rest of this is approved.

---

## 14. The decision

I have written no code and will write none until you say so. What I need is one
of:

1. **Approve as staged** — I begin at stage 0, commit 1, and stop at push point
   2 for review.
2. **Approve stage 0 only** — the four bug fixes, no architecture, and we talk
   again.
3. **Amend** — tell me which refusals in §11 you disagree with, or which seams
   in §3 you want dropped or added.

If you want a smaller version of this: stages 0 and 1 alone are about 6,000
lines, fix all four bugs, deliver six of the seven mini-libraries, and leave the
shape of `app/`, the views and the status bar untouched. That is the version I
would choose if the branch had to be judged on value per line changed.
