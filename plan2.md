# Plan 2

A review of avis-imgv against a different question from the one [plan.md](plan.md)
asked. That plan asked whether the viewer could do the work. This one asks
whether a person can operate it: whether what it does can be found, understood,
adjusted, and reached from where they already are.

The short answer is that the engine and the feature set are finished and the
interface is not. The program holds 110 settings and offers controls for 63 of
them. It has 33 hover-help call sites in 40,504 lines of source. Right-clicking
the photograph draws nothing, because the menu it would draw is empty on a fresh
install. Almost everything the README describes is real, and almost none of it
says so on screen.

## How this was arrived at

Every file under `src/` was read again, this time for what it puts in front of a
person rather than for what it computes. Eight readers each took one surface —
the configuration, the chrome, the image view, the contact sheet, the folder
modes, the marks and tags, the help and feedback, the input model — and wrote
down every place a user would struggle, with a file and a line for each. That
produced 299 findings.

Alongside them, six readers went out to the field: the complaints people make
about IrfanView, FastStone, XnView MP, nomacs, qimgv, ImageGlass, JPEGView and
Geeqie; the complaints photographers make about Photo Mechanic, FastRawViewer,
Lightroom Classic, Capture One, digiKam, darktable and Aftershoot; what people
say about preferences windows in particular; what the research says about
discoverability and onboarding; the platform conventions for context menus; and
the evidence on choosing a control for a value. That produced 112 themes, each
with the pages it came from.

The settings architecture was not designed once. Four designs were written
independently and deliberately in tension — a category tree, settings reached
from the object they affect, a searchable index, and a task-shaped set of pages
— and three judges scored them: a photographer twenty minutes into their first
session, a Photo Mechanic user of fourteen years, and the person who would have
to write it in egui. The tree won two votes of three; what the losing designs
were right about was grafted in, and §13 records what was left out and why.

Each of the eleven argument chapters, §1 to §11, was then drafted, and each was
handed to a second reader whose job was to refute it: to open every cited line,
recount every number with a command, and check every claim about another program
against the page it came from. 2,071 claims were checked and 282 were wrong —
mostly line numbers that had drifted and counts that had been taken from a
summary rather than from the source. All 282 were corrected. §12 and §13 — the
order of the work, and the list of what is refused — were written last, out of
those eleven, and went instead to the final pass, which recounted every figure
in the document from the source itself, found 22 places where two chapters
disagreed, and settled each one.

Where a claim below names a file and a line, it was opened. Where it names a
number, the command that produced it is recorded in the working notes and the
figure in the text is that command's output. Where it says what somebody else's
program does, or what its users say about it, there is a page behind it.

## What this plan is for

| | |
|---|---|
| §1 | Where the interface stands: first launch, the ways in, and the arithmetic of the gap |
| §2 | What people complain about in comparable programs, and whether it happens here |
| §3 | The settings that cannot be reached, and the window that reaches them |
| §4 | Where each setting belongs, and why the current grouping is the struct's rather than the user's |
| §5 | The right control for each value — and why a slider over every value beats a list of four presets |
| §6 | Settings that do not exist at all, and which of them should |
| §7 | Right-click on the thing itself: a menu for every surface a pointer can land on |
| §8 | Getting from one thing to the thing next to it, and the dead ends where you cannot |
| §9 | The mouse, which can do almost nothing and cannot be reconfigured, and the keyboard for people who do not know it |
| §10 | A program that explains itself: tooltips, empty states, first run, and the features nothing on screen mentions |
| §11 | Configurable without being complicated — the discipline that keeps the rest of this from making a preferences browser |
| §12 | The plan: nine stages, 99 items, each traceable to the chapter that argued for it |
| §13 | Deliberately not doing |

The plan answers a brief of eleven demands, and an instruction to verify. The
map below gives each demand and the chapter that answers it, so that the plan
can be checked against the brief rather than read through for it.

| The brief asked for | Answered in |
|---|---|
| Every feature and the whole GUI, gone through in detail | Three passes rather than one chapter, from three directions. **§1** takes the chrome: what is on screen at launch, the four ways in, the five windows, the eleven menu items. **§3.1** takes the configuration, one field at a time, all 110 of them. **§10.1** takes the features the README publishes, one at a time, against what the running program says about each |
| How well and how easily someone would work with it | **§2**, which puts the complaints made about seventeen comparable programs to this one, seventy-four rows, each with a verdict; **§8**, which lists the dead ends; **§9**, which follows the mouse and the keyboard of somebody who does not know the keys |
| Whether everything that ought to be settable can be set | **§3** for the settings that exist and cannot be reached; **§6** for the ones that do not exist |
| Whether features are described well enough for a new user to understand them | **§10** |
| Whether it is easy to click through between related features | **§8** |
| Whether anything with settings can be right-clicked and changed from there | **§7** |
| Whether all settings are logically grouped | **§4** |
| Whether settings have enough options — a slider over every value, rather than a few presets | **§5** |
| What people most often complain about in similar programs, and whether the same happens here | **§2** |
| Maximally configurable and still simple to operate and understand | **§11** |
| One setting reachable by several routes | **§11.5** states the rule and its limit — several routes to one control, never several copies of it; **§3.3** gives every settings page two routes and usually three; **§7** supplies the third, from the object itself |

The instruction to verify everything and correct what is wrong is the pass
described above, and the corrections it made are not marked in the text: what is
left is what survived it.

## 1. Where the interface stands

The five stages of the first plan are finished. Every verb a culling tool needs
is in the binary: reject and pick, colour labels, a selection, delete to the
bin, move and copy to destination slots, an undo journal, filtering and sorting
where the photographs are, a compare view with a shared viewport, a histogram,
clipping and focus overlays, virtual stacks, hierarchical keywords, a filmstrip,
session restore. On the folder the README publishes — 120 24-megapixel JPEGs,
larger than the cache, on a 24-core Ryzen — `--benchmark` reports 501 images in
11.50 s, 43.6 images a second, median frame 2.70 ms (`README.md:291`). The suite
is at 887 tests: 880 in `src/`, seven in `tests/`.

None of that is visible when the window opens.

This chapter is the survey the rest of the plan argues from: what is on screen
at launch, the four ways in, the arithmetic of the settings gap, the five
windows and the eleven menu items, and the single channel the program has for
telling anybody anything. It is the same shape of argument as section 1 of the
first plan — the engine was never the problem — except that what is missing now
is not a feature. It is a way in.

### 1.1 What a person meets on first launch

The window is made with a fixed title, `"Avis Image Viewer"`
(`src/main.rs:52`), which never changes: the only viewport commands the program
ever sends are `Fullscreen` and `Close` (`src/app/mod.rs:154`, `:686`,
`src/app/settings.rs:76`, `src/app/chrome.rs:180`), so the title bar never says
which folder is open.

Every panel starts closed. In one block of `App::new`:

| Surface | Starting state | Where |
|---|---|---|
| Menu bar | hidden | `src/app/mod.rs:201` |
| Metadata, histogram and cache panel | hidden | `src/app/mod.rs:202` |
| Frame timings | hidden | `src/app/mod.rs:203` |
| Rating and tags panel | hidden | `src/app/mod.rs:212` |
| Keyboard editor | closed | `src/app/mod.rs:217` |
| Slideshow settings | closed | `src/app/mod.rs:218` |
| Filter and stack bar | hidden | `src/app/mod.rs:230` |
| Cheat sheet | closed | `src/app/mod.rs:235` |
| Filmstrip | follows `filmstrip_height`, which defaults to `0.0` | `src/app/mod.rs:175`, `:234`, `src/config/defaults.rs:299-301` |
| Mode | always Image, whatever it was yesterday | `src/app/mod.rs:200` |

With no argument on the command line the working directory is crawled
(`src/crawler.rs:46`, `:65-78`). The code's own comment says what that is worth:
"the working directory of a viewer started from a desktop icon is nobody's
choice" (`src/crawler.rs:28-29`). On a genuinely first run there is no session
to fall back on either (`src/app/mod.rs:260-285`), so what a double-clicked
desktop entry opens is whatever happens to be in the directory it was launched
from.

What is then on screen is: a dark grey window (`src/ui/theme.rs:23`, colours at
`:26-31`, `panel_fill` at `:38`); a bottom bar holding an empty jump field
hinting "go to" (`src/view/image_view/bottom_bar.rs:215`), the text `0/0`
(`:114-117`), no file name and the reading `0.0%` — with the zoom slider beside
it suppressed, because a slider sitting at its floor would look like a
magnification (`:253-257`); and, in the middle of a mid-grey central panel, the
three words **"No images here"** (`src/view/image_view/layout.rs:42`, backdrop
at `:12`).

That is the whole of the empty state. There is no button, no drop target — no
`dropped_files` or `hovered_files` handler exists anywhere in `src/` — no recent
folders, though the session file remembers sixty-four of them
(`src/session.rs:30`, `:56-65`), and no line of text saying which key opens
anything. Grepping `first_run|welcome|onboard|tour` across `src/` returns
nothing.

A person who has just installed the program and does not already know that `F1`
exists has, at this point, run out of interface.

### 1.2 The four ways in

The mouse is not one of them. It works — the jump field, the zoom slider and its
label, the filter bar, the contact sheet, the destination panel all take clicks —
but only on what is already drawn, and what is drawn is decided elsewhere. There
are four ways to reach anything that is not already on screen, and they reach
very different amounts of it.

| Way in | What it reaches | How it is found | Where |
|---|---|---|---|
| The menu bar | 11 items: three in File, six mode radios, two settings windows | `F1`, and nothing on screen says so | `src/app/panels.rs:26-82`, `src/config/defaults.rs:68-70` |
| The keyboard | 69 rows in the registry, plus a second layer of keys compiled into the views | the cheat sheet on `?`, and nothing on screen says so either | `src/config/bindings.rs:85-439`, `src/app/input.rs:113-115` |
| The configuration file | 110 settings | its location never appears on screen | `src/config/mod.rs:18-49`, `src/config/load.rs:14-17` |
| The command line | three flags and a path | `--help`, printed before the window is made | `src/main.rs:10-21`, `:26-29` |

**The menu bar** is three menus. File offers *Open Folder*, *Open Files* and
*Send rejected to the bin…* (`src/app/panels.rs:33-54`). Mode offers six radio
rows, which the source notes is also where a user finds out which mode they are
in (`:56-65`; labels at `src/app/mode.rs:37-46`). Settings offers *Keyboard…*
and *Slideshow…* (`src/app/panels.rs:67-77`). `MenuAction` has six variants and
that is the whole of it (`:12-23`): no View menu, no Edit menu, no Tools menu,
no Help menu, no About, no Quit entry — `Alt+Q` quits (`src/app/mod.rs:686`,
`src/config/defaults.rs:64-66`) and appears in no menu. *Open Files* takes the
first of the files picked, finds its parent and opens the whole parent folder,
discarding the rest of the selection (`src/app/settings.rs:27-38`), although the
command line does honour a multi-file collection (`src/crawler.rs:53-61`).

**The keyboard** is where the program actually lives. `bindings::all()` returns
69 rows — 58 written-out bindings plus five colour labels and six ratings
generated from `Label::CHOICES` and `MAX_RATING` (`src/config/bindings.rs:85-439`,
loops at `:420-436`) — each with a sentence of its own, and both the editor and
the cheat sheet are generated from that one table. Beside it sits a second,
invisible layer. `Escape` alone has seven separate meanings compiled into seven
files: leave a deletion alone (`src/app/cull.rs:178`), close an overlay
(`src/app/input.rs:212`), surrender keyboard focus (`src/app/mod.rs:802`),
cancel the destination panel (`src/ui/destinations.rs:114`), abandon a key
capture (`src/ui/keys.rs:261`), clear the grid selection
(`src/view/grid_view/mod.rs:630`) and stop comparing
(`src/view/image_view/input.rs:117`). `Home`, `End`, `Page Up` and `Page Down`
in the image view (`src/view/image_view/input.rs:109-112`), `Tab` and `/` in the
comparison (`:115-116`), the arrows, `Home`, `End` and `Enter` in the contact
sheet (`src/view/grid_view/mod.rs:559-605`), the digits and `Enter` in the
destination panel (`src/ui/destinations.rs:118-140`), `Y`, `N` and `Enter` in
the delete confirmation (`src/app/cull.rs:178-191`), `F10` for the frame timings
(`src/app/input.rs:102-104`) and `?` for the cheat sheet itself (`:113-115`) all
carry meanings the registry does not hold. None of them can be rebound, none
appears in the keyboard editor and none appears on the cheat sheet (§10) — and
two, `Page Down` and the digits, are keys the registry *does* bind to something
else (`src/config/defaults.rs:307-309`, `:360-364`, `:455-463`), so the same
press means one thing in one view and another in the next with nothing on screen
to say so. One further shortcut is not in the registry at all: the one inside a
user action (`src/config/mod.rs:542`), which the editor cannot reach (§9).

**The configuration file** holds 110 settings and is the only way to reach
forty-seven of them (§1.3, §3). `Config::path()` exists
(`src/config/load.rs:14-17`) and has two callers, neither of them the interface:
the tag catalogue resolver (`src/annotations/catalog.rs:190`) and a test in the
logging module (`src/logging.rs:191`). There is no menu entry that opens the
file, no line on screen that gives its location, and no reveal-in-folder; the
name `config.json` never appears on screen (`src/config/load.rs:16`, `:58`). The
path is written to the log (`src/config/load.rs:46`, `:59`), which sits beside
the configuration and is announced the same way — into itself
(`src/logging.rs:37-40`, `src/main.rs:34-36`). On a fresh install the file the
user is expected to edit is written with `serde_json::to_string` — one line, not
pretty-printed (`src/config/load.rs:69`); only later saves are formatted
(`:42`).

**The command line** handles `--help` before anything else and prints eleven
lines: a usage line, a two-line entry for `PATH`, and four options, one of them
wrapped (`src/main.rs:10-21`, `:26-29`). It does not mention `?`, the
configuration file or the log. `--slideshow`, `--fullscreen` and `--benchmark`
are matched by `any(|arg| arg == …)` (`src/main.rs:43-45`); anything else is
treated as a path.

Three of the four are behind a key or a path that nothing on screen mentions,
and the fourth is printed before the window exists. Nielsen Norman's own
statement of the accelerator heuristic is that shortcuts are "secondary ways of
accomplishing the same task" and that a good one "doesn't get in the way of a
new user (who very likely is not aware of it at all)" —
<https://www.nngroup.com/articles/flexibility-efficiency-heuristic/>. The word
that matters is *secondary*. Here they are primary, and for the menu bar, the
side panel, the tag panel, the filter bar, the directory tree and the cheat
sheet they are the only route.

### 1.3 The arithmetic of the settings gap

`Config` declares nine serialisable fields of its own, eight of them sections
holding 110 leaf fields between them; the ninth, `version`, is the file-format
number and is never shown, and `partial` and `migrated` carry `#[serde(skip)]`
and are not in the file at all (`src/config/mod.rs:18-49`). So the file has 111
keys and **110 settings**. Sixty of the 110 are `sc_*` shortcuts and the
keyboard editor reaches every one of them, across 69 rows; the slideshow window
reaches three more; **forty-seven are reachable nowhere in the running program**,
of which six can be nudged for the session by a key that never writes them back
and forty-one cannot be changed at all; and twenty-six do not take effect until
the next launch — twenty-five wholly, one half. §3 owns the field-by-field
inventory, the restart mechanism and the validation, and those five figures are
its; what follows is what they mean for the interface.

**All sixty of the settings the editor reaches are keys.** It covers nothing
else. Not one photographic setting is reachable from inside the running program:
not whether a raw file is browsed through its embedded preview or developed
(`raw.source`, `src/config/mod.rs:85`, `:102-111`, `src/app/stores.rs:71`), not
the thumbnail resolution, not the cell aspect, not the destination folders, not
the name of the rejected folder, not the list of metadata tags, not the
interface text scaling. The destination panel's *Choose a folder…* comes closest
and is not close: it picks a folder for one errand and forgets it, never
touching `cull.destinations` (`src/app/cull.rs:353-371`). The Settings menu is
two entries long (`src/app/panels.rs:67-77`) and neither entry is about
photographs.

**Six more can be changed with a key or a drag, and are then forgotten.** `O`
cycles the overlay corner on the image view's own clone of the configuration
(`src/view/image_view/mod.rs:278-280`); bare `+` and `-` change `self.columns`
and never `grid_view.images_per_row` (`src/config/defaults.rs:318-323`,
`src/view/grid_view/mod.rs:524-528`, `:643`); the matching pair in the image
view — `Ctrl` with the same two keys (`src/config/defaults.rs:247-252`) — moves
`self.images_shown` and never `image_view.nr_images_shown`
(`src/view/image_view/mod.rs:284`, `:289`); `Ctrl+Shift+A` flips
`self.advancing` and never `tags.advance_after_marking` (`src/app/mod.rs:706`);
the filmstrip key flips a visibility that started life as
`grid_view.filmstrip_height` (`:713`, `:175`); and the tag panel's splitter is
draggable but its width is egui's, not the configuration's
(`src/ui/tag_panel/mod.rs:61-64`). Nothing a user does writes the file except
the keyboard editor and the slideshow window, which reach it through one
function (`src/app/settings.rs:70`, `:94` → `:104-110`); the only other caller of
`Config::save` is the migration writing a brought-forward file back at startup
(`src/config/load.rs:101`). Hand-editing the file sets where all six *start*.
Nothing records where they got to. A photographer who sets four thumbnails per
row, turns the filmstrip on and switches advance-after-marking on gets none of
it back tomorrow — and the side panel they opened and the rejects they hid are
worse off still, having no configuration field to be forgotten from (§4 draws
the line between a setting and session state).

**Twenty-six of the forty-seven would still need a relaunch even if something
could write them.** The store configuration is built once, in `App::new`
(`src/app/mod.rs:162-165` → `src/app/stores.rs:32-66`), and neither
`ImageView::set_config` (`src/view/image_view/navigate.rs:95-98`) nor
`GridView::set_config` (`src/view/grid_view/mod.rs:195-197`) touches a
`StoreConfig`, a `Loader` or a runtime counter, so all six of `cache.*`, the
five LibRaw options, the cache-shaped fields of both views and the tag
catalogue are captured at startup and never read again; `general.text_scaling`
is applied exactly once (`src/app/mod.rs:150` → `:888`). §3 has the field-by-field
table and what unsticks each one. What matters here is that nothing anywhere
marks a setting as needing a restart, and that the only two things the interface
*can* change — keys and the slideshow — both take effect at once
(`src/app/settings.rs:57-70`, `:85-95`). The model the interface teaches is
exactly wrong for these twenty-six.

Nothing about a file-only configuration is unusual in this class of program, and
§2 has the comparison. What is unusual is the ratio. ImageGlass ships a layered
JSON configuration (<https://imageglass.org/docs/app-configs>) and documents no
setting as file-only: the GUI is the primary editor and the file is the escape
hatch. Here the file is not the escape hatch; it is the product — and the
example shipped for people to copy from carries 103 of the 110 fields
(`examples/config.json`), which is the closest thing to a settings interface the
program has.

### 1.4 What the program says about itself

There are 33 hover-help call sites in `src/` — 31 `on_hover_text`, one
`on_hover_text_at_pointer` and one `on_hover_ui`, and no
`on_disabled_hover_text` anywhere — spread across 11 of the 139 source files, in
40,504 lines. Thirty-nine files name an `egui::Ui` or an `egui::Context`;
twenty-eight of those have no tooltip at all. They are also unevenly placed,
which is §10's subject along with what a tooltip may and may not carry.

The rest of the explanatory surface is thinner still. There is no Help menu, no
About window and no version string: `grep -rn "CARGO_PKG_VERSION" src/` returns
nothing, and `Config.version` is a file-format number
(`src/config/mod.rs:24-25`). Nothing in `src/` opens a URL or a file manager, so
there is no link to the README and no in-app documentation. The cheat sheet is
the best piece of documentation in the program — generated from the live binding
table, so it shows the user's own keys (`src/ui/cheat_sheet.rs:48`) — and it is
behind one key that nothing on screen mentions and no menu names
(`src/app/input.rs:113-115`); §10 has what is wrong with it and what it leaves
out. Its own doc comment states the problem it was built for: "The README has
them all, and the README is not on screen while somebody is culling"
(`src/ui/cheat_sheet.rs:4-5`).

`F1` opens the menu bar (`src/config/defaults.rs:68-70`) — three menus, none of
them Help. The source records that the cheat sheet was considered for `F1`,
which it calls the obvious companion to `?`, and rejected because the menu
already had it (`src/app/input.rs:106-112`).

Everything else the program has to say goes through one channel: a stack of dark
red boxes in an area anchored to the top of the window
(`src/ui/notice.rs:69-102`). Every message uses the same fill, `rgb(72,32,32)`
(`:91`) — the startup warning that part of the configuration could not be read
(`src/app/mod.rs:248-253`), the startup warning that two commands are on the
same key (`:240-242`, text at `src/ui/keys.rs:214`), the confirmation "Moved 12
photograph(s) to Selects" (`src/app/cull.rs:468-478`), and the failure to write a
sidecar (`src/annotations/writer.rs:128`, surfaced at `src/app/mod.rs:759-763`)
are indistinguishable. A line holds for six seconds and fades over six hundred
milliseconds (`src/ui/notice.rs:16`, `:19`); at most four are kept and the oldest
beyond that is dropped without a word (`:22`, `:56-58`); a repeat is counted
rather than stacked (`:44-47`); and the whole area is drawn `interactable(false)`
(`:80`), so nothing in it can be clicked, copied or dismissed. There is no
history. A warning that two commands have landed on the same key, or that the
configuration file is not being written over, is gone 6.6 seconds after launch
and cannot be recovered.

### 1.5 Five windows, eleven menu items, and a right-click that does nothing

Every `egui::Window` in the program:

| Window | Title | Opened by | Where |
|---|---|---|---|
| Delete confirmation | `Move to the bin` / `Delete for good` | `Delete`, `Shift+Delete`, or File ▸ Send rejected to the bin… | `src/app/cull.rs:137` |
| Slideshow settings | `Slideshow` | Settings ▸ Slideshow… only | `src/app/panels.rs:92` |
| Cheat sheet | `Keys — <mode>` | `?` only | `src/ui/cheat_sheet.rs:70` |
| Move / Copy destinations | `Move to…` / `Copy to…` | `Alt+M`, `Alt+C` | `src/ui/destinations.rs:60`, opened at `src/app/cull.rs:309` |
| Keyboard editor | `Keyboard` | Settings ▸ Keyboard… only | `src/ui/keys.rs:49` |

Five. Two of them are the settings surface. Two are transient prompts. One is
the cheat sheet. The third culling key is not among them, because it opens
nothing: `Shift+X` builds a slot out of `cull.rejected_folder` and carries the
photograph there without asking, saying so afterwards in the notice band and
nowhere else (`src/app/cull.rs:322-334`, `:468-478`). Only *Move to…* and *Copy
to…* set `self.asking`, the one field the panel is drawn from (`:296-315`,
`:337-340`).

The whole program contains two `egui::Slider` widgets — the stack tolerance
(`src/ui/filter_bar.rs:147`) and the zoom percentage
(`src/view/image_view/bottom_bar.rs:262`) — against seventeen `DragValue`s, of
which twelve are under `src/view/organize/`: nineteen numeric controls in
40,504 lines, and §5 has what that vocabulary should become. The slideshow
window's motion radios, each with a sentence of its own underneath
(`src/app/panels.rs:110-117`, text at `src/config/mod.rs:503-524`), are the best
control in the program, and the only place a configuration enum is drawn at all.
Four more enums in the source carry an `ALL` list, a `label()` or both, written
out and referenced by nothing but their own tests: `Prefer`
(`src/organize/pairs.rs:44-53`), the type behind `raw.pair_with_jpeg`, whose
labels read "Show both", "Show the JPEG" and "Show the raw"; `Corner`
(`src/view/image_view/overlay.rs:35-41`, `:57-65`), the type behind
`image_view.overlay_corner`; `Badges` (`src/view/grid_view/cell.rs:34`); and
`Overlay` (`src/decoder/overlays.rs:33-41`), whose "Off", "Clipping" and "Focus
peaking" would name what `C` has just done. `RawSource` and `RawQuality` have
neither (`src/config/mod.rs:102-120`) and are plain JSON strings. The labels a
settings window needs are, in four cases out of six, already written.

And right-click does nothing. Two surfaces register a menu through the same
function — the photograph (`src/view/image_view/mod.rs:170` →
`interaction.rs:70-79`) and a contact-sheet cell
(`src/view/grid_view/mod.rs:460-464`) — from a configured `context_menu` list
that `default_ctx_menu()` returns empty (`src/config/defaults.rs:165-167`), and
`show_context_menu` returns before drawing anything when the list is empty
(`src/actions/user_action.rs:147-149`). A third surface, the zoom percentage,
carries the only menu that exists on a default install: nine entries, four fit
commands and five magnifications (`src/view/image_view/bottom_bar.rs:283-303`,
percentages at `:12`), reached by right-clicking a forty-five-point label in the
bottom-right corner (`:279`). It is good, and nothing suggests it is there. A
fourth surface reads the button and does something else with it: a right-click
on a directory-tree row **opens** the folder (`src/ui/tree.rs:266`), which is the
only `secondary_clicked()` call site in `src/`. §7 has the full map of what each
surface should offer.

Two smaller absences belong here. There is no double-click handling anywhere —
`double_clicked` has zero occurrences in `src/` — and no mouse button can be
reassigned, because `PointerButton` occurs nowhere either (§9). And nothing the
user arranges by hand survives the session: `eframe` is built with
`features = ["wgpu"]` alone (`Cargo.toml:13`), the `persistence` feature is
absent, and `impl eframe::App for App` (`src/app/mod.rs:790`) implements
`on_exit` (`:866`) but no `save`. Both side panels are `resizable(true)`
(`src/app/chrome.rs:107`, `src/ui/tag_panel/mod.rs:62`); a dragged width lives in
egui's own memory, which without persistence is never written down. Neither
panel has a way to close it with the mouse, either: there is no close control in
`src/app/chrome.rs:99-128` or `src/ui/tag_panel/mod.rs:52-97`, and egui's
`SidePanel` provides none of its own.

### 1.6 The five verbs, and where each one stands

| | Where it stands |
|---|---|
| **Browsing** | Everything works and nothing announces itself: the menu bar, the side panel, the tag panel, the filter bar, the filmstrip and the directory tree are each behind one key, the window opens with all of them closed (`src/app/mod.rs:200-235`), and the title bar never names the folder (`src/main.rs:52`). |
| **Marking** | The three axes are complete and the panel that shows them is well made, but it is behind `K` (`src/config/defaults.rs:355-357`), it has no close button, and it returns before drawing anything when no photograph is open, so on an empty folder the key does nothing at all and says nothing (`src/app/tagging.rs:98-101`). |
| **Sorting** | The strongest part of the interface — the filter bar is the most explained surface in the program (§10) — and the one part with no memory: the seven filter rules (`src/view/narrow.rs:22-34`), the sort key and the stacking settings are `Default::default()`-ed every launch and have no configurable defaults (`src/app/mod.rs:228-229`, `src/view/narrow.rs:36-48`, `:138-144`). §6 gives them fields. |
| **Configuring** | Sixty of the 110 settings are keys, three are the slideshow, and forty-seven are a JSON file whose location the program never puts on screen — twenty-six of which need a relaunch that nothing mentions. |
| **Learning** | 33 tooltips in 40,504 lines, no Help menu, no About, no version, no path to the configuration or the log, and a cheat sheet that is genuinely good and is opened by a key nothing on screen mentions (`src/app/input.rs:113-115`). |

### 1.7 The thesis

The engine is finished and the feature set is finished. What is left is not a
missing capability but a missing account of itself: a program with six modes,
sixty-nine rows in its keyboard registry, a hundred and ten settings and eleven
menu items, which opens on three words of grey text and explains none of it.
Every serious piece of work it can do — developing a raw file rather than
showing the camera's preview, sizing the thumbnail cache to the card in the
machine, naming the folders that culled frames are moved to, choosing what the
caption under a thumbnail says — requires closing the program, finding a JSON
file whose location it will not tell you, editing it correctly at the first
attempt because a bad value is reported only to a log whose location it will not
tell you either, and starting again. The fix is not more features. It is a
settings window that covers every field with a real control rather than a preset
(§3, §4, §5), fields for the constants that never got one (§6), right-click
menus on the things that have settings (§7), a route from a photograph to the
thing that explains it (§8), a mouse and a keyboard that are documented where
they are used (§9, §10), and enough on screen at launch for somebody to find the
second thing without being told the first. The rest of this plan is that work,
in the order §12 sets, and none of it touches the decoder.

## 2. What people complain about, and whether it happens here

The project owner asked for the common problems people have with programs of
this kind to be looked up and then checked for in this one. This chapter is
that check. Two hundred and sixty-eight distinct issue threads, forum posts,
reviews and vendor documents were gathered across nomacs, qimgv, ImageGlass,
JPEGView, XnView MP, Geeqie, IrfanView, FastStone, darktable, digiKam,
Lightroom Classic, Lightroom cloud, Adobe Bridge, Capture One, Photo Mechanic,
FastRawViewer and Aftershoot, together with the published interface guidance
from Microsoft, Apple, GNOME, Android, Nielsen Norman and two CHI papers. Every
complaint below carries the URL it came from. Every statement about avis-imgv
carries a file and a line that was read. What the interface is today, in its own
terms, is §1; what to do about each finding belongs to §3 onwards, and this
chapter points at the chapter that owns each repair rather than proposing it a
second time.

Themes are ordered by how many independent sources raise them, most first. Each
table's **Verdict** column takes one of exactly three values:

- **Same** — the complaint applies here for the same reason it applies there.
- **Not here** — it does not, and the code says why.
- **Worse** — it applies here more severely than in the program complained about.

Two caveats on the evidence, recorded because they change how much weight the
counts deserve. Reddit is unreachable to the tooling that gathered this, so
r/photography and r/software are represented only by substitutes. And a
complaint count measures how loud a project's users are, not how many users it
has: nomacs supplies a disproportionate share below because it has an open
tracker and a maintainer who argues in it.

Counting the verdicts before reading them: of the seventy-four rows in this
chapter, twenty-three are **Same**, forty are **Not here** and eleven are
**Worse**. The shape of that is the finding, and it is not flattering in the way
a summary usually is. What avis-imgv gets right is almost all in the file layer
— sidecars, pairing, atomic writes, the undo journal, the section-by-section
configuration reader. What it gets wrong is almost all in the layer above:
nothing that lets a person reach any of it. It has built a careful engine and
left the controls off.

---

### 2.1 Settings that do not survive

The highest-volume theme in the whole corpus: more than a dozen threads across
six projects and seven years. The shape is always the same — a person configures
the program, and the configuration is gone.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **An update erases everything.** "Can you stop deleting my settings every time there's an update?" — closed as not planned. | [ImageGlass #2041](https://github.com/d2phap/ImageGlass/issues/2041); the same again in [#1958](https://github.com/d2phap/ImageGlass/issues/1958) (7 reactions, `igconfig.json` resets about weekly), #659, #600. Adobe's version has 24 replies: *"everytime Photoshop updates it erases all of my customizations"* — [community.adobe.com](https://community.adobe.com/questions-712/please-stop-making-updates-erase-settings-1168058) | **Not here** | The file carries a version — currently 1 — and is migrated forward one step at a time, and a step only fires on a file old enough to need it and only when what it finds is still the *old default* (`src/config/migrate.rs:25`, `:55`, `:81`–`88`). What was moved is said out loud in the corner at startup (`src/app/mod.rs:244`). |
| **The config file is regenerated from scratch on exit, so keys the build does not recognise are stripped.** Roll back a version to test something and every newer setting is gone. *"a direct consequence of the approach of regenerating config file from scratch each time Geeqie is closed."* | [Geeqie #569](https://github.com/BestImageViewer/geeqie/issues/569) | **Same** | Read directly rather than inferred: `from_json` names `version` and eight sections and builds a `Config` out of those alone (`src/config/load.rs:125`, `:145`–`159`), so any other top-level key in the document is discarded on the way in. Inside a section the same happens by serde default — no struct in the crate carries `deny_unknown_fields`. `save` then serialises the struct (`src/config/load.rs:42`), so what was dropped is gone. It only bites when the file is rewritten — on a migration (`src/config/load.rs:100`) and on any change made in the keyboard editor (`src/app/settings.rs:104`) — but that is Geeqie's bug with a smaller blast radius. |
| **One malformed line costs the whole file.** The advice given to an ImageGlass user was literally *"delete `igconfig.xml`"*. | [ImageGlass Google Group](https://groups.google.com/g/imageglass/c/eegkAKlKRCw) | **Not here** | The document is read section by section, so a section that will not parse costs that section and nothing else (`src/config/load.rs:125`, `:166`–`182`). A file that was only partly understood is never written back over (`src/config/mod.rs:42`, `src/config/load.rs:27`), and the user is told (`src/app/mod.rs:248`). A byte order mark from a Windows editor is tolerated (`src/config/load.rs:131`). This is the best-engineered part of the configuration by a distance. |
| **The app cannot write its config and says nothing.** Wrong permissions on `AppData/Roaming/XnViewMP`: *"I set them, then the next time I use XnView, I have to set them again."* | [XnView t=46390](https://newsgroup.xnview.com/viewtopic.php?t=46390); silently written to `VirtualStore` in [t=35049](https://newsgroup.xnview.com/viewtopic.php?t=35049) | **Not here** | A failed write is reported on screen, not only logged (`src/app/settings.rs:105`–`108`). |
| **A crash mid-write leaves half a config file.** Implied by every thread above; nobody names it because they cannot see it. | — | **Same** | Sidecars are written to a temporary and renamed over the original (`src/annotations/sidecar.rs:88`). Neither of the other two files the viewer writes is: the configuration is a plain `fs::write` (`src/config/load.rs:45`) and so is the session (`src/session.rs:110`). The keyboard map and the per-folder positions are the two things a person cannot rebuild, and they are the two written without a rename. |
| **No way back to the defaults.** *"The preferences are too complicated for my limited brain power and time and I have made zero progress in restoring them to the default"* — uninstalling and reinstalling did not help, because the config outlived the uninstall. The author's answer to a reset request was *"It's the same as deleting xnview.ini, is it really needed?"* | [XnView t=40425](https://newsgroup.xnview.com/viewtopic.php?t=40425), [t=4350](https://newsgroup.xnview.com/viewtopic.php?t=4350); [nomacs #1321](https://github.com/nomacs/nomacs/issues/1321) — "reset all settings" exists and is incomplete | **Same** | There is one reset button and it covers key bindings only: *"Put everything back to the defaults"* walks the registry and writes each default back (`src/ui/keys.rs:111`–`121`). The other fifty settings have no reset, per-setting or global, and nothing anywhere shows which of them differ from the default. darktable marks a changed setting with a bullet and resets it on a double-click of its label ([manual](https://darktable-org.github.io/dtdocs/en/preferences-settings/overview/)); VS Code has a per-setting gear and an `@modified` filter ([docs](https://code.visualstudio.com/docs/configure/settings)). Both belong to the settings window, which is §3's. |
| **Key bindings are not covered by "Export Settings".** | [nomacs #328](https://github.com/nomacs/nomacs/issues/328) | **Not here** | There is nothing to leave out: the bindings are fields of the same eight sections as everything else, in the one `config.json` (`src/config/mod.rs:18`–`33`, `:147`–`172`, `:389`–`433`). Copying that one file carries the whole configuration. |
| **The config has absolute paths baked into it, so it does not travel between machines.** | [digiKam bug 267131](https://bugs.kde.org/show_bug.cgi?id=267131) | **Not here** | A destination path may be relative and is then taken against the open folder, so a configured `Selects` follows the shoot (`src/config/mod.rs:219`–`221`, applied at `src/app/cull.rs:392`–`395`). A relative `catalog_file` is taken against the configuration file (`src/annotations/catalog.rs:184`–`193`). |

---

### 2.2 Settings that exist only in a configuration file, and settings nobody can find

Fourteen sources across two research passes, and it is the single largest gap
between what this program can do and what a person can reach.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **The manual tells you to hand-edit the config file for a setting that has no interface.** darktable's own manual: *"You can set the period of inactivity … by altering the `backthumbs_inactivity` setting in `darktablerc`"*. And on the file itself: *"a text file of over 1000 lines"*, with a community reply that *"There will be settings in `darktablerc` that you won't find on the UI"*. | [docs.darktable.org](https://docs.darktable.org/usermanual/development/en/preferences-settings/lighttable/), [discuss.pixls.us/t/37056](https://discuss.pixls.us/t/explanation-for-darktablerc-parameters-needed/37056); IrfanView's FAQ does the same for `INI_Folder` ([irfanview.com/faq.htm](https://www.irfanview.com/faq.htm)) | **Worse** | Counted field by field: there are 110 settings, and §3 has the inventory. Sixty are shortcuts (`src/config/mod.rs`, sixty fields matching `pub sc_`); the keyboard editor reaches all sixty, as sixty-nine rows, because the ratings and the labels are lists — fifty-eight fixed bindings plus five labels plus six ratings (`src/config/bindings.rs:85`, `:420`–`436`). Of the other fifty, the slideshow window reaches three: hold time, motion and the zoom creep (`src/app/panels.rs:100`, `:111`, `:125`). **Forty-seven have no control that writes them.** Six of the forty-seven can be nudged for the session and revert at the next launch: `overlay_corner` (`src/view/image_view/mod.rs:279`), `nr_images_shown` (`:284`, `:289`), `images_per_row` (`src/view/grid_view/mod.rs:515`–`527`), `advance_after_marking` (`src/app/mod.rs:706`), whether the filmstrip is up (`:713`), and the tag panel's width as a dragged splitter (`src/ui/tag_panel/mod.rs:62`–`64`). **The remaining forty-one can only be changed by editing the file.** The Settings menu has exactly two entries, `Keyboard…` and `Slideshow…` (`src/app/panels.rs:67`–`77`). darktable at least has a preferences window for the settings that are not in `darktablerc`; there is nothing here for the other forty-one to sit outside of. |
| **"I would like to be able to change the file sort order on the fly without having to edit the INI file."** JPEGView is the viewer that stayed INI-first, and this is the request it gets. | [sourceforge.net/p/jpegview](https://sourceforge.net/p/jpegview/discussion/693396/thread/b398fc31/) | **Same** | Sort order while browsing is in the filter bar and is not a config setting at all (`src/ui/filter_bar.rs:249`), so that specific ask is met. But `cell_aspect`, `thumbnail_resolution`, `caption_format`, `overlay_format`, `metadata_tags`, `destinations`, `rejected_folder`, `catalog_file` and the whole cache budget are file-only, and `images_per_row` is the halfway case that is worse than either: a key or `Ctrl`+wheel changes it (`src/view/grid_view/mod.rs:515`–`527`), `set_columns` writes it nowhere but the view (`:643`–`646`), and the next launch reads the file again (`:77`). |
| **A typo in a text config breaks everything, and there is no way to know what the valid values are.** *"A single typo, or a stray comma could break everything."* … *"What are valid options for 'runs-on'? Where would I find them?"* … *"have a `sudo` mode which lets users hand-stitch a config together … But have a GUI first."* | [shkspr.mobi](https://shkspr.mobi/blog/2020/06/theres-nothing-i-hate-more-than-text-config-files/) | **Same** | The typo half is handled: a bad section costs that section (`src/config/load.rs:166`). The "what are the valid values" half is not. `raw.source`, `raw.quality` and `raw.pair_with_jpeg` are closed sets of strings that nothing in the program enumerates (`src/config/mod.rs:80`, `:104`–`122`, `src/organize/pairs.rs:32`–`42`); their members exist only in the README and in `examples/config.json`, which lists 103 of the 110 fields. `Prefer::ALL` and `Corner::ALL` already carry a human label for every member and are used by nothing but their own tests (`src/organize/pairs.rs:411`, `src/view/image_view/overlay.rs:153`). The one closed set that *is* enumerated on screen is `slideshow.motion`, as radio buttons with a sentence each (`src/app/panels.rs:111`, `src/config/mod.rs:503`–`524`), which is the shape the other four should take — §5 owns which control each of them gets. |
| **The config file cannot be opened from inside the program.** ImageGlass's layered JSON design is described the right way round: the GUI is the primary editor and the file is the escape hatch. | [imageglass.org/docs/app-configs](https://imageglass.org/docs/app-configs) | **Same** | `Config::path()` exists (`src/config/load.rs:14`) and has two callers outside `config/`, neither of them the interface: the catalog's relative-path resolution (`src/annotations/catalog.rs:190`) and a test in the logging module, which asserts that the log sits beside the configuration (`src/logging.rs:191`). No menu item opens the file, reveals its folder, or opens the log — which `logging::path()` puts in that same folder (`src/logging.rs:37`–`40`) and which the program names once at startup, in the log itself and nowhere a person will look (`src/main.rs:34`), so one command would reveal both. And the file a new user finds there is not the pretty-printed one the code's own doc comment promises (`src/config/load.rs:19`–`23`): `save` uses `to_string_pretty` (`:42`), but the first-run path still writes `to_string` (`:69`, written at `:84`), so a stock `config.json` is one long line until something rewrites it. The file section of the settings window is §3's. |
| **The documentation names a location that does not match reality.** *"I'm sorry, but I didn't find the network preferences"* — the manual named a screen that does not exist. XnView's version: Edit ▸ Preferences does nothing, because there is no such item; the thing wanted was under Settings ▸ General. | [nomacs #1260](https://github.com/nomacs/nomacs/issues/1260), [XnView t=42284](https://newsgroup.xnview.com/viewtopic.php?t=42284) | **Same** | `README.md:594` gives the configuration as `~/.config/avis-imgv/config.json` and gives no other path. On this machine it is `%APPDATA%\avis-imgv\avis-imgv\config\config.json` — checked on disk — because the organisation and application names are both `avis-imgv` (`src/lib.rs:35`–`36`) and `directories` puts one inside the other. A Windows user following the README will not find the file. |
| **The setting exists and nobody can find it.** Seven years of argument about nomacs's scroll wheel ended with a collaborator pointing out that the two checkboxes to swap it had been there the whole time. | [nomacs #237](https://github.com/nomacs/nomacs/issues/237) | **Worse** | The equivalent here is `image_view.scroll_navigation` (`src/config/mod.rs:374`, default `true` at `src/config/defaults.rs:136`), read once a frame and reachable from nowhere (`src/view/image_view/interaction.rs:18`). nomacs at least had a checkbox that could be found by looking. Here there is nothing to find. The wheel itself is §9. |
| **The preferences window needs a search box.** Five separate darktable issues by five people, all closed by the stale bot. *"Many programs that have complex settings … allow searching for names of settings."* Android's guidance: *"For complex or deep settings hierarchies, add search functionality."* | [darktable #3423](https://github.com/darktable-org/darktable/issues/3423), [#6706](https://github.com/darktable-org/darktable/issues/6706), [#6174](https://github.com/darktable-org/darktable/issues/6174), [#3604](https://github.com/darktable-org/darktable/issues/3604), [#9598](https://github.com/darktable-org/darktable/issues/9598); [developer.android.com](https://developer.android.com/design/ui/mobile/guides/patterns/settings) | **Same** | Nothing here is searchable. The keyboard editor draws its sixty-nine rows in four fixed sections with no filter field (`src/ui/keys.rs:78`–`106`, `src/config/bindings.rs:67`). Two of those darktable issues ask specifically for search over *shortcuts*, by action and by key, which is precisely the window this program has. §3 specifies the search field. |

The tension worth naming, because the same corpus contains the opposite
complaint. IrfanView's reviewers: *"Way too many options and configurability.
It gets confusing quick"*, *"too many features"*, *"my eyes hurt"*
([snapfiles.com](https://www.snapfiles.com/userreviews/101840/irfan.html)).
Havoc Pennington's essay, which every open-source maintainer cites: *"Too many
preferences means you can't find any of them"*
([ometer.com](https://ometer.com/free-software-ui.html)). Nielsen Norman
measured it: interface customisation reached 83% task success, product
customisation 66%, and about 45% of the failures were findability
([nngroup.com](https://www.nngroup.com/articles/customization-of-uis-and-products/)).
The reconciliation visible in the threads is that people complain about option
*count* when options are flat, undifferentiated and unsearchable, and about
option *absence* when the one behaviour they want to change is locked. Nobody in
this corpus complains about a setting that is easy to reach and obviously named.
So the answer to forty-one file-only settings and six that quietly revert is not
forty-seven checkboxes in a flat list; it is a small number of named pages,
grouped by what the person is trying to do, with a search field over all of them.
Where the line between too many and too few actually falls is §11's argument;
this chapter supplies its evidence. Which pages there are, and which field lands
on which, is §4.

---

### 2.3 The mouse: the wheel, the right button, and the second click

The loudest single complaint in the viewer corpus, sustained over seven years
across at least eight accounts, with users saying they uninstalled over it. The
mouse itself is §9's chapter and the menus on the second button are §7's; what
follows is only whether each complaint lands here.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **The wheel does the wrong thing and cannot be rebound.** *"I'm happy even if these settings aren't made default, just being able to bind it would be more than enough."* … *"won't function in any way except counter to every single popular graphics editor … Uninstalled in two minutes."* … *"make them all configurable so that users can always set it up the way they prefer."* The follow-up asks for mouse inputs to live *in the shortcut manager* alongside the keys. | [nomacs #237](https://github.com/nomacs/nomacs/issues/237) (32 comments, open since 2018), [#347](https://github.com/nomacs/nomacs/issues/347) | **Worse** | Not quite as bare as it first looks, and still worse than nomacs. `Ctrl`+wheel already zooms the photograph — egui's default `zoom_modifier` is `COMMAND`, so it arrives as a zoom event rather than a scroll, and the view routes it through the same command as the keys (`src/view/image_view/interaction.rs:26`–`29`). Turning `scroll_navigation` off leaves the plain wheel panning, because the pan delta is applied regardless (`interaction.rs:40`–`45`). So there are three wheel behaviours and exactly one boolean in the JSON to choose between two of them (`interaction.rs:18`, `src/config/mod.rs:374`). What cannot be done at all: swap which of them is on `Ctrl`, put "next image" anywhere but the bare wheel, bind the middle button or the side buttons, or see any of it in the keyboard editor — the registry is sixty-nine keyboard rows and nothing else (`src/config/bindings.rs:85`, `:420`–`436`). nomacs at least shipped two checkboxes and a settings page nobody could find; here there is neither. The `mouse.*` fields that would fix it are §6; the behaviour is §9. |
| **Gestures wired up to the wrong axis.** *"Two-finger left/right swipe has no effect. Pinch-to-zoom displays the previous/next image."* Four separate ImageGlass issues over eight years. | [ImageGlass #951](https://github.com/d2phap/ImageGlass/issues/951), #305, #497, #686 | **Not here** | Pinch is routed through the same command as the keys, so it holds the point under the fingers (`src/view/image_view/interaction.rs:26`–`29`), and the navigation path explicitly stands down while a pinch or a `Ctrl`+wheel zoom is in progress (`src/view/image_view/input.rs:193`). What is *not* clean: with `scroll_navigation` on, one wheel notch is read twice. `input::scroll_navigation` reads `raw_scroll_delta.y` without consuming it (`input.rs:197`), and `smooth_scroll_delta` is then written into the viewport (`interaction.rs:40`–`45`) — after the navigation has already changed the image. egui drains a line-unit notch into `smooth_scroll_delta` over several frames, so the shove lands on the frame just arrived at and goes on for a moment after. It only moves anything when that frame is zoomed past fit, because pan clamps to zero otherwise (`src/view/image_view/canvas.rs:337`–`338`) — but that is exactly the case somebody scrolling a zoomed photograph is in. §9 owns the repair. |
| **"I want to drive everything from the right mouse click and never touch the keyboard."** | [discuss.pixls.us/t/58799](https://discuss.pixls.us/t/another-dt-ui-discussion/58799) | **Worse** | Three surfaces register a right-click menu — the photograph (`src/view/image_view/interaction.rs:76`), a contact-sheet cell (`src/view/grid_view/mod.rs:461`) and the zoom percentage (`src/view/image_view/bottom_bar.rs:283`) — through two `response.context_menu(…)` registrations in the whole source (`src/actions/user_action.rs:152`, `bottom_bar.rs:283`). Only the last of the three draws anything on a fresh install: `default_ctx_menu()` returns an empty vector (`src/config/defaults.rs:165`–`167`) and `show_context_menu` returns before drawing when the list is empty (`src/actions/user_action.rs:147`–`149`). **Right-clicking the photograph or a cell therefore does nothing at all out of the box**, and the only way to make it do anything is to hand-write external shell commands into `image_view.context_menu` in the JSON. Not one of the program's own commands — rate, reject, label, move, copy, delete, open in the gallery, compare, stack — is reachable from a right click. The one menu that exists has nine entries, all of them about zoom (`bottom_bar.rs:12`, `:284`–`303`). §7 has the full map of what should be on which surface. |
| **Inconsistent left and right click reads as hidden.** *"The inconsistency in using left and right mouse clicks … every time I have to try and figure out if I should click this icon one way or the other."* | [discuss.pixls.us/t/56268](https://discuss.pixls.us/t/darktable-ui-work/56268) | **Worse** | In the directory tree, left-click expands or collapses a folder and **right-click opens it** (`src/ui/tree.rs:266`–`268`). It is the only `secondary_clicked()` in the whole source, it is the only right click anywhere that does something by default besides the zoom readout, and nothing says so: the README's single mention of right-click is the empty user-action menu (`README.md:877`). `PointerButton` appears nowhere in `src/`, so no code distinguishes which button is dragging either. Every convention in the context-menu research puts a menu on the second button, not a primary action. |
| **Nobody asks to edit the context menu; they ask for the same action not to be in four places.** *"the 3 icons on the thumbnail panel are redundant … the same actions are already in two other places plus hotkeys."* | [qimgv #37](https://github.com/easymodo/qimgv/issues/37) | **Not here** | The opposite problem obtains: most commands have exactly one route, and it is a key. Several routes to one command is what this plan argues for, and the corpus does not contradict it — the qimgv complaint is about *icons on a panel*, not about a command being on both a key and a menu. |
| **Double-click is asked for repeatedly, and the filings disagree about what it should do.** Two closed ImageGlass issues want it on fullscreen — one of them wanting the second double-click to come back out again — while a third records that ImageGlass already uses it for the actual-size toggle and asks for single click to do the same. Picview ships the gesture as a four-way setting, *"Toggle Zoom (default), Full screen, Close window, None"*; XnView as a *"Switching mode"*; and IrfanView's author refuses to make it configurable at all, on the ground that *"more options are not always a good move as they make programs harder to support"*. | [ImageGlass #648](https://github.com/d2phap/ImageGlass/issues/648) *"Left double-click = Full Screen"*, [#909](https://github.com/d2phap/ImageGlass/issues/909), [#381](https://github.com/d2phap/ImageGlass/issues/381); [picview.chitaner.com](https://picview.chitaner.com/blog/mouse_keyboard_trackpad/); [XnView t=29695](https://newsgroup.xnview.com/viewtopic.php?t=29695). The IrfanView sentence is second-hand, from search results: the thread itself returns HTTP 502 (`irfanview-forum.de/forum/program/support/11816-`). | **Worse** | `double_clicked` appears nowhere in `src/`, so none of the positions in that argument is held here: the gesture is not bound to anything at all. Double-clicking the photograph does nothing; double-clicking a grid cell does what a single click does (`src/view/grid_view/mod.rs:444`); double-clicking a slider or a value does not reset it, which is the darktable idiom users call *"the biggest 'Aha moment'"* ([darktable.info](https://darktable.info/en/system-ui-2/shortcuts-keyboard-layout/shortcuts-hidden-features/)). Nor are the mouse's fourth and fifth buttons read: with no `PointerButton` in `src/`, the back and forward buttons on a normal mouse are dead. |

---

### 2.4 Sidecars, colour labels and keywords: whose truth is it

Thirteen sources. This is the theme where avis-imgv is furthest ahead of the
field, and it is worth being specific about why, because the reasons are all in
files that already exist.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **Colour labels are text, and the text has to match exactly between apps.** *"When you assign a color label you aren't actually assigning a color, but rather text in metadata."* Bridge's defaults are Select, Second, Approved, Review and To Do rather than colour names — Bridge's red is "Select" — and Lightroom shows a white swatch for anything that does not match its own set. Camera Bits: *"the text label for each class must be exactly the same in each app."* | [asktimgrey.com 2017](https://asktimgrey.com/2017/02/14/white-color-labels/), [asktimgrey.com 2022](https://asktimgrey.com/2022/07/12/color-label-mismatch/), [camerabits.freshdesk.com](https://camerabits.freshdesk.com/support/solutions/articles/48000223643-using-star-ratings-and-color-classes-with-adobe-lightroom-and-other-apps) | **Not here** | Setting a label from the interface stores the canonical English name whatever the interface ever says (`src/annotations/mod.rs:105`, `:121`), and that is what reaches `xmp:Label` (`src/metadata/xmp/write.rs:210`). Reading matches an alias table as well as the canonical name (`src/metadata/xmp/mod.rs:153`–`173`), and a string that matches nothing is kept as it is rather than thrown away (`src/metadata/xmp/mod.rs:184`–`186`). |
| **Case is treated as a difference.** A user's Bridge labels were lowercase `select`, Lightroom's were `Select`, and every label came into Lightroom white until the case was made to match by hand. | [lightroomqueen.com](https://www.lightroomqueen.com/community/threads/mismatch-between-bridge-and-lr-classic-color-labels.41913/) | **Not here** | Matching is `eq_ignore_ascii_case` on both the canonical name and every alias (`src/metadata/xmp/mod.rs:157`–`161`). |
| **…except that the alias table contradicts itself.** | Found here. | **Worse** | `"To Do"` is listed as an alias of **both** `Red` (`src/metadata/xmp/mod.rs:168`) and `Purple` (`:172`), and `Label::of` walks `CHOICES` in order and returns the first match (`:153`–`163`), so Purple's entry is unreachable and a "To Do" label always draws red. One of the two is wrong, and it is the one on Red: the same table already gives Red its documented Bridge name, "Select" ([asktimgrey.com](https://asktimgrey.com/2017/02/14/white-color-labels/)). One line. Worth a second look while that line is being written: the other half of the table — In Progress, Done, On Hold (`:169`–`172`) — is attributed to Lightroom by the code's own comment and appears in no source in this corpus, so it should be checked against a real Lightroom label-set export before anything is built on it. |
| **Hierarchical keywords flatten or duplicate between programs, and the exact XMP container matters.** darktable wrote `rdf:Seq` where Lightroom and exiftool write `rdf:Bag`, and digiKam collapsed the hierarchy on import — *"every level of the branch and leaves are selected"*. Tools that read only `dc:subject` see the flat form. | [darktable #4095](https://github.com/darktable-org/darktable/issues/4095), [discuss.pixls.us/t/38824](https://discuss.pixls.us/t/hierarchical-keywords/38824), [docs.digikam.org](https://docs.digikam.org/en/setup_application/metadata_settings.html) | **Not here** | `rdf:Bag` (`src/metadata/xmp/write.rs:270`, `:296`), and both `dc:subject` and `lr:hierarchicalSubject` are written, deliberately beside each other rather than instead of each other (`src/metadata/xmp/write.rs:231`, `:236`–`245`). |
| **Sidecar naming is the classic interop trap.** Lightroom and Photoshop write `DSC1234.xmp`; darktable, exiftool and digiKam write `DSC1234.NEF.xmp`. digiKam needs an explicit "compatible with commercial programs" checkbox to bridge them, and when both files exist digiKam reads the wrong one first so the other's updates never land. | [discuss.pixls.us/t/32422](https://discuss.pixls.us/t/darktable-xmp-sidecar-tags-not-being-read-by-digikam/32422) | **Not here** | Both forms are read, the specific one first, and a write edits whichever file already exists rather than adding a second beside it (`src/annotations/sidecar.rs:29`–`43`, `:61`–`64`) — so the digiKam precedence bug cannot happen. What is missing is digiKam's checkbox: a *new* sidecar is always written in the `photo.jpg.xmp` form (`src/annotations/sidecar.rs:18`–`23`), and there is no setting anywhere to make it the Adobe form. A Lightroom-only photographer's ratings will not be seen. The field that does not exist yet is §6's. |
| **"Save metadata to files" being a manual step.** A folder showed 3 three-star frames in FastRawViewer and 31 in Lightroom, because Lightroom does not write rating changes to XMP even with "automatically write changes" on. FastRawViewer historically kept two tag sets and preferred its own. | [fastrawviewer.com/node/360](https://www.fastrawviewer.com/node/360) | **Not here** | There is no database and no second tag set. Every mark is queued to a sidecar as it is made (`src/annotations/writer.rs:1`–`5`), and a write that fails reaches the person who pressed the key rather than a log line (`src/ui/notice.rs:1`–`9`). |
| **A sidecar is a shared document and gets truncated or replaced.** A darktable develop history, Capture One's `.cos` "malformed junkola", residual `acdsee:` fields resurrecting tags after every rewrite. | [discuss.pixls.us/t/38824](https://discuss.pixls.us/t/hierarchical-keywords/38824) | **Not here** | An existing document is edited event by event with everything unrecognised passed through untouched, and when it cannot be rewritten safely the writer returns nothing and the file is left alone rather than replaced (`src/metadata/xmp/write.rs:32`, `:46`, `:112`). The result is written to a temporary and renamed over the original (`src/annotations/sidecar.rs:88`). |
| **Orphaned sidecars accumulate and nobody's tool cleans them up.** *"I have a lot of orphaned XMP files that were attached to raw files that I've deleted."* The answers were bash scripts. A third-party tool exists for nothing else. | [discuss.pixls.us/t/47345](https://discuss.pixls.us/t/clean-up-orphaned-xmp-files/47345), [photo-sidecar-cleaner](https://github.com/sisyphosloughs/photo-sidecar-cleaner) | **Not here** | Binning or deleting a photograph takes its sidecars with it (`src/organize/files.rs:126`–`134`, `:139`–`147`), and moving or copying one takes them along (`:23`, `:91`). The Adobe-form sidecar is only followed when no other image in the folder shares the stem, so renaming the JPEG of a pair cannot walk off with the raw's ratings (`src/organize/files.rs:52`–`68`). |
| **Rejection has to round-trip.** Every program in the comparison writes `xmp:Rating = -1`. A rating filter — *"only show images with > 3 stars"* — has been open in nomacs since 2017. | [nomacs #138](https://github.com/nomacs/nomacs/issues/138) | **Not here** | Reject is `xmp:Rating = -1` (`src/metadata/xmp/mod.rs:75`, `:231`–`239`), pick is mirrored into `digiKam:PickLabel` (`src/metadata/xmp/write.rs:216`), and the filter bar has all seven rules, including a stars range, a flag rule and a label rule (`src/view/narrow.rs:22`–`34`). |

---

### 2.5 Features that exist and cannot be found

Thirteen sources, and the pattern is consistent enough to be a rule: across the
whole corpus, the complaint is almost never that the feature is missing. It is
that the route to it is hidden, or lies.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **A preference silently disables a core action, and the program only says so inside a settings screen.** A user who bought FastRawViewer to cull could not reject a frame; the shortcut list itself carried the note *"change disabled. b/c XMP Reject use is disabled"*. | [fastrawviewer.com/node/577](https://www.fastrawviewer.com/node/577) | **Worse** | `filmstrip_height` defaults to `0.0` (`src/config/defaults.rs:299`), and `show_filmstrip` returns immediately when the height is not positive (`src/app/views.rs:137`–`141`). `Ctrl+T` still flips the flag (`src/app/mod.rs:713`), still appears in the keyboard editor and still appears on the cheat sheet in every mode, with the description *"Show or hide the strip of thumbnails under the photograph"* (`src/config/bindings.rs:100`–`105`). **Pressing it does nothing, silently, on a stock install**, and the only cure is to hand-edit a number in the JSON. FastRawViewer at least printed the reason next to the key. egui has no disabled-hover text anywhere in this program to print it with — `on_disabled_hover_text` has zero call sites — which is §10's problem to solve. |
| **The single-key workflow the tool is for is behind a preference filed under "Accessibility", documented in a footnote.** | [docs.camerabits.com](https://docs.camerabits.com/support/solutions/articles/48000317837-keyboard-shortcuts-windows) | **Same** | `advance_after_marking` defaults to `false` (`src/config/defaults.rs:466`), which is the safe default and defensible. It can be turned on from a key (`Ctrl+Shift+A`, `src/config/defaults.rs:473`) and its state is named in the status bar as `Advancing` (`src/view/image_view/bottom_bar.rs:144`–`151`), which is more than a preference documented in a footnote does. But there is no settings window to find it in, and nothing on screen says the key exists. |
| **Put the keyboard shortcut in the tooltip.** Requested in 2007, re-requested in 2011, unresolved in 2012: *"the tooltip … would not just describe the function but also mention the keyboard shortcut"*, wanted as `Fullscreen (F11)`. Asked again for DEVONthink. Microsoft's own guideline says to do it: *"Whenever appropriate, make tooltips more helpful by providing keyboard shortcuts and default values … Put this additional information in parentheses."* Capture One shipped it as a feature. | [XnView t=13913](https://newsgroup.xnview.com/viewtopic.php?t=13913), [DEVONthink](https://discourse.devontechnologies.com/t/show-keyboard-shortcuts-in-tooltips/73248), [learn.microsoft.com](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tooltips-and-infotips) | **Same** | Thirty-three hovers in the whole tree — thirty-one `on_hover_text`, one `on_hover_text_at_pointer`, one `on_hover_ui` — and **not one gives the key for the control it is attached to**. Exactly one names a key at all, and only by accident: the File menu's "Send rejected to the bin…" says *"Every picture in this folder marked with X"* (`src/app/panels.rs:48`), which is the reject key only because the default happens to be `x` (`src/config/defaults.rs:446`). One other comes close and then does not: the stacked-folder badge says *"the key that opens it shows the rest"* without saying which key (`src/view/image_view/bottom_bar.rs:139`–`141`). The menu bar names no keys either, though `sc_next_mode` and `sc_toggle_gallery` both exist (`src/app/panels.rs:33`–`77`). §10 owns the hover policy. |
| **Half-tooltipped interfaces are worse than none.** *"because only some icons had tooltips, users stopped expecting them and missed the ones that existed."* Microsoft: *"If you provide tips for some objects, you should provide them for all similar objects."* | [nngroup.com](https://www.nngroup.com/articles/tooltip-guidelines/), [learn.microsoft.com](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tooltips-and-infotips) | **Same** | The distribution is exactly the failure mode: eight of the thirty-three are in the filter bar and seven in the group panel, against one in the whole contact sheet and none at all on the photograph, in the directory tree or in the navigator; the thirty-three sit in eleven of the 139 source files, and twenty-eight of the thirty-nine files that name an `egui::Ui` or `egui::Context` carry no hover at all. A person who learns from the filter bar that hovering explains things will hover in the gallery and learn nothing. §10 has the file-by-file breakdown. |
| **A shortcut is fine as a fast path and a defect as the only path.** *"Recognition decreases cognitive burden; it's much easier for users to recognize a visible, labeled icon or action than it is to recall a keyboard shortcut."* Accelerators are *"secondary ways of accomplishing the same task"*. | [nngroup.com](https://www.nngroup.com/articles/usability-heuristics-complex-applications/), [nngroup.com](https://www.nngroup.com/articles/flexibility-efficiency-heuristic/) | **Same** | Sixty-nine bound commands against eleven items in the whole menu bar, and only five of the eleven are commands: Open Folder, Open Files, Send rejected to the bin…, Keyboard…, Slideshow…, the other six being the mode list (`src/app/panels.rs:33`–`77`; §1). The cheat sheet, the tag panel, the side panel, the filmstrip, compare, the destinations panel, undo, select-all, the navigator, the directory tree, flatten and watch have no route but a key. So does the filter bar itself (`F3`), and everything inside it inherits that: the seven rules, the sort, the suspend toggle and the whole stacking group — which does have proper mouse controls, a "Stacks" toggle, "Fold all", the gap and the tolerance (`src/ui/filter_bar.rs:95`–`155`) — is behind a bar nothing on screen mentions. Firefox's telemetry is the caution against the opposite over-correction: menu-click counts *understate* how much a command is used, because items like New Tab are *"predominately driven by keyboard shortcuts"* — so a menu entry earns its place as the thing that teaches the key, not as the thing that gets clicked ([blog.mozilla.org](https://blog.mozilla.org/metrics/2010/03/15/menu-item-usage-study-part-i/)). |
| **A cheat sheet has to be filtered, not complete.** Krita's printed shortcut list runs to about 67 pages; the community's real answer is hand-curated per-workflow lists. Photo Mechanic's is a 13 MB PDF of 100+ shortcuts. | [krita-artists.org](https://krita-artists.org/t/keyboard-shortcut-cheat-sheet/8311), [docs.camerabits.com](https://docs.camerabits.com/support/solutions/articles/48000317837-keyboard-shortcuts-windows) | **Not here** | `?` draws a sheet generated from the registry, narrowed to the sections that apply to the current mode, showing the user's own keys rather than the documentation's (`src/ui/cheat_sheet.rs:28`–`36`, `:46`–`67`). That is ahead of both Krita and Photo Mechanic. |
| **…and the sheet's own discoverability is the whole problem.** A palette *"rewards the person who has memorized your app and does nothing for the person opening it on day one."* RStudio's answer is to put the palette in a menu as well as on a key. | [uxpatterns.dev](https://uxpatterns.dev/patterns/advanced/command-palette), [docs.posit.co](https://docs.posit.co/ide/user/ide/guide/ui/command-palette.html) | **Same** | The sheet opens on `?` and on nothing else. The key is hardcoded to `Key::Questionmark` (`src/app/input.rs:113`), so it is neither in the registry nor rebindable nor listed on the sheet it opens, and there is no Help menu, no About and no version string anywhere on screen — `CARGO_PKG_VERSION` appears nowhere in `src/`. The Help menu that would carry `Keys…  ?` is §10's. |
| **First-run tours do not work; empty states do.** Tutorials *"don't make users faster or more successful … they make them perceive the tasks as more difficult"* (SEQ 4.92 against 5.49, p=0.047). Empty states should *"help users discover unused features"* and *"provide direct pathways for getting started."* | [nngroup.com](https://www.nngroup.com/articles/mobile-tutorials/), [nngroup.com](https://www.nngroup.com/articles/empty-state-interface-design/) | **Same** | There is correctly no tour, and §13 records that as a refusal rather than an oversight. There is also nothing in the empty state: the gallery renders a bare centred label, `"No images here"` or `"Nothing matches the filter"`, with no count, no button to clear the rules and no route to opening a folder (`src/view/grid_view/mod.rs:263`–`272`). The second of those two strings appears at exactly the moment a new user has filtered themselves into a corner. |

---

### 2.6 What is remembered between one run and the next

Eleven sources across five projects, and it is the theme where a partial answer
does the most damage: Geeqie's users cannot form a model of what will survive,
because some things do and some do not. The rule that decides which of the three
kinds a given value is — setting, session state or ephemeral — is §4's.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **Window size and position are not remembered.** JPEGView sizes the window to each image and the filer had to pre-empt being dismissed for asking. lximage-qt's filer notes IrfanView already does it. | [JPEGView #276](https://github.com/sylikc/jpegview/issues/276), [#217](https://github.com/sylikc/jpegview/issues/217), [lximage-qt #122](https://github.com/lxqt/lximage-qt/issues/122) | **Not here** | Size, position and maximised state are written on exit and read before the window is made (`src/session.rs:33`–`51`, `src/main.rs:49`, `:96`–`110`), with a sanity check so a minimised window does not come back as an unfindable sliver. |
| **"How do I resize thumbnails and let nomacs remember this size as default?"** Open since December 2025. | [nomacs #1479](https://github.com/nomacs/nomacs/issues/1479) | **Same** | The column count is changed by key or by `Ctrl`+wheel and held in `GridView.columns` (`src/view/grid_view/mod.rs:515`–`527`, `:643`–`646`). It is never written back to `grid_view.images_per_row` and never put in the session, so it is read from the configuration at construction (`src/view/grid_view/mod.rs:77`) and reverts on every launch. The badge cycle is the same (`:530`–`532`), and so are the panes side by side (`src/view/image_view/mod.rs:129`, `:284`). |
| **Partial persistence is worse than none.** Geeqie's "Configure this window…" settings do not survive a restart while its general preferences do. Two Geeqie windows at once and whichever closes last wins the config file. | [Geeqie #966](https://github.com/BestImageViewer/geeqie/issues/966), [#1324](https://github.com/BestImageViewer/geeqie/issues/1324) | **Same** | `Session` holds exactly three things: the window, the folder, and the last photograph per folder (`src/session.rs:56`–`65`). Everything else the user sets during a session is a field on `App` and dies with it (`src/app/mod.rs:57`–`136`): the side panel, the menu bar, the tag panel, the metrics readout, the filter bar's visibility, the filter and sort rules themselves, whether stacks are on, whether the filmstrip is up, whether the folder is flattened, whether the watcher is running, and whether advance-after-marking is on. eframe is built without the `persistence` feature (`Cargo.toml:13`) and `impl eframe::App` implements only `update` and `on_exit`, no `save` (`src/app/mod.rs:790`), so egui's own memory — the side panel's dragged width, every window's position — goes too. |
| **Per-folder position is what actually matters in a cull.** Implicit in the Adobe undo thread: clearing a filter to fix a mistake *"returns you to the beginning rather than your position"*. | [community.adobe.com](https://community.adobe.com/feature-requests-681/p-undo-does-not-work-for-picking-and-rejecting-661142) | **Not here** | Sixty-four folders' positions are kept, most recent first (`src/session.rs:30`, `:118`–`142`), and re-opening a folder lands on the frame that was being looked at. Nothing else in the comparison was found doing this per folder. |
| **State information in the config file makes it un-versionable.** *"state information should not be saved in the config files."* | [nomacs #423](https://github.com/nomacs/nomacs/issues/423) | **Not here** | Session state is a separate `session.json` beside the configuration, and `config.json` is only written when a setting actually changes — from two places, both of them settings windows (`src/session.rs:69`–`71`, `src/app/settings.rs:70`, `:94`). The file can be put under version control. |
| **The session is restored even after the setting is turned off.** Not filed by anyone; found here. | — | **Worse** | `Session::load()` is called unconditionally before the window is built (`src/main.rs:49`) and its geometry is applied unconditionally (`src/main.rs:96`–`110`). `restore_session` is consulted only when deciding whether to *record* the position (`src/app/mod.rs:588`), whether to *write* the file (`:867`) and which folder to open (`:260`). Turning the setting off therefore stops the geometry being updated but does not stop it being applied, so the window keeps coming back at whatever size it was when the setting was last on. |

---

### 2.7 Destructive work: delete, the bin, and undo

Ten sources. The single angriest issue in the whole corpus is here, and so is
the one list that judges several programs at once against the same standard: of
nine Linux mobile viewers a user listed, three were disqualified over **delete**
and a fourth over having no bin, which is the largest single category in the
list.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **The program silently modifies files on disk.** Pressing R to rotate rewrites the file: *"the file on disk is silently modified. This changes the modification date on the file and changes the file size, all without the knowledge or consent of the user."* The reporter uninstalled from every machine they owned. | [nomacs #799](https://github.com/nomacs/nomacs/issues/799) | **Not here** | The viewer never rewrites a photograph. Marks go to sidecars beside the file (`src/annotations/sidecar.rs:1`–`5`), and the only operations that touch an original are the ones the user explicitly asks for: rename, capture-time shift, move, copy, delete. |
| **A confirmation dialogue is not a substitute for reversibility.** *"mistakes happen and if I hit delete (and after the dialog confirms it) the file is gone forever."* And the whole argument in miniature: the modal is hated (*"please make it so it can be permanently disabled"*), hold-to-confirm is ambiguous under key repeat (*"what happens if you hold it for longer than that?"*), and instant delete needs undo. The only answer everyone in that thread would have accepted is immediate and reversible. | [nomacs #480](https://github.com/nomacs/nomacs/issues/480), [qimgv #37](https://github.com/easymodo/qimgv/issues/37) | **Not here** | One photograph to the bin is done without asking, on the ground that the bin *is* the asking; a permanent delete, and any delete of more than one photograph, asks first and names the count (`src/app/cull.rs:65`–`91`, `:38`–`55`). `Enter` answers yes to the bin and does not answer yes to a permanent delete, which costs a key nobody presses by accident (`src/app/cull.rs:188`–`191`). Undo brings files back out of the platform's bin where the platform allows it, and says plainly that it cannot on macOS rather than pretending (`src/organize/journal.rs:220`–`262`). |
| **Three of nine viewers fail on the single verb "delete".** KDE Pix: *"no button to delete a picture"*. imv: *"(d)elete does not work"*. nsxiv: *"no delete"*. Four if FuriOS's *"no bin/no flatpak"* is counted. The other five are one each — scrolling, scaling, an onscreen keyboard, *"no interaction"*, *"does not fit mobile"* — five different problems against one repeated one. | [linmob.net](https://linmob.net/weekly-update-31-2025/) | **Not here** | `Delete` bins, `Shift+Delete` deletes outright (`src/config/defaults.rs:435`–`440`), and a menu entry collects every rejected frame in the folder and hands the lot to the same machinery (`src/app/cull.rs:99`–`120`, `src/app/panels.rs:46`). |
| **The record of the decision is destroyed while the files survive.** *"Lightroom got stuck on the delete and couldn't delete the photos but was MORE than happy to remove them from my catalog!"* — a complete re-cull. The community's advice was defensive: move rejects to a folder first. | [community.adobe.com](https://community.adobe.com/t5/lightroom-classic-discussions/lightroom-classic-continues-to-break-workflows-especially-when-deleting/m-p/12517273) | **Not here** | There is no catalogue to lose. A file that could not be binned is counted and reported and stays where it is (`src/app/cull.rs:217`–`266`), and the reject-to-a-subfolder idiom is there as `Shift+X` for the cards and shares the bin does not reach (`src/config/mod.rs:182`–`188`, `src/config/defaults.rs:413`, `src/app/cull.rs:323`). Multiple destination folders were a FastRawViewer feature request ([fastrawviewer.com/node/33](https://www.fastrawviewer.com/node/33)); the panel here takes nine and the digit keys reach them (`src/ui/destinations.rs:74`, `:122`–`138`) — though only two are configured out of the box (`src/config/defaults.rs:386`–`397`) and the list is one of the forty-one settings with no editor (§3). |
| **Undo does not cover the cull decision.** *"when picking or rejecting an image in Lightroom mobile, the undo button does not work"*, and *"There needs to be a history of pics/rejections when it is that easy to swipe up / down by mistake."* Camera Bits' forum has a thread titled "Undo button for incorrectly deleted photos". | [community.adobe.com](https://community.adobe.com/feature-requests-681/p-undo-does-not-work-for-picking-and-rejecting-661142) | **Not here** | The journal records the inverse before the operation runs, covers marks, moves, copies and binning, batches a whole selection into one step so undoing a two-hundred-frame rating is one press, and says what it undid (`src/organize/journal.rs:27`–`43`, `:103`–`144`, `src/app/tagging.rs:216`–`236`). |
| **…but three operations are outside it.** | — | **Worse** | The bulk rename, the capture-time shift and the group tidy are called straight from their button handlers with no journal entry at all (`src/view/organize/rename.rs:94`, `src/view/organize/timeshift.rs:110`, `src/view/organize/group/mod.rs:101`) — `journal.record` is called from exactly four places and none of them is these (`src/app/cull.rs:247`, `:458`, `:463`, `src/app/tagging.rs:235`–`236`) — and none of the three appears in `Step` (`src/organize/journal.rs:28`–`43`). These are the three most destructive things the program does: one renames every file in a folder, one rewrites EXIF in place, one moves every group into a subfolder. The journal is also in memory only (`src/app/mod.rs:227`), so closing the program discards it. |
| **Undo that happens silently is as frightening as none.** The first plan's reasoning for describing a step before doing it. | `plan.md` | **Same** | `Step::describe` exists and produces a sentence (`src/organize/journal.rs:46`–`68`), and `App::undo` reads it *before* the undo runs (`src/app/cull.rs:502`) — then performs the undo and reports afterwards, *"Undone: put 200 file(s) back"* (`src/app/cull.rs:525`). The sentence is built at the right moment and shown at the wrong one. Nothing is put in front of the user before the fact. |

---

### 2.8 The cull loop: auto-advance, the ground moving, and latency

Eight sources across four products and a seven-year span. The loop is look ▸
judge ▸ mark ▸ next, and every complaint in this section is one of those four
steps failing.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **No auto-advance after a rating.** *"the culling process is very frustrating in Darktable. I have to click all over the page to give a star rating"* … *"In Lightroom I could use the number pad to give a star rating and the arrow keys to advance."* An entire thread titled "Quick rating and auto-advance in Linux a la Photo Mechanic". | [discuss.pixls.us/t/13870](https://discuss.pixls.us/t/library-culling-workflow-can-this-be-streamlined/13870), [discuss.pixls.us/t/3446](https://discuss.pixls.us/t/quick-rating-and-auto-advance-in-linux-a-la-photo-mechanic/3446) | **Not here** | A rating, flag, label or tag advances when the mode is on, and the mode is a key of its own rather than a held modifier, because on a Slovak or German layout the digits *are* the shifted characters of the top row (`src/app/input.rs:192`–`200`, `src/app/mod.rs:706`). Two things about it are not on screen anywhere. Marking a selection never advances, whatever the mode says (`src/app/mod.rs:808`–`812`), which is right and unexplained. And the setting's own doc comment promises a one-shot that does not exist — *"Holding shift with any of those keys advances once whatever this says"* (`src/config/mod.rs:166`–`167`) — while `advances()` takes no modifier at all (`src/app/input.rs:198`–`200`). The README has it right (`README.md:128`–`132`); the comment a settings window would be generated from does not, which is one reason §3's window cannot simply be generated from the fifty-seven fields that have no doc comment and the fifty-three that do without reading them. |
| **Rating an image makes the layout move under you.** digiKam: *"It moves the image to the end of the carousel and I have to find my way back to where I left off."* darktable: *"When I assign a star or color rating to an image in the lighttable section … in the grid view, the gri[d] scrolls automatically"* — narrowed to the case where the top row is partly visible. Lightroom Classic silently reverts the sort to Capture Time while you compare two frames. | [discuss.pixls.us/t/16569](https://discuss.pixls.us/t/assigning-star-ratings-workflow/16569), [discuss.pixls.us/t/59458](https://discuss.pixls.us/t/applying-a-star-rating-scrolls-the-lighttable-why/59458), [community.adobe.com](https://community.adobe.com/t5/lightroom-classic-discussions/lightroom-classic-changing-sort-order-to-capture-time-constantly/m-p/14602184) | **Same** | With the filter bar untouched nothing moves, because re-narrowing is skipped when the rules are default and the sort is by name (`src/view/narrow.rs:146`–`151`, `src/app/mod.rs:650`–`657`). The moment somebody sorts by stars — which is the obvious thing to do while culling — every rating re-sorts the collection and `set_visible` forces a scroll to the cursor (`src/view/grid_view/mod.rs:103`–`111`). The cursor does follow the photograph rather than the position, which is better than digiKam; but the sheet is dragged to wherever that photograph has just been moved to, which is darktable's complaint exactly. |
| **When the marked frame leaves the collection, the cursor jumps to the beginning.** The corollary everybody names and nobody implements. | [community.adobe.com](https://community.adobe.com/feature-requests-681/p-undo-does-not-work-for-picking-and-rejecting-661142) | **Not here** | `forget` removes the photograph and leaves the cursor at the same index, so what it lands on is the next one (`src/app/cull.rs:274`–`287`), and `set_visible` falls back to the nearest surviving index when the frame is filtered away (`src/view/grid_view/mod.rs:107`–`109`). |
| **Which image did that keystroke just apply to?** *"it strangely fails at one delicate task — to deliver confidence and fidelity about the question what image you're actually rejecting/rating!"* … *"if I am too fast with my actions, it can happen that the program has not yet registered the mouse to be over the image."* | [discuss.pixls.us/t/33795](https://discuss.pixls.us/t/frustration-with-culling-mode/33795) | **Not here** | One rule, read by marking, tagging, moving and deleting alike: the selection when the contact sheet has one, and otherwise the frame under the keyboard cursor (`src/app/tagging.rs:270`–`282`, `:262`–`268`), never whatever the pointer happens to be over. The status bar draws the marks of the frame on screen so a keystroke with the tag panel closed is not silent (`src/view/image_view/bottom_bar.rs:185`–`206`). |
| **Rating badges are hidden by default in the mode built for culling.** *"When using 'file manager' lighttable mode, I can set and see star ratings … Then I switch to culling mode … I don't see star ratings"* — by design, on hover only. Lightroom's badges *"simply don't display due to space constraints"* when thumbnails get small, and the official advice is to make the filmstrip bigger. | [discuss.pixls.us/t/20517](https://discuss.pixls.us/t/culling-mode-ratings-stars-do-not-show/20517), [lightroomqueen.com](https://www.lightroomqueen.com/disappearing-thumbnail-icons/) | **Not here** | Badges are drawn under every cell without hovering and are cycled between none, marks, and marks with the caption by one key (`src/view/grid_view/mod.rs:530`–`532`, `src/config/mod.rs:465`–`468`). |
| **Latency between frames is the whole game, and the fix is prefetch.** *"you will have a small delay (~1sec depending on hardware)"*; holding the arrow key down warms the cache and *"after this you can move seamlessly back and forth"*. | [discuss.pixls.us/t/47593](https://discuss.pixls.us/t/darktable-culling-implement-prefetch/47593) | **Not here** | This is the thing the program was built to do. Navigation is a texture swap over a resident window, and the README's published benchmark — 120 24-megapixel JPEGs, larger than the cache, on a 24-core Ryzen — is 501 images in 11.50 s, 43.6 images a second, median frame 2.70 ms (`README.md:291`). |
| **A background job is experienced as blocking whenever it starves the thing you are trying to do.** *"Cache generation has always been a number one issue for me in Bridge… Now you force me to sit idle and wait for 2 hours?"* … *"sometimes more than 5 minutes just to make a selection, apply a filter."* Adobe's reply was that it is *"not blocking"*; the user disputed it. | [adobebridge.uservoice.com](https://adobebridge.uservoice.com/forums/905377-report-bugs/suggestions/35759968-bridge-cache-generation-is-blocking-work) | **Same** | The scan for stacks and the folder-job scan both run off the UI thread and report their progress (`src/app/stacking.rs:52`, `src/ui/filter_bar.rs:113`, `src/view/organize/mod.rs:255`–`269`). Four things do not, and the mechanism is what matters rather than how bad it feels: all four are called on the UI thread with no progress and no way to cancel. `crawler::crawl` walks the tree from `open_directory` (`src/app/mod.rs:679`, `src/crawler.rs:121`), which with flattening on is an arbitrarily deep walk. `rename::apply` renames every planned file inside the button's click handler (`src/view/organize/rename.rs:94`), `timeshift::apply` reads and rewrites the EXIF of every planned file in the same place (`src/view/organize/timeshift.rs:110`), and `gather::apply` moves every group into a subfolder in the same place again (`src/view/organize/group/mod.rs:101`). |

---

### 2.9 Sort order, filters, and how they are scoped

Eight sources across four projects. Two distinct complaints that are usually
filed as one.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **The viewer does not agree with the file manager, or with humans.** *"is there a way to make JPEGview respect the order of how the pictures are sorted on a folder?"* — closed not planned. ImageGlass's "use Explorer sort order" stopped working in 9.1.6.14 — closed not planned. XnView has been asked for it since at least 2022; a different user in November 2025: *"I don't understand why it's not implemented yet?!? YEARS have passed."* | [JPEGView #268](https://github.com/sylikc/jpegview/issues/268), [ImageGlass #1943](https://github.com/d2phap/ImageGlass/issues/1943), [XnView t=42896](https://newsgroup.xnview.com/viewtopic.php?t=42896) | **Same** | There is no per-folder sort of any kind, no reading of the platform's own folder view settings, and no setting to change the default order: the collection is sorted with `crawler::sort` at open (`src/app/mod.rs:680`) and the filter bar's order is a runtime choice that starts at Name every launch (`src/view/narrow.rs:112`–`121`, `src/app/mod.rs:228`). The `browsing.sort` and `browsing.descending` fields that would fix it are §6's. |
| **"Remember previously-used display sort order."** | [JPEGView #216](https://github.com/sylikc/jpegview/issues/216) | **Same** | `Narrowing` is `Default` on construction and is not in `Session` (`src/app/mod.rs:228`, `src/session.rs:56`–`65`). Every filter rule and the sort key are discarded on exit. Whether that should be a setting or session state is the three-way test in §4. |
| **Filter and sort scope is decided by two interacting settings neither of which is visible where the behaviour is felt.** *"it used to be that when navigating between folders … LR would remember my filter settings"*; the answer turned out to be "Lock Filters" checked **and** "Remember Each Source's Filters Separately" unchecked. | [lightroomqueen.com](https://www.lightroomqueen.com/community/threads/filters-not-remembered-when-moving-away-from-collection.45428/) | **Not here** | There is one scope and it is global-for-the-session, so there is nothing to be confused by. That is the right starting point; the risk to avoid is adding per-folder filters later as a *second* setting rather than as one legible choice — which is why §6 proposes `browsing.filter_follows_folder` as one field and not two. |
| **A hidden filter bar leaves the filter on and the user cannot see why photographs are missing.** *"if the Filter bar is hidden but the Library filters are turned on, then you may not see the photos you expect to."* | [mastering-lightroom.com](https://mastering-lightroom.com/lightroom-filter-bar/) | **Not here** | The status bar reads `12/40 (+160)` whenever anything is held back, with *"160 more are hidden by the filter"* on hover, and it is drawn whether or not the filter bar is up (`src/view/image_view/bottom_bar.rs:114`–`129`). `\` sets the rules aside without forgetting them and is a labelled toggle in the bar as well as a key (`src/ui/filter_bar.rs:280`–`291`, `src/config/defaults.rs:426`). This is better than Lightroom. |
| **…except in the gallery.** | — | **Worse** | The contact sheet has no equivalent readout. When a filter empties it, the whole answer is a centred `"Nothing matches the filter"` with no count, no statement of which rule did it and no way to clear it from there (`src/view/grid_view/mod.rs:263`–`272`) — while the Clear button that would fix it sits in a bar the sheet does not draw (`src/ui/filter_bar.rs:49`–`57`). It is the plainest instance of the dead end §8 is about: the program says what is wrong and offers no route to the thing that fixes it. |
| **Recursive browsing.** nomacs's recursive scan shows only the selected folder's images, and if the root has none, only the first subfolder's — 11 reactions, the second highest non-meta issue in that tracker. qimgv has two independent requests for it. | [nomacs #297](https://github.com/nomacs/nomacs/issues/297), [qimgv #252](https://github.com/easymodo/qimgv/issues/252) | **Not here** | Flattening folds every sub-directory into the collection, with a cycle guard so a `Pictures/latest -> .` symlink cannot send it round for ever (`src/crawler.rs:121`–`142`). It is on a key with no visible route to it, and it runs on the UI thread — see §2.5 and §2.8. |

---

### 2.10 Settings that need a restart, and settings that do nothing

Eight threads across two projects, and one of them is the compound failure that
makes people stop trusting a settings screen altogether: prompt for a restart ▸
restart ▸ nothing happened ▸ hand-edit a file.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **The settings tab says "Please Restart nomacs to apply changes"; you restart; the theme has not changed.** The working fix was to hand-edit `themeName312` in the config. Setting icon sizes prompts a restart, then crashes the settings editor, and the next launch has the wrong sizes anyway. | [nomacs #1062](https://github.com/nomacs/nomacs/issues/1062), [#912](https://github.com/nomacs/nomacs/issues/912) | **Same** | Twenty-six settings do not take effect until the next launch: twenty-five wholly, and `grid_view.filmstrip_height` half so, because its height is read live every frame (`src/app/views.rs:138`) while whether the strip comes up at all is decided once (`src/app/mod.rs:175`). The twenty-five are captured at startup into structures with no setter — `text_scaling` (`src/app/mod.rs:150`), `decode_threads` into the thread pool (`:162`), `output_icc_profile` (`:163`), the cache, residency and raw development values that go through `stores::image_store` and `stores::thumbnail_store` (`:164`–`165`, `src/app/stores.rs:32`–`81`), and `catalog_file`, `recent_tags` and `advance_after_marking` (`src/app/mod.rs:176`, `:209`–`210`). §3 has the field-by-field table and how each one becomes live or rebuild; only `decode_threads` resists both. Two cases that look like this and are not: `pair_with_jpeg` needs the folder re-opened rather than the program restarted (`src/app/mod.rs:307`), and `restore_session` describes the next launch by nature. **Nothing anywhere says any of it** — not the interface, which has no window to say it in, and not the README, whose only occurrence of the word "restart" is about the slideshow clock (`README.md:511`). |
| **A setting is written and never read back, so the change is accepted and discarded.** The nomacs "Hide All Panels" shortcut is saved to `settings.ini` and never read, so the key silently stays `F` for ever. | [nomacs #186](https://github.com/nomacs/nomacs/issues/186) | **Not here** | Checked field by field: every one of the 110 settings has at least one read outside `config/` and outside its own defaults. The nearest miss is `slideshow.image_frame_background_color_override`, which is read (`src/view/image_view/mod.rs:555`–`564`) but only while a slideshow is actually running, and which falls back to the hardcoded grey without a word when the hex string does not parse — `Color32::from_hex(hex).ok()` followed by `unwrap_or(BACKGROUND)`. A typo there is silently ignored, which is the small version of the same complaint; validation is §3's. |
| **Two commands on one key, with the loser doing nothing and saying nothing.** | Found on the author's own machine, per `src/ui/keys.rs:188`–`194` | **Not here** | Every clash is detected at startup and named, in a sentence built from the registry's own names for the two commands — *"Zoom in and More side by side are both on Plus"*, which is what that machine's file produced (`src/ui/keys.rs:214`, names at `src/config/bindings.rs:283`, `:333`) — and shown in the editor beside the row that clashes (`src/ui/keys.rs:154`, `:171`–`218`, `src/app/mod.rs:240`). A stock install does not have that clash: zoom-in is bare `Plus` and more-side-by-side is `Ctrl+Plus` (`src/config/defaults.rs:247`–`255`). It arose in a file written by an older build, which is the case the check exists for: serde fills in the keys that are missing and never the ones that have since moved. Nothing else in the corpus does this. |
| **No way to clear a binding.** *"No indication on how to remove a keyboard shortcut."* The discovered workaround is to cause a conflict on purpose. | [nomacs #71](https://github.com/nomacs/nomacs/issues/71) | **Same** | The editor arms a row and takes the next key; `Escape` cancels and every other key becomes the binding (`src/ui/keys.rs:245`–`280`). There is no key and no button that unbinds. The row can *display* `"unbound"` (`src/ui/keys.rs:143`) but there is no way to reach that state from the interface. §9 owns the editor's faults, this among them. |
| **The rebinding surface forgets across the UI language, or across a restart.** nomacs saves shortcuts under *translated* names, so they break when the language changes; XnView's multi-key shortcuts reset to defaults after a restart. | [nomacs #1539](https://github.com/nomacs/nomacs/issues/1539), [XnView t=46390](https://newsgroup.xnview.com/viewtopic.php?t=46390) | **Not here** | Bindings are stored as named struct fields, not as display strings (`src/config/mod.rs:389`–`433`), and written the moment they change (`src/app/settings.rs:94`). The program is English-only, so the translation trap does not yet exist — the lesson to carry is not to key persisted data off a display string if it ever does. |

The fix for the first row is not expensive and it has a reference
implementation. darktable shows a toast when the preferences dialogue is closed
if anything changed that needs a restart, and documents the behaviour in its
manual ([PR#5957](https://github.com/darktable-org/darktable/pull/5957),
[dtdocs](https://darktable-org.github.io/dtdocs/en/preferences-settings/overview/)).
The notice band that would carry the same message already exists here, holds a
line for six seconds over four lines of depth, and already carries the migration
lines at startup (`src/ui/notice.rs:16`, `:19`, `:22`, `src/app/mod.rs:244`).

---

### 2.11 RAW+JPEG, stacks, and the tier above viewers

Fewer sources, because general-purpose viewers do not have these features and
therefore attract no complaints about them. That silence is not evidence of
indifference.

| The complaint | Who says it | Verdict | What is here |
|---|---|---|---|
| **RAW and JPEG have to be one frame for the purposes of a decision.** *"I can't really use it as the only DAM software in my workflow because it is not capable of grouping RAW and JPG files"* — he culled in Zoner instead. Geeqie was recommended because it *"groups raw and jpg files deleting both together"*. | [discuss.pixls.us/t/3000](https://discuss.pixls.us/t/culling-process-in-digikam/3000) | **Not here** | One half is browsed and the other follows it through every command; which half is a three-way setting including "do not pair" (`src/organize/pairs.rs:32`–`42`, `src/config/mod.rs:80`). A rating, flag, label or keyword is applied to both files and each gets its own sidecar (`src/app/tagging.rs:209`–`212`); a deletion takes both (`src/app/cull.rs:73`–`77`); and the status bar says `RAW+JPEG` while a paired frame is on screen, because everything that follows is about to happen to two files (`src/view/image_view/bottom_bar.rs:65`–`71`, `:150`). |
| **Stacks hide photographs and silently change the scope of bulk actions.** *"missing photos are buried in a stack"*, *"The present method for identifying stacks is not noticeable enough"*, and metadata changes behave differently *"whether selected stack collapsed or expanded"*. | [lightroomqueen.com](https://www.lightroomqueen.com/community/threads/stacks-why-bother.44251/) | **Not here** | Stacks are a view over the folder and write nothing, and the control that turns them on says so on hover (`src/ui/filter_bar.rs:96`–`100`). The status bar always says which frame of which run is on screen and colours the badge when the run is folded (`src/view/image_view/bottom_bar.rs:131`–`142`). |
| **Automatic burst grouping mis-groups, and its rules are invisible.** *"using autostack LR sometimes grouped together two or three stacks"*, and the error message *"Current Slider settings does not allow auto stacking"* is *"useless in informing the user of the problem."* | [lightroomqueen.com](https://www.lightroomqueen.com/community/threads/why-no-auto-stack-by-fixed-number-of-bracket-shots.50182/), [lightroomqueen.com](https://www.lightroomqueen.com/community/threads/stack-by-visual-similarity-what-am-i-missing.53788/) | **Not here** | The gap and the similarity tolerance are dragged live in the filter bar with the run count updating as they move — *"Drag it and watch the runs join up or come apart"* (`src/ui/filter_bar.rs:137`, `:147`–`150`). Watching the grouping change is a better answer than any error message. It is also one of only two `egui::Slider` widgets in the program, against seventeen drag values (§5). |
| **AI culling costs back the time it saves.** *"If I have to check the result of AI, then it hasn't saved me time and is fruitless."* And *"With AI culling, I see a situation where you have to explain to the customer that AI deleted the only photo you had of their aged Aunt Susie."* | [digitalcameraworld.com](https://www.digitalcameraworld.com/tech/software/lightroom-classics-assisted-photo-culling-is-a-great-idea-but-is-really-annoying-but-i-have-found-a-solution), [community.adobe.com](https://community.adobe.com/feature-requests-676/p-ai-culling-665458/index4.html) | **Not here** | The instruments are measurements, not verdicts: a real histogram, clipping counts, a clipping overlay and focus peaking on one key (`src/config/mod.rs:354`–`356`, `src/decoder/histogram.rs`, `src/decoder/overlays.rs`, `src/ui/histogram.rs`). LibRaw's refusal of an AF-point overlay on the ground that the metadata *"becomes very misleading if the subject or camera is moved"*, and its decision to ship focus peaking instead, is the same judgement ([fastrawviewer.com/node/826](https://www.fastrawviewer.com/node/826)). |
| **A catalogue is a liability.** Photo Mechanic Plus was withdrawn from sale because its catalogue rested on *"a 3rd-party technology that is now unsupported"* which is *"actively breaking Photo Mechanic Plus for most users"*. The browsing half survived; it owns no database. | [home.camerabits.com](https://home.camerabits.com/photo-mechanic-plus-is-no-longer-available-for-purchase/) | **Not here** | Nothing persists that cannot be rebuilt by rescanning: the caches are in memory, and `session.json` and `recent_tags.json` degrade to a first run if they are deleted (`src/session.rs:74`–`93`). |

---

### 2.12 What avis-imgv is placed to answer better than any of them

Four complaints in this corpus are structural in the programs they were filed
against — the maintainers are not being lazy, the architecture will not permit
the fix — and avis-imgv can answer all four from what it already has.

**One. Synchronised zoom across more than two frames.** The Adobe idea board has
carried this for years: Survey View should have a 100% option, because Compare
View *"only allows two images at a time, making it tedious when reviewing dozens
of photos"*, and one commenter dates the ask at fifteen years
([community.adobe.com](https://community.adobe.com/t5/lightroom-classic-ideas/p-survey-view-n-should-get-a-zoom-100-option/idi-p/14447983)).
FastRawViewer's author refused the same request on the ground that the program
cannot be made to do it without being rebuilt: *"two-files view requires full
program internals re-design. It is build around single file view"*, and *"do not
expect two (or more) files view before FastRawViewer 2.0"*
([fastrawviewer.com/node/33](https://www.fastrawviewer.com/node/33)). The first
plan argues for linked zoom already (`plan.md`, §3.2). The reason it is cheap
here is `Viewport`: zoom and pan are a UV rectangle over a resident texture,
one viewport is held for the whole view (`src/view/image_view/mod.rs:81`), and
only the leading pane feeds its pan back
(`src/view/image_view/canvas.rs:178`–`200`). Comparing four frames at 100%
needs no new decode of the frames already resident, and how many neighbours are
held at full resolution is a number in the configuration rather than an
architectural limit (`src/config/mod.rs:245`–`246`) — one of the forty-one
nothing on screen can reach.

**Two. Folder-wide instruments as sort and filter keys.** Automatic culling
draws its worst reviews for opaque scoring — *"If I have to check the result of
AI, then it hasn't saved me time"*
([digitalcameraworld.com](https://www.digitalcameraworld.com/tech/software/lightroom-classics-assisted-photo-culling-is-a-great-idea-but-is-really-annoying-but-i-have-found-a-solution)),
and an Aftershoot user: *"I noticed a few things I knew I had shot that had been
badly selected"*
([forums.macrumors.com](https://forums.macrumors.com/threads/thoughts-on-aftershoot.2396263/)).
What photographers say they trust instead is a measurement they can see.
A single-file viewer cannot offer a folder-wide one, because it only ever has
one file open. avis-imgv works over the whole folder and already computes both
halves: a sharpness score for every frame the folder scan reaches
(`src/organize/scan.rs:167`), and a histogram with clipping counts for every
frame the decoder touches (`src/decoder/mod.rs:338`). What is missing is the
reach. `SortKey::Sharpness` exists but lives in `organize::sort` and is
therefore only available in the three folder-job modes that draw no photographs
(`src/organize/sort.rs:30`, `:37`), while the browsing views sort by name,
stars, label and flag only (`src/view/narrow.rs:112`–`121`) and none of the
seven filter rules is a numeric measurement (`src/view/narrow.rs:22`–`34`).
"Show me the sharpest frame of every burst" is one enum variant and one filter
rule away from working,
and a viewer that opens one file at a time cannot follow it there.

**Three. Sidecars that other programs' work survives.** The single most
expensive class of bug in the culling corpus is metadata interop: `rdf:Seq`
against `rdf:Bag`, colour labels as untranslated text, `IMG.xmp` against
`IMG.CR3.xmp`, Lightroom's manual "Save Metadata to Files", and sidecars still
carrying an abandoned program's namespaces. avis-imgv already does the four
things that matter — pass through what it does not understand, refuse to write
rather than truncate, `rdf:Bag`, and both `dc:subject` and
`lr:hierarchicalSubject` (`src/metadata/xmp/write.rs:46`, `:112`, `:231`, `:236`,
`:270`) — and it writes to a temporary and renames over the original, so an
interrupted write cannot leave half a develop history behind
(`src/annotations/sidecar.rs:88`). The remaining work is one setting for which
filename form to *create* (§6) and one duplicated alias (§2.4).

**Four. Configuration as a file that is also a first-class interface.**
ImageGlass's layered-JSON documentation describes the shape the whole corpus
implies: the GUI is the primary editor and the file is the escape hatch
([imageglass.org/docs](https://imageglass.org/docs/app-configs)). VS Code's
Settings editor is *"the user interface that enables you to review and modify
setting values that are stored in a settings.json file"* — search, a modified
marker, and a per-setting reset over a file that remains the truth
([code.visualstudio.com](https://code.visualstudio.com/docs/getstarted/settings)).
avis-imgv already has the harder half and none of the easier half: a versioned,
migrated, section-tolerant, byte-order-mark-tolerant file that refuses to
overwrite what it could not read (`src/config/load.rs:99`–`104`, `:125`–`159`,
`src/config/migrate.rs:55`), a registry that already carries a name, a section
and a sentence of prose for every command (`src/config/bindings.rs:27`–`36`),
and a cheat sheet already generated from it. What is missing is a window over
the same registry, extended to the fifty behaviour settings: §3 specifies it and
§12 says when each part of it is built. Every part of it is a thing darktable,
VS Code or Microsoft's own guidance says to do, and every part is already
half-built here. One caution carried over from §2.8: that window would be
generated from the doc comments on the configuration fields, and at least one of
those comments already describes behaviour the code does not have.

## 3. The settings that cannot be reached

**Forty-seven of the hundred and ten settings have no control anywhere in the
running program, and forty-one of those cannot be changed at all while it
runs.** They are hand-edited JSON or they are nothing.

The derivation is short. The configuration file holds 111 keys: `version` at
the root and 110 settings across the eight sections `Config` declares
(`src/config/mod.rs:18-49`; `partial` and `migrated` carry `#[serde(skip)]` at
`:41` and `:47` and never reach the file). Sixty of the 110 are keyboard
shortcuts (`grep -c "pub sc_" src/config/mod.rs` returns 60) and the keyboard
editor reaches every one of them, drawing them as 69 rows because `sc_rating`
holds six and `sc_label` five — 58 `binding!` entries at
`src/config/bindings.rs:85-418` plus eleven pushed in two loops at `:420-436`,
drawn by `src/ui/keys.rs:60-129`. Three more — `seconds_per_image`, `motion`
and `percent_zoom` — can be changed from `Settings ▸ Slideshow…`
(`src/app/panels.rs:92-135`). That is the whole of it. The `Settings` menu has
two entries and no third (`src/app/panels.rs:67-77`).

Sixty and three from a hundred and ten leaves forty-seven: every cache budget,
every raw-development option, the ICC profile, the text scale, the tag
catalogue, the destination folders, the three template strings, both
context-menu lists, the user actions, and every number that decides how the
contact sheet is laid out.

Six of the forty-seven have a runtime effect that a key can nudge and that is
then thrown away on exit: the overlay corner (`o` cycles the image view's own
copy at `src/view/image_view/mod.rs:279`), how many pictures are shown side by
side (`:284`, `:289`), thumbnails per row (`src/view/grid_view/mod.rs:524-527`),
whether the filmstrip is up (`src/app/mod.rs:713`), advance-after-marking
(`src/app/mod.rs:706`) and the width of the tag panel, which is a draggable
splitter (`src/ui/tag_panel/mod.rs:61-66`). None of the six is ever written
back. `App::save_settings` — the only thing in the program that writes the file
during a session — is called from exactly two places, `src/app/settings.rs:70`
and `:94`, both of them the two windows that exist. (`Config::save` itself has
one other caller: `src/config/load.rs:101`, which writes a migrated file back
out at startup.) **The remaining forty-one cannot be changed at all while the
program is running.**

There is a second, quieter failure underneath that one. Because `Config::new()`
is called once (`src/app/mod.rs:147`) and the store configuration is built from
it immediately (`:162-165`), **twenty-six of the 110 would still not take effect
even if a control existed** — twenty-five of them wholly, and
`grid_view.filmstrip_height` by half — because nothing re-reads them. Nothing
anywhere says so, and the two editable things that do exist both apply the
moment they change, so the mental model the interface teaches is exactly wrong
for the fields it does not reach.

§1 states the same gap from the interface's side. This chapter is the inventory
behind it, the window that closes it, and the repairs that window depends on.

---

### 3.1 Every field, what it is, and whether anything reaches it

The **Reachable today** column has four values: *nowhere*, *keyboard editor*,
*slideshow window*, and *runtime only* for the six above — a key changes the
behaviour, nothing writes it down.

The **Type** column is the census §5 works from: 24 numbers, 8 booleans, 7
strings, 5 enums, 6 lists and the 60 shortcuts. `raw.highlight_mode` is a `u8`
behaving as an enum and is counted with the numbers.

The **Takes effect?** column answers a narrower question than it looks. The
whole file is read once at startup, so on a strict reading every field needs a
restart. What is being asked here is: *given the fan-out the program already
performs when the keyboard editor writes* — `self.config =
self.settings.general.clone()`, the same for `tag_config`, then
`ImageView::set_config` and `GridView::set_config`
(`src/app/settings.rs:89-92`) — would a change take effect? **Live** means yes.
**Restart** means no: the value was consumed at construction and no code path
re-reads it. That is the distinction a settings window has to be honest about,
and §3.5 is about closing it.

Which page each field lands on is §4's, field by field; the eleven page names
are in §3.2.

#### `raw` — 6 fields, none reachable

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `pair_with_jpeg` | Which half of a raw+JPEG pair is browsed, or whether to pair at all | `src/config/mod.rs:80` | nowhere | enum | Next folder open (`src/app/mod.rs:307`, `src/app/chrome.rs:85`) |
| `source` | Show the JPEG the camera embedded, or develop the sensor data | `:85` | nowhere | enum | **Restart** (`src/app/stores.rs:71`) |
| `quality` | How much work to spend demosaicing | `:88` | nowhere | enum | **Restart** (`src/app/stores.rs:72-76`) |
| `camera_white_balance` | Use the white balance the camera recorded | `:92` | nowhere | boolean | **Restart** (`src/app/stores.rs:77`) |
| `auto_brighten` | Stretch the histogram to use the whole range | `:95` | nowhere | boolean | **Restart** (`src/app/stores.rs:78`) |
| `highlight_mode` | What to do with blown highlights: clip, leave, blend, rebuild | `:99` | nowhere | number | **Restart** (`src/app/stores.rs:79`) |

`Prefer::ALL` and `Prefer::label()` — "Show both", "Show the JPEG", "Show the
raw" — are written at `src/organize/pairs.rs:44-54` and referenced nowhere but a
test (`:409-416`). The sentences that would make this setting legible already
exist.

#### `cache` — 6 fields, none reachable, all restart-bound

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `ram_budget_mb` | Ceiling on decoded pixels kept in RAM, across both views | `src/config/mod.rs:236` | nowhere | number | **Restart** (`src/app/stores.rs:34`, `:50`) |
| `decode_threads` | Decode worker threads; zero picks one per core less one, up to eight | `:239` | nowhere | number | **Restart** (`src/app/mod.rs:162`) |
| `previews_resident` | Camera thumbnails held on the card to stand in for a picture still decoding | `:244` | nowhere | number | **Restart** (`src/app/stores.rs:39`) |
| `full_resolution_neighbours` | Full-size copies held either side of the one on screen | `:246` | nowhere | number | **Restart** (`src/app/stores.rs:40`) |
| `gpu_budget_mb` | Ceiling on what the two caches may hold on the adapter | `:254` | nowhere | number | **Restart** (`src/app/stores.rs:36`, `:52`) |
| `upload_budget_ms` | How long a frame may spend moving decoded pictures onto the card | `:260` | nowhere | number | **Restart** (`src/app/stores.rs:41`, `:60`) |

Two of the sentences that would be shown under these controls are attached to
the wrong field: the doc comment "Half a frame at sixty a second, which leaves
the rest for drawing" sits above `default_gpu_budget_mb` rather than above
`default_upload_budget_ms`, run together with the gigabyte sentence that belongs
there (`src/config/defaults.rs:35-43`). §3.4 proposes indexing these comments,
which means they have to be right first. `decode_threads` needs the same care in
the other direction: the field's default is `0` (`src/config/defaults.rs:16-18`)
and the eight the README's prose names (`README.md:298`) is the ceiling `Loader`
applies to that zero (`MAX_DEFAULT_WORKERS`, `src/cache/loader.rs:252`,
`:256-260`), which the README's own table states correctly (`README.md:619`).
The control has to say "one per core, less one, up to eight", not "8".

#### `general` — 25 fields, 21 of them shortcuts

Four of the twenty-five are not keys, and they are the whole of this section's
non-keyboard content:

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `output_icc_profile` | The screen profile everything is converted to | `src/config/mod.rs:266` | nowhere | string | **Restart** (`src/app/mod.rs:163`) |
| `text_scaling` | How large the interface text is | `:268` | nowhere | number | **Restart** (`src/app/mod.rs:150`) |
| `metadata_tags` | Which metadata the side panel lists, in order | `:270` | nowhere | list | Live (`src/app/chrome.rs:117`) |
| `restore_session` | Open where the last run left off | `:277` | nowhere | boolean | Live (`src/app/mod.rs:588`, `:867`) — it describes the next launch, which is not the same as needing one |

`output_icc_profile` and `text_scaling` are this section's only restart-bound
rows, and both are among the twenty-five §3.5 unsticks. The other twenty-one are
keys, reached by the keyboard editor and live the moment they change; they are
drawn separately below because that is how the registry will separate them
(§12), and because twenty-one identical last columns are not a table anybody
reads.

| Shortcut | What it does | Where |
|---|---|---|
| `sc_toggle_gallery` | Switch between the image and the contact sheet | `:280` |
| `sc_next_mode` | Move round the modes: image, gallery, bulk rename, shift capture time, group shots, slideshow | `:283` |
| `sc_exit` | Close the viewer | `:285` |
| `sc_menu` | Show or hide the menu bar | `:287` |
| `sc_navigator` | Type a path to open instead of picking one | `:289` |
| `sc_dir_tree` | Open the folder tree beside the image | `:291` |
| `sc_flatten_dir` | Read the pictures out of every sub-folder as though they were one | `:293` |
| `sc_watch_directory` | Pick up pictures that appear or change while the viewer is open | `:295` |
| `sc_toggle_side_panel` | Show or hide the metadata and cache readout down the side | `:297` |
| `sc_filmstrip` | Show or hide the strip of thumbnails under the photograph | `:300` |
| `sc_stacks` | Show the folder stacked: every burst, bracket and timelapse as one cell | `:304` |
| `sc_toggle_stack` | Show what is inside the run of frames the cursor is on, or fold it back up | `:307` |
| `sc_standing_back` | Walk the frames of a closed stack without opening it | `:310` |
| `sc_standing_forward` | The same, forwards | `:312` |
| `sc_previous_stack` | Step to the run of frames before this one, over a burst rather than through it | `:316` |
| `sc_next_stack` | Step to the run of frames after this one | `:318` |
| `sc_delete` | Send the picture on screen to the platform's bin, along with its sidecar | `:321` |
| `sc_delete_permanently` | Delete it outright, for the cards and shares that have no bin | `:324` |
| `sc_fullscreen` | Fill the screen, and give it back | `:327` |
| `sc_filter` | Show or hide the bar that narrows and orders the folder | `:330` |
| `sc_suspend_filter` | Set the filter aside without forgetting it | `:334` |

#### `image_view` — 36 fields, 22 of them shortcuts

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `overlay_corner` | Which corner the photograph's own details are drawn in, if any | `src/config/mod.rs:345` | runtime only (`src/view/image_view/mod.rs:279`) | enum | Live |
| `overlay_format` | What those details say, in the template grammar | `:348` | nowhere | string | Live |
| `overlay_text_size` | How large they are | `:350` | nowhere | number | Live |
| `sc_overlay` | Move the photograph's own details round its corners, and off again | `:353` | keyboard editor | shortcut | Live |
| `sc_marks` | Mark what has clipped, then what is in focus, then nothing | `:356` | keyboard editor | shortcut | Live |
| `nr_loaded_images` | Pictures decoded either side of the one on screen | `:359` | nowhere | number | **Restart** (`src/app/stores.rs:37`) |
| `gpu_resident_images` | Pictures kept as textures, ready to draw with no upload | `:362` | nowhere | number | **Restart** (`src/app/stores.rs:35`) |
| `max_image_edge` | Cap on the longest edge of a decoded picture; zero means the largest the card takes | `:366` | nowhere | number | **Restart** (`src/app/stores.rs:38`) |
| `nr_images_shown` | How many pictures are shown side by side | `:368` | runtime only (`src/view/image_view/mod.rs:284`, `:289`) | number | **Restart** (`src/view/image_view/mod.rs:129`) |
| `should_wait` | Wait for the full decode before drawing rather than showing the preview | `:370` | nowhere | boolean | Live (`src/view/image_view/mod.rs:299`) |
| `frame_size_relative_to_image` | How wide the white frame is, against the shorter edge | `:372` | nowhere | number | Live (`src/view/image_view/navigate.rs:96`) |
| `scroll_navigation` | Whether the wheel moves through the folder | `:374` | nowhere | boolean | Live (`src/view/image_view/interaction.rs:18`) |
| `enlarge_to_fit` | Whether a picture smaller than the window is enlarged to fill it | `:381` | nowhere | boolean | Live (`src/view/image_view/mod.rs:533`) |
| `name_format` | What the bottom bar says about the picture | `:383` | nowhere | string | Live (`src/view/image_view/mod.rs:661-673`) |
| `user_actions` | Outside programs on their own keys | `:385` | nowhere — and their `shortcut` is the one shortcut in the file the keyboard editor cannot reach | list | Live (`src/view/image_view/input.rs:131-140`) |
| `context_menu` | Outside programs on the right-click menu over the photograph | `:387` | nowhere | list | Live (`src/view/image_view/interaction.rs:76`) |
| `sc_fit` | Show the whole picture, as large as the window allows | `:390` | keyboard editor | shortcut | Live |
| `sc_frame` | Show or hide the white border around the photograph | `:392` | keyboard editor | shortcut | Live |
| `sc_zoom` | Double the magnification, returning to fitted once it goes far enough | `:394` | keyboard editor | shortcut | Live |
| `sc_next` | Move to the next picture in the folder | `:396` | keyboard editor | shortcut | Live |
| `sc_prev` | Move to the one before it | `:398` | keyboard editor | shortcut | Live |
| `sc_one_to_one` | One screen pixel for each pixel of the photograph | `:400` | keyboard editor | shortcut | Live |
| `sc_repeat_place` | Put this picture at the zoom and position the last one was left at | `:403` | keyboard editor | shortcut | Live |
| `sc_fit_horizontal` | Make the picture exactly as wide as the window | `:405` | keyboard editor | shortcut | Live |
| `sc_fit_vertical` | Make it exactly as tall | `:407` | keyboard editor | shortcut | Live |
| `sc_fit_maximize` | Fill the window, cropping whichever side overflows | `:409` | keyboard editor | shortcut | Live |
| `sc_latch_fit_maximize` | Carry on filling the window as you move through the folder | `:411` | keyboard editor | shortcut | Live |
| `sc_more_images_shown` | Show one more picture beside the current one | `:413` | keyboard editor | shortcut | Live |
| `sc_less_images_shown` | Show one fewer | `:415` | keyboard editor | shortcut | Live |
| `sc_compare` | Pin this picture and the next side by side, sharing one zoom and one pan | `:419` | keyboard editor | shortcut | Live |
| `sc_zoom_in` | Magnify a little more | `:421` | keyboard editor | shortcut | Live |
| `sc_zoom_out` | Magnify a little less | `:423` | keyboard editor | shortcut | Live |
| `sc_pan_up` | Move the view up, for as long as the key is held | `:427` | keyboard editor | shortcut | Live |
| `sc_pan_down` | Move the view down | `:429` | keyboard editor | shortcut | Live |
| `sc_pan_left` | Move the view left | `:431` | keyboard editor | shortcut | Live |
| `sc_pan_right` | Move the view right | `:433` | keyboard editor | shortcut | Live |

The four pan keys carry a caveat the editor does not state: modifiers are
ignored on them, because the key is read as held rather than as a pressed
shortcut — `panning` takes only `shortcut.kbd_shortcut.logical_key` and asks
`input.key_down` on it (`src/view/image_view/input.rs:158-161`). The registry
row says so, and §9 has the rest of what the keyboard editor does not tell
anybody.

#### `grid_view` — 14 fields, 6 of them shortcuts

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `images_per_row` | How many thumbnails fit across | `src/config/mod.rs:439` | runtime only (`src/view/grid_view/mod.rs:524-527`) | number | **Restart** (`src/view/grid_view/mod.rs:77`, `src/app/stores.rs:54`) |
| `cell_aspect` | How wide a cell's picture is against its height | `:447` | nowhere | number | Live (`src/view/grid_view/mod.rs:259`) |
| `preloaded_rows` | Rows of thumbnails read ahead of the scroll | `:449` | nowhere | number | **Restart** (`src/app/stores.rs:54`) |
| `thumbnail_resolution` | Longest edge of a decoded thumbnail | `:452` | nowhere | number | **Restart** (`src/app/stores.rs:55`) |
| `gpu_resident_thumbnails` | Thumbnails kept as textures | `:455` | nowhere | number | **Restart** (`src/app/stores.rs:51`) |
| `context_menu` | Outside programs on the right-click menu over a cell | `:457` | nowhere | list | Live (`src/view/grid_view/mod.rs:461`) |
| `sc_scroll` | Move half a row down the contact sheet | `:460` | keyboard editor | shortcut | Live |
| `sc_more_per_row` | Fit one more thumbnail across, making them smaller | `:462` | keyboard editor | shortcut | Live |
| `sc_less_per_row` | Fit one fewer, making them larger | `:464` | keyboard editor | shortcut | Live |
| `sc_cycle_badges` | Cycle what is drawn under each thumbnail: nothing, the marks, or the marks and the file name | `:468` | keyboard editor | shortcut | Live |
| `filmstrip_height` | How tall the strip of thumbnails under the photograph is; zero turns it off | `:475` | runtime only for the on/off (`src/app/mod.rs:713`) | number | Live for the height (`src/app/views.rs:138`); **Restart** for whether it starts up (`src/app/mod.rs:175`) |
| `caption_format` | What the line under each thumbnail says | `:482` | nowhere | string | Live (`src/view/grid_view/mod.rs:495`) |
| `sc_select` | Pick the photograph under the cursor out, or put it back | `:484` | keyboard editor | shortcut | Live |
| `sc_select_all` | Pick out every photograph on show, or put them all back | `:486` | keyboard editor | shortcut | Live |

`caption_format` is the worst-served field in the file. It works, it is live, it
is documented — and it is invisible unless the badge mode has been cycled to
`Full`, which is not the default and which resets to `Marks` on every launch
(`src/view/grid_view/mod.rs:482`, `:84`, `src/view/grid_view/cell.rs:22-31`).
Somebody can set it perfectly and see nothing.

#### `tags` — 12 fields, 7 of them shortcuts

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `categories` | Keywords kept permanently to hand, in groups | `src/config/mod.rs:128` | nowhere — the panel adds seen and recent keywords and has never been able to write a category (`src/app/tagging.rs:135-169`) | list | **Restart** (`src/app/mod.rs:209`) |
| `catalog_file` | A keyword list exported from another program, read at startup | `:138` | nowhere | string | **Restart** (`src/app/mod.rs:209`) |
| `recent_tags` | How many recently used keywords to remember between sessions | `:141` | nowhere | number | **Restart** (`src/app/mod.rs:210`) |
| `panel_width` | Starting width of the keyword panel, in points | `:144` | runtime only — the splitter drags (`src/ui/tag_panel/mod.rs:61-66`) and nothing is kept | number | Passed live every frame (`src/app/tagging.rs:124`) and ignored: egui prefers its own stored width over `.default_width()` after the first frame (`src/ui/tag_panel/mod.rs:64`). Stuck for the session, not by a startup capture |
| `sc_toggle_tag_panel` | Show or hide the panel for stars and keywords | `:147` | keyboard editor | shortcut | Live |
| `sc_rating` | Six keys, no stars to five | `:150` | keyboard editor, as six rows | shortcut | Live |
| `sc_pick` | Mark the picture on screen as one to keep | `:153` | keyboard editor | shortcut | Live |
| `sc_reject` | Mark it as one to throw out | `:156` | keyboard editor | shortcut | Live |
| `sc_unflag` | Take whichever of those two marks it carries back off it | `:159` | keyboard editor | shortcut | Live |
| `sc_label` | Five keys: red, yellow, green, blue, purple | `:163` | keyboard editor, as five rows | shortcut | Live |
| `advance_after_marking` | Move on to the next picture as soon as one is marked | `:169` | runtime only (`src/app/mod.rs:706`) | boolean | **Restart** (`src/app/mod.rs:176`) |
| `sc_toggle_advance` | Turn that on and off | `:172` | keyboard editor | shortcut | Live |

The doc comment on `advance_after_marking` (`src/config/mod.rs:166-167`) says
"holding shift with any of those keys advances once whatever this says". Nothing
implements that: `advances()` reads the mode flag and nothing else
(`src/app/input.rs:198-200`), and the comment directly above it explains why a
modifier was rejected — "on a Slovak or German keyboard the digits are the
shifted characters of the top row" (`:194-197`). **The sentence that would be
shown under this checkbox is wrong, and would be copied straight into the window
if nobody read the code.** It has to be rewritten before it is displayed.

#### `cull` — 6 fields, 4 of them shortcuts

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `destinations` | Folders a photograph can be sent to with one keystroke | `src/config/mod.rs:181` | nowhere — "Choose a folder…" picks one and throws it away (`src/app/cull.rs:354-371`) | list | Live (`src/app/cull.rs:381-386`) |
| `rejected_folder` | What the folder for rejected frames is called | `:188` | nowhere | string | Live (`src/app/cull.rs:323`) |
| `sc_move` | Send the picture on screen to one of the folders on the panel | `:191` | keyboard editor | shortcut | Live |
| `sc_copy` | Put a copy of it in one of them, leaving the photograph where it is | `:193` | keyboard editor | shortcut | Live |
| `sc_reject_folder` | Move it into the folder for the frames that are not staying | `:195` | keyboard editor | shortcut | Live |
| `sc_undo` | Put back whatever the last thing that touched a file did | `:197` | keyboard editor | shortcut | Live |

Only the first nine destinations are drawn, silently (`take(9)`,
`src/ui/destinations.rs:74`), and the digit keys that reach them are hardcoded,
along with `Enter` and `Escape` (`src/ui/destinations.rs:113-140`). A tenth
configured destination cannot be reached even with the mouse.

#### `slideshow` — 5 fields, 3 of them reachable

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `seconds_per_image` | How long each picture is held | `src/config/mod.rs:529` | slideshow window (`src/app/panels.rs:100-101`) | number | Live |
| `percent_zoom` | How much closer it creeps over the whole show | `:531` | slideshow window, and only when the motion is Zoom (`src/app/panels.rs:119-127`) | number | Live |
| `motion` | What it does with a picture while it is up: still, zoom, reveal | `:533` | slideshow window (`src/app/panels.rs:110-117`) | enum | Live |
| `start_with_frame_enabled` | Whether the white frame is on when a show starts | `:535` | nowhere — not in the slideshow window | boolean | Live (`src/view/image_view/navigate.rs:86`) |
| `image_frame_background_color_override` | The ground behind the picture during a show | `:537` | nowhere | string | Live (`src/view/image_view/mod.rs:555-565`) |

The three-way radio for `motion`, with a sentence under each variant drawn from
`Motion::description()` (`src/app/panels.rs:110-117`,
`src/config/mod.rs:514-523`), is the best control in the program and the
template the rest of the window follows.

#### `version` — the one key that is not a setting

| Field | What it does | Where | Reachable today | Type | Takes effect? |
|---|---|---|---|---|---|
| `version` | Which build's conventions the file was written to; drives migration | `src/config/mod.rs:25` | nowhere | number | n/a |

It belongs in the footer, shown because a person reporting a problem needs to be
able to read it and read-only because a hand-set version either re-applies
migrations that were already applied or suppresses ones that were not
(`src/config/migrate.rs:55-75`). A value greater than or equal to `CURRENT` is
left alone entirely (`:62-65`).

#### The nested types

Five value types appear inside those fields, and every one of them has
**required** members with no serde default. A single missing key inside a
`Destination`, `TagCategory`, `UserAction`, `ContextMenuEntry` or `Shortcut`
fails the whole enclosing section, which is then replaced by its defaults and
`partial` is set (`src/config/load.rs:175-182`).

| Type | Members | Where |
|---|---|---|
| `Destination` | `label`, `path` — both required | `src/config/mod.rs:215-221` |
| `TagCategory` | `name`, `tags` — both required | `:225-228` |
| `UserAction` | `shortcut`, `exec` required; `callback` optional | `:541-545` |
| `ContextMenuEntry` | `description`, `exec` required; `callback` optional | `:548-552` |
| `Shortcut` | `key`, `modifiers` — both required, deserialised through `ShortcutData` | `src/config/shortcut.rs:14-22`, `:47-51` |

#### The count

| | Fields |
|---|---|
| Reachable through the keyboard editor | 60 |
| Reachable through the slideshow window | 3 |
| Reachable nowhere, but nudgeable by a key and lost on exit | 6 |
| Reachable nowhere by any means | 41 |
| **Total** | **110** |

Cutting the other way: **26 would not take effect even if a control existed** —
25 wholly, plus `grid_view.filmstrip_height`, whose height is read live and
whose visibility is decided once. Three more look restart-bound and are not, and
counting them would blunt the badge (§3.5): `raw.pair_with_jpeg` needs the
folder re-opened rather than the program restarted, `general.restore_session` is
read live and merely describes the next launch, and `tags.panel_width` is stuck
for the session by egui's stored panel width rather than by a startup capture.

---

### 3.2 The window

One `egui::Window`, eleven pages down the left, a search box above them, a
footer under both. It is `App::show_keyboard` with a longer body: that function
already draws a window with `&mut self.settings` in hand, runs a four-line
fan-out when something changed, and calls `save_settings()`
(`src/app/settings.rs:80-95`). The two `set_config` implementations it calls
have one and two lines of body respectively
(`src/view/grid_view/mod.rs:195-197`,
`src/view/image_view/navigate.rs:95-98`). Nothing below needs a new file format,
a second viewport or a second commit model.

```
┌ Settings ─────────────────────────────────────────────────────────────────┐
│ ┌───────────────────┐ ┌────────────────────────────────────────────────┐  │
│ │ 🔍 grey           │ │  The photograph                                │  │
│ ├───────────────────┤ │                                                │  │
│ │ Opening a folder  │ │  The frame and the ground behind it            │  │
│ │ The photograph  ●2│ │   ● The ground behind a photograph   ▓ #777777 │  │
│ │ The contact sheet │ │     · right-click the photograph            ⌖  │  │
│ │ Stars, flags…   ●1│ │     Black · 18% grey · Mid grey · White · …    │  │
│ │ Keywords          │ │                                                │  │
│ │ Moving and delet… │ │   Width of the white frame  [────●───]  20.0 % │  │
│ │ Raw files         │ │     · right-click the photograph            ⌖  │  │
│ │ Slideshow         │ │                                                │  │
│ │ Speed and memory  │ │                                                │  │
│ │ The window        │ │                                                │  │
│ │ Keys and mouse    │ │                                                │  │
│ ├───────────────────┤ │                                                │  │
│ │ The file…         │ │                                                │  │
│ └───────────────────┘ └────────────────────────────────────────────────┘  │
│ One change needs a restart: decode threads.   [Restart now]  [Later]      │
└───────────────────────────────────────────────────────────────────────────┘
```

`#777777` is the ground the program actually draws
(`src/view/image_view/layout.rs:12`) and 20 % is what
`frame_size_relative_to_image` defaults to (`src/config/defaults.rs:133-135`); a
mock-up that shows values the program does not hold is a mock-up that gets built
wrong.

900 × 600 comfortable, 720 × 480 floor, which fits inside the 1092 × 614 logical
space of a 1366 × 768 laptop at 125 %. darktable's preferences window put its
Close button below the bottom of a 14-inch screen and could not be dismissed at
all (<https://github.com/darktable-org/darktable/issues/3858>).

Four rules, each with a source:

- **Vertical navigation, one level, no nesting.** Microsoft: "Use vertical tabs
  if: the property window has eight or more tabs", and "Don't nest tabs"
  (<https://learn.microsoft.com/en-us/windows/win32/uxguide/win-property-win>).
  GNOME's archived guidance is blunter: "Avoid nested tabs like the plague"
  (<https://wiki.gnome.org/Design(2f)HIG(2f)Planning(2f)Configuration.html>).
- **Ordered by how often it is wanted.** Same GNOME page: most-used options at
  the top and in the first tab.
- **Nothing is called General, Advanced or Settings.** Microsoft names those
  three as labels to avoid — "Avoid generic tab labels that could apply to any
  tab" — and adds "Present properties in terms of user goals, not technology."
- **Immediate apply, no OK/Cancel/Apply.** Microsoft carves out this exact case:
  a property *inspector* uses "an immediate commit … so there is no need for OK,
  Cancel, and Apply buttons". Apple's settings windows are modeless with
  immediate apply and no Save/Cancel/Done
  (<https://zenn.dev/usagimaru/articles/b2a328775124ef?locale=en>, quoting the
  HIG page, which serves nothing to a fetch). `App::save_settings` already runs
  the moment anything moves (`src/app/settings.rs:104-110`).

The widget vocabulary is four lines, and §5 assigns it field by field: a number
gets a slider where the range means something to a person and a drag value where
it does not; a choice of up to five gets a radio group with a sentence under
each row, on the model of `motion`, and a combo box above that; a boolean gets a
tick with the sentence beside it rather than under it; a list gets a table with
add, remove and reorder. Nineteen numeric controls exist in the whole program
today — 17 `DragValue` and 2 `Slider` (§5) — which is the measure of how little
of this vocabulary is in use.

The one qualification on immediate apply is arithmetic. `stores::image_store`
and `stores::thumbnail_store` are pure functions of the configuration and
`ImageStore::new` takes the result by value and keeps it
(`src/app/stores.rs:32-66`, `src/cache/store/mod.rs:60`, `:117-122`), so
applying a changed budget means building a fresh store and re-seeding it. A
`ram_budget_mb` slider on true per-frame apply would do that sixty times a
second. So **a rebuild-class field commits when the gesture ends** —
`drag_stopped()` on a slider, focus loss on a box, the click itself on a radio.
One gesture, one rebuild. The cost is stated permanently under the group in real
units, because it is measurable: `--benchmark` walks a folder one image per
frame and reports what it managed, and the figure the README publishes — 120
24-megapixel JPEGs on a 24-core Ryzen — is 43.6 images a second
(`README.md:291`). The banner is computed from the folder in hand and that
machine's own benchmark, not from a number written into the source:

> Changing these empties the cache. This folder holds 2,030 photographs and, at
> the 43.6 images a second this machine last measured, takes about 47 seconds
> to fill again.

**The commit rule is not the window's alone, and it is not only about
rebuilds.** Every route that writes a field writes it through the same setter,
so every route inherits the same coalescing. The values being made authoritative
are not pressed once: `+`, `−` and a Ctrl-wheel in the contact sheet call
`set_columns` once per notch (`src/view/grid_view/mod.rs:516-528`), `Ctrl`+`+`
and `Ctrl`+`−` move `images_shown` once per press
(`src/view/image_view/mod.rs:282-290`), and the splitters §5 puts on
`tags.panel_width`, `general.side_panel_width` and `grid_view.filmstrip_height`
produce a new value on every frame they are dragged. `Config::save` serialises
the entire document and does one `fs::write` (`src/config/load.rs:42-45`), and
the repairs §12 orders put a modification-time stat in front of every write and
a temporary file and a rename behind it. So the rule reads: coalesce on
`drag_stopped()` or key release, and flush the file at most once every few
hundred milliseconds and once on exit. Without that a dragged splitter writes
`config.json` sixty times a second, and each write is a stat, a serialise, a
temporary file and a rename.

#### The eleven pages

| Page | What it is for |
|---|---|
| **Opening a folder** | What turns up when a folder opens, in what order, and whether it remembers where you got to |
| **The photograph** | How a picture looks when it is the only thing on screen |
| **The contact sheet** | How many thumbnails, how large, and what they say |
| **Stars, flags and labels** | Which key marks a frame, and whether it moves on afterwards |
| **Keywords** | Getting a keyword list in, and keeping the ones in use to hand |
| **Moving and deleting** | Where rejects go and what Delete does |
| **Raw files** | Preview against develop, and the four knobs underneath |
| **Slideshow** | How long each picture holds and what it does while it is up |
| **Speed and memory** | The numbers that decide how much of the machine is used |
| **The window** | Text size, colour, which panels come up |
| **Keys and mouse** | The whole key map, the fixed keys, the user actions, and the mouse |

That is the order the list is drawn in. Which field lands on which page is §4's,
row by row; two of those placements need a defence and §4 makes it — **The
photograph** is the longest page, and **Speed and memory** gathers fields
declared in three different structs because they answer one question.

All 110 existing settings are placed on those eleven pages, and **Keys and
mouse** holds two of them outright: `image_view.user_actions` and
`image_view.scroll_navigation`. The plan adds 35 fields (§6.2), one of which is
`mouse.wheel`, and a migration retires `scroll_navigation` into it —
`src/config/migrate.rs` exists to do exactly this and already reports what it
moved (`src/config/mod.rs:43-48`). Two further fields are added outside that
list, because §6.2 enumerates no shortcut and both of these are keys:
`general.sc_settings`, the `Ctrl+,` that opens this window (§3.3), and
`general.sc_context_menu`, the `Shift+F10` that opens a context menu from the
keyboard (§7). **Thirty-seven new fields, then: the pages carry 146 rows and the
file 147 keys**, `version` read-only in the footer being the one that is not a
row. §11.2 argues six of the 146 back out again, which is where the smaller
number the window actually draws comes from; §12 does that arithmetic where it
orders the work.

Twelve of the 35 in §6.2's list are runtime state the program already keeps and
throws away on exit — which is a setting under §4's three-way test rather than
session state — eight are hardcoded constants given a name, nine are the mouse
and six are genuinely new choice; the two added on top are keys, one for the
window this chapter proposes and one for the menus §7 proposes. **What is
mostly being added is persistence, not choice.** That is the answer to the
objection that 146 settings is too many, and Pennington's test is the one that
objection rests on:
"Can said annoyance be made to go away for all users without requiring a
preference? If so, just do that" (<https://ometer.com/free-software-ui.html>).
Nothing here can, because these are the values people actually disagree about —
how much RAM, which grey, how many across. §11 argues the total properly,
including which of them should have been decided rather than exposed.

---

### 3.3 Two routes to every page, and usually three

Microsoft: "Don't make commands only available through context menus. Like
shortcut keys, context menus are alternative means of performing commands and
choosing options."
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus>). Apple
says the same. **Nothing below is reachable only by right-click**, and the list
on the left of the window always works.

Route one is always that list. `Ctrl+,` opens the window on the page last used,
and `Settings ▸ All settings…` is the third entry on a menu that has had two
since it was written (`src/app/panels.rs:67-77`). Plain `Comma` is
`sc_standing_back` (`src/config/defaults.rs:192-194`), so the modified key is
free — and like every other key in the program it belongs in the registry rather
than being hardcoded, as `general.sc_settings`. That field is not one of §6.2's
thirty-five, which enumerates no shortcut at all; nor is
`general.sc_context_menu`, the `Shift+F10` binding §7 introduces. They are the
two the plan adds on top, and §3.2 counts them.

The two existing entries stay as deep links to a page rather than as windows of
their own, because they are the only settings routes anybody has learned:
`Settings ▸ Keyboard…` opens **Keys and mouse** and `Settings ▸ Slideshow…`
opens **Slideshow**. §10 adds `Help ▸ Keyboard…`, which is the same deep link.

The other two routes are the object itself. §7 has the full map of what each
menu carries and in what order; this table says only which surface leads to
which page.

| Page | Route 2 — right-click the thing itself | Route 3 |
|---|---|---|
| Opening a folder | The filter bar (`src/ui/filter_bar.rs:14`; no menu there today) | The folder tree (`src/ui/tree.rs:266`, repaired first) |
| The photograph | The photograph (`src/view/image_view/mod.rs:170` → `interaction.rs:70-80`) | The zoom label, the only menu that draws anything out of the box (`src/view/image_view/bottom_bar.rs:283`) |
| The contact sheet | A cell or the sheet's ground (`src/view/grid_view/mod.rs:460-464`) | The filmstrip (`src/view/grid_view/filmstrip.rs:63-67`) |
| Stars, flags and labels | The marks strip (`src/view/image_view/bottom_bar.rs:184-205`) | The **Advancing** flag (`:148`) |
| Keywords | Inside the tag panel (`src/ui/tag_panel/mod.rs:61-66`) | The panel's empty state, which links to the catalogue control |
| Moving and deleting | A destination slot on the panel (`src/ui/destinations.rs:57-103`) | **None.** The delete confirmation is transient and §7.13 refuses it a menu, so this page has two routes rather than three |
| Raw files | The photograph, when what is on screen is a raw | The **RAW+JPEG** flag (`src/view/image_view/bottom_bar.rs:150`) |
| Slideshow | The photograph during a slideshow | `Settings ▸ Slideshow…`, kept |
| Speed and memory | The cache readout, which already prints held-against-budget for RAM, the adapter and the metadata read-ahead (`src/app/panels.rs:206-238`) | The frame-timings strip (`src/ui/perf_metrics.rs:74`) |
| The window | The menu bar (`src/app/panels.rs:29`) | The side panel (`src/app/chrome.rs:106`) |
| Keys and mouse | `Settings ▸ Keyboard…`, kept | `?` opens the cheat sheet (§10), whose footer opens this page |

#### Several routes to one setting, counted

Three routes to a page is one destination reached three ways. The brief asks for
one *setting* reachable several ways, which is a different claim and a smaller
number, because §7.2 caps a menu's settings group at four rows. Gathered from
§5.8's continuous controls and from the settings rows §7.3 to §7.7 already
sketch, **thirty rows carry a control somewhere other than their own page**:

| Page | Fields with a second control | Where the second control is |
|---|---|---|
| Opening a folder | `browsing.sort`, `browsing.descending`, `browsing.flag`, `browsing.stack_by_default`, `group.max_gap`, `group.tolerance`, `group.min_frames`, `general.start_in` (all §6) | The position counter and the stack place in the bottom bar (§7.4), the sort and flag chips and the `Stacks` toggle in the filter bar (§7.6), the stack badge on a cell (§7.5), the new mode indicator (§7.7) |
| The photograph | `image_view.overlay_corner`, `overlay_format`, `overlay_text_size`, `enlarge_to_fit`, `nr_images_shown`, `general.backdrop` (§6) | The photograph's own menu and the overlay's (§7.3), the **Comparing** flag (§7.4) |
| The contact sheet | `grid_view.images_per_row`, `grid_view.filmstrip_height`, `grid_view.badges` (§6) | A cell's menu and the filmstrip's (§7.5); a rail in the filter bar and a drag handle on the strip's top edge (§5.8) |
| Stars, flags and labels | `tags.advance_after_marking` | The **Advancing** flag (§7.4) |
| Keywords | `tags.panel_width`, `tags.recent_tags` | The panel's menu (§7.6); `panel_width` also has the splitter (§5.8) |
| Moving and deleting | `cull.destinations` | A slot's menu renames it, points it elsewhere and reorders it (§7.6) |
| Raw files | `raw.pair_with_jpeg` | The **RAW+JPEG** flag (§7.4) |
| Slideshow | `slideshow.seconds_per_image`, `motion`, `percent_zoom` | `Settings ▸ Slideshow…`, kept (`src/app/panels.rs:92-135`) |
| Speed and memory | `cache.ram_budget_mb`, `cache.gpu_budget_mb` | The cache readout's menu (§7.6) |
| The window | `general.text_scaling`, `general.panels_at_start` (§6), `general.side_panel_width` (§6) | The menu bar's own menu (§7.7), the side panel's menu (§7.6) and its splitter (§5.8) |
| Keys and mouse | none | The page is the keyboard editor widened; no menu carries a binding |

The residue is the honest part of the answer. The other hundred and sixteen rows
have **one control, reached three ways**: the page list, `Ctrl+,` or
`Settings ▸ All settings…`, and the search box, which finds a row by its name,
its doc comment, another program's word for it or its JSON path (§3.4). That is
a defensible reading of the brief and it is what the plan delivers. A second
control for every row would put seven settings rows on each of the twenty
surfaces §7 maps, against the four §7.2's sources permit.

One caveat that follows from the same ceiling. No menu exceeds twelve rows
including the last (§7.2), and the settings group is the group that gives way
when something is added later — the cell menu is already at twelve (§7.5). So a
row in the table above can lose its second control silently, and the table is
where anybody checking would notice.

#### What the routes depend on

Four things that belong to other chapters, stated once so the dependency is
visible:

- Every context menu is flat, and its last row is always
  `More settings… (<page name>)`. That row is what makes the gesture worth
  making even when it misses: somebody hunting for something the menu does not
  carry still ends one click from the right page, having learned that page's
  name. §7 owns the shape, the ordering and the reason there is one submenu.
- **Shift+F10** opens the menu for whichever surface last held the keyboard
  cursor. Without it there is no keyboard route to any context menu and a
  trackpad without a secondary click has none at all. §7 explains why the
  Windows Menu key cannot join it.
- Two gestures have to be repaired before any right-click route works: the right
  button pans the photograph and then releases into a menu
  (`src/view/image_view/interaction.rs:41-42`; §9), and `secondary_clicked()` on
  a folder row *opens the folder* (`src/ui/tree.rs:266-268`; §7).
- Every menu-bearing surface needs the same marker and the same closing clause
  in its tooltip, or the gesture stays invisible. There are 33 hover calls in 11
  of the 139 files today, which is close enough to none that the habit has never
  formed; §10 owns the hover policy and §7 the clause.

---

### 3.4 Search, reset, and the file

#### Search

A box at the top of the navigation list, holding the cursor when the window
opens. It filters the list to the pages with a match and shows the matching
rows, and **the rows are the controls** — the value changes in the result.

This is the best-evidenced request in the whole research. darktable has five
separate issues asking for search in preferences or shortcuts, filed by
different people over several years: #3423, #6706, #6174, #3604 and #9598.
Three of the five were closed with no activity, by the label the project's
stale bot applies — `no-issue-activity` on #3423, #6706 and #6174 — while the
other two, #3604 and #9598, carry no such label. The fifth of them, #9598, is
a *regression* report: search existed in the old shortcut preferences, was
lost in the new ones, and users noticed
(<https://github.com/darktable-org/darktable/issues/3423>,
<https://github.com/darktable-org/darktable/issues/6706>,
<https://github.com/darktable-org/darktable/issues/6174>,
<https://github.com/darktable-org/darktable/issues/3604>,
<https://github.com/darktable-org/darktable/issues/9598>). Five independent
filings by five people is the argument; how each was closed is a detail.
Android's guidance is explicit: "For complex or deep settings hierarchies, add
search functionality so users can find the correct preference"
(<https://developer.android.com/design/ui/mobile/guides/patterns/settings>).

Four layers of vocabulary, in this order, and all four are data rather than
cleverness:

| Layer | What | Why |
|---|---|---|
| Name and sentence | Written in the photographer's words. `nr_loaded_images` becomes "Pictures decoded either side" | Microsoft: "Present properties in terms of user goals, not technology" |
| The doc comments of `src/config/mod.rs` | Indexed verbatim | They are unusually good — "the bin does not reach a memory card or a network share" (`:184-186`), "a DNG written by Camera Raw carries a 256 pixel preview and nothing else" (`:377-379`) — and are read by nobody. Indexing them means "burst", "grey" and "sidecar" hit without anybody authoring a synonym, and they are maintained by whoever changes the field. 57 of the 110 have no doc comment at all, which is the list of what to write first |
| Authored aliases | Other programs' vocabulary, and the complaint rather than only the noun, taken from the survey in §2 | "Standard preview" is Lightroom's name for `thumbnail_resolution`; "resources" is darktable's for the memory budget (<https://docs.darktable.org/usermanual/4.6/en/preferences-settings/processing/>); "color class" is Photo Mechanic's for a colour label. "Blurry" is what an XnView user called degraded thumbnails in his own words — "Thumbnails quality decreased (became blurry) in Browser when you increased their size from default 128x96 to, eg, 500x500" (<https://newsgroup.xnview.com/viewtopic.php?t=47571>) — so *blurry* is indexed against `thumbnail_resolution` |
| An equivalence table, ~20 lines, applied to the query and not to the index | `memory ram` · `gpu graphics vram card adapter` · `thumb thumbnail` · `colour color` · `grey gray` · `pic picture photo photograph image frame` · `reject discard x` · `delete bin trash recycle` · `raw cr2 cr3 nef arw dng raf` | A new field inherits half its vocabulary for nothing. The `colour`/`color` line is not optional: the program is written in British spelling throughout and half its users will type the other one |

Plus a stop-word list, so that "where do rejects go" becomes "rejects" and lands
on `cull.rejected_folder` rather than finding nothing under a strict all-tokens
rule. Three rules about ranking: an exact field-path match wins outright, so
`cache.ram_budget_mb` pasted out of a forum post lands on the control (VS Code's
`@id` filter, made implicit —
<https://code.visualstudio.com/docs/configure/settings>); **nothing reorders by
use**, because a list that moves under the hand defeats the muscle memory that
makes a search box worth having; and the result is never empty — a failed AND is
re-run as an OR under "Nothing matched all of that. The closest:", with the path
to `config.json` on the line below.

Search also matches the JSON path, and every row's own right-click offers **Copy
setting name**, yielding `cache.ram_budget_mb`. The registry is keyed on the
path and never on the label: nomacs stored shortcuts under their *translated*
names, so they break when the interface language changes
(<https://github.com/nomacs/nomacs/issues/1539>).

`Ctrl+L` stays what it is. The navigator (`src/ui/navigator.rs`) is a path bar;
its Enter means "open this folder" and a settings search's would mean "change
this value". The two overlays name each other in their footers instead of
merging.

#### Changed from the default

- **A bullet in the left gutter**, computed by comparing against
  `Config::default()` — which `src/ui/keys.rs:112` already builds for its own
  reset — **for every row the window draws, or not shipped at all.** darktable
  ships this and its own bug is the warning: only *generated* preferences show
  the marker, so hard-coded and Lua ones silently do not, and the marker cannot
  be trusted (<https://github.com/darktable-org/darktable/issues/19765>).
- **The comparison runs over `serde_json::Value`, not over the structs**, and
  that choice is what makes the all-or-nothing promise keepable. Comparing a
  field with its default needs `PartialEq` on that field's type, and three do
  not have it: `UserAction` and `ContextMenuEntry` derive
  `Deserialize, Serialize, Clone` (`src/config/mod.rs:540`, `:547`) and
  `Callback` derives `Clone, Debug` (`src/actions/callback.rs:11`) — which
  covers `image_view.user_actions` and both `context_menu` lists. `Shortcut` is
  not among them: it has a hand-written `PartialEq` that sorts the modifier list
  before comparing, so `["ctrl","shift"]` and `["shift","ctrl"]` are the same
  shortcut (`src/config/shortcut.rs:31-45`), and `Destination`, `TagCategory`,
  `RawSource`, `RawQuality` and `Motion` all derive it (`src/config/mod.rs:214`,
  `:224`, `:103`, `:114`, `:490`). Three derive lines would close the gap.
  Walking `serde_json::to_value(&config)` against `to_value(&Config::default())`
  path by path closes it without any, and is the walk the bundle loader, "What I
  have changed" and "Save what I have changed…" all need anyway — so it is built
  once and used four times.
- **The bullet is a button.** Clicking it restores the default; its own
  right-click names the value it would restore — *Put this back to 4096 MB*
  (`default_ram_budget_mb`, `src/config/defaults.rs:11-13`). A reset that does
  not say what it is resetting to is a leap. Both halves of that sentence come
  out of the same walk: the current value and the default value are already
  `serde_json::Value`s and print as themselves.
- **A count against each page in the navigation list**, so "I changed something
  and I do not remember where" is answerable without opening eleven pages.
- **What I have changed**, in the footer, listing every non-default field with
  its page. VS Code's `@modified`.

#### Reset, which always carries a scope

Three scopes and never a button without one: **this setting** · **this page** ·
**everything**.

The page and everything scopes show a from→to preview before writing anything:
per row, the field named by its sentence rather than its path, what will change,
what will not, and the restart consequence aggregated into one line.
**everything** writes `config.json.bak` first. When the same feature was asked
for in XnView the author's reply was "It's the same as deleting xnview.ini, is
it really needed?", and two other users immediately flagged the danger —
"definitelly with confirmation dialog, or/and removed options could be saved to
file, e.g. xnview.ini.bak. Otherwise...he could lost all settings!"
(<https://newsgroup.xnview.com/viewtopic.php?t=4350>).

The existing button is renamed. "Put everything back to the defaults"
(`src/ui/keys.rs:111-121`) resets 69 key rows and does so with no confirmation
of any kind. It becomes "Put the 69 key bindings back", it confirms, and it
names the count.

#### The file

The configuration file has no address inside the program. `Config::path()`
exists, returns the right answer on every platform, and has two callers, neither
of them the interface: the keyword-catalogue resolver
(`src/annotations/catalog.rs:190`) and a test in the logger
(`src/logging.rs:191`). Worse, the address the README gives is wrong here: it
says `~/.config/avis-imgv/config.json` (`README.md:594`) while the code uses
`ProjectDirs::from("com", "avis-imgv", "avis-imgv")`
(`src/config/load.rs:14-17`, `src/lib.rs:34-36`), which on Windows is
`%APPDATA%\avis-imgv\avis-imgv\config\config.json` and on macOS
`~/Library/Application Support/com.avis-imgv.avis-imgv/config.json`. The README
is right on Linux and wrong on the other two.

So the footer, below a separator so it does not read as a twelfth page, carries
rows that are searchable like everything else:

| Row | Where | Why |
|---|---|---|
| The path, selectable, with **Open it** and **Show me the folder** | `src/config/load.rs:14-17` | The file the README tells people to hand-edit has no address in the program, and the README's address is wrong on two of three platforms |
| The log file's path, same treatment | `src/logging.rs:37` | Every silent failure in §3.6 ends there. The log's own path is written into the log (`src/main.rs:34`) and nowhere a person can see it |
| `version`, read-only | `src/config/mod.rs:25` | The one key nobody should change |
| **What I have changed** | — | Above |
| **Save what I have changed…** / **Load a file somebody sent me…** | — | Below |
| **Put everything back…** | — | Confirms with a preview, names the count, writes `config.json.bak` first |
| **Restart now** | `src/session.rs:99`, `src/app/mod.rs:866-872` | Saves the session and relaunches. `on_exit` only writes the session when `restore_session` is on, so the button calls `Session::save` itself rather than relying on the exit path |
| A red banner when `partial` is set, naming the section | `src/config/load.rs:27-32`, `:178` | Below |
| The migration report | `Config::migrated`, `src/config/mod.rs:48` | Today a six-second band (`src/app/mod.rs:244-246`) |

One more thing about the file itself, which costs three characters to fix: **on
a fresh install the file people are told to hand-edit is written as one line.**
`fetch_cfg` writes `serde_json::to_string(&default_cfg)` — compact
(`src/config/load.rs:69`) — while `save()` is pretty-printed (`:42-45`), and
`save()` only runs after the keyboard or slideshow editor, or after a migration
(`:101`). The doc comment on `save` describes the one-liner as a thing of the
past (`:19-23`); for anybody who has never opened either editor it is the
present.

And `examples/config.json`, which the README calls "fully populated"
(`README.md:594-595`), is not: it holds 103 of the 110 settings. It has no
`version` key, no `tags.catalog_file`, and none of the six stacking shortcuts
(`general.sc_stacks`, `sc_toggle_stack`, `sc_standing_back`,
`sc_standing_forward`, `sc_previous_stack`, `sc_next_stack`). Somebody who
copies it wholesale gets `version: 0` and both migration steps re-applied
(`src/config/migrate.rs:36-49`).

#### A file that could not be read

When one section fails to parse, `partial` is set, **all saving is blocked for
the rest of the session** (`src/config/load.rs:27-32`), and the user is told
once, for 6.6 seconds, by a band that holds six seconds and fades over 600
milliseconds (`src/app/mod.rs:248-253`, `src/ui/notice.rs:16-19`). Which section
failed reaches only the log (`src/config/load.rs:178`). There is no persistent
indicator and no way to ask again.

Instead: a red bar across the top of the settings window, permanent for the
session, naming the section — *"The `cache` section of your configuration file
could not be read, so nothing here is being saved."* — with **[Show me the
file]** and **[Use the defaults for that section and start saving again]**, and
every control below it drawn *disabled rather than hidden*. That is Microsoft's
rule for an inapplicable page, and the reason it gives is the right one: greying
out the whole thing would mean "users looking for a specific property would be
forced to look on all other tabs".

The keyboard editor has a matching gap worth closing at the same time:
`keys::State.status` (`src/ui/keys.rs:30`) is read at `:123-124` and assigned
nowhere, so a save that succeeded says nothing and a save that failed goes
through the same fading band (`src/app/settings.rs:104-110`). §9 has the
editor's other faults.

#### Sending a configuration to another machine

Kept, in the smallest form that answers what has actually been asked for.
digiKam has no export at all — "I can't for the love of God find a way to export
my Digikam settings… I checked and rechecked all the options in the settings
pane: nothing"
(<https://discuss.kde.org/t/no-way-to-export-digikam-setting/9991>); XnView's
answer from the author is one line, "you can export xnview.ini"
(<https://newsgroup.xnview.com/viewtopic.php?t=47091>). No request for settings
*profiles* in an image viewer turned up anywhere in the research; the pattern
belongs to developer tools
(<https://code.visualstudio.com/docs/editor/profiles>). Profiles, and any form
of sync, go to §13.

**A bundle is a patch, never a snapshot.** It names only the fields it sets:

```json
{
  "name": "The laptop",
  "written_by": "avis-imgv 0.4.0",
  "sets": {
    "cache.ram_budget_mb": 1536,
    "cache.gpu_budget_mb": 256,
    "image_view.nr_loaded_images": 96,
    "raw.source": "preview"
  }
}
```

Four fields named, the other 142 untouched. The reason is documented: Geeqie
loses settings across a version change because it regenerates the file from
scratch on exit, and the reporter's own diagnosis is "a direct consequence of
the approach of regenerating config file from scratch each time Geeqie is
closed" (<https://github.com/BestImageViewer/geeqie/issues/569>). A patch cannot
do that, and a bundle written by an older build stays valid because the fields
it does not know about are the fields it does not name.

**How a patch reaches a typed field.** The bundle names dotted paths and untyped
values; the registry's accessor pairs read and write Rust values. Nothing maps
one to the other directly, and nothing should: they meet through serde, the same
way the changed-from-default walk does.
`serde_json::to_value(&config)` gives a document, the dotted path is walked into
it, the leaf is replaced, and `serde_json::from_value` gives a `Config` back.
That is the `serde_json::Map` machinery `from_json` already keeps
(`src/config/load.rs:133-142`), and it supplies the from→to preview its text for
nothing, because both sides of every row are already values that print as
themselves. Two failures are reported **per row rather than failing the whole
load**, because a bundle from another build is the ordinary case and not the
exceptional one:

- a path this build does not know — *"`cache.vram_budget_mb`" is not a setting
  in this build*; and
- a value of the wrong type — *"`cache.ram_budget_mb`" wants a number and this
  file has "1536"*.

Either draws its row unticked with the complaint beside it, and the rest of the
bundle loads. Export is the same walk in the other direction: `to_value` of the
configuration and of `Config::default()`, and every path where the two differ
becomes a line in `sets`.

Four rules:

1. **"Save what I have changed…" writes only the fields that differ from the
   default.** That is a small, readable, diffable file — and it is what makes
   the footer's "What I have changed" a file rather than a list.
2. **Key bindings are opted into, never included by default.** A bundle is a
   settings bundle or a keys bundle and the save dialogue asks which. A shared
   file that silently rebinds `x` is the Adobe complaint in miniature — "It's
   very frustrating that everytime Photoshop updates it erases all of my
   customizations. All keyboard and mouse shortcuts, all preferences, all get
   reset back to default"
   (<https://community.adobe.com/questions-712/please-stop-making-updates-erase-settings-1168058>).
3. **Machine-specific paths are listed separately and unticked.**
   `cull.destinations`, `tags.catalog_file` and the `exec` strings in
   `user_actions` name folders on one computer. digiKam bug 267131 is exactly
   absolute paths baked into an exported configuration
   (<https://bugs.kde.org/show_bug.cgi?id=267131>).
4. **Loading one shows the from→to preview first**, with a tick per row so any
   field can be left alone.

Bundles live in `<config dir>/bundles/*.json`, beside `config.json`. Export is
copying the file.

---

### 3.5 Restart

Twenty-six settings would not take effect even with a control: twenty-five that
are read once at construction and never re-read, and
`grid_view.filmstrip_height`, which is half so. Nothing in the program says so.
The position taken here is that **a restart badge is a bug report, not a
feature**: `Effect` is a field on every registry row with three values —
**Live**, **Rebuild**, **Restart** — and after the work below, Restart holds one
row in the whole register.

#### The twenty-five that become live or rebuild

Twenty-four of the twenty-five wholly restart-bound fields, plus the
restart-bound half of `grid_view.filmstrip_height`. The first five rows below
are one call, not one fix each.

| Field(s) | Read once at | Why it does not take effect | How it is fixed |
|---|---|---|---|
| `cache.ram_budget_mb`, `gpu_budget_mb`, `previews_resident`, `full_resolution_neighbours`, `upload_budget_ms` | `src/app/stores.rs:34-41`, `:50-60` via `src/app/mod.rs:164-165` | `ImageStore::new` takes a `StoreConfig` by value and holds it in `self.config` (`src/cache/store/mod.rs:60`, `:117-122`) | Give `ImageView` and `GridView` a `rebuild(StoreConfig)` — which is **not** `set_images`; see below → **Rebuild** |
| `raw.source`, `quality`, `camera_white_balance`, `auto_brighten`, `highlight_mode` | `src/app/stores.rs:69-81` | Baked into `StoreConfig.raw` at construction | Same rebuild, image store only → **Rebuild** |
| `image_view.nr_loaded_images`, `gpu_resident_images`, `max_image_edge` | `src/app/stores.rs:35`, `:37`, `:38` | Same | Same rebuild, image store only → **Rebuild** |
| `grid_view.preloaded_rows`, `thumbnail_resolution`, `gpu_resident_thumbnails` | `src/app/stores.rs:51`, `:54`, `:55` | Same | Same rebuild, thumbnail store only → **Rebuild** |
| `general.output_icc_profile` | `src/app/mod.rs:163` | An `Arc<str>` handed to both stores at construction, alongside the `StoreConfig` rather than inside it | Same rebuild, which therefore takes the profile as a second argument → **Rebuild** |
| `general.text_scaling` | `src/app/mod.rs:150` → `:888-894` | `apply_text_scaling` is called once | Call it again when the field changes, from a base style captured once rather than from the current one; see below → **Live** |
| `image_view.nr_images_shown` | `src/view/image_view/mod.rs:129` | `set_config` replaces `self.config` and never touches `self.images_shown` (`src/view/image_view/navigate.rs:95-98`) | Add `self.images_shown = config.nr_images_shown.clamp(1, MAX_IMAGES_SHOWN);` to `set_config` — the clamp already written at `:129`, and `MAX_IMAGES_SHOWN` is visible from the child module (`src/view/image_view/mod.rs:51`) → **Live** |
| `grid_view.images_per_row` | `src/view/grid_view/mod.rs:77`, and `src/app/stores.rs:54` for the preload radius | `set_config` replaces `self.config` and never touches `self.columns` (`src/view/grid_view/mod.rs:195-197`) | Add `self.set_columns(config.images_per_row.clamp(1, MAX_COLUMNS));` — `set_columns` exists at `:643` and `MAX_COLUMNS` at `:42`, both in the same `impl` → **Live**, plus the thumbnail-store rebuild for the radius |
| `grid_view.filmstrip_height` (whether it starts up) | `src/app/mod.rs:175` | Visibility is derived from the height once, at startup | Split into `filmstrip_visible` and `filmstrip_height` — which also makes "keep the height, hide the strip" expressible, and it is not today, and which is what lets the height control have a sensible floor instead of carrying "off" as the value 0 → **Live** |
| `tags.advance_after_marking` | `src/app/mod.rs:176` | `App::advancing` is seeded once and then flipped by the key at `:706` | Make `:706` write the field and hand it to the same coalesced save as every other route (§3.2), which is how the runtime toggle stops being a thing that is thrown away → **Live** |
| `tags.categories`, `catalog_file`, `recent_tags` | `src/app/mod.rs:209-210` | `Catalog::configured` and `RecentTags::load` run once | Rebuild the catalogue from the configuration the same way the stores are rebuilt → **Rebuild** |

Twenty of the twenty-five become Rebuild rather than Live — seventeen through
the store rebuild and three through the same treatment of the keyword catalogue.
That is a real distinction and it is stated on the row: the value takes
effect at once, and the cache empties to do it. Five become Live, one of them
the restart-bound half of `grid_view.filmstrip_height`; three of those five
are a single line each.

**What `rebuild` costs, stated because the row above hides it.** `rebuild` is
not `set_images`. `ImageView::set_images` resets `visible` to
`Visible::everything`, clears the viewports and re-selects
(`src/view/image_view/navigate.rs:22-32`); `GridView::set_images` clears the
selection and forces `cursor`, `current` and `scroll_to` back to zero
(`src/view/grid_view/mod.rs:90-100`); and `ImageStore::set_paths` underneath
bumps the generation, empties every cache and queue it holds, drains the
in-flight results and sets `cursor = 0` (`src/cache/store/mod.rs:216-245`).
Opening a folder is the one case where losing all of that is correct. Moving the
RAM slider mid-cull is not: it would lose the filter, a two-hundred-frame
selection, the place in the folder and every per-photograph zoom and pan, on
every gesture. So `rebuild` calls `store.set_paths` and then puts back by hand
the `Visible` in force, the `Selection`, `cursor` and `current`, the
`Viewports`, the comparison and the marking mode — and that list is the real
cost of the row. Both views must also keep the `RenderState`, the `Arc<Loader>`
and the `Arc<str>` profile that `ImageStore::new` consumes today
(`src/view/image_view/mod.rs:99-111`, `src/view/grid_view/mod.rs:69-76`);
`RenderState` is `Clone` and the other two are already `Arc`s. The display edge
re-arrives on the next frame (`src/app/mod.rs:796-797`). The call goes beside
the existing fan-out at `src/app/settings.rs:89-92`.

Two riders on it:

- **Rebuild only the store the changed field feeds.** The thumbnail store is
  handed `raw::Options::default()` and literal zeroes for `previews_resident`
  and `full_resolution_neighbours` (`src/app/stores.rs:58-64`), so the five
  `raw.*` fields and two of the `cache.*` ones cannot affect it and must not
  empty it. The reverse holds for the three `grid_view.*` store fields.
- **A rebuilt store loses its metadata read-ahead.** `Scanned` is a field of the
  store, constructed empty (`src/cache/store/mod.rs:86`, `:194`), and it is what
  the side panel, the status bar and the filter read
  (`src/cache/scanned.rs:1-7`). Those three blank until the front of each file
  has been read again. The banner in §3.2 has to say so as well as saying the
  cache empties.

**And what the text-size row costs.** `apply_text_scaling` clones the *current*
style and multiplies its sizes (`src/app/mod.rs:888-894`), so calling it twice
compounds: 1.25 then 1.5 gives 1.875. It must scale from a base captured once,
and that base cannot be `egui::Style::default()`: that carries
`Visuals::default()`, so setting it would replace the theme's six hand-picked
colours, five widget palettes and `override_text_color`
(`src/ui/theme.rs:26-57`) with egui's stock dark, the first time anybody touched
the rail. Capture the style immediately after `apply_theme`
(`src/app/mod.rs:149`), scale from that, and capture it again whenever
`general.theme` changes — which §6.3 makes possible in the same release.

`Context::set_zoom_factor` is not the equal alternative it looks. It changes
`pixels_per_point`, `longest_edge_in_pixels` multiplies the monitor size by
`ctx.pixels_per_point()` (`src/app/mod.rs:881-886`), and that number is handed
to `ImageStore::set_display_edge` on every frame (`:796-797`) — a value whose
stated contract is "Only ever raised, and in coarse steps" and which never
lowers (`src/cache/store/mod.rs:262-276`). Raising the text size that way
raises the decode edge for the rest of the session: more RAM per photograph,
slower decodes, a cache holding fewer frames, and lowering the text size again
undoes none of it.
Adopting it would mean making `set_display_edge` follow the zoom factor down as
well as up, which changes the store's contract and re-decodes the folder each
way. The stored base style is the route.

The same repair has a second half that is easy to miss. Once the configuration
is authoritative, the keys that nudge these values at runtime —
`overlay_corner`, `nr_images_shown`, `images_per_row`,
`advance_after_marking`, the filmstrip toggle — have to write the field too,
or the next save from the settings window will snap the view back to whatever
the file still says. They write it through
the coalesced setter of §3.2, not through a `Config::save` per keystroke.

`tags.panel_width` is not one of the twenty-six and needs the same treatment for
a different reason: `.default_width()` is honoured only while egui has no stored
width for that panel id (`src/app/tagging.rs:124` →
`src/ui/tag_panel/mod.rs:64`), so a mid-session change does nothing. Make the
field authoritative both ways — write the dragged width back into it from the
`InnerResponse` that `show_animated` returns, and draw one frame with
`.exact_width(width)` when the settings window is what changed it → **Live**.

#### The one that genuinely cannot

**`cache.decode_threads`.** The pool is spawned once in `Loader::new`, each
thread is `.expect`-ed, and one `Arc<Loader>` is shared by both views
(`src/cache/loader.rs:108-133`, `src/app/mod.rs:162`). Draining a running pool
mid-session is a larger job than a settings chapter should smuggle in, and
pretending otherwise would be dishonest; §13 records that it is not attempted.
This is the one row that keeps the badge, and the badge exists for it.

**`general.interface_font`, when it exists (§6), is not a second one.**
`Context::set_fonts` is an ordinary runtime call on the live context, and the
program already makes it — `apply_fonts` is handed `&cc.egui_ctx` at startup and
calls `ctx.set_fonts(fonts)` (`src/ui/theme.rs:77-101`), which nothing stops
being called again. What the field would need is a way to enumerate installed
families and read their bytes, and no dependency in `Cargo.toml` does that. So
the obstacle is a dependency, not a restart, and the field does not carry the
badge.

#### How it is marked, and what is deliberately not marked

- **A badge on the row**, `↻`, with "takes effect the next time the viewer
  starts" written *under the control rather than in a tooltip*. NN/g's rule is
  that "directly actionable information, like field requirements, shouldn't be
  in a tooltip" (<https://www.nngroup.com/articles/tooltip-guidelines/>), and a
  restart requirement is a field requirement. §10 owns the rest of the hover
  policy.
- **A persistent footer while anything is waiting** — *"One change needs a
  restart: decode threads."* — with **[Restart now]**, a button that saves the
  session (`src/session.rs:99`) and relaunches. darktable's own fix for this
  complaint was a toast on closing the dialogue
  (<https://github.com/darktable-org/darktable/pull/5957>); a button that does
  the thing is strictly better than a notice about needing to do it.
- **One banner across a page, not one marker per row, wherever a page is
  uniformly restart-bound.** After the rebuild work no page is, so the banner is
  never drawn — the rule is written down for whichever page next becomes
  uniform.
- **A setting about the next launch is not a restart.**
  `general.restore_session`, and the new `general.start_in`,
  `general.start_fullscreen` and `general.panels_at_start`, have taken effect
  the moment they are set; they simply describe a future event. They get a
  sentence — "this is what happens when the viewer next starts" — and no
  badge. A badge
  means *your change has not taken effect*, and using it for these four is what
  teaches people to ignore it. The same goes for `raw.pair_with_jpeg`, which
  needs the folder re-opened and says so.

§12 says when this work happens and what finishes it.

---

### 3.6 Validation

`Config::from_json` does no range validation at all — it catches structural
parse failures per section and nothing else (`src/config/load.rs:125-162`), and
`grep clamp src/config/` finds nothing. **That is the correct state for
`Config`, and it must stay that way**, for a mechanical reason: `Config::save`
serialises the whole struct and writes the whole document
(`src/config/load.rs:42-45`), so a value clamped on load would be written back
clamped the first time anything unrelated changed, destroying somebody's
deliberate 8,192 MB budget.

So the rule is:

> **A value outside a control's range is shown, marked out of range, and left
> exactly as written. The window never rewrites a field it did not edit. The
> bound lives at the consumer, not in the configuration.**

That reconciles the two demands — cap the threads, do not rewrite the file —
that would otherwise be in conflict, and it means hand-editing always wins,
including hand-editing to a value the window cannot produce.

#### Every unguarded number

The ranges below are the ones §5 draws its controls to.

| Field | Guard today | What a bad value does | Where the bound goes |
|---|---|---|---|
| `general.text_scaling` | **none** | `0.0` multiplies every text style by zero, so the whole interface is invisible — including the menu that would undo it | A floor in `apply_text_scaling` (`src/app/mod.rs:888-894`); 50–300 % |
| `cache.decode_threads` | **no upper cap** | The ceiling of eight applies only to the configured `0` (`src/cache/loader.rs:109-113`, `:252`); a typed `1000` spawns exactly that many threads and each is `.expect`-ed, so a typo can take the process down on a machine that refuses the spawn | A cap in `Loader::new` before the spawn loop (`src/cache/loader.rs:122-130`) |
| `cache.ram_budget_mb` | `.max(1)` (`src/app/stores.rs:97`) | No ceiling; `40960` for `4096` swaps the machine | 256–65536, logarithmic, with the cache readout's own held-of-budget line beside it (`src/app/panels.rs:206-238`). The machine's fitted RAM is what darktable's presets are stated against, but nothing in `Cargo.toml` reads it, so it is a dependency decision rather than a free label |
| `cache.gpu_budget_mb` | `.max(1)` (`src/app/stores.rs:89`) | Never checked against what the adapter has, and cannot be: wgpu reports texture limits, not memory — `src/cache/gpu.rs:131` reads `max_texture_dimension_2d` and there is no VRAM total anywhere in the safe API | 128–16384, with the same held-of-budget readout (`StoreStats.gpu_bytes` / `gpu_budget_bytes`, `src/cache/mod.rs:111-112`) rather than a claim about the card |
| `cache.upload_budget_ms` | **none** | `0` is legal (`Duration::from_millis(0)`, `src/app/stores.rs:41`); `10000` blocks a frame for ten seconds | 1–16 ms, with "8 ms is half a frame at 60 Hz" beside it — the sentence already in the source, once it is moved back above the field it describes (`src/config/defaults.rs:35-43`) |
| `cache.previews_resident` | 0 means off (`src/cache/store/previews.rs:27`) | Unbounded upwards; the radius is half the value (`:18`, `:32`) | 0–256, with the megabytes the store already reports (`StoreStats.preview_bytes`) |
| `cache.full_resolution_neighbours` | 0 means off (`src/cache/store/mod.rs:156-160`) | Unbounded. The doc comment says only "a number to raise only if there is memory going spare" (`src/config/defaults.rs:27-30`), which is a warning without a quantity | 0–8, with the megabytes taken from what is already held rather than guessed (`StoreStats.at_full_resolution` and `resident_bytes`, `src/cache/mod.rs:101`, `:104`) |
| `image_view.nr_loaded_images` | **none** | Unbounded; trimmed in practice by the RAM budget, which is not the same as being bounded | 0–4096, logarithmic, with the note the store can already compute: the budget divided by the mean size of what it is holding (`resident_bytes` over `in_ram`, `src/cache/mod.rs:99`, `:104`) |
| `image_view.gpu_resident_images` | **none** | Unbounded | 1–64, derived MB |
| `image_view.max_image_edge` | `0` means no limit (`src/app/stores.rs:106-108`); the configured value is already capped at the adapter limit (`src/cache/store/mod.rs:162-167`) | The upper end is safe. The lower end is not guarded at all: `64` silently makes every photograph a postage stamp and nothing says why | A tick for "As large as the card allows (16384 px here)", then 512–32768. The number in that label is `gpu.max_texture_edge()` (`src/cache/gpu.rs:129-132`), already read at `src/cache/store/mod.rs:165`; it has to be exposed through `StoreStats` to be shown |
| `image_view.frame_size_relative_to_image` | **none** | `5.0` draws a frame five times the shorter edge of the picture (`src/view/image_view/canvas.rs:356-357`) | 0–25 % of the shorter edge; the default is 20 % (`src/config/defaults.rs:133-135`) |
| `grid_view.preloaded_rows` | **none** | Unbounded | 0–20 |
| `grid_view.gpu_resident_thumbnails` | **none** | Unbounded | 8–2048, step 8, derived MB |
| `grid_view.thumbnail_resolution` | `0` means no limit, and is likewise capped at the adapter limit (`src/cache/store/mod.rs:162-167`) | Same as `max_image_edge`: safe above, unguarded below | A tick for "Match this display (2560 px)", from `longest_edge_in_pixels` (`src/app/mod.rs:881-886`), then 128–4096 |
| `grid_view.filmstrip_height` | `> 0` check only (`src/app/views.rs:139`) | `5000.0` claims the whole window — the strip is a panel with `.exact_height(height)` (`src/view/grid_view/filmstrip.rs:66`) | 60–400 pt, which only works once §3.5 has split "off" out into `filmstrip_visible`; today the default *is* 0 (`src/config/defaults.rs:299-301`) |
| `grid_view.images_per_row` | `.max(1)` (`src/view/grid_view/mod.rs:77`, `src/app/stores.rs:54`) | `MAX_COLUMNS` bounds the `+` key (`:524`) and not the configured value | 1–16, the same ceiling both ways |
| `raw.highlight_mode` | **none** | Anything 0–255 is handed to LibRaw as an `i32` (`src/decoder/raw/mod.rs:106`) | Four named choices, with a passes box 3–9 inside the fourth |
| `tags.panel_width` | `min_width(180.)` at draw time (`src/ui/tag_panel/mod.rs:65`) | `-40` is stored happily and silently ignored | 180–800 pt |
| `tags.recent_tags` | `.max(1)` (`src/annotations/recent.rs:27`) | `0` cannot turn the list off, though it reads as though it should | 0–100, where 0 genuinely means off |
| `slideshow.percent_zoom` | GUI only, `0.0..=200.0` (`src/app/panels.rs:126`) | The file may hold `-500` | The same range, enforced the same way, and the file may still say otherwise |
| `slideshow.seconds_per_image` | GUI `1..=600` (`src/app/panels.rs:101`), `.max(1)` at use (`src/view/image_view/slideshow.rs:36`) | The file may hold `0` | 1–600 s, logarithmic |

Three fields are already defended correctly and are the pattern to copy:
`cell_aspect`, where non-finite or ≤ 0 falls back to 1.0 with a test
(`src/view/grid_view/layout.rs:44-48`, `:127-131`); `overlay_text_size`,
`.max(8.0)` (`src/view/image_view/overlay.rs:107`); and `nr_images_shown`,
`.clamp(1, MAX_IMAGES_SHOWN)` (`src/view/image_view/mod.rs:129`).

#### Every string, path and enum that fails in silence

Most of these end in a `tracing::error!` or `tracing::warn!` in a file whose
path the program never states. `Config::check()` runs once at load and returns
`Vec<(field, complaint, what was used instead)>`, drawn as a strip at the top of
the settings window that does not fade — one row per complaint, each with a
**[Fix]** button that opens the owning page and focuses the control. **The route
from the complaint to the control is the whole value**, and it is the same rule
§8 applies between views.

| Field | What happens today | Where | What the row says instead |
|---|---|---|---|
| `general.output_icc_profile` | Only three substrings resolve, matched by `contains`; anything else logs once per decode and the picture renders uncorrected | `src/metadata/icc.rs:11-15`, `:22-28`, `src/decoder/color.rs:30-32` | "'sRGV' is not a profile this build knows. Colours are not being converted." A choice of the three bundled profiles and nothing else — there is no path that loads a profile from a file, and the README's answer is to edit `src/metadata/icc.rs` and rebuild (`README.md:586-587`), so a free-text box would offer a choice the program cannot honour |
| `general.metadata_tags[]` | A misspelled exiftool tag is skipped with a bare `continue` | `src/app/panels.rs:151-154` | Two columns: every tag found on the open photograph on the left, the chosen list on the right |
| `tags.catalog_file` | A relative path is resolved against the *configuration* directory, not the working one; a bad path yields a `tracing::warn!` and a shorter keyword list | `src/annotations/catalog.rs:53-61`, `:180-194` | A path picker with **Read it now**, reporting "2,114 keywords in 31 groups" or "No file there", and a line stating which directory a relative path is taken against |
| `cull.rejected_folder` | Blank makes the reject-to-folder key a silent no-op | `src/app/cull.rs:323-326` | A box that refuses empty: "Empty, so the reject-to-folder key does nothing" |
| `cull.destinations[].path` | An empty path is dropped without a word | `src/app/cull.rs:386` | "Empty, so this one is skipped", and for a relative path, "taken against whichever folder is open" |
| `cull.destinations` beyond nine | Silently truncated by `take(9)` | `src/ui/destinations.rs:74` | The table draws the tenth greyed with "the panel reaches nine" |
| `image_view.overlay_format`, `name_format`, `grid_view.caption_format` | An unknown placeholder expands to nothing | `src/metadata/template.rs:287` | The `PLACEHOLDERS` table (`src/metadata/template.rs:321-339`) beside the box as clickable chips, and a live before/after from the photograph currently open — which the bulk-rename view already has as its "Now / Would become" table and nothing else does (`src/view/organize/rename.rs:26`, rows built at `:119-133`) |
| `slideshow.image_frame_background_color_override` | A hex parse failure is swallowed by `.ok()` and the default ground is drawn | `src/view/image_view/mod.rs:555-565` | Not possible: named swatches, a picker, and a validated hex box |
| `user_actions[].exec`, `context_menu[].exec` | A program that does not exist logs and returns `false`; the source itself asks `//Show toast with result?` | `src/actions/user_action.rs:88-99` | A **Test** button that runs it against the open photograph and reports what came back |
| `*.callback` | Any unrecognised string becomes `NoAction`, so `"Relaod"` is accepted and does nothing | `src/actions/callback.rs:39-46` | Not possible: a named choice over the five |
| `Shortcut.key` | An unknown key name becomes the sentinel `Ctrl+Alt+Shift+Cmd+F20`, which no keyboard has | `src/config/shortcut.rs:138-151`, `:178-187` | "Not a key name — this command cannot be reached", and the row is listed as unbound |
| `Shortcut.modifiers[]` | An unknown modifier is logged and ignored | `src/config/shortcut.rs:164-175` | The editor writes the list; a hand-edited one is reported |
| A missing member of `Destination`, `TagCategory`, `UserAction`, `ContextMenuEntry` or `Shortcut` | The whole enclosing section falls back to defaults and `partial` is set for the session | `src/config/load.rs:175-182` | The red bar of §3.4, naming the section |

The `Shortcut.key` row is worth stating plainly, because it is the worst of
them: **a typo in a key name makes a command permanently unreachable and the
only record is a line in a log file whose location the program never gives.**

#### Getting back from a `text_scaling` of `0`

Four ways, in the order somebody would find them:

1. **It cannot be produced through the window.** The floor is 50 %.
2. **It cannot take effect if it arrives from the file**, once the floor above
   is added to `apply_text_scaling` (`src/app/mod.rs:888-894`) — the interface
   stays readable and the value is listed in the `Config::check()` strip as out
   of range.
3. **The file has an address.** The footer prints `Config::path()` with **Open
   it** and **Show me the folder**, and those rows are searchable.
4. **`--reset-text-size` on the command line**, alongside the three flags that
   already exist (`src/main.rs:43-45`, `src/lib.rs:39`), for the case where the
   interface is unreadable for some other reason.

---

### 3.7 What holds it together, and what it is weakest at

Everything above falls out of one table. `src/config/bindings.rs` already is
that table for the 60 shortcuts: `binding!` builds a struct literal whose
non-capturing closures coerce to `fn` pointers (`:69-82`), so the widened
version — the registry — can be a `static` slice rather than the `Vec` that
`bindings::all()` allocates on every frame the editor or the cheat sheet draws
(`src/ui/keys.rs:61`, `src/ui/cheat_sheet.rs:48`). Each row carries page, group,
label, sentence, aliases, path, kind, `Effect` and the accessor pair. The pages,
the search, the changed-from-default bullet, the per-field reset, the restart
footer, the load-time check, the export and the cheat sheet are all views over
it, written once. `bindings::all()` becomes a filtered view, so
`src/ui/keys.rs` and `src/ui/cheat_sheet.rs` keep working unchanged.

Two tests keep it honest, and without them the register drifts back to the 63
fields the program reaches today:

| Test | What it asserts |
|---|---|
| `every_field_is_in_the_index` | A `serde_json::to_value(Config::default())` walk that fails the build if the file carries a key the registry has not heard of, or the registry an entry the file does not. This is the generalisation of the count assertion that already exists at `src/config/bindings.rs:530-541`, whose own comment says why: "The count is what stops a shortcut being added to the configuration and quietly left out of the editor". It is the same walk the changed-from-default marker and the bundle loader use (§3.4) |
| `the_index_answers_these_questions` | A table of `(query, expected field)`, every phrase taken from §2's survey of what people ask other programs — "blurry thumbnails" (the XnView reporter's own word, §3.4), "where do rejects go", "why is my raw small", "color class", "text too small", "cr3". It fails the build the moment a new setting steals an old query. It cannot catch an omission, only a regression |

Three things this design is weak at, said plainly.

**The category boundaries are guesses.** NN/g's own instruction is to choose a
disclosure split using "task analysis and field studies", "frequency-of-use
statistics" and "observational usability testing"
(<https://www.nngroup.com/articles/progressive-disclosure/>). None of that was
done. Eleven names came from reading the source and borrowing the vocabulary
other tools use, and "Opening a folder" against "The window" for
`restore_session` was settled with a mirror because it could not be settled with
evidence.

**The best-evidenced request is search, not structure.** darktable has five
issues asking for preferences search and none asking for better categories. It
may be that the tree is the secondary route to the search rather than the other
way round, in which case most of §3.2 is scaffolding around a filtered flat list
and the eleven names matter mainly as the headings the results sit under.

**None of this is the most valuable work.** The most useful change to
`decode_threads` is not a nicer control but a cap in `Loader::new` so a typo
cannot take the process down (`src/cache/loader.rs:122-130`). The most useful
change to the twenty-six restart-bound settings is rebuilding the stores, not
labelling them. A settings window earns its place here because forty-seven
settings have no control at all — but every hour spent on the sliders of **Speed
and memory** is an hour not spent making that page unnecessary.

## 4. Where each setting belongs

The configuration has eight sections. They are the eight Rust structs, in the order the `Config`
struct declares them (`src/config/mod.rs:26-33`), and nothing else decided them. The file on disk
follows that order, because `serde` serialises fields in declaration order. The README's
configuration chapter uses the same eight divisions in a *different* order — Cache, General,
Image view, Grid view, Raw, Cull, Tags, Slideshow (`README.md:611,629,647,662,676,713,723,738`)
— which is the first sign that the order carries no meaning: the person documenting the file did
not keep it and lost nothing by not keeping it. Either way, the only map a user is given of their
own settings is a map of somebody else's data model.

Nor is it a complete map. `slideshow.motion`, `image_view.overlay_corner`,
`image_view.overlay_format`, `image_view.overlay_text_size` and `grid_view.caption_format` appear
nowhere in the README at all, and `motion` is the best control the program has
(`src/app/panels.rs:110-115`).

The settings window itself is §3's, the control each row gets is §5's, and the fields that do not
exist yet are §6's. This chapter owns the map: why the eight sections mislead, what the eleven pages
are, what every one of the 110 fields is called on the page that holds it, which fields need to
appear twice, where the 69 key rows go, and the rule that decides whether a thing the user changes
at runtime belongs in `config.json` at all.

### 4.1 What the sections hold today

| Section | Fields | What it actually holds |
|---|---|---|
| `image_view` | 36 (`src/config/mod.rs:338-434`) | How a photograph is drawn, how it is zoomed and panned, what the overlay says — **and** three cache budgets, the wheel behaviour, the external-program list and the context menu |
| `grid_view` | 14 (`:437-487`) | The contact sheet — **and** three more cache budgets, **and** the filmstrip, which is drawn under the photograph rather than in the sheet (`src/app/views.rs:137-147`) |
| `general` | 25 (`:264-335`) | The screen profile, the text scale, the side-panel tag list, session restore, and twenty-one keys, six of which are about stacks and two of which are about the bin |
| `cache` | 6 (`:233-261`) | Six of the twelve numbers that decide the caches. Two of the six do nothing for the contact sheet at all |
| `slideshow` | 5 (`:527-538`) | The slideshow — **and** the only background colour anywhere in the program that can be set (`src/view/image_view/mod.rs:555-565`) |
| `tags` | 12 (`:124-173`) | Two unrelated subjects: the keyword catalogue and its panel, and the star, flag and label keys |
| `raw` | 6 (`:73-100`) | How a raw is decoded — **and** which files appear in the folder at all (`pair_with_jpeg`, `:80`) |
| `cull` | 6 (`:177-198`) | Destinations and the rejected folder. The one coherent section in the file |

110 leaf fields, plus `version` (`:25`), which is bookkeeping rather than a setting. Sixty of the 110
are `sc_*` shortcuts, and the keyboard editor reaches all sixty, drawn as 69 rows — 58 `binding!`
rows (`src/config/bindings.rs:85-418`) plus five colour labels and six star ratings generated at
`:420-436`. The slideshow window reaches three more (`src/app/panels.rs:100`, `:111`, `:125`). Those
two windows are the only places a change of the user's reaches the file: they are the only callers of
`App::save_settings` (`src/app/settings.rs:70`, `:94`). So the remainder is exact — **forty-seven
fields have no control anywhere in the running program**, six of which a key nudges for the session
and never writes back,
leaving forty-one that cannot be changed at all while the program runs. §1 counts the same gap from
the interface's side; §3 has the field-by-field inventory and what is stuck behind a restart.

### 4.2 Six ways the struct layout misleads

Each of these is a real question a photographer asks — §2 has where the questions come from — and
the section name is what stops them answering it.

| The question | Where the answer is | Why the section is wrong |
|---|---|---|
| *"How much memory is it using?"* | Six fields in `cache` (`:236-260`), three in `image_view` (`:359,362,366`), three in `grid_view` (`:449,452,455`) | Twelve fields decide one thing. `stores::image_store` and `stores::thumbnail_store` read all twelve and build the same `StoreConfig` from them (`src/app/stores.rs:32-66`). The section boundary cuts straight through a single subject |
| *"Why is `cache` not the cache?"* | `cache.previews_resident` and `cache.full_resolution_neighbours` are passed as literal `0` to the thumbnail store (`src/app/stores.rs:58-59`) | Two of the six fields under the heading "How much of the machine the viewer is allowed to use" (`:230-231`) apply to the image view only. A user tuning the contact sheet is reading four sentences that are not about it |
| *"How wide is the keyword panel?"* | `tags.panel_width` (`:144`), immediately above `tags.sc_toggle_tag_panel` (`:147`) | A panel width sits between a keyword catalogue and a keystroke, because all three happen to be about the same panel. It is a measurement of the screen and belongs with the other panel measurements — of which there are none, because the side panel's width is the literal `340.` (`src/app/chrome.rs:110`) |
| *"Where do I set the colour profile?"* | `general.output_icc_profile` (`:266`) | `general` is the bag. It holds the colour profile, the text scale, the metadata list, session restore, `sc_delete` (`:321`), `sc_filter` (`:330`) and six stack keys (`:304-318`). Microsoft's own rule is verbatim: *"Avoid generic tab labels that could apply to any tab, such as General, Advanced, or Settings"*, and separately *"Avoid General pages. You aren't required to have a General page"* (<https://learn.microsoft.com/en-us/windows/win32/uxguide/win-property-win>) |
| *"How do I show two pictures side by side?"* | `image_view.nr_images_shown` (`:368`), documented as "Images displayed side by side" (`README.md:654`) | The name is the count, not the thing. To find it you must already know the section, the noun and the abbreviation. The keyboard editor has the answer — it calls the same subject "More side by side" (`src/config/bindings.rs:331-336`) — and the configuration file does not |
| *"How do I change the grey behind the photograph?"* | You cannot, unless a slideshow is running: `slideshow.image_frame_background_color_override` (`:537`), applied only while `self.slideshow` is `Some` (`src/view/image_view/mod.rs:556-563`); otherwise the hardcoded `Color32::from_rgb(119, 119, 119)` (`src/view/image_view/layout.rs:12`) | The one settable ground colour in the program is filed under the one mode most people never use |

There is a seventh, and it is the plainest. `grid_view.filmstrip_height` (`:475`) and
`general.sc_filmstrip` (`:300`) are the same feature in two sections, and the thing they control is
drawn under the photograph in the image view (`src/app/views.rs:137-147`). No section name in the
file contains the word.

### 4.3 The program already does the regrouping, in one place

`src/config/bindings.rs` is a second view over the same fields: a flat list, grouped into four
sections that are **not** the struct sections, with names written for a person rather than for
`serde`. Its own module comment says so — "The keyboard map lives in the configuration as three
dozen separate fields, which is the right shape for reading it and the wrong shape for showing it
to somebody" (`src/config/bindings.rs:3-5`).

Eleven of its 69 rows already sit in a section that is not their struct:

| Rows | Struct | Editor section | Where |
|---|---|---|---|
| `sc_stacks`, `sc_toggle_stack`, `sc_standing_back`, `sc_standing_forward`, `sc_previous_stack`, `sc_next_stack` | `general` | Gallery | `src/config/bindings.rs:106-141` |
| `sc_move`, `sc_copy`, `sc_reject_folder`, `sc_undo` | `cull` | General | `src/config/bindings.rs:190-213` |
| `sc_toggle_tag_panel` | `tags` | General | `src/config/bindings.rs:148-153` |

It renames as well as regroups: `image_view.sc_overlay` is drawn as "What it says about itself"
(`src/config/bindings.rs:351`), `grid_view.sc_cycle_badges` as "What the cells say" (`:375`),
`general.sc_delete` as "To the bin" (`:180`). The mechanism that makes this safe is the accessor
pair — each row holds `Field::Fixed(fn(&Config) -> &Shortcut, fn(&mut Config) -> &mut Shortcut)`
(`src/config/bindings.rs:20-24`, built by the `binding!` macro at `:70-82`) — so the display name is
a label and the field is a function. Nothing on disk moves.

Two parts of the program besides the editor read those four section names: the cheat sheet filters
by them for the mode on screen (`src/ui/cheat_sheet.rs:28-36`, `:54-66`), and the clash checker
compares only within a section (`src/ui/keys.rs:171-186`). Both are why the section axis has to
survive the regrouping (§4.7).

**The proposal is that table, widened from 60 fields to 110.** The widened table is *the registry*,
the successor to `src/config/bindings.rs`; §12 says when it is built. The shape is not a new idea in
this codebase — it is the existing idea applied to the other 50 fields — but one part of it does not
widen unchanged, and both §4.6 and §4.7 rest on that part.

`Field::Fixed` holds `fn(&Config) -> &Shortcut` and `fn(&mut Config) -> &mut Shortcut`
(`src/config/bindings.rs:20-24`), and `Binding::get` and `Binding::set` hand back and take a
`Shortcut` (`:41-63`). That works because all sixty rows address one Rust type. The other fifty
address eighteen: `usize` (`src/config/mod.rs:141`), `u32` (`:366`), `u64` (`:260`), `u8` (`:99`),
`f32` (`:144`), `bool` (`:277`), `String` (`:188`), `Option<String>` (`:138`), five `Vec` element
types — `TagCategory` (`:128`), `Destination` (`:181`), `String` (`:270`), `UserAction` (`:385`),
`ContextMenuEntry` (`:387`) — and five enums: `Prefer` (`:80`), `RawSource` (`:85`), `RawQuality`
(`:88`), `Corner` (`:345`), `Motion` (`:533`). An `fn` pointer cannot be generic, so a variant per
*widget kind* will not hold them: one enum variant cannot carry both
`fn(&mut Config) -> &mut RawSource` and `fn(&mut Config) -> &mut Motion`, and one list variant cannot
carry `Vec<String>`, `Vec<Destination>`, `Vec<TagCategory>`, `Vec<UserAction>` and
`Vec<ContextMenuEntry>`. So there are two ways to widen it. Either `Field` grows a variant per
concrete type — eighteen of them, and a match arm per variant in every consumer: the renderer, the
search, the changed-from-default bullet, the per-field reset, the export and `Config::check()` — or
the type is erased at the row boundary: `fn(&Config) -> serde_json::Value` to read and
`fn(&mut Config, serde_json::Value) -> Result<(), _>` to write, with §12's `Kind` deciding only how
the value is drawn.

**The erasure is the one to build**, and the reason is not taste: three other things in this plan
already need exactly it. The build-time index test walks `serde_json::to_value(Config::default())`
(§3.7); the changed-from-default bullet is a row compared against that same walk (§3.4); and an
exported bundle is a map of JSON path to JSON value (§3.4). It also keeps the widened table a
`static` slice rather than the `Vec` `bindings::all()` allocates on every frame the editor or the
cheat sheet draws (`src/ui/keys.rs:61`, `src/ui/cheat_sheet.rs:48`), which eighteen typed variants
would not. `Kind` then stays what §12 makes it — a widget vocabulary — instead of being asked to
carry a type as well. The sixty shortcut rows keep the three variants they have: the editor captures
a keystroke rather than editing a value, and `Rating` and `Label` index into a `Vec<Shortcut>`
(`src/config/bindings.rs:44-45`, `:54-62`).

### 4.4 The rule: the file is storage, the page is a view

> **No JSON key is renamed. Every field in §4.5 keeps the name and path it has today, and the
> display name is a label held in the registry beside it. One key is proposed to move, once, and
> that move costs a migration step; nothing else moves.**

The no-rename half is not caution for its own sake. Three facts make a renamed key worse than no
rename:

1. **A key nothing recognises is discarded in silence.** No struct in `src/config/` carries
   `deny_unknown_fields`, so `serde_json::from_value` accepts a document still containing
   `image_view.nr_images_shown` after the field has been renamed, ignores it, and returns the
   default. An unrecognised key inside a section is not a parse failure, so it never reaches the arm
   that sets `partial` (`src/config/load.rs:175-182`): nothing is reported, nothing is logged, and
   the next `save()` writes the whole struct back (`:42-45`) with the user's value gone.
2. **A shared file must stay readable by an older build.** The migration machinery exists and
   reports what it moved (`src/config/migrate.rs:36-49`, shown to the user through
   `Config::migrated`, `src/config/mod.rs:43-48`), but it runs on the already-parsed `Config`
   (`src/config/migrate.rs:55`), which is after the point at which an unrecognised key has already
   been dropped.
3. **Nothing is gained.** The whole reason to rename is so a person can find the setting, and a
   person finds a setting on a page or through the window's search box (§3), not by reading JSON.
   The path stays as it is and becomes a search token in its own right, so `cache.ram_budget_mb`
   pasted from a forum post lands on the control.

**A key can be moved only one way, and it costs a step.** `Config::from_json` reads the document
into a `serde_json::Map` first and only then deserialises section by section
(`src/config/load.rs:133`, `:151-158`). A move is therefore a rewrite of that map before the typed
section is built — read the old path, write the new one, record it in `migrated`. That mechanism
does not exist today. It is needed exactly once, for `image_view.scroll_navigation`, and only if
§6's `mouse` section is built; that is the single exception the rule above names.

**Adding a section is free.** A section the document does not contain costs nothing: `section()`
returns `T::default()` when the key is absent (`src/config/load.rs:170-172`), and there is a test for
it (`a_missing_section_costs_nothing`, `:199-208`). So the three new sections §6 needs — `browsing`,
`group` and `mouse` — can be introduced without a version bump and without touching a single
existing file. That the cost is nought is not an argument for more of them: every other new field
goes into the section that already holds the subject it is about, which is §4.9's table and §12's
rule at the stage that writes them.

### 4.5 The eleven pages, and every field on them

Ordered by how often somebody wants them, not alphabetically. Nothing is called General, Advanced,
Miscellaneous or Other. Each page is reached three ways: the list down the left of the settings
window, `Ctrl+,` or `Settings ▸ All settings…` (§3), and a right-click on the thing the page is
about, whose last row is `More settings… (<page name>)` (§7). In every table below, **the second
column is the JSON key as it is today and as it stays** — with the one exception noted on page 11 —
and the first column is what the page says. The control each row gets is §5's.

#### Page 1 — Opening a folder

> *What turns up when I open a folder, in what order, and whether it remembers where I got to.*

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Raw and JPEG of the same frame | `raw.pair_with_jpeg` | `src/config/mod.rs:80` | Moves off the Raw files page. It decides which files are in the folder, not how one is decoded; being in `raw` is what made it invisible. `Prefer::label()` already reads "Show both / Show the JPEG / Show the raw" (`src/organize/pairs.rs:47-53`) and is called from nothing but a test (`:411-415`) |
| Read the sub-folders as one | `general.sc_flatten_dir` | `:293` | |
| Notice files appearing while it is open | `general.sc_watch_directory` | `:295` | |
| Show the filter bar | `general.sc_filter` | `:330` | |
| Show everything for a moment | `general.sc_suspend_filter` | `:334` | Editor name today: "Show everything" (`src/config/bindings.rs:222`) |
| Show the folder stacked | `general.sc_stacks` | `:304` | |
| Open or close a stack | `general.sc_toggle_stack` | `:307` | |
| Frame standing for a stack, back | `general.sc_standing_back` | `:310` | |
| Frame standing for a stack, forward | `general.sc_standing_forward` | `:312` | |
| Previous stack | `general.sc_previous_stack` | `:316` | |
| Next stack | `general.sc_next_stack` | `:318` | |
| Open where the last run left off | `general.restore_session` | `:277` | *Mirrored* on The window |

Plus the browsing and stacking defaults §6 argues for: `browsing.sort`, `browsing.descending`,
`browsing.flag`, `browsing.min_stars`, `browsing.max_stars`, `browsing.label`,
`browsing.stack_by_default`, `browsing.filter_follows_folder`, `group.max_gap`, `group.tolerance`,
`group.min_frames`. Plus the three startup fields, `general.start_in`, `general.start_fullscreen`
and `general.start_folder`: they describe what a launch starts with, which is what this page is for,
and their home is the page whose last row already says what a launch remembers. `restore_session`
sits beside them in the file as well as on the page (`src/config/mod.rs:277`), and the mode
indicator's menu — whose tick writes `general.start_in` — ends in `More settings… (Opening a
folder)`, which is this page (§7). The window keeps the two fields about which panels a launch
opens, and gets a line pointing here for the rest (§4.6).

The three `group.*` thresholds are here rather than on the contact sheet because they decide what
counts as one stack rather than how a sheet is drawn. Every key that shows a stacked folder, opens a
stack and steps through them is on this page (`src/config/mod.rs:304-318`), the setting that turns
stacking on at launch is `browsing.stack_by_default` two lines above, and the folded list the
thresholds produce is handed to the image view and the contact sheet alike
(`src/app/mod.rs:436-439`), so filing them under the sheet would name one of the two places they are
read.

#### Page 2 — The photograph

> *How a picture looks when it is the only thing on screen.*

The longest page; six groups, the first two open and the rest collapsed, which §3 defends.

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Enlarge a photograph smaller than the window | `image_view.enlarge_to_fit` | `src/config/mod.rs:381` | |
| Wait for the full decode before drawing | `image_view.should_wait` | `:370` | |
| Fit · Fill · Keep filling · Fit width · Fit height · Actual pixels | `image_view.sc_fit`, `sc_fit_maximize`, `sc_latch_fit_maximize`, `sc_fit_horizontal`, `sc_fit_vertical`, `sc_one_to_one` | `:390,409,411,405,407,400` | Editor names kept: Fit, Fill, Keep filling, Fit width, Fit height, Actual pixels |
| Zoom step · Zoom in · Zoom out · Repeat the last view | `image_view.sc_zoom`, `sc_zoom_in`, `sc_zoom_out`, `sc_repeat_place` | `:394,421,423,403` | |
| Pan up · down · left · right | `image_view.sc_pan_up`, `sc_pan_down`, `sc_pan_left`, `sc_pan_right` | `:427,429,431,433` | Read as held keys through a closure that looks only at `logical_key`, so modifiers are ignored on these four (`src/view/image_view/input.rs:158-161`); the page says so |
| Next image · Previous image | `image_view.sc_next`, `sc_prev` | `:396,398` | |
| Where the details are drawn | `image_view.overlay_corner` | `:345` | `Corner::ALL` and `label()` exist (`src/view/image_view/overlay.rs:35-41`, `:57-65`) and are called from nothing but the test module (`:148` onwards) |
| What they say | `image_view.overlay_format` | `:348` | |
| How big | `image_view.overlay_text_size` | `:350` | |
| Move it round the corners | `image_view.sc_overlay` | `:353` | |
| What the bottom bar says | `image_view.name_format` | `:383` | |
| Mark clipping and focus | `image_view.sc_marks` | `:356` | |
| Width of the white frame | `image_view.frame_size_relative_to_image` | `:372` | |
| Show the frame | `image_view.sc_frame` | `:392` | |
| How many at once | `image_view.nr_images_shown` | `:368` | The name the editor already uses for its keys is "side by side"; the page says "How many at once", with "side by side" as the unit on the control |
| One more · One fewer · Compare | `image_view.sc_more_images_shown`, `sc_less_images_shown`, `sc_compare` | `:413,415,419` | |
| Programs on the photograph's menu | `image_view.context_menu` | `:387` | Empty by default (`default_ctx_menu()`, `src/config/defaults.rs:165-167`), and `show_context_menu` returns before drawing anything when it is empty (`src/actions/user_action.rs:147-149`), so right-clicking the photograph does nothing out of the box (§7) |
| Longest edge of a photograph | `image_view.max_image_edge` | `:366` | *Mirror.* Home is Speed and memory |
| The ground during a slideshow | `slideshow.image_frame_background_color_override` | `:537` | *Mirror.* Home is Slideshow |
| The strip underneath | `grid_view.filmstrip_height` | `:475` | *Mirror.* Home is The contact sheet |
| The wheel moves through the folder | `image_view.scroll_navigation` | `:374` | *Mirror.* Home is Keys and mouse |

Plus, from §6: `image_view.zoom_step`, `zoom_step_factor`, `zoom_step_max`, `pan_speed` and `page`.
The wheel and drag behaviour those last two describe is §9's.

One clickable line, not a control: *"The grey behind every picture is on **The window**."* The field
is `general.backdrop`, and it is a window setting rather than a picture one because the same grey is
hardcoded twice — behind the photograph (`src/view/image_view/layout.rs:12`) and behind a
contact-sheet cell (`src/view/grid_view/mod.rs:32`) — with a third, darker one under the strip
(`src/view/grid_view/filmstrip.rs:31`). The slideshow row above overrides it, and only while a
slideshow is running (`src/view/image_view/mod.rs:555-565`).

#### Page 3 — The contact sheet

> *More or fewer thumbnails across, and what they tell me.*

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| How many fit across | `grid_view.images_per_row` | `src/config/mod.rs:439` | |
| Thumbnail detail | `grid_view.thumbnail_resolution` | `:452` | Home is here, deliberately adjacent to the row above, because the two are read against each other: the cell width comes from `images_per_row` (`src/view/grid_view/layout.rs:31-42`) and a thumbnail decoded below it goes soft. *Mirrored* on Speed and memory |
| More per row · Fewer per row | `grid_view.sc_more_per_row`, `sc_less_per_row` | `:462,464` | |
| Shape of a cell | `grid_view.cell_aspect` | `:447` | |
| Scroll down | `grid_view.sc_scroll` | `:460` | |
| Cycle what the cells say | `grid_view.sc_cycle_badges` | `:468` | Editor name: "What the cells say" |
| The caption | `grid_view.caption_format` | `:482` | Invisible unless the cells are showing names: gated on `self.badges.shows_name()` (`src/view/grid_view/mod.rs:482`), and `badges` resets to `Marks` every launch (`:84`). §6's stored default is what makes this field reachable at all |
| Pick out · Pick out everything | `grid_view.sc_select`, `sc_select_all` | `:484,486` | |
| How tall the strip is | `grid_view.filmstrip_height` | `:475` | Home is here, with the rest of the thumbnail settings and the store that draws it (`src/app/views.rs:132-136`). *Mirrored* on The photograph |
| Show or hide the strip | `general.sc_filmstrip` | `:300` | |
| Programs on a cell's menu | `grid_view.context_menu` | `:457` | The same type and the same `default_ctx_menu` as the image view's (`:387`), with a tick reading "use the same list as the photograph" |

Plus `grid_view.badges`, `grid_view.filmstrip_visible` and `grid_view.filmstrip_edge` from §6.

#### Page 4 — Stars, flags and labels

> *Which key rates, and whether it moves on afterwards.*

The page uses the photographer's word. `tags` today means two things, and this is the half that is
not about keywords.

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| No stars … Five stars | `tags.sc_rating` | `src/config/mod.rs:150` | Six rows, generated by `Field::Rating(i)` over `0..=MAX_RATING` (`src/config/bindings.rs:429-436`, `MAX_RATING = 5` at `src/metadata/xmp/mod.rs:68`) |
| Keep · Reject · No flag | `tags.sc_pick`, `sc_reject`, `sc_unflag` | `:153,156,159` | |
| Red · Yellow · Green · Blue · Purple | `tags.sc_label` | `:163` | Five rows, from `Label::CHOICES` (`src/metadata/xmp/mod.rs:118-124`) |
| Move on after marking | `tags.advance_after_marking` | `:169` | A setting, not a shortcut; `tags.sc_toggle_advance` (`:172`) is the key that flips it |
| Turn moving-on on and off | `tags.sc_toggle_advance` | `:172` | |

Plus `tags.sidecar_naming` from §6.

#### Page 5 — Keywords

> *My Lightroom keywords in here, and the ones I use to hand.*

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Keywords kept to hand | `tags.categories` | `src/config/mod.rs:128` | |
| A keyword list from another program | `tags.catalog_file` | `:138` | A relative path is taken against the configuration directory, not the working one (`src/annotations/catalog.rs:180-193`); the row states that |
| How many recent keywords to remember | `tags.recent_tags` | `:141` | |
| Width of the panel | `tags.panel_width` | `:144` | *Mirrored* on The window |
| Show the panel | `tags.sc_toggle_tag_panel` | `:147` | |

#### Page 6 — Moving and deleting

> *Where the rejects go and what Delete does.*

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Folders a photograph can be sent to | `cull.destinations` | `src/config/mod.rs:181` | Only the first nine are drawn (`take(9)`, `src/ui/destinations.rs:74`), and the panel that draws them only picks from the list — nothing in the program writes the field (`src/app/cull.rs:384` is the one read) |
| Name of the rejected folder | `cull.rejected_folder` | `:188` | |
| Move to… · Copy to… · To the rejected folder · Undo | `cull.sc_move`, `sc_copy`, `sc_reject_folder`, `sc_undo` | `:191,193,195,197` | |
| To the bin · Delete for good | `general.sc_delete`, `sc_delete_permanently` | `:321,324` | The two keys that destroy files are on the page about destroying files, not in `general` |

Plus `cull.confirm` from §6 — three booleans, deciding whether moving several photographs to the
bin, emptying the rejects and undoing a step that touched several files each ask first — and the
lifting of the nine-slot cap §6 argues for. It is added to `cull`, beside the folders a photograph
is sent to, rather than to a `marks` section of its own: what it guards is files being moved, which
is what `cull` and this page are both about.

#### Page 7 — Raw files

> *My raws look wrong, or they are slow.*

Every field here is hand-edited JSON today, not one of them can be reached from anywhere in the
running program, and all five are Rebuild-class once they can (§3).

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| What to show | `raw.source` | `src/config/mod.rs:85` | Per-variant sentences already written (`:106-110`) |
| How much work to spend demosaicing | `raw.quality` | `:88` | The enum carries one sentence (`:113`) and the three variants none (`:117-119`) |
| Use the camera's white balance | `raw.camera_white_balance` | `:92` | |
| Stretch the histogram | `raw.auto_brighten` | `:95` | |
| Blown highlights | `raw.highlight_mode` | `:99` | A `u8` acting as a four-way enum, named only in a doc comment (`:96-97`); §5 says what control that gets |

One clickable line, not a control: *"Which of a raw+JPEG pair is browsed is on **Opening a
folder**."* §4.6 explains why that is a link and not a mirror.

#### Page 8 — Slideshow

> *Hold each picture longer, and stop it drifting.*

The one page that exists already (`src/app/panels.rs:92-135`), carried across unchanged and given the
two fields it never showed.

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Hold each picture for | `slideshow.seconds_per_image` | `src/config/mod.rs:529` | Drawn today (`src/app/panels.rs:100`) |
| While it is up | `slideshow.motion` | `:533` | Drawn today with a sentence under each variant (`src/app/panels.rs:110-115`) — the best control in the program, and the only field of the five the README never mentions |
| Creep closer by | `slideshow.percent_zoom` | `:531` | Drawn today, only when the motion drifts (`src/app/panels.rs:119-130`) |
| Start with the frame on | `slideshow.start_with_frame_enabled` | `:535` | Not in the window today |
| The ground behind the picture | `slideshow.image_frame_background_color_override` | `:537` | Not in the window today. *Mirrored* on The photograph |

#### Page 9 — Speed and memory

> *It is using four gigabytes, and I want to know which number does that.*

Twelve numbers that decide one thing, currently spread across three sections, on one page. Eleven of
them live here; the twelfth, `grid_view.thumbnail_resolution`, is a mirror, for the reason given on
page 3.

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Decoded pictures in memory | `cache.ram_budget_mb` | `src/config/mod.rs:236` | Split between the two stores at `src/app/stores.rs:96-103` |
| Pictures on the graphics card | `cache.gpu_budget_mb` | `:254` | `src/app/stores.rs:88-93` |
| Decode threads | `cache.decode_threads` | `:239` | The one field of the 110 that cannot be made live or rebuilt, so the one row that keeps a restart notice (§3) |
| Time per frame spent moving pictures to the card | `cache.upload_budget_ms` | `:260` | |
| Pictures decoded either side | `image_view.nr_loaded_images` | `:359` | The struct says `image_view`; the field is a preload radius (`src/app/stores.rs:37`) |
| Rows of thumbnails read ahead | `grid_view.preloaded_rows` | `:449` | `src/app/stores.rs:54` |
| Camera thumbnails standing in | `cache.previews_resident` | `:244` | Passed as `0` to the thumbnail store (`src/app/stores.rs:58`), so this is an image-view setting that lives in `cache` |
| Full-size copies either side | `cache.full_resolution_neighbours` | `:246` | Same: `0` for thumbnails (`src/app/stores.rs:59`) |
| Photographs kept as textures | `image_view.gpu_resident_images` | `:362` | `src/app/stores.rs:35` |
| Thumbnails kept as textures | `grid_view.gpu_resident_thumbnails` | `:455` | `src/app/stores.rs:51` |
| Longest edge of a photograph | `image_view.max_image_edge` | `:366` | `src/app/stores.rs:38`. *Mirrored* on The photograph |
| Thumbnail detail | `grid_view.thumbnail_resolution` | `:452` | *Mirror.* Home is The contact sheet |

`grid_view.images_per_row` is not on this page even though the thumbnail preload radius is computed
from it (`src/app/stores.rs:54`). It is a layout choice with a cache consequence, not a cache
setting, and the consequence is stated on its own row instead.

#### Page 10 — The window

> *The text is too small, the grey is wrong, and which panels come up.*

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Text size | `general.text_scaling` | `src/config/mod.rs:268` | Read once and applied at startup (`src/app/mod.rs:150`) |
| Screen profile | `general.output_icc_profile` | `:266` | Only three substrings resolve (`src/metadata/icc.rs:11-15`) |
| What the side panel lists | `general.metadata_tags` | `:270` | Drawn at `src/app/panels.rs:151-163` |
| Width of the keyword panel | `tags.panel_width` | `:144` | *Mirror.* Home is Keywords |
| Show the side panel · Show the menu bar · Open the folder tree · Type a path | `general.sc_toggle_side_panel`, `sc_menu`, `sc_dir_tree`, `sc_navigator` | `:297,287,291,289` | |
| Fullscreen · Switch between picture and sheet · Next mode · Quit | `general.sc_fullscreen`, `sc_toggle_gallery`, `sc_next_mode`, `sc_exit` | `:327,280,283,285` | |
| Remember the window and where you were | `general.restore_session` | `:277` | *Mirror.* Home is Opening a folder |

Plus `general.panels_at_start`, `general.side_panel_width`, `general.theme` and `general.backdrop`
from §6. All four are added to `general` rather than to `startup` and `appearance` sections of their
own: the text scale, the screen profile and session restore are already in `general` and already on
this page, and an `appearance` section would split four settings that answer one question — the
text scale, the screen profile, the theme and the grey — across two places in the file. §12 rules
both sections out in the same terms, and §4.4's rule is why the choice cannot be left until the code is written —
adding a section is free and renaming one is not.

One clickable line, not a control: *"What the viewer starts with — the mode, fullscreen and a
folder — is on **Opening a folder**."* Those three are `general.start_in`,
`general.start_fullscreen` and `general.start_folder`, filed in `general` beside
`general.restore_session`, whose home page is Opening a folder and which is mirrored above.

#### Page 11 — Keys and mouse

| Reads as | JSON key | Where | Note |
|---|---|---|---|
| Programs on their own keys | `image_view.user_actions` | `src/config/mod.rs:385` | Each entry carries a `shortcut` (`:542`) — the only shortcut in the file the keyboard editor cannot reach, because `bindings::all()` has no row for it (§9) |
| The wheel moves through the folder | `image_view.scroll_navigation` | `:374` | Home is here. *Mirrored* on The photograph. **The one key whose path moves**: if §6's `mouse` section is built it becomes `mouse.wheel`, by the map rewrite of §4.4 and a migration step that says so |

Plus the `mouse` section §6 adds, whose behaviour — the wheel, the drag, the buttons, the double
click — is §9's; and the whole key map, which is §4.7.

#### The count

| Page | Existing fields placed |
|---|---|
| 1 Opening a folder | 12 |
| 2 The photograph | 31 |
| 3 The contact sheet | 13 |
| 4 Stars, flags and labels | 7 |
| 5 Keywords | 5 |
| 6 Moving and deleting | 8 |
| 7 Raw files | 5 |
| 8 Slideshow | 5 |
| 9 Speed and memory | 11 |
| 10 The window | 11 |
| 11 Keys and mouse | 2 |
| | **110** |

Mirrors are counted once, on their home page. Checked the other way round, the eight structs land
whole: `image_view` 31 + 3 + 2, `grid_view` 12 + 2, `general` 11 + 1 + 2 + 11, `cache` 6,
`slideshow` 5, `tags` 7 + 5, `raw` 5 + 1, `cull` 6. If `scroll_navigation` is later retired into
`mouse.wheel`, page 11 holds one existing field and the total is 109, which is the figure §3 uses
after the migration.

Plus `version` (`src/config/mod.rs:25`), which is read-only in the window's footer and is not on a
page. It is the one field a person should never change.

### 4.6 Settings that belong in two places

One setting may be reachable by several routes, and that is preferable to hiding it. Two mechanisms,
and they are not the same thing.

**A mirror is one field drawn twice.** The registry row carries a home page and a list of mirror
pages; both renderings call the same accessor pair, so whichever page is on screen reads and writes
the one field. That is the pattern `bindings.rs` already uses — `Field::Fixed(read, write)`
(`src/config/bindings.rs:20-24`) — and it is why there is no state to keep in step: a mirror is not
a copy of a value, it is a second call to the same function. Every mirror row is marked as one and
names its home, so the person who found it twice knows it is once.

Seven mirrors, and each exists because the field genuinely answers two different questions:

| Field | Home | Also on | Why |
|---|---|---|---|
| `general.restore_session` | Opening a folder | The window | It restores the folder and the photograph **and** the window's size and place — `Session` holds `window`, `folder` and `positions` in one struct (`src/session.rs:56-65`) |
| `image_view.max_image_edge` | Speed and memory | The photograph | "Why does zooming look soft" is a picture complaint with a decode-ceiling answer (`src/app/stores.rs:38`) |
| `grid_view.thumbnail_resolution` | The contact sheet | Speed and memory | It is a memory budget, but it is only meaningful read against the cell width, which is on the sheet |
| `tags.panel_width` | Keywords | The window | It is a panel measurement whichever struct it lives in, and the side panel's width is on The window |
| `slideshow.image_frame_background_color_override` | Slideshow | The photograph | Today it is the only settable ground colour in the program (`src/view/image_view/mod.rs:555-565`), and that is the question people arrive with |
| `grid_view.filmstrip_height` | The contact sheet | The photograph | The field is in `grid_view`; the strip is drawn under the photograph (`src/app/views.rs:137-147`) |
| `image_view.scroll_navigation` | Keys and mouse | The photograph | It is a mouse binding and it is also "how I move through a folder" |

**A cross-link is a row that is not a control.** One line, clickable, naming the page that owns the
field. It is used where a field belongs unambiguously to one subject but is looked for from another.
There are three, and only the first is about a field that exists today:

| The link | On | Points at | Fields |
|---|---|---|---|
| *"Which of a raw+JPEG pair is browsed is on **Opening a folder**."* | Raw files | Opening a folder | `raw.pair_with_jpeg` |
| *"What the viewer starts with — the mode, fullscreen and a folder — is on **Opening a folder**."* | The window | Opening a folder | `general.start_in`, `start_fullscreen`, `start_folder` |
| *"The grey behind every picture is on **The window**."* | The photograph | The window | `general.backdrop` |

Making any of the three a mirror would say the two pages disagree about what the field is, when in
fact one of them is simply where people look first. It is the settings window's case of the rule §8
states for every surface: a page that shows somebody the wrong thing ends in a route, not a dead
end.

The difference matters because a mirror costs a place on two pages for ever and a link costs a line.
Seven mirrors out of 110 fields is the ceiling, not a floor. The four new fields above take links
for the same arithmetic: a link is one line on the page somebody guessed wrong, where a mirror is a
row that has to be read, searched, marked as changed and reset on two pages for ever.

**And a third route that is not a page at all.** The window's search matches the JSON path as well as
the display name (§3), and the registry is keyed on the path and never on the label. nomacs saves
shortcuts under their *translated* names, so they break when the interface language changes
(<https://github.com/nomacs/nomacs/issues/1539>). This program has one language today; the rule costs
nothing to keep now and cannot be retrofitted later.

### 4.7 Where the shortcuts live

Sixty fields, 69 rows. Three arrangements are possible and the third is right.

**One page, as today.** The keyboard editor lists all 69 in four sections (`src/ui/keys.rs:78-107`).
What it is good at is the thing rebinding actually needs: seeing the whole map at once, so a
collision is visible before it is made. What it is bad at is that a key is severed from the thing it
does — `tags.advance_after_marking` is a checkbox with no home and `tags.sc_toggle_advance` is a row
in a list, and they are the same feature (`src/config/mod.rs:169-172`). Its other faults are §9's.

**Beside the setting only.** Every shortcut drawn on the subject page next to the thing it triggers.
This reads better on every individual page and it breaks two things that are not about reading.
`clash()` compares bindings only within an editor section, on the stated ground that "the gallery and
the image view are never on screen at once" (`src/ui/keys.rs:169-170`, the function at `:171-186`).
The cheat sheet picks its sections by mode (`src/ui/cheat_sheet.rs:28-36`). Both need the **mode** a
binding is filed under, and the subject pages do not carry it: page 1 alone holds `general.sc_stacks`,
filed as Gallery (`src/config/bindings.rs:106-111`), and `general.sc_filter`, filed as General.

**Both, from one row.** The registry row carries `page` — the subject — and `section` — the mode
scope — as two separate fields, and is drawn twice: on its subject page, next to the setting it
belongs to, and in the map on Keys and mouse, grouped by section as today. It is the mirror mechanism
of §4.6 applied to 69 rows instead of seven, and it works for the same reason: one accessor pair
behind both drawings, so there is no copy to keep in step.

What that buys, beyond finding things:

- The map keeps the mode axis, so `clash()` keeps working and the cheat sheet (§10) keeps working,
  both from the widened table, with `bindings::all()` reduced to a filtered view of it. Whether the
  section-scoped clash check is itself too narrow is §9's question; the axis has to exist either way.
- Every row on a subject page can name the mode it is filed under, in one grey word. That word is not
  decoration: it is what decides whether the binding appears on the cheat sheet for the mode on screen
  (`src/ui/cheat_sheet.rs:54-66`), and today nothing outside the cheat sheet says it.
- `image_view.user_actions[].shortcut` gains a row for the first time, because the widened table has
  to enumerate the whole file rather than a hand-maintained list of 58 macros (§9).

The one thing this does not do is duplicate the *editor*. There is one place to press a key and have
it captured — the row — and it is the same row whichever page draws it.

### 4.8 Setting, session state, or ephemeral

Two different failures are usually described as one. The first is a field that exists and a runtime
control that never writes to it: six of the forty-seven unreachable fields are nudged by a key or a
splitter for the session and thrown away on exit, which §3 lists and repairs. The second is state
with no field at all, which is §6's chapter. Both need the same question answered first, and it is
this chapter's to answer, because it is the question of where a value lives before it is a question
of which page draws it.

**Three kinds of memory, and every runtime value is exactly one of them.**

| Kind | Written to | Written by | It describes |
|---|---|---|---|
| **Setting** | `config.json` (`src/config/load.rs:16`) | The user, through the settings window | How this person works, on every folder and in every session |
| **Session state** | `session.json` (`src/session.rs:72`) | The viewer, on exit | Where they were: the window, the folder, the photograph, and the position in each of the last 64 folders (`src/session.rs:30`) |
| **Ephemeral** | nothing | — | This minute |

The program already draws the first two apart, and draws them correctly, in that file's own doc
comment: *"Not settings — those are the configuration, and the user writes them. This is where they
were"* (`src/session.rs:3-4`). What it does not have is the third, and the third is what stops the
other two swallowing everything a user touches.

The test, in order:

1. **Would a fresh install be wrong without it?** If the value is a habit — how somebody browses,
   how large they like their thumbnails, whether they want the panel up — it is a **setting**, and it
   goes on one of the eleven pages.
2. **Is it a place rather than a preference?** If restoring it tomorrow puts somebody back where they
   were and carries no opinion, it is **session state**. Window geometry and the per-folder position
   are the whole of this category today, and they work.
3. **Would restoring it start somebody in a state they did not ask for and cannot see the cause of?**
   Then it is **ephemeral**, and nothing writes it down. This is the category the other two steal
   from, and the reason to name it is that a stored value with no visible cause is worse than no
   value at all.

Applying it to the state the program currently keeps and throws away:

| What | Where it lives today | Kind |
|---|---|---|
| Sort order and direction | `Narrowing::default()` per session (`src/app/mod.rs:228`) | Setting, and a session memory of the last hand-set value |
| Flag, star and label rules | `Rules::default()` (`src/view/narrow.rs:36-48`) | Setting |
| Text rules — name fragment, extension list, keyword | The same `Rules` struct (`src/view/narrow.rs:29,31,33`) | **Ephemeral**: they are per-shoot, and a stored default opens every folder pre-filtered by something typed a month ago with nothing on screen saying why |
| Filter suspended | `Narrowing::suspended` (`src/view/narrow.rs:143`) | **Ephemeral**: its whole purpose is momentary — set the rules aside without forgetting them — and persisting it starts a session with the user's own filter switched off |
| Stacking thresholds — gap, tolerance, minimum run | `group::Settings::default()` (`src/organize/group/mod.rs:45-53`), in two independent copies (`src/app/stacking.rs:28`, `src/view/organize/mod.rs:56`) that disagree about their own ranges — a gap of 1–600 s against 1–3600 s, a tolerance of 0–32 against 0–64 (`src/ui/filter_bar.rs:134`, `:147`; `src/view/organize/group/mod.rs:60`, `:65`) | Setting, read by both |
| Whether the folder is stacked | `Stacking::default()` per session (`src/app/mod.rs:229`) | Setting for the default; **ephemeral** for which stacks are open now, which is about this folder and which `Stacking` already keeps across a re-detection and no further (`src/app/stacking.rs:33-35`) |
| Badge mode | `Badges::default()` per launch (`src/view/grid_view/mod.rs:84`), cycled at `:530-532` | Setting for the default, with the key left exactly as it is |
| Panel visibility — menu, side panel, keyword panel, filter bar, filmstrip | Four `false` literals in `App::new` (`src/app/mod.rs:201`, `:202`, `:212`, `:230`) plus the filmstrip, derived from a height (`:175`) | Setting: which panels a person works with is how they work. Not session state, because that would tie it to `restore_session`, which promises something else — a folder and a photograph (`src/session.rs:3-12`) |
| Panel widths | `tags.panel_width` (`src/config/mod.rs:144`); the side panel's is the literal `340.` (`src/app/chrome.rs:110`) | Setting: a width is a measurement of somebody's screen and their eyesight, identical in every session |
| The metrics panel — `F10`, hardcoded and not a binding (`src/app/input.rs:102`) | `metrics_visible: false` (`src/app/mod.rs:203`) | **Ephemeral**: a diagnostic, and a preference for it would be a preference for a debugging tool |
| Window geometry, open folder, per-folder position | `session.json` already (`src/session.rs:56-65`) | Session state, and it works |

§6 argues each of the settings above one at a time and cites this rule; §4.9 says which section each
lands in. Two things are ruled out here rather than argued: a second, per-folder configuration scope,
and any surface that reorders itself by how often a row is used. Both go to §13 with their reasons.

### 4.9 Where the new fields land

Fourteen of the thirty-five new fields the plan adds (§3 counts them all; §6 argues for them) are the
ones the three-way test above turns up: seven in a new `browsing` section, three in a new `group`
section, and four added to sections that already exist. The rest arrive with §6's own arguments, and
the table says where each of those lands too, so that no new field needs a section invented for it
later.

| Section | Fields | New section? |
|---|---|---|
| `browsing` | `sort`, `descending`, `flag`, `min_stars`, `max_stars`, `label`, `stack_by_default`, and `filter_follows_folder` from §6 | yes |
| `group` | `max_gap`, `tolerance`, `min_frames` | yes |
| `mouse` | §6's table, behaviour in §9 | yes |
| `grid_view` | `badges`, `filmstrip_visible`, and `filmstrip_edge` from §6 | no |
| `image_view` | `zoom_step`, `zoom_step_factor`, `zoom_step_max`, `pan_speed`, `page`, all §6's | no |
| `general` | `panels_at_start`, `side_panel_width`, and `start_in`, `start_fullscreen`, `start_folder`, `theme`, `backdrop` from §6 | no |
| `tags` | `sidecar_naming` from §6 | no |
| `cull` | `confirm` from §6 | no |

**Three new sections, and no fourth.** `startup`, `appearance` and `marks` are each a section
somebody could argue for, and none of them is added, because the rule is the one §4.4 states: a
field goes beside the fields it is about. A startup field joins `general` next to
`general.restore_session`, which is the same subject; a theme and a backdrop join `general` next to
the text scale and the screen profile, which are the other two settings about how the window looks;
a confirmation before files are moved joins `cull` next to the folders they are moved to. The three
that are added exist because there is nothing for them to join: nothing in the file describes how a
folder is browsed, what makes a burst one stack, or what the mouse does. §12 states the same ruling
at the stage that writes them.

The section is named for what the user does, not for the module behind it: `browsing`, not `narrow`
after `src/view/narrow.rs`, because the JSON path is a search token a person reads (§4.4). Whether
thirty-five more fields is thirty-five too many is §11's question, and the eleven pages are half of
its answer: the count a person meets is the count on one page.

A file written before any of them keeps working without a version bump, because a section the
document does not carry costs nothing (`src/config/load.rs:170-172`, with the test at `:199-208`) and
a field it does not carry falls back to its `#[serde(default = …)]`. Nothing already on disk is read
differently. That is the whole reason §4.4's rule is worth holding to: adding is free and moving is
not.

## 5. The right control for the value

The brief asks for a slider covering every value in preference to a dropdown of a few presets.
Fifty of the hundred and ten settings are not shortcuts (§3.1), and they divide into
twenty-four numbers, eight booleans, seven strings, five enums and six lists. Worked through
field by field, the brief's rule holds for eighteen of the twenty-four numbers; the other six
are whole counts nobody drags, and the interesting failures are elsewhere — in the fields that
look like numbers and are not quantities at all. What follows decides the control for all
fifty, and says for each what the range is, what the step is, what unit is printed beside it,
whether the rail is linear or logarithmic, and what moves on screen while the value is being
changed. Ranges are taken from §3.6, which owns them; the choice of control is decided here.

### 5.1 What the program has today

Nineteen numeric controls exist in 40,504 lines of Rust: seventeen `egui::DragValue` calls and
two `egui::Slider` calls. Two of the nineteen write a configuration field. The rest set a value
that lives for the session or for one job and is gone at exit, because eframe is built without
the persistence feature (`Cargo.toml:13`) and `App` implements no `save()`.

| Control | Where | What it sets | Kept? |
|---|---|---|---|
| `DragValue`, `1..=600`, suffix `" s"` | `src/app/panels.rs:100-103` | `slideshow.seconds_per_image` | yes |
| `DragValue`, `0.0..=200.0`, suffix `" %"` | `src/app/panels.rs:125-128` | `slideshow.percent_zoom` | yes |
| `radio_value` over `Motion::ALL`, `Motion::description()` indented under each variant | `src/app/panels.rs:110-117`, sentences at `src/config/mod.rs:514-523` | `slideshow.motion` | yes |
| A button per binding: click arms the row, the next key pressed becomes the shortcut | `src/ui/keys.rs:146-149`, armed at `:64-73` | the 60 `sc_*` fields of `src/config/mod.rs`, drawn as 69 rows | yes |
| `Slider`, `0..=32`, `show_value(false)` | `src/ui/filter_bar.rs:147` | stacking tolerance | no |
| `DragValue`, `0..=64` | `src/view/organize/group/mod.rs:65` | the same stacking tolerance, in the group-shots mode | no |
| `Slider`, `1.0..=1600.0`, `.logarithmic(true)`, `show_value(false)` | `src/view/image_view/bottom_bar.rs:262-266` | zoom | no |
| `DragValue`, `1.0..=600.0` / `1.0..=3600.0`, suffix `" s"` | `src/ui/filter_bar.rs:133`, `src/view/organize/group/mod.rs:59` | the grouping gap, twice, with two different ranges | no |
| Twelve more `DragValue`s | star ranges (`src/ui/filter_bar.rs:165`, `:171`, `src/view/organize/controls.rs:135`, `:137`), minimum frames (`src/view/organize/group/mod.rs:72`), rename counters (`src/view/organize/rename.rs:64`, `:67`, `:70`), the time shift (`src/view/organize/timeshift.rs:29`, `:31`, `:33`, `:35`) | one job's parameters | no |
| `ComboBox` of four fixed sizes | `src/view/organize/group/mod.rs:77-82`, values at `src/view/organize/thumbnails.rs:21-26` | thumbnail height in the group panel: Names only · 48 · 96 · 180 pt | no |

The important number is the last column. Three widgets and a grid of sixty-nine buttons write
the configuration; everything else the program draws for a number is a control whose value
cannot be kept. The judgement about widgets is mostly sound — a `DragValue` for seconds, a
logarithmic rail for zoom, a radio group with a sentence per variant for the slideshow motion.
What is wrong is where they point.

Three details are worth taking from that table.

The two `Slider`s both switch off the number egui offers beside the rail, and in the zoom case a
read-only `Label` was put in its place (`src/view/image_view/bottom_bar.rs:278-281`) — 45 points
wide, printing `{:.1}%`, sensing clicks only so that it can carry a context menu. So the one
place in the viewer where a slider and its number sit side by side is the one place the number
cannot be typed into. The five percentages a person would actually want — 200, 100, 75, 50, 25 —
exist as `PERCENTAGES` (`:12`) and are offered on that right-click menu (`:298-301`), which is
the only context menu the program draws out of the box and is specified in full in §7.4. The
stops exist. They are simply not on the rail.

The second is that the same field gets two different controls and two different ranges depending
on which panel is drawing it. `group::Settings::tolerance` (`src/organize/group/mod.rs:40`) is a
slider capped at 32 in the filter bar and a drag value capped at 64 in the group panel;
`max_gap` (`:37`) is a drag value stopping at 600 seconds in one and 3600 in the other. Nothing
enforces agreement because each call site writes its own range inline. That is the argument for
a widget vocabulary held in one place (§12) rather than a habit.

The third is the one control in the program the brief's rule condemns outright: the
thumbnail-size `ComboBox` in the group panel, four fixed heights with nothing between 96 and 180
and nothing above 180, for a value that is continuous, cheap to change and visible the instant
it changes.

Everything else is JSON. Sixty of the 110 settings are shortcuts and the keyboard editor reaches
all of them; the slideshow window reaches three more; **forty-seven have no control anywhere**,
and every performance number, every raw option, every format string and every enum but `Motion`
is in that set (§3.1).

### 5.2 The rule, and the five tests that decide each row

The brief's rule is right as a default and needs three qualifications, all of them published.

- **Prefer a continuous control over presets.** WinUI states the exception rather than the rule:
  "Don't create a continuous slider if the range of values is large and users will most likely
  select one of several representative values from the range. Instead, use those values as the
  only steps allowed."
  (<https://learn.microsoft.com/en-us/windows/apps/design/controls/slider>)
- **A slider is for a relative quantity, not an exact one.** Microsoft's decision list is the
  clearest anywhere: "Does the setting seem like a relative quantity? If not, use radio
  buttons… Is the setting an exact, known numeric value? If so, use a numeric text box… Does
  the setting have a range of four or more values? If not, use radio buttons."
  (<https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-sliders>) NN/g agrees from the
  other end: sliders "work best when the specific value does not matter to the user, but an
  approximate value is good enough" (<https://www.nngroup.com/articles/gui-slider-controls/>).
- **Where both are wanted, use both.** Microsoft: "Don't use both a slider and a numeric text
  box for the same setting… **Exception:** Use both controls when the user needs both immediate
  feedback and the ability to set an exact numeric value." NN/g's phrasing of the same pairing
  is "separate, linked controls for coarse and fine adjustment… Both display the same value and
  adjusting one immediately changes the other accordingly"
  (<https://www.nngroup.com/articles/sliders-knobs/>). GNOME allows the pair only when
  "immediate feedback for changes in the spin box's value is possible"
  (<https://developer.gnome.org/hig/patterns/controls/spin-buttons.html>), which is a real
  constraint here and is answered in §5.9.

That gives five tests, applied in order to every field below.

1. **Is it an ordered quantity?** No → radio group with a sentence per variant.
2. **Fewer than four meaningful values?** Yes → radio group. (Microsoft, above.)
3. **Is the exact number reproducible on another machine?** Yes → the rail must carry a typed
   box, and the box is the truth.
4. **Do the values people choose cluster at one end?** Yes → logarithmic. Microsoft states the
   rule directly — "Consider using a non-linear scale if the range of values is large and users
   will likely select values at one end of the range" — and Baymard measured the cost of
   ignoring it: "50% of the slider width is used to control just 2% of the products", and "83%
   of top sites with sliders wrongly use linear scales"
   (<https://baymard.com/blog/slider-interfaces>). That sample is e-commerce price filters;
   what transfers is the arithmetic of a skewed range, not the finding.
5. **Is it a whole count with a value almost everybody keeps?** Yes → a stepper. NN/g: use one
   when "the numeric field has a most commonly selected value and most users will not deviate
   insignificantly from it" (<https://www.nngroup.com/articles/input-steppers/>). There is
   nothing to explore along the rail — nobody drags a preload radius to see what happens. A
   stepper is not a preset list; it is the continuous control at a scale where continuity has
   stopped meaning anything. egui's nearest thing is `DragValue`, which is a drag-and-type
   field with no increment buttons of its own; where the buttons earn their place they are two
   small `Button`s either side of it.

### 5.3 What egui gives free, and the one default that must be turned off

Three facts about egui 0.33 (`Cargo.toml:13`) decide how much of this costs anything.

**The pair Microsoft carves out is one call.** `Slider`'s value display is itself a `DragValue`
(`egui-0.33.0/src/widgets/slider.rs:918-943`), constructed with the slider's own `suffix`,
`prefix`, `custom_formatter` and `custom_parser`. It can be clicked and typed into. So
`Slider::new(…).show_value(true)` *is* the linked coarse-and-fine pair, with the unit already
on it, and both of the program's existing sliders switch it off. The rail also has
`.logarithmic(bool)` (`:225`), `.smallest_positive(f64)` for a logarithmic range that includes
zero (`:234`), `.step_by(f64)` (`:316`), `.text()` for a label on the same line (`:195`), and
`.custom_formatter`/`.custom_parser` (`:435`, `:479`) — which is how a stored multiplier of
1.25 is shown and typed as 125 %.

**There are no tick marks.** `grep -i tick` over `egui-0.33.0/src/widgets/slider.rs` returns
nothing. Every source asks for them — Microsoft: "Use tick marks and a value label when users
need to know the exact value of the setting they choose"; WinUI: "if there are only 10 snap
points, show tick marks"; GNOME: "it is helpful to mark significant values along the length of
the slider with text or tick marks"
(<https://developer.gnome.org/hig/patterns/controls/sliders.html>). Two things stand in, and
the second is better than a tick mark anyway:

- A thin strip painted under the rail from the `Response`'s own rect — twenty lines, written
  once, used by every slider in the window.
- Where there are six or fewer significant values, a row of small text buttons under the rail
  that *set* the value: `100%  125%  150%  175%  200%`. A tick you cannot click is decoration;
  a named stop you can click is the preset list the brief is suspicious of, restored to the
  rail where it costs nothing and hides nothing. This is the shape Photoshop uses for its two
  cache numbers — three named workload buttons, and both numbers still on screen
  (<https://photoshopguides.github.io/Performance>) — and it is why §5.7 refuses named memory
  levels while keeping named memory buttons.

**And the default that must be turned off in every row.** `SliderClamping::Always` is egui's
default (`slider.rs:73-75`) and it clamps *existing* values, not only edited ones;
`DragValue::clamp_existing_to_range` defaults to `true` the same way (`drag_value.rs:75`).

The consequence is already live in the program, and it is worse than it looks. `old_value` is
read before anything else happens (`drag_value.rs:464`); the clamp is applied to the borrowed
value (`:514-516`); the clamped value is written straight back through the reference
(`:523-526`); and the response is then marked changed because the value moved (`:666-668`). The
slideshow window declares `.range(1..=600)` (`src/app/panels.rs:101`) and hands `changed`
straight to `save_settings()` (`src/app/settings.rs:70`). So a hand-written
`"seconds_per_image": 900` is not merely clamped in memory: it is written back to disk as 600
on the frame the Slideshow window is opened, with nobody touching anything.
`slideshow.percent_zoom` has the same shape at `:126`.

Every control in a settings window therefore passes `SliderClamping::Edits` (`slider.rs:286`) —
"Users cannot enter new values that are outside the range. Existing values remain intact
though" (`:68-71`) — or `clamp_existing_to_range(false)` on a bare `DragValue`
(`drag_value.rs:154`). This is not a detail. `grep clamp src/config/` finds nothing, and §3.6
rules that it must stay that way and that the bound belongs at the consumer, so the widget's
own clamp is today the only thing standing between a hand-edited file and the value it holds.
Turning it off is the whole of "the file may be more permissive than the window" reduced to one
argument per widget, and without it a settings page is a program that quietly edits the file it
was opened to read.

### 5.4 Units, defaults, snapping, and showing the value before it is committed

**Units go on screen and never in a tooltip.** Microsoft: "Always use a value label if a user
needs to know the units to make sense of the setting… include the units (such as pixels)."
NN/g is explicit about the negative case: "Instructions or other directly actionable
information, like field requirements, shouldn't be in a tooltip"
(<https://www.nngroup.com/articles/tooltip-guidelines/>). The unit, the range and any restart
consequence are field requirements, so they are drawn under the control and not hovered for;
that is the one rule this chapter takes from the hover policy, which is §10's. egui puts the
unit inside the value box through `.suffix()`, which the slideshow window already does
(`src/app/panels.rs:102`, `:127`).

There is direct evidence here for why a unit cannot live in a comment. Two doc comments that
carry the arithmetic have drifted off the fields they describe. "Half a frame at sixty a
second, which leaves the rest for drawing" sits at `src/config/defaults.rs:35`, at the top of
the block above `default_gpu_budget_mb` (`:38`), while `default_upload_budget_ms` (`:41`) — the
field it describes — has no comment at all. And a sentence about `PageDown` and the space bar
sits at `src/config/defaults.rs:293-297`, in the block above `default_filmstrip_height` (`:299`),
with the filmstrip's own sentence tacked on at `:298`. Nobody noticed, because nobody reads
them. Put the same sentence under the control and it is read every time the page is opened.

**The default is marked in the gutter, not in the label**, by the changed-from-default bullet
that §3.4 specifies; every row below inherits it, so there is no column for it here.

**Snapping is `.step_by`, and it is used sparingly.** WinUI: "Always show tick marks when step
points aren't obvious… if the slider is 200 pixels wide and has 200 snap points, you can hide
the tick marks because users won't notice the snapping behavior." So `gpu_resident_thumbnails`
snaps to 8 and `max_image_edge` to 128 silently, while `text_scaling` snaps to nothing and
offers its five named stops as buttons.

**Six fields must be shown before they are committed**, on four rows — the three template
fields take one preview between them. WinUI states the first case verbatim: "A slider setting
text size could render some sample text of the right size beside the slider."

| Value | What is drawn beside the control | Where the drawing code already is |
|---|---|---|
| `general.text_scaling` | A line of sample text at the chosen size, and — because the setting is applied by multiplying every text style rather than by setting `pixels_per_point` — the whole window once the value is committed | `src/app/mod.rs:888-894` |
| `grid_view.thumbnail_resolution` | The cell width the current column count produces, in points and in pixels, and whether the chosen resolution is below it | `Layout::new` computes the cell width at `src/view/grid_view/layout.rs:38-42`; pixels are that times `pixels_per_point` |
| `image_view.overlay_format`, `name_format`, `grid_view.caption_format` | The rendered line, from the photograph currently open, plus the vocabulary table as clickable chips | `PLACEHOLDERS` at `src/metadata/template.rs:321-339`, drawn as a hover table at `src/view/organize/rename.rs:145-154` and an insert menu at `:44-50` — the only place in the program that does either |
| `image_view.overlay_text_size` | The sample overlay line at that size, over the photograph if the overlay is up | `src/view/image_view/overlay.rs:107` |

The template case is the sharpest. At the top level of a template an unrecognised placeholder
returns `None` (`src/metadata/template.rs:287`) and is then expanded to the empty string by
`unwrap_or_default()` (`:117`), so `{nmae}` disappears without a word. And `caption_format` is
additionally invisible unless the badge mode has been cycled to `Full`
(`src/view/grid_view/mod.rs:482`, `src/view/grid_view/cell.rs:56-58`), which is not the
default — `Badges::default()` is `Marks` (`cell.rs:27`) — and is rebuilt at every launch
(`src/view/grid_view/mod.rs:84`). A preview is not a nicety for these three; it is the only way
to tell a correct template from a typo, and it is the row §3.6 relies on when it reports a bad
placeholder.

### 5.5 Every numeric field

Twenty-four numbers, plus `version` (`src/config/mod.rs:25`), which is not a setting and is
drawn read-only. "Now" is the control that exists today; the ranges are §3.6's.

| Field | Where | Now | Proposed | Range | Step | Unit shown | Scale | Live feedback |
|---|---|---|---|---|---|---|---|---|
| `cache.ram_budget_mb` | `src/config/mod.rs:236`, default 4096 at `defaults.rs:11` | none — JSON | Rail + typed box, four filling buttons above | 256–65536 | 128 | `MB` | **log** | The split it makes, and what is held now, from `panels::cache_stats` (`src/app/panels.rs:168-196`, `:206`) |
| `cache.gpu_budget_mb` | `:254`, default 1024 at `defaults.rs:38` | none — JSON | Rail + box | 128–16384 | 64 | `MB` | **log** | The split — 896 for photographs, 128 for thumbnails at the default (`src/app/stores.rs:88-93`) — and the on-GPU counts at `src/app/panels.rs:176-180` |
| `cache.decode_threads` | `:239`, default 0 at `defaults.rs:16` | none — JSON | Tick "Choose for me (8 workers)" over a rail + box | 1–max(16, cores) | 1 | `decode workers` | linear | The measured sentence of §5.7, and a mark at 8 |
| `cache.upload_budget_ms` | `:260`, default 8 at `defaults.rs:41` | none — JSON | Rail + box, behind the group's disclosure | 1–16 | 1 | `ms a frame` | linear | The frame line `PerfMetrics::display_metrics` already draws (`src/ui/perf_metrics.rs:74-82`) |
| `cache.previews_resident` | `:244`, default 16 at `defaults.rs:23` | none — JSON | Rail + box | 0–256 | 1 | `camera thumbnails`, 0 reads "Off" | linear | `StoreStats.preview_bytes` (`src/cache/mod.rs:114`), which is measured rather than derived |
| `cache.full_resolution_neighbours` | `:246`, default 1 at `defaults.rs:31` | none — JSON | Stepper | 0–8 | 1 | `either side` | linear | "≈ 96 MB each" — 24 megapixels at four bytes, which is this plan's arithmetic, not the source's: the doc comment at `defaults.rs:90-94` does it for the *reduced* copy (eleven megabytes) and not for this one |
| `image_view.nr_loaded_images` | `:359`, default 512 at `defaults.rs:95` | none — JSON | Rail + box | 0–4096 | 1 | `photographs either side` | **log** | What the budget actually allows — `resident_bytes` over `in_ram` (`src/cache/mod.rs:99`, `:104`), around 326 at a 4 GB budget and eleven megabytes a copy — the image store gets 3,584 MB of that budget and the contact sheet the other 512, since `preload_radius` is this field (`src/app/stores.rs:37`) and the budget beside it is `split(cache.ram_budget_mb).0` (`:34`, `:96-103`). The doc comment at `defaults.rs:90-94` says the trimming happens and nothing on screen says what it trims to |
| `image_view.gpu_resident_images` | `:362`, default 8 at `defaults.rs:98` | none — JSON | Stepper | 1–64 | 1 | `textures` | linear | Derived megabytes |
| `image_view.max_image_edge` | `:366`, default 0 at `defaults.rs:102` | none — JSON | Tick "As large as the card allows" over a rail + box | 512–32768 | 128 | `px, longest edge` | **log** | The adapter's own ceiling, which already wins over the configured one (`src/cache/store/mod.rs:164-167`, from `gpu.max_texture_edge()` at `src/cache/gpu.rs:129-132`) |
| `image_view.nr_images_shown` | `:368`, default 1 at `defaults.rs:242` | `Ctrl +` / `Ctrl −`, session only (`src/view/image_view/mod.rs:282-290`) | Rail, eight stops, no box | 1–8 | 1 | `side by side` | linear | The view re-tiles |
| `image_view.overlay_text_size` | `:350`, default 15 at `defaults.rs:148` | none — JSON | Rail + box | 8–48 | 0.5 | `pt` | linear | The sample line at that size; the overlay itself if it is up |
| `image_view.frame_size_relative_to_image` | `:372`, default 0.2 at `defaults.rs:133` | none — JSON | Rail + box, in per cent | 0–25 | 0.5 | `% of the shorter edge` | linear | Live on the photograph |
| `grid_view.images_per_row` | `:439`, default 5 at `defaults.rs:281` | `+` / `−`, session only (`src/view/grid_view/mod.rs:524-527`) | Rail + box, a stop at every value | 1–16 | 1 | `across` | linear | "cells are 480 pt wide here", from `Layout::new` (`src/view/grid_view/layout.rs:38-42`) |
| `grid_view.cell_aspect` | `:447`, default 1.5 at `defaults.rs:368` | none — JSON | Four named buttons, then a rail | 0.5–3.0 | 0.01 | read as `1 : 1.50` | linear | Live |
| `grid_view.preloaded_rows` | `:449`, default 1 at `defaults.rs:284` | none — JSON | Stepper | 0–20 | 1 | `rows either side` | linear | The radius it actually produces: `(rows + 8) × across` (`src/app/stores.rs:54`), so 1 row at 5 across is 45 thumbnails, not 5 |
| `grid_view.thumbnail_resolution` | `:452`, default 512 at `defaults.rs:287` | none — JSON | Tick "Match this display (2560 px)" over a rail + box | 128–4096 | 32 | `px, longest edge` | linear, argued in §5.7 | The cell width in pixels, and whether this is below it |
| `grid_view.gpu_resident_thumbnails` | `:455`, default 256 at `defaults.rs:290` | none — JSON | Stepper | 8–2048 | 8 | `textures` | linear | Derived megabytes |
| `grid_view.filmstrip_height` | `:475`, default 0.0 at `defaults.rs:299` | none — JSON; `Ctrl+T` toggles a flag that the default height makes inert (`src/app/mod.rs:713`, `src/app/views.rs:137-141`) | Tick "Show the strip" over a rail + box | 60–400 | 2 | `pt` | linear | Live, and the strip's own top edge drags the same field |
| `tags.panel_width` | `:144`, default 260 at `defaults.rs:351` | the splitter, which persists nothing | Rail + box | 180–800 | 4 | `pt` | linear | Live; the splitter writes the same field. 180 is the floor the panel already enforces (`src/ui/tag_panel/mod.rs:65`) |
| `tags.recent_tags` | `:141`, default 12 at `defaults.rs:347` | none — JSON | Stepper | 0–100 | 1 | `keywords remembered`, 0 reads "Off" | linear | — |
| `general.text_scaling` | `:268`, default 1.25 at `defaults.rs:49` | none — JSON | Rail + box **in per cent**, through `custom_formatter`/`custom_parser`, five named stops under it | 50–300 | 1 | `%` | linear | Sample text at the size; the window itself on commit |
| `slideshow.seconds_per_image` | `:529`, default 15 at `defaults.rs:501` | `DragValue` (`src/app/panels.rs:100`) | The existing box, with a rail above it | 1–600 | 1 | `s` | **log** | Live |
| `slideshow.percent_zoom` | `:531`, default 25 at `defaults.rs:505` | `DragValue` (`src/app/panels.rs:125`) | Rail + the existing box | 0–200 | 1 | `% larger by the end` | linear | Live while a slideshow runs |
| `raw.highlight_mode` (the passes) | `:99`, default 0 at `defaults.rs:496` | none — JSON | Stepper inside the fourth radio row — see §5.7 | 3–9 | 1 | `passes` | linear | The photograph, once the store is rebuilt |

Eighteen rails, six steppers. `tags.recent_tags` reads 0 as off, which the field cannot do today
because `.max(1)` is applied at use (§3.6); the control is drawn that way once the floor moves.

**Five rails are logarithmic**, and each for the same reason: the values people pick cluster at
one end. RAM runs from a quarter of a gigabyte to sixty-four and nobody chooses forty-one. VRAM
spans a factor of 128, which is on its own enough. Preload runs 0 to 4096 and the interesting
part is 0–64. Slideshow seconds run 1 to 600 and the interesting part is three to twenty. The
decode ceiling runs 512 to 32768 and the interesting part is the band around this monitor's own
longest edge. `smallest_positive` (`slider.rs:234`) is what keeps the two ranges that include
zero usable.

**The unit column is not decoration.** Three of these numbers are not what they appear:
`ram_budget_mb` is split between two stores by `split` (`src/app/stores.rs:96-103`) with a floor
of 64 MB on the thumbnail half (`:19`), so at 256 MB the contact sheet gets a quarter rather
than the usual eighth; `preloaded_rows` is multiplied by eight assumed visible rows and by the
column count before it reaches the store (`:54`); and `nr_loaded_images` is a ceiling the budget
trims, which its own doc comment says and no screen does. In each case the number typed and the
number that governs are different, and the second belongs under the first.

### 5.6 Every enum, every boolean, and the rest of the fifty

Eight fields choose among named alternatives, and they are not eight further settings on top of
the twenty-four numbers. Five are Rust enums; one is `raw.highlight_mode`, a `u8` behaving as an
enum and already counted among the numbers; one is `general.output_icc_profile`, a `String` with
three legal values and one of the seven strings; and one is `Callback`, a nested type that
appears inside three of the six list fields rather than being a setting of its own. All eight
become radio groups with a sentence per variant, because none has more than five values and none
of the values is a quantity — Microsoft's fourth test. `Motion` already shows how it looks
(`src/app/panels.rs:110-117`).

| Field | Where | Now | Proposed | Variants | Where the labels already are |
|---|---|---|---|---|---|
| `raw.pair_with_jpeg` | `src/config/mod.rs:80` | none — JSON | Radio, sentences | 3 | `Prefer::ALL` and `label()` at `src/organize/pairs.rs:45-53`, called nowhere outside its own tests |
| `raw.source` | `:85` | none — JSON | Radio, sentences | 2 | Per-variant doc comments at `src/config/mod.rs:106-110` |
| `raw.quality` | `:88` | none — JSON | Radio, sentences with the cost | 3 | `src/decoder/raw/mod.rs:23-31` — see §5.7 |
| `raw.highlight_mode` | `:99` | none — JSON | Radio of 4, with a passes stepper in the fourth row | 4 (+ passes) | A doc comment at `src/config/mod.rs:96-97` and a README line (`README.md:684`) — see §5.7 |
| `image_view.overlay_corner` | `:345` | `o` cycles it, on the view's own clone of the configuration, which is never written back (`src/view/image_view/mod.rs:279`) | A 3 × 3 arrangement of radio dots shaped like the corners, Off in the middle | 5 | `Corner::ALL` (`src/view/image_view/overlay.rs:35-41`) and `label()` (`:57`), called nowhere outside its own tests |
| `slideshow.motion` | `:533` | radio (`src/app/panels.rs:111`) | Unchanged | 3 | `Motion::label()` and `description()` at `src/config/mod.rs:506-523` |
| `general.output_icc_profile` | `:266` | none — JSON, a free string | Radio of 3, and no "Other…" | 3 | `BUILT_IN` at `src/metadata/icc.rs:11-15`, matched by `contains` at `:26` |
| `Callback`, in the three lists that carry one (`image_view.user_actions` `:385`, `image_view.context_menu` `:387`, `grid_view.context_menu` `:457`) | struct fields at `:544`, `:551` | none — JSON, a free string | Named radio | 5 | `src/actions/callback.rs:39-46` |

Two of those deserve a note. `output_icc_profile` is a free text box today whose three legal
values are matched by substring, and every other value skips colour conversion with a message
that reaches the log alone (`src/decoder/color.rs:30-32`); there is no code path that loads a
profile from a file, so the radio has no "Other…" row to offer and §3.6 says the same. `Callback`
is worse: an unrecognised string deserialises to `NoAction` (`src/actions/callback.rs:44-45`), so
`"Relaod"` is accepted and silently does nothing for ever. Neither is a case where a free control
gives more freedom. Both are cases where a free control gives a trap with no feedback, and the
radio group is the narrower control that is also the more capable one.

**The eight booleans** are checkboxes — a control the program currently uses in exactly one
file (`src/view/organize/timeshift.rs:71`) — with the label naming the setting rather than the
instruction and the doc comment's own sentence beneath it.

| Field | Where | Default | Note |
|---|---|---|---|
| `raw.camera_white_balance` | `src/config/mod.rs:92` | true | "Without it colours come out noticeably wrong" is already written at `:89-90` and belongs under the box |
| `raw.auto_brighten` | `:95` | true | — |
| `tags.advance_after_marking` | `:169` | false | The doc comment at `:164-167` says holding shift advances once regardless. Nothing implements that — `advances()` reads the mode flag alone (`src/app/input.rs:198-200`) — and the comment beside that function (`:192-197`) says the modifier was rejected deliberately, because on Slovak and German layouts the digits are shifted and every rating would advance. Two comments in the same repository contradict each other; the wrong one has to go before either is put on screen |
| `general.restore_session` | `:277` | true | Lists what is restored, from the doc comment at `:271-275` |
| `image_view.should_wait` | `:370` | true | — |
| `image_view.enlarge_to_fit` | `:381` | true | The DNG postage-stamp sentence at `:375-379` is the best justification of a default in the file |
| `slideshow.start_with_frame_enabled` | `:535` | false | Not in the slideshow window today (`src/app/panels.rs:85-138`) |
| `image_view.scroll_navigation` | `:374` | true | **Not really a boolean** — see below |

`scroll_navigation` is a two-item action list with one item missing. With it on, the wheel over
the photograph steps to the next or previous frame (`src/view/image_view/input.rs:192-202`,
called at `src/view/image_view/interaction.rs:18-22`); with it off, the wheel pans a picture
larger than the window and has nothing to do on one that fits
(`src/view/image_view/interaction.rs:40-45`). Turning a setting off should choose a different
behaviour, not remove one. So the control is not a checkbox but a radio group of four — next and
previous · zoom · pan · nothing — which the migration fills from the boolean. What the wheel
should then do is §9's, and the field it becomes is §6's; the point here is only that a boolean
is the wrong control for a value with four states.

**The rest of the fifty** are the six remaining strings — `general.output_icc_profile` having
been drawn above as a radio — and the six lists, and none of them is a free text box.

| Field | Where | Control |
|---|---|---|
| `tags.catalog_file` | `src/config/mod.rs:138` | A path row — a box, a **Browse…** button and a **Read it now** check, because a relative path here resolves against the configuration directory and not the working one (`src/annotations/catalog.rs:53-62`) |
| `cull.rejected_folder` | `:188` | A text box that refuses empty; blank makes the reject-to-folder key a silent no-op (§3.6) |
| `slideshow.image_frame_background_color_override` | `:537` | A swatch that opens egui's own picker (`Ui::color_edit_button_srgba`, `egui-0.33.0/src/ui.rs:2336`) beside a validated hex box. Today a hex that fails to parse is swallowed by `.ok()` (`src/view/image_view/mod.rs:558-564`) |
| `image_view.overlay_format`, `name_format`, `grid_view.caption_format` — three fields, one control | `:348`, `:383`, `:482` | A text box with the `PLACEHOLDERS` chips and a live rendering, per §5.4 |
| `tags.categories`, `cull.destinations`, `general.metadata_tags`, `image_view.user_actions`, and the two `context_menu` lists | `:128`, `:181`, `:270`, `:385`, `:387`, `:457` | A row editor — add, remove, reorder, and one control per member chosen by the same five tests, so a `Callback` member is a radio and a path member is a path row. Not a JSON array in a box. What each list's rows say is §4.5 and §3.6 |

That closes the fifty, and the arithmetic is meant to be checked against the tables: the
twenty-four numbers of §5.5, then the five enums and `general.output_icc_profile` as radio
groups, the eight booleans as checkboxes, the six remaining strings, and the six lists.

### 5.7 Seven arguments made properly

#### The memory budget wants a logarithmic rail and a typed box, and no named levels

`cache.ram_budget_mb` spans 256 MB to 64 GB — a factor of 256. A linear rail puts every value
below four gigabytes in the first six per cent of its length, which is the failure Baymard
measured. So the rail is logarithmic.

The argument is over whether the rail should exist at all. Two programs in the field replaced
their memory numbers with names. darktable's preferences once carried a page of raw numbers —
thumbnail cache in megabytes, a background thread count, a host memory limit for tiling, a
minimum buffer size, most of them marked as needing a restart, one with two magic values
("values below 500 treated as 500", "0 omits limits")
(<https://docs.darktable.org/usermanual/3.6/en/preferences-settings/cpu-gpu-memory/>) — and now
carries one list of four: small, default, large, unrestricted, described as fractions of the
machine, "roughly 20% of system memory and 40% of GPU memory" up to "darktable may attempt to
use more memory than your system has available"
(<https://docs.darktable.org/usermanual/4.6/en/preferences-settings/processing/>).
FastRawViewer, which is the closest thing on the market to this program, does the same for the
card: "Minimal / Minimal+ (slowest, 512MB or less video RAM)", "Optimal", "Maximal (fastest,
2GB+ video RAM recommended)"
(<https://www.fastrawviewer.com/usermanual17/performance-settings>).

Neither is copied, for two reasons, and the second is decisive.

The first is that the number is what travels. It is what a forum answer quotes, what a note to
the other two machines says, and what somebody types when they want this machine to behave
like that one. "Large" does not travel; it means a different number on every computer it lands
on. What the presets were standing in for is not the name but the *arithmetic* — how much of
this machine, what it costs, what it is holding now — and the arithmetic goes beside the box
rather than being hidden behind a word. FastRawViewer's manual does exactly this, and its
worked example is the model: "a 20-file cache of such JPEGs will take up 1.6 GB instead of 60
to 80 MB". This program already computes half of that sentence and prints it nowhere near a
control: `panels::cache_stats` and `memory` draw resident bytes against budget bytes tier by
tier (`src/app/panels.rs:168-196`, `:206`), and `defaults.rs:90-94` does the
megabytes-per-photograph arithmetic in a comment. Putting that readout beside the control is
half the answer; the route the other way, from the readout to the setting behind it, is §8's.

The second reason is that darktable's form is not available here. Expressing a budget as a
fraction of the machine requires knowing how much memory the machine has, and this program
cannot: nothing in `src/` queries system memory and no dependency in `Cargo.toml` provides one.
wgpu is worse — it reports the adapter's limits (`src/cache/gpu.rs:129-132` reads
`max_texture_dimension_2d`) but not its memory, so "of the 8 GB this adapter reports" cannot be
drawn either. Any percentage-of-machine phrasing costs a new dependency. Absolute megabytes are
what this program can honestly say, and they are also the thing worth saying.

So the four names survive as four buttons above the boxes — *Modest*, *Balanced*, *Generous*,
*Everything this machine has* — carrying absolute figures, filling the boxes in, and then
getting out of the way. Photoshop's dialogue has the same shape for its two obscure cache
numbers: three named workload buttons, and both numbers still on screen
(<https://photoshopguides.github.io/Performance>). That is the brief's "one setting reachable
by several routes" applied to a number, and it costs one row.

`gpu_budget_mb` takes the same shape. It has a second reason to show its arithmetic: the value
is split between the two stores by `gpu_split` (`src/app/stores.rs:88-93`), so a card holding
"1024 MB" is holding 896 for photographs and 128 for thumbnails, and neither number appears
anywhere.

#### The thread count is a rail from one to the machine's cores, and the automatic value is not what it looks like

`decode_threads: 0` means "pick a sensible number" (`src/cache/loader.rs:108-113`). The sensible
number is `(cores − 1).clamp(1, 8)` (`src/cache/loader.rs:256-259`), because
`MAX_DEFAULT_WORKERS` is 8 (`:252`). On a 24-core machine the automatic value is **8**, not 23.
Any label that says otherwise is wrong, and the tick must print what `default_worker_count()`
actually returns — which it can, because `thread::available_parallelism()` is already called
there.

The rail runs 1 to the larger of 16 and the core count, with a mark at 8, and the sentence
under it is the measurement already in the source: "Measured on a 24 core machine, eight
workers sustained 42 images a second and twelve sustained 39, while each worker holding a whole
decoded image cost another 130MB of peak memory" (`src/cache/loader.rs:246-251`). That is
better than any label, because it tells the reader the number is not monotonic — more is slower
past eight — which no rail can express on its own.

Two rules come with it. The automatic value is a **tick above a disabled number**, never a zero
in the number itself: the sentinel means the field has two meanings, and the VS Code CodeQL
extension shipped precisely that bug, a settings UI that "doesn't accept 0 as input" over a CLI
where "0 is passed to the corresponding CLI commands, where it is interpreted as 'use one
thread per core on the machine'" (<https://github.com/github/vscode-codeql/issues/603>). And
the window's range is not the guard: the spawn loop `.expect`s every thread
(`src/cache/loader.rs:122-129`), so a hand-typed 10000 either starts ten thousand decode workers
or panics the process the moment the operating system refuses one. The cap belongs in
`Loader::new` (§3.6), and it is the same rule the file-is-more-permissive-than-the-window
argument of §5.3 depends on everywhere else.

#### `raw.highlight_mode` is two controls, not one slider

LibRaw documents the parameter as "0-9: Highlight mode (0=clip, 1=unclip, 2=blend,
3+=rebuild)" (<https://www.libraw.org/docs/API-datastruct.html>), and this program passes it
straight through as an `i32` with no validation (`src/decoder/raw/mod.rs:106`). The README
repeats the prose (`README.md:684`) and the doc comment repeats it again
(`src/config/mod.rs:96-97`). It is a `u8` acting as a four-way enum with a tail.

A slider from 0 to 9 with named stops is the wrong reading of the brief's rule, because 1 is
not less of anything than 2. Clip, leave unclipped and blend are three different treatments of
the same pixels, not three amounts of one treatment. Only the tail is a quantity: 3 upwards is
the same treatment with more work spent on it. So it is a radio group of four — **Clip · Leave
unclipped · Blend · Rebuild** — with a `3–9` stepper enabled only inside the fourth row. This
is the one place where the brief's rule and the material genuinely disagree, and splitting the
field satisfies both: the enum part gets names, the quantity part gets a continuous control.

#### `RawQuality` is a radio group because its values are algorithms

The three variants are Fast, Balanced and Best (`src/config/mod.rs:113-120`), and they map onto
LibRaw's algorithm numbers 0, 2 and 3 (`src/decoder/raw/mod.rs:35-41`) — bilinear, patterned
pixel grouping and adaptive homogeneity-directed, which the README names (`README.md:681`) and
the `Demosaic` enum's own doc comments describe (`src/decoder/raw/mod.rs:23-31`). Three things
follow.

The numbers are not consecutive: 1 is VNG, which this program does not offer. A rail would have
to hide that or lie about it. The values are not ordered by a single quantity either — AHD is
LibRaw's own default, as the doc comment at `:29` says, and is not merely "more" bilinear; it
is a different reconstruction with different artefacts. And there are three of them, which is
Microsoft's threshold verbatim: "Does the setting have a range of four or more values? If not,
use radio buttons." NN/g adds the reason to prefer radios over a dropdown at this size: they
"have lower cognitive load because they make all options permanently visible so that users can
easily compare them" (<https://www.nngroup.com/articles/listbox-dropdown/>).

So: three radio rows, each with the algorithm named and the cost stated, in the shape `Motion`
already uses. `raw.source` (two variants, `src/config/mod.rs:106-110`) and `raw.pair_with_jpeg`
(three, `src/organize/pairs.rs:45-53`) go the same way for the same reason, and all three have
their sentences already written in the source and drawn nowhere.

#### `cell_aspect` gets four buttons and then a rail, and the escape hatch is not optional

The useful values are not spread across the range; they are the four shapes cameras shoot.
WinUI states the case for steps over a continuous rail, quoted in §5.2. But a preset list with
no way past it fails in a documented and specific way: Lightroom's Standard Preview Size is a
dropdown whose largest fixed value is 2880 px, and a user with a 4K monitor found there was
nothing between that and "Auto", which produced 6720 px previews and hurt performance — "there
are very few relevant standard preview size settings for people using a 4k monitor"
(<https://community.adobe.com/t5/lightroom-classic-bugs/p-need-more-settings-for-standard-preview-sizes/idi-p/12248778>).
That is eight replies and one other person, so it is not a groundswell; it is a clean
demonstration of the failure mode. Four buttons — Square · 3:2 · 4:3 · 16:9 — then **Custom**,
revealing a rail from 0.5 to 3.0.

The same escape hatch is owed to the one preset dropdown the program already has: the four
thumbnail heights in the group panel (`src/view/organize/group/mod.rs:77-82`,
`src/view/organize/thumbnails.rs:21-26`) become a rail from 0 to 240 pt with those four as
named stops, and gain nothing else.

#### `thumbnail_resolution` is deliberately not logarithmic, and it sits next to the column count

Its range spans 128 to 4096, a factor of 32, which by the rule in §5.2 would earn a logarithmic
rail. It does not get one, because every value in it is chosen against the cell width rather
than against a budget, and the cell width is printed on the row above it. A linear rail reads
directly against the number it is being compared to; a logarithmic one does not.

The two fields are adjacent for a reason documented in another program's support forum. An
XnView MP user raised thumbnails from 128×96 to 500×500 and found them blurry; the answer
required three settings across two dialogues — embedded-thumbnail use, "create from original
image if embedded thumbnail is smaller than thumbnail size", and a caching size that had to be
changed "to match new rendering size (500x500) or slightly exceed it" — followed by a rebuild
and a restart (<https://newsgroup.xnview.com/viewtopic.php?t=47571>). That is one user and one
answer, not a chorus, but this program has the same latent pair: `images_per_row` sets how wide
a cell is drawn and `thumbnail_resolution` sets how much of it was decoded, and they live in
the same struct with nothing connecting them. Put them on consecutive rows (§4.5), print the
cell width under the first and "below that they go soft" under the second, and the trap closes
in two lines.

The tick above it, "Match this display", has a computable answer. Capture One is set up the
same way, and the published walkthrough of its preferences says to "set the preview to the same
size in pixels as your display or one position higher", warning that too low a value makes the
program re-read the raw file constantly
(<https://imagealchemist.net/capture-one-preferences-part-2/>; Capture One's own support pages
refused fetching, so this is a third party describing them). The number is already computed in
this program: `longest_edge_in_pixels` reads the monitor (`src/app/mod.rs:881-886`) and hands
it to the image store every frame (`:797`) — and to the image store only. Nothing hands it to
the grid, which is why the thumbnail cache never learns how large a cell has become.

#### `upload_budget_ms` should probably not be on the page at all

Eight milliseconds a frame is a developer's unit, and it is the strongest candidate in this
chapter for the argument §11.2 makes about settings that should not exist. No published guidance
was found on putting a per-frame time budget in front of an end user; what the neighbours do is
instructive, but it is precedent rather than evidence and is offered as such. darktable keeps its equivalent knob, the
OpenCL "micro nap", out of preferences entirely and in the configuration file
(<https://docs.darktable.org/usermanual/4.0/en/special-topics/mem-performance/>).
FastRawViewer names its equivalents rather than numbering them — "Parallel GPU data upload",
and a "Synchronous GPU Operations" mode described as "slower but more stable"
(<https://www.fastrawviewer.com/usermanual17/performance-settings>).

The decision here is to keep the number, behind a disclosure on **Speed and memory** (§4.5),
with the arithmetic beside it — "8 ms is half a frame at sixty a second" — and with the live
frame line that `PerfMetrics::display_metrics` already draws (`src/ui/perf_metrics.rs:74-82`) on
the same page. NN/g requires the disclosure's label to set "clear expectations for what users
will find when they progress to the next level"
(<https://www.nngroup.com/articles/progressive-disclosure/>), which "Advanced" does not and
"Show the exact numbers" does. The sentence this field needs already exists and sits in the
comment block above the wrong function (`src/config/defaults.rs:35-38`); moving it is most of
the work.

### 5.8 Where a stepped keystroke wants a continuous control beside it

Four values have no control on any page today. Two are stepped by a key pressed again and
again, one by dragging a splitter, and the fourth cannot be changed in the running program at
all. Nothing the three gestures do survives the exit.

| Value | The gesture today | Where | What goes beside it |
|---|---|---|---|
| Thumbnails across | `+` / `−`, one column a press, ceiling `MAX_COLUMNS = 16` — which the keys respect and the configuration does not, since `GridView::new` applies only `.max(1)` (`src/view/grid_view/mod.rs:77`), unlike `nr_images_shown`, which is clamped at construction (`src/view/image_view/mod.rs:129`) | `src/view/grid_view/mod.rs:524-527`, `:42` | A rail in the filter bar (`src/ui/filter_bar.rs`), 1–16. ui-patterns states the rule: "Don't use settings for frequently accessed actions — move those to toolbars" (<https://ui-patterns.com/patterns/settings>). Lightroom is described as doing exactly that — thumbnail size a slider on the Grid-view toolbar, *preview* size a preference — but Adobe's own page could not be fetched and that description is second-hand |
| Photographs side by side | `Ctrl +` / `Ctrl −`, clamped 1–8 | `src/view/image_view/mod.rs:282-290`, `MAX_IMAGES_SHOWN` at `:51` | A rail of eight stops on the settings page. Eight values do not need a box; they need the picture to re-tile, which it does |
| The filmstrip's height | nothing. `Ctrl+T` flips `filmstrip_visible`, but `show_filmstrip` returns early while the height is zero, and zero is the default (`src/config/defaults.rs:299`) — so on a configuration nobody has edited, the key does nothing at all | `src/app/mod.rs:713`, `src/app/views.rs:137-141` | A drag handle on the strip's top edge, writing the same field as the rail; and `Ctrl+T` giving the strip a height when it has none. The tick over the rail is what splits "off" out of the height, which §3.6's 60 pt floor depends on |
| The tag panel's width | dragging the splitter, persisted nowhere | `src/ui/tag_panel/mod.rs:62-65` | The splitter keeps working and writes `tags.panel_width`. Note that `SidePanel::default_width` is used only until egui has stored a width for that id, and `PanelState::store` is private (`egui-0.33.0/src/containers/panel.rs:47`), so the rail has to pass `.exact_width()` on the frame the value changes — there is no `.width()` method (`panel.rs:175-209`) |

The metadata side panel belongs in that table and cannot be, because it has no field at all: its
width is a hardcoded `.default_width(340.)` (`src/app/chrome.rs:110`). It, the zoom step
(`ZOOM_STEP = 1.25`, `src/view/image_view/input.rs:17`) and the pan speed (`PAN_SPEED = 1.5`,
`:21`) all need the field before they can have a control, which is §6's argument; when they get
one, the first is a rail with a live edge to drag and the other two are rails with a visible
effect and no exact value anybody would want to reproduce — the one clean case in this chapter
where NN/g's "the specific value does not matter" applies without qualification.

The principle in all of these is the same, and is the strongest form of the brief's
several-routes request: the keystroke stays, the rail is added, and both write one field
through one setter, so `Config::save` runs and the value survives the session. The zoom rail
(`src/view/image_view/bottom_bar.rs:262`) is the counter-example already in the program — a
control that exists, works, and writes nothing anybody can keep.

### 5.9 Where the rule loses, in one list

The rail is preferred everywhere except in the cases below, and each refusal has a reason that
is about the value rather than about the interface.

| Field | Control | Why not a rail |
|---|---|---|
| `raw.source`, `raw.quality`, `raw.pair_with_jpeg` | Radio, sentences | Two and three and three variants; the values are treatments, not amounts (`src/decoder/raw/mod.rs:23-31`, `src/organize/pairs.rs:45-53`) |
| `raw.highlight_mode` | Radio of 4 + a stepper in the fourth row | Three treatments and one quantity in one `u8` (<https://www.libraw.org/docs/API-datastruct.html>) |
| `general.output_icc_profile` | Radio of 3 | Exactly three strings resolve, by substring (`src/metadata/icc.rs:11-15`, `:26`); anything else fails into the log alone (`src/decoder/color.rs:30-32`) |
| `image_view.overlay_corner` | 3 × 3 of radio dots | Five positions on two axes; a rail would have to flatten them into one |
| `slideshow.motion` | Radio, sentences | Already right (`src/app/panels.rs:110-117`) |
| `Callback` | Named radio | A free string whose unknown values become `NoAction` in silence (`src/actions/callback.rs:44-45`) |
| `cell_aspect` | Four buttons, then a rail | Four camera shapes, and everything between them fits nothing — with the Custom escape hatch, which is not optional |
| `decode_threads`, `max_image_edge`, `thumbnail_resolution` | A tick above the control | So that `0` never has to mean two things (<https://github.com/github/vscode-codeql/issues/603>) |
| `full_resolution_neighbours`, `gpu_resident_images`, `preloaded_rows`, `recent_tags`, `gpu_resident_thumbnails` | Steppers | Whole counts sitting hard against a default almost nobody moves, where the exact value is what a second machine needs and there is nothing along the rail to look at (<https://www.nngroup.com/articles/input-steppers/>) |
| `tags.catalog_file`, `cull.rejected_folder`, the six lists | Path row, validated box, row editor | Not quantities at all |

And one refusal in the other direction, which is the honest cost of all of the above.
`ram_budget_mb`, `gpu_budget_mb` and the rest of the memory group are read once when the stores
are built (`src/app/stores.rs:32-66`, called from `src/app/mod.rs:164-165`), so a rail dragged
across the page gives no feedback until the stores are rebuilt. GNOME's rule for pairing a rail
with a box requires that "immediate feedback for changes in the spin box's value is possible"
(<https://developer.gnome.org/hig/patterns/controls/spin-buttons.html>), and by that test these
fields get a box alone. The plan chooses the rail anyway, and pays for it by committing on the
end of the gesture (§3.4) and by making those stores Rebuild rather than Restart, which is §3.5's
mechanism and §12's ordering. If the rebuild is not done, the rail should not be drawn — a
control whose effect appears at the next launch is worse than a number, because it looks like it
is doing something.

`cache.decode_threads` is the one field where that escape is not available. The pool is spawned
once in `Loader::new` and shared by both views (`src/app/mod.rs:162`), so it is the single
setting that can be made neither Live nor Rebuild (§3.5). Its rail is drawn anyway, because the
number is worth reading and worth copying to another machine, and it carries the one line no
other control in the window needs: the workers change at the next launch.

## 6. Settings that do not exist

There are two ways a viewer can refuse to be configured. The first is a field
that exists and has no control: `raw.source` decides whether a raw file is
browsed as its embedded preview or developed from the sensor, and the only way
to change it is a text editor and a restart. That is §3. The second is the one
here — a behaviour with no field at all, fixed in a `const` or a `Default`
impl, where the text editor does not help either and the only remedy is a
recompile.

The configuration holds 110 settings. The keyboard editor reaches 60 of them
and the slideshow window 3, which leaves 47 reachable nowhere in the running
program (§3.1, and the arithmetic in §1.3). This chapter is about the things
that are not among the 110.

### 6.1 The test

An entry earns its place only if both of these hold.

1. **It is fixed here.** A named constant, a `Default` impl, or a value written
   into a widget call, with the `file:line`. "It would be nice if" is not
   evidence.
2. **Somebody disagrees, and can be named.** Either somebody outside this
   repository has asked for it — in a tracker, a forum or a vendor's own
   documentation, with a URL — or the program already contradicts itself about
   the value, so that the two people who would choose differently are both
   written into the source. Where the outside evidence is one filer with two
   reactions, this chapter says so rather than dressing it up; where a row rests
   on the second limb, it says that too and names the two places that disagree.

The second test is §11.8's field budget written for this chapter — "a new field
requires either the two reasonable people who would choose differently, or the
name of somebody who asked" — with the first of those two limbs tightened so it
cannot be met by imagining the two people. Two constants that disagree, or two
surfaces offering the same number over different ranges, are those two people
already written down. One row below rests on that limb alone, `image_view.page`
(6.4), and it says so.

A third question decides not whether a field exists but which file it lands in,
and the rule for it is §4's (§4.8): a **setting** goes in `config.json` and
describes the user, **session state** goes in `session.json` and describes where
they were, and an **ephemeral** value goes nowhere. Several rows below need the
first two together — a setting for what a fresh install does, and session state
so the value survives lunch. The program already keeps session state
(`src/session.rs:53-65`) and puts nothing in it but the window, the open folder
and the last position in each of 64 folders (`src/session.rs:30`), because
eframe is built without the `persistence` feature (`Cargo.toml:13`) and the
`eframe::App` impl has only `update` and `on_exit` (`src/app/mod.rs:791`,
`:866`) and no `save()`.

Where neither limb of the second test is met but the first is strong, the entry
goes in 6.13 with its reason and is sent to §13. The point of that section is
that this chapter should be finishable, not a standing wish list.

### 6.2 The list

Pages are the eleven of §4.5, in the order §3.2 draws them; the placement
argument is §4's and the control is §5's (§5.5, §5.6). Thirty-nine fields, which
is the register §3.2 counts into the plan's totals — 148 rows on the pages and
149 keys in the file — and which §4.9 assigns to sections. Thirteen of them are
runtime state the program already keeps and throws away on exit, eight are
compile-time constants given a name, nine are the mouse, and nine are genuinely
new choice. What follows the table is the argument for each.

| Setting | Page | Default | Where it is fixed now | Kind |
|---|---|---|---|---|
| `browsing.sort`, `browsing.descending` | Opening a folder | `Name`, ascending | `src/view/narrow.rs:116-117` | setting + session state |
| `browsing.flag`, `browsing.min_stars`, `browsing.max_stars`, `browsing.label` | Opening a folder | everything shown | `src/view/narrow.rs:36-48` | setting + session state |
| `browsing.filter_follows_folder` | Opening a folder | `true` | `src/app/mod.rs:297` never resets it | setting |
| `browsing.stack_by_default` | Opening a folder | off | `Stacking::default()` (`src/app/mod.rs:229`) | setting — argued in §4.8 |
| `group.max_gap`, `group.tolerance`, `group.min_frames` | Opening a folder | `60 s`, `12`, `2` | `src/organize/group/mod.rs:45-53` | setting — argued in §4.8 |
| `general.start_in` | Opening a folder | `Image` | `src/app/mod.rs:200` | setting |
| `general.start_fullscreen` | Opening a folder | off | `src/main.rs:44` (a flag only) | setting |
| `general.start_folder` | Opening a folder | none | `src/main.rs:43-45`, `src/app/mod.rs:255-285` | setting |
| `image_view.zoom_step` | The photograph | `1.25` | `src/view/image_view/input.rs:17` | setting |
| `image_view.zoom_step_factor` | The photograph | `2.0` | `src/view/image_view/zoom.rs:45` | setting |
| `image_view.zoom_step_max` | The photograph | `8.0` | `src/view/image_view/zoom.rs:11` | setting |
| `image_view.pan_speed` | The photograph | `1.5` | `src/view/image_view/input.rs:21` | setting |
| `image_view.page` | The photograph | `10` | `src/view/image_view/mod.rs:49` | setting |
| `grid_view.badges` | The contact sheet | `Marks` | `src/view/grid_view/cell.rs:27-28`, `src/view/grid_view/mod.rs:84` | setting + session state |
| `grid_view.filmstrip_visible` | The contact sheet | off | derived from a height at `src/app/mod.rs:175`, applied at `:234` | setting + session state |
| `grid_view.filmstrip_edge` | The contact sheet | `Bottom` | `src/view/grid_view/filmstrip.rs:63` | setting |
| `grid_view.click_opens` | The contact sheet | off — a click picks out, a double-click opens | `src/view/grid_view/mod.rs:444-458`, `src/app/views.rs:84-87` | setting |
| `tags.sidecar_naming` | Stars, flags and labels | `photo.cr2.xmp` | `src/annotations/sidecar.rs:14-18` | setting |
| `cull.confirm` | Moving and deleting | all on | `src/app/cull.rs:85-88` | setting |
| more than nine destinations | Moving and deleting | two shipped | `src/ui/destinations.rs:74`, `:122-140` | a cap to lift, not a field |
| `general.theme` | The window | `Dark` | `src/ui/theme.rs:23` | setting |
| `general.backdrop` | The window | `#777777` | `src/view/image_view/layout.rs:12`, `src/view/grid_view/mod.rs:32`, `src/view/grid_view/filmstrip.rs:31-32` | setting |
| `general.panels_at_start` | The window | all closed | `src/app/mod.rs:201-203`, `:212`, `:230` | setting + session state |
| `general.side_panel_width` | The window | `340` | `src/app/chrome.rs:110` | setting + session state |
| `mouse.wheel`, `mouse.wheel_reversed` | Keys and mouse | next/previous, not reversed | `src/view/image_view/input.rs:197-201` | setting |
| `mouse.sheet_wheel` | Keys and mouse | scroll the sheet | `src/view/grid_view/mod.rs:274` — an ordinary `ScrollArea` | setting |
| `mouse.ctrl_wheel` | Keys and mouse | `Zoom` | `src/view/image_view/interaction.rs:26-29`, `src/view/grid_view/mod.rs:516-528` | setting |
| `mouse.drag` | Keys and mouse | left | `src/view/image_view/interaction.rs:41-42` | setting |
| `mouse.double_click` | Keys and mouse | fit ↔ 100% | nothing: `grep -rn double_click src/` returns nothing | setting |
| `mouse.middle`, `mouse.back`, `mouse.forward` | Keys and mouse | nothing, previous, next | nothing: no `PointerButton` anywhere in `src/` | setting |
| `menus.settings_rows` | Keys and mouse | on | nothing: the built-in rows do not exist yet (§7.11) | setting |

Two of those defaults are not the present behaviour, and both are marked as such
where they are argued: `mouse.drag` ships as *left* rather than *any* (6.5), and
`grid_view.click_opens` ships off (6.6). In each case the present behaviour is
one of §9.2's faults, and shipping it as the default would re-create the fault on
a fresh install. Everywhere else the shipped default is what the program does
today.

Three things that are not new fields belong beside this list, one line each.
`grid_view.images_per_row` exists and has no control and no write-back, which is
an orphaned pair and §4.8 has it. `grid_view.filmstrip_height` exists and stores
a height and a visibility in one number; the visibility half becomes
`grid_view.filmstrip_visible` above and the height stays the field it is (6.6).
The slideshow window draws three of its five fields and omits
`start_with_frame_enabled` and `image_frame_background_color_override`
(`src/config/mod.rs:535`, `:537`; window at `src/app/panels.rs:92-135`) — a
missing control, not a missing field, and therefore §3.1's and §5's.

### 6.3 The theme, and the ground behind the photograph

#### The theme

`apply_theme` calls `ctx.set_theme(ThemePreference::Dark)` unconditionally
(`src/ui/theme.rs:23`), above a doc comment that gives the reason: "The theme is
deliberately dark whatever the desktop prefers: a light surround shifts how the
photograph in front of it reads" (`src/ui/theme.rs:17-18`). There is no field,
no menu entry, and no command-line flag.

The reason is sound and the conclusion is too wide. What sits around the
photograph is the backdrop, and the backdrop is a *separate* hardcoded value
(`src/view/image_view/layout.rs:12`) that the theme does not touch. What
`set_theme` actually decides is the colour of the menu bar, the side panel, the
keyboard editor and the slideshow settings window — none of which is around a
photograph, and two of which are windows that cover it. Somebody working in a
bright room, or on a laptop whose desktop theme follows the sun, is asking for
something the stated reason does not forbid.

Users expect this to be a setting, and expect it to follow the system: ImageGlass
#434, "Sync dark/light theme with Windows 10", 6 reactions and since shipped
(<https://github.com/d2phap/ImageGlass/issues/434>); nomacs #943, "Use system
theme by default?", whose filer's first experience was "I opened the app for the
first time and found it very hard to read any of the menu text because it was a
black font against a dark gray background"
(<https://github.com/nomacs/nomacs/issues/943>). The cautionary tale is
nomacs #1062, where the theme setting exists, prompts for a restart, and does not
change the theme after one — the working fix being to hand-edit `themeName312`
in the config (<https://github.com/nomacs/nomacs/issues/1062>). A theme setting
that lies is worse than none, so this one has to be **Live** in §3.5's sense,
and it can be: `apply_theme` is a free function over the context called from one
place, `App::new` (`src/app/mod.rs:149`), and calling it again from the settings
handler repaints with the new palette. Nothing in `src/` holds a `Visuals` of its
own — the only two writers are `set_theme` and `set_visuals_of`, both inside that
function (`src/ui/theme.rs:23`, `:33`).

**Name:** *Interface theme*. **Values:** Dark · Light · Follow the desktop.
**Default:** Dark — the existing behaviour, and the existing reason is good
enough to keep for a fresh install even though it is not good enough to enforce.
**Cost:** one enum field, and a light palette to sit beside the six colours at
`src/ui/theme.rs:26-31`. The palette is the work and it is not a small one; the
wiring is an afternoon.

#### The ground behind the photograph

Three greys, in three files, none of them reachable:

| Where | Value | What it is behind |
|---|---|---|
| `src/view/image_view/layout.rs:12` | `rgb(119,119,119)` | the photograph |
| `src/view/grid_view/mod.rs:32` | `rgb(119,119,119)` | each contact-sheet cell |
| `src/view/grid_view/filmstrip.rs:31-32` | `rgb(38,38,38)` / `rgb(70,70,70)` | the filmstrip and its cells |

There is exactly one override and it is in the wrong place:
`slideshow.image_frame_background_color_override`, consulted only while a
slideshow is running (`src/view/image_view/mod.rs:555-565`), and not offered by
the slideshow settings window either (6.2). So the one person the viewer lets
choose a backdrop is the one using it as a photo frame, and the photographer
judging a print against a neutral ground cannot.

This is a well-attested ask, and the reason people give is not taste.
Geeqie #476: "Some images have black borders which are not distinguishable from
the black background. When it is needed to be able to see these, preferences
should offer a way to change the background color with a color picker with
manual hex RGB entry" (<https://github.com/BestImageViewer/geeqie/issues/476>).
ImageGlass #1902, "Command to set ImageGlass background to black, white or red
etc etc", asks for it bound to a hotkey
(<https://github.com/d2phap/ImageGlass/issues/1902>); ImageGlass #2177 asks for a
*separate* fullscreen backdrop, for OLED
(<https://github.com/d2phap/ImageGlass/issues/2177>); Geeqie #219 asks for a
solid colour in place of the transparency checkerboard
(<https://github.com/BestImageViewer/geeqie/issues/219>). ImageGlass #494 asks
for a configurable border round the image, because images extending to the edge
of the window "might not [be] ideal for critical image viewing", and in the same
breath asks to be able to change the background colour "from the 'Menu', or the
right click menu" (<https://github.com/d2phap/ImageGlass/issues/494>) — which is
the print-judging case in another form, and which this viewer already
half-implements as the white frame (`src/view/image_view/canvas.rs:374`, size at
`image_view.frame_size_relative_to_image`, colour hardcoded). The right-click
half of that request is §7's, and the photograph's menu is where it lands.

**Name:** *Behind the photograph*. **Value:** a colour. Black, mid grey and
white are the three the print-judging case actually reaches for, and getting to
one of them should not mean hunting for `#000000` in a colour wheel — how that
is drawn is §5.6's. **Default:** `#777777`, the present value, whose own comment
argues for it: "neutral enough not to shift how a photograph reads against it"
(`src/view/image_view/layout.rs:10-11`). **Scope:** one setting read by all
three places, with the filmstrip deriving its two greys from it rather than
carrying its own; the existing slideshow override stays as the per-mode
exception it already is. **Cost:** one `Color32` field and replacing three
`const` reads. The image view already routes its fill through
`ImageView::background_colour` (`src/view/image_view/mod.rs:555-565`, called at
`:520`), so half the plumbing is done.

### 6.4 Zoom step, pan speed, and what a page is

Five compile-time constants govern how far every movement key moves:

| Constant | Value | Where | What it does |
|---|---|---|---|
| `ZOOM_STEP` | `1.25` | `src/view/image_view/input.rs:17` | one press of `+` or `-` |
| the doubling in `zoom::step` | `2.0` | `src/view/image_view/zoom.rs:45` | one press of `Space` |
| `MAX_STEP` | `8.0` | `src/view/image_view/zoom.rs:11` | where `Space` wraps back to fitted |
| `PAN_SPEED` | `1.5` | `src/view/image_view/input.rs:21` | panels per second under a held `w/a/s/d` |
| `PAGE` | `10` | `src/view/image_view/mod.rs:49` | `PageUp` / `PageDown` in the image view |

Two of the five have a doc comment that argues for the number and two only say
what it is. `ZOOM_STEP` argues: "A quarter each way: small enough to arrive at a
particular framing, large enough that crossing a useful range does not take
twenty presses" (`src/view/image_view/input.rs:14-16`). That is right for a
24-megapixel file on a 1440p screen and wrong for a 100-megapixel file on a 4K
one, where crossing from fitted to actual pixels is not "a useful range" but a
factor of eight. `MAX_STEP` and `PAN_SPEED` are simply stated
(`src/view/image_view/zoom.rs:10`, `src/view/image_view/input.rs:19-20`).

The doubling is worth naming separately, because it is easy to miss: `Space`
does not step by `ZOOM_STEP` at all. `zoom::step` writes `viewport.zoom * 2.0`
until it passes `MAX_STEP` and then returns to fitted
(`src/view/image_view/zoom.rs:43-49`). So there are two independent zoom
increments in the program, one bound to `+`/`-` and one to `Space`, and neither
is settable.

Pan speed has the best direct evidence. nomacs #242, "panning via keyboard is
undocumented and unconfigurable", complains that neither of the two available
pan gestures "allow more precise control of the panning speed in a large and/or
highly zoomed-in image" (<https://github.com/nomacs/nomacs/issues/242>) — which
is exactly the case here, since `PAN_SPEED` is a fraction of the *panel* per
second and therefore moves a proportionally larger distance across the
photograph the further in you are.

`PAGE = 10` is the one row in this chapter admitted on 6.1's second limb alone.
Nobody outside has filed for it. The program contradicts itself about it: the
contact sheet's page key — `sc_scroll`, `PageDown` by default
(`src/config/defaults.rs:307-309`) — scrolls half a row
(`src/view/grid_view/mod.rs:297-301`) while the image view's moves ten
photographs (`src/view/image_view/input.rs:111-112`,
`src/view/image_view/mod.rs:252-253`), and neither number is stated anywhere on
screen. Those are the two people who would choose differently, and they are both
in this repository. The source has thought about it and reached half an answer —
"A round number rather than a screenful, because the image view shows one
photograph and a screenful of one is one" (`src/view/image_view/mod.rs:47-48`) —
which explains why ten is not a screenful without explaining why it is ten rather
than twenty-five, and does not touch the disagreement with the sheet.

**Names:** *Zoom step*, *Zoom-step key multiplier*, *Largest zoom step*, *Pan
speed*, *Page size*. **Useful ranges:** roughly 1.05 to 2.0 for the zoom step,
1.5 to 8 for its multiplier and ceiling, 0.25 to 5 panels a second for the pan,
and 2 to 100 for the page — §5.5 turns those into controls, and the zoom step is
the one that has to be logarithmic, for which the pattern already exists in the
magnification slider (`src/view/image_view/bottom_bar.rs:262-263`).
**Defaults:** the five values above, unchanged. **Cost:** five fields; the
constants are each read in one or two places
(`src/view/image_view/input.rs:87-88`, `:185`,
`src/view/image_view/zoom.rs:44-45`, `src/view/image_view/mod.rs:252-253`).

### 6.5 The mouse

`PointerButton` does not appear anywhere in `src/`, and `grep -rn double_click
src/` returns nothing. The one place that distinguishes a button at all is the
directory tree, through `label.secondary_clicked()` (`src/ui/tree.rs:266`). So
every button pans (`src/view/image_view/interaction.rs:41-42`), a wheel notch
both advances the collection and pans what is now on screen
(`src/view/image_view/input.rs:197-201` and
`src/view/image_view/interaction.rs:40`, `:45`), Ctrl+wheel is welded to zoom in
the image view and to the column count in the sheet
(`src/view/image_view/interaction.rs:26-29`,
`src/view/grid_view/mod.rs:516-528`), and the middle, back and forward buttons
do nothing. §9 owns what each of those gestures should do and what has to change
in the input path first; this section is only the argument that the answers
belong in the configuration.

The wheel is the loudest complaint in the whole viewer corpus (§2), and the
important thing about it is what the complainants ask for. nomacs #237 has run
since 2018 across thirty-two comments from sixteen accounts
(<https://github.com/nomacs/nomacs/issues/237>). The line to design against is
**vosian**'s: "personally I'm happy even if these settings aren't made default,
just being able to bind it would be more than enough." Nobody is asking to win
the argument about the default. They are asking to be allowed to lose it
locally. **cjrobe**, five years later: "The fact that Nomad won't function in
any way except counter to every single popular graphics editor makes in a
non-starter for me. Uninstalled in two minutes." The endgame is instructive too:
in 2025 a collaborator pointed out that two existing checkboxes, *Mouse Wheel
Zooms* and *Next Image on Horizontal Zoom*, would already swap the functions —
while adding "I can't say if 3.16/3.17 had this.. maybe maybe not" — and
conceded that mouse controls "need a refactor and should have their own settings
section". Whenever the checkboxes arrived, seven years of the thread went by
without anybody finding them, which is the argument for putting the whole
gesture table in one place rather than scattering three booleans (§4.5).

Buttons: nomacs #864, "Mouse Shortcuts" — "Would love to be able to change what
the mouse buttons do in nomacs. Specifically, I'm searching for a way to make
double left click open the thumbnail preview instead of going into fullscreen,
and middle click to close the nomacs instance", with the filer noting they could
not find it "in the nomacs settings, or the keyboard shortcuts menu, or the
config file" (<https://github.com/nomacs/nomacs/issues/864>). nomacs #347 asks
for the mouse to move *into* the shortcut manager, listing "Scroll Up, Scroll
Down, Left Click, Middle Click, Right Click" and noting that this "would allow
for any key combination, e.g. Shift + Right Mouse Click"
(<https://github.com/nomacs/nomacs/issues/347>). nomacs #237 again,
**PinkSerenity**: "add an option where you are presented with the four different
scroll actions and where you can assign buttons to all of them."

Double-click: ImageGlass #648, "Left double-click = Full Screen", 5 reactions
(<https://github.com/d2phap/ImageGlass/issues/648>); ImageGlass #909, "add an
option to mouse double click to fullscreen … dpuble click in fullscreen also
goes back to window/frameless mode"
(<https://github.com/d2phap/ImageGlass/issues/909>); ImageGlass #381 asks for
single left-click to switch between actual size and fit-to-window
(<https://github.com/d2phap/ImageGlass/issues/381>). Three filings against a
click that currently does nothing, disagreeing about what it should do instead —
which is the argument for making it a setting rather than for picking one.

| Setting | Values | Default | Why that default |
|---|---|---|---|
| *Wheel, on the photograph* | next/previous · zoom · pan · nothing | next/previous | What `scroll_navigation: true` does now (`src/config/defaults.rs:136-138`) |
| *Wheel, in the contact sheet* | scroll · next/previous · nothing | scroll | Present behaviour: the sheet is an ordinary `ScrollArea` (`src/view/grid_view/mod.rs:274`) |
| *Reverse the wheel* | on/off | off | Present behaviour; the complaint is that it cannot be reversed, not that it is wrong |
| *Ctrl and the wheel* | the same four | zoom | Present behaviour, and what the README documents (`README.md:795`) |
| *Drag with* | left · middle · right · any | **left** | Not the present behaviour: see below |
| *Double-click* | any command in the registry, or nothing | fit ↔ 100% | Picview's own default for the same gesture, and ImageGlass's existing behaviour; and it is reversible, unlike fullscreen |
| *Middle click* | any command, or nothing | nothing | No safe consensus: nomacs #864 wants close, others want pan |
| *Back / Forward buttons* | any command | previous / next | The only thing those buttons mean anywhere |

**On the two wheel fields.** They are two fields rather than one because the
views want different answers and a person who changes one has not thereby said
anything about the other. Somebody who sets the photograph's wheel to
next/previous — which is today's default — has not asked for a contact sheet
that jumps one photograph per notch and can no longer be scrolled, which at
sixteen columns and four hundred frames is unusable. The sheet's wheel therefore
carries its own field with its own default, and today's answer is the right one
to ship: `ScrollArea::vertical` at `src/view/grid_view/mod.rs:274` scrolls, and
wheel-down means later, which is what every list widget in the toolkit does.
That the image view disagrees with it about direction is §9.2's second fault and
§9.3 settles which of the two changes; the field pair is what makes the answer
sayable either way. The demand is nomacs #237's, quoted above, which asks for
"the four different scroll actions" and the ability to assign to each.

**On the drag default.** *Any* is what the program does today
(`is_decidedly_dragging()` is button-agnostic,
`src/view/image_view/interaction.rs:41-42`), and it is the one place in this
chapter where shipping today's behaviour would be wrong. With any button
panning, whether the right button opens a menu or moves the photograph is decided
by egui's `max_click_dist: 6.0` points and `max_click_duration: 0.8` seconds
(`egui-0.33.0/src/input_state/mod.rs:111-112`): move six points on the way down
and the menu never appears. Every right-click menu §7 proposes is reachable only
by a steady hand until the pan is restricted to one named button, which is what
§12 Stage 0 item 8 does before any field exists ("Left button only for now") and
what §9.2's fourth fault argues for. So *left* is the shipped default and *any*
remains a legal value for anybody who wants the present behaviour back.

On the double-click default: Picview ships it as a four-way setting — "Toggle
Zoom (default), Full screen, Close window, None" — and ImageGlass #381's filer
notes in passing that in ImageGlass "that same behavior is already accessible
using double left click", meaning the 100%/fit toggle
(<https://github.com/d2phap/ImageGlass/issues/381>). That is the evidence for
fit ↔ 100% as the default; it is not that every viewer does it.

The values are commands, not a bespoke enum, so the vocabulary is the one the
registry already publishes and the keyboard editor already renders
(`src/config/bindings.rs:85-436`). That is also what makes the count bearable:
adding a gesture adds a row, not a vocabulary. Directory Opus already does this,
with nine values for the left button alone. The counterweight is worth writing
down rather than arguing round — IrfanView's author's position on making
double-click configurable is that "more options are not always a good move as
they make programs harder to support", which is right about a bespoke setting
per gesture and wrong about this one, for the same reason: the values are
already there and already documented, one sentence per command. §11 has the
general form of that argument.

**Cost.** Lower than it looks: egui 0.33 already exposes
`PointerButton::{Primary, Secondary, Middle, Extra1, Extra2}`
(`egui-0.33.0/src/data/input.rs:581-597`), and the central panel's response is
already upgraded with `.interact(Sense::click())`
(`src/view/image_view/layout.rs:88`) while each grid cell already senses clicks
(`src/view/grid_view/mod.rs:391`). §9 has the three pieces of real work.

**One more field on the same page, and it is not a gesture.** §7 gives roughly
twenty surfaces a context menu, and every one of those menus ends with a small
group of settings rows and a route to the settings window. Somebody who wants
their own entries and not the program's needs one switch for it, and that is
`menus.settings_rows`, default on, on *Keys and mouse* beside the two menu lists
the configuration already carries (`image_view.context_menu`,
`grid_view.context_menu`, `src/config/mod.rs:387`, `:457`). The evidence is
ImageGlass #1342: "There are too many options in the right-click menu, some
functions I can't use, and sometimes I even click accidentally"
(<https://github.com/d2phap/ImageGlass/issues/1342>). It sits in a `menus`
section of its own for one reason, and the reason is thin: the two fields it
governs are in two different sections, so there is no section it could go beside
the ones it is about, which is the rule every other new section here follows
(§4.9). A section holding a single boolean is the weakest thing in this register
and is written down as such rather than hidden. What turning it off leaves
behind, and why nothing becomes unreachable, is §7.11's.

### 6.6 The contact sheet

#### Which badges the cells start with

`Badges::{None, Marks, Full}` has a `#[default]` of `Marks`
(`src/view/grid_view/cell.rs:27-28`), set fresh in `GridView::new`
(`src/view/grid_view/mod.rs:84`), cycled by `Ctrl+I`
(`src/view/grid_view/mod.rs:530-531`, `src/config/defaults.rs:373-375`), and
absent from `GridViewConfig` (`src/config/mod.rs:437-487`). The enum's doc
comment gives the reason: "Cycled with one key rather than settled in the
configuration, because how much a person wants to see changes with what they are
doing: everything while triaging, nothing while looking"
(`src/view/grid_view/cell.rs:19-21`).

The observation is right and the conclusion does not follow. That a value
changes several times an hour is an argument for a key, which it has; it is not
an argument against a starting value. And the cost of getting the start wrong is
real: the file name lives in `Full`, so `grid_view.caption_format` — a
configurable template whose grammar the README documents at length
(`README.md:330-353`) while never naming the field — is invisible until the user
presses a key (`src/view/grid_view/mod.rs:482`). The key itself is documented,
in the cheat sheet the registry generates and in the README twice
(`README.md:671`, `:836`). Nothing beside the cells it governs says so, which is
the part that matters, and which §10 answers.

Outside evidence that the *default* badge state is a live argument: darktable's
culling mode hides stars unless the pointer is over the cell, which is
deliberate — "By default, on culling mode, block overlays with infos and stars is
visible only when hovering the image with mouse cursor" — and a user filed a bug
against it anyway, because the default was wrong for the task
(<https://discuss.pixls.us/t/culling-mode-ratings-stars-do-not-show/20517>).
Lightroom's badges "simply don't display due to space constraints" when
thumbnails get small, and the official advice is to make the filmstrip bigger
(<https://www.lightroomqueen.com/disappearing-thumbnail-icons/>).

**Name:** *What the cells say to begin with*. **Values:** the three that exist.
**Default:** `Marks`, unchanged. **Needs both kinds:** the setting for a fresh
install, and session state so the choice survives a restart mid-cull. **Cost:**
one field; `Badges::ALL` already exists (`src/view/grid_view/cell.rs:34`) and is
currently referenced only by a test (`:283`, `:290`), the cycle itself going
through `next()`.

While there: `CAPTION_HEIGHT` is a flat 20 points whenever badges are on,
whatever the cell size (`src/view/grid_view/cell.rs:15`, `:48`) with the font
capped at 13 (`:90`), so at sixteen columns the strip is proportionally enormous
and at one column the stars are a sliver in a wall. That is a layout bug rather
than a missing setting, and it should be fixed by scaling with the cell against a
floor. A *badge size* field would be the wrong answer to it, and goes to §13.

#### What a plain click does

`src/view/grid_view/mod.rs:444-458` handles a click: `Ctrl` toggles the
selection, `Shift` extends it, and a plain click writes `self.selected`, which
`src/app/views.rs:84-87` reads on the same frame to switch to the image view. So
one click leaves the surface, and the only way back is `Backspace`
(`src/config/defaults.rs:53-55`). Nothing in
`src/` reads a double-click at all — `grep -rn double_click src/` returns
nothing.

The sheet already has every part of the other model and uses it everywhere
except here: a cursor, a multiple selection, `Ctrl`-click, `Shift`-click, `Space`
to pick out (`src/view/grid_view/mod.rs:619-624`) and `Enter` to open (`:605-607`).
The plain click is the one gesture that contradicts the rest, which is §9.2's
sixth fault and §9.3's behaviour table. Both limbs of 6.1 are met. Inside, the
contradiction is the sheet's own five other gestures. Outside, viewers make this
transition a setting rather than a decision: XnView has *Settings → Interface →
Switching mode*, "Use double click to switch between: Browser <> Fullscreen |
Viewer <> Fullscreen" (<https://newsgroup.xnview.com/viewtopic.php?t=29695>),
and ImageGlass documents `"MouseClickActions": contains click, double click
events of left / right / wheel button` as a configuration key
(<https://imageglass.org/news/announcing-imageglass-moon-9-0-beta-2-78>).

**Name:** *A click in the contact sheet*. **Field:** `grid_view.click_opens`.
**Values:** picks the photograph out, and a double-click opens it · opens it.
**Default:** off — a click picks out. This is the second of the two defaults in
6.2 that is not the present behaviour, and the reason is the same as the drag's:
what the program does today is a fault rather than a preference, and §12 Stage 7
item 5 makes the change whether or not anybody ever touches the field. **Cost:**
a `double_clicked()` branch beside the existing `clicked()` one, and moving the
mode switch behind it.

#### Whether the filmstrip is there at all

`filmstrip_visible` is a bool on `App` (`src/app/mod.rs:123`), seeded once from a
height — `settings.grid_view.filmstrip_height > 0.0` (`:175`, applied at `:234`)
— flipped by `Ctrl+T` (`:713`) and thrown away on exit. The height's default is
zero (`src/config/defaults.rs:299-301`), so on a configuration nobody has edited
`Ctrl+T` flips a boolean that `show_filmstrip` then ignores, because it returns
while the height is zero (`src/app/views.rs:139`). The binding is in the
registry and on every mode's cheat sheet, and it does nothing at all.

One number is carrying two decisions and they are not the same decision: how
tall the strip is, and whether it is up. Splitting them is §3.5's repair and this
is the field it needs. It also makes "keep my height, hide the strip"
expressible, which it is not today, and it lets the height control have a floor
instead of carrying "off" as the value zero (§5.5). The outside evidence is the
panel-persistence theme collected in 6.11 — qimgv #66 and #400, Geeqie #966 —
which is the same request about the same panel; the internal limb is the key
that cannot act.

**Name:** *Show the filmstrip*. **Default:** off, which is what a fresh install
does now and what the height's own comment argues for: "The strip takes room
from the photograph, so it is asked for" (`src/config/defaults.rs:296-298`).
**Needs both kinds:** the setting for a fresh install, session state so `Ctrl+T`
survives a restart. **Cost:** one field, read in `App::new` in place of the
derivation at `:175`, and dropping the `height <= 0.0` guard from
`src/app/views.rs:139` and `src/view/grid_view/filmstrip.rs:59`.

#### Which edge the filmstrip is on

`egui::TopBottomPanel::bottom("filmstrip")`
(`src/view/grid_view/filmstrip.rs:63`). There is no field, and the strip is
drawn before the image so the photograph is fitted to what is left
(`src/app/views.rs:96-99`), which is the right half of the behaviour.

The demand is for the *other* half. nomacs #305: "Sometimes I want to view what
is at the top of the photo currently selected, which means I have to zoom in and
pull the photo down to see what's hidden under the thumbnail panel", with two
fixes proposed — "Maybe the current photo can adjust to be below the bounds of
the panel. Or maybe the thumbnail panel can have a vertical option, on the left
or right side of the screen" (<https://github.com/nomacs/nomacs/issues/305>).
avis-imgv already does the first. The second matters here for a reason nomacs did
not have: the viewer's own default says most cameras shoot three to two
(`src/config/defaults.rs:366-369`), so on a 16:9 monitor a horizontal photograph
fitted to the window leaves bars at the sides and nothing at top or bottom — the
strip takes room from the picture where there is none to take, and would take
none where there is plenty.

**Name:** *Filmstrip edge*. **Values:** Bottom · Top · Left · Right.
**Default:** Bottom. **Cost:** four or five times what it looks like, and worth
stating in full because this sentence is what decides whether the field ships.
`egui::TopBottomPanel` and `egui::SidePanel` are separate structs sharing no
trait (`egui-0.33.0/src/containers/panel.rs:110`, `:601`), so the drawing body
cannot be handed a different constructor: it is either duplicated or lifted out
and called from both arms of a match. Five things inside those forty lines are
bound to the axis (`src/view/grid_view/filmstrip.rs:63-95`) — `.exact_height`
becomes `.exact_width` (`egui-0.33.0/src/containers/panel.rs:703` against
`:209`), `ScrollArea::horizontal` becomes `::vertical`, `ui.horizontal` becomes
`ui.vertical`, `horizontal_scroll_offset` becomes `vertical_scroll_offset`, and
the centring arithmetic reads `available_height` in place of `available_width`.
And the cell size is derived from the panel's thickness — `let cell = (height -
8.0).max(16.0)` (`:70`) — so a strip on the left or right is given a width
instead, which means `show_filmstrip` (`src/app/views.rs:137-147`) has to supply
one and the field that today stores a height has to be read as whichever
measurement the chosen edge wants. Small work, and not one line.

#### The thresholds that decide what a run is

`group::Settings::default()` is hardcoded at 60 seconds, a tolerance of 12 and a
minimum of two frames (`src/organize/group/mod.rs:45-53`), and the two surfaces
that expose those numbers do not agree about their own ranges: the filter bar
allows a gap of 1–600 s and a tolerance of 0–32 (`src/ui/filter_bar.rs:134`,
`:147`), the group panel 1–3600 s and 0–64 (`src/view/organize/group/mod.rs:60`,
`:65`). They describe the camera and the way somebody shoots rather than the
folder in front of them, and the program asks for them again every session, in
two places, with two answers.

Both limbs of 6.1 are met. The internal one is that disagreement: one value, two
surfaces, two ranges, and the two people who would choose differently have both
already written a `range()` call. The outside one is a run of requests for
grouping rules that can be stated and checked. On Lightroom Queen, **studio_2**
asks "why isn't there (or I missed it) simply the command to stack by fixed
number of shots?", because "using autostack LR sometimes grouped together two or
three stacks which means some extra time wasted", and **John R Ellis** replies
that "Many have asked for better auto-stacking since time immemorial -- it's a
fairly popular feature request"
(<https://www.lightroomqueen.com/community/threads/why-no-auto-stack-by-fixed-number-of-bracket-shots.50182/>).
A second thread records the other half of it, which is that the rule is invisible
when it fails: "The message 'Current Slider settings does not allow auto
stacking' is useless in informing the user of the problem"
(<https://www.lightroomqueen.com/community/threads/stack-by-visual-similarity-what-am-i-missing.53788/>).
§2.11 quotes both. A gap, a tolerance and a minimum run that are named, settable
and shown on screen are what that asks for. §4.8 makes the placement argument and
§5.5 settles the single range.

### 6.7 Browsing defaults

#### Sort order

`SortBy` defaults to `Name` in code (`src/view/narrow.rs:116-117`) and lives
only in `Narrowing`, which is `Default::default()`-ed once per session
(`src/app/mod.rs:228`). It is reachable — the combo and the direction arrow are
in the filter bar (`src/ui/filter_bar.rs:249-277`) — and it is forgotten on
exit, and there is no way to say what it should start as.

JPEGView #216, "Remember previously-used display sort order": the filer switches
between name-sort and date-sort depending on the task, and "the sorting order
keeps getting reset to the one set in the `.ini` when I open a new image"
(<https://github.com/sylikc/jpegview/issues/216>). ImageGlass #1943 is the same
nerve from the other side — a regression where the viewer stopped honouring
Explorer's sort order, closed as not planned
(<https://github.com/d2phap/ImageGlass/issues/1943>).

**Name:** *Order folders by*. **Fields:** `browsing.sort`,
`browsing.descending`. **Default:** Name, ascending — which is what happens now,
and which the crawler already produces naturally (`src/crawler.rs:212-217`).
**Needs both kinds.** **Cost:** two fields; the combo already exists and
`SortBy::ALL` and `label()` are already written (`src/view/narrow.rs:124-134`).

#### Filter rules, and whether they follow you

Two separate holes.

`Rules::default()` is hardcoded to "show everything"
(`src/view/narrow.rs:36-48`), so the one rule the source itself describes as
"the one people leave on" — `FlagRule::NotRejected`
(`src/view/narrow.rs:58-59`) — cannot be made the default. nomacs #138, "Filter
by star rating", open since July 2017 with 6 reactions, asks for something
smaller still: "only show images with > 3 stars"
(<https://github.com/nomacs/nomacs/issues/138>). Four fields cover the
persistable half of `Rules`: `browsing.flag`, `browsing.min_stars`,
`browsing.max_stars`, `browsing.label`. The three text rules — name fragment,
extension list, keyword — stay ephemeral, for the reason §4.8 gives.

And `self.narrowing` is never reset in `open_within` (`src/app/mod.rs:297`), so
the rules follow you to the next card with no control over whether they should
and — because `hides_anything()` is read nowhere but a label inside the bar
itself (`src/ui/filter_bar.rs:296`) and the bar starts hidden
(`src/app/mod.rs:230`) — no sign that they did. Lightroom has the same question
and answers it with two interacting settings, "Lock Filters" and "Remember Each
Source's Filters Separately", whose combination decides one observable behaviour
and neither of which is discoverable from where the behaviour is felt
(<https://www.lightroomqueen.com/community/threads/filters-not-remembered-when-moving-away-from-collection.45428/>).
The lesson is to make it one legible choice, not two.

**Names:** *Start each folder with* (the four rules) and *Keep the filter when
the folder changes* (`browsing.filter_follows_folder`). **Defaults:** everything
shown; and keep the filter — the present behaviour, and the friction Lightroom
users complain about is the *forgetting*, not the remembering. That default is
only defensible once the bar reports itself when closed, which is §8's and
§10's; if that does not ship, this default flips. **Needs both kinds** for the
rules, a setting only for the scope. **Cost:** `Rules` already derives
`PartialEq` and `Clone` (`src/view/narrow.rs:21`) and would need `Serialize`;
the editing widgets exist.

#### What a folder job forgets

The three folder-job modes — bulk rename, shift capture time, group shots — draw
their own sort, their own filter and their own parameters, and every one of those
values is `Default`-ed once in `OrganizeView::new` (`src/view/organize/mod.rs:88-113`),
constructed once per session (`src/app/mod.rs:199`), and written nowhere on exit.
A photographer who renames every card `shoot_{date}_{counter}` types it again on
every card. That is the same defect the browsing sort has above, and the one
JPEGView #216 filed against — "the sorting order keeps getting reset" — with the
difference that this half of the program is not in the table at all, because §4.8
applied its three-way test to the eleven values the browsing views keep and not to
these.

The test settles them, and it does not settle them all the same way.

| What | Where it lives today | Kind |
|---|---|---|
| Rename template | `Options::template`, defaulted to `{name}` (`src/organize/rename/mod.rs:59`, `:73`), typed at `src/view/organize/rename.rs:34-59` | **Session state** |
| Counter start, step, digit width, extension case | The same struct (`src/organize/rename/mod.rs:60-67`, defaulted at `:74-77`), drawn at `src/view/organize/rename.rs:61-81` | **Session state**, with the template they belong to |
| The job's sort key and direction | `SortKey::default()`, `Direction::default()` (`src/organize/sort.rs:14-33`, `:58-63`), drawn at `src/view/organize/controls.rs:41-74` | **Session state** |
| Capture-time offset — days, hours, minutes, seconds, direction | `Offset::default()` (`src/organize/timeshift.rs:22-46`), drawn at `src/view/organize/timeshift.rs:23-50` | **Ephemeral** |
| Which timestamp fields are ticked | `chosen_fields`, empty meaning all (`src/view/organize/mod.rs:53-54`, `:101`) | **Ephemeral** |
| The job's filter — ten fields over seven rules | `Filter::new()` (`src/organize/filter.rs:12-42`), drawn at `src/view/organize/controls.rs:76-154` | **Ephemeral** |

None of the six is a setting, and that is §4.8's first question answered rather
than dodged: a fresh install is not wrong with `{name}`, a counter at 1, four
digits, name order and no filter. What is wrong is the forgetting, which is
§4.8's second question, and the answer to that question is `session.json` — which
costs no field, no page and no row in the settings window, and therefore nothing
against §11.8's budget.

The two ephemeral rows are ephemeral for §4.8's third reason and it is worth
naming, because the rename panel and the shift panel look alike and are not. A
restored rename template shows its whole effect the moment the mode opens: the
panel draws a `Now / Would become` row for every file in the job before anything
is clicked (`src/view/organize/rename.rs:26`). A restored capture-time offset
shows the arithmetic in the same way (`src/view/organize/timeshift.rs:18`) but
not the *reason* — "+1 hour" was true of a card shot on a camera left on winter
time and is false of the next card, and a stored offset is a value with no
visible cause, which is the category §4.8 names. The filter is ephemeral for the
same reason the browsing text rules are, given above: it is per-shoot, and a
folder that opens pre-filtered by something typed a month ago, with a second
control set that matches by different rules from the browsing one (§8.4), is
worse than no memory.

**Cost:** six more values in `Session` and reading them back in
`OrganizeView::new`, which already takes nothing and would take a borrow of the
session. `Options` and `SortKey` already derive `Clone` and `PartialEq`
(`src/organize/rename/mod.rs:57`, `src/organize/sort.rs:13`) and would need
`Serialize`. Nothing on the pages changes.

### 6.8 What new sidecars are called

`path_for` always produces the full-name form — `DSC001.cr2.xmp` — and the doc
comment gives the reason: "The whole file name is kept, so `DSC001.jpg.xmp` and
`DSC001.cr2.xmp` stay apart — a raw and a JPEG of the same frame are different
images with possibly different keywords" (`src/annotations/sidecar.rs:14-18`).
`candidates` reads both forms, most specific first (`:26-29`), and `write` edits
whichever already exists rather than adding a second beside it (`:58-64`). That
is careful work and it covers the reading side properly.

What has no field is which form gets *created* for a photograph that has no
sidecar yet. The source's own note says "Adobe writes `DSC001.xmp` next to a raw
file; darktable and exiftool write `DSC001.cr2.xmp`"
(`src/annotations/sidecar.rs:27-28`). A photographer whose other tool is
Lightroom will find that avis-imgv's ratings are invisible there, and nothing on
screen will explain why. digiKam treats this as important enough to ship an
explicit checkbox, "Sidecar file names are compatible with commercial programs"
(<https://docs.digikam.org/en/setup_application/metadata_settings.html>), and
the pixls.us thread that documents the trap also documents the sharp edge:
where both `IMG0815.xmp` and `IMG0815.CR3.xmp` exist, digiKam reads the former
first and the latter's updates never land
(<https://discuss.pixls.us/t/darktable-xmp-sidecar-tags-not-being-read-by-digikam/32422>).

**Name:** *Name new sidecars*. **Values:** `photo.cr2.xmp` (darktable,
exiftool) · `photo.xmp` (Adobe). Show the two examples rather than the two
vendor lists — the file name is the thing the user can check.
**Default:** `photo.cr2.xmp`, the present behaviour, because it is the only one
of the two that can tell a raw's keywords from its JPEG twin's, which is a
correctness property and not a preference. **Cost:** one enum consulted at
`src/annotations/sidecar.rs:60-64`; `candidates` already reads both, so nothing
downstream changes.

### 6.9 Which operations ask first

One confirmation exists in the whole program. `delete_open_image` puts up a
window when the deletion is permanent or covers more than one photograph, and
carries out a single move to the bin without asking (`src/app/cull.rs:85-88`),
with the reasoning stated: "One photograph to the bin needs no asking, because
the bin is the asking" (`src/app/cull.rs:62-63`). `bin_rejected` always asks
(`:118-122`). None of it is configurable, and nothing else in the program asks
anything — including "Put everything back to the defaults" in the keyboard
editor, which overwrites all sixty-nine rows on one click with no confirmation
and no undo (`src/ui/keys.rs:111-121`; that one is a fault of the editor, and
§9 has it).

The best discussion of this anywhere in the corpus is qimgv #37
(<https://github.com/easymodo/qimgv/issues/37>). **nick-s-b**: "if there's a
modal confirmation for moving to trash, please make it so it can be permanently
disabled." The maintainer proposed hold-to-delete instead — "I plan on going
without modal dialogs altogether (hate that shit). You will hold delete for like
half a second and it will remove the file" — and nick-s-b answered immediately:
"what happens if you hold it for longer than that? Will it start deleting other
files?" **dustysys**: "I agree that deletion should be immediate or the program
will feel unresponsive." The only answer everyone in that thread would have
accepted is *immediate and reversible*. nomacs #480 says the same from the other
end: "mistakes happen and if I hit delete (and after the dialog confirms it) the
file is gone forever" — the point being that a confirmation dialog is not a
substitute for reversibility, because people click through dialogs by reflex
(<https://github.com/nomacs/nomacs/issues/480>).

That settles the shape. avis-imgv already has the undo journal
(`src/organize/journal.rs:97`, `Command::Undo` at `src/app/mod.rs:712`,
`App::undo` at `src/app/cull.rs:496`), so the confirmations it can afford to
switch off are the ones the journal covers, and the one it cannot is permanent
deletion.

**Name:** *Ask before*. **Field:** `cull.confirm`, a small struct of booleans —
moving more than one photograph to the bin · emptying the rejects · undoing a
step that touched more than one file. **Default:** all on. **Never on the
list:** deleting for good, and putting the keyboard back to the defaults;
neither has an inverse anywhere, so both always ask and neither is a setting.
**Cost:** three call sites; two of the three confirmations do not exist yet and
are a window each, of the shape `show_pending_delete` already is
(`src/app/cull.rs:126`).

### 6.10 More than nine destinations

`cull.destinations` is a `Vec<Destination>` with no length limit in the
configuration, and two hard nines in the drawing code: the panel iterates
`.take(9)` (`src/ui/destinations.rs:74`) and the key handler builds a nine-long
digit array (`src/ui/destinations.rs:122-140`). A tenth configured destination
is silently unreachable — not greyed, not scrolled past, not mentioned.

The digit cap is correct and should stay: there are nine digits and the whole
point of the panel is that the digit is the gesture. The *drawing* cap is not.
FastRawViewer's feature-request thread has multiple users asking for predefined
destination folders beyond its single default `_Rejected`, and absolute-path
support for the rejected folder was added in response
(<https://www.fastrawviewer.com/node/33>) — so a photographer with more than
nine buckets is a person who exists. avis-imgv already resolves relative
destinations against the open folder (`src/app/cull.rs:381-398`), which is what
makes a long list useful: `Selects`, `To edit`, `Client A`, `Client B`, `Web`,
`Print`, `Portfolio`, `Family`, `Archive` is nine before anybody has thought
hard.

**Change:** the panel lists every configured destination, scrolls when it does
not fit, and shows a digit beside the first nine and nothing beside the rest,
which are reached with the arrow keys or the mouse. **Default:** the two shipped
(`src/config/defaults.rs:386-397`), unchanged. **Cost:** removing `.take(9)`,
wrapping the list in a `ScrollArea`, and arrow-key movement in
`src/ui/destinations.rs`.

The editing side belongs here too, and it is worse than the cap. "Choose a
folder…" picks an ad-hoc destination, uses it once and discards it
(`src/ui/destinations.rs:97`, `src/app/cull.rs:354-371`), so the panel that
exists to hold destinations is the one place in the program you cannot add one.
That is a control for an existing field, so it is §3.1's and §5's; it is small
work and it belongs with the cap, whenever §12 puts the cap.

While in that file: the digit keys are consumed with `egui::Modifiers::NONE`
(`src/ui/destinations.rs:137`) under a comment that says the opposite — "Any
modifiers, because on a Slovak or German layout the digits are the shifted
characters of the top row" (`:135-136`). The comment names a real problem the
code does not solve, and it is a keyboard defect rather than a missing setting
(§9).

### 6.11 What a launch starts with

#### Which mode, and which folder

`Mode::default()` is `Image` (`src/app/mod.rs:200`), and the only way to start
anywhere else is `--slideshow` on the command line (`src/main.rs:43`).
`--fullscreen` is the same shape (`src/main.rs:44`). There is no `start_in`
field and no start-folder field; without a path argument the viewer opens the
working directory, or the last session's folder when `restore_session` is on
(`src/app/mod.rs:255-285`).

qimgv #203 asks for exactly the first of these: "could there be an option to
open the program in file browser view when the program is opened by running the
.exe? I appreciate that double-clicking an image should open the full-image
view" (<https://github.com/easymodo/qimgv/issues/203>). For a viewer whose
contact sheet is a triage surface rather than a file picker, the ask is stronger
here than it was there: somebody who opens avis-imgv to cull a card wants the
sheet, and somebody who opens it from a file manager wants the photograph they
clicked. Both are true at once, which is what makes it a setting and not a
change of default.

The startup folder is thinner evidence and better reasoning. `restore_session`
covers "where I was", and a photographer who imports to the same directory every
time wants "where I always am" — the two are different, and the second is
currently the working directory of whatever launched the process, which the
source itself calls out: "the working directory of a viewer started from a
desktop icon is nobody's choice" (`src/app/mod.rs:258-259`).

**Names:** *Start in* (`general.start_in`: Image · Gallery · Slideshow,
mirroring `Mode::ALL` (`src/app/mode.rs:28-35`) minus the three folder-job
modes), *Start fullscreen* (`general.start_fullscreen`, the flag made
permanent), and *Start in this folder* (`general.start_folder`, a path, empty
meaning the present behaviour). All three sit beside `general.restore_session`,
which is the same subject. **Defaults:** Image; off; empty. **Precedence,
stated on the row:** a named path on the command line always wins, then the
restored session, then the startup folder, then the working directory — which is
the order `App::new` already implements (`src/app/mod.rs:260-285`) with the
startup folder inserted. **Cost:** three fields and a folder picker; `rfd` is
already a dependency (`Cargo.toml:21`) and the File menu already uses it
(`src/app/settings.rs:19-37`).

#### Which panels are open, and how wide

Five things start hidden, in code, unconfigurably: `menu_visible`,
`side_panel_visible`, `metrics_visible` (`src/app/mod.rs:201-203`),
`tag_panel_visible` (`:212`) and `filter_visible` (`:230`). Only the filmstrip
has a default at all, and it is derived from a height rather than stated
(`src/app/mod.rs:175`, `:234`; `grid_view.filmstrip_height`, zero by default).
The result on a first launch is a window containing a photograph, a grey backdrop
and one thin status bar, with no menu bar and no hint that `F1` produces one —
which is §10's first-run problem as much as it is a missing field.

Panel persistence is the highest-volume theme in the viewer corpus. qimgv #66
and #400 are two independent filings of the same request — pin the thumbnail
panel so it does not autohide (<https://github.com/easymodo/qimgv/issues/66>, six
reactions; <https://github.com/easymodo/qimgv/issues/400>, three). Geeqie #966 is
the more precise version: settings from "Configure this window…" do not persist
while "Other preferences save fine", and a partial persistence failure is worse
than a total one because the user cannot form a model of what will survive
(<https://github.com/BestImageViewer/geeqie/issues/966>).

Widths are half-done. `tags.panel_width` exists and defaults to 260
(`src/config/defaults.rs:351-353`), passed to `SidePanel::default_width`
(`src/ui/tag_panel/mod.rs:64`) — which egui honours only when it has no stored
`PanelState` for that id, since `show_inside_dyn` starts at `default_width` and
then overwrites it from the loaded state
(`egui-0.33.0/src/containers/panel.rs:251-255`). The metadata side panel's width
is the literal `340.` in the panel constructor (`src/app/chrome.rs:110`). Two
panels, one field between them, and because eframe carries no persistence that
`PanelState` dies with the process, so neither remembers a drag.

**Names:** *Open with* (`general.panels_at_start` — the menu bar, the side
panel, the keyword panel and the filter bar) and *Side panel width*
(`general.side_panel_width`, default 340, beside the tag panel width that
already exists). The filmstrip is not in that list: its visibility has to be
split out of a height before it can be a state at all, so it carries its own
field beside the strip's other two, which is 6.6's. **Default:** all panels off,
which is the present behaviour and which the "just show me the picture" audience
is right about; the fix for a blank first launch is a first-run hint (§10), not a
defaulted-open panel. **Needs both kinds** — the setting for a fresh install and
session state so a dragged splitter survives a restart, which is the qimgv and
Geeqie complaint precisely. **Cost:** the two fields, the same values in
`Session`, and reading them in `App::new`.

### 6.12 Language

There is no localisation of any kind: every string is an English literal, the
registry included (`src/config/bindings.rs:85-436`), and no
internationalisation crate is in `Cargo.toml`.

The demand elsewhere is real but thin, and it comes as offers rather than
complaints. qimgv #79, "interface localization", open since 2019, is one line —
"Is there easy way to change interface localization? May be po files for
poedit?" (<https://github.com/easymodo/qimgv/issues/79>) — and what it produced
five years later was a translation somebody made and attached
(<https://github.com/easymodo/qimgv/issues/530>), followed by Japanese
(<https://github.com/easymodo/qimgv/issues/633>). ImageGlass has the same shape
in #230, a Korean translation offered
(<https://github.com/d2phap/ImageGlass/issues/230>). Nobody in this corpus is
blocked by the absence of translations; people who want one write one. It is
therefore sent to §13, with two prerequisites recorded so that nothing done
before then makes it impossible:

- **Bindings must be stored under a stable English identifier.** nomacs stores
  its shortcut settings under the *translated* command name, so changing the
  interface language orphans the user's key map: "Switch language to German …
  Check settings.ini (German settings entry:
  Bild%20an%20Fenster%20anpassen)", and a config that has been through two
  languages holds both entries
  (<https://github.com/nomacs/nomacs/issues/1539>). avis-imgv would walk
  straight into that, because the registry keys on `&'static str` names.
- **`Label::name` must stay English whatever the interface says.** Camera Bits'
  own documentation: "the text label for each class must be exactly the same in
  each app", and if the system language is not English "the label name text in
  the Color Classes section should be manually entered to match the language
  used by the other program"
  (<https://camerabits.freshdesk.com/support/solutions/articles/48000223643-using-star-ratings-and-color-classes-with-adobe-lightroom-and-other-apps>).
  avis-imgv already gets this right (`src/metadata/xmp/mod.rs:126-135`), and it
  is a correctness property, not a display choice.

One localisation problem exists here already and is not solved by translating
anything: `/` drops a pane and is read with `Modifiers::NONE`
(`src/view/image_view/input.rs:116`, `:119`), which on the Slovak, German and
French layouts — where slash is a shifted character — makes it unpressable and
unrebindable. That is a keyboard fix (§9), like the destination digits in 6.10.

### 6.13 Asked for elsewhere, refused here

Each of these is something a reader could reasonably expect to find in this
chapter, and each is listed so that leaving it out reads as a decision rather
than an oversight. Some are sourced requests against a comparable program,
refused here on other grounds; the rest are things this research went looking for
and did not find, which is worth recording too. All of them go to §13, which does
the refusing; the reason column is the evidence for it, kept so the next person
does not have to look again.

| Asked for | Where | Why not here |
|---|---|---|
| Writing marks into the photograph instead of a sidecar | digiKam offers three modes (<https://docs.digikam.org/en/setup_application/metadata_settings.html>) | The source's own reason holds: "Rewriting a photograph to change a star is both slow and risky, and every raw converter already looks for a sidecar" (`src/annotations/sidecar.rs:3-5`). A star that rewrites a 60 MB raw contradicts the one thing this program is for |
| A settings search box | Absent from the *viewer* corpus — nobody asks nomacs, qimgv, ImageGlass or XnView for one, and the nearest thing is XnView's "Basic / Expanded" proposal (<https://newsgroup.xnview.com/viewtopic.php?t=25200>), which is progressive disclosure and not search. In a program that has a preferences window to search it is the best-evidenced request there is: five separate darktable issues (§3.4, §11.8) | Not this chapter's to refuse either way — the window's search is §3.4's, and §3.4 builds it. Recorded because the viewer corpus was searched for it deliberately and it is not there: those users ask for the setting to be where the documentation said it was (<https://newsgroup.xnview.com/viewtopic.php?t=42284>) |
| A "Basic / Advanced" split | Same XnView thread, where the dissent is "Virus scanner tools have introduced such modes a long time ago and actually I think those modes were rather confusing than really helping" | The plain/advanced line is §11.3's. The evidence against it from this chapter's side: the fields divide by subject cleanly and by expertise badly — `raw.source` is the most consequential setting a raw shooter can change and lands in Advanced under any honest rule |
| Per-monitor or per-profile settings | No demand found in the corpus | One global file, one profile (`src/config/load.rs:14-17`), and the foreseeable cost is the Geeqie failure: with several windows open, "depending on which instance I then close last, the wrong settings also get stored" (<https://github.com/BestImageViewer/geeqie/issues/1324>) |
| Configurable clipping and focus-peaking colours, and the peaking threshold | No outside request found | Red `rgb(255,40,40)`, blue `rgb(60,120,255)`, green `rgb(120,255,90)` and a 5 % share (`src/decoder/overlays.rs:57-61`, `:78`). Nothing inside the program disagrees about them either — one constant each, one reader each — so neither limb of 6.1 is met. The threshold is the arguable one, since "the strongest 5 % of gradients" is a judgement about a photograph rather than a colour, and it stays here until somebody argues it |
| Configurable zoom preset percentages | No outside request found | 200/100/75/50/25 in the zoom label's menu (`src/view/image_view/bottom_bar.rs:12`), a conventional list, and the slider beside it already covers 1–1600 % continuously and logarithmically (`:249-250`, `:262-263`) |
| A per-cell badge size | No outside request found | The complaint it would answer is real — `CAPTION_HEIGHT` is a flat 20 points at every cell size (`src/view/grid_view/cell.rs:15`, `:48`) — but it is a layout bug and the fix is to scale with the cell against a floor, not to hand the arithmetic to the user (6.6) |
| A window-size preference separate from the session | JPEGView #276 (<https://github.com/sylikc/jpegview/issues/276>); lximage-qt #122, noting that "Windows image viewers like Irfanview already have this capability too" (<https://github.com/lxqt/lximage-qt/issues/122>) | Already done: the geometry is session state (`src/session.rs:32-51`, applied at `src/main.rs:96-111`) and the window never resizes to the image. Nothing to add |

## 7. Right-click on the thing itself

Microsoft's desktop guidance says that many users right-click regularly and
"expect to find context menus anywhere"
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus>). GNOME's
pointer guidance says the secondary action "should display additional actions
for whatever is being pointed at, typically through a context menu"
(<https://developer.gnome.org/hig/guidelines/pointer-touch.html>).

In avis-imgv there are two `response.context_menu(…)` registrations in 139 files
(`src/actions/user_action.rs:152`, `src/view/image_view/bottom_bar.rs:283`).
Between them they put a menu on three surfaces — the photograph, a contact-sheet
cell and the zoom percentage — of which exactly one draws anything on a fresh
install. The only other reading of the secondary button anywhere is a single
`secondary_clicked()`, on a directory-tree row, and it opens no menu: it opens
the folder (`src/ui/tree.rs:266`). `PointerButton` does not occur in `src/` at
all (§1 has the survey).

This chapter gives the menu for every surface a pointer can land on, one
ordering rule that all of them obey, the code that has to change before any of
them can be drawn, and what happens to the entries a user has already written
into their configuration file. The menus are this chapter's; the settings
window they lead to is §3's, the pages they name are §4's, the control a row
draws is §5's, the routes between surfaces are §8's, and the button itself —
the drag it collides with, the wheel, the double click — is §9's.

---

### 7.1 Where the secondary button goes today

| Surface | Where | What it does |
|---|---|---|
| The photograph | `src/view/image_view/mod.rs:170` → `src/view/image_view/interaction.rs:70-80` | Passes `image_view.context_menu` to `actions::show_context_menu` |
| A contact-sheet cell | `src/view/grid_view/mod.rs:460-464` | Passes `grid_view.context_menu` to the same function |
| The zoom percentage | `src/view/image_view/bottom_bar.rs:275-307` | The only built-in menu in the program: nine entries — Fit to screen / Fill screen / Fit horizontal / Fit vertical, a separator, then 200 / 100 / 75 / 50 / 25 % (`PERCENTAGES`, `:12`) |
| A row of the directory tree | `src/ui/tree.rs:266-268` | No menu — `label.secondary_clicked()` **opens the folder** |

`show_context_menu` begins by returning when the list is empty
(`src/actions/user_action.rs:147-149`), and both lists default to
`default_ctx_menu() -> vec![]` (`src/config/defaults.rs:165-167`, wired at
`src/config/mod.rs:598` and `:638`). On a stock install the first two rows of
that table do nothing: not an empty menu, not a disabled one — no popup is
registered at all.

What the menu can hold is one type: `ContextMenuEntry { description, exec,
callback }` (`src/config/mod.rs:547-552`), an external command line. Rendered as
a flat run of `ui.button(&entry.description)` inside `ui.set_max_width(300.)`
(`src/actions/user_action.rs:152-164`) — no separators, no groups, no checkable
rows, no built-in commands. There is no code path by which a rating, a flag, a
colour label, a keyword, a destination, a stack, a rename, a delete or any of
the 110 settings (§3) can appear on a right-click menu, although the program
performs every one of those things.

The gesture is also contested: `handle_pointer` pans on
`i.pointer.is_decidedly_dragging()` with no button check
(`src/view/image_view/interaction.rs:41-42`), so a right-button drag pans the
photograph and then releases into whatever menu is registered. The conflict is
§9's to resolve; the one line that has to change for these menus to be drawn is
in §7.8.

---

### 7.2 The ordering rule

One rule, stated once, obeyed by every menu below.

```
  1  verbs on this object          most used first, at most seven
     ────────
  2  copy and show                 only where the object is a file on disk
     ────────
  3  the settings that govern      at most four, adjectives not verbs
        this surface
     ────────
  4  More settings… (<page>)       always last, never varies, never removed
```

Where the rule comes from, and where it deliberately departs:

| Rule | Evidence | What is done here |
|---|---|---|
| Twelve rows, and a hard ceiling of fifteen | GNOME: "Menus should contain between three and twelve items" (<https://developer.gnome.org/hig/patterns/controls/menus.html>); NN/g: "fewer than 10–12 items" (<https://www.nngroup.com/articles/contextual-menus/>); Microsoft: "Don't put more than 15 items within a context menu" | No menu below exceeds twelve rows including the last one |
| Groups of seven or fewer, separated | Microsoft, same page: "Organize the menu items into groups of seven or fewer strongly related items… Put separators between the groups" | Four slots, separators only between slots |
| One level, and one exception | Microsoft: "**Avoid using submenus** to keep context menus simple"; GNOME: "**Don't nest submenus**"; Apple: use submenus "with caution and be sure to keep them to one level" (<https://leopard-adc.pepas.com/documentation/UserExperience/Conceptual/AppleHIGuidelines/XHIGMenus/XHIGMenus.html>) | One submenu in the program, which is Apple's caution taken at its word: the five turns behind *Turn* (`src/ui/menus.rs:167-217`), one verb said five ways that would otherwise take five of the twelve rows. Nothing else nests. A setting with more than two values is drawn as an inline control on one row — swatches, a corner grid, a star row — which counts as one item, because the ceiling is about decisions and not about pixels. What the egui defect actually does in 0.33 is in §7.12 |
| Most-used at the top | NN/g: "Place the ones used most often at the top"; GNOME: "Items at the top and bottom of the menu are more noticeable" | Slot 1 is ordered by frequency in a cull, not by the order the source happens to define the commands in |
| Remove rather than disable | Microsoft: "Remove rather than disable context menu items that don't apply to the current context" | A verb that cannot act is absent. The exception Microsoft names — "Always have the commands that complete related sets" — keeps Fit beside Actual pixels even when one of them is already in force |
| The count goes in the label | digiKam labels its own row "Move 2 Files to Trash" (<https://docs.digikam.org/en/menu_descriptions/context_menus.html>) | "Move 24 photographs to the bin", not "Move to the bin" |
| Bold the row a double-click performs | Microsoft: "Display the default command using bold… The default command is invoked when users double-click or select an object and press Enter" | Only two objects have a double-click verb — a grid cell and a tree row — and only they carry a bold row |
| Slot 3 may hold verbs, where the object **is** a setting | GNOME: "Do not mix different types of menu item within each group" | A destination slot, a metadata row and a keyword category are each one element of a stored list, so *Rename this slot…*, *Take this line off the panel* and *Move it up* are settings written as verbs and stay in slot 3. Nothing else in slot 3 is a verb |
| Nothing is reachable **only** by right-click | Microsoft: "Don't make commands only available through context menus"; Apple: "Always ensure that contextual menu items are also available as menu commands"; NN/g agrees | Every verb below already has a key or a menu-bar entry, or gains one in the same commit; every setting below has a home on a settings page, and the last row names it |

Slot 3 draws a field, not a control of its own invention: whatever control §5
gives that field on its settings page is the control the menu row uses, so the
two cannot drift apart. The sketches below show the control §5 is expected to
choose, not a second decision.

The rule has a second half, and it is the one that keeps the map below honest. A
row that sets something needs somewhere to write it, so **slot 3 draws a setting
only where the field exists in `Config` today or is one of the thirty-five §6
proposes.** Six surfaces wanted one that is neither — five rows below, because
two of them want the same thing for the same reason. Each carries a verb or a
plain sentence instead, and none carries a tick that would write nowhere.

| The surface | What it wanted | Why there is no field |
|---|---|---|
| `Filling` (§7.4) | which fit a photograph opens at | Nothing stores one. `Command::Fit` is applied on demand and recorded nowhere (`src/view/image_view/mod.rs:257`) |
| The filmstrip (§7.5) | whether its thumbnails show marks | The strip draws none at all (`src/view/grid_view/filmstrip.rs:101-155`). The repair is to call what a sheet cell already calls — `cell::caption` (`src/view/grid_view/cell.rs:83`) — which is drawing work, not a setting |
| The histogram (§7.6) | which marking mode a photograph opens with | `ImageView::marking` is a private field of the view (`src/view/image_view/mod.rs:71`) and is in no configuration struct |
| The directory tree (§7.6) | whether folders with no pictures directly inside are listed | A silent refusal to repair before it is a preference to offer; §7.6 has the paragraph |
| `Flattened` and `Watching` (§7.4) | the folder-opening default behind each word | `src/config/mod.rs:73-538` holds no field for either. `sc_flatten_dir` and `sc_watch_directory` (`:292-295`) bind the keys and store nothing |

They are recorded here rather than invented here: the register of settings that
ought to exist is §6's, and a menu row that writes to nothing is worse than a
menu one row shorter. This chapter asks for exactly one field of its own, and it
governs the menus themselves (§7.11).

Four departures, stated rather than hidden.

**Delete stays with the verbs, and settings come before it in Microsoft's
canonical order but after it here.** Microsoft's order is primary / secondary /
transfer / object settings / object commands (Delete, Rename) / Properties. The
order built here is verbs, then settings, then the settings page, because the
settings group has to sit next to the "More settings…" row it belongs to — and
the reason for putting settings on a context menu in this program at all is
that forty-seven of the 110 settings are reachable nowhere in the running
program and forty-one cannot be changed at all while it runs (§3). Delete sits
last inside slot 1, never first, never bold. The only delete on any menu is the
one that goes to the platform's bin and is journalled for `Ctrl+Z`
(`src/app/cull.rs:65-91`, `src/app/tagging.rs:232-239`; `default_sc_undo`,
`src/config/defaults.rs:416-418`). "Delete for good" is on no menu at all
(§7.13).

**A bold default row follows Microsoft and departs from Apple.** Apple's rule is
flat — "**Don't set a default item.** If the user opens the menu and closes it
without selecting anything, no action should occur"
(<https://leopard-adc.pepas.com/documentation/UserExperience/Conceptual/AppleHIGuidelines/XHIGMenus/XHIGMenus.html>)
— and the two platforms disagree. This is a program with no macOS-specific
chrome, whose only double-clickable objects are a grid cell and a tree row, and
on those two a bold first row is the only thing that tells anyone the
double-click exists at all, given that `grep -rn "double_clicked" src/` returns
nothing today. The second half of Apple's rule — no action on dismissal — is
honoured everywhere.

**The key is shown, weak and right-aligned, on verb rows.** Microsoft states the
opposite as an explicit exception — "Don't display shortcut key assignments
within context menus… they are optimized for efficiency" — and Apple agrees. Both
wrote that rule for a program whose menu bar is visible and already shows the
accelerator. Here the menu bar starts hidden (`menu_visible: false`,
`src/app/mod.rs:201`) and lists no keys at all (`src/app/panels.rs:26-82`), and
the cheat sheet is a window that any keypress closes
(`src/ui/cheat_sheet.rs:106-121`), so it cannot be held open beside the
photograph. The key comes from the registry the cheat sheet already reads
(`bindings::all()`, `src/config/bindings.rs:85`, rendered by
`ui::keys::describe`, `src/ui/keys.rs:221-230`), so a rebind is reflected and the
menu never lies about a key the way documentation does.

**The menu does not close when a setting is toggled.** Excel's status-bar menu
is the precedent: "The Customize Status Bar popup menu stays open so you can
select and deselect multiple options"
(<https://www.howtogeek.com/247261/how-to-customize-and-use-the-status-bar-in-excel/>).
egui allows it, but not through the call the obvious reading would use. Menus
default to `PopupCloseBehavior::CloseOnClick`
(`egui-0.33.0/src/containers/popup.rs:79-93`, applied by `MenuConfig::default`,
`containers/menu.rs:78-86`), and `Response::context_menu` takes a closure and
nothing else (`response.rs:940`) — there is no parameter for a `MenuConfig`. The
config a menu actually reads is the one `Popup::show` builds from the popup's own
`close_behavior` and stashes in the `UiStack` (`popup.rs:576-585`), so the route
is `Popup::context_menu(&response)` with
`.close_behavior(PopupCloseBehavior::CloseOnClickOutside)` (`popup.rs:248`,
`:326`). `close_behavior` is one flag for the whole popup and cannot say
"these rows close it and those do not", so every slot-1 row calls `ui.close()`
itself (`ui.rs:1272`) — which is what `show_context_menu` already does for the
user's own entries (`src/actions/user_action.rs:161`). Two lines in the shared
renderer, and one line per verb.

---

### 7.3 The photograph, and what is drawn on it

#### The photograph

The panel response is already `Sense::click()` (`src/view/image_view/layout.rs:88`),
so this one costs nothing but the menu.

```
  Fit to the window                       f
  Actual pixels                       Alt + 1
  Fill the window                         m
  Compare with the next one               n
  Move 1 photograph to the bin       Delete
  ────────
  Copy the path
  Copy the picture
  Show it in the file manager
  ────────
  The ground behind it   ■ ■ ■ ■
  Details in the corner  ⠿ (a 3 × 3 grid of dots, off in the middle)
  Enlarge a small picture to fit          ☑
  ────────
  More settings… (The photograph)
```

| Row | Field or command | Where |
|---|---|---|
| Fit / Actual pixels / Fill | `Command::Fit`, `Command::ZoomToPercent(100.0)`, `Command::Fill` | `src/view/image_view/mod.rs:257`, `:271-275`, `:258`; bound at `src/view/image_view/input.rs:81`, `:89`, `:82`; defaults `f`, `Alt + 1`, `m` (`src/config/defaults.rs:168-170`, `:226-228`, `:235-237`) |
| Compare | `sc_compare`, default `n` | `src/config/defaults.rs:261-263` |
| Move to the bin | `Command::Delete` | `src/app/input.rs:34`, `src/app/cull.rs:65-91` |
| Copy the path | new; `Context::copy_text` | `egui-0.33.0/src/context.rs:1534` |
| Copy the picture | new; `Context::copy_image` | `egui-0.33.0/src/context.rs:1543` |
| Show it in the file manager | new; a platform shim | nothing in `src/` does this today, and no crate for it is in `Cargo.toml` (grepped `open::that`, `explorer`, `reveal`, `xdg-open`: none) |
| The ground behind it | `general.backdrop` *(one of §6's thirty-five; it does not exist today)* | hardcoded `Color32::from_rgb(119, 119, 119)`, `src/view/image_view/layout.rs:12`, passed to `layout::show` at `:33` |
| Details in the corner | `image_view.overlay_corner` | `src/config/mod.rs:345`; `Corner::label()` exists at `src/view/image_view/overlay.rs:57-65` and is called from no UI code (grepped: the only callers are its own test) |
| Enlarge a small picture | `image_view.enlarge_to_fit` | `src/config/mod.rs:381` |

"Copy the picture" needs one caveat written down before it is built, because the
obvious implementation is wrong. `store.decoded()` hands back a `DecodedImage`
(`src/cache/store/mod.rs:359-361`) whose `surface` is `width * height * 4` bytes
of RGBA (`src/decoder/mod.rs:30-35`), which wraps into an `egui::ColorImage` with
no conversion — but `full_size` is documented as the size "`surface` may be a
reduction of" (`:69-70`), and the orientation "is left to the GPU, which does it
by sampling the texture in a different order" (`:71-75`). A clipboard copy taken
straight from the cache is therefore liable to be downscaled and lying on its
side.

Decoding the file again is the honest answer, and it is not a one-line one. The
only decode path off the frame thread is the `Loader` pool
(`src/cache/loader.rs:106-133`), so a decode written into a menu handler runs on
the frame thread: with `raw.source` set to develop that is "about a second per
image" by the configuration's own doc comment (`src/config/mod.rs:81-83`), with
the window frozen and nothing on screen. It is the defect §2.8 records against
`crawler::crawl` and the three folder-job applies, reproduced in a new place. So
the row submits the copy through the loader as an ordinary job, says so in the
notice band, and puts the result on the clipboard when it arrives. The cheaper
alternative, if that is too much for a first cut, is to copy what the cache
already holds and label the row *Copy the picture (screen size)*, which is honest
and is less than the plain words promise. Either way it is not the one-line
`ctx.copy_image(…)` the table in §7.12 implies
(`egui-0.33.0/src/context.rs:1543`).

"Copy to Clipboard" is already in the shipped example configuration as a shell
pipeline, `magick {} png:- | wl-copy` (`examples/config.json:44-48`, and the same
entry again for the sheet at `:151-155`). That entry exists because the program
does not do it; egui hands pixels to the clipboard on every platform without
ImageMagick, `wl-copy` or a shell at all, and the paragraph above is about
getting the right pixels rather than about the handing over. Nothing in the
example copies a path or reveals a file — those two rows are new work either
way.

The overlay corner row earns its place twice. `Command::CycleOverlay` writes to
`ImageView::config`, the view's private copy (`src/view/image_view/mod.rs:278-280`),
which is never saved and is overwritten wholesale the next time
`ImageView::set_config` runs (`src/view/image_view/navigate.rs:95-98`, called from
`src/app/settings.rs:91`). A menu row that writes the settings field is the first
route by which the choice survives a restart.

#### The overlay itself

`overlay::show` paints with `ui.painter()` and allocates nothing
(`src/view/image_view/overlay.rs:93-145`; grepping the file for `allocate`,
`Sense` and `response` returns nothing), so the caption over the photograph is
not a widget and cannot be hit-tested. It gets its own rect and
`Sense::click()`, drawn behind the plate so the photograph's own menu still
answers everywhere else.

```
  Move it round the corners               o
  Hide it
  ────────
  Copy what it says
  ────────
  What it says        [ {name} … ]   (opens the template editor)
  How big it is       ──●───   15 pt
  ────────
  More settings… (The photograph)
```

`o` is `sc_overlay` (`src/config/defaults.rs:152-154`). `overlay_format` and
`overlay_text_size` (`src/config/mod.rs:348`, `:350`) appear nowhere in
`src/ui/` or `src/app/` today. Six rows, and the object under the pointer is the
only place in the program where a user can see what the template produced and
change the template in the same gesture.

#### The empty background

`"No images here"` and `"Nothing matches the filter"` are a centred label with no
sense (`src/view/image_view/layout.rs:41-47`; the same pair in the sheet at
`src/view/grid_view/mod.rs:263-271`). The panel behind them does sense clicks
(`layout.rs:88`), so the menu costs nothing. Which menu appears depends on which
of the two strings is on screen, because the second is a dead end with a cause:

```
  — with the store empty —          — with a filter hiding everything —

  Open a folder…                     Clear the filter
  Open files…                        Show everything for a moment      \
  Type a path…              Ctrl+L   Show the filter bar              F3
  ────────                           ────────
  <the last six folders>             Stars 3 to 5, Rejected hidden   (what is on)
  ────────                           ────────
  More settings… (Opening a folder)  More settings… (Opening a folder)
```

The first two rows are the File menu's own verbs (`src/app/panels.rs:33-42`),
which carry no key; the third is the navigator, which does (`sc_navigator`,
`Ctrl+L`, `src/config/defaults.rs:72-74`). The last six folders are already kept
— `Session::positions`, a `VecDeque` of folder-and-photograph pairs capped at 64
(`src/session.rs:64`, `:30`, `:127-141`) — and are read today only to restore a
position, never shown. The filter branch names the rules that are hiding the
photographs, which today are readable only inside a bar that starts hidden
(`filter_visible: false`, `src/app/mod.rs:230`); the sheet has no status bar of
its own (`src/view/grid_view/mod.rs:310`) and `hides_anything()` has exactly one
non-test call site, the count inside that same bar (`src/ui/filter_bar.rs:296`).

---

### 7.4 The bottom bar, field by field

Every element of `bottom_bar::ui` (`src/view/image_view/bottom_bar.rs:105-182`)
is a plain label except the jump field, the slider and the percentage. A label
does not sense clicks, and egui's own documentation names that as the cause of a
dead right-click: "Make sure the widget senses clicks (e.g. `Button` does,
`Label` does not)" (`egui-0.33.0/src/response.rs:925`). There are seven label
call sites in the bar — `:119`, `:138`, `:153`, `:161`, `:193`, `:198`, `:203` —
drawing twelve labels when every flag is on, because `:153` is inside a loop over
six. Two of the seven already build an `egui::Label` and take `.sense(…)`
directly (`:124`, `:163`); the other five are `ui.label(…)`, which returns a
`Response` and has nowhere to put a sense, so each becomes
`ui.add(egui::Label::new(…).sense(Sense::click()))`. The idiom is in this file
already: the zoom percentage is the one label in the bar written that way
(`:280`), and it is the one label in the bar with a working menu.

The bar is a summary that cannot be acted on — its own source says so, "the bar
is a summary, not a control" (`bottom_bar.rs:184-185`). That is defensible for
left-click and is exactly what a right-click is for.

| Field | Where | Menu |
|---|---|---|
| **"go to" box** | `:212-238` | `Go to a number` (focus it) · `Copy this position` · ──── · `The number counts what the filter is showing, from 1` (a sentence, not a control) · ──── · `More settings… (Opening a folder)`. The field is 1-based and bounded by `status.total`, which is the filtered count (`:235-237`, `Status::total` and `Status::hidden` at `:85-89`), and nothing on screen says either. Which numbers the box counts follows from the filter and is not a preference, so the menu states the rule rather than offering to change it |
| **Position counter** | `:119-129` | `Show the filter bar  F3` · `Clear the filter` · `Show everything for a moment  \` · ──── · `Order by:  Name ● Stars ○ Colour label ○ Flag ○` (one inline radio row, `SortBy::ALL`, `src/view/narrow.rs:124-134`) · `Newest first ☐` · ──── · `More settings… (Opening a folder)`. `"7/312 (+18)"` is the only place outside the filter bar that reports a filter is on, and there is no route from it to the bar that produced it (§8) |
| **Stack place** | `:131-142` | `Open this run  e` · `Frame standing for it: back , / forward .` · `Show the whole folder unstacked  Ctrl+G` · ──── · `Longest gap inside one run  ──●──  60 s` · `How alike two frames must be  ──●──  12` · ──── · `More settings… (Opening a folder)`. Its tooltip today says "the key that opens it shows the rest" without naming the key, and carries thirty literal spaces from a wrapped source line (`:140`) |
| **`Flattened`** | `:145` | `Stop reading the sub-folders  Ctrl+F` · ──── · `More settings… (Opening a folder)`. Two rows, because there is no field: `sc_flatten_dir` binds the key (`src/config/mod.rs:292-293`) and the state it flips is stored nowhere (§7.2) |
| **`Watching`** | `:146` | `Stop watching this folder  Ctrl+W` · ──── · `More settings… (Opening a folder)`. The same shape and the same reason (`sc_watch_directory`, `src/config/mod.rs:294-295`) |
| **`Filling`** | `:147` | `Stop filling  Ctrl+M` · `Fit the whole picture  f` · ──── · `Enlarge a small picture to fit ☑` (`image_view.enlarge_to_fit`, `src/config/mod.rs:381`, which decides what filling does to a picture smaller than the window and is drawn nowhere) · ──── · `More settings… (The photograph)`. Which fit a photograph *opens* at is stored nowhere and is not drawn here (§7.2) |
| **`Advancing`** | `:148` | `Stop advancing  Ctrl+Shift+A` · ──── · `Move on after a mark ☑` · ──── · `More settings… (Stars, flags and labels)`. One tick, because one boolean covers a rating, a flag and a label alike (`advances`, `src/app/input.rs:198-200`; `advancing` seeded from `tags.advance_after_marking`, `src/app/mod.rs:176`). This word is the only place that field (`src/config/mod.rs:169`) is visible anywhere |
| **`Comparing`** | `:149` | `Leave the comparison  Escape` · `Drop this pane  /` · ──── · `Panes side by side  ●───── 2` (`nr_images_shown`, `src/config/mod.rs:368`, clamped to 8 by `MAX_IMAGES_SHOWN`, `src/view/image_view/mod.rs:51`) · ──── · `More settings… (The photograph)`. `Escape` and `/` are hard-coded (`src/view/image_view/input.rs:116-117`), appear in no cheat sheet, and `/` is unreachable on a layout where slash is shifted |
| **`RAW+JPEG`** | `:150` | `Show both ● / Show the JPEG ○ / Show the raw ○` · ──── · `More settings… (Raw files)`. `Prefer::ALL`, its three labels and the sentence explaining each are written at `src/organize/pairs.rs:34-53` and drawn nowhere; this word is the only place `raw.pair_with_jpeg` (`src/config/mod.rs:80`) is visible |
| **Marks strip** | `:186-205` | see below |
| **File name** | `:159-164` | `Copy the name` · `Copy the path` · `Show it in the file manager` · ──── · `What this line says  [ $(#File Name#)… ]` (`image_view.name_format`, `src/config/mod.rs:383`, rendered at `src/view/image_view/mod.rs:661-673`) · ──── · `More settings… (The photograph)`. Truncated with `.truncate()` and no `on_hover_text`, so a long name is unrecoverable from this screen — while the metadata panel two feet away does give long values a hover (`src/app/panels.rs:161-162`) |
| **Zoom slider** | `:252-273` | `Fit  f` · `Actual pixels  Alt+1` · ──── · `200 % · 100 % · 75 % · 50 % · 25 %` (one inline row) · ──── · `More settings… (The photograph)`. Its only label is the emoji `🔎` (`:265`) and it shows no value (`.show_value(false)`, `:264`) |
| **Zoom percentage** | `:275-307` | The menu that exists, kept, with `Copy the magnification` added and `More settings… (The photograph)` appended. It is one of only two `egui::Slider`s in the program (§5) and the only working context menu, and nothing signals it exists |

#### The star / flag / colour strip

Three read-only glyphs at `bottom_bar.rs:186-205`. The colour swatch has a
tooltip (`:199`); the flag and the stars do not. The keyword panel forty pixels
to the left draws the identical glyphs as buttons
(`src/ui/tag_panel/mod.rs:101-126` for the stars, the flag and colour rows
following at `:130` and `:154`).

```
  ☆ ☆ ☆ ☆ ☆                    1 – 5        (inline, click to set, click again to clear)
  ⚑ Keep      ✖ Reject      no flag   p x u
  ■ ■ ■ ■ ■                    6 – 9, then Ctrl + 9
  ────────
  Show the keyword panel                 k
  ────────
  Move on after a mark                   ☑
  ────────
  More settings… (Stars, flags and labels)
```

The strip that reports a rating becomes the strip that sets one, and it names
the panel that does the rest — which is the "and what do I do about it?" rule
§8 states in general. Setting a rating with a mouse is possible today in exactly
one place, the keyword panel's star row (`src/ui/tag_panel/mod.rs:101-126`), and
reaching it requires already knowing `k` (`src/config/defaults.rs:355-357`).
The colour keys run `6`, `7`, `8`, `9`, `Ctrl+9`, because the row runs out at
nine (`default_sc_label`, `src/config/defaults.rs:455-463`).

Rating and colour label are canonically context-menu items in a photo tool:
digiKam's thumbnail menu carries "Assign Labels ‣ Pick / Color / Rating"
(<https://docs.digikam.org/en/menu_descriptions/context_menus.html>). Photo
Mechanic exposes the colour class three ways — a click on the visible label
under the thumbnail, the `Image` menu, and a number key
(<https://docs.camerabits.com/support/solutions/articles/48001252564-color-class-ratings>),
which is the redundancy Apple, Microsoft and NN/g all ask for.

---

### 7.5 The contact sheet, the stacks and the strips

#### One cell

`handle_cell_interaction` (`src/view/grid_view/mod.rs:429-465`) already has the
response and already calls `show_context_menu` with an empty list.

```
  Open                                       Enter      (bold — the double-click verb)
  Pick it out                                Space
  ☆ ☆ ☆ ☆ ☆     ⚑ ✖     ■ ■ ■ ■ ■                       (three inline rows)
  Send it to…                              Alt + M
  Move 1 photograph to the bin              Delete
  ────────
  Copy the path
  Show it in the file manager
  ────────
  Thumbnails across    ──●────────  5
  Under each thumbnail  nothing ○ marks ● marks and a caption ○
  ────────
  More settings… (The contact sheet)
```

Twelve rows, at the ceiling; anything added later displaces something. `Enter`
already opens the cell under the cursor (`src/view/grid_view/mod.rs:605`) and
`Space` picks it out (`sc_select`, `src/config/defaults.rs:312-314`). `Open` is
bold because it becomes the double-click verb: `grep -rn "double_clicked" src/`
returns nothing today, and a plain click opens the photograph and leaves the
sheet (`grid_view/mod.rs:455-457` → `src/app/views.rs:84-87`), which is why the
only mouse routes into a selection are Ctrl-click and Shift-click (`:451-454`) —
the two gestures every file manager reserves for adding to a selection and
extending one.

The two settings rows are not idle. `images_per_row` (`src/config/mod.rs:439`) is
the *starting* column count and is never written back at runtime: `+`, `-` and a
pinch or Ctrl-wheel zoom (`grid_view/mod.rs:516-528`) all go through
`set_columns` (`:643-646`), which touches `GridView::columns` and nothing else,
and nothing in the grid calls `Config::save` (grepped: the only callers are
`src/app/settings.rs:105` and `src/config/load.rs:101`). `Badges` has three
states, is cycled by `Ctrl+I` (`src/view/grid_view/cell.rs:36`,
`default_sc_cycle_badges`, `src/config/defaults.rs:373-375`), has no field of its
own in `GridViewConfig` (`src/config/mod.rs:437-487`) and resets on every launch;
the field it needs is §6's.

One repair comes with the menu: the clickable rect is allocated only inside the
branch that has a texture (`grid_view/mod.rs:389-394`; the placeholder branch
returns `None` at `:381-384`), so a cell that has not decoded — or that failed
and shows `✖` — cannot be right-clicked, left-clicked or hovered for its name,
because the hover lives on the same line as the click (`:394`). The rect is
allocated in both branches, and the menu drawn over a placeholder omits the verbs
that need pixels and keeps the rest.

#### Several selected cells

Right-clicking inside a selection acts on the selection and does not collapse
it; right-clicking outside one moves the cursor first. The count is in the
label, as digiKam writes it.

```
  Rate 24 photographs      ☆ ☆ ☆ ☆ ☆
  Flag 24                  ⚑ ✖
  Colour 24                ■ ■ ■ ■ ■
  Send 24 somewhere…                       Alt + M
  Move 24 photographs to the bin            Delete
  Drop the selection                        Escape
  ────────
  Copy 24 paths
  ────────
  Marking a set never advances                        (a sentence, not a control)
  ────────
  More settings… (Stars, flags and labels)
```

Every one of those verbs already reads the selection through one funnel,
`App::marked_paths` (`src/app/tagging.rs:277-283`), and every one is journalled
as a single undo step however many frames it touched (`:232-239`). Taking a whole
selection to the bin asks first (`src/app/cull.rs:85-88`), which the menu does not
change. The sentence at the foot is not decoration: advance is suppressed while a
selection exists (`src/app/mod.rs:811-812`) and nothing on screen says so.

The selection counter itself — `"{n} selected · Escape to clear"`
(`grid_view/mod.rs:329`) — is an `Area` built with `.interactable(false)`
(`:322`), so it cannot carry a menu until that line changes. It gets one: `Drop
the selection` · `Pick out everything  Ctrl+A` · ──── · `Invert` · ──── · `More
settings… (The contact sheet)`.

#### The stack badge

`cell::stack` paints with `ui.painter()` throughout and has no `Sense`
(`src/view/grid_view/cell.rs:190-228`), so the plate reading `❏ 17` is not a
widget: the one thing on the cell that says a burst is hiding behind it is the
one thing on the cell that cannot be pointed at. It gets its own rect in front
of the cell's.

```
  Open this run                                  e     (bold — double-click opens it too)
  Frame standing for it, back / forward         , .
  Step to the previous run / the next    Ctrl + ← →
  Fold every run  ·  Open every run
  Rate all 17            ☆ ☆ ☆ ☆ ☆
  ────────
  Longest gap inside one run   ──●──   60 s
  How alike two frames must be ──●──   12
  ────────
  More settings… (Opening a folder)
```

"Rate all 17" is on this menu because of a trap: a collapsed stack contributes
only its standing frame to `visible` (`src/view/stacks.rs:193-227`), so `Ctrl+A`
over a stacked folder selects one frame per burst and a rating reaches one frame
while the badge says seventeen. The badge is the right place to offer the other
reading of itself. The four glyphs are drawn on the cell already
(`grid_view/mod.rs:403-411` calling `stacks::glyph`, `src/view/stacks.rs:362-369`)
and nothing anywhere says that `◐` is an HDR bracket, `◎` a focus stack, `⏱` a
timelapse and `❏` a plain series; that legend belongs on the same rect's hover,
which §10 writes.

#### The filmstrip, and a thumbnail in it

The strip is a `TopBottomPanel::bottom` with `exact_height`
(`src/view/grid_view/filmstrip.rs:63-67`); its cells are `Sense::click()` and
only `clicked()` and `hovered()` are read (`:108`, `:150`, `:154`).
Right-clicking the strip's background rather than a thumbnail:

```
  Hide the strip                        Ctrl + T
  ────────
  How tall it is        ──●────   96 pt
  Which edge it sits on   bottom ● top ○
  ────────
  More settings… (The contact sheet)
```

(`grid_view.filmstrip_edge` is one of §6's thirty-five, fixed at
`src/view/grid_view/filmstrip.rs:63` as `TopBottomPanel::bottom`.)

That menu answers the program's worst dead key. `filmstrip_height` defaults to
`0.0` (`src/config/defaults.rs:299-301`) and `show_filmstrip` returns
immediately at that height (`src/app/views.rs:137-141`) while `Ctrl+T` still
flips the flag (`src/app/mod.rs:713`), so a key listed in the keyboard editor
(`src/config/bindings.rs:100-105`) and in the cheat sheet does nothing at all and
says nothing. Once the height is a control rather than a JSON field the key
works; until then the menu cannot be reached either, which is why this one row
also belongs on the contact sheet's settings page (§4) and in the startup check
(§3). `filmstrip_height` is also the one field that is half restart-bound — the
height is read live every frame, whether the strip comes up is decided once at
`src/app/mod.rs:175` (§3).

A thumbnail in the strip gets the cell menu of §7.5 minus the sheet-only rows —
Open, the three mark rows, Send it to…, the bin, a separator, Copy the path, a
separator, `How tall it is`, `Which edge it sits on`, then `More settings… (The
contact sheet)`. The strip draws no stars, no flag, no colour, no reject dimming,
no name and no tooltip (`filmstrip.rs:101-155`), so during a cull it is the one
thing beside the photograph that cannot report what has been decided about its
neighbours. The menu is the shortest route to *setting* a mark from the strip. It
is not the repair for the strip's silence, and no tick on it can be: the sheet
cell already draws exactly those marks from the same data (`cell::caption`,
`src/view/grid_view/cell.rs:83-137`, called at `grid_view/mod.rs:413`), and the
strip's `draw_cell` simply does not call it. That is drawing work, not a setting,
and §7.2's table records it as one of the six this map wanted and does not draw.

---

### 7.6 The bars and the panels

#### The filter bar

The bar is one `horizontal_wrapped` (`src/ui/filter_bar.rs:29-60`) holding
fifteen interactive controls when stacking is on and twelve when it is off,
alongside seven static labels. Right-clicking its background, between the chips:

```
  Clear the filter
  Show everything for a moment                 \
  Hide the bar                                F3
  ────────
  Start every folder like this                       (writes the browsing fields)
  ────────
  More settings… (Opening a folder)
```

`Rules` has seven fields — `min_stars`, `max_stars`, `flag`, `label`,
`name_contains`, `extensions`, `keyword` (`src/view/narrow.rs:22-34`) — and none
of them is stored anywhere between runs. The fields that would store them are
`browsing.min_stars`, `browsing.max_stars`, `browsing.flag` and `browsing.label`
(§6 argues for them; §4 puts them on *Opening a folder*). Microsoft's toolbar
guidance is the closest published precedent for a menu on a bar: "For
customizable toolbars, display the context menu for customizing the toolbar… For
other toolbars, do nothing"
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-toolbars>). This
bar is customisable in the sense that matters — what it starts as — so it gets
one.

Each chip gets its own, and each is short because a chip governs one rule:

| Chip | Where | Menu |
|---|---|---|
| `Stars … to …` | `:160-176` | `Any rating` · `Unrated only` · `3 and up` · ──── · `At least this many stars to start with  ●─── 0` · ──── · `More settings… (Opening a folder)`. Two `DragValue`s today, min and max, with tooltips and no preset |
| flag combo | `:178-192` | The five `FlagRule` values as inline radios (`src/view/narrow.rs:62-79`) · ──── · `Show these flags in every folder to start with ☑` (a tick that stores what is set) · ──── · `More settings… (Opening a folder)` |
| label combo | `:194-227` | The five colours of `Label::CHOICES` (`src/metadata/xmp/mod.rs:118-124`) plus `No label` and `Any label` as swatches · ──── · `More settings… (Stars, flags and labels)` |
| `Name contains` | `:229-247` | `Clear this box` · `Paste` · ──── · `Matches the file name only, anywhere in it, ignoring case` (a sentence) · ──── · `More settings… (Opening a folder)` |
| `Keyword` | `:229-247` | `Clear this box` · ──── · `<the keywords seen in this folder>` as a picked list · ──── · `More settings… (Keywords)`. The program already knows the folder's vocabulary (`AnnotationStore::known_tags`, used at `src/app/tagging.rs:181-195`) and the imported catalog (`src/config/mod.rs:128-138`) and offers neither |
| `Types` | `:229-247` | `Clear this box` · ──── · `<the extensions present in this folder>` as ticks · ──── · `More settings… (Opening a folder)` |
| `Show everything` | `:280-291` | `Put the rules back` · ──── · `More settings… (Opening a folder)` |
| `Stacks` toggle | `:95-101` | `Fold all` · `Open all` · ──── · `Show the folder stacked to start with ☑` (`browsing.stack_by_default`, §6) · `Smallest run worth stacking  ●── 2` · ──── · `More settings… (Opening a folder)`. `Settings::min_frames` (`src/organize/group/mod.rs:42`, default 2 at `:50`) is reachable in Group-shots mode (`src/view/organize/group/mod.rs:71-73`) and nowhere in the browsing views |
| `Gap` / `Alike` | `:129-155` | `Put it back to 60 s` / `to 12` · ──── · `Remember this as the default ☑` · ──── · `More settings… (Opening a folder)`. Neither is saved today, and the same two settings have different ranges in the two places they appear — 1–600 s and 0–32 here (`:134`, `:147`), 1–3600 s and 0–64 in Group shots (`src/view/organize/group/mod.rs:60`, `:65`). One range, decided in §3's validation table, applies to both |
| `Clear` | `:49-57` | `Clear the rules` · `Clear the rules and the order` · ──── · `More settings… (Opening a folder)`. Today `Clear` leaves `sort` and `descending` untouched (`:54-56`), so a folder left ordered by stars stays that way with the bar reporting the plain total |
| count label | `:295-306` | Same menu as the position counter in §7.4 |

#### The sort control

`Order by` plus the `▲`/`▼` button (`src/ui/filter_bar.rs:249-278`) is the only
sort control anywhere the photographs are, and it is behind `F3` with no
shortcut of its own (`src/config/bindings.rs:214-225` registers only "Filter" and
"Show everything").

```
  Name  ●     Stars  ○     Colour label  ○     Flag  ○      (inline radios, SortBy::ALL)
  Reverse it
  ────────
  Sort every folder this way to start with           ☑
  ────────
  More settings… (Opening a folder)
```

The four values come from `SortBy::ALL` and `label()`
(`src/view/narrow.rs:124-134`); the tick writes `browsing.sort` and
`browsing.descending` (§6). Capture time is not among the four, although
`SortKey::Captured` exists and is offered by the folder-job modes
(`src/organize/sort.rs:23`, `:36-43`) — a missing value rather than a missing
menu, and this menu is where its absence gets noticed.

#### The right-hand side panel

`SidePanel::right("image_metadata")` (`src/app/chrome.rs:106-127`) holds three
blocks whose only interaction is a hover: the metadata values carry
`on_hover_text` (`src/app/panels.rs:161-162`), the histogram plot senses hover
and its two figures carry tooltips (`src/ui/histogram.rs:43`, `:131`), and the
cache lines carry nothing. Nothing in the panel answers a click of either button.
The panel background:

```
  Hide this panel                            i
  ────────
  Copy everything here
  ────────
  What this panel lists…                            (opens the tag picker)
  How wide it is        ──●────   340 pt
  ────────
  More settings… (The window)
```

`i` is `sc_toggle_side_panel` (`src/config/defaults.rs:208-210`).
`general.metadata_tags` (`src/config/mod.rs:270`) decides the rows and a
misspelled tag is skipped in silence (`src/app/panels.rs:152-154`). The panel's
width is resizable and never persisted (`chrome.rs:107`, `:110`); eframe is built
without the `persistence` feature (`Cargo.toml:13`) and `App` implements no
`save()`, so the drag is lost on exit. The field that would keep it is
`general.side_panel_width` (§6).

**One metadata row** (`src/app/panels.rs:156-163`) is the strongest case in the
program for a menu on a row, because the row is the only thing that knows its own
tag name:

```
  Copy the value
  Copy "Lens Model: RF 24-70mm F2.8 L IS USM"
  ────────
  Take this line off the panel
  Move it up  ·  Move it down
  ────────
  More settings… (The window)
```

("Lens Model" is one of the thirteen tags in `default_metadata_tags`,
`src/config/defaults.rs:113-127`.) Right-clicking a list's own headings to choose
which of them are shown is a settled pattern in data grids — AG Grid documents it
as the column menu (<https://www.ag-grid.com/javascript-data-grid/column-menu/>)
— and it is the shape people already expect from a list of named fields.

#### The histogram

`histogram::show` allocates its plot with `Sense::hover()`
(`src/ui/histogram.rs:43`) and draws `Blown %` / `Crushed %` as tooltipped labels
(`:108-134`). The numbers say how much is clipped and there is no route from them
to the overlay that says *where*, which is a separate key, `c`
(`src/config/defaults.rs:156-158`).

```
  Mark what has clipped                      c
  Mark what is in focus                      c   (the same key, one step further round)
  Take the marks off                         c
  ────────
  Copy the numbers
  ────────
  More settings… (The photograph)
```

Three verbs on one key, which is the point: `c` cycles Off → Clipping → Focus
peaking and back (`Overlay::next`, `src/decoder/overlays.rs:43-49`), the three
names are already written and reach no screen (`Overlay::ALL` and `label()`,
`:33-41`, whose only callers are tests), and nothing tells anyone which one comes
next. The menu is the first place the cycle is spelled out. There is no settings
group, because
there is nothing to put in it: the marking mode lives only in `ImageView::marking`
(`src/view/image_view/mod.rs:71`), is in no configuration struct, is reported by
no status flag, and resets every launch; §6 does not ask for a field and this
chapter does not draw a control for one (§7.2). `Marks::is_showing()` exists and
is called only by a test (`src/view/image_view/marks.rs:75-77`, `:88`).

#### The cache readout

`panels::cache_stats` (`src/app/panels.rs:168-196`) prints how full every tier is
and every budget behind those numbers is JSON-only (`CacheConfig`,
`src/config/mod.rs:233-261`). This is the clearest case in the program of a
readout with its controls removed.

```
  Empty the caches and read the folder again
  ────────
  Copy this readout
  ────────
  Decoded pictures in memory   ──●────   4096 MB
  Pictures on the graphics card ─●────   1024 MB
  ────────
  More settings… (Speed and memory)
```

Two sliders, at the two defaults that matter (`default_ram_budget_mb`,
`default_gpu_budget_mb`, `src/config/defaults.rs:11-13`, `:38-40`); the other
four fields of `CacheConfig` are behind the last row. Both are Rebuild fields in
§3's sense — they commit when the drag ends, not on every frame — because a
budget only reaches a store through `StoreConfig` (`src/app/stores.rs:32-66`),
which `ImageStore::new` takes by value (`src/cache/store/mod.rs:117`) and which is
built only when a view is constructed (`src/view/image_view/mod.rs:111`,
`src/view/grid_view/mod.rs:76`). `set_config` (`src/app/settings.rs:91-92`)
replaces a view's configuration and not its store, so re-seeding is what a change
costs.

The frame-timings strip above the window (`src/ui/perf_metrics.rs:74-81`, drawn
at `src/app/mod.rs:822-826`) gets the shortest menu in the program: `Hide this
strip  F10` · ──── · `Copy these figures` · ──── · `More settings… (Speed and
memory)`. `F10` is hard-coded (`src/app/input.rs:102-104`), is in no registry and
appears in no cheat sheet, so the first row is the only place the key is ever
written down.

#### The keyword panel

`SidePanel::left("tag_panel")` (`src/ui/tag_panel/mod.rs:61-66`). Several of its
`selectable_label`s carry a tooltip (`:142`, `:170`, `:196`, `:285`), but none of
them carries a menu: `grep -rn "context_menu\|secondary" src/ui/` returns exactly
one line, `tree.rs:266`.

The panel background:

```
  Hide this panel                            k
  ────────
  Read the keyword list again
  ────────
  How wide it is       ──●────   260 pt
  How many recent keywords to keep   ●──   12
  ────────
  More settings… (Keywords)
```

(`tag_panel_width` 260 and `recent_tags` 12 are the shipped defaults,
`src/config/defaults.rs:351-353`, `:347-349`.) `Catalog::configured` runs once,
at startup (`src/app/mod.rs:209`); nothing rebuilds it, so editing the keyword
file means restarting the viewer, and a file that cannot be read is a
`tracing::warn!` that never reaches the notices
(`src/annotations/catalog.rs:53-62`). "Read the keyword list again" is a verb
that does not exist yet and is two lines.

**A tag chip** — either the offered kind (`:266`, `:284`) or the on-image kind
(`:194-200`):

```
  Add it to this photograph        (or: Take it off this photograph)
  Add it to all 24
  ────────
  Show me everything tagged this
  ────────
  Copy the whole path      Places|Slovakia|Tatras
  ────────
  More settings… (Keywords)
```

Two of those rows repair real traps. A single left-click on an on-image chip
*removes* the tag with no confirmation and no separate hit target for the `×`
(`:195-199`) — the only warning is a tooltip reading "Remove" (`:196`) — and
removal takes every hierarchy path ending in that leaf with it
(`src/annotations/mod.rs:183-196`). The menu names the action instead of
performing it on a mis-click. "Show me everything tagged this" is the `ShowOnly`
verb §8 defines, and the panel has never had it: the only actions it can emit are
`SetRating` / `SetFlag` / `SetLabel` / `AddTag` / `RemoveTag`
(`src/ui/tag_panel/mod.rs:31-39`), so clicking a keyword can never narrow the
folder to it, although the filter bar's `Keyword` box does exactly that
(`src/view/narrow.rs:265-275`).

**A category heading** is `ui.label(RichText::new(title).strong())` at
`src/ui/tag_panel/mod.rs:258` and needs a sense before it can carry anything:

```
  Add every keyword under this to the photograph
  ────────
  Rename this group…
  Add a keyword to it…
  ────────
  More settings… (Keywords)
```

`tags.categories` is a JSON array (`src/config/mod.rs:128`) and no `Action` the
panel can emit reaches it — the panel writes keywords to the annotation store and
the recent list and nothing else (`src/app/tagging.rs:153-162`, `:172`). digiKam
gives every one of its sidebars a menu of its own, the Tags list included
(<https://docs.digikam.org/en/menu_descriptions/context_menus.html>); this is the
same idea at a smaller scale.

#### A destination slot

`destinations::ui` draws up to nine slots as buttons (`src/ui/destinations.rs:57-103`,
`take(9)` at `:74`); two ship (`default_destinations`,
`src/config/defaults.rs:386-397`). The panel is one of the five windows (§1.5)
and the only one of the five that gets a menu, because it is the only one whose
rows are stored settings that nothing else can reach. §7.13 refuses the other
four, the keyboard editor included, and gives a separate reason for each.

```
  Move here                                  3
  Copy here
  ────────
  Show this folder in the file manager
  ────────
  Rename this slot…
  Point it somewhere else…
  Move it up  ·  Move it down
  ────────
  More settings… (Moving and deleting)
```

`take(9)` silently drops a tenth destination, and "Choose a folder…" (`:97`)
picks an ad-hoc folder that is used once and thrown away
(`src/app/cull.rs:354-370`). The menu on the ad-hoc row therefore also carries
**Remember this folder as slot 3**, naming whichever slot is next free, which is
the missing half of the feature.

#### The directory tree, and a folder in it

This one starts with a repair. `label.clicked()` expands and collapses,
`label.secondary_clicked()` **opens the folder** (`src/ui/tree.rs:262-268`), and
there is no tooltip, no menu and no hint text — the window says only "Directory
Tree" (`:222`). Opening on double-click is what Microsoft's default-command rule
assumes (§7.2), and the secondary button cannot both open a folder and show a
menu. Taking the gesture back costs whoever has learned it, and that is the price
of a gesture that was never the secondary button's to have.

After: double-click and `Enter` open (`Enter` already does, `:303-307`), single
click still expands, and right-click gives a menu.

```
  Open                                              (bold — the double-click verb)
  Read it and its sub-folders as one       Ctrl + F
  ────────
  Copy the path
  Show it in the file manager
  ────────
  More settings… (Opening a folder)
```

Five rows and no settings group, because the one thing this surface would have
put in a settings group turns out on inspection to be a repair.
`get_selected_path` returns `None` unless `utils::is_valid_path` finds a
supported file *directly* inside (`src/ui/tree.rs:339-345`,
`src/utils.rs:54-67`), so opening a parent folder you meant to flatten does
nothing and says nothing — the same test gates the navigator's `Enter`
(`src/ui/navigator.rs:83-89`). Offering "hide folders with no pictures directly
inside" as a preference would be dressing that silence up as a choice. Either
the folder opens and is flattened, or `Open` is absent on a folder that cannot
be opened — §7.2's remove-rather-than-disable rule — with a sentence in its
place saying there are no pictures directly inside this one. Both are repairs
and neither needs a field. The silent refusal is the only outcome the menu takes
away.

The tree's own background gets: `Collapse everything` · `Show this folder in the
file manager` · ──── · `More settings… (Opening a folder)`. It is the cheapest
menu in the chapter: the tree is an `Area` that is already `.interactable(true)`
and not movable (`src/ui/tree.rs:207-212`), and an `Area` in that state hands
back a response already sensing clicks
(`egui-0.33.0/src/containers/area.rs:506-513`), so the menu hangs off the
response `show` already returns and there is no sense to add.

---

### 7.7 The chrome

#### The menu bar

`TopBottomPanel::top("menu")` with three `menu_button`s — File, Mode and
Settings, eleven items between them and no Help (`src/app/panels.rs:29-77`; §1
has the item-by-item list, and the Help menu that becomes the fourth is §10's).
Microsoft's rule for chrome is explicit: "For customizable toolbars, display the
context menu for customizing the toolbar… Provide a context menu with the
following commands: a check box list to display the available toolbars,
Lock/Unlock toolbars, Customize…"
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-toolbars>). Excel's
status bar is the same pattern at the other end of the window.

```
  ☑ Menu bar                                F1
  ☐ Filter bar                              F3
  ☐ Photograph details                       i
  ☐ Keywords                                 k
  ☐ Strip of thumbnails                Ctrl + T
  ☐ Frame timings                          F10
  ────────
  Which of these come up at startup…
  Text size          ──●────   125 %
  ────────
  More settings… (The window)
```

The menu stays open while the ticks are clicked, which is the Excel behaviour
quoted in §7.2. Five of the six flags start `false` in code and have no
configuration field at all (`src/app/mod.rs:201-203`, `:212`, `:230`). The sixth
is the exception that proves the row is needed: `filmstrip_visible` is seeded
from `settings.grid_view.filmstrip_height > 0.0` (`:175`, `:234`), so one panel
already remembers whether it should come up and the other five do not, by
accident rather than by decision. "Which of these come up at startup…" writes
`general.panels_at_start`, which does not exist (§6) and belongs beside
`general.restore_session` rather than in a section of its own. `Text size` is
`general.text_scaling`, which defaults to 1.25 (`src/config/defaults.rs:49-51`)
and is one of the twenty-five wholly restart-bound fields until §3's repair
lands.

#### The mode indicator

There is no mode indicator. `Mode::label()` returns "Image", "Gallery", "Bulk
rename", "Shift capture time", "Group shots", "Slideshow"
(`src/app/mode.rs:37-46`) and is drawn in exactly three places: the radio list in
the hidden menu bar (`src/app/panels.rs:56-65`), the cheat sheet's title
(`src/ui/cheat_sheet.rs:70`), and the heading of a folder-job panel
(`src/view/organize/mod.rs:149`). In the image view and the contact sheet, which
is where people spend their time, nothing says which mode is on screen; `F2`
cycles all six (`src/config/defaults.rs:60-62`) and three of the six draw no
photographs.

So the indicator is added — one word at the left end of the bottom bar, before
the position counter — and it carries the menu:

```
  Image  ●     Gallery  ○     Bulk rename  ○
  Shift capture time  ○     Group shots  ○     Slideshow  ○
  ────────
  Start in this mode                        ☑
  ────────
  More settings… (Opening a folder)
```

Six inline radios over two lines, from `Mode::ALL` (`src/app/mode.rs:28-35`),
which is the same list the menu bar already renders; the tick writes
`general.start_in` (§6). In the folder-job modes the word is drawn in the panel
heading that already exists (`ui.heading(mode.label())`,
`src/view/organize/mod.rs:149`) and the menu hangs off that instead.

#### The folder-job panels

Three of the six modes draw no photographs — Bulk rename, Shift capture time and
Group shots (`src/app/mode.rs:28-35`) — and between them they are the densest
settings surfaces in the program. Twelve of the seventeen `DragValue`s and four
of the seven `ComboBox`es in the whole of `src/` are inside
`src/view/organize/`: more numeric and choice controls than the rest of the
program put together (§5 has the counts). The one `checkbox` in the program is
there too (`src/view/organize/timeshift.rs:71`). None of those controls carries a
menu, and the sort-and-filter block shared by all three modes carries no tooltip
either — it is one of the twenty-eight files that draw an interface and explain
nothing (§10.1).

The rule here is different from everywhere else in this chapter, and it is worth
saying rather than leaving to be inferred. On the photograph or the contact sheet
a setting is hidden and the menu is the first place it appears at all; on a
folder-job panel the control is already on screen. So the menu's job is not to
expose the setting but to say what it means, put it back, and name the page where
the same value would be remembered between runs — which is why slot 1 usually
carries a reset here and slot 3 is often empty.

| Surface | Where | Menu |
|---|---|---|
| The sort row | `src/view/organize/controls.rs:41-74` | `Reverse it` · `Put the order back to Name` · ──── · `Sort by:  Name ● Capture time ○ Type ○ Size ○ Rating ○ Sharpness ○ Other metadata… ○` (one inline row, `SortKey::CHOICES` and `label()`, `src/organize/sort.rs:36-56`) · ──── · `More settings… (Opening a folder)`. This dropdown is the only place in the program that offers Capture time or Sharpness as an order; the filter bar over the photographs offers four values and neither of those (§7.6) |
| The filter block | `controls.rs:76-155` | `Clear the filter` · `Close it` · ──── · `A file has to pass every rule that is filled in` (a sentence, not a control; the ten fields of `Filter`, `src/organize/filter.rs:12-31`) · ──── · `More settings… (Opening a folder)`. A `Clear` button exists already but only appears once the filter is non-empty (`controls.rs:86-88`), so the menu is the only route that is in the same place every time |
| The grouping row | `src/view/organize/group/mod.rs:53-89` | `Group them again now` · ──── · `Longest gap inside one run  ──●──  60 s` · `How alike two frames must be  ──●──  12` · `Smallest run worth keeping  ●──  2` · `Thumbnails:  names only ○ small ○ medium ● large ○` (`thumbnails::SIZES`, `src/view/organize/thumbnails.rs:21-26`) · ──── · `More settings… (Opening a folder)` |
| One group's kind | `group/mod.rs:139-145` | `HDR bracket ○  Focus stack ○  Timelapse ○  Series ●` (one inline row, `Kind::ALL` and `label()`, `src/organize/group/classify.rs:31-40`) · ──── · `Not a group` (the button at `:151-157`) · ──── · `More settings… (Opening a folder)`. The sentence explaining each of the four is already written, as a doc comment on the variant (`classify.rs:19-26`), and reaches no screen; §10 puts it on the hover |
| The template box | `src/view/organize/rename.rs:34-59` | `Insert a placeholder…` · `Clear it` · ──── · `Copy the first name this would produce` · ──── · `More settings… (Opening a folder)`. `Insert…` is a `menu_button` beside the box already (`:44-57`); the right button reaches the same list without moving the pointer off the box being edited |
| The counter and extension row | `rename.rs:61-80` | `Put the counter back to 1, step 1, 4 digits` (the shipped defaults, `src/organize/rename/mod.rs:71-79`) · ──── · `Extension:  keep as it is ● lowercase ○ UPPERCASE ○` (`Extension::CHOICES`, `:36-45`) · ──── · `More settings… (Opening a folder)` |
| The offset row | `src/view/organize/timeshift.rs:23-50` | `Put it back to zero` · `Turn it round — forward ↔ back` · ──── · `Forward when the photographs were taken later than the camera thought` (the sentence already written at `:46-49`, where it is a hover on a row of four numbers) · ──── · `More settings… (Opening a folder)` |
| The timestamp ticks | `timeshift.rs:52-80` | `Tick them all` · `Untick them all` · ──── · `Nothing ticked means every timestamp these files carry` (a sentence; the shorthand is at `:67-69` and is stated nowhere on screen) · ──── · `More settings… (Opening a folder)` |

Three of those rows are the same three settings twice over, and that is the
argument for the table rather than an accident of it. `group.max_gap`,
`group.tolerance` and `group.min_frames` are §6's fields for the thresholds fixed
at `src/organize/group/mod.rs:45-53`; §7.5 puts two of them on the stack badge on
the contact sheet, and §7.6 puts the same two on the filter bar's `Gap` and
`Alike` chips and the third on its `Stacks` toggle. Without this table they would
be right-clickable everywhere except in the mode that exists to set them, which
is the reverse of what anyone would guess. The same three carry the range
disagreement
§7.6 records — 1–3600 s and 0–64 here (`group/mod.rs:60`, `:65`), 1–600 s and
0–32 in the filter bar (`src/ui/filter_bar.rs:134`, `:147`) — and one range,
decided in §3's validation table, applies to both.

---

### 7.8 What has to change before any of this can be drawn

None of the menus above are expensive. What is expensive is that most of the
surfaces are not widgets — and the surfaces that are not widgets include every
panel this chapter hangs a menu on, not only the labels inside them.

A panel's returned response carries the child `Ui`'s own sense, and `UiBuilder`
leaves that at `Sense::hover()` (`egui-0.33.0/src/ui_builder.rs:158-167`; the
response is built with `sense: self.sense` at `ui.rs:1209`, and no panel sets
one — `panel.rs:281`, `:389`). That is exactly why the program's own central
panel has to write `.interact(Sense::click())` on the response it gets back
(`src/view/image_view/layout.rs:88`), the line §7.3 counts as costing nothing.
Five panels have never had that line written, and with two bare labels and a
heading beside them, eight of the menus above cannot open until they do. The
directory tree is the one exception and needs nothing at all (§7.6).

| Change | Where | Why |
|---|---|---|
| `.sense(Sense::click())` on seven label call sites in the bottom bar | `src/view/image_view/bottom_bar.rs:119`, `:138`, `:153`, `:161`, `:193`, `:198`, `:203` | egui: "Make sure the widget senses clicks… `Label` does not" (`egui-0.33.0/src/response.rs:925`). This is the commonest cause of a dead right-click in an egui program. Five of the seven are `ui.label(…)` and become `ui.add(Label::new(…).sense(…))`; two build a `Label` already (§7.4) |
| `UiBuilder::sense(Sense::click())`, or `.interact(Sense::click())` on the returned response, for five panels | the filter bar `src/ui/filter_bar.rs:24-26`; the metadata side panel `src/app/chrome.rs:106`; the keyword panel `src/ui/tag_panel/mod.rs:61`; the filmstrip `src/view/grid_view/filmstrip.rs:63`; the menu bar `src/app/panels.rs:29` | Every "right-click the background of this panel" menu in §7.5, §7.6 and §7.7 depends on it. Without it the panel answers a hover and nothing else |
| A sense on the filter bar's count label and on the frame-timings line | `src/ui/filter_bar.rs:304`, `src/ui/perf_metrics.rs:75` | Bare `ui.label` and `ui.monospace` calls, taking the same treatment as the bottom bar's five |
| A sense on the folder-job panel heading | `src/view/organize/mod.rs:149` | `ui.heading(mode.label())`, which the mode indicator's menu hangs off in the three folder modes (§7.7) |
| A rect for the overlay | `src/view/image_view/overlay.rs:93-145` | Painter-only today |
| A rect for the stack badge | `src/view/grid_view/cell.rs:190-228` | Painter-only today |
| A rect for the caption strip under a cell | `src/view/grid_view/mod.rs:367-370` | The response covers `picture` only (`:379`), so the strip that draws the stars is outside every hit test |
| Allocate the cell rect in both branches | `src/view/grid_view/mod.rs:381-394` | A cell that has not decoded, or has failed, is dead to the mouse and to the hover |
| `.interactable(true)` on the selection counter | `src/view/grid_view/mod.rs:322` | The one piece of chrome that reports a selection cannot be acted on |
| `Sense::click()` on the metadata rows and the cache lines | `src/app/panels.rs:156-163`, `:173-195` | Labels |
| `Sense::click()` on the histogram plot and its two figures | `src/ui/histogram.rs:43`, `:108-134` | `Sense::hover()` on the plot; the two figures are plain labels |
| A button check on the pan | `src/view/image_view/interaction.rs:41-42` | A right-button drag pans and then releases into the menu, because `is_decidedly_dragging()` is asked with no button check at all. Panning becomes whichever button `mouse.drag` names — any button by default, which is the present behaviour stated honestly (§6.5). The field is §6's and the behaviour §9's |
| Double-click opens; right-click gives a menu | `src/ui/tree.rs:262-268` | The gesture is currently inverted, and nothing on screen says so |
| A double-click handler at all | nowhere in `src/` — `grep -rn "double_clicked" src/` returns nothing | Two menus bold a row that a double-click performs; the row would be a lie without it |

The menus themselves are one function per surface returning a list of rows, and
one shared renderer that draws the four slots, the separators, the weak key
column and the last row. The settings rows are registry rows — the same table §3
builds — so a menu row and a settings page row are the same declaration rendered
twice, and neither can drift from the other. Which of these lands when is §12's.

---

### 7.9 How anyone finds out the gesture exists

NN/g is blunt about this: gesture-revealed menus "are not discoverable and have
still not become standard", and the recommendation is to "include visual elements
in the UI to indicate that a contextual menu is available"
(<https://www.nngroup.com/articles/contextual-menus/>). Microsoft says the same
in different words — context menus are for advanced users unless you add an
affordance, and suggests a drop-down arrow where you need to reach everyone
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus>).

**One affordance, everywhere or nowhere.** Every surface that carries a menu
gets the same six-point chevron on hover, in the weak text colour, and the last
four words of its hover text are always *"Right-click for more."* Some surfaces
marked and some not is worse than none marked, and the program is already in that
state — 33 hovers in 11 of its 139 files (§10) — so a hover that says nothing is
the normal experience and a hover that says something is a surprise. The
four-word clause is the whole of what this chapter writes; the wording of the
rest of the hover, the evidence for the rule and the policy behind both are
§10's.

**Three other places say it once each, and none of the three is drawn here.**
The cheat sheet gains one sentence beside "These are the keys as configured. Any
key closes this." (`src/ui/cheat_sheet.rs:99`) naming the gesture — one line in a
window a beginner already opens, adjacent to generated code, costing nothing
(§10). The empty state says it once: `"No images here"` is the first thing most
people see, because with no arguments the crawler reads the working directory,
and the home screen it becomes (§7.3) ends with a quiet line naming `Ctrl+,` and
the right-click. And a first-run notice, which is §10's to word, arrives with two
constraints from this chapter. It cannot be recorded in `Session`, which is only
written when `general.restore_session` is on (`on_exit`,
`src/app/mod.rs:866-872`), so somebody who has turned that off would be greeted
every launch; the condition that is free is `fetch_cfg` taking the
`ErrorKind::NotFound` branch and writing a default file (`src/config/load.rs:51`,
`:67-88`), recorded in a one-line marker file beside the configuration. And it
cannot use the notice band as that band stands: `Notices` draws on a dark-red
popup frame (`src/ui/notice.rs:91`) that reads as an error, and is
`.interactable(false)` (`:80`), so the band needs a second, unalarming style
before anything friendly can be said in it.

---

### 7.10 The keyboard route

Shift+F10 and the Menu key are the standard keyboard equivalents of right-click
on Windows, and WCAG 2.1.1 (Level A) requires all functionality to be
keyboard-operable. (Both of those are well-established convention rather than
something the research could cite to a primary specification; the notes flag the
same gap.) A trackpad user with no secondary click has the same problem from the
other direction: Picview documents "two-finger tap: equivalent to right-click"
(<https://picview.chitaner.com/blog/mouse_keyboard_trackpad/>), which works only
where the platform provides it.

egui supplies nothing for this. What it does supply is the parts needed to build
it:

- `Response::secondary_clicked()` already returns true if "the widget was
  pressed-and-held on a touch screen" (`egui-0.33.0/src/response.rs:174-180`), so
  touch long-press is handled without any work.
- `Popup::context_menu(response)` is `Popup::menu(response)` with
  `.at_pointer_fixed()` (`egui-0.33.0/src/containers/popup.rs:248-260`), and
  `Popup::menu` on its own anchors to the widget's rect (`:237-243`, via
  `from_response`, `:217-224`). A menu opened from the keyboard therefore uses
  `Popup::menu(&response).open_memory(SetOpenCommand::Bool(true))` — both are
  public (`:308`, `:96`) — and appears at the object, which is the correct place
  when there is no pointer.

**Shift+F10 only. The dedicated Menu key is not reachable.** egui 0.33 has no
`Key::ContextMenu` variant — the enum runs `F1` through `F35`
(`egui-0.33.0/src/data/key.rs:151`, `:185`) and the printable keys, and grepping
that file for `Menu` returns nothing, so it has no name for the key between the
right Alt and the right Control. Reading it would mean a patched winit
translation or a raw platform event, which is not worth it for a key that is
absent from most laptop keyboards. Shift+F10 is the route, and it is registered
like any other binding so it can be rebound. This is the reason the other
chapters name Shift+F10 and stop.

**It collides with F10 today, and the collision is invisible.** The
frame-timings strip is toggled by `ctx.input(|i| i.key_pressed(egui::Key::F10))`
(`src/app/input.rs:102-104`), and `key_pressed` counts key events without
looking at modifiers at all (`egui-0.33.0/src/input_state/mod.rs:814-832`), so
Shift+F10 already toggles the metrics strip. The same is true of `?`
(`src/app/input.rs:113-115`).

Moving both to `consume_key` is necessary but not sufficient, and it is worth
being exact about why. `consume_key` matches modifiers with
`Modifiers::matches_logically`, which returns true when the pattern has no Shift
and the event does (`egui-0.33.0/src/data/input.rs:804-813`), so
`consume_key(Modifiers::NONE, F10)` still fires on Shift+F10. What fixes it is
the ordering egui's own documentation prescribes — "you should match most
specific shortcuts first" (`input_state/mod.rs:785-789`): the context-menu
binding consumes `Modifiers::SHIFT + F10` before either of the other two is
asked. It is the same defect that exact-modifier matching already removed from
the configurable bindings, surviving in the two places that bypass the shortcut
system.

The route itself: `App` records which surface last had the keyboard cursor — the
image view's focused pane, the sheet's cursor cell, the keyword panel's selected
row, the tree's `selected_index` (`src/ui/tree.rs:242`) — and the key sets
`open_context_for(surface)`. Each surface, drawing itself that frame, sees its
own id in that field and opens its popup with `Popup::menu` instead of waiting
for a secondary click. Arrow keys and `Enter` inside an egui menu already work.

Two consequences worth writing down. The keyboard route must not be blocked by
`utils::are_inputs_muted` (`src/utils.rs:41-47`) the way every other shortcut is,
or the menu becomes unreachable the moment a text box has focus — and that
function treats *any* focused widget as mute (`:46`), which in this program is
often. And `Escape` closes the menu before it does anything else, because
`Escape` is already overloaded: it surrenders focus (`src/app/mod.rs:802-804`),
leaves a comparison (`src/view/image_view/input.rs:117`) and clears the selection
(`src/view/grid_view/mod.rs:626-633`), and a menu that swallows one press is
better than a menu that dismisses and drops a selection in the same keystroke.

---

### 7.11 The entries the user already has

`context_menu` today means "external programs to put on the right-click menu",
and after this chapter it means exactly the same thing. No stored value changes
and `migrate::CURRENT` (`src/config/migrate.rs:25`) is not bumped: a migration
step exists to fix a default that *moved* (`src/config/migrate.rs:1-18`), and
nothing has moved. What changes is where the entries are drawn.

**The user's entries are their own group, and the program never reorders,
renames or removes one.** They sit after slot 1 and before the settings group,
separated on both sides, on the two surfaces whose fields they are —
`image_view.context_menu` on the photograph (`src/config/mod.rs:387`) and
`grid_view.context_menu` on a cell (`:457`).

```
  <the built-in verbs>
  ────────
  Copy to Clipboard                                (the user's own entries,
  Delete                                            in the user's own order)
  ────────
  <copy and show>
  ────────
  <the settings>
  ────────
  More settings… (The photograph)
```

Three things follow, and each is a decision rather than an accident.

**The menu gets longer, and that is the complaint ImageGlass received.** "There
are too many options in the right-click menu, some functions I can't use, and
sometimes I even click accidentally"
(<https://github.com/d2phap/ImageGlass/issues/1342>) — the accidental-click point
matters more than the length. So: the ceiling of twelve counts the user's rows
too, and where a user has enough entries to breach it, theirs are kept and the
*settings* group is dropped first, with the "More settings…" row surviving. The
user's list is the part they chose; the program's part yields.

**One route out, and only one field.** `general.menus_show_settings_rows`,
default on, on the *Keys and mouse* page (§4) beside the two program lists:
turning it off leaves the verbs, the user's entries, the copy group and the last
row, so nothing becomes unreachable — the settings window is still one click away
from every menu. It is the one field this chapter asks for on its own account,
and it goes in `general` rather than in a `menus` section of its own, because a
section holding one boolean is a migration and a heading bought for nothing;
§4.9's placement table is where it lands, beside the other fields that describe
the chrome rather than a view. There is no line in the source to name as its
present value, because no menu in the program draws a settings row at all; the
nearest thing is the early return that decides whether a menu is drawn in the
first place (`src/actions/user_action.rs:147-149`). That is the whole of the
configurability offered for the built-in rows, and
the reason is Microsoft's: "Provide a good default configuration. Users shouldn't
have to customize their toolbars for common scenarios. Don't depend upon users
customizing their way out of a bad initial configuration"
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-toolbars>). The
darktable request for a fully configurable image menu was closed as not planned
(<https://github.com/darktable-org/darktable/issues/14857>) — its reporter did
want the list configurable, "slightly different for everyone", and asked only
that destructive actions be kept off it. The judgement to curate instead is this
plan's, not theirs, and it rests on the length rules in §7.2 rather than on that
issue; §11 makes the same argument about settings in general.

**One collision is real and is flagged, not fixed.** The shipped example
configuration contains an entry described `"Delete"` whose command moves the file
to `$HOME/trash` (`examples/config.json:49-53`, and the same entry again for the
sheet at `:156-160`). Anybody who copied that example now has a row called
Delete two rows below a built-in row called "Move 1 photograph to the bin", doing
something different. The program does not silently rename or drop it. It becomes
one row in the startup check §3 specifies for the other silent failures, naming
the field path — `image_view.context_menu[1]` — saying that a built-in row has
the same name and that two rows on the same menu will read as the same command,
and offering to rename it, remove it or leave it. Checked on load; does nothing
until it is asked.

**Two rows disappear from most people's configurations of their own accord**, and
should be pointed out rather than migrated away: "Copy to Clipboard" and the
trash entry are now built in, on every platform, without ImageMagick, `wl-copy`
or `notify-send`. The same two pipelines are also in the example's `user_actions`
list (`examples/config.json:23-42`), so somebody who copied it has each of them
twice over. They are deleted on the *Keys and mouse* page, where §5 specifies the
editor for the two context-menu lists; the one requirement this chapter adds to
that editor is a Test button, which answers the question the source itself asks
in a comment, `//Show toast with result?` (`src/actions/user_action.rs:90`).

---

### 7.12 What egui gives, and what it does not

Measured against the pinned version: `eframe` 0.33.0 with the `wgpu` feature
added to its defaults, which is where egui comes from — egui is not a dependency
in its own right (`Cargo.toml:13`; `grep -n egui Cargo.toml` returns nothing).
The line reads `features = ["wgpu"]` and does not set `default-features =
false`, so the build is `wgpu` *plus* eframe's whole default set — accesskit,
default_fonts, glow, wayland, web_screen_reader, winit/default, x11
(`eframe-0.33.0/Cargo.toml`, `[features] default`). What this plan draws from the
line is unaffected: `persistence` is not among those defaults and is not asked
for either, which is why the side panel forgets its width (§7.6).

| Wanted | State |
|---|---|
| A menu on a right-click | `Response::context_menu` (`egui-0.33.0/src/response.rs:940`) — **but** the thing it hangs off must sense clicks, which the docs state at `:925`. That is why seven label call sites in the bottom bar and five whole panels need a line changed before any of this draws (§7.8) |
| Touch long-press | Free: `secondary_clicked()` "also returns true if the widget was pressed-and-held on a touch screen" (`response.rs:174-180`) |
| Submenus | `SubMenuButton` exists (`containers/menu.rs:341-396`), popups try alternative alignments before giving up (`Popup::get_best_align`, `containers/popup.rs:465-486`) and the fallbacks can be named outright (`align_alternatives`, `:289`). Used once, for the turns. egui #5251 reports a submenu near the right screen edge covering its parent (<https://github.com/emilk/egui/issues/5251>), and the metadata panel sits against that edge (`src/app/chrome.rs:106`) — but 0.33 tries `RectAlign::symmetries` before `MENU_ALIGNS` (`emath-0.33.0/src/rect_align.rs:118-132`, `:234-236`), so a `RIGHT_START` submenu with nowhere to go folds to `LEFT_START` and opens on the far side of its parent instead. Checked against the right edge of a 1920-wide window: the five turns opened to the left of the menu, whole and covering nothing |
| A menu that stays open while ticks are clicked | Yes, but not through `Response::context_menu`, which takes a closure and nothing else (`response.rs:940`). The route is `Popup::context_menu(&response).close_behavior(PopupCloseBehavior::CloseOnClickOutside).show(…)` (`containers/popup.rs:248`, `:326`), because the `MenuConfig` a menu reads is the one `Popup::show` builds from the popup's own close behaviour (`popup.rs:576-585`). One flag for the whole popup, so slot-1 rows call `ui.close()` themselves (`ui.rs:1272`) |
| Arbitrary widgets inside a menu | Free: the closure is a `Ui`, so swatches, a star row and a 3 × 3 corner grid all work without a new widget |
| A menu opened at a widget rather than the pointer | `Popup::menu(&response)` (`containers/popup.rs:237-243`); `Popup::context_menu` is the same thing forced `.at_pointer_fixed()` (`:248-260`). This is what the keyboard route uses |
| Copy text and copy an image | `Context::copy_text` and `Context::copy_image` (`context.rs:1534`, `:1543`). `copy_image` takes a `ColorImage`, which the cached `Surface` (`src/decoder/mod.rs:30-35`) wraps into without conversion — but see the caveat in §7.3 about size, orientation, and which thread does the decoding |
| A bold "default" row | Nothing built in; `Button::new(RichText::new(…).strong())` is the whole implementation |
| Checkmark and radio menu items | No first-class menu widget; `ui.checkbox` and `ui.radio_value` inside the closure. Both already appear in this program — `radio_value` in the slideshow window (`src/app/panels.rs:111`), `checkbox` in the capture-time view (`src/view/organize/timeshift.rs:71`) — outside the nineteen numeric controls §5 counts, which are the seventeen `DragValue`s and the two `Slider`s and nothing else |
| A keyboard route to a context menu | Nothing, and no `Key::ContextMenu` in the key enum either (`egui-0.33.0/src/data/key.rs`), so the Menu key cannot be read at all. §7.10 is roughly forty lines of our own, on Shift+F10 |
| A context menu on a drag source | egui #7390 reports `context_menu()` no longer opening on a `dnd_drag_source` since 0.32 (<https://github.com/emilk/egui/issues/7390>); not verified against 0.33 here. Moot today: `grep -rn "drag_started\|is_being_dragged\|dnd_" src/` returns nothing, so it matters only if the sheet or the strip ever becomes draggable |

---

### 7.13 What is deliberately not on any menu

These are the menu map's own exclusions. What the plan as a whole declines to
build is §13's list, and the two entries below marked for it go there in one line
each.

**"Delete for good."** `Shift+Delete` exists and confirms
(`src/app/cull.rs:65-91`), and it stays a key. The darktable reporter who asked
for a configurable image menu asked specifically that destructive actions be kept
off it (<https://github.com/darktable-org/darktable/issues/14857>), and a
permanent delete two pixels from "Copy the path" is the mis-click ImageGlass's
reporter described.

**Anything a menu would be the only route to.** Every verb in this chapter has a
key or a menu-bar entry; every setting has a page. §7.2 has the rule and the
three sources behind it. It is also the practical answer: a trackpad without a
secondary click, a keyboard-only session and a screen reader all arrive at the
same requirement.

**Submenus.** Argued in §7.2 and §7.12. One of them, for the turns; everywhere
else a menu that wants five choices draws them inline.

**A menu editor** (§13). One boolean (§7.11), and nothing else. digiKam's
thumbnail menu runs past fifty entries with submenus
(<https://docs.digikam.org/en/menu_descriptions/context_menus.html>) and is the
counter-example rather than the model; every published guideline puts the ceiling
at a dozen, and a menu whose contents each user assembles cannot be documented,
cannot be taught by the cheat sheet, and cannot be relied on by the rest of this
plan.

**A "More…" overflow row** (§13). Windows 11's "Show more options" is the
most-complained-about context menu in current use, and Microsoft's own account of
the problem is that the menu "appear[s] cluttered with a long list of actions,
something that has been bothering users for a long time"
(<https://www.windowslatest.com/2025/11/06/microsoft-admits-windows-11s-right-click-menu-is-cluttered-confirms-fix-with-a-new-ui-feature/>).
An overflow that is hit on almost every invocation is worse than a longer menu.
The twelve-row ceiling is enforced by dropping rows, not by hiding them behind
one.

**A menu on the notice band, the cheat sheet, the slideshow window and the
delete confirmation** — for four different reasons, which is worth spelling out
rather than covering with one word. Microsoft sanctions a dead right-click
explicitly ("For other toolbars, do nothing"), and a right-click that does
nothing is not automatically a defect; a right-click on the thing that carries a
setting and does nothing is. So:

- **The notice band.** It fades after six seconds (`src/ui/notice.rs:16`), carries
  no setting, and is built `.interactable(false)` (`:80`) — it would have to stop
  being so before it could answer any button at all. §10 wants it clickable for a
  different reason; if that lands, this refusal is reconsidered with it.
- **The delete confirmation.** Genuinely transient, and carrying one decision
  that the two buttons already state (`src/app/cull.rs:137`).
- **The cheat sheet.** Not transient and not a settings surface: it is the
  program's reference window (§1.5), and any keypress closes it
  (`src/ui/cheat_sheet.rs:106-121`), which is a defect §10 owns rather than one a
  menu repairs. Under §3.3 every row in it already opens the *Keys and mouse*
  page with that row armed, and a row one click from its setting does not need a
  menu as well.
- **The slideshow window.** Refused not because it is transient — it is one of
  the two windows that *are* the settings surface today, and the only place three
  of the 110 settings can be reached at all (§1.5, §3.1) — but because §3.3
  retires it. `Settings ▸ Slideshow…` becomes a deep link to the *Slideshow*
  page, at which point there is no window left to right-click, and the rows on
  that page carry the settings window's own row menu.

**A menu on the keyboard editor's rows** — for the last of those reasons, and it
needs saying, because the editor is the largest settings surface in the program:
sixty-nine rows over sixty fields (§3.1), and the only window a user opens *in
order to change a setting*. §9.5 gives its rows what they are missing — a
per-row reset, an unbind, a scope that tells the truth about clashes — as glyphs
and keys rather than as a menu, and §3.3 turns the window itself into the *Keys
and mouse* page. Once it is a page, its rows are settings-window rows and inherit
that window's row menu: **Show me where this is** and **Copy setting name**
(§3.4, §12). Building a second, separate menu on `src/ui/keys.rs` in the
meantime would be building it twice.

## 8. Getting from one thing to the thing next to it

The viewer is full of true statements. `7/312 (+18)`.
`series 2 · frame 4 of 17 · stack 2 of 9` (`view/stacks.rs:83-93`).
`Blown 3.4%`. `Images: 812/2030 in RAM • 256 on GPU`. `RAW+JPEG`. `Advancing`.
`3 selected · Escape to clear`. `Places|Slovakia|Tatras`. Every one of them is the
answer to a question the user asked, and almost none of them is a place they can
act from. The position counter is an `egui::Label`
(`view/image_view/bottom_bar.rs:119-129`). The stack sentence is an `egui::Label`
(`bottom_bar.rs:131-142`). The clipping percentages are `egui::Label`s
(`ui/histogram.rs:124-131`). The selection count and the notices are drawn inside
the only two `Area`s in `src/` built with `.interactable(false)`
(`view/grid_view/mod.rs:320-322`, `ui/notice.rs:78-82`). `Sense::click()` appears
four times in the whole of `src/` — a grid cell (`view/grid_view/mod.rs:391`), a
filmstrip cell (`view/grid_view/filmstrip.rs:108`), the zoom readout
(`bottom_bar.rs:280`) and the central panel (`view/image_view/layout.rs:88`) — and
egui is explicit that this is the gate: "Make sure the widget senses clicks (e.g.
`Button` does, `Label` does not)" (`egui-0.33.0/src/response.rs:925`).

This chapter takes every pair of features that a photographer would expect to be
one click apart, asks whether a route exists, and says what it should be. The
method was to read the source for each pair rather than to reason about it: the
tables below cite the line that either carries the route or is the place the
route should have been. The rule they all come from is §8.10; the four things
that have to be built are §8.11.

Two facts frame the whole thing.

The first is that **the program builds exactly one context menu of its own**, on
the zoom percentage label — nine entries, four fit commands and five
magnifications (`bottom_bar.rs:275-307`). There are two `context_menu()`
registrations, three surfaces that register a menu, one of which draws anything on
a fresh install, and one `secondary_clicked()`, on a directory-tree row, where it
opens the folder (`ui/tree.rs:266-268`). §7 owns that map and everything that has
to change before a menu can be drawn at all. What matters here is the shape of
what exists: a `ContextMenuEntry` can only ever hold an external shell command
(`config/mod.rs:547-552`), both configured lists default to `vec![]`
(`config/defaults.rs:165-167`), and `show_context_menu` returns before registering
anything when the list is empty (`actions/user_action.rs:147-149`). The single
mechanism that every other program uses to link one thing to another is, here, an
external-program launcher that ships switched off.

The second is that **the command vocabulary has no verb for either direction of
travel**. There are two `Command` enums — thirty-one variants for the application
(`app/input.rs:12-64`) and twenty-five for the image view
(`view/image_view/input.rs:25-67`), fifty-six commands between them. They are
toggles, marks, moves, zooms and mode changes, and not one of them means "narrow
the collection to this value" or "take me to where this thing lives". Every
cross-link below would have to be invented at its own call site today, which is
why none of them exists.

### 8.1 Between the two views, and between a frame and its run

| From | To | Route today | Where | What the route should be |
|---|---|---|---|---|
| The photograph | the contact sheet, at this photograph | **Yes.** `set_mode(Grid)` calls `grid_view.focus_on(image_view.selected_index())`, reached by `Backspace`, `F2`, or Mode in the menu bar | `app/views.rs:27-29`, `config/defaults.rs:53-55`, `app/mod.rs:687-692`, `app/panels.rs:56-65` | Keep. Add the mouse half: the `7/312` counter opens the sheet at this frame |
| The contact sheet | the photograph under the cursor | **Yes.** A plain click sets `selected`, which the app consumes into `select_path` + `set_mode(Image)`; `Enter` does the same from the keyboard | `view/grid_view/mod.rs:455-457`, `app/views.rs:84-87`, `grid_view/mod.rs:605-607` | Keep the verb. Which button and how many clicks carry it is §9's |
| The photograph | its stack, opened | **Yes, by key only.** `E` runs `toggle_stack`, which reads `cursor_index()` and so works from either view — and turns stacking on first if it is off | `config/defaults.rs:186-188`, `app/mod.rs:477-493`, `:519-523` | Also from the two places the stack is *shown*: the bar's `series 2 · frame 4 of 17 · stack 2 of 9` becomes a button that opens and closes the run (`view/stacks.rs:83-93`), and the cell's count plate becomes a widget |
| The stack place in the status bar | opening that stack | **No.** A label whose hover refers to "the key that opens it" without naming it | `bottom_bar.rs:131-142` | Click opens or closes. Naming the key in the hover is §10's rule to write |
| The stack plate on a grid cell | opening that stack | **No.** `cell::stack` is `ui.painter()` throughout, with no `Sense` | `view/grid_view/cell.rs:190-228` | The plate becomes a control. `Ui::interact(rect, id, Sense::click())` takes `&self` in egui 0.33 (`egui-0.33.0/src/ui.rs:1131`), so this needs no change to `cell::stack`'s signature |
| A stack glyph (`◐ ◎ ⏱ ❏`) | what kind of run it is | **No.** The glyph is drawn on the plate and nothing on screen says what it means | `view/grid_view/mod.rs:407`, `view/stacks.rs:362-369` | Hover names the kind, and the menu offers to re-read the run as another kind. The words already exist as `Kind::label()` and only the organiser uses them (`organize/group/classify.rs:33-40`, `view/organize/group/mod.rs:139-145`) |
| The photograph, or its stack | Group shots, with that run selected | **No.** `set_mode` hands the organiser `all_paths()` — the whole collection, both halves of every pair — and there is no `scroll_to` anywhere in `view/organize/` | `app/views.rs:35-38`, `app/mod.rs:395-397`, `view/organize/group/mod.rs:123-133` | `Reveal(Group of this frame)`: enter Group shots, expand that group, scroll to it |
| A burst the sheet is showing folded | the group the organiser would make of it | **No, and the two can disagree.** Two independent `group::Settings` values, tuned by two control sets that do not offer the same ranges | `app/stacking.rs:28` and `ui/filter_bar.rs:129-155` vs `view/organize/mod.rs:56` and `view/organize/group/mod.rs:53-89` | One setting, one answer, two views of it |

The pattern in this group is that the *verb* exists and is bound to a key, while
the *noun* is drawn as paint. `E` opens a stack; the two places on screen that say
a stack is there cannot.

The last row deserves its numbers, because they are the argument. Both settings
start at the same defaults — gap 60 s, tolerance 12, minimum 2 frames
(`organize/group/mod.rs:45-51`) — so a user who touches neither will never see the
disagreement. Touch either and the controls do not span the same space: the filter
bar offers a gap of 1–600 s and a tolerance of 0–32, and no minimum-frames control
at all (`ui/filter_bar.rs:129-155`); the organiser offers 1–3600 s, 0–64, and a
minimum of 2–50 frames (`view/organize/group/mod.rs:53-89`). There are tolerance
and gap values the sheet cannot express and the organiser can. Two answers to "is
this one burst?" is a defect whether or not anybody ever navigates between them.

### 8.2 From a mark to the set of photographs carrying it

One decision governs every row of this table, and it is worth stating once rather
than reading it off the cells: **the plain click keeps whatever job the surface
already gives it, and the narrowing verb goes on that surface's menu — except
where the surface gives the click no job at all.** In the keyword panel a click
on a star sets the rating (`ui/tag_panel/mod.rs:101-126`); on a contact-sheet cell
it opens the photograph (`view/grid_view/mod.rs:455-457`), which §9.3 turns into
selecting it, while Ctrl and Shift belong to the selection (`:451-454`, recorded
in §9.1 and kept by §9.3); in the status bar it does nothing whatever
(`bottom_bar.rs:184-205`). So the bar is the one mark surface whose click is free,
and the only row below that takes the verb as a gesture. No modifier carries it
anywhere: a gesture that rates on one surface and filters on the next is worse
than a menu row on both, and on a cell there is no free modifier left to give it.

| From | To | Route today | Where | What the route should be |
|---|---|---|---|---|
| The stars in the status bar | "show me only these" | **No.** `marks()` is documented in its own doc comment as "a summary, not a control"; the stars are `ui.label` with no `Sense` and no hover | `bottom_bar.rs:184-186`, `:202-205` | Click a star: `ShowOnly(stars ≥ n)`. The strip is drawn only when the photograph is already marked (`:202`), so it could never have been the way to rate an unrated frame; setting a rating from the bar is its menu's, §7.4 |
| The stars on a grid cell | the same | **No.** The caption strip is painted, and it sits outside the cell's interactive rect, which covers the drawn picture only | `view/grid_view/cell.rs:83-140`, `view/grid_view/mod.rs:366-370`, `:378-398` | The strip joins the cell's response, so both buttons reach it (§8.12), and the verb is a row on the cell's menu (§7.5). Not a gesture: the plain click means the cell (`grid_view/mod.rs:455-457`) and Ctrl and Shift are the selection's (`:451-454`) |
| The stars in the tag panel | the same | **No.** The panel can emit five `Action`s and none of them is a filter | `ui/tag_panel/mod.rs:31-39`, `:101-126` | The row's menu (§7.6). The click keeps the job it has, which is to set the rating (`:101-126`) |
| A colour label anywhere | "show me only these" | **No.** The bar's swatch carries the only hover among the three marks; the cell's swatch is paint | `bottom_bar.rs:196-200`, `cell.rs:96-101` | The same verb, by the same three routes |
| A pick or reject flag | "show me only these" | **No.** No hover on the bar's flag glyph either | `bottom_bar.rs:187-194` | The same verb, by the same routes. `FlagRule::NotRejected` already exists, and the code's own comment calls it "the one people leave on" |
| Any of the above | the filter bar that would express it | **No.** The only route is `F3`, then two `DragValue`s and two combos | `config/defaults.rs:422-424`, `ui/filter_bar.rs:160-227` | `ShowOnly` writes the rule *and* opens the bar, so the user sees what was set and can widen it |
| The hidden count `(+18)` | the filter that hid them | **No.** A hover explains the number; the counter is inert | `bottom_bar.rs:119-129` | Click opens the bar with the responsible rules highlighted |
| `Nothing matches the filter` in the sheet | getting the photographs back | **Partly, unsaid.** `\` already suspends every rule without forgetting them, but the empty screen is a bare centred label that does not say so. `Clear`, which forgets them, is only inside the bar and has no binding | `view/grid_view/mod.rs:263-272`, `config/defaults.rs:425-427`, `ui/filter_bar.rs:49-57` | The empty state names the rules that emptied it and carries a **Show everything** button — which is `SuspendFilter`, already a command, already on `\`, and already the bar's own wording (`ui/filter_bar.rs:280-291`) |

This is the largest hole in the whole surface, and it is one verb wide. The
application already knows how to narrow: `Rules` has seven fields
(`view/narrow.rs:22-34`), `apply_narrowing` recomputes and hands the result to
both views without re-decoding anything (`app/mod.rs:428-441`), and marks already
re-trigger it (`app/mod.rs:639-658`). What is missing is a way to say "that one"
while pointing at it.

The comparison with Lightroom cuts both ways here, and the honest reading is worth
stating (§2 sorts the field's complaints into the ones that happen here and the
ones that do not). Lightroom's own users report a filter you cannot see from where
its effect is felt: *"if the Filter bar is hidden but the Library filters are turned
on, then you may not see the photos you expect to in the Grid View"*
(<https://mastering-lightroom.com/lightroom-filter-bar/>). avis-imgv is better than
that, and the credit belongs where it is due: the position counter reads
`7/312 (+18)` with "18 more are hidden by the filter" on hover whether or not the
bar is up (`bottom_bar.rs:114-129`). The defect that remains is narrower and
entirely about routes. The counter that reports the loss cannot open the rule that
caused it; and `open_within` resets the pairs, the marks and the stacks but never
`self.narrowing` (`app/mod.rs:297-342`), so a rule set on yesterday's folder is
still on for the first card of today, announced only by a number in brackets.
Whether that carry-over is what the user wanted is a setting that does not exist
yet — `browsing.filter_follows_folder` (§6) — and either answer needs the route in
this table, because a rule nobody can see from the picture is a rule nobody can
undo.

### 8.3 From a selection to a command

| From | To | Route today | Where | What the route should be |
|---|---|---|---|---|
| A selection | move / copy to a destination | **Yes.** `Alt+M` / `Alt+C` → `send_somewhere` → `marked_paths()`, which returns the selection when the sheet has one | `config/defaults.rs:406-411`, `app/cull.rs:296-315`, `app/tagging.rs:277-283` | Keep. Add the mouse route: the selection bubble becomes a button |
| A selection | delete, rate, flag, label, tag | **Yes**, through the same funnel | `app/cull.rs:65-91`, `app/tagging.rs:121-135`, `:277-283` | Keep |
| The `3 selected` bubble | acting on the selection, or clearing it | **No.** `.interactable(false)` | `view/grid_view/mod.rs:314-332` | A button. Everything its menu would offer is a command that already exists (§7) |
| A selection | Bulk rename scoped to it | **No.** Entering a folder job hands over `all_paths()`, silently discarding both the selection and the filter | `app/views.rs:35-38`, `app/mod.rs:395-397` | The mode takes a scope. Its header says `187 of 2030 files · the selection`, with a control to switch to the whole folder |
| A selection | Shift capture time, Group shots | **No**, same line | `app/views.rs:35-38` | The same scope control, shared by all three |
| A selection held in the sheet | acting on it after leaving the sheet | **No, and silently.** `marked_paths` is gated on `self.mode == Mode::Grid`, so outside the sheet a held selection is ignored and the command acts on one photograph; the bubble is drawn inside `GridView::ui`, and even `Escape` is handled there | `app/tagging.rs:277-283`, `view/grid_view/mod.rs:305`, `:626-633`, `app/views.rs:16-38` | The count moves to a place both views draw, and every command that acts on more than one frame says how many before it acts |
| A destination chosen by browsing | a saved slot | **No.** `Answer::Browse` uses the folder once and forgets it; `cull.destinations` is JSON-only | `app/cull.rs:352-373`, `config/mod.rs:180-181` | "Keep this as slot 4" on the panel, written through `save_settings` (`app/settings.rs:104-110`) |

`marked_paths` (`app/tagging.rs:277-283`) is the right idea and is already
described in its own doc comment as "one rule, read by marking, tagging, moving
and deleting alike". The defect is that three of the six modes do not read it, and
that leaving the contact sheet quietly takes the rule away without taking the
selection away.

### 8.4 From a keyword to what it means

A keyword is the only mark in the program that is a piece of text somebody wrote,
and it is the one that travels furthest: the same word is a chip on the
photograph, a row in a tree, a line in a file on disk, a substring in the
browsing filter and a whole string in the folder jobs' filter. The rows below
follow one word through those places, and out to the folder its file sits in,
asking at each what joins it to the next. The last row is the one that costs a
user real time, and it is not a
missing route but a disagreement: the same word typed into the two filters
returns two different sets of photographs.

| From | To | Route today | Where | What the route should be |
|---|---|---|---|---|
| A keyword chip on the photograph | every photograph carrying it | **No.** A single left-click on the chip *removes* the tag; there is no `context_menu` anywhere in the panel | `ui/tag_panel/mod.rs:190-201`, `:31-39` | The chip's menu, where §7.6 already writes the row as *Show me everything tagged this*; the click keeps applying and removing as now (§8.2). The filter's keyword rule is a case-insensitive substring test over the whole hierarchical keyword, so `Slovakia` finds `Places\|Slovakia\|Tatras` (`view/narrow.rs:265-275`, separator at `metadata/xmp/mod.rs:36`) |
| A keyword in the catalogue tree | the same | **No**, same reason | `ui/tag_panel/mod.rs:276-294` | Same verb, same route |
| A keyword | the catalogue file that defines it | **No.** The catalogue is built once at startup from `tags.catalog_file`, resolved against the configuration directory, and a file that cannot be read is a `tracing::warn!` | `app/mod.rs:209`, `annotations/catalog.rs:46-65`, `:184-194` | A route from the panel's heading that names the file, opens it and reloads it. `Catalog::configured` runs once outside the tests, so editing the list means restarting the viewer (§3.5) |
| A keyword | the folder its file lives in | **No.** `Config::path()` has two callers, neither of them the interface | `config/load.rs:14-17`, `annotations/catalog.rs:190`, `logging.rs:191` | §8.6 — one route, used by the catalogue, the log and the configuration alike |
| A keyword in the browsing filter | a keyword in the folder-job filter | **No.** Two filter types that do not share state, with different matching rules: the bar takes a case-insensitive substring of the whole hierarchical keyword, the organiser takes a case-insensitive match on the *whole* stored keyword | `view/narrow.rs:265-275` vs `organize/filter.rs:119-127`, `organize/mod.rs:121-126` | One filter, two presentations. Until then, "rename everything I tagged" means typing the word twice and getting two different answers — `Slovakia` matches in the bar and matches nothing in the organiser, where only `Places\|Slovakia\|Tatras` in full will do |

### 8.5 From a readout to the setting behind it

A readout is something the program has already worked out and put on screen — a
number, a state word, a file name — and behind almost every one of them stands
either a setting that decided how it is shown or a command that would act on what
it reports. The eleven rows below are the ones where the readout is of no use on
its own, ordered from the photograph outwards. Two of them are worth stopping on:
`Blown 3.4%`, the sharpest dead end in the program, for the reason its row gives;
and the Shift capture time preview, which is not a routing fault at all but a
wrong preview standing in front of a job that rewrites files, and is taken on its
own after the table.

| From | To | Route today | Where | What the route should be |
|---|---|---|---|---|
| The overlay on the photograph | the setting that formats it | **No.** `o` cycles the corner into the view's private copy of the config; `overlay_format` and `overlay_text_size` are read in one function and nowhere else, and appear in no UI | `view/image_view/mod.rs:278-280`, `:628-648` | A menu on the overlay carrying the corner, the text size and a route to what it says (§7; which control each of those gets is §5). The corner picker's labels already exist and are called only from a test (`view/image_view/overlay.rs:57-65`, `:154`) |
| The overlay corner | surviving a restart | **No, and worse.** The corner is never written back, and rebinding any key in `Settings ▸ Keyboard…` calls `set_config`, which replaces the whole struct and snaps the corner to the value on disk. So does "Put everything back to the defaults" | `app/settings.rs:85-94`, `ui/keys.rs:109-126`, `view/image_view/navigate.rs:95-98` | The cycle writes through `save_settings`, as the slideshow window already does (`app/settings.rs:70`). It is one of the six session-only settings §3 counts |
| A metadata row in the side panel | the `metadata_tags` setting that decides which rows appear | **No.** The rows are drawn from `general.metadata_tags`, which is read at one line and has no control | `app/panels.rs:151-164`, `app/chrome.rs:117` | A menu on the row: copy the value, copy `Tag: value`, hide this row, choose which rows — the last opening **The window**, where `general.metadata_tags` lives, with that tag highlighted (§7.6, §4.5 page 10) |
| A metadata row | the value, in the clipboard | **No.** Hover shows the whole value and nothing takes it | `app/panels.rs:161-162` | The same menu |
| `Blown 3.4%` / `Crushed 0.2%` | the clipping overlay that shows *where* | **No.** Two labels with hovers, in a plot allocated `Sense::hover()`; the overlay is a separate key, `c`, in a different subsystem | `ui/histogram.rs:108-134`, `:43`, `config/defaults.rs:156-158`, `view/image_view/mod.rs:281` | `Blown` becomes a toggle for the clipping overlay and `Crushed` for its shadow half. This is the sharpest dead end in the program: the number is in the right-hand panel, the picture of the number is over the photograph, and the only thing joining them is a key nothing names |
| The cache readout | the budgets it is reporting against | **No.** Six `CacheConfig` fields, all JSON-only, all folded into the stores once at construction | `app/panels.rs:168-248`, `config/mod.rs:233-260`, `app/stores.rs:32-60` | Each line links to its row on **Speed and memory** (§4.5). Note the dependency: the stores take no setter, so a control alone would still need a restart until they do (§3.5) |
| `n image(s) could not be opened` | *which* images | **No.** A count and nothing else | `app/panels.rs:193-195` | Click narrows the collection to them — the same `ShowOnly` verb, with a rule `Rules` does not yet have |
| `Filling` · `Comparing` · `Flattened` · `Watching` · `Advancing` · `RAW+JPEG` | turning any of them off | **No.** Six bare `ui.label`s, no hover, no key named | `bottom_bar.rs:144-155` | Four are toggles for commands that exist — `ToggleFlatten` (`app/mod.rs:698`), `ToggleWatcher` (`:699-701`), `ToggleAdvance` (`:706`), `ToggleFillLatch` (`view/image_view/mod.rs:259-262`) — so those four words become buttons. `Comparing` is left by `Escape` (`view/image_view/input.rs:117`). `RAW+JPEG` is not a command at all but the JSON-only `raw.pair_with_jpeg` (`config/mod.rs:74-80`), so that word needs a route to a *setting* — the case that makes the fourth mechanism necessary rather than optional |
| The file name in the bar | reveal in the file manager, copy the path, open with | **No.** A truncated `Label` with no hover, over a template nothing can configure, and nothing in `src/` opens a file manager — `rfd` is a picker and `trash` is the bin | `bottom_bar.rs:159-164`, `view/image_view/mod.rs:651-673`, `Cargo.toml:21,38` | The machinery already exists: `actions::execute` spawns a program with an argument vector and six path placeholders (`actions/user_action.rs:26-54`, `:75-100`). Only the program's name is platform-specific |
| The preview rows in Bulk rename and Shift capture time | the photographs they describe | **No.** Each row is three `ui.label`s, with no thumbnail, although the scan already decodes the camera's thumbnail into `Entry` wherever the file embeds one | `view/organize/table.rs:60-81`, `organize/mod.rs:59-61` | The thumbnail in the row, and a click that opens that frame |
| A Shift capture time preview | the frames it will actually change | **No — and the preview can be blank in front of a job that still runs.** The "would become" column is computed from `Date/Time Original` alone, and the button counts something else | `organize/timeshift.rs:185-210`, `:102-107`, `view/organize/timeshift.rs:99-130`, `:132-158` | One column per ticked field, and a button that counts what the preview shows. Stated in full below |

The last row is not a route at all. It is a wrong preview in front of a job that
rewrites files, which is why it is written out here rather than left in a cell.

`planned_for` builds each row's "would become" from one field and one only.
`before` is the capture time whether or not it was ticked; `after` is the shifted
value of the capture time *among the ticked fields* (`organize/timeshift.rs:185-210`;
`CAPTURE_TAG` is `"Date/Time Original"`, `organize/mod.rs:40`). Untick that field
and leave `Modify Date` ticked — both are ordinary fields the scan finds and the
checkbox row offers (`metadata/tags.rs:73`, `view/organize/timeshift.rs:63-75`) —
and `plan.after` is `None` for every file, so the preview column reads `—` all the
way down (`view/organize/timeshift.rs:150-153`). Nothing else changes. `changes()`
is `!fields.is_empty()` and is still true for every file that carries a
`Modify Date` (`organize/timeshift.rs:104-106`), so the button still counts them
all, still reads `Change 412 file(s)` for a folder of that size, and is still
enabled, an offset having been set (`view/organize/timeshift.rs:99-109`). Pressing
it rewrites the modify time of four hundred and twelve files while the screen
showed no change to any of them. The repair is one column per ticked field — or a
row per field, which is the same table transposed — and a button that counts what
the preview shows.

### 8.6 From a warning to the evidence

The notice band is the program's entire vocabulary for something having gone
wrong: six seconds, four lines deep, a 600 ms fade, no history and no way to ask
what a line meant (`ui/notice.rs:15-22`, `:78-82`). The five rows below are the
failures a user is likeliest to meet. In every one of them the evidence exists
somewhere — a file on disk, a row in the keyboard editor, a line in the log — and
the notice that reports the failure is the one surface in the program that cannot
reach it. What each notice should *say* is §10's; where it should be able to go
is here.

| From | To | Route today | Where | What the route should be |
|---|---|---|---|---|
| A notice about a failed sidecar write | the file that failed, or the log | **No.** The notice band is `.interactable(false)`, holds for 6 s and fades over 600 ms, keeps at most four lines, and has no history | `ui/notice.rs:78-82`, `:15-22`, `:56-58`, `app/mod.rs:758-763` | Notices become clickable: a failure carries a route to the file (narrow to it) and to the log. A notice worth showing is worth keeping. What each one *says* is §10's |
| Any failure | the log file | **No.** `logging::path()` opens the log (`logging.rs:86`) and writes its own path into it (`main.rs:34-36`); nothing puts it on screen | `logging.rs:37-40`, `:86`, `main.rs:34-36` | One `Reveal` target — the folder holding the log, the configuration, the keyword list and the recent tags — reachable from any failure notice, and from the Help menu §10 places |
| A startup warning about clashing keys | the keyboard editor | **No.** The clash is said as a notice and is gone 6.6 seconds after launch; the editor also flags clashes, but only if you open it | `app/mod.rs:240-242`, `ui/keys.rs:151-159`, `:195-218` | The notice carries "Fix it", which opens **Keys and mouse** scrolled to the offending row |
| A startup warning that part of the configuration could not be read | the configuration file | **No.** The same fading notice as everything else, gone in 6.6 seconds, and the file is silently not written from then on | `app/mod.rs:248-253`, `config/load.rs:24-32`, `ui/notice.rs:69-99` | The same folder route, and a persistent marker in the settings window until it is resolved (§3.2) |
| Per-file failures in a folder job | anything at all | **No.** `tracing::warn!` only; `Notices` appears nowhere under `src/view/` | `view/organize/rename.rs:97-99`, `view/organize/timeshift.rs:113-115` | The same notice route, and a list of the files that did not move |

### 8.7 From the keys to what they do

| From | To | Route today | Where | What the route should be |
|---|---|---|---|---|
| The cheat sheet | the keyboard editor | **No, and structurally blocked.** The rows are `ui.label` pairs, the footer is a sentence and not a control, and *any* click anywhere closes the window | `ui/cheat_sheet.rs:88-92`, `:96-100`, `:104-121` | The footer sentence becomes a button opening **Keys and mouse**, and every row opens it at that binding. The close-on-click rule is narrowed to clicks outside the sheet first, since neither can be clicked until it is |
| The keyboard editor | trying the key | **No.** A row is a name, a key button and a description, and none of them goes anywhere | `ui/keys.rs:131-162` | "Show me" on a row: switch to the mode the binding belongs to and leave the sheet open on it. The registry already records the mode, since the cheat sheet filters by it (`ui/cheat_sheet.rs:28-36`) |
| A key that appears to do nothing | why | **No.** `Ctrl+T` flips `filmstrip_visible` and `show_filmstrip` returns immediately because the default height is zero. The binding is in the registry, in the "General" section, which every mode's cheat sheet shows | `app/mod.rs:713`, `app/views.rs:137-141`, `config/defaults.rs:299-301`, `config/bindings.rs:100-105`, `ui/cheat_sheet.rs:28-36` | A command that cannot act says so, and offers the setting that would let it: "The strip has no height yet — set one" |
| `?` | anything | **Nothing on screen names it.** It is hard-coded outside the registry — no `ShowKeys` binding exists anywhere under `src/config/` — so it is in neither the editor nor the sheet it opens | `app/input.rs:106-115` | Named in `Help ▸ Keys…  ?`, in the sheet's own footer, and in the empty state (§10) |

The editor's other faults — no search, no unbind, no per-row reset, clashes
scoped by section — are §9's. The route into it and the route back out of it are
the parts that belong here.

FastRawViewer's forum has the same shape of failure: a user who bought the tool to
cull could not reject a frame, because a preference had disabled XMP rejects — and
the shortcut list *said so*, inside a settings screen, rather than at the moment
the key was pressed (<https://www.fastrawviewer.com/node/577>). `Ctrl+T` here is
that bug exactly, with nothing anywhere that says so.

### 8.8 The dead ends, plainly

A dead end is a place where the program shows a value and offers no way to act on
it. Sixteen, in the order a photographer would meet them:

1. The stars, flag and colour label in the status bar (`bottom_bar.rs:184-205`).
2. The stars, flag and label on a contact-sheet cell (`cell.rs:83-140`).
3. The hidden-by-filter count `(+18)` (`bottom_bar.rs:119-129`).
4. The stack sentence in the bar (`bottom_bar.rs:131-142`) and the stack plate on
   a cell (`cell.rs:190-228`).
5. The six mode words — `Flattened`, `Watching`, `Filling`, `Advancing`,
   `Comparing`, `RAW+JPEG` (`bottom_bar.rs:144-155`).
6. The file name, which is a configurable template nothing can configure
   (`bottom_bar.rs:159-164`, `config/mod.rs:382-383`).
7. The overlay on the photograph (`view/image_view/overlay.rs:93-145`).
8. `Blown` and `Crushed` (`ui/histogram.rs:108-134`).
9. Every metadata row in the side panel (`app/panels.rs:151-164`).
10. Every line of the cache readout, including `n image(s) could not be opened`
    (`app/panels.rs:168-248`).
11. The selection count bubble (`view/grid_view/mod.rs:314-332`).
12. `Nothing matches the filter` and `No images here`
    (`view/grid_view/mod.rs:263-272`, `view/image_view/layout.rs:40-49`).
13. `Could not open this image` — no path, no reason, no retry
    (`view/image_view/layout.rs:130-134`).
14. Every notice, including the two that report a broken configuration
    (`ui/notice.rs:78-82`).
15. Every row of the cheat sheet (`ui/cheat_sheet.rs:88-92`).
16. Every row of a folder-job preview table (`view/organize/table.rs:60-81`).

And four places where a route exists but nothing on the surface points to it:
opening a stack (`E`, `app/mod.rs:477-493`), narrowing by a rating (`F3` then two
`DragValue`s, `ui/filter_bar.rs:160-176`), the zoom menu's nine entries (a context
menu on a `Label` with no affordance, `bottom_bar.rs:275-307`), and setting the
filter aside (`\`, whose only control is inside a bar the empty screen it would
rescue does not mention, `config/defaults.rs:425-427`,
`ui/filter_bar.rs:280-291`).

### 8.9 Why it came out this way

Two structural causes, both worth naming because they will keep producing dead
ends otherwise.

**The panels disagree about what "the current photograph" is.** The metadata and
histogram panel reads `image_view.active_metadata()` and
`image_view.active_histogram()` (`app/chrome.rs:114-122`); the tag panel reads
`marked_path()`, which is the *grid cursor* in the sheet
(`app/tagging.rs:263-268`). Open both while browsing the contact sheet and the
right-hand panel describes whatever the image view was last left on while the
left-hand panel describes the cell under the cursor. There is no shared answer to
"what is being looked at", so a link from one surface to another has nothing to
carry.

**The context menu was designed as a plug-in point rather than as a menu.** A
`ContextMenuEntry` is `{ description, exec, callback }` (`config/mod.rs:547-552`)
— it can run another program and nothing else. There is no representation for "a
menu row that runs one of this program's own commands", which is why the
photograph and the cell have no rating, no flag, no label, no keyword, no
destination, no stack verb, no reveal, no copy path and no rename, although every
one of those verbs exists a few files away.

### 8.10 The rule

> **Every value the viewer draws is either a control or carries a route to one.**
> If a surface shows a *value*, it offers the command that changes it and the
> setting that decides how it is shown. If it shows a *set* — a count, a rating, a
> run of frames, a keyword — it offers to show that set on its own. If it shows a
> *failure*, it offers the evidence.

The test is mechanical, which is the point: pick any pixel that carries
information and ask "and what do I do about it?" The answer may not be "press a
key nobody told you about", and it may not be "edit `config.json` and restart".

The corollary is that redundancy is a feature and not a defect. Nielsen Norman
Group, on contextual menus: *"Make sure the commands in contextual menus are also
available from the application's main menu"*, and *"Options presented to users in
contextual menus should be the same, regardless of how users interact with the
system to reveal these menus"*
(<https://www.nngroup.com/articles/contextual-menus/>). The same body of work
insists that an accelerator is a *secondary* route — *"Shortcuts—unseen by the
novice user—speed up the interaction for the expert users"*
(<https://www.nngroup.com/articles/flexibility-efficiency-heuristic/>). A key is a
fine second way to reach a command. It is a defect as the only way, which is what
`E` (`config/defaults.rs:186-188`), `F3` (`:422-424`), `c` (`:156-158`), `o`
(`:152-154`), `Ctrl+F` (`:80-81`), `Ctrl+W` (`:84-85`) and `Ctrl+M` (`:238-239`)
are today: seven commands with no other route, none of them among the menu bar's
eleven items (§1.5).

### 8.11 The four mechanisms

Everything above is four things, not sixty. Each is small, and each pays for
itself across many rows of the tables.

**1. A "show only these" verb.** One command, `Command::ShowOnly`, which writes one
field of `Rules` (`view/narrow.rs:22-34`), calls `apply_narrowing`
(`app/mod.rs:428-441`) and raises the filter bar so the change is visible and
reversible. It writes that one field and leaves the other six as it found them —
the rules combine with "and", so a filter nobody has touched is the whole folder
(`view/narrow.rs:19-20`) — which means two of them in succession narrow twice, and
widening and clearing are the bar's own controls (`ui/filter_bar.rs:49-57`,
`:160-227`). No modifier carries the verb,
for the reason §8.2 gives. It reaches every surface that draws a mark by one of
two routes: as the plain click in the status bar (`bottom_bar.rs:184-205`), the
one mark surface with no other job, and as a menu row everywhere else — the cell
caption (`cell.rs:83-140`), the tag panel's stars, flags, swatches and keyword
chips (`ui/tag_panel/mod.rs:101-205`), and the filmstrip once it draws marks at
all (`view/grid_view/filmstrip.rs:101-155` draws none today). The
`n could not be opened` line takes the click too (`app/panels.rs:193-195`), being
a count rather than a mark. Its inverse already exists: `SuspendFilter` on `\`
sets the rules aside without forgetting them (`app/mod.rs:729-732`,
`config/defaults.rs:425-427`).

One constraint on the payload, because it decides the shape. `Command` derives
`Copy` and is taken by value (`app/input.rs:11`, `:198`; `app/mod.rs:684`), and
three of `Rules`' seven fields are `String`s (`view/narrow.rs:29-33`). So
`ShowOnly` either carries a `Copy` payload naming which rule and which value, or
`Command` stops being `Copy` and the two call sites that take it by value are
adjusted. The first is the smaller change and is enough for stars, flags and
labels; a keyword rule needs the second.

This one verb closes every row of §8.2, the `(+18)` counter, the failed-image
count, and the empty state.

**2. A "reveal in…" verb.** One command, `Command::Reveal(Target)`, where the
target is a place rather than a feature: *the sheet at this frame* (which
`focus_on` already does, `view/grid_view/mod.rs:204-208`), *the run this frame
belongs to*, *Group shots at this run*, *the filter control that produced this
number*, *the settings row behind this readout*, *the keyword catalogue*, *the
configuration file*, *the log*. Four of those are already half-built: `go_to(index)`
puts both views on one store position (`app/mod.rs:527-534`), `focus_on` scrolls
the sheet, `Config::path()` and `logging::path()` know where the files are
(`config/load.rs:14-17`, `logging.rs:37-40`), and `actions::execute` already spawns
a program with a safe argument vector (`actions/user_action.rs:75-100`), which is
what "reveal in the file manager" needs once the platform's program name is
chosen. What has to be written is a scroll-to-group in the organiser — there is no
`scroll_to` anywhere under `view/organize/` — and a focus-a-row in the settings
window, which `Ctrl+,` and `Settings ▸ All settings…` open (§3.2).

The last target is also the one route to the files: the folder holding
`config.json`, `avis-imgv.log`, `recent_tags.json` and the keyword list, reached
from any notice that mentions one of them, and from the Help menu §10 owns.

**3. One selection that every command reads.** `marked_paths`
(`app/tagging.rs:277-283`) is already this, for marks, tags, delete, move and copy.
Three changes finish it: the folder jobs take a scope instead of `all_paths()`
(`app/views.rs:35-38`) and say in their header what the scope is; the count is
drawn in a place both views draw rather than inside `GridView::ui`
(`view/grid_view/mod.rs:305`), and the `Mode::Grid` gate comes off `marked_paths`
so leaving the sheet does not silently reduce a selection of two hundred to one;
and the bubble becomes a button whose menu is the list of commands that read it.
The stack caveat has to be said out loud on that button too: a folded run
contributes only its standing frame to `visible` (`view/stacks.rs:193-227`), and
`Ctrl+A` selects exactly what `visible` holds (`view/selection.rs:73-81`), so
`Ctrl+A` over a stacked folder selects one frame per burst while the plate says
`❏ 17`. Lightroom users describe the same failure from the other side — metadata
changes behaving differently "whether selected stack collapsed or expanded", and
"missing photos are buried in a stack"
(<https://www.lightroomqueen.com/community/threads/stacks-why-bother.44251/>). That
thread is also the argument for making the plate a control at all: what its users
ask for is auto-expand on hover and the ability to filter by stack membership,
because "the present method for identifying stacks is not noticeable enough".

**4. Links in context menus.** `ContextMenuEntry` gains a variant that names one of
the program's own commands, and `default_ctx_menu()` stops returning an empty
vector (`config/defaults.rs:165-167`); the user's configured external commands are
appended to the built-in menu rather than replacing it. Which surface carries which
menu, what is on it and in what order is §7's map, and the turns are the one
submenu anywhere (§7.2). The part that belongs here is the last group of every menu: the
route out — *Show only these*, *Reveal in…*, and `More settings… (<page name>)`,
which is always last and never varies (§7.2). Shift+F10 opens the same menu from
the keyboard, and §7 explains why that key and no other.

One more thing has to change for the fourth mechanism to work at all: the cheat
sheet closes on *any* click, including one inside its own window
(`ui/cheat_sheet.rs:104-121`), so it cannot carry a link until that rule is
narrowed to clicks outside it.

### 8.12 What this does not require

No new panel, no new window, and no new view. Two `Command` variants, one field on
`ContextMenuEntry`, one scope argument on `set_mode`, the `Mode::Grid` gate off
`marked_paths`, and interaction added to the sixteen places listed in §8.8. Most of
those are already `Label`s and take a `.sense(Sense::click())`; the painted ones —
the cell caption, the stack plate, the overlay — take
`Ui::interact(rect, id, Sense::click())`, which takes `&self` in egui 0.33
(`egui-0.33.0/src/ui.rs:1131`) and so changes no signatures. Two `Area`s lose their
`.interactable(false)` (`ui/notice.rs:80`, `view/grid_view/mod.rs:322`; there are
only two in the whole of `src/`). The narrowing engine, the selection funnel, the
mode switcher, the scroll-to-index, the safe process spawn and the notice system
all exist and are all already correct; what is missing is that nothing currently
*points* at them.

The one route in this chapter that costs real work is the shared burst detector —
one `group::Settings` behind both the folded sheet and Group shots
(`app/stacking.rs:28` vs `view/organize/mod.rs:56`), with one set of controls
spanning one range — and that is worth doing on its own merits. §12 says when.

*Finished when:* right-clicking a photograph offers its rating, its keywords, its
destinations and "show me only these"; clicking three stars in the status bar
leaves three stars and better on screen and the filter bar open saying so; clicking
`Blown 3.4%` puts the clipping mask on the photograph; the selection count is a
button that survives leaving the contact sheet, and the folder jobs say what they
are about to act on; the cheat sheet has a way into **Keys and mouse** and a
startup clash notice has a way to the row that clashed; and every notice about a
failure can be clicked to see what failed.

## 9. The mouse, and the keyboard for people who do not know it

The keyboard has a registry. Sixty-nine rows over the sixty shortcut fields the
configuration carries (§3.1), each with a name and a sentence, every one of them
rebindable from inside the program and printed on a cheat sheet that shows the
user's own keys rather than the documentation's: `src/config/bindings.rs:85-439`,
`src/ui/keys.rs:34-129`, `src/ui/cheat_sheet.rs:46-121`. Fifty-eight of the rows
come from the `binding!` macro and name fifty-eight distinct fields; the other
eleven are the five colour labels and the six star ratings pushed in two loops at
`src/config/bindings.rs:420-436`, reaching the two `Vec<Shortcut>` fields the
macro cannot.

The mouse has none of that. Every gesture is compiled into the view that reads
it, none of them can be remapped by any means, and the single setting that
governs any of them at all — `image_view.scroll_navigation`, a boolean that turns
wheel navigation off (`src/config/mod.rs:373-374`, default `true` at
`src/config/defaults.rs:136-138`) — appears in no menu, no window and no editor.
It is documented in the README's configuration table (`README.md:658`) and
nowhere the running program can reach. It is one of the two existing fields that
land on the *Keys and mouse* page (§4.5), and the migration that gives the mouse
a section of its own retires it into `mouse.wheel` (§6). `PointerButton` occurs
**zero** times in `src/`; so do `double_clicked`, `middle_clicked`,
`drag_started`, `dropped_files` and `clipboard` — six greps over the crate, six
counts of zero. Right-clicking the photograph, on a stock install, produces
nothing at all (§7.1).

That asymmetry is the chapter. It matters more here than in most programs for
three reasons. A culling tool is used one-handed for hours, so the hand that is
not on the keyboard should be doing something. Nothing on screen names a key, so
somebody who has not read the README has only the pointer to find things with
(§1.4). And the loudest, longest-running complaint in the whole viewer corpus is
about the mouse wheel: nomacs issue 237 ran from 2018 to 2025 across sixteen
accounts, at least one of whom said they had uninstalled over it, and the
capability being asked for had existed behind two checkboxes nobody could find
(<https://github.com/nomacs/nomacs/issues/237>). avis-imgv is in the same
position with `scroll_navigation`, minus the checkboxes.

### 9.1 What the pointer does today

**The image view.** All of it is compiled in; the `Where` column is the only
documentation that exists.

| Gesture | What happens | Where |
|---|---|---|
| Wheel up | **Next** photograph | `src/view/image_view/input.rs:197-201` — `raw_scroll_delta.y > 0` is a wheel *up* (egui-0.33.0 `src/input_state/mod.rs:243-244`, winit-0.30.12 `src/event.rs:957-958`; both versions pinned in `Cargo.lock`) |
| Wheel down | Previous photograph | same |
| Wheel, either way | **Also pans the image**, on the same frame and the several after it | `src/view/image_view/interaction.rs:40`, `:45` → `canvas.rs:336-338`; the smoothing is egui-0.33.0 `src/input_state/mod.rs:511-526` |
| Ctrl + wheel, or a trackpad pinch | Zoom, anchored under the pointer, with no pan | `src/view/image_view/interaction.rs:26-29`, `mod.rs:180-221`; Ctrl is egui's `zoom_modifier` and its delta never reaches `smooth_scroll_delta` (`input_state/mod.rs:114`, `:476-491`) |
| **Shift + wheel** | **Nothing moves forward.** Shift is egui's `horizontal_scroll_modifier`, so the delta is moved onto the X axis before `raw_scroll_delta` accumulates and `raw_scroll_delta.y` is zero; the image pans sideways instead | egui-0.33.0 `src/input_state/mod.rs:115`, `:451-464`; then `src/view/image_view/input.rs:197-201` returns `None` |
| Alt + wheel | Exactly what a bare wheel does — Alt is the `vertical_scroll_modifier`, which is what a vertical wheel already is | `input_state/mod.rs:116`, `:459-462` |
| Drag with **any** button | Pan, if the image is larger than the panel | `src/view/image_view/interaction.rs:41-42`; `is_decidedly_dragging()` tests `any_down()`, which is over every button (egui-0.33.0 `src/input_state/mod.rs:1451-1452`, `:1541-1546`) |
| Drag when the image is fitted | Nothing, silently — the pan is clamped to zero slack | `src/view/image_view/canvas.rs:347-353` |
| Left click on the photograph | Nothing. The panel is upgraded to `Sense::click()` and nothing reads `clicked()` on its response: it is returned as `Shown.response` and passed only to `handle_pointer` and `handle_context_menu`, neither of which asks about clicks | `src/view/image_view/layout.rs:15-20`, `:86-90`; `src/view/image_view/mod.rs:169-170` |
| Double click | Nothing | no `double_clicked` in `src/` |
| Middle click | Nothing | no `middle_clicked` in `src/` |
| Side buttons (Back / Forward) | Nothing — although egui-winit already delivers them as `Extra1` / `Extra2` | egui-winit-0.33.0 `src/lib.rs:1111-1112` |
| Right click on the photograph | Nothing by default; the user's `exec` entries when configured | `src/view/image_view/interaction.rs:70-80` → `src/actions/user_action.rs:147-149`, default `vec![]` at `src/config/defaults.rs:165-167` |
| Right click while comparing | Addresses the **focused** pane, whichever pane was clicked | `interaction.rs:71` uses `active_path()`, which is `store.path(self.cursor)` (`navigate.rs:104-106`); the whole central panel is one response (`layout.rs:86-88`) |
| Drag the zoom slider | Zoom, 1 %–1600 %, logarithmic | `src/view/image_view/bottom_bar.rs:252-273` |
| Right click the zoom percentage | The nine entries of the one menu that exists out of the box | `bottom_bar.rs:283-304`, list at `:12` |
| Left click the zoom percentage | Nothing | `bottom_bar.rs:278-281` — `Sense::click()` with no `clicked()` reader |
| Click the "go to" field | Focus it. Deliberately unreachable any other way | `bottom_bar.rs:218-225` |
| Hover anything in the view | Nothing: no cursor change, and three tooltips, all of them in the bottom bar | `set_cursor_icon` is called only at `src/view/grid_view/filmstrip.rs:151` and `src/view/grid_view/mod.rs:437`; the tooltips are `bottom_bar.rs:126`, `:139`, `:199` |

**The contact sheet.**

| Gesture | What happens | Where |
|---|---|---|
| Left click a cell | Opens it **and leaves the sheet** | `src/view/grid_view/mod.rs:444-456`, then `src/app/views.rs:84-87` switches to `Mode::Image` |
| Ctrl / Cmd + click | Toggle that cell in the selection | `src/view/grid_view/mod.rs:451-452` |
| Shift + click | Extend the selection to it | `:453-454` |
| Right click a cell | The user's `exec` entries, on the **hovered** cell's path only — a two-hundred-frame selection is ignored | `:460-464` |
| Wheel | Scroll the sheet — wheel *down* moves **forward** | `:274`, an ordinary egui `ScrollArea` with `ScrollSource::ALL` |
| Ctrl + wheel | More or fewer thumbnails per row, capped at sixteen; wheel *up* makes them bigger | `:516-528`, `MAX_COLUMNS` at `:42` |
| Hover a cell | Pointing hand, and the file name as a tooltip | `:394`, `:436-438` |
| Drag | Nothing. No rubber band, no drag to reorder, no drag out | no `drag_started` in `src/` |
| Click a badge — a star, a label chip, the flag | Nothing. The badges are painted, not allocated | `src/view/grid_view/cell.rs` draws through `ui.painter()` at `:88`, `:145`, `:160`, `:195`, `:241`, and contains no `Sense`, no `Response` and no `clicked` in its 310 lines |

**Everything else.**

| Where | Gesture | What happens | Source |
|---|---|---|---|
| Filmstrip cell | Left click | Jump to that photograph | `src/view/grid_view/filmstrip.rs:108`, `:154` |
| Filmstrip cell | Hover | Pointing hand, and nothing else — no name, no marks | `filmstrip.rs:150-152` |
| Filmstrip cell | Right click, Ctrl click, Shift click | Nothing | `filmstrip.rs:108` senses `click()` only |
| Directory tree row | Left click | Expand or collapse — never open. Expanding moves the highlight the keys act on to that row; collapsing moves it only if the highlight was inside what was folded away | `src/ui/tree.rs:262-264` → `:76-86`; the two writes are `:132` (open) and `:110-111` (close) |
| Directory tree row | **Right click** | **Opens that folder** — the only *pointer* route into a folder | `src/ui/tree.rs:266-268` |
| Bottom bar: stars, flag, colour chip | Click | Nothing; they are `ui.label` | `src/view/image_view/bottom_bar.rs:186-205` |
| Bottom bar: `Flattened`, `Watching`, `Filling`, `Advancing`, `Comparing`, `RAW+JPEG` | Click, hover | Nothing, and no tooltip | `bottom_bar.rs:144-155` |
| Bottom bar: the file name | Hover | Nothing — it is truncated with no `on_hover_text` | `bottom_bar.rs:161-164` |
| Tag panel: stars, flags, swatches | Click | Set, and click again to clear | `src/ui/tag_panel/mod.rs:101-179` |
| Destination panel | Click a slot, or "Choose a folder…" | Send there / open a picker | `src/ui/destinations.rs:74-83`, `:97-99` |
| Cheat sheet | Any click | Closes it | `src/ui/cheat_sheet.rs:109-118` |
| Menu bar, filter bar, side panel, metrics panel | Right click | Nothing anywhere | no `context_menu` call in `src/app/panels.rs`, `src/ui/filter_bar.rs`, `src/app/chrome.rs`, `src/ui/tag_panel/mod.rs` |

There are exactly two `response.context_menu(…)` registrations in the crate:
`src/actions/user_action.rs:152`, which the photograph and the grid share and
which returns before registering anything when the list is empty, and
`src/view/image_view/bottom_bar.rs:283`, the zoom label. §7.1 counts the surfaces
rather than the call sites and states what each of them does. The explanatory
text is as thin: 33 hover tooltips across 11 of the 139 files in `src/`, and
neither the star strip, nor the flag, nor the mode words is among them (§10.7).

### 9.2 Seven faults that are not "missing features"

All seven are live on a default install with nothing configured. They are not
absent capabilities; they are the pointer doing the wrong thing, or two things,
or nothing at all where the code plainly means to do something.

**1. The wheel does two jobs at once.** One notch over a photograph that is
zoomed in calls `Command::Next` (`src/view/image_view/input.rs:197-201`) *and*
writes `smooth_scroll_delta` into `viewport.scroll_delta` (`interaction.rs:40`,
`:45`), which pans the photograph that has just arrived (`canvas.rs:336-338`).
The order is explicit: the navigation runs at `interaction.rs:18-22`, the delta
is stored at `:45`, and the canvas applies it to the new image's viewport.
Nothing guards the second against the first. It shows whenever the arriving
photograph has slack — a restored viewport (`navigate.rs:160-162`) or the fill
latch is enough. Directory Opus, Picview and the modern Windows Photos app all
give the wheel exactly one job at a time and let you say which
(<https://www.gpsoft.com.au/help/opus12/Documents/Prefs/Viewer_Mouse_Buttons.htm>,
<https://picview.chitaner.com/blog/mouse_keyboard_trackpad/>,
<https://learn.microsoft.com/en-us/answers/questions/2433200/how-to-change-mouse-wheel-as-next-prev-in-windows>).

**2. The wheel disagrees with itself between the two views.** In the image view
wheel-*up* moves forward (`src/view/image_view/input.rs:198`); in the contact
sheet wheel-*down* moves forward, because it is an ordinary egui `ScrollArea`
(`src/view/grid_view/mod.rs:274`). The same wrist movement means "later" in one
view and "earlier" in the other, and `Backspace` switches between them
(`src/config/defaults.rs:53-55`). There is no direction setting.

**3. Shift + wheel silently does nothing, and nothing in the program knows why.**
egui's default `horizontal_scroll_modifier` is Shift (egui-0.33.0
`src/input_state/mod.rs:115`), so a Shift + wheel is rewritten into a purely
horizontal delta at `:451-462`, before `raw_scroll_delta` is accumulated at
`:464`. `raw_scroll_delta.y` is then zero and `scroll_navigation` returns `None`
(`src/view/image_view/input.rs:192-201`). The photograph pans sideways instead.
This is not a decision anybody in this codebase made; it is a default inherited
from the toolkit, and it is exactly the modifier a plan that wants a Shift +
wheel meaning has to claim back. The claim itself is one line —
`horizontal_scroll_modifier` is a public field of a public `Options` member
(egui-0.33.0 `src/input_state/mod.rs:87`, `src/memory/mod.rs:281`) — but it has
to be made deliberately, and against the toolkit's own advice; §9.3 says what
that costs.

**4. Every button pans, and whether the right button pans or opens a menu is
decided by a threshold nobody can see.** `is_decidedly_dragging()` is
button-agnostic (egui-0.33.0 `src/input_state/mod.rs:1451-1452`, `:1541-1546`),
so a right-button drag moves the image. The menu, when one is configured, opens
on `response.secondary_clicked()` (egui-0.33.0
`src/containers/popup.rs:248-251`), and a *click* is by definition not a drag. So
the two gestures do not collide — they are separated by `max_click_dist: 6.0`
points and `max_click_duration: 0.8` seconds (`input_state/mod.rs:111-112`). Move
six points or hold for eight tenths of a second and the menu you asked for never
appears; the image pans instead. An XnView user hit the same shape of problem
from the other direction, having configured the right button to pan and finding
the context menu opening instead
(<https://newsgroup.xnview.com/viewtopic.php?t=42107>) — that particular report
turned out to be the reporter's own machine, but the structural point stands: one
button cannot own a drag and a menu without a rule, and a rule expressed as a
distance and a duration is invisible. The repair is a `PointerButton` check on
the pan, so that panning belongs to whichever button `mouse.drag` names (§6) and
the secondary button is left to the menu.

**5. A drag begun on the zoom slider keeps panning the photograph.**
`handle_pointer` gates on `response.contains_pointer()`
(`src/view/image_view/interaction.rs:16`), which egui documents as true "even if
some other widget is being dragged" (egui-0.33.0 `src/response.rs:279-289`) and
computes for "all widgets that contain the pointer this frame, regardless if the
user is currently clicking or dragging" (`src/interaction.rs:51-56`).
`is_decidedly_dragging()` is true for the whole slider drag. So dragging the
slider and letting the pointer stray up over the image pans it under a drag that
was never about it. Read off the API contract rather than measured on a machine,
but the contract is explicit on both halves.

**6. A single click in the contact sheet leaves the contact sheet.**
`src/view/grid_view/mod.rs:456` sets `self.selected`, and
`src/app/views.rs:84-87` switches mode on the same frame. The sheet already has
everything needed to behave otherwise: a cursor, a selection, `Ctrl`+click,
`Shift`+click, `Space` to pick out (`src/view/grid_view/mod.rs:619-624`) and
`Enter` to open (`:605-607`). Plain click is the one gesture that contradicts
that model, and the only way back is `Backspace`. A culling tool's contact sheet
is a surface you act *on*: Camera Bits' documentation for Photo Mechanic
describes applying a colour class by selecting the colour-class label on a photo
in a preview window or a contact sheet
(<https://docs.camerabits.com/support/solutions/articles/48001252564-color-class-ratings>).

**7. The directory tree's pointer and its keyboard disagree, and neither is
documented.** `src/ui/tree.rs:262-268`: left click expands or collapses, right
click opens the folder. There is also a whole hardcoded keyboard nobody is told
about — `ArrowDown`/`ArrowUp` move the highlight, `ArrowRight` opens the node,
`ArrowLeft` closes it, `Space` toggles it, and `Enter` opens the folder
(`src/ui/tree.rs:277-307`). None of it is in the registry, on the cheat sheet or
in the README, whose only mention of the tree is one row saying `T` opens it
(`README.md:764`); the heading is "Directory Tree" and carries no hint
(`src/ui/tree.rs:222`). A new user left-clicks a folder and watches it expand,
repeatedly. Microsoft's guidance is that a context menu must never be the only
route to a command
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus>), and
Apple's is the same — "Always ensure that contextual menu items are also
available as menu commands"
(<https://leopard-adc.pepas.com/documentation/UserExperience/Conceptual/AppleHIGuidelines/XHIGMenus/XHIGMenus.html>).
The coupling between the pointer and the highlight is half-made rather than
absent: opening a row assigns `selected_index` to it (`src/ui/tree.rs:132`),
closing one assigns only when the highlight was inside the subtree being
removed, and then to the row that was clicked (`:110-111`). So after folding a
sibling away, the row under the pointer and the row `Enter` opens are different
rows, and nothing on screen says which is which. A left click should open the
folder and move the highlight to it in every case; §7.6 gives the row its menu.

Two further faults are the secondary button's, and are argued where the menus
are. A right-click on the photograph or on a thumbnail registers no popup at all
rather than an empty one, because `show_context_menu` returns before registering
anything when the list is empty (`src/actions/user_action.rs:147-149`) and both
lists default to empty (`src/config/defaults.rs:165-167`, wired at
`src/config/mod.rs:387` and `:457`); and the one menu that does exist, on the
zoom percentage, hangs off a `Label` with no chevron, no tooltip and no cursor
change (`src/view/image_view/bottom_bar.rs:275-307`). §7.1 and §7.9 own both.

### 9.3 What each gesture should do

The governing decision: **the pointer gets the same treatment as the keyboard — a
default that is right for a photographer, a name in a list, and a way to change
it.** Not more gestures for their own sake. The counterweight worth keeping in
mind is IrfanView's author, who has argued against making double-click
configurable on the grounds that "more options are not always a good move as they
make programs harder to support" — quoted second-hand from search results,
because the forum thread itself returns HTTP 502
(`irfanview-forum.de/forum/program/support/11816-`). The answer to it is that the
number of *gestures* stays small and fixed; it is the *mapping* that opens up,
and the vocabulary it opens onto is the registry's, which already carries a
sentence per command.

The behaviour below is this chapter's. The field names and the shipped defaults
are §6's.

| Gesture | Image view | Contact sheet | Filmstrip | Notes |
|---|---|---|---|---|
| **Left click** | Nothing (reserved) | **Select**, move the cursor | Select, move the cursor | The sheet stops leaving on a click |
| **Left double click** | Toggle fit ↔ 100 %, or fullscreen, or nothing | **Open** in the image view | Open in the image view | The mapping is the setting: Picview offers Toggle Zoom / Full screen / Close window / None (<https://picview.chitaner.com/blog/mouse_keyboard_trackpad/>), and ImageGlass #471 asked for fullscreen here specifically *instead of* the 100 % snap (<https://github.com/d2phap/ImageGlass/issues/471>) |
| **Left drag** | Pan, when there is slack | Rubber-band select | — | Never both on one surface; see below |
| **Middle click** | Nothing, and rebindable to any command | Nothing, and rebindable | Nothing, and rebindable | The shipped default is §6.5's, and it is "nothing"; that is how the GNOME objection below is answered |
| **Middle drag** | Pan, always, even when fitted | Autoscroll | Autoscroll | The one gesture that pans regardless of slack |
| **Right click** | Context menu, on the press | Context menu | Context menu | Opening on the press is what stops a right drag swallowing the menu; the button the pan answers to becomes `mouse.drag`'s (fault 4, §6) |
| **Side button 1 (Back)** | Previous photograph | Previous photograph | Previous | `Extra1` already arrives (egui-winit-0.33.0 `src/lib.rs:1111`) |
| **Side button 2 (Forward)** | Next photograph | Next photograph | Next | `Extra2`, ditto |
| **Wheel** | One photograph, or scroll, or zoom, or nothing | Scroll the sheet | Scroll the strip | One job only, chosen in settings |
| **Ctrl + wheel** | Zoom about the pointer | Thumbnails per row | Cell size | Fixed by convention: GNOME's pointer table gives Ctrl+wheel to zoom in scrolling views (<https://developer.gnome.org/hig/guidelines/pointer-touch.html>). Already true in both views today |
| **Shift + wheel** | Ten photographs at a time | Ten rows | — | The same step `PageUp`/`PageDown` use (`PAGE`, `src/view/image_view/mod.rs:49`). Requires taking Shift back from egui (fault 3) |
| **Alt + wheel** | Pan horizontally when zoomed | — | — | Directory Opus uses Alt+wheel for horizontal scroll (<https://www.gpsoft.com.au/help/opus12/Documents/Prefs/Viewer_Mouse_Buttons.htm>). Also needs egui's `vertical_scroll_modifier` moved off Alt |
| **Hover** | Grab cursor when there is slack; nothing when there is not | Name, marks and dimensions | Name and marks | The cursor is the only honest signal that a drag will do something |

Six points of substance behind that table.

**The wheel's direction must be a setting, and the default must be consistent
with the sheet.** Wheel-down moves forward in avis-imgv's own contact sheet
(`src/view/grid_view/mod.rs:274`) and in every list widget the toolkit provides;
the image view should agree, which means the present default is the one that
changes. A reverse flag covers the people whose muscle memory says otherwise, at
the cost of one boolean. The wheel's *job* is a four-way choice — navigate,
scroll, zoom, nothing — because that is the shape every viewer that has solved
this converges on: Picview's three ("zoom (default), page, move"), Directory
Opus's three plus an accumulate option, the modern Windows Photos app's two radio
buttons. One trap to avoid, from nomacs #1281: when the wheel's usual job is
turned off it must fall back to something, not to nothing
(<https://github.com/nomacs/nomacs/issues/1281>).

**The modifier wheel gestures are not free.** Shift and Alt are both spoken for
by egui before this crate sees an event (`src/input_state/mod.rs:115-116`,
`:451-462`). Claiming them means either writing `Options::input_options` at
startup (`src/memory/mod.rs:281`) or reading `Event::MouseWheel` off `RawInput`
directly. Both are a few lines, but the plan should say so rather than pretend
the modifiers are lying about unused — and it should say which of the two is
contested. egui's own documentation for `horizontal_scroll_modifier` reads "The
default is SHIFT, and it is STRONGLY recommended to NOT change this"
(`src/input_state/mod.rs:86`); `vertical_scroll_modifier`, which is the Alt one,
carries no such line (`:89-92`). So Alt can be taken quietly and Shift cannot.
If the argument for a Shift + wheel step does not carry, the honest alternative
is to leave `InputOptions` alone and read the wheel event before egui folds the
axis, which changes nothing for any other widget.

**Drag-to-pan and drag-to-select never need to share a button, because they never
share a surface.** The contact sheet has nothing to pan, so a left drag there is
always a selection. What must *not* be adopted is the size-dependent rule Picview
documents — "when image exceeds window size, dragging moves within the image
instead" — which is the invisible mode that produced the XnView confusion in
fault 4. Instead: the left drag always claims the pan in the image view, the
cursor says whether there is anywhere to go, and the middle drag pans
unconditionally so that a fitted photograph is not a dead surface — the request
in nomacs #919, "allow panning when scaled image is smaller than viewport"
(<https://github.com/nomacs/nomacs/issues/919>). Contact-sheet cells are to carry
both a rubber-band drag and a right-click menu, and egui has a reported quarrel
with that combination (§7.12), so it is to be tested before it is designed around;
if it fails, the selection drag belongs on the background rather than on the
cells.

**Done, and the image view does rubber-band after all.** The paragraph above
assumed there was nothing to mark out on a photograph. There is: the part of it
a person wants magnified, and the part of it they want on the clipboard
(`src/view/image_view/area/mod.rs:1`). Where the objection lands is on the
*rule*, not on the gesture, and it is answered rather than ignored — a
size-dependent meaning is an invisible mode only while nothing on screen says
which of the two the button is about. The pointer says it, and says it before
the first rectangle exists: a cross wherever a drag would mark
(`area/pointer.rs:147`), the arrows for a side wherever it would move one, and
the ordinary arrow wherever it would pan. Picview's fault was the silence, not
the rule. The half of the rule the plan was right about survives untouched: the
drag is only ever *given* to the marking where the canvas was already clamping
every pan to nothing (`area/view.rs:140`), so nothing that used to move the
photograph has stopped moving it, and `mouse.mark_area` is there for the two
photographers who would answer differently (`src/config/mouse.rs:201`).

**The middle button is contested, and that is why it must be bindable to
nothing.** GNOME says outright that "it is not recommended to make use of this in
app designs" (<https://developer.gnome.org/hig/guidelines/pointer-touch.html>),
and nomacs #1188, "Close the app by wheel mouse button", was closed as not
planned (<https://github.com/nomacs/nomacs/issues/1188>). Against that,
Directory Opus makes the middle button as configurable as the left one,
FastStone toggles fullscreen with it and IrfanView exits fullscreen with it.
Shipping it bindable, with "nothing" as the default value, satisfies both
positions; making it the only route to anything does not.

**Double-click must never be the only route to anything.** GNOME's pointer
guidance says actions that are physically demanding, "such as double-clicking or
chording", should be avoided
(<https://developer.gnome.org/hig/guidelines/pointer-touch.html>); on a trackpad
the secondary click is a two-finger tap and the middle button does not exist at
all. Every gesture in the table above is a second or third route to a command
that already has a key.

**The side buttons must fire on the down-stroke.** nomacs #451 records what
happens otherwise: a viewer that waits to see whether a side-button click is a
double-click makes navigation feel "slow", and a double press still advances only
one frame (<https://github.com/nomacs/nomacs/issues/451>). Side buttons get no
double-click meaning, ever. Whether Windows delivers `MouseButton::Back` and
`Forward` at all depends on the driver — the XnView side-button thread records
inconsistent behaviour across mice — so the binding must be visible in the editor
even on a machine where nothing ever arrives on it.

### 9.4 The menus the pointer should reach

Six surfaces carry this chapter's argument: the photograph, a contact-sheet cell,
a filmstrip cell, the bottom bar, a panel or bar, and a control that has a
setting behind it. They are a subset of the roughly twenty surfaces §7 maps, and §7 owns
every entry on every one of them, the ordering rule, the rule that no menu is
nested, and the `More settings… (<page name>)` row that closes each one.

Two things in that map belong to this chapter. First, the menu on a control:
right-clicking a thing that has a setting behind it offers *Bind a key…* beside
its settings rows, which makes the control the route to its own key. That is the
direct answer to the ImageGlass request to "change keybinds by ctrl+right
clicking on the menu and picking them on the GUI instead of having to edit
configs", which the maintainer granted in principle — "Yes, I will add UI for
hotkey setting" (<https://github.com/d2phap/ImageGlass/discussions/1702>).
Second, every one of those menus opens on the press rather than the release, and
stops competing with the pan for the right button (fault 4). Microsoft's toolbar
guidance is to display a context menu "on right-click on mouse down, not mouse
up" (<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-toolbars>),
which removes the ambiguity at its source instead of tuning a threshold.

That last one is not what the ready-made call does, and the plan should not
pretend otherwise. `Response::context_menu` is `Popup::context_menu`, which
opens on `response.secondary_clicked()` (egui-0.33.0
`src/containers/popup.rs:248-251`), and `secondary_clicked` is
`clicked_by(PointerButton::Secondary)` (`src/response.rs:178-180`) — a
release-time event, decided by the same `max_click_dist: 6.0` and
`max_click_duration: 0.8` that fault 4 is about (`src/input_state/mod.rs:111-112`).
Opening on the press means setting the popup's open state rather than asking for
a click: watch `i.pointer.button_pressed(PointerButton::Secondary)`
(`src/input_state/mod.rs:1417`) while `response.contains_pointer()`, and draw
with `Popup::menu(&response).at_pointer_fixed()`, handing the press to
`open_memory` (`src/containers/popup.rs:237-243`, `:308-313`, `:341-344`). That
is one helper of about thirty lines, written once and called wherever §7.12's
table names `Response::context_menu`. It changes what a menu is built on, not
what a menu costs, so §7's arithmetic stands.

The keyboard route to any of them is Shift+F10; §7.10 owns the reason the Menu
key cannot be read at all. On the general question of how many routes one command
may have, §11.5 rules that several routes to one control are what everybody
endorses and several copies of one control are where it goes wrong; the ceiling
on a single menu is §7.2's twelve rows, and the rule that a stepped key keeps its
meaning when a continuous control is added beside it is §5.8's.

### 9.5 The keyboard editor, for people who open it

`src/ui/keys.rs` is one of the two settings surfaces in the program — the other
reaches three of the five slideshow fields (§3.1) — and it is behind a menu bar
hidden until `F1` (`src/app/mod.rs:201`, `src/config/defaults.rs:68-70`,
`src/app/panels.rs:67-77`). Ten things are wrong with it.

| # | What | Where | What to do |
|---|---|---|---|
| 1 | **No per-row reset.** The only reset is "Put everything back to the defaults", with no confirmation, which discards every customisation to fix one row. | `src/ui/keys.rs:111-121` | A reset glyph on each row, a reset on each section heading, and a confirmation on the global one |
| 2 | **No way to clear a binding.** A row can be rebound but not unbound. The word "unbound" is what the row falls back to when `Binding::get` returns `None` (`src/ui/keys.rs:143`, `src/config/bindings.rs:43-44`), which only a hand-edited `tags.sc_rating` or `tags.sc_label` shorter than the six and five rows the registry pushes can cause (`src/config/bindings.rs:420-436`) — and `set` on such a row then silently does nothing (`:52-61`). | `src/ui/keys.rs:140-144` | The armed row takes `Delete` or `Backspace` as "no key" |
| 3 | **Clash detection is scoped to a section, so the collisions that matter are never reported.** "General" is live in every mode — `input::collect` runs unconditionally every frame (`src/app/mod.rs:807`, `src/app/input.rs:117-140`) — so a General binding colliding with an Image-view or Gallery one is a real clash and is skipped by design. The test at `src/ui/keys.rs:333-348` enshrines exactly that case: it puts **Quit** on the Gallery's scroll key and asserts no clash. | `src/ui/keys.rs:169-186`, `:333-348` | Compare by *scope* — where a binding is actually read — not by the heading it is filed under. Image-view against Gallery stays a non-clash, because those two are never on screen together (`src/app/views.rs:80-93`), which is what the doc comment at `keys.rs:169-170` already says |
| 4 | **No search, no filter, no mode view.** Sixty-nine rows in four headings — General 20, Image view 22, Gallery 12, Ratings and tags 15 (`src/config/bindings.rs:67`) — and no way to ask "what is on `Space`?" or "what works in the contact sheet?", where forty-seven of the sixty-nine are live: General, Gallery and Ratings and tags together, which is the split the cheat sheet already makes for that mode (`src/ui/cheat_sheet.rs:30`). In this list the twelve filed under *Gallery* cannot be told apart from the thirty-five that merely also work there. | `src/ui/keys.rs:78-107` | A filter box that matches name, description **and key**, and a "show only this mode" toggle |
| 5 | **A save is never confirmed.** `State::status` is declared and read, and written nowhere in the crate — `grep -n status src/ui/keys.rs` returns three lines, one declaration and two reads. A failed save reaches the notice band; a successful one says nothing. | `src/ui/keys.rs:30`, read at `:123-124`; failure path at `src/app/settings.rs:104-110` | Write it, or delete the field and put the confirmation in the notice band with the failures |
| 6 | **The editor is not modal and the viewer keeps listening.** `keys::show` opens a plain window and never mutes (`utils::set_mute_state` is called from `src/app/cull.rs` and `src/app/input.rs` only), and clicking a row does not focus it either, so `are_inputs_muted` stays false. `input::collect` runs at `src/app/mod.rs:807`, the editor at `:840`, the views at `:846`, and `shortcut::consume` removes the event (`src/config/shortcut.rs:80-93`). So arming a row and pressing `Delete` sends the photograph on screen to the bin — `sc_delete` defaults to `delete` (`src/config/defaults.rs:435-437`) — and the capture silently fails; pressing an Image-view key both rebinds *and* fires. | `src/ui/keys.rs:49-55`, `src/app/mod.rs:807`, `:840`, `:846` | Mute while a row is armed, and read the key before anything else does |
| 7 | **The editor cannot produce two of the five modifiers.** `captured()` emits only `ctrl`, `alt`, `shift`, folding `modifiers.command` into `"ctrl"`; the configuration accepts `cmd` and `mac_cmd`. | `src/ui/keys.rs:266-278` vs `src/config/shortcut.rs:11-12`, `:170-171` | Emit `cmd` on macOS; or drop `cmd`/`mac_cmd` from the file format, since `cmd_ctrl_matches` already reconciles them (`shortcut.rs:108`) |
| 8 | **One shortcut field the editor cannot reach: `UserAction::shortcut`.** All sixty `sc_*` fields in `Config` are covered — fifty-eight of them one row each, and the two `Vec<Shortcut>` fields contributing eleven more — but a user action's key is a `Shortcut` living inside a `Vec<UserAction>` on `ImageViewConfig` only, invisible to the editor, to the cheat sheet and to clash detection. | `src/config/mod.rs:542`, `:385`; registry at `src/config/bindings.rs:85-439` | User actions become rows in the same table |
| 9 | **The consequence of 8, shipped in the example.** `examples/config.json:34-41` binds a user action to plain `delete`, and `:261-264` leaves `sc_delete` on `delete`. `input::collect` runs first and consumes the event, so the shipped example's trash action can never fire, and no warning exists because user actions are not in the clash registry. The same trap exists inside the image view, where built-in bindings are consumed at `src/view/image_view/input.rs:100-122` and user actions at `:127`, `:131-141`. | `examples/config.json:34-41`, `src/app/mod.rs:807`, `src/config/shortcut.rs:80-93` | Fixed by 8 |
| 10 | **The same binding is spelled two ways.** Defaults store `"ArrowRight"` (`src/config/defaults.rs:214-216`); the editor writes `Key::name()`, which is `"Right"` (`src/ui/keys.rs:278`; egui-0.33.0 `src/data/key.rs:485`, `:489`). Both parse, so nothing breaks — but a hand-edited file and an editor-written file disagree, and `examples/keys.txt:22-23` documents `pageup` / `pagedown`, which `capitalize_first_char` turns into `Pageup` / `Pagedown` and `Key::from_name` rejects, since it accepts `"PageUp"` and `"PageDown"` and nothing else (egui-0.33.0 `src/data/key.rs:344-345`), silently collapsing the binding to the unreachable `F20`-plus-every-modifier sentinel (`src/utils.rs:78-84`, `src/config/shortcut.rs:138-151`, `:178-187`). Those are the only two of the seventy-two key names in `keys.txt` that fail; the file's other eight lines are two headings, a blank and the five modifiers. | `src/utils.rs:78-84`, `src/config/shortcut.rs:178-187` | Canonicalise on write, accept both on read, and regenerate `keys.txt` from `Key::ALL` so it cannot drift |

The corpus says every one of these is a real failure mode elsewhere. nomacs #71
is "no indication on how to remove a keyboard shortcut", and the workaround found
by its reporter was to cause a conflict on purpose
(<https://github.com/nomacs/nomacs/issues/71>). nomacs #186 accepts a rebind,
writes it, and never reads it back
(<https://github.com/nomacs/nomacs/issues/186>). nomacs #328 leaves shortcuts out
of "Export Settings" (<https://github.com/nomacs/nomacs/issues/328>).
ImageGlass #608 ran three years and was closed by telling the user to edit
`igconfig.json` and find the valid key names by reading the source
(<https://github.com/d2phap/ImageGlass/issues/608>).

Two adjacent problems belong here rather than in a section of their own.

**A focused text field silences the entire viewer, and nothing says so.**
`are_inputs_muted` is `explicit-mute || memory.focused().is_some()`
(`src/utils.rs:41-46`). The filter bar has three text fields
(`src/ui/filter_bar.rs:229-247`), the tag panel one
(`src/ui/tag_panel/mod.rs:207-215`), the folder jobs eight
(`src/view/organize/controls.rs:61`, `:100`, `:104`, `:121`, `:127`, `:141`,
`:148`, `:161`); `Escape` is the only way out (`src/app/mod.rs:802-804`) and
`Alt+Q` is the only shortcut that survives (`src/app/input.rs:93-96`, default at
`src/config/defaults.rs:64-66`). nomacs has issues in this family filed by a
collaborator on the project — Ctrl+O doing nothing until you click the central
widget, Ctrl+Z working only with the viewport focused
(<https://github.com/nomacs/nomacs/issues/1190>,
<https://github.com/nomacs/nomacs/issues/1234>). The fix is cheap: when a field
has focus, the bar says so and names the key that gets out (§10).

**The "go to" field is deliberately unreachable from the keyboard.** It
surrenders focus if it gains it without a click, because `Tab` means "the other
pane" while comparing (`src/view/image_view/bottom_bar.rs:218-225`). The
reasoning is sound and the result is a control that cannot be operated without a
mouse. It should have its own key instead, in the same table as everything else.

### 9.6 Commands that exist with no route anybody will find

| Command | The only way to it | Where |
|---|---|---|
| Zoom to 200 / 75 / 50 / 25 % | Right-click the zoom percentage — no affordance of any kind (§7.9) | `src/view/image_view/bottom_bar.rs:283-304` |
| Open a folder from the tree | Right-click the row, or `Enter` on a highlight that only an expanding click or the arrow keys move | `src/ui/tree.rs:266-268`, `:303-307`, `:132` |
| Open Folder, Open Files, Send rejected to the bin | The File menu, behind `F1` | `src/app/panels.rs:33-54`, `src/app/mod.rs:201` |
| Keyboard…, Slideshow… | The Settings menu, behind `F1` | `src/app/panels.rs:67-77` |
| Go straight to a mode | The Mode menu, which calls `set_mode` directly. `Command::SetMode` exists and is handled but is **never constructed** — the only keys are `F2` to cycle and `Backspace` to toggle | `src/app/panels.rs:56-65`, `src/app/settings.rs:40`; declared `src/app/input.rs:18`, handled `src/app/mod.rs:694`, produced nowhere |
| `Home`, `End`, `PageUp`, `PageDown` | Nothing on screen mentions them; they are outside the registry by design | `src/view/image_view/input.rs:108-122` |
| The tree's arrows, `Space` and `Enter` | Nothing anywhere mentions them | `src/ui/tree.rs:277-307` |
| `Tab`, `/`, `Escape` while comparing | One sentence inside the *Compare* row's description, which the cheat sheet does not render | `src/config/bindings.rs:308`; the sheet prints key and name only, `src/ui/cheat_sheet.rs:88-91` |
| `?` itself, and `F10` | Nowhere on screen | `src/app/input.rs:102-115` |
| "Choose a folder…" in the destination panel | Click only, while every other row in that panel has a key — the nine slots a digit, the repeat row `Enter` | `src/ui/destinations.rs:97-99`; the keys are `:118` and `:122-140`, the rows they answer `:74-83` and `:85-94` |
| Clear the filter, change the sort key or direction, fold or open every stack | Filter-bar clicks | `src/ui/filter_bar.rs:49-57`, `:229-247`, `:249-262` |
| Add a keyword | Tag-panel clicks. `K` opens the panel (`src/config/defaults.rs:355-357`) and nothing calls `request_focus` on the search box | `src/ui/tag_panel/mod.rs:207-215` |
| The filmstrip | `Ctrl+T` toggles a flag, and the strip refuses to draw while `filmstrip_height` is `0.0`, which is the default. The cheat sheet advertises the key. | `src/app/views.rs:137-141`, `src/config/defaults.rs:299-301`, `src/config/bindings.rs:100-105` |

The pattern is the one the whole corpus reports: **the capability exists and the
route to it is hidden.** JPEGView's answer to "please make the mouse wheel change
image" was "change `NavigateWithMouseWheel=true` in the INI file" — the feature
had shipped, and the user could not find it
(<https://sourceforge.net/p/jpegview/feature-requests/94/>). §10 owns what the
program should say about itself; the point here is narrower and mechanical. A
command whose only route is a gesture nobody can see is a command with one home,
and one home is not enough.

### 9.7 Commands a user will look for and not find

| What they press or click | What exists | Evidence |
|---|---|---|
| Right-click for **Rotate** | No rotate command anywhere. EXIF orientation is honoured by permuting texture coordinates (`src/view/texture.rs:50-65`), and `Rotate90Cw` is an enum variant in `src/metadata/orientation.rs:13`, not a verb | grep for a rotate command over `src/`: nothing outside orientation labels and `imageops` |
| **Copy the path**, **Reveal in file manager**, **Open with** | None. `clipboard` occurs zero times in `src/` | measured |
| **Double-click** to toggle fit ↔ 100 % or to go fullscreen | Nothing | no `double_clicked` in `src/` |
| **Ctrl+O** to open a folder | Nothing; the menu only | `src/app/panels.rs:33-42` |
| **Shift+F10** for a context menu | Nothing, and egui provides no keyboard route to a context menu either (§7.10) | no handler in `src/` |
| **Drag a file onto the window** to open it | Nothing, though egui-winit already collects the paths | egui-winit-0.33.0 `src/lib.rs:468-473`; no `dropped_files` in `src/` |
| **Middle-click** to close, to go fullscreen, or to open in a new pane | Nothing | no `middle_clicked` in `src/` |
| **Side buttons** for previous / next | Nothing, though the events arrive | egui-winit-0.33.0 `src/lib.rs:1111-1112` |
| Click a **star in the bar** to rate | Inert label | `src/view/image_view/bottom_bar.rs:202-204` |
| Click **`Filling`** or **`Comparing`** to turn it off | Inert label; the keys are `Ctrl+M` and `Escape` and the bar names neither | `bottom_bar.rs:144-155` |
| Hover the **truncated file name** to read it | No tooltip, though the side panel gives one to its long values | `bottom_bar.rs:161-164` vs `src/app/panels.rs:161-162` |

Two of these deserve to be in the plan on their own account. **Rotate** is the
single most-expected verb after delete in a viewer, and nomacs #799 is the
angriest issue in the corpus precisely because nomacs implements it by silently
rewriting the file on disk — "the file on disk is silently modified… all without
the knowledge or consent of the user"
(<https://github.com/nomacs/nomacs/issues/799>) — so avis-imgv should implement
it the other way, as an orientation written to the sidecar and composed with the
EXIF one before `to_texture` is called (`src/view/texture.rs:34`, `:50-65`). The
composition also has to reach `displayed_size` (`src/view/texture.rs:71-77`) and
the decoder's stored dimensions (`src/decoder/mod.rs:264-266`), which is the part
that is not free. **Drag a file onto the window** is nearly free: egui-winit
already pushes every dropped path into `RawInput::dropped_files`
(egui-winit-0.33.0 `src/lib.rs:468-473`), and the collection-opening code already
exists (`src/app/mod.rs:297-302`).

### 9.8 One table of commands, four homes each

The registry in `src/config/bindings.rs` is already most of what is needed. It
holds a section, a name, a sentence of prose and a pair of accessors reaching the
configuration field, for the sixty-nine rows `all()` builds
(`src/config/bindings.rs:26-36`, `:85-439`), and it is the single source for both
the editor (`src/ui/keys.rs:61`) and the cheat sheet
(`src/ui/cheat_sheet.rs:48`). Its own module comment already states the intent:
"Adding a shortcut to the configuration and not to this list means it cannot be
changed from the interface, so the two are meant to be edited together"
(`src/config/bindings.rs:8-9`).

The proposal is to finish the job it started: make that table the definition of a
command, not merely of a key. §3.1 and §4.7 describe the same table's other face,
as the index behind the settings window; this section is the command side of it.

```rust
pub struct Binding {
    /// Stable across renames; what the configuration file names.
    pub id: &'static str,
    /// Where it is read, which is what a clash is about.
    pub scope: Scope,          // Everywhere | ImageView | Gallery | Compare | Overlay | FolderJob
    /// Where it is shown, which is a different question.
    pub group: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// As today.
    pub key: Field,
    /// New: a pointer gesture, in the same file, edited in the same window.
    pub mouse: Option<GestureField>,
    /// New: which menu-bar menu it appears under, and in what order.
    pub menu: Option<MenuHome>,
    /// New: which right-click menus offer it.
    pub context: &'static [Surface],
}

pub struct Gesture {
    pub button: Button,        // Left | Middle | Right | Back | Forward | Wheel
    pub kind: Kind,            // Click | DoubleClick | Drag | WheelUp | WheelDown
    pub modifiers: Vec<String>,// the same vocabulary as a Shortcut
}
```

What that buys, item by item.

- **Adding a command becomes one row.** Today it is six edits: a `Command`
  variant (`src/app/input.rs:12-64` or `src/view/image_view/input.rs:25-67`), a
  field in `src/config/mod.rs`, a default in `src/config/defaults.rs`, a row in
  `src/config/bindings.rs`, a line in the collector's `bindings` array
  (`src/app/input.rs:117-140`), and an arm in `App::apply`
  (`src/app/mod.rs:684`) — plus a menu entry in `src/app/panels.rs` and a table
  row in `README.md` if it is to be found. With the table owning the homes, the
  collector, the menus, the context menus and the cheat sheet are generated from
  it, and what is left is the variant, the arm and the row.
- **The mouse becomes configurable at all**, in the same window, in the same
  file, with the same clash reporting. This is what nomacs #347 asked for — move
  the mouse settings *into* the shortcut manager so that any combination, "e.g.
  Shift + Right Mouse Click", can be bound
  (<https://github.com/nomacs/nomacs/issues/347>). It is also where ImageGlass
  arrived, the long way round: version 9 beta 2 announced `MouseClickActions` and
  `MouseWheelActions` as keys in `igconfig.json`
  (<https://imageglass.org/news/announcing-imageglass-moon-9-0-beta-2-78>), only
  the wheel got a settings screen at first, and a later release covered the
  clicks, the wheel click and the side buttons as well — that last step is from
  search results rather than the announcement page itself. Config file → partial
  UI → full UI is the path; avis-imgv is at the start of it and can skip to the
  end.
- **Clash detection becomes correct.** Two bindings clash when their scopes can
  be live at once, which `Scope` states and the section heading does not. That is
  fault 3 of the editor: today a General binding — live in every mode, because
  `input::collect` is called unconditionally (`src/app/mod.rs:807`) — can share a
  key with any Image-view or Gallery binding and nothing is said, and the test at
  `src/ui/keys.rs:333-348` asserts that silence is correct. It is not: `Quit` on
  the Gallery's scroll key means that pressing the scroll key quits. `input::collect`
  runs first (`src/app/mod.rs:807`) and `shortcut::consume` takes the event out of
  `input.events` (`src/config/shortcut.rs:80-93`), so by the time the sheet asks
  for the same key (`src/view/grid_view/mod.rs:298`, drawn at
  `src/app/mod.rs:846`) there is nothing left: the folder never scrolls, and
  nothing warns. The cases the section rule gets *right* — `Space` as zoom step
  (`src/config/defaults.rs:211-213`) against `Space` as "pick this one out"
  (`:312-314`), `PageDown` as ten photographs
  (`src/view/image_view/input.rs:111`) against half a row
  (`src/config/defaults.rs:307-309`), `Plus`/`Minus` as zoom (`:253-258`) against
  thumbnails per row (`:318-323`) — must keep being got right, because the two
  views are never on screen together (`src/app/views.rs:80-93`). A scope states
  both facts; a heading only happens to.
- **Every context menu becomes a filter over the table**, so a menu cannot
  contain a command that has no key, and a key cannot exist with no menu home —
  the redundancy rule Apple, Microsoft and NN/g all state and which §7 enforces
  surface by surface.
- **The cheat sheet gains the mouse and the hardcoded keys**, because they stop
  being hardcoded: `Home`, `End`, `PageUp`, `PageDown`, `Tab`, `/`, `Escape`,
  `?`, `F10`, the tree's six and the grid's arrows all become rows with a scope
  and a default, and the ones that should not be rebindable are marked rather
  than hidden (§10.5). `/` in particular is unreachable on the Slovak, German and
  French layouts where slash is a shifted character, and today there is no way to
  move it (`src/view/image_view/input.rs:116`, read with `Modifiers::NONE` at
  `:119`).
- **The configuration file gets one section instead of sixty scattered fields.**
  The `sc_*` fields stay as they are for one release and are migrated by
  `src/config/migrate.rs`, which already moves a binding forward when a default
  moves (`:36-49`, `:55-75`) and already reports it in the notice band
  (`src/app/mod.rs:244-246`).

Two things the table must not do. It must not become the place where *every*
setting lives — it is a command registry, and a command is a verb; the fifty
non-shortcut settings belong in the settings window (§3.2). And it must not be
allowed to grow a row with no key, no gesture, no menu and no context home,
because that is how a command becomes undiscoverable in the first place; a test
that asserts every row has at least two homes costs four lines and prevents the
entire class.

### 9.9 What finishes this chapter

*Finished when:* right-clicking the photograph, a contact-sheet cell, a filmstrip
cell, the bottom bar or any panel produces a menu on a stock install with nothing
configured, and the menu opens on the press; every gesture in §9.3's table can be
changed from the same window as the keys, with "nothing" a legal value for all of
them; the wheel does one job at a time, moves forward the same way in both views,
and both its job and its direction are settings, with Shift and Alt meaning what
§9.3 says they mean rather than what egui's defaults left them meaning; a single
click in the contact sheet selects and a double click opens; the side buttons walk
the folder and a file dragged onto the window opens; a left click in the directory
tree opens the folder and moves the highlight the keys act on to that row; the
keyboard editor can reset one row, clear one row, be searched, be filtered to a
mode, confirm that it saved, report a General-versus-Image-view clash, and capture
`Delete` without sending the photograph to the bin; the cheat sheet lists the
mouse; and no command is reachable by exactly one route, with a test that says so.

## 10. A program that explains itself

Everything in this chapter follows from one measurement. Across 139 source
files and 40,504 lines there are **33 places where the viewer explains
something on hover** — 31 `on_hover_text`, one `on_hover_text_at_pointer`
(`src/view/grid_view/mod.rs:394`) and one `on_hover_ui`
(`src/view/organize/rename.rs:42`). There is no `on_disabled_hover_text`
anywhere, so a control that is greyed out never says why. The 33 are in **11
files**; the other 128 have none. Thirty-nine files name an `egui::Ui` or an
`egui::Context`, and twenty-eight of those thirty-nine carry no hover at all
(§1). Two of the eleven — `src/ui/keys.rs` and `src/ui/destinations.rs` —
contribute four of the thirty-three between them. (Anyone recounting this
should match `on_hover`. A word-boundary match on
`on_hover_text\b\|on_hover_ui\b` returns 32 in 10 files, because it misses
`on_hover_text_at_pointer`.)

That is not a small number by accident of style. It is small in a particular
shape: twenty of the thirty-three are in three files — the filter bar (8), the
group panel (7) and the tag panel (5) — while the contact sheet has one in 706
lines (`src/view/grid_view/mod.rs:394`), the image view's own 714 lines have
none (`src/view/image_view/mod.rs`), the folder-job sort and filter controls
have none (`src/view/organize/controls.rs`), and the whole cache readout has
none (`src/app/panels.rs:168-248`). That shape is the worst one to be in.
Nielsen Norman's tooltip guidelines record the failure directly: on a site
where only some icons carried tooltips, users stopped expecting them and
missed the ones that existed
(https://www.nngroup.com/articles/tooltip-guidelines/). Microsoft's Win32
guidance says the same as a rule — *"If you provide tips for some objects, you
should provide them for all similar objects for which users are likely to want
supplemental information"*
(https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tooltips-and-infotips).
Thirty-three hovers spread this thinly buy less than they cost.

The second measurement decides what to do about it. The registry — the table
at `src/config/bindings.rs` that every key list is generated from — already
carries a written sentence for every binding, in the `description` field
(`:33-34`). It draws **69 rows**: 58 built by the `binding!` macro, of which
54 are outside the "Ratings and tags" section — a count the registry asserts
on itself at `src/config/bindings.rs:530-541` — plus five colour labels and
six star ratings pushed in loops (`:420-436`). Those 69 rows sit over 60
configuration fields, because `tags.sc_rating` and `tags.sc_label` are lists
rather than single keys (§3).

`grep -rn "\.description" src/` finds six call sites. Three of them draw a
binding's sentence, and all three are inside the keyboard editor
(`src/ui/keys.rs:135`, `:154`, `:157`); a fourth is a test
(`src/config/bindings.rs:476`); the remaining two belong to a user action's
own label and to `Motion::description()`. The cheat sheet, which is generated
from the same registry, renders only the key and the name
(`src/ui/cheat_sheet.rs:85-91`). The explanatory prose mostly exists. It is
not on screen.

### 10.1 What the README documents and the program never says

"In-app explanation" below means a label, a tooltip, a cheat-sheet row, a
notice or a panel line that a running user can reach. A key *binding* listed
on the cheat sheet explains the key, not the feature.

| README section | Feature | In the program | Where |
|---|---|---|---|
| How it works (`README.md:20`) | Five cache tiers, the priority queue, mip chains, the automatic preload trim | **Nothing.** Bare numbers with no vocabulary: "Decoded", "On the GPU", "Thumbnails standing in", "Metadata read ahead", "N ready to zoom into at full resolution", none with a hover | `src/app/panels.rs:168-248` |
| Metadata (`README.md:70`) | In-process EXIF/ICC/XMP, the container list | **Nothing.** No format list anywhere in the GUI; `formats::supported_extensions()` is used in exactly one place, as a file-dialog filter | `src/app/settings.rs:29` |
| Marks (`README.md:82`) | Why three axes rather than one | Partial. Panel headings and three hovers; the *reason* stars, flags and labels are separate is nowhere | `src/ui/tag_panel/mod.rs:112`, `:142`, `:170` |
| Tags with levels (`README.md:103`) | `Places\|Slovakia\|Tatras`, the tree, `lr:hierarchicalSubject`, "narrowing by Slovakia finds everything below it" | **Almost nothing.** The tree draws and the hover shows the path, but nothing says a bar-separated path may be *typed*, and nothing says the filter matches parents | `src/ui/tag_panel/mod.rs:196`, `:285`; the substring match is `src/view/narrow.rs:265-274` |
| `catalog_file` (`README.md:110`) | Import a keyword list; indentation is the hierarchy | **Nothing.** No importer, no file picker, and a list that fails to read is a `tracing::warn!` | `src/annotations/catalog.rs:54-62` |
| Advance after marking (`README.md:128`) | `Ctrl + Shift + A` | Partial. The status bar prints the bare word "Advancing" with no hover | `src/view/image_view/bottom_bar.rs:148` |
| Getting rid of them (`README.md:134`) | `Delete` to the bin, `Shift + Delete` outright, "the cursor stays where it is" | Partial. The modal wording is good; the cursor-stays rule is never stated | `src/app/cull.rs:137-166` |
| Somewhere else (`README.md:155`) | `Alt+M`/`Alt+C`, destinations that follow the shoot, "the same key twice repeats" | Partial. Slots and paths on hover; the double-tap skip is unexplained | `src/ui/destinations.rs:78`, `:89`; the behaviour is `src/app/cull.rs:293-303` |
| Taking it back (`README.md:168`) | `Ctrl + Z`, 200 steps (`src/organize/journal.rs:24`), and "says what it is about to do first" | **Nothing beyond one fading notice**, and the promise is not kept: `undo()` acts first and reports afterwards | `src/app/cull.rs:496-525` |
| Where they are stored (`README.md:181`) | Sidecar naming, the XMP field table, "rejecting clears the stars" | **Nothing.** A user wondering where a rating went has no in-app answer | — |
| Dependencies / Build (`README.md:209`, `:220`) | LibRaw, feature flags | **Nothing on screen.** Whether this build can develop raw goes to the log | `src/main.rs:38-41` |
| How fast is it (`README.md:283`) | `--benchmark` — the published figure is 43.6 images/s, 501 images in 11.50 s, median frame 2.70 ms, on 24-megapixel JPEGs on a 24-core Ryzen (`README.md:291`) | **The result goes to the log and the window closes.** Nothing is drawn | `src/app/chrome.rs:172-180` |
| Working on the whole folder (`README.md:304`) | Sort and filter semantics, "rules combine with and", "the order is what the counter follows" | Partial. Labels and seven hint texts; **zero tooltips** in the whole file | `src/view/organize/controls.rs` (hints at `:63`, `:106`, `:123`, `:129`, `:143`, `:150`, `:163`) |
| Templates (`README.md:324`) | The whole grammar, `$( … )`, `{{`, the legacy `#Tag#` (`src/metadata/template.rs:170-184`) | **Reachable in exactly one mode**: the rename box's hover and the *Insert…* menu beside it. The same grammar drives `image_view.name_format`, `image_view.overlay_format` and `grid_view.caption_format`, none of which has any GUI | `src/view/organize/rename.rs:42`, `:44-57`, `:141-155` over `src/metadata/template.rs:321-338`; the three fields at `src/config/mod.rs:348`, `:383`, `:482` |
| Bulk rename (`README.md:357`) | Collision detection, the temp-name swap, sidecars follow | Partial. `"{n} cannot be renamed"` in red and a per-row reason; the swap guarantee and the sidecar rule are unstated | `src/view/organize/rename.rs:104-109` |
| Shift capture time (`README.md:370`) | Which timestamps move; the in-place guarantee (`README.md:379`) | Partial. One good hover on the direction; "the maker notes and the pixels are untouched" — the thing that makes it safe — is unstated | `src/view/organize/timeshift.rs:46-49` |
| Group shots (`README.md:387`) | Four kinds and the rule for each; hash similarity; `hdr1`/`stack1` naming | Partial. A dropdown of four names, a "(read as X)" note, and seven hovers in the file — none of which says what a kind *is*, though the doc comments already do | `src/view/organize/group/mod.rs:139-143`, `:313`; the sentences are at `src/organize/group/classify.rs:19-26` |
| Comparing two frames (`README.md:424`) | `N`, `Tab`, `/`, shared zoom, "marks apply only to the focused pane" | Partial. One binding sentence, the word "Comparing" and a blue border. The border is never explained; the marks rule is never stated | `src/view/image_view/layout.rs:80`, `:127`; `src/view/image_view/bottom_bar.rs:149` |
| Narrowing the folder down (`README.md:446`) | `F3`, `\`, the `2/27 (+2)` count | **Good.** Eight hovers here and one on the count | `src/ui/filter_bar.rs`, `src/view/image_view/bottom_bar.rs:126` |
| Stacks (`README.md:465`) | `Ctrl+G`, the glyphs, "the sharpest frame stands for the run", "nothing is written" | Partial. Two hovers. **The four glyphs have no legend**, and the sharpest-frame rule (`README.md:491`) is nowhere | glyphs at `src/view/stacks.rs:362-368`, drawn at `src/view/grid_view/cell.rs:190-228`, a file with zero hovers |
| Slideshow (`README.md:507`) | Three motions | **The best-explained feature in the program**: each radio carries an indented sentence, and a footer explains the arrow keys | `src/app/panels.rs:110-117`, `:134`, text from `Motion::description()` at `src/config/mod.rs:514-523` |
| Changing the keys (`README.md:527`) | The editor | **Good.** An instruction line, per-row descriptions, clash notes | `src/ui/keys.rs:75`, `:135`, `:151-158` |
| Supported formats (`README.md:535`) | The list | **Nothing in the GUI** | — |
| Raw files (`README.md:541`) | `raw.source` preview vs develop, the resolution catch, LibRaw fallback | Partial. A `Preview Size` metadata row if configured; the six `raw.*` fields are JSON-only, and a build without LibRaw says so only in the log | `src/config/defaults.rs:124`, `src/main.rs:38-41` |
| Colour management (`README.md:578`) | lcms2, profile matching, the three bundled profiles | **Almost nothing.** `output_icc_profile` is JSON-only, and the only tag on screen that touches this is `Color Space` — the EXIF hint, not the embedded profile's name | `src/config/mod.rs:266`, `src/config/defaults.rs:126`, `src/metadata/icc.rs:31` |
| Raw and JPEG together (`README.md:690`) | Pairing rules, "everything acts on both" | Partial. The bare word "RAW+JPEG" with **no tooltip**, though the field's own doc comment says why it matters | `src/view/image_view/bottom_bar.rs:150`; the comment at `:65-70` |
| Configuration (`README.md:592`) | 110 settings | Sixty are keys and reachable through the editor; three more are reachable, all in the Slideshow window; forty-seven are reachable nowhere (§3). Nothing on screen names the file, which is `config.json` (`src/config/load.rs:16`) | Settings menu is `src/app/panels.rs:67-77`; `Config::path()` (`src/config/load.rs:14-17`) has three callers — `Config::save` itself (`src/config/load.rs:34`), `src/annotations/catalog.rs:190` resolving a relative catalog path, and a test at `src/logging.rs:191`. None of them puts the path on screen |
| Default shortcuts (`README.md:747`) | The key tables | **Good** — the cheat sheet is the in-app equivalent, except for the keys hardcoded outside the registry, which are in no in-app list at all | see below |
| Picking several out (`README.md:852`) | Selection semantics, "the first photograph decides the mark", selection survives filtering | Partial. A corner badge and an "Applies to N photographs" line; the two rules are nowhere | `src/view/grid_view/mod.rs:329`, `src/ui/tag_panel/mod.rs:73` |
| User actions and context menu (`README.md:875`) | Placeholders, no-shell splitting, callbacks | **Nothing, and worse:** both list defaults are empty, so `show_context_menu` returns before drawing and right-clicking a photograph does nothing at all (§7) | `src/config/defaults.rs:165-167`, `src/actions/user_action.rs:147-149` |
| Font (`README.md:909`) | Bundled Atkinson Hyperlegible | **Nothing.** Compile-time only, behind `#[cfg(feature = "custom_font")]` | `src/ui/theme.rs:20-21` |
| Tools (`README.md:915`) | `dump_metadata`, `develop_raw` | **Nothing**, correctly — they are CLI examples | — |

Two entries in that table are the whole argument in miniature. The slideshow
window is the only place in the viewer where a control carries a sentence of
prose next to it (`src/app/panels.rs:115`), and it is the least important
feature in the program. The cache readout is five tiers of the most
technically interesting thing the viewer does (`src/app/panels.rs:168-248`),
and it has no words at all.

**The keys that are in no list.** `?` and `F10` are hardcoded outside the
registry (`src/app/input.rs:113-115`, `:102-104`) and so appear in neither the
keyboard editor, which iterates `bindings::all()` (`src/ui/keys.rs:61`), nor
the cheat sheet, which reads the same table (`src/ui/cheat_sheet.rs:48`). They
are not alone. The image view's `Home`, `End`, `Page Up`, `Page Down`, `Tab`,
`/` and `Escape` are a fixed array (`src/view/image_view/input.rs:108-118`);
the contact sheet's arrows, `Home`, `End`, `Enter` and `Escape` are another
(`src/view/grid_view/mod.rs:560-565`, `:581-584`, `:605`, `:630`). The README
lists several of them (`README.md:767`, `:778`, `:784`); the running program
lists none. This is separate from the one *rebindable* field the editor cannot
reach, `UserAction::shortcut`, which is §9's.

There is one further contact-sheet gesture, and it is not a binding at all.
**Shift held while the arrows are walked extends the selection over everything
walked over** — listed in the README as `Shift + arrows | Pick out everything
walked over` (`README.md:842`), and the gesture that makes it possible to mark
two hundred frames from the keyboard. It is implemented in two halves.
The shift state is read on its own, before anything is consumed
(`src/view/grid_view/mod.rs:557`), and the arrows are then claimed with
`egui::Modifiers::NONE` (`:570`); whether the walk extends the run or starts a
new one is decided afterwards, at `:591-600`. The source says why the two
halves cannot be one: *"egui's own matching ignores a shift it was not asked
about: the arrow keys are claimed with no modifiers and would swallow the
shifted ones too, so whether shift was down has to be asked separately"*
(`:553-556`). So it is not a binding with a modifier on it; it is a modifier
read beside a binding, which is why no list in the program can show it and why
the README is the only place it is written down.

That distinction matters for the repair. A read-only registry row (§10.5) can
carry the fixed keys because each of them is one `consume_key` call with one
modifier set. The shift-extension is not, and giving it a row of its own would
put a second `Shift + ←` in the table beside the plain `←` that actually
claims the press, which is a clash the editor would then have to explain away.
It belongs on the arrow rows themselves, as a second line of the description —
"Hold `Shift` to pick out everything walked over" — so the sheet says it, the
editor says it, and nothing pretends it can be rebound separately from the
arrows it modifies.

Note also what the README does *not* document, which is where the GUI is
silent too: `image_view.overlay_corner`, `overlay_format`, `overlay_text_size`
and `grid_view.caption_format` (`src/config/mod.rs:345`, `:348`, `:350`,
`:482`) appear in no README table. They are the four settings that put words
on the photograph and under the thumbnails, two of them driven by the template
grammar — the features a photographer is most likely to want to change and
least likely to find. Half the configuration is undocumented at the
declaration as well: 57 of the 110 settings have no doc comment above them
(§3), so there is no sentence to lift even when a control is finally drawn.

### 10.2 The states the program does not have

There are four empty states and four loading states in the product. Three of
the eight draw nothing at all, two draw a bare spinner, and three draw a bare
label.

| State | What is drawn now | Where |
|---|---|---|
| No photographs at all | The words "No images here", centred — on the image view's grey backdrop (`src/view/image_view/layout.rs:12`, `#777777`), on the theme's panel fill in the contact sheet | `src/view/image_view/layout.rs:42`, `src/view/grid_view/mod.rs:265` |
| Filtered to nothing | "Nothing matches the filter" — the identical treatment | `src/view/image_view/layout.rs:44`, `src/view/grid_view/mod.rs:267` |
| Tag panel with nothing open | **Nothing whatever.** The function returns before drawing, so pressing `K` on an empty folder flips a flag and no pixel changes | `src/app/tagging.rs:99-101` |
| Metadata panel with nothing open | The word "Loading…", for ever | `src/app/panels.rs:147` |
| One photograph decoding | A bare `Spinner`, a third of the panel high, no text | `src/view/image_view/layout.rs:137` |
| One thumbnail decoding | A bare `Spinner`, a third of the cell | `src/view/grid_view/mod.rs:665` |
| Several photographs decoding at once | **Nothing.** No count, no bar, no "37 outstanding" | `grep -rn "Spinner" src/` returns exactly three sites; none is folder-wide |
| A folder being crawled | **Nothing, and the window stops repainting**: the crawl is synchronous on the UI thread | `src/app/mod.rs:678-681`, `src/crawler.rs:121` |

Empty states are the one onboarding surface with evidence behind it. NN/g's
three guidelines for empty states in complex applications are: communicate
system status, help users discover unused features, and *"provide direct
pathways for getting started with key tasks"*
(https://www.nngroup.com/articles/empty-state-interface-design/). Neither of
the two labels currently on screen does any of those three.

What each state should be:

- **No photographs.** Name the folder that is open. Nothing names it in this
  state: the window title is the fixed string "Avis Image Viewer"
  (`src/main.rs:52`), and the only place a folder is ever named on screen is
  the metadata panel's `Directory:` row (`src/config/defaults.rs:127`), which
  needs a photograph to have one. Then offer *Open Folder…*, the recent
  folders the session file already holds — `Session.positions` is a deque of
  (folder, photograph) pairs, most recent first, capped at 64
  (`src/session.rs:64`, `:30`, `:139-140`) — and one line saying `Press ? for
  the keys`. This is the only screen a first-run user is guaranteed to see,
  and it is currently three words.
- **Filtered to nothing.** Say what is hiding them and offer to stop: "Nothing
  matches. 2,030 photographs are hidden by the rules in the filter bar
  (`F3`)." with a *Show everything* button that sets `narrowing.suspended`
  (`src/view/narrow.rs:143`) rather than clearing the rules. That button
  already exists inside the filter bar (`src/ui/filter_bar.rs:282-286`);
  putting a second one where the problem is visible is the point, not a
  duplication. This exact state without the explanation is a recurring
  Lightroom support question — *"if the Filter bar is hidden but the Library
  filters are turned on, then you may not see the photos you expect to"*
  (https://mastering-lightroom.com/lightroom-filter-bar/).
- **The tag panel with nothing open.** Draw the panel with its headings greyed
  and one line: "Nothing is open to tag." A key that does nothing at all is
  indistinguishable from a key that is not bound.
- **The metadata panel with nothing open.** "No photograph open" rather than
  "Loading…", which is a lie that never resolves.
- **Decoding.** See §10.10: the numbers needed for a determinate bar do not
  exist, and the ones that do exist say something else.
- **A folder being crawled.** The crawl has to leave the UI thread before
  anything can be drawn about it, and there is no interim measure. Setting the
  window title first does not work: `ViewportCommand`s are queued and applied
  after `update` returns (and there is no `ViewportCommand::Title` anywhere in
  `src/` today), so a title set immediately before a blocking crawl reaches
  the screen only once the crawl has finished. Either the crawl moves to a
  worker or it is chunked across frames; nothing else shows anything.

### 10.3 The first run

There is none. `grep -rniE "first[_ ]run|welcome|onboard|tour" src/` returns
two hits, both incidental prose in comments (`src/session.rs:16`,
`src/view/stacks.rs:505`); there is no mechanism. Every panel starts closed —
`menu_visible: false` (`src/app/mod.rs:201`), `side_panel_visible: false`
(`:202`), `metrics_visible: false` (`:203`), `tag_panel_visible: false`
(`:212`), `filter_visible: false` (`:230`), `cheat_sheet_visible: false`
(`:235`) — and the filmstrip follows `grid_view.filmstrip_height`, which
defaults to zero (`src/config/defaults.rs:299`, read at `src/app/mod.rs:175`).
On a genuinely first run there is no session, so the working directory is
crawled (`src/crawler.rs:65-79`), and the code itself concedes what that
means: *"the working directory of a viewer started from a desktop icon is
nobody's choice"* (`src/crawler.rs:28-29`). The likeliest first screen in the
product is a grey window, no menu bar, one status bar, and the words "No
images here".

**A tour is not the answer.** The evidence against them is unusually clean.
NN/g's tutorial study found that *"tutorials don't make users faster or more
successful at completing tasks; on the contrary, they make them perceive the
tasks as more difficult"* — a Single Ease Question mean of 4.92 with a
tutorial against 5.49 without, *"statistically significant (p=0.047)"*
(https://www.nngroup.com/articles/mobile-tutorials/). The paradox of the
active user is older and blunter: *"Users never read manuals but start using
the software immediately"*
(https://www.nngroup.com/articles/paradox-of-the-active-user/), and Krystal
Higgins's reading of it is that this *"is the result of fundamental human
behavior, and is not a design problem to be solved"*
(https://www.kryshiggins.com/active-user-paradox/). Nothing in the research
turned up a first-run tour in professional desktop creative software; the
nearest desktop analogue, Blender's splash screen, is a file chooser plus one
preference, not a tour
(https://docs.blender.org/manual/en/latest/advanced/app_templates.html). The
tour goes to §13.

So the first run should be four things, none of them modal:

1. **The menu bar visible.** `menu_visible` starts `true` when there is no
   session file — which needs a check on `Session::path()` existing, because
   `Session::load` hands back a default for a missing file and for an
   unreadable one alike (`src/session.rs:75-93`) — and thereafter remembers
   what the user left it at. That makes it session state rather than a
   setting, which is the distinction §4 draws; the viewer already writes its
   own session file on the way out (`src/app/mod.rs:866-872`), so this is a
   field on `Session`, not eframe persistence, which is not compiled in
   (`Cargo.toml:13`). `F1` is currently the only door to the menu
   (`src/config/defaults.rs:68-70`) and is itself undiscoverable; a viewer
   whose first frame shows no menu at all has no visible path to anything.
   Mozilla's menu telemetry is the counter-argument to deleting menus for
   tidiness: menu clicks systematically understate use, because items like New
   Tab are *"predominately driven by keyboard shortcuts"*
   (https://blog.mozilla.org/metrics/2010/03/15/menu-item-usage-study-part-i/)
   — the menu is where people *learn* the key.
2. **A real empty state**, as in §10.2, because it is what a desktop-launched
   first run actually lands on.
3. **One line in the status bar**, on the first session only: `Press ? for the
   keys · F1 for the menu`, dismissed by pressing either. NN/g's rule for
   instructional overlays is that they work when they *"focus on a single
   interaction"* and fail when they bombard
   (https://www.nngroup.com/articles/mobile-instructional-overlay/). One line,
   once.
4. **A notice saying the configuration file was created, and where.** The
   defaults are written on a missing file with every failure logged and
   nothing said (`src/config/load.rs:61-92`). A user who is told, once, that
   `config.json` now exists at `Config::path()` is a user who can find the
   forty-seven settings that live only there (§3).

### 10.4 A Help menu

There is no Help menu, no About, and no version string: `grep -rn
"CARGO_PKG_VERSION" src/` returns nothing, and `Config.version`
(`src/config/mod.rs:25`) is a file-format number that is never shown. The menu
bar has three menus, eleven items and six `MenuAction` variants (§1;
`src/app/panels.rs:12-23`, `:26-79`). `F1` is the help key by convention on
Windows; here it toggles the menu bar, and the comment at
`src/app/input.rs:110-112` records that `F1` was considered for the cheat
sheet and rejected *because* it was already the menu.

A Help menu is cheap and it is the surface that makes everything else in this
chapter reachable by mouse. It is the menu bar's fourth menu, and it should
contain, in this order:

| Entry | Does | Why |
|---|---|---|
| **Keys…  `?`** | Opens the cheat sheet — the entry and the surface are the same thing, named once here and once in §10.5 | Gives `?` a visible home. `?` is hardcoded outside the registry (`src/app/input.rs:113-115`), so it appears in neither the keyboard editor (`src/ui/keys.rs:61`) nor the cheat sheet it opens |
| **Keyboard…** | A deep link to the **Keys and mouse** page of the settings window (§3), the same page `Settings ▸ Keyboard…` opens | One page reachable by several routes is the intent, not a duplication. After the settings window exists neither entry opens a window of its own |
| **What the marks mean** | A small window: the four stack glyphs, the badge vocabulary, the overlay colours, the pane border | The legend for a visual language that has none anywhere (`src/view/grid_view/cell.rs`, `src/view/stacks.rs:362-368`, `src/decoder/overlays.rs:57-61`, `src/view/image_view/layout.rs:127`) |
| **Template placeholders…** | The `PLACEHOLDERS` grid, as a window rather than a hover | The same table is already drawn twice, both times inside one mode: on hover (`src/view/organize/rename.rs:141-155`) and in the *Insert…* menu (`:44-57`). Three configuration fields use the same grammar and reach it from nowhere |
| separator | | |
| **Open the configuration file** | Reveals `Config::path()` in the file manager | `src/config/load.rs:14-17`. It has three callers — `Config::save` itself (`:34`), `src/annotations/catalog.rs:190` resolving a relative catalog path, and a test at `src/logging.rs:191` — and none of them puts the path on screen. There is no opener crate in `Cargo.toml`, so this is a platform `Command::new` — the same machinery user actions already use (`src/actions/user_action.rs:93`) |
| **Open the log file** | Reveals `logging::path()` | `src/logging.rs:37-40`. `logging::path()` reaches the user in exactly one place, `src/main.rs:34`, where it writes the path into the log, which only helps somebody who has already found it; its only other production caller is `open_log` (`src/logging.rs:86`), which uses it to open the file and says nothing |
| **Open the manual** | The README, on the web or beside the binary | The cheat sheet's own doc comment concedes the gap: *"The README has them all, and the README is not on screen while somebody is culling"* (`src/ui/cheat_sheet.rs:4-5`) |
| **About** | Version from `CARGO_PKG_VERSION`, the wgpu adapter, whether LibRaw is present, the config and log paths, and a copy button | Everything a bug report needs. The adapter has to be read at construction, where `cc.wgpu_render_state` is available (`src/app/mod.rs:157-160`), and kept as a string. LibRaw availability comes from `decoder::raw::version()` and currently reaches only the log (`src/main.rs:38-41`), so a user whose raws all open as previews has no way to learn why |

The About window is not decoration. Three of the most confusing behaviours in
the product — raw files opening as small previews, a keyword list that
silently does not load, decodes that silently fail — are all diagnosable from
one window that names the build and the two file paths.

The settings window itself is reached from `Settings ▸ All settings…` and
`Ctrl+,` (§3); Help does not duplicate that route.

### 10.5 The cheat sheet

The cheat sheet is the best thing in this chapter and it is nearly invisible.
It is generated from the registry (`src/ui/cheat_sheet.rs:48`), so it shows
the keys actually bound rather than the ones the documentation remembers, and
it is narrowed by mode (`:28-36`). That combination is ahead of the field:
Krita's equivalent is Settings → Configure Krita → Keyboard shortcuts with a
print button that yields roughly 67 pages, and the community's actual answer
is hand-curated filtered lists
(https://krita-artists.org/t/keyboard-shortcut-cheat-sheet/8311); Photo
Mechanic's offline artefact is a 13 MB PDF of 100+ shortcuts
(https://docs.camerabits.com/support/solutions/articles/48000317837-keyboard-shortcuts-windows).
Completeness is the enemy and filtering is the feature; this one already
filters.

What is wrong with it is entirely reachability and density.

| Change | Why | Where |
|---|---|---|
| Put it in the Help menu as `Help ▸ Keys…  ?`, with the key named beside it | The only route is `?`, which nothing on screen mentions | `src/app/input.rs:113-115`; the menu is `src/app/panels.rs:67-77` |
| Show the hardcoded keys in the sheet, and in the keyboard editor as read-only rows | `?`, `F10`, and the image and grid views' fixed arrays are outside `bindings::all()`, so a user browsing every key the viewer listens for will not find any of them. They want registry entries with a read-only field variant — `Field` currently offers only `Fixed`, `Rating` and `Label`, all writable (`src/config/bindings.rs:20-24`) — rather than being absent. What else is wrong with the editor is §9's | `src/app/input.rs:102-104`, `:113-115`; `src/view/image_view/input.rs:108-118`; `src/view/grid_view/mod.rs:560-565`, `:581-584` |
| Say on the arrow rows that `Shift` extends the selection | It is the one documented contact-sheet gesture with no row shape at all, because the modifier is read beside the binding rather than consumed with it (`src/view/grid_view/mod.rs:557`, `:591-600`; the reason is at `:553-556`). A row of its own would collide with the plain arrow row that claims the press; a second line on the arrow rows collides with nothing | `README.md:842`; §10.1 |
| Show each binding's `description` | The sentence already exists on every row and only the keyboard editor reads it. Two columns of key and name is a reminder; three columns is an explanation | sheet at `src/ui/cheat_sheet.rs:85-91`, prose at `src/config/bindings.rs:33-34` |
| Add a search box **and change the close rule with it** | The image view's sheet is General (20) plus Image view (22) plus Ratings and tags (15) — 57 rows in one window, which is past scannable; the gallery's is 20 plus 12 plus 15, or 47. But a search box cannot coexist with the current dismissal: **any** key press closes the sheet (`:110-119`), so the first character typed would dismiss it. The rule has to become Escape, a click outside, or any key while the box has no focus | `src/ui/cheat_sheet.rs:70`, `:106-119` |
| Do not reorder it | A flat, spatially stable surface is what makes experts fast. Scarr et al. found CommandMaps *"significantly faster than both menus and the Ribbon"* for experienced users and no different for novices (http://hci.cs.umanitoba.ca/Publications/details/improving-command-selection-with-commandmaps); the stability reading is the research notes' inference from that, not the paper's own causal claim | — |
| Show it for the folder jobs too | Rename, TimeShift and Group get "General" only (`:34`), which is correct about the *keys* and leaves those three modes with no in-app explanation of anything but hover text | `src/ui/cheat_sheet.rs:28-36` |
| Never push it | It is opened by one key and closed by any (`:106-119`), which is right. It must never appear unbidden | `src/ui/cheat_sheet.rs:106-119` |

One further idea worth taking from the literature rather than from other photo
software: ExposeHK's finding is that showing hotkeys *overlaid on the controls
themselves* — while a modifier is held — makes the visible path rehearse the
keyboard path, and three studies reported increased hotkey use
(https://hal.science/hal-01894253, search-summary only; the PDF at
https://www.csse.canterbury.ac.nz/andrew.cockburn/papers/ehk.pdf would not
render). In this program that means painting each visible control's bound key
beside it while a modifier is held. Two caveats before anyone costs it. It
cannot hang off `?`, which on most layouts is itself `Shift` plus a key and so
is not a modifier to hold; it wants a held `Alt`, or a toggle. And it is not
free: `keys::describe` (`src/ui/keys.rs:221-230`) renders a shortcut as text
and the registry maps every command to a field, but nothing today connects a
*drawn control* to its binding, and every call site would have to say which
command it is. It belongs after the Help menu and the tooltips, not before
them.

### 10.6 A command palette

A palette is the second-best version of two things this program needs anyway,
and it should not be built before them.

The case for one is real. RStudio's palette is *"instant, searchable access to
all of a program's commands"*, it is opened from `Ctrl+Shift+P` **or the Tools
menu**, each row *"displayed with their bound keyboard shortcuts, if any, so
that you know how to invoke the command directly with the keyboard next
time"*, and — the part that matters most here — it searches *settings* as well
as commands and lets you change them inline
(https://docs.posit.co/ide/user/ide/guide/ui/command-palette.html). With 110
settings, sixty of them keys and forty-seven reachable nowhere (§3), a
searchable settings surface is not a luxury.

The case against building it first is also real, and specific:

- **No photo or culling application appears to ship one.** The survey behind
  this plan covered Lightroom, Capture One, Photo Mechanic, FastRawViewer,
  darktable and digiKam and found nothing of the kind; that is a negative
  finding from one survey rather than a proof, and the researcher's own
  reading was that it is "either an opportunity or a signal that culling is
  too keyboard-dominated to need one". The closest thing in the field is
  Photoshop's Discover panel, which is search plus tutorials plus stock, and
  which produced a support thread of the form *"The new Discover panel is a
  total nuisance. It pops up in front of my work all the time"* and *"If I
  close it, it just jumps back up"*
  (https://community.adobe.com/t5/photoshop-ecosystem-discussions/how-do-i-disable-the-discover-panel/td-p/13616538).
  A palette must be strictly pull, never push, and its key must be rebindable.
- **There is no name people will search for.** Inkscape's own naming survey
  found "Command Palette", "Search Actions", "Operator Search" and "Command
  Menu" all in current use across Sublime, VS Code, GIMP, Scribus, Blender and
  Chrome DevTools (https://gitlab.com/inkscape/inbox/-/issues/3217). Whatever
  it is called, it needs a visible entry point — which is the Help menu again.
- **Hidden expert layers before the plain path works are the documented
  mistake:** *"Do not introduce hidden power-user behavior before the plain
  path is already strong"*
  (https://uxpatterns.dev/patterns/advanced/command-palette).
- **A search over commands has its own traps.** darktable's shortcut search
  drew a bug report titled *"NG Input: shortcut search is unintuitive and no
  clues are given"*, in which up/down cycling results was *"far from
  obvious"*, Enter set a binding instead of dismissing, and Escape offered to
  delete shortcuts (https://github.com/darktable-org/darktable/issues/9378).

The two things it is the second-best version of are the settings window's own
search (§3) and the cheat sheet's search box (§10.5), which is a palette over
the keys for a tenth of the work. Both come first; whether the palette itself
is ever built is §13's to record. If it is, it should be one query box over
three indexes (commands, settings, folders), opened from the Help menu as well
as a rebindable key, showing each row's bound key, never reordering by
recency, with a footer reading `↑↓ move · ⏎ run · esc close`, and no keystroke
inside it may ever mutate a binding.

### 10.7 The tooltip policy

**What deserves one.** A tooltip is supplemental text, and Microsoft's rule is
the one to adopt wholesale: *"the text must be supplemental — that is, not
essential to the primary tasks. If it is essential, put it directly in the UI
so that users don't have to discover or hunt for it"*, and *"Never use tips as
a substitute for good design"*
(https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tooltips-and-infotips).
So:

1. Anything that will **write to disk** gets one, and it says so plainly.
2. Anything whose label is a **term of art** — Blown, Crushed, Alike, Filling,
   Advancing, RAW+JPEG, Metadata read ahead — gets one that spends the term.
   Whether any of those words is the wrong word is §11's; the tooltip explains
   whatever word survives.
3. Anything **truncated or abbreviated** gets the whole of it (this is the one
   pattern the program already does well: `src/app/panels.rs:158-162`).
4. Anything whose **units, range or "what does zero mean"** are not obvious
   gets one, quoting the range the control is specified with (§5).
5. Anything with a **bound key** gets that key in parentheses. This is the
   single best-corroborated recommendation in the research: asked for in the
   XnView newsgroup in 2007 and again in 2011, wanting `Fullscreen (F11)`
   (https://newsgroup.xnview.com/viewtopic.php?t=13913); asked for in
   DEVONthink, wanting `See Also & Classify (⌃S)` and *"to all tooltips with a
   keyboard shortcut"*
   (https://discourse.devontechnologies.com/t/show-keyboard-shortcuts-in-tooltips/73248);
   mandated by Microsoft — *"Whenever appropriate, make tooltips more helpful
   by providing keyboard shortcuts and default values… Doing so makes tooltips
   helpful for labeled controls even when they otherwise just repeat the
   label"*; and shipped by Capture One as "Enhanced tooltips", which show a
   description, the default shortcut key and a link to the help article,
   toggled at Preferences → General
   (https://support.captureone.com/hc/en-us/articles/360015552357-Enhanced-tooltips,
   403 on fetch, search summary only). `keys::describe`
   (`src/ui/keys.rs:221-230`) already renders a shortcut as text, so the
   rendering costs nothing.
6. Anything **backed by a configuration field with no GUI** names the field,
   so the tooltip is also the index into the JSON file. Once a settings window
   exists, the same clause names the page instead (§3).
7. Anything that **carries a right-click menu** ends with the four-word clause
   `Right-click for more.` — nothing about the menu's contents, which are
   §7's.

**What does not.** A tooltip that repeats its label is worse than none:
*"Tooltips with obvious or redundant text are not beneficial to users"*
(https://www.nngroup.com/articles/tooltip-guidelines/). The two existing
hovers that break this are `src/ui/filter_bar.rs:266` — "Descending" on a `▼`
button whose neighbour already says the sort key — and
`src/ui/tag_panel/mod.rs:112`, `"{n} star(s)"` on the nth star. Neither adds
anything. Neither should be deleted either, because half-tooltipped is the
worst state; they should be made to earn their keep ("Newest first" / "Set the
rating to {n}. Pressing it again clears it.").

And a tooltip is never the place to say a value needs a restart. Of the 110
settings, 26 do not take effect until the next launch (§3); the answer to that
is to make the field live or rebuildable, not to warn about it on hover. A
warning that a control does not work is a control that does not work.

**What a good one says.** One or two sentences, in the same register as the
registry's descriptions and the doc comments — which is to say plain, about
photographs rather than about software, and specific about consequence. The
best hover already in the program is the model:

> "Forward when the photographs were taken later than the camera thought — a
> camera left on winter time in summer needs an hour forward."
> — `src/view/organize/timeshift.rs:46-49`

**Two literal bugs to fix while touching these files.** Two hover strings
carry runs of stray whitespace from broken string continuations and render
with a gap in the middle: `src/view/image_view/bottom_bar.rs:140` ("folded
up;⟨spaces⟩the key") and `src/view/organize/group/mod.rs:253` ("frames of
the⟨spaces⟩same scene"). And `src/view/image_view/bottom_bar.rs:126-129`
passes an **empty string** to `on_hover_text` when nothing is hidden, so egui
still lays out and paints an empty tooltip frame.

#### 10.7.1 The worked list

Seventy places across sixteen files, with the text proposed. Where a key is
shown as `{key}` it is rendered from the configuration through
`keys::describe` (`src/ui/keys.rs:221-230`), so a rebound key stays correct.
Several rows stand for a loop rather than one widget — the two menu file
entries, the six mode radios, the two cache rows, the four memory rows, the
six status-bar flags, the three filter text boxes — and are one edit each,
with a match on the label inside the loop.

**The menu bar and the side panel** — `src/app/panels.rs`, currently two
hovers in the file.

| Where | Widget | Proposed text |
|---|---|---|
| `src/app/panels.rs:33-40` | Open Folder | "Open every photograph in a folder — and its sub-folders, while flattening is on." |
| `src/app/panels.rs:33-40` | Open Files | "Open a photograph in the folder it lives in." *(Conditional: today this takes only the first of the files picked and opens its parent folder — `src/app/settings.rs:27-38`. The tooltip is honest only once that is fixed; until then it should say what the command does.)* |
| `src/app/panels.rs:60` | each Mode radio | Per mode, from a new `Mode::description()` beside `Mode::label`. The doc comments at `src/app/mode.rs:10-22` are the starting point, but four of the six are one short phrase and want a second clause about consequence. E.g. Bulk rename, from "Renaming the whole folder at once": "Rename every file in this folder from a template. Nothing is written until the button is pressed." |
| `src/app/panels.rs:68` | Keyboard… | "Change any of the sixty-nine bound keys. `?` shows the same list without changing it." |
| `src/app/panels.rs:73` | Slideshow… | "How long each photograph is held, and how it moves while it is up." |
| `src/app/panels.rs:143` | "Image Metadata" heading | "What the file says about itself. Which tags appear, and in what order, is `general.metadata_tags`." |
| `src/app/panels.rs:170` | "Cache" heading | "How much of this folder is decoded and ready to draw, against the budgets in the `cache` section of the configuration." |
| `src/app/panels.rs:173-181` | "Images:" row | "How many photographs of this folder are decoded in memory, and how many of those are uploaded as textures." |
| `src/app/panels.rs:173-181` | "Thumbnails:" row | "The same, for the contact sheet's thumbnails." |
| `src/app/panels.rs:183-187` | "…ready to zoom into" | "Photographs held at their own pixels as well as at screen size, so zooming into them shows detail rather than a magnified copy. `cache.full_resolution_neighbours`." |
| `src/app/panels.rs:194` | "N could not be opened" | "Files the decoder refused. Their names and the reason are in the log." |
| `src/app/panels.rs:230-238` | "Decoded:" | "Screen-sized copies of the photographs, in ordinary memory. `cache.ram_budget_mb`." |
| `src/app/panels.rs:230-238` | "On the GPU:" | "The same pixels again as textures, with their mip chains. `cache.gpu_budget_mb`." |
| `src/app/panels.rs:230-238` | "Thumbnails standing in:" | "The camera's own thumbnails, read from the front of each file, holding the place until the real decode lands." |
| `src/app/panels.rs:230-238` | "Metadata read ahead:" | "EXIF read ahead of where you are, so the panel is already filled when you arrive." |
| `src/app/panels.rs:241-247` | "N MiB held in all" | "What the four tiers above hold together. The process will be larger than this: staging buffers and allocator slack are not counted." |

**The histogram** — `src/ui/histogram.rs`, one hover site.

| Where | Widget | Proposed text |
|---|---|---|
| `src/ui/histogram.rs:39` | "Tones" heading | "How this photograph's brightnesses are spread. The grey fill is overall brightness; the three colours are the red, green and blue channels drawn over it." *(`LUMA` is grey at `:21`, drawn first at `:53`.)* |
| `src/ui/histogram.rs:43` | the plot (the `Response` is discarded into `_`, though the rect is already allocated with `Sense::hover()`) | "Black at the left, white at the right. Heights are square-rooted, so a large flat sky does not flatten everything else." *(`:93`)* |
| `src/ui/histogram.rs:112-131` | Blown / Crushed | Keep the two existing sentences and add: "Called out in amber above a tenth of a per cent, which is more than the specular highlights every photograph has." *(`WORTH_SAYING = 0.1` at `:30`, `WARNING` at `:25`.)* |

**The status bar** — `src/view/image_view/bottom_bar.rs`, three hover sites.

| Where | Widget | Proposed text |
|---|---|---|
| `src/view/image_view/bottom_bar.rs:126` | the position count | "Where you are in the folder, in the order the filter bar sets." — and keep "{n} more are hidden by the filter" when there are any, instead of today's empty string at `:127`. |
| `src/view/image_view/bottom_bar.rs:139` | the stack place | "One run of frames, shown as a single photograph. Amber means the run is folded up; `{sc_toggle_stack}` opens it." *(and fix the stray whitespace at `:140`)* |
| `src/view/image_view/bottom_bar.rs:144-156` | "Flattened" | "The sub-folders of this folder are in the collection too. `{sc_flatten_dir}`." |
| `src/view/image_view/bottom_bar.rs:144-156` | "Watching" | "Files appearing, changing or going in this folder are being followed. `{sc_watch_directory}`." |
| `src/view/image_view/bottom_bar.rs:144-156` | "Filling" | "Every photograph fills the window as you move through the folder, rather than fitting inside it. `{sc_latch_fit_maximize}`." |
| `src/view/image_view/bottom_bar.rs:144-156` | "Advancing" | "A rating, flag or label moves to the next photograph by itself. `{sc_toggle_advance}`." |
| `src/view/image_view/bottom_bar.rs:144-156` | "Comparing" | "Several photographs are pinned side by side and share one zoom. Marks act on the framed pane only; Tab moves the frame, / drops it, Escape leaves." *(all three hardcoded at `src/view/image_view/input.rs:115-117`)* |
| `src/view/image_view/bottom_bar.rs:144-156` | "RAW+JPEG" | The sense of the field's own doc comment (`:65-70`), tightened for a tooltip: "This frame is a raw and a JPEG shot together. A rating, a move or a deletion is about to happen to both files." |
| `src/view/image_view/bottom_bar.rs:160-164` | the name | The plain file name and its folder — the line on screen is rendered through `image_view.name_format` (`src/view/image_view/mod.rs:661-673`) and need not contain either. |
| `src/view/image_view/bottom_bar.rs:212-216` | the "go to" field | "Type a position and press Enter." |
| `src/view/image_view/bottom_bar.rs:262` | the zoom slider | "Magnification, as a percentage of the photograph's own pixels — 1 % to 1,600 %, logarithmically." *(`:249-250`, `:263`)* |
| `src/view/image_view/bottom_bar.rs:278-281` | the percentage | "Magnification. Right-click for more." *(the fit commands and set percentages behind it are §7's; the label is 45 points wide — `:278-281`)* |

**The filter bar** — `src/ui/filter_bar.rs`, already the best-covered file;
ten widgets still have none.

| Where | Widget | Proposed text |
|---|---|---|
| `src/ui/filter_bar.rs:166` | fewest stars | Keep "Fewest stars to show", and add: "Both at nought shows only the unrated." *(`src/view/narrow.rs:231-235`)* |
| `src/ui/filter_bar.rs:181` | the flag combo | "Which of keep, reject or neither a photograph has to carry." |
| `src/ui/filter_bar.rs:206` | the label combo | "Which colour label a photograph has to carry." |
| `src/ui/filter_bar.rs:239-240` | "Name contains" | "Part of the file name, ignoring case." |
| `src/ui/filter_bar.rs:239-240` | "Keyword" | "A keyword, or one of its levels: `Slovakia` finds everything filed under it." *(`src/view/narrow.rs:265-274`)* |
| `src/ui/filter_bar.rs:239-240` | "Types" | "Extensions to show, comma separated: `jpg, cr3`." |
| `src/ui/filter_bar.rs:252-253` | "Order by" | "The order the arrow keys, the counter and the contact sheet all follow." |
| `src/ui/filter_bar.rs:117` | "N runs · M frames" | "How many runs the detector found in this folder, and how many frames are inside them." |
| `src/ui/filter_bar.rs:120-125` | Fold all / Open all | "Close every run down to one cell, or open all of them." |
| `src/ui/filter_bar.rs:304` | the count | "How many photographs the rules leave, out of the folder." |

**The contact sheet and the filmstrip** — one hover between them.

| Where | Widget | Proposed text |
|---|---|---|
| `src/view/grid_view/mod.rs:394` | a thumbnail | Extend the file name with the marks in words and the stack: "DSC_0417.NEF — three stars, kept, red · one of 17 in a burst". The data is already to hand: `show_row` is handed `marks: &[Marks]` and `stacks: &Stacks` (`:334-341`). |
| `src/view/grid_view/mod.rs:329` | "N selected · Escape to clear" | "The next mark, move or deletion applies to all of these." **This needs one other change first:** the badge sits in an `Area` built with `.interactable(false)` (`:322`), which takes the layer out of pointer handling, so a tooltip on it never fires. Either the flag comes off, or the badge moves into a panel. |
| `src/view/grid_view/mod.rs:661` | the ✖ | The file name and "This file could not be decoded. The reason is in the log." — today it is one glyph with no tooltip and no name, and `show_placeholder` is not handed the path (`:659`), so it takes one argument more. |
| `src/view/grid_view/filmstrip.rs:101` | a strip cell | The file name, as the sheet has, from `store.path(index)`. The strip has none. |

**The overlays and the directory surfaces.**

| Where | Widget | Proposed text |
|---|---|---|
| `src/ui/tree.rs:249` | a folder row | "Click to open or close it. Right-click, or Enter, to open the folder." *(This is the reverse of every file browser — `:262`, `:266` — so the tooltip is a plaster over a behaviour that §9 argues should change.)* |
| `src/ui/navigator.rs:43` | the path box | "Type a folder. Tab completes the highlighted suggestion; Enter opens it." *(`:79`, `:83`)* |
| `src/ui/destinations.rs:97` | "Choose a folder…" | "Pick a folder this once. It is not added to the numbered slots — those are `cull.destinations` — but it does become what `Enter` repeats." *(`src/app/cull.rs:293-303`)* |
| `src/ui/perf_metrics.rs:75` | the timings line | "Frame timings. F10 hides them again." |
| `src/ui/keys.rs:111` | "Put everything back to the defaults" | "Puts every key back. Nothing else in the configuration is touched." *(the button resets bindings only — `:111-121`)* |

**The tag panel** — five hovers, three gaps.

| Where | Widget | Proposed text |
|---|---|---|
| `src/ui/tag_panel/mod.rs:73` | "Applies to N photographs" | "Everything on this panel will be applied to all of them." |
| `src/ui/tag_panel/mod.rs:210-211` | the search box | "Type to search, or type a new keyword. Bars make levels: `Places\|Slovakia\|Tatras`." |
| `src/ui/tag_panel/mod.rs:223` | "+ Add …" | "Puts it on this photograph and keeps it in Recent for the rest of the session." |

**The folder jobs** — `src/view/organize/`, where the destructive buttons are.

| Where | Widget | Proposed text |
|---|---|---|
| `src/view/organize/controls.rs:43-49` | "Sort by" | "The order of the table below, and what the rename counter follows." |
| `src/view/organize/controls.rs:63` | the sort tag box | "Any tag the metadata reader knows, spelled as exiftool spells it." |
| `src/view/organize/controls.rs:79-87` | the Filter toggle | "Every rule below has to be true at once." |
| `src/view/organize/controls.rs:111-115` | the size boxes | "Leave either empty for no bound." *(`:163`)* |
| `src/view/organize/controls.rs:119-123` | the metadata tag box | "Any tag the metadata reader knows, spelled as exiftool spells it." |
| `src/view/organize/controls.rs:139-143` | "Tagged:" | "Keywords, comma separated. A photograph passes if it carries any of them." |
| `src/view/organize/controls.rs:146-150` | "but not:" | "…and none of these." |
| `src/view/organize/rename.rs:61-70` | counter start / step / digits | "What the `{counter}` placeholder puts in the name: where it starts, how far it steps, and how many digits it is padded to." |
| `src/view/organize/rename.rs:73-77` | the extension combo | "Whether the extension is kept as it is, lower-cased or upper-cased." |
| `src/view/organize/rename.rs:91` | "Rename N file(s)" | "Renames the files on disk at once. Sidecars follow their photographs. **This cannot be undone** — the journal does not cover folder jobs." *(`Step` has no rename variant — `src/organize/journal.rs:28` — and nothing in `organize/` calls `journal.record`)* |
| `src/view/organize/timeshift.rs:69-72` | the field checkboxes | "With none ticked, every timestamp the file carries is moved." |
| `src/view/organize/timeshift.rs:107` | "Change N file(s)" | "Rewrites the capture times inside the files. Maker notes and pixels are not touched. **This cannot be undone.**" |
| `src/view/organize/group/mod.rs:57-62` | the gap | "The clock, not the picture: frames closer together than this are one run whatever they show." |
| `src/view/organize/group/mod.rs:71-73` | "At least: N frames" | "Runs shorter than this are left as loose frames." |
| `src/view/organize/group/mod.rs:98` | "Tidy N group(s) into folders" | "Moves each group's frames into a folder of its own, named for what it is. **This cannot be undone.**" |
| `src/view/organize/group/mod.rs:139-143` | the kind combo | Per kind, from the `Kind` doc comments verbatim (`src/organize/group/classify.rs:19-26`): "The same view at different exposures, for merging." / "The same view at different focus distances, for merging." / "A camera on a timer." / "Frames of the same thing, to choose between." |
| `src/view/organize/group/mod.rs:194`, `:252` | the sharpness score | Replace both with the caveat that matters: "How sharp this frame looked, for ranking it against the others of the same scene. It says nothing useful across different scenes — a wall outscores a portrait at f/1.4." *(and fix the stray whitespace at `:253`)* |

That is seventy rows across sixteen files. Five of them augment a hover that
already exists (`src/ui/filter_bar.rs:166`, `src/ui/histogram.rs:112-131`,
`src/view/image_view/bottom_bar.rs:126`, `src/view/organize/group/mod.rs:194`
and `:252`), so the count goes from 33 to **98**, and the file count from 11
of 139 to 16 — five files that currently have none
(`src/view/grid_view/filmstrip.rs`, `src/ui/tree.rs`, `src/ui/navigator.rs`,
`src/ui/perf_metrics.rs`, `src/view/organize/controls.rs`) gain their first.
More to the point, it takes every surface that is currently silent — the cache
readout, the status bar's flags row, the folder-job controls, the destructive
buttons — above zero, which is what the consistency rule is actually about.

### 10.8 Eleven things that need a sentence, not a label

These are the features a photographer has to be *told* about once. Each gets
one sentence, and each sentence needs a home that is not a hover — a legend, a
panel heading, or a line under the control.

| Feature | The sentence | Where it should appear |
|---|---|---|
| **The histogram** | "How this photograph's brightnesses are spread, from black on the left to white on the right; the coloured curves are the red, green and blue channels." | Under the "Tones" heading, `src/ui/histogram.rs:39`. It is drawn only inside the side panel (`src/app/chrome.rs:120-121`), which is behind `I` and in no menu, so a user who does not press `I` concludes there is no histogram. |
| **Clipping overlay** | "Paints the highlights that have gone pure white in red, and the shadows that have gone pure black in blue — the two things a screen cannot show you, because it draws 250 and 255 the same." | Nowhere at all today. `Overlay::label()` exists (`src/decoder/overlays.rs:35-41`) and is used in **one test**: pressing the key sets `self.marking = self.marking.next()` (`src/view/image_view/mod.rs:281`) and nothing else happens. The state must reach the status-bar flags row (`src/view/image_view/bottom_bar.rs:144-156`, whose `Flags` struct at `:57-79` has no field for it) and the colours must be in the "What the marks mean" legend (`src/decoder/overlays.rs:57-61`). |
| **Focus peaking** | "Marks the sharpest twentieth of this photograph's edges in green — relative to this frame, so it always marks something, and comparing two frames means comparing how much is marked." | Same place. The 5 % rule (`MARKED_SHARE`, `src/decoder/overlays.rs:78`) is the fact that makes peaking legible and it is written only in a doc comment; whether it should become a settable field is §6's, but it has to be *said* either way. The silent failure matters too: `peaking()` returns `None` for a frame under three pixels on a side and for one whose gradients yield no threshold (`:124-146`), so on a soft photograph the key draws nothing and says nothing. |
| **Sharpness score** | "How sharp a frame looked, for ranking it against the other frames of the same scene — never across different scenes, where a wall outscores a portrait at f/1.4." | Beside the score in the group panel (`src/view/organize/group/mod.rs:194`, `:252`) and beside the "Sharpness" entry in the folder-job sort dropdown (`src/organize/sort.rs:30`, offered at `src/view/organize/controls.rs:45`). The caveat is written twice in the repository — `src/organize/sort.rs:26-29` and `docs/changelog.md:95-96` — and in neither place a user can read it, and a user sorting by it will draw exactly the wrong conclusion. |
| **Virtual stacks** | "Shows every burst, bracket and timelapse as one cell standing for the whole run — worked out from the clock and from what the frames look like, every time. Nothing is written to disk." | The existing hover (`src/ui/filter_bar.rs:98`) is right and is reachable only with the filter bar open. The four glyphs — `◐` HDR, `◎` focus stack, `⏱` timelapse, `❏` series (`src/view/stacks.rs:362-368`) — have **no legend anywhere**, and the rule that the sharpest frame stands for a folded run (`README.md:491`) is in no part of the program. |
| **RAW+JPEG pairing** | "This frame is a raw and a JPEG shot together: one is browsed and the other follows it, so a rating, a move or a deletion happens to both files." | The status bar prints "RAW+JPEG" with no hover (`src/view/image_view/bottom_bar.rs:150`) while the reason sits five lines above it as a doc comment (`:65-70`). |
| **The template grammar** | "Anything in braces is replaced from the photograph; anything inside `$( … )` disappears when what is in it is missing." | Today it is one `on_hover_ui` and one *Insert…* menu on one text box in one mode (`src/view/organize/rename.rs:42`, `:44-57`, grid at `:141-155`). The same grammar drives three configuration fields with no GUI at all (`src/config/mod.rs:348`, `:383`, `:482`). It needs a Help-menu window and a link from every field that uses it. |
| **Hierarchical keywords** | "A keyword written `Places\|Slovakia\|Tatras` is filed under each of its levels, and narrowing by `Slovakia` then finds everything below it." | The tag panel's search box says only "Search tags or categories" (`src/ui/tag_panel/mod.rs:210-211`) and never says a path may be typed; the filter bar's keyword box (`src/ui/filter_bar.rs:239-240`) never says it matches parents, though it does (`src/view/narrow.rs:265-274`). |
| **Sidecars** | "Ratings, flags, labels and keywords are written into an XMP file beside the photograph, which is what Lightroom, Bridge, darktable and digiKam read." | Nothing in the GUI says where a rating goes. This is the feature the audience cares most about — the one thing forum users consistently praise about darktable and FastRawViewer and distrust about Lightroom (§2) — and the program never mentions it. |
| **The cache tiers** | "The folder is held at three sizes at once: the camera's thumbnail for instant display, a screen-sized copy of everything within reach, and the photograph's own pixels for the few nearest you." | Under the "Cache" heading (`src/app/panels.rs:170`), where the numbers currently stand alone. |
| **Colour management** | "Converted from the profile embedded in the file — or the closest of sRGB, Adobe RGB and Display P3 matched by name — to the output profile in `general.output_icc_profile`." | The only colour tag in the default list is `Color Space` (`src/config/defaults.rs:126`), which is the EXIF hint rather than the embedded profile's name. `Profile Description` — which the reader already produces (`src/metadata/mod.rs:33`, from `src/metadata/icc.rs:31`) — is not in the defaults. Adding it is a one-line change that makes colour management visible. |

Two of these eleven — the clipping overlay and focus peaking — are worth
calling out separately, because they are the only features in the program
whose *entire* user-facing existence is an unexplained change of colour on the
photograph. A photographer who has never seen focus peaking presses one key,
gets a green stipple with no caption, presses it again, and it is gone. The
program has the words "Focus peaking" in its source
(`src/decoder/overlays.rs:39`) and shows them to nobody.

### 10.9 Errors: what a user sees when something goes wrong

| What happens | What the user sees now | Where |
|---|---|---|
| **A file will not decode** | Image view: "Could not open this image". Contact sheet: the glyph `✖`, no name, no reason. The reason never leaves the worker: the error payload is matched into `_` and only the index is kept | `src/view/image_view/layout.rs:132`, `src/view/grid_view/mod.rs:659-661`, `src/cache/store/decode.rs:85-89`; a panicking decoder becomes `DecodeError::Unsupported` at `src/cache/loader.rs:214-224` |
| **A folder is empty** | Three words, centred. No folder name, no *Open Folder…*, no way to tell an empty folder from a filtered-out one except by reading which of two labels appeared | `src/view/image_view/layout.rs:42-44`, `src/view/grid_view/mod.rs:265-267` |
| **A sidecar cannot be written** | **The one path that is properly plumbed.** The writer logs and pushes `"Could not save {name}: {e}"` onto a queue; `report_problems` drains it into the notice band every frame. Repeats are counted rather than stacked | `src/annotations/writer.rs:112-130`, `src/annotations/mod.rs:285-287`, `src/app/mod.rs:759-761`, `src/ui/notice.rs:44-47` |
| **A sidecar cannot be *read*** | **Nothing — not even a log line.** `.ok()` and `find_map` swallow both the I/O failure and the parse failure, and the store seeds the entry from `Xmp::default()` (`src/annotations/mod.rs:296-302`), so the photograph browses as unrated with nothing to say a sidecar exists. The file itself is safe — `xmp::update` refuses a document it cannot rewrite (`src/metadata/xmp/write.rs:33-47`) and the write error is reported — so the first the user hears of it is a failure notice after a keystroke, on a photograph they had no reason to think was wrong | `src/annotations/sidecar.rs:46-51` |
| **The configuration is partly unreadable** | One notice: "Part of the configuration file could not be read; those settings are at their defaults and the file is not being written over". It does not say *which* section, though `src/config/load.rs:179` knows the name, and does not say where the file is | `src/config/load.rs:166-183`, `src/app/mod.rs:249-253` |
| **A user action's program is missing** | **Nothing at all.** The spawn failure is a `tracing::error!` and the code carries the admission: `//Show toast with result?` … `//Provide the error to the user in the future` | `src/actions/user_action.rs:90-99` |
| **A keyword list will not load** | **Nothing.** The tag panel shows the configured categories and no more; the failure is a `tracing::warn!`. `README.md:126` calls this deliberate, which is right about not refusing to start and wrong about saying nothing | `src/annotations/catalog.rs:54-62` |
| **A folder job fails on some files** | An aggregate — "{n} cannot be renamed" — with which file and why in the log only | `src/view/organize/rename.rs:104-109`; same shape at `src/view/organize/timeshift.rs:113-115`, `src/view/organize/group/mod.rs:104-106` |

The scale of the gap: `src/` holds 79 `tracing` statements — 28 `error`, 17
`warn`, 22 `info`, 9 `debug`, 3 `trace` — and exactly **20** `notices.say`
sites (`grep -rn "\.say(" src/` returns 23, three of them `src/ui/notice.rs`'s
own tests). Of the 28 errors, three also reach the screen:
`src/app/cull.rs:238-239`, `src/app/settings.rs:106-108`,
`src/annotations/writer.rs:112-130`. Twenty-five error paths are invisible, in
a program that never names the log file on screen: the only place
`logging::path()` reaches the user is `src/main.rs:34`, where it writes the
path *into the log*, and its other production caller opens the file and says
nothing (`open_log`, `src/logging.rs:86`).

The notice band itself has four faults, all in `src/ui/notice.rs`:

1. **Everything is the same alarm red.** `Color32::from_rgb(72, 32, 32)`
   (`:91`) for "Moved 12 photograph(s) to Selects" (`src/app/cull.rs:474`) and
   for "Access is denied" alike. Success and failure are indistinguishable.
2. **Nothing can be clicked.** `interactable(false)` (`:80`), so a message
   cannot be copied, dismissed, or acted on. A "could not be opened" notice
   that opened the log would be worth more than the whole side panel. This is
   the "and what do I do about it?" rule of §8 applied to the one surface that
   currently answers it with nothing.
3. **There is no history.** Six seconds and 600 ms of fade (`:16`, `:19`), at
   most four lines with older ones dropped (`:22`, `:56-57`), and no way to
   see any of them again. A rating session on a write-protected card is lost
   silently after the band fades.
4. **Startup notices race the folder crawl.** The clash, migration and
   partial-config notices are said inside `App::new` (`src/app/mod.rs:241`,
   `:245`, `:249-253`), before the synchronous crawl at `:260-285`, while the
   hold starts at `Instant::now()` inside `say()` (`src/ui/notice.rs:46`,
   `:53`). On a large folder several of the six seconds are spent before the
   band is drawn for the first time at `src/app/mod.rs:855`.

What to do, in order of value:

- **Give the band severity.** Three fills — a neutral one for information,
  amber for a warning, the existing red for a failure — and no other change.
- **Make it clickable, with a history.** `interactable(true)`, a click that
  opens a small "Recent messages" window holding the last hundred, and that
  window is also where "12 files could not be opened" becomes a list of names.
  Reversibility and recoverability are themselves discoverability features:
  *"the ability to easily get out of trouble encourages exploration, which
  facilitates learning and discovery of features"*
  (https://www.nngroup.com/articles/user-control-and-freedom/, search summary
  only).
- **Say which section** in the partial-config notice, and offer to open the
  file. `src/config/load.rs:179` already has the name and the serde error.
- **Never emit a bare `format!("{e}")`.** Three sites do —
  `src/app/cull.rs:239`, `:442`, `:449` — so the user sees `Access is denied.
  (os error 5)` with no verb and no file name. Each should read "Could not
  move DSC_0417.NEF: …".
- **Report the read failure of a sidecar**
  (`src/annotations/sidecar.rs:46-51`). The file cannot be destroyed by it —
  the writer already refuses — but a photograph that carries a sidecar the
  reader could not understand browses as unrated, and the person culling has
  no way to know that what they are looking at is not what is on disk.
- **Report a user action that will not start**
  (`src/actions/user_action.rs:90-99`). Right-clicking "Open in GIMP" with
  GIMP not installed currently does nothing whatever, which is the shape of
  failure the research keeps finding: the FastRawViewer user whose reject key
  did nothing because a preference had disabled it, and who was told so only
  inside a settings screen (https://www.fastrawviewer.com/node/577).
- **Never put anything essential in the status bar.** Microsoft: *"status bars
  are easy to overlook. So easy, in fact, that many users don't notice status
  bars at all"* and *"Users should never have to know what is in the status
  bar"*
  (https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-status-bars).
  The flags row is the right place for a *summary*; it is not the right place
  for the only mention of a mode.

### 10.10 Progress

There are three progress readouts in the product and none of them covers what
a user waits on most.

| What is measured | Form | Where |
|---|---|---|
| A folder scan, in a folder job | Spinner plus "Reading the folder… {done} of {total}" | `src/view/organize/mod.rs:260-267` |
| Stack detection | The weak label "reading {done}/{total}", **only while the filter bar is open** | `src/ui/filter_bar.rs:113` |
| One image, one thumbnail | A bare spinner, no text | `src/view/image_view/layout.rs:137`, `src/view/grid_view/mod.rs:665` |

Opening a folder of two thousand starts a pool of decode workers
(`src/app/mod.rs:162`) and a preview reader, and the user gets spinners
appearing and vanishing per cell with no answer to "how far through is it" or
"is it stuck". The stack scan is worse: `Ctrl + G` announces itself with one
six-second notice (`src/app/mod.rs:466`) and then the only readout lives in a
bar the user must open with `F3` (`src/config/defaults.rs:422-424`), which
`Ctrl + G` does not open (`src/app/mod.rs:460-470`). NN/g's formulation of the
heuristic is the whole argument: *"Progress indicators inform users of the
current working state of the system and reduce uncertainty"*
(https://www.nngroup.com/articles/usability-heuristics-complex-applications/).

One correction to the obvious fix before it is costed. **`StoreStats` cannot
give a folder-wide percentage.** `in_ram` is the length of the LRU cache
(`src/cache/store/mod.rs:460`) and the preload radius is trimmed to what the
budget can hold (`:474-482`), so the store never intends to hold a large
folder at once: `in_ram < total` is permanently true on any folder bigger than
the window, and a bar driven by it would never go away. The stat that says
"work is outstanding" is `loading`, which is `self.requested.len()` (`:470`).

Three changes:

1. **An indeterminate decode mark**, not a determinate bar: a thin strip or a
   small spinner at the foot of the window while `loading > 0`, gone when it
   is not. It answers "is it stuck", which is the question the spinners fail.
2. **A determinate bar only where a total exists.** Two already do: the folder
   scan (`src/view/organize/mod.rs:260`) and the stack read
   (`src/ui/filter_bar.rs:113`) both compute `done`/`total` and both draw it
   somewhere the user has to have opened first. Move both to the same strip,
   so `Ctrl + G` has a readout without `F3`.
3. **Take the crawl off the UI thread.** `crawler::crawl` has exactly one
   caller, `App::open_directory` (`src/app/mod.rs:678-681`), and that is
   reached from nine places — startup (`src/app/mod.rs:265`), flatten
   (`:755`), a callback (`:782`), the navigator and the tree
   (`src/app/chrome.rs:137`, `:148`), undo (`src/app/cull.rs:521`), both menu
   open commands (`src/app/settings.rs:24`, `:35`) and a folder job finishing
   (`src/app/views.rs:166`). On a deep tree or an SMB share every one of them
   stops the window repainting with nothing on screen to say why, and as
   established in §10.2 there is no interim measure that works without moving
   it.

### 10.11 What finishes this chapter

- Every README section in §10.1's table either has an in-app explanation or an
  explicit decision that it needs none.
- The hover count is 98, spread over sixteen files, and every panel that draws
  widgets is above zero. Two files stay at zero by construction —
  `src/view/grid_view/cell.rs` and `src/view/image_view/canvas.rs` paint and
  allocate nothing that can carry a tooltip.
- The keys hardcoded outside the registry — `?`, `F10`, and the image and grid
  views' fixed arrays — appear in the cheat sheet and in the keyboard editor,
  marked unrebindable, and `?` appears in the Help menu. The contact sheet's
  shift-extension appears with them, on the arrow rows rather than as a row of
  its own.
- The cheat sheet shows each binding's sentence and can be searched, with a
  close rule that survives typing.
- A Help menu exists with the eight entries of §10.4, and About names the
  version, the adapter, whether LibRaw is present, and the two file paths.
- The four empty states each say what is happening and offer the next action;
  the two that concern a folder name it.
- Decoding shows that it is happening, and everything that has a real total
  shows the total in one place.
- Every one of the eight failure paths in §10.9's table reaches the screen,
  the band has three severities and a history, and no message is a bare
  `format!("{e}")`.
- The words "Clipping" and "Focus peaking" appear somewhere a user can read
  them, and the four stack glyphs have a legend.

When each of these is done is §12's.

## 11. Configurable without being complicated

The brief asks for two things that are normally traded against each other:
maximally configurable, and still simple enough to operate and understand. They
are only in tension if the number of settings is what makes a program hard. It
is not. The number a person has to *understand* is the number they must decide
before the program works, and that number is set by four things — the defaults,
what is shown at once, what the fields are called, and whether you have to go
looking. The total is nearly irrelevant to all four.

This chapter is the budget the rest of the plan draws against. §3 designs the
window and owns the field-by-field inventory, §4 decides which page each field
goes on, §5 decides which control it gets, §6 argues for the fields that do not
exist yet. This one says how many there should be, which of the existing ones
should not exist at all, which defaults are wrong, and what all of it is called.

Havoc Pennington's line is quoted everywhere as an argument against options —
"Too many preferences means you can't find any of them"
(<https://ometer.com/free-software-ui.html>) — and the operative half of it is
*find*, not *too many*. NN/g measured the same distinction: interface
customisation reached an average completion rate of 83 % while product
customisation reached 66 %, and poor findability "was responsible for 45% of the
many task failures on these sites"
(<https://www.nngroup.com/articles/customization-of-uis-and-products/>). The
program does not become simple by having fewer settings. It becomes simple by
having defaults nobody has to touch, names people recognise, and one place that
always answers.

---

### 11.1 The hundred and ten, and what a field costs

The file carries 111 keys — 110 settings across eight sections plus `version`,
which is bookkeeping (`src/config/mod.rs:18-49`) — and **forty-seven of the 110
can be reached nowhere in the running program**. §3.1 owns that arithmetic and
sets it out field by field; the total and the forty-seven are all this chapter
draws on.

A hundred and ten is not many, and the shape that makes a larger number workable
is published: VS Code's settings editor is "the user interface that enables you
to review and modify setting values that are stored in a settings.json file",
with search, a modified marker and a per-setting reset
(<https://code.visualstudio.com/docs/configure/settings>). The problem is not the
total. It is that the program has already made a hundred and ten decisions
adjustable, told nobody where, and given forty-seven of them no interface at all.

What a field actually costs is the sentence that explains it, and that is the
count worth watching, because the plan intends to turn doc comments into help
text and search terms (§3.4, §10.7). **Fifty-seven of the 110 carry no doc comment
at their declaration** — 37 shortcuts and 20 others — including
`general.text_scaling` (`src/config/mod.rs:268`), `general.output_icc_profile`
(`:266`), `general.metadata_tags` (`:270`), `grid_view.images_per_row` (`:439`),
`image_view.should_wait` (`:370`), `image_view.frame_size_relative_to_image`
(`:372`) and every field of `SlideshowConfig` (`:526-537`). A few are documented
somewhere else instead — `cache.full_resolution_neighbours` on its default
function (`src/config/defaults.rs:27-33`), `slideshow.motion` on the enum
(`src/config/mod.rs:489-524`) — which is worse than useless once the comment is
the help string, because the help is then present for some fields and absent for
others with no pattern. The darktable community's complaint about `darktablerc`
is exactly this and nothing more: "a text file of over 1000 lines" with no
per-key explanation, answered by pointing at `darktableconfig.xml.in` in the
source (<https://discuss.pixls.us/t/explanation-for-darktablerc-parameters-needed/37056>).
The drift is already visible: two doc comments sit in three lines above
`default_gpu_budget_mb` (`src/config/defaults.rs:35-38`) and the first of them
belongs to `default_upload_budget_ms` (`:41`), which has none — §5.4 uses the
same pair as the argument for putting units on screen rather than in a comment.
Nobody has noticed because nobody reads these. The moment they are rendered
beside a slider, somebody will.

---

### 11.2 What should not be settable at all

Microsoft's warning is that a property window "become[s] a dumping ground for an
odd assortment of low-level, technology-based settings"
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/win-property-win>), and
XnView's own moderator explains how it happens: the settings are enormous because
"they have mostly been requested by other users"
(<https://newsgroup.xnview.com/viewtopic.php?t=45380>). Pennington's test is the
one to apply before building any control: "Can said annoyance be made to go away
for all users without requiring a preference? If so, just do that."

Three questions decide it, in order. **Does the field have a right answer the
program can work out?** Then compute it. **Do two fields decide one thing, with
one of them always winning?** Then delete the loser. **Is the field a number
standing in for a decision nobody made?** Then make the decision.

Seven places in this program fail one of those — eight fields, because one row
below is the same idea written twice. §3.2 starts from all 110 being placed on a
page; this is the argument for taking six of those controls off the pages before
they are built. **The control goes and the key stays**, so nothing on disk breaks
and §4.4's no-rename rule holds; the one exception is
`image_view.scroll_navigation`, which is the single key the plan proposes to
migrate (§4.4).

| Field | Where | Why it should not be a setting | What it becomes |
|---|---|---|---|
| `image_view.nr_loaded_images` | `src/config/mod.rs:357-359`, default 512 at `src/config/defaults.rs:90-97` | It is trimmed to what the RAM budget holds: `configured.min(fits / 2)` (`src/cache/policy.rs:136-151`, called from `src/cache/store/mod.rs:475-483`). The number in the file is the number in force only while nothing is resident to measure — `if resident_count == 0 \|\| resident_bytes == 0 { return configured; }` (`src/cache/policy.rs:142-144`) — which is the state at startup and again after every folder is opened, because `set_paths` empties the cache (`src/cache/store/mod.rs:216-236`). From the first decode onwards the trim applies, and what it trims to depends on the monitor — see below | No control. The budget is the setting; the radius is a readout beside it — "about 22 either side here" |
| `cache.upload_budget_ms` | `:260`, default 8 at `src/config/defaults.rs:41-43` | The misfiled comment in §11.1 is its derivation: half a frame at sixty a second. The frame time is measured every frame and already read elsewhere (`input.stable_dt`, `src/view/image_view/interaction.rs:58`). GNOME's principle is the whole answer — "If something can be done automatically, do it automatically" (<https://developer.gnome.org/hig/principles.html>) | Computed from the frame budget. Left in the file for the person debugging a stutter, with no control and no page |
| `image_view.gpu_resident_images` | `:360-362` | The doc comment on `cache.gpu_budget_mb` states the objection against its own neighbours: the counts "bound how *many* textures stay resident, which is not a memory bound: two hundred thumbnails and two hundred sixty megapixel photographs are the same number and a thousandfold difference in what the card is holding" (`:247-254`). GPU residency has already been bounded by bytes in the work that has shipped (`plan.md:37`) | No control. The bytes bound the card; the counts are vestigial |
| `grid_view.gpu_resident_thumbnails` | `:453-455` | As above | No control |
| `image_view.context_menu` **and** `grid_view.context_menu` | `:387` and `:457` | One idea written twice: the same type, the same empty default (`default_ctx_menu()`, `src/config/defaults.rs:165-167`), and the same table would edit both | One control over both lists, with a column saying where each entry appears (§7.11) |
| `image_view.scroll_navigation` | `:374` | A bare boolean answering "does the wheel do anything" and unable to answer "what". It is the first step of a mouse table that was never built | Migrated into `mouse.wheel`, which is a real question with real answers (§6.5, §9) |
| `raw.highlight_mode` | `:96-99`, a `u8` | A number pretending to be an enum — 0 clips, 1 leaves unclipped, 2 blends, 3 and up rebuild — handed to LibRaw as an `i32` with no validation (`src/decoder/raw/mod.rs:106`). "Highlight mode 7" is not a decision anybody makes; it is a decision nobody made | Named choices with a passes box on the last one (§5.7) |

The first row deserves the arithmetic, because it is the clearest case of a
number that does not mean what it says. The 4096 MB default budget is split
seven-eighths to the image store (`THUMBNAIL_SHARE`, `src/app/stores.rs:15`,
`:96-103`), giving 3584 MiB, and a quarter of that is held back for full
resolution copies (`FULL_RESOLUTION_SHARE`, `src/cache/store/mod.rs:39`, `:157`,
`:176`), so the cache the radius is measured against holds 2688 MiB. What sits in
it is not the photograph: every decode is reduced to the monitor's longest edge
rounded up to a multiple of 512 (`set_display_edge`,
`src/cache/store/mod.rs:266-276`, fed from `longest_edge_in_pixels`,
`src/app/mod.rs:797`, `:881-886`). A 24 megapixel frame is therefore about 45 MiB
on a 4K monitor and about 11 MiB on a 1080p one, and `budgeted_radius` — which
also holds three quarters of the budget for the window it is centred on
(`WINDOW_SHARE`, `src/cache/policy.rs:126`) — turns 512 into roughly **22** on
the first and **90** on the second. So the shipped default is a ceiling the
program reaches only in the moment before it has decoded anything, and from the
first decode onwards the number in force is a property of the screen the user
happens to have. The source says as much in the one place nobody looks: "what is
actually held is trimmed to what fits, so this only has to be large enough not to
be the limit itself" (`src/config/defaults.rs:90-94`).

`cache.decode_threads` (`:237-239`) is the interesting near-miss and it stays.
Zero already means "choose for me" and is the default
(`src/config/defaults.rs:15-18`), so the field is an escape hatch rather than a
question, which is the right shape. What is wrong is that `Loader::new` spawns
exactly what it is told and `.expect`s each thread (`src/cache/loader.rs:106-133`),
so a number the operating system will not grant takes the process down at
startup rather than being refused; the cap belongs at the consumer and §3.6
places it. It is worth more than any page in the settings window.

The general rule, stated once so the rest of the plan can be measured against it:

> **A new field is added only when two reasonable people would choose
> differently. Everything else is a decision, and the decision belongs in the
> code.** A field that exists because nobody could be bothered to choose is a
> question asked of every user for ever.

---

### 11.3 Where the line between plain and advanced falls

NN/g's rules for progressive disclosure bite here
(<https://www.nngroup.com/articles/progressive-disclosure/>): "initially, show
users only a few of the most important options"; "offer a larger set of
specialized options upon request"; the way from the first level to the second
must be visible and must say what is behind it; and "designs that go beyond 2
disclosure levels typically have low usability because users often get lost when
moving between the levels".

Two levels. Not three, and not a page called Advanced — Microsoft names
"General", "Advanced" and "Settings" as page labels to avoid, and adds "Don't
call properties advanced based solely on technological measures", which is
precisely what an Advanced page in this program would be, since every field on it
would be there for being about the cache. §3.2 and §4.5 already rule out those
page names; this is the reason they are ruled out.

The split cannot be made from frequency data, because there is none. It can be
made from a property of each field that is knowable today:

> **A field is plain when a person can tell from the screen whether they have got
> it right. It is advanced when they need a number, a unit or a consequence
> explained to them first.**

`grid_view.images_per_row` is plain: drag it, count the columns, stop. It stays
plain even though it is arithmetically identical to a cache radius
(`src/app/stores.rs:54`), because the arithmetic is not what the person is
looking at. `cache.ram_budget_mb` is advanced: nothing on the photograph changes
when it moves, and the only feedback the program has is a memory readout in the
side panel (`src/app/panels.rs:206`) which is nowhere near the control and does
not say how much the machine has (§8.5). That is a property of the *feedback*,
not of the technology, which is what Microsoft asks for and what an
Advanced-versus-Basic split never delivers.

Applied to the hundred and ten today, and then to the file the plan leaves
behind — 145 keys, of which the six of §11.2 have no control (§3.2, §12):

| Level | What is on it today | Today | Finished |
|---|---|---|---|
| Plain — visible with nothing opened | Every `raw.*` decoding choice and which half of a raw+JPEG pair is browsed (6); the keyword list, how many recent tags are kept, the panel width, advancing after a mark (4); the destination slots and the rejects folder's name (2); text size, what the side panel lists, whether the last session comes back (3); the overlay's corner, its text and its size, how many are shown side by side, whether to wait for the decode, the white frame, whether a small picture is enlarged, what the status bar says (8); how many fit across, the shape of a cell, the thumbnail detail, the filmstrip's height, what the caption says (5); all five slideshow fields (5) | 33 | 61 |
| Second — behind a per-group disclosure on the same page | The arithmetic: five of the six `cache.*` fields, the thumbnail preload radius, the decode ceiling, the ICC profile, the keyword catalogue path, and the two tables of external commands | 11 | 18 |
| No control — the key stays in the file | The six of §11.2 | 6 | 6 |
| Not on a page at all | The 60 shortcut fields, which have their own page, their own editor and their own cheat sheet (§4.7, §10.5) | 60 | 60 |
| | | **110** | **145** |

The counts are exact and add up on both sides, which is the point of writing them
down: a split that does not account for every field is a split that will quietly
grow a third level. Today's column adds to the 110 in the file. The finished
column adds to the 145 keys the plan leaves and, dropping the six that have no
control, to the **139 rows** the window draws (§3.2, §12).

The thirty-five fields §6 adds have to be assigned by the same test rather than
inherited from the page they land on. **Seven of them are second level:**

- `image_view.zoom_step_factor` and `image_view.zoom_step_max` — the doubling
  `Space` applies and the magnification at which it wraps back to fitted
  (`src/view/image_view/zoom.rs:45`, `:11`). Neither is legible from one press;
  you have to know that `Space` and `+` are two independent increments before
  either number means anything (§6.4).
- `group.max_gap`, `group.tolerance` and `group.min_frames`
  (`src/organize/group/mod.rs:45-53`). A tolerance of 12 has no unit a
  photographer owns, and the consequence — which frames join and which come apart
  — is a property of the folder rather than of the screen.
- `browsing.filter_follows_folder` (`src/app/mod.rs:297` never resets the rules).
  Whether a filter carries into the next folder cannot be told from one folder;
  it takes two, and today it takes two with the filter bar hidden (§6.7).
- `tags.sidecar_naming` (`src/annotations/sidecar.rs:14-18`). The consequence is
  which file some other program finds, which is not on this screen at all.

**The other twenty-eight are plain**, and most of them obviously so: change
`general.backdrop` and the grey behind the photograph changes
(`src/view/image_view/layout.rs:12`); change `grid_view.badges` and the cells say
something different (`src/view/grid_view/cell.rs:27-28`); change
`image_view.zoom_step` and one press of `+` goes a different distance
(`src/view/image_view/input.rs:17`); change any row of the mouse table and the
gesture does what the row says (§6.5). The browsing and startup defaults are
plain for the same reason a step is: the next folder, or the next launch, is the
readout. That is 33 + 28 plain and 11 + 7 second level, which is the arithmetic
in the table.

One field is genuinely borderline — `grid_view.thumbnail_resolution` has no
visible effect of its own, but it is drawn on the plain level anyway, immediately
under `images_per_row`, because "why did my thumbnails go blurry when I made them
bigger" is the question the pair exists to answer and separating them is what made
it hard to answer elsewhere
(<https://newsgroup.xnview.com/viewtopic.php?t=47571>). §5.7 puts the two
controls side by side for the same reason.

Two properties of this split are load-bearing and easy to lose.

**Disclosure hides depth, never access.** A collapsed group is still indexed, so
searching "vram" opens the group and lands on the control (§3.4). NN/g's stated
failure mode for progressive disclosure is hiding without a discoverable path,
which relocates complexity rather than reducing it. The search box is what makes
the second level cheap; without it the collapse is a wall.

**The disclosure is inside the group, not a separate page.** Microsoft: "Don't
scroll property pages" and "Don't nest tabs"; Android puts the threshold for
subscreens at fifteen settings
(<https://developer.android.com/design/ui/mobile/guides/patterns/settings>). Of
the 139 rows the finished window draws, sixty-nine sit on **Keys and mouse** —
the sixty shortcut fields, whose home is the key map even though §4.7 mirrors
each of them beside the setting it triggers, plus `image_view.user_actions` and
the eight rows of the mouse table (§4.5, §6.2). Seventy behaviour rows are left
for the other ten pages: seven a page on average, and sixteen on the largest,
**Opening a folder**, once §6's browsing, stacking and startup defaults land on
it. Sixteen is one past Android's threshold, and it is exactly the case the
per-group disclosure is for: four of those sixteen are second level by the test
above — the three grouping thresholds and whether the filter follows the folder —
so twelve rows are open when the page is, arranged in groups rather than in one
column (§4.5). Nothing forces a third level and nothing needs a subscreen. A page
that fits is a page you can scan, and scanning is what search is competing with.

The corollary about controls belongs to §5 and is stated there field by field: a
control covering every value beats a list of presets unless the values between
the presets are meaningless rather than merely unpopular (§5.7).

---

### 11.4 Defaults, and the six that are wrong

Every field placed behind a disclosure is a promise that its default is right.
Microsoft states the obligation plainly: "Assume that most users won't change the
settings." NN/g measured it: "users rarely utilize fancy customization features,
making it important to optimize the default user experience, since that's what
most users stick to"
(<https://www.nngroup.com/articles/the-power-of-defaults/>). The GNOME
configuration guidance is the strongest form — "a well-designed interface does
not require configuration and will operate effectively with no modification"
(<https://wiki.gnome.org/Design(2f)HIG(2f)Planning(2f)Configuration.html>).

Most of this program's defaults are good, and several are argued for in the
source where nobody can read them — the four-gigabyte budget "generous on a
modern machine and still small enough not to push a 16 GB laptop into swap"
(`src/config/defaults.rs:9-13`), the 3:2 cell aspect against the forty-four per
cent of grey a square grid drew (`src/config/mod.rs:440-447`), the filmstrip off
because "it is a second row of pixels competing with the photograph for the
window" (`:469-475`). Those are the right kind of default: chosen, and defended.

Six are wrong, in three different ways.

**Defaults that make a feature look unimplemented.**

| What | Where | Why it is wrong | What it should be |
|---|---|---|---|
| `default_ctx_menu()` returns an empty vector, and `show_context_menu` returns before drawing anything when the list is empty | `src/config/defaults.rs:165-167`, `src/actions/user_action.rs:142-149` | Right-clicking the photograph or a grid cell does nothing at all out of the box. The gesture does not read as unconfigured, it reads as unimplemented — and the one menu that exists on a fresh install is on the zoom percentage (`src/view/image_view/bottom_bar.rs:283`), which nobody will find by accident. NN/g's Business Insider case is the same shape: because only some elements responded, users stopped expecting any of them to (<https://www.nngroup.com/articles/tooltip-guidelines/>) | A built-in menu that exists before anything is configured, with the user's own entries appended below a separator. §7 has the full map |
| `grid_view.caption_format` defaults to `"{name}.{ext}"`, and the caption line is painted only when badges are cycled to `Full` | `src/config/defaults.rs:304-306`; the gate at `src/view/grid_view/mod.rs:482` and the paint at `src/view/grid_view/cell.rs:131-139`; `Badges` defaults to `Marks` at `src/view/grid_view/cell.rs:27-28` | Two defaults contradict each other: the caption is configured and, with the shipped badge mode, is never drawn. The strip under the thumbnail *is* drawn — it carries the marks — so the space is already spent and the line is the part that is missing. This is FastRawViewer's reject bug in miniature: the function was live, a preference the user had never seen gated it, and the shortcut list said so only inside a settings screen (<https://www.fastrawviewer.com/node/577>) | Either badges default to `Full`, or the caption row states the dependency and carries the button that satisfies it |

The second of those deserves its own note, because the contradiction is between
two deliberate decisions rather than between a decision and an oversight. The
doc comment above `Badges` says why there is no field: "Cycled with one key
rather than settled in the configuration, because how much a person wants to see
changes with what they are doing: everything while triaging, nothing while
looking" (`src/view/grid_view/cell.rs:19-21`). That reasoning is right and the
cycling key stays. What it does not settle is *which of the three the program
starts in*, and the caption default was written as though the answer were `Full`.
§4.8's rule resolves it without adding a question: the cycle is state, the
starting point is a setting, and they live in different files.

**Defaults that were never given a field.**

| What | Where | Why it is wrong | What it should be |
|---|---|---|---|
| `Rules::default()` sets `flag: FlagRule::Any`, while a variant of the same enum is documented as "Everything except the rejects, which is the one people leave on" | `src/view/narrow.rs:36-48`, `:58-59`; the whole `Narrowing` is `Default::default()`-ed per session at `src/app/mod.rs:228` | The source names the value people leave on and gives them no way to leave it on. The defect is the missing field, not the shipped value: a filter that hides files on startup must be something the user asked for. Android's rule is that a default should "represent the default most users would choose" *and* "be neutral and pose little risk" (<https://developer.android.com/design/ui/mobile/guides/patterns/settings>), and those pull in opposite directions here | `Any` stays the shipped default; a settable `browsing.flag` — "start every folder like this" — makes the comment true (§6.7) |
| The delete confirmation threshold is hardcoded: one photograph goes to the bin without asking, two or more ask | `src/app/cull.rs:85` | The keystroke you make by accident is the single one, on the photograph in front of you; the selection of forty is deliberate. Whether that argument wins is arguable — the bin is recoverable, and the code's own comment makes the opposite case at `:62-64` — but there is no field, no name and nothing on screen that says where the line is | A named threshold, defaulting to asking above one, with a permanent delete that always asks whatever it says (§6.9) |

**Defaults expressed in a way that cannot be defended.**

| What | Where | Why it is wrong | What it should be |
|---|---|---|---|
| `general.text_scaling` defaults to `1.25`, a bare multiplier applied to every text style on top of whatever the desktop already scales by, unclamped, read once | `src/config/defaults.rs:49-51`, applied at `src/app/mod.rs:150` and `:888-894` | `0.0` sets every font size to zero, including the menu that would undo it, and there is no control anywhere to undo it with. "A single typo, or a stray comma could break *everything*" is the whole complaint against file-only settings (<https://shkspr.mobi/blog/2020/06/theres-nothing-i-hate-more-than-text-config-files/>) — and here the typo is a single character. There is a second trap waiting: `apply_text_scaling` clones the *current* style and multiplies it, so calling it again to make the field live would compound 1.25 into 1.5625 | Per cent with a floor, applied from a stored base style rather than from the current one, so it is idempotent. §3.6 places the floor and §3.5 makes the field live |
| `image_view.frame_size_relative_to_image` defaults to `0.2`, unclamped | `src/config/defaults.rs:133-135`, used at `src/view/image_view/canvas.rs:356-370` | `5.0` makes the stroke five times the shorter edge and the picture is clamped to one pixel (`:365-367`). The default itself is fine; the absence of a bound is not | A percentage, 0 to 25, clamped at the consumer rather than on load (§3.6) |

Changing a default is safe here in a way it is not in most programs, and the
machinery is already built. `src/config/migrate.rs:15-18` states the rule and
enforces it: "Every one of them checks that what it finds is the *old default*
before touching it: a setting the user has actually changed is theirs, and a
migration that flattens it is worse than the clash it was avoiding." A default
can therefore be moved without stepping on anybody who chose otherwise, provided
a migration step is written and `CURRENT` is bumped (`:25`). That is the licence
to fix the six above rather than adding a field for each of them.

---

### 11.5 Several routes, one definition

The owner asked for one setting to be reachable by several routes, and is right
to. Microsoft's caution is only about the reverse — "Don't make commands only
available through context menus"
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus>) — and
Apple's is the same: "Always ensure that contextual menu items are also available
as menu commands", because "a contextual menu is hidden by default and a user
might not know it exists, so it should never be the only way to access a command"
(<https://leopard-adc.pepas.com/documentation/UserExperience/Conceptual/AppleHIGuidelines/XHIGMenus/XHIGMenus.html>).
Several routes *to* one control is the pattern everyone endorses; several
*copies* of one control is where it goes wrong. §3.3 lays out the routes and §7
the menus; three rules keep the copies from appearing, and they are cheap now and
expensive to retrofit.

**One definition, many renderings.** A field has one row in the registry, which
carries its name, its sentence, its unit, its range and its accessors, and every
route draws that row (§3.7). No route may invent a range. The program already
shows what happens when two renderings drift: the slideshow window bounds
`seconds_per_image` to `1..=600` and `percent_zoom` to `0.0..=200.0`
(`src/app/panels.rs:101`, `:126`) while the file accepts anything, so the same
field has two ranges depending on which route was taken. With one row there is
one range, and §3.6 decides what happens when the file holds a value outside it.

**One name everywhere, and the name is the path.** `cache.ram_budget_mb` is what
the row is keyed on, what search matches, and what "copy setting name" yields —
never the label. nomacs stored keyboard shortcuts under their *translated* names
and broke every one of them when the interface language changed
(<https://github.com/nomacs/nomacs/issues/1539>). Nothing in the program is
translated today and §13 keeps it that way for now, which makes the lesson free
to take and expensive to skip.

**One unit everywhere, spelled out.** Four fields express "how much" in four
notations, and no two of them are readable together:

| Field | Where | Written as | Reads as |
|---|---|---|---|
| `general.text_scaling` | `src/config/defaults.rs:49-51` | `1.25` | a multiplier |
| `grid_view.cell_aspect` | `:368-370` | `1.5` | a ratio |
| `image_view.frame_size_relative_to_image` | `:133-135` | `0.2` | a fraction |
| `slideshow.percent_zoom` | `:505-507` | `25.0` | a percentage |

They should read `125 %`, `1 : 1.50`, `20 %` and `25 %`. Only two of the hundred
and ten are ever drawn with their unit attached, both in the slideshow window
(`src/app/panels.rs:102`, `:127`); the program's only other suffixed widgets are
on the grouping gap, which is not a configuration field at all
(`src/ui/filter_bar.rs:135`, `src/view/organize/group/mod.rs:58-61`). Where the
suffix goes and how the value is snapped is §5.4's.

Every route ends at the same page and says so: the last row of a context menu is
`More settings… (<page name>)`, always, so somebody hunting for something the
menu does not carry still ends one click from the right place, having learned the
page's name (§7.2). Two repairs come first, and neither is a settings question:
a drag with any button pans the photograph, so a right-button drag releases into
whatever menu is registered (§9), and in the directory tree
`label.secondary_clicked()` *opens the folder*, so the gesture is not merely
unused but spoken for and inverted (`src/ui/tree.rs:266-268`, §7.1).

---

### 11.6 Names, and the words this program invented

Users cannot find what they cannot name. The darktable naming thread is the
clearest statement of the cost — "none of these modules do the thing that is
written on the tin, and that close to none of these are the industry-standard
terms", and "I'm the guy who is always searching for the modules even though I
made my own tool set, because I simply have no clue where the modules are
located" (<https://discuss.pixls.us/t/another-dt-ui-discussion/58799>).
Microsoft's rule is the fix: "Present properties in terms of user goals, not
technology."

This program's prose is unusually good, and its invented vocabulary is mostly
kept where it belongs — inside. The rule that produced that is worth naming
because the settings window will test it forty-seven more times:

> **An internal word may name a type, a module or a function. It may not name a
> control, a label or a configuration key.**

Judged against it, six terms, three of which are already right:

| Term | Where it lives | What it means | Does it reach the person? | What the interface should say |
|---|---|---|---|---|
| **errand** | `src/ui/destinations.rs:13-27` | moving or copying to a destination slot | **No** — `Errand::verb()` yields "Move" and "Copy", and the window title reads "Move to…" (`:60`) | Nothing. This is the pattern to copy: an internal noun with a `verb()` that is the only thing anybody sees |
| **narrowing** | `src/view/narrow.rs:1-8`, the `Narrowing` type at `:138` | filtering and ordering the open folder | **No** — the only user-facing use is prose in a binding description, "the bar that narrows and orders the folder" (`src/config/bindings.rs:217`) | Nothing. The bar is the *filter bar*, which is what every other program calls it and what the binding is already named. The new fields take `browsing.*` for the same reason (§4.9, §6.7) |
| **preview tier** | `src/cache/store/previews.rs:1` | the camera's own thumbnail, standing in until the decode lands | **No** on screen — the memory readout says "Thumbnails standing in" (`src/app/panels.rs:218-221`) — but **yes** in the file, as `cache.previews_resident` (`src/config/mod.rs:240-244`) | "Camera thumbnails standing in", the readout's own words, so the setting and the number it moves are named the same |
| **resident** | `cache.previews_resident`, `image_view.gpu_resident_images`, `grid_view.gpu_resident_thumbnails` (`src/config/mod.rs:244`, `:362`, `:455`) | held in memory or on the card | **Yes**, in the file. It is a cache word, not a photographer's word | "Kept on the graphics card". Two of the three lose their control in §11.2 anyway |
| **standing back / standing forward** | `src/config/bindings.rs:118-129`, labelled "Frame standing for a stack" and "Next frame standing for a stack" | which frame of a closed burst is the one on show | **Yes**, in the keyboard editor and the cheat sheet | "Which frame shows the stack" and "Show the next frame instead". The present participle is fine prose and a poor label: it cannot be searched for, because nobody will type "standing" |
| **stack** | `src/view/stacks.rs`, and `Kind::FocusStack` at `src/organize/group/classify.rs:16-27` | *two different things* | **Yes**, both | See below |

**"Stack" is the one real naming defect**, and it is three defects wearing one
word.

The same object is called three things in two adjacent pieces of interface. The
filter bar's toggle says "Stacks" (`src/ui/filter_bar.rs:97`); the count drawn
beside it says "41 runs · 903 frames" (`:117`); and the status bar under the
photograph calls the same thing a series, a stack, an hdr or a timelapse
depending on what kind it is. A person cannot search for a thing that has three
names, and cannot tell that the three are one thing.

The status-bar wording comes from `Place::describe()`
(`src/view/stacks.rs:81-92`), drawn at `src/view/image_view/bottom_bar.rs:138`,
which builds its line from `Kind::folder()` — a function whose documented job is
to give "the stem of the folder a group of this kind is tidied into"
(`src/organize/group/classify.rs:42-51`). It returns `hdr`, `stack`, `timelapse`,
`series`. A folder-naming convention is being used as display text, which is why
the bar says "series" in lower case and why, for a focus stack, it reads *"stack
3 · frame 4 of 17 · stack 3 of 41"* — the word "stack" meaning the kind of group
in one clause and the group's number in another. `Kind::label()` sits immediately
above it and returns "HDR bracket", "Focus stack", "Timelapse", "Series"
(`:33-40`); it is not used here.

And `describe()` prints the same field twice: `self.stack` appears as the series
number and again as "stack N of M" (`src/view/stacks.rs:87`, `:90`), where its
own doc comment's example — "series 3 · frame 4 of 17 · stack 6 of 41" — shows
two different numbers. The test beneath it records what the code actually
produces: `"series 1 · frame 1 of 3 · stack 1 of 2"` (`src/view/stacks.rs:527-530`).

The decision: **one word, "stack", for the object; `Kind::label()` for the kind,
always spelled in full so "Focus stack" is never shortened to "stack"; and the
filter bar's count says "41 stacks · 903 frames".** That is three one-line
changes and it makes the feature searchable for the first time.

Two smaller naming rules, both from the research and both cheap:

- **No generic verbs in labels.** Android: don't "use generic terms, such as:
  Set, Change, Edit, Modify, Manage, Use, Select, or Choose", and don't repeat
  the section title. This is why the entry that opens the window is
  `Settings ▸ All settings…` rather than `Settings ▸ Settings…` (§3.3), and why
  none of the eleven pages is called General (§4.5).
- **Show the state, do not describe the setting.** Android again: "show the
  setting status instead of describing the setting". nomacs #1090 is a pure
  wording complaint of the same family — "Color Settings" should be "Theme", and
  a "General" heading inside a "General" section is noise
  (<https://github.com/nomacs/nomacs/issues/1090>).

---

### 11.7 What stays in the file

The window does not have to carry everything, and this is what keeps the count
down: a field can lose its control and keep its key
(`cache.upload_budget_ms` is the case in §11.2), and a hand edit can produce a
value the window cannot (§3.6). ImageGlass documents the relationship the right
way round — the GUI is the primary editor and the file is the escape hatch, with
one sentence saying which file the program writes: "This is the only file
ImageGlass writes to when it saves" (<https://imageglass.org/docs/app-configs>).
That only holds if the file survives being written by the window, and today it
does not. Three things have to change; §3.4 owns the footer that gives the file
an address and the state of `examples/config.json`.

**The window never rewrites a field it did not edit.** `Config::save` serialises
the whole struct and writes the whole document (`src/config/load.rs:42-45`), and
no struct in `src/config/` carries `deny_unknown_fields`, so a key the build does
not recognise is accepted and ignored on the way in (`src/config/load.rs:166-183`)
and dropped on the way out. That is Geeqie's bug, whose reporter diagnosed it
himself (§2.1). Today the blast radius is small, because the file is written from
two places in a session (`src/app/settings.rs:70`, `:94`) and once more by the
loader after a migration (`src/config/load.rs:101`); a settings window that saves
on every gesture makes it certain. The fix is small and belongs with the window
rather than after it: keep the `serde_json::Map` produced at load
(`src/config/load.rs:133-142`) and merge the serialised struct into it on save
rather than replacing it, recursing one level into each section so a note written
inside `cache` is not lost when that section's object is replaced wholesale. What
the merge does *not* give back is key order: `serde_json` is taken without
`preserve_order` (`Cargo.toml:17`), so its `Map` is a `BTreeMap` and a merged
document comes out alphabetical where writing the struct follows the declaration
order. That is a smaller, separate loss, and one feature flag fixes it if it is
thought worth fixing.

**A comment costs the whole file, and should cost nothing.** serde_json
implements RFC 8259, which has no comments, and `from_json` parses the entire
document into a map before it looks at any section
(`src/config/load.rs:133-142`). So a single `//` sets `partial`, blocks all
saving for the rest of the session, hands back the defaults for every section,
and says so once, for 6.6 seconds, through a band that then fades
(`src/config/load.rs:24-32`, `src/ui/notice.rs:16`, `:19`). The README promises
the opposite in as many words: "A section the viewer cannot make sense of costs
that section and nothing else, and a file that was only partly understood is
never written back over — so one misplaced comma cannot quietly replace
everything you had configured with the defaults" (`README.md:598-601`). Half of
that is true and half is not, because the section-by-section reading happens
*after* a single `from_str` over the whole file. So: **strip line and block
comments in a pre-pass before `from_str`**, the same shape and the same place as
the byte-order-mark strip already there (`src/config/load.rs:126-131`), which
exists for exactly this reason — a file somebody had merely opened in Notepad and
saved "parsed as nothing at all and silently handed back the defaults for
everything". Say plainly that comments are not written back, because they cannot
survive `to_string_pretty`. And once unknown keys survive a round trip, a note
can be a key: `"_note": "1536 because the laptop has 8 GB"` is preserved by the
merge, sits beside the value it describes, needs no parser change, and survives
the window as a comment would not.

**A hand edit made while the program is running is invisible, and then
destroyed.** The configuration is read once (`src/app/mod.rs:147`), and the next
in-app save writes the in-memory copy over whatever was edited. That is a silent
data-loss path of the same family as the six in `plan.md` §2.1, and it becomes
reachable the moment there is a window somebody might touch. **Record the file's
modification time at load and read it again before every write; if it has moved,
do not write.** Say what happened, and offer *Read the file again* or *Keep what
is on screen and overwrite*. That needs no watcher, no thread and no dependency.
Watching the file is optional and, if built, watches to *notice* and never to
reload: `notify` is already a dependency (`Cargo.toml:23`) and `DirectoryWatcher`
already starts, stops and drains a watcher (`src/app/watcher.rs:35-77`), but an
editor saving in two steps presents a truncated document, which parses as
nothing, which sets `partial`, which blocks saving for the session.

---

### 11.8 The failure mode is finding, and the budget that keeps it honest

The thing to avoid is documented in detail, and it is not "too many settings". It
is a window nobody can navigate. darktable's own preferences rework described the
state it replaced as "two pages of miscellaneous preference options with lots of
scrolling and hunting required to find what you're after"
(<https://github.com/darktable-org/darktable/pull/4747>), and five separate
people over several years asked darktable for a search box in preferences —
issues #3423, #6706, #6174, #3604 and #9598, all closed by a stale bot
(<https://github.com/darktable-org/darktable/issues/3423>). That is the
best-evidenced request in the whole of the research, and it is a request for
*finding*, not for fewer options. Which is why §3.4's search box is the
load-bearing part of the window, and the eleven page names are mainly the
headings its results sit under.

What this chapter contributes to that window is one mechanism, written as
something that can fail, because a principle nobody can fail is decoration:

> **A field budget. A new field requires either the two reasonable people who
> would choose differently, or the name of somebody who asked.** It has failed
> the moment the count rises with neither written down.

§6 applies that test row by row to the fields it proposes, and the plan's total
comes to **139 rows on the pages and 145 keys in the file** (§3.2, §12).
Thirty-five fields are added and `image_view.scroll_navigation` retires into one
of them, which is how 111 keys become 145; six of those keys have no control of
their own under §11.2, which is how 145 keys become 139 rows. That is a
defensible number for a specific reason: what is mostly being added is
persistence rather than choice (§3.2). It stops being defensible the moment a
field is added because it was easier than deciding. XnView's honest explanation of how a settings screen becomes enormous —
"they have mostly been requested by other users" — is what the alternative looks
like after fifteen years, and the answer is not to refuse users but to write the
name down beside the field.

Four things this chapter wants ruled out, each already in §13: no page named for
what a field is made of, no per-folder or per-monitor scope, no named profiles or
kits, and no settings screen that is the only route to anything a person can see
the effect of.

## 12. The plan

Nine stages, in the order they should be done. Each is a commit or a small run of them, each
leaves the viewer working, and each says what finishes it.

The arithmetic the whole plan is measured against is §1's and §3's. Today the configuration
file holds 111 keys — 110 settings and `version` (`src/config/mod.rs:18-49`). Sixty of the 110
are shortcuts and the keyboard editor reaches all sixty, drawn as 69 rows; the slideshow window
reaches three more; **forty-seven are reachable nowhere in the running program**, six of which a
key nudges for the session and never writes back, leaving forty-one that cannot be changed at
all while it runs. Twenty-six do not take effect until the next launch — twenty-five wholly and
`grid_view.filmstrip_height` by half. There are 33 hover-help call sites in 11 of the 139 source
files, 19 numeric controls in 40,504 lines, and one right-click menu that draws anything on a
fresh install. When the nine stages are done: every setting that keeps a control has one, the four
that lose theirs say on the page where their value now comes from, one row still needs a restart,
roughly a hundred surfaces explain themselves, and about twenty carry a menu.

The order is what unblocks what, with one deliberate exception. The registry — the table both
the file and the window read — is the foundation: the pages, the search, the changed-from-default
marker, the per-field reset, the restart footer, the load-time check, the export and the cheat
sheet are all views over it, and none of them can be written first. It ought to be Stage 1. It is
Stage 2, because everything in Stage 1 is a day or two each, needs nothing from the registry, and
is felt the moment it lands. A default context menu changes the program on the day it ships; so do
seventy tooltips. Making the twenty-six restart-bound fields take effect is the most valuable work
in the plan and the most expensive, and it is last. §5.9 states what putting it last costs and what
follows from it: a control whose effect appears only at the next launch is worse than a *number*,
because it looks like it is doing something, so where the value cannot take effect until the stores
are rebuilt the rail is not drawn at all. The five store-bound numbers on **Speed and memory** are
therefore typed boxes carrying the restart badge in Stage 3 and become rails in Stage 8. **Raw
files** is radios and ticks rather than rails — a radio cannot pretend to be taking effect the way a
dragged rail can — so it is drawn in full in Stage 3 with the badge on every store-bound row, and
loses the badge at the same rebuild.

One rule governs the whole of it, from §11.2, and the plan can be measured against it: **a new
field is added only when two reasonable people would choose differently, or somebody outside this
repository has asked for it by name.** Thirty-five fields are added under that rule (§6, counted
in §3.2) and six of today's controls are taken away under it (§11.2), so the file grows from 111
keys to 145 and the pages carry 139 rows — §3.2's 144, less the five that item 7 of Stage 3 takes
off. Three further keys come from outside §6's list, and each is named where it is built:
`grid_view.filmstrip_visible`, which is §3.5's split of the filmstrip's height rather than a new
choice; `menus.settings_rows`, which is §7.11's; and `general.last_settings_page`, which is a
remembered position rather than a choice and has a registry row but no control. What is mostly being
added is persistence, not choice.

---

### Stage 0 — Stop the program editing files nobody asked it to edit

None of this is a settings question. Three of the repairs are the difference between a settings
window that reads the configuration file and one that silently rewrites it, two of them block
every right-click menu in Stage 5, and the rest are wrong on their own terms and cheap.

1. **Turn egui's clamping off before any control is drawn.** `SliderClamping::Always` is the
   default and it clamps *existing* values, not only edited ones: the clamped value is written
   back through the borrowed reference and the response is then marked changed
   (`egui-0.33.0/src/widgets/slider.rs:73-75`, `drag_value.rs:75`, `:514-526`, `:666-668`). The
   slideshow window declares `.range(1..=600)` (`src/app/panels.rs:102`) and hands `changed`
   straight to `save_settings()` (`src/app/settings.rs:70`), so a hand-written
   `"seconds_per_image": 900` is rewritten to 600 on the frame the window is opened, with nobody
   touching anything. Every control in the plan passes `SliderClamping::Edits` or
   `clamp_existing_to_range(false)`. Without this, a settings window is a program that quietly
   edits the file it was opened to read (§5.3).
2. **Merge on save; never replace.** `Config::save` serialises the struct and writes the whole
   document (`src/config/load.rs:42-45`), and no struct in `src/config/` carries
   `deny_unknown_fields`, so a key this build does not know is accepted in silence
   (`:166-183`) and dropped on the way out. That is Geeqie's defect, whose reporter diagnosed it
   himself — "a direct consequence of the approach of regenerating config file from scratch each
   time Geeqie is closed" (<https://github.com/BestImageViewer/geeqie/issues/569>). Keep the
   `serde_json::Map` produced at load (`src/config/load.rs:133-142`) and merge the serialised
   struct into it, recursing one level into each section. Today the file is written twice a
   session; a window that saves on every gesture makes the loss certain (§11.7, §2.1).
3. **Read the file's modification time before every write, and refuse if it has moved.** The
   configuration is read once (`src/app/mod.rs:147`) and the next in-app save writes the in-memory
   copy over whatever was hand-edited meanwhile. Say what happened and offer *Read the file again*
   or *Keep what is on screen*. No watcher, no thread, no dependency (§11.7). The recorded time has
   to be replaced after every *successful* write, or the program's own save is the edit the next one
   refuses — and that includes the write nobody sees: `fetch_cfg` calls `save()` itself when a
   brought-forward file has been migrated, before any interface exists (`src/config/load.rs:99-104`),
   so on a migrated configuration the very first in-app save would otherwise refuse. Re-read the time
   from the path rather than assuming the one that was written: item 4 replaces the write with a
   rename over the original, which produces a different file with a different timestamp.
4. **Write `config.json` and `session.json` atomically.** Both are a plain `fs::write`
   (`src/config/load.rs:45`, `src/session.rs:110`) while sidecars already write to a temporary and
   rename over the original (`src/annotations/sidecar.rs:88`). The keyboard map and the per-folder
   positions are the two things a person cannot rebuild, and they are the two written without a
   rename (§2.1).
5. **Strip line and block comments in a pre-pass before `from_str`.** `from_json` parses the whole
   document before it looks at any section (`src/config/load.rs:133-142`), so one `//` sets
   `partial`, blocks all saving for the session and hands back every default — while
   `README.md:598-601` promises the opposite. The same shape and the same place as the
   byte-order-mark strip already there (`src/config/load.rs:126-131`). Say plainly that comments
   are not written back (§11.7).
6. **Cap the decode pool inside `Loader::new`.** `worker_count` arrives from the file and the
   spawn loop `.expect`s every thread (`src/cache/loader.rs:108-133`), so `"decode_threads": 1000`
   is a panic with no message. The cap goes between the file and the loop, never into `Config`:
   the file keeps what it says and the consumer refuses to act on the impossible part of it
   (§3.6, §11.2).
7. **Floor `apply_text_scaling`, and apply it from a stored base style.** `0.0` multiplies every
   text style to nothing, including the menu bar that would let anybody undo it
   (`src/app/mod.rs:888-894`). It also clones the *current* style and multiplies it, so calling it
   twice compounds 1.25 into 1.5625 — which is what makes the field live in Stage 8 (§3.6, §11.4).
8. **Restrict panning to one named button.** `is_decidedly_dragging()` is called with no button
   check (`src/view/image_view/interaction.rs:41-42`) and `PointerButton` occurs nowhere in
   `src/`, so a right-button drag pans the photograph and releases into whatever menu is
   registered. Left button only for now; the field arrives in Stage 7 (§9.2, §7.8).
9. **Give the folder tree its right button back.** `label.secondary_clicked()` performs the
   primary action while a left click expands (`src/ui/tree.rs:262-268`), so the gesture is both
   spoken for and inverted, and nothing on screen says so. `Enter` already opens the highlighted
   row (`:303-307`); add a double-click, move the highlight on a left click, and leave the right
   button doing nothing until Stage 5 gives it a menu. Taking a gesture back costs whoever has
   learned it, which is the price of having taken it in the first place (§7.6, §9.2).
10. **`"To Do"` is an alias of two colours.** It is listed under both Red
    (`src/metadata/xmp/mod.rs:168`) and Purple (`:172`), and `Label::of` returns the first match
    (`:153-163`), so Purple's entry is unreachable. The Red one goes: the same table already gives
    Red its documented Bridge name, "Select"
    (<https://asktimgrey.com/2017/02/14/white-color-labels/>). While that line is being written,
    check In Progress, Done and On Hold (`:169-172`) against a real Lightroom label set; the code's
    own comment attributes them to Lightroom and no source in the research confirms it (§2.4).
11. **Apply the restored geometry only when `restore_session` is on.** `Session::load()` is called
    unconditionally before the window is built and its geometry applied unconditionally
    (`src/main.rs:49`, `:96-110`), while `restore_session` is consulted only when deciding whether
    to *record* it (`src/app/mod.rs:588`, `:867`). Turning the setting off currently stops the
    geometry being updated and not being used (§2.6).

*Finished when:* a configuration naming a thousand decode threads starts the viewer; opening the
slideshow window leaves a hand-written `"seconds_per_image": 900` alone; a key this build does not
recognise survives a save; a `text_scaling` of `0.0` still draws a readable menu bar; dragging the
photograph with the right button moves nothing; right-clicking a folder in the tree does not open
it; and a "To Do" label draws purple.

---

### Stage 1 — The program says what it is doing

Out of dependency order on purpose. Nothing here needs the registry, each item is a day or two,
and together they answer most of what somebody arriving from Photo Mechanic or FastStone tries in
the first five minutes and finds nothing. The rule the stage follows is uniformity: NN/g's
Business Insider case is that because only *some* icons carried tooltips, users stopped expecting
them and missed the ones that existed (<https://www.nngroup.com/articles/tooltip-guidelines/>),
and the program is in exactly that state with 33 hovers in 11 of 139 files, twenty of them in
three files.

1. **A default right-click menu on the photograph and on a grid cell.** `default_ctx_menu()`
   returns an empty vector (`src/config/defaults.rs:165-167`) and `show_context_menu` returns
   before registering anything when the list is empty (`src/actions/user_action.rs:147-149`), so
   on a fresh install right-clicking a photograph does nothing at all — although the panel already
   senses clicks (`src/view/image_view/layout.rs:88`). The default is verbs, not configuration:
   fit, actual pixels, fill, compare, move to the bin, copy the path, copy the picture, show it in
   the file manager. The user's own entries are appended in their own order under a separator and
   are never reordered, renamed or removed (§7.3, §7.5, §7.11, §11.4). One level, the turns behind
   *Turn* being the single exception (§7.2). "Copy the picture" copies the file's own pixels decoded at full size and
   turned upright, not `store.decoded()`, whose `surface` may be a reduction lying on its side
   (`src/decoder/mod.rs:30-35`, `:69-75`) — or it does not ship (§7.3).
2. **The six dead words become doors.** `Flattened`, `Watching`, `Filling`, `Advancing`,
   `Comparing` and `RAW+JPEG` are bare `ui.label`s with no tooltip and no way to act on them
   (`src/view/image_view/bottom_bar.rs:144-155`). Each gains `Sense::click()`, a tooltip and a
   one-line menu carrying its verb. Two matter more than the other four: **Advancing** is the only
   place `tags.advance_after_marking` is visible anywhere, and **RAW+JPEG** the only place
   `raw.pair_with_jpeg` is. The three sentences the second needs are already written and drawn
   nowhere — "Show both", "Show the JPEG", "Show the raw" (`src/organize/pairs.rs:44-53`). About
   eighty lines and no new configuration field (§7.4, §8.5). Four of the six carry a verb the
   program already has and nothing else; these two carry a *setting*, and they are the only rows in
   the plan that write one without going through the registry, which does not exist until Stage 2.
   They call `save_settings()` directly, which is what the slideshow window already does
   (`src/app/settings.rs:70`), and Stage 5 re-points them at their registry rows so that a menu row
   and a settings-page row become one declaration rendered twice, which is the condition everything
   in Stage 5 rests on (§11.5). `raw.pair_with_jpeg` needs the folder re-opened rather than a
   restart (`src/app/mod.rs:307`, `src/app/chrome.rs:85`), so the row re-opens it.
3. **A Help menu, and an About window.** The menu bar is three menus and eleven items with no Help
   (`src/app/panels.rs:26-79`). Help gains **Keys… `?`**, **Keyboard…**, **What the marks mean**,
   **Template placeholders…**, then **Open the configuration file**, **Open the log file**, **Open
   the manual** and **About** (§10.4). About carries the version — `CARGO_PKG_VERSION` appears
   nowhere in `src/` — the wgpu adapter read at construction (`src/app/mod.rs:157-160`), whether
   this build has LibRaw, which is decided at `src/main.rs:38-41` and told only to the log, and
   both file paths with a copy button. Three of the most confusing behaviours in the program are
   diagnosable from that one window. `Config::path()` and `logging::path()` already return the
   right answer on every platform and are called by nothing the user can see
   (`src/config/load.rs:14-17`, `src/logging.rs:37-40`); the "show me the folder" shim is a
   platform `Command::new`, the same machinery user actions already use
   (`src/actions/user_action.rs:93`).
4. **Seventy tooltips, so that no panel that draws widgets is at zero.** The worked list is
   §10.7.1: sixteen files, five of which have none today — the filmstrip, the directory tree, the
   navigator, the frame-timings strip and the whole of `src/view/organize/controls.rs`, which is
   the sort and filter apparatus of three modes. Every line of the cache readout
   (`src/app/panels.rs:168-248`), the histogram (`src/ui/histogram.rs`), the status bar's flags,
   the three destructive folder-job buttons — each of which says **this cannot be undone**,
   because the journal does not cover them (`src/organize/journal.rs:28`) — and the four stack
   glyphs. Anything with a bound key names it, rendered through `keys::describe`
   (`src/ui/keys.rs:221-230`) so a rebind stays correct; Microsoft asks for exactly that
   (<https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tooltips-and-infotips>). Units,
   ranges and consequences go *under* the control and never in a tooltip (§10.7, §5.4). Fix the two
   hover strings carrying stray whitespace from broken continuations
   (`src/view/image_view/bottom_bar.rs:140`, `src/view/organize/group/mod.rs:253`) and the empty
   string handed to `on_hover_text` at `bottom_bar.rs:126-129`, which lays out and paints an empty
   frame. The count goes from 33 to 98 (§10.11).
5. **The four empty states, and the home screen.** "No images here" is four words on grey
   (`src/view/image_view/layout.rs:42`, `src/view/grid_view/mod.rs:265`) and is the first thing
   most people see, because with no argument the crawler reads the working directory, which the
   source itself calls "nobody's choice" (`src/crawler.rs:28-29`, `:46`). It becomes **Open a
   folder**, **Open files**, the last six folders the session file already keeps and has never
   shown (`src/session.rs:30`, `:56-65`), and one quiet line naming `?` and the right-click.
   "Nothing matches the filter" names the rules that emptied the folder and carries a **Show
   everything** button, which is `SuspendFilter`, already a command and already the bar's own
   wording (`src/ui/filter_bar.rs:280-291`). The tag panel returns before drawing anything when
   nothing is open (`src/app/tagging.rs:99-101`), so `K` on an empty folder changes no pixel; it
   draws greyed with one line. The metadata panel says "No photograph open" rather than
   "Loading…", which is a lie that never resolves (`src/app/panels.rs:147`) (§10.2, §7.3, §8.2).
6. **A first run that is not a tour.** The menu bar starts visible when there is no session file —
   which needs a check on `Session::path()` existing, because `Session::load` hands back a default
   for a missing file and an unreadable one alike (`src/session.rs:75-93`) — and thereafter
   remembers what it was left at. One status-bar line on the first session only, `Press ? for the
   keys · F1 for the menu`, dismissed by pressing either. And one notice saying the configuration
   file has been created, and where (`src/config/load.rs:61-92`) (§10.3).
7. **The notice band gets three severities, a history, and a way into it that is not the band.**
   Everything is the same alarm red today — `Color32::from_rgb(72, 32, 32)` (`src/ui/notice.rs:91`)
   for "Moved 12 photograph(s) to Selects" and for "Access is denied" alike — nothing in it can be
   clicked (`:80`), and there is no history: six seconds, 600 ms of fade, at most four lines, older
   ones dropped without a word (`:16`, `:19`, `:22`, `:56-58`). A neutral fill, an amber one and the
   existing red; a **Messages** window holding the last hundred, each row carrying whatever route it
   has (§8.6, Stage 6); and two ways in — a small count at the right-hand end of the status bar, and
   `Help ▸ Recent messages…` from the menu Stage 1 adds, so it is reachable in every mode and not
   only where a status bar is drawn. **The band itself stays `.interactable(false)`.** It is an
   `egui::Area` anchored `Align2::CENTER_TOP` at `egui::Order::Foreground` (`:78-81`), and during a
   cull something is in it after nearly every move, copy, delete and undo, so making it interactive
   would hand a strip across the top of the photograph to the band for 6.6 seconds at a time —
   including the photograph's own menu that item 1 has just added and the drag-to-pan Stage 0 item 8
   has just repaired. A band that takes the pointer is a worse defect than a band that cannot be
   clicked. Five silent failures reach the screen for the first time: a sidecar that cannot be
   *read* (`src/annotations/sidecar.rs:46-51`, which swallows both the I/O and the parse failure),
   a user action whose program is missing (`src/actions/user_action.rs:90-99`, where the source
   asks `//Show toast with result?`), a keyword list that will not load
   (`src/annotations/catalog.rs:54-62`), the section named in a partial-configuration warning
   (`src/config/load.rs:178-179`), and per-file folder-job failures. No message is a bare
   `format!("{e}")` — three sites are (`src/app/cull.rs:239`, `:442`, `:449`) (§10.9).
8. **A focused text field says so, and names the key that gets out.** `are_inputs_muted` is
   `explicit-mute || memory.focused().is_some()` (`src/utils.rs:41-46`), so the whole viewer goes
   deaf while any field holds focus — the filter bar's three (`src/ui/filter_bar.rs:229-247`), the
   tag panel's one (`src/ui/tag_panel/mod.rs:207-215`), the folder jobs' eight
   (`src/view/organize/controls.rs:61`–`:161`) — with `Escape` the only way out
   (`src/app/mod.rs:802-804`) and `Alt+Q` the only shortcut that survives
   (`src/app/input.rs:93-96`). Nothing on screen says any of it, so the symptom is a viewer that has
   stopped answering its keys. One line in the status bar while `memory.focused()` is `Some`:
   *"Typing — `Escape` to get the keys back."* §9.5 sets the problem out and calls the fix cheap; it
   is four lines beside the tooltips of item 4, and it is the least expensive repair in the stage
   (§9.5, §10.7).
9. **Undo says what it is about to do before it does it.** `Step::describe` exists and produces a
   sentence, and `App::undo` reads it *before* the undo runs (`src/organize/journal.rs:46-68`,
   `src/app/cull.rs:502`) — then performs the undo and reports afterwards. The sentence is built at
   the right moment and shown at the wrong one; a silent bulk undo is as frightening as none
   (§2.7).
10. **The cheat sheet becomes reachable and readable.** It is the best documentation in the program
    and it is behind one key nothing mentions (`src/app/input.rs:113-115`). Put it in Help; draw
    each binding's `description`, which already exists on every row and is read only by the keyboard
    editor (`src/config/bindings.rs:35-36`, `src/ui/cheat_sheet.rs:85-91`); add a search box — the
    image-view sheet is 20 + 22 + 15 = 57 rows and the gallery's 20 + 12 + 15 = 47 — and with it
    change the close rule, because *any* key press closes the window today (`:106-119`) and the
    first character typed would dismiss it. Escape, or a click outside, or any key while the box has
    no focus. Never push it (§10.5). The route *out* of the sheet — its footer becoming a button and
    every row opening the page that owns it — needs a settings window to lead to and is Stage 6's.
11. **What the marks mean, and the template window.** Two small windows off the Help menu. The
    first is the legend for a visual language that has none: `◐` HDR bracket, `◎` focus stack, `⏱`
    timelapse, `❏` series (`src/view/stacks.rs:362-369`), the badge vocabulary, the overlay colours
    (`src/decoder/overlays.rs:57-61`) and the focused pane's border. The second is the
    `PLACEHOLDERS` grid (`src/metadata/template.rs:321-339`), already drawn twice inside one mode
    (`src/view/organize/rename.rs:44-57`, `:141-155`) and reachable from none of the three
    configuration fields that use the same grammar. Say the clipping and peaking sentences of §10.8
    somewhere a person can read them: `Overlay::label()` returns "Off", "Clipping", "Focus peaking"
    and is called by a test alone (`src/decoder/overlays.rs:35-41`), and the marking mode reaches
    no status flag at all, so put its name in the flags row while it is on (§10.4, §10.8).
12. **The names, and the strip under a cell.** "Stack" is one word for three things (§11.6).
    `Place::describe` builds its line from `Kind::folder()` — a function whose documented job is to
    name a *folder*
    (`src/organize/group/classify.rs:42-51`) — so a focus stack reads *"stack 3 · frame 4 of 17 ·
    stack 3 of 41"*, and it prints `self.stack` twice where its own doc comment's example shows two
    different numbers (`src/view/stacks.rs:87`, `:90`, the test at `:527-530`). Use `Kind::label()`
    (`classify.rs:33-40`), spelled in full; make the filter bar's count read "41 stacks · 903
    frames" (`src/ui/filter_bar.rs:117`); and rename the two standing-frame bindings to "Which
    frame shows the stack" and "Show the next frame instead" (`src/config/bindings.rs:118-129`),
    because nobody will search for "standing". Three one-line changes and the feature becomes
    findable. One layout correction belongs with them, because §13 refuses the badge-size field on
    the strength of it: `CAPTION_HEIGHT` is a flat 20 points whatever the cell measures
    (`src/view/grid_view/cell.rs:15`, `:48`), so at sixteen columns the strip is proportionally
    enormous and at one column the stars are a sliver in a wall. It becomes a fraction of the cell
    against a floor; the font inside it already scales that way and only its cap has to move
    (`(strip.height() * 0.6).min(13.0)`, `:90`) (§6.6).
13. **The three files people are told to read.** `README.md:594` gives the configuration as
    `~/.config/avis-imgv/config.json`, which is right on Linux and wrong on Windows and macOS,
    where `ProjectDirs::from("com", "avis-imgv", "avis-imgv")` (`src/lib.rs:34-36`) puts it under
    `%APPDATA%\avis-imgv\avis-imgv\config\`. `examples/config.json`, which the README calls "fully
    populated", holds 103 of the 110 fields — no `version`, no `tags.catalog_file` and none of the
    six stacking shortcuts — so somebody who copies it wholesale gets `version: 0` and both
    migration steps re-applied (`src/config/migrate.rs:36-49`). And `fetch_cfg` writes
    `serde_json::to_string` on a fresh install (`src/config/load.rs:69`) while `save()` writes
    `to_string_pretty` (`:42`), so the file the README tells people to hand-edit is one long line
    until something rewrites it. Regenerate the example from `Config::default()` and fix all three
    (§3.4, §2.2).

*Finished when:* right-clicking a photograph on a fresh install offers something; all six words in
the status bar's flag row can be clicked and all say what they mean; `?` and `F10` are named
somewhere inside the program; the configuration file and the log file can be opened from a menu;
every panel that draws a number or a glyph explains it; clicking into the filter bar says on screen
that the keys have gone quiet and which key brings them back; a failed sidecar read reaches the
screen and is still there ten minutes later; and the four stack glyphs have a legend.

---

### Stage 2 — One table the file and the window both read

The foundation. Almost nothing on screen changes in this stage — the keyboard editor is the
exception, and it is the registry's first consumer and the proof that it works. Everything from
Stage 3 onwards is a view over one table rather than a second copy of the truth.

1. **Widen `src/config/bindings.rs` into a table over every field.** The pattern already holds:
   `binding!` builds a struct literal whose non-capturing closures coerce to the `fn` pointers
   `Field::Fixed` holds (`:20-24`, `:69-82`), so the widened table can be a `static` slice rather
   than the `Vec` that `bindings::all()` allocates on every frame the editor or the cheat sheet
   draws (`src/ui/keys.rs:61`, `src/ui/cheat_sheet.rs:48`). Each row carries page, group, label,
   sentence, aliases, JSON path, `Kind`, `Effect`, `Scope` and the accessor pair the file already
   uses. It is not a new idea in this codebase; it is the existing idea applied to the other fifty
   fields, and the module's own comment already states it (§3.7, §4.3).
2. **`bindings::all()` becomes a filtered view,** so `src/ui/keys.rs` and `src/ui/cheat_sheet.rs`
   compile and draw exactly what they draw now: 69 rows over 60 fields — 58 from `binding!`, plus
   the five colour labels of `Label::CHOICES` and the six ratings of `0..=MAX_RATING` pushed at
   `src/config/bindings.rs:420-436` (§4.7).
3. **`Kind`,** the widget vocabulary the rows are drawn from: bool, integer, float, enum, string,
   path, template, colour, list-of-record, shortcut, gesture, and `Run` for rows that are buttons
   rather than values — the configuration file's path, the log file's path, restart. `Run` rows are
   what make "config file" a searchable query (§3.4, §3.7).
4. **`Effect`** — `Live`, `Rebuild`, `Restart` — recorded per row now and honoured in Stage 8.
   Twenty-six fields are restart-bound today and nothing anywhere says so (§3.5).
5. **`Scope` replaces the section heading for clash detection.** `clash()` compares only within an
   editor section on the stated ground that the gallery and the image view are never on screen at
   once (`src/ui/keys.rs:169-186`) — but "General" is live in every mode, because `input::collect`
   runs unconditionally every frame (`src/app/mod.rs:807`), so a General binding colliding with an
   image-view one is the collision that actually bites, and the test at `src/ui/keys.rs:333-348`
   asserts that silence is correct. It is not: Quit on the Gallery's scroll key means the folder
   scrolls and the program exits. A scope states where a binding is *read*; a heading only happens
   to (§9.5, §9.8).
6. **Aliases, the doc comments, and one equivalence table applied to the query.** The doc comments
   of `src/config/mod.rs` are indexed verbatim — they are unusually well written, "the bin does not
   reach a memory card or a network share" (`:184-186`), "a DNG written by Camera Raw carries a 256
   pixel preview and nothing else" (`:375-379`) — and are read by nobody. The authored aliases
   carry other programs' vocabulary and the complaint rather than the noun: "standard preview" is
   Lightroom's name for `thumbnail_resolution`, "color class" is Photo Mechanic's for a colour
   label — in their spelling, which is the point of an alias, since the equivalence table below
   already maps `colour color` — "resources" is darktable's for the memory budget, "blurry
   thumbnails" is the literal
   wording of the XnView report (<https://newsgroup.xnview.com/viewtopic.php?t=47571>). The
   equivalence table is about twenty lines — `colour color`, `grey gray`, `memory ram`,
   `gpu graphics vram card adapter`, `raw cr2 cr3 nef arw dng` — plus a stop-word list, so that
   "where do rejects go" becomes "rejects" and lands on `cull.rejected_folder` (§3.4).
7. **Fifty-seven fields have no doc comment; write them, and move the two that drifted.** The
   comment is about to become the help text and a search term, so its absence stops being free
   (§11.1). "Half a frame at sixty a second" sits above `default_gpu_budget_mb` and belongs above
   `default_upload_budget_ms` (`src/config/defaults.rs:35-43`); a sentence about `PageDown` sits
   above `default_filmstrip_height` (`:293-299`). And the doc comment on
   `tags.advance_after_marking` describes behaviour the code does not have — "holding shift with
   any of those keys advances once whatever this says" (`src/config/mod.rs:166-167`) — while
   `advances()` reads the mode flag alone and the comment beside it explains why a modifier was
   rejected (`src/app/input.rs:192-200`). One of the two has to go before either is put on screen
   (§3.1, §5.6).
8. **The fixed keys become read-only rows.** `Home`, `End`, `PageUp`, `PageDown`, `Tab`, `/` and
   `Escape` (`src/view/image_view/input.rs:108-118`), `F10` and `?` (`src/app/input.rs:102-115`),
   the contact sheet's arrows (`src/view/grid_view/mod.rs:562-605`), the tree's six
   (`src/ui/tree.rs:277-307`) and the destination digits and `Enter`
   (`src/ui/destinations.rs:113-140`). They are entered in the registry rather than left out, so
   the clash checker can see them and a search for "cheat sheet" finds one. `Field` currently
   offers only writable variants (`src/config/bindings.rs:20-24`) and needs a read-only one. In the
   same stage, `image_view.user_actions[].shortcut` gains rows — it is the one shortcut in the file
   the editor cannot reach (`src/config/mod.rs:542`), which is why the shipped example's trash
   action can never fire: it is bound to plain `delete` alongside `sc_delete`, and `input::collect`
   consumes the event first (`examples/config.json:34-41`, `src/config/shortcut.rs:80-93`) (§9.5,
   §10.5).
9. **`Config::check()`,** run once at load, returning `Vec<(field, complaint, what was used
   instead)>` and drawing nothing yet. It covers the ways a value can be wrong that currently reach
   only a log file whose location the program never states: the three-substring ICC match
   (`src/metadata/icc.rs:11-15`, `src/decoder/color.rs:30-32`); the silently skipped metadata tag
   (`src/app/panels.rs:151-154`); the unreadable keyword list (`src/annotations/catalog.rs:53-62`);
   the blank `cull.rejected_folder` that makes its key a no-op (`src/app/cull.rs:323-326`); an
   empty destination path, silently dropped (`:386`); the tenth destination, silently truncated
   (`src/ui/destinations.rs:74`); the unrecognised `Callback` string that deserialises to
   `NoAction`, so `"Relaod"` is accepted (`src/actions/callback.rs:39-46`); the unknown modifier,
   logged and dropped (`src/config/shortcut.rs:164-175`); and the unknown key name that becomes the
   sentinel `Ctrl+Alt+Shift+Cmd+F20` (`:138-151`) — which is to say a typo in a key name makes a
   command permanently unreachable and the only record is a log line (§3.6).
10. **The keyboard editor's ten faults, now that the table can answer them.** A per-row reset and a
    per-section reset, with a confirmation on the global one, which today walks all 69 rows on one
    click with no confirmation of any kind (`src/ui/keys.rs:111-121`); `Delete` or `Backspace` on
    an armed row means "no key", a state `describe` can already render and nothing can produce
    (`:140-144`); a filter box over name, description **and** key; a "show only this mode" toggle;
    `State.status` written, since it is declared and read and assigned nowhere (`:30`, `:123-125`),
    so a successful save says nothing; mute while a row is armed, because arming a row and pressing
    `Delete` currently sends the photograph on screen to the bin and the capture silently fails
    (`src/app/mod.rs:807`, `:840`); `cmd` emitted on macOS (`src/ui/keys.rs:266-278`); and key names
    canonicalised on write, accepting both spellings on read, with `examples/keys.txt` regenerated
    from `Key::ALL` — `pageup` and `pagedown` are the two of its eighty names that
    `capitalize_first_char` turns into something `Key::from_name` rejects
    (`src/utils.rs:78-84`) (§9.5).
11. **The two tests, without which the plan is a promise.** `every_field_is_in_the_index` walks
    `serde_json::to_value(Config::default())` and fails the build if the file carries a key the
    registry has not heard of, or the registry a row the file does not; it is the generalisation of
    the count assertion already at `src/config/bindings.rs:530-541`, whose own comment says why.
    `the_index_answers_these_questions` is about sixty `(query, expected field)` rows, every phrase
    taken from a real complaint — "blurry thumbnails", "where do rejects go", "why is my raw
    small", "color class", "text too small", "cr3". It cannot catch an omission, only a
    regression, and it is the only mechanism anybody proposed that keeps a synonym list alive
    (§3.7).

*Finished when:* adding a field to `Config` without adding a row fails `cargo test`; the keyboard
editor and the cheat sheet draw what they drew before, plus the fixed keys and the user actions;
the editor can reset one row, clear one row, be searched, confirm that it saved, and capture
`Delete` without deleting the photograph; a General binding colliding with an image-view one is
reported; and `Config::check()` on a file holding `"output_icc_profile": "sRGV"` names the field,
the complaint and the substitute.

---

### Stage 3 — Every setting has a control

The window. It is `App::show_keyboard` with a longer body: that function already draws a window
with `&mut self.settings` in hand, runs a four-line fan-out when something changed, and calls
`save_settings()` (`src/app/settings.rs:80-95`); both `set_config` implementations are one and two
statements (`src/view/grid_view/mod.rs:195-197`, `src/view/image_view/navigate.rs:95-98`). No new
file format, no second viewport, no shadow state file. It comes after the registry because every
page is a filter over it, and before the routes because the routes need pages to route to.

1. **The shell.** An `egui::Window` modelled on the keyboard editor (`src/ui/keys.rs:49-55`),
   900 × 600 comfortable and 720 × 480 floor, which fits the 1092 × 614 logical space of a
   1366 × 768 laptop at 125 % — darktable's preferences window put its Close button below the
   bottom of a 14-inch screen where it could not be dismissed
   (<https://github.com/darktable-org/darktable/issues/3858>). Vertical navigation, one level, no
   nesting; ordered by how often a thing is wanted; and nothing called General, Advanced,
   Miscellaneous or Other (§3.2).
2. **The eleven pages, in this order:** Opening a folder · The photograph · The contact sheet ·
   Stars, flags and labels · Keywords · Moving and deleting · Raw files · Slideshow · Speed and
   memory · The window · Keys and mouse. Which field lands on which is §4.5, row by row. Seven
   fields are drawn on two pages as mirrors — one registry row, one accessor pair, two renderings,
   each naming its home — and `raw.pair_with_jpeg` gets a clickable cross-link on Raw files rather
   than a mirror, because one of the two pages is simply where people look first (§4.5, §4.6).
3. **The widget vocabulary, written once and used by every page.** A rail with its value box for a
   continuous quantity — which is one call in egui, since `Slider`'s value display *is* a
   `DragValue` (`egui-0.33.0/src/widgets/slider.rs:918-943`) and both of the program's existing
   sliders switch it off; a `DragValue` for a whole count sitting against a default nobody moves; a
   radio group with a sentence under each variant for fewer than five choices, on the model of
   `Motion` (`src/app/panels.rs:110-117`), which is the best control in the program; a tick above a
   number for anything meaning "automatic", so zero never means two things. egui has no tick marks
   — `grep -i tick` over its slider returns nothing — so a thin strip painted from the response's
   own rect stands in, and where there are six or fewer significant values they are small text
   buttons under the rail that *set* it (§5.2, §5.3). §5.5's table is eighteen rails and six
   steppers; item 7 takes two of each off it, and five of the remaining rails are held back until
   Stage 8 — the two budgets, the camera-thumbnail count, the decode ceiling and the thumbnail
   resolution, which are read once when the stores are built (`src/app/stores.rs:32-66`) and so
   would move under the hand and change nothing. They are drawn as the typed box §5.9 prescribes,
   with the restart badge, and become rails when the rebuild lands. `cache.decode_threads` is the
   one store-bound number that keeps its rail from the start, because it can never be made live and
   the number is still worth reading and copying to another machine (§5.9). Eleven rails and four
   steppers, then, and sixteen rails after Stage 8. The two store-bound *steppers* need no such
   treatment: a stepper is already the typed number §5.9 asks for.
4. **Units on screen, never in a tooltip, and four values previewed before they are committed.**
   Text size reads 50–300 % rather than a multiplier, with sample text beside it at the chosen size;
   `cell_aspect` reads `1 : 1.50`; the frame width reads per cent of the shorter edge. Four fields
   have their `1.25`/`1.5`/`0.2`/`25.0` notations reconciled (§11.5). The three template fields get
   the `PLACEHOLDERS` chips and a live rendering from the photograph currently open, which is the
   only way to tell a correct template from a typo — an unknown placeholder expands to nothing
   today (`src/metadata/template.rs:287`) — and `thumbnail_resolution` prints the cell width it is
   being judged against (§5.4).
5. **The thirty-five fields that do not exist yet, drawn with the pages that carry them** (§6.2).
   Seven browsing defaults and `browsing.filter_follows_folder` in a new `browsing` section; three
   grouping thresholds in a new `group` section, which also settles the disagreement where the
   filter bar allows a gap of 1–600 s and a tolerance of 0–32 and the group panel 1–3600 s and 0–64
   (`src/ui/filter_bar.rs:134`, `:147`, `src/view/organize/group/mod.rs:60`, `:65`);
   `general.start_in`, `start_fullscreen`, `start_folder`, `panels_at_start` and `side_panel_width`
   beside `restore_session`, with the precedence stated on the row — a path on the command line,
   then the restored session, then the startup folder, then the working directory, which is the
   order `App::new` already implements (`src/app/mod.rs:255-285`); `image_view.zoom_step`,
   `zoom_step_factor`, `zoom_step_max`, `pan_speed` and `page`, five compile-time constants
   governing every movement key (`src/view/image_view/input.rs:17`, `:21`, `zoom.rs:11`, `:45`,
   `mod.rs:49`); `grid_view.badges` and `grid_view.filmstrip_edge`; `tags.sidecar_naming`;
   `cull.confirm`; `general.theme` and `general.backdrop`. That is twenty-seven; the mouse's eight
   arrive in Stage 7 and make thirty-five. **Three new sections and no others** — `browsing`,
   `group` and `mouse`. Everything else goes beside the fields it is about: the sidecar naming into
   `tags`, the confirmations into `cull`, the theme and the backdrop into `general`, which is where
   §6.2 puts them and what the pages assume. There is no `startup` section (§6.11 argues the three
   startup fields belong beside `general.restore_session`, which is the same subject), no `marks`
   section and no `appearance` section. A section the document does not carry costs nothing
   (`src/config/load.rs:170-172`, with a test at `:199-208`), so nothing already on disk is read
   differently either way, and the reason to decide it once is that a field with two candidate paths
   is a field whose search token, whose migration and whose forum answer are all provisional (§6.2,
   §11.5).

   One more key arrives with them and is not one of the thirty-five:
   **`grid_view.filmstrip_visible`**. It is not a new choice but §3.5's split of
   `grid_view.filmstrip_height`, which stores a height and a visibility in one number
   (`src/app/mod.rs:175`, `src/app/views.rs:137-141`). Without the split the height rail cannot have
   a floor, because "off" is the value zero (`src/config/defaults.rs:299-301`), and "keep the height,
   hide the strip" cannot be said at all (§3.5, §5.5).
6. **The light palette is the work; the theme switch is an afternoon.** `apply_theme` calls
   `ctx.set_theme(ThemePreference::Dark)` unconditionally (`src/ui/theme.rs:23`) above a doc
   comment giving a reason that is sound and too wide: what surrounds the photograph is the
   backdrop, which is a separate hardcoded value (`src/view/image_view/layout.rs:12`) the theme
   does not touch. It must be **Live**, and it can be — the function is called from one place and
   nothing in `src/` holds a `Visuals` of its own — because a theme setting that lies is worse than
   none, which is nomacs #1062 (<https://github.com/nomacs/nomacs/issues/1062>). One backdrop field
   is read by all three greys, with the filmstrip deriving its two from it
   (`src/view/grid_view/filmstrip.rs:31-32`) and the slideshow override staying the per-mode
   exception it already is (§6.3).
7. **Take six controls off the pages before they are built** (§11.2). `image_view.nr_loaded_images`
   is trimmed to what the budget holds (`src/cache/policy.rs:136-151`), so the number in the file
   has never been the number in force — at a 4096 MB default it is about 22 on a 4K monitor and
   about 90 on a 1080p one; it becomes a readout beside the budget. `cache.upload_budget_ms` is
   half a frame at sixty a second and the frame time is measured every frame
   (`src/view/image_view/interaction.rs:58`), so the default is computed rather than asked for —
   but the field stays in the file for whoever is debugging a stutter, a written value still wins,
   and it is therefore still one of the `cache.*` fields Stage 8 rebuilds (§11.2, §3.5).
   `image_view.gpu_resident_images` and `grid_view.gpu_resident_thumbnails` are counts where the
   bound is bytes, which the doc comment on `cache.gpu_budget_mb` argues against its own neighbours
   (`src/config/mod.rs:247-254`). The two `context_menu` lists are one idea written twice and get
   one table with a column saying where each entry appears. `image_view.scroll_navigation` is a
   boolean that cannot answer "what", and retires into `mouse.wheel` in Stage 7. Six fields lose a
   control of their own; five of them come off §3.2's count of 144, `scroll_navigation` having
   already retired into the mouse section. **139 rows are drawn over 145 keys in the file.** Four of
   the six end with no control anywhere: the two GPU counts, the loaded-image radius and the upload
   budget. Each keeps a line on the page that used to hold it, saying where its value now comes
   from — the budget that trims it, the bytes that bound it, the frame time that computes it — so
   that the answer to "where did that setting go" is on the page rather than in this document.
8. **The commit model, one across all rows.** No OK, Cancel or Apply — which is what the program
   already does (`src/app/settings.rs:104-110`) and what Microsoft carves out for a property
   inspector. The one qualification is arithmetic: `stores::image_store` and
   `stores::thumbnail_store` are pure functions of the configuration and `ImageStore::new` takes a
   `StoreConfig` by value (`src/app/stores.rs:32-66`, `src/cache/store/mod.rs:117-122`), so a
   `ram_budget_mb` rail on true per-frame apply would rebuild the cache sixty times a second. A
   `Rebuild` row commits when the gesture ends — `drag_stopped()` on a rail, focus loss on a box,
   the click itself on a radio. One gesture, one rebuild, and the cost stated permanently under the
   group in real units, computed from the folder in hand and this machine's own throughput; the
   README's published figure is 43.6 images a second on 120 24-megapixel JPEGs on a 24-core Ryzen
   (`README.md:291`) (§3.2).
9. **The search box** at the top of the navigation list, holding the cursor when the window opens,
   filtering the same rows drawn by the same function so the value changes in the result. An exact
   JSON-path match wins outright, so `cache.ram_budget_mb` pasted from a forum post lands on the
   control. Nothing reorders by use. Never an empty result: a failed AND is re-run as an OR under
   "Nothing matched all of that. The closest:", with the path to `config.json` beneath it. This is
   the best-evidenced request in the whole research — five darktable issues by five people, every
   one closed by a stale bot (<https://github.com/darktable-org/darktable/issues/3423>, #6706,
   #6174, #3604, #9598) (§3.4).
10. **Three ways in, and one that stays what it is.** `Ctrl+,` opens the window on the page last
    used; `Settings ▸ All settings…` is the third entry on a menu that has had two since it was
    written (`src/app/panels.rs:67-77`); and `Keyboard…` and `Slideshow…` are kept as deep links to
    two pages, because they are the only settings routes anybody has learned. Plain `Comma` is
    `sc_standing_back` (`src/config/defaults.rs:192-194`), so the modified key is free, and like
    every other key it is a registry row rather than a hardcoded one. `Ctrl+L` is not merged with
    the search: the navigator's Enter means "open this folder" and a settings search's would mean
    "change this value", and the two overlays name each other in their footers instead
    (`src/ui/navigator.rs:83-88`) (§3.3).
11. **The page persists into `config.json` through the registry, not into the session file.**
    `on_exit` returns early when `restore_session` is off (`src/app/mod.rs:866-872`), so a
    preference kept there is a preference some users silently do not have. It has to be done by
    hand either way: eframe is built without the `persistence` feature (`Cargo.toml:13`) and `App`
    implements no `save()` (§3.2, §4.8). The key is `general.last_settings_page`, and it is entered
    in the registry as a read-only row — like `version` (`src/config/mod.rs:25`), a key the window
    writes and nobody sets. It needs the row whether or not it needs a control, because
    `every_field_is_in_the_index` fails the build for a key the registry has not heard of (Stage 2,
    item 11).
12. **Clamp at the consumer as each control lands, and never in `Config`.** The ranges are §3.6's
    and the control's range and the consumer's clamp are written from the same registry row so they
    cannot drift. `frame_size_relative_to_image` at `5.0` draws a frame five times the shorter edge
    and leaves the photograph one pixel wide (`src/view/image_view/canvas.rs:356-370`);
    `filmstrip_height` at `5000` claims the whole window (`src/app/views.rs:137-146`);
    `tags.panel_width` accepts `-40` (`src/config/mod.rs:144`); `raw.highlight_mode` is handed to
    LibRaw as an `i32` with no validation (`src/decoder/raw/mod.rs:106`) and becomes four named
    choices with a passes box in the fourth (§5.7). Three fields are already defended correctly and
    are the pattern to copy: `cell_aspect`, `overlay_text_size` and `nr_images_shown` (§3.6).
13. **The list and template editors, and the group panel's one preset dropdown.**
    `cull.destinations` gets a table with label, path, folder picker, drag handle and the digit
    that reaches it, drawing the tenth greyed rather than dropping it; `tags.categories` a tree
    editor; `general.metadata_tags` two columns, every tag found on the open photograph on the left;
    `catalog_file` a path picker with **Read it now** reporting the count. And the four fixed
    thumbnail heights in the group panel (`src/view/organize/group/mod.rs:77-82`) become a rail
    from 0 to 240 pt with those four as named stops — the one control in the program the brief's
    rule condemns outright (§5.6, §5.7).

*Finished when:* every setting that keeps a control has one, and the four that lose theirs say on
the page where their value now comes from; typing
"shape of the cells" finds `grid_view.cell_aspect` and changing it there redraws the contact sheet;
the raw source can be switched from the embedded preview to a full develop without opening a text
editor; the grey behind the photograph can be changed and the interface can be made light; and the
window reopens on the page it was left on, whether or not `restore_session` is on.

---

### Stage 4 — The window says what is wrong, and what has been changed

This is what makes the window the place a value is *explained* rather than only set. It is
separated from Stage 3 because none of it is needed to make a control work and all of it is needed
before anybody trusts one.

1. **The problems strip, from `Config::check()`.** A band across the top of the window that does
   not fade, one row per complaint, each with a **[Fix]** button that opens the owning page and
   focuses the control. The route from the complaint to the control is the whole value; today every
   one of those failures ends in a log file whose path is written only into that same log
   (`src/main.rs:34-36`) (§3.6).
2. **A permanent red bar when `partial` is set, naming the section.** A section that fails to parse
   blocks all saving for the session (`src/config/load.rs:24-32`), the user is told once for 6.6
   seconds (`src/ui/notice.rs:16-19`), and the section's name reaches only the log (`:178`). The bar
   carries **[Show me the file]** and **[Use the defaults for that section and start saving
   again]**, and every control below it is drawn *disabled rather than hidden* — Microsoft's rule
   for an inapplicable page, whose reason is that greying the whole thing out "would force users to
   look on all other tabs" (§3.4).
3. **The migration report and the startup clash notices get a permanent home in the footer**
   instead of a six-second band (`Config::migrated`, `src/config/mod.rs:43-48`, said at
   `src/app/mod.rs:240-246`). A warning that two commands are on one key is gone 6.6 seconds after
   launch today and cannot be recovered (§1.4, §3.4).
4. **A changed-from-default bullet in the left gutter, for all 139 rows or not shipped at all.**
   darktable's own bug is the warning: only *generated* preferences show its marker, so it cannot
   be trusted (<https://github.com/darktable-org/darktable/issues/19765>). It is computed against
   `Config::default()`, which `src/ui/keys.rs:112` already builds for its own reset. The bullet is a
   button: clicking it restores the default and its own right-click names the value it would
   restore — *Put this back to 4096 MB* — because a reset that does not say what it is resetting to
   is a leap. A count per page in the navigation list answers "I changed something and I do not
   remember where" without opening eleven pages (§3.4).
5. **Reset always carries a scope:** this setting · this page · everything. The page and everything
   scopes show a from→to preview before writing anything, naming each field by its sentence rather
   than its path. **everything** writes `config.json.bak` first, which is what two XnView users
   asked for when the same feature was requested there — the author's reply was "It's the same as
   deleting xnview.ini, is it really needed?"
   (<https://newsgroup.xnview.com/viewtopic.php?t=4350>). And the existing button is renamed: "Put
   everything back to the defaults" (`src/ui/keys.rs:111-121`) resets 69 key rows with no
   confirmation, so it becomes "Put the 69 key bindings back", it confirms, and it names the count
   (§3.4).
6. **An out-of-range value is shown, marked out of range, and left exactly as written.**
   `Config::save` writes the whole document, so a value outside a control's range has to be kept in
   the struct rather than clamped on load — otherwise the first save after any unrelated change
   destroys somebody's deliberate 8,192 MB budget. `Config::from_json` does no range validation and
   that is the correct state for it (`src/config/load.rs:125-162`). Hand-editing always wins,
   including hand-editing to a value the window cannot produce (§3.6).
7. **The footer, below a separator so it does not read as a twelfth page.** The configuration
   file's path, selectable, with *Open it* and *Show me the folder*; the log file the same;
   `version` read-only, the one key a person should never change (`src/config/mod.rs:25`); **What I
   have changed**, listing every non-default field with its page; and **Restart now**, which calls
   `Session::save` itself rather than relying on the exit path and relaunches. All of them are `Run`
   rows in the registry, so they are searchable like everything else (§3.4).
8. **Export and import as a patch, never a snapshot.** "Save what I have changed…" writes only the
   fields differing from their defaults — a small, readable, diffable file that goes into version
   control and onto the other two machines — so a bundle written by an older build stays valid,
   because the fields it does not know about are the fields it does not name. Key bindings are
   opted into and never included by default: a shared file that silently rebinds `x` is the Adobe
   complaint in miniature. Machine-specific paths — `cull.destinations`, `tags.catalog_file`, the
   `exec` strings — are listed separately and unticked, which is digiKam bug 267131
   (<https://bugs.kde.org/show_bug.cgi?id=267131>). Loading one shows the same from→to preview with
   a tick per row. Bundles live in `<config dir>/bundles/*.json`; export is copying the file (§3.4).
9. **Two failures answered where they happen.** A **Test** button on the three action editors that
   runs the command against the open photograph and reports what came back, answering the question
   the source itself asks — `//Show toast with result?` (`src/actions/user_action.rs:88-99`) — and
   **Read it now** beside `tags.catalog_file`, reporting "2,114 keywords in 31 groups" or "No file
   there", where today a failure is a `tracing::warn!` and the panel simply shows fewer keywords
   (`src/annotations/catalog.rs:53-62`). The row also states that a relative path is taken against
   the configuration directory and not the working one (`:180-194`) (§3.6).

*Finished when:* a configuration with a misspelled ICC profile opens the viewer with a complaint
that names the field and a button that goes to it; a file with one broken section says which
section, permanently, and offers a way out; every changed field can be found and put back one at a
time; and the settings of one machine can be carried to another as a file naming four values rather
than a hundred and forty.

---

### Stage 5 — Right-click the thing itself

The brief asks that a setting be reachable from the thing it is about. This stage is that, and it
is safe only because every route writes the same registry row through the same setter, so a menu
row and a settings page row are one declaration rendered twice and neither can drift. It comes
after Stage 3 because every menu's last row names a page, and after Stage 0 because two gestures
had to be repaired before any of it could be drawn. The two settings rows Stage 1 put on the bottom
bar are re-pointed here as well, so that when this stage ends nothing in the program writes a
setting except through one row (§11.5).

Nothing here is reachable *only* by right-click. Microsoft: "Don't make commands only available
through context menus … context menus are alternative means"
(<https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-menus>); Apple: "Always ensure that
contextual menu items are also available as menu commands."

1. **The surfaces become widgets.** Seven `ui.label` call sites in the bottom bar
   (`src/view/image_view/bottom_bar.rs:119`, `:138`, `:153`, `:161`, `:193`, `:198`, `:203`), the
   metadata rows and the cache lines (`src/app/panels.rs:156-195`), the histogram plot and its two
   figures (`src/ui/histogram.rs:43`, `:108-134`) each take `.sense(Sense::click())`; the overlay
   (`src/view/image_view/overlay.rs:93-145`), the stack badge
   (`src/view/grid_view/cell.rs:190-228`) and the caption strip under a cell
   (`src/view/grid_view/mod.rs:367-370`) are painter-only and take a rect through `Ui::interact`,
   which takes `&self` in egui 0.33 and so changes no signatures; the cell rect is allocated in
   both branches, so a cell that failed to decode stops being dead to the mouse and to the hover
   (`:381-394`); and the selection counter loses its `.interactable(false)` (`:322`). egui names
   this as the commonest cause of a dead right-click: "Make sure the widget senses clicks (e.g.
   `Button` does, `Label` does not)" (`egui-0.33.0/src/response.rs:925`) (§7.8). **And every one of
   these menus opens on the press, not on the release** — Microsoft's toolbar guidance is to show a
   context menu "on right-click on mouse down, not mouse up", which removes the ambiguity with a
   drag at its source instead of tuning a threshold (§9.4). That is not what
   `Response::context_menu` does: `Popup::context_menu` keys off `response.secondary_clicked()`
   (`egui-0.33.0/src/containers/popup.rs:248-259`), which is reported on the release. So the shared
   helper opens `Popup::menu(&response)` from `pointer.button_pressed(PointerButton::Secondary)`
   while the response is hovered, and every surface goes through that helper. One function, and the
   only place in the plan that `PointerButton` is read besides Stage 0 item 8.
2. **One shape for every menu**, obeyed by all of them: the verbs on this object, most used first
   and at most seven; copy and show, where the object is a file on disk; the two or three settings
   that are about this object; and **More settings… (<page name>)**, always last, never varying,
   never removed. Twelve rows including the last, which is GNOME's and NN/g's ceiling. One level,
   with one exception: egui #5251 places a submenu with its right edge against the screen
   edge where long text covers its parent (<https://github.com/emilk/egui/issues/5251>), and every
   panel in this program is against an edge — but 0.33 folds a submenu that will not fit to the far
   side of its parent (§7.12), which is what the five turns behind one word are worth. A choice of
   five that is five different decisions is still an inline radio row, which counts
   as one item because the ceiling is about decisions and not about pixels. The count goes in the
   label — "Move 24 photographs to the bin" — and slot 1 closes the menu while slots 3 and 4 do not
   (§7.2).
3. **About twenty surfaces get one** (§7.3–§7.7): the photograph, the overlay, the empty
   background in both its states, every element of the bottom bar including the marks strip and the
   mode indicator, a cell, a selection, the stack badge, the filmstrip and a thumbnail in it, the
   filter bar and each of its chips, the sort control, the side panel and one metadata row, the
   histogram, the cache readout, the frame-timings strip, the keyword panel, a tag chip, a category
   heading, a destination slot, the menu bar, the directory tree and a folder in it. Each names the
   page it leads to (§3.3's table) and each carries the verbs that already exist a few files away.
4. **A mode indicator, because there is none.** `Mode::label()` is drawn in three places, none of
   them where people spend their time (`src/app/mode.rs:37-46`, `src/app/panels.rs:56-65`,
   `src/ui/cheat_sheet.rs:70`, `src/view/organize/mod.rs:149`), while `F2` cycles all six and three
   of the six draw no photographs. One word at the left end of the bottom bar, carrying the six
   inline radios and a tick that writes `general.start_in` (§7.7).
5. **Four places the control belongs in the view rather than the window.** A size rail in the
   filter bar for `grid_view.images_per_row`, which is where Lightroom keeps it — on the Grid
   toolbar, not in preferences; a drag handle on the filmstrip's top edge for `filmstrip_height`
   (`src/view/grid_view/filmstrip.rs:63-66`); the tag panel's splitter for `tags.panel_width`; and
   the side panel's edge for the new `general.side_panel_width`, which is a hardcoded
   `default_width(340.)` today (`src/app/chrome.rs:106-110`). All four write the registry field
   through the same setter, so `Config::save` runs and the value survives the session — which is
   the thing none of the program's existing in-view controls does (§5.8, §7.6).
6. **The reverse trip, so somebody who opened the window out of habit learns the shorter route.**
   Every row carries a grey suffix naming its object — "· right-click the cache readout" — and a
   small ⌖ that opens the panel, scrolls to the block and flashes it. Every row's own right-click
   offers **Show me where this is**, which closes the window, switches to the mode where the field
   has a visible effect and flashes the thing it controls, and **Copy setting name**, yielding
   `cache.ram_budget_mb`. The traffic runs the other way too: any menu whose object has a registry
   row carries **Bind a key…** beside that object's settings rows, which makes the control the route
   to its own key and closes the loop the keyboard editor otherwise owns alone. It is the direct
   answer to the ImageGlass request to "change keybinds by ctrl+right clicking on the menu and
   picking them on the GUI instead of having to edit configs", which the maintainer granted in
   principle — "Yes, I will add UI for hotkey setting"
   (<https://github.com/d2phap/ImageGlass/discussions/1702>) — and it is one row calling the capture
   the editor already runs (§9.4). The registry is keyed on the path and never on the label:
   nomacs stored shortcuts under their *translated* names and broke every one when the interface
   language changed (<https://github.com/nomacs/nomacs/issues/1539>) (§3.4).
7. **A keyboard route: Shift+F10, and no other.** egui has no `Key::ContextMenu` — the enum runs
   `F1` to `F35` and grepping it for `Menu` returns nothing (`egui-0.33.0/src/data/key.rs`) — so the
   dedicated Menu key cannot be read at all. `App` records which surface last held the keyboard
   cursor, the key sets `open_context_for(surface)`, and each surface opens `Popup::menu(&response)`
   anchored to its own rect rather than at the pointer. About forty lines. It collides with F10 and
   `?` today, both of which are read with `key_pressed`, which ignores modifiers entirely
   (`src/app/input.rs:102-115`), so the context-menu binding consumes `SHIFT + F10` first — egui's
   own instruction is to "match most specific shortcuts first". It must not be blocked by
   `are_inputs_muted`, which treats any focused widget as mute (`src/utils.rs:41-47`), and `Escape`
   closes the menu before it does anything else, because `Escape` already means six other things
   (§7.10).
8. **Uniform or none.** Every surface carrying a menu gets the same 6-pt chevron on hover in the
   weak text colour (§7.9), and the last four words of its hover text are always *"Right-click for
   more."* — §10.7's rule 7, and §10 owns the wording, which is why it is *more* and not *set it*:
   most of these menus carry verbs as well as settings, and §10.7.1's worked row for the zoom
   percentage already reads "Magnification. Right-click for more." All of them, or none — the same
   finding as Stage 1's, applied to a second affordance (§7.9, §10.7).
9. **The destination panel becomes a place destinations can be added.** `take(9)`
   (`src/ui/destinations.rs:74`) silently drops a tenth, and "Choose a folder…" picks an ad-hoc
   folder used once and thrown away (`src/app/cull.rs:354-371`), so the panel that exists to hold
   destinations is the one place you cannot add one. Every configured destination is listed, the
   list scrolls, the first nine keep their digits and the rest are reached with the arrow keys, and
   the ad-hoc row's menu carries **Remember this folder as slot 3**. The digit cap stays: there are
   nine digits and the digit is the gesture (§6.10).
10. **One boolean, and one check row.** `menus.settings_rows`, default on, on Keys and mouse:
    turning it off leaves the verbs, the user's entries, the copy group and the last row, so nothing
    becomes unreachable. That is the whole of the configurability offered for the built-in rows
    (§7.11), and §13's refusal of a menu editor rests on it. It is one of the three keys in the plan
    that are not among §6's thirty-five, and the only one this stage adds; it earns its place under
    §11.2's rule on the first limb, because a person who wants a four-row menu and a person who
    wants a nine-row one are both being reasonable. And the shipped example's entry described
    `"Delete"`, whose command moves the file to `$HOME/trash` (`examples/config.json:49-53`), now
    sits two rows below a built-in row called
    "Move 1 photograph to the bin"; it becomes one row in the Stage 2 check, naming
    `image_view.context_menu[1]`, offering to rename it, remove it or leave it, and doing nothing
    until it is asked.

*Finished when:* every page can be reached by at least two routes that do not go through the
Settings menu; dragging the filmstrip's edge changes a value that is still there on the next launch;
every menu appears on the press of the button rather than on its release; a key can be bound from
the thing it acts on; Shift+F10 opens the menu for whatever last had the keyboard; the bottom bar
says which mode is on screen; and no surface in the program carries a menu without saying that it
does.

---

### Stage 6 — Getting from one thing to the thing next to it

The viewer is full of true statements that cannot be acted on — `7/312 (+18)`, `Blown 3.4%`,
`series 2 · frame 4 of 17`, `3 selected`, `n image(s) could not be opened`. Sixteen of them are
listed in §8.8. This stage is four mechanisms rather than sixteen repairs, and it comes here
because three of the four are carried by the menus Stage 5 has just drawn.

1. **`Command::ShowOnly`,** which writes one field of `Rules` (`src/view/narrow.rs:22-34`), calls
   `apply_narrowing` (`src/app/mod.rs:428-441`) and raises the filter bar so the change is visible
   and reversible. Plain click replaces the rule, ctrl-click adds to it. It is offered wherever a
   mark is drawn: the status bar's marks, the cell caption, the tag panel's stars, flags, swatches
   and keyword chips, the filmstrip, and the `n could not be opened` line
   (`src/app/panels.rs:193-195`). One constraint decides its shape: `Command` derives `Copy` and is
   taken by value (`src/app/input.rs:11`, `src/app/mod.rs:684`), and three of `Rules`' seven fields
   are `String`s, so a `Copy` payload naming rule and value covers stars, flags and labels, and a
   keyword rule needs `Command` to stop being `Copy`. This one verb closes every row of §8.2, the
   hidden-count `(+18)`, the failed-image count and the empty state (§8.11).
2. **`Command::Reveal(Target)`,** where the target is a place: the sheet at this frame, the run this
   frame belongs to, Group shots at this run, the filter control behind this number, the settings
   row behind this readout, the keyword catalogue, the configuration file, the log. Four are already
   half-built — `go_to` (`src/app/mod.rs:527-534`), `focus_on`
   (`src/view/grid_view/mod.rs:204-208`), the two `path()` functions, and `actions::execute`, which
   already spawns a program with a safe argument vector (`src/actions/user_action.rs:75-100`). What
   has to be written is a scroll-to-group in the organiser, where no `scroll_to` exists, and a
   focus-a-row in the settings window (§8.11). The first place it lands is §8.5's own last pair of
   rows: every row of a Bulk rename or Shift capture time preview gets the camera's thumbnail beside
   its two names, and a click that opens that frame. The scan already decodes that thumbnail into
   `Entry` wherever the file embeds one (`src/view/organize/table.rs:60-81`,
   `src/organize/mod.rs:59-61`), so the picture is in hand and is being discarded; today somebody
   renaming four hundred files is reading paths and trusting them. That is the sixteenth of §8.8's
   sixteen dead ends; item 8 repairs the other half of the same table, which is the column that
   lies (§8.5).
3. **One selection every command reads.** `marked_paths` is already the rule for marks, tags,
   delete, move and copy (`src/app/tagging.rs:277-283`); three changes finish it. The folder jobs
   take a scope instead of `all_paths()` (`src/app/views.rs:35-38`) and say in their header what
   they are about to act on — `187 of 2030 files · the selection`. The count moves out of
   `GridView::ui` into a place both views draw. And the `Mode::Grid` gate comes off, so leaving the
   contact sheet does not silently reduce a selection of two hundred to one. The stack caveat is
   said on the button: a folded run contributes only its standing frame to `visible`
   (`src/view/stacks.rs:193-227`), so `Ctrl+A` over a stacked folder selects one frame per burst
   while the plate says `❏ 17` (§8.11).
4. **Notices carry routes — on their rows in the history, not on the band.** A failure that names a
   file offers to narrow to it; anything that ends in the log offers the log; the startup clash
   notice carries "Fix it", which opens Keys and mouse scrolled to the row that clashed (§8.6).
   Stage 1 gave the band severity and a history and left it non-interactive on purpose, so the
   buttons live on the rows of the **Messages** window and in Stage 4's permanent footer, which is
   where the startup clash and the partial-configuration warning already have a home that does not
   fade. Nothing a person has six seconds to click; everything a person can go back to. This is the
   part that needed `Reveal`.
5. **One burst detector behind both surfaces.** Two independent `group::Settings` values
   (`src/app/stacking.rs:28`, `src/view/organize/mod.rs:56`), tuned by two control sets that do not
   span the same ranges, so there are tolerance and gap values the contact sheet cannot express and
   the organiser can. Two answers to "is this one burst?" is a defect whether or not anybody
   navigates between them. One setting, from the `group` section Stage 3 added, read by both. This
   is the one route in §8 that costs real work and it is worth doing on its own merits (§8.1,
   §8.12).
6. **One keyword filter behind both surfaces.** The browsing bar takes a case-insensitive substring
   of the whole hierarchical keyword and the organiser a match on the whole stored keyword
   (`src/view/narrow.rs:265-275` against `src/organize/filter.rs:119-127`), so "rename everything I
   tagged Slovakia" means typing the word twice and getting two different answers (§8.4).
7. **The panels agree about what "the current photograph" is.** The metadata and histogram panel
   reads `image_view.active_metadata()` while the tag panel reads `marked_path()`, which is the
   grid cursor in the sheet (`src/app/chrome.rs:114-122`, `src/app/tagging.rs:263-268`), so with
   both open the two describe different photographs. There is no shared answer for a link to carry
   (§8.9).
8. **Two dead ends that are also wrong.** `Blown 3.4%` and `Crushed 0.2%` become toggles for the
   clipping overlay's two halves, which today is a separate key in a different subsystem
   (`src/ui/histogram.rs:108-134`, `src/view/image_view/mod.rs:281`); and the Shift-capture-time
   preview computes its "would become" column from `Date/Time Original` alone, so unticking that
   field while leaving `Modify Date` ticked makes every row read `—` while the button still says
   `Change 412 file(s)` and is enabled (`src/organize/timeshift.rs:185-210`,
   `src/view/organize/timeshift.rs:99-158`). One column per ticked field, and the button counts what
   the preview shows (§8.5).
9. **A command that cannot act says so and offers the setting that would let it.** `Ctrl+T` flips
   `filmstrip_visible` and `show_filmstrip` returns immediately while the height is zero, which is
   the default (`src/app/mod.rs:713`, `src/app/views.rs:137-141`,
   `src/config/defaults.rs:299-301`), while the binding is advertised in the editor and on the cheat
   sheet in every mode. It is FastRawViewer's disabled-reject bug exactly
   (<https://www.fastrawviewer.com/node/577>). Stage 3's `filmstrip_visible` field fixes the
   mechanism; this is the sentence for whatever the next one is (§8.7).
10. **The cheat sheet gets its way out.** Stage 1 made it reachable and searchable and left it a
    list of statements. §8.7's first row is the route: the footer sentence
    (`src/ui/cheat_sheet.rs:96-100`) becomes a button opening **Keys and mouse**, and every row in
    the sheet opens that page with its own row armed — which is Route 3 for that page in §3.3's
    table, and the half of §8's closing line that was otherwise unscheduled. It waits for this stage
    because there was no page to open until Stage 3 and no `Reveal` to open it with until item 2.
    The registry already records each binding's mode, since the sheet filters by it
    (`src/ui/cheat_sheet.rs:28-36`), so the row knows where it is going (§8.7, §3.3).

*Finished when:* clicking three stars in the status bar leaves three stars and better on screen with
the filter bar open saying so; clicking `Blown 3.4%` puts the clipping mask on the photograph; the
selection count is a button that survives leaving the contact sheet, and the folder jobs say what
they are about to act on; the sheet and Group shots agree about what a burst is; a rename preview
shows the photographs it is about to rename; the cheat sheet has a way into **Keys and mouse** and a
startup clash notice has a way to the row that clashed; and every notice about a failure can be
opened again after it has faded and followed to what failed.

---

### Stage 7 — The mouse, and the keyboard for people who do not know it

The most-repeated request across every viewer in the research, and the place this program is
furthest behind the field. It is here rather than earlier because the gesture table is registry
rows (Stage 2), it needs a page to be edited on (Stage 3), and its first repair was Stage 0's
button check. nomacs #237 ran from 2018 to 2025 across sixteen accounts, at least one of whom said
they had uninstalled over it, and the capability being asked for had existed behind two checkboxes
nobody could find (<https://github.com/nomacs/nomacs/issues/237>). avis-imgv is in the same
position with `scroll_navigation`, minus the checkboxes.

1. **A `mouse` section of eight fields:** `wheel`, `wheel_reversed`, `ctrl_wheel`, `drag`,
   `double_click`, `middle`, `back`, `forward` — §6.2's list and §6.5's defaults, and no ninth. The
   wheel's job in the contact sheet is not a field: the sheet is an ordinary `ScrollArea`
   (`src/view/grid_view/mod.rs:274`) and scrolling it is what a wheel does there, which is item 3's
   argument for making the image view agree rather than the other way about. The values are
   commands from the registry, not a bespoke enum per gesture, so adding a gesture adds a row and
   not a vocabulary — which is also the answer to IrfanView's author's objection that "more options
   are not always a good move as they make programs harder to support" (§6.5, §9.8).
2. **`scroll_navigation` migrates into `mouse.wheel`.** It is the one key the plan moves, and it
   costs a rewrite of the `serde_json::Map` before the typed section is built
   (`src/config/load.rs:133`, `:151-158`) plus a step in `src/config/migrate.rs`, which already
   reports what it moved. `true` becomes "next or previous", `false` becomes "nothing" — except
   that a binding turned off must fall back to *something*: with nomacs's zoom checkbox unticked
   the wheel did nothing at all (<https://github.com/nomacs/nomacs/issues/1281>) (§4.4, §9.3).
3. **The wheel does one job at a time.** Today one notch over a zoomed photograph calls
   `Command::Next` *and* writes `smooth_scroll_delta` into the arriving image's viewport
   (`src/view/image_view/input.rs:197-201`, `src/view/image_view/interaction.rs:40-45`), so the
   photograph that has just arrived is shoved. And the two views disagree about direction: wheel-up
   moves forward in the image view and wheel-down moves forward in the sheet, which is an ordinary
   `ScrollArea` (`src/view/grid_view/mod.rs:274`). The sheet is right and the image view changes,
   with a reverse flag for the people whose muscle memory says otherwise (§9.2, §9.3).
4. **Claim Shift and Alt back from egui, deliberately.** Shift is egui's
   `horizontal_scroll_modifier` and Alt its `vertical_scroll_modifier`
   (`egui-0.33.0/src/input_state/mod.rs:115-116`, `:451-464`), so Shift+wheel silently pans
   sideways and `raw_scroll_delta.y` is zero before this crate sees it. Writing
   `Options::input_options` at startup is a few lines; the point is that it is a decision and not
   an inheritance (§9.2, §9.3).
5. **A single click in the contact sheet selects; a double click opens.** Not a field: a decision,
   made in code, which is what §11.2's rule prescribes for a value two reasonable people would not
   disagree about. The sheet already has a cursor, a selection, `Ctrl`-click, `Shift`-click, `Space`
   and `Enter`, and the plain click is the one gesture that contradicts all of them — it sets
   `self.selected` and `src/app/views.rs:84-87` changes mode on the same frame, so the only way back
   is `Backspace` (`src/view/grid_view/mod.rs:444-457`, §9.2 fault 6). A culling tool's contact sheet
   is a surface you act *on*: Camera Bits describes applying a colour class by selecting the label
   on a photo in a contact sheet
   (<https://docs.camerabits.com/support/solutions/articles/48001252564-color-class-ratings>), which
   is not a thing you can do on a surface that closes when you touch it. §9.3's table states the
   behaviour. `Open` is the bold default row on a cell's menu, which is a lie until this lands
   (§9.2, §9.3, §7.5).
6. **The left drag in the sheet is a rubber band, and the middle drag scrolls it.** Drag-to-pan and
   drag-to-select never have to share a button, because they never share a surface: the image view
   has nothing to rubber-band and the sheet and the strip have nothing to pan (§9.3). What must not
   be copied is the size-dependent rule Picview documents — "when image exceeds window size,
   dragging moves within the image instead"
   (<https://picview.chitaner.com/blog/mouse_keyboard_trackpad/>) — which is a mode with nothing on
   screen to say which one you are in, and is §9.2's fourth fault in another program. One risk is
   tested before anything is designed around it: egui #7390 reports `context_menu()` no longer
   opening on a drag source since 0.32, unverified against 0.33
   (<https://github.com/emilk/egui/issues/7390>, §7.12), and a cell has to carry both. If it cannot,
   the selection drag goes on the sheet's background rather than on the cells, which is §9.3's own
   fallback and costs the drag nothing but its starting point.
7. **The side buttons walk the folder, and they fire on the down-stroke.** `Extra1` and `Extra2`
   already arrive (`egui-winit-0.33.0/src/lib.rs:1111-1112`) and nothing reads them. No
   double-click meaning, ever: nomacs #451 records a viewer that waited to see whether a side-button
   click was a double, making navigation feel slow and still advancing one frame
   (<https://github.com/nomacs/nomacs/issues/451>). The binding is visible in the editor even on a
   machine where nothing ever arrives on it (§9.3).
8. **The middle button exists and defaults to nothing,** which satisfies GNOME's advice against
   relying on it and every shipping viewer's disagreement with it at the same time; a middle drag
   pans unconditionally, so a fitted photograph is not a dead surface — nomacs #919
   (<https://github.com/nomacs/nomacs/issues/919>). A drag begun on the zoom slider stops panning
   the photograph: `handle_pointer` gates on `contains_pointer()`, which egui documents as true
   "even if some other widget is being dragged" (`src/view/image_view/interaction.rs:16`) (§9.2,
   §9.3).
9. **Dropping a file on the window opens it.** egui-winit already pushes every dropped path into
   `RawInput::dropped_files` (`egui-winit-0.33.0/src/lib.rs:468-473`), nothing in `src/` reads it,
   and the collection-opening code exists (`src/app/mod.rs:297-302`). Nearly free (§9.7).
10. **Rotate, written to the sidecar and never to the photograph.** It is the most-expected verb
    after delete, and nomacs #799 is the angriest issue in the whole corpus precisely because nomacs
    implements it by rewriting the file — "the file on disk is silently modified… all without the
    knowledge or consent of the user" (<https://github.com/nomacs/nomacs/issues/799>). Here it is an
    orientation in the sidecar composed with the EXIF one before `to_texture`
    (`src/view/texture.rs:34`, `:50-67`); the composition also has to reach `displayed_size`
    (`:73-79`) and the decoder's stored dimensions (`src/decoder/mod.rs:264-266`), which is the part
    that is not free (§9.7).
11. **Two keyboard leftovers.** The "go to" field surrenders focus if it gains it without a click,
    because `Tab` means "the other pane" while comparing
    (`src/view/image_view/bottom_bar.rs:218-225`) — sound reasoning and a control that cannot be
    operated without a mouse, so it gets a key of its own. And `/` drops a pane read with
    `Modifiers::NONE` (`src/view/image_view/input.rs:116-119`), which on the Slovak, German and
    French layouts makes it unpressable and unrebindable; the same shape as the destination digits,
    whose comment names the problem the code does not solve (`src/ui/destinations.rs:135-137`).
    Both are registry rows after Stage 2 (§9.5, §6.12).
12. **The cheat sheet lists the mouse,** and a test asserts that no command has fewer than two
    homes — a key, a gesture, a menu entry or a context menu. Four lines, and it prevents the entire
    class of command that exists with no route anybody will find (§9.8, §9.9).

*Finished when:* the wheel can be made to zoom instead of navigate and its direction reversed; a
single click in the contact sheet selects and a double click opens; a drag across the sheet picks
out what it crosses; a double-click on the photograph does something and can be told what; the thumb
buttons step through the folder;
dragging a file onto the window opens it; a photograph can be rotated without the file being
touched; and a configuration written before this stage opens with its `scroll_navigation` carried
across and the carry named on screen.

---

### Stage 8 — The change takes effect while the window is still open

The architectural stage, and the one the whole plan is for. Twenty-six settings do not take effect
until the next launch and nothing anywhere says so, while the two things the interface can change
today both apply immediately — so the mental model it teaches is exactly wrong for all
twenty-six. **A restart badge is a bug report, not a feature.** After this stage exactly one field
carries it. The other half of the stage is the same class of problem seen from the other side: work
the program does after being asked, with nothing on screen to say it is happening.

1. **`Stores::rebuild(StoreConfig, Arc<str>)`.** `stores::image_store` and
   `stores::thumbnail_store` are already pure functions of the configuration
   (`src/app/stores.rs:32-66`) and `ImageStore::new` takes a `StoreConfig` by value
   (`src/cache/store/mod.rs:117-122`), so "rebuild" means constructing a fresh store and re-seeding
   it with `set_paths` and `set_cursor` — which is exactly how a folder is opened today
   (`src/view/image_view/navigate.rs:23-32`, `src/view/grid_view/mod.rs:90-100`). Both views must
   first keep the `RenderState`, the `Arc<Loader>` and the `Arc<str>` profile that `ImageStore::new`
   consumes; `RenderState` is `Clone` and the other two are already `Arc`s. The call goes beside
   the existing four-line fan-out (`src/app/settings.rs:89-92`). That is seventeen fields at once:
   five of the six `cache.*` — every one but `decode_threads`, and including `upload_budget_ms`,
   which has no control but is still read from the file — the five `raw.*`, `nr_loaded_images`,
   `gpu_resident_images`, `max_image_edge`, `thumbnail_resolution`, `gpu_resident_thumbnails`,
   `preloaded_rows`, and `general.output_icc_profile`, which is the reason for the second argument
   (§3.5). With the rebuild in place the five boxes Stage 3 held back become rails, which is the
   condition §5.9 set for drawing them at all: the two budgets, the camera-thumbnail count, the
   decode ceiling and the thumbnail resolution. Eleven rails become sixteen, and the badge comes off
   every one of them.
2. **Five become Live, three of them one line each.** `apply_text_scaling` called again from a
   stored base (Stage 0 made it idempotent); `self.images_shown = config.nr_images_shown.clamp(1,
   MAX_IMAGES_SHOWN)` in `ImageView::set_config`, which is the clamp already written at
   `src/view/image_view/mod.rs:129`; `self.set_columns(config.images_per_row.clamp(1,
   MAX_COLUMNS))` in `GridView::set_config`, where `set_columns` exists at
   `src/view/grid_view/mod.rs:643` and `MAX_COLUMNS` at `:42` and the `+`/`−` keys already respect
   it while the configuration path does not; `tags.advance_after_marking` written by the key that
   flips it (`src/app/mod.rs:706`); and the filmstrip's visibility, which Stage 3 split out of its
   height (§3.5).
3. **The keyword catalogue is rebuilt the way the stores are.** `Catalog::configured` and
   `RecentTags::load` run once (`src/app/mod.rs:209-210`), so `tags.categories`, `catalog_file` and
   `recent_tags` are restart-bound today, and editing a keyword list means restarting the viewer
   (§3.5, §8.4).
4. **`tags.panel_width` takes effect both ways.** `.default_width()` is honoured only while egui has
   no stored width for that panel id (`src/ui/tag_panel/mod.rs:64`), so a mid-session change does
   nothing. Draw one frame with the width forced when the window is what changed it, and write the
   dragged width back into the field from the `InnerResponse` (§3.5).
5. **The runtime toggles are written where they belong.** Once the configuration is authoritative,
   the keys that nudge these values — the overlay corner, panes side by side, thumbnails across,
   advance-after-marking, the filmstrip, the badge mode — must write the field too, or the next save
   from the settings window snaps the view back to whatever the file still says. Preferences go to
   `config.json` through the registry; only *position* stays in the session file, because `on_exit`
   returns early when `restore_session` is off (`src/app/mod.rs:866-872`) (§3.5, §4.8).
6. **The one that genuinely cannot, said plainly.** `cache.decode_threads`: the pool is spawned once
   in `Loader::new`, each thread is `.expect`-ed, and one `Arc<Loader>` is shared by both views
   (`src/cache/loader.rs:108-133`, `src/app/mod.rs:162`). Draining a running pool mid-session is a
   larger job than this plan should smuggle in, and pretending otherwise would be dishonest (§3.5).
7. **How it is marked.** A `↻` badge on the row, with "takes effect the next time the viewer starts"
   written *under* the control rather than in a tooltip, because a restart requirement is a field
   requirement (§10.7). A persistent footer while anything is waiting — *"One change needs a
   restart: decode threads."* — with **[Restart now]**, which saves the session and relaunches;
   darktable's own fix for this complaint was a toast on closing the dialogue
   (<https://github.com/darktable-org/darktable/pull/5957>), and a button that does the thing is
   strictly better. A setting about the *next launch* is not a restart: the whole startup group —
   `restore_session` and the new `start_in`, `start_fullscreen`, `start_folder` and
   `panels_at_start` — gets a sentence and no badge, and so does `raw.pair_with_jpeg`, which needs
   the folder re-opened. A badge means *your change has not taken effect*, and using it for changes
   that have is what teaches people to ignore it (§3.5).
8. **`--reset-text-size` on the command line,** beside the three flags that already exist
   (`src/main.rs:43-45`) and in the `--help` text, for the case where the interface is unreadable
   for some other reason (§3.6).
9. **Chunk the folder crawl across frames.** `crawler::crawl` has one caller,
   `open_directory` (`src/app/mod.rs:677-681`), reached from nine places, and on a deep tree or an
   SMB share every one of them stops the window repainting with nothing on screen to say why —
   which is the eighth of §10.2's eight states and the only one that draws nothing because the
   program is not drawing at all. There is no interim measure short of this: a `ViewportCommand` is
   applied after `update` returns, so even setting the title first reaches the screen only once the
   crawl has finished. **Chunked, not moved to a worker.** `open_directory` is three synchronous
   lines — crawl, sort, `open_within` — and every one of the nine has a follow-on that assumes the
   collection is in hand when it returns: startup lands on a path (`src/app/mod.rs:260-285`), undo
   restores a position (`src/app/cull.rs:521`), a finished folder job re-opens and re-focuses
   (`src/app/views.rs:166`), the tree and the navigator set the base path
   (`src/app/chrome.rs:137`, `:148`), and the watcher re-crawls on every change. Making the
   crawl asynchronous turns it into a two-phase operation that has to carry each caller's
   `selected`, its base path and its follow-on across frames, plus a rule for a second open arriving
   while the first is still walking. The saving grace is that eight of the nine hand their follow-on
   *in*, as `open_directory`'s two arguments, and ignore what it returns. So `open_directory` stashes
   `(path, selected)`, `update` walks a slice of the tree per frame and calls `open_within` when the
   walk finishes, and eight call sites are untouched. One is not: `src/app/views.rs:166-167` calls
   `set_images(self.all_paths())` on the next line, which is a follow-on that has to move into the
   completion. A second open arriving mid-walk replaces the first — the same rule the watcher's
   re-crawl already needs. That is the shape to build; a worker thread is a further step and does
   not belong inside a ten-item stage (§10.2, §10.10).
10. **Progress that answers "is it stuck".** An indeterminate mark at the foot of the window while
    `loading > 0` — `StoreStats` cannot give a folder-wide percentage, because `in_ram` is the
    length of the LRU cache and the preload radius is trimmed to what the budget holds
    (`src/cache/store/mod.rs:460`, `:470-482`), so `in_ram < total` is permanently true on any large
    folder and a bar driven by it would never go away. Determinate bars only where a total exists:
    the folder scan (`src/view/organize/mod.rs:260-267`) and the stack read
    (`src/ui/filter_bar.rs:113`) both compute one and both draw it somewhere the user must have
    opened first, so `Ctrl+G` announces itself with a six-second notice and then reports nothing
    unless `F3` is pressed (§10.10).

*Finished when:* moving the RAM budget slider empties the cache and refills it without leaving the
window; switching the raw source from preview to develop redraws the photograph on screen; changing
the text size changes it while the window is open; the contact sheet's columns and the image view's
panes follow the configuration and not only the keys; every runtime toggle is where it was left on
the next launch; opening a folder of two thousand over a network share does not freeze the window;
and exactly one row in the whole window carries the restart badge.

---

**What each stage costs, and what it buys.** Roughly seven thousand lines against the 40,504 in
`src/` — the nine figures below add to 7,000 — and a quarter of it is one table.

| Stage | Roughly what it costs | What the user gets |
|---|---|---|
| 0 — Stop editing files nobody asked about | ~150 lines over nine files; days | A configuration file that survives being read by the program, and a viewer that does not die of a typo |
| 1 — The program says what it is doing | ~700 lines, most of it prose; two weeks | A right-click that does something, a Help menu, an About window, ninety-eight explanations where there were thirty-three, and a first screen that offers a folder |
| 2 — One table both read | ~1,800 lines, of which 250 are tests; two to three weeks | Nothing visible except a keyboard editor that can search, reset one row, unbind one row and report the clashes that matter |
| 3 — Every setting has a control | ~1,650 lines; four weeks | Forty-one settings that could only be hand-edited, six more that a key nudged and threw away, and thirty-five that did not exist, get a control and a search box — less the four that item 7 takes off the pages |
| 4 — The window says what is wrong | ~450 lines; a week | A bad value names itself and offers a button; every change can be found and put back; a configuration fits in a four-line file that travels |
| 5 — Right-click the thing itself | ~700 lines; two to three weeks | About twenty surfaces answer the second button, each one click from the page it belongs to, and four controls move to where the effect is |
| 6 — One thing to the next | ~450 lines; a week and a half | Clicking a mark shows only those; clicking a number opens what caused it; the selection survives leaving the sheet; the cheat sheet leads into the keys |
| 7 — The mouse | ~450 lines; two weeks | Eight gestures that can be changed, a wheel that does one job, a drag that picks out thumbnails, thumb buttons, drag-and-drop, and rotation that leaves the file alone |
| 8 — The change takes effect | ~650 lines, about 150 of them the chunked crawl; three to four weeks | Twenty-five of the twenty-six restart-bound settings apply while the window is open, and a slow folder says so |

No stage leaves the viewer unusable and none has to land whole. Stage 2 is the largest single piece
of typing and it is invisible; Stage 3 can ship one page at a time; Stage 8 can ship one `Effect`
row at a time. If the work stops after any stage, what shipped is coherent on its own — which is
the argument for Stage 1 being where it is.

## 13. Deliberately not doing

Some of these are proposals the chapters make and this plan declines; some are things the research
shows people ask for that would be wrong here; the last few are roads not taken while the four
competing settings architectures were being reconciled, recorded so that nobody has to work out
again why they were refused.

- **No command palette.** Architecture C put search first and would be faster for anybody who can
  always name the thing; the power-user judge chose it for that reason. It is not built, because
  the only palette chassis to reuse calls `request_focus()` on its query field every frame and
  compensates the text cursor by hand after ArrowUp (`src/ui/navigator.rs:47`, `:73-77`), and a
  `DragValue`, `TextEdit` or `Slider` inside a result row cannot hold focus against that without a
  hand-rolled hand-off — while a focused slider and the result list both want the arrow keys.
  Everything the palette was *for* survives inside the settings window: the aliases, the
  equivalence table, the stop words, the exact-path match, the refusal to reorder, the never-empty
  result and the corpus test. No photo or culling application in the survey ships one, and
  darktable's shortcut search drew a bug report titled "shortcut search is unintuitive and no clues
  are given" (<https://github.com/darktable-org/darktable/issues/9378>).
- **No second file recording where a value came from.** Architecture D's `default` / `yours` /
  `kit:<name>` origin map is a second document that must be updated on every write from every route
  and can silently disagree with the first; D names the failure honestly — a lost map promotes a
  kit's values to "yours" and the kit becomes un-undoable. The one bit actually needed,
  differs-from-default, is computed against `Config::default()`, which `src/ui/keys.rs:112` already
  builds for its own reset.
- **No named built-in kits or profiles.** Once eight named kits exist, "which kit am I on?" is a
  question the program has to answer on every page for ever. No request for settings *profiles* in
  an image viewer turned up anywhere in the research; the pattern belongs to developer tools
  (<https://code.visualstudio.com/docs/editor/profiles>). D's eight job names survive where they
  earn their place — as the headings of the search box's "start here" list, and as four buttons
  that fill in the memory boxes and then get out of the way.
- **No pages organised by job.** D's fifteen job pages answer "Culling" better than anything else
  written and lose three of the novice's four questions to ambiguity: the grey behind the
  photograph is on "Judging one photograph" and again on "The screen it's on", neither of which
  contains the word background, and the rejects folder is on "Sending the files somewhere", which
  contains neither "reject" nor "delete".
- **No page named for what a field is made of, and none called General, Advanced, Miscellaneous or
  Other.** Microsoft names all four as labels to avoid, and the plain/advanced line cannot be drawn
  by technology anyway: `raw.source` is the most consequential setting a raw shooter has and lands
  in Advanced under any honest rule (§11.3). Disclosure is inside a group on the page it belongs
  to, never a second page.
- **No deferred Apply button for the cache fields.** Architecture B proposed one; commit-on-gesture-
  end does the same work — one rebuild per gesture — without breaking the promise the other rows
  make, and Microsoft warns against mixing commit models in one surface.
- **No named levels for the memory budgets.** darktable replaced six numbers with four resource
  levels and FastRawViewer names its GPU memory Minimal / Optimal / Maximal
  (<https://www.fastrawviewer.com/usermanual17/performance-settings>). Neither is copied: the
  number is what a forum answer quotes and what somebody types to make this machine behave like
  that one, and a name does not travel where a number does. Expressing a budget as a fraction of
  the machine is not available either — nothing in `src/` queries system memory and wgpu reports
  the adapter's texture limits rather than its memory (`src/cache/gpu.rs:129-132`), so any
  percentage-of-machine phrasing costs a dependency. The four names survive as four buttons above
  the boxes.
- **No six booleans for what the status bar shows.** Architecture B proposed them and named them as
  the row it would cut first. The status-bar flags become doors; they do not become configuration.
- **No submenus, with one exception.** egui #5251 places a submenu with its right edge against the
  screen edge where long text can cover its parent (<https://github.com/emilk/egui/issues/5251>),
  and every panel in this program sits against an edge. Re-tested against 0.33: a submenu that will
  not fit folds to the far side of its parent instead, which is what the five turns behind *Turn*
  are worth (§7.12). Where a menu needs five choices that are five different decisions it still
  draws them as an inline radio row (§7.2).
- **No "Show more options" overflow row.** Windows 11's is the most-complained-about context menu in
  current use, and Microsoft's own account is that the menu "appear[s] cluttered with a long list of
  actions, something that has been bothering users for a long time". An overflow hit on almost every
  invocation is worse than a longer menu; the twelve-row ceiling is enforced by dropping rows
  (§7.13).
- **No menu editor.** One boolean decides whether the settings rows appear
  (`menus.settings_rows`, §7.11) and that is all. digiKam's thumbnail menu runs past fifty entries
  with submenus and is the counter-example rather than the model; a menu each user assembles cannot
  be documented, cannot be taught by the cheat sheet and cannot be relied on by the rest of this
  plan. darktable closed the equivalent request as not planned
  (<https://github.com/darktable-org/darktable/issues/14857>).
- **No "Delete for good" on any menu.** `Shift+Delete` stays a key that confirms
  (`src/app/cull.rs:65-91`); a permanent delete two pixels from "Copy the path" is exactly the
  mis-click ImageGlass's reporter described
  (<https://github.com/d2phap/ImageGlass/issues/1342>) (§7.13).
- **No menu on the notice band, the cheat sheet or the delete confirmation** — three surfaces that
  are transient or reference and carry no setting between them, and Microsoft sanctions a dead
  right-click explicitly: "For other toolbars, do nothing". §7.13's own rule is the test — "a
  right-click on the thing that carries a setting and does nothing" is a defect — and by that test
  the slideshow window, which §7.13 lists beside the other three, does not belong with them: it
  carries three of the 110 settings today (`src/app/panels.rs:92-135`, §3.1). It is excluded for a
  different reason, which is that after Stage 3 it no longer exists. `Settings ▸ Slideshow…` becomes
  a deep link to the **Slideshow** page (§3.3), which carries all five `slideshow.*` fields rather
  than the three the window draws, and that page is reached by right-clicking the photograph while a
  show is running, like every other page (§3.3's route table). So nothing that carries a setting is
  left without a menu, which is what the brief asked; three things that carry none are left without
  one on purpose.
- **No first-run tour and no welcome window.** NN/g measured tutorials making tasks feel *harder* —
  a single-ease score of 4.92 against 5.49, p=0.047
  (<https://www.nngroup.com/articles/mobile-tutorials/>) — and the paradox of the active user says
  people start immediately regardless. The empty state, the one status line and the "start here"
  list carry the whole onboarding load (§10.3).
- **No hotkeys painted over the controls while a modifier is held.** ExposeHK reported increased
  hotkey use across three studies and it is a real idea, but it cannot hang off `?`, which is itself
  a shifted character on most layouts, and nothing today connects a *drawn control* to its binding,
  so every call site would have to say which command it is. It belongs after the Help menu and the
  tooltips, not instead of them (§10.5).
- **No reordering by use, in any list.** No recency, no frequency, no most-recently-used at the top
  of the search results, the pages, the cheat sheet or any menu. A flat, spatially stable surface is
  what makes a repeated action fast, and a list that moves under the hand defeats the muscle memory
  a search box exists to build (§3.4).
- **No localisation.** Not while every string in the program is an English literal and no
  internationalisation crate is in `Cargo.toml`. The demand elsewhere arrives as offers rather than
  complaints — qimgv #79 produced a translation somebody attached five years later — so nobody is
  blocked. Two prerequisites are honoured now because they cannot be retrofitted: bindings are
  keyed on a stable English path and never on a display name, which is nomacs #1539
  (<https://github.com/nomacs/nomacs/issues/1539>), and `Label::name` stays English whatever the
  interface says, which is a correctness property rather than a display choice
  (`src/metadata/xmp/mod.rs:126-135`) (§6.12).
- **No per-folder settings and no per-monitor profiles.** Nobody in the corpus asked for either:
  §6.13's row records "No demand found in the corpus" against per-monitor and per-profile settings,
  and the nearest per-folder ask is Lightroom's per-source filters — *"it used to be that when
  navigating between folders … LR would remember my filter settings"* — which is one field here,
  `browsing.filter_follows_folder`, and not a second configuration scope (§2.9, §6.7). The refusal
  rests on the cost, which Geeqie has already paid: with several windows open, "depending on which
  instance I then close last, the wrong settings also get stored"
  (<https://github.com/BestImageViewer/geeqie/issues/1324>). A second scope means a second set of
  precedence rules on every page for ever. One file, one profile.
- **No reading of the platform's own folder-view sort order.** JPEGView and ImageGlass have both
  closed this as not planned, and the ask underneath it — "the viewer does not agree with how I
  sorted the folder" — is answered by `browsing.sort` and `browsing.descending`, which are settings
  the user can see (§2.9, §6.7).
- **No cloud, no account, no sync.** Bundles are files in a folder beside `config.json` and are
  copied by hand.
- **No writing marks into the photograph.** digiKam offers three metadata modes; the source's own
  reason holds — "Rewriting a photograph to change a star is both slow and risky, and every raw
  converter already looks for a sidecar" (`src/annotations/sidecar.rs:3-5`). A star that rewrites a
  60 MB raw contradicts the one thing this program is for (§6.13).
- **No hold-to-delete, and no way to switch off the confirmation on a permanent delete.** qimgv's
  maintainer proposed hold-to-delete and its own reporter answered it in one line — "what happens if
  you hold it for longer than that? Will it start deleting other files?"
  (<https://github.com/easymodo/qimgv/issues/37>). The answer everybody in that thread would have
  accepted is immediate and reversible, which is what the undo journal already provides; so
  `cull.confirm` covers only the operations the journal covers, and deleting for good and putting
  the keyboard back to the defaults always ask (§6.9).
- **No configurable clipping and focus-peaking colours, and no settable peaking threshold.** Red,
  blue, green and a five-per-cent share (`src/decoder/overlays.rs:57-61`, `:78`), and nobody outside
  this repository has asked. The threshold is the arguable one, because "the strongest 5 % of
  gradients" is a judgement about a photograph rather than a colour — but §6's second test is
  disqualifying, and what it needs is a *sentence* saying what it is, which §10.8 writes (§6.13).
- **No configurable zoom preset percentages, and no badge-size field.** 200/100/75/50/25 is a
  conventional list and the rail beside it already covers 1–1600 % logarithmically
  (`src/view/image_view/bottom_bar.rs:12`, `:249-263`). The badge strip's problem is not a missing
  setting: `CAPTION_HEIGHT` is a flat 20 points whatever the cell size
  (`src/view/grid_view/cell.rs:15`, `:48`), so at sixteen columns it is enormous and at one column
  the stars are a sliver. That is a layout bug, and it is fixed rather than made settable — the
  scaling is item 12 of Stage 1, which is what this refusal rests on (§6.6, §6.13).
- **No field for what a single click in the contact sheet does.** The click selects and the double
  click opens, decided in code (Stage 7, item 5). §9.3 states it as behaviour and not as a setting,
  and it fails both limbs of §11.2's rule: the sheet's own cursor, selection, `Ctrl`-click,
  `Shift`-click, `Space` and `Enter` all already assume it, so there is nothing for two reasonable
  people to disagree about, and nobody outside this repository has asked for the other behaviour.
  The gestures that are genuinely contested — the wheel's job, the double click's, the middle
  button's — are the eight `mouse.*` fields, and they are settings for that reason (§9.3, §11.2).
- **No `general.interface_font` field.** It is not a restart problem — `ctx.set_fonts` is an
  ordinary runtime call and the program already makes it (`src/ui/theme.rs:77-101`) — but nothing in
  `Cargo.toml` can enumerate installed families or read their bytes, so the obstacle is a dependency
  and the field would offer a choice the program cannot honour (§3.5).
- **No window-size preference separate from the session.** Asked for against JPEGView and
  lximage-qt; already done here. The geometry is session state, written on exit and read before the
  window is made (`src/session.rs:32-51`, `src/main.rs:96-111`), and the window never resizes itself
  to the image (§6.13).
- **The undo journal is not extended to the three folder jobs.** Bulk rename, the capture-time shift
  and the group tidy are called straight from their button handlers with no journal entry
  (`src/view/organize/rename.rs:94`, `timeshift.rs:110`, `group/mod.rs:101`) and none appears in
  `Step` (`src/organize/journal.rs:28-43`) — they are the three most destructive things the program
  does. This plan's answer is to say so on the button, in Stage 1's tooltips
  (**this cannot be undone**), because covering them is a file-operations change of the kind
  `plan.md` §2 owns and not an interface one. It belongs in the next such plan, and the journal's
  being in memory only (`src/app/mod.rs:227`) belongs with it (§2.7).
- **The three folder-job applies stay on the UI thread.** `rename::apply`, `timeshift::apply` and
  `gather::apply` each work through every planned file inside a button's click handler with no
  progress and no cancel. They are behind an explicit button press on a screen that has just shown
  the whole plan; the crawl, which is the one that happens without being asked for, is the one Stage
  8 moves (§2.8, §10.10).
- **No settings screen that is the only route to anything a person can see the effect of.** Four
  controls move into the view for that reason (§5.8); the rule is the general form of it, and it is
  the one thing every guideline in the research agrees on.
- **The sixty `sc_*` fields are not gathered into a `keys` section.** §9.8 proposes it — "the
  configuration file gets one section instead of sixty scattered fields", migrated by
  `src/config/migrate.rs` — and it is declined on the three grounds the last bullet below rests on
  (§4.4). Sixty renames is sixty chances at the silent discard rather than one. Migration runs on
  the already-parsed `Config` (`src/config/migrate.rs:55`), by which point an unrecognised key has
  gone, so a file written after the move and read by an older build loses every binding rather than
  one. And nothing is gained: the registry already groups what the file scatters, and the path is a
  search token rather than a filing system (§4.4, §3.4). A tidier document is not worth sixty key
  moves.
- **And nothing on disk is renamed or moved,** with one exception. No field changes its JSON path,
  no field moves between structs, and the file format does not change: a key nothing recognises is
  discarded in silence, because no struct carries `deny_unknown_fields` and an unrecognised key
  inside a section never reaches the arm that sets `partial` (`src/config/load.rs:175-182`). The
  eleven pages are a second view over the same registry — exactly the relationship
  `src/config/bindings.rs` already has with the sixty `sc_*` fields. The exception is
  `image_view.scroll_navigation`, which becomes `mouse.wheel` in Stage 7 and costs a map rewrite
  and a migration step that says what it moved (§4.4).

