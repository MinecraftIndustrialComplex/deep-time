# Volcanoes, magma and lava

Status of this page: `stage 2 (constants checked against sources on 2026-10-02, package WP-2)`.

Ledger ids on this page: `VOL-001` to `VOL-073` ([ledger/VOL.md](ledger/VOL.md)) and `MAG-001` to
`MAG-042` ([ledger/MAG.md](ledger/MAG.md)). Of these 115 rows, 81 are `game-choice`, 28 `uncited`,
2 `verified`, 2 `abstract-only`, 1 `cited-unverified` and 1 `calibrated` (stage 1: 47
`game-choice`, 63 `uncited`, 4 `cited-unverified`, 1 `calibrated`).

**Read this first.** Most of this feature is design. Stage 2 read the sources that exist for each
number and sorted the rows: 81 are rules or values with no Earth counterpart, or deliberate
departures from Earth, and are now labelled as such; 28 are physical values for which no source
was found or the source read does not give the number. No volcano size in the code comes from a
measurement of Earth's volcanoes. Where a source was read, this page says how the code compares
with it. The one paper the code cited, for the spacing of arc volcanoes, says in its abstract that
there is no characteristic spacing; the citation is corrected. Two departures from Earth that shape
worlds are questions for Ben (see "Open questions"): stratovolcanoes are aimed at about three
times the height of Earth's typical one, and dead ocean volcanoes are sunk twice now that the
crust bends under them.

Scale used throughout: one block is 300 m across and 20 m up, so 1 km is 3.3 blocks across and
50 blocks up, and every slope is drawn 15 times steeper than it is.

## What you see in the world

**Arc volcanoes.** Lines of cones stand along the landward side of ocean trenches, where one plate
sinks under another, at least about 60 km (200 blocks) apart. They occur on continents and as island
chains. A cone is 1.2 to 3 km tall (60 to 150 blocks), most of them near 3 km, with a concave
profile that steepens toward the top. At the default setting it is 13 to 28 km in radius (45 to 93
blocks), twice as wide as its physical shape, so that it does not read as a spike. It has a summit
crater at least 2 km (7 blocks) across. Its rock is andesite over tuff (volcanic ash turned to
rock), or rhyolite over tuff where the crust is thick.

**Shield volcanoes and island chains.** Over a hotspot (a fixed plume of hot mantle) under ocean
floor, a broad, gently sloping basalt shield rises 3.5 to 6.5 km above the sea floor, 49 to 80 km in
radius. Because the plate moves over the plume, a new shield starts about every 70 km and the older
ones go extinct, erode and sink, leaving a chain of islands and seamounts (drowned volcanoes) that
gets older away from the plume. Most shields do not reach the surface; in the reviewed 1,000 Myr run
only 3 islands and 8 seamounts stood at the end.

**Calderas.** Volcanoes on thick continental crust, and hotspot centres under continents, can
collapse after a large eruption into a caldera: a flat-floored basin with steep walls, 6 to 18 km
(20 to 60 blocks) across and roughly 550 to 850 m (28 to 42 blocks) deep, ringed by a sheet of tuff
up to 250 m thick that thins outward.

**Flood basalts.** A large igneous province becomes a basalt plateau 1,500 m (75 blocks) thick at
its centre, built of eight flows with thin ash beds between. Where the pile is at the surface, the
terrain is cut into steps 80 m (4 blocks) high.

**Magma.** Under each volcano that is still active at the present there is a chamber of lava, its
top 3 km (150 blocks) below the crater floor. In blocks it is a tall lens (for example 19 across and
57 high under a large arc cone) because heights are drawn 15 times larger than widths. A narrow
column of lava rises from it to just under the crater, capped by magma blocks. A quarter of the
active volcanoes on dry land have an open lava lake in the crater instead, 4 to 20 blocks across.
Everything molten is walled in by smooth basalt, so lava never touches water or an open cave.
Extinct volcanoes have no magma.

**Lava elsewhere.** The random lava lakes and springs of vanilla Minecraft and of Terralith are
removed. The only other lava is a layer at the very bottom of the world (below y -342 in the default
world). On the measured planet a point on land was a median 658 blocks from the nearest active
volcano and 1,178 blocks from the nearest open lava lake.

## The model in plain words

The volcano stage runs once per plate step (about 2 Myr), after the surface stage, and keeps a
catalogue of individual volcanoes that ride on their plates.

1. **Where arcs are.** The plate model flags cells above a subducting plate as "arc". The stage
   turns that into an arc magma flux between 0 and 1: the convergence speed divided by 80 mm/yr,
   capped at 1, and multiplied by 0.7 on oceanic crust (VOL-003, no source found; most arcs in a run
   reach 1). It then thins the flagged band to its centre line, the volcanic front (VOL-004).
2. **Birth of arc volcanoes.** On the front, wherever the flux is at least 0.25, a new
   stratovolcano (a steep cone of lava and ash) is placed if no other cone stands within 60 km
   times a random factor of 0.8 to 1.2 (VOL-002, VOL-039). Its planned height runs from 1.2 km at
   the threshold flux to 3 km at full flux (VOL-019); its radius follows height to the power 0.8
   (VOL-020); its life is drawn between 2 and 8 Myr (VOL-038). If the crust under it is continental
   and at least 42 km thick it is silicic: richer in silica, explosive, and able to form a caldera
   (VOL-005).
3. **Birth of hotspot volcanoes.** At each live hotspot a new volcano starts once the previous one
   has drifted 70 km away (VOL-007): a shield on oceanic crust, a low silicic centre on continental
   crust (VOL-006). It stays active while it is within 70 km of the plume.
4. **Growth.** An active volcano closes the gap to its target height with a time constant of
   1 Myr (VOL-041). It stops growing where the ground under it would pass a ceiling of 6.8 km above
   the sea, 0.65 of that for shields, scaled with gravity and crustal strength (VOL-032, VOL-034).
   What it builds is added to the terrain and written into the rock record as tuff, andesite,
   rhyolite or basalt (VOL-056 to VOL-059). Since 2026-10-02 the surface stage's flexure treats the
   cone as a load, so the crust under it bends down and the cone's summit stands lower by that much
   ([flexure.md](flexure.md)); the cone's own height above its surroundings is unchanged.
5. **Erosion.** The surface stage erodes a cone like any other land. The stage measures that
   change over the cone's footprint and lowers the volcano's recorded height to match (VOL-051).
6. **Death.** A stratovolcano goes extinct when its planned life ends or the arc flux under it has
   been too low for two steps (VOL-010). An extinct volcano on ocean crust sinks at 25 m/Myr, up to
   1.5 km (VOL-048), on top of the flexure and of the plate model's deepening of old ocean floor.
   It leaves the catalogue when worn below a tenth of its peak height, or after 250 Myr (VOL-049),
   or when its plate carries it into a trench (VOL-050).
7. **Calderas.** A silicic cone collapses into a caldera with a chance of 0.04 per Myr; a
   continental hotspot centre with 0.12 per Myr (VOL-045). The radius is drawn between 3 and 9 km
   (VOL-029) and the depth is half the cone's height, capped at 400 m plus 5 % of the radius
   (VOL-030). Half of the removed rock is laid around it as a tuff sheet (VOL-046). The volcano
   then dies.
8. **Flood basalts.** Each large igneous province the plate model creates is laid as eight flows,
   88 % basalt and 12 % tuff each (VOL-052, VOL-060).
9. **Block scale.** The amplifier restores each volcano's true profile, which the coarse grids
   blur, and adds the crater or caldera floor. Sharp detail on an extinct volcano fades with a time
   constant of 5 Myr (VOL-063 to VOL-072).
10. **Magma in the game.** The bake lists the volcanoes that are still active. The Minecraft side
    gives each a chamber (an ellipsoid whose radius is a tenth of the volcano's, between 1.2 and
    4.5 km), a conduit and a plug, and gives the most active quarter of those that can hold one a
    crater lava lake (MAG-001 to MAG-026).

## Equations and constants

Shapes. `ρ` is the distance from the centre divided by the basal radius `R`; each profile is the
fraction of the full height `H` at that distance.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| VOL-016 | Stratovolcano profile | (1 − ρ)^1.5: steepest at the summit, 1.5 H/R; mean slope 0.8 H/R | – | none; compared with Grosse et al. 2014 | `uncited` |
| VOL-017 | Shield profile | (1 − ρ²)^1.5: steepest at ρ = 0.71, 1.5 H/R | – | none; compared with Clague & Sherrod 2014, Grosse et al. 2014 | `uncited` |
| VOL-018 | Silicic centre profile | (1 − ρ²)² | – | none | `uncited` |
| VOL-019 | Stratovolcano height | 1.2 to 3.0, linear in arc flux | km | none; Grosse et al. 2014 gives Earth's distribution | `uncited` |
| VOL-020 | Stratovolcano radius | 14 × (H / 3 km)^0.8 ÷ flank steepness | km | none; Grosse et al. 2014 for the height-to-width ratio | `uncited` |
| VOL-021 | Shield height above the sea floor | 3.5 to 6.5, by plume strength | km | none; USGS for Mauna Loa | `uncited` |
| VOL-022 | Shield radius | 40 × (H / 6.5 km)^0.8 ÷ flank steepness | km | none | `uncited` |
| VOL-023 | Continental hotspot centre | 0.9 tall, 35 radius | km | none; Christiansen 2001 for Yellowstone | `uncited` |
| VOL-024 | Flank steepness | 0.5 (radius doubled) | – | Ben, decisions log 2026-09-30 | `game-choice` |
| VOL-026 | Summit crater | radius max(1 km, 0.07 R); depth 0.06 H | km | agent, for visibility; Grosse et al. 2014 compared | `game-choice` |
| VOL-029 | Caldera radius | uniform 3 to 9, at most 0.6 R | km | none; Christiansen 2001, USGS compared | `uncited` |
| VOL-030 | Caldera depth | min(0.5 H, 400 m + 0.05 × radius) | m | none; Grosse et al. 2014 compared | `uncited` |
| VOL-031 | Shield summit caldera | radius 0.08 R, depth min(0.04 H, 250 m), on shields over 4 km | m | none | `uncited` |
| VOL-032 | Summit ceiling | 6.8 above the sea; × 0.65 for shields; scaled by crust strength over gravity | km | Earth's highest volcano, secondary source | `abstract-only` |

The slopes of the profiles, worked out from the code: a 3-km stratovolcano over its physical
14 km radius is 18° at the summit and 10° on average; at the default doubled radius 9° and 5°. The
tallest shield is 14° at its steepest over 40 km, 7° at 80 km. In blocks every slope is 15 times
steeper.

Rules and rates.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| VOL-003 | Arc magma flux | clamp(convergence / 80, 0, 1), × 0.7 on oceanic crust | – (convergence in mm/yr) | none | `uncited` |
| VOL-002 | Flux needed for a volcano | 0.25 | – | agent | `game-choice` |
| VOL-039 | Least spacing of arc volcanoes | 60 × random(0.8 to 1.2) | km | agent; de Bremond d'Ars et al. 1995 (abstract) and Hildreth 2007 compared | `game-choice` |
| VOL-005 | Crust thickness for silicic volcanoes | 42 | km | none | `uncited` |
| VOL-007 | Spacing along a hotspot chain | 70 | km | none | `uncited` |
| VOL-038 | Stratovolcano lifetime | 2 to 8 | Myr | agent, build log 2026-09-30; Hildreth 2007 compared | `game-choice` |
| VOL-041 | Growth time constant | 1 | Myr | none; consistent with Clague & Sherrod 2014 | `uncited` |
| VOL-045 | Caldera eruption rate | 0.04 (silicic cone), 0.12 (continental hotspot) | per Myr | none; Christiansen 2001 compared | `uncited` |
| VOL-048 | Subsidence of extinct ocean volcanoes | 25, up to 1,500 m in all | m/Myr | none; Clague & Sherrod 2014 compared | `uncited` |
| VOL-052 | Flood basalt pile | 8 flows; thickness and radius from the plate model (1,500 m; 900 km on an Earth-sized planet) | – | agent | `game-choice` |
| VOL-056 to VOL-060 | Rock laid down | arc: 35 % tuff, 65 % andesite; silicic: 45 % tuff, 55 % rhyolite; shield: basalt; flood basalt: 88 % basalt, 12 % tuff | share | none | `uncited` |
| VOL-065 | Fading of an extinct volcano's fine detail | e^(−age / 5 Myr) for the crater, three times slower for the cone | Myr | agent (a look setting) | `game-choice` |

Magma and lava.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| MAG-002 | Depth of the chamber's top below the crater floor | 3.0 | km | agent, under Ben's 2026-10-01 decision; Poland et al. 2014, Clague & Sherrod 2014, Rasmussen et al. 2022 (abstract) compared | `game-choice` |
| MAG-003 | Chamber radius | 0.1 × volcano radius, kept within 1.2 to 4.5 | km | agent; Poland et al. 2014 compared | `game-choice` |
| MAG-004 | Chamber half-height | 0.2 × its radius | – | agent; Kilauea's sill-like source (Poland et al. 2014) | `game-choice` |
| MAG-019 | Share of able volcanoes with a crater lava lake | 0.25, most active first | – | agent, build log 2026-10-01 ("a player should be able to find one") | `game-choice` |
| MAG-020 | Earth's persistent lava lakes | "a handful (eight in a secondary list) against about 1,350 potentially active volcanoes" | count | USGS (1,350, read); the eight from a secondary list | `abstract-only` |
| MAG-027 | Lava floor | open space below world floor + 10 blocks is lava | blocks | vanilla Minecraft's rule, moved to the world's floor; Ben, decisions log 2026-10-01 (kept as a setting, not yet decided) | `game-choice` |
| MAG-028, MAG-029 | Vanilla's lava rules the mod replaces | lava below y −54; cave lakes at y −10 or lower may be lava | blocks | Minecraft 1.21.1 source, read | `verified` |
| MAG-034 | Random lava lakes and springs | removed | – | Ben, decisions log 2026-10-01 | `game-choice` |

The full tables, with file and symbol for each row, are in the ledger files.

## Earth against the mod

Earth, from the sources read in stage 2:

| Quantity | Earth | Source and how it was read | The code |
|---|---|---|---|
| Height of stratovolcanoes | median 1.02 km, mean 1.13 km, s.d. 0.55 km, range 0.17–4.57 km (512 subaerial stratovolcanoes) | Grosse et al. 2014, table 1, full text | aimed at 1.2–3.0 km, most near 3 km |
| Height to basal width of stratovolcanoes | median 0.13, range 0.02–0.30; 0.11 or less over 20 km wide | Grosse et al. 2014, table 1 and p. 7 | 0.09–0.11 at the physical radius, 0.045–0.054 at the default |
| Basal width of stratovolcanoes | median 7.9 km, range 2–66 km | Grosse et al. 2014, table 1 | 13–28 km physical, 27–56 km at the default |
| Flank slopes of stratovolcanoes | mean flank 17.7°, steepest average 27.7° at 0.79 of the height (medians) | Grosse et al. 2014, table 1 | 10° mean and 18° at the summit for 3 km over 14 km; half that at the default |
| Shield flanks | lava flows dip 2°–10°; subaerial shields' mean flank 8.1° (median of 89) | Clague & Sherrod 2014, p. 114; Grosse et al. 2014, table 1 | 14° at the steepest for the tallest physical shield, 7° at the default |
| Large summit craters and calderas on composite volcanoes | 0.6–11 km across, median 2.2 km; depth under 100 to 860 m, median 240 m (smaller craters not measurable) | Grosse et al. 2014, p. 8 | craters at least 2 km across, 4 km on a 3-km cone, 72–180 m deep; calderas 6–18 km across, up to 550–850 m deep |
| Spacing of arc volcanoes | no characteristic spacing: spacings follow a Gamma distribution, as random points do | de Bremond d'Ars et al. 1995, abstract only | a least distance of 60 km, so cones stand evenly |
| Mean spacing of large arc centres | 27 km (Alaska Peninsula), 42 km (Cascades, Andean Southern Volcanic Zone); 87 km between the 17 largest Cascade centres | Hildreth 2007, tables 4 and 10, full text | 60 km least (the mean spacing a run makes was not measured) |
| Lifespan of large arc centres | mostly 0.1–1 Myr | Hildreth 2007, p. 64 | 2–8 Myr (a game choice: one cone stands for a centre over a 2-Myr step) |
| Mauna Loa above the sea floor | about 9 km (4.17 km above the sea, 5 km of submarine flank), the sea floor pushed down another 8 km | USGS Mauna Loa page, fetched | shields 3.5–6.5 km, summits at most 4.4 km above the sea |
| Hawaiian growth | principal growth in the first 1–2 Myr | Clague & Sherrod 2014, abstract | growth time constant 1 Myr; activity window 0.7–2.8 Myr at 25–100 mm/yr |
| Subsidence of Hawaiian volcanoes | more than 1 km in about 1 Myr under the growing load (2.6 m/kyr off Hawaii), then about 10 m/Myr as the plate cools | Clague & Sherrod 2014, pp. 130–131 | flexure under the load (since 2026-10-02), the plate model's ocean floor, and 25 m/Myr more after death |
| Yellowstone | plateau about 2,400 m against 1,600–1,900 m beside it; caldera 45 by 85 km; caldera-forming eruptions at 2.1, 1.3 and 0.64 Ma | Christiansen 2001, full text | silicic centre 0.9 km tall; caldera radius at most 9 km; 0.12 calderas per Myr |
| Caldera sizes | Crater Lake 8.0 by 9.7 km; Long Valley 16 by 32 km | Wikipedia (secondary); USGS Long Valley page (stage 1) | 6–18 km across |
| Shallow magma storage | Hawaiian shields 1–7 km; Kilauea 1–2 km under the caldera centre, 2–4 km under its south part; arc volcanoes 6 ± 3 km | Clague & Sherrod 2014, abstract; Poland et al. 2014, p. 194; Rasmussen et al. 2022, abstract only | 3 km under every crater |
| Kilauea's reservoir | "a few cubic kilometers", sill-like | Poland et al. 2014, pp. 194, 199 | 1.4–18 km³ (1.2–2.8 km radius, half-height 0.2 of it) |
| Potentially active volcanoes | about 1,350, besides the spreading ridges | USGS FAQ, fetched | – |
| Persistent lava lakes | eight volcanoes in a list | Wikipedia (secondary); Lev et al. 2019 not read | a quarter of the active volcanoes on land |
| Highest volcano | Ojos del Salado, 6,893 m | Wikipedia citing the Global Volcanism Program (secondary) | ceiling 6.8 km |
| Deccan Traps | section 2,500 m thick; bulk erupted within about 1 Myr | Courtillot & Renne 2003, library text | pile 1,500 m; 8 flows |
| Flood basalt provinces | 16 over about 345 Myr, about one per 22 Myr | Courtillot & Renne 2003, abstract (stage 1); the division is ours | one per 59 Myr in a 1,000 Myr run |

Measured in the mod. The first rows are from the build log; the stage 2 rows are from the runs
described under them.

| Quantity | The mod | Run |
|---|---|---|
| Volcanoes at the end of a run | 250 in the catalogue; 203 active stratovolcanoes, 2 active shields | `earthlike_quick_16k`, scattered continents, 500 Myr, 2026-09-30 (seed not stated; the preset's is 1) |
| Stratovolcanoes born in a run | 24,017 (most erode away) | same |
| Calderas | 295, and 27 summit calderas on shields | same |
| Flood basalt provinces | 9 in 500 Myr; 17 in 1,000 Myr | same; and run review, `earthlike_quick_16k`, supercontinent, 1,000 Myr, seed 1, 2026-10-01 |
| Arcs | 713 arcs, 61 % continental, 39 % island arcs | run review, 1,000 Myr |
| Hotspot tracks | 35 tracks, median 4,750 km (scaled to Earth's size) over 78 Myr | run review, 1,000 Myr |
| Active volcanoes at the present | 123: 119 arc, 3 caldera complexes, 1 shield; 100 on land; 16 with a crater lava lake | `earthlike_quick_16k`, supercontinent, 500 Myr, 2026-10-01 |
| Distance from a land point to the nearest active volcano on land | median 658 blocks (10th percentile 169, 90th 1,309) | same; 4,000 points |
| Effect on land height | with volcanoes off, land above 2 km falls from 0.307 to 0.170 of the land (Earth: 0.102) | hypsometry runs, `earthlike_quick_16k`, 500 Myr, 2026-10-01 (before flexure) |

MEASURED_TABLE

## Simplifications and limits

- **Three kinds only.** There are no cinder cones, no volcanoes at mid-ocean ridges or in rifts,
  and no fissure eruptions. The code's own comment notes that ridges and rifts are where lava is
  common on Earth (MAG-038); they are not exported to the game at all.
- **A volcano is a centre, not an eruption.** With steps of about 2 Myr, one "stratovolcano" stands
  for a volcanic centre that rebuilds itself in one place. Hence lifetimes of 2 to 8 Myr against
  the 0.1 to 1 Myr of the large Cascade centres (Hildreth 2007).
- **No eruptions, magma supply or chemistry.** Height grows toward a target on a fixed clock.
  Eruption rate, erupted volume and magma composition are not modelled. "Silicic" is a yes or no set
  by crust thickness.
- **Evenly spaced cones.** A least distance between cones makes them stand at nearly regular
  intervals. Earth's arc volcanoes are spaced at random (de Bremond d'Ars et al. 1995, abstract),
  with gaps of 60 to 136 km and large centres as little as 2 km apart (Hildreth 2007, table 10).
- **Volcano positions follow the plate model's arc flag**, which covers broad areas; the stage
  reduces it to a line itself. When the plate model changes (see [plates.md](plates.md)), so will
  where volcanoes stand. Most arcs reach the highest flux, so cone height hardly varies with the
  speed of subduction.
- **Lengths do not shrink with the planet.** A volcano's radius is in real kilometres on a planet
  an eighth of Earth's size, so volcanoes are large for their world. Flood basalt provinces, by
  contrast, are scaled to the planet (about 110 km in radius at 16k).
- **Magma chambers are one fixed shape** at one depth, whatever the volcano's size, age or kind.
- **Too much high land from volcanoes.** Cones add about a fifth of everything the surface erodes
  in a run and push the share of high land well above Earth's (see the tables). This was kept as it
  is (Ben, decisions log 2026-10-01: volcano density stays 1, revisited after the height fix and
  the ice branch land).
- **Activity saturates.** Most arc volcanoes have the highest activity value, so which of them get
  a crater lake is decided by their catalogue number, not by anything physical.
- **Hardwired couplings.** The hotspot strength scale (VOL-008) mirrors the plate model's random
  plume strength range in another file, and the ceiling's normalisation (VOL-034) repeats the plate
  model's default crust strength. If either changes alone, volcano heights shift silently.

## Game choices

| What | Why | Who and when |
|---|---|---|
| Which kinds exist (arc stratovolcanoes, hotspot shields and chains, calderas, flood basalts) | scope for version 1 | Ben, decisions log 2026-09-29 |
| Flank steepness 0.5: every volcano twice as wide as its physical profile (VOL-024) | at 15 times vertical exaggeration a physical cone is a spike | Ben, decisions log 2026-09-30 ("Setting, gentler default") |
| Young volcanoes are smooth; old ones regain rugged ground (VOL-068) | looks | Ben, decisions log 2026-09-30 |
| Small arc segments counted on their subduction event (VOL-015) | event volume | Ben, decisions log 2026-09-30 ("Count small ones") |
| Volcano density 1 (VOL-012) | kept although it raises the land | Ben, decisions log 2026-10-01 |
| Lava only at active volcanoes; random lava lakes and springs removed (MAG-001, MAG-034) | "only at active volcanoes, ridges and hotspots" is where lava is on Earth | Ben, decisions log 2026-10-01 |
| The lava floor at the bottom of the world (MAG-027) | a Minecraft convention with no counterpart on Earth; kept as a setting until decided | open; Ben, decisions log 2026-10-01 |
| One chamber shape at one depth, 3 km (MAG-002 to MAG-005) | a body a player can find under every active volcano; 3 km lies inside Earth's range | agent, build log 2026-10-01, under Ben's "Volcanoes is a go ahead" |
| A quarter of able volcanoes have an open lava lake (MAG-019), though persistent lava lakes are rare on Earth | "a player should be able to find one" | agent, build log 2026-10-01 |
| Lava is sealed in smooth basalt and never touches water (MAG-010, MAG-011, MAG-041) | lava pools were found overlapping the ocean | agent, build log 2026-10-01, after Ben's report |
| Arc volcanoes at least 60 km apart (VOL-039) | keeps the doubled cones apart; one cone stands for a large long-lived centre | agent, build log 2026-09-30; reviewed WP-2 2026-10-02 |
| Summit crater at least 2 km across (VOL-026) | a real crater would be under three blocks wide | agent, build log 2026-09-30 |
| Stratovolcano lifetime 2 to 8 Myr (VOL-038) | one step is 2 Myr | agent, build log 2026-09-30 |
| Eight flows in a flood basalt pile (VOL-052), terraces 80 m high (VOL-036) | visible layering and steps | agent, build log 2026-09-30 |
| Implementation rules: the volcanic front, the flux threshold, merging, the blind step, retirement, rendering and fading (VOL-002, VOL-004, VOL-008 to VOL-011, VOL-027, VOL-028, VOL-040, VOL-042, VOL-043, VOL-047, VOL-049 to VOL-051, VOL-053 to VOL-055, VOL-061 to VOL-073) | how the stage turns plate data into cones and cones into blocks; no Earth counterpart | agent, build log 2026-09-30 and 2026-10-01; labelled WP-2 2026-10-02 |

## Open questions

Questions for Ben (STATUS.md, pending questions, 2026-10-02), recommendation first:

1. **Stratovolcano heights** (VOL-019, VOL-003). Earth's median stratovolcano stands 1.0 km above
   its base (Grosse et al. 2014); the code aims at 1.2 to 3.0 km and, because most arcs reach full
   flux, most cones aim at 3 km. See the measured table for what a lower range does to the land
   above 2 km. QUESTION1_DETAIL
2. **Island subsidence counted twice** (VOL-048). Since cones are flexure loads (decisions log
   2026-10-02), the crust sinks under a growing volcano, and the plate model deepens old ocean
   floor under it. The extra 25 m/Myr for dead ocean volcanoes repeats both. Recommended: set it to
   0 by default and keep the setting. QUESTION2_DETAIL
3. **Continental hotspot calderas** (VOL-045). Yellowstone collapsed three times in 1.5 Myr
   (Christiansen 2001); the code gives a continental hotspot centre a 10 to 30 % chance of one
   caldera in its life. Recommended: raise the continental hotspot's rate to about 1 per Myr (the
   arc rate unchanged), so most continental hotspot centres end in a caldera, as on Earth. A small
   change to the look of a few places per planet; no effect on the hypsometry.

Not yet settled, for a later package or a geologist:

4. **Arc spacing is regular, Earth's is random** (VOL-039). A random spacing with the same mean
   would put some cones close together and leave long gaps, as in the Cascades. Not proposed
   now: with cones twice their physical width, close pairs would merge into ridges.
5. **The cone's profile** (VOL-016). Real stratovolcanoes are steepest a little below a flatter
   summit region (at 0.79 of the height, Grosse et al. 2014); the code's cone is steepest at the
   summit. The amplifier and the crater already flatten the top, so the difference in blocks is
   small; not measured.
6. **Crater and summit caldera sizes scale with the doubled radius** (VOL-026, VOL-031): a 3-km
   cone has a 4-km crater, the tallest shield a 13-km summit caldera, two to four times Earth's.
   Whether the flank setting should widen craters too was never decided.
7. **The arc flux law** (VOL-003) has no source, and the same law is coded three times (here,
   RCK-059 and the deposit engine).
8. **Shield heights and radii** (VOL-021, VOL-022): Mauna Loa is 9 km above the sea floor; the
   code's strongest plume makes 6.5 km. Reunion, Galapagos and a morphometric source for oceanic
   shields were not read.
9. **Flood basalt flows and terraces** (VOL-036, VOL-052): no source read gives flow thicknesses;
   a real trap pile holds many more than eight flows.
10. **Persistent lava lakes** (MAG-020): the count rests on a secondary list; Lev et al. 2019 is
    the review to read.
11. **Chamber depth for arcs** (MAG-002): arc magmas are stored at 6 ± 3 km (abstract only), deeper
    than the 3 km used everywhere. Kept as Ben's 2026-10-01 setting.
12. **The volcano ceiling** (VOL-032, VOL-034) takes a crustal strength of 140 MPa from the plate
    model; see [isostasy-and-height-limit.md](isostasy-and-height-limit.md). Ojos del Salado's
    height was read only in a secondary source.
13. **Flood basalt frequency**: the 1,000 Myr run made 17, about a third of Earth's rate if the
    Courtillot & Renne list is taken at face value. That is the plate model's doing.
14. **Where the lava floor goes** is undecided, and so is whether a median 658 blocks to the
    nearest volcano is acceptable for players (STATUS.md, pending questions).

## Sources

Keys refer to [bibliography.md](bibliography.md).

| Key | Used for | How it has been read |
|---|---|---|
| Grosse2014 | stratovolcano and shield heights, widths, slopes; crater and caldera sizes (VOL-016 to VOL-020, VOL-026, VOL-029, VOL-030) | in full (repository copy of the submitted version), WP-2 |
| Hildreth2007 | arc volcano spacing, the volcanic front, lifespans (VOL-004, VOL-038, VOL-039) | in full, WP-2 |
| ClagueSherrod2014 | Hawaiian growth, subsidence, shield slopes, reservoir depths (VOL-007, VOL-009, VOL-017, VOL-041, VOL-048, MAG-002) | in full, WP-2 |
| Christiansen2001 | Yellowstone: plateau height, caldera size, caldera-forming eruptions (VOL-006, VOL-023, VOL-029, VOL-045, VOL-046) | the summary, physiography and chronology sections, WP-2 |
| Poland2014 | Kilauea reservoir depths, size and shape (MAG-002 to MAG-004) | the reservoir sections, WP-2 |
| Bremond1995 | arc volcano spacing (VOL-039) | abstract only; the full text is free to read but refused scripted access |
| Rasmussen2022, ChaussardAmelung2014 | magma storage depths under arc volcanoes (MAG-002) | abstracts only, WP-2 |
| TenBrink1991 | hotspot volcano spacing (VOL-007) | abstract only, WP-2 |
| Courtillot2003a | Deccan thickness, trap durations, flood basalt frequency (VOL-036, VOL-052) | library text, WP-2 (stage 1: abstract) |
| USGS-MaunaLoa, USGS-ActiveVolcanoesFAQ | Mauna Loa's size, the count of active volcanoes (VOL-021, MAG-020) | fetched, WP-2 |
| MinecraftSources | vanilla's lava rules (MAG-028 to MAG-031) | source code read, WP-2 |
| Lev2019 | persistent lava lakes (MAG-020) | not read (refused scripted access) |
| Szabo1986 | subsidence of Hawaii | abstract only (stage 1); superseded by Clague & Sherrod 2014 |
| Scaillet2008, Przeor2022 | magma storage depths | abstracts only (stage 1); no longer cited in code comments |
| Wikipedia pages on named volcanoes | Crater Lake, Ojos del Salado, Fuji, lava lakes | secondary (stage 1) |

Session note: [`../research/notes/session-lava-realism.md`](../research/notes/session-lava-realism.md).
Extraction notes, with 21 flags: [extraction-notes/VOL.md](extraction-notes/VOL.md) (flags 1 to 5
are settled in the ledger rows; flag 4 rested on reading radii as widths).
Build log: STATUS.md, "P1-21 volcanoes as landforms" (2026-09-30), "Magma under active volcanoes"
and "Lava floods" (2026-10-01), "Science stage 2, WP-2: volcanoes and magma" (2026-10-02).
