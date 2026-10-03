# Research notes and designs: index

Every research note and design document of Deep Time, one line each, with the date the document
states for itself. The science documentation built from them is in [`../science/`](../science/README.md);
the sources they cite are listed in [`../science/bibliography-research-notes.md`](../science/bibliography-research-notes.md).

How to read this index:

- **Authority.** Ben's decisions (`../deeptime-design-decisions.md`) win over the plan
  (`../deeptime-implementation-plan.md`), which wins over the research report, which wins over the
  notes. The plan's §1.2 lists the report's positions that were later overturned; the "Still holds?"
  column below says where a document was overtaken.
- **Notes are dated snapshots.** They record what was found on the day, and say in their own "Gaps"
  or "Sources" sections what could not be read. They are not corrected afterwards.
- **"Session" notes** (2026-10-01 and 2026-10-02) record assessments the coordinating session gave in
  conversation. Each says what was sourced and what was the model's own unchecked knowledge.

## The research report

| Document | Date | What it answers | Still holds? |
|---|---|---|---|
| [`geologic-history-worldgen-report.md`](geologic-history-worldgen-report.md) | 2026-09-27 (no date line; its counts are dated) | Can a geologic-history simulation run inside a NeoForge 1.21.1 world on published models, and what should be built first? | The choice of models stands (Cortial-style plates, Noddy-style structure, implicit stream-power erosion, a mineral-systems ore engine). Overturned: pure-Java engine, 8 m per block, MIT licence, bordered map, bit-for-bit determinism and others (plan §1.2). |

## Research notes (`notes/`)

| Document | Date | What it answers | Still holds? |
|---|---|---|---|
| [`tectonics_engines.md`](notes/tectonics_engines.md) | 2026-09-27 | Which plate-tectonics and 3-D geology tools exist, and what a mod can embed, port or must leave offline. | Tool survey stands. "Port to Java" became Rust; the plate driver it recommends is being replaced by a force balance (decisions log 2026-10-02). |
| [`surface_processes_climate.md`](notes/surface_processes_climate.md) | 2026-09-27 | Which erosion, weathering, stratigraphy, hydrology and climate methods fit a game's time budget. | Algorithm list stands. The banded climate it recommends is being replaced by a per-cell model (P1-26). |
| [`ore_genesis_modeling.md`](notes/ore_genesis_modeling.md) | 2026-09-27 | What a simulated history must track to place ore, coal and oil; how deposits are counted and sized (USGS three-part method). | Stands, refined: counts became "deposits that survive to the present" (decisions log 2026-09-29). |
| [`minecraft_integration_architecture.md`](notes/minecraft_integration_architecture.md) | 2026-09-27 | How simulated terrain gets into a NeoForge world (bake, runtime generator, density functions). | Core strategy adopted. Its bordered-world and CLI-first framing was overturned. |
| [`mod_compatibility_biomes_ores_fantasy.md`](notes/mod_compatibility_biomes_ores_fantasy.md) | 2026-09-27 (access dates; no date line) | How TerraBlender, Terralith, Tectonic and the biome mods inject biomes and terrain, and what survives a custom generator. | Findings stand. Fantasy terrain as overlays was replaced by simulated sky islands. |
| [`wrapped_planet_worlds.md`](notes/wrapped_planet_worlds.md) | 2026-09-27, §7 added 2026-09-28 | How a world can wrap east to west like a planet, and what happens at the seam and the poles. | Adopted (Toroidal World, cylinder, polar caps). Only the world sizes changed. |
| [`runtime_packaging_options.md`](notes/runtime_packaging_options.md) | 2026-09-27 | Where a heavy simulation can run for a mod: JVM, native code, WebAssembly, GPU, sidecar. | Its lean toward pure Java was overturned (Rust over JNI). Facts on native loading were used. |
| [`aeronautics_rust_bridge.md`](notes/aeronautics_rust_bridge.md) | 2026-09-27 | How Create Aeronautics / Sable ships and calls a Rust engine from Java, and what to copy or avoid. | Adopted. |
| [`player_demand_prior_art.md`](notes/player_demand_prior_art.md) | 2026-09-27 | Whether players want a geologic-history world generator; prior art and risks. | A dated snapshot; download counts have aged. |
| [`geologist_youtuber_feedback.md`](notes/geologist_youtuber_feedback.md) | 2026-10-01 | What a professional geologist who reviews game geology praises and criticises, from 15 videos. | Taken up (decisions log 2026-10-01). His numbers were not independently checked. |
| [`run-review-1a0f969d358.md`](notes/run-review-1a0f969d358.md) | 2026-10-01 | Is what one 1,000-Myr run shows accurate? A scientific review of a timelapse. | Findings taken up by the plate and climate designs. Has a list of Earth figures written "from memory, not checked". |
| [`grid-artifact-audit.md`](notes/grid-artifact-audit.md) | 2026-10-01 | Does the finished world show the grids it was computed on? | A dated measurement; several findings open. Its Earth river-sinuosity figure is from memory. |
| [`world-projection-options.md`](notes/world-projection-options.md) | 2026-10-02 | Which map projection should the Minecraft world use (Peirce quincuncial, its equal-area relatives, tori against Lambert)? | Research note; Peirce, Lee, Gringorten and Snyder read through secondary sources, Petroff 2021 in full; Toroidal World read as code, not run as a torus. |
| [`rotated-seam-worlds-prior-art.md`](notes/rotated-seam-worlds-prior-art.md) | 2026-10-02 | Has anyone built a walkable world whose seams turn (a half-turn quincuncial or orbifold world)? | Research note: nobody has; Toroidal World has unused half-turn vocabulary; the Minecraft mod sweep was narrower than planned (see the note); nothing was prototyped. |
| [`immersive-portals-pillowcase-prototype.md`](notes/immersive-portals-pillowcase-prototype.md) | 2026-10-02 | Can a quincuncial (pillowcase) world be built from Immersive Portals – CE seams, and does it work with MIC's pack? | A prototype: works in a small pack; MIC's pack crashes with it (Petrolpark, CBC Modern Warfare), Toroidal World and IP-CE kick every join, Sable crashed near seams; Distant Horizons not shown working. |
| [`high-co2-forcing-and-snowball-escape.md`](notes/high-co2-forcing-and-snowball-escape.md) | 2026-10-02 | A CO₂ forcing law valid to 10⁶ ppm and the cloud forcing on a snowball planet, for P1-26 task T6. | Research note: recommends the HEXTOR clear-sky OLR table (Kopparapu et al. 2013 model, MIT licence) scaled to Myhre's doubling; finds the linear OLR already carries about 20 W/m² of cloud greenhouse. Escape figures are computed, not sourced. |
| [`vertical-scale-study.md`](notes/vertical-scale-study.md) | 2026-10-02 | How many metres should a block stand for vertically (today 20 m, 15× exaggeration)? Two baked planets measured at 20 to 80 m per block and through non-linear mappings, against vanilla 1.21.1 and the MIC pack's terrain, with pictures, costs and options. | Measurement study; two planets of one preset and seed, five reference squares per world; changes no defaults. |
| [`session-rust-core-vs-java.md`](notes/session-rust-core-vs-java.md) | 2026-09-28 and 2026-10-01 | The Rust core against a full-Java implementation. | Coordinator's assessment; no Java comparison was ever measured. |
| [`session-gpu-vs-cpu.md`](notes/session-gpu-vs-cpu.md) | 2026-09-28 and 2026-10-01 | GPU against CPU optimisation, and whether one second per million simulated years is reachable. | Coordinator's assessment; speed-up ranges are estimates. |
| [`session-lava-realism.md`](notes/session-lava-realism.md) | 2026-10-01 | Where lava exists on a real planet; the lava floor as a game convention. | Coordinator's assessment; the real-world statements are unchecked. |
| [`session-mountain-height-limit.md`](notes/session-mountain-height-limit.md) | 2026-10-01 | Why mountains were too tall, and why the height limit should emerge from crustal strength instead of a cap. | Coordinator's assessment; built as "height limit, stages 1 and 2". |
| [`session-polar-climate-bands.md`](notes/session-polar-climate-bands.md) | 2026-10-01 | Why polar climate came out as bands, and why climate should be computed per cell. | Coordinator's assessment; led to the P1-26 design. |
| [`session-supercontinent-heating.md`](notes/session-supercontinent-heating.md) | 2026-10-02 | Whether heating of the mantle under supercontinents exists. | Coordinator's assessment from its own knowledge; no literature check. |
| [`session-distant-horizons-version.md`](notes/session-distant-horizons-version.md) | 2026-09-28 and 2026-10-01 to 2026-10-02 | Which Distant Horizons versions the far-terrain generator works with. | A sub-agent's finding on the real jars; follow-up questions open. |

## Designs (`../design/`)

| Document | Date | What it answers | Still holds? |
|---|---|---|---|
| [`p1-26-per-cell-climate.md`](../design/p1-26-per-cell-climate.md) | 2026-09-30, approved 2026-10-01 | How climate is computed per cell (a seasonal 2-D energy balance with sea ice and a moisture budget) instead of in latitude bands. | Approved, not built. Six of its references are marked "cited from memory". |
| [`plate-collisions-and-welding.md`](../design/plate-collisions-and-welding.md) | 2026-10-01, decided 2026-10-02 | How to fix collisions that raise land in one step, the missing Wilson cycle, and the mega-plate. | Diagnosis and targets stand; superseded by the force-balance design where they differ. |
| [`plate-force-balance.md`](../design/plate-force-balance.md) | 2026-10-02 | Whether plate speeds can come from a balance of forces (slab pull, ridge push, drag, collision resistance). | The newest design; chosen 2026-10-02, not built. Three of its figures could not be confirmed (see the ledger's verification plan). |
| [`plate-check-baseline.md`](../design/plate-check-baseline.md) | 2026-10-01 | What the `deeptime plate-check` tool measures on the plate model as it is today. | The "before" picture for the plate rewrite. |

## Plan, decisions, reviews and formats

| Document | Date | What it is |
|---|---|---|
| [`../deeptime-design-decisions.md`](../deeptime-design-decisions.md) | 2026-09-27, log to 2026-10-02 | Ben's decisions, with an append-only log; the top authority. |
| [`../deeptime-implementation-plan.md`](../deeptime-implementation-plan.md) | 2026-09-28, edited since | The implementation plan; §8 specifies each engine stage. |
| [`../plan-reviews/2026-09-28-cold-read-review.md`](../plan-reviews/2026-09-28-cold-read-review.md) | 2026-09-28 | First cold-read review of the plan (30 gaps, resolved in plan revision 2). |
| [`../plan-reviews/2026-09-28-cold-read-review-2.md`](../plan-reviews/2026-09-28-cold-read-review-2.md) | 2026-09-28 | Second cold-read review (29 gaps, fixed by the tasks that hit them). |
| [`../../crates/deeptime-format/FORMAT.md`](../../crates/deeptime-format/FORMAT.md) | no date line (format 0.3) | The world-data format: which fields exist and in which units. |
| [`../../crates/deeptime-format/EVENTS.md`](../../crates/deeptime-format/EVENTS.md) | no date line | The event log: which geologic events each stage records. |

## Topics with no research note of their own

The science pages in `../science/` are written from the code and the build log for these, because no
literature note exists: rivers and lakes; glaciers and ice sheets; volcanoes; meteor impacts; rock
classes and metamorphism; folds and faults (beyond the tool survey); the mapping from climate to
biomes; isostasy and flexure; sea level; hotspots; caves; petroleum systems (one paragraph in the
ore note). Plate motion and climate have designs with reference lists, not literature notes.
