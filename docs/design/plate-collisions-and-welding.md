# Plate collisions and welding: design

Status: **design, nothing built** (2026-10-01). Ben chose "Design, then fix".

**Built 2026-10-02:** the collision state and real shortening (task 2), the belt, resistance and
weld on stall (tasks 3 and 4, as the force-balance design's stage 1). What was built and where it
departs from 4.1 (1C) and 4.2 (2B) is in `docs/design/plate-force-balance.md`, "As built: task 2"
and "As built: tasks 3 and 4"; the settings of section 5 are built except
`supercontinent_heating_myr` (3B, not needed: see the measurements there).

**Decided 2026-10-02** (decisions log): question 1 → full force-balance motion; question 2 → decide
about the mega-plate when measured; question 3 → both sides of the suture rise, with a foreland
basin; question 4 → the start-up jump is a separate small task afterwards. The extension is
`docs/design/plate-force-balance.md`. It supersedes this document where they differ: the belt of
4.1 (1C) becomes two-sided (its section 3.5), the resistance and weld rule of 4.2 (2B) become terms
of the balance (its 3.4), the settings of section 5 and the plan of section 7 are replaced by its
sections 6 and 9. The diagnosis (section 2), the Earth notes (section 3) and the acceptance table
(section 6) stand.
Scope: three plate-stage faults found by the run review
(`docs/research/notes/run-review-1a0f969d358.md`): collisions pop land out of the sea in one
frame, rifted oceans close again at once (no Wilson cycle), and welding builds a mega-plate.

Statements are marked **measured** (from runs made for this document), **read** (from the code)
or **inferred**.

## 1. Summary

**Diagnosis.** One rule causes two of the faults and feeds the third.

- `plates/subduction.rs` counts "continental interpenetration" as *relative speed × time* from the
  moment three continental samples overlap. It reaches the 300 km threshold after a median
  **8 Myr**, when the real shortening is a median **87 km** (measured).
- `plates/collision.rs` then welds the two whole plates at once (P1-27's suture rule) and lifts
  everything within up to 2,000 km of the contact by the same 1,400 m in that one step.

**Recommended design** (one mechanism, three effects):

1. **Uplift is the crust that was shortened.** While continental crust goes under, the crust
   consumed each step is stacked into a belt a few hundred km wide behind the front. The rate
   follows the convergence speed, the belt follows the suture, and the height is limited by the
   crust spreading that already exists. No uplift cap and no fixed duration.
2. **A collision resists convergence, and plates weld only once it has stalled.** Resistance grows
   with the height of the belt. Two plates become one when their convergence has stayed near zero
   and the shortening was real.
3. **No separate rule for the mega-plate at first.** It is the fast re-welding. A run with welding
   delayed (a crude stand-in) has one plate above half the planet in 4 % of steps instead of 59 %
   (measured, one run). A rift-hazard term for long-assembled supercontinents is the fallback if
   the target is missed.

**Did crust spreading already change any of this?** No (measured). Spreading and the ×0.7 uplift
lowered the surge from 2,000 m to 1,400 m. One collision still lifts up to 28 % of the planet and
adds up to 5 % of the planet as new land in a step. The mega-plate and plate lifetimes are as the
review found them.

**The main risk** is the planet's stock of continental crust. Real shortening consumes
continental area, and the stand-in run lost a third of it (0.44 → 0.29 of the planet). The
resistance is what keeps shortening to a few hundred km per collision; arc growth has to make up
the rest. Section 6 has targets for it and section 7 a calibration task.

## 2. Diagnosis

### 2.1 How it was measured

- Runs: `earthlike_quick_16k`, supercontinent start, 1000 Myr, 2 Myr steps, the coupled pipeline up
  to the volcanoes (`deeptime stage volcanoes`), on master e8668ca (crust spreading included).
  Test VM, 2 threads per run, 1,030–1,120 s each.
- Baseline: seeds 1–4. Variants (seed 1 only, so indicative): no welding
  (`collision_suture_frac=1e9`), a higher threshold (`collision_threshold_km=1500`), a narrow surge
  (`collision_distance_km=500`).
- Command: `tools/review/plate-dynamics.sh 1000 1,2,3,4 supercontinent base` through
  `tools/remote/mac.sh`. It needs a binary built with `tools/review/plate-dynamics-trace.patch`, a
  throwaway instrument that writes one line per step (every plate's sample count, land samples) and
  one per collision. `tools/review/plate-dynamics.py` turns that and the event list into the
  figures below. Section 6 proposes `deeptime plate-check` to replace the patch.
- Plate shares are exact (sample counts). The review read them from the `plates` keyframe layer,
  which holds the plate id modulo 20, so two plates can share a colour; the exact numbers agree
  with it anyway.
- Land is the share of all L0 samples above the plate model's sea datum at the end of a step. The
  review used the surface frames, so the two differ a little.
- Lengths are Earth-normalized km. A set's length and width come from its second moments
  (exact for a uniform strip); elongation is length ÷ width.
- "Real shortening" is rough: the subducting plate's continental samples that went under since
  the contact began, times a sample's area, over the contact's length. A sample is 71 km across,
  so it cannot resolve less than that.

### 2.2 The three faults, measured

Mean of seeds 1–4, range in brackets.

| Measure | Master now | Review (older run) | Earth |
|---|---|---|---|
| Largest plate, median share of the planet | 0.58 [0.50–0.66] | — | 0.20 today |
| Steps with one plate above 50 % | 59 % [50–69] | 51 % | none today |
| Steps with one plate above 30 % | 83 % | 80 % | |
| Largest plate, worst step | 0.90 [0.84–0.93] | 0.98 | |
| Plates above 5 % of the planet, median | 3.75 | 3 | 7 |
| Median plate lifetime (plates that died) | 45 Myr [38–56] | 32 Myr | 100–200 Myr for large plates |
| Plates dead within 20 Myr | 33 % | 39 % | |
| Rift-born plates: median lifetime | 16–22 Myr | 18 Myr | an ocean opens for 100–200 Myr |
| Rift-born plates alive after 50 Myr | 19 % [14–22] | 17 % | |
| Deaths by welding / by absorption of a tiny plate / consumed | 100 / 49 / 2 per run | 59 of 65 rift-born welded | |
| Most land gained in one step (after the first 10 Myr) | 3.8 % of the planet [2.6–4.6] | 6.0 % | |
| Most land raised by one collision in its step | 4.5 % [2.9–5.2] | | |
| Area one collision lifts by more than 10 m: median, most | 3.2 %, 23 % [15–28] of the planet | up to 15 % | |
| Uplift per collision | 1,400 m, every one | 2,000 m, every one | varies enormously |
| Lifted area (above 100 m): length × width, median | 7,100 × 2,800 km | "round blobs" | Himalaya 2,400 × 150–350 km |
| Lifted area: elongation, median (share at 3 or more) | 2.2 (25 %) | | 5–15 for young belts |
| Collisions per 100 Myr | 10 [9–12] | 11 | about 3–4 (P1-27's figure) |

More, all measured:

- **A collision fires at the first moment the counter allows.** Contact age at the collision:
  median 8 Myr, 90th percentile 25 Myr. Convergence there: median 54 mm/yr. The counter reads
  350 km; the continental crust that actually went under, over the contact's length, is a median
  87 km and a 90th percentile of 269 km (seed 1). 87 km is about one row of samples.
- **Rift-born plates are welded back into the plate family they left.** Of 51–67 welded rift-born
  plates per run, 14–20 went to the plate holding their parent's crust and 4–12 to a sibling of
  the same rift, after a median 11–21 Myr. The rest went to other plates after 12–27 Myr.
- **The mega-plate is rebuilt after every breakup.** A plate above half the planet is split by a
  rift 13–18 times per run. One plate is above half again after a median 17 Myr [14–22].
- **Welds move huge plates.** The plate that dies in a weld covers a median 19 % of the planet and
  up to 86 %. In half the welds (40–62 of 92–116) the dying plate is the larger of the two, so the
  mega-plate keeps changing its id. This inflates "plate deaths" and is why 60–77 % of the planet
  changes plate colour in one frame.
- **The top land jumps are collisions.** 17–20 of the 20 largest gains per run fall on a collision
  step. Steps without a collision gain at most 1.2–1.6 %.
- **The first 4 Myr add 4–6 % land with no collision.** This is the run's start-up (the review's
  "sharp start"), not part of this design.

### 2.3 Which rule causes what

**Fault 1: land pops up in round areas.** Read from `plates/collision.rs`:

- `collide()` applies the whole uplift in the step the collision fires (the loop over `dist`,
  lines 172–198). There is no later uplift from the collision.
- The amount is `min(collision_coefficient_per_km × area, collision_uplift_max_m)`. For any
  continent the first term is tens of km, so the cap (1,400 m) is always the answer. Measured:
  every collision is 1,400 m. This is a hand cap doing the work.
- The footprint is everything within `r` of the contact, `r = collision_distance_km ×
  √(speed/100 × area/A₀)`, clamped to 2,000 km. Measured median 1,440 km. After a weld the
  Dijkstra runs over both former plates, so the surge spreads to both sides. At 1,000 km from the
  contact the uplift is still 56 % of the peak.
- Variant, measured: with `collision_distance_km=500` a collision lifts at most 3.2 % of the planet
  and raises at most 0.9 % as land; elongation rises to 3.2. But the world ends with land 0.15 and
  a median land height of 288 m. **The wide surges are a main builder of land today**, so any fix
  needs a recalibration of the hypsometry.
- Crust spreading does not act here: it starts at 5.1 km and a surge raises lowland to about 2 km.

**Fault 2: no Wilson cycle.** Read from `plates/subduction.rs`, `resolve()`:

- `c.depth_km += conv × dt`, where `conv` is the *magnitude* of the relative velocity at the
  contact's centroid (lines 290–296). It is not the component across the boundary, so plates
  sliding past each other count as converging.
- It accumulates whenever at least three unlocked continental samples overlap anywhere along the
  contact, and never decays.
- At 300 km the collision fires; if the continent is at least `collision_suture_frac` (0.1) of an
  average plate, which is 0.7 % of the planet, the whole plate is welded (`collision.rs` 105–129).
- A rift cuts along a noise-warped line (`rift_warp_rad` 0.2, about 1,300 km of warp). Inferred:
  when the fragments move apart, some interlocking stretches of that line overlap by a sample or
  two. That is enough for the counter, and four steps later the fragment is welded back.
- Variant, measured: with the threshold at 1,500 km (so a weld needs about 30 Myr of contact),
  rift-born plates live a median 85 Myr and 68 % survive 50 Myr; the 8 welded back to their parent
  lived a median 128 Myr.

**Fault 3: the mega-plate.** Read from `collision.rs` 123–129: a weld moves every group of the
subducting plate, oceans included, to the other plate.

- Measured: with no welding at all, one plate is above half the planet in 19 % of steps (59 %
  with), plates live a median 180 Myr and 88 % of rift-born plates survive 50 Myr. So the review's
  attribution to the suture rule is right. That run is not a fix: it brings back P1-27's fault
  (active margins 0.67 of the coast against 0.30, land 0.46, median land height 1,610 m,
  continental crust 0.52).
- Measured: with the threshold at 1,500 km, one plate is above half in 4 % of steps, the median
  largest share is 0.26, and 6 plates are above 5 %. The mega-plate is split often enough already
  (rift hazard grows with plate area); it only persists because the pieces re-weld in 17 Myr.
  The active share of the margin stays at 0.37, so welding later keeps P1-27's gain.
- That run has its own fault: continental crust falls from 0.44 to 0.29 of the planet and land to
  0.16. Plates plough on at 70 mm/yr for 30 Myr and the crust that goes under is consumed at each
  resampling. Nothing in the model resists a collision.

**Related: continents shred.** Not measured here. Inferred: 630 small terrane transfers and 100
welds per run, each followed by a resampling, keep cutting and re-gluing continental crust. Fewer,
longer collisions should help; section 6 keeps the review's two measures as targets to watch.

## 3. What Earth does

Checked online on 2026-10-01 unless marked "from memory". Paywalled papers were confirmed from
their abstracts or from secondary pages.

**Plate sizes.**
- 52 plates in Bird's model. The Pacific is 103.3 million km², 20 % of the Earth. The seven largest
  (Pacific, North America, Eurasia, Africa, Antarctica, Indo-Australia, South America) cover about
  92 %. P. Bird 2003, *Geochem. Geophys. Geosyst.* 4(3):1027; areas from
  <https://en.wikipedia.org/wiki/List_of_tectonic_plates>. *Checked in WP-13a against Bird's own
  Table 1 (areas in steradians, full text): the Pacific is 2.577 sr, 20.5 % of the sphere; exactly
  seven plates hold over 5 % (Pacific, Africa, Antarctica, North America, Eurasia, Australia,
  South America); those seven cover 81 %, not 92 % (Bird splits India, Somalia and others off).*
- About seven large plates and many small ones for the last 200 Myr: Morra et al. 2013, *EPSL*
  373:93–101; Mallard et al. 2016, *Nature* 535:140–143 (slabs set the layout of the large
  plates). Whether the two groups are distinct populations is disputed
  (<https://www.geosociety.org/gsa-today/june-2018/broken-sheetson-the-numbers-and-areas-of-tectonic-plates>:
  the areas cannot be shown to come from more than one size distribution; read in WP-13a).
- No source gives the largest plate's share through a supercontinent's life. From memory: the
  plate carrying Pangaea was well under half the planet, with several oceanic plates in the
  surrounding ocean.

**Plate lifetimes.** No published statistic was found. Cases: the Pacific plate was born about
190 Ma and still exists; the Farallon plate subducted from the Early Jurassic and split about
23 Ma (Lonsdale 2005, *Tectonophysics* 404:237–264); Izanagi was gone by about 55 Ma and Kula by
about 48 Ma (<https://en.wikipedia.org/wiki/Pacific_plate>, `/Izanagi_plate`, `/Kula_plate`). So
large plates last 100–200 Myr or more, small ones tens of Myr. *The three Wikipedia figures
(190, 55, 48 Ma) were re-read in WP-13a; Lonsdale 2005 was not read.*

**The supercontinent and Wilson cycles.**
- Period about 300–500 Myr (<https://en.wikipedia.org/wiki/Supercontinent_cycle>; Nance 2022,
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC9796656/>). A supercontinent lasts 100–300 Myr
  (Mitchell et al. 2021, *Nat. Rev. Earth Environ.* 2:358–374). Pangaea assembled about
  335–325 Ma and began to break up about 200–175 Ma. *Checked in WP-13a: 300–500 is the
  Wikipedia page's figure only. Nance 2022 (read) gives "intervals of ca. 500 m.y." and Pangaea's
  tenure as c. 325–200 Ma; Mitchell et al. 2021 (accepted manuscript, read) give a cycle "often
  considered to last 400–800" Myr with about 600 Myr dominant, and tenures of 100–300 Myr.
  Wikipedia gives Pangaea's assembly as about 335 Ma and its break-up from about 175 Ma.*
- Oceans open for 100–200 Myr before they close: the Iapetus from about 600 to 420 Ma
  (<https://en.wikipedia.org/wiki/Iapetus_Ocean>); the Atlantic has been opening for about
  175–180 Myr. J. T. Wilson 1966, *Nature* 211:676–681. *Checked in WP-13a (Wikipedia only):
  the Iapetus opened in phases between 750 and 520 Ma and was gone by about 420 Ma; the
  Atlantic's opening is dated 200–170 Ma. Wilson 1966 was not read.*

**Collisions.**
- India and Asia met 59–50 Ma. Convergence fell from about 150 to about 40 mm/yr between 50 and
  35 Ma, **and the thickening of Tibet's crust was the main cause of the slowdown** (Copley,
  Avouac & Royer 2010, *J. Geophys. Res.* 115:B03410). It still continues, 50 Myr on. *The
  abstract (read in WP-13a) gives "∼15 to ∼4 cm/yr" and "crustal thickening in the Tibetan
  Plateau was the dominant cause".*
- Other collisions: Alleghanian about 325–260 Ma (65 Myr); Uralian about 70–90 Myr
  (<https://en.wikipedia.org/wiki/Alleghanian_orogeny>, `/Uralian_orogeny`). So a collision
  builds its range over tens of Myr. *Checked in WP-13a: the Alleghanian page gives 325–260 Ma;
  the Uralian page gives no length, only a start in the Late Carboniferous (from c. 323 Ma) and an
  end in the Triassic to Early Jurassic, about 70–140 Myr.*
- Shape: the Himalaya is 2,400 km long and 150–350 km wide (151–350 on the page); the Alps 1,200 × 250 km; the Urals
  2,500 km long. Tibet, behind the Himalaya, is 2,500 × 1,000 km at a mean above 4,500 m
  (Wikipedia pages of each). So young belts are 5–15 times longer than wide, and a long-lived
  collision grows a plateau behind the belt.
- Rates: rock uplift about 5 mm/yr in the Himalaya, 0.5–2 mm/yr in the western Alps (Nocquet et
  al. 2016, *Sci. Rep.* 6:28404); more than 1 mm/yr of exhumation in southern Tibet at 17–12 Ma
  (Tremblay et al. 2015, *PNAS* 112:12030). An orogen-wide mean surface uplift of 0.1–1 mm/yr is
  from memory.

**What happens to the plates.**
- Slab pull is the main driving force; plates attached to slabs move 3–4 times faster than the
  others, and continents add drag (Forsyth & Uyeda 1975, *Geophys. J. R. Astron. Soc.*
  43:163–200, from its abstract). *Corrected in WP-13a (claims.md fix 18), full text read:
  Forsyth & Uyeda give two groups, slab-attached plates at 6–9 cm/yr and the others at 0–4 cm/yr
  (p. 178); their Table 1 gives a ratio of mean speeds of 3.8. The words "∼4 times faster" are
  Conrad & Lithgow-Bertelloni 2004's (abstract).*
- The buoyant continent resists going down; the slab can break off (Davies & von Blanckenburg
  1995, *EPSL* 129:85–102).
- Two plates are one plate when they no longer move relative to each other. This is the
  definition of a plate; no source states it as a finding. India and Australia show the reverse:
  one plate now splitting into India, Capricorn and Australia (Royer & Gordon 1997, *Science*
  277:1268–1274).
- Why supercontinents break up is open. Insulation heats the mantle under them (Gurnis 1988,
  *Nature* 332:695–699; Coltice et al. 2007, *Geology* 35:391–394, up to about 100 °C). Others
  find the heating small (Heron & Lowman 2011, about 50 °C; Lenardic et al. 2011) and put the
  cause in the subduction around the supercontinent (Bercovici & Long 2014, *Geophys. Res. Lett.*
  41:6659–6666).

## 4. Options

Run time: every option below works on the fronts and distance fields the plate step already
computes. None adds a measurable cost. Fewer welds mean fewer forced resamplings (100 per run now,
83 with the higher threshold, 50 with no welds), which is a small saving.

### 4.1 Fault 1: collision uplift

**1A. Keep the surge, make it smaller.** A narrower radius and an amplitude from the area.
- Measured with radius 500 km: land per collision falls to 0.9 % of the planet at most.
- Still one frame, still every collision the same, still a cap. Rejected.

**1B. Spread the same surge over a fixed time.** Store the suture and apply 1,400 m over
`collision_orogeny_myr` in a belt along it.
- Removes the pop and the blob.
- Height and duration are still settings, identical for every collision. The suture must be
  tracked through resamplings. One more distance field per step. Not emergent.

**1C. Uplift from the crust that was shortened (recommended).**
- Mechanism. A front where continental crust goes under shortens by `v_n × dt` per step (`v_n`:
  convergence across the front). That crust, of thickness `H`, is stacked into a belt of width `W`
  on the overriding plate. A thickening `T` raises the surface by `T × (ρ_m − ρ_c)/ρ_m` (0.18, the
  Airy share already in `pools::ISOSTATIC_FACTOR`). So

      dz(d) = share × H × 0.18 × (v_n × dt) / W × k(d/W),   ∫ k = 1 over the belt

  with `d` the distance behind the front. With `share` 0.5, `H` 35 km, `W` 400 km and 50 mm/yr
  the belt rises a mean 0.4 mm/yr: 4 km in 10 Myr.
- Where. In `subduction::fronts_and_uplift`, for fronts whose incoming crust is continental (the
  `oceanic_slab` flag exists). It replaces Cortial's weak `h(z̃)` term there and the whole surge in
  `collide()`.
- What limits it. Height: crust spreading above the strength height, so a long collision widens
  into a plateau (Tibet) and gravity matters. Duration: the resistance of 4.2. Erosion works
  throughout. No cap, no fixed duration.
- Land per step: the belt is `W ×` front length, about 0.3 % of the planet for a 4,000 km front.
- Stamps. The Himalayan orogeny, folds and metamorphism go where the belt rises, each step, as
  the Andean ones do. `collision_orogen_core` goes away and the orogen provinces become belts.
- With spreading: it is the intended partner. With fault 2: it needs convergence to last, so it
  needs 4.2. Cost: a recalibration of the hypsometry, since the wide surges build much of today's
  land.

### 4.2 Fault 2: the Wilson cycle

**2A. Count real shortening, keep welding at the threshold.** Replace the counter with the
continental crust that went under, over the front's length.
- Stops welds along transform and scissor contacts, so most quick re-welds.
- A real collision would still weld after 300 km, about 6 Myr, with the uplift of 1C cut off at
  that moment. Mountains would be low and collisions still alike.

**2B. Resistance, and welding once convergence has stalled (recommended, with 2A's measure).**
- Entry. A contact is a collision while continental crust of the subducting plate goes under and
  the continent is at least `collision_suture_frac` of an average plate. Smaller terranes still
  transfer alone (Cortial), but on real shortening.
- Resistance. The belt's weight pushes back. In the units of the present speed law:

      r = collision_resistance_mm_yr × (belt height / strength height) × min(1, front / slab_pull_front_km)

  The plates' target convergence across the front is reduced by `r`, shared between the two plates
  by area, and never reversed. The belt height is the mean of what 1C built; the strength height is
  the one `surface/spread.rs` uses, so gravity and `crust_strength_mpa` enter here too.
- Emergent variety. A plate pulled by a long slab elsewhere keeps converging against a short
  front for tens of Myr and grows a plateau (India). A plate with no slab stalls once the belt is
  about 2 km high (Alps, Urals). Copley et al.'s finding for India is this mechanism.
- Weld. When the shortening passed `collision_threshold_km` (now real) and convergence across the
  front has stayed below `weld_speed_mm_yr` for `weld_after_myr`, the two plates become one, as
  P1-27's suture does now. The larger plate keeps its id. Collisions that stall before the
  threshold stay two plates and may part again.
- P1-27's gain is kept: nothing leaves the incoming plate until the weld, so its other coasts keep
  their ocean and stay passive.
- Timescale: the speed law relaxes over `slab_pull_relax_myr` (20 Myr), so a collision lasts about
  20–50 Myr. Inferred, to be measured.
- Risks: the crust budget (section 1); a collision that never stalls keeps eating a continent
  (targets in section 6 catch it).

**2C. Forbid re-welding by rule.** A rift-born plate cannot weld for N Myr, or not to its parent.
- Small. A hand rule that hides the faulty counter. Rejected.

**2D. Full force-balance plate motion.** Each plate's rotation from the balance of slab pull,
ridge push, mantle drag and collision resistance (Forsyth & Uyeda).
- The complete form of 2B, and it would also fix slab-less plates moving at 45 mm/yr.
- It replaces the speed law for every plate, so every calibration since P1-03 is redone (speeds,
  margins, ocean ages, hypsometry). A supercontinent start has no slabs, so it needs a mantle
  term to get going. Its own design document. Not needed for these three faults.

### 4.3 Fault 3: the mega-plate

**3A. Nothing beyond 2B, and the larger plate keeps its id (recommended first).**
- The measured cause is the 17 Myr re-weld. Rifts already split a plate above half the planet
  13–18 times per run.
- Inferred from the stand-in run (4 % of steps above half). To be confirmed by the build.

**3B. A rift hazard that grows under a long-assembled supercontinent (fallback).**
- Mechanism: the mantle under a large continent warms for 100–200 Myr (Gurnis; Coltice). The rift
  rate of a plate gets a factor `1 − exp(−t / supercontinent_heating_myr)` times its continental
  area, with `t` the time since the plate last rifted or welded.
- Also stops fresh fragments from rifting again at once.
- The physics is disputed (section 3), and the present hazard already grows with area. Build only
  if the target is missed after 2B.

**3C. A size limit, or a forced rift above it.** A hand cap. Rejected.

**3D. Leave welding as it is and split oceans off welded plates.** New boundaries would have to
be invented inside a plate with no relative motion to justify them. Rejected.

### 4.4 Recommendation

1C + 2B (using 2A's measure) + 3A, with 3B as the fallback. They are one mechanism: shortening
builds the belt, the belt resists, the stall welds. 2D is the honest long-term answer for plate
motion as a whole; it is a separate, larger piece of work (question 1).

## 5. Settings

| Setting | Change | Default | Editor field, lang key |
|---|---|---|---|
| `tectonics.collision_belt_km` | new; replaces `collision_distance_km` | 400 | existing `collision_belt_width` repointed; hint rewritten |
| `tectonics.collision_stacking_share` | new: share of the shortened crust that stacks into the belt | 0.5 | `collision_stacking`, `deeptime.editor.field.collision_stacking` |
| `tectonics.collision_resistance_mm_yr` | new: convergence a full-height belt on a full-length front cancels | 100 | `collision_resistance` |
| `tectonics.weld_speed_mm_yr` | new | 5 | `weld_speed` |
| `tectonics.weld_after_myr` | new | 10 | `weld_wait` |
| `tectonics.collision_threshold_km` | meaning changes: real shortening | 300 | existing |
| `tectonics.collision_suture_frac` | kept: above it a collision, below it a terrane transfer | 0.1 | existing `suture_size`; hint rewritten |
| `tectonics.collision_orogeny_myr` | kept: how long a belt reads as an active collision after its last uplift | 30 | existing |
| `tectonics.collision_uplift_max_m`, `collision_coefficient_per_km`, `collision_orogen_core`, `collision_distance_km` | removed | | fields and lang keys removed |
| `tectonics.supercontinent_heating_myr` | only with 3B | 150 | `supercontinent_heating` |

Each lives in `params.rs` (`TectonicParams`, with validation), the 13 preset files and
`tests/presets/planet_world_4k.json`, `crates/deeptime-viewer/assets/editor_fields.json` and
`neoforge/.../lang/en_us.json`. The plate-only presets get the same rules.

## 6. Acceptance targets

On `earthlike_quick_16k`, supercontinent start, 1000 Myr, seeds 1–4. Plate targets are asserted
on the mean of the four runs; the chaos between seeds is the bracketed range in section 2.2.

| Measure | Now | Target |
|---|---|---|
| Largest plate, median share | 0.58 | 0.15–0.35 |
| Steps with one plate above 50 % | 59 % | at most 10 % |
| Plates above 5 % of the planet, median | 3.75 | 5–9 |
| Median plate lifetime (plates that died) | 45 Myr | at least 80 Myr |
| Plates dead within 20 Myr | 33 % | at most 15 % |
| Rift-born plates alive after 50 Myr | 19 % | at least 60 % |
| Rift-born plates welded back to their own rift family within 50 Myr | about 40 % | at most 10 % |
| Land raised by one collision in one step | up to 4.5 % of the planet | at most 0.5 % |
| Most land gained in any step (after the first 10 Myr) | 3.8 % | at most 1.5 % (other processes reach 1.2–1.6 % today) |
| Collision belt width, median (ground stamped by one collision over its life) | 2,800 km | at most 800 km |
| Collision belt elongation, median | 2.2 | at least 4 |
| Collision length, onset to weld or parting, median | one step | 15–50 Myr; 90th percentile under 120 Myr |
| Peak uplift per collision | 1,400 m, all | spread: 90th ÷ 10th percentile at least 2 |
| Collisions per 100 Myr | 10 | 2–6 |
| Continental crust, share of the planet, every step | 0.38–0.45 at the end (the review: 0.37–0.45 through its run) | 0.37–0.45 |
| Active share of the continental margin (P1-27) | 0.30 [0.28–0.32] | 0.25–0.40 |
| Ocean floor, 90th percentile age | 208–269 Myr | at most 230 Myr |

Earth, checked in WP-13a (2026-10-02; rows CHK-001 to CHK-019 of `docs/science/ledger/CHK.md`):
the largest plate is 20.5 % of the sphere and seven plates hold over 5 % (Bird 2003, Table 1);
today's ocean floor has a 90th percentile age of 133 Myr, a median of 55 Myr and 0.6 % older than
180 Myr (Seton et al. 2020 age grid, area-weighted, computed). The 230 Myr ceiling is therefore
lenient by about 100 Myr; it stays until the plate rewrite is measured (question in STATUS.md).

Hypsometry, unchanged from the 2026-10-01 recalibration: `tools/review/p1-17-hyps.sh`, 500 Myr,
seeds 1–3 × both starts, mean of six:

| Measure | Target |
|---|---|
| Land share | 0.27–0.30 |
| Median land height | 430–640 m |
| Land above 2 km | 0.10–0.13 |
| Highest L1 cell | 5.5–7 km |
| 99th percentile of land | 4.2–5.5 km |
| Sample-steps at the plate model's elevation guard | 0 |

Note: at 1000 Myr master's land share is already low (0.19–0.26 in the four baseline runs), the
review's "drifting low". The hypsometry gate stays at 500 Myr; the 1000 Myr value is reported.

Also reported, not asserted: the review's continent measures (pieces of continental crust; edge
share), to see whether fewer transfers and welds help.

**The check.** A reusable metric is missing, so the plan proposes it rather than building it here:

- The plates stage records a `notes.plates.dynamics` series next to `notes.plates.margins`: per
  step the largest plate's share, the plates above 5 %, the land samples; per collision its onset,
  end, real shortening, front length, peak uplift, the land it raised per step and the second
  moments of the ground it stamped.
- `deeptime plate-check --in <checkpoint> [--events <world data>]` prints the table above and
  exits non-zero when a value is outside its range, as `strata-check` does. The ranges are
  arguments with the defaults above.
- An ignored release test and `tools/review/plate-check.sh` (four seeds, about 18 minutes each at
  2 threads) run it. It is too slow for the default `cargo test`.

## 7. Build plan

Each task ends with the gates (cargo fmt, clippy, test, deny; `./gradlew build test`) and a
STATUS entry with the `plate-check` table. Tasks 2–4 change the built-in planets; GameTests are
the coordinator's, once, after task 5.

| # | Task | Model | Settings |
|---|---|---|---|
| 1 | **`plate-check`.** The `notes.plates.dynamics` series, the CLI check, a unit test on a short run, `tools/review/plate-check.sh`. Baseline table on master recorded. Remove `plate-dynamics-trace.patch` and point `plate-dynamics.py` at the note, or delete both. | Sonnet (well specified: section 6 and the patch show every quantity) | none |
| 2 | **Real shortening and the collision state.** `Contact` gains the continental area that went under and its front length; `depth_km` becomes area ÷ length. A collision is a state of the contact with an onset event. Terrane transfer on real shortening. Welding still at the threshold in this task, so its effect is measured alone. | Opus (contacts, resampling and events interact) | `collision_threshold_km` hint |
| 3 | **Belt uplift.** The law of 4.1 in `fronts_and_uplift`; the surge and its four settings removed; orogeny, fold and metamorphism stamps per step; the root added to the rock pools as the cordillera does. Tests: the volume raised equals the crust shortened × share × 0.18 within 10 %; a straight collision's belt has elongation above 4. | Opus (cross-subsystem: surface feedback, pools, provinces) | `collision_belt_km`, `collision_stacking_share`; four removed |
| 4 | **Resistance and welding on stall.** The speed law in `plates/mod.rs` gets target vectors and the collision term; weld conditions; the larger plate keeps its id; `orogeny_phase` written at the end with its real length. Tests: two continents with no slab stall and weld within 60 Myr; with a long slab on the incoming plate they are still converging after 40 Myr; a contact with no shortening never welds. | Opus (kinematics) | `collision_resistance_mm_yr`, `weld_speed_mm_yr`, `weld_after_myr` |
| 5 | **Recalibration.** Ensembles for the hypsometry (`collision_stacking_share`, `collision_belt_km`, `subduction_uplift_mm_yr`), the crust budget (`arc_continental_m`), the margins and the plate targets; then the full presets. Test ranges updated. | Opus (interacting knobs; the sweeps themselves are scripted) | preset values only |
| 6 | **Views and docs.** Events in the workbench show a collision's length and shortening; the plan's §8.1 and P1-27 text, the editor hints and the lang file brought in line; the review's two continent measures added to `plate-check` as reported values. | Sonnet | none |
| 7 | **Fallback: supercontinent heating (3B).** Only if "steps with one plate above 50 %" is still over 10 % after task 5. | Opus | `supercontinent_heating_myr` |

Order: 1, then 2 → 3 → 4 in sequence (each builds on the last), then 5, 6. Task 2 alone is
expected to remove most quick re-welds; tasks 3 and 4 must land together in a merge batch, since 3
without 4 has nothing to stop a collision except the old threshold.

## 8. Questions for Ben

1. **How far should the motion model go?**
   - **(a, recommended) Collision resistance inside the present speed law, welding on stall.**
     Fixes the three faults; plate speeds, margins and ocean ages keep their calibration.
   - (b) Full force-balance plate motion now (slab pull, ridge push, drag, collision). More
     complete and more emergent; every plate calibration is redone and it needs its own design.
   - (c) Only the real-shortening counter and a longer threshold. Smallest; collisions stay
     identical and the uplift still needs a cap.
2. **If the mega-plate target is still missed after the fix, what then?**
   - **(a, recommended) Add the supercontinent-heating rift term (3B) as task 7.**
   - (b) Accept a looser target (one plate above half the planet in up to 25 % of steps), on the
     ground that a supercontinent's plate is large on Earth too.
3. **Which side of the suture rises?**
   - **(a, recommended) The overriding plate only, for now.** One belt behind the front; simple
     and enough for the targets.
   - (b) Both sides, with a foreland basin on the incoming plate (the Himalaya proper is Indian
     crust). More faithful; better done with the folds-and-faults work.
4. **The start-up land jump** (4–6 % of the planet in the first 4 Myr, no collision; the review's
   "sharp start").
   - **(a, recommended) A separate small task after this one.**
   - (b) Fold it into task 5's recalibration.
   - (c) Leave it.
