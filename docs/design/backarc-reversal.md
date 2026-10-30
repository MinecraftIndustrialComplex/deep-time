# A new subduction zone behind a narrow overriding block: design

Status: **design** (2026-10-30, the coordinator; Ben: "Design, then build (Recommended)", decisions
log 2026-10-30), built on the branch `backarc-reversal` behind `tectonics.collision_backarc_reversal`.
For the collision-front lane (STATUS Follow-ups, "Collision belts at the coast").

## Why

On the window split's arm (win-split2, 8 runs against base31's same seeds) the collision high
coasts by a belt along the coast are the far coasts of overriding blocks: the nearest collision
front is on the coast's own plate (0.764) a median 510 km away, and the sea in front is that plate's
own, mostly ocean floor (plates.md "The collision window split"). The belt (PLT-241, 400 km) and the
interior's share (PLT-652, decaying over L = 2W/π, PLT-733) stack crust right up to the block's far
coast, because a collision against a continent goes on until it stalls and welds (PLT-246) or its
incoming continent transfers (PLT-516).

Earth's collisions against narrow upper plates also start subduction behind the block. Stern 2004
(full text, PLT-371's row) names two induced nucleations: transference, and polarity reversal, which
"also follows collision, but continued convergence in this case results in a new subduction zone
forming behind the magmatic arc"; "the new subduction zone nucleates in the originally overriding
plate", whose back arc is "relatively thin and weak". In the Banda arc the Wetar Ridge overthrusts
the South Banda Basin along the Wetar Thrust, "which takes up some of the convergence" (Harris 2011).

The model has transference (the ocean floor behind the incoming continent ruptures) and a reversal at
the contact itself for arcs with no continent behind them (PLT-056, PLT-371). It has no rupture
behind a continental overriding block.

## The rule

A mirror of transference (PLT-516, `collision::collide`), on the overriding side:

1. **When.** As for transference: an unstalled collision of a continent (a terrane of
   `collision_suture_frac` or more) whose subduction has failed (the jam, `Collision::jam_km`) and
   whose plates have converged by PLT-518's critical convergence (100 km) since.
2. **The block.** The overriding plate's visible continental crust connected, through its own
   continental samples, to the overriding samples next to the collision's seeds.
3. **Its back margin.** The block's samples with an edge to the overriding plate's own visible ocean
   floor that lies farther from the front than they do (x, the distance to the nearest seed): the
   coast behind the block, not a forearc in front of it.
4. **The force there.** The collision's push (`forces::Answer::collision_n`, as transference reads
   it), carried to the back margin by the thin sheet's decay (PLT-733): push × mean(e^(−x/L)) over
   the back margin, L = 2W/π with W the front's length (`Collision::front_km`). A wide continent's
   far coasts lie many L behind the front and see almost none of it; a narrow block's see most.
5. **The test.** That force over the back margin's length reaches PLT-517's 3 × 10^12 N/m × g/g⊕
   (Gurnis et al. 2004, the force that starts a self-sustaining subduction zone). Where transference
   is due too, the margin with the larger force per metre over the threshold ruptures.
6. **What happens.** The block moves to the incoming plate (each of its groups gets a twin there, as
   a transfer does the other way, PLT-068), the incoming plate carries its momentum (PLT-080), and
   the collision ends. The overriding plate keeps its ocean floor behind the block, and the force
   balance decides what the new boundary between it and the block does: the floor, oceanic, goes
   under the block's continental crust (the subduction zone behind the arc). The incoming
   continent's crust that crossed the front stays under (PLT-276).
7. **Recorded.** `backarc_reversals` in the run summary; the collision event carries
   `backarc_reversal` 1.

No new constant: the threshold is PLT-517's, the convergence PLT-518's, the decay PLT-733's.

## What is the project's reading

Carrying the collision's push to the back margin by the thin sheet's decay (the sheet's solutions
are for a semi-infinite sheet, and none treats a finite one: PLT-241's note); taking x as the
distance to the nearest seed; and the tie-break between the two margins.

## Measuring it

An 8-run arm with the switch on against base32's seeds 1 to 4 (cached), with the default, `guard`,
`collcoast` and `coastset` measures: how often it fires per 100 Myr, the collision high coasts by a
belt along the coast, coast with high ground, and the V1 checks. On by default if it is built and
correct and the guard stays clean (Ben, 2026-10-23), whatever V1 does.

Unit tests: a narrow block reverses where the full half-continent of the same fixture does not; it
waits short of the convergence, without the jam, with the switch off and on a stall.
