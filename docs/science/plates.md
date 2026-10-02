# Plate tectonics

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The planet's outer shell is split into rigid plates that move, pull apart at ridges, sink under one another at subduction zones (where one plate dives beneath another), and collide to raise mountain belts. Today's code moves plates by speed rules taken largely from a computer-graphics paper (Cortial and others 2019) and tuned by hand; it also makes ocean floor that deepens with age, hotspots and large igneous provinces, and sets the crust's thickness and height. **This model is about to be replaced** by a balance of forces (slab pull, ridge push, drag, collision resistance; `docs/design/plate-force-balance.md`, chosen 2026-10-02), with collisions that shorten crust instead of lifting it in one step (`docs/design/plate-collisions-and-welding.md`). The ledger rows describe the code as it is today and are marked `pending-rewrite`; the constants of the new design are ledgered as `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [PLT](ledger/PLT.md) | plate setup | PLT-001 to PLT-021, PLT-203, PLT-205, PLT-209 to PLT-211 (26) |
| [PLT](ledger/PLT.md) | plate motion | PLT-022 to PLT-028 (7) |
| [PLT](ledger/PLT.md) | subduction | PLT-029 to PLT-067, PLT-206 to PLT-207 (41) |
| [PLT](ledger/PLT.md) | collisions | PLT-068 to PLT-081 (14) |
| [PLT](ledger/PLT.md) | rifting | PLT-082 to PLT-091 (10) |
| [PLT](ledger/PLT.md) | ridges and ocean crust | PLT-092 to PLT-107, PLT-204, PLT-212 (18) |
| [PLT](ledger/PLT.md) | hotspots | PLT-108 to PLT-120 (13) |
| [PLT](ledger/PLT.md) | crust and isostasy (plates) | PLT-121 to PLT-142, PLT-208 (23) |
| [PLT](ledger/PLT.md) | plate-check targets | PLT-143 to PLT-173, PLT-213 to PLT-221 (40) |
| [PLT](ledger/PLT.md) | designed, not yet coded | PLT-174 to PLT-202 (29) |

Rows by status: `verified` 2, `cited-unverified` 51, `from-memory` 8, `abstract-only` 22, `calibrated` 19, `game-choice` 66, `uncited` 53; `pending-rewrite` 183; `not-yet-coded` 29.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/PLT.md](extraction-notes/PLT.md).
- Designs: `docs/design/plate-force-balance.md`, `docs/design/plate-collisions-and-welding.md`, `docs/design/plate-check-baseline.md` (the measured state of today's model).
- Accuracy review of one run: `docs/research/notes/run-review-1a0f969d358.md`.
- Work package in the verification plan: WP-13 (deferred until the rewrite lands) ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- The accuracy review calls today's model "a convincing cartoon of plate tectonics, not yet an accurate simulation": no Wilson cycle (rifted pieces weld back after a median 18 Myr), one plate over half the planet in about half of all frames, collisions that raise round blobs in one step.
- The force-balance design could not confirm three figures: the gross size of slab pull (from textbook examples, not found in Turcotte & Schubert itself), a factor of 8 for continental drag attributed to Forsyth & Uyeda, and the wedge-taper reference (Davis, Suppe & Dahlen 1983, cited from memory).

## Plate-check: boundary mix and arc heights

Added 2026-10-02 (PLT-213 to PLT-221; measured values in `docs/design/plate-check-baseline.md`).
`deeptime plate-check` now records and judges two things the plate rules were not judged on before.
They exist because a workbench run 28 Myr after a supercontinent start showed nearly every boundary
between ocean plates as a subduction zone with a continuous volcanic arc high enough to carry snow.

**What happens on Earth.** Of the 260,487 km of present plate boundary in Bird's PB2002 model
(Bird 2003, Table 3), 19.7 % is subduction zone (51,310 km), 36.4 % is spreading (ocean ridges
25.8 %, continental rifts 10.5 %), 28.4 % is transform, 8.8 % is continental collision and 6.7 % is
oceanic convergence without a Benioff zone or arc. Subduction is concentrated round the Pacific;
most of the length inside an ocean is ridge and transform. Pangaea is commonly described as ringed by subduction (not sourced here), so a
supercontinent planet may sit above 19.7 %, which is why the target (15-25 %) is a range and the
mix over the first 50 Myr after a supercontinent start is reported beside it, without a judgment.

Arcs differ by what they stand on. Intra-oceanic arcs (the Marianas, Tonga-Kermadec, Izu-Bonin)
are chains of volcanoes mostly below the sea: along the Mariana arc 20 of the 26 active volcanic
centres are submarine (Baker et al. 2008), the Kermadec arc's vents lie 250-1,660 m below the sea
(de Ronde et al. 2003), and the highest island-arc cones are about 1 km. Continental arcs stand on thick
crust and are high: the Cascades' basement under the chain is 1.2-2 km and a dozen cones exceed
3 km (Hildreth 2007); the Central Andes' arc sits on a 4.5 km plateau (Gregory-Wodzicki 2000).

**What the model measures.** Every step, the plates stage counts the boundary by kind (ocean-ocean
and ocean-continent subduction, collision, ridge, rift, other) and the elevation of its arc cells
(median, 90th percentile, share above the sea datum) split into oceanic and continental arcs. The
definitions are PLT-220 and PLT-221. Nothing a plate rule reads depends on them.

**The model today** (baseline of 2026-10-02, mean of four 1000 Myr supercontinent runs): 56 % of its
boundary is subducting (191,000 km a step against Earth's 51,310 km; its spreading share, 39.5 %, is
close to Earth's 36.4 %, and the model has almost no transform boundary, 1.3 % against 28 %). Its
oceanic arcs are mostly submarine (14 % of arc cells above the sea, crest 312 m) and its continental
arcs stand 1.7 km median, 2.9 km crest, all inside the Earth ranges; the subducting share is the one
measure outside its range.

**Limits.** Lengths are counts of lattice samples times the sample spacing (about 70-80 km), so the
subducting share is an estimate in the same unit as the slab front length, not a surveyed length.
Bird's classes are not the model's: his transform class is by azimuth against the relative
velocity, the model's "other" also holds converging boundaries that have no contact yet. The arc
comparison is by cells (area) against Earth's count of volcanoes. The oceanic arc crest limit
(1 km) and the Andean crest limit (6 km) are from memory (PLT-217, PLT-219). The target ranges
fail today's model by design: they are what the force-balance and belt tasks are to move.
