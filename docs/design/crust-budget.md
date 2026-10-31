# The continents' crust budget against Stern & Scholl 2010: design

Status: **built behind switches, off** (2026-10-31, the crust-budget lane, branch `crust-budget`; the
coordinator's task after the freeboard landed, PR #49): PLT-760 to PLT-763; section 5 designed only. Ledger PLT-760 onward; plates.md "The crust budget
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

## The budget row by row (base34-s4, km³ a year, means of 8)

`tools/review/loss-budget.py` against Stern & Scholl 2010's Table 1 (p. 12, the library text, read
again for this design), on base34's seeds 1 to 4 (master f80d75a8, key 122e4803cded), whose budget
is the arm sedmargin's to the last digit (the same switches on the same planets). Over base34's 24
runs the net is −1.06, and over the late half (`--from -500`) −1.33, with sediment subduction 2.93
and the arcs 1.48:

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

### 3. Sediment subduction: the share stays; the line is read in place (PLT-762, built)

The row is 2.4 times Stern & Scholl's 1.1, and the first reading was that PLT-660's 0.83 at
accretionary margins was the wrong share: Stern & Scholl (p. 15) write that "large accretionary
bodies at sediment-dominated margins sequester approximately 70% of all sediments entering
subduction zones". Read against its sources, it is not:

- von Huene & Scholl 1991 (the abstract, the research sub-agent) give 70 % as the share
  *subducted* at large prisms ("roughly 70% of the incoming trench floor section is subducted
  beneath the frontal accretionary body"), 80 % at small and medium prisms, all at type 2 margins.
  Stern & Scholl's sentence reads it the other way round (the sub-agent: probably garbled;
  Scholl & von Huene 2007's Table 2 was not obtained).
- Clift & Vannucchi 2004 count underplating as accretion ([8]) and take the subducted share as the
  delivery less the prism's growth ([38]); their 83 % is what passes the forearc wedge to depth
  ([39], "at least 50 km below the forearc wedge"). Clift et al. 2009's Table 1 (read): of the
  1,768,761 km³/Myr delivered to 40,225 km of trenches, 1,648,091 go down, 0.93.

So the model's share of what reaches its trenches, 2.60 of 2.90 (0.90), is Clift's. What differs is
how much reaches them, per km of trench:

| per km of trench, km³/km/Myr | base34-s4 | Clift et al. 2009 | Stern & Scholl 2010 |
|---|---|---|---|
| sediment delivered | 52.7 (2.90 km³/yr over 55,180 km of front) | 44.0 (40,225 km) | |
| sediment subducted | 47.1 | 41.0 | 25 (1.1 AU over ~43,000 km) |

The model's trench is longer (55,180 km of oceanic front, Bird 2003's subduction zones 51,310)
and each km takes 1.2 times Clift's. The sources disagree by 1.6 times between themselves, and
von Huene & Scholl's long-term 1.0 km³/yr corrects for "high rates of terrigenous seafloor
sedimentation during the past 30 m.y." So the sediment row is within Clift's per-km physics;
nothing is changed in the share, and no rule is added to lower it.

One units error is fixed: Clift & Vannucchi's 1 km (PLT-660's accretionary pile and PLT-442's
erosive margin's cover) is "the thickness of sediment (uncorrected for porosity) on the subducting
plate at the trench axis" ([31]), and the model's pools are rock (surface/pelagic.rs: no pore
water). `tectonics.trench_pile_in_place` reads the line as the rock a 1 km porous column holds
under ERO-215's porosity law, 536 m (C&V used Sclater & Christie's sand-shale mix, not read; the
model's own law stands in). On base34-s4's maps 0.065 of the trench samples hold 536 to 1,000 m of
rock and 0.04 of the trench sediment (`tools/review/crustcycle/probe.py`): a small change, and the
erosive fronts shrink by the same samples.

Test: `the_trench_pile_line_is_read_in_place`.

### 4. Arc magma at sutures (PLT-763, built)

`tectonics.collision_arc_magma`. Stern & Scholl's Table 1 books 0.2 AU of additions at
crust-suturing subduction zones; Scholl & von Huene 2009 (the abstract): "During collisional
suturing at fossil CSSZs, global arc magmatic addition is estimated at 0.2 AU. This assessment
presumes that in the past the global length of crustal collision zones averaged c. 6000 km".
The model's arcs need an oceanic slab, so a collision front made none. The rule gives a collision
front's overriding band (PLT-042's band, past the slab's ends none, none while incipient) 0.2 AU
over 6,000 km, 33.3 km³/km/Myr (the agent's division, as PLT-045's 100 is 1.5 AU over 14,500 km),
at any convergence, into the arc pools of its crust's law, booked as `arc_s` and in neither arc
row. The model's collision front at any time is 18,411 km on base34-s4 (Stern & Scholl: 6,000
long-term, 12,000 today; Bird 2003's continental convergent boundary 23,003 km), so at the
source's per-km rate it would add about 0.6 km³ a year, three times the 0.2: the model's sutures
are longer than Scholl & von Huene's average, which is the collision lane's finding, not this rule's
to correct.

Test: `a_collision_front_adds_scholl_and_von_huenes_suture_arc`.

### 5. Rifts, hotspots and volcanic rifted margins (designed, not built)

The row is 0.05 against 0.5, and the sources give no rate for it: Stern & Scholl pool rifts,
hotspots and volcanic rifted margins at "approximately 0.5 AU" with no derivation (the research
sub-agent found none either); Clift et al. 2009 give continental flood basalts 0.12 km³ a year
with their intrusive roots, and oceanic plateaus 1.1 if every one accreted; Schubert & Sandwell
1989 an upper bound of 3.7 for plateau accretion. What a volcanic margin adds is measured on two
margins: the US East Coast, "up to 25 km" of igneous crust "only for about 80–100 km seaward", "as
much as 2.7 × 10^6 km3" over 2,000 km, in "only 5–8 m.y." (Kelemen & Holbrook 1995, abstract),
about 1,350 km³ per km; the North Atlantic, a crustal volume of 6.6 × 10^6 km³ over more than
2,600 km of margin in about 3 Myr (Eldholm & Grue 1994, abstract), about 2,500 km³ per km with
the onshore basalts. Volcanic margins are "the majority of passive continental margins worldwide"
(Franke 2013), half in Clift et al. 2009's own count of subducted margins.

The rule this supports: at a resampling, a gap point whose nearest edge sample is continental crust
(the first ocean floor between two continents that rifted apart) and that lies within 100 km of it
(Kelemen & Holbrook's 80 to 100 km) takes igneous crust to 20 km instead of the ocean column's 7
(mafic plutonic under mafic volcanic, PLT-104's proportion), as continental crust over its ocean
basement (PLT-331), on a volcanic margin. It needs a sourced test for which breakups are volcanic.
White & McKenzie 1989 tie the igneous thickness to the mantle's temperature (the abstract only;
the law is in the full text, not obtained), and the model's own hot mantle at a breakup is its
plume heads (PLT-117's LIPs, which raise the rifting rate of the plate above, `lip_rift_boost`),
whose rules are uncited (PLT-108 to PLT-120, `pending-rewrite`). So this addition waits on the
hotspots' rewrite: a sourced plume flux, which would give both the hotspots' own addition and the
volcanic margins. Not built: a 50 % draw per breakup would be a forcing term, not a law.

## What is left

- The hotspots' rewrite with sources (plume births, fluxes, LIP volumes), then the volcanic margins
  of section 5 on it.
- Subduction erosion (0.49 against 1.4): the erosive front is 13,395 km against Clift &
  Vannucchi's 22,925 at the planet's convergence (PLT-442's 60 mm/yr), and the forearc band holds
  0.40 of what PLT-441 asks (`forearc_short` 0.72 km³ a year): an intra-oceanic forearc has nothing
  beyond its 7 km of ocean crust to give. Earth's erosive forearcs retreat into the arc (Scholl &
  von Huene 2010, ~2.5 km/Myr), `tectonics.erosion_retreat` (PLT-523, off); a loss, so it would
  deepen the drain.
- The land's export: the trench takes what the continents shed (3.8 to 4.2 km³ a year off them
  against Clift et al. 2009's Cenozoic rivers' 1.8), the surface lane's.
