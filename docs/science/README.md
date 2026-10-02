# Deep Time: the science documentation

This folder says what Deep Time simulates, which published work each part rests on, and how far
each number in the code has been checked against its source. It is written for curious players,
modpack makers and geologists. It tries to be exact where it matters and plain everywhere else.

**Read this first: what has and has not been checked.** Deep Time was written by AI agents
directed by one person who is not a geologist. Many constants in the code were written by an agent
from what it remembered of a paper, without the paper open. Stage 1 of this documentation
(2026-10-02) lists every such claim in a ledger and marks honestly where it came from. It verified
almost nothing. Stage 2 checks each claim against its source and corrects the code. Until a row
says `verified`, treat its value as unconfirmed.

## What the mod simulates

Deep Time makes a Minecraft world by simulating the history of a small planet, from a starting
arrangement of continents through hundreds of millions of years, and then turning the result into
blocks. Plates move, collide and sink. Climate, rivers and ice wear the land down and lay the debris
as layers of sediment. Volcanoes build, meteors strike, rock is buried, heated and folded, and ore
forms where the history made the conditions for it. The world you play in is the last frame of
that history: its mountains stand where plates collided, its coal lies where swamps were buried,
and the rock under your feet is in the order it was laid down.

It is a game model, not a research model. Each process is the simplest published form that gives
the right kind of landscape in minutes of computing, and several things are deliberately not
physical so that the world is playable. The pages say which.

## The pipeline

The simulation runs in stages. The first group runs together, step by step, through the whole
history; the rest run once at the end.

| Stage | What it does | Page |
|---|---|---|
| Plates | Moves the plates; makes and destroys ocean floor; raises mountains at collisions and arcs above subduction zones; places hotspots. | [plates.md](plates.md) |
| Climate | Temperature, rain, ice and sea level for the geography of the time; a carbon cycle and orbital cycles over deep time. | [climate.md](climate.md) |
| Surface | Rivers erode, hillsides creep, sediment is carried and laid down as layers; the crust rises and sinks under the changing load. | [erosion-and-sediment.md](erosion-and-sediment.md), [isostasy-and-height-limit.md](isostasy-and-height-limit.md), [flexure.md](flexure.md) |
| Volcanoes | Volcanoes are born at arcs and hotspots, grow, die and erode; flood basalts cover provinces. | [volcanoes-and-magma.md](volcanoes-and-magma.md) |
| Impacts | Meteors strike at Earth's rate; craters form, age and are buried. | [impacts.md](impacts.md) |
| Structure | Turns the recorded history into rock columns: layers, metamorphism, granite bodies, folds and faults. | [rock-and-metamorphism.md](rock-and-metamorphism.md), [strata.md](strata.md), [folds-faults-intrusions.md](folds-faults-intrusions.md) |
| Deposits | Places ore, coal and oil where the history made them, in numbers and sizes from USGS models. | [deposits.md](deposits.md) |
| Young landscape | Reruns the last stretch of erosion on a finer grid; carves glacial valleys. | [young-landscape-and-amplifier.md](young-landscape-and-amplifier.md), [glacial-landforms.md](glacial-landforms.md) |
| Amplify | Adds block-scale landforms to the kilometre-scale terrain. | [young-landscape-and-amplifier.md](young-landscape-and-amplifier.md) |
| Bake | Converts metres to blocks; works out rivers and lakes, climate classes and biomes; writes the world data. | [rivers-and-lakes.md](rivers-and-lakes.md), [biomes.md](biomes.md), [world-scale-and-bake.md](world-scale-and-bake.md) |
| The game | The Minecraft side reads the world data and builds chunks from it: terrain, strata, ore, water, ice, magma. | every page's "What you see in the world" |

## Feature pages

Two pages are written in full as the pattern for the rest. The others are stubs: a paragraph, the
ledger ids that belong there, and what is already known to be wrong or unchecked. Stage 2 fills
them in from [_template.md](_template.md).

| Page | State | Ledgers |
|---|---|---|
| [impacts.md](impacts.md) | written (stage 1) | IMP |
| [volcanoes-and-magma.md](volcanoes-and-magma.md) | checked against sources (stage 2, WP-2, 2026-10-02) | VOL, MAG |
| [plates.md](plates.md) | stub; model about to be rewritten | PLT |
| [climate.md](climate.md) | stub; model about to be rewritten | CLM, MOI |
| [biomes.md](biomes.md) | stub | BIO |
| [erosion-and-sediment.md](erosion-and-sediment.md) | stub | ERO |
| [isostasy-and-height-limit.md](isostasy-and-height-limit.md) | stub | ISO |
| [flexure.md](flexure.md) | written with the feature (2026-10-02) | ISO-040 to ISO-069 |
| [hypsometry-and-calibration.md](hypsometry-and-calibration.md) | stub | HYP |
| [rivers-and-lakes.md](rivers-and-lakes.md) | stub | HYD |
| [young-landscape-and-amplifier.md](young-landscape-and-amplifier.md) | stub | YNG, AMP |
| [glacial-landforms.md](glacial-landforms.md) | stub | GLA |
| [rock-and-metamorphism.md](rock-and-metamorphism.md) | stub | RCK |
| [strata.md](strata.md) | stub | STR |
| [folds-faults-intrusions.md](folds-faults-intrusions.md) | stub | DEF |
| [deposits.md](deposits.md) | stub | DEP, ORE |
| [provinces.md](provinces.md) | written with P1-10 (2026-10-02) | PRV |
| [world-scale-and-bake.md](world-scale-and-bake.md) | stub | SCL, BAK |
| [pole-cap.md](pole-cap.md) | stub | POL |
| [geothermal-features.md](geothermal-features.md) | written (stage 2, WP-16) | GEO |

Four ledgers have no page of their own; their rows belong with the pages of the subsystems they touch
(added in stage 2, package WP-0): PRS (presets other than `earthlike_quick_16k`, the test presets and the
in-game world presets), HNT (the editor's hint texts), CRE (constants in the plate geometry, noise, plate
state and deformation lattice code) and CHK (the `plate-check` target ranges).

Other files here:

- [claims.md](claims.md): the ledger. Every physical constant, law, threshold, rate and
  calibration target in the code, with its source and status. Its rows are in `ledger/`, one file per
  subsystem. It ends with the verification plan for stage 2.
- [bibliography.md](bibliography.md): every source the code relies on, one entry per work, with its
  full reference, how it has been read so far, and whether the project's source library (a folder of
  open copies kept off the repository, never distributed) holds a readable copy.
  [bibliography-research-notes.md](bibliography-research-notes.md) lists the further sources the
  research notes and designs cite; mentions that are the same work as a bibliography entry carry
  `[= Key]`, the rest are marked `[unresolved]`.
- [spot-checks.md](spot-checks.md): what stage 1 found when it tried to read 403 values in their
  sources (176 matched, 134 in part, 19 did not, 74 could not be read). Leads for stage 2, not
  verification.
- `extraction-notes/`: working notes from building the ledger: measured comparisons with Earth
  found in the build log, and flags where a comment, the code and a document disagree.
- [`../research/README.md`](../research/README.md): index of the research notes and designs.

## How to read the status marks

Every ledger row carries exactly one of these.

| Mark | Meaning |
|---|---|
| `verified` | The source was fetched and the value read in it. The row says what was read and where. |
| `cited-unverified` | The code or a document names a source. Nothing records that anyone read the value in it. |
| `from-memory` | An agent wrote the value from what it remembered and said so, or said it could not check. |
| `abstract-only` | Only the abstract or a secondary summary of the source was read. |
| `calibrated` | The value was tuned until a run matched a target. The row names the target and where the target's Earth figure comes from. |
| `game-choice` | Deliberately not physical, for playability or looks. The row says why and who decided: Ben, with the date in the decisions log, or an agent, with the build-log entry. |
| `uncited` | No source is given anywhere. |
| `merged` | Retired in stage 2 as a duplicate of another row, which now owns the claim. The row says which; it is not counted. |

Two further marks can follow the status:

- `pending-rewrite`: the row describes code that is about to be replaced (today's plate motion
  and today's banded climate). Stage 2 does not verify these.
- `not-yet-coded`: the constant exists only in an approved design.

A `game-choice` is not a fault. A model of a planet at 300 m per block needs many. The fault would
be to present one as physics.

## Standing limits

These hold for every page and are not repeated on each.

- **Scale.** One block is 300 m horizontally and 20 m vertically. The default planet is 16,384
  blocks round: about 4,915 km, an eighth of Earth's circumference and about 1.5 % of its area.
  Heights are therefore stretched 15 times against widths. A real 2 km crater bowl would be 7
  blocks across and 21 deep. Slopes, river widths, layer thicknesses and ore bodies are adjusted so
  they can be seen and used; each adjustment is a `game-choice` row.
- **A small planet at Earth's rates.** Rates per area (impacts, deposits) are Earth's, applied to a
  planet with 1.5 % of Earth's surface. Lengths in the plate model are scaled to the planet; lengths
  of landforms (a volcano's radius, a crater's diameter) are mostly not. Pages say which is which.
- **Time.** Histories run for hundreds of millions of years in steps of about 2 Myr. Nothing
  faster than a step is resolved: a volcano here stands for a volcanic centre that rebuilds over
  millions of years, not one eruption.
- **Runs are not bit-reproducible.** The same seed is not required to give the same world on
  different machines (Ben, decisions log 2026-10-01: no reproducibility requirement).
  Measured numbers on these pages name the run they came from and will differ a little on a rerun.
- **Measurements come from few runs.** Most "Earth against the mod" numbers are from one or a few
  runs of one preset.
- **No geologist has reviewed this.** Nobody on the project is a geologist. Errors of
  understanding, not only of numbers, are likely. Review by geologists is invited: the ledger is
  built so that a reader can check one row at a time and say "this is wrong".
- **Two models are about to change.** Plate motion and climate are being rewritten
  (`docs/design/plate-force-balance.md`, `docs/design/p1-26-per-cell-climate.md`).

## The rule for contributors

A feature is not done until its page here, its ledger rows and the bibliography are updated. Every
constant in code carries a ledger id or a `game-choice` note. A value that could not be checked
against its source is marked `from-memory`, never cited as if it had been read. (`CLAUDE.md`,
"Science documentation is part of done"; Ben, decisions log 2026-10-02.)
