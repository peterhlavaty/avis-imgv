# Change Log

## 2026-09-01

- **A drag begun on the zoom slider no longer moves the photograph.** egui
  reports that a widget contains the pointer "even if some other widget is
  being dragged", so letting the pointer stray up over the picture panned it
  under a drag that was never about it. A pan now has to have started on the
  photograph.

- **A photograph can be turned, and the file is not touched.** `[` and `]`, and
  a row each way on the second button. The turn is written to the sidecar as
  `tiff:Orientation` and composed with the camera's own before anything is
  drawn, so what is on screen is one turn however many went into it. It takes
  the selection with it, goes on both halves of a raw-and-JPEG pair, survives a
  restart, and `Ctrl + Z` puts both the sidecar and the picture back.

- **The cheat sheet lists the mouse.** Thirteen gestures with what each one
  currently does, read from the settings rather than written out, and every row
  opens the control behind it. A gesture nobody is told about is a gesture
  nobody has.
- **`/` is a binding now.** It dropped a pane in a comparison and was read as a
  bare `/`, which on the Slovak, German and French layouts is Shift and a
  digit: unpressable, and unrebindable, because a key read that way is a key
  the editor cannot see.
- **`Ctrl + J` puts the cursor in the "go to" box.** It could be reached by
  clicking and by nothing else, because the key that would have landed in it
  means "the other pane" while comparing.
- **Six keys said they were read in the contact sheet and were read
  everywhere.** Stacking and the five keys around it are taken by the
  unconditional collector whatever is on screen, so a new key given the same
  default would have been silently eaten — which is what happened, and is why
  there is now a test that the shipped defaults do not collide with each other.

- **A click in the contact sheet picks a photograph out. Two clicks open it.**
  The plain click was the one gesture that contradicted everything else the
  sheet has — a cursor, a selection, `Ctrl`-click, `Shift`-click, `Space` and
  `Enter` — and the only way back from it was `Backspace`. A culling tool's
  contact sheet is a surface you act *on*. `grid_view.click_opens` puts the old
  behaviour back.
- **A drag across the sheet picks out everything it crosses**, and the middle
  button drags the sheet about. The two never have to share a button, because
  the image view has nothing to rubber-band and the sheet has nothing to pan.
- **The thumb buttons walk the folder**, on the down-stroke and with no
  double-click meaning ever. They have been arriving all along and nothing read
  them.
- **The wheel pressed, and two clicks on the photograph, run whatever they are
  bound to.** Thirty-three commands to choose from, or nothing, which is what
  the middle button ships as. Two clicks ship as fit ↔ actual pixels, which is
  its own way back.
- **Dropping a file on the window opens its folder, on that file.** The paths
  have been arriving all along and nothing read them.

- **The wheel does one job, and you can say which.** A notch used to move to
  the next photograph *and* shove the one that had just arrived, in that order,
  with nothing guarding the second against the first. It now does one of four
  things — one photograph, zoom, move the photograph, or nothing — set in
  `mouse.wheel`, with `mouse.ctrl_wheel` for the same four with Ctrl held.
- **Wheel down is forward in both views.** It was wheel up in the image view
  and wheel down in the contact sheet, so the same movement of the wrist meant
  "later" in one and "earlier" in the other, and one key switched between them.
  `mouse.wheel_reversed` turns both round for anybody whose muscle memory says
  otherwise.
- **Shift and Alt with the wheel do something.** Shift moves ten photographs at
  a time in the image view and ten rows in the contact sheet; Alt moves the
  photograph sideways. Both were silently spent by the toolkit before this
  crate saw them — Shift is egui's horizontal scroll modifier, so
  `raw_scroll_delta.y` was zero and the photograph panned sideways instead.
- **One named button moves the photograph.** `mouse.drag` ships as the left
  one. Every button used to pan, so whether a right drag opened the menu or
  moved the picture was decided by six points of travel and eight tenths of a
  second, neither of which is on screen anywhere. `any` puts the old behaviour
  back, and the wheel pressed and dragged always pans whatever it says.
- **The double click, the middle button and the two thumb buttons are
  bindable**, to any of about thirty commands or to nothing. Nothing is what
  the middle button ships as.
- **`image_view.scroll_navigation` moved to `mouse.wheel`** and the viewer says
  so on the first launch after the change. `true` becomes "one photograph";
  `false` becomes "move the photograph", which is what the wheel was actually
  doing, rather than "nothing", which would be a dead wheel nobody asked for.

- **Getting from one thing to the thing next to it.** The viewer was full of
  true statements nobody could act on. Clicking a star, a flag or a colour label
  — in the status bar or in the keyword panel — now narrows the folder to the
  photographs carrying it, with the filter bar raised so the change is visible
  and reversible. So does a keyword chip. `Blown 3.4%` and `Crushed 0.2%` paint
  the clipping mask over the photograph, which is what they are describing and
  which used to be a key in a different subsystem.
- **The cheat sheet leads into the keys.** Every row opens the keyboard editor
  with its own binding armed, and the footer opens it with nothing armed. It was
  the best documentation in the program and it led nowhere.
- **A message can be gone back to.** Every failure in the history offers the log;
  a startup key clash offers the keys. A count at the right-hand end of the
  status bar says how many have not been read, so the history is reachable
  without the menu bar being up.
- **One selection, one photograph, one keyword, one burst.** The selection was
  read only in the contact sheet, so opening one of two hundred picked-out
  photographs silently reduced it to one. The metadata panel followed the image
  view while the keyword panel followed the sheet's cursor, so with both open
  they described different photographs. The browsing bar matched a substring of
  the whole hierarchical keyword and the folder jobs matched the stored one, so
  the same word typed in the two places gave two different answers. All four are
  one answer now.
- **A key that cannot act says so.** `Ctrl + T` showed the strip of thumbnails
  and the strip had no height, so on a fresh install the key was advertised in
  two places and changed no pixel. It gives the strip a height and says it did.
- **The shift-capture-time preview shows every field it will change.** It
  computed its "would become" column from the capture time alone, so unticking
  that field while leaving another ticked made every row read an em dash while
  the button still offered to change four hundred files. The button counts what
  the preview shows now.
- **A folder job says what it is about to act on** before it acts, rather than
  leaving it to be inferred from a button label.

- **Right-click the thing itself.** About twenty surfaces answer the second
  button now: the photograph, a cell, the mode indicator, every word of the
  status bar, the position counter, the run a frame belongs to, the marks, the
  file name, the metadata panel and each of its rows, the cache readout, the
  histogram and its two figures, the folder tree, the destination panel. Each
  carries the verbs that were already a few files away, and each names the
  settings page it leads to.

  Every one of them opens on the **press** of the button rather than on its
  release, which is Microsoft's toolbar guidance and which removes the ambiguity
  with a drag at its source instead of tuning a threshold. Each draws the same
  chevron on hover and ends its hover text with the same four words, so a
  surface either says it has a menu or does not have one.

  Nothing is reachable *only* by right-click: the last row of every menu is the
  page that holds the same decision, and `menus.settings_rows` turns those rows
  off without making anything unreachable — which is why there is no menu
  editor.
- **A mode indicator.** `F2` cycles six modes and three of them draw no
  photographs, and the only places the mode was named were the menu bar, the
  cheat sheet's title and the organiser's own heading. One word at the left end
  of the status bar, carrying the six as radios.
- **Four controls move to where the effect is.** How many thumbnails fit across
  is on the filter bar, which is where Lightroom keeps it; the strip's height,
  the keyword panel's width and the metadata panel's width are the edges of
  those panels. All four write the configuration field the settings window
  reads, so the value is still there on the next launch — which is the thing
  none of the viewer's in-view controls used to do.
- **Clicking a mark shows only those.** A star, a flag or a colour label in the
  status bar narrows the folder to the photographs carrying it, with the filter
  bar raised so the change is visible and reversible.
- **`Shift + F10` opens the menu of whatever has the keyboard.** egui cannot
  read the dedicated Menu key at all — its key list runs F1 to F35 — so this is
  the only keyboard route there is. It works while a text field has the cursor,
  which is exactly when somebody needs it.
- **The destination panel is a place destinations can be added.** The tenth and
  later were dropped in silence; they are all listed now, the first nine keep
  their digits and the rest are reached with a click. The ad-hoc "Choose a
  folder…" row offers to keep the folder as a slot.
- **A right click in the folder tree opens a menu** rather than opening the
  folder, which is what it did before — inverted, unsaid, and in the way of
  this.

- **The window says what is wrong, and what has been changed.** A file only
  partly read draws a permanent red bar naming what it costs, with every control
  below it disabled rather than hidden — greying the whole window out would
  force somebody to look on every other page to find out why. A number outside
  what its control can produce is marked where it is drawn and left exactly as
  written. The migration report and the startup key clashes get a permanent home
  in the footer, where they used to have six seconds and no way back. And the
  keyword file has a **Read it now** beside it, reporting "2,114 keywords in 31
  groups" or why it could not be read — which used to be a log line and a panel
  that quietly showed fewer keywords.

- **A settings window.** Forty-one settings could only be reached by editing a
  JSON file, six more were nudged by a key and thrown away on exit, and
  twenty-seven did not exist. All of them now have a control, on one of eleven
  pages named for what somebody is doing rather than for what the field is made
  of: Opening a folder Â· The photograph Â· The contact sheet Â· Stars, flags and
  labels Â· Keywords Â· Moving and deleting Â· Raw files Â· Slideshow Â· Speed and
  memory Â· The window Â· Keys and mouse. `Ctrl + ,` opens it on the page it was
  last left on, and **Settings → All settings…** is the third entry on a menu
  that has had two since it was written.

  Every page is a view over one table, so a row's name, its sentence, its
  control, its range and when it takes effect are all one declaration. Nothing
  is called General, Advanced, Miscellaneous or Other.
- **A search box, with the cursor already in it.** Typing "blurry thumbnails"
  finds the thumbnail resolution and lets you change it there. An exact path
  pasted from a forum post — `cache.ram_budget_mb` — lands on the control and
  nothing else. It never returns nothing: a query where no row matches every
  word is re-run as an "any word" search under a line saying so.
- **Every change is written as it is made.** No OK, no Cancel, no Apply. The one
  qualification is arithmetic: a rail that rebuilt the caches on every frame
  would rebuild them sixty times a second, so a row whose effect is a rebuild
  waits for the gesture to end.
- **A bullet beside every row that differs from its default**, which is a button:
  clicking it puts that one field back. The page list carries a count, so "I
  changed something and I do not remember where" costs one glance rather than
  eleven pages. Reset always carries a scope — this setting, this page, or
  everything — and the wider two say what they would change before changing it.
  The global one copies the file to `config.json.bak` first.
- **What is wrong with the file is drawn across the top of the window**, one row
  per complaint, each with a button that goes to the control. A misspelled screen
  profile, a keyword list that is not there, a rejects folder with no name whose
  key therefore does nothing, a destination with no path, a key name that is not
  a key name. All of those used to reach a log file whose own path the program
  never stated. Nothing is changed by it: a value outside what a control can
  produce is shown, marked, and left exactly as written.
- **Settings can be carried to another machine as a patch.** "Save what I have
  changed…" writes only the fields that differ from their defaults — a small,
  readable, diffable file. Key bindings are opted into and never included by
  default, and machine-specific paths are left out for the same reason.
- **Twenty-seven settings that did not exist.** What a folder opens as: its
  order, its filter, whether its bursts are folded, and whether the filter
  survives opening another folder. What a run of frames is — one set of
  thresholds now, read by the contact sheet's stacking *and* by Group shots,
  which used to hold two that could not even express the same values. What a
  launch starts with: the mode, fullscreen, a folder, and which panels are up.
  Five constants that governed every movement key: the zoom step, the step-zoom
  factor and ceiling, the pan speed, and how many photographs a page is. What is
  drawn under a cell, whether the strip is up and which edge it sits against, and
  whether a single click opens a photograph. What new sidecars are called —
  `photo.cr2.xmp` or `photo.xmp`, which is the difference between Lightroom
  seeing your ratings and not. Which of the reversible things ask first. And the
  theme and the ground behind the photograph.
- **A light interface.** The theme was dark whatever anybody wanted, above a
  reason that is sound and too wide: a light surround does shift how a photograph
  reads, but what surrounds the photograph is the *backdrop*, which is now its own
  field. Changing either takes effect at once — a theme setting that only applied
  at the next launch would be worse than none.
- **Six controls taken away.** A field is a setting only when two reasonable
  people would choose differently. The photographs kept either side is trimmed by
  the RAM budget, so the number in the file was never the number in force; the
  two GPU counts are counts where the bound is bytes; the upload budget is
  computed from the frame time the viewer already measures. Each of the four
  keeps a line on the page that used to hold it, saying where its value comes
  from now — so the answer to "where did that setting go" is on the page.
- **The slideshow window becomes a page.** It drew three of the five slideshow
  fields and omitted the other two.

- **One table the file and the window both read.** The configuration is a
  hundred and eleven fields spread over a dozen structs, which is the right
  shape for JSON and the wrong shape for a person. There is now one row per
  field carrying where it is drawn, what it is called, what it means in a
  sentence, other programs' words for it, its path in the file, what kind of
  control it wants, when a change takes effect, where it is read, and the pair
  of accessors that reach it.

  Almost nothing on screen changes for it. It is the foundation the settings
  window, the search, the changed-from-default marker, the per-field reset and
  the load-time check are all views over, and the keyboard editor is its first
  consumer and the proof it works.

  A test walks `Config::default()` and fails the build if the file carries a key
  the table has never heard of, or the table names a key the file does not
  carry. Another asks it thirty-five questions in the words somebody would
  actually type — "blurry thumbnails", "where do rejects go", "why is my raw
  small", "color class", "text too small", "cr3" — and fails if any of them
  stops landing.
- **The keyboard editor answers ten complaints.** A search box over the name,
  the sentence and the key itself. A reset per row and per section, and the
  global one is named ("Put the 69 key bindings back"), confirmed, and no longer
  walks every row on one unconfirmed click. Delete or Backspace on an armed row
  means "no key", a state the list could already draw and nothing could produce.
  The viewer goes deaf while a row is armed — pressing Delete used to send the
  photograph on screen to the bin *and* fail to capture. A successful change
  says what it did, in a status line that had been declared and read and never
  written. And `cmd` is emitted on macOS rather than folded into `ctrl`.
- **Clashes are decided by where a key is read.** The check compared only
  within a heading, on the sound ground that the gallery and the image view are
  never on screen at once — but "General" is live in *every* mode, so the
  collision it was blind to is the one that bites: Quit on the gallery's scroll
  key means the folder scrolls and the program exits. Every binding now records
  where it is read, and that is what the checker compares.
- **The keys the program reads for itself are in the list.** `?`, `F10`,
  `Escape`, `Home`, `End`, `PageUp`, `PageDown`, `Tab`, `/`, the contact
  sheet's arrows, the tree's six and the destination digits are drawn read-only,
  so the clash checker can see them and a search for "cheat sheet" finds one.
  So is the shortcut on a user action, which was the one shortcut in the file
  the editor could not reach — which is why the shipped example's trash action
  could never fire.
- **`Config::check()`.** What is wrong with a file, said once at load: a screen
  profile that matches nothing, a keyword list that is not there, a blank
  rejects folder whose key therefore does nothing, a destination with no path, a
  tenth destination where there are nine digits, an action with no command, a
  key name that is not a key name, and a number outside what its control can
  produce. Every one of those reached only a log file whose own path the program
  never stated. Nothing is changed by it: an out-of-range value is reported and
  left exactly as written, because hand-editing wins.

## 2026-08-31

- **The right button does something.** On a fresh install, right-clicking a
  photograph used to do nothing at all: the default entry list is empty and the
  menu returned before registering anything when it was. There is now a menu on
  the photograph and on a cell — fit, actual pixels, fill, compare, move to the
  bin, copy the path, copy the picture, show it in the file manager — with
  whatever you configured appended under a separator, in your order, unchanged.

  **Copy the picture** puts the file's own pixels on the clipboard, decoded at
  full size and turned the right way up, on a thread of its own so a sixty
  megapixel raw does not stop the window. The count goes in the label, so a
  selection of twenty-four right-clicked says "Move 24 photographs to the bin".
- **A Help menu, and an About window.** The menu bar was three menus and eleven
  items with no Help. It now carries the keys, the keyboard editor, a legend for
  the marks, the template placeholders, the recent messages, the configuration
  file, the log file, the manual and About — which names the version, the
  graphics adapter, whether this build can develop a raw file, and both file
  paths with a button that copies them. Three of the most confusing behaviours
  in the program are diagnosable from that one window; all three used to reach a
  log file whose own path was written only into that log.
- **Six dead words become doors.** **Flattened**, **Watching**, **Filling**,
  **Advancing**, **Comparing** and **RAW+JPEG** sat in the status bar as bare
  labels with no tooltip and no way to act on them. Each now says what it means
  and carries the verb that turns it off. Two of them carry a setting rather
  than a mode: **Advancing** is the only place `tags.advance_after_marking` is
  visible in the running program, and **RAW+JPEG** the only place
  `raw.pair_with_jpeg` is — and its three sentences, "Show both", "Show the
  JPEG", "Show the raw", had been written since pairing was built and drawn
  nowhere. Changing it re-reads the folder rather than waiting for a restart.
  The marking overlay joins them, so a photograph covered in red says why.
- **A screen that offers a folder.** "No images here" was four words on grey,
  and it is the first thing most people see, because with no argument the
  crawler reads the working directory. It becomes **Open a folder**, **Open
  files**, and the last six folders visited — a list the session file has been
  keeping since positions were remembered and has never shown to anybody. A
  folder emptied by the filter is a different screen: it names the rules that
  emptied it and offers **Show everything**. The tag panel draws a line instead
  of nothing, and the metadata panel says "No photograph open" rather than
  "Loading…", which was a lie that never resolved.
- **A first run that is not a tour.** The menu bar starts visible when there is
  no session file, and thereafter is wherever it was left. One line in the
  corner names `?` and `F1`, and goes as soon as either is pressed. And a notice
  says where the configuration file has just been created.
- **Notices have severities, and a history.** Everything was the same alarm red
  — for "Moved 12 photographs to Selects" and for "Access is denied" alike — and
  there was no history: four lines, six seconds, and the rest dropped without a
  word. There are now three fills, and **Help → Recent messages…** holds the last
  hundred. The band itself stays untouchable on purpose: during a cull something
  is in it after nearly every gesture, and a band that takes the pointer would
  own a strip across the top of the photograph — including its own new menu —
  for six seconds at a time.
- **A focused text field says so.** The whole viewer goes deaf while any field
  holds focus, with `Escape` the only way out, and nothing on screen said any of
  it — so the symptom was a viewer that had stopped answering its keys. One line
  in the corner now says which key brings them back.
- **Undo says what it is about to do.** The sentence has always been built at
  the right moment and shown at the wrong one: it was read before the undo ran
  and reported afterwards. One file still goes back without asking; anything
  more says what it would do and waits.
- **The cheat sheet is reachable and readable.** It was the best documentation
  in the program, behind one key nothing mentioned. It is now in the Help menu,
  it draws each binding's sentence — which has existed on every row all along
  and was read only by the keyboard editor — and it has a search box. With a box
  to type in, "any key closes it" had to go: Escape, a click outside, or any key
  while the box does not hold the cursor.
- **Ninety-five explanations, where there were thirty-three.** Five panels that
  had none — the filmstrip, the folder tree, the navigator, the frame timings
  and the whole sort-and-filter apparatus of three modes — now explain
  themselves, as do every line of the cache readout, the histogram and its two
  figures, and the stack badge on a cell. The three folder jobs each say **this
  cannot be undone**, because the journal does not cover them. Two hover strings
  carrying stray whitespace from broken continuations are fixed, and so is the
  empty string that used to lay out and paint a frame for nothing.
- **Two stack glyphs that were drawing empty boxes.** `◐` is in Hack, which is
  the *monospace* family, and `❏` is in none of the fonts loaded at all, so both
  drew tofu wherever they appeared. They become `◑` and `▣`, which are in the
  proportional chain. Writing the legend is what made it visible.
- **"Stack" stops being one word for three things.** The status bar built its
  line from the function that names a *folder*, so a focus stack read "stack 3 Â·
  frame 4 of 17 Â· stack 3 of 41" — one word for two things, and the same number
  twice. It reads `Focus stack 3 Â· frame 4 of 17 Â· stack 6 of 41` now. The filter
  bar counts stacks rather than runs, and the two frame-standing keys are called
  "Which frame shows the stack" and "Show the next frame instead", because
  nobody searches for "standing".
- **The strip under a cell is a fraction of the cell.** It was a flat twenty
  points whatever the cell measured, so at sixteen columns it was proportionally
  enormous and at one column the stars were a sliver in a wall.
- **The three files people are told to read.** The README gave the configuration
  path as `~/.config/avis-imgv/config.json`, which is right on Linux and wrong on
  Windows and macOS; all three are now listed. `examples/config.json` held 103 of
  the 110 fields with no `version`, so copying it wholesale meant both migration
  steps re-applied — it and `examples/keys.txt` are generated from the defaults
  now, by `cargo run --example write_defaults`.

## 2026-08-31

- **The viewer stops editing files nobody asked it to edit.** Opening the
  slideshow window used to rewrite a hand-written `"seconds_per_image": 900`
  down to 600 on the frame it appeared, with nobody touching anything: egui
  clamps the value it is handed whether or not it was edited, writes the
  clamped number back and then reports a change. Every numeric control in the
  program now says so explicitly.

  Saving is a merge rather than a replacement. A key this build has never heard
  of — one a newer build wrote, one a plugin left — is kept and written back
  where it was, rather than dropped on the way out. And a save that would write
  over a file edited since the viewer read it is refused: the viewer says so and
  offers to read the file again or to keep what is on screen.

  `config.json` and `session.json` are written beside themselves and renamed
  over the original, the way sidecars already were. The keyboard map and the
  per-folder positions are the two things here a person cannot rebuild.
- **Comments in the configuration file cost nothing.** `//` to the end of a
  line and `/* */` are taken out before the document is parsed. One of them used
  to mean the whole file could not be read, which blocked every save for the
  session and handed back the defaults for everything — while the manual said
  the opposite. They are not written back: a save writes JSON.
- **A typo no longer kills the viewer.** `"decode_threads": 1000` was a panic
  with no message; the pool is capped at 64 between the file and the spawn loop,
  and the file keeps what it says. `"text_scaling": 0.0` multiplied every text
  style to nothing, including the menu bar that would have let anybody undo it;
  it is floored at half size and capped at three times, and applying it twice no
  longer compounds.
- **Two gestures that were spoken for.** Dragging the photograph with the right
  button used to pan it and then release into whatever menu was registered;
  panning is the left button now. In the folder tree a *right* click opened the
  folder while a left click expanded it, which is both inverted and unsaid: a
  left click moves the highlight, a double click opens, and the right button is
  left free.
- **A "To Do" label draws purple.** It was listed against red as well, and the
  first match wins, so a frame Bridge had labelled "To Do" drew red here and
  purple was unreachable. Red keeps the name Bridge actually gives it, "Select".
- **`restore_session` turned off now means the window is not restored.** It
  only ever decided whether the geometry was *recorded*, so turning it off
  stopped the window being remembered and did not stop it being used.

- **Stacks, on `Ctrl + G`.** A folder shot properly is mostly repetition: five
  frames of one expression, three exposures of one view, a hundred from a
  camera on a timer. A contact sheet that shows all of them shows the same
  photograph five times over. Stacked, thirteen frames become six cells — each
  with a count and a glyph for what kind of run it is — and the six are six
  different photographs.

  Nothing is written. Lightroom keeps its stacks in a catalogue and Bridge in a
  hidden file beside the pictures; these are worked out from what the files
  already say, every time, so turning them off leaves nothing behind. `E` opens
  the run under the cursor, `,` and `.` walk a folded one without opening it,
  `Ctrl` and the arrows step over a burst rather than through it, and the
  status bar says `series 2 Â· frame 4 of 17 Â· stack 6 of 41` the whole time.

  The frame standing for a folded run is the sharpest one that could be
  measured, which is the question a burst is usually asking. The filter bar
  carries the rest: how many runs were found, fold and open all, the longest
  pause that is still one run, and a slider for how alike two frames have to be
  — a judgement rather than a number, so it is dragged and watched.

  Built on the same list of positions the filter narrows, so stacking composes
  with filtering and ordering and nothing is decoded twice for it. A rule that
  hides the frame standing for a run leaves the run standing on the next frame
  that survived, rather than taking the whole burst out of the folder.
- **Keywords with levels.** A tag written `Places|Slovakia|Tatras` is filed
  under its levels rather than flattened: the path goes into
  `lr:hierarchicalSubject`, which is where Lightroom, darktable, digiKam,
  Bridge and exiftool all look, and the keyword itself still goes into
  `dc:subject` so a program that has never heard of hierarchies finds it
  anyway. Writing only the path would leave that second kind of program seeing
  an untagged photograph.

  The panel draws them as a tree instead of a wrapped row of chips — forty
  keywords three levels deep is a wall of words in which the same leaf appears
  under two parents with nothing to tell them apart. A keyword on the image
  shows its own name with the path on hover, taking a keyword off takes the
  paths that end in it with it, and the shortcut that toggles one recognises it
  by its name however it is filed.

  Narrowing by `Slovakia` now finds everything below it, which is most of the
  reason to have levels at all.
- **A keyword list read from a text file**, on `tags.catalog_file`. Every photo
  application can export its keywords as an indented list, and a photographer
  with years of them in Lightroom or digiKam should not have to type them again
  into a JSON file. Indentation makes the hierarchy, tabs or spaces; a line
  with bars in it is taken as a path as it stands, so a flat export reads as
  well as an indented one; `#` starts a note. The outermost level becomes a
  category in the panel, a relative name is taken against the configuration
  file, and a list that cannot be read is a warning in the log rather than a
  refusal to start.

## 2026-08-30

- **A filmstrip under the photograph**, on `Ctrl + T` and
  `grid_view.filmstrip_height`. The contact sheet answers "what is in this
  folder" and the image view answers "what is this frame", and neither answers
  "what is either side of this one" — which is the question culling is made
  of. It draws from the contact sheet's own store, whose textures are resident
  whichever view is on screen, so it costs nothing but the drawing. It follows
  what is on show, so a filtered collection has a filtered strip, keeps the
  current frame in view, and clicking one goes to it.
- **Marks over the picture, on `C`.** First what has clipped — red where a
  highlight is past saving, blue where a shadow has gone, by the same rule the
  histogram counts by so the picture and the number beside it cannot disagree.
  Then focus peaking, which marks the edges that are actually sharp: at
  anything under 100% a slightly missed focus looks exactly like a hit one,
  which is why people zoom into every frame of a burst.

  Drawn through the photograph's own texture coordinates, so a mask follows
  the zoom and the pan for nothing and a quarter turn turns it too, and built
  only while the overlay is on rather than on every decode.
- **A histogram, and how much of the frame has gone.** Three channels and a
  brightness fill in the side panel, with the two numbers a screen cannot
  show: what proportion of the photograph has clipped at each end. A monitor
  renders 250 and 255 as the same white, so "is that sky recoverable" is a
  question the picture itself cannot answer.

  Counted on the decode workers, which already touch every pixel of every
  photograph in the folder — so it costs about three per cent of throughput
  and it is known for the whole folder rather than for the frame on screen.
  Getting there took fixed-point luminance: the same sum in floating point,
  with a round, cost fifteen per cent.
- **A sharpness score, and the sharpest frame of a burst marked.** Choosing
  between five frames of the same thing is mostly one question — which is in
  focus — and it is the question a contact sheet is worst at answering, because
  at thumbnail size everything looks acceptable. Measured on the folder scan's
  worker from the thumbnail it already decodes, so it costs nothing extra, and
  shown beside each frame with the sharpest of a group in bold. It is also a
  sort key.

  Deliberately never used to decide anything: a photograph of a wall outscores
  a portrait at f/1.4 and the portrait is the keeper. What a number does well
  is rank *frames of the same scene*, where the only thing that differs is the
  focus. A file with no thumbnail to measure has no score rather than a zero,
  which would sort it in among the blurred ones.
- **What the photograph says about itself, on the photograph.** `O` moves it
  round the corners and off again. The status bar has the same information and
  is in the wrong place for it: a viewer running fullscreen for a slideshow or
  a review has no chrome at all, and the eye is on the picture. Over the drawn
  rectangle rather than the panel, so a letterboxed photograph gets its caption
  on the photograph.
- **The line under each thumbnail is a template too.** It was the file name and
  nothing else; it is `grid_view.caption_format` now, so a sheet can be
  labelled by shutter speed while somebody is looking for the frame that was
  not blurred. It falls back to the name while the scan has not reached a file,
  rather than showing an empty strip.
- **One template grammar instead of two.** The status bar took
  `$( • Æ#Aperture#)` and the bulk rename took `{date}_{n}`, each with a
  vocabulary the other could not reach — so a rename could not put a
  photograph's ISO in a name without a differently spelled placeholder, and
  the status bar could not say a capture date at all. There is one now, it
  understands both spellings so every template anybody has written still means
  what it meant, and it knows a good deal more: the exposure, the lens, the
  camera, the size, the marks, and every part of the capture time.
- **CI on Windows and macOS as well as Linux, and on every branch.** A viewer
  is not much use if it only builds where it was written, and the two things
  that differ across platforms — path handling and the bin — are exactly what
  the file operations are made of. Clippy also runs over
  `--no-default-features` and `--features libraw`, which found a real warning
  the moment it was added: without the bundled font, three imports in the
  theme were unused.
- **A golden-file test for the XMP writer.** A round trip cannot see the shape
  of a document change — rename a namespace prefix and this viewer still reads
  its own output perfectly while Lightroom stops seeing the rating. Five
  recorded sidecars now hold the writer to what it agreed, and each is also
  read back to check the record is of something correct rather than only
  something stable.
- **It opens where you left off.** The window's size and place, the folder
  that was open, and — the one that earns its keep — which photograph was
  being looked at in each of the last sixty-four folders. Culling a shoot is
  rarely one sitting, and starting somebody at the first frame again throws
  away the only piece of state that took any effort to build. A path named on
  the command line always wins; `general.restore_session` turns it off.
- **The keys, on `?`.** Generated from the same table the key editor writes,
  so it shows the keys that are actually bound rather than the ones the
  documentation remembers, and narrowed to what is on screen — the image
  view's keys are no use while looking at a contact sheet. Any key closes it.
- **The log goes to a file** beside the configuration, and a panic is written
  into it with its backtrace before the process ends. A viewer started from a
  desktop icon has no terminal, so everything it had to say went to a standard
  error nobody would ever see, and a crash left nothing at all — which is the
  one moment there is something worth reading. It is still written to the
  terminal as well when there is one, and started again once it passes a
  megabyte.
- **A full resolution copy is only decoded for a photograph that was
  reduced, and only once its ordinary decode has said so.** It used to be
  asked for speculatively, for every photograph within reach, before the
  preload window had been queued — so a folder of large JPEGs was decoded
  twice over and the second, expensive decode competed with the one browsing
  was waiting on. Throughput on a folder of 4624ÃÂ2600 JPEGs went from 37
  images a second to 51, and the slowest frame from 54ms to 14ms. A
  photograph smaller than the display cap now costs one decode rather than
  two, because its ordinary copy already is its own pixels.
- **The GPU cache is bounded by bytes, not only by texture count.** A count is
  not a memory bound: two hundred thumbnails and two hundred sixty-megapixel
  photographs are the same number. `cache.gpu_budget_mb` sets the ceiling, and
  the upload loop stops when it is reached rather than evicting a neighbour to
  make room and wanting it back next frame.
- **The metadata read ahead of the decoders is bounded too.** Every file the
  preview reader touched left its tags behind and nothing ever took one out,
  entirely outside the budget the rest of the cache is held to — a folder of
  ten thousand photographs built ten thousand of them and kept them all.
- **The memory readout says what is actually held.** It counted the decoded
  pixels in RAM and called that the total, while the textures on the adapter —
  the same pixels again, plus a third for the mip chain — the thumbnails
  standing in for them, and the metadata all went unmentioned. It now shows
  each tier against its ceiling and what they add up to: on a folder where the
  old figure was 2323 MiB, the honest one is 2494 MiB against a process
  holding 2670 MB.
- **Four kinds of work that every frame was doing again.** None of them shows
  up in the benchmark, which moves to a new photograph every frame and never
  opens a panel — they are what the viewer does while somebody is looking at
  something rather than racing past it.
  - The four preload windows each store computes are kept between frames and
    rebuilt only when the cursor, the collection or the radius has moved. The
    duplicate check inside them, which scanned everything collected so far on
    every step and so squared the radius, is replaced by the arithmetic that
    says where the one duplicate can be: where the window reaches half way
    round and meets itself.
  - The folder watcher copied every path in the folder on every frame to
    decide what a change was about, before finding out there were no changes.
    It looks at the events first and builds an index only when there are any.
  - The keyword list the tag panel offers — a walk over every entry in the
    folder, sorted and deduplicated — is rebuilt when the annotations change
    rather than on every frame the panel is open.
  - The rename plan and the table under it — a new name for every file in the
    folder, and two strings per row — are worked out when the selection or the
    template changes, which is when they can differ.
- **A raw and a JPEG shot together are one photograph.** A camera set to
  raw+JPEG writes two files of the same frame; browsing both means walking the
  shoot twice, rating everything twice, and letting the two copies disagree —
  reject the JPEG, keep the raw, and what survives the cull is the opposite of
  what was decided.

  One of them is browsed and the other follows it, through every rating, flag,
  colour label, keyword, move, copy and deletion; each keeps its own sidecar.
  `raw.pair_with_jpeg` decides which is browsed, or turns pairing off. The
  status bar says `RAW+JPEG` when the photograph on screen is a pair, and the
  delete question counts photographs rather than files. The folder jobs still
  see every file, because a rename that renamed only half of each pair would
  break the pairing it depends on.
- Fixed: a configuration file with a byte order mark in front of it — which is
  what Notepad's "UTF-8" writes — parsed as nothing at all and silently handed
  back the defaults for everything.
- EXIF text fields are read as UTF-8 first and fall back to Latin-1, rather
  than always the latter. Cameras write Latin-1 and Adobe's software writes
  UTF-8; pure ASCII reads the same either way, and Latin-1 with an accent in
  it is almost never valid UTF-8, so nothing that used to be readable stops
  being so.
- **The configuration file has a version, and is brought forward.** A default
  that moves is the one change `serde` cannot absorb: it fills in the keys a
  file is missing, never the ones that have since moved, so an older file
  keeps the old binding for ever and two commands end up on one key.

  Two such bindings are put right, both found on a real configuration: `Space`
  on the contact sheet, which is now "pick this one out" and was "scroll down
  half a row"; and `Plus`/`Minus`, which were both "more images side by side"
  and "zoom in" — zoom won, so the side-by-side view was simply unreachable,
  which is what the startup clash warning had been reporting without being
  able to do anything about it.

  A step only ever touches a setting that still holds the *old default*, so a
  binding anybody has actually chosen — including choosing the old one back —
  is left alone. What was moved is said in the corner rather than done
  quietly, and a file that was only partly understood is migrated in memory
  and left alone on disk.
- **A Fujifilm raw's clock can be shifted.** A RAF keeps its EXIF inside the
  JPEG it embeds, and only one caller knew that — so the capture-time shift,
  which asks the container directly, found no timestamps in a RAF at all and
  silently declined to move a Fuji shoot's clock. The container unpacks it
  now, so every caller gets it, and a RAF also picks up the thumbnail inside
  that JPEG, which puts it on the contact sheet as fast as a JPEG.
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
