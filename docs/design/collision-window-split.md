# The collision orogeny's window, split by use: design

Status: **design** (2026-10-29, the coordinator), not built. For the collision-front lane (STATUS
Follow-ups, "Collision belts at the coast"). The sources are those read for PLT-079's note on
2026-10-29 (plates.md "Step 20", bibliography `EnglandThompson1984`, `ThompsonEngland1984`,
`Hopkinson2017`, `Weller2021`, `Rey2001`). Each window below says whether a source gives it or it
is the project's reading.

## What one window does today

`tectonics.collision_orogeny_myr` (30 Myr, PLT-079, `game-choice`) measures from a sample's
**last** collision stamp (`Sample::orogeny_myr`, renewed every step the belt, the wedge or the
interior share raises it by PLT-247's 5 m/Myr). While `t − orogeny_myr < 30` a Himalayan sample:

| use | where | what it does |
|---|---|---|
| lock | `plates/subduction.rs`: `locked`, `locked_against` | cannot be forced under again; a contact whose subducting side is mostly such crust reverses |
| regime | `plates/mod.rs`: `finish_samples` | `regime::COLLISION` on the L1 layer `plates/regime` |
| Andean guard | `plates/subduction.rs` (the arc's effects) | an arc's uplift does not overwrite the Himalayan stamp |
| event | `plates/collision.rs` | the Himalayan `orogeny_phase` (now onset to end) |

and the regime feeds, in the structure and surface stages:

| reader | where | what the regime means there |
|---|---|---|
| collision plutons | `structure/plutons.rs`: `intensity` (`collision_pluton_factor` 0.3) | S-type granite magmatism |
| stocks | `structure/deform/intrude.rs` | a stock at full rate whatever the convergence; granite |
| regional metamorphism | `structure/mod.rs` (the burial factor) | thickened crust's burial; also `orogeny_age < 30.0`, a literal copy of the window |
| fold-belt traps | `structure/traps.rs` | the active orogen (distance from it); no trap inside it |
| fault density | `surface/fields.rs` (`regime::COLLISION` → 1.0) | the deposits' pathway field |
| the V1 measure | `plates/stats.rs` | the collision regime's share of the land |

Provinces already use a window of their own (`orogen_max_age_myr`, `provinces.rs`).

## What the sources time differently

- **Melt** (PLT-079's note): models need "a time period of at least 20 to 30 million yrs between
  collision and melting" (Hopkinson et al. 2017, p. 58); the Himalaya's leucogranites came about 25
  to 35 Myr after its 50 ± 10 Ma collision (Weller et al. 2021); thickened crust stays hot for tens
  of Myr (Thompson & England 1984, Fig. 8: a melt field from about 20 to past 80 Myr after
  thrusting). Today's regime puts the collision granites in the first 30 Myr after a stamp, during
  the thrusting, and stops them about when the sources start them.
- **Thrusting**: no source gives a window; thrusting goes on 30 to 50 Myr after a collision's onset
  (the Himalaya, the Greenland Caledonides; Weller et al. 2021), i.e. while the plates converge,
  which is what the stamp already records.
- **Erosion and relief**: 50 to 200 Myr (England & Thompson 1984, p. 901). The surface stage
  erodes the relief itself; no window is needed.

## The split

1. **Track the onset.** A sample gets `orogeny_onset_myr` (f32): the time its current collision
   orogeny began, set where a stamp makes a sample Himalayan that was not, or whose stamp had
   expired (`t − orogeny_myr ≥ collision_orogeny_myr`). Kept through resamplings like
   `orogeny_myr`. An L1 layer `plates/orogeny_onset_age` (Myr) beside `plates/orogeny_age`. (A
   checkpoint and plate-state change; no backward compatibility before release.)
2. **The regime is the active orogen.** `regime::COLLISION` while the sample is stamped this step
   or the last (`t − orogeny_myr ≤ 2 dt`): the thrusting, as the stamp records it. Folds, fault
   density, the fold-belt traps and the V1 measure read it. The project's reading (no source gives
   a tail); the measure's collision regime falls, the V1 numbers do not move (they read elevation).
3. **Melt in its own window.** The collision plutons and the granite stocks read a melt flag: a
   continental Himalayan sample with `onset_age ≥ 20 Myr` (Hopkinson 2017's lower bound, `verified`
   from the note) and `t − orogeny_myr ≤ 60 Myr` (the melt field's reach past the thickening:
   Thompson & England's Fig. 8 runs past 80 Myr after thrusting for one instantaneous event; taking
   60 after the last thickening is the project's reading, to check against the figure). Two new
   ledger rows; the plutons' rate (`collision_pluton_factor`) is unchanged.
4. **Metamorphism** keeps the thickened column's burial, read from the melt window too (peak
   metamorphism and melting are the same thermal relaxation: England & Thompson 1984), and the
   literal `30.0` in `structure/mod.rs` goes.
5. **The lock and the event keep `collision_orogeny_myr`** (30 Myr, a stabilizer, `game-choice`):
   it stops a continent at a junction being handed round, not a geologic clock.

## Measuring it

One 8-run arm with the default, `boundary`, `collcoast` and `guard` measures, and from the structure
stage's events: collision plutons and stocks per Myr, their age after their collision's onset
(should centre 20 to 40 Myr, not 0 to 10), and how many of them sit in active belts against in
belts that stopped converging. Unit tests: the onset survives a resampling and a renewed stamp; the
melt flag is false at onset + 10 and true at onset + 25 under continued thickening, and false 61
Myr after the last stamp.

## What it leaves

The window of the lock stays a game choice. The melt window's end is the project's reading of one
figure; a source giving the duration of post-collisional magmatism in Myr after convergence ends
would replace it.
