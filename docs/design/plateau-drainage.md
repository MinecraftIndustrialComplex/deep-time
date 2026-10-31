# What wears the plateaus, and lakes that pass on what their water balance leaves: design

Status: **measured on cached runs, designed and built; on by default since 2026-10-31** (the coordinator's call, decisions log 2026-10-31; its arm ran, 2026-10-31, the plateau lane,
branch `plateau-drainage` on `collision-orogen`, built and off; branch `plateau-drainage-on`, on by default with the
review's fixes, R3 and "The review"). `surface.closed_basins` (`crates/deeptime-core/src/surface/closed.rs`;
ERO-217, ERO-218), with a split of the flat high ground by its water (ERO-220). The controlled plateau is ERO-219. The
arm plat-cb0 against orog-cum ("The arm"): the law is sound and cheap and gives the planets internally drained land,
but it does not keep the plateaus; their flat tops wear on frozen ground, whose runoff is the next lead.

## Why

The orogen lane (docs/design/collision-orogen.md, O4; STATUS build log 2026-10-31): Tibet-class high ground forms in the
largest collisions (one base34-s4 run had about 3.6 million km² above 4 km at -100 Myr) and is gone 100 Myr later; flat
high ground wears at 109 to 129 m/Myr though its Köppen class is already arid or polar, against the Altiplano's 0.02
mm/yr; land above 4 km is about 0.0013 of the land against Earth's 0.0195 (coast-high-ground-audit-2026-10-30.md). The
gap is old: the erosion recalibration of 2026-10-18 left "Plateau wear, still a gap" (the controlled plateau's interior
at 131 m/Myr against southern Tibet's under 10, ERO-174; STATUS follow-up "High plateaus still wear too fast"), the
measured hillslope diffusivity alone took the flat high ground only from 63 to 46 m/Myr (ERO-177), and the critical
slope's scale is Montgomery & Brandon's (ERO-013). So the question here is which term wears a plateau's flat top, before
any law is chosen, with K's calibration kept (Ben: "Results that don't align with earth but are still perfectly playable
and physics backed > Results that are forced to align with earth by introducing extremely weird forcing functions").

## What wears the plateaus (cached runs only)

Runs: base34 (key 122e4803cded, argkey cd665cef, both starts x seeds 1 to 12) and orog-cum (key f2d6412d801f, argkey
d3261b0a, seeds 1 to 4, the two orogen switches). Tools: `tools/review/plateau/` (`bands.py`, `cohort.py`,
`climate.py`, `closed.py`, all run by `wear.sh`), on the VM:
`tools/remote/mac.sh -- tools/remote/detach.sh start plateau-wear --kind light bash tools/review/plateau/wear.sh
orogcum=<folder> base34=<folder>` (`ONLY=<script>` for one of them), output in `/data/ban/deeptime-runs/plateau/`.
Medians of the runs.

### The surface's processes on the flat high ground

From each run's `erosion_bands` line (`surface::erosion_bands`, whole-run area x time sums; "flat" is land above 2 km
whose 3 x 3 relief at the step's start is under 500 m). m/Myr:

| class | arm | land share | net | stream power (of it the critical slope's cut) | diffusion | ice | rest |
|---|---|---|---|---|---|---|---|
| flat high ground | base34 | 0.0052 | 137.8 | 103.2 (49.2) | 31.2 | 9.4 | -6.8 |
| flat high ground | orog-cum | 0.0027 | 134.3 | 97.0 (50.8) | 37.1 | 9.8 | -7.3 |
| steep high ground | base34 | 0.0199 | 516.3 | 465.2 (242.7) | 55.5 | 6.2 | -10.9 |
| above 4 km | base34 | 0.0020 | 500.1 | 443.4 (409.8) | 57.1 | 12.6 | -9.2 |
| above 4 km | orog-cum | 0.0011 | 732.9 | 649.0 (630.2) | 85.3 | 12.9 | -12.0 |

Of the flat high ground's gross lowering, stream power without the cut is 0.373 and 0.366, the cut 0.340 and 0.316,
diffusion 0.214 and 0.241, ice 0.066 and 0.060; per run its net is 114.6 to 180.9 (base34) and 113.9 to 227.7 m/Myr.
"Rest" (net less the gross processes: the dissolved load and the dust less the deposition on the class) is a few m/Myr:
almost nothing the flat high ground loses is laid back on it.

Reading. The cut acts on a cell's link to its receiver when, after the solve, the cell stands more than S_c = 0.15 times
the link above it (`spl.rs`), 720 m over a 4.8-km link. A cell that was flat at the step's start (under 500 m of 3 x 3
relief) passes that only if its receiver was lowered in the same solve, by its own stream power or by the cut below it:
on the flat high ground the cut is the threshold hillslope behind an incising river, the plateau's dissection by its
drainage, not a wear of its own (on the controlled plateau, where no plate step raises anything, it cuts 0.6 m/Myr of
131, ERO-176). So the plateau's flat top is worn by **stream power through its drainage, with the cut it drives (0.69 to
0.71 of the gross), then diffusion (0.21 to 0.24) and ice (0.06 to 0.07)**. Above 4 km the cut is 0.83 of the gross, on
ground that is steep at the step's start.

### Where the rivers that wear it get their water

At the run's end (`climate.py` on the dumped final fields, the flat high ground's cells): arid by Köppen 0.356 (base34)
and 0.680 (orog-cum), polar 0.519 and 0.064; below 0 °C mean 0.604 and 0.200, of which under the climate's ice of 100 m
or more (where the surface's rivers do not erode, `surface/mod.rs`) the flat high ground's 0.392 and 0.013; below 0 °C
without ice (under 10 m) only 0.004 and 0.029. So the frozen-ground runoff of `surface/climate.rs` (PET 0 below 0 °C, so
all of P runs off, ERO-010) reaches almost no ice-free flat high ground: the cold half of base34's plateaus is under ice,
and its precipitation reaches the rivers below as the ice's melt would. The flat high ground the rivers wear is the
ice-free, mostly arid part; the controlled plateau with an arid interior (below) wears at 3.8 m/Myr, not 134, so on the
planets its rivers carry water from elsewhere: wet or glaciated ground upstream, and every depression's whole catchment
over its spill. **Corrected by the arm:** this was one frame, the run's end, and the whole run says otherwise. The
erosion bands' split over every step (`flatf`, "The arm") puts 0.67 of the flat high ground's area x time below 0 °C,
wearing at 141 m/Myr, and the two arms' final frames disagree (below 0 °C 0.81 and 0.20 of the flat high ground).

At the run's end (`closed.py`: a priority flood from the sea on the final surface, then the steepest descent), 0.224
(base34) and 0.340 (orog-cum) of the land above 2 km, 0.340 and 0.544 of the flat high ground and 0.278 and 0.307 of
all land drain into a depression deeper than 1 m; the largest such catchment is a median 340 and 306 cells (7,800 and
7,000 km² on the map, about 0.5 million km² Earth-normalized). On the plate model's own elevation (before the surface's
offset) 0.209 and 0.313 of the land above 2 km does. Every one of them spills today (ERO-080: the flood routes every
closed basin over its rim): the surface makes depressions on its high ground, and none of them is internally drained.

### The plates' thinning against the surface's

The crust-budget maps every 50 Myr carry each visible sample's thickness, elevation, collision stamp and rock pools
(`plates/crust_budget.rs`). `cohort.py` follows the samples a collision stamped at the same steps (orogeny 2, the same
stamp times) from one map to the next, and splits their thinning by its mark in the pools: the surface's erosion strips
the column from the top and never the mafic plutonic lower crust (`pools::strip`), the plates' collapse and
lower-crustal flow move the column's own mix ("thinned in proportion", `plates/collapse.rs`), so the lower crust's fall
is the plates' share. Weighted by the high ground at the first map, m/Myr of crust:

| high ground | arm | thinning | the plates (in proportion) | the top (erosion less deposition) | surface height | cohort kept |
|---|---|---|---|---|---|---|
| above 3 km | base34 | 138.2 | 78.5 | 59.7 | -19.0 | 0.46 |
| above 3 km | orog-cum | 112.5 | 58.9 | 53.6 | -15.9 | 0.50 |
| above 4 km | base34 | 190.2 | 131.2 | 59.0 | -25.4 | 0.37 |
| above 4 km | orog-cum | 172.1 | 119.5 | 52.6 | -23.4 | 0.42 |

On the cohorts that kept 0.8 of their samples or more (no later stamp), the plates take 65.7 and 46.2, the top 53.3 and
51.1 m/Myr. So **the plates take 0.57 (above 3 km) to 0.69 (above 4 km) of a plateau's crust after its collision**, the
surface 0.31 to 0.43. At most snapshots 0.75 or more of the high ground was stamped under 10 Myr before (302 of 424 pairs
on base34): it stands while its collision holds it (the collapse waits 10 quiet Myr, PLT-259), and the collapse after
that is sourced post-orogenic collapse (the Variscan root thinned in about 25 Myr, PLT-390; Tibet's lithosphere at
10^22 Pa s, PLT-414). A plateau gone 100 Myr after its collision ended is what those sources expect, so the surface's
part is what sets how much plateau stands while a collision lasts and how much of it reaches the sea. The top's 51 to
60 m/Myr of rock lowers the surface by about a seventh of it (Airy, PLT-281's 7.02). Limits: a resampling's blend mixes
a cohort's samples with their neighbours', and crust a cohort's sample gains from another (the flow) counts against its
loss; per-sample means, and samples a later orogeny stamps leave the cohort.

### Against the continents' sediment flux

The land's mean net lowering (the bands' area x time shares times their net) is 40.1 m/Myr on base34 and 37.1 on
orog-cum; the flat high ground is 0.018 and 0.011 of it, the steep high ground 0.255 and 0.239, the land under 1 km 0.524
and 0.535, 1 to 2 km 0.202 and 0.218. The surface sends 3.86 and 3.03 km³ a year to the sea (Earth-normalized, the run's
`to_sea_km3` over 1,000 Myr). So the plateaus' own wear is 1 to 2 % of what the continents shed, and the continent-wide
excess the crust-budget lane found (3.8 to 4.2 km³ a year to the trenches' catchments against Clift et al. 2009's 1.8;
docs/design/crust-budget.md on branch `crust-budget`) is not this term: it is the land under 2 km (0.72 to 0.75 of the
lowering) and the steep high ground (0.24 to 0.26), whose regions erode at Montgomery & Brandon's rates since the
recalibration (ERO-179, ERO-180; inferred, not measured here). What the law below adds there is internal drainage on
arid lowland too: on Earth "Endorheic basins occupy 20% of the Earth's land surface but they collect only about 2% of
global river runoff" (Garcia-Castellanos et al. 2003, p. 8-1), and none of the model's land is.

## What Earth's plateaus do (sources read)

Read by a Sonnet research sub-agent on 2026-10-31; quotes as it printed them (bibliography, "Internally drained basins").

- **The plateaus are internally drained, and their interiors barely erode.** "Low internal relief, aridity, and
  internal drainage characterize the Puna-Altiplano Plateau and the Tibetan Plateau" (Sobel, Hilley & Strecker 2003,
  abstract). Southern Tibet's peneplain: "low local and catchment-wide erosion rates of 6-11 m/m.y. and 11-16 m/m.y."
  (Hetzel et al. 2011, p. 983); local rates "cluster between 5 and 11 m Ma-1 (mean = 8.1 m Ma-1)", catchments "9 to 16 m
  Ma-1 (mean = 11.8 m Ma-1)", and "As the peneplain region belongs to the internally drained part of the plateau (Fig.
  1), the sediment produced cannot leave the interior of the plateau" (Strobl et al. 2012, abstract and section 5.3).
  The Altiplano-Puna: "erosion rates on the internally drained plateau (0.02 ± 0.0004 mm/yr, n = 1)" against "0.50 ±
  0.3 mm/yr (n = 15) at the steep east-facing orographic barriers, where mean annual rainfall rates are up to 2 m/yr"
  (Bookhagen & Strecker 2012, p. 103). Lal et al. 2003 (abstract): "low erosion rates of <30 mm/ka in southern and
  central Tibet".
- **A closed lake's level is its water balance.** "The balance is expressed by the equation I + P A_L = E' A_L"
  (Langbein 1961, p. 4); "In absence of significant underground flow, the level of a closed lake is determined by the
  water balance between precipitation and evaporation, which in a steady state situation can be written as P . Wcb = E
  . Wcl" (Garcia-Castellanos 2007, p. 387, eq. 12); "If the lake water level reaches the outlet, drainage becomes open
  and outlet incision leads to the integration of the lake into the drainage network (drainage capture). During
  capture, the output discharge of the lake is P . Wb - E . Wl" (the same page). "the water discharge leaving a lake is
  determined by the water balance between precipitation and evaporation, which for a steady-state water flow can be
  written as Qw(x = 0) = P . lb - E . ll" and "By definition, for a closed lake Qw(x = 0) = 0" (Garcia-Castellanos 2006,
  pp. 285-286, eq. 2). "Previous computer models of long-term surface transport did not account for evaporation ...
  evaporation at the surface of lakes must be explicitly incorporated if closed drainage basins are to be generated"
  (Garcia-Castellanos 2007, p. 387).
- **Evaporation keeps a basin closed, and closure keeps its rock.** "the evaporation rate at the lake surface is revealed
  as a key factor triggering drainage closure (endorheism) and significant lake life extension by preventing outlet
  erosion" (Garcia-Castellanos 2006, p. 283); "Even before drainage closure is reached by tectonic uplift, evaporation
  reduces water discharge across the uplifting area, limiting the erosion rates" (p. 285). "Closed drainage (or
  endorheism) implies that, although some precipitation occurs in the highlands, its effect in terms of mass removal from
  the orogen is null"; in the dry model "most of the eroded rock is trapped within the orogen and at orogenic scale the
  effective mass removal is small", in the wet one "up to 16.5 km of basement rock is eroded" (Garcia-Castellanos 2007,
  p. 380). "a high plateau develops only for precipitation values lower than evaporation, i.e., P < 800 mm/yr" (Fig. 8
  caption, p. 383). "In contrast, where internally drained basins persist ... mass storage in the arid interior of the
  orogen prevents a steady state condition from developing" (Sobel et al. 2003, [62]).
- **What fills and opens a closed basin.** "Channels connected to closed basins must eventually aggrade as the basin
  fills and base level continually rises" (Sobel et al. 2003, [60]); "the sediment delivered to the lake displaces its
  shore, so the lake's altitude must increase to preserve a length sufficient to evaporate the collected water"
  (Garcia-Castellanos 2006, p. 288); "In closed drainage basins, either a climate change to wetter climatic conditions, or
  the sediment overfill of the lake (both inducing lake level rise) or a basin capture undertaken by an adjacent river can
  trigger drainage opening" (pp. 284-285); "arid areas in close proximity to large orographic gradients may represent
  transitional parts of the landscape in which efficient headward erosion allows external drainage to extend into the
  low-precipitation highlands" (Sobel et al. 2003, [58]). "Within the plateau, erosion products are stored locally within
  the basins, while within the broken foreland, transiently stored sediments are eventually exported from the orogen"
  (Strecker et al. 2009, p. 645).
- **Lake evaporation on the plateaus.** Lake Titicaca's "evaporation rate of 1100 mm/yr" (Garcia-Castellanos 2006, p.
  284); Ew = 1270 mm/yr from Blodgett et al. 1997's model; Tibet's 75 large lakes "Ea (ranging from 650 to 1300 mm) are
  approximately ... 1000 mm" a year (Wang et al. 2020).
- **Not this law.** Sobel et al. 2003 model the defeat of a river by an uplifting barrier (their eq. 13, with
  precipitation entering only through K and the transport coefficient), and contain no lake balance; "Whereas in Sobel et
  al. (2003) the initiation of internal drainage is considered coincident with the defeat of the river, in the present
  model river defeat implies only the formation of a lake, and internal drainage only develops if the lake surface becomes
  large enough to evaporate the water discharge collected by the lake" (Garcia-Castellanos 2006, p. 288). The model's
  defeat is the flood's depression; the closure is the balance. Hillslope transport rises with moisture in dry places
  (Richardson, Perron & Schurr 2019, abstract: "D increases strongly with increasing moisture among relatively dry
  sites"), but with no law read beyond the abstract nothing is built from it.

## The rule

**R1. A lake passes on what its water balance leaves** (ERO-218). The surface already takes each lake's balance for the
flexure's water (`lakes::water_levels`, ISO-126, on the bake's budget, HYD-050 and HYD-051): the runoff its catchment
sends it against what its surface loses, the open water's evaporation less the rain on it. With `surface.closed_basins`
the stream power's drainage area (the runoff-weighted area, ERO-010, ERO-082) leaves each lake's exit cells at the share
of the lake's inflow that flows out, (inflow - loss) / inflow at its spill (Garcia-Castellanos 2006, eq. 2), instead of
whole (`routing::accumulate_passing`, `closed::pass_shares`). An exit is a lake cell whose water leaves the lake for good:
a lake cell draining onto a shoal of its own lake (ground at the fill, so no lake cell) that drains back in is none.

**R2. A lake that cannot fill to its spill is a closed basin and passes nothing** (ERO-217). Where the inflow does not
cover the losses of the lake at its spill, the balance holds it below the rim (`lakes.rs`: its level is the shore that
balances) and the share is 0: below the spill the rivers carry only their own catchment, the spill is cut only from
outside, and the basin's water ends in it.

Nothing else changes, and what the sources describe follows from the model's own lakes:
- the depression stays a lake for the solve (`spl.rs`: no incision in it, the sediment kept up to its room, HYD-110), so
  what its ranges shed stays on the plateau's crust and fills it;
- it opens when a wetter climate's balance fills it to the spill, or when a river outside cuts its rim below its floor
  (capture), since every step's flood and balance are new, or when its sediment fills it (R3);
- a humid lake fills and spills less what it evaporates.

**R3. A basin its sediment fills opens** (ERO-217; the review of 2026-10-31). "In closed drainage basins, either a
climate change to wetter climatic conditions, or the sediment overfill of the lake (both inducing lake level rise) or a
basin capture undertaken by an adjacent river can trigger drainage opening" (Garcia-Castellanos 2006, pp. 284-285); "the
material shed from the range into the basin may eventually overtop the range and reintegrate the drainage system"
(Sobel et al. 2003, [26]). R2 alone did not do it: HYD-110 raises every lake cell by one share of its depth, so the lake
keeps its cells, and the balance its losses, until the room is nearly gone, and the sediment past the room left at the
spill with only the spill's own water (a closed lake passes none), settled there and dammed it; the next step's flood
found a new closed lake behind it. Now a lake whose exits the solve leaves at the flood's level (its room taken) has no
room for water either and passes all of it, and the step is solved again with its area passed on
(`closed::open_filled`): the spill is cut by the basin's water and the sediment goes on down the valley. A shallow pit
has little room, so a one-cell lake the sediment reaches fills and opens within its step; the surface's lakes are any
depression the flood fills by more than 1 cm, as the solve's are, where the bake draws only lakes of 1 km² and a block
deep (what the world shows, not the water).

Simplifications (the agent's): the depression is the solve's lake up to its spill whatever the level of the water in it
(in the sources the floor between the balance's shore and the spill is the aggrading basin, which at 4.8-km cells and
2-Myr steps the lake stands for); the share is the bake's budget applied to the surface's runoff-weighted area, two
runoffs that differ on frozen ground; a lake its sediment fills is open for the whole step (it filled within it), and the
flexure's water over it stays the balance's of the step's start; the lakes' evaporation is the bake's 200 + 60 T+ mm a year (HYD-040, HYD-041), about half Tibet's measured
1,000 and Titicaca's 1,100 to 1,270, so the model closes fewer high basins than their measured evaporation would (not
changed: HYD-040 is calibrated on Earth's lakes as a whole).

**The controlled plateau** (ERO-219, `plateau_check.rs`,
`an_arid_plateau_behind_its_rim_keeps_its_rock`; the rimmed plateau 4 km high, a rim 500 m above it, a wet outer flank,
the presets' law, 10 Myr; m/Myr, interior then edge):

| case | interior: stream power, cut, diffusion, net | edge: stream power, cut, diffusion, net |
|---|---|---|
| arid interior (runoff weight 0.05), spills (no balance) | 2.8, 0.9, 2.3, 3.8 | 115.3, 47.9, 16.7, 121.2 |
| arid interior, closed by its balance | 0.0, 0.0, 0.8, 0.5 | 104.8, 46.8, 14.2, 111.7 |
| humid interior (weight 1), spills (no balance) | 126.7, 0.5, 15.2, 129.9 | 264.2, 2.9, 17.2, 266.3 |
| humid interior, with its balance (a lake losing 20 mm a year) | 126.6, 0.5, 15.1, 129.8 | 264.3, 2.9, 17.2, 266.3 |

The arid interior already wears slowly on its own runoff (3.8 against 129.9 m/Myr), and closed it sends almost none of
its rock away (0.5), while the wet flank still wears (111.7 against 121.2); a humid interior spills about as before. On
the planets the flat high ground wears at 134 to 138 m/Myr, far above this arid interior's 3.8: its rivers bring water
from upstream, and the depressions that a third to a half of it drains into all spill. R2 takes that water away where the
balance closes them.

## What it does not do

- The plates' share (0.57 to 0.69 of a plateau's crust after its collision): the sourced collapse and flow, untouched.
- Edge peaks: in Vanderhaeghe et al. 2003 "a plateau with no significant surface slope develops between" a pro- and a
  retro-edge peak (collision-orogen.md); the model's plateau stands at the strength height with no range above it, so
  its depressions are the surface's and the plates' own relief, not ranges that dam it. The law closes what depressions
  there are; whether a plateau has the ranges that make an Altiplano is the orogen's.
- Frozen ground's runoff (ERO-010's PET 0 below 0 °C): left here, on a final-frame read (0.004 to 0.029 of the flat high
  ground) that the arm overturned: over the run 0.67 of the flat high ground is below 0 °C and it wears at 141 m/Myr. It
  is the next lead ("The arm").
- The bake's lakes keep HYD-050's balance for the world. The young landscape (`young.rs`, the L2 refinement of the last
  step for the game) routed as before until the review of 2026-10-31; it now takes the same law on L1's budget upsampled,
  so the last step no longer cuts the spill of a basin the history kept closed with the whole basin's area.
- The runoff weight's 5 % floor (ERO-011, uncited): kept, since K was calibrated with it (2026-10-18).
- No knob, cap or forcing term: the share is each lake's balance on the bake's existing budget.

## How it is judged

Unit tests: `surface::closed` (an arid basin is closed and passes no water; a humid one spills less its lake's
evaporation, the share against an independent count; passing everything is the plain accumulation to the bit; and,
since the review of 2026-10-31, the passing accumulation against its definition with random shares, a lake draining
over its own shoal with one way out, a chain of two lakes applying each share once, and a 0.5 m deep arid basin a range
overfills in one step, whose spill rises 660 m closed and is cut 186 m once it opens) and
`surface::plateau_check::an_arid_plateau_behind_its_rim_keeps_its_rock` (the table above; since the review it also
checks the first step's lakes: the arid interior's 871-cell lake passes nothing, the humid one 0.69). Gate on the branch (VM,
`tools/review/plateau/gate.sh full`, on c4fd3e38): fmt and clippy (workspace, all targets) clean, deeptime-core 1,150
passed and 12 ignored, its integration tests passed.

**The arm (one cumulative 8-run arm; to be queued by the coordinator).** On base35-s4 (the next 24-run base, after
`collision-orogen-on` lands, so with the two orogen switches on by default): merge `origin/master` into this branch,
then from it

    tools/review/pool/pool.sh run plat-cb --seeds 1,2,3,4 --set surface.closed_basins=true +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps
    tools/review/pool/pool.sh table --base base35-s4 --arm plat-cb

and `tools/review/plateau/wear.sh platcb=<its folder> base35=<base35's folder>` with `tools/review/orogen/hiseries.py`
over both. If it is wanted before base35: from this branch as it is (key e751b2344693 at d8cdbea2, the orogen switches
off by default), `pool.sh run plat-cb0 --seeds 1,2,3,4 --set tectonics.collision_two_wedges=true --set
tectonics.collision_retro_plateau=true --set surface.closed_basins=true` with the same two environment words, against
orog-cum (the switch off is orog-cum's physics: `pool.sh alias HEAD f2d6412d801f --check-seed 1` first, which runs one
seed). Read: the guard, `v1census`, the high ground through time (above 2 and 4 km, mean and largest), the summary's new
`flatc`, `flatd` and `flatf` classes of the erosion bands (the flat high ground draining to closed basins, dry, frozen)
and `closed_lake_steps`, `partial_lake_steps`, `closed_km2_myr`; the land's mean lowering and the sea's sediment
(`bands.py`); and, for K's calibration, the regions' erosion against Montgomery & Brandon (the bands' `wz` classes, as
`tools/review/erosion-recal.py` reads them). No V1 number is a gate (decisions log 2026-10-23).

## The arm: plat-cb0 against orog-cum (2026-10-31)

Queued by the coordinator before base35 existed. `pool.sh alias HEAD f2d6412d801f --check-seed 1` was identical (world
state 628e6d3abce00513 and `rec.json`), so the switch off is orog-cum's physics. Then `pool.sh run plat-cb0 --seeds 1,2,3,4
--set tectonics.collision_two_wedges=true --set tectonics.collision_retro_plateau=true --set surface.closed_basins=true
+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps` ran on key e751b2344693, argkey 5e8a748c, 8 runs.
Orog-cum is f2d6412d801f, d3261b0a, 8 runs.

The reads (VM): `tools/review/plateau/wear.sh platcb0=<folder> orogcum=<folder>`, which now also runs `series.py` (land
above 2, 3 and 4 km per 50-Myr map, and the run's extremes) and `tools/review/orogen/hiseries.py`. Also
`tools/review/plateau/recal.sh` (erosion-recal.py's relief and region table) and `closedshare.py`. Outputs are in
`/data/ban/deeptime-runs/plateau/` (`wear-platcb0.txt`, `wear-orogcum.txt`, `recal.txt`). Values are medians of the 8
runs, closed against open.

**The pool's tables (the coordinator's, 8 and 8).** Every row is within spread except three:
- **Inside spread:** land 0.233 against 0.205 (p 0.33), crust under the sea 0.538 against 0.605 (p 0.51), land above
  2 km 0.016 against 0.013 (p 0.65), coast with high ground 0.109 against 0.067 (p 0.38), land under ice 0.069 against
  0.037 (p 0.23). The V1 criteria met are a median 6 in both.
- **The guard's worst:** 10,482 against 9,945 m (p 0.005); 0 steps beyond one Earth relief in all 16 runs.
- **The plates' peak at the end:** 4,463 against 4,162 m.
- **collcoast's "belt not elongated"** (`geom_round`): 0.256 against 0.075 (p 0.005).

**Did closed basins keep the arid plateau interiors? No.** The flat high ground wears as before:

| class (part of the flat high ground) | arm | net m/Myr (per run) | stream power (the cut) | diffusion | ice | gross shares: stream power, cut, diffusion, ice |
|---|---|---|---|---|---|---|
| flat high ground | closed | 141.4 (114 to 196) | 103.8 (51.7) | 37.7 | 8.9 | 0.372, 0.316, 0.250, 0.054 |
| flat high ground | orog-cum | 134.3 (114 to 228) | 97.0 (50.8) | 37.1 | 9.8 | 0.366, 0.316, 0.241, 0.060 |
| draining to a closed basin, 0.30 | closed | 52.8 (47 to 84) | 24.6 (10.9) | 28.5 | 9.2 | 0.201, 0.181, 0.467, 0.121 |
| dry (runoff weight up to 0.1), 0.22 | closed | 37.6 (25 to 90) | 13.4 (6.8) | 24.2 | 0.5 | 0.157, 0.189, 0.621, 0.009 |
| frozen (mean below 0 °C), 0.67 | closed | 140.8 (100 to 211) | 99.4 (69.0) | 38.4 | 12.2 | 0.208, 0.443, 0.255, 0.073 |

The three parts overlap. The stream-power share (0.37, or 0.69 with the cut it drives) did not fall: 0.372 against 0.366.

The closed part's 53 m/Myr and the dry part's 38 compare with Earth's internally drained plateaus as follows:
- southern Tibet's peneplain: "6-11 m/m.y." locally and "11-16 m/m.y." catchment-wide (Hetzel et al. 2011, p. 983);
- Strobl et al. 2012: means 8.1 and 11.8 m/Myr;
- southern and central Tibet: "<30 mm/ka", that is under 30 m/Myr (Lal et al. 2003, abstract);
- the internally drained Altiplano-Puna: 0.02 mm/yr, 20 m/Myr (Bookhagen & Strecker 2012, p. 103).

So the closed part wears 3 to 5 times Tibet's catchment rates and twice Lal's bound. The dry part wears 2 to 3 times
Tibet's catchment rates, just above Lal's bound. Diffusion is half and more of what wears both, and the law does not
touch diffusion: κ 2 m²/yr on relief under 500 m at 4.8-km cells (ERO-177's κ alone was tried). Stream power is a third.
The flat high ground as a whole wears at its frozen part's rate.

Why the law moved so little:
- **R2 takes water only below a closed lake.** Above the lake, inside its catchment, the rivers keep their own area and
  their sediment stays in the lake (HYD-110), as it already did. So the law only changes ground that drains over the
  spill, and on the planets closed basins form where that water was already small.
- **The frozen flat high ground, 0.67 of it, is where the wear is.** It wears at 141 m/Myr, two thirds of it by stream
  power and the cut. The surface's runoff there is all of the precipitation (`surface/climate.rs`: PET 60 mm/°C x T, 0
  below 0 °C, ERO-010). The bake's budget dropped that rule on 2026-10-01 (`HydroParams::budget`: "in a cold one the land
  still evaporates in its summer (before 2026-10-01 a land with a mean below freezing evaporated nothing, and all its
  precipitation ran off)"). The bake's budget has three parts:
  - rain only in the thawed share of the year (HYD-036);
  - snow melted by degree-days (HYD-037), so what does not melt is the glaciers', not the rivers';
  - PET from Holdridge's biotemperature (HYD-029, HYD-030, HYD-035).

  That budget is the one the closure's balance already uses, so a frozen lake's balance and its rivers disagree. The
  summary does not split the frozen part by ice. Its 99 m/Myr of stream power says its rivers run, and under 100 m of
  ice they do not.

The plateaus' crust and high ground did not change beyond spread:
- **Cohorts above 3 km:** thinning 105.3 m/Myr, plates 47.6 and top 57.7, against 112.5, 58.9 and 53.6.
- **Cohorts above 4 km:** 152.5, 81.0 and 71.5 against 172.1, 119.5 and 52.6.
- **Land above 2, 3 and 4 km,** each run's mean over its maps after the first 100 Myr: 0.0232, 0.0086 and 0.0010
  against 0.0245, 0.0091 and 0.0012. The largest: 0.062, 0.030 and 0.0101 against 0.076, 0.040 and 0.0085. Earth's land
  above 4 km is 0.0195.

**What the law does do.**
- **Internally drained land, where there was none.** Over the run 0.127 of the land (0.093 to 0.148) drains to a basin
  that passes nothing (`closedshare.py`, against the dynamics record's land share; the surface's own land is smaller,
  so this is a lower bound). Earth: "Endorheic basins occupy 20% of the Earth's land surface" (Garcia-Castellanos et
  al. 2003, p. 8-1).
- **Lake counts.** Lake-substeps closed: 3.6e4 a run. Lakes passing part of their inflow: 6.3e4.
- **Sediment.** The lakes keep 0.285 km³ a year against 0.242. The sea gets 3.36 against 3.03, within spread run by
  run. The land's mean lowering is 36.8 against 37.1 m/Myr.
- **K's calibration is untouched.** By relief class and by region the land erodes as before:
  - regions of 500 m relief or more: 219.9 against 218.2 m/Myr;
  - regions of 1,000 m or more: 394.2 against 400.9 m/Myr;
  - all land: 37.5 against 38.1 m/Myr (`recal.txt`).
- **The cost** is 3.7 s of the surface's 445 s a run (the timing line's `closed`).

**What the guard's 500 m larger worst means.** The worst is the farthest any L1 cell stood from the sea after any step
(`surface/guard.rs`). The guard records the distance, not the side or the step. Paired by start and seed, the closed arm's
worst is larger in all 8 runs, by 40 to 1,362 m.

| run extreme, m (medians) | closed | orog-cum |
|---|---|---|
| guard worst (any step) | 10,482 | 9,945 |
| deepest, the maps' samples (every 50 Myr) | 9,465 | 9,454 |
| deepest, the final surface | 8,875 | 8,824 |
| highest, the maps' samples | 5,876 | 5,718 |
| highest, the final surface | 5,092 | 4,628 |
| the largest one-step rise of a cell | 7,562 | 7,106 |
| the largest one-step fall of a cell | 7,119 | 7,370 |

The worst stands past every snapshot on both sides, so it is a one-step transient. The standing deep side did not move.
The standing high side, and the largest one-step rise, rose by 0.2 to 0.5 km. The plates' peak rose 0.3 km (4,162 to
4,463). That fits rock kept on the orogens: high ground held a little higher, and larger transients on it when the
plates move it. Which side the worst is on is not recorded.

Either way it is not a divergence:
- 10.5 km is under the Challenger Deep's 10,925 m (ISO-071);
- it is 0.044 of the guard's 237-km bound;
- no step went beyond one Earth relief.

A signed worst and its step would settle the side. That is a measurement-only change to `guard.rs` and `run.rs`, and
the next arm's key changes anyway after the merge.

**The belt shape (collcoast's `geom_round`, 0.256 against 0.075).** This is a final-frame read: the share of the
collision high coasts whose collision cells within 400 km are round, not elongated. The law does not act on it.
- **The population it is read on changed.** High coasts rose from 0.092 to 0.107 (coastset). The collision belt's rate
  on its coast rose from 0.353 to 0.523, and the rifted margin's share fell from 0.125 to 0.058.
- **The end frame's collision land doubled:** 0.025 against 0.013 of the land, per run 0.006 to 0.039 against 0.002 to
  0.045.

So more of the final high coasts sit on broad collision regions. Not explained here.

**Verdict on default-on: yes (done 2026-10-31: `surface.closed_basins` is on by default, the coordinator's call, decisions log 2026-10-31).**
- **The project's rule is met** (CLAUDE.md, decisions log 2026-10-23: physics goes on once built and correct; the rule
  it replaces goes off with it):
  - unit tests against its sources;
  - no divergence or guard failure on the pool;
  - K's calibration intact;
  - under 1 % of the surface's time.
- **The rule it replaces is ERO-080's routing of every depression over its rim.**
- **It gives the planets internally drained land,** 0.13 of it against Earth's 0.20.

It is not the fix for O4. The plateaus wear on their frozen flat tops. The next sourced law: the surface's runoff from
the bake's own budget (`HydroParams::budget`, the closure's balance), in place of `surface/climate.rs`'s PET 0 below
0 °C. One budget for the rivers and the lakes, no knob. K was calibrated on the surface's runoff (2026-10-18), so that
arm reads the regions' table again rather than retuning K.
