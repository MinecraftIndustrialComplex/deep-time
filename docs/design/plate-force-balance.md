# Force-balance plate motion: design

Status: **design approved 2026-10-02; task 2 built** (see "As built: task 2" at the end of
section 9). It extends
`docs/design/plate-collisions-and-welding.md` (the "collisions design" below) after Ben's decisions
of 2026-10-02: full force-balance motion; nothing designed for the mega-plate yet; both sides of a
suture rise, with a foreland basin; the start-up land jump is a separate small task afterwards.

Statements are marked **measured**, **read** (from the code) or **inferred**.

## 1. Summary

**The design.**

- Each step, every plate's rotation is the one at which the forces on it balance. Driving: slab
  pull (by trench length, slab length and slab age), ridge push and slab suction. Resisting: drag
  under the plate (more under continents), resistance to the slab's sinking, resistance along
  plate boundaries, and the weight of collision belts.
- Ridge push and collision resistance are one term: the gravitational potential energy of the
  model's own relief at a plate's edge. Gravity and crust strength enter there.
- Two plates are one plate when they have stopped moving relative to each other. That single rule
  replaces welding on contact, and also heals rifts that fail.
- The belt of the collisions design becomes two-sided: stacking on the overriding plate, a wedge
  on the incoming plate, and a flexural basin in front of the wedge that the surface stage fills.

**What the measurements say** (open loop: the balance computed on the present model's plate
geometry, four runs; section 4).

- A balance does produce Earth's best-known pattern. Speed against the share of a plate's edge
  attached to slab: rank correlation 0.76–0.84, against 0.47–0.59 in the present model.
- With drag under the plate alone it is not usable: 13–19 % of plate-steps are faster than
  150 mm/yr (up to 2,500), and continental plates stand still.
- Slab resistance alone does not fix that. Boundary resistance does: with it 1–3 % of plate-steps
  exceed 150 mm/yr, oceanic plates do 66–72 mm/yr and continental ones 27–33 (Earth: about 85 and
  26–28).
- The raw balance jumps: a plate's rotation changes by a median 16–18 % from one step to the next
  and by 62–70 % at the 90th percentile (present model: 5 % and 9 %). It needs slab-length
  tracking and a short relaxation time to be smooth.

**Honest assessment.**

- This is the right long-term model and it is feasible. It removes six hand rules (section 3).
- It is also the riskiest change since the plate model was written. Every plate calibration moves,
  and the closed loop (the balance driving the geometry that feeds it) is not tested by anything
  here.
- The build order keeps a fallback at every stage: the collision and belt work lands first on the
  present speed law, and the balance replaces the speed law as the last step.
- Estimate: 19–21 agent-days of work beyond task 1, 14–16 of them on the critical path. A
  quarter of it is recalibration, and that is the uncertain part.

## 2. How plates move today (read)

**State.** Each plate has one angular-velocity vector `omega` (its Euler pole and rate) in the
mantle frame. Each step every rigid group turns by it (`plates/kinematics.rs`, `advance`).

**Where a plate's motion comes from.**

| When | Rule | Where |
|---|---|---|
| Start of a run | A random pole at least 30° from the plate; speed drawn from `plate_speed_mm_yr` (20–70), times a slow-continent factor | `init.rs` 234–239, `kinematics::random_motion` |
| Every step: direction | The pole turns towards the plate's trenches by `slab_pull_rad_per_1000km_myr` per 1,000 km of front | `mod.rs` 212–215, `kinematics::pull_axis` |
| Every step: speed | Relaxes over `slab_pull_relax_myr` (20) towards a target: the larger of `slab_pull_speed_mm_yr` (80) × min(1, front ÷ 3,000 km) and a base of 45 mm/yr, times 1 − 0.4 × continental share | `mod.rs` 205–224 |
| Every step: noise | A random walk of 2 % per √Myr | `kinematics::jitter` |
| Every step: limit | Capped at `max_plate_speed_mm_yr` (100) | `kinematics::cap_speed` |
| Rift | Each fragment gets the parent's motion plus 20–50 mm/yr away from the other fragments | `rifting.rs` 231–250 |
| Subduction initiation | The new plate of old floor gets the parent's motion plus 30–60 mm/yr towards the rest | `rifting.rs` 429–439 |
| Collision or weld | The receiving plate takes the area-weighted mean of the two motions | `collision.rs` 200–208 |

**Rules that a force balance makes emergent.**

1. The base speed: a plate with no slab moves at 45 mm/yr for no reason.
2. The slab speed and its front-length ramp.
3. The slow-continent factor.
4. The speed cap.
5. The speeds handed out at rifts and at subduction initiation.
6. The motion after a collision, and (collisions design) the slowdown, the stall and the weld.

**Measured, the present model** (four 1000 Myr runs, plates of at least 30 samples, per step):
speed 41 / 71 / 83 mm/yr at the 10th, 50th and 90th percentile. Nothing is slower than 5 mm/yr.
Oceanic plates (under 20 % continental) 73–74 mm/yr, continental ones (over 50 %) 42–49: a ratio
of 1.6. Earth's spread is far wider: about 85 mm/yr for the fastest group and 26–28 for the
slowest (Zahirovic et al. 2015).

## 3. The force model

### 3.1 The balance

Mantle flow is slow and viscous, so a plate has no inertia: at every moment the torques on it sum
to zero (Forsyth & Uyeda 1975). Per plate `p`, with `W` its angular velocity:

    M_p W_p  =  T_p  +  (terms that couple it to its neighbours)

- `M_p` collects every resistance that grows with speed. It is a symmetric positive 3 × 3 matrix.
- `T_p` collects every force that does not depend on speed.
- The neighbour terms come from boundary and collision resistance, which act on *relative* motion.
  So all plates are solved together: 3 unknowns per plate, at most 120 for 40 plates, one small
  dense solve per step.

The matrix is positive definite, so the solve has one answer and cannot oscillate by itself.

### 3.2 The terms

Forces are per unit length of boundary or per unit area, on the Earth-normalized sphere. Only
ratios between the terms and one overall scale matter; section 6 lists the settings.

| Term | Law | Acts on | Notes and sources |
|---|---|---|---|
| **Slab pull** (drives) | Per length of trench: `slab_pull × thickness(age) × slab length ÷ slab_depth_km`, towards the trench. Thickness ∝ √age, zero for crust younger than about 10 Myr. Continental crust in the trench pulls nothing. | The subducting plate | The dominant force; an order of magnitude above the rest (Forsyth & Uyeda 1975, abstract). Only 8–12 % of a slab's weight reaches the plate, up to twice ridge push (Schellart 2004, *GRL* 31:L07611). Speed follows slab age at the trench (Carlson, Hilde & Uyeda 1983, *GRL* 10:297). |
| **Slab resistance** (resists) | Per length of trench, proportional to the speed towards it | The subducting plate | Gives slab-attached plates a "terminal velocity" (Forsyth & Uyeda). Replaces the speed cap. |
| **Slab suction** (drives) | Per length of trench, `slab_suction_share` of the pull, towards the trench | The overriding plate | Pull and suction together explain why subducting plates are about four times faster than the others (Conrad & Lithgow-Bertelloni 2002, *Science* 298:207; 2004, *JGR* 109:B10407; from abstracts). |
| **Edge energy: ridge push and belt weight** (drives or resists) | Per length of edge, `U(edge) − U(plate interior)`, pushing the plate away from high ground. `U` is the gravitational potential energy of an isostatic column at that elevation: `ρ g h (H + h (1 + ρ_c/(ρ_m − ρ_c)) / 2)` on land; for sea floor the same from its depth. | Every plate, at every edge | A ridge stands high, so it pushes: about 3 × 10¹² N/m for old plates (Parsons & Richter 1980, *EPSL* 51:445). A 5 km belt on 35 km crust gives 6.7 × 10¹² N/m by this formula; Tibet is quoted at 6–7 × 10¹² (Molnar & Lyon-Caen 1988, GSA Spec. Pap. 218). Ridge push is under 10 % of the driving force (Lithgow-Bertelloni & Richards 1998, *Rev. Geophys.* 36:27). |
| **Collision resistance** (resists) | Along a collision front, a resistance to convergence whose strength is the belt's edge energy; see 3.4 | Both plates, on their relative motion | India slowed from 150 to 40 mm/yr as Tibet thickened (Copley, Avouac & Royer 2010). |
| **Basal drag** (resists) | Per area, proportional to speed; `keel_drag` times more under continental crust | Every plate | Continental roots raise the traction 2–5 times (Conrad & Lithgow-Bertelloni 2006, *GRL* 33:L05312, abstract). A factor of 8 is often attributed to Forsyth & Uyeda; not confirmed. |
| **Boundary resistance** (resists) | Per length of any plate boundary, proportional to the relative speed across and along it | Both plates | Forsyth & Uyeda's transform resistance. Small for large plates. Needed for small ones (section 4). |
| **Mantle flow** (drives, weak) | The basal drag acts on the speed relative to a slow, smooth, slowly changing flow of `mantle_flow_mm_yr` | Every plate | A stand-in for convection the model does not have. It replaces the random walk and starts a planet with no slabs (question 1). |

Known sizes, for orientation: slab pull about 1–3 × 10¹³ N/m gross, ridge push 2–3.5 × 10¹² N/m,
a high plateau 6–7 × 10¹² N/m. Gross slab pull is from textbook examples and was not confirmed in
Turcotte & Schubert itself.

### 3.3 Slabs

A contact keeps one number: the length of slab hanging from it.

- It grows by the convergence each step, up to `slab_depth_km` (700: the upper mantle).
- A new subduction zone therefore starts weak and gains pull over about 10 Myr.
- When continental crust enters the trench along a stretch, that stretch pulls nothing. When it
  has done so for `slab_breakoff_myr` (10; van Hunen & Allen 2011: 10–25 Myr), the slab there is
  gone and the length restarts from zero.
- Subduction initiation no longer hands out a speed. It plants a slab of a starting length on the
  old floor, and the balance moves the plate.

### 3.4 Collisions

The collisions design stands, with three changes.

- **Resistance.** Its "resistance in mm/yr" becomes the collision term of the balance. The force
  is the belt's edge energy, applied against the convergence. It is passive: it can stop two
  plates, not push them apart. In the solve it is a drag whose coefficient is the force divided by
  the last step's convergence (bounded below by `weld_speed_mm_yr`), the usual way to put a
  yield-type resistance into a linear solve.
- **Why collisions now stall by themselves.** The trench stretch that the continent entered stops
  pulling (3.3) while the belt's resistance grows. A plate with long slabs elsewhere keeps going
  and widens a plateau; a plate without them stops.
- **Welding.** Two plates become one when their relative motion along their whole shared boundary
  has stayed below `weld_speed_mm_yr` for `weld_after_myr`. No shortening threshold is needed: in
  a force balance two plates do not share a rotation by chance. The same rule merges the two sides
  of a rift that did not open.

### 3.5 Both sides of the suture, and the foreland basin

This replaces section 4.1, option 1C, of the collisions design.

- **Budget.** As before: the crust shortened in a step, times its thickness, times
  `collision_stacking_share`, raises ground by the Airy share (0.18).
- **Overriding side.** A belt `collision_belt_km` wide behind the front, as before. It grows into
  a plateau by crust spreading.
- **Incoming side: a wedge.** Ground on the incoming plate rises towards the front along a fixed
  surface slope, `wedge_taper_deg` (1.5°), up to the height of the belt at the front. So the
  wedge's width is the belt's height over that slope: 190 km for a 5 km belt. Crust moves through
  the wedge and under the front; the wedge stays at the front. What the wedge takes is subtracted
  from the overriding side's budget. Source for a fixed taper: Davis, Suppe & Dahlen 1983, *JGR*
  88:1153 (from memory).
- **Foreland basin.** The belt and wedge are a load on the incoming plate. Beyond the wedge's toe
  the plate bends down by the line-load flexure profile
  `w(x) = w₀ e^(−x/α) (cos(x/α) + sin(x/α))`, with `α = (4D / (Δρ g))^¼` and `D` from the elastic
  thickness (Turcotte & Schubert). For `foreland_elastic_km` 30–70, α is 93–176 km (computed), a
  foredeep 200–400 km wide and a low forebulge beyond it. `w₀` follows from the load per length of
  front, so a higher belt has a deeper basin, and lower gravity a wider one.
- **Fill.** The subsidence reaches the surface stage as negative uplift. Its rivers and marine
  deposition fill the basin from the belt. The deflection is tracked per sample and follows the
  load, so the basin rebounds as the belt wears down.
- Earth, for the test: the Himalayan foreland is about 450 km wide and 3 to over 7 km deep
  (<https://en.wikipedia.org/wiki/Himalayan_foreland_basin>). Foredeeps of 100–300 km and 2–8 km,
  forebulges tens to a few hundred metres high (DeCelles & Giles 1996, *Basin Res.* 8:105; via a
  secondary source).

**Feeding folds and faults (P2-01 stage B).** `structure/deform/emplace.rs` reads L1 fields each
step: the collision regime (full shortening), a foreland within `foreland_km` of any active orogen
on every side, the fold trend, convergence and the ground's slope (thrusts dip under high ground).
With this design:

| Structure stage input | Today | Change |
|---|---|---|
| Collision regime | The surge's core, both former plates | The belt and the wedge. Fold trains and thrusts then follow the suture as two belts |
| Shortening strength | 1 everywhere in the regime | The step's convergence across the front, so a slowing collision makes fewer new structures |
| Fold tightness | From a drawn shortening | From the contact's real shortening over the belt's width (a new L1 field, `plates/shortening`) |
| Thrust dip | Under the high ground | Unchanged; on the wedge that is towards the suture, which is right |
| Foreland | Distance to an orogen, all round | The flexural basin (a new L1 field, `plates/foreland_m`): only on the incoming side, gentle folds and thrusts in its fill, none on the forebulge |
| Unconformities | Cover younger than a structure is undeformed | Unchanged. The basin's fill onlaps the forebulge, which stage B's rule already draws |
| Stocks in collision belts | In the regime | Unchanged, now in a belt |

The `foreland_km` setting of the structure stage goes; its distance rule stays only for Andean
margins.

## 4. Open-loop test of the balance (measured)

**Method.** Four runs of the present model (`earthlike_quick_16k`, 1000 Myr; supercontinent and
scattered starts, seeds 1 and 2), traced each step: per plate its areas, boundary lengths, drag
tensors and the summed lever arms of each force (`tools/review/plate-dynamics-trace.patch`).
`tools/review/plate-forces.py` solves the balance per plate and step for several sets of ratios
and scales the result to an area-weighted RMS speed of 45 mm/yr. The plates keep moving by the
present rules, so this tests the balance on realistic geometry, not the closed loop.
Simplifications: the slab term uses age but not slab length; slab resistance uses the trench's
summed arm; boundary resistance acts against a fixed frame, not the neighbour.

Range over the four runs:

| | Present model | Basal drag only | + slab resistance | + slab and boundary resistance |
|---|---|---|---|---|
| Speed, 10th / 50th / 90th percentile, mm/yr | 41 / 71 / 83 | 0–5 / 60–72 / 165–209 | 13–17 / 54–61 / 161–202 | 27–31 / 62–68 / 95–109 |
| Fastest plate-step, mm/yr | 99 (the cap) | 2,000–2,500 | 1,500–2,100 | 450–700 |
| Plate-steps above 150 mm/yr | 0 | 13–19 % | 11–17 % | 1.3–3.1 % |
| Oceanic / continental plates, median mm/yr | 73 / 42–49 | 62–79 / 0–5 | 54–62 / 14–21 | 66–72 / 27–33 |
| Speed against slab share of the edge (rank correlation) | 0.47–0.59 | 0.76–0.84 | 0.32–0.53 | 0.69–0.77 |
| Change of rotation per step, median / 90th pct | 5 % / 9 % | 16–19 % / 60–74 % | 22–26 % / 86–96 % | 16–18 % / 62–70 % |
| Plate-steps turning about a pole near themselves | 0.5–2 % | 10–14 % | 24–32 % | 13–17 % |

Columns: ridge push 0.1 of slab pull throughout; from the third column on, suction 0.3 and keel
drag 3; slab resistance 3 and boundary resistance 3 times the basal drag of an average starting
plate.

What it shows:

1. **The pattern is there.** Fast oceanic plates, slow continental ones and the slab correlation
   come out without being put in.
2. **Small plates are the problem.** Speed falls with plate area (rank correlation −0.5 to −0.7).
   A small plate has little drag under it, so any imbalance spins it. Slab resistance damps only
   the motion towards the trench. Boundary resistance ties a small plate to its neighbours, which
   is also what happens on Earth.
3. **Continents need suction, keels and boundary coupling to move at all.** With drag only they
   stand still, because the present geometry gives them huge plates and few slabs.
4. **The answer is noisy from step to step.** Fronts are found per sample (71 km), so torques jump
   as samples enter and leave. Slab-length tracking smooths the largest term; a relaxation time of
   a few steps smooths the rest.

## 5. Options

Cost in simulation time: the forces are summed in loops the plate step already runs, and the solve
is a dense system of at most 120 unknowns. Neither is measurable against a pipeline step of about
two seconds. The flexure needs one more bounded distance field per step on the incoming side;
estimated at a few per cent of the plate stage, under 1 % of a run (not measured).

**5.1 When to solve.**
- **(a, recommended) Every step, then relax towards the answer** over `motion_relax_myr` (6).
  Forces change at every step; the relaxation is the smoothing section 4 asks for, and it stands
  for the time slabs and mantle take to respond.
- (b) Every step, applied at once. Purest; section 4 measures 62–70 % jumps at the 90th percentile.
- (c) Every 10 Myr. Cheaper by nothing that matters, and collisions would be missed for 10 Myr.

**5.2 Collision resistance: viscous or plastic.**
- **(a, recommended) Plastic, linearised with the last step's speed.** A force set by the belt's
  weight, whatever the speed: the stall is sharp and the height it happens at depends on gravity.
- (b) Viscous: resistance proportional to convergence. Simplest and always stable, but convergence
  only decays and never stops, and the belt's height would set nothing.
- (c) A full contact solve (plates stick or slip). Exact; an iterative solver for little gain.

**5.3 Slab pull as slabs lengthen and break.**
- **(a, recommended) One slab length per contact**, as 3.3.
- (b) From the trench alone (age and length of trench). What the open-loop test used. New
  subduction zones pull at full strength at once and nothing breaks off.
- (c) Slab length per trench sample. Along-trench detail (tears) that the plate model cannot use.

**5.4 A plate with no slab.**
- **(a, recommended) Ridge push, suction, its neighbours, and a weak mantle flow.** It drifts at
  5–25 mm/yr, as Africa and Antarctica do.
- (b) No mantle flow. A planet that starts with no slabs stands still until the first subduction
  initiation; a supercontinent with none on its rim never moves.
- (c) Keep a base speed. The rule being removed.

**5.5 Edge energy or separate laws for ridge push and belts.**
- **(a, recommended) One edge-energy term from the model's elevation.** Ridge push, the push of a
  high plateau and the pull of a deep trench come from one formula with gravity in it.
- (b) Ridge push by ridge length and plate age, belt resistance by belt height, as two laws. Easier
  to tune one at a time; two more settings and no link between them.

**5.6 Flexure.**
- **(a, recommended) The line-load profile in the plate model**, in front of collision wedges.
- (b) A general elastic-flexure solve in the surface stage, replacing its blurred local isostasy.
  Right for every load (ice, volcanoes, deltas). A new solver on L1 each step: an estimated 5–10 %
  of run time and 3–4 more agent-days. Worth having later; not needed for this.

**5.7 Rifting.** The decision is to design nothing for the mega-plate now. For the record, the
choice that will come up:
- (a) Keep the present hazard (by area, continental share and flood basalts) and let the balance
  decide whether a rift opens. This is what the plan below builds.
- (b) A hazard from the plate's own tension, which the balance can report (section 8).

## 6. What it replaces

**Code** (read).
- `mod.rs` 204–228, the motion block: replaced by the force sums, the solve and the relaxation.
  New module `plates/forces.rs`.
- `kinematics.rs`: `pull_axis`, `jitter` and `random_motion` go; `cap_speed` stays as a guard.
- `subduction.rs`: `Pull` (front length and axis) is replaced by the force sums; `Contact` gains
  the slab length.
- `rifting.rs`: fragments keep the parent's motion; the initiation plants a slab.
- `init.rs`: plates start at rest plus the mantle flow.
- `collision.rs`: as in the collisions design; the momentum mix at a weld stays (area-weighted),
  and the balance takes over the next step.

**Settings** (each is a field of `tectonics` in the 13 presets and the test planet, an editor field
in `editor_fields.json` and a lang key `deeptime.editor.field.<id>`).

| New | Default | Editor id |
|---|---|---|
| `plate_speed_scale` (the one overall scale: the RMS speed the balance is normalised to at calibration) | 45 mm/yr | `plate_speed` (repointed) |
| `slab_pull` (relative strength, 1 = reference) | 1 | `slab_pull` |
| `slab_depth_km` | 700 | `slab_depth` |
| `slab_breakoff_myr` | 10 | `slab_breakoff` |
| `slab_resistance` | 3 | `slab_resistance` |
| `slab_suction_share` | 0.3 | `slab_suction` |
| `edge_energy` (ridge push and belt weight, relative to slab pull) | 0.1 | `ridge_push` |
| `keel_drag` | 3 | `keel_drag` |
| `boundary_resistance` | 3 | `boundary_resistance` |
| `mantle_flow_mm_yr` | 5 | `mantle_flow` |
| `motion_relax_myr` | 6 | `motion_relax` |
| `wedge_taper_deg` | 1.5 | `wedge_taper` |
| `foreland_elastic_km` | 50 | `foreland_stiffness` |
| `weld_speed_mm_yr`, `weld_after_myr`, `collision_belt_km`, `collision_stacking_share` | as the collisions design | as there |

| Removed | Why |
|---|---|
| `plate_speed_mm_yr`, `continental_speed_factor` | speeds come from the balance |
| `slab_pull_speed_mm_yr`, `slab_pull_front_km`, `slab_pull_relax_myr`, `slab_pull_rad_per_1000km_myr` | the pseudo slab pull |
| `motion_jitter_per_sqrt_myr` | replaced by the mantle flow |
| `rift_speed_mm_yr`, `subduction_initiation_speed_mm_yr` | speeds handed out by rule |
| `collision_resistance_mm_yr`, `collision_threshold_km` for welds | the collisions design's stand-ins (the threshold stays for terrane transfer) |
| `structure.deformation.foreland_km` for collisions | the basin field replaces it |
| `max_plate_speed_mm_yr` | kept, raised to 250, as a guard that is reported and should never bind |

**Calibrations that move.**

| Calibration | Why it moves | Knobs |
|---|---|---|
| Plate speeds | all of it | the ratios of section 3.2, `plate_speed_scale` |
| Sea-floor age | spreading and subduction rates change; slow plates keep old floor | `subduction_initiation_*`, `slab_pull` |
| Active margin share (P1-27) | which margins subduct depends on where slabs form and persist | `subduction_initiation_rate`, `_clearance_km` |
| Hypsometry | subduction uplift scales with convergence speed, whose spread widens; belts replace surges | `subduction_uplift_mm_yr`, `collision_stacking_share`, cordillera |
| Continental crust budget | shortening per collision is set by the balance | `arc_continental_m`, `collision_stacking_share` |
| Rift rules | a rift opens only if forces open it; failed rifts heal | `rift_lambda0` |
| Arc and volcano rates, deposits that read convergence | follow speed | re-checked, not retuned first |

## 7. Acceptance targets

The table of the collisions design (its section 6) stands. Added, on `earthlike_quick_16k`,
1000 Myr, seeds 1–4, both starts where stated; "now" is the present model.

| Measure | Now | Target | Earth |
|---|---|---|---|
| Plate speed, area-weighted RMS | about 65 mm/yr | 35–55 | about 40 |
| Plate speed, 10th / 50th / 90th percentile of plate-steps | 41 / 71 / 83 | at most 20 / 30–60 / 70–110 | slowest group 26–28, median 40, fastest 85 |
| Oceanic plates ÷ continental plates, median speed | 1.6 | at least 2 | about 3 |
| Plates with 20 % or more of their edge on slab ÷ plates with under 5 %, median speed | 1.6 | at least 2.5 | 3–4 |
| Speed against slab share of the edge, rank correlation | 0.47–0.59 | at least 0.65 | the Forsyth & Uyeda result |
| Speed against plate area, rank correlation | −0.36 to −0.46 | between −0.4 and +0.2 | none (Forsyth & Uyeda) |
| Plate-steps above 150 mm/yr | 0 (capped) | at most 0.5 % | rare, short |
| Steps on which the speed guard binds | not counted | 0 | |
| Change of a plate's rotation per step, median / 90th pct | 5 % / 9 % | at most 8 % / 20 % | |
| Convergence 20 Myr after a collision's onset ÷ at onset, median | 0 (welded) | 0.2–0.6 | India: 0.27 in 15 Myr |
| Sea-floor age, median | 56–100 Myr | 50–80 | 64 |
| Sea floor older than 180 Myr / older than 250 Myr | 12 % (the review) / 5–12 % | at most 10 % / 3 % | close to none / one small basin |
| Supercontinent tenure (largest connected continent holds 70 % or more of the continental crust, until it holds under 50 %) | about 360 Myr from the start (the review's run) | 100–300 Myr | 100–300 Myr |
| Assembly-to-assembly period | about 650 Myr (the review's run) | 300–600 Myr | 300–500 Myr |
| Foredeep width and depth in front of a belt above 3 km, median | none | 150–450 km, 2–7 km of fill | 100–450 km, 2–8 km |
| Wedge on the incoming side: width, median | none | 100–350 km | Himalaya 150–350 km |

Notes:
- "Now" for the RMS is an estimate from the percentiles; `plate-check` will give it.
- The area correlation is negative today and in the open-loop test because of the small plates.
  The target says the balance must not make large plates slow merely for being large.
- The supercontinent rows need a run that assembles one; a scattered start is the test of
  assembly, a supercontinent start of tenure.

**More for `plate-check` to report** (task 1 is under way from the collisions design; these are
additions):

- Per plate and step: RMS speed, share of the edge on slab, continental share, the change of its
  rotation since the last step, whether its pole lies inside it.
- The force budget per plate: the size of each torque term, and the plate's mean in-plane stress
  (tension or compression) from its edge forces.
- Per contact: slab length, convergence across the front per step (the collision slowdown), the
  wedge's and foredeep's width and depth.
- Per step: connected pieces of continental crust and the largest one's share (the supercontinent
  measures and the review's two continent measures); sea-floor age percentiles; guard binds.

## 8. The mega-plate: what the physics says

Nothing is designed here, as decided. The question was whether rim subduction in a force balance
breaks a supercontinent by itself.

**Not by itself in this model, and probably not on Earth either.**

- In the balance a plate is rigid. Trenches all round a supercontinent pull its rim outward
  (suction), the pulls cancel, the plate stands still and is in tension. A balance has no way to
  break a plate: something must decide that it fails.
- On Earth the tension from the rim is real, but the forces available (a few 10¹² N/m) are widely
  held to be too small to tear thick, cold continental lithosphere without weakening by magma or
  heat (Buck 2004; Bialas, Buck & Qin 2010; from abstracts). Models differ: slab rollback alone can
  disperse a supercontinent (Bercovici & Long 2014), rim subduction can supply the tension (Dal
  Zilio et al. 2018), or the upwelling under the supercontinent dominates and old orogens are the
  weak lines (Zhang et al. 2018; Huang et al. 2019, 20–50 MPa of extension in the interior).

**What the balance does change** (inferred).

- The model already has the decider: rifts by area, continental share and flood basalts. Today
  the pieces get a random push and re-weld in 17 Myr. Under the balance, pieces of a supercontinent
  with trenches on their outer sides are pulled apart and stay apart; pieces with nothing pulling
  them do not open, and heal.
- So a supercontinent ringed by subduction should break at its next rift, and one without a ring
  should persist. That is close to what is believed of Pangaea.

**What to measure before deciding anything.**

1. The largest plate's share over time (the existing target).
2. The mega-plate's mean stress: is a plate above 40 % of the planet in tension, and for how long
   before it rifts?
3. Of the rifts of such a plate, the share whose fragments are still separate 50 Myr later, split
   by whether the fragments had slab on their outer edge.
4. Supercontinent tenure.

If tension is common and rifts of plates in tension succeed, nothing more is needed. If
mega-plates sit in tension for hundreds of Myr waiting for a rift, the next step is a hazard that
grows with tension (5.7 b) and, after it, the heating term.

## 9. Build plan

Task 1 (`plate-check`) is under way and taken as given. Days are agent-days; a day is one agent's
full working session with its gates.

| # | Task | Model | Days | Settings | Depends on |
|---|---|---|---|---|---|
| 1 | `plate-check` and baseline (collisions design) | Sonnet 5.5 | under way | none | |
| 1b | `plate-check` additions of section 7 that need no new physics: speeds, slab share, rotation change, sea-floor age, continental pieces | Sonnet 5.5 | 1 | none | 1 |
| 2 | Real shortening and the collision state (collisions design, task 2) | Opus | 1.5 | `collision_threshold_km` hint | 1 |
| 3 | Two-sided belt: stacking, wedge, flexural foreland; surge removed; stamps; L1 fields `plates/shortening`, `plates/foreland_m` | Opus | 3 | `collision_belt_km`, `collision_stacking_share`, `wedge_taper_deg`, `foreland_elastic_km`; four removed | 2 |
| 4 | Resistance in the present speed law and welding on stalled relative motion (collisions design, task 4). **The fallback state.** | Opus | 2 | `collision_resistance_mm_yr`, `weld_speed_mm_yr`, `weld_after_myr` | 3 |
| 5 | Forces, read-only: `plates/forces.rs` sums every term each step and solves the balance, slab length on contacts, results recorded in the note but not applied; `plate-check` gains the force budget and stress. Open-loop numbers of section 4 reproduced in the model. | Opus | 2.5 | all of section 6's force settings (inert until task 6) | 1b |
| 6 | Structure stage reads the belt, wedge, shortening and basin (3.5) | Sonnet 5.5 | 1 | `structure.deformation.foreland_km` narrowed | 3, and P2-01 stage B merged |
| 7 | **The switch.** Motion from the balance with relaxation; rifts and initiation without speeds; plates start at rest; collision term in the solve; old motion settings removed; guard reported. Tests: two-plate cases with known answers; a continent on a slab-pulled plate slows and stalls; no plate-step above the guard in a 200 Myr run. | Opus | 3 | removals of section 6; `collision_resistance_mm_yr` removed | 4, 5 |
| 8 | Recalibration: speeds, sea-floor age, margins, hypsometry, crust budget, on the quick preset; then the full presets | Opus | 4–6 | preset values | 7 |
| 9 | Views and docs: workbench shows a plate's forces and a collision's history; plan §8.1; editor hints and lang file | Sonnet 5.5 | 1 | none | 7 |

- **In parallel:** track A is 2 → 3 → 4 (collisions and belts); track B is 1b → 5 (forces,
  read-only). Task 6 runs beside 4 or 5. Tasks 7 to 9 are in sequence, with 9 beside 8.
- **Land together:** 3 and 4 (a belt with nothing to stop the collision is worse than today).
  7 lands with at least a first pass of 8, or the built-in planets are uncalibrated on master.
- **Checkpoints for Ben:** after task 4 the three review faults should be fixed on the old motion
  law (a usable state). After task 5 the model reports what the balance *would* do, closed-loop
  risk still unpaid. Task 7 is the point of no easy return.
- **Total:** 19–21 agent-days beyond task 1. The critical path is 2 → 3 → 4 → 7 → 8: 13.5–15.5
  days in sequence. In calendar time that depends on how many sessions run back to back; at two
  in sequence per calendar day it is about 7–8 days.
- Calibration is the uncertain part: each ensemble is 4–6 runs of 18 minutes, and five families of
  targets interact. If it goes as the hypsometry did (three passes), add 3–4 days.

### As built: task 2 (2026-10-02)

Branch of the plate-motion task 2 agent; `plates/shortening.rs`, `state::Collision`. Where it
departs from the plan above and from the collisions design's task 2:

- **What is counted.** Not "continental area that went under ÷ front length" (the collisions
  design's wording) but the convergence across the front, integrated: the flux of the relative
  velocity through the overriding plate's edge (Voronoi edges of the lattice) divided by the
  front's length. On a straight front the two are the same quantity; the flux form needs no count
  of hidden samples (which a resampling consumes) and gives exactly zero for strike-slip on the
  lattice's jagged edges. A first version, the velocity along each edge sample's mean normal, read
  19 % of a strike-slip speed as convergence: the Fibonacci lattice's normals lean one way.
- **The front** is where continental crust of the subducting plate stands at the overriding
  plate's edge (within 1.75 spacings), not where it lies in the overlap: the overlap is consumed at
  every resampling and comes back over a few steps. The state survives 3 steps with no front.
- **The state** carries onset, last step, shortening, crust crossed (km²), convergence now, at
  onset, peak and 20 Myr after onset, front length and peak, centroid and outward normal, and the
  share of the front that is continental on the overriding side. It ends in a transfer or weld
  (`CollisionRec`, whose `start_myr` is now the state's onset), or with no transfer (a new
  `PartingRec`: parted, reversed, closed, emptied, open). `collide` writes a `BeltRec` with the
  convergence at onset and 20 Myr later; task 3 adds the wedge and foredeep to it.
- **The interim trigger.** The plan said "welding still at the threshold in this task, so its
  effect is measured alone". Measured on `earthlike_quick_16k` (supercontinent, 1000 Myr) with real
  shortening at 300 km: one plate above half the planet in 24 % of steps instead of 59 %, rift-born
  plates welded back 28 % instead of 42 %, but continental crust ends at 0.37 of the planet instead
  of 0.42 and its lowest step is 0.34 (target at least 0.37), because a collision now lasts 10 Myr
  instead of one step and nothing stacks or resists. Thresholds of 100–250 km do not help the
  mega-plate (53–72 % of steps). So the trigger is a setting, `tectonics.collision_on_shortening`,
  **off in every built-in preset**: the old count (relative speed × time, kept exactly as
  `Contact::speed_count_km`) fires transfers and welds and the planets are those of master; the
  collision state runs beside it and is recorded. Task 4 removes the setting and the old count.
- **Hooks for tasks 3–5.** Task 3 stacks each step's `area_km2` growth behind `centroid` along
  `normal`, on both plates, and fills `BeltRec`'s wedge and foredeep. Task 4 reads
  `convergence_mm_yr` for the stall and the resistance, and replaces `shortening::advance`'s
  trigger. Task 5 starts a slab's break-off clock at `onset_myr` on the stretch `front_km` long.

## 10. Risks

1. **The closed loop is untested.** Section 4 is open loop. Geometry and forces feed each other:
   a plate that speeds up lengthens its slab, which pulls harder. Slab resistance and the slab
   depth limit bound that, on paper.
2. **Small plates.** Measured: 1–3 % of plate-steps above 150 mm/yr even with boundary resistance,
   and 13–17 % turning about a pole near themselves. The model makes many small plates (old-floor
   slivers from subduction initiation, rift fragments). Mitigations, in order: boundary resistance
   against the real neighbour (stronger than the test's fixed frame); the relaxation; a larger
   minimum plate size. The guard at 250 mm/yr stays and its binding is reported; if it binds in
   more than a handful of steps the balance is not right yet.
3. **Noise at the quick preset's resolution.** Fronts are one sample (71 km) wide and torques jump
   as samples cross. Measured: 62–70 % changes at the 90th percentile before any smoothing.
4. **Dead or frozen planets.** With no slabs nothing drives. A supercontinent start has none. The
   mantle flow and slab planting at initiation are there for this; if they are too weak the
   history stalls, and if too strong they are the old base speed under another name.
5. **Everything is recalibrated at once.** Hypsometry took three passes in the last week on a
   fixed plate model. Here the plate model moves too.
6. **Run time.** Not a risk from the solve. A wider spread of speeds makes the fastest plate
   decide the resampling interval; the count of resamplings is to be watched.
7. **Other work in flight** reads plate fields (P1-26 climate, folds and faults, deposits). Their
   tests assert properties, not planets, but their calibrations were made on today's planets.

**Fallback.** If the balance cannot be made stable at the quick preset's resolution within
task 7's budget plus two days, master stays at the state after task 4: the present speed law with
collision resistance and welding on stall. That state fixes the review's three faults, keeps
today's speed calibration, and has the two-sided belts and basins. The force sums of task 5 stay
in as a reported diagnostic. The balance can then be retried at the full preset's resolution
(250,000 samples, fronts 45 km) or after smoothing the fronts.

## 11. Questions for Ben

1. **What moves a plate that nothing pulls, and a planet that starts with no slabs?**
   - **(a, recommended) Ridge push, suction and neighbours, plus a weak mantle flow** (5 mm/yr, a
     setting) and slabs planted by subduction initiation. Honest label: the mantle flow is a
     stand-in, not physics the model computes. Without it some planets will not start.
   - (b) No mantle flow. Purest. Supercontinent starts and slab-less plates stand still until a
     subduction zone forms; some histories will be dull for the first 50–100 Myr.
   - (c) Keep a base speed for slab-less plates. This is the rule you asked to remove.
2. **The speed guard.**
   - **(a, recommended) Keep `max_plate_speed_mm_yr` at 250 as a reported guard**, as with the
     30 km elevation guard: it should bind in no step, and a test says so.
   - (b) Remove it. If a small plate runs away, the run shows it instead of hiding it; it can also
     wreck a long run.
3. **How the work lands.**
   - **(a, recommended) In two stages**: collisions and belts on the present speed law first
     (tasks 2–4, a usable state and the fallback), then the switch to the balance (5, 7, 8).
     Costs about two days of work that the switch then replaces.
   - (b) One stage: everything on the balance, landed once. Saves those two days. No fallback, and
     master keeps today's faults for the whole build.
4. **Flexure.**
   - **(a, recommended) The line-load profile in front of collision wedges only.**
   - (b) A general flexure solve in the surface stage now, for every load. About 5–10 % of run
     time and 3–4 more days. Better physics for ice sheets and volcanoes too; not needed for the
     three faults.
5. **Calibration budget.** The plan gives recalibration 4–6 days and treats the quick preset as
   the gate, the full presets after it.
   - **(a, recommended) Accept that the built-in planets change twice** (after task 4 and after
     task 8), with a light calibration at the first.
   - (b) Calibrate fully at both points. About 3 more days.
   - (c) Hold everything on a branch until task 8 is done. Master stays as it is for a week or more
     and the branch drifts from the other agents' work.

Left listed, out of scope: the start-up land jump (4–6 % of the planet in the first 4 Myr), a
separate small task after this work.

## 12. Sources

Checked online on 2026-10-02 by a sub-agent; most papers are paywalled, so figures are from
abstracts or secondary pages unless a link is given. Items it could not confirm are marked in the
text.

- Forsyth & Uyeda 1975, *Geophys. J. R. Astron. Soc.* 43:163–200,
  <https://academic.oup.com/gji/article/43/1/163/586101>.
- Schellart 2004, *Geophys. Res. Lett.* 31:L07611 (abstract read in full).
- Conrad & Lithgow-Bertelloni 2002, *Science* 298:207–209; 2004, *J. Geophys. Res.* 109:B10407;
  2006, *Geophys. Res. Lett.* 33:L05312.
- Lithgow-Bertelloni & Richards 1998, *Rev. Geophys.* 36:27–78.
- Carlson, Hilde & Uyeda 1983, *Geophys. Res. Lett.* 10:297–300.
- Parsons & Richter 1980, *Earth Planet. Sci. Lett.* 51:445–450.
- Zahirovic, Müller, Seton & Flament 2015, *Earth Planet. Sci. Lett.* 418:40–52,
  <https://www.earthbyte.org/tectonic-speed-limits-from-plate-kinematic-reconstructions/>.
- Molnar & Lyon-Caen 1988, GSA Special Paper 218:179–207; Copley, Avouac & Royer 2010,
  *J. Geophys. Res.* 115:B03410.
- van Hunen & Allen 2011, *Earth Planet. Sci. Lett.* 302:27–37.
- DeCelles & Giles 1996, *Basin Research* 8:105–123; Turcotte & Schubert, *Geodynamics* (flexure).
- Coltice, Husson, Faccenna & Arnould 2019, *Sci. Adv.* 5:eaax4295,
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC6821462/>: "Slabs pull plates at rapid rates and tear
  continents apart, with keels of continents only slowing down their drift when they are not
  attached to a subducting plate."
- Bercovici & Long 2014, *Geophys. Res. Lett.* 41:6659–6666; Dal Zilio, Faccenda & Capitanio 2018,
  *Tectonophysics* 746:312–324; Zhang et al. 2018, *Geosci. Front.* 9:997–1007; Huang et al. 2019,
  *Geochem. Geophys. Geosyst.* 20:4830–4848,
  <https://par.nsf.gov/biblio/10127684-modeling-inception-supercontinent-breakup-stress-state-importance-orogens>.
- Buck 2004, in Karner et al. (eds), *Rheology and Deformation of the Lithosphere at Continental
  Margins*, 1–30; Bialas, Buck & Qin 2010, *Earth Planet. Sci. Lett.* (volume not confirmed).
- From memory, not checked: Davis, Suppe & Dahlen 1983, *J. Geophys. Res.* 88:1153–1172.
