# Plate tectonics

Status of this page: `stub` (stage 1). The ledger rows exist; the page is to be written in stage 2 from [_template.md](_template.md).

The planet's outer shell is split into rigid plates that move, pull apart at ridges, sink under one another at subduction zones (where one plate dives beneath another), and collide to raise mountain belts. Today's code moves plates by speed rules taken largely from a computer-graphics paper (Cortial and others 2019) and tuned by hand; it also makes ocean floor that deepens with age, hotspots and large igneous provinces, and sets the crust's thickness and height. **This model is about to be replaced** by a balance of forces (slab pull, ridge push, drag, collision resistance; `docs/design/plate-force-balance.md`, chosen 2026-10-02), with collisions that shorten crust instead of lifting it in one step (`docs/design/plate-collisions-and-welding.md`). The ledger rows describe the code as it is today and are marked `pending-rewrite`; the constants of the new design are ledgered as `not-yet-coded`.

## Ledger ids that belong here

| Ledger | Feature | Ids (count) |
|---|---|---|
| [PLT](ledger/PLT.md) | plate setup | PLT-001 to PLT-021, PLT-203, PLT-205, PLT-209 to PLT-211 (26) |
| [PLT](ledger/PLT.md) | plate motion | PLT-022 to PLT-028 (7) |
| [PLT](ledger/PLT.md) | subduction | PLT-029 to PLT-067, PLT-206 to PLT-207 (41) |
| [PLT](ledger/PLT.md) | collisions | PLT-068 to PLT-081, PLT-222 to PLT-227 (20) |
| [PLT](ledger/PLT.md) | rifting | PLT-082 to PLT-091 (10) |
| [PLT](ledger/PLT.md) | ridges and ocean crust | PLT-092 to PLT-107, PLT-204, PLT-212 (18) |
| [PLT](ledger/PLT.md) | hotspots | PLT-108 to PLT-120 (13) |
| [PLT](ledger/PLT.md) | crust and isostasy (plates) | PLT-121 to PLT-142, PLT-208 (23) |
| [PLT](ledger/PLT.md) | plate-check targets | PLT-143 to PLT-173, PLT-213 to PLT-221 (40) |
| [PLT](ledger/PLT.md) | designed, not yet coded | PLT-174 to PLT-180, PLT-191 to PLT-202 (19) |
| [PLT](ledger/PLT.md) | the force balance's settings, coded and recorded, not applied (page [plate-forces.md](plate-forces.md); constants in [FRC](ledger/FRC.md)) | PLT-181 to PLT-190 (10) |

Rows by status (after WP-13a and plate-motion task 2, 2026-10-02): `verified` 17, `cited-unverified` 45, `from-memory` 4, `abstract-only` 14, `calibrated` 13, `game-choice` 68, `uncited` 46; `pending-rewrite` 144; `not-yet-coded` 25.

## Material for the stage 2 writer

- Extraction notes (measured Earth-against-mod numbers from the build log, simplifications, flags): [extraction-notes/PLT.md](extraction-notes/PLT.md).
- Designs: `docs/design/plate-force-balance.md`, `docs/design/plate-collisions-and-welding.md`, `docs/design/plate-check-baseline.md` (the measured state of today's model).
- Accuracy review of one run: `docs/research/notes/run-review-1a0f969d358.md`.
- Work package in the verification plan: WP-13 (deferred until the rewrite lands) ([claims.md](claims.md#verification-plan-for-stage-2)).

## Known before the page is written

- Since 2026-10-02 (plate-motion task 2) a contact keeps a collision state with its real shortening: the convergence across the collision front, integrated (ledger PLT-222 to PLT-226). The built-in presets still transfer terranes and weld plates on the older count of relative speed times time (PLT-068, PLT-227), so the worlds are unchanged; the measurements behind that choice are in the build log entry of that date.

- The accuracy review calls today's model "a convincing cartoon of plate tectonics, not yet an accurate simulation": no Wilson cycle (rifted pieces weld back after a median 18 Myr), one plate over half the planet in about half of all frames, collisions that raise round blobs in one step.
- The force balance is computed and recorded every step since 2026-10-02 (task 5), not yet applied: [plate-forces.md](plate-forces.md), constants in ledger FRC; PLT-181 to PLT-190 are coded.
- The force-balance design could not confirm three figures: the gross size of slab pull, a factor of 8 for continental drag attributed to Forsyth & Uyeda, and the wedge-taper reference. WP-13a settled all three; see the next section.

## The planned force balance: its constants and targets checked (WP-13a)

Checked 2026-10-02, before the force balance is built, so that it is built on numbers that were
read. Nothing here is in the game yet; the rows are PLT-174 to PLT-202 (constants) and the targets
PLT-143 to PLT-162 and CHK-001 to CHK-057. No default of the design changed. Two plate-check ranges
did.

**What moves plates on Earth.** Forsyth and Uyeda (1975) solved for the forces that balance the
torques on twelve plates. The pull of sinking slabs and the resistance to their sinking came out an
order of magnitude larger than anything else. Plates attached to slabs move at 6–9 cm/yr, close to
a terminal velocity at which the slab's weight and the mantle's resistance balance; plates without
slabs move at 0–4 cm/yr; and drag under the continents is about eight times the drag under the
oceans. Later work splits the slabs' effect into a direct pull from upper-mantle slabs and a
"suction" from flow driven by lower-mantle slabs, and finds slabs supply over 90 % of the driving
force and ridge push under 10 % (Lithgow-Bertelloni and Richards 1998). Continental roots raise the
coupling to the mantle two to five times (Conrad and Lithgow-Bertelloni 2006).

**The constants.**

| Ledger id | Quantity | Design value | Source and how it was read | Status |
|---|---|---|---|---|
| PLT-182 | slab pull law | ∝ slab length × √(age of floor) | Faccenna et al. 2012, eq. 1 (after Turcotte & Schubert 1982); full text | `verified` (the 10 Myr cut-off is not sourced) |
| PLT-183 | size of slab pull | of order 10¹³ N/m | 1.4 × 10¹³ N/m for 80 Myr floor, from Faccenna et al.'s eq. 1 (computed) | `verified` |
| PLT-184 | slab length, break-off time | 700 km; 10 Myr | 700 km as Faccenna et al.; break-off 10 (young slab) to 20–25 Myr (old slab), van Hunen et al. 2011 abstract | `abstract-only` |
| PLT-185 | slab resistance | 3 | terminal velocity: Forsyth & Uyeda p. 178; the 3 chosen by the open-loop test | `calibrated` |
| PLT-186 | slab suction share | 0.3 of pull, overriding plate only | Conrad & Lithgow-Bertelloni 2002, 2004 (abstracts): suction 30–40 % of the forces, on both plates; their suction ÷ pull is 0.43–0.67 | `abstract-only` (not the same quantity; question for Ben) |
| PLT-187 | edge energy (ridge push, belt weight) | 0.1 of slab pull | ridge push 0.06 of slab pull per unit length (Forsyth & Uyeda Table 6); under 10 % of the driving force (Lithgow-Bertelloni & Richards 1998) | `verified` |
| PLT-188 | drag under continents | 3 times | 2–5 (Conrad & Lithgow-Bertelloni 2006, full text); about 8 (Forsyth & Uyeda, p. 189) | `verified` |
| PLT-189 | boundary resistance | 3 | Forsyth & Uyeda's transform resistance; the 3 chosen by the open-loop test | `calibrated` |
| PLT-192 | wedge surface slope | 1.5° | Lesser Himalayan wedge 1–1.6° (DeCelles & Carrapa 2021, full text); a critical wedge keeps its taper (Davis, Suppe & Dahlen 1983, abstract) | `verified` |

The design's 1.5° "taper" is a surface slope. In the theory of critical wedges the taper is the
surface slope plus the dip of the fault beneath; the Himalayan front has a taper of 6–7.5° made
mostly of that dip. The foreland line-load profile of the design (PLT-193) will not be built: the
general flexure already bends the plate under every load (decisions log 2026-10-02).

**Earth's figures for the targets.**

| Quantity | Earth | Source and how it was read |
|---|---|---|
| Area-weighted RMS plate speed | about 47 mm/yr today; 50–100 mm/yr over the last 200 Myr | Forsyth & Uyeda 1975 Table 1 (computed); Zahirovic et al. 2015 §3.2 (full text) |
| Plate speeds, 10th / 50th / 90th percentile | 11 / 33 / 80 mm/yr (today's 12 plates); median of 3,952 plate-samples over 200 Myr 42 | Forsyth & Uyeda Table 1 (computed); Zahirovic et al. §3.5 |
| Oceanic ÷ continental plate speed | 2–3 | Zahirovic et al., abstract and §3.5 |
| Slab-attached ÷ other plate speed | 3.0 (200 Myr); about 4 (today) | Zahirovic et al. §3.5; Conrad & Lithgow-Bertelloni 2004 (abstract); 3.8 from Forsyth & Uyeda Table 1 |
| Rank correlation of speed with slab share / with area | 0.71 / −0.34 (12 plates) | computed from Forsyth & Uyeda Table 1 |
| Sea-floor age: median, 90th percentile, share older than 180 Myr | 55 Myr, 133 Myr, 0.6 % (oldest 339 Myr) | Seton et al. 2020 age grid, area-weighted (computed) |
| Supercontinent tenure; cycle | 100–300 Myr; 400–800 Myr, about 600 dominant | Mitchell et al. 2021 (accepted manuscript, full text); Nance 2022 gives about 500 |
| Largest plate; plates over 5 % of the sphere | 20.5 %; seven | Bird 2003, Table 1 (full text) |
| India–Asia convergence after collision | about 15 to about 4 cm/yr between 50 and 35 Ma | Copley et al. 2010 (abstract) |

Two plate-check ranges were wrong and were changed: the area-weighted RMS speed (35–55 → 40–100
mm/yr; the old top left out every Earth value of the last 200 Myr) and the supercontinent period
(300–600 → 400–800 Myr; 300–500 was a Wikipedia figure the cited papers do not give). A test in
`plates/check.rs` (`earth_figures_pass_their_targets`) holds every Earth figure above inside the
range it anchors. Three targets are lenient against Earth and are questions for Ben in STATUS.md:
the sea floor's 90th percentile ceiling (230 Myr against Earth's 133), the collision length (15–50
Myr against Earth's examples of 50–140) and, for the design, the speed scale and the suction share.

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
