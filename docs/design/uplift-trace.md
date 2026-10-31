# The rock the plates lift above the sea, by process: a trace

Status: **measured, nothing built** (2026-10-31, the runoff lane's lead, branch `uplift-trace` from
`surface-runoff-budget` 1803fb56). A measurement-only ledger in the gap budget (`plates/gap_budget.rs`, the land
ledger) and one trace of base34's seed 1 on both starts. The verdict is at the end.

## Why

The continents export 3.82 km³ of rock a year on base34-s4 (the crust-budget lane, docs/design/crust-budget.md), 2.16
times Clift et al. 2009's long-run 1.77, and the export is the plates' supply, not the erosion law (two erosion-law arms
left it, one plate law moved it; docs/design/surface-runoff-budget.md, "The continents' export"). The coordinator's
questions: split the rock the plates lift above the sea by process, against Earth's long-run numbers where sources give
them. First, accretion at resamplings (1.15 km³ a year) and the transition (0.59) send 0.45 of the export back to the
continents: is that rock lifted and eroded again, and is any of it counted twice in the books? Then the collisions'
thickening, the flow and collapse that spread it, the arcs, the isostatic response to erosion, and the continents' net
loss. Is the excess a wrong law, a missing law or a double count?

## What was measured

**The land ledger** (`GapBudget::set_sea`, `LandSums`, the summary's `land_ledger` lines;
`tools/review/uplift/ledger.py`). It uses the gap budget's brackets: every process of a plate step, the surface's
feedback split into its parts, and a resampling's blend and accretion. Per part it records:
- the change of the land's height above the sea, H = max(z − sea, 0) over the visible samples (`lift`; its rises alone,
  `up`; on continental crust, `lift_c`);
- the change of the columns' thickness by where each column stood before it: above the sea (`t_land`), continental
  crust under it (`t_shelf`), ocean floor (`t_ocean`), hidden (`t_hidden`).

The sea is the climate's for the plate step, and its own move between steps is booked apart. A booking split over
several parts walks them in order, so a sample crossing the shore still adds up. The parts and the sea's term add up to
the change of the land's volume above the sea: the closure is 0.0000 km³ a year on both runs. Tests:
`the_land_ledger_adds_up_to_the_land_above_the_sea`, and the plates' `the_gap_budget_changes_nothing_and_adds_up` now
checks the closure on a run. `tools/review/uplift/gapsplit.py` reads the gap budget's own rates by part and thickness
band.

**The trace.** `tools/review/pool/pool.sh run upl-trace --seeds 1 +DEEPTIME_GAP_BUDGET=1 +DEEPTIME_CRUST_BUDGET=1
+DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`: key 93891af8f54e at 760aa5f0, argkey ea7131f2, 2 runs, 17 minutes each. Both
worlds are base34-s4's seed 1 bit for bit (`tools/review/uplift/samestate.py`): all 108 and 109 dumped fields, the
globals and `rec.json` are identical, and only the gap budget's eight part fields are added. So the crust-budget lane's
ti-base books, on the same worlds, read alongside.

## The split

Km³ a year, Earth-normalized; the mean of the two runs over 1,000 Myr. The late 500 Myr give the same within 0.03 on
every row. Columns:
- H: the land's height above the sea, the volume raised (+) or lowered (−);
- the crust's thickness change under land columns, under the shelf (continental crust under the sea) and on the ocean
  floor.

| part | H | crust under land | under the shelf | on the ocean floor |
|---|---|---|---|---|
| erosion and deposition | −4.67 | −4.84 | +1.03 | +4.64 |
| its local isostatic rebound | +3.71 | | | |
| the settling to the Airy height (`settle.rs`) | +0.78 | | | |
| the flexure: regional answer −0.24, roots +0.13, released moment −0.07, rest −0.14 | −0.31 | | | |
| sea water and ice, local | +0.10 | | | |
| volcanoes and impacts | +0.15 | | | |
| the carry (L1 advection, sampling, clamps) | −0.24 | | | |
| collision belts (`belts::stack`) | +0.79 | +3.90 | +5.30 | +0.19 |
| the rate cap (`rate::limit`) | −0.24 | −2.10 | +0.76 | +1.33 |
| collapse | −0.09 | −1.17 | +0.94 | +0.23 |
| lower-crustal flow | +0.02 | −0.31 | +0.31 | 0 |
| founder | 0.00 | −0.19 | +0.08 | +0.11 |
| arcs | +0.07 | +0.30 | +0.23 | +0.95 |
| accretion at resamplings | +0.09 | +0.69 | +0.30 | +0.35 |
| a resampling's blend | −0.10 | +0.11 | +0.94 | −1.04 |
| subduction (crust hidden) | −0.07 | | | |
| the transition | −0.02 | −0.08 | 0 | +0.08 |
| hotspots | +0.01 | +0.02 | +0.01 | +0.02 |
| change of the land above the sea; the sea's own move | −0.032; −0.012 | | | |

Rifting, shortening, the plate surface step, the finish and the burial each move under 0.005 on every column.

The land above the sea fell from 8.2 × 10^7 to 5.0 × 10^7 km³ over the run, nearly all in the first half. In the late
half it is steady: +0.002 a year, with the sea's fall from 250 to 97 m adding +0.046. So the land's export is resupplied
there, and the split above is a steady state's.

## The double-count check: no double count

- **The books agree.** The erosion takes 4.84 km³ a year off land columns and lays 1.03 on the shelf: 3.81 leaves the
  continental crust. The crust budget's export is 3.82.
- **No rock reaches hidden crust.** `t_hidden` is 0 for every part, so the feedback lays nothing on crust that is
  already under another plate, which a resampling would then accrete a second time.
- **The flows back are a real cycle, and only the accretion's part is lifted.**
  - Accretion at resamplings puts 0.69 km³ a year of crust under land columns, 0.30 under the shelf and 0.35 under
    ocean floor. It lifts the land by 0.09 km³ a year of H, a tenth of what the parts other than erosion and its
    rebound add (0.94). This is the sediment that erosion laid on the floor and the plates carried under the margin: the
    accretionary margins' underplating, which Earth has too.
  - The transition lifts nothing. It takes 0.08 km³ a year off land columns towards the ocean side (H −0.02). Its +0.59
    to +0.62 for the continents (ti-base) is crust labelled continental under the sea.
  - So of the 0.45 of the export that flows back, about 0.65 km³ a year (the accretion under land; 7 × 0.09) is eroded
    again. The rest stays under the sea.

## The parts, against Earth

**Collisions.** The belts are the one large plate lift: +0.79 km³ a year of H. They stack 9.2 km³ a year of crust (3.90
under land columns, 5.30 under the shelf), and their height rule is Airy's: the gap budget books no gap to them. On
ti-base, 9.90 km³ a year crosses the collision fronts and 0.948 of it is kept. That is 0.408 km² a year at 24.3 km of
crust, 22.1 mm/yr across 18,589 km of front.
- Against Bird 2003's continental convergent boundaries: 0.43 km² a year, 23,003 km at 18.7 mm/yr (Table 3, as
  loss-budget.py quotes it).
- Against Stern & Scholl 2010's losses at the sutures: 0.7 km³ a year, 115 km³/km/Myr over 6,000 km (the model loses
  0.51).
- Against the rifted margins lost in Cenozoic collisions: 0.43 km³ a year at an average of 18 km of crust (Clift et al.
  2009, section 5.1).

So the model's collisions take in Earth's area a year and keep Earth's share of it. The crust that crosses is 1.35
times Clift's margins' thickness.

**The flow and collapse that spread it.** They move crust off land columns and lay it under the sea:
- the rate cap: 2.10 km³ a year off land columns, 0.76 of it to the shelf and 1.33 onto ocean floor;
- collapse: 1.17 off land, 0.94 of it to the shelf;
- flow: 0.31 off land, all of it to the shelf;
- founder: 0.19 off land.

Their lift is −0.24, −0.09, +0.02 and 0.00 of H. The rate cap is a step's thickening ceiling (`rate.rs`): a numerical
rule that here moves more crust off the land than the sourced collapse does.

**Arcs.** +0.07 of H; 0.30 km³ a year under land columns, 0.95 on ocean floor. Stern & Scholl's arcs add 2.5 km³ a year
(1.5 intra-oceanic, 1.0 continental); the model's add 1.48 (ti-base). Like Earth's intra-oceanic arcs, they build mostly
under the sea, and they are not the export's source.

**The isostatic response to erosion.** In H the surface's rebound gives back 0.795 of erosion's lowering. On the
thickness bands that stand above the sea (30 to 70 km), it gives back 0.80 to 0.81: the surface's `isostasy` 0.8, for
rock at about 2,640 kg/m³. The settling (ISO-078) then moves every continental column to the plate model's Airy height
(`collapse::iso_height`), within the step. That height takes any change of thickness at the column's mean density,
2,830 kg/m³ (PLT-281), 7.0 m of crust per m of height. So after erosion the settling lifts the eroded land by another
0.045 to 0.11 of the lowering on those bands (`gapsplit.py`, `bandsettle` read), and the response comes to 6/7, about
0.86.
- Airy for the eroded rock's own density gives 0.80 to 0.82: 2,640 to 2,700 kg/m³ over the mantle's 3,300. The
  plate model's own `ISOSTATIC_FACTOR` (5.5, upper crust 2,700 kg/m³) is documented "for rock eroded from the top of a
  column". The coordinator's 5/6 is 0.83.
- The settling's other lift repairs what the carry and the blend take off the land's height. They take −0.24 and −0.10,
  and the settling gives most of it back.

The column-mean density for light top rock is a wrong detail of the settling's target: eroded land stands 0.057 m higher
per m eroded than its rock's density holds it. It does not set the long-run export, by mass balance (the agent's
derivation). Every continental column is held at its Airy height, so the land's volume above the sea is the land's
crust above the sea-level thickness over 7.0. In a steady state the rock eroded then equals the rock the plates add
above that thickness, whatever the factor. The factor sets how high the land stands for that supply, not the supply.

**The continents' net loss.** The continental volume falls 1.08 km³ a year on ti-base (Stern & Scholl: additions 3.2,
losses "> 3.2"; Clift et al. 2009: arcs would need about 3.8 km³ a year to keep the volume constant). The land above
the sea is steady in the late half, though, so the loss is crust under the sea. It is not the land's export running
down its stock.

## Verdict: a missing (or too weak) sink for collisional crust, not a double count

- **Not a double count.** The books close, nothing is laid on hidden crust, and the flows back are the accretionary
  cycle: 0.65 km³ a year of the export is accreted rock eroded again.
- **Not the erosion law**, as the runoff lane found. **Not the isostatic response:** 6/7 against the eroded rock's 0.80
  to 0.82 sets the land's heights, not its long-run export.
- **The supply is the collisions'.** They take in Earth's area a year (Bird's 0.43 km² a year) and keep Earth's share
  (Stern & Scholl's and Clift's losses). Their stacked crust is the land's only large plate lift.
- **The model erodes too large a share of that crust.** Its export is 0.41 of what its collisions stack (3.81 of 9.2).
  Earth's long-run 1.77 is 0.12 to 0.23 of Bird's flux at 18 to 35 km of crust (7.7 to 15 km³ a year; the agent's
  arithmetic, not a source's number). On Earth, then, most of the thickened crust is returned by something other than
  erosion. In the model the only other sinks are:
  - the collapse and flow (1.5 km³ a year off land columns);
  - founder (0.19);
  - the numerical rate cap (2.1).

  Rifting thins nothing: under 0.005 on every column. Earth's continental extension, which stretches thickened crust
  back over rifted margins and extended orogens and lowers it below the sea, has no law here.

The lead is a sourced extension law: the stretching of continental crust where plates diverge within or at a continent
(β of rifted margins, orogenic extension). It should be read against the collapse and the 2-D thin sheet (PLT-733),
which already spread part of it, and against the rate cap's 2.1 km³ a year. That this is the missing sink is the
agent's inference from Bird's flux and Clift's export. The trace shows only that nothing else the model does returns
collisional crust without erosion.

Separately, a wrong detail worth its own look: the settling's Airy height for eroded top rock (column mean, PLT-281)
against the plate model's own upper-crust factor (5.5). It moves the land's heights and hypsometry, not the export.
