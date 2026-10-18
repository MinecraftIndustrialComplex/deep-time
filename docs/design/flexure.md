# Lithospheric flexure in the surface stage: design

Status: **built** (2026-10-02; sections 1–11 are the design as written before the code, corrected
where the build changed it; section 12 is what was measured; section 13 the artifacts found after
merging, their causes and fixes). Ben's decision of 2026-10-02 (decisions
log): "General flexure everywhere": every load bends the crust, not only collision belts. This note
replaces option 5.6 (a) of `docs/design/plate-force-balance.md` (the line-load profile in the plate
model) and is what its task 3 (the two-sided belt) relies on for the foreland basin.

Statements are marked **measured** (a prototype or a run made for this note), **read** (from the
code) or **inferred**.

## 1. Summary

- The lithosphere is a thin elastic plate floating on the mantle. A load is carried partly by the
  mantle pushed aside under it (buoyancy) and partly by the plate's stiffness, which spreads the
  sinking over a flexural wavelength. A narrow load on a stiff plate barely sinks under itself and
  pulls its surroundings down with it: a moat round a volcanic island, a foreland basin in front
  of a belt, a low bulge beyond.
- Today every load is compensated where it lies (Airy), with erosion and deposition blurred over
  50 km. The solve replaces that blur and the "where it lies".
- Loads: crust thickened by the plate model, crust moved by the plateau spreading, rock eroded and
  deposited, volcanic edifices, water, and land ice.
- Stiffness varies: young ocean floor, arcs, rifts and young orogens are weak, old ocean floor and
  cratons are stiff. A plate that stiffens keeps the shape it has.
- Method: the plate equation on the L1 raster, solved by conjugate gradients with a spectral
  preconditioner. A prototype needed 4 to 10 iterations; the built solver needs about 20 on a
  real planet (section 12).
- Two editor settings: how strongly flexure replaces local isostasy (0 = today's behaviour), and
  a scale on the elastic thickness.

## 2. Isostasy today (read)

| Where | What it does |
|---|---|
| Plate model, every uplift law | Uplift is an elevation change. Crust thickened by `T` raises the ground by `T / 5.5` (`pools::ISOSTATIC_FACTOR`, ρ_m 3,300 and ρ_c 2,700 kg/m³): the root is local and immediate. |
| Surface step 5, `isostasy` | The step's net erosion and deposition `dm`, times `isostasy` (0.8, and 0.7 of that under water), blurred over `isostasy_radius_km` (50 km: three box passes of 10 cells on the quick preset), is taken off the elevation. |
| Surface step 5b, `spread.rs` | Crust above its strength height flows sideways: a conservative diffusion of elevation, so again Airy. |
| Volcano stage | A cone's growth is added to the elevation with no compensation at all. The next surface step erodes it and compensates the erosion. |
| Ocean floor | Depth follows age (thermal subsidence); a trench is a prescribed depth. Neither is a load. |

So the model has no place where a load on one cell moves another, except the 50 km blur.

## 3. The model

### 3.1 Equation

Deflection `w` (m, down positive) of a thin elastic plate over a fluid mantle:

    ∇²(D ∇²w) + ρ_m g w = g q

- `q`: the load, kg/m². `D = E Te³ / (12 (1 − ν²))`: flexural rigidity, with `Te` the effective
  elastic thickness, `E` 70 GPa and `ν` 0.25 (Turcotte & Schubert).
- The flexural parameter `α = (4D / (ρ_m g))^¼` sets every width: a line load makes a basin
  `3πα/4` wide and a bulge at `πα`. With `Te` 10 / 30 / 70 km, `α` is 30 / 67 / 127 km.
- With `D = 0` the equation is Airy isostasy, `w = q / ρ_m`. That is the link to today's model.
- Gravity enters twice: in `α` (lower gravity, wider basins: `α ∝ g^−¼`) and nowhere else, since
  load and buoyancy both carry `g`.
- Where `D` varies, the operator is `∇²(D ∇²w)`. The full plate equation has further terms in the
  second derivatives of `D`; they vanish where `D` varies in one direction and are small where it
  is smooth. Left out, and `Te` is smoothed (3.4).

### 3.2 Loads

A load is a change of the mass in a column since the reference state. The reference is the start
of the run: the starting planet is taken to be in equilibrium as it stands, so inherited relief
(a passive margin, a craton's root) bends nothing.

| Load | Source | Counted as |
|---|---|---|
| Crust thickened by the plate model | `plates/uplift_mm_yr` where it is positive, on continental crust, or on oceanic crust in an arc or hotspot regime | Uplift `u` is Airy elevation: the thickening is `u / (1 − k)`, `k` = ρ_c/ρ_m on land, (ρ_c − ρ_w)/(ρ_m − ρ_w) under water |
| Crust moved by plateau spreading | the elevation change of `spread.rs` | the same conversion, both signs |
| Erosion and deposition | the step's net change `dm` | ρ = `isostasy` × ρ_m on land; under water the density that today's 0.7 factor implies (2,300 kg/m³: sediment) |
| Volcanic edifices | what later stages added to the surface since the last step (the volcano stage's cones) | rock at ρ_c, no root yet |
| Water | the depth of sea over the cell, against the depth at the reference | ρ_w 1,030. A basin that sinks below the sea is loaded by its water; a rising sea loads the shelves and the ocean floor |
| Land ice | `climate/ice_thickness_m` | ρ_i 917 (section 7) |

Not loads: the ocean floor's thermal subsidence (the lithosphere itself contracts), the trench
profile and negative uplift in general (the plate model pulling a sample to trench depth), and
crust that arrives under a cell from another plate (its own loads arrive with it).

### 3.3 Combining with the existing isostasy without double counting

The model keeps what it has: elevations are Airy elevations, and the plate model's root is already
in them. Flexure is a correction to where the sinking happens. Per step:

1. The surface stage keeps two fields with the crust: the load `Q` (m of mantle displaced under
   Airy) and the deflection `w` the solve gave for it.
2. This step's loads are added to `Q`. For each one the Airy sinking **already applied by someone
   else** is noted as `a`: the plate model's root under its uplift, the root moved by spreading.
   For erosion, deposition, cones, water and ice nothing has been applied (the surface's own Airy
   step is switched off when flexure is on).
3. The solve gives the new deflection `w + δw` for the whole load.
4. The elevation changes by `−(δw − a)`.

- With `D = 0` the solve returns `δw` = the Airy sinking of every load: step 4 reproduces today's
  isostasy exactly (without the blur), and plate uplift is unchanged.
- Each kilogram is in `Q` once, and each metre of sinking is applied once: by the plate model
  (`a`) or by the solve (`δw − a`).
- **As built after the follow-up (section 13):** each step solves only for the step's load
  increments; the carried plate is taken to be in equilibrium. The design first re-solved the
  whole load every step so that "nothing drifts"; the carried state's interpolation error then
  acted as a force. What a solve leaves unfinished is carried with the crust and solved with the
  next step's loads, so nothing is dropped either. When a belt wears down or ice melts the
  negative increment brings the plate back.
- The plate model learns the result through the existing feedback (the surface's elevation over
  its own).

**A plate that stiffens keeps its shape.** A belt built on hot, weak crust has a local root. When
that crust cools and stiffens, nothing should move: the new stiffness is added stress-free at the
shape the plate has. Without this, every old belt would slowly rise as its root is "spread".
The model (as built after section 15): when `D` grows nothing moves and later loads meet the
stiffer plate; when it falls to `r·D` (reheating, a cell becoming a plate's broken edge), the
plate stops holding up the share `1 − r` of the load it kept away from local isostasy,
`ρ_m (Q + ice − w − unsolved)`, and that is a force. (The first build kept a frozen moment `M₀`
and re-solved the whole load; from section 13 to section 15 the plate carried its bending moment
`m = D ∇²w` and released `L((1 − r)·m)`, which section 15 replaced.) This is the observed rule
that a load records the plate's strength at the time of loading (Watts 1978: seamounts built on
young floor keep a low `Te`).

**Water** is a load that depends on the answer (a basin that sinks takes more water). The solve
handles its own share exactly: cells under the sea get the restoring density ρ_m − ρ_w.

### 3.4 Elastic thickness

From the plate model's own thermal state, on L1 each step:

| Crust | Law | Values |
|---|---|---|
| Oceanic | `Te = te_ocean_km_sqrt_myr × √age`, at most `te_ocean_max_km` | 2.7 km/√Myr, 40 km: 8.5 km at 10 Myr, 27 km at 100 Myr |
| Continental | `Te = te_craton_km × (1 − e^(−t/te_recovery_myr))`, `t` the time since the last orogeny there (never: a craton) | 80 km, 300 Myr: 23 km at 100 Myr, 51 at 300, 77 at 1,000 |
| Arcs and rifts (the `regime` field), and everything else at least | `te_min_km` | 5 km |

- Then smoothed over about 80 km (two box passes of two solve cells): heat spreads sideways, and
  the solver converges far better on a smooth field (measured, section 5).
- **A plate boundary is a break** (added while building). Cells along a boundary between two
  plates are at `te_min_km`. As built after section 13 the boundary cells are blurred and the
  share of stiffness kept is `1 − 2.5 × blur` (0 on the boundary rows, then 0.17 and 0.72); the
  first build had the next row half way, a staircase on the raster. Without it the first runs had the old,
  stiff subducting plate holding up the arc across the trench: arcs 10 km above their isostatic
  level, a 5 km hole on the other side (measured). A collision front between two plates is a
  break too, until the plates weld; then the basin forms through the continuous plate.
- All of it times the editor's `elastic_thickness_scale`.
- Sources (section 11): oceanic `Te` follows the depth of the 450 ± 150 °C isotherm of the cooling
  plate, 2.7 km × √Myr measured under seamounts; continental `Te` is 10–20 km at the low end and
  70–90 km in cratons, and is governed by the time since the last orogeny. Real continental `Te`
  also depends on whether crust and mantle are coupled and on how sharply the plate is bent;
  neither is modelled.

### 3.5 Which kilometres

Plate lengths are Earth-normalized: the model's plates, belt widths and subduction distances are
Earth's, mapped onto the smaller planet. Surface physics is in real metres at map scale
(decisions file, "Tectonics scaling"). On the quick preset one L1 cell is 4.8 real km and 39
Earth-normalized km.

**Flexure uses Earth-normalized lengths** (inferred, and a question for Ben, section 10). A
foreland basin is then as wide against its belt as on Earth (the 150–450 km of the acceptance
table are plate kilometres). In real metres a basin would be eight times wider against the same
belt: every belt a knife edge on a plate that cannot feel it.

Consequences, stated plainly:
- Heights and depths are not scaled (they never are).
- On the quick preset `α` is 0.5 to 4 cells. Basins and moats are a few cells wide: resolved, not
  finely.
- Features that the surface draws in real metres (a volcano stage cone) are, to the plate, eight
  times wider than they look, so they are compensated more locally than a real cone would be.

## 4. Where it sits in the step

After the plateau spreading (new step 5c), before the outputs. One call, `flexure::step`, and the
existing isostasy lines scaled by `1 − strength`. State is carried with the crust by the same
"same crust" interpolation as the elevation (`advect.rs`); crust born this step, or arriving from
another plate, starts unloaded and flat.

## 5. Numerical method

The problem: about 500,000 unknowns on the quick preset, a fourth-order operator whose stiffness
varies a thousandfold (`D ∝ Te³`), solved every 2 Myr step.

**Measured on a prototype** (Python, 1,024 × 512 cells of 39 km, a synthetic `Te` field of 8–70 km
with continents, cratons and ocean ages, 40 disc loads; iterations to a relative residual of
10⁻⁴ from zero, and from the last solution to within 2 m after a 300 m load change):

| Method | Cold | Warm | Notes |
|---|---|---|---|
| A. One spectral solve, uniform `Te` | exact | exact | No weak belts and no cratons: narrow belts on a plate of average stiffness stand 2–3 times too high |
| B. Spectral solves at several `Te`, blended by the local `Te` | 1 pass | 1 pass | Not a solution of the equation: errors up to 3.5–6 km of 21 km, compensation off by 3–6 % |
| C. Conjugate gradients, diagonal preconditioner | 213 | 92 | 2,400 cold and 380 warm at three times the stiffness |
| D. Conjugate gradients, one spectral preconditioner (mean `D`) | 94 | 43 | |
| **E. Conjugate gradients, preconditioned by B with three `Te` levels** | **10** | **4** | 30 and 13 at three times the stiffness; 76 cold when `Te` is not smoothed |
| F. Multigrid | not built | | The standard answer for this operator; coefficient jumps and a fourth-order operator make a robust one a project of its own |
| G. Convolution with the point-load response | not built | | Cost grows with the square of the kernel radius: about 4,000 cells per cell at the stiffest setting. Exact only for uniform `Te` |

**Choice: E.**
- The operator is the 5-point Laplacian applied twice with `D` between, on the raster in block
  metric (the surface's own metric: wrap in x, closed rows), so it is symmetric and conserves the
  load exactly.
- The preconditioner: split the residual by three smooth weights in `ln α`, solve each part
  exactly for its uniform `Te` with a Fourier transform in x and a cosine transform in y (`rustfft`,
  already a dependency), and recombine with the same weights. It is symmetric, so conjugate
  gradients converges for any `Te` field; where `Te` is uniform it is the exact inverse and one
  iteration suffices.
- As built, the blend sits between two damped Jacobi sweeps (a symmetric product). The sweeps take
  the cell-scale error the blend leaves along plate boundaries: 13 iterations instead of 27 on a
  run's own stiffness field (measured in the prototype).
- Stops when the preconditioned residual, an estimate of the error in metres, is under
  `tolerance_m` (2 m; a block is 20 m), or after `max_iterations` (60), reported.
- **Solve grid.** At most about 40 Earth-normalized km per cell is needed (`α` is never below
  18 km). Presets with finer L1 cells (the full presets: 19.5 km) solve on L1 coarsened by two
  and interpolate the deflection, so the solve is 1,024 × 512 for every built-in preset.

## 6. Cadence and cost

- **Every step.** The deflection carried from the last step is the starting point, so only the
  step's changes are solved for. A setting `every_steps` allows less often; loads between solves
  then wait, uncompensated, for the next solve.
- Why not relax slowly instead: the mantle answers in thousands of years, a step is 2 Myr. The
  equilibrium is the physical answer.
- **Estimate** (inferred from the prototype): one iteration is three forward and three inverse
  transforms of a 1,024 × 512 field plus the operator, about 15–25 ms on two threads; 4–8
  iterations a step; 60–200 ms against a step of 1.7 s (424 s for 250 steps on the quick preset,
  two threads): **4–12 % of run time**, the low end if warm starts work as in the prototype.
  To be measured; the promise was 5–10 %.

## 7. Ice

The climate stage publishes land ice each step (read): `climate/ice_thickness_m` (ice at the end
of the step), `ice_max_thickness_m`, `ice_time_share`.

- **The hook:** `flexure::step` takes an ice thickness per cell and adds `ρ_i × thickness` to the
  load. The surface stage passes `climate/ice_thickness_m` when the field exists and zero
  otherwise. Ice is not kept in the carried load: it is read fresh each step, so when it goes the
  ground comes back at the next solve.
- 3 km of ice sinks the ground about 830 m under a wide sheet (ρ_i/ρ_m), with a bulge round it.
- **What a 2 Myr step cannot show:** rebound takes thousands of years. Ice ages cycle inside a
  step; the load is the ice standing at the step's end, and the ground of the next step's erosion
  is the ground under that ice. The time-mean load (`ice_time_share` × thickness) is the
  alternative; the hook takes either.
- The ice branch's mass balance (`climate/ice_balance_m_yr`) is not needed: flexure wants the
  ice's weight, not its budget. If that branch changes the thickness field's name, one line moves.

## 8. What the belt work gets

Task 3 of the force-balance design planned `plates/foreland_m` from a line-load profile. Instead:

- It only has to raise the ground: the wedge on the incoming plate and the stack on the overriding
  one, as uplift. The surface stage turns uplift into load, and the basin and bulge follow from the
  incoming plate's `Te`.
- L1 fields, every step: `surface/flexure_m` (how far the ground stands above (+) or below (−)
  where local isostasy would put it: negative in forelands and moats, positive on bulges and on
  loads the plate holds up), `surface/deflection_m` (total sinking), `surface/elastic_thickness_km`.
  The structure stage's foreland (its table in 3.5 there) can be `surface/flexure_m` below about
  −100 m beside a collision regime.
- `foreland_elastic_km` of that design is not needed; its role is `te_craton_km` and the scale.

## 9. Settings and tests

| Setting (`surface.flexure.*`) | Default | Editor |
|---|---|---|
| `strength`: 0 = local isostasy as before, 1 = the solve, between: a blend of the two corrections | 0 in the struct, 1 in the pipeline presets | `flexure` (Advanced · Surface) |
| `elastic_thickness_scale` | 1 | `plate_stiffness`, 0.25–3 |
| `te_ocean_km_sqrt_myr`, `te_ocean_max_km`, `te_craton_km`, `te_recovery_myr`, `te_min_km` | 2.7, 40, 80, 300, 5 | preset only |
| `every_steps`, `tolerance_m`, `max_iterations` | 1, 2, 60 | preset only |

Tests (properties, not planets):
- A point load: depth under it against `P / (8 ρ_m g l²)`, `l = (D / ρ_m g)^¼`.
- A line load: depth, first zero crossing at `3πα/4`, bulge at `πα` with 4.3 % of the depth.
- The zero crossing moves as `Te^¾`.
- Compensation: the buoyancy of the deflection equals the load, uniform and varying `Te`.
- `strength` 0: no state, no fields, the step is today's.
- Stiffest and weakest editor settings: converges, finite, compensated.
- A stiffening plate does not move; a load added afterwards meets the stiff plate.
- Water: a load under the sea sinks by its Airy amount with `ρ_m − ρ_w`.

Measured on `earthlike_quick_16k`, 500 Myr: run time, hypsometry, a hotspot's moat, the basin in
front of the largest belt.

## 10. Honest assessment and questions

- **Hypsometry will move.** Erosion's rebound is no longer blurred over 50 km: in weak belts each
  valley rebounds by itself, on cratons it is spread wider than today. Belts whose edge lies on
  stiff crust stand higher at the edge and have a basin beside them. Not recalibrated here.
- **Today's collision uplift** is one step over up to 2,000 km. It makes a broad load, so the
  basin in front of it is shallow until the belt work lands.
- **Resolution.** Two to eight cells across a basin on the quick preset.
- **Volcano cones** sink into the plate (they are loads now). A cone's own height is unchanged;
  its summit elevation is lower by the sinking.

Questions for Ben, recommendation first:

1. **Which kilometres?** (a, recommended) Earth-normalized, as the plates: basins in proportion to
   belts. (b) Real metres: correct for the surface's real-size features, eight times too wide for
   everything the plate model builds.
2. **Ice load:** (a, recommended) the ice at the step's end, as the climate publishes it;
   (b) the time-mean over the step.
3. **Default.** (a, recommended) On in the built-in presets now, hypsometry recalibrated in the
   calibration task that follows the plate work. (b) Off until then.

## 11. Sources

Checked online on 2026-10-02 by a sub-agent against the publisher's record (Crossref) and the
abstract; "full text" where the paper itself was read. Figures quoted are from those.

- D. L. Turcotte & G. Schubert, *Geodynamics*, chapter 3: `D = E h³ / (12 (1 − ν²))`,
  `α = (4D / Δρ g)^¼`, the line-load profile `w₀ e^(−x/α) (cos + sin)` with `w₀ = V α³ / 8D`
  and its zero at `3πα/4` (checked in lecture notes on the chapter; the bulge at `πα` with
  `e^(−π)` = 4.3 % of the depth follows from the profile). `E` 70 GPa and `ν` 0.25 are the values
  of its worked example; other examples there use 100 GPa.
- J. F. Brotchie & R. Silvester 1969, "On crustal flexure", *J. Geophys. Res.* 74(22):5240–5252:
  the point-load response `w = (P l² / 2πD) kei(r/l)`, `l = (D / Δρ g)^¼`, so `P l² / 8D` under
  the load (as quoted in Wickert 2016, eq. 5–6).
- A. B. Watts 1978, "An analysis of isostasy in the world's oceans 1. Hawaiian-Emperor Seamount
  Chain", *J. Geophys. Res.* 83(B12):5989–6004: oceanic `Te` "corresponds closely to the
  450 ± 150 °C isotherm" at the time of loading; 20–30 km along the chain.
- S. Calmant, J. Francheteau & A. Cazenave 1990, "Elastic layer thickening with age of the oceanic
  lithosphere", *Geophys. J. Int.* 100(1):59–67 (full text): `Te (km) = (2.7 ± 0.15) √t (Ma)` from
  38 seamounts; olivine rheology predicts 3.3–4.3. The default is the measured 2.7 (the design
  first had 3.0; the runs of section 12 say which they used).
- A. B. Watts & U. S. ten Brink 1989, *J. Geophys. Res.* 94(B8):10473–10500: `Te` 40 km under
  Hawaii; P. Wessel 1993, *J. Geophys. Res.* 98(B7):12177–12190: 25–35 km. A sinking of about 5 km
  under Oahu is quoted from these by Bianco et al. 2005. The arch's distance and height were not
  confirmed in a primary source.
- E. B. Burov & M. Diament 1995, "The effective elastic thickness (Te) of continental
  lithosphere: What does it really mean?", *J. Geophys. Res.* 100(B3):3905–3927: 70–90 km for
  cratons, 10–20 km at the low end; thermal age is one of three controls (with crust-mantle
  decoupling and plate curvature), which this model does not have.
- M. Pérez-Gussinyé & A. B. Watts 2005, *Nature* 436:381–384: mean `Te` above 60 km for lithosphere
  older than 1.5 Gyr, under 30 km for younger.
- Z. Lu, P. Audet, C.-F. Li, J. Zhang & S. Li 2024, "Strength of continental lithosphere governed
  by the time since the last orogeny", *Earth Planet. Sci. Lett.* 643:118894 (abstract):
  continental `Te` 5 to about 140 km, mean 50 km, set by the time since the last orogeny. The
  recovery law here (80 km, 300 Myr) has that form; its two numbers are this note's choice, not
  the paper's.
- G. D. Karner & A. B. Watts 1983, *J. Geophys. Res.* 88(B12):10449–10477: `Te` 80–100 km for the
  Himalayan foreland, 25–50 km for the Alps. H. Lyon-Caen & P. Molnar 1983, *J. Geophys. Res.*
  88(B10):8171–8191: rigidity 0.7 × 10²⁵ N m under the Ganga basin (about 100 km), a hundred
  times less under the Greater Himalaya: stiff foreland, weak belt.
- P. G. DeCelles & K. A. Giles 1996, "Foreland basin systems", *Basin Research* 8(2):105–123 (full
  text): "Foredeep depozones are typically 100–300 km wide and 2–8 km thick"; forebulges 60–470 km
  wide and tens to a few hundred metres high (the flexural prediction, not a survey).
- K. Wieczerkowski, J. X. Mitrovica & D. Wolf 1999, *Geophys. J. Int.* 139(1):69–86: postglacial
  relaxation times of 3–5 kyr in Fennoscandia. A 2 Myr step is the equilibrium.
- A. D. Wickert 2016, "Open-source modular solutions for flexural isostasy: gFlex v1.0", *Geosci.
  Model Dev.* 9(3):997–1017. The usual open flexure code, and GPL-3: its source was not opened and
  nothing is ported. The solver here is written from the equation.

## 12. As built and measured (2026-10-02)

`earthlike_quick_16k`, 500 Myr, `deeptime stage volcanoes`, two threads, the test VM (x86_64);
`tools/review/p1-17-hyps.sh` with `surface.flexure.strength` 0 and 1. The commands are in the
build log entry.

**Run time** (the same seed with and without, side by side on the same machine load):

| Run | Without | With | |
|---|---|---|---|
| scattered, seed 1 | 547 s | 584 s | +6.7 % |
| supercontinent, seed 1 | 534 s | 579 s | +8.4 % |

- The flexure step itself is 85–107 s a run, 15–16 % of the run without it: over the 5–10 %
  promised. The run grows by less because the old isostasy (6 s) and most of the plateau
  spreading (30 → 4 s: far less crust stands above its strength height) fall away.
- 20–24 iterations a solve, not the prototype's 4–10: plate boundaries are sharp breaks in the
  stiffness. 0–1 solves of 250 stop at the iteration limit.
- One run each: a tolerance of 5 m saves one iteration (nothing); at most 12 iterations a solve
  cuts the step by a third and leaves 177 of 250 solves unfinished (the rest is solved in the
  following steps); a solve every second step halves it.
- What would cut it properly: single-precision transforms (about a quarter), or a real multigrid.

**Hypsometry** (seeds 1, 2 × both starts; mean [min, max]; not recalibrated). The final tree
(this branch merged with master 1aa0579, oceanic law 2.7 km/√Myr):

| | land | median, m | p90, m | above 2 km | above 3 km | sea median, m | shelf share of the sea |
|---|---|---|---|---|---|---|---|
| Earth | 0.289 | 427 | 2,022 | 0.102 | | 4,062 | |
| without | 0.299 [0.271, 0.340] | 578 [409, 724] | 1,927 [1,637, 2,377] | 0.097 [0.068, 0.142] | 0.035 [0.025, 0.057] | 3,408 [3,217, 3,925] | 0.115 [0.087, 0.133] |
| with | 0.287 [0.252, 0.315] | 676 [569, 753] | 2,219 [1,925, 2,392] | 0.130 [0.092, 0.149] | 0.030 [0.018, 0.042] | 3,843 [3,654, 4,175] | 0.061 [0.049, 0.066] |

- Two earlier ensembles of the same day, before the merge and with the oceanic law at 3.0: with
  flexure, land 0.266 and 0.297, median 774 and 638 m, p90 2,164 and 2,001 m, above 2 km 0.126 and
  0.102, sea 4,096 and 3,798 m (the second read its loads from the plate flags, which are empty on
  a fifth of the steps); without, land 0.274, median 578 m, p90 2,159 m, above 2 km 0.122, sea
  3,437 m. Four chaotic runs move a mean by about ±0.02 and ±100 m.
- So: the land share does not move outside the noise. The median rises by 60–200 m in every
  comparison. The p90 and the share above 2 km move either way.
- Clear in every comparison: the sea is 360–690 m deeper and the shelves about half as wide; the
  highest 1 % of land cells is at 3.1–3.8 km instead of 4.0–5.3 km; the plateau spreading moves
  5–10 thousand km³ a run instead of 160–400 thousand. The highest cell is 4.7–6.8 km (5.7–6.8 km
  without).
- 1,000 Myr, supercontinent, seed 1, one pair (law 3.0): land 0.225 → 0.305, median 502 → 774 m,
  p90 2,019 → 2,225 m, above 2 km 0.102 → 0.136, sea 2,998 → 3,751 m; run time 1,177 → 1,295 s
  (+10 %).
- Why, as far as it was looked at (not isolated by experiment): rebound from erosion is local in
  weak belts instead of spread over 400 plate km; volcano cones now sink; the shelf's sediment and
  the sea's own weight load the margins.

**What the pictures show** (`review/out/flexure/`, `tools/review/flexure-views.sh`):

- The ground stands more than 100 m off local isostasy on 14 % of the planet (half above, half
  below) and more than 500 m on 1.6 %; the extremes of one planet are −3.3 and +2.4 km.
- Nearly all of it lies along active margins and island arcs, as narrow paired bands a few cells
  wide, and round volcanic loads on old ocean floor: an island held up by 600 m with a ring up to
  480 m low round it.
- Belt interiors are at local isostasy (weak crust, as designed). In front of the largest
  collision belt the continent shows bands of about 100 m: today's one-step, 2,000 km wide uplift
  makes no foredeep of the 2–7 km in the acceptance table. That needs the belt work's wedge.
- Along margins the pattern alternates from cell to cell. Cell-to-cell roughness of the land rises:
  median 8.7 → 10.2 m, 99th percentile 294 → 391 m (one pair of runs).
- Stripes within 10° of the poles, where the raster's block metric stretches the plates' motion.
  They are outside the playable band (pole cap at 75°).

**Not verified.** That `strength` 0 is bit-identical to the build before this work (it is the same
code path, and a test holds it equal to a preset without the section; no run of the old binary was
compared). The full presets (the coarsened solve is unit-tested only). Other gravities and
`strength` between 0 and 1 on a planet. GameTests.

## 13. Artifacts after merging: causes and fixes (2026-10-02, follow-up)

Two artifacts shipped with the first build: the ground zigzagged from cell to cell along active
margins (land roughness, 99th percentile, 294 → 391 m in one pair of runs), and stripes ran across
the polar regions. A zigzag is measured as the share of cells where a cell and its next neighbour
(along x or y) are both strict extrema of opposite kind by more than 20 m (`tools/review/
flexure_artifacts.py`), over margin cells (within three cells of a plate boundary), plate
interiors, and the band poleward of 70°.

**Three causes, found in this order** (each with a planet measurement and a controlled test):

1. **The carried state acted as a force** (the main cause of both). Each step re-solved the whole
   load against the deflection and frozen moment carried from the last step. Carrying them with
   the crust on the raster interpolates them (bilinear, and near the pole a plate's rotation
   shifts each row by many cells); new crust and crust arriving from another plate restart from
   zero next to cells that carry kilometres. None of that is a load, but in a whole-load solve the
   mismatch is a residual, and the solve moved the ground to remove it. Measured on a 200 Myr
   planet with the response split linearly by source (a temporary diagnostic): **the carried
   state's residual alone moved the ground by up to 4.6–6.2 km in a step**, with a mean cell-scale
   energy of 10–25 m, about equally near restarted crust, near cells whose stiffness jumped more
   than fourfold in the step, and elsewhere (interpolation of the moment and deflection).
   Controlled test `carrying_a_bent_plate_moves_nothing` (a bent craton sheared row by row as
   rotation about the pole shears it, no new load): the old formulation moves the ground by
   1.15 m in a step; the new one by under 1 cm.
   **Fix:** solve only for the step's load increments. The carried plate is taken to be in
   equilibrium, as the real plate is; the solve's own unfinished residual is carried with the
   crust and added to the next step; the plate carries its bending moment, which stays when the
   plate stiffens and is released, as a force, when it weakens.
2. **The plate boundary was a staircase.** The break (boundary cells at the floor, the next row
   half way) was applied after the stiffness was smoothed, so it stepped from row to row with the
   raster. Controlled test `a_moving_margin_bends_smoothly` (an ocean plate moving 0.37 cells a
   step under a craton that advances 0.23 cells a step along a smoothly wavy boundary, a sediment
   fan and an arc): with the staircase the deflection zigzags beside the boundary (3–70 m); with
   the boundary cells blurred before the stiffness share is taken, nowhere by more than 10 m.
   On the planet the staircase was the polar problem: the polar band's elevation zigzag was 3.5 %
   [1.4, 5.0] with it and 1.3 % [0.9, 1.9] without it (flexure off: 1.5 %), over four planets.
3. **The plate model's sample lattice reached the ground as relief.** The plate model's uplift is
   interpolated onto L1 from samples about 71 Earth-normalized km apart; L1 cells are 39 km. A
   two-cell wave in it is beyond what the plate model resolves: aliasing. Measured: the uplift
   field itself zigzags in 4.8 % of margin cells by more than 5 m a step (2.5 % by more than
   20 m). Local isostasy passed it to the ground at its own height; a stiff plate passed the whole
   thickening behind it, 5.5 times higher. Controlled test
   `the_plate_models_sample_lattice_is_not_a_load` (uplift alternating column by column on a
   craton): the ground steps by 500 m between columns for a 500 m uplift, against more than
   1,500 m unfiltered. **Fix:** only the part of the plate model's root the plate model resolves
   bends the plate (binomial passes, `(1, 2, 1)/4`, set by the sample spacing: two on the quick
   presets, zero response to the two-cell wave); the rest stays locally compensated, as the plate
   model made it. Erosion, deposition and spreading are computed on L1 and are not filtered.

**What remains** (the final code's per-step response split by source, margin cells, 200 Myr): the
step's erosion and deposition (marine fill of kilometres in trench and margin cells) 1.6–4.1 % of
margin cells zigzag by more than 5 m, plate uplift with water and ice 1.3–3.6 %, the moment
released where a moving boundary weakens the plate 1.0–3.1 %, the carried unfinished residual
under 0.1 %. Deposition is computed at this resolution and is a real narrow load; the released
moment follows the plate boundaries' own cell-scale steps from the plate model.

**Ruled out:** the spectral-blend preconditioner leaving a high-frequency residual (the carried
unfinished residual's response zigzags in under 0.1 % of cells); latitude-dependent terms in the
operator or the preconditioner (there are none: the raster is in block metric, the same metric as
every surface process, and a ring load bends the plate the same at every longitude, tested).
Regime: unlike `plates/flags`, the regime field is populated on the last step of a run, which is
a resampling step (checked on a checkpoint); along margins its code changes in a cell-scale
zigzag in about 1 % of cells. Its flicker from step to step was not measured on its own; a
stiffness that drops shows in the released-moment share above.

**Measured on the merged tree** (this branch merged with master 412d517; `earthlike_quick_16k`,
test VM, 2 threads).

Artifacts, 200 Myr, seeds 1–2 × both starts, mean [min, max] (`S=… LEN=200 tools/review/
flexure-ensemble.sh off=…:surface.flexure.strength=0 fin=…`):

| | elevation zigzag at margins | interior | polar band | land roughness p99 | sea roughness p99 |
|---|---|---|---|---|---|
| flexure off | 11.1 % [8.5, 12.9] | 1.15 % | 1.48 % [1.16, 1.77] | 237 m [177, 332] | 1,507 m |
| first build (one planet; off there: 10.8 %, 1.50 %) | 19.3 % | 2.2 % | 1.57 % | 308 m | 2,109 m |
| fixes 1 only (incremental), same ensemble | 12.8 % [10.4, 13.7] | 1.6 % | 3.45 % [1.43, 5.04] | 250 m | 1,796 m |
| final (fixes 1–3) | 13.1 % [10.4, 15.0] | 1.54 % | 0.87 % [0.49, 1.27] | 274 m [123, 338] | 1,636 m |

**The roughness, explained** (the 500 Myr pair below, the planet of the first complaint, 294 →
391 m then): land roughness p99 289 m without, 372 m with. Away from coasts flexure makes the land
smoother: land whose four neighbours are land has p99 271 m without and 225 m with (at margins
563 → 451 m). The rise is all coast: coast cells within three cells of a plate boundary double
(1,378 → 2,830), because arcs held up by the stiff plate's edge emerge as islands, and an island
cell beside deep water reads as rough. 85 % of the roughest 1 % of land cells lie at margins with
flexure (19 % without); a tenth of them zigzag. That is a change in where land is, not noise.

**Run time**, the same planet with and without, side by side (500 Myr, supercontinent, seed 1):
557 → 576 s, **+3.3 %**. The flexure step is 56 s (10 % of the run without it; 85–107 s before
the fixes), 9.3 iterations a solve (20–24 before), none at the limit; the old isostasy (6 s) and
most of the spreading solve (24 → 4 s) fall away.

**Hypsometry**, 500 Myr, seeds 1–2 × both starts (`tools/review/p1-17-hyps.sh`), not recalibrated:

| | land | median, m | p90, m | above 2 km | sea median, m | shelf share of sea | highest 1 % of land | highest cell |
|---|---|---|---|---|---|---|---|---|
| Earth | 0.289 | 427 | 2,022 | 0.102 | 4,062 | | | |
| without | 0.299 [0.271, 0.340] | 578 [409, 724] | 1,927 | 0.097 | 3,408 [3,217, 3,925] | 0.115 | 4.0–4.8 km | 6.8 km |
| with | 0.290 [0.252, 0.331] | 636 [598, 712] | 1,997 | 0.102 | 4,053 [3,755, 4,718] | 0.059 | 3.0–3.4 km | 5.0–5.3 km |

**Not explained:** a few cells per planet stand far below their local-isostatic level (−11.8 km
at one polar trench in the final build; the first build had −6.2 and −8.8 km on its planets). The
share of the planet more than 500 m below is 0.55 % (first build 0.71 %).


## 14. The spreading's root, and the divergence on large planets (2026-10-18)

Section 13 left the spreading's root bending the plate unfiltered ("Erosion, deposition and
spreading are computed on L1 and are not filtered"). That was a fourth cause, and the one that
diverged. The spreading (`spread.rs`) is an implicit diffusion of the surface, solved before the
flexure. The flexure took its change as crust with its Airy root already in the elevation, and a
stiff plate cannot sink at the cell scale, so it lifted the change to the whole thickening behind
it: up to 1/(1 − ρ_c/ρ_m) = 7 times. For a wave of the surface the two steps multiply its height by
G = 1 − F·λ/(1 + λ), F up to 7 and λ = D·Δt·k²: |G| > 1 once λ is above about 0.4.

The spreading uses the map's metres (`dx` = 32 blocks × `m_per_block_h`), the flexure the plate
model's Earth-normalized ones (section 3.5). At 1:100 the spreading is strong enough at the cell
scale to hold every plateau at the strength height, where D is nearly nil. On the Earth-sized
planet (1,222.5 m per block, 39 km cells) it is 144 times weaker per cell, cells that the plate
model lifts stand far above the strength height (D up to 8 D₀), λ for a checkerboard reaches about
2.5, and the split grew it. Measured on `earthlike_quick_32k` at 1,222.5 m per block (scattered
start, 100 Myr, a trace of every phase of the step): the start of the trouble is a plate-model
spike (one cell to 9.2 km, 4.6 mm/yr) 40 Myr in; 30 to 50 Myr in, a craton (Te 80 km, no uplift)
holds a checkerboard in which the spreading lowers a peak by 8.7 km and raises its neighbour by 12.8 km,
the flexure turns these into −47 and +56 km, and stream power and marine fill then work on peaks
of 60 km and pits of −45 km. The solve converges (12 to 14 iterations); it is the coupling that is
unstable. With the spreading off the same planet stays near 14 km; with the flexure off, 6.6 km.

**Fix.** The spreading's root stays locally compensated (ISO-070): its load and root still go into
the plate's load (they cancel in the residual) and the gap budget, but not into the root that
bends the plate. The spreading's law is the floating answer of the lower crust's flow; the split
is then stable for any cell size. Unit test
`the_spreading_on_a_stiff_plate_does_not_grow_a_checkerboard` (39 km cells, a plateau at three
times the strength height with a ±300 m checkerboard: it now flattens; before, the plateau's
range went from ±300 m to ±1.8 km in the first step); unit test
`a_planet_of_large_cells_keeps_its_surface_physical` (the small coupled planet, seed 11, at
1,221 m per block, 100 Myr: 29.9 km before, 4.0 km after, asserted within twice Earth's relief of
the sea); and the guard (`guard.rs`, ISO-072) stops any run whose surface leaves twelve Earth
reliefs from the sea. With the old coupling and the guard the Earth-sized planet's 500 Myr bake
stops after 92 s (−388 Myr, a cell at 263.8 km), where it had run 3.56 h to 3.3 × 10²⁹ m.

**What is left.** The plate model now and then lifts a patch by tens of km in one step (up to
16.5 mm/yr, 33 km a step over hundreds of cells, against its 30 km ceiling); the flexure holds
part of the root behind it up (ISO-069) for a step or two, and erosion takes it down. That is
scale-independent (the plate model and the flexure are Earth-normalized) and is there with and
without the fix: the highest cell after the flexure in four traced 500 Myr runs of
`earthlike_quick_32k` was 112 km (1:300) and 66 km (1:100) before the fix, 69 km and 92 km after.
A guard at five Earth reliefs (98.9 km) stopped a shipped bake on such a transient
(`earthlike_quick_16k`, scattered, seed 2, −272 Myr, 105.8 km), hence twelve. Which plate process
makes the spikes was not traced here; it was on 2026-10-18 (a resampling's accretion and the
collision belts), and the plate step's rate limits now bound them (docs/science/plates.md, "The
fastest a column thickens and rises"; build log 2026-10-18 "Plate spikes").

## 15. The polar holes: a weakened plate lets go of its load, not of its moment (2026-10-18)

With the plate step's rate limits in, the guard's worst cells were on L1's polar rows. Over six
500 Myr runs of `earthlike_quick_16k`, single cells there rose or fell by tens of km within a few
steps under plate uplift of 2 km or less; the worst was a hole two cells wide that fell to 59.6 km
below the sea (supercontinent, seed 2). A per-process trace of the surface stage (a temporary
diagnostic, not kept) put the polar picks on the flexure, and within it on the released moment
(section 3.3 as built after section 13). At −290 Myr a craton cell at 70° that a plate boundary
reached (Te from 80 km to the 5 km floor) released 17.2 km of ground in one step.

**Cause.** The released force was `L((1 − r)·M)`, with `r = D_now / D_prev` and `M = D ∇²w`. For a
plate in balance `L(M) = ρ_m (Q − w)` (in the solve's units, with ice and the unsolved part), so
where `r` is uniform the release is `(1 − r)` of the load the plate holds from local isostasy.
Along a boundary `r` jumps from cell to cell: a craton cell keeps (5/80)³ of its rigidity, its
neighbours all of theirs. The Laplacian of a moment that jumps at the cell scale is several times
the load held, with opposite signs in neighbouring cells. The polar rows have more of it, for
three reasons:

- poleward of 60° a row is taller on the planet than the plate model's sample spacing (about
  71 km), so its boundaries step from row to row;
- a rotating plate carries crust about ten cells a step east to west there, so boundaries sweep
  many cells each step;
- the rigidity was carried by a linear blend, which reads a weak cell beside a stiff one as
  nearly as stiff, so crust carried far seemed to weaken where nothing changed.

**Fix.** The release is the force `(1 − r) ρ_m (Q + ice − w − unsolved)` at each cell where `D`
fell (ISO-056). Where `r` is uniform it equals the old force. Where `r` is not uniform, it is
bounded by the cell's own departure from local isostasy. The ground then moves towards the Airy
answer, the limit a viscoelastic plate relaxes to (Eshagh & Tenzer 2021; Buoninfante et al.
2023), and never past it. The moment is no longer carried (`FlexState::moment` removed). The
rigidity is carried as its logarithm (ISO-075). `StepOut::released_m` now reports the load let go.

Three candidates were tried over the first 110 Myr of two runs (scattered seed 1, supercontinent
seed 2), with the trace's bands at 45, 60, 70 and 80°: the departure rule; the departure rule
with the logarithmic carry; and no release at all (a bound, not a candidate). Below 45°, the
largest per-step total change of the surface stage was:

| Candidate | Scattered seed 1, up and down (km) | Supercontinent seed 2, up and down (km) |
|---|---|---|
| Departure rule | 4.7 and 4.6 | 5.2 and 4.5 |
| With the logarithmic carry | 4.5 and 4.4 | 4.8 and 4.6 |
| No release | 3.7 and 4.4 | 4.8 and 4.6 |

The polar bands came down to the same size. The departure rule with the logarithmic carry was
kept.

**Test.** `a_plate_that_weakens_settles_towards_local_isostasy` bends a 64 × 48 plate with a disc
load. Then, for ten steps, a plate boundary steps between rows across it (`10 + 3k + 2·((i/2) mod 2)`).
The test asserts three things:

- no step moves the ground by more than 1.05 times the most the plate held (the largest
  |flexure| of the step before);
- nothing goes more than a quarter of that past local isostasy;
- the boundary lets something go.

The old rule fails it at the third step: 11.2 km against 8.0 km, and 4.2 km past against 1.9 km.

`a_moving_margin_bends_smoothly` now leaves out of its deflection check the cells where the load
itself zigzags by more than 10 m. With the logarithmic carry that is 17 m at the trough of the
margin's wave. The deflection may follow its load there, and the flexure field (load less
sinking) stays checked everywhere.

**Measured.** Six runs of `earthlike_quick_16k`, 500 Myr, seeds 1 to 3, both starts. Mean
[min, max]. Before: master 9091c163 with the step records. After: this branch merged with master
19cb11e0.

| | Before | After |
|---|---|---|
| Guard's worst (m) | 30,814 [19,599, 59,631] | 15,471 [13,187, 18,101] |
| Steps beyond 1, 2, 3 and 6 Earth reliefs | 25, 8, 1, 0 | none |
| Largest rise and fall in a step, polar rows (m) | 16,148 and 12,543 | 5,016 and 5,535 |
| Largest rise and fall in a step, the rest (m) | 15,012 and 15,652 | 6,593 and 5,628 |
| L1's peak (m) | 7,288 | 7,031 |
| Land share | 0.199 | 0.207 |

Away from the poles the planets diverge cell by cell (mean |Δh| 2.6 km, and as much in the plate
model's own elevation) but keep their hypsometry:

| Away from the poles | Before | After |
|---|---|---|
| Land share | 0.201 | 0.211 |
| Land's median (m) | 726 | 715 |
| Land's 99th percentile (m) | 5,053 | 5,065 |

The full table, and the comparison with Earth's fastest measured rates (ISO-076), are in
docs/science/flexure.md, "The polar holes".

**What is left.** Single cells still change by 5 to 10 km in a step. In the 110 Myr traces this
came from the fluvial step, marine fill, the glacial step and the flexure. That is within
Earth's fastest measured rates, but above what a region sustains over 2 Myr. The release happens
at once within the step; relaxation takes about 1 Myr (Beaumont 1978). The share `1 − r` is this
design's own, from the two limits.
