# The collision orogeny's window, split by use: design

Status: **built** (2026-10-30, the coordinator, branch `window-split`; the design of 2026-10-29
revised after the Graphide review of PR #42). Ledger PLT-734 to PLT-736; plates.md "The collision
window split". For the collision-front lane (STATUS Follow-ups,
"Collision belts at the coast"). The sources are those read for PLT-079's note on 2026-10-29
(plates.md "Step 20", bibliography `EnglandThompson1984`, `ThompsonEngland1984`, `Hopkinson2017`,
`Weller2021`, `Rey2001`). Each window below says whether a source gives it or it is the project's
reading.

## What one window does today

`tectonics.collision_orogeny_myr` (30 Myr, PLT-079, `game-choice`) measures from a sample's
**last** stamp (`Sample::orogeny_myr`, renewed every step the belt, the wedge or the interior
share raises it by PLT-247's 5 m/Myr). While `t − orogeny_myr < 30` a Himalayan sample:

| use | where | what it does |
|---|---|---|
| lock | `plates/subduction.rs`: `locked`, `locked_against` | cannot be forced under again; a contact whose subducting side is mostly such crust reverses |
| regime | `plates/mod.rs`: `finish_samples` | `regime::COLLISION` on the L1 layer `regime` |
| Andean guard | `plates/subduction.rs` (the arc's effects) | an arc's uplift does not relabel the sample Andean (after 30 Myr it does, and `orogeny_myr` becomes the arc's) |

The Himalayan `orogeny_phase` event does not read the window: it runs from the collision's onset to
its transfer or weld (`plates/collision.rs`).

The regime feeds, in the structure, surface and deposit stages:

| reader | where | what the regime means there |
|---|---|---|
| collision plutons | `structure/plutons.rs`: `intensity` (`collision_pluton_factor` 0.3) | S-type granite magmatism |
| stocks | `structure/deform/intrude.rs` | three things: the gate (where stocks rise at all), the full rate whatever the convergence, granite |
| regional metamorphism | `structure/mod.rs` (the burial factor) | thickened crust's burial; also `orogeny_age < 30.0`, a literal copy of the window |
| fold-belt traps | `structure/traps.rs` | the active orogen (distance from it); no trap inside it |
| fault density | `surface/fields.rs` (`regime::COLLISION` → 1.0) | the deposits' pathway field |
| provinces | `provinces.rs` | an orogen province, OR'd with the province's own window (`orogen_max_age_myr`, 500 Myr), so no change |
| the measures | `plates/stats.rs`, `measure_coastset.py` | the coast in a collision belt; the regime's share of the land |
| the timelapse | the keyframe layer `regime` | the collision colour |

Folds and thrusts do not read the regime: they read the active collision's footprint
(`plates/collision_mm_yr`, `structure/deform/emplace.rs: strain`).

## What the sources time differently

- **Melt** (PLT-079's note): models need "a time period of at least 20 to 30 million yrs between
  collision and melting" (Hopkinson et al. 2017, p. 58); the Himalaya's leucogranites came about 25
  to 35 Myr after its 50 ± 10 Ma collision (Weller et al. 2021); thickened crust stays hot for tens
  of Myr: Thompson & England (1984, Fig. 8, a 50 km thrust) show no melt at 0 Myr after the
  thrust, a small field at 20, a large one at 50, still one at 80 and none at 120. Today's regime
  puts the collision granites in the first 30 Myr after a stamp, during the thrusting, and stops
  them about when the sources start them.
- **Thrusting**: no source gives a window; thrusting goes on 30 to 50 Myr after a collision's onset
  (the Himalaya, the Greenland Caledonides; Weller et al. 2021), i.e. while the plates converge,
  which is what the stamp already records.
- **Erosion and relief**: 50 to 200 Myr (England & Thompson 1984, p. 901). The surface stage
  erodes the relief itself; no window is needed.

## The split

1. **Track the collision.** A sample gets `collision_onset_myr` (when its current collision
   orogeny began) and `collision_myr` (its last collision stamp), both set only by a collision's
   stamp (`Sample::stamp_collision`), so an arc that later relabels the sample Andean leaves them
   alone. A stamp starts a new orogeny (onset = now) where the sample had none, or where its last
   collision stamp is more than the melt window's end ago (item 3): crust still hot from a
   collision that is thickened again goes on melting, and the onset does not hang on the lock's
   30 Myr. Both ride resamplings with the orogeny's other labels (the nearest old sample's).
2. **The regime is the active orogen.** `regime::COLLISION` while a Himalayan sample was stamped
   this step or the last (`t − orogeny_myr < 1.5 dt`: ages 0 and dt, half a step for the f32
   stamp's rounding). The fault density, the fold-belt traps, the measures and the timelapse read
   it. The project's reading (no source gives a tail). The V1 numbers do not move (the plates and
   the terrain do not read the regime); the measures' collision shares fall.
3. **Melt in its own window.** A plate-side flag, the L1 layer `plates/collision_melt` (1 or 0):
   continental crust whose collision began `collision_melt_after_myr` (20) or more ago and was
   last stamped `collision_melt_end_myr` (80) or less ago. The start is Hopkinson et al.'s lower
   bound; the end is the last of Thompson & England's panels with a melt field, counted from the
   last stamp instead of one thrust (the project's reading). Both are `tectonics.*` params, since
   item 1 uses the end. Readers: the plutons' collision term (the rate `collision_pluton_factor`
   unchanged); and for the stocks all three uses: the gate, the full rate and granite. An active
   belt before onset + 20 gets no collision stocks (an arc's still do), since its crust has not
   melted yet.
4. **Metamorphism** keeps the thickened column's burial, read from the melt flag too (peak
   metamorphism and melting are the same thermal relaxation: England & Thompson 1984), and the
   literal `30.0` in `structure/mod.rs` goes.
5. **The lock and the Andean guard keep `collision_orogeny_myr`** (30 Myr, a stabilizer,
   `game-choice`): it stops a continent at a junction being handed round, not a geologic clock.
   The guard's relabelling no longer ends anything else, since the melt reads item 1's fields.

## Measuring it

One 8-run arm on the default, against base31, with the default, `boundary`, `collcoast` and
`guard` measures and the structure stage's summary: plutons, stocks and metamorphosed volume per
run. The plates and the terrain are unchanged, so the V1 numbers must match base31's exactly; the
coastset's collision shares and the margins' collision count fall by definition. Unit tests: the
onset survives renewed stamps, an Andean relabelling and a restamp within the melt window, and a
restamp after it starts a new one; the melt flag is false at onset + 10 and true at onset + 25
under continued thickening, true 80 Myr after the last stamp and false at 81.

## What it leaves

The window of the lock stays a game choice. The melt window's end is the project's reading of one
figure; a source giving the duration of post-collisional magmatism in Myr after convergence ends
would replace it.
