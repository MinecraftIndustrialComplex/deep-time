# Plate boundaries that organize to the motion: design notes (the boundary-geometry lane)

Ben, 2026-11-01: "Boundary geometry (Recommended)": read Earth's sources on how boundaries organize to the motion
(ridge and transform segments, ridge jumps, microplates where a boundary changes class), then build a sourced rule
behind a switch and run one 8-run cumulative arm (decisions log 2026-11-01). No knob, cap or forcing term moves a V1
number; the rule goes on by default once it is correct and the pool shows no divergence or guard failure.

## The question

The model's boundaries keep the outline they were made with while the plates' motion changes under them: 0.63 to 0.65
of its boundary lies on pairs of plates that both open and close against each other, Earth's 0.17 to 0.22 at its 52
plates and 0.33 at the model's 16 (corrected 2026-11-02 from 0.09 to 0.135: plates.md "Earth's mixed pairs
corrected"; plates.md
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

Read by a Sonnet research sub-agent, 2026-11-01 (14 sources, 12 in full; bibliography.md "The sources of the
boundary-geometry lane").

- **The ridge-trench encounter.** Stock & Lee 1994 (full text): "As a spreading center approaches a trench in a
  subduction setting, the intervening subducted plate may fragment into smaller pieces down to linear dimensions of
  ~100 km (microplates)" (p. 1472); "This ridge death caused a 220- x 220-km fragment to become sutured to the Pacific
  plate" (p. 1482); the microplates "became attached to the Pacific plate when their ridge boundaries with the Pacific
  plate stopped spreading" (p. 1484); "None of these microplates became attached to the North America plate" (p. 1485);
  subduction of two fragments "might have effectively stopped when these microplates were born ..., several millions of
  years before the microplates actually ceased to exist" (p. 1484); lifetimes of the Californian microplates 2 to 11
  Myr (p. 1480). Thorkelson 1996: spreading may cease before the ridge meets the trench, the buoyant young plate breaking
  "into nonsubducting microplates"; in other cases "even the youngest parts of the oceanic plates are drawn into the
  trench"; floor younger than about 10 Ma is positively buoyant (Cloos 1993). Shuck et al. 2025: subduction "may
  terminate when a mid-ocean ridge approaches a trench", the microplates "captured by the PAC plate as subduction
  terminated", gradually along strike. Atwater 1970: "when the Farallon plate ceases to exist, the ridge also ceases".
- **Ridges and the motion.** Menard & Atwater 1968 (abstract): spreading centres "become uniformly reoriented
  essentially perpendicular to the faults"; Hey et al. 2010: reorientation by rift propagation at about 110 km/Myr;
  Harper et al. 2021: seesaw propagation not clustered at reorganizations; van der Linden & van Hinsbergen 2023: pole
  shifts are taken up by gradual changes of strike, ridge jumps only occasionally. The model already draws its ridges
  normal to the present motion at each resampling (PLT-502, PLT-507), so nothing is built here.
- **One class per pair.** Bird 2003 [125]: "a single class cannot always be specified for an entire plate boundary
  curve, or even for an entire plate boundary segment, because the class may change repeatedly with local changes in
  the azimuth of the boundary". No source read says why a pair would keep one class; the mixed share is a measurement to
  move by a law, not a law of its own.

## The rule built: a remnant joins the plate across its ridge (PLT-860)

`tectonics.remnant_capture` (`ridges::capture_remnants`, called in `ridges::bury` right after each resampling, before
the fission): a subducting plate's ocean floor between a front where it goes down (an active contact, the plates
closing across the edge) and a ridge of its own (another plate's ocean floor moving away, neither within 20 degrees of
the boundary), front and ridge at most 220 km apart through it, goes to the plate across the ridge. It answers the
mechanism frontstay.py found: the front ends where the strip runs out, and the next resampling's gap fill makes new
floor of the same pair at the trench, where a contact starts again on young floor. With the rule the ridge dies with
its remnant, as Stock & Lee's did, so the boundary there becomes the overriding plate's against the plate across the
ridge (a transform or whatever their motion makes), or closes where that plate is the overriding plate itself.

The width is the one measured case the source gives (the 220 km fragment, p. 1482): the fragments go down to ~100 km,
under two of the 16k lattice's 71 km spacings, where the strip has already gone between resamplings. Not built:
subduction stopping on buoyant young floor before the ridge arrives (Thorkelson's first case; the force balance's
buoyancy line, FRC-005, already reads it), and the microplates' own independent motion (the fission, PLT-532, gives a
cut piece its parent's rotation and the force balance moves it from the next step).

### The first arm moved plate corners (bg-cum, 2026-11-01)

The first form read each lattice edge's class on that edge's own way and had no test of the strip's shape. bg-cum
(8 runs) moved 68,000 to 80,000 samples a run, about 1,500 at every resampling (`plates/remnant_captured_samples` in
each run's `fields/globals.json`), where a 220 km remnant between two 71 km rows is a few dozen samples. Most were
plate corners: where a subducting plate's boundary turns from a trench to a ridge, every sample within 220 km of the
corner is within reach of both rows. An edge's own sign also picks the edges that lean with the motion: under a
motion 45 degrees oblique to a right-angled corner, the front's edges and the ridge's face 132 degrees apart (cosine
-0.67) where the boundaries do 90 (unit test `remnant_capture_leaves_a_plates_corner`). Its numbers against
base38-s4 (no census change significant):

| Measure | bg-cum | base38-s4 |
|---|---|---|
| Under the sea | 0.593 | 0.630 |
| Land | 0.193 | 0.178 |
| Largest landmass | 0.072 | 0.061 |
| Criteria met (mean of runs) | 5.38 | 4.88 |
| Guard, m | 10,374 | 10,438 |
| Boundary on pairs that both open and close | 0.615 | 0.630 |
| Convergent within 1,000 km of the same pair's divergent | 0.389 | 0.456 |
| Fronts with no 15 km crust within 200 km | 0.430 | 0.399 |
| Intra-oceanic front, km | 27,159 | 26,240 |
| Continental front, km | 23,952 | 26,864 |

The rule now reads each crossing edge's class on the boundary's normal on its side (the chord of the crossings to the
same plate within 90 degrees of the edge, from the plate's own samples within two lattice steps: PLT-310's normal with
the two sides of a thin strip kept apart) and captures only where the front and the ridge face more than 120 degrees
apart (`STRIP_COS`, a game-choice with no physical constant). bg-cum2 reads it.

### The second arm: on by default (bg-cum2, 2026-11-02)

bg-cum2 (key 2bf028b87fb3, 8 runs, both starts x seeds 1 to 4, crust-budget maps) and its look run (argkey a81b6c6b,
scattered_continents seed 1, 2-Myr maps in -1000 to -890 and -560 to -500 Myr) against base38-s4 and look2 (the same
seed with the rule off):

| Measure | bg-cum2 | base38-s4 | Earth |
|---|---|---|---|
| Samples captured a run | 8,171 to 12,161 | | |
| Boundary on pairs that both open and close (`polenear.py`) | 0.594 | 0.630 | 0.17 to 0.22; 0.33 at 16 plates |
| Convergent within 1,000 km of the same pair's divergent | 0.446 | 0.456 | 0.14 to 0.29; 0.25 at 16 plates |
| Fronts with no 15 km crust within 200 km (`forecrust.py`) | 0.395 | 0.401 | 0.165 |
| Intra-oceanic front, km (`islrec.py`) | 26,244 | 26,240 | 14,500 |
| Continental front, km | 27,874 | 26,864 | |
| Island arcs' rate per km of front, km³/km/Myr (`cbread.py`) | 71.5 | 70.7 | |
| Sediment subducted, km³/yr of rock | 3.14 | 3.09 | about 1.0 |
| Continental crust under the sea (census) | 0.585 (p 0.23) | 0.630 | at most 0.56 |
| Land, largest landmass | 0.207, 0.080 | 0.178, 0.061 | |
| Criteria met, mean of runs | 5.25 | 4.88 | of 7 |
| Guard, m | 10,650 (p 0.80) | 10,438 | |
| Continental samples added per resampling (`shape_jumps.py`) | +25.5 | +25.6 | |

The front's life on its rock (`frontstay.py`, the look runs), the share of the front still a front on the same rock:

| | look2, off | bg-look2, on |
|---|---|---|
| intra-oceanic after 10 Myr, early / mid window | 0.579 / 0.661 | 0.625 / 0.654 |
| intra-oceanic after 16 Myr | 0.384 / 0.458 | 0.480 / 0.585 |
| continental after 16 Myr | 0.361 / 0.516 | 0.469 / 0.497 |

About 200 samples a resampling, about 1.0 million km² (this page's arithmetic at 5,100 km² a sample): a 220 km strip
along about 4,600 km of the 54,000 km of front every 20 Myr. Fronts stay longer on their rock and the mixed pairs' share
falls a little; the intra-oceanic front's length and the crust behind the fronts do not move, so they are not this
mechanism's: the strips the rule hands across are a small part of where fronts end. The rule went on by default (the
coordinator's call, decisions log 2026-11-02): sourced, tested, no divergence or guard failure.

## How the arm will be read

One cumulative arm of 8 runs against base38-s4 (`pool.sh run bg-cum2 --seeds 1,2,3,4 +DEEPTIME_CRUST_BUDGET=1
+DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps` with the switch), and one 2-Myr look run for `frontstay.py`
(`+DEEPTIME_SNAPSHOT_MYR=2:-560:-500`). Read, against base38-s4 and against Earth where Earth has a number:

- the boundary on pairs that both open and close, and the convergent boundary within 1,000 km of the same pair's
  divergent one (`tools/review/crustcycle/polenear.py`; Earth 0.09 to 0.135 and 0.125 to 0.272 as first read, corrected
  to 0.17 to 0.22 and 0.14 to 0.29 on 2026-11-02);
- the front's life on its rock (`frontstay.py`: 0.58 to 0.66 after 10 Myr today) and why it ends;
- the crust behind the fronts (`forecrust.py`: Earth 0.165 with no 15 km crust within 200 km);
- the young floor at the intra-oceanic fronts and their length (`islrec.py`, `oofronts.py`, `cbread.py`);
- the census (`--measure v1census`), the guard (`--measure guard`), the shape watch (`shape_jumps.py`).

## What is left: the model's longest boundaries (2026-11-02, a read)

With Earth's pairs keyed whichever way Bird names them, Earth's mixed share is 0.17 to 0.22, and 0.33 with its
small plates merged down to the model's 16 (`tools/review/crustcycle/earthmerge.py`; plates.md "Earth's mixed pairs
corrected"). Like for like the model mixes 1.8 times as much (0.594), all of it on its pairs over 8,000 km: up to
8,000 km its pairs mix as Earth's do. The longest pairs' younger plates are young (born a median 84 Myr before the
run's end), a third of their boundary wraps one plate, and the pairs a subduction initiation made are mixed along all
of it. The next read is how a young plate comes to share over 8,000 km with one neighbour: the initiation's fracture
across its whole plate and round the parent's continents (PLT-690, PLT-503), and the rift's warped Voronoi edges.
