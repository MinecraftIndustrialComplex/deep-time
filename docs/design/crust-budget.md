# The continents' crust budget against Stern & Scholl 2010: design

Status: **in progress** (2026-10-31, the crust-budget lane, branch `crust-budget`; the coordinator's
task after the freeboard landed, PR #49). Ledger PLT-760 onward; plates.md "The crust budget
against Stern & Scholl 2010, row by row (2026-10-31)" holds the readings and the arm.

## Why

With PLT-660 and PLT-661 on (Ben: "Turn both on now", decisions log 2026-10-31) the continents
drain: on the arm sedmargin (8 runs, key 31c7c70bd7c7, argkey 5818ecce; base34's physics) the net
change of the continental volume is −0.98 km³ a year (Earth about 0), and the thinning lane found
the drain real rock, not a label (sea-level.md "Why the continents thin with the margin's sediment
continental"). This design reads the budget row by row against its sources, finds in the code why
each row is where it is, and fixes what a source's law says the model does wrong. It adds no knob,
cap or forcing term to move a row towards Earth's number (Ben: "Results that don't align with earth
but are still perfectly playable and physics backed > Results that are forced to align with earth
by introducing extremely weird forcing functions").

## The budget row by row (sedmargin, km³ a year, means of 8)

`tools/review/loss-budget.py` against Stern & Scholl 2010's Table 1 (p. 12, the library text, read
again for this design):

| row | model | Stern & Scholl 2010 | why the model is there (the code) |
|---|---|---|---|
| intra-oceanic arcs | 1.05 | 1.5 | front 25,744 km (S&S 14,500) at 40.9 km³/km/Myr; 0.31 of the arc fronts make no arc (incipient, PLT-610 and PLT-681; past the slab's ends, PLT-561); v_n at the island fronts 43.6 mm/yr against PLT-666's 86.6, so the mature rate runs at half; the early pulse (PLT-668) ends within a step at the axis, when the arc's own crust turns its column continental (PLT-047's 8 km at 4.7 km/Myr) and the column takes the continental law; the felsic cones the arcs count inside their rate (PLT-286) were in neither arc row |
| continental arcs | 0.60 | 1.0 | front 29,436 km (S&S 27,500) at 20.4 against the law's 33.1 at v_n 55.5 mm/yr; the same band share and cones |
| crust-suturing zones (juvenile) | 0 | 0.2 | no rule |
| rifts, hotspots, volcanic rifted margins | 0.05 | 0.5 | the hotspots' own basalt only (their rules uncited, PLT-108 to PLT-120, `pending-rewrite`); no magmatic addition at breakup (PLT-294 trades the continental side's crust for ocean crust on the ocean side, adding none) |
| sediment subduction | 2.60 | 1.1 (+0.1 at sutures); Clift et al. 2009 1.65 | the trenches receive 2.90 km³ a year (the one-way slab's 0.33 of it all subducted); a quarter of the trench samples, piles over 1 km of rock (mean about 4.7 km), carry 0.9 of it; PLT-660 sends 0.83 down at accretionary margins and all elsewhere |
| subduction erosion | 0.49 | 1.4 (+0.3 at sutures); Clift et al. 2009 1.33 | erosive fronts 13,395 km (Clift & Vannucchi 2004: 22,925) at PLT-441's 90 asks 1.21; the forearc band holds 0.49 of crust beyond 7 km (`forearc_short` 0.72) |
| suturing losses | 0.50 | 0.7 | PLT-242's share at the crossing crust of 24 km |
| resampling drops, rate cap (model only) | 0.24 | 0 | numerical |
| net change of the continental volume | −0.98 | about 0 | the additions 1.70 (the cones not counted), the losses 3.83, and the flows back (accretion at resamplings 1.15, the transition 0.59) |

## The fixes

### 1. An intra-oceanic arc keeps the island law on the crust it made (PLT-760, built)

`tectonics.island_arc_own_crust`. The arc's law (island, PLT-045 and PLT-668; continental, PLT-285)
was the column's label: magma on oceanic crust took the island law, on continental crust the
continental. PLT-047 turns a column continental once its arc crust reaches 8 km, which the early
pulse (500 km³/km/Myr over the band's 106 km, 4.7 km/Myr at the axis) builds in 1.7 Myr, under one
2-Myr step. So at the axis a young island arc ran its pulse for one step of its ten Myr and then took
the continental arcs' 35 × v_n / 58.6; the rest of its band followed as each column passed 8 km.
Stern & Scholl's front-loaded law (p. 11: "500 km3/million years for first 10 million years plus 30
km3/million years for 60 million years thereafter") is for the intra-oceanic arcs they list, the
Izu-Bonin-Mariana and the Aleutians among them, arcs whose crust is far thicker than 8 km: the law is
the arc's, by its setting, not the label the model gives its crust.

The rule: a contact's arc takes its setting at its onset (PLT-669): intra-oceanic where none of its
oceanic fronts that step has visible continental crust of the overriding plate within
`arc_distance_km` (PLT-370's test), stored as `Contact::arc_intra_oceanic`, cleared with the onset at
a polarity reversal and kept by a split's copies. With the switch on, magma that lands on crust an
island arc made (`subduction::island_arc_crust`: ocean floor, or continental crust over an ocean
basement, PLT-331, whose own crystalline crust is at least half the island-arc magma it took) under
an arc that began intra-oceanic follows the island law (its pulse while young, then 30 × v_n / 86.6),
its pools, and the island accumulator. A continent's own crust under any arc, and every column under
an arc that began at a continent, keep the label's law. The column's height still follows its kind
(Airy on continental crust, the ocean's line on oceanic). The "at least half" is a game-choice line
(a matured arc's own crust is its arc magma less what eroded, so it holds at any share up to one).

Test (written first): `an_intra_oceanic_arc_keeps_the_island_law_on_the_crust_it_made`.

### 2. The additions measured whole (PLT-761, built)

The arcs count the volcano stage's felsic cones inside their rate (PLT-286: magma less the cones,
the cones in the pools at the feedback), so `arc_c` and `arc_o` left the cones out and the budget's
arc rows were short by them (all felsic cones together 0.31 km³ a year on sedmargin). New globals:
the cones each arc counted, by the arc's law (`arc_cones_c`, `arc_cones_o`); the hotspots' basalt on
continental crust and the cones they counted (`hotspot_magma_c`, `hotspot_cones`,
`hotspot_cones_c`); the intra-oceanic arcs' front past incipient (`front_io`). loss-budget.py adds
the cones to the arc rows and reads the hotspot row on continental crust (S&S's additions are to
continental crust; ocean floor's seamounts go down with their slab, PLT-286).

### 3. Sediment subduction

(after the research sub-agent's report)

### 4. Additions at breakup and at sutures

(after the research sub-agent's report)

## What is left

(at the end)
