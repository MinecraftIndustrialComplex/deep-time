# Create World UX (P1-11c)

Status: **built** on branch worktree-agent-a319724fce0152057 (2026-10-20). Ben, 2026-10-19: "The UX
for world creation fucking sucks, it should default to deeptime, you click create, then it should
have a fancy guide for helping players do stuff. (Though honestly sensible defaults is probably
best.)", "The world gen UI / UX also feels like shit." and "Keep in mind that we should also have a
much better customization screen for any power users who want to do it." Decisions log 2026-10-19
(one-click Create World).

The plan's page model (§7.12: Setup, Run, Review) stays. What changes is how a player gets there, what
the run page says, and how the setup page is organised. The dedicated server's first start (P1-13)
is unchanged.

## 1. The flow

1. **Singleplayer, Create New World** opens Create World on **Deep Time** (`deeptime:planet`).
   The other world types stay in the World tab. The client setting `defaultWorldType` (on by
   default, `deeptime-client.toml`) turns this off. Without the natives (an unsupported platform) the
   game's own default stays, so Create keeps working.
2. **Create** on a Deep Time world that has no world data opens the **generation page**, and the
   history starts at once with the sensible defaults: the `earthlike_quick` preset at 16k, a
   supercontinent start, 1,000 Myr, a 4 GiB cap. There is no setup page in the way.
3. When the history is done, **Play** binds the finished run and creates the world exactly as the
   game's Create button does. Esc returns to Create World with the run still going, and Create opens
   it again. Cancel (the left button while it runs) stops the run first.
4. **Customize** (World tab, or "Customize…" on the generation page) opens the Customize page
   (section 3). Its Generate starts the run and goes to the generation page. From the generation page,
   Customize comes back with "Back to the run".

Guidance stays light. The first-run guidance panel appears only when the natives are missing (it says
why and what to do). Every button has a one-line tooltip. The status sentence says what is happening
and what Play does.

### How the default is set

NeoForge has no hook for the default world type, so `mixin.client.CreateWorldScreenMixin`
(the client section of `deeptime.mixins.json`) does two things:

- `@ModifyArg` on `CreateWorldScreen.openFresh`'s `Minecraft.setScreen` call hands the new screen
  to `CreateWorldFlow.fresh`, which selects `deeptime:planet` the way the World tab's button does
  (`WorldCreationUiState.setWorldType`). `createFromExisting` (Re-Create) is not touched: it keeps the
  world's own type.
- `@Inject(HEAD, cancellable)` on `onCreate` (the Create button and Enter) asks
  `CreateWorldFlow.interceptCreate`. `CreateWorldGuard` returns one of four results:
  - NONE: not Deep Time, or already simulated. The game goes on.
  - NATIVES: cancelled. The button already says why.
  - NOT_SIMULATED or STAGING_MISSING: cancelled, and the generation page opens.

Play calls the real `onCreate` through `CreateWorldScreenInvoker` after the run is bound, so the
guard returns NONE and the game creates the world.

## 2. The generation page

`screen.GenerationPage` replaces RunPage and EventTicker. The map stays the hero. Text uses the
game's own font, buttons and panels (Minecraft's dark list background), sized for GUI scale 2 and 3.

From top to bottom:

- **Title**:
  - "Creating your planet" while it runs;
  - "Your planet is ready" when it is done;
  - "Simulation stopped" or "Your planet could not be simulated" otherwise.
- **One status sentence** in plain words (`model.GenerationStatus`): "Moving the continents, 640
  million years ago", "Writing your world…", "Your planet's history is complete. Press Play to
  explore it."
- **A real progress bar** with the time left, "expected to take 9 min" before it starts and "took 8
  min" after it ends. The bar and the time left follow the estimate's stage seconds (`model.Estimate`:
  the coupled plates group is about three quarters of a run, not one stage in nine), paced by the
  run's own speed so far, and count the bake (writing the world, about 0.8 of amplify's time) after
  the stages; the session's own equal-weight total is used only when no estimate matches the run.
- **One time convention everywhere** (`model.Phrase`): "N million years ago" (billion and thousand
  where they read better). The ticker's "t = 640 Myr" and "-360 Ma" are gone.
- **The map**, with the time scrubber, play/pause and "● Newest map" under it. The status line says
  where the run is; the scrubber says what the map shows ("Map: 250 million years ago"). The map is a
  keyframe, so while the run goes it can be up to one keyframe behind the status line, and the
  scrubber's tooltip says so. "Stop here" and "Pick this epoch" are optional and stay beside the
  scrubber. Where the panel sits beside the map (a wide window), the map spans its column at the
  projection's π:1, top-aligned, and the band under the time row shows the layer's key (the same
  `model.Legend` as the Map tab, cut with "…" when it does not fit).
- **A panel with three tabs**:
  - **History**: the event log grouped and counted (`model.Milestones`). It is cut into time
    windows (a fiftieth of the run, at least 1 Myr), with one row per kind per window ("8 new
    ocean ridges formed"), newest first (by window, then by the row's newest event), so the time
    column reads in order. Each row has its kind's colour, the same as the map's markers, so the
    list is also the markers' legend. Event kinds from the Rust core are mapped to player text on
    the Java side (`deeptime.milestone.<kind>`, `.many`); no Rust event was renamed. A row's text
    wraps to two lines (`model.TextWrap`), and every phrase fits two lines of the narrowest column
    (100 px at 1920x1080, GUI scale auto) with a three-digit count (TextWrapTest).
  - **Map**: layer groups and layers, "Show as a globe" and "Show events on the map", and the shown
    layer's **legend** (`model.Legend`). The legend is read from the same `ramps.json` the picture is
    coloured with:
    - classes with their headings;
    - named codes;
    - a colour bar with values in the layer's unit.
  - **More**: "Keep going" (more history past today), "Final map and spawn" (the review page), and
    **Show details**. The review page follows the same layout rule as this page (the summary beside
    the map, or under it in two columns on a tall window) and names ore provinces with the deposit
    layers' words ("MVT Zn-Pb").
- **Show details** (also F3) shows the developer's line under the bar: stage percentages and native
  memory. It is off by default.
- **One bar of actions**:
  - Back (or Cancel while running) on the left;
  - Customize… beside it;
  - **Play**, the primary action, on the right.

  Play is disabled with a tooltip ("Play is ready once the simulation finishes") until the run is
  done.

On a tall window (2000x1500) the map spans the width and the panel sits below it, with History and
Map side by side. On a wide one (1920x1080) the panel sits beside the map. Nothing is cut off at GUI
scale 3. Text that can run long wraps to two lines; what still does not fit ends in "…" at a word
and is shown whole in its tooltip.

## 3. Customize (power users)

`screen.CustomizePage` replaces SetupPage. It opens only through Customize. It edits **the same
parameter set as the browser workbench**: `EditorFields` reads
`crates/deeptime-viewer/assets/editor_fields.json`, the file the workbench builds its form from (281
settings in 18 sections today).

### Topics

`model.CustomizeTopics` sorts the catalogue into six tabs. Inside a tab the everyday sections come
first and the fine-tuning (`adv_*`) sections after them, each under its own heading. The tab's own
description (its button's tooltip) sits above the first heading.

| Tab | Catalogue sections |
|---|---|
| Planet | the world's size, start and history length; planet, scale |
| Plates | adv_plates (plates and continents) |
| Climate | biomes, playability, adv_climate, adv_biomes |
| Surface | terrain, adv_surface, adv_water, adv_volcanoes, adv_impacts (surface and rivers) |
| Rocks | multipliers, essentials (moved from playability), adv_rocks, adv_deposits, adv_underground, adv_geothermal (rocks and deposits) |
| Advanced | the memory cap, fantasy (Phase 3, shown disabled), search over every setting |

### Rows

Every setting is one row:

- its label;
- on the next two lines, what it does: as many whole sentences of the catalogue's hint as fit
  (`TextWrap.summary`), else its first sentence cut at a word with "…" (before an open bracket when
  the cut would fall inside one); the whole hint is in the tooltip anywhere on the row but the control;
- its unit and range, right-aligned at the right of the first of those lines, the description
  running up to it ("0.1 to 99.9 m/s²", "on or off", "3 choices", "0 or more" for the seed);
- its control:
  - a number box, red while the text is not valid;
  - an on/off button;
  - a choice button;
- a **Reset** button back to the preset's value, shown only when the setting is changed.

A changed setting has an accent bar and an accent label. The tooltip adds "Preset: <value>". A
setting the chosen preset does not carry says so ("Not in this preset: start from another preset to
change it."), and the Phase 3 placeholders say when they arrive.

### Presets, estimate and share codes

- **Start from** cycles the preset families (plus an imported one). It sits above the list, with
  "N changes from the preset" beside it. Changes are kept when the preset is switched. **Reset all**
  puts every setting back to the preset's value, and the timeline and cap back to their defaults.
  `EditorModel.edited` counts a setting as changed when its value differs from the preset's, or when
  its text is not valid.
- **A live estimate** sits above the buttons: "8 min to simulate, up to 3.1 GiB of memory (cap 4
  GiB)". It updates on every change. Its status line shows, in order:
  - a blocking problem in red (over the cap, an invalid value);
  - "A long run: 42 min on this computer" when the estimate is over 30 minutes
    (`CustomizeTopics.SLOW_SECONDS`);
  - else the first warning or "Ready".
- **Generate** is disabled while a blocking problem stands.
- **Share codes** both ways:
  - Copy code puts a `DT1.` code of the effective settings on the clipboard (the workbench reads the
    same codes);
  - Paste code reads one from the clipboard and starts from it.

### Search

Raw preset paths appear only under Advanced. The search box there matches every word the player
types, in any case, against a setting's label, id, preset path and hint. It lists up to 80 matches
in catalogue order, each with its path on the second line, so a setting can be found by what it does
("river") or by its path ("hydro.enabled").

## 4. Checks

- JUnit, plain Java (no Minecraft types in `screen.model`):
  - PhraseTest: the time convention;
  - MilestonesTest: windows, grouping and counts;
  - LegendTest: every ramp has a legend, units;
  - CustomizeTopicsTest: every setting in exactly one tab, everyday before fine-tuning, search,
    ranges, the slow warning;
  - EditorModelTest: the one-click defaults, generate, changes and resets;
  - TimelapseModelTest: the bar and the time left against a real run's stage timings, the bake;
  - TextWrapTest: wrapping and cutting, every History phrase in two lines of the narrowest column,
    every setting's summary in its two lines (with a model of the font's widths that is never
    narrower than the real one).
- ScreenSourcesTest: every language key the screens name exists, including the run-time keys (event
  kinds, stages, legend codes, tabs, headings), and no en_us value has an em dash.
- The client walk (`tools/client/vm-client.sh`, `ScreenshotDriver`) shoots each step on the test VM:
  - Create World on Deep Time;
  - the generation page while running and when done;
  - the Map and More tabs;
  - Customize (Planet, Climate, Rocks, search);
  - Play into the world.

  It takes `--gui-scale` and `--size`, and `--mods mic` for the MIC jar set.
