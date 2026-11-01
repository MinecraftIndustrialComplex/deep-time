# A continental column's height from its whole buoyancy: design

Status: **on by default since 2026-11-01** (`tectonics.column_buoyancy`; the coordinator's call, decisions log
2026-11-01, after the arm cb-on: "The arm cb-on and the default") (the low-continents lane's
lead, the coordinator's task of 2026-10-31; branch `continental-buoyancy`). Ledger PLT-830 to PLT-836 (PLT-837 to 849
kept for what the arm asks). Science page: `docs/science/continental-buoyancy.md`.

## Why

The low-continents diagnosis (sea-level.md, "Why base35's continents stand low, against Earth (2026-10-31)", revised
after PR #56's review and re-review) found base35's continents low because its crust at least 20 km thick covers 0.48
of the planet instead of Earth's 0.38, 13 % thinner than Earth's: the larger area holds the sea 292 m over Earth's
fill (means; +492 m of it the area, -68 m the residual, of which the steering's offset about -47), and the thin crust
stands low. In logarithms of the area (per-run means):

- **The thickness carries about 0.6 of the excess.** Two parts:
  - The start class (continental crust on no ocean basement) thins from 35.1 to 33.5 km, its column at the sea on
    PLT-281's line (33.6 km under base35's sea; base34 32.8 against 32.6). Against Earth's 36.4 km this is 0.37 of
    the excess. Its cause is a hypothesis, not measured: on PLT-281's one Airy line, z = z_0 + (T - 35.1 km) / 7.02
    over the datum, thick crust stands high (400 to 1,320 m over Earth's ice-free crust at 36 to 44 km) and wears to
    the column at the sea.
  - The mix with crust converted on ocean floor (0.22): at the end that crust is 0.34 of the planet, 0.71 of the crust
    at least 20 km thick, at 30.9 to 33.4 km (island-arc crust 0.157 of the planet at 31.3 km), and only 0.40 of it
    stands dry, so it holds most of the drowned crust. It stands under the sea, which by the same hypothesis fills
    rather than wears, so its thickness is what the conversion builds (arcs maturing, PLT-047 and PLT-760; the
    thickening's spread, PLT-287; restacking onto upper plates on ocean floor).
- **The start's volume, 0.27** (14.8 against Earth's 14.0 km planet-mean, 6 % over); 0.44 if the start's thinner
  column (35.1 against 36.4 km) is counted as the start's.
- **The net volume, 0.13**: ocean crust taken into the columns (+0.61) net of own crust lost (-0.48).

Earth holds its thick crust low on cold, keeled lithosphere and its thin crust high on hot lithosphere: in CRUST1.0 the
Basin and Range and Cordillera stand 1,407 to 1,834 m on 30 to 45 km of crust and the cratons 140 to 660 m on 35 to 45
km, with alike crustal densities (m4cold.py); Hyndman & Currie 2011 put a hot back-arc at the sea on about 27 km and a
craton on about 38 km, about 1,600 m apart at equal crust.

**The lead** (PR #56's re-review's restatement, taken by the coordinator):

1. The height law first, with the start's column set to match it. On any craton line a 35.1 km start column would
   drown: Hyndman & Currie's craton line gives it a dry height of about -409 m, about 595 m under a datum sea under the
   water load, and this law gives -95 m at 35 km on the craton's state. So the law and the start's column are one
   design.
2. The start's volume, a smaller second part.
3. What the conversion builds, measured before the conversion rules are ruled out as a lever.

The law is expected to move the start class's part (its column at the sea: on a cold, keeled lithosphere thicker crust
stands at the sea, 35.7 km with the sea at the datum against PLT-281's 31.4, so by the hypothesis less of it wears
away) and the converted crust's heights (through its lithosphere, PLT-820 and PLT-821: crust with no state on young
floor reads its floor's thin L and stands high), not the converted crust's thickness, which the conversion rules set.

## The law

A column floats on the asthenosphere (Lachenbruch & Morgan 1990, eq. 1 as Zoback & Mooney 2003 print it): its dry
height is the free asthenosphere's height H_f plus Σ (ρ_a - ρ) dz / ρ_a down the column. `plates/buoyancy.rs`:

- the crust's pools at their depths with Christensen & Mooney 1995's rock densities (PLT-832; sediment by shale's
  porosity, PLT-833), expanded by α T on ISO-138's geotherm, the eclogite root as before (PLT-278 to PLT-280);
- the mantle lithosphere's thermal contraction from the Moho to L, less 0.4 of the keel's (Kaban et al. 2003, PLT-831);
- α 3.28 × 10⁻⁵ /K and ρ_m0 3,330 kg/m³, Parsons & Sclater's plate as Stein & Stein 1992 print it, with T_m 1,350 °C
  (PLT-830): ρ_a 3,183 kg/m³;
- H_f from the ridge: GDH1's 2,600 m of water over 7 km of ocean crust at T_m, -2,580 m (H_0 2.58 km against Zoback &
  Mooney's 2.78 ± 0.35);
- L as the heat-flow law reads it, `thermal::sample_l_km` (the lid's floor, PLT-820; crust with no state from its
  floor's age, PLT-821).

No constant is fitted. The tests hold it to Earth (Kaban et al.'s 0.35 km as the tolerance): the five cratons' 40 to
45 km band and the Basin and Range and Cordillera's 30 to 40 km bands on CRUST1.0's own crusts; Hyndman & Currie's
slopes at their mean crusts (126 and 124 m/km against 141 ± 27 and 131 ± 12) and gap (1,846 m against ~1,600); H_0; GDH1 on young floor.
Hasterok & Chapman 2007's mantle density stays out until their text is read.

## Where it applies

- **The settling** (`settle.rs`, ISO-078): every continental column's target height, in place of PLT-281's line plus
  ISO-110's term. The settling goes the whole way each step (τ 4 kyr), so this sets every continental height.
- **The collapse's thin sheet** (PLT-414): the column's potential energy at the law's height, against the normal
  column at `continent_elevation_m`'s middle as before.
- **Not changed:** the processes' within-step caps (`collapse::toward` in the collapse, the arcs, the flow, the
  peaks, the ridges and subduction), the collapse's room (PLT-412, T_n + 7.02 S) and T_n itself (35.1 km); the
  measurement budgets (gap budget, crust budget) keep reading PLT-281's line. The settling moves every column to the
  law's height after them; a cap that stops a process at PLT-281's height only shapes the step's path. Moving the caps
  to the law is a follow-up if the arm shows them binding.
- **The switch:** `tectonics.column_buoyancy`, on by default since 2026-11-01; needs `thermal_state`; excludes
  `thermal_isostasy_m` (params.rs).

## The start

Whether Ben chose the start's area or column (the coordinator's check): he did not. The decisions log's only row is
2026-09-29, the meaning of the editor's "ocean fraction" (the water's share, with flooded shelves). The values behind
the start's 0.4225 of the planet (`ocean_fraction` 0.7, P1-03; `continental_shelf_share` 0.29, Cogley 1984, P1-17)
and the 35.1 km column (P1-03, "design choice: 35 km", PLT-131) were agents' (git log -S on both; ledger PLT-131,
HYP-013, HYP-014). So the start goes with the law as one design, under the same switch, with no question for Ben.

With `column_buoyancy` each start continent takes the thickness at which the law holds Rudnick & Gao's three-layer
column (Hacker, Kelemen & Behn 2015's Fig. 1, PLT-834) at the height the preset gives it, on the craton's state the
thermal state starts it with (L 225 km, K 200 km): 34.8 km at the shelf break (-130 m, under a datum sea), 35.7 km at
the coast, 39.3 km at the inland middle (525 m) and 41.9 km at 900 m (PLT-835). The start's area stays the preset's.
Its volume then rises from 14.8 to about 16 km planet-mean (our arithmetic: about 0.71 of the start dry at a mean
39.3 km and 0.29 on the shelf at about 35.3 km, over 0.4225 of the planet), about 15 % over Earth's 14.0.

That is the second part, kept apart on purpose: the arm measures the law with the start's heights held, and the area
is the preset's values, not Ben's. If the arm shows the start's volume carrying the excess, the next step is the
start's area from Earth's like-for-like measure (`continental_shelf_share` from the share of Earth's crust at least 20
km thick under the sea, 0.248 by RLC-030, and `ocean_fraction` at Earth's 0.71, which give 0.384 of the planet), a
change to two preset values the editor shows; it would go to Ben as a question then, with the arm's numbers.

## What the arm measures

One 8-run arm, `column_buoyancy` on, against base35-s4 (the pool's base35 seeds 1 to 4 on both starts): the census
(land, the continental label under the sea, RLC-032's check), lowcont.py's `area` (the start class's thickness against
its column at the sea under the law, the converted crust's dry share and height by source, the log factors) and
`model` (the split's line term, the sea against Earth's fill), and the guards. What would say the law works: the
start class keeps more of its thickness, the converted crust stands higher on its hot lithosphere, the line term
moves toward Earth's at 36 to 44 km, and the drowned share falls, with no guard failure.

## The arm cb-on and the default (2026-11-01)

cb-on ran the branch at 1b85d0d0 with `column_buoyancy` on (8 runs, base35-s4's starts and seeds, key ecc2f3958a5b,
argkey 46913569). Its control is hc-on, the same code with the switch off, and base35-s4. It was read with
`pool.sh table` (census and `--measure guard`) and `tools/review/freeboard/lowcont.sh`
(`/data/ban/deeptime-runs/lowcont-cbon/`: area, model, m5sea).

**The census** (medians of 8, Mann-Whitney p against base35-s4):

| | land | largest landmass | continental crust under the sea | land above 2 km | coast with high ground | land under ice | collisions per 100 Myr | crust's net change | criteria median, mean | sea after 100 Myr / end, m |
|---|---|---|---|---|---|---|---|---|---|---|
| base35-s4 | 0.216 | 0.048 | 0.594 | 0.009 | 0.071 | 0.062 | 4.85 | +0.113 | 5, 5.38 | 280 / 339 |
| cb-on | 0.186 (0.021) | 0.063 (0.80) | 0.650 (0.083) | 0.034 (0.021) | 0.152 (0.010) | 0.067 (0.65) | 5.55 (0.21) | +0.102 (0.57) | 5, 4.75 (0.079) | 329 / 122 |

**The guard** (`measure guard`): worst 10,289 m against 10,304 (p 0.88); no step beyond two Earth reliefs; the largest
one-step rise and fall 7,274 and 7,369 m against 7,144 and 7,176; the highest plate 4,517 against 4,413 m; the plate
cells' largest excess over Airy 3,453 against 2,990 m (p 0.021).

**Against "What the arm measures"** (lowcont.py, cb-on against hc-on, final frame, medians):
- **The start class keeps more of its thickness:** 37.4 km at the end against 33.9, dry share 0.75 against 0.70, its
  median 65 m over the sea against 52. Met.
- **The line term moves toward Earth's at 36 to 44 km:** the p50 height over the sea by 1-km band is 17, 67, 40 and
  136 m at 36, 38, 40 and 42 km, against Earth's 430, 340, 380 and 420 and hc-on's 370, 637, 877 and 1,154 (PLT-281's
  line). That is the flatness of Earth's cold line, as the M5 lane's note asked (above), about 300 m under Earth's
  level. At 44 km the law gives 1,280 m against Earth's 437, the keel's step. Met in its slope; low in its level.
- **The converted crust stands higher on its hot lithosphere:** not met. Crust on ocean floor that is island-arc crust
  by cb_origin's rule (0.149 of the planet, 31.6 km) stands 493 m under the sea at its median against 226 with the
  line, dry share 0.28 against 0.35. Collision belts on ocean floor are 197 m under against 16 m over. Their mafic
  pools now weigh as basalt and gabbro (PLT-832); under PLT-281's line every crust weighed the mean crust's density.
- **The drowned share falls:** not met. Continental crust under the sea is 0.650 against 0.602, and the sea ends at 126
  m against 347.

**Reading (the coordinator's).** The law does what its sources say. A thick, cold, felsic column stands on its rock and
keel, and a column of mafic arc crust stands as low as its rock. Under the old line the model's large stock of
converted crust (sea-level.md "Why base35's continents stand low": most of the drowned crust) stood as high as
granite. The law shows that stock for what it is. The lever for the drowned share is that excess of converted crust,
not the height law. The coast with high ground (0.152 against its 0.11) and the land above 2 km rise because the thick
columns stand higher. These are V1 realism checks, not a gate.

**The default.** On, the coordinator's call (decisions log 2026-11-01) under Ben's rule of 2026-10-23: physics goes
on by default once it is built and correct (unit tests against its sources, no divergence or guard failure on the
pool), and the rule it replaces goes off with it. Here PLT-281's line is replaced where a column settles and in the
collapse's sheet; ISO-110 was already off. The branch was merged with master and PR #58 before the flip. A fresh arm,
cb-on2 (the branch's default against the next base), reads it on the merged head.

## Why thermal isostasy (ISO-110) is built and off, and whether that still holds

- **Built** 2026-10-23 with the lithosphere's thermal state (gap 4 stage 1, commits 63f99656 and cce71817; ISO-110 to
  ISO-112, lithosphere-thermal-state.md). Its form adds a term linear in the lithosphere's depth L to PLT-281's line,
  z_th = E (L_ref - L) / (L_cold - L_hot), E = 1,600 m (Hyndman & Currie 2011).
- **Off as an open fault, not a V1 matter** (lithosphere-thermal-state.md "On by default (2026-10-23)"; ISO-110's row):
  the add-on needs a frame, L_ref, the depth at which PLT-281's line stands. The craton frame (225 km) takes the line
  for the craton's, which holds only if the sea stands 938 m over the datum; Hyndman & Currie's frame reads L_ref from
  their two sea-level crusts and so moves with the sea, L_ref = 60 + (S + 629 m) x 165 / 1,600 km, feeding the sea's
  error into every column's height. The page's recommendation: "the physics, not a frame: give a continental column
  the height of its whole lithosphere's buoyancy (the crust's density, the mantle lithosphere's thermal contraction and
  the keel's depletion) against the ridge's column, as Lachenbruch & Morgan (1990) do ... in place of PLT-281's line, so
  the sea follows the columns' volume and no L_ref is needed".
- **Deferred before that** (decisions log 2026-10-20, the coordinator: per-column thermal isostasy "deferred, behind the
  orogen-extent and crust-loss work, since it would only lower the old fifth of the high ground"). That work has landed
  (the orogen's two wedges, PR #50; the crust budget, PR #53).
- **The M5 lane's note on the cold line** (2026-10-25, ISO-110's row): ISO-110 in the datum frame overshoots cold thick
  crust (a craton at 45 to 50 km 965 to 1,678 m against Earth's 340 m), because CRUST1.0's cold platforms rise only 1 to
  27 m per km of crust from 35-40 to 45-55 km against Airy's 142: "the crust past about 40 km acts as rock of about 3,200
  to 3,300 kg/m3". A thermal term alone cannot give that.

So the reason ISO-110 is off still holds for its form (a frame on a median line), and this design removes the frame
rather than choosing one: every column's height is its buoyancy against the ridge's column, whose depth the model
already carries (GDH1's 2,600 m, PLT-352). ISO-110 stays off with the new law on (the switches exclude each other).
The cold line's flatness is a separate term of the crust's own density (below, "The lower crust").

## With the lid's floor (PR #58) and crust with no state

The law reads L through `thermal::sample_l_km`, the heat-flow law's reading: never above the lid's floor, 1.25 times
the crust (PLT-820), and for crust with no state its floor's lithosphere (PLT-821). PR #58's review (finding 3) found
the stored L can sit under the floor after a resampling until the next `thermal::step`; the hotspot lane is storing the
floored L at refresh. The law floors L on read, so it reads the same L either way. Its finding 10 (ISO-110's
interaction: PLT-821 moves crust with no state from a thermal height of 0 to about +2 km at E = 1,600 m) has its
counterpart here: crust made continental on young floor reads a thin, hot lithosphere and stands high under the law
too, by its own column's buoyancy rather than by ISO-110's linear term, and sinks as `thermal::step` cools it. That is
the converted crust's height the arm reads. If the floor is later solved on ISO-138's own geotherm (finding 6), the
law follows it through `thermal::lid_floor_km`.
