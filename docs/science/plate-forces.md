# Plate forces: the force balance that moves the plates

Status of this page: `stage 2 (constants checked against sources on 2026-10-02)` for the sources
the project's library holds; the rest is marked `from-memory` in the ledger.

Ledger ids on this page: `FRC-001` to `FRC-030`, `PLT-181` to `PLT-191`, `PLT-022`, `PLT-037`,
`PLT-270` to `PLT-273` and `PLT-275` (see [claims.md](claims.md)).

## What you see in the world

Since the switch (task 7 of `docs/design/plate-force-balance.md`, 2026-10-04) the plates move
because of the forces on them. Every 2-Myr step the plate model works out the forces on each plate
and the motion at which they balance, and each plate's motion turns towards that answer over
`motion_relax_myr` (6 Myr). There is no speed rule any more: no speed handed out at the start, at a
rift or at a new subduction zone, no base speed for a plate nothing pulls, no slower speed for
continents, no random walk and no speed cap that the model leans on.

What a player should notice is in the speeds and in what they do to the map: plates attached to
sinking slabs move several times faster than continents with no slab; a planet starts at rest and
gets going as its first subduction zones form; a rift opens only where something pulls its sides
apart, and heals where nothing does; and a continent that reaches a trench slows, the slab that
pulled it breaks off and the growing mountain belt holds it, until the two plates stop and weld.

## The model in plain words

The plates stage moves the plates, finds where they overlap and which one sinks (a subduction
front), and then sums the forces:

1. **Slabs.** Every subduction zone keeps the length of slab hanging from it, known by its two
   plates. The slab grows by the convergence at the oceanic front each step, up to `slab_depth_km`
   (700 km). When continental crust has filled at least half of the front for `slab_breakoff_myr`
   (10 Myr), the slab breaks off. A slab that no oceanic front feeds for a step keeps hanging and
   pulling along the two plates' shared boundary, and is gone after `slab_breakoff_myr` without a
   front; so a contact that flickers off for a step or two keeps its slab. When old ocean floor
   breaks away to start a new subduction zone, it starts with a slab of `initiation_slab_km`
   (150 km): that slab pulls the new plate towards its parent and opens the trench.
2. **Driving forces.** Along each subduction front where the sinking crust is oceanic, slab pull
   draws the sinking plate towards the trench, in proportion to the slab's length and to the square
   root of the crust's age; crust younger than 10 Myr does not pull. The overriding plate is drawn
   towards the trench too (slab suction, a share of the pull). At every plate edge, high ground
   pushes the plate away from itself and low ground draws it: a mid-ocean ridge pushes ("ridge
   push"), the edge of a high plateau pushes back. A weak, slow flow of the mantle drags every plate
   a little.
3. **Resisting forces.** The mantle drags on the base of each plate, more under the deep roots of
   continents. A sinking slab resists sinking faster. Plates rub against their neighbours along
   every boundary. Where a continent has reached a trench (a collision), two forces resist further
   convergence, each whatever the speed, so they can stop two plates but never push them apart:
   the weight of the mountain belt (set by its height), and the buoyancy of the continental
   lithosphere the collision has already dragged down the trench, which grows as the collision
   shortens (continental lithosphere is lighter than the mantle, Cloos 1993; `collision_buoyancy`).
4. **The balance.** A plate has no inertia, so all torques on it add up to zero. Every resisting
   force grows in proportion to the plates' speeds (the collision's is linearised at the step's
   convergence), so the balance of all plates together is one linear system, solved exactly each
   step.
5. **The motion.** Each plate's rotation moves the share 1 − e^(−δt/τ) of the way towards the
   balance's answer each step, τ = `motion_relax_myr`: a stand-in for the time slabs and mantle
   take to respond, and the smoothing of the step-to-step noise of a balance found on a lattice. A
   speed guard (`max_plate_speed_mm_yr`, 250 mm/yr) caps any plate that would run away and is
   counted every time it does; it is meant never to bind.
6. **At the start, rifts and welds.** Plates start at rest (the starting ocean floor's ages come
   from an earlier motion drawn for the purpose and not kept). Rift fragments keep their parent's
   motion; the balance decides whether they part. Two plates a rift separated that keep moving
   together (relative speed under `weld_speed_mm_yr` along their whole shared boundary for
   `weld_after_myr`) become one again. Two plates whose collision has stalled weld as before
   ([plates.md](plates.md)).
7. **The record.** With the dynamics record on (the default), each plate's torques, mean in-plane
   stress and balance answer, and each subducting contact's slab length, go into the run's record,
   which `deeptime plate-check` reads.

The stage reads the plates' geometry, crust type, crust age and elevation; what it changes is the
plates' motion.

## Equations and constants

Forces are per metre of boundary (N/m) or per square metre of plate (Pa), on the plate model's
Earth-sized sphere of radius R = 6,370 km. A force f at the unit position x has the torque R x × f.

**The balance.** With Ω_p each plate's angular velocity,

    Σ_terms T_p(Ω) = 0  for every plate p,   i.e.   M Ω = T,

T the torques that do not depend on speed (pull, suction, edge energy, the mantle flow's drag) and
M the resistances (basal drag, slab, boundary and collision resistance), a symmetric positive
definite matrix of 3 rows per plate, solved by Cholesky factorisation.

**Slab pull** on the sinking plate, per metre of oceanic front, towards the trench:

    F_sp · slab_pull · s(a) · min(L, D) / D,     s(a) = √(min(a, 180 Myr) / 80 Myr), 0 for a < 10 Myr

F_sp the reference pull, a the crust's age at the trench, L the slab's length, D `slab_depth_km`.

**Edge energy** at a plate's boundary sample, per metre, pushing away from the edge:

    (U(edge) − Ū) · edge_energy / 0.1

with U the gravitational potential energy of the column (Ū the plate's mean):

    sea floor:         U = −C d²,  d its depth below the ridge crest (no deeper than the old floor)
                       C = g (ρ_m − ρ_w)² π / (4 ρ_m α ΔT) · (1 + 2 ρ_m α ΔT / (π (ρ_m − ρ_w)))
    continental crust: U = ρ g h (H + h (1 + ρ / (ρ_m − ρ_c)) / 2),  h its height above sea level

**Collision resistance** per metre of collision front, against the convergence (plastic: put into
the linear solve as a drag of this force over max(convergence, 1 mm/yr)):

    (U(belt) − U(incoming))⁺ · edge_energy / 0.1  +  collision_buoyancy · 2.25 · F_sp · (T / 30 km) · min(S, D) / D

T the incoming continental crust's thickness, S the collision's shortening so far, D
`slab_depth_km`: the second term is the up-dip push of S km of continental lithosphere dragged
down the trench, 0.09 / 0.04 = 2.25 times the down-dip pull of the same length of 80-Myr slab
(Cloos 1993's density contrasts).

**Basal drag** per square metre: c_b (v − v_flow), times `keel_drag` under continental crust, with
c_b fixed by the speed scale: (1 + slab_resistance + boundary_resistance) c_b A v_s = F_sp L_ref.

**Mean in-plane stress** of a plate: σ = (Σ r·f + ∫ r·b dA) / (2 A h), r a point's offset from the
plate's centroid, f the edge forces at the balance's answer, b the basal drag, h 100 km.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| PLT-181 | Speed scale: the reference plate's speed | 45 | mm/yr | Earth's area-weighted RMS about 47 mm/yr today (WP-13a) | see the ledger |
| FRC-002, PLT-183 | Reference slab pull F_sp (at Earth's gravity) | 3.0 × 10¹³ | N/m | design's gross figure; Faccenna et al. 2012 give 1.4 × 10¹³ for 80-Myr floor (WP-13a) | `from-memory` |
| PLT-182 | Slab pull strength `slab_pull` | 1 | relative | Faccenna et al. 2012 eq. 1, slab length × √age (WP-13a, read); Forsyth & Uyeda 1975 (read: slab forces dominate) | `verified` (the law; the 10 Myr cut-off unsourced) |
| FRC-004 | Slab thickness factor s(a) | √(min(a, 180)/80) | — | √age: Faccenna et al. 2012 (WP-13a, read); the 80 and 180 Myr: the design's trace | `from-memory` |
| FRC-005 | No pull below this crust age | 10 | Myr | Cloos 1993 (not read) | `from-memory` |
| PLT-184 | Slab depth, break-off time | 700, 10 | km, Myr | van Hunen & Allen 2011 (abstract) | `abstract-only` |
| PLT-185 | Slab resistance | 3 × the reference plate's drag per 3,000 km | — | Forsyth & Uyeda 1975 (read: the terminal velocity) | `calibrated` (by the design's open-loop test) |
| PLT-186 | Slab suction | 0.3 of the pull | share | Conrad & Lithgow-Bertelloni 2002, 2004 (abstracts) | `abstract-only` |
| PLT-187, FRC-009 | Edge energy | 0.1 (the energy as computed) | — | Lithgow-Bertelloni & Richards 1998 (read: <10 % of the driving force) | `verified` (the share) |
| FRC-006 | ρ_c, ρ_m, ρ_w | 2750, 3300, 1000 | kg/m³ | typical values | `from-memory` |
| FRC-007 | Reference crust H | 35 | km | design | `from-memory` |
| FRC-008 | α, ΔT | 3 × 10⁻⁵, 1300 | 1/K, K | Turcotte & Schubert (not read) | `from-memory` |
| FRC-012 | C at Earth's gravity | 3.3 × 10⁵ | N/m³ | Turcotte & Schubert's ridge push (not read) | `from-memory` |
| FRC-013 | Ridge crest = continent at sea level | U = 0 for both | — | Coblentz et al. 1994 (not read) | `from-memory` |
| FRC-014 | Land column energy | 6.7 × 10¹² N/m for a 5 km belt on 35 km crust | N/m | Molnar & Lyon-Caen 1988 (not in the library) | `from-memory` |
| PLT-188 | Keel drag | 3 | × | Conrad & Lithgow-Bertelloni 2006 (read: 2-5, about 3 on average) | `verified` |
| PLT-189 | Boundary resistance | 3 × the reference plate's drag per its perimeter | — | Forsyth & Uyeda 1975 (read: transform resistance) | `calibrated` (by the design's open-loop test) |
| PLT-190, FRC-015 | Mantle flow: degree-2, from two antipodal upwellings | 5 | mm/yr | Ben, decisions log 2026-10-02 | `game-choice` |
| FRC-016 | Basal drag coefficient c_b (defaults) | 2.5 × 10¹⁴ | Pa s/m | defined by the speed scale | `game-choice` |
| FRC-010 | Lithosphere thickness for the stress | 100 | km | Conrad & Lithgow-Bertelloni 2006's reference (read) | `verified` |
| FRC-011 | Collision drag's speed floor | 1 | mm/yr | under the weld speed (5), so a held collision can weld | `game-choice` |
| FRC-028 | Continental lithosphere's buoyancy over 80-Myr slab's weight | 0.09 / 0.04 = 2.25 | ratio | Cloos 1993 (abstract, read) | `abstract-only` |
| FRC-029 | Crust thickness of that figure (buoyancy in proportion to thickness) | 30 | km | Cloos 1993 (abstract, read); the proportion is a game choice | `abstract-only` |
| FRC-030, PLT-275 | Collision buoyancy law and strength `collision_buoyancy` | 1 | relative | Cloos 1993 (abstract): continental lithosphere "resists subduction" | `game-choice` (the law's form) |
| PLT-191 | Relaxation time (used from the switch on) | 6 | Myr | the design | `game-choice` |

Two checks of the formulas against the design's figures (unit test
`the_column_energy_gives_ridge_push_and_plateau_weight_of_earths_size`): a ridge crest 3.5 km above
100-Myr floor pushes with 4.0 × 10¹² N/m (the design quotes about 3 × 10¹², Parsons & Richter 1980;
Forsyth & Uyeda 1975 quote McKenzie's "of the order of 300 bars" over 80 km, 2.4 × 10¹²); a 5 km belt
on 35 km crust stands at 6.7 × 10¹² N/m (Tibet is quoted at 6-7 × 10¹²). With the defaults the basal
drag coefficient corresponds to an asthenosphere of 2.5 × 10¹⁹ Pa s over 100 km and a basal shear
stress of 0.35 MPa at 45 mm/yr; Lithgow-Bertelloni & Richards 1998 put a weak asthenosphere "of the
order of 10^19 Pa s", and Forsyth & Uyeda 1975 found "drag at 8 cm/yr roughly 5 bars" (0.5 MPa).

## Earth against the mod

The balance as it would move the plates of today's model (open loop: the plates kept moving by the
old rules), `earthlike_quick_16k`, supercontinent start, 1000 Myr, seeds 1-4, 2026-10-02, master
2c60ca3f plus this feature, `tools/review/plate-check.sh` (mean of the four runs; full table in
`docs/design/plate-check-baseline.md`, "Added 2026-10-02: the force balance").

| Quantity | Earth | Source of the Earth figure | The balance | Today's motion (same runs) |
|---|---|---|---|---|
| Plate speed, area-weighted RMS | about 47 mm/yr today; 50-100 over the last 200 Myr | Forsyth & Uyeda 1975 Table 1 (computed) and Zahirovic et al. 2015 §3.2, read (WP-13a) | 37.1 mm/yr | 59.2 |
| Plate speeds, 10th / 50th / 90th percentile | 11 / 33 / 80 mm/yr (today's 12 plates) | Forsyth & Uyeda 1975 Table 1, computed (WP-13a) | 11 / 49 / 93 mm/yr | 41 / 72 / 86 |
| Oceanic over continental plates, median speed | 2-3; 81 against 28 mm/yr for under 25 % against over 50 % continental | Zahirovic et al. 2015, read | 3.7 | 1.6 |
| Slab-attached over slab-free plates | 6-9 cm/yr against 0-4 cm/yr; 79 against 26 mm/yr (more than 25 % of the perimeter subducting against none) | Forsyth & Uyeda 1975, read; Zahirovic et al. 2015, read | 3.1 | 1.5 |
| Speed against slab share, rank correlation | 0.71 (12 plates) | Forsyth & Uyeda 1975 Table 1, computed (WP-13a) | 0.43 | 0.41 |
| Share of the driving force from slabs | ">90%" | Lithgow-Bertelloni & Richards 1998, read | slab pull 0.86 of the driving torque (median, by size) | — |
| Interior stress of a supercontinent | 20-50 MPa extension (a model, homogeneous supercontinent) | Huang et al. 2019, abstract | median -0.3 MPa; in tension in 46 % of the steps a plate holds over 40 % of the planet | — |

What it says: the balance gives Earth's spread of speeds and its contrasts (oceanic against
continental, slab against no slab) that today's rules do not, with nothing put in for them; its
RMS is a little under Earth's (the speed scale is calibrated later). It is noisy: a plate's balance
rotation changes by a median 25 % from one step to the next (90th percentile 84 %), against 5 % and
9 % today. A variant with slabs pulling in full from the start halves that noise and lifts the slab
correlation from 0.37 to 0.57 (300 Myr, seed 2): slabs restart from zero whenever their contact is
new, reversed or reopened, and today's contacts flicker. Without boundary resistance small plates
run away (1 % of plate-steps above 250 mm/yr). The mega-plate is close to neutral, not in the strong
tension the supercontinent models find. The balance costs 0-2 % of the plates stage.

**Closed loop, since the switch** (the balance moves the plates; `earthlike_quick_16k`, 1000 Myr,
seeds 1-4, both starts, 2026-10-05; master 2e07d954 on the old speed law against the branch;
`tools/review/force-balance-ensemble.sh` and `plate-check`):

| Quantity | Earth | The balance (closed loop) | Old speed law (master) |
|---|---|---|---|
| Plate speed, area-weighted RMS | about 47 mm/yr today; 50-100 over 200 Myr | 41.4 mm/yr | 57.9 |
| Plate speeds, 10th / 50th / 90th percentile | 11 / 33 / 80 mm/yr | 13 / 41 / 78 | 36 / 66 / 84 |
| Oceanic over continental plates, median speed | 2-3 | 2.37 | 1.61 |
| Slab-attached over slab-free plates | about 3-4 | 1.87 | 1.69 |
| Speed against slab share, rank correlation | 0.71 | 0.45 | 0.54 |
| Convergence 20 Myr after a collision's onset over at onset | India about 0.27 in 15 Myr (Copley et al. 2010) | 0.46 | 0.07 |
| Plate-steps where the speed guard (250 mm/yr) binds | — | 0 | 1,015 (the old cap, 100 mm/yr) |
| A plate's rotation change per step, median / 90th percentile | — | 5.8 / 25 % | 5.1 / 9.6 % |

The spread of speeds and the oceanic-continental contrast come out of the forces; the slab
correlation is weaker than Earth's and weaker than the open-loop test's, and the 90th percentile of
the step-to-step change is still above the design's 20 %. See `docs/design/plate-force-balance.md`
§13, "Tasks 7 and 8", for the rest.

## Simplifications and limits

- The open-loop table above computed the forces on plates that the old rules moved; the closed-loop
  table is the switch.
- One slab length per contact; along-trench tears are not represented. A slab's age is that of the
  crust at the trench now, not the history of what went down.
- Rigid plates: the balance cannot break a plate. The stress is an index of the edge forces (flat
  plate, 100 km thick), not a stress field; for plates covering a large part of the sphere it is
  only its sign and rough size.
- Boundaries are found per sample (about 71 km on the quick planet), so torques jump as samples
  cross: the main source of the step-to-step noise.
- The sea floor's potential energy is the half-space cooling form, the land's an Airy column; both
  are from the textbook, not checked here. Trenches are given the old floor's energy.
- The mantle flow is a fixed degree-2 pattern, a stand-in, not convection.

## Game choices

- The speed scale is defined by a reference plate (FRC-016), so the balance has one fixed scale
  instead of being rescaled each step (agent, build log 2026-10-02).
- The mantle flow's shape: two antipodal upwellings, fixed for the run, axis from the seed (FRC-015;
  the term itself is Ben's choice, decisions log 2026-10-02).
- A slab's trench counts as blocked when continental crust holds half its front (FRC-019).
- The collision drag's floor of 1 mm/yr (FRC-011), under the weld speed so that held collisions weld.

## Open questions

- Slab pull's size (3 × 10¹³ N/m) and the √age law are textbook values from memory; the library
  has neither Turcotte & Schubert nor Schellart 2004.
- Forsyth & Uyeda's own fit has suction at about 0.08 of slab pull per length (Table 6); the code
  uses the design's 0.3 (Conrad & Lithgow-Bertelloni, from abstracts).
- Slab tracking restarts a slab at zero when its contact is new, reversed or reopened; that is the
  main source of the step-to-step noise and of the weaker slab correlation (variants in
  `docs/design/plate-check-baseline.md`). Task 7 should keep a slab across a contact's flicker.
- The reference slab pull, 3 × 10¹³ N/m, is about twice the 1.4 × 10¹³ that WP-13a computed from
  Faccenna et al. 2012 for 80-Myr floor. Speeds do not depend on it (the drag is scaled with it);
  the stresses and the ridge push's share do. Kept so that a ridge pushes with about 0.1 of the pull.

## Sources

- ForsythUyeda1975: full text in the library, read 2026-10-02 by a sub-agent (quotes with line
  numbers in the ledger).
- LithgowBertelloniRichards1998: full text in the library, read 2026-10-02.
- ConradLB2006: full text in the library, read 2026-10-02.
- Zahirovic2015: full text in the library, read 2026-10-02.
- Coltice2019: full text in the library; its abstract ("slabs pull plates at rapid rates and tear
  continents apart, with keels of continents only slowing down their drift when they are not
  attached to a subducting plate") was read 2026-10-02.
- Huang2019: abstract only.
- TurcotteSchubert_Geodynamics, ParsonsRichter1980, MolnarLyonCaen1988, Cloos1993, Coblentz1994,
  vanHunenAllen2011, ConradLB2002, ConradLB2004, Schellart 2004: not in the library; from memory or
  from abstracts, as marked.
