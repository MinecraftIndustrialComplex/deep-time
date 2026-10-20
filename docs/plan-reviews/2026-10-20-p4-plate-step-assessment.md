# The 4-Myr plate step (P4): independent assessment

2026-10-20. Reviewed: merge 74a8768a (P4 opt-in, `tectonics.time_step_myr=4` with
`surface.substeps=2`), the P4 section of docs/science/plates.md, the author's `build/p4/`
(compare.py, metrics.py, selection-rule.txt, out/screen, out/fullcmp), the run logs in
/data/ban/deeptime-runs/p4/, the code diff 74a8768a^1..74a8768a and master's plates/flow.rs. No new
simulations. "C20" is the default (2-Myr step); "twin" is C20 with the serial flood fill (A/A).

## Findings

**1. The comparison does not describe today's master.** Every run is on 0c368e27. Since then master
has merged the continent fix (bc2e5472: lower-crustal flow PLT-300/301 and settling ISO-078; land
0.16 to 0.22 before, 0.31 to 0.32 after), the CO2 cap (abc76bf3, CLM-206, x1.75; the p4s2 logs show
`present_co2_factor=13.805`, so the runs were uncapped) and the sea-floor young stage and amplifier
(49f5f543, which changes baked metrics). The new flow carries two once-per-step limits (finding 5)
that push a 4-Myr step toward thicker crust, the same direction as P4's main effect. The high-ground,
climate and baked numbers below would all move. plates.md says so itself.

**2. P4 makes high ground worse against Earth, which is what Ben asked to protect.** Means over the
nine full-16k seed pairs (Earth from HYP-001..006, ISO-071 and the climate page):

| | Earth | C20 | p4s2 | Read |
|---|---|---|---|---|
| highest cell, coupled, m | 8,849 (a point) | 6,754 | 9,071 (11,265 in su1) | 9 of 9 up, ranges apart |
| land above 1 / 2 / 3 km, share of land (baked) | 0.253 / 0.102 / 0.048 | 0.441 / 0.227 / 0.108 | 0.507 / 0.284 / 0.159 | further from Earth; 2 km shift is chaos-sized |
| land median / p90, m (baked) | 427 / 2,022 | 857 / 3,074 | 1,048 / 3,673 | further |
| land share | 0.289 | 0.208 | 0.215 | no change |
| sea median depth, m; hypsometry L1 to Earth | 4,062; 0 | 2,914; 0.595 | 3,144; 0.532 | closer (ocean side) |
| Köppen B / D share of land | 0.32 / n.a. | 0.215 / 0.089 | 0.259 / 0.043 | B closer; D outside C20's range |
| T, °C; natural CO2, ppm (uncapped) | 14.1; 280 | 19.7; 673 | 20.6; 1,167 | further; CO2 within chaos |
| stratovolcanoes standing / active | | 497 / 120 | 384 / 72 | born 9 of 9 down |
| strata log layers / horizons per land column | | 6.53 / 16.03 | 4.43 / 15.67 | 8 of 9 down / unchanged |

Every high-ground row already overshoots Earth under C20 and p4s2 moves it further; the only
Earth-ward moves come from the ocean floor and the dry climates. The strata a player digs are
unchanged (horizons); what coarsens is the time record (event times and the strata log land on 4-Myr
step ends, deterministically). **Two of nine p4s2 worlds (sc1, sc4) end with no active
stratovolcano**, and the screen's P4 arms also reach zero; C20 never does (minimum 56 in the full
runs, 12 in the screen) and neither does the twin. The cause was not examined.

**3. The flag counts are not comparable between rows, and the twin is too small to be the floor.**
The rule flags a metric on Mann-Whitney p <= 0.1, |d| >= 0.8, all of five or more pairs one way,
Wilcoxon p <= 0.1, or a twin-difference test; it ignores direction, so moving toward Earth or toward
more plate checks met counts against P4. Its null rate depends on the design:

- Matched row (two pairs a side): d is roughly t-distributed with 2 degrees of freedom, so
  P(|d| >= 0.8) is about 0.51 under the null and Mann-Whitney cannot reach p <= 0.1. The A/A matched
  row flags 483 of 1,146 (42 %), exactly the null. That row says nothing.
- Full A/A row (C20 nine runs on both starts, twin two scattered-start runs): null P(|d| >= 0.8)
  about 0.33, observed 293 of 1,146 (26 %). The sides differ in starts, which already shows (log
  layers per land column 4.80 vs 6.53, outside C20's range).
- p4s2 (nine pairs): 461 of 1,157 (40 %), above a null of roughly 0.15 to 0.25. P4 does change worlds
  beyond chaos in aggregate, but which metrics cannot be read off the twin.

Chaos within one pair is large: land above 2 km 0.384 vs 0.221 (sc1) and 0.196 vs 0.328 (sc2);
natural CO2 1,460 vs 534 (sc1). The screen A/A moved "plate checks met" by +2.5, outside C20's range,
so p4s2's 26.8 to 25.7 (the screen had +1.5) is noise. Robust: highest cell, belt peak p50 (1,392 to
2,281 m, outside range), stratovolcanoes born, log layers, and the step-quantized times. Not robust:
plate checks met, land share, land above 2 km, CO2.

**4. The selection picked P4's worst high-ground variant, on a margin inside the noise.** The rule was
written first (good; its Wilcoxon part was added later, the author reports the same order). Scores:
p4s2 84, p4fs2 96, p4 and p4fix 108, A/A 32. A flag count over about 207 correlated metrics has an sd
of roughly 7 to 10, so 84 against 96 is not a separation. p4s2 won only on plate-check flags (48 vs
65); on the headline numbers it lost to p4fs2 (30 (36) vs 28 (31)), and on high ground it was the
worst arm in the screen (highest cell 9,255 m vs p4fs2 7,985 and C20 7,038; land above 2 km 0.334 vs
0.290 and 0.248). The unmerged rebound hold-back (29bfe825, afaec853) alone takes about 1.2 km off
(p4 8,989 to p4fix 7,790 m). The full runs use fresh seeds and resolution, so their p values are not
inflated by the pick, but they test the wrong arm for Ben's question.

**5. Code.** The P4 diff leaves the 2-Myr path identical: `blur_owed` (surface/advect.rs) returns
None at dt <= 2 Myr (ERO-185), `spread_steps()` is 1 at 2 Myr (ERO-186), the climate carry's blur
keys on the plate step so it is off at C20, and the VOL-040 slot test (volcanoes/mod.rs:1035) gives
the same answer at 2 Myr. Collapse, relaxation, trench share, subduction initiation, weathering, the
PLT-288 uplift cap, rifting and the resample bounds all scale correctly with the step. No bug in the
diff. The step-dependent limit that biases P4 is on master, outside the diff:

- crates/deeptime-core/src/plates/flow.rs:64 `MAX_SUBSTEPS = 64` per plate step, applied at :174
  to :175, and the active set (channel columns plus two rings) found once per step at :117; flow::step
  runs once per plate step (plates/mod.rs:444). A 4-Myr step gets the same 64 substeps and two rings
  as a 2-Myr one, so where the cap binds the thickest crust drains about half as fast per Myr at full
  16k (plates.md's own estimate) and a channel front reaches half as far per Myr. Both mean thicker
  crust and higher ground under P4. Not in the measured runs (no flow on 0c368e27); in any re-run.
- Minor: the carry blur's stencil uses the receiving cell's variance, so it is not exactly
  conservative where the variance varies; the spread loop sums `flow.cells` over sub-steps (a
  diagnostic double count at P4); the VOL-071 10-Myr arc cadence becomes 12 Myr; hotspot births stay
  one per step; contacts come from 4 Myr of unsub-stepped motion (the last three are in the author's
  audit).

**6. Timing.** The paired bench is one seed, two counterbalanced rounds, with about 18 CPUs of other
load: ratios 0.73 and 0.75. The nine full pairs (not run side by side) give a coupled-stage ratio of
median 0.79, range 0.63 to 0.97, sums 9,504 vs 7,415 s (0.78); the plates stage is steady at 0.55 to
0.64 of C20, the surface stage ranges 0.73 to 1.17. Against the 900 s gate (1,242.8 s last
measured), 0.75 to 0.79 gives 930 to 980 s: P4 alone does not meet it.

## Recommendation

**(c), with (a) meanwhile.** Keep C20 the default and P4 opt-in as merged; that costs nothing, since
the 2-Myr path is identical. Do not make P4 the default on this evidence and do not re-run it as is:
the one clearly real world effect is more and higher mountains on worlds that already overshoot
Earth, and master's flow adds a step-dependent push the same way. Before a re-run:

1. Sub-step `flow::step` at no more than 2 Myr (or scale `MAX_SUBSTEPS` by dt/2 and rebuild the two
   rings each 2 Myr) so the flow law does not depend on the step.
2. Merge the rebound hold-back and run P4 with it (the p4fs2 shape: best headline score, lower peaks).
3. Find why some P4 worlds end with no active stratovolcano.
4. Re-run on master: 9 C20, 9 P4 and a 9-run twin on the same seeds and starts, so the A/A row has the
   P4 row's design; fix beforehand a directional, Earth-referenced score (highest cell, land above
   1/2/3 km, hypsometry, CO2 against the cap, active volcanoes) instead of a count of differences.

Cost: 27 full-16k runs at 870 to 1,270 s plus about 110 s of bake each, three at a time on the VM:
about 3 hours of wall, about $5 at $38 a day. A quick-16k screen of the fixed P4 (six runs a setting,
the existing design) first is cheaper and catches a failed fix. The speed gain (about 21 %) is real
and the high-ground cost is real; Ben asked for the second to be protected, and it is his call.
