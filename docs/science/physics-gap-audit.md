# Physics gap audit

Status of this page: `stage 1 (an audit of the code and the docs, 2026-10-22; no runs)`. It is a
review, not a feature page, so it has no ledger ids of its own. It cites the ledger ids of the
code it reviews. Nothing in the code changed.

What was read: every module of `crates/deeptime-core/src/plates/`, the physics modules of
`surface/`, `sea_level.rs`, `steering.rs`, `climate/`, `hydro/`, `bake/water.rs`,
`bake/project.rs` and `crates/deeptime-format/src/{rivers,water}.rs`; the ledgers PLT, FRC, ISO,
CRE, HYP, STE, CLM, HYD and parts of ERO, GLA and MOI; the science pages; the plan, the research
report and notes, the decisions file with its log, and STATUS.md. Five read-only sub-agents did the
module sweeps; the judgments and the ranking are this page's. Line numbers are those of master
8c8b3b53. A "~" marks a line that was not re-opened.

The seven V1 item 1 measures (decisions log 2026-10-21) are used throughout as M1 to M7:

| Id | Measure | Target | D6 median |
|---|---|---|---|
| M1 | Land, share of the planet | 0.26 to 0.32 | 0.300 |
| M2 | Largest landmass, share of the planet | at least 0.12 | 0.124 |
| M3 | Continental crust under the sea, share of it | at most 0.25 | 0.278 |
| M4 | Land above 2 km, share of the land | at most 0.12 | 0.271 |
| M5 | Coast with high ground | at most 0.15 | 0.411 |
| M6 | Land under ice | at most 0.15 | 0.170 |
| M7 | Collisions per 100 Myr | 2 to 6 | 1.80 |

## Verdict

Deep Time does the vertical balance of the planet well: whole-column Airy isostasy, elastic
flexure, ocean floor that deepens with age, a sea that holds the planet's water, stream-power
erosion, a seasonal energy-balance climate and a weathering thermostat. What it lacks is most of
what takes high ground away. Apart from erosion, Earth removes high ground in four ways: the
lithosphere spreads under its own weight over whole continents, cooling lithosphere sinks, dense
roots drop off, and trenches take crust and sediment back into the mantle. The model has none of
these as physics. In their place it has rules: a 200 km same-plate collapse that stops at 50 km of
crust (2.65 km of height), a channel flow its own build log measures as negligible, a 70 km cap
that sends excess crust to the mantle, and no sediment subduction at all. Meanwhile the sources of
high ground (arcs and collisions) run at Earth's rate per kilometre of front, but over two to five
times Earth's length of active front, because rigid lattice plates do not split their boundaries
into ridges and transforms. So land above 2 km grows over a run from 0 to 0.20 to 0.41 of the land
(STATUS.md:1274). On Earth it stays near 0.05 by rock, 0.10 counting ice. The thin viscous sheet
being built now is the largest of these gaps, but it is not the only one. The trench budget, the
lithosphere's thermal state and the boundary organisation are first-order too. The first two are
what the sheet needs if it is to thin orogens without drowning the continents, as the hot-collapse
switch PLT-390 did. The boundary organisation and per-column thermal isostasy were known
deferrals. The continent-scale sheet itself, root removal, cratons and the continental side of
rifted margins were on no list before 2026-10-21. That is an oversight. The sheet's source, England
& McKenzie 1982, was cited on 2026-10-01, but was built only as a surface-height rule above
5.1 km. Separately, for what a player sees: rivers are single D8 chains that never branch. So
there are no distributaries, no delta plains above the sea, no floodplains, no meanders and no fans.
Deltas, floodplains and meanders were a known Phase 2 deferral (P2-03); fans and avulsion were never
named. The planet's discharges are Earth-like for its size. At 300 m by 35 m, a block cannot show
levees or the relief of a delta plain, but deltas, floodplains, meanders and fans all fit at the
block scale. The climate has no first-order gap of its own that moves the seven measures. Its ice
miss (M6) follows the high ground.

## The ranked first-order gaps

"First-order" here means a process that shapes Earth's large-scale relief or what a player meets,
and that the model does not have as physics.

| Rank | Gap | Class now | M1 to M7 it moves (expected) | Known or oversight | Build order |
|---|---|---|---|---|---|
| 1 | Continent-scale spreading of the lithosphere under its own weight (thin viscous sheet), across plate boundaries | (c) rules, (e) at continent scale | M4 and M5 down, large; M3 and M1 worse unless gap 3 lands too | Oversight until 2026-10-21; now being built | 1st (in progress) |
| 2 | Plate boundary organisation: ridge and transform segmentation, trenches that face their convergence, rifting and subduction starting from stress | (c) | M4 and M5 down, large; M1 down unless gap 3 lands too; M7 sign unknown | Known deferral (decisions log 2026-10-21) | 4th: ask Ben now, build after V1 item 1 |
| 3 | The trench budget: sediment subduction and subduction erosion, with arcs on normal convergence | (e) | M3 down and M1 held, but only with PLT-312; enables PLT-390 and gap 1 | Known, no task (plates.md:2537 "no rule") | 2nd |
| 4 | The lithosphere's thermal and mechanical state: thermal isostasy, stretched and cooled rifted margins, strength from temperature, removal of dense roots, cratons | (e), with (c) stand-ins | M4 down, medium on its own and large through gap 1; M3 and M5 through margins | Thermal isostasy deferred (2026-10-17, 2026-10-20); the rest oversight | 3rd, a minimal strength field with gap 1 |
| 5 | River and delta landforms: distributaries and avulsion, delta plains, floodplains, meanders, fans, braids | (e) | none | Deltas, floodplains, meanders, braids deferred (P2-03); fans and avulsion oversight | Own track, start now |
| 6 | Flexural foreland basins in front of ranges | (c) rule; flexure exists | M4 and M5 down, small | Known: "later work" (flexure.md) | After gap 1 |

### Gap 1: continent-scale spreading under the lithosphere's own weight

- **What it controls on Earth.** Thickened lithosphere stands high and has more gravitational
  potential energy than its surroundings. It flows outward until the forces at its edges balance
  that energy. England & McKenzie (1982) model continents as a thin viscous sheet. Houseman &
  England (1986) show that with a power-law rheology (n above 3) and an Argand number of 1 to 10,
  a plateau forms and then grows sideways instead of upward. Molnar & Lyon-Caen (1988) note
  that the crests of high ranges and plateaus extend while their flanks still shorten. The same
  physics thins quiet orogens (the Basin and Range, the Aegean). It also makes the diffuse
  deforming zones that cover about 15 % of Earth's surface (Gordon 1998).
- **What the model does.** Within a plate it uses three rules. Quiet crust over T_c = 50 km
  (PLT-256) releases its excess toward the nearest thinner sample on the same plate within
  200 km (PLT-257; the same-plate test is at `plates/collapse.rs:392`). Channel flow moves
  thickness above 50 km within two rings of samples (PLT-300, `plates/flow.rs:194`). A 70 km cap
  sends the rest to the mantle (PLT-258). The surface stage diffuses height only above the
  strength height h_s = 5.1 km (`surface/spread.rs`, ISO-018 to ISO-024). Gravitational potential
  energy enters the force balance only as an edge force on rigid plates (PLT-187,
  `plates/forces.rs:169`). It never deforms them.
- **What a player sees.** Plateaus up to four times Tibet's area on 60 to 70 km of crust
  (plates.md:3184). Their interiors find their nearest room 400 to 440 km away, so they never
  collapse (plates.md:3600). Old ranges keep 50 km roots and stand 2.65 km high.
- **Measures, expected.** M4 and M5 fall, by a large amount. Quiet crust over 51 km is 0.27 to
  0.44 of the high ground (STATUS.md:386). Spreading that crust alone takes M4 from 0.271 toward
  0.15 to 0.20 (arithmetic: 0.271 x (1 - 0.27 to 0.44)). More follows if the sheet also slows
  convergence. The nearest measured test is the hot-collapse arm (PLT-390 with PLT-391). It cut
  the high ground from 0.29 to 0.17, but the sea rose 200 to 280 m, land fell 0.269 to 0.220 and
  drowned crust rose 0.358 to 0.484. So M1 and M3 get worse unless gap 3 holds the sea. Spreading
  onto the drowned margin can also thicken its 28 to 30 km crust above the sea-level crossing
  (31.4 km). That would lower M3. M7: the sheet resists convergence, so collisions may weld
  sooner. The size is unknown.
- **What it takes.** A solve on the plate lattice for the depth-averaged velocity of the
  continental lithosphere, with n = 3. It is driven by gradients of the column energy the force
  balance already computes (`forces::column_energy`), with the plates' motion as the stiff part.
  Crust thickness is advected by the strain rate and kept in the rock pools. It replaces the
  reach, room and same-plate rules (PLT-254 to PLT-259, PLT-300) and the spreading half of
  `crust_max_km`. It needs a viscosity that keeps cratons stiff and hot orogens weak; see gap 4.
  The Argand number is the check against Houseman & England.
- **Known or oversight.** Oversight, found late. The source was cited on 2026-10-01 (ISO-019,
  `cited-unverified`; the session note says it was cited from memory). It was built only as a
  height threshold and a same-plate reach. The short reach was justified by Rey, Teyssier &
  Whitney's 150 km channel extrusion length (plates.md:154, 2203). That length belongs to a
  lower-crustal channel. Whole-lithosphere spreading works over the width of the orogen or more
  (England & Houseman 1986). No document proposed or ruled out a lithosphere-scale solve until
  plates.md:3711 (2026-10-21): "would be a flow solve, not a constant".
- **State (2026-10-23).** A stand-in is on by default, not the solve: the collapse measured from
  the sea with its cap (PLT-412, PLT-413), a thin sheet's strain rate from the column energy
  (PLT-414, PLT-415) and the reach at a plate's size (PLT-257, 5,000 km); with gap 3 it takes M4
  from 0.261 to 0.161 over 24 runs (plates.md, "On by default (2026-10-23)").

### Gap 2: plate boundary organisation

- **What it controls on Earth.** Spreading ridges break into short segments normal to the plates'
  relative motion, joined by transforms along it. Trenches face their convergence. Bird (2003)
  maps 52 plates. Subduction is 19.7 % of the boundary, transforms and non-contact convergence
  28.4 % + 6.7 %, and collision 8.8 % (the repo's read of Bird, plates.md:2408 ff.). Continents
  rift where stress and weakness allow. Subduction starts at weak zones and at old margins.
- **What the model does.** Plates are regions of a Fibonacci lattice, 24 of them, and their
  boundaries follow the lattice. Rifting is a Poisson hazard with random Voronoi fragments
  (PLT-082 to PLT-091, `plates/rifting.rs:40-86`). Subduction starts on old floor by another
  Poisson rule (PLT-062 to PLT-067, `plates/rifting.rs:293`). Reorganisation has no law of its own:
  plates.md:3535 calls it "the open law". The stress the force balance computes is recorded but
  does not act (FRC-022).
- **What a player sees.** Too much young mountain belt, and too many coasts with ranges on them.
  The subducting share is 0.31 to 0.347 (Earth 0.197). The active share of the continental margin
  is 0.47 (Earth 0.25 to 0.40). Collision fronts total 28,800 km (Earth 6,000 to 12,000).
  Collisions last up to 146 Myr at the 90th percentile (Earth at most 120).
- **Measures, expected.** M4 and M5 fall, by a large amount. The high ground is young orogen: by
  last orogeny it is 0.54 to 0.72 collisional and 0.25 to 0.43 Andean (plates.md:2372). With
  per-km stacking unchanged, the collision-built part should shrink roughly in proportion to the
  front's 2.4 to 4.8 times excess (inference, not measured). M1 falls unless gap 3 lands with it.
  Island arcs build crust at 2.7 times Earth's rate, and that holds the land up: PLT-312 alone
  took land to 0.23 to 0.25 and raised M4 (plates.md:2439 ff.). M7: more, shorter collisions are
  plausible, but the sign is not known.
- **What it takes.** Split spreading boundaries into ridges and transforms by the relative
  motion. Orient trenches. Start rifts and subduction from the computed stress. The plate count
  may need a finer lattice (the quick lattice cannot hold plates under 0.0377 sr, PLT-372). This
  is how plates are made and moved, Ben's scope.
- **Known.** Deferred until after V1 item 1 (decisions log 2026-10-21; the row on the
  plate-geometry merge). That row judged that none of the seven criteria needs it. This audit
  disagrees for M4 and M5. The docs' own list of what is left names this first (plates.md:2486,
  "The extent of young orogen is the high ground").
- **State (2026-10-23, the docking lane).** V1 item 1 on base3 (key 88c144b6c2cb): the high coast
  is held by a few long continent-continent collisions that never end (states 60 Myr or older,
  0.482 of the last 30 Myr's front-time), not by the count of dockings (55.7 sutures per 100 Myr,
  Macdonald et al. 2019's 19 to 39; plates.md "Docking: what stands at the collision fronts").
  Their incoming plates are the largest, pulled on by slabs elsewhere after their own broke off,
  and PLT-078 let a continent weld only on a stall that never came. Built: Stern 2004's
  transference with Gurnis et al. 2004's force (3 × 10^12 N/m) and critical convergence (100 km)
  (PLT-516 to PLT-518), on by default on the lane's branch, not merged; off is bit-identical. Arm
  dock-T (8 runs): no guard failure; collisions 33 / 77 Myr at p50 / p90 against 48 / 143, coast
  with high ground 0.354 to 0.114 and land above 2 km 0.153 to 0.053 (both near Earth). But the
  long collisions were also the model's sink of continental area: without them the crust grows
  from 0.42 to 0.57 of the planet (Cogley 1984: 0.41), the sea rises about 900 m, the land falls
  to 0.129 and the largest landmass to 0.021, and one plate holds half the planet in 16.5 % of
  the steps. Next, another lane: the missing area sink of continental crust (the forearc's
  erosion, PLT-441, thins the edge but never consumes it; nothing turns continental crust
  oceanic). PLT-064's clearance is not replaced.
- **State (2026-10-23, later).** Every stage on by default on the lane's branch, not merged
  (plates.md, "The arms"). Under Ben's rule of 2026-10-23 each cumulative 8-run arm on the new
  base (key 15660e3c5636, base2's seeds 1 to 4 as the control) was read for divergence and the
  guard: g2-A (ridge segments), g2-AC1 (+ initiation off the passive margin), g2-AC1C2 (+ the
  stress weights and rifts along thin lithosphere) and g2-AC1C2R (+ PLT-371's reversal judged by
  the arc's crust). No run failed and none had a step beyond two Earth reliefs, so all six
  switches are on and the rules they replace (PLT-067's split, PLT-100/101's midline in the
  ridge's bands, the hazards' share between plates, PLT-087's great-circle distance, PLT-056's
  overlap judgement) run only with them off. The two realism checks that fire on the base still
  fire on every arm, unmoved at p ≥ 0.57: land above 2 km 0.126 on the control, 0.110 to 0.146
  on the arms (fires above 0.07); coast with high ground 0.310, 0.254 to 0.330 (fires above
  0.11). The docking episodes stay about 124 to 139 per 100 Myr (Macdonald et al. 2019: 15 to
  46). What moved: the ocean-ocean share of the subducting boundary 0.300 to 0.347 (Stern &
  Scholl's 0.35, p < 0.001), the reversals 16.3 to 12.1 per 100 Myr with R (p 0.005), and the
  front in collision 28,600 to 33,800 km with R (p 0.010). The transform share did not move
  (0.213 to 0.215 against Bird's 0.283): the ridge's segments do not show in the boundary's
  classes on the quick lattice. Next: induced initiation near trenches, its own stage (it would
  replace PLT-064's clearance).
- **State (2026-10-23).** Measured and built, every stage off by default and not yet on the pool
  (plates.md, "Plate boundary organisation"; PLT-500 to PLT-507). The before measures (base
  7f9d8f115049, 24 runs) answer "orient trenches": the convergence across the oceanic front is
  Bird's (53.1 against 53.2 mm/yr), so no law is added for it (PLT-501). The subduction excess is
  a shortage of transform (0.7 times Bird's) more than of ridge, and more than half of the coast
  at a front is at a front an initiation made, because PLT-067's split runs along the
  continent's edge. Built behind switches, each tested against its source: ridge segments normal
  to the motion with transforms along it (PLT-502, replaces PLT-100/101's midline in its bands),
  initiation off the passive margin at the arc's reach (PLT-503, replaces PLT-067's split), rifts
  and initiations weighted by the force balance's stress (PLT-504, PLT-505) and rifts along thin
  lithosphere (PLT-506, needs gap 4's thermal state); reorganisation is their composition
  (PLT-507). Off is bit-identical (`pool.sh alias`, seed 1, 1000 Myr). The arms wait for the new
  base after the flexure and gap 4 lanes merge; the plate count (PLT-372) is not changed. The
  collision belts that hold 0.69 of the high coasts (gap 4's `coastset`) are mostly dockings of
  small terranes and arcs at active margins, about 130 episodes per 100 Myr against Macdonald et
  al. 2019's 15 to 46 arc-continent sutures, not continent-continent fronts running out to sea
  (the measure `collcoast`, PLT-508; plates.md "Collision belts on the coast"). The belts' spread
  inland is not built here.

### Gap 3: the trench budget

- **What it controls on Earth.** Trenches return crust to the mantle: subducted sediment plus
  forearc crust scraped from below. Stern & Scholl (2010, as read by the repo) put the total near
  2.5 km³ a year. Clift & Vannucchi (2004) find margins erode where convergence is above about
  6 cm a year and the trench holds less than 1 km of sediment. That keeps ocean sediment near
  GlobSed's 3.37 x 10⁸ km³ and lets arcs at about 1.5 km³ a year balance the losses.
- **What the model does.** No sediment subducts. Every overridden column's pools accrete onto
  the column on top at resampling (`plates/ridges.rs`, `pools::accrete`). Subduction erosion has no
  rule (plates.md:2537). The ocean holds 1.6 to 2.1 x 10⁹ km³ of sediment, five to six times
  Earth's. Oceanic-labelled crust at least 10 km thick covers 0.325 of the planet (Earth's 10 to
  20 km crust covers 0.074). The sea ends 540 m over its calibrated datum (STATUS.md:253 ff.).
- **What a player sees.** Shallow, sediment-choked ocean basins. A high sea that drowns continental
  margins.
- **Measures, expected.** This gap enables more than it moves directly. Subducting the sediment
  alone (the sub80 arm) took the census's sea from 538 m to -181 m, but land fell from 0.247 to
  0.216, M3 did not move (p 0.56) and the crust lost 0.132. The continents are held up by arc
  crust grown from accreted pools. Landed with arcs on normal convergence (PLT-312) and a loss
  budget, the sea should hold while the hot window (PLT-390) or the sheet (gap 1) thin orogens.
  Expected: M3 down, M1 held, and the M4 cut kept. The size cannot be estimated from the measured
  arms.
- **What it takes.** A share of the downgoing plate's sediment pools goes to the mantle at
  contacts and at resampling. The overriding forearc erodes where Clift & Vannucchi's conditions
  hold. Arc output is taken on the normal convergence (PLT-312, already built and off). The crust
  budget books every loss against Stern & Scholl's table (`plates/crust_budget.rs` already
  books every volume). This touches plate code only, with no format change.
- **Known.** Yes: plates.md:2537 ("no rule"), sea-level.md's open questions and STATUS.md:253.
  PLT-312 is off "until the continental crust's losses are on Earth's numbers too"
  (plates.md:2459). No task holds it.
- **State (2026-10-23).** Sediment subduction (PLT-440, s = 0.12), subduction erosion (PLT-441,
  PLT-442) and the trench's release (PLT-443) are on by default with gap 1's stand-in: M1 0.306
  in band, M3 0.238, criteria 3.42 a run against 2.33 (p 0.004, 24 runs; plates.md, "On by
  default (2026-10-23)"); PLT-312 stays off and the sea still stands 840 m over the datum.

### Gap 4: the lithosphere's thermal and mechanical state

- **What it controls on Earth.**
  - *Thermal isostasy.* Hot lithosphere stands higher. Over continental heat flow of 35 to
    100 mW/m² the elevation range is about 3 km at equal crust (Hasterok & Chapman 2007). Hot
    back-arcs reach sea level on about 27 km of crust, cold cratons on about 38 km; at equal
    thickness that is about 1,600 m (Hyndman & Currie 2011).
  - *Rifted margins.* A stretched margin subsides by McKenzie's (1978) law. The thermal
    lithosphere is 113 to 139 km thick (Steckler & Watts 1978). Wide shelves and coastal plains
    result.
  - *Strength.* A plateau's support and its collapse depend on the Moho temperature (Rey,
    Teyssier & Whitney 2010).
  - *Root removal.* Dense mantle lithosphere under a thickened belt drops off. The belt rises and
    then extends (Bird 1979; Houseman, McKenzie & Molnar 1981; England & Houseman 1989; Kay & Kay
    1993).
  - *Cratons.* Depleted, buoyant, stiff keels stay put (Jordan 1978).
- **What the model does.** It uses one mean crust density, 2,830 kg/m³ (PLT-281), with an eclogite
  root (PLT-278 to PLT-282) and instant settling (ISO-078). No sample carries a lithosphere base or
  a temperature (`plates/state.rs`). Heat flow is a proxy, 55 + 35·exp(-t/60), used only by the
  deposit stage (PLT-137, `plates/mod.rs:1309`). "Craton" is a regime label (PLT-139). Rifted
  margins are a linear thickness taper over W = 200 km with τ 20 Myr; the ocean side then follows
  the age-depth law (PLT-261 to PLT-264, `plates/transition.rs:155,208`). The 70 km cap stands for
  "the foundering of an eclogitic root" (PLT-258). The rise cap, 3.5 mm/yr (PLT-288, Garzione
  2006), bounds an event the source says needs root removal, and the model cannot produce one.
- **What a player sees.** Old ranges like the Urals stand too high. Passive margins have no shelf:
  median 0 km against Earth's 84 to 88 km (sea-level.md:569). Slopes are 2.5 to 3.2 degrees. Slope
  and deeper ground hold 0.29 of the continental crust; Earth's hold 0.16.
- **Measures, expected.** M4 falls by a medium amount directly. The old fifth of the high ground
  would sink by up to 1,600 m (plates.md:2378). It falls by a large amount through gap 1: without
  a strength that depends on temperature, the sheet spreads cratons as readily as hot orogens.
  Earth's collapsed orogens reach 26 to 37 km only where hot (STATUS.md:386). M5 falls: margins
  become low coastal plains and shelves. M3's sign is uncertain. Thermal subsidence drowns
  stretched crust, but a stretched margin narrower than the taper drowns less. M6 follows M4.
- **What it takes.** This is an architecture change.
  - Each `Sample` gets a thermal age or the depth of its thermal lithosphere. Rifting, orogeny and
    plumes reset it; it cools as a plate.
  - An isostatic term from it goes into the Airy height (`plates/collapse.rs:222`,
    `plates/settle.rs:81`).
  - At rifts, a stretching factor and cooling replace the taper.
  - A strength comes from it for gap 1 and for the collapse.
  - A root-removal rule on a Rayleigh-Taylor timescale replaces the founder half of
    `crust_max_km`.
  - The heat-flow proxy (PLT-137) can then be computed from the same state.
- **Known or oversight.**
  - Thermal isostasy: a known deferral, made on 2026-10-17 and again on 2026-10-20 (decisions log;
    plates.md:2495), on the ground that it lowers only the old fifth of the high ground. That
    reasoning holds for M4 on its own. It does not hold for gap 1, which needs the strength
    contrast.
  - The continental side of rifted margins: never mentioned. The taper "stands for McKenzie's
    thermal subsidence" on the ocean side only (plates.md:294).
  - Root removal: never mentioned as a mechanism. The papers sit in the bibliography, with no
    ledger row or deferral note.
  - Cratons as stiff, buoyant keels: never mentioned.
- **State (2026-10-23).** Stages 1 to 3 are built and, under Ben's rule of 2026-10-23 (physics on
  once built and correct; decisions log "Always, fix V1 elsewhere"), on by default except two parts
  ([lithosphere-thermal-state.md](lithosphere-thermal-state.md), PLT-460 to PLT-478, ISO-110 to
  ISO-112). On: the state (each sample's thermal lithosphere depth and keel, PLT-460, with the heat
  flow from it, PLT-467, in place of PLT-137's proxy, and the final pass that fixes the release
  pace's 33.7 mW/m² bake fault, PLT-469, on by default on its own too), the strength from the thermal age for the
  thin sheet and the flexure's Te (ISO-112, in place of the orogeny's date), rifted margins drawn
  by a stretching factor β (crust T_n / β on a smoothstep across the proximal, necking and
  hyperextended domains, lithosphere and keel L / β; PLT-475 and PLT-476, in place of PLT-261's
  linear ramp) and root removal (a collision belt's crust over 50 km has its mantle lithosphere
  drop off to the Moho at 10 Myr, Houseman & Molnar 1997, and the founder sinks at that pace;
  PLT-477 and PLT-478, in place of PLT-258's all-at-once removal). The switches for stages 2 and 3
  are gone with the rules they replaced. The correctness evidence: unit tests against the sources,
  and no step beyond 2 Earth reliefs on any run of any arm (8 runs each for stages 2 and 3 alone
  and with the strength, 24 for the strength); no arm ran all four at once, which master's next
  24-run base does. Off: thermal isostasy (ISO-110), an open fault, since neither frame is right by
  its sources while the model's sea stands 538 to 861 m over its datum (gap 1; Hyndman & Currie's
  frame is right once the sea holds at the datum, at L_ref 125 km), and the backstop (PLT-470), a
  stand-in that reweights the kinematic source PLT-048 rather than a force balance. The diagnosis
  of the high coasts (the `coastset` measure over the base's 24 runs): 0.690 sit at collision
  belts, 0.085 at rifted margins, so stage 2 can move coast_hi by 0.03 at most; the lever is
  collision belts at open coasts, which is gap 2's.

### Gap 5: river and delta landforms

This gap gets its own section below, "Rivers and deltas". In short: nothing in the code builds a
delta, distributary, floodplain, levee, meander, fan or braid. It moves none of M1 to M7, but it is
what a player meets most.

- **State (2026-10-23).** Stage 1 of 4 is merged (master 84fb7d7c): delta plains above the sea at
  river mouths, sized by the BQART load and the Nienhuis area fit against the shelf's room, and
  distributaries across them as new chains in the river file (its layout 2, `from`). On the D5
  bake 359 of 1,054 loaded mouths get a plain, 0.39 % of the land (Earth's 0.57 % is reported
  beside it, not aimed at), and 233 rivers split into 632 distributaries; 24 of the 25 largest
  rivers have a plain and 15 split (rivers-and-lakes.md, "Stage 1 on the D5 bake"). Stage 1b, on
  branch worktree-agent-a9e4e25c4744d5a75: the projected world's plain stands above the sea on
  97 % of its cells (67 % in stage 1; HYD-146), no plain column carries an ocean biome (752 did;
  BIO-185), and the 8 plains counted as having no path are fronts the main stem already holds
  (HYD-147, not changed). Floodplains and levees, meanders, fans and braids remain.

### Gap 6: flexural foreland basins

- **What it controls on Earth.** A thrust belt loads the stiff plate beneath it. The plate bends
  into a basin 150 to 450 km wide holding 2 to 7 km of fill: the Ganges plain, the Po, Mesopotamia,
  the sub-Andean plains. Molnar & Lyon-Caen (1988) note that an elastic plate over a fluid is a
  good first approximation for foredeeps.
- **What the model does.** Elastic flexure is built and on (ISO-040 to ISO-069). But the plate
  model settles every column to its own Airy height, so a range is compensated in place. The
  measured foredeep is 41 km wide and 113 m deep (plates.md:784). flexure.md says: "Deep basins in
  front of mountain ranges (the Ganges plain) do not appear yet. They need the plate model to build
  a range as a wedge on the edge of a stiff plate, which is later work." The foreland line-load
  profile (PLT-193) "will not be built: the general flexure already bends the plate under every
  load" (plates.md:397, decisions log 2026-10-02). The measurement shows that premise did not hold.
- **Measures, expected.** M4 and M5 fall by a small amount: lowland appears beside ranges. The
  larger effect is on what a player sees. These basins are where Earth's big lowland rivers,
  megafans and inland seas lie.
- **What it takes.** The plate model hands the belt's load to the surface flexure instead of
  settling it locally. The wedge (PLT-192) sits on the plate edge. Sediment from the range,
  already a flexure load, fills the foredeep. Best done after gap 1, which changes how belts
  thicken.

### Recommended order, and why

1. **Gap 1, the sheet** (in progress). Build it with a minimal strength field from the start, for
   example a thermal age reset by orogeny, so that cratons do not spread.
2. **Gap 3, the trench budget, with PLT-312.** It is cheap, it touches plate code only, and it is
   the precondition the docs name for PLT-312 and PLT-390. Without it, every way of thinning
   orogens drowns the continents.
3. **Gap 4, the rest of the lithosphere state.** This is thermal isostasy, the stretched and
   cooled margin in place of the taper, and root removal in place of the founder cap.
4. **Gap 2, boundary organisation.** It is Ben's scope and big. Ask him now, so the answer is in
   hand if gaps 1, 3 and 4 leave M4 and M5 short.
5. **Gap 5, rivers.** A separate subsystem (hydro, the bake, the format), so it can start now. Do
   delta plains and distributaries first.
6. **Gap 6, forelands,** after gap 1.

## Rivers and deltas

Ben has never seen a navigable river or a river delta in the pack. The code explains the delta
directly. The navigability has likely causes, listed below, but needs the measurement another
agent is running.

### What the model does

- **Routing.** D8: one receiver per cell (`surface/routing.rs:2-12`). The file explains why: the
  O(n) implicit stream-power solve of Braun & Willett (2013) and Yuan et al. (2019) needs a single
  receiver. The bake's network follows the largest donor, and every chain has exactly one `into`
  (`hydro/network.rs` `run`, ~606-1200). A river can join others but can never split.
- **Channel size.** Width = clamp(Q^0.5, 1, 64) blocks and depth = clamp(0.5 Q^0.4, 1, 12) blocks.
  Q is in m³/s (`hydro/mod.rs:405-407`, ~488-491, ~619-628; HYD-001 to HYD-010). The exponents are
  Leopold & Maddock's (1953) downstream values. The coefficients are game choices.
- **Caps.** They never bind. Width reaches 64 blocks at Q = 4,096 m³/s, and depth reaches 12
  blocks at Q of about 2,822. The planet's largest river carries 528 to 1,345 m³/s (STATUS.md:6231,
  7550). That makes it 23 to 37 blocks wide and 6 to 9 blocks deep.
- **The mouth.** The mouth fill (`surface/marine.rs:131`, ERO-050 to ERO-052) raises sea cells to
  sea level - `fill_depth_m` - 0.0005 x distance, and the presets set `fill_depth_m` to 0.0
  (`presets/earthlike_quick_16k.json:185-186`). The fill never rises above sea level. It is a
  disc around the mouth, not a lobe. It runs on the 4.8 km L1 grid only. The L2 stage drops the
  load it sends to the sea ("their loads to the sea leave the grid, as the rivers' do here",
  `surface/young.rs:740`). The L3 amplifier lets sea-bound sediment leave too
  (`surface/amplify/kernels.rs` ~365-420, AMP-010 to AMP-014).
- **Plan form.** Chains are smoothed by a two-pass Gaussian ("it does not create meanders", HYD-019,
  `hydro/network.rs` ~1543-1570) and drawn as straight capsules
  (`crates/deeptime-format/src/rivers.rs:223-330`). The median sinuosity, 1.035, is an output, not a
  setting (HYD-027).

### Landform by landform

| Landform | Earth | The model | Class | Physics or block scale | Known? |
|---|---|---|---|---|---|
| Distributaries and avulsion | Rivers on fans and deltas split, and jump to a new course every 28 years (the Kosi) to 1,400 years (the Mississippi) (Slingerland & Smith 2004). Delta avulsions sit near the backwater length; fan avulsions sit at the slope break (Ganti et al. 2014) | None. One receiver per cell, one `into` per chain | (e) | Physics. Drawn channels are 5 to 37 blocks wide, so splits fit. Single avulsions are far below the 2 Myr step, so a delta's network has to be drawn as its steady pattern, not as a history | Avulsion: never named. "Braided mouths": P2-03 (plan:4270) |
| Delta plain above the sea | Delta area grows with discharge and sediment load and shrinks with offshore room (Syvitski & Saito 2007). Lobes build out and are abandoned (Paola et al. 2011) | An underwater disc, filled to sea level at most, on L1 only. L2 and L3 drop the sea-bound load | (c) knobs on (a) mass conservation | Both. The plain's few metres of relief are under one 35 m block, so it can only be a flat at sea level. But lobes 10 to 100 km long are 30 to 300 blocks, and fit | "deltas": P2-03 (plan:4270). Dropping the load at L2 and L3: never named |
| Floodplain | Flats kilometres wide built by overbank deposition | No floodplain process. Flats can come only from deposition in the erosion solves (Yuan G = 1, `surface/spl.rs`), and that is not measured | (e) | Physics. Flats kilometres wide fit | "HAND floodplains": P2-03 |
| Natural levees | Ridges a few metres high along the banks | None | (e) | Block scale. Under one 35 m step; at most a bank block | P2-03 by implication |
| Meanders | Wavelength 10 to 14 channel widths, bend radius 2 to 3 widths (Leopold & Wolman 1960); sinuosity commonly 1.1 to 1.5 (the repo's figure, from memory) | Gaussian smoothing only; 1.035 at the median | (c) smoothing; (e) meandering | Both. At drawn widths a meander is 230 to 500 blocks long, which fits. A real channel of Q = 500 is about 200 m wide, under one block | "meanders beyond the smoothing": P2-03; audit option B2 (`docs/research/notes/grid-artifact-audit.md:290-298`) |
| Braided rivers | Steep, sediment-rich, wide belts | None. The amplifier routes multiple-flow (AMP-006, AMP-007), but the drawn rivers come from the bake's D8 | (e) | Physics. Braid belts kilometres wide fit | "braided mouths" only |
| Alluvial fans | Semi-conical bodies built where flows leave a range and spread (Blair & McPherson 1994) | No fan form. Deposition at a range front happens, but not as a fan | (e) form; (a) deposition | Physics. Fans tens of kilometres across are 100 blocks or more | Never named |
| Channel size | W and D grow as Q^0.5 and Q^0.4 (Leopold & Maddock 1953) | Same exponents, coefficients scaled to blocks. Channels cover 2.2 to 2.6 % of the land, Earth's 0.58 % (rivers-and-lakes.md, Allen & Pavelsky 2018 as read by the repo) | (b) exponents, (c) coefficients | Game choice: about four times Earth's channel area on purpose (HYD-003, HYD-005, HYD-008) | Documented |
| Discharge and catchment | See the next table | Real m³/s from Fu-Budyko runoff over real areas (`hydro/network.rs:222-227`). No storage, groundwater or seasons (HYD-011) | (a) accumulation, (c) thresholds | Scale-honest | Documented |

### Are the discharges Earth-like for the planet's size?

Yes. The arithmetic below uses the planet's own figures and Earth's well-known values, which are
from memory except where cited.

| Quantity | Earth | The planet | Earth's scaled to the planet |
|---|---|---|---|
| Surface | 510 million km² | 7.69 million km² (16,384 blocks x 300 m round) | 1.5 % |
| Land | 149 million km² | 2.31 million km² (M1 0.300) | |
| Largest landmass | Afro-Eurasia, 0.165 of the surface | 0.95 million km² (M2 0.124) | |
| Total river flow to the sea | about 1.2 million m³/s | about 20,000 m³/s at Earth's mean runoff of about 0.27 m a year | |
| Largest river | Amazon, about 209,000 m³/s, about 17 % of the total; Congo about 41,000 m³/s, 3.4 % | 528 to 1,345 m³/s (STATUS.md:6231, 7550), 2.6 to 6.7 % of the total | Amazon's share: about 3,400 m³/s; Congo's share: about 680 m³/s |
| Largest basin, share of its continent | Amazon 0.34 of South America; Congo about 0.12 of Africa; Mississippi about 0.13 of North America | about 106,000 km², 0.11 of the largest landmass, if runoff is 400 mm a year (the river sub-agent's assumption) | |

The largest river sits between a Congo-share and an Amazon-share river. Its basin is a typical
share of its continent. No Amazon is possible on a planet with 1.5 % of Earth's area. A river of
528 to 1,345 m³/s is Rhine to Elbe class in absolute terms. Earth's big deltas sit on rivers of
10⁴ m³/s and more. Smaller rivers build deltas too, wherever their sediment beats the room offshore
(Syvitski & Saito 2007). The model has no process for any of them.

### Why a player finds no navigable river (inference, to check against the measurement)

1. **Size is not the problem.** Every drawn river is at least one block deep and wide. The largest
   are 23 to 37 blocks wide. The water surface joins the sea at the mouth (`water.rs:140-185`).
2. **The 35 m block makes waterfalls.** A river's surface falls in whole-block steps (HYD-071), and
   a boat cannot climb them. A river with slope s drops one block every 35/s metres. At s = 10⁻⁴
   (Earth's big lowland rivers) that is once per 350 km, or 1,170 blocks. At 10⁻³ it is once per
   35 km (117 blocks). At 10⁻² it is once per 3.5 km (12 blocks). This is the block scale.
3. **The model's rivers are short and steep.** The median land point is 148 to 166 km from the
   coast; Earth's is 433 km (plates.md:2362). 0.41 of the coast has high ground behind it (M5),
   and 0.27 of the land is above 2 km (M4). A river falling 2 km over 150 km has a slope of about
   10⁻², so it drops a block every dozen blocks. This is gaps 1 and 2, not the river code.
4. **The lowlands where Earth's navigable rivers run are missing or drowned.** That means foreland
   basins (gap 6), coastal plains on passive margins (gap 4) and the drowned margin (M3).
5. **Chains break.** The bake re-routes D8 on the L3 surface, while the amplifier carved the
   valleys with multiple-flow routing, so a drawn river can leave its valley
   (grid-artifact-audit.md:295). `project_rivers` splits chains at jumps over 256 blocks
   (`bake/project.rs:1998-2110`). A chain ending in a dry sink is labelled `Mouth::Sea`.

### What adding them takes

1. **Delta plains.** Let the mouth fill build up to sea level plus up to one block, where supply
   beats the room offshore. Spread it along the flow as lobes, not a disc. Keep the sea-bound load
   at L2 and L3 and lay it at the mouth, instead of dropping it.
2. **Distributaries.** Give the bake's network a graph with splits, not a tree. On the fill, split
   a chain at an apex near the backwater length (Ganti et al. 2014) into a steady set of branches.
   This changes the rivers' world-data format, which is Ben's call.
3. **Floodplains.** HAND flats along the valley floor (the planned P2-03), with overbank
   deposition. Levees as a bank texture only.
4. **Meanders.** Draw the centreline as meanders scaled to the drawn width (10 to 14 widths,
   Leopold & Wolman 1960), kept inside the valley floor (audit option B2).
5. **Fans.** Where a valley leaves a range onto low ground, fix an apex and spread the deposit and
   the channel radially. This needs the splits from step 2.
6. **Waterfalls.** Navigability upstream is mostly a matter of slope (gaps 1 and 2) and of the
   block. Locks or ramps in the game are a game choice, not physics.

## Second-order gaps

These are physical and missing or simplified. Each either moves M1 to M7 little or follows from
the first-order gaps.

| Gap | Earth | The model | Effect | Known? |
|---|---|---|---|---|
| Dynamic topography | About ±500 m at 10⁴ km wavelength, and significant at 10³ km (Hoggard, White & Al-Attar 2016); flow-induced deflection under about 300 m (Molnar, England & Jones 2015); transient (Flament, Gurnis & Müller 2013) | None (`plates/forces.rs:197-204` moves plates sideways only) | Epeiric seas and continental flooding over 10 to 100 Myr; small on M1 and M3 | Never mentioned |
| Mantle convection pattern, supercontinent insulation | Gurnis 1988: a supercontinent insulates and breaks | Fixed degree-2 flow, 5 mm/yr (FRC-015); "changes nothing measurable" (`docs/design/plate-check-baseline.md:571`) | Supercontinent rhythm, M7 | Insulation debated and deferred 2026-10-02 (`session-supercontinent-heating.md:64-65`) |
| Plume population and LIPs | Earth has 26 to 69 plumes; LIP basalt covers 5.9 % of the planet | About 3 live plumes; LIP basalt covers 0.05 to 0.8 % (plates.md:518-528; STATUS.md:12551); random, no thermal link (PLT-108 to PLT-120) | Rift sites, CO2 pulses | Known, "left for its own task" |
| Ice sheet mass balance and flow | Ice needs snowfall; sheets flow (Glen's law) | Extent from the Ohmura T_ELA, which bottoms out at -2.4 degC with no snow (`climate/glaciers.rs:26-30`); Nye profile capped at 3,000 m (CLM-103); flow built but off (CLM-147); no height feedback (CLM-228) | M6. The 2026-10-21 measurement found the plateau ice comes from cold, not from the dry rule, so M6 follows M4 | Flow off by Ben's 2026-10-02 decision; the dry floor noted in climate.md:379 |
| Orbital forcing before the last 3 Myr | Orbital cycles pace every ice age | Circular orbit outside a 3 Myr window (CLM-031, CLM-038) | Cyclothems, glacial sea-level swings in older ice ages | Ben 2026-09-29, "Recent ice-age cycles" |
| Atmospheric circulation and rain | Hadley width depends on rotation; monsoons come from land-sea heating | Fixed belts at 30 and 60 degrees; Earth's 17-point zonal rain table x 1.42 (CLM-178, CLM-201); moisture-budget rain built but off (`rain_model` "belts") | Rain on supercontinent interiors, so erosion and weathering there | Monsoons "a follow-up" (decisions log 2026-10-06) |
| Carbon cycle beyond silicates | Organic burial, carbonate weathering and burial, lithology (basalt weathers faster) | Silicate thermostat with supply limit and seafloor weathering; organic carbon "left out" (`climate/carbon.rs:20`); fixed buffer R = 5; one 1.26 mol/kg for all rock | CO2 levels (natural 525 to 1,576 ppm), hence M6 | Organic burial stated as left out; lithology never mentioned |
| Clouds and water-vapour feedback | State-dependent | Folded into a linear OLR and fixed albedos (CLM-003, CLM-004, CLM-007 to CLM-009) | Climate sensitivity | Stated (climate.md:451) |
| Ocean heat transport | 1.5 to 2 PW | 3.2 PW, target not met (CLM-133); conveyor prescribed at 30 Sv (CLM-131) | Polar warmth | Known |
| Permafrost and periglacial processes | Frost weathering, solifluction, thermokarst lakes | None; frozen ground gives no runoff (HYD-038) | Cold-land erosion and lakes | Never mentioned |
| Map-scale hillslope transport | Hillslope κ about 0.0036 m²/yr (Roering, as ERO-177) | κ = 2 m²/yr, 550 times, uncited (ERO-023); plateaus wear 63 m/Myr on the flat against Tibet's under 10 | Lowers M4 slightly, the wrong way for the right reason | Known issue ("plateaus wear too fast") |
| Sea level terms | Thermal expansion, groundwater, mantle water exchange | None (sea-level.md:688-702); steering's offset stands in for mantle exchange | Tens of metres | Known |
| Viscous GIA | Relaxation over 10³ to 10⁴ years | Elastic only (ISO-064) | None at 2 Myr steps | Stated |
| Uplift-weathering link (Raymo & Ruddiman) | Uplift raises weathering | Only through the erosion supply cap (CLM-152 to CLM-157) | CO2 | Never mentioned; partly emergent |

## Deliberately out of scope

| Item | Where it is ruled out |
|---|---|
| A 3-D mantle convection solve | "Offline research only" (`docs/research/notes/tectonics_engines.md:94,508`) |
| Viscoelastic flexure and glacial cycles inside a step | flexure.md:124-145, 299 |
| Cloud state dependence, CO2 condensation, Rayleigh scattering | climate.md:451 |
| Stationary eddies, salinity, coastal upwelling, sea-ice drift | per-cell-climate.md:404-447 |
| Dry land below sea level | Ben, 2026-10-01 (plan:4267) |
| Talus and scree, 3-D crags | plan:2395-2396 |
| Single avulsion events in time | below the 2 Myr step by construction (this page) |

## Known deferral or oversight

| Mechanism | Verdict | Evidence |
|---|---|---|
| Continent-scale spreading (thin viscous sheet) | Oversight until 2026-10-21 | Source cited 2026-10-01 (ISO-019), built as a 5.1 km threshold; no doc proposed or ruled out a lithosphere-scale solve before plates.md:3711 |
| Diffuse deformation across plate boundaries | Oversight | The same-plate filter is in every spreading rule (`plates/collapse.rs:392`, `plates/flow.rs:194`, `plates/rate.rs:179`); no doc discusses crossing it |
| Boundary segmentation, reorganisation | Known deferral | Decisions log 2026-10-21; plates.md:3535 "the open law" |
| Stress-driven rifting | Oversight as physics | Poisson hazard (PLT-082 to PLT-091); no doc asks for stress-driven rifting |
| Sediment subduction, subduction erosion | Known, no task | plates.md:2537, 2570; sea-level.md open questions; STATUS.md:253 |
| Per-column thermal isostasy | Known deferral | Decisions log 2026-10-20; plates.md:2495 (2026-10-17) |
| Continental stretching at rifted margins | Oversight | No hits for a stretching factor; the taper stands for McKenzie only on the ocean side (plates.md:294) |
| Rift-flank uplift | Oversight | Never mentioned |
| Root removal, delamination | Oversight | Bibliography only; PLT-258 names the cap as its stand-in |
| Cratons as stiff, buoyant keels | Oversight | Regime label only (PLT-139) |
| Foreland basins | Known, with a failed premise | flexure.md "later work"; PLT-193 dropped 2026-10-02 because "the general flexure already bends the plate" |
| Hotspot population | Known | plates.md:518-528 |
| Dynamic topography | Oversight (second-order) | Never mentioned |
| Supercontinent insulation | Known deferral | session-supercontinent-heating.md:64-65 |
| Flowing ice sheets | Known deferral | CLM-147, Ben 2026-10-02 |
| Monsoons | Known follow-up | Decisions log 2026-10-06 |
| Orbital cycles before 3 Myr | Known game choice | Ben 2026-09-29 |
| Organic carbon burial | Stated simplification | `climate/carbon.rs:20` |
| Permafrost | Oversight (second-order) | Never mentioned |
| Deltas, floodplains, braided mouths, meanders | Known deferral | P2-03 (plan:4265-4271) |
| Avulsion, alluvial fans | Oversight | Never named |
| Sea-bound load dropped at L2 and L3 | Oversight | `surface/young.rs:740` comment; no deferral |

## Scope failures: every reach limit, clamp, cap and tuned knob

"Stands in for" names the missing physics. A knob marked "target" was tuned to hit a number.
Pure numerical guards are listed at the end of each group.

### plates/

| Limit | Where | Value | Ledger | Stands in for |
|---|---|---|---|---|
| Collapse reach | `params.rs:549`; `plates/collapse.rs:122,371-372` | 200 km | PLT-257 | Continent-scale spreading (gap 1) |
| Same-plate filter | `plates/collapse.rs:392`; `plates/flow.rs:194`; `plates/rate.rs:179` | | PLT-254, PLT-300 | Deformation across sutures and boundaries (gap 1) |
| Room only up to T_n | `plates/collapse.rs:381-395` | 35.1 km (founder and rate cap to 50 km) | PLT-254, PLT-277 | Thickening of neighbours by flow (gap 1) |
| Collapse floor T_c | `params.rs:545` | 50 km, so 2.65 km high | PLT-256 | Strength from temperature (gap 4) |
| Quiet window | `plates/collapse.rs:269` | 10 Myr | PLT-259 | A comparison of potential energy with boundary force (gap 1) |
| Collapse time | `params.rs:543` | 15 Myr | PLT-255 (target) | Thermal weakening time (gap 4) |
| Hot-collapse window, off | `params.rs:556,561` | 0 (arm 40 Myr) | PLT-390, PLT-391 | Hides the sea and crust budget (gap 3) |
| Thickness cap | `params.rs:565`; `plates/collapse.rs:140-190` | 70 km | PLT-258 | Root removal and continental subduction (gap 4) |
| Thickening cap | `params.rs:579`; `plates/rate.rs:100-175` | 5 mm/yr | PLT-287 | Guard; excess lost to the mantle |
| Rise cap | `params.rs:583` | 3.5 mm/yr | PLT-288 | Bounds an event the model cannot make (gap 4) |
| Channel flow | `params.rs:570`; `plates/flow.rs:67-80` | K0 1,500 km²/Myr, two rings | PLT-300 (target: one length), PLT-301 | A regional flow solve (gap 1) |
| Belt strength height | `plates/belts.rs:564,720-770` | 5.1 km, 6 passes | PLT-253 | Sideways plateau growth; the overflow is lost (gap 1) |
| Belt geometry | `params.rs:513` | 400 km | PLT-241, PLT-243 | Where the deformation goes (gap 1) |
| Collision stacking | `params.rs:517` | 0.95 | PLT-242 (verified, kept as target) | |
| Cordillera ceiling | `params.rs:674`; `plates/subduction.rs:1097-1100` | 3,000 m | PLT-050 (calibrated) | Arc root spreading and foundering (gaps 1, 4) |
| Cordillera strain | `params.rs:667` | 1.7 mm/yr at every arc | PLT-048, PLT-320 | Shortening set by the force balance |
| Fixed slab geometry | `params.rs:640,642`; `plates/subduction.rs:287,927` | arc axis 200 km, half-width 60 km, reach 800 km | PLT-035, PLT-042, PLT-043 | Slab dip and rollback |
| Slab limits | `plates/forces.rs:101-143`; `params.rs:739,741` | age factor at most 1.5, 700 km, breakoff 10 Myr | FRC-004, FRC-005 (from-memory), PLT-184 | Slab dynamics |
| Drag set | `params.rs:735-763` | speed scale 45 mm/yr; keel 3; slab and boundary resistance 3 | PLT-181, PLT-185, PLT-189 (calibrated to RMS speed) | Mantle viscosity structure |
| Mantle flow | `plates/forces.rs:197-204` | degree 2, 5 mm/yr | FRC-015 | Convection |
| Motion relax | `params.rs:765` | 6 Myr | PLT-191 | Mantle response |
| Collision lifecycle | `plates/shortening.rs:84-104`; `params.rs:530-532` | weld under 5 mm/yr for 10 Myr | PLT-223 to PLT-226, PLT-246 (target) | Suturing set by forces |
| Collision buoyancy | `params.rs:755` | 1.0 | PLT-275 (target) | Continental subduction resistance |
| Rift hazard | `params.rs:621-631`; `plates/rifting.rs:37-86` | 0.006 per Myr, 2 to 4 pieces | PLT-082 to PLT-091 | Stress- and plume-driven rifting (gap 2) |
| Subduction start gates | `plates/rifting.rs:331-337`; `params.rs:712-717` | floor over 150 Myr, 1,000 km clear | PLT-062 to PLT-067 | A reorganisation law (gap 2) |
| Plate size floor | `min_plate_area_frac` | 0.003 | PLT-091 | Resolution: 24 plates against Bird's 52 (gap 2) |
| Margin taper | `params.rs:590,592`; `plates/transition.rs:155` | W 200 km, 20 Myr | PLT-262, PLT-263 | Stretching and cooling (gap 4) |
| Instant settling | `params.rs:574`; `plates/settle.rs:81` | 0.004 Myr | ISO-078 (chosen by dry share) | Thermal buoyancy (gap 4) |
| Hotspots and LIPs | `params.rs:681-699` | 8 at start, 3 per 100 Myr, 0.3 mm/yr | PLT-108 to PLT-120 | Plume dynamics |

Numerical guards: elevation clamp [-10,000, 30,000] m (PLT-052), speed guard 250 mm/yr (PLT-022),
uplift ceiling off (PLT-041), flow substeps (`plates/flow.rs:72,80`), rate slack 1 m. Resampling
every 20 to 120 Myr drops 0.76 to 0.89 km³ of crust a year (PLT-098, PLT-099). That is a
numerical artefact, and it is one of the model's crust sinks.

### surface/, sea level, steering

| Limit | Where | Value | Ledger | Stands in for |
|---|---|---|---|---|
| Steering, land band | `steering.rs` ~137-150; presets `steering` | 0.25 to 0.35, last 10 % of the run | STE-002 to STE-004 (game choice) | A closed loop on M1 |
| Steering, sea offset | same | ±200 m at 2 m/Myr | STE-006 | Mantle water exchange, as dressed (`steering.rs:8-24`) |
| Steering, CO2 | same | factor at most 2, 0.2 per Myr; 10 to 20 degC | STE-005, STE-007 | Climate on target |
| Start: ocean share, shelf share | presets | 0.7, 0.29 | HYP-013, HYP-014 (calibrated) | Initial state |
| Subduction uplift | presets | 0.55 mm/yr | HYP-019, PLT-035 to PLT-039 (calibrated) | Arc and forearc dynamics |
| Stream power K | presets `surface.k_spl` | 1.5e-9 | ERO-004 (calibrated to measured rates) | |
| Threshold slope | presets | 0.15 | ERO-013 | Landsliding at map scale |
| Hillslope κ | presets | 2 m²/yr, 550 times the hillslope value | ERO-023, ERO-177 | Sub-cell relief |
| Marine κ, wave base | presets | 100 m²/yr, 60 m | ERO-049 (calibrated), ERO-045 | Wave and tide transport |
| Mouth fill depth | presets | 0.0 m | ERO-051 | A delta plain (gap 5) |
| Height-limit spreading | `surface/spread.rs:60`; presets | 140 MPa, D0 30,000 km²/Myr, at most 2 | ISO-018 to ISO-024 | Continent-scale spreading above 5.1 km only (gap 1) |
| Flexure rigidity recovery | `surface/flexure.rs:150` | 300 Myr | ISO-051 (uncited) | Lithosphere cooling (gap 4) |
| Glacial erosion caps | `surface/glacial.rs:161-184` | accumulation 2 m/yr, till 100 m | GLA game choices | Ice mass balance |
| Episodic events | `surface/params.rs` (`events.rs` not read) | 200 and 80 m/Myr, 3,000 km² | not read | Rule, not physics |

Numerical guards: flexure solve cell 45 km (ISO-058), surface guard at 12 times Earth's relief
(ISO-072), lake fill 200,000 cells, local isostasy 0.8 over 50 km (zero while flexure is on).

### climate/

| Limit | Where | Value | Ledger | Stands in for |
|---|---|---|---|---|
| Present-day CO2 nudge | `climate/mod.rs:350-405, 1670` | land ice at most 0.15, CO2 factor at most 1.75 | CLM-148 to CLM-151, CLM-206 | The climate on target at the end; hides M6 |
| Atmosphere diffusivity scale | `climate/params.rs:425` | 1.3 | CLM-176 (calibrated to 14.1 degC) | Cloud, vapour and eddy physics |
| Zonal ratio, wind heat transport | `climate/params.rs:426-427` | 0.2, 0.2 | CLM-121, CLM-125 (calibrated) | Stationary eddies |
| Flux correction | `climate/percell.rs:616-618, 721` | at most 60 W/m² | CLM-179 (calibrated) | 1-D against 2-D disagreement |
| Linear OLR, albedos | `climate/params.rs:446-450` | 203.3 + 2.09 T; 0.33, 0.25, 0.62 | CLM-003, CLM-004, CLM-007 to CLM-009 | Clouds and water vapour |
| Ice cap | `climate/params.rs` | 3,000 m | CLM-103 (calibrated) | Ice flow |
| Ice-albedo link | `climate/mod.rs:45` | 1.0 | CLM-025 (calibrated) | |
| Orbit window | `climate/params.rs:519`; `climate/orbit.rs:27` | 3 Myr | CLM-031, CLM-038 | Orbital cycles before |
| Weathering normaliser, supply scale | `climate/carbon.rs:113`; params | 5.147e7; 2.0 | CLM-061, CLM-156 (calibrated) | Lithology and soils |
| Ocean buffer | `climate/carbon.rs:86` | R = 5 | CLM-045 to CLM-080 | Carbonate chemistry |
| Conveyor | `climate/ocean2d.rs:52-78` | 30 Sv, one sinker | CLM-131, CLM-132, CLM-204 | Overturning dynamics |
| Rain table, scale | `climate/belts.rs:109` | 17 Earth values x 1.42 | CLM-178, CLM-201 | Moisture transport |
| Belt latitudes, ITCZ clamp | `climate/belts.rs:31-99` | 30 and 60 degrees; 22 degrees | CLM-178 | Hadley dynamics |
| Lapse rate | `climate/lapse.rs` | 6.5 K/km | MOI-033 | |

### hydro/ and the bake

| Limit | Where | Value | Ledger | Stands in for |
|---|---|---|---|---|
| One receiver per cell | `surface/routing.rs:2-12`; `hydro/network.rs` | D8 | | Splits (gap 5) |
| Width and depth caps | `hydro/mod.rs` ~488-491 | 64 and 12 blocks | HYD-006, HYD-007 | Never bind |
| Smoothing | `hydro/network.rs` ~1543-1570 | 8 blocks | HYD-019 | Meanders (gap 5) |
| River threshold | presets `hydro` | 2 m³/s, 50 km² | HYD-009, HYD-010 | Game choice |
| Dry beach band | | 35 m (one block) | HYD-107 | Game choice |

## Full inventory

Classes: (a) solved as physics, a PDE or a conservation law; (b) a sourced empirical law; (c) a
game-choice rule or a knob tuned to a target; (d) present but off by default; (e) absent.

### Plates, crust and the lithosphere

| Process | What the code does | Class | Where | Ledger |
|---|---|---|---|---|
| Plate driving forces | Torque balance over all plates, solved by Cholesky. Drivers: slab pull, suction, ridge push with high-ground weight, a weak mantle flow. Resistance: basal and keel drag, slab, boundary and collision. Plates relax toward it over 6 Myr | (a) form, (c) coefficients | `plates/forces.rs:1-90`; `plates/mod.rs:565,641` | PLT-181 to PLT-191, FRC-001 to FRC-030 |
| Slab pull | √(age/80) capped at 1.5, zero under 10 Myr, scaled by slab length | (b) | `plates/forces.rs:101-143` | FRC-004, FRC-005, PLT-182 |
| Ridge push, column energy | Half-space column potential energy | (a)/(b) | `plates/forces.rs:169` | FRC-009 to FRC-014, PLT-187 |
| Mantle convection | Fixed degree-2 flow | (c) | `plates/forces.rs:197-204` | FRC-015 |
| Dynamic topography | None | (e) | | |
| Seafloor spreading | Kinematic; 7 km of new crust at resampling; since 2026-10-23 the ridge is cut into 600 km segments normal to the motion, transforms between | (b) | `plates/ridges.rs`: segment_ridges, resample; `plates/pools.rs:150` | PLT-100 to PLT-106, PLT-502 |
| Ocean cooling | GDH1 age-depth | (b) | `plates/pools.rs:292-306` | PLT-350 to PLT-356 |
| Subduction start | Poisson on old floor; since 2026-10-23 shared between plates by their compression and started off the passive margin, at the new arc's reach | (b)/(c) | `plates/rifting.rs`: maybe_initiate_subduction, initiate | PLT-062 to PLT-067, PLT-503, PLT-505 |
| Subduction uplift | Cortial's transfer functions | (b)/(c) | `plates/subduction.rs` | PLT-035 to PLT-060 |
| Arc crust | Fixed volume per km per Myr times min(v/100, 1) | (b) rate, (c) geometry | `plates/subduction.rs:1051-1052` | PLT-042 to PLT-047, PLT-285 |
| Arcs on normal convergence | Built, off | (d) | `plates/subduction.rs:782` | PLT-312 |
| Arc polarity reversal | Since 2026-10-23 on by default: judged by the crust the overriding arc stands on | (b)/(c) | `plates/subduction.rs`: resolve, continent_within | PLT-056, PLT-371 |
| Cordillera | Pure shear at 1.7 mm/yr up to 3,000 m | (b)/(c) | `plates/subduction.rs:1053-1100` | PLT-048, PLT-050, PLT-320 |
| Accretion | Terranes and hidden pools onto the column above | (b)/(c) | `plates/collision.rs`; `plates/pools.rs:191` | PLT-076 to PLT-078, PLT-276 |
| Sediment subduction, subduction erosion | None | (e) | | (plates.md:2537) |
| Collision and suturing | Begins at 10 mm/yr over 3 samples; welds under 5 mm/yr for 10 Myr | (c) | `plates/shortening.rs:84-104` | PLT-222 to PLT-226, PLT-246 |
| Crustal thickening | Mass-conserving, 0.95 kept, over a 400 km belt and a 1.5 degree wedge | (b)/(c) | `plates/belts.rs:383-770` | PLT-241 to PLT-253, PLT-400 |
| Orogenic collapse | Quiet crust over 50 km releases to room on the same plate within 200 km | (c) | `plates/collapse.rs:263-480` | PLT-254 to PLT-259 |
| Hot-orogen collapse | Toward 35.1 km for 40 Myr after orogeny; off | (d) | `plates/collapse.rs:268-286` | PLT-390, PLT-391 |
| Lower-crustal channel flow | Conservative diffusion of thickness above 50 km, two rings, same plate | (a) form, (c) K0 | `plates/flow.rs` | PLT-300, PLT-301 |
| Continent-scale spreading | None | (e) | | |
| Isostasy | Whole-column Airy, one density, eclogite root, instant settling | (a)/(b) | `plates/settle.rs:81`; `plates/collapse.rs:222`; `plates/pools.rs:63-110` | PLT-278 to PLT-282, ISO-043, ISO-044, ISO-078 |
| Thermal isostasy | Built, off: an open fault while the sea is off its datum (2026-10-23) | (d) | `plates/thermal.rs`: uplift_m | ISO-110, ISO-111 |
| Lithosphere temperature, thickness | Since 2026-10-23 on by default: each sample's thermal lithosphere depth and keel, relaxed towards hot or cold targets; heat flow and the strength (thin sheet, Te) from it | (b)/(c) | `plates/thermal.rs` | PLT-460 to PLT-469, ISO-112 |
| Root removal | Since 2026-10-23: a collision belt over 50 km heats to its Moho over 10 Myr; the 70 km cap's no-room excess founders bottom-first at that pace | (b)/(c) | `plates/thermal.rs`; `plates/collapse.rs` | PLT-258, PLT-477, PLT-478 |
| Cratons | Regime label | (e) | `plates/mod.rs:1352` | PLT-139 |
| Rifting | Poisson by continental share and LIP boost, since 2026-10-23 shared between plates by their tension; the pieces grow through the plate and meet where the lithosphere is thin | (c) | `plates/rifting.rs`: maybe_rift, rift | PLT-082 to PLT-091, PLT-504, PLT-506 |
| Rifted margins | Stretching profile (a smoothstep in thickness, since 2026-10-23) W = 200 km, 20 Myr; the lithosphere thins by the same β | (b)/(c) | `plates/transition.rs` | PLT-261 to PLT-264, PLT-294, PLT-475, PLT-476 |
| Plumes, hotspots, LIPs | Random points, swells, piles | (c) | `plates/hotspots.rs` | PLT-108 to PLT-120 |
| Crust budget | Recorded, not enforced | (c) | `plates/crust_budget.rs` | PLT-302, PLT-401 |

### Surface, isostasy, sea level

| Process | What the code does | Class | Where | Ledger |
|---|---|---|---|---|
| Fluvial incision | Braun & Willett implicit stream power with Yuan erosion-deposition, n = 2 | (b) | `surface/spl.rs` | ERO-003 to ERO-013 |
| Hillslope transport | Implicit linear diffusion, κ 2 m²/yr | (b) form, (c) κ | `surface/diffusion.rs` | ERO-023, ERO-177 |
| Marine sedimentation | Submarine diffusion, mouth fill, slope failure at 2.5 degrees | (b)/(c) | `surface/marine.rs` | ERO-045 to ERO-052, ERO-168 |
| Carbonates | Bosscher & Schlager factory | (b) | `surface/carbonate.rs` | ERO-063 to ERO-197 |
| Glacial erosion | Hergarten shallow-ice flux | (b)/(c) | `surface/glacial.rs` | GLA |
| Flexure | Thin elastic plate, variable Te, loads from crust, sediment, water and ice | (a) | `surface/flexure.rs` | ISO-040 to ISO-069 |
| GIA | Elastic equilibrium only | (a) elastic | `surface/flexure.rs` | ISO-064 |
| Height-limit spreading | Height diffusion above 5.1 km | (b) form, (c) D0 | `surface/spread.rs` | ISO-018 to ISO-024 |
| Eustasy | Exact fill of the planet's water less land ice | (a) | `sea_level.rs:88-169` | ISO-085 to ISO-088, CLM-210 |
| Steering | End-weighted controllers on sea offset and outgassing | (c), on in the shipped presets | `steering.rs` | STE-002 to STE-010 |
| Permafrost | None | (e) | | |

### Climate

| Process | What the code does | Class | Where | Ledger |
|---|---|---|---|---|
| Energy balance | Seasonal 2-D per-cell EBM with a 1-D fine clock | (a) balance, (c) diffusivities | `climate/ebm2d.rs`; `climate/ebm.rs` | CLM-001 to CLM-024, CLM-119 to CLM-125 |
| CO2 forcing | Radiative-convective table | (b) | `climate/co2.rs:133-145` | CLM-167 to CLM-172 |
| Faint young Sun | Gough 1981 | (b) | `climate/clock.rs:274-280` | CLM-002, CLM-075 |
| Orbital cycles | Analytic stand-in, last 3 Myr only | (c), (d) earlier | `climate/orbit.rs` | CLM-027 to CLM-040 |
| Carbon cycle | Atmosphere and ocean pool, degassing from ridges, subduction and LIPs | (a) mass balance, (b) fluxes | `climate/carbon.rs` | CLM-045 to CLM-080 |
| Silicate weathering | Runoff, temperature and CO2 law, supply-limited | (b) | `climate/mod.rs:854-954` | CLM-061, CLM-152 to CLM-157 |
| Seafloor weathering | 20 % of Earth's, capped | (b) | `climate/carbon.rs:288-348` | CLM-158 to CLM-162 |
| Organic carbon, carbonates, lithology | None | (e) | `climate/carbon.rs:20` | |
| Sea ice | Latent heat, conduction | (a) | `climate/ebm.rs:97-117` | CLM-092 to CLM-098 |
| Ice-albedo | Smoothstep about -10 degC | (b) | `climate/ebm.rs` | CLM-007 to CLM-012 |
| Ice sheets | Extent from T_ELA, Nye profile; flow off | (b)/(c), (d) flow | `climate/glaciers.rs`; `climate/sheet.rs` | CLM-099 to CLM-118, CLM-147, GLA-067 to GLA-076 |
| Atmospheric heat transport | Diffusion plus wind advection | (a)/(c) | `climate/ebm2d.rs:62-66` | CLM-120 to CLM-125, CLM-176 |
| Ocean heat transport | Stommel gyres, prescribed conveyor | (a) gyres, (c) conveyor | `climate/ocean2d.rs` | CLM-128 to CLM-133 |
| Belts, ITCZ, monsoon regimes | Prescribed | (c) | `climate/belts.rs` | CLM-178, CLM-201 |
| Moisture-budget and upwind rain | Built, off | (d) | `climate/moisture.rs`; `climate/upwind.rs` | CLM-139 to CLM-146, CLM-190 to CLM-200 |
| Orographic rain | Smith & Barstad linear theory | (b) | `climate/orographic.rs` | MOI-034 to MOI-050 |
| Mass-elevation effect | Plateau warmth fit | (b) | `climate/lapse.rs:120-132` | CLM-225 to CLM-231 |
| Clouds, vegetation, other gases | None | (e) | | |
| Present-day nudge | CO2 searched to an ice target | (c) | `climate/mod.rs:350-405` | CLM-148 to CLM-151, CLM-206 |

### Rivers

| Process | What the code does | Class | Where | Ledger |
|---|---|---|---|---|
| Routing | D8 on the filled surface | (a) accumulation | `surface/routing.rs` | |
| Lakes | Priority flood and closed hollows | (a) | `surface/pflood.rs` | HYD-047 to HYD-052 |
| Discharge | Fu-Budyko runoff over real areas | (b) | `hydro/network.rs:222-227` | HYD-011, HYD-034, HYD-035 |
| Channel geometry | Q^0.5 and Q^0.4 in blocks | (b) exponents, (c) coefficients | `hydro/mod.rs:405-407` | HYD-001 to HYD-010 |
| Bed shape | Parabolic cut | (c) | `crates/deeptime-format/src/water.rs:140-185` | HYD-022 |
| Smoothing | Two-pass Gaussian | (c) | `hydro/network.rs` | HYD-019 |
| Distributaries, deltas, floodplains, levees, meanders, braids, fans | None | (e) | | |

## Sources

How each was read for this page. "Record and abstract" means the publisher's record and the
abstract or a summary were read through a web search on 2026-10-22: enough for the claim used
here, not a full read. That is not the ledger's `verified`. "From memory" means not checked.

| Source | Used for | Read |
|---|---|---|
| England, P. & McKenzie, D. 1982, Geophys. J. R. Astron. Soc. 70:295-321 (correction 73:523-532, 1983) | Thin viscous sheet, the Argand number | Record and abstract |
| Houseman, G. & England, P. 1986, J. Geophys. Res. 91(B3):3651-3663; England & Houseman 1986, 91:3664-3676 | n above 3 and Ar 1 to 10 make a plateau | Record and abstract |
| Molnar, P. & Lyon-Caen, H. 1988, GSA Special Paper 218:179-207 | Crests extend while flanks shorten; elastic plates fit foredeeps | Record and abstract |
| Gordon, R. G. 1998, Annu. Rev. Earth Planet. Sci. 26:615-642 | Diffuse boundaries, about 15 % of the surface | Record and abstract |
| Houseman, G., McKenzie, D. & Molnar, P. 1981, J. Geophys. Res. 86(B7):6115-6132 | Convective removal of thickened lithosphere | Record |
| England, P. & Houseman, G. 1989, J. Geophys. Res. 94:17561 | Extension during convergence, Tibet | Record |
| Bird, P. 1979, J. Geophys. Res. 84(B13):7561-7571 | Delamination, uplift with a 30 Myr half-life | Record and abstract |
| Kay, R. W. & Kay, S. M. 1993, Tectonophysics 219:177-189 | Delamination under the Puna | Record and secondary summaries |
| Jordan, T. H. 1978, Nature 274:544-548 | The tectosphere, cratonic keels | Record and abstract |
| McKenzie, D. 1978, Earth Planet. Sci. Lett. 40:25-32 | Rift and margin subsidence | Record |
| Steckler, M. S. & Watts, A. B. 1978, Earth Planet. Sci. Lett. 41:1-13 | Thermal lithosphere 113 to 139 km | Record and abstract |
| Hyndman, R. D. & Currie, C. A. 2011, Geology 39(8):783-786 | Hot and cold regimes, about 1,600 m | Record and abstract; the repo read the full text |
| Hasterok, D. & Chapman, D. S. 2007, J. Geophys. Res. 112, B06414 and B06415 | About 3 km of thermal elevation range | Record and abstract |
| Rey, P. F., Teyssier, C. & Whitney, D. L. 2010, Lithosphere 2(5):328-332 | Channel flow limit, Moho temperature | Record and secondary summaries |
| Clift, P. & Vannucchi, P. 2004, Rev. Geophys. 42, RG2001 | Erosive margins | Record and abstract |
| Stern, R. J. & Scholl, D. W. 2010 | Crust production and loss | The repo's read (PLT-045, PLT-285); not re-checked |
| Bird, P. 2003, Geochem. Geophys. Geosyst. 4(3):1027 | Boundary shares | Record; the shares are the repo's read |
| Forsyth, D. & Uyeda, S. 1975, Geophys. J. R. Astron. Soc. 43:163-200 | Slab forces dominate | Record and abstract |
| Gurnis, M. 1988, Nature 332:695-699 | Supercontinent insulation | Record |
| Hoggard, M. J., White, N. & Al-Attar, D. 2016, Nature Geosci. 9:456-463 | Dynamic topography, about ±500 m at 10⁴ km | Record and abstract |
| Molnar, P., England, P. & Jones, C. H. 2015, J. Geophys. Res. 120:1932-1957 | Flow-induced deflection under about 300 m | Record and abstract |
| Flament, N., Gurnis, M. & Müller, R. D. 2013, Lithosphere 5(2):189-210 | Dynamic topography is transient | Record and abstract |
| Wise, D. U. 1974, in The Geology of Continental Margins, Springer, 45-58 | Freeboard within ±60 m for over 80 % of post-Precambrian time | Record and abstract |
| Baldwin, J. A., Whipple, K. X. & Tucker, G. E. 2003, J. Geophys. Res. 108(B3):2158 | Post-orogenic decay times | Record and abstract |
| Montgomery, D. R. & Brandon, M. T. 2002, Earth Planet. Sci. Lett. 201:481-489 | Erosion rates against relief | Record |
| Walker, J. C. G., Hays, P. B. & Kasting, J. F. 1981, J. Geophys. Res. 86(C10):9776-9782 | Weathering thermostat | Record |
| Gough, D. O. 1981, Solar Phys. 74:21-34 | Faint young Sun | Record and abstract |
| Laskar, J. et al. 2004, Astron. Astrophys. 428:261-285 | Orbital solution, reliable to about 40 to 50 Myr | Record and abstract |
| Budyko, M. I. 1969, Tellus 21:611-619; Sellers, W. D. 1969, J. Appl. Meteorol. 8:392-400 | Energy-balance climate | Record and abstract |
| Leopold, L. B. & Maddock, T. 1953, USGS Prof. Paper 252 | Hydraulic geometry exponents | Record; exponents from the repo's stage-1 read |
| Leopold, L. B. & Wolman, M. G. 1960, GSA Bull. 71:769-794 | Meander wavelength 10 to 14 widths, radius 2 to 3 | Record and secondary summaries |
| Slingerland, R. & Smith, N. D. 2004, Annu. Rev. Earth Planet. Sci. 32:257-285 | Avulsion recurrence 28 to 1,400 years | Record and abstract |
| Ganti, V. et al. 2014, Geophys. Res. Lett. 41:7882-7890 | Avulsion near the backwater length on deltas, at the slope break on fans | Record and secondary summaries |
| Jerolmack, D. J. & Mohrig, D. 2007, Geology 35:463-466 | Channel mobility and planform | Record only; content from memory |
| Syvitski, J. P. M. & Saito, Y. 2007, Glob. Planet. Change 57:261-282 | Delta area against discharge, load and room | Record and abstract |
| Paola, C. et al. 2011, Annu. Rev. Mar. Sci. 3:67-91 | Delta lobes and growth | Record and abstract |
| Blair, T. C. & McPherson, J. G. 1994, J. Sediment. Res. 64(3a):450-489 | Alluvial fans | Record and abstract |
| Milliman, J. D. & Farnsworth, K. L. 2011, River Discharge to the Coastal Ocean, Cambridge | Global river data | Record only; the Amazon and Congo figures are from memory |
| Turcotte & Schubert, Geodynamics; Allen & Allen, Basin Analysis; Burbank & Anderson, Tectonic Geomorphology; Pierrehumbert, Principles of Planetary Climate; Hartmann, Global Physical Climatology; Cuffey & Paterson, The Physics of Glaciers | The reference list of mechanisms | From memory; not opened |

## What this audit did not do

- It ran nothing. Every model figure is from the build log, the science pages or the code
  constants. Arithmetic marked as such is this page's.
- Not read in full: `plates/subduction.rs` lines 1 to 940, `plates/forces.rs` 520 to 770,
  `surface/events.rs`, `surface/lithology.rs`, `surface/weathering.rs`, `climate/moisture.rs`,
  `climate/upwind.rs`, the Java `Rivers.java` and `WaterBodies`.
- The river findings on navigability are inferences. A separate agent is measuring the D5 world's
  rivers and mouths.
- Stale text found: the PLT-190 and PLT-191 ledger rows say the forces are "recorded and not
  applied", but `plates/mod.rs:565,641` applies them. The HYD ledger header and HYD-048 say 20 m per
  block vertically; the presets use 35. sea-level.md:696 says "no hotspot swells", while PLT-111 and
  PLT-112 build local ones.
