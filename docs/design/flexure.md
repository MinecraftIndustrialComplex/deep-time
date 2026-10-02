# Lithospheric flexure in the surface stage: design

Status: **built** (2026-10-02; sections 1–11 are the design as written before the code, corrected
where the build changed it; section 12 is what was measured). Ben's decision of 2026-10-02 (decisions
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
- The deflection is solved for the whole load every step, not accumulated from increments, so
  nothing drifts: when a belt wears down or ice melts, the load is gone from `Q` and the plate
  comes back.
- The plate model learns the result through the existing feedback (the surface's elevation over
  its own).

**A plate that stiffens keeps its shape.** A belt built on hot, weak crust has a local root. When
that crust cools and stiffens, nothing should move: the new stiffness is added stress-free at the
shape the plate has. Without this, every old belt would slowly rise as its root is "spread".
The model: per cell, the rigidity `D` and a frozen bending moment `M₀` are kept with the crust.
The bending moment is `D ∇²w − M₀`. When `D` grows, the step is solved with the old `D` and then
`M₀ += ΔD · ∇²w`; when it falls (reheating), `M₀` shrinks in proportion. Loads applied after the
stiffening meet the stiff plate. This is the observed rule that a load records the plate's
strength at the time of loading (Watts 1978: seamounts built on young floor keep a low `Te`).

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
  plates are at `te_min_km`, the next row half way to it. Without it the first runs had the old,
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

