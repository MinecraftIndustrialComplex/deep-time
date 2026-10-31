# Collision orogens as two wedges about the suture: design

Status: **built behind the switch** (2026-10-29, the coordinator; PLT-730, PLT-731, `fill_two_wedges` in
`plates/belts.rs`, plates.md "Step 20"). **On by default since 2026-10-31** (the coordinator's call, decisions log
2026-10-31). It was off from 2026-10-29: the arm tw-wedges raised the coast with high
ground (0.159 to 0.207) and put collision land nearer the sea, so the lead below is refuted (plates.md "Step 20").
Read again on the freeboard physics (2026-10-31, the orogen lane): docs/design/collision-orogen.md. For V1's one missed target, the coast
with high ground (0.159 on base29 against at most 0.11; the coast in a collision belt 0.257 of the
coast). Undoable; built behind a switch, measured on the pool against base29, on by default only if it
is correct and the numbers do not get worse elsewhere.

## What the model does

`plates/belts.rs`, `stack_front`, with `collision_belt_taper` on (PLT-600, PLT-601, since 2026-10-25):

- **The belt** on the overriding plate is the critical wedge, its toe at the front: its surface rises
  from the overriding edge's height at the front, `z_front`, at `wedge_slope_deg` (1.5 degrees,
  PLT-192) up to the strength height, where it widens as a plateau. The step's crust raises the ground
  nearest the front first.
- **The wedge** on the incoming plate takes its share first, but only up to a profile falling away
  from `z_front`: `wedge_reach_km = (z_front - (sea - 500 m)) / tan(wedge_slope_deg)` (PLT-244). Against
  an overriding edge 500 m or more under the sea there is no wedge, and the belt takes everything.

On base29 (24 runs, the dock tally after the first 100 Myr, medians) the overriding edge is dry for only
0.334 of the front-time of collisions under 10 Myr old (0.551 at 10 to 30 Myr), and those two classes
hold 0.58 of the front-time. So most collision crust starts its orogen on the overriding plate's edge,
which at an active margin is the coast, and land reaches 2 km about 76 km inland of the coast
(2,000 m / tan 1.5 degrees from where the surface crosses the sea; from a toe 200 m under the sea that is
84 km from the toe): inside the measure's 200 km.

## What Earth's orogens do

- A doubly vergent orogen grows about the point S where the lower plate's mantle detaches: a pro-wedge
  over the lower plate at the minimum critical taper, a retro-wedge over the upper plate, steeper, at
  the maximum (Willett, Beaumont & Fullsack 1993, p. 372). "More mass is stored in the pro-wedge than
  the retro-wedge" (Naylor & Sinclair 2008, p. 21); with their assumed surface slopes, 1.5 degrees pro
  and 2.5 degrees retro, a 3 km orogen has a 115 km pro-wedge and a 69 km retro-wedge (pp. 8-9), so the
  pro-wedge holds about tan 2.5 / (tan 1.5 + tan 2.5), 0.62, of it.
- At an active margin: "a large accretionary wedge obeying pro-wedge mechanics and a smaller
  landward-verging retro-wedge ... overlying the arc basement" (Willett et al. 1993, p. 373).
- Taiwan: the 90 km belt is the Chinese margin's crust, the lower plate, and the Luzon arc with the
  Longitudinal Valley suture is its backstop (Dahlen 1990, pp. 83-84; Simoes et al. 2007, p. 1).

So the model has the geometry mirrored: its toe sits at the suture and its wedge grows into the upper
plate, where Earth's toe sits out on the lower plate and its larger wedge rises toward the suture.

## The change

`tectonics.collision_two_wedges` (a switch; ledger rows to be opened):

1. The orogen's crest stands at the front (the suture, S). The step's crust is split between the two
   sides by the wedges' own cross-sections at their tapers: the incoming plate's pro-wedge at
   `wedge_slope_deg` (PLT-192, 1.5 degrees) and the overriding plate's retro-wedge at a retro slope
   (Naylor & Sinclair's 2.5 degrees is their assumption). Built: the retro slope as Taiwan's ratio to the
   pro slope, `retro_slope_ratio` 1.55 (2.9 and 4.5 degrees, Davis, Suppe & Dahlen 1983; PLT-731), so
   2.33 degrees. Pro share = tan(retro) / (tan(pro) + tan(retro)), 0.61.
2. Each side rises from its toe toward the crest at its slope, the crest height following the
   accumulated cross-section, `A = H^2 / 2 * (1/tan(pro) + 1/tan(retro))`, so neither side is capped
   by the other's edge height (the end of PLT-244's cap).
3. Past the strength height the crest widens as a plateau on the overriding side, as the taper belt
   does now (Tibet behind the Himalaya).
4. Everything else stays: the stacking share (PLT-242), the crust that crosses whole (PLT-400), the
   no-room rule (PLT-700), the overriding plate's interior share (PLT-652), the foredeep beyond the
   pro-wedge's toe.

Where the incoming plate's pro-wedge crust goes when its samples cross the front is the existing
restack path (PLT-400): it crosses whole and is stacked again on the far side. That keeps the volume;
the arm reads whether the pro-wedge then holds its share or drains into the belt.

## How it is judged

One 8-run arm against base29 (`pool.sh run tw-wedges --set tectonics.collision_two_wedges=true`, seeds
1 to 4 both starts) with the `v1census`, `boundary`, `collcoast` and `coastset` registries: the coast in
a collision belt and the coast with high ground should fall; land above 2 km, the highest plate
elevation and the plate check should not get worse. Unit tests: the split against the tapers' formula,
volume kept, a crest that rises at the front with a submerged overriding edge.
