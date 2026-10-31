# A hotspot's cones and its tail as one budget: design

Status: **on by default since 2026-10-31** (the coordinator's call, decisions log 2026-10-31: a
books fix for the default; built 2026-10-31, the hotspot lane, branch `hotspot-lip`):
`tectonics.hotspot_cone_budget` (PLT-803), with the measurement PLT-804. A correctness fix of the
default as much as of the sourced tails (section 1): it holds whatever `hotspots_sourced` says. The
arms are in section 5, the review's fixes in section 6.

## 1. Why: rock no rate made

Plate task 8 (2026-10-17, PLT-286) hands the volcano stage's cones to the plate model's pools as
volcanic rock, so that eroding a cone strips its own basalt and not the crust under it, and counts
them inside the sourced rates: an arc's cones against its volcanic share, a hotspot's against its
own basalt. A hotspot counts the cones that stand on the samples of its 150 km kernel (PLT-111) at
the step they are handed. But the volcano stage builds a hotspot's volcanoes at the game's size and
not by the plume's flux:

- a new one once the hotspot's earlier ones have drifted `hotspot_spacing_km` from the plume
  (VOL-007), 70 km of the game's metres;
- a shield 3.5 to 6.5 km high by the plume's strength (VOL-021), its basal radius 40 km at the top
  of the range, scaled by height to the power 0.8 and divided by `flank_steepness` 0.5 (VOL-022): 49
  to 80 km in the game's metres. On the 16k planet (radius 782 km) that is 400 to 650 km on Earth's
  sphere (the agent's arithmetic), far past the kernel; on continental crust a silicic centre, 70
  km.

What lies outside the kernel counts against nothing. Measured on cached runs
(`tools/review/crustcycle/cones.py`, km³ a year, Earth-normalized):

| | cones handed | of them mafic | counted by arcs | by hotspot kernels | flood-basalt plateaus | counted by nothing | the hotspots' own basalt |
|---|---|---|---|---|---|---|---|
| base34 (master f80d75a8, 24 runs) | 0.83 (0.56 to 1.08) | 0.52 | | | 0.02 | | 0.048 |
| base34's worlds, measured (cbt-base, 2 runs; hs-off, 1) | 0.82 | 0.53 | 0.23 | 0.06 | 0.02 | 0.50 (0.46 to 0.54) | 0.055 |
| base34-s4's worlds with PLT-804 (hs-base, 8 runs, after the build) | 0.84 (0.72 to 0.99) | 0.53 | 0.25 | 0.06 | 0.02 | 0.50 (0.39 to 0.63) | 0.049 |
| cb-cum2 (8 runs) | 0.79 | 0.46 | 0.27 | 0.06 | 0.02 | 0.45 (0.30 to 0.53) | 0.043 |
| hs-cum (8 runs, Sleep's 37 tails) | 5.00 | 4.4 | 0.25 | 0.60 | 0.33 | about 3.8 | 0.94 (the LIP domes in) |

(base34's runs predate the arcs' and hotspots' cone globals, PLT-761; cbt-base, hs-off and hs-base
are base34's worlds bit for bit with them.) So master's default hands about 0.5 km³ a year of rock no
rate made, ten times the hotspots' own basalt and as much as Stern & Scholl's whole row for rifts,
hotspots and volcanic margins; nearly all of it mafic, so shields, which only hotspots build (the
felsic cones the arcs did not count are at most 0.06). The cones handed to continental crust are
booked in the surface stage's part of the crust budget, with erosion (`apply_surface_feedback`), so
loss-budget.py's row "off the continents by the surface stage" nets them: on base34-s4's worlds
(hs-base) the cones handed to continental crust are 0.31 km³ a year (0.27 to 0.37), the row reads
3.82 and the erosion alone 4.13. The hotspots' cones alone hand 0.64 km³ a year there (PLT-804's
volcano-stage global), thirteen times the hotspots' own basalt (0.049).

## 2. The rule (`tectonics.hotspot_cone_budget`, PLT-803)

A hotspot's cones are its tail's erupted rock. The tail's magma over a step is its rock R (sourced:
PLT-793's magma over the step; the preset's: PLT-115's three times the rise over the kernel), of it
the lavas a share e (sourced: a seventh, PLT-794; the preset's rock is all lava, e = 1). On the
default (the preset's hotspots) R is PLT-115's three times the rise under PLT-112's rate and PLT-114's
ceilings, all three uncited: the budget makes the cones agree with the tail's books, and the books
are no better sourced than the preset's hotspots are. So:

1. **The budget.** After a plate step each live tail that lives through the next step leaves its
   cones e R of that step to hand (`Hotspot::cones.budget_km3`, Earth-normalized km³); a tail that
   dies before the next step leaves none, since nothing would net them.
2. **Counted by the cone's own hotspot.** In the volcano stage's next frame, a hotspot's volcanoes
   (`Volcano::hotspot`, shields and silicic centres alike) build their relief as now. The rock their
   growth would hand, B (the cells' metres times their area: the L1 grid is Lambert's equal-area
   projection, every cell `cell_m`² of the game's sphere, scaled to the model sphere by (R_model /
   R_game)²), is handed at the share f = min(1, budget / B); the cone's relief stays whole. Each
   volcano keeps the rock it holds (`Volcano::rock_m` of its `built_m`), so a caldera's collapse
   takes back, and its ignimbrite sheet spreads, only that share of rock. Each hotspot's cones hand
   by its own budget, and a growth whose footprint holds no cell hands nothing and backs nothing
   (whatever its hotspot's other cones handed). What the hotspot's cones handed in the frame, net
   of calderas, is written on the hotspot (`cones.handed_km3`, and of it on continental cells),
   wherever it lies. Their rock reaches the plate model apart from the other cones'
   (`surface/volcanic_hotspot_felsic_m`, `surface/volcanic_hotspot_mafic_m`): the pools take it as
   any cone's rock, but it is not among the cones an arc or a kernel counts inside its rate by place
   (`apply_surface_feedback` returns only the others), since the tail nets it by hotspot.
3. **Netted by the tail.** At the next plate step the tail adds R − C of its rock, C what its cones
   handed: its intrusions (1 − e) R whole and its lavas e R − C (with the step's lag a shortfall
   comes off the intrusions). C is netted between nothing and the step's R, so the tail never adds
   less than nothing nor more than its rate; the rest carries to the next step, of either sign:
   cones past the step's R, cones over a step with no rock (every kernel column at PLT-114's
   ceiling, or none visible), a caldera's take-back beyond what the cones had handed since. A dead
   tail nets nothing: what its record holds, at its death and from its cones' calderas after, leaves
   it as unnetted (`plates/hotspot_cones_unnetted_km3`). The kernel counts no cone by place. So over
   a tail's life the pools hold its rock R, the cones' share included, and no cone hands rock its
   tail did not make, but for the unnetted.
4. **The rest of a cone is relief.** The part of a cone's height its rock does not back stands as
   relief over rock the plate model already holds (the ocean crust and the tail's intrusions under
   it); the surface's erosion of that relief strips the column under it, as eroding any relief does.
   This brings back, for the budgeted shields only, what plate task 8 removed for every cone (an
   eroded cone stripping crust it did not add); the alternative, rock from nothing, is the larger
   error (section 1). How much of that relief stands above the sea, where the rivers wear it, has
   not been measured.

Not changed: the arcs (their cones outside the arc band, at most 0.06 km³ a year, are a separate
fix), the flood basalts' plateaus (a province's record volume, PLT-796), the islands' subsidence
(no rock), the cones' shapes and heights (VOL-021, VOL-022: the game's).

## 3. Measurement (PLT-804, always on; globals only, the world unchanged)

- `plates/cone_continental_km3`: the cones' rock the pools took on continental crust (felsic and
  mafic), so the surface stage's part of the crust budget can be read with the cones out.
- `volcanoes/hotspot_cones_built_km3`: what the hotspots' cones would hand at full (net of
  calderas); `volcanoes/hotspot_cones_handed_km3` and `..._handed_continental_km3`: what they handed
  (the same as built with the switch off).
- With the switch: `plates/hotspot_cones_by_id_km3` (and `..._continental_km3`), the cones the tails
  netted by their hotspot; `plates/hotspot_tail_rock_km3`, the tails' rock R before netting;
  `plates/hotspot_cones_unnetted_km3`, what dead tails' records held that no tail netted. The
  kernel's `plates/hotspot_cones_km3` stays 0.

## 4. Tests

- `a_budgeted_tail_nets_its_cones_by_their_hotspot` (plates/tests.rs): a Hawaii tail with cones
  handed for it adds R − C, its lavas e R − C, its intrusions (1 − e) R; the kernel counts none of
  the cones on its samples; it leaves e R as the next frame's budget, none if it dies before the
  next step; a take-back beyond what was handed is carried and the tail adds its whole rock; a step
  with no rock carries what was handed, of either sign; a dead tail's record leaves it as unnetted.
- `a_budgeted_hotspots_cones_join_the_pools_but_no_rate_by_place` (plates/tests.rs): the same rock
  on `surface/volcanic_felsic_m` and on `surface/volcanic_hotspot_felsic_m` leaves the pools and
  the cone globals the same, and only the former among the cones the arcs and kernels count.
- `the_l1_cells_cover_the_model_sphere_and_cones_hand_at_most_their_budget` (volcanoes/tests.rs, one
  test): the share f, and the cells' Earth-normalized area summing to the model sphere's.
- `two_hotspots_cones_hand_and_take_back_each_their_own_share` (volcanoes/tests.rs): two hotspots'
  cones hand by their own budgets (half and all), apart from the other cones' rock; `rock_m` is the
  share of each growth, none for a footprint without cells; the calderas' share and their
  take-back and sheet from the right rock.
- `a_hotspots_cones_hand_the_pools_at_most_its_tails_lavas` (volcanoes/tests.rs, a 60 Myr pipeline
  with Sleep's tails): the cones handed by id at most a seventh of the tails' rock, the cones built
  more than handed, and the tails' basalt plus their cones the tails' rock.

## 5. The arms

In plates.md "Hotspots, plume heads and volcanic margins from sources (2026-10-31)", the
subsection "The cone budget". Key b58b6e4506f5 (a92a07f0), 8 runs each:

- hs-base (every switch off): base34-s4 bit for bit; the cones hand 0.84 km³ a year, 0.50 counted
  by nothing, the hotspots' cones 0.64 against their own basalt 0.049; 0.31 on continental crust
  (the surface stage's row 3.82, the erosion alone 4.13).
- hs-budget (the crust-budget switches and the budget, against cb-cum2): the cones handed 0.79 to
  0.34, the uncounted 0.45 to 0.03; the net change of the continental volume −0.37 to −0.50 km³ a
  year; no V1 number moves (p ≥ 0.19), criteria 6 to 6, the sea 348 to 307 m; the guard clean.
- hs-cum2 (with the three hotspot switches, against hs-cum): the hotspots' cones 6.00 at full, 0.05
  handed; the net +3.11 to +0.53, the sea 1,321 to 679 m, criteria 4 to 4.5 (cb-cum2 6); the guard
  clean. The tails still put 0.43 km³ a year on continental crust: the lid is next.

Verdict: on by default, whatever the hotspot switches say (a correctness fix of master's default).
The coordinator turned it on on 2026-10-31 (decisions log); the hotspot lane's three switches stay
off until the lid is built.

## 6. The review's fixes (an Opus review of 4d9ea6f5, 2026-10-31)

- **Netted twice.** `fronts_finish` runs before the hotspots and nets every felsic cone within
  `subduction_distance_km` of an oceanic slab's front into the arc's rate; silicic centres and every
  caldera's ignimbrite are felsic, so a continental hotspot's near a front was netted by the arc and
  by its tail (on hs-budget 0.004 to 0.007 km³ a year, about a fifth of what the tails netted by id,
  the reviewer's reading; loss-budget.py counted it in both `arc_cones_c` and `hot_by_id_c`, cones.py
  subtracted it twice). The budgeted hotspots' cones now reach the plate model apart (section 2,
  item 2), and the kernel no longer reads the cones at all under the budget.
- **The pending's carry.** A negative pending (calderas taking back more than was handed) was
  netted uncapped (the tail added more than its rate), cleared on a step with no rock while a
  positive one was carried, and dropped at a tail's death. Now netted between nothing and the
  step's rock, carried of either sign, and booked as unnetted at death (section 2, item 3).
- **A footprint without cells.** A cone whose growth held no cell gained `rock_m` at its hotspot's
  share when a sibling cone had cells, so its caldera would take back rock never handed. Only a
  growth that held cells now backs rock.

These change the runs with the budget on (none of the arms above is bit for bit after them); the
default without the budget is unchanged.
