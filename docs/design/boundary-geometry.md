# Plate boundaries that organize to the motion: design notes (the boundary-geometry lane)

Ben, 2026-11-01: "Boundary geometry (Recommended)": read Earth's sources on how boundaries organize to the motion
(ridge and transform segments, ridge jumps, microplates where a boundary changes class), then build a sourced rule
behind a switch and run one 8-run cumulative arm (decisions log 2026-11-01). No knob, cap or forcing term moves a V1
number; the rule goes on by default once it is correct and the pool shows no divergence or guard failure.

## The question

The model's boundaries keep the outline they were made with while the plates' motion changes under them: 0.63 to 0.65
of its boundary lies on pairs of plates that both open and close against each other, Earth's 0.09 to 0.135 (plates.md
"Where the pairs' relative poles lie"). Seen from the arcs (plates.md "Why the island arcs do not grow: a front's life
on its rock"), a subduction front lasts about 20 Myr on its rock and most fronts end where the subducting plate's own
ridge, 200 to 300 km away, reaches the trench: young floor (18 to 23 Myr at the median) between, an open gap behind it
until the next resampling, which fills it with young floor of the same pair, and a contact often starts on it again (the
step-after onsets on gap fill, about a third of the intra-oceanic front). That one geometry gives:

- the thin forearcs: no crust of 15 km within 200 km behind 0.399 of the front against Earth's 0.165 (forecrust.py);
- the young-floor excess at the intra-oceanic fronts (0.46 of the young floor going down there against Earth's 0.09 to
  0.15; plates.md "The intra-oceanic fronts' length");
- the intra-oceanic front's length (26,440 km against Stern & Scholl's 14,500), since an arc that never grows never
  turns its front continental (PLT-047, PLT-370);
- subduction erosion short of forearc crust (arc-stock.md, open question 4).

## What the model has today

- Ridges in gaps are drawn normal to the pair's present relative motion at each resampling, with transforms between
  segments (PLT-502); the old segments stay in the plates as fracture zones (PLT-507). PLT-507 says the reorganization
  itself has no law: the motion relaxes to the force balance and the next rifts and initiations move with it.
- A gap point goes to its two nearest plates by path through the gap (`ridges::dijkstra_two`), then to one side of the
  segment or of the midline.
- Any overlap of converging plates is a contact (`subduction::resolve`), incipient until 100 km of slab (PLT-610,
  PLT-611); a front stands only while the subducting plate's visible crust is within two spacings (PLT-582).
- Fronts are fixed in the plate that holds them (PLT-710: Heuret & Lallemand's Vd = 0).

## Sources

(To be filled from what was read: ridge reorganization after a change of spreading direction, the ridge-trench
encounter and microplate capture, ridge jumps, boundary classes per pair.)

## Candidate rules

(After the sources.)

## How the arm will be read

One cumulative arm of 8 runs against base38-s4 (`pool.sh run bg-cum --seeds 1,2,3,4 +DEEPTIME_CRUST_BUDGET=1
+DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps` with the switch), and one 2-Myr look run for `frontstay.py`
(`+DEEPTIME_SNAPSHOT_MYR=2:-560:-500`). Read, against base38-s4 and against Earth where Earth has a number:

- the boundary on pairs that both open and close, and the convergent boundary within 1,000 km of the same pair's
  divergent one (`tools/review/crustcycle/polenear.py`; Earth 0.09 to 0.135 and 0.125 to 0.272);
- the front's life on its rock (`frontstay.py`: 0.58 to 0.66 after 10 Myr today) and why it ends;
- the crust behind the fronts (`forecrust.py`: Earth 0.165 with no 15 km crust within 200 km);
- the young floor at the intra-oceanic fronts and their length (`islrec.py`, `oofronts.py`, `cbread.py`);
- the census (`--measure v1census`), the guard (`--measure guard`), the shape watch (`shape_jumps.py`).
