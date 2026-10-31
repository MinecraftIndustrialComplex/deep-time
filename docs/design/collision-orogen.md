# The collision orogen as a doubly vergent wedge and its plateau: design

Status: **designed and built behind switches** (2026-10-31, the orogen lane, branch `collision-orogen`; Ben's
order "Freeboard, then orogen (Recommended)", decisions log 2026-10-30). The two wedges (PLT-730, PLT-731) were
built on 2026-10-29 and kept off; this design reads them again on the freeboard physics (PLT-660, PLT-661, on since
2026-10-31) and adds one rule, the plateau's retro-ward growth (PLT-740, `tectonics.collision_retro_plateau`).
Defaults are the coordinator's call on the lane's report. plates.md "The collision orogen (PLT-740)" holds the
arms' tables.

**Measured** (2026-10-31): the cumulative arm orog-cum (O1 + O2, 8 runs) against base34-s4 and base34: the guard
clean; the plateau rule placed a median 0.0005 of the kept crust (it binds only in the largest collisions); both
two-wedge arms shrink the largest landmass (0.094 on base34 to 0.075 and 0.074), raise the continental crust under the
sea (0.540 to 0.590 and 0.605, over the check's 0.56) and the crust's net change (+0.063 to +0.100 and +0.094), and
lower the high ground through time (above 2 km 0.030 to 0.022 and 0.024; above 4 km 0.0023 to 0.0008 and 0.0012); the
coast with high ground is not resolved (0.101 and 0.067 against base34's 0.078: two arms of the same physics but for a
thousandth of the crust differ by more than either differs from the base).

## Why

The coast-with-high-ground audit (docs/research/notes/coast-high-ground-audit-2026-10-30.md) found 96 % of the
crust a collision keeps on the overriding plate, Taiwan's upper plate (the Coastal Range, 0 to 1.68 km, Lai et al.
2022) standing far lower than the model's narrow upper blocks (a median 3.4 km), and no Tibet-class plateau (land
over 4 km 0.0013 of the land against Earth's 0.0195; the high ground a median 255 km from the sea against 980).
With the freeboard on, the land above 2 km fell to 0.013 (arm sedmargin, 8 runs; base33-s4 0.046), so the orogen is
asked to rebuild the mountains by physics, with no knob, cap or forcing term (Ben: "Results that don't align with
earth but are still perfectly playable and physics backed > Results that are forced to align with earth").

## The two wedges on the freeboard physics (the arm orog-tw-sedm, 2026-10-31)

base34 (the 24-run base on the freeboard merge, master f80d75a8, key 122e4803cded) did not exist yet when this arm
ran. The freeboard merge only turns PLT-660 and PLT-661 on, so the arm sedmargin (key 31c7c70bd7c7, argkey 5818ecce:
both switches set, seeds 1 to 4 of both starts, the crust-budget environment) is base34-s4's physics, and its eight
final states are base34-s4's bit for bit (`state_sha256` equal run by run, checked 2026-10-31); the two wedges were run
against it with `--set tectonics.collision_two_wedges=true` (argkey c7aa72cc), and `pool.sh table --base base34-s4`
reads the same. Medians of 8, Mann-Whitney p in brackets:

| measure | sedmargin | two wedges |
|---|---|---|
| land | 0.238 | 0.214 (0.23) |
| largest landmass | 0.097 | 0.075 (0.028) |
| continental crust under the sea | 0.527 | 0.590 (0.13) |
| land above 2 km (target at most 0.07) | 0.013 | 0.017 (0.96) |
| coast with high ground (at most 0.11) | 0.064 | 0.101 (0.23) |
| continental crust's net change | +0.065 | +0.100 (0.010) |
| criteria met per run | 6.5 | 5.5 (0.16) |
| sea over the datum at the end, m | 169 | 221 |
| the guard's worst, m; steps beyond two Earth reliefs | 9,964; 0 | 9,948 (0.88); 0 |
| highest plate elevation, m | 4,288 | 4,476 (0.72) |
| the incoming plate's wedge, share of the kept crust | 0.054 | 0.545 |
| the overriding interior's share (PLT-652) | 0.072 | 0.051 |
| no-room crust stacked above the strength height (PLT-700) | 0.021 | 0.010 |
| incoming crust at the front above the sea (states under 30 Myr) | 0.320 | 0.543 |
| the overriding edge on the front above the sea | 0.336 | 0.409 |

The collision regime's land within 150 km of the sea 0.591 to 0.656, its median distance to the sea 121 to 104 km
(0.19; `collcoast`); the rifted margins' share of the high coasts 0.042 to 0.122 (`coastset`: the pro-wedge on the
incoming plate's rifted margin). The M5 breakdown on the census grid (`tools/review/m5diag`, Earth's SRTM15+ beside):
M5 0.067 to 0.099 (Earth 0.084), M4 0.013 to 0.017 (0.055), the high ground near the coast f_near 0.49 to 0.52
(0.115), the land within 200 km of the coast 0.374 to 0.457 (0.292); the coast within 200 km of a collision 0.052 to
0.069 of the coast (0.093), 0.48 to 0.44 of it high (0.135).

Reading. The mechanism works on dry margins as on drowned ones: the pro-wedge holds 0.545 of the kept crust, near
the 60 % of the shortening that Wolf et al. 2021 (p. 13) find on the pro side once the retro-wedge shortens, and the
guard is clean. The freeboard did not dry the colliding margins: with PLT-661 the distal margins are continental crust
under the sea, and they are what collides first (the incoming crust at the front above the sea fell from 0.586 on
base33 to 0.320 on sedmargin). The two wedges do not rebuild the mountains: for the same cross-section their crest
stands lower than the one-sided taper belt's (H = sqrt(2A / (cot α + cot β)) against sqrt(2A tan α), 0.78 of it at
the default slopes, the lane's arithmetic), and the pro-wedge fills drowned margins first.

## Where the high ground goes (the crust-budget maps, every 50 Myr after the first 100)

`tools/review/orogen/hiseries.py`: the land above 2 km, mean over a run's snapshots (median of the runs), is 0.048 on
base33 (24 runs), 0.027 on sedmargin and 0.022 with the two wedges; its largest 0.103, 0.071 and 0.051; the land above
4 km 0.0050, 0.0020 and 0.0008 on average, 0.022, 0.010 and 0.0056 at its largest. So Tibet-class ground forms and
does not last. One run shows it (`orostats.py`, sedmargin supercontinent-1): a collision from −166 to −104 Myr (front
5,529 km, 5,848 km of shortening) left 0.065 of the land above 2 km at −100 Myr, 724 samples (about 3.6 million km²)
above 4 km on crust 65 to 68 km thick; 100 Myr later the land above 2 km was 0.011.

The wear: on two runs of each arm (the summary's `erosion_bands`), flat high ground (above 2 km, relief under 500 m)
loses 109 to 129 m/Myr of rock and steep high ground 465 to 509 m/Myr. The flat high ground is already dry or cold by
its Köppen class (`hiclimate.py`, medians over the runs: arid 0.475 on base33, 0.567 on sedmargin; polar 0.428 and
0.268). The Altiplano-Puna's internally drained interior erodes at 0.02 ± 0.0004 mm/yr (one sample; Bookhagen &
Strecker 2012, pp. 103-104) and southern Tibet's at under 0.01 km/Myr since 12-11 Ma (Tremblay et al. 2015, ERO-174).

## What Earth's orogens do (sources read)

- **Two wedges about S.** A pro-wedge over the lower plate at the minimum critical taper, a retro-wedge over the
  upper plate at the maximum (Willett, Beaumont & Fullsack 1993, p. 372); "more mass is stored in the pro-wedge than
  the retro-wedge" (Naylor & Sinclair 2008, p. 21); "Lp tan αp = Lr tan αr" (Roe, Whipple & Fletcher 2008, eq. 26).
  "Convergence is partitioned to 60% pro- and 40% retro-side deformation after onset of retro-wedge shortening"
  (Wolf et al. 2021, p. 13). PLT-730, PLT-731.
- **The plateau.** In Vanderhaeghe et al. 2003's models (a 35 km crust converging at 1 cm/yr over subducting mantle,
  the retro mantle fixed) the orogen stays back-to-back wedges until the base of the thickened crust weakens; then "a
  plateau with no significant surface slope develops between" a pro- and a retro-edge peak, and "In most models, the
  plateaux grow laterally at constant thickness between characteristic edge peaks" (p. 27, p. 41). The growth is
  retro-ward: "it is the net retro-ward flow that accounts for retro-ward growth of the plateau" (p. 41), and "the
  hottest crust is located below the retro-plateau" (p. 30); the pro-side edge stays near S (the research
  sub-agent's reading of their Fig. 11c). Wolf et al. 2021 (p. 25): a large hot plateau forms "located mostly on the
  retro-side". Medvedev & Beaumont (preprint, p. 3): "plateaus grow outward by tectonic thickening and accretion of
  adjacent crust in the transition zone between the plateau and the surrounding foreland". Molnar & Lyon-Caen 1988
  (abstract, PLT-700): "convergence can continue as the range builds outward, growing laterally into a high
  plateau".
- **What sets the plateau's height.** The flanking wedges' strength, not the weak layer under the plateau: "once a
  plateau develops ... the plateau crustal thickness is insensitive to the properties of the weak basal layer ... and
  is determined mainly by the properties of the cold crust" (Vanderhaeghe et al. 2003, pp. 28-29); Dahlen 1990 (p. 96,
  PLT-600): the brittle-plastic transition limits the height of very wide wedges. The model's strength height
  (PLT-253, 5.1 km at 1 g) stands for both.
- **Why a plateau lasts.** "Low internal relief, aridity, and internal drainage characterize the Puna-Altiplano
  Plateau and the Tibetan Plateau"; "large, internally drained areas persist indefinitely when uplift overwhelms the
  fluvial systems and defeats the channel network"; "Most coupled tectonic-landscape development models prevent the
  development of internal drainage and so may overestimate the voracity of erosion" (Sobel, Hilley & Strecker 2003,
  abstract). The low relief "reflects basin isolation from the foreland base level as erosional efficiency decreases
  in the arid plateau" (Strecker et al. 2009, pp. 644-645).

## The rules

**O1. The orogen is two critical wedges about the front** (PLT-730, PLT-731, built 2026-10-29). Unchanged. The
pro-wedge on the incoming plate's samples rides to the front and crosses whole (PLT-400), which stands for a wedge
that stays at the front while the lower plate passes under it: in the sources the orogen is fixed to S and the retro
mantle, so its pro-wedge is accreted lower-plate crust that no longer moves with the lower plate. Whether it goes on
by default is the coordinator's call on the arms (the table above and the cumulative arm).

**O2. Past the strength height the plateau grows retro-ward without a set width** (new, PLT-740,
`tectonics.collision_retro_plateau`, with O1 only). Today the retro flank is the overriding plate's continental
samples within `collision_belt_km` (PLT-241, 400 km, a `game-choice`) of the front; once all of them stand at the
strength height, what is left goes to the no-room rule (PLT-700): stacked above the strength height by the belt's
profile, nearest the front, where the step's ceiling (PLT-287) and the surface's channel flow (`surface::spread`) move
it on. With the switch the retro flank reaches as far through the overriding plate's continental crust as the step's
crust needs: when the fill leaves crust with every sample of the flank at its limit, the flank is searched again to
twice the reach (through continental samples of the overriding plate only: a plateau grows through its own crust, not
across a sea), until the crust is placed or the plate's connected continental crust is all taken. The fill is the same
bisection (one crest H, `min(h_s, H − d tan β)` on the retro side), so the plateau stands at the strength height with
its retro edge moving away from the front and the retro flank beyond it at β, and the pro edge stays at the front:
Vanderhaeghe et al.'s plateau "at constant thickness" growing retro-ward. Only what the whole connected crust cannot
hold goes to PLT-700. Nothing is created: the crust placed is the step's. The crust the plateau took beyond the belt's
reach is counted (`plates/collision_retro_plateau_km3`, crust budget column `plateau`; measurement only).

**O3. What limits the overriding plate's share.** No new rule; three things already do. (1) While the crest is below
the strength height the retro-wedge at the greater taper holds tan α / (tan α + tan β) of an even-ground
cross-section, 0.39 at the default slopes. (2) A narrow overriding block holds only what its retro flank reaches
before its far coast (its far side stands at H − w tan β for a block w wide, below the crest); the pro-wedge takes the
rest, as Taiwan's belt is the lower plate's margin with the arc as its backstop (Dahlen 1990, pp. 83-84; Simoes et al.
2007, p. 1). (3) Past the strength height every new step's crust goes to the retro plateau (O2): a mature large
orogen holds most of its crust on the upper plate, as Tibet does. The interior's thin-sheet share (PLT-652, PLT-733)
stays.

**O4. An interior plateau** needs a long, large collision (Wolf et al. 2021, p. 25: a large hot plateau "If crustal
shortening creates CA > ~12.5 × 10^9 m2", about 360 km of shortening of a 35 km crust, the lane's arithmetic), the
plateau's retro-ward growth (O2), and a surface that lasts. The model makes the first two in its large collisions
(above: 724 samples above 4 km); the third is the surface stage's: flat high ground that is arid by its climate class
still wears at 109 to 129 m/Myr, against the Altiplano's 20 m/Myr. That wear is the lever for a Tibet-class plateau at
a run's end, and it is not this stage's (STATUS Follow-ups, "High plateaus still wear too fast"; ERO-174); the
sources above (internal drainage defeating the channel network under aridity) are its lead.

## What it does not do

- Small terranes (Ben's third item): a fragment under 50 samples still docks for 0 to 2 Myr and is stacked by these
  rules.
- The lower crust going down with the slab (PLT-654, parked).
- The thermal trigger of the plateau (Vanderhaeghe et al. 2003, p. 43: the Moho above 700 °C after about 45 Myr at
  1 cm/yr; Wolf et al. 2021, p. 14): the strength height applies from a collision's onset, as before. Before that
  trigger the sources' wedges thicken without limit (Vanderhaeghe et al. 2003, p. 34), so a trigger would let young
  orogens stand higher, not lower; the model's orogens rarely reach the strength height (PLT-700's crust 0.010 to
  0.021 of the kept), so it is left out.

## How it is judged

Unit test (`belts.rs`, `the_retro_plateau_grows_past_the_belt_through_its_own_crust`): with the pro side and the
retro flank within the belt's reach at the strength height, the switch places the step's crust on the lowland beyond
`collision_belt_km`, at most at the strength height; a sea strip between two continents of the overriding plate stops
it; the crust kept is the same as without the switch; off, the no-room rule takes it and nothing beyond the belt
rises. One cumulative 8-run arm (O1 + O2) against base34-s4 (or
sedmargin, its physics), with `v1census`, `guard`, `collcoast` and `coastset`, the dock tally and the high ground
through time. No V1 number is a gate (decisions log 2026-10-23).
