# Meteor impacts

Status of this page: `stage 1 (written from the ledger; one source read, one error found)`,
2026-10-02.

Ledger ids on this page: `IMP-001` to `IMP-141` ([ledger/IMP.md](ledger/IMP.md)). Of these 141
rows, 47 are `game-choice`, 43 `uncited`, 34 `from-memory`, 14 `cited-unverified` and 3
`abstract-only`.

**How far this has been checked.** The agent that wrote the impact code could not fetch its main
source (Collins, Melosh & Marcus 2005) and wrote the crater-scaling constants from memory, saying
so in the code and the build log. In stage 1 that paper was fetched and read. Ten of the constants
match it. One does not: the depth of complex craters (IMP-042, below). Three equation numbers cited
in code comments are wrong although their values are right. The other sources (the impact rate, the
peak-ring and central-peak sizes, the diffusivity) have still not been read.

Scale used throughout: one block is 300 m across and 20 m up.

## What you see in the world

Craters are rare. On the default planet, at Earth's rate of impacts, a 500 Myr history made 567
craters, of which 71 still existed at the end and 19 could be seen as landforms, 7 of them as dry
bowls on land. A player may never meet one. The impact rate is a setting.

The smallest crater is 2 km across (7 blocks). Up to 3.2 km (11 blocks) a crater is a simple bowl
with a raised rim. Larger craters are "complex": a flat floor, slumped walls and a central peak
about a fifth of the crater's width across. From about 36 km (120 blocks) the central peak gives way
to a ring of peaks halfway to the rim. The largest crater a run may make is 300 km (1,000 blocks).

Craters are drawn at half their true depth by default, because at 15 times vertical stretch a true
2 km bowl would be 7 blocks across and 21 deep. At the default, a 2 km crater is about 11 blocks
deep when fresh and a 20 km crater about 18.

Under the floor lies impact breccia (rock shattered and mixed by the impact; cobblestone until a
pack supplies a block), and in larger craters a sheet of impact melt rock on the floor (smooth
basalt). A thin blanket of breccia lies around the rim.

Craters age. Soil creep rounds the rim and shallows the bowl, rivers cut the rim, sediment fills
the floor, and a lake may stand in it. An old crater may show nothing at the surface and survive
only as a lens of breccia in the rock column under later layers. Craters on ocean floor are carried
into trenches with their plate. The oldest crater still visible in the measured run was 288 Myr old.

Four kinds of ore go with craters: nickel, copper and platinum-group metals under very thick melt
sheets (as at Sudbury, Canada); zinc, copper and gold veins from hot water circulating in the
shattered rock; diamonds formed by shock where the target held carbon; and uranium in the sediment
that fills a crater basin. All four are too rare to be a world's only source of those materials.

## The model in plain words

The impact stage runs once per plate step, after the volcano stage.

1. **How many.** The expected number of impacts in a step is Earth's cratering rate per square
   kilometre per year (IMP-001), scaled to the smallest crater simulated with a power law of size
   (IMP-003), times the planet's own area, the step length and the rate setting. The actual number
   is drawn from a Poisson distribution, the standard distribution for counts of rare independent
   events (IMP-009).
2. **Where and how large.** Each impact lands at a point chosen evenly over the sphere (IMP-010).
   Its size is drawn from the same power law, between 2 km and 300 km, and is read as the crater the
   body would make on Earth's land (IMP-011, IMP-012).
3. **From crater to impactor.** That Earth crater is turned back into an impactor diameter by
   inverting the scaling law at Earth's gravity, for a rocky body of 3,000 kg/m³ arriving at 20 km/s
   at 45 degrees (IMP-020, IMP-021, IMP-024, IMP-029). Every impactor has that same speed, density
   and angle.
4. **The sea.** If the point is under water, the crater is shrunk by a factor that falls in a
   straight line with the ratio of water depth to impactor diameter, reaching zero at 6.3: a body
   less than about a sixth as wide as the water is deep leaves no crater in the sea floor (IMP-065,
   IMP-066).
5. **From impactor to crater on this planet.** The transient crater, the bowl just after
   excavation and before its walls collapse, comes from the scaling law at this planet's gravity
   (IMP-030). The final crater is 1.25 times wider if simple, and wider still by a power law if
   complex (IMP-031, IMP-032). The change from simple to complex is at 3.2 km on Earth and moves
   inversely with gravity (IMP-034, IMP-036).
6. **Depth, rim, breccia, melt.** Rim height, the volume and thickness of the breccia lens, the
   depth and the melt volume follow from further laws of the same paper (IMP-037 to IMP-042,
   IMP-062, IMP-063). Everything vertical is then multiplied by the depth setting (IMP-130).
7. **Into the world.** A crater at least two coarse grid cells across (9.6 km on the default
   planet) is cut into the terrain the deep-time simulation erodes, and its breccia and melt are
   written into the rock record. A smaller one is drawn only at block scale, with its breccia as a
   separate body (IMP-132, IMP-093, IMP-094). A new crater destroys smaller ones inside it (IMP-060).
8. **Ageing.** Each later step, the crater turns with its plate. Sediment laid inside the rim adds
   to its cover; erosion strips the cover and then wears the crater (IMP-078). Soil creep is
   modelled as a blur that widens with time: the profile is smoothed by a width of √(12 κ t), with
   κ = 0.01 m²/yr times a factor for the rock (IMP-072 to IMP-075). A crater worn through its depth
   and its breccia is removed (IMP-079).
9. **Deposits.** For each crater the stage records melt thickness, a measure of hot-water
   circulation, a measure of shock, and whether a basin remains; the four impact deposit types read
   these (IMP-097 to IMP-102).
10. **Block scale.** The amplifier draws each surviving crater's aged profile, times the share of
    its relief that is left, after the last flooding step, so that the bake's own water decides
    whether a bowl holds a lake (IMP-128 to IMP-135).

## Equations and constants

Symbols: `L` impactor diameter (m), `ρi` and `ρt` impactor and target density (kg/m³), `v` speed
(m/s), `g` surface gravity (m/s²), `θ` angle from the horizontal, `D_tc` transient crater diameter,
`D` final rim-to-rim diameter, `D_c` the simple-to-complex transition diameter, `E` impact energy
(J).

The "Stage 1 check" column gives what was found when Collins, Melosh & Marcus 2005 was read (full
text, open copy in Imperial College's repository; the depth law was read by the stage 1 lead, the
rest by a sub-agent that quoted the passages into [bibliography.md](bibliography.md)). A match
there does not yet change the ledger status: stage 2 confirms and marks rows `verified`.

| Ledger id | Quantity | As coded | Source cited | Status | Stage 1 check |
|---|---|---|---|---|---|
| IMP-001 | Earth's cratering rate, craters ≥ 20 km | 5.6 × 10⁻¹⁵ per km² per year | Grieve & Shoemaker 1994 | `cited-unverified` | not read (book chapter). Grieve 1987 gives an older 5.4 ± 2.7 × 10⁻¹⁵ |
| IMP-003 | Size distribution | N(≥ D) ∝ D^−1.8 | Grieve & Shoemaker 1994 | `cited-unverified` | supported above about 20 km by Grieve 1987; used down to 2 km on the code's own authority (IMP-004) |
| IMP-020 | Impactor speed | 20 km/s | Collins et al. 2005 | `from-memory` | partial: 20 km/s is the top of the paper's 12 to 20 km/s range for asteroids; the paper gives no average |
| IMP-021 | Impactor density | 3,000 kg/m³ | none | `uncited` | at the top of the paper's 2,000 to 3,000 for stony bodies |
| IMP-023 | Target density | 2,700 kg/m³ | Collins et al. 2005 | `from-memory` | partial: the paper uses 2,500 (sedimentary) and 2,750 (crystalline) on land |
| IMP-024 | Impact angle | 45° | Gilbert 1893; Shoemaker 1962 | `from-memory` | matches the paper's statement of the most probable angle |
| IMP-030 | Transient crater diameter | D_tc = 1.161 (ρi/ρt)^(1/3) L^0.78 v^0.44 g^−0.22 sin^(1/3) θ | Collins et al. 2005, eq. 21 | `from-memory` | match. The paper says the law holds for impactors larger than "a couple of hundred meters"; the code uses it down to about 84 m |
| IMP-031 | Final diameter, simple crater | D = 1.25 D_tc | eq. 22 | `from-memory` | match |
| IMP-032 | Final diameter, complex crater | D = 1.17 D_tc^1.13 / D_c^0.13 | eq. 27 | `from-memory` | match |
| IMP-033 | Which of the two applies | the larger of the two | none | `uncited` | differs: the paper switches to the complex law above D_tc = 2.56 km |
| IMP-034 | Transition diameter on Earth | D_c = 3.2 km | Grieve 1987 | `from-memory` | partial: 3.2 km is Collins et al.'s value; Grieve 1987 gives about 2 km (sedimentary) and 4 km (crystalline) |
| IMP-036 | Transition against gravity | D_c ∝ 1/g | Pike 1980 | `from-memory` | not read |
| IMP-037 | Transient crater depth | D_tc / (2√2) | – | `from-memory` | match (the paper's eq. 25) |
| IMP-038 | Rim height | 0.07 D_tc⁴ / D³ | "eq. 25" | `from-memory` | value matches; it is the paper's eq. 48 |
| IMP-039, IMP-040 | Breccia lens volume and thickness | 0.032 D³; 2.8 V (d_tc + h) / (d_tc D²) | eqs. 24, 26 | `from-memory` | match (volume is eq. 23) |
| **IMP-042** | **Depth of a complex crater** | **0.294 D^0.301 km** (D in km) | **eq. 28** | `from-memory` | **mismatch: the paper's eq. 28 is 0.4 D^0.3 km** |
| IMP-043 | Cap on that depth | at most 0.2 D | none | `uncited` | not in the paper |
| IMP-045 | Peak rings begin | 40 km × 8.87 / g (36 km on Earth) | Alexopoulos & McKinnon 1994 | `from-memory` | not read (access refused) |
| IMP-048 | Central peak width | 0.22 D | Pike 1985 | `from-memory` | not read (scan without text) |
| IMP-049 | Peak height | 0.5 of floor depth; 0.35 for a ring | "design value" | `game-choice` | – |
| IMP-054 | Ejecta thickness at distance r | D_tc⁴ / (112 r³) | "eq. 32" | `from-memory` | value matches; it is the paper's eq. 47 |
| IMP-062 | Melt volume | 8.9 × 10⁻¹² E sin θ m³ | "eq. 12" | `from-memory` | value matches; it is the paper's eq. 30 |
| IMP-063 | Melt sheet thickness | 4 V_melt / (π D_tc²) | eq. 31 | `from-memory` | match |
| IMP-065, IMP-066 | Crater size under water | factor max(0, 1 − (depth / L) / 6.3) | Davison & Collins 2007 | `abstract-only` | partial: the paper (read in full in stage 1) finds no crater beyond a ratio of about 8; 6.3 is the code's own straight-line fit to three summary numbers |
| IMP-067 | What the fit was aimed at | the sea removes two thirds of craters under 1 km, a third at 30 km, few above 100 km | Davison & Collins 2007 | `abstract-only` | the three numbers match the paper |
| IMP-072, IMP-073 | Ageing by soil creep | a blur of variance 2 κ t | Culling 1960; Fassett & Thomson 2014 | `from-memory` | not read |
| IMP-074 | Diffusivity κ | 0.01 m²/yr × rock factor | Fernandes & Dietrich 1997 and others | `from-memory` | not read. The one diffusion paper read, Fassett & Thomson 2014, is lunar: 5.5 m²/Myr, about 1,800 times lower, so it does not test the coded value |
| IMP-130 | Depth setting | 0.5 | Ben, decisions log 2026-09-30 (the volcano precedent) | `game-choice` | – |
| IMP-013, IMP-015 | Size limits | 2 km to 300 km | – | `game-choice` | – |

A check of the whole chain: Collins et al.'s Table 6 gives final craters of 1.2, 23.7 and 186 km for
three worked examples (a 40 m iron body, and stony bodies of 1.75 km and 18 km). The coded laws give
about 1.18, 23.8 and 185.6 km (a sub-agent's arithmetic, not a test in the repository).

The rows for impact rock (IMP-084 to IMP-096), the four deposit types (IMP-097 to IMP-127) and the
block-scale shape (IMP-128 to IMP-136) are in the ledger file. The deposit sizes, grades and
densities there are placeholders: the data file says "design values: no USGS model exists".

## Earth against the mod

The build log compares the model's crater population with nothing on Earth. There is no check
against the list of Earth's confirmed impact structures. What was measured is one run:

Run: `deeptime run --preset earthlike_quick_16k --start scattered_continents --length-myr 500
--threads 4`, 2026-10-01, on the impacts branch before folds and faults were merged; seed not
stated in the build log (the preset's is 1). Planet area 7.69 million km², about 1.5 % of Earth's.

| | under 5 km | 5–10 | 10–20 | 20–50 | 50–100 | over 100 km | all |
|---|---|---|---|---|---|---|---|
| craters formed | 448 | 81 | 28 | 5 | 4 | 1 | 567 |
| of them in the sea floor | 134 | 22 | 16 | 0 | 3 | 1 | 176 |
| impactors the sea stopped | 637 | 89 | 9 | 0 | 0 | 0 | 735 |
| eroded through | 354 | 62 | 16 | 5 | 1 | 1 | 439 |
| subducted | 38 | 9 | 8 | 0 | 2 | 0 | 57 |
| at the present: fresh | 6 | 1 | 2 | 0 | 0 | 0 | 9 |
| degraded | 7 | 2 | 0 | 0 | 1 | 0 | 10 |
| buried | 28 | 5 | 0 | 0 | 0 | 0 | 33 |
| eroded to their roots | 15 | 2 | 2 | 0 | 0 | 0 | 19 |

Other figures from that run:

| Quantity | Earth | Source of the Earth figure | The mod |
|---|---|---|---|
| Impactors in 500 Myr | – | – | 1,302 drawn, 1,359 expected |
| Share stopped by the sea | – | – | 735 of 1,302 (56 %) |
| Craters ≥ 20 km formed | one per 350,000 years on all of Earth, from the coded rate (our arithmetic) | Grieve & Shoemaker 1994, not read | one per 23 Myr expected on this planet; 10 formed in 500 Myr |
| Largest crater | Vredefort, quoted as 170 to 300 km | Wikipedia (secondary); the code says 300 km with no source | 112 km (eroded away by the end) |
| Oldest crater that still shows | – | – | 288 Myr |
| Impact deposits surviving | – | – | 10 hydrothermal, 3 basin uranium, 0 diamond, 0 nickel |
| A 2.1 km crater, 5 Myr old | – | – | 7 blocks across, 6 deep |
| A 4.9 km crater, 40 Myr old | – | – | 3.6 blocks deep, with a small lake |

## Simplifications and limits

- **One kind of impactor.** Every body is rocky, arrives at 20 km/s and strikes at 45 degrees.
  There are no iron bodies and no comets, and no spread of speeds or angles.
- **One kind of target.** The same crust density and the same 3.2 km transition are used for all
  rock, though the code's own comment quotes 2 km for sedimentary and 4 km for crystalline targets.
- **A constant rate.** Earth's present-era rate is used for the whole history, with no early heavy
  bombardment. The power law measured for craters above 20 km is used down to 2 km.
- **Earth's rate on a small planet.** The rate per area is Earth's; the planet has 1.5 % of
  Earth's area, so it collects 1.5 % of Earth's impacts. Crater sizes are not scaled down.
- **Size limits.** Nothing under 2 km (it could not be drawn) and nothing over 300 km (a larger
  impact would resurface much of so small a planet, which is not simulated).
- **The sea rule is a straight line** fitted to three numbers from a paper's summary, not the
  paper's curves, and water depth is read from a grid with 4.8 km cells.
- **Craters are round.** No oblique shapes, no secondary craters, no rays.
- **Ageing is a blur plus a budget.** Soil creep is a smoothing filter; river erosion and
  sedimentation enter only as their average inside the rim.
- **No effect on anything else.** An impact does nothing to the climate or to life, and craters are
  not deformed by later folding.
- **Small craters make no ore.** Impact deposits form only where the breccia is in the coarse rock
  record, so not in craters under 9.6 km on the default planet, although one deposit type's note
  says "from about 8 km".
- **Impact ore is invented.** No published grade and tonnage model exists for impact deposits. The
  sizes, grades and numbers are placeholders.

## Game choices

| What | Why | Who and when |
|---|---|---|
| Impacts exist, at a rate that is a setting with Earth's as 1 (IMP-006) | wanted "in full", before phase 3 | Ben, decisions log 2026-09-29; started 2026-10-01 |
| Craters drawn at half depth (IMP-130) | a true bowl is a shaft at 15 times vertical stretch | agent, following Ben's 2026-09-30 answer for volcano flanks; **not yet confirmed by Ben** |
| Smallest crater 2 km, largest 300 km (IMP-013, IMP-015) | 2 km is 7 blocks; above 300 km the planet-wide effects are not modelled | agent, build log 2026-10-01; the cap is **an open question for Ben** |
| Default rate stays at Earth's although few craters are then seen (IMP-140) | the decision asked for Earth's rate | agent; **an open question for Ben** (about 5 times was offered) |
| Peak heights (IMP-049), flat-floor radius 0.55 (IMP-051), ejecta drawn to 3 radii (IMP-056) | shape values chosen for looks within observed ranges | agent, build log 2026-10-01 |
| "Fresh" means 75 % of depth left; "shows" means 40 m of relief, two blocks (IMP-080, IMP-081) | thresholds for the catalogue and the map | agent |
| Stand-in blocks: cobblestone and smooth basalt (IMP-092) | no impact rock blocks exist yet | agent; **an open question for Ben** |
| Impact deposits do not replace the pack's scattered diamond, nickel, platinum and uranium ore (IMP-126) | impact deposits are too rare to be the only source | agent; **an open question for Ben** |
| All tonnages, grades and densities of impact deposits (IMP-105 to IMP-125) | no model exists | agent, marked as placeholders in the data |

## Open questions

1. **IMP-042 is wrong against its own citation.** The code computes the depth of a complex crater
   as 0.294 D^0.301 km and cites Collins et al. 2005, eq. 28. That equation reads
   `d_fr = 0.4 D_fr^0.3` (both in km), taken from Herrick et al. 1997 for craters on Venus. The
   coded law makes every complex crater about 27 % shallower (0.72 km against 0.98 km at 20 km).
   Where 0.294 and 0.301 come from was not traced. The coding agent had flagged exactly this doubt.
   Not fixed in stage 1 (no code changes); first item of the verification plan.
2. **A step in depth at the transition.** At 3.2 km a simple crater is about 640 m deep by the
   simple law and about 417 m by the complex law, so depth drops by a third across the transition
   (the extraction agent's arithmetic). The comment says the complex law "is continued from" the
   simple one; the code takes a minimum. With the paper's 0.4 D^0.3 the complex depth at 3.2 km would
   be about 570 m, a smaller step. To be re-examined with item 1.
3. **Which diameter law applies near the transition** differs from the paper (IMP-033).
4. **Wrong equation numbers** in comments for rim height, melt and ejecta (IMP-038, IMP-062,
   IMP-054).
5. **The rate has not been read in its source.** Grieve & Shoemaker 1994 is a book chapter; the
   coded 5.6 × 10⁻¹⁵ is close to an older published 5.4 ± 2.7 × 10⁻¹⁵ but is not confirmed.
6. **An editor hint is wrong by a factor of a million.** It says "one crater of 20 km or more per
   180 million km² per year"; the coded rate is one per 1.8 × 10¹⁴ km² per year (IMP-018).
7. **Speed.** 20 km/s is the top of the cited range, not an average (IMP-020). Le Feuvre &
   Wieczorek 2011, also cited, was not readable.
8. **The scaling law is used below its stated range**: the smallest craters come from impactors of
   about 84 m, where the paper says "a couple of hundred meters" (IMP-030), and where the atmosphere
   matters.
9. **The diffusivity** of 0.01 m²/yr and the fade times that follow from it (a 2 km bowl in about
   10 Myr) rest on sources not yet read (IMP-074, IMP-083).
10. **The nickel deposit's note disagrees with its own numbers**: it says a crater of "about 170 km
    and more" is needed, but the coded melt law reaches the 500 m threshold at about 107 km (the
    extraction agent's arithmetic; IMP-105, IMP-112).
11. **Peak rings from 36 km on Earth** is scaled from Venus; whether that fits Earth's craters was
    not checked (IMP-045).
12. **Possible double counting of erosion** for craters cut into the coarse terrain (extraction
    notes, flag 17): a suspicion from reading the code, not tested.
13. **No comparison with Earth's crater record** exists. Stage 2 should compare the size
    distribution and survival against the Earth Impact Database.

## Sources

Keys refer to [bibliography.md](bibliography.md).

| Key | Used for | How it has been read |
|---|---|---|
| Collins2005 | crater scaling, melt, ejecta (IMP-020 to IMP-063) | written from memory by the coding agent; full text read in stage 1 |
| DavisonCollins2007 | craters under water (IMP-065 to IMP-067) | abstract only by the coding agent; full text read in stage 1 |
| Grieve1987 | transition diameters; an older rate | full text (scan) read in stage 1 |
| GrieveShoemaker1994 | the rate and the size slope | not read (book chapter) |
| Herrick1997 | origin of the paper's depth law | not read |
| Pike1980, Pike1985, AlexopoulosMcKinnon1994 | transition against gravity; central peaks; peak rings | not read |
| Culling1960, Fernandes1997, Roering1999 | hillslope diffusion | not read |
| FassettThomson2014 | crater ageing by diffusion (on the Moon) | abstract read in stage 1 |
| Neukum2001, Bland2006, LeFeuvre2011 | constancy of the rate; atmospheric break-up; speeds | not read |
| Naldrett2004, Masaitis1998, GrieveMasaitis1994, Abramov2004, Osinski2013 and others | impact deposits | not read |

Extraction notes, with 22 flags: [extraction-notes/IMP.md](extraction-notes/IMP.md). Build log:
STATUS.md, "Meteor impacts (P2-08)" (2026-10-01).
