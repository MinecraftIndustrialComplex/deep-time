# Run review: workbench run `1a0f969d358`

Scientific review of one simulated history: `earthlike_quick_16k`, supercontinent start, 1000 Myr,
2 Myr steps, seed 1, made on master about 044c112 / e6a7af0 on 2026-10-01. Reviewed 2026-10-01.
The question was Ben's: is what happens in the timelapse video accurate?

Nothing in the simulation, the presets or the run was changed for this review.

## Two-minute summary

**Verdict.** The video is a convincing *cartoon* of plate tectonics, not yet a simulation a
geologist would call accurate. The planet-wide numbers are good. The behaviour is not: the things a
geologist watches for in a timelapse mostly do not happen the way they do on Earth.

**What is right** (measured):
- Plates move at Earth-like speeds: median 52 mm/yr between plates, Earth about 40.
- Continental crust is conserved: 37–45 % of the planet all run, Earth about 40 %.
- Land is 21–39 % of the planet, Earth 29 %.
- The sea floor's median age is 68 Myr, Earth 64 Myr.
- Height statistics are Earth-like: two peaks (land just above the sea, ocean floor at 3–5 km depth),
  11 % of land above 2 km.
- Ridges, trenches, island arcs, hotspot chains and flood basalts exist and sit where they should.
- The picture is clean: no seam, no grid, no flicker.

**What is wrong** (measured, most visible first):
1. **The ice is a ruled white band.** Sea ice ends at one latitude all round the planet, in 2° steps,
   and never goes away in 1000 Myr. It fills the top and bottom third of the picture. Known, being
   fixed (P1-26).
2. **Continents do not keep their shapes.** They fray into ribbons and hundreds of specks. By
   about 500 Myr ago nothing recognisable from the start is left.
3. **There is no real Wilson cycle.** A piece that rifts off is welded back on after a median of
   18 Myr. Oceans rarely get to open and close.
4. **Every collision welds two plates into one.** For half the run a single plate covers more than
   half the planet (98 % at worst). Earth's largest plate is 20 %.
5. **Collisions pop land out of the sea in one frame.** A collision lifts a round area up to 15 % of
   the planet by up to 2 km in a single 2 Myr step. Mountains are round blobs, not long belts.

**How a geologist would react.** "Nice rendering, the right ingredients, the right speeds. But the
continents are dissolving, the mountains appear in blobs, and why is there a white stripe?" The first
thing they would call out is the ice band. The second is the continents.

**Was it accurate?** In its ingredients and its global numbers, yes. In what the continents, the
plates and the ice do over time, no. It is fair to send the video as "a work-in-progress procedural
planet with real plate kinematics". It is not fair to call it an accurate tectonic simulation yet.

## How this was measured

Three kinds of statement are kept apart below.
- **Measured**: computed from the run's data.
  - *Frames*: all 500 plate-model keyframes (`world/keyframes/kf-*.dtk`, 1024 × 512, one per
    2 Myr), each layer's colours turned back into values with `ramps.json`, weighted by cos(latitude)
    so areas are true areas. Elevation is read from the `elevation` layer (20 m per block), so heights
    above about 5 km are coarse.
  - *Events*: the run's 7,219 events (`events.jsonl`).
  - *Status lines*: the run's own final statistics.
- **Seen**: looked at in the video's frames (`review/out/runs/1a0f969d358/stamped/`) or in layer
  images from the workbench.
- **Inferred**: my reading of the two.

**Scale.** The planet is small (radius 782 km), but the plate model runs as if on an Earth-sized
sphere (`plates/normalise.rs`). Speeds in events are already Earth-normalised. Areas below are given
as a share of the planet, which needs no conversion. "Earth-equivalent" means multiplied back up to
Earth's size.

**One gap.** A last script that counted the mega-plate intervals and the land jumps exactly failed
after I had deleted its inputs. Those two counts below come from the 50-Myr samples and the top-of-list
values I already had, and say so.

**Frames looked at.** Video: 0000, 0010, 0025, 0050, 0075, 0100, 0125, 0150, 0175, 0193, 0194, 0200,
0225, 0250, 0275, 0300, 0325, 0350, 0375, 0400, 0425, 0450, 0475, 0483, 0484, 0485, 0498, 0499.
Workbench layers: plates, crust age and regime of frames 250, 430 and 499; elevation, volcanoes,
Köppen, rainfall and rivers of frame 499.

## 1. Plate kinematics

**Speeds: realistic.** (Measured, events.)

| | n | median | 90th pct | max | Earth |
|---|---|---|---|---|---|
| Convergence at new subduction zones | 786 | 52 mm/yr | 106 | 162 | full rates 20–150; limit about 200 |
| Spreading at new ridges | 737 | 52 | 106 | 167 | East Pacific Rise about 150 |
| Convergence at collisions | 107 | 51 | 89 | 139 | India–Eurasia 50 now, up to 190 at 55 Ma |
| A new plate's own speed | 165 | 65 | 95 | 100 (the cap) | RMS about 40; fastest plates about 85 |

- These are a little fast, about 1.3–1.5 × Earth's typical values, and inside Earth's limits.
- Speeds are steady through the run: the median per 100 Myr stays between 45 and 74 mm/yr.
- Hotspot tracks give the same answer independently: 35 tracks, median 4,750 km (Earth-equivalent) in
  78 Myr, so 63 mm/yr.

**Plate sizes: wrong.** (Measured, frames and events.)
- The largest plate covers more than 30 % of the planet in 80 % of frames, more than 50 % in 51 %,
  and more than 70 % in 26 %. The worst is 98 % at 862 Myr ago.
- Earth's largest plate, the Pacific, is 20 %. Earth has seven plates above 5 % of its area (the
  seven-plate count is sourced; the 5 % reading is from memory). This run has a median of three.
- Rift events confirm it: 14 of the 29 rifts split a plate covering more than half the planet.
- Seen in the plates layer of frame 430 (138 Myr ago): one plate fills the map and the others sit
  inside it like islands.
- The number of plates looks fine on its own: 7–22 alive, 20 at the end.

**Plate turnover: too fast.** (Measured, events.)
- 165 plates are born and 157 die in 1000 Myr.
- Median lifetime is 32 Myr. 39 % die within 20 Myr.
- Earth's large plates last hundreds of Myr. I have no sourced figure for Earth's turnover rate.

**Does motion look continuous?** (Measured and seen.)
- Mostly yes. Continents slide smoothly; the mean picture change per frame is even through the run.
- The jumps are not in the motion. They are land appearing at collisions (section 5).
- When plates weld, 60–77 % of the planet changes plate in one step. That shows only in the plates
  layer, not in the video.

**Inferred.** The mega-plate comes from the P1-27 rule that a colliding continent "sutures its whole
plate to the receiving one". With about 10 collisions per 100 Myr, plates merge faster than rifts
split them. While one plate holds most of the continents, they cannot move relative to each other.

## 2. The supercontinent cycle

**It rifts and disperses: yes.** (Seen: 0000–0200. Measured, frames.)
- The supercontinent splits into three at 998 Myr ago, along narrow seaways.
- The largest connected piece of continental crust falls from 100 % to 26–40 % of all continental
  crust between 640 and 380 Myr ago.

**It reassembles: only on paper.** (Measured and seen.)
- From 260 to 100 Myr ago 70–94 % of the continental crust is again one connected piece.
- It does not look like a supercontinent. The largest *landmass* then holds only 26–57 % of the land.
  The connected crust is a web of ribbons with flooded gaps (seen: 0375, 0400, 0425, 0450).
- Timescale: together to 640 Myr ago, apart 640–380, together again from about 360. That is one
  cycle in roughly 650 Myr. Earth's is quoted from about 400 to 800 Myr, so the period is plausible.

**The sequence is not a Wilson cycle.** (Measured, events.)
- Of 65 plates born by rifting, 59 died, all by being welded onto another plate.
- Their median lifetime was 18 Myr. 56 % were welded back within 20 Myr and 83 % within 50 Myr.
- 12 were welded back onto the very plate they rifted from.
- The first rift shows it: fragment 14 leaves at 998 Myr ago and is welded back at 978.
- On Earth an ocean opens for 100–200 Myr or more before it closes (the Atlantic has been opening
  for about 180 Myr).
- Rifts happen at a steady 2–5 per 100 Myr. Collisions are 107 in 1000 Myr (known issue).

**Good detail.** 14 of the 17 flood-basalt events are followed by a rift within 20 Myr. That link is
real on Earth (the Central Atlantic province and the opening of the Atlantic).

## 3. Continents

**Shape: they shred.** (Measured, frames. Seen: 0200 onward; crust-age layer of frames 430 and 499.)
- Share of continental pixels that lie on an edge: 1 % at the start, 12 % at 800 Myr ago, 23 % at
  500, 29 % at the end. It rises all run and never recovers.
- Separate pieces of continental crust: 1 at the start, about 300 by 800 Myr ago, 440–580 after
  that. The pieces smaller than 0.1 % of the planet hold 4–8 % of all continental crust.
- In the crust-age layer the continents are moth-eaten: grey continental cells and old blue oceanic
  cells mixed like salt and pepper, with no crisp margin.
- In the video this reads as continents turning into noodles and island swarms. The starting
  supercontinent's outline is unrecognisable by about 600 Myr ago.
- P1-27 ("coherent continents") has landed and crust spreading is in progress, so this is partly
  known. The measurement says it is still the second most visible problem.

**Amount: conserved.** (Measured, frames.)
- Continental crust is 42.3 % of the planet at the start and 39.6 % at the end, between 37.3 % and
  45.4 % throughout. Earth is about 40 % (from memory).
- No single step changes it by more than 1 % of the planet.

**Land fraction: Earth-like, drifting low.** (Measured, frames and status.)
- 33 % at the start, 21–39 % through the run, 24 % at the end (the run's own figure is 22.1 %).
  Earth is 29 %.
- The reason it drifts low: the share of continental crust under water rises from 14–25 % early to
  44 % at the end, with a peak of 52 %. Earth's is about 27 % (derived: 29 % land on 40 % crust).

**Coastlines.** (Seen.) Smooth and rounded at the scale of the video, with a turquoise shelf all
round. No fjords, deltas or drowned valleys at this resolution, which is fair for a 1024-pixel map.

## 4. Oceans

**Age: the middle is right, the old tail is too fat.** (Measured, frames and status.)

| | this run (typical) | Earth |
|---|---|---|
| Median age | 68 Myr | mean 64 Myr |
| 90th percentile | 189 Myr | oldest large areas 180–200 Myr |
| Share older than 180 Myr | 12 % (up to 22 %) | close to none |
| Share older than 250 Myr | 3 % (up to 10 %) | one small basin, about 340 Myr |
| 99th percentile | 300 Myr (up to 392) | |

- The run's own gate, `ocean_younger_250`, reads 0.936.
- Inferred: old floor riding on the mega-plate has no trench to go down.

**Ridges and trenches: good.** (Seen: crust-age and regime layers, frames 430 and 499.)
- Ridges are long lines with floor ageing evenly on both sides, as on real sea-floor age maps.
- Trenches show in the video as dark lines beside island chains (0010, 0025, 0050).
- Two things look wrong in the layers, not in the video. Ridges and trenches form closed rings round
  small plates. The back-arc class covers 15–33 % of the planet; this is the 800 km halo P1-27
  already noted.

**Island arcs: good.** (Seen: 0010–0075.) Curved chains of volcanic islands with snow-capped
cones, beside a trench, are the most Earth-like thing in the early video.

## 5. Mountains

**Where: right. Shape and timing: wrong.** (Measured and seen.)
- Ranges do form at subduction margins (564 "Andean" phases) and at collisions (107 "Himalayan").
- Every collision is identical: 2,000 m of uplift, 30 Myr long. Real ones vary enormously.
- The uplift lands in one step. The collision events I checked record up to 15,000 of the 100,000 plate samples lifted by one
  collision, so up to 15 % of the planet at once.
- Seen in 0193 → 0194 (612 → 610 Myr ago) and 0483 → 0484 (32 → 30 Myr ago): shelf sea turns into
  land across a whole region in one frame, and new lakes appear.
- Measured: land grows by 6.0 % of the planet in one step at 610 Myr ago, 5.8 % at 30, 4.9 % at 494,
  4.6 % at 706. Each of those four has a collision event in that same step.
- In the regime layer, collision zones are round blobs. They cover up to 20 % of the planet (at
  198 Myr ago). Earth's collision mountains are belts: the Himalaya is about 2,400 km long and a few
  hundred wide.

**Heights: plausible.** (Measured, frames and status.)
- Median land height 610 m. Earth's mean is about 800 m (from memory).
- 99th percentile about 4.1 km; highest point 6.4 km at the end. Everest is 8.8 km.
- 11 % of land is above 2 km and 1.1 % above 4 km. Earth's figures are about the same (from memory).
- Cordilleras are low, as the known list says. In the video the Andean-type margins are thin
  white-capped lines, not broad ranges.

**Lifetime: plausible, not directly measured.**
- Erosion on ground above 2 km averages 57 m/Myr (measured). Earth's mountain rates span roughly
  20 m/Myr (old ranges) to over 1,000 (the Himalaya), from memory.
- Inferred: 2 km of relief would wear down in the order of 100 Myr once rebound is allowed for. That
  matches Earth, where the 300-Myr-old Appalachians and Urals are low hills.
- I did not track individual ranges through the frames.

## 6. Volcanism

**Arcs: right place.** (Seen: volcano layer, frame 499. Measured, events.)
- Active cones sit in curved lines on the overriding side of trenches.
- 713 arcs, 61 % continental and 39 % island arcs. Cone heights 1.2–3.0 km.

**Hotspot chains: good, hard to see.** (Measured, events.)
- 35 tracks. Median length 4,750 km (Earth-equivalent) over 78 Myr. Hawaii–Emperor is about
  6,000 km over about 80 Myr (from memory).
- Chains carry their volcanoes into trenches: 124 of about 370 built were subducted.
- Only 3 islands and 8 seamounts are recorded standing, so the video shows almost no Hawaii-like
  chains.

**Flood basalts: right size, too regular, too few.** (Measured and seen.)
- 17 in 1000 Myr. Earth averages one per 20–30 Myr, so 33–50.
- Every one is the same: 1,500 m thick, 2.0–2.7 million km² (Earth-equivalent). That is the size of
  the Deccan or Siberian Traps.
- In the volcano layer they are identical perfect discs. In the video they barely show.

## 7. Climate and ice, as seen in the video

**The white band.** (Seen: every frame from 0025. Measured, frames.)
- From about 950 Myr ago to the present, the top and bottom of the picture are solid white with a
  dead-straight, softly blurred edge.
- The edge is at one latitude all round the planet: in every frame, at least a tenth of all longitudes
  share the exact same edge latitude.
- It moves in 2° jumps (the climate model's bands) and stays put in 76–80 % of steps.
- Median edge 60° north and south. It reaches 46° at 850 Myr ago.
- The *amount* is not wrong: sea ice covers a median 14 % of the ocean, and Earth's is about 12 %.
  The *shape* is wrong. Earth's ice edge wanders by 20° or more with currents and coasts (from
  memory).
- **Pole stretch makes it look worse.** The area poleward of 60° is 13 % of the planet but a third
  of an equirectangular picture.
- This is the known banding. P1-26 replaces it.

**A permanent ice age.** (Measured, events and frames.)
- The run records one glaciation, from 934 Myr ago to the present, still going.
- Land ice covers a median 10 % of land and peaks at 20 % at 596 Myr ago. Earth today is about 10 %,
  but Earth today is in an ice age. Over the last 540 Myr Earth was ice-free about 80 % of the time.
- Global mean temperature starts at 16 °C, drops to 1 °C at 850 Myr ago, sits at 6–11 °C for most
  of the run and ends at 13 °C. Earth today is about 14–15 °C.
- The cold start-up is abrupt: 16 → 12 °C in the first 4 Myr.
- This is the known "somewhat cold". In the video it is very visible.

**Snow in the tropics.** (Seen: 0125, 0150, 0300. Measured.)
- White patches sit on mountain ranges at 20–30° latitude, and land ice reaches within 4° of the
  equator in a typical frame.
- The area is small (0.15 % of the planet within 30° of the equator), so these are mountain glaciers,
  which Earth has too. But the tropics are 3–8 °C colder than Earth's for most of the run, so the
  snow line is low and the patches are large. No ice sheet sits at the equator.

**Deserts and forests.** (Seen and measured.)
- Land colours run in east–west stripes: green near the equator, tan at 20–30°, green again, then
  grey-tan. On the starting supercontinent (0000–0025) the stripes are plain.
- Latitude alone explains 82 % of land temperature and 60 % of land rainfall (measured). The
  rainfall map of the last frame is ruled bands over the sea with mountain shading on land.
- The present-day climate zones, as shares of land:

| | tropical | arid | temperate | cold | polar |
|---|---|---|---|---|---|
| This run, present | 12 % | 28 % | 23 % | 9 % | 28 % |
| Earth (Peel et al. 2007) | 19 % | 30 % | 13 % | 25 % | 13 % |

- Too much polar, too little cold-continental. Arid is right.

**Sea level.** (Measured, events.)
- Twelve swings between −185 m and +102 m. Earth's long-term range is about −100 to +200 m (from
  memory).
- One fall is fast: 177 m in 10 Myr at 860–850 Myr ago, as the ice grows.

## 8. Rivers and lakes

**Rivers: good.** (Seen.)
- They run from high ground to the coast, join into trees, and thicken downstream.
- They move with the continents and re-route after collisions.
- At this scale they are drawn very wide, so early frames look like a continent of blue veins.

**Lakes: too big early on.** (Measured, frames. Seen: 0000, 0025, 0050.)
- In the first frame lakes cover 11 % of the land. Earth's lakes cover 3.7 % of ice-free land.
- The largest lake is 1 % of the planet at the start and 2.3 % at 928 Myr ago. Earth-equivalent that
  is 5–12 million km²; the Caspian Sea is 0.37 million.
- After the first 150 Myr it settles: lakes are a median 2 % of land. The largest is still a median
  0.13 % of the planet, about twice the Caspian.
- Inferred: the starting supercontinent is flat with closed hollows, so they fill.

**Last frame.** (Seen: 0498 → 0499.) The final frame draws the baked world's own rivers and lakes, so
several lakes vanish and rivers thin between the last two frames. Minor.

## 9. Visual artifacts

| Looked for | Found? | Evidence |
|---|---|---|
| Seam at the map edge | No | Measured: the colour step across the seam equals the step across the middle of the map (4.7 against 4.7). |
| Grid or cell pattern | No | Seen. Only in the crust-age layer (speckle), not in the video. |
| Flicker | No | Measured: frame-to-frame change is even. Every fifth frame changes 12 % more. |
| Sudden jumps | Yes | Land appearing at collisions (section 5). The ice edge stepping 2°. |
| Straight lines | Yes | The ice edge. The vegetation stripes. |
| Pole distortion | Yes, and expected | Polar land and ice are smeared wide across the top and bottom rows. |
| Sharp start | Yes | Frame 0000 has no ice and bare polar land; by 0010 the south is white. |

## 10. Overall

**Impressive and defensible.**
1. The global numbers. Speeds, crust conservation, land fraction, sea-floor median age and height
   statistics are all Earth-like, and the surface model's mass balance closes (residual −0.0005 km³).
2. The ocean. Ridges with evenly ageing floor on both sides, trenches with island arcs, hotspot
   tracks of the right length, flood basalts of the right size that trigger rifts.
3. The picture. No seam, no grid, no flicker; rivers that drain properly; shelves, rain shadows and
   snow lines that follow the terrain.

**What a geologist would call out, in order.**
1. The straight white ice bands.
2. Continents losing their shape.
3. Land popping up in round patches.
4. No ocean ever opening and closing properly.
5. A billion-year ice age.

## Ranked issues

Ranked by how visible in the video and how wrong. "Size" is a rough guess for the new ones.

| # | Issue | Visible | Wrong | Status | Size if new |
|---|---|---|---|---|---|
| 1 | Sea ice ends at one latitude, in 2° steps, as a permanent band | Very | Very | In progress (P1-26) | |
| 2 | Continents shred: edge share 1 % → 29 %, 440–580 pieces | Very | Very | Partly: P1-27 landed, crust spreading in progress. Still open by this measure | |
| 3 | Collision uplift lands in one step over a round area up to 15 % of the planet; land jumps up to 6 % of the planet per frame; orogens are blobs | High | High | **New** (related to the known collision count) | Medium: spread the uplift over the orogeny and shape it along the suture |
| 4 | Rifted pieces weld back after a median 18 Myr; no Wilson cycle | High | High | **New** | Large: plate-model rules |
| 5 | Welding builds a mega-plate: over half the planet in 51 % of frames | Indirect (continents stop moving apart) | High | **New**, a side effect of P1-27's suture rule | Large: same work as 4 |
| 6 | Permanent ice age, cold tropics, snow on low-latitude ranges | High | Medium | In progress (known cold bias, P1-26) | |
| 7 | East–west vegetation and rainfall stripes | Medium | Medium | In progress (P1-26) | |
| 8 | Too many collisions (107 per 1000 Myr), orogen-heavy provinces | Medium | Medium | Known | |
| 9 | Continents over-flooded late: 44 % of crust under water, land 22–24 % | Medium | Low | Partly (P1-17 hypsometry) | |
| 10 | Giant lakes in the first 150 Myr: 11 % of land, one lake 2.3 % of the planet | Medium | Medium | **New** | Small |
| 11 | Old sea floor: 12 % older than 180 Myr, Earth about none | Not in video | Medium | **New** (the gate reads 0.936 and passes) | Small to medium; likely eases with 5 |
| 12 | Flood basalts are identical discs, and half as frequent as Earth's | Low | Low | **New** | Small |
| 13 | Cordilleras low | Low | Medium | Known | |
| 14 | Hotspot island chains almost never stand above the sea | Low | Low | **New** | Small |
| 15 | Abrupt climate start; last frame's lakes differ from the frame before | Low | Low | **New** | Small |

## Sources

Sourced by web search for this review:
- Pacific plate 103 million km², about 20 % of Earth; seven major plates:
  <https://en.wikipedia.org/wiki/Pacific_plate>, <https://arxiv.org/pdf/1011.2752>
- Sea-floor age, mean 64.2 Myr, oldest large areas 180–200 Myr: Müller et al. 2008,
  <https://agupubs.onlinelibrary.wiley.com/doi/10.1029/2007gc001743>; Herodotus Basin about 340 Myr:
  <https://www.nature.com/articles/ngeo2784>
- Plate speeds, median RMS about 40 mm/yr over 200 Myr, fastest plates about 85, limit about 200:
  Zahirovic et al. 2015,
  <https://earthbyte.org/Resources/Pdf/Zahirovic_2015_Tectonic_speed_limits_plate_reconstructions.pdf>
- Supercontinent cycle length and the Rodinia and Pangaea dates:
  <https://en.wikipedia.org/wiki/Supercontinent_cycle>, <https://www.nature.com/articles/s43017-021-00160-0>
- Climate zones by land area: Peel et al. 2007,
  <https://hess.copernicus.org/articles/11/1633/2007/hess-11-1633-2007.pdf>
- Sea ice about 7 % of Earth's surface and 12 % of the oceans: <https://en.wikipedia.org/wiki/Sea_ice>
- Lakes 3.7 % of non-glaciated land: Verpoorter et al. 2014,
  <https://agupubs.onlinelibrary.wiley.com/doi/10.1002/2014GL060641>
- Flood basalts one per 20–30 Myr:
  <http://www.largeigneousprovinces.org/sites/default/files2/Ernst%20and%20Youbi%202017%20PPP.PDF>
- Greenhouse for about 80 % of the last 540 Myr:
  <https://en.wikipedia.org/wiki/Greenhouse_and_icehouse_Earth>

From memory, not checked: Earth's continental crust at about 40 % of its area; land-height shares
above 2 and 4 km; mean land height; mountain erosion rates; the sea-ice edge's wander; Hawaii–Emperor
length and age; the long-term sea-level range; the Caspian's area; the count of Earth plates above 5 %.

In the repo: `docs/deeptime-implementation-plan.md` §8, `STATUS.md` (P1-27 entry),
`docs/design/p1-26-per-cell-climate.md`, `crates/deeptime-core/src/plates/normalise.rs`,
`crates/deeptime-core/presets/earthlike_quick_16k.json`.

## Housekeeping

- Analysis ran on the Azure test VM, single process, niced. The scratch folder
  `/Volumes/Graphide/ban/deeptime-runs/tectonic-review/` is deleted.
- The workbench was not stopped or restarted. Asking it for 37 layer images added that many cached
  PNGs to the run's `frames/` folder.
- Nothing was committed.
