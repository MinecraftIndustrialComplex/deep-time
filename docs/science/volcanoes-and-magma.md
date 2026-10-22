# Volcanoes, magma and lava

Status of this page: `stage 2 (constants checked against sources on 2026-10-02, package WP-2)`.

Ledger ids on this page: `VOL-001` to `VOL-074` ([ledger/VOL.md](ledger/VOL.md)) and `MAG-001` to
`MAG-057` ([ledger/MAG.md](ledger/MAG.md)). Of these 131 rows (the 15 rows MAG-043 to MAG-057 of 2026-10-02 are all `game-choice`; the tallies in the next sentence are those of the first 116), 84 are `game-choice`, 25 `uncited`,
2 `verified`, 2 `abstract-only`, 1 `cited-unverified` and 2 `calibrated` (after stage 2: 81 `game-choice`, 28
`uncited`, 1 `calibrated`; stage 1: 47 `game-choice`, 63 `uncited`, 4 `cited-unverified`, 1 `calibrated`).
Ben's decisions of 2026-10-02 changed three defaults and added one setting (see "Decisions of 2026-10-02").

**Read this first.** Most of this feature is design. Stage 2 read the sources that exist for each
number and sorted the rows: 81 are rules or values with no Earth counterpart, or deliberate
departures from Earth, and are now labelled as such; 28 are physical values for which no source
was found or the source read does not give the number. No volcano size in the code comes from a
measurement of Earth's volcanoes, except the stratovolcano height range, which Ben set to Earth's
median on 2026-10-02 (VOL-019). Where a source was read, this page says how the code compares
with it. The one paper the code cited, for the spacing of arc volcanoes, says in its abstract that
there is no characteristic spacing; the citation is corrected. Two departures from Earth that shaped
worlds were settled by Ben on 2026-10-02 (see "Decisions of 2026-10-02"): stratovolcanoes stood about
one and a half times the height of Earth's typical one, and dead ocean volcanoes were sunk twice now
that the crust bends under them.

Scale used throughout: one block is 300 m across and 20 m up, so 1 km is 3.3 blocks across and
50 blocks up, and every slope is drawn 15 times steeper than it is.

## What you see in the world

**Arc volcanoes.** Lines of cones stand along the landward side of ocean trenches, where one plate
sinks under another, at least about 60 km (200 blocks) apart. They occur on continents and as island
chains. A cone is planned at 0.6 to 2 km tall (30 to 100 blocks; 1.2 to 3 km until 2026-10-02); in the runs
measured, live cones stood a median 1.0 km and all standing cones 0.87 km (Earth's median: 1.0 km). It has a concave
profile that steepens toward the top. At the default setting it is 7.6 to 20 km in radius (25 to 67
blocks), twice as wide as its physical shape, so that it does not read as a spike. It has a summit
crater at least 2 km (7 blocks) across and 240 m (7 blocks) deep, Earth's median depth (since
2026-10-22; before it 6 % of the cone's height, a block or two in the game). Its rock is andesite over tuff (volcanic ash turned to
rock), or rhyolite over tuff where the crust is thick.

**Gullied flanks** (since 2026-10-22, Ben's D2). A stratovolcano's summit stays smooth, but its
flanks are cut into radial gullies that run from below the summit down to its foot. A young cone
has many narrow ones (16 round it), sharp V-cuts between rounded ribs; an older one has fewer and
wider ones (8), broad valleys between sharp divides, cut deeper, as on Earth (O'Hara et al. 2024).
How much of the cone they take follows its age: about 16 % of its volume on a cone a few thousand
years old, 23 % on an active cone 2 Myr old, up to 40 % on old ones. Once the volcano is dead
they fade on the crater's clock while the mountains' own rugged texture takes the cone over. The
gullies are 8 or more blocks apart round the cone (the game cannot draw finer ones), wider than
Earth's 1 to 2 km.

**Shield volcanoes and island chains.** Over a hotspot (a fixed plume of hot mantle) under ocean
floor, a broad, gently sloping basalt shield rises 3.5 to 6.5 km above the sea floor, 49 to 80 km in
radius. Because the plate moves over the plume, a new shield starts about every 70 km and the older
ones go extinct, erode and sink, leaving a chain of islands and seamounts (drowned volcanoes) that
gets older away from the plume. Most shields do not reach the surface; in the reviewed 1,000 Myr run
only 3 islands and 8 seamounts stood at the end.

**Calderas.** Volcanoes on thick continental crust, and hotspot centres under continents, can
collapse after a large eruption into a caldera: a flat-floored basin with steep walls, 6 to 18 km
(20 to 60 blocks) across (less on a small cone: at most 0.6 of its radius) and up to 550 to 850 m
(28 to 42 blocks) deep, ringed by a sheet of tuff
up to 250 m thick that thins outward.

**Flood basalts.** A large igneous province becomes a basalt plateau 1,500 m (75 blocks) thick at
its centre, built of eight flows with thin ash beds between. Where the pile is at the surface, the
terrain is cut into steps 80 m (4 blocks) high.

**Magma.** Under each volcano that is still active at the present there is a magma system that runs
the whole depth of the world, as current volcanology describes real arc and hotspot systems (a
"trans-crustal" column of stacked storage bodies, not one chamber). At the top, 3 km (150 blocks)
below the crater floor, is the shallow chamber, a lens of lava (for example 19 across and 57 high
under a large arc cone, because heights are drawn 15 times larger than widths). A narrow column of
lava rises from it to just under the crater, capped by magma blocks. **Below it** a column of flat lava
lenses, each wider than tall and 5 to 11 blocks thick, hangs 28 blocks (980 m at 35 m a block) apart, joined by thin
vertical conduits, and widens with depth (the lowest lens is about twice as wide as the highest)
until a last lens sits on the lava floor at the bottom of the world, so the lava does not end. The
lenses sit in a body of altered hot rock (basalt with a magma block in about every 25), a little
broader than the lenses. Shields have broader lenses than arc cones, silicic centres a larger shallow
chamber. A quarter of the active volcanoes on dry land have an open lava lake in the crater instead
of the plug, 4 to 20 blocks across. Everything molten is walled in by smooth basalt, so lava never
touches water or an open cave: open space within 1.5 blocks of a chamber or lens, measured across and
up from its surface, becomes basalt (MAG-011; since 2026-10-03 the shell is that thick everywhere,
where before it was the ellipsoid's radii plus 1.5 and thinner than a block over the flanks of lenses
more than about 50 blocks in radius). A cave that reaches the narrow column from above leaves its
lava capped by magma blocks, except under a lava lake: there the column is lava all the way up into
the lake's floor, whatever the cave, walled in by its basalt sleeve (MAG-018; since 2026-10-17, when
a cave entrance in the crater of the test planet's shield, open to the sky down to y 46, 41 blocks
under the lake's floor, left the column an open shaft with the lake's lava hanging over it).
Extinct volcanoes have no magma.

**Lava elsewhere.** The random lava lakes and springs of vanilla Minecraft and of Terralith are
removed. The only other lava is a layer at the very bottom of the world (below y -502 in the default
world). On the measured planet a point on land was a median 658 blocks from the nearest active
volcano and 1,178 blocks from the nearest open lava lake.

## The model in plain words

The volcano stage runs once per plate step (about 2 Myr), after the surface stage, and keeps a
catalogue of individual volcanoes that ride on their plates.

1. **Where arcs are.** The plate model flags cells above a subducting plate as "arc". The stage
   turns that into an arc magma flux between 0 and 1: the convergence speed divided by 80 mm/yr,
   capped at 1, and multiplied by 0.7 on oceanic crust (VOL-003, no source found; most arcs in a run
   reach 1). It then thins the flagged band to its centre line, the volcanic front (VOL-004).
   A step in which the plate model resampled its crust, which clears the flags, says nothing
   about the arcs: no volcano is born or dies in it (VOL-011). On a lattice coarser than the arc
   band the plate model flags each slab strip's sample nearest the arc's axis, so a planet with
   subduction always has arcs (PLT-043, since 2026-10-19; [plates.md](plates.md)).
2. **Birth of arc volcanoes.** On the front, wherever the flux is at least 0.25, a new
   stratovolcano (a steep cone of lava and ash) is placed if no other cone stands within 60 km
   times a random factor of 0.8 to 1.2 (VOL-002, VOL-039). Its planned height runs from 0.6 km at
   the threshold flux to 2 km at full flux (VOL-019); its radius follows height to the power 0.8
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
   been too low for two steps (VOL-010). An extinct volcano on ocean crust sinks with the flexure of the
   plate under its weight and the plate model's deepening of old ocean floor; the setting
   `island_subsidence_m_myr` adds a fixed rate on top (up to 1.5 km in all), 0 by default since
   2026-10-02 (VOL-048).
   It leaves the catalogue when worn below a tenth of its peak height, or after 250 Myr (VOL-049),
   or when its plate carries it into a trench (VOL-050).
7. **Calderas.** A silicic cone collapses into a caldera with a chance of 0.04 per Myr (VOL-045); a
   continental hotspot centre with its own rate, 1 per Myr while it is active (VOL-074; 0.12 until
   2026-10-02), which ends the centre, so the hotspot builds a new one once the old has drifted away. The radius is drawn between 3 and 9 km
   (VOL-029) and the depth is half the cone's height, capped at 400 m plus 5 % of the radius
   (VOL-030). Half of the removed rock is laid around it as a tuff sheet (VOL-046). The volcano
   then dies.
8. **Flood basalts.** Each large igneous province the plate model creates is laid as eight flows,
   88 % basalt and 12 % tuff each (VOL-052, VOL-060). Since REV-2 (2026-10-19) the strata log
   marks these flows with their own provenance (`LIP_FLOW`), so they can be told from shield and
   arc basalt after burial, erosion and rifting; the geologic provinces count a column as a large
   igneous province by them alone (PRV-006).
9. **Block scale.** The amplifier restores each volcano's true profile, which the coarse grids
   blur, and adds the crater or caldera floor. Sharp detail on an extinct volcano fades with a time
   constant of 5 Myr (VOL-063 to VOL-072). Since 2026-10-22 the cone is laid on the ground from
   its vent (onlap, VOL-080, VOL-081): within 0.4 of its radius it stands on the level of the
   ground at its vent, burying lower ground, so its summit is the vent even on a mountainside; from
   there to its foot the fill fades and the cone lies on the ground beneath it, and ground that
   rises above the cone shows through. Before, the profile was added to sloping ground, so the
   highest point of a cone on a range's flank sat up the slope from its vent (Ben's volcano 27476,
   2026-10-22: 35 blocks up the slope and 30 blocks above the vent column).
   **Gullies.** After the world map is drawn, the bake cuts each standing stratovolcano's flanks
   into radial gullies on the world's own grid, in world blocks (VOL-100 to VOL-105). It reads the
   cone from the drawn terrain: its foot is the mean height of the ring 1.0 to 1.3 radii out, its
   top the highest point within 0.2. The upper 30 % of its height over the foot and its crater keep
   no gullies. Below, 8 to 16 basins tile the cone divide to divide, each cut deepest on its axis;
   the cut's depth is scaled so the cuts take O'Hara's eroded share of the cone's volume over its
   foot for its age (the log-mean of the times since its first and its last activity; an active
   cone last erupted 1 kyr ago). Dead cones' cuts fade by e^(−dead / 5 Myr). Nothing is cut below
   one block over the sea, at a lake or a river, or under ice, so the coast and the closed basins
   do not change.
10. **Magma in the game.** The bake lists the volcanoes that are still active. The Minecraft side
    gives each a chamber (an ellipsoid whose radius is a tenth of the volcano's, between 1.2 and
    4.5 km), a conduit and a plug, and gives the most active quarter of those that can hold one a
    crater lava lake (MAG-001 to MAG-026). Below the chamber, down to the lava floor, each also gets a
    crust-deep column (MAG-043 to MAG-056, Ben's decision of 2026-10-02): up to `magma_lens_count`
    (24) lava lenses about `magma_lens_spacing_blocks` (28) apart, wider than tall (half-height one
    fifth of the radius, 2 to 5 blocks), the lowest about 2.2 times as wide as the highest, joined by
    dike-like conduit segments, in a body of altered hot rock whose width is `magma_mush_width`.
    Lens radius is the chamber's radius times `magma_lens_width` (1.5) times a kind share: shields 1.4, arc cones 1.0, silicic centres 0.9 (their
    shallow chamber is 1.6 times larger), and quiet volcanoes' lenses are down to 0.7 of an active one's.
    The column's bottom comes from the level's own `min_y`, not from a number.

## Equations and constants

Shapes. `ρ` is the distance from the centre divided by the basal radius `R`; each profile is the
fraction of the full height `H` at that distance.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| VOL-016 | Stratovolcano profile | (1 − ρ)^1.5: steepest at the summit, 1.5 H/R; mean slope 0.8 H/R | – | none; compared with Grosse et al. 2014 | `uncited` |
| VOL-017 | Shield profile | (1 − ρ²)^1.5: steepest at ρ = 0.71, 1.5 H/R | – | none; compared with Clague & Sherrod 2014, Grosse et al. 2014 | `uncited` |
| VOL-018 | Silicic centre profile | (1 − ρ²)² | – | none | `uncited` |
| VOL-019 | Stratovolcano height | 0.6 to 2.0 (1.2 to 3.0 until 2026-10-02), linear in arc flux | km | Grosse et al. 2014 gives Earth's distribution; Ben, 2026-10-02 | `calibrated` |
| VOL-020 | Stratovolcano radius | 10 × (H / 2 km)^0.8 ÷ flank steepness (14 × (H / 3 km)^0.8 until 2026-10-02) | km | Grosse et al. 2014 for the height-to-width ratio; Ben, 2026-10-02 | `game-choice` |
| VOL-021 | Shield height above the sea floor | 3.5 to 6.5, by plume strength | km | none; USGS for Mauna Loa | `uncited` |
| VOL-022 | Shield radius | 40 × (H / 6.5 km)^0.8 ÷ flank steepness | km | none | `uncited` |
| VOL-023 | Continental hotspot centre | 0.9 tall, 35 radius | km | none; Christiansen 2001 for Yellowstone | `uncited` |
| VOL-024 | Flank steepness | 0.5 (radius doubled) | – | Ben, decisions log 2026-09-30 | `game-choice` |
| VOL-026 | Summit crater depth | 240 m, at most 0.37 H (VOL-090); 0.06 H until 2026-10-22 | m | Grosse et al. 2014 (Earth's median); Ben, 2026-10-22; Pike & Clow 1981 for the cap | `verified` |
| VOL-091 | Summit crater radius | max(1 km, 0.07 R) | km | agent, for visibility; Grosse et al. 2014 and Pike & Clow 1981 compared | `game-choice` |
| VOL-100 | Gullies' summit zone | none above 0.70 of the cone's height over its foot, full below 0.55; none in the crater | share of height | O'Hara et al. 2024 (summit region the upper 30 %); Ben, 2026-10-22 (D2) | `game-choice` |
| VOL-101 | Eroded share of a cone's volume | 5.36 log10(age, yr) − 1.56 %, kept within 5 to 40 %; age the log-mean of first and last activity | % | O'Hara et al. 2024, Fig. 4h | `verified` |
| VOL-102 | Last eruption of an active cone | 1 | kyr | agent | `game-choice` |
| VOL-103 | Gully width angle and count | 20° at 10 % eroded to 50° at 35 %; 360° over it in fours, 8 to 16 | degrees, count | O'Hara et al. 2024, §4.1, §4.3 | `verified` |
| VOL-104 | Gully cross-profile and drawing rules | (1 − 2d)^p, p 1.5 young to 0.7 old; onset at 8 to 12 blocks' spacing; axis at most 0.8 of the height; masks | – | agent | `game-choice` |
| VOL-105 | A dead cone's gullies | × e^(−dead / 5 Myr) | – | agent, after the 2026-09-30 decision | `game-choice` |
| VOL-029 | Caldera radius | uniform 3 to 9, at most 0.6 R | km | none; Christiansen 2001, USGS compared | `uncited` |
| VOL-030 | Caldera depth | min(0.5 H, 400 m + 0.05 × radius) | m | none; Grosse et al. 2014 compared | `uncited` |
| VOL-031 | Shield summit caldera | radius 0.08 R, depth min(0.04 H, 250 m), on shields over 4 km | m | none | `uncited` |
| VOL-032 | Summit ceiling | 6.8 above the sea; × 0.65 for shields; scaled by crust strength over gravity | km | Earth's highest volcano, secondary source | `abstract-only` |

The slopes of the profiles, worked out from the code: a 2-km stratovolcano over its physical
10 km radius is 17° at the summit and 9° on average; at the default doubled radius 8.5° and 4.6°. The
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
| VOL-045 | Caldera eruption rate | 0.04 (silicic cone; tall shield, summit caldera) | per Myr | none | `uncited` |
| VOL-074 | Caldera eruption rate of a continental hotspot centre | 1.0 (0.12 until 2026-10-02) | per Myr | Christiansen 2001 compared; Ben, 2026-10-02 | `game-choice` |
| VOL-048 | Extra subsidence of extinct ocean volcanoes | 0 (25 until 2026-10-02), up to 1,500 m in all | m/Myr | Clague & Sherrod 2014 compared; Ben, 2026-10-02 | `game-choice` |
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
| Height of stratovolcanoes | median 1.02 km, mean 1.13 km, s.d. 0.55 km, range 0.17–4.57 km (512 subaerial stratovolcanoes) | Grosse et al. 2014, table 1, full text | aimed at 0.6–2.0 km; standing cones measured at a median 0.87 km, live ones 1.04 km (six runs below); with the old 1.2–3.0 km: 1.48 and 1.88 km |
| Height to basal width of stratovolcanoes | median 0.13, range 0.02–0.30; 0.11 or less over 20 km wide | Grosse et al. 2014, table 1 and p. 7 | 0.08–0.10 at the physical radius, 0.04–0.05 at the default (0.09–0.11 and 0.045–0.054 until 2026-10-02) |
| Basal width of stratovolcanoes | median 7.9 km, range 2–66 km | Grosse et al. 2014, table 1 | 7.6–20 km physical, 15–40 km at the default (13–28 and 27–56 km until 2026-10-02) |
| Flank slopes of stratovolcanoes | mean flank 17.7°, steepest average 27.7° at 0.79 of the height (medians) | Grosse et al. 2014, table 1 | 9° mean and 17° at the summit for 2 km over 10 km; half that at the default |
| Shield flanks | lava flows dip 2°–10°; subaerial shields' mean flank 8.1° (median of 89) | Clague & Sherrod 2014, p. 114; Grosse et al. 2014, table 1 | 14° at the steepest for the tallest physical shield, 7° at the default |
| Large summit craters and calderas on composite volcanoes | 0.6–11 km across, median 2.2 km; depth under 100 to 860 m, median 240 m (smaller craters not measurable) | Grosse et al. 2014, p. 8 | craters at least 2 km across, 2.8 km on a 2-km cone, 240 m deep (36–120 m until 2026-10-22); calderas 6–18 km across (smaller on a small cone), up to 550–850 m deep |
| Spacing of arc volcanoes | no characteristic spacing: spacings follow a Gamma distribution, as random points do | de Bremond d'Ars et al. 1995, abstract only | a least distance of 60 km, so cones stand evenly |
| Mean spacing of large arc centres | 27 km (Alaska Peninsula), 42 km (Cascades, Andean Southern Volcanic Zone); 87 km between the 17 largest Cascade centres | Hildreth 2007, tables 4 and 10, full text | 60 km least (the mean spacing a run makes was not measured) |
| Lifespan of large arc centres | mostly 0.1–1 Myr | Hildreth 2007, p. 64 | 2–8 Myr (a game choice: one cone stands for a centre over a 2-Myr step) |
| Mauna Loa above the sea floor | about 9 km (4.17 km above the sea, 5 km of submarine flank), the sea floor pushed down another 8 km | USGS Mauna Loa page, fetched | shields 3.5–6.5 km, summits at most 4.4 km above the sea |
| Hawaiian growth | principal growth in the first 1–2 Myr | Clague & Sherrod 2014, abstract | growth time constant 1 Myr; activity window 0.7–2.8 Myr at 25–100 mm/yr |
| Subsidence of Hawaiian volcanoes | more than 1 km in about 1 Myr under the growing load (2.6 m/kyr off Hawaii), then about 10 m/Myr as the plate cools | Clague & Sherrod 2014, pp. 130–131 | flexure under the load (since 2026-10-02) and the plate model's ocean floor; a fixed extra rate after death is a setting, 0 by default since 2026-10-02 (25 m/Myr before) |
| Yellowstone | plateau about 2,400 m against 1,600–1,900 m beside it; caldera 45 by 85 km; caldera-forming eruptions at 2.1, 1.3 and 0.64 Ma | Christiansen 2001, full text | silicic centre 0.9 km tall; caldera radius at most 9 km; a centre collapses at 1 per Myr while active (0.12 until 2026-10-02) |
| Caldera sizes | Crater Lake a basin 8 by 10 km across; Long Valley 16 by 32 km | USGS "Geology of Crater Lake National Park" (read by WP-15); USGS Long Valley page (stage 1) | 6–18 km across |
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
| Active volcanoes on dry land whose highest column within 0.35 radii is at the vent (within 1.6 crater radii or 2 blocks of it; VOL-082) | 0.19 → 0.68 (Ben's preset); earthlike seeds 1 to 3: 0.14 → 0.55, 0.18 → 0.43, 0.10 → 0.57. Median offset of that column from the vent 17.2 → 5.3 blocks (Ben's preset), 19.5 → 7.0, 25.4 → 21.6, 18.7 → 6.2; median height of it over the vent column 10.3 → 2.3 blocks (Ben's preset) | `deeptime volcano-census` on Ben's preset (λc pinned at 10.5) and `earthlike_quick_16k` seeds 1 to 3, scattered continents, 500 Myr, 2026-10-22, master fe61452b against the onlap (VOL-080); the simulation byte-identical, only the bake differs |
| Visible summit craters on dry land (VOL-082: the lowest of 8 sector rims at least a block above the crater floor) | 0.00 to 0.08 of them, before and after the onlap; measured crater depth (mean sector rim over the floor) a median 1.0 to 2.2 blocks after it, against the catalogue's 2.2 to 3.0 (VOL-026: 0.06 H) | same |
| Visible summit craters with the crater at Earth's median depth (VOL-026 since 2026-10-22: 240 m, 6.86 blocks, at most 0.37 of the cone) | all active volcanoes, seeds 1 to 3: 0.03 → 0.53, 0.03 → 0.41, 0.05 → 0.55 of 180, 115 and 119 craters; on dry land 0.05 → 0.69, 0.05 → 0.41, 0.00 → 0.64 of 61, 39 and 42 (64, 39 and 42 before: three vents of seed 1 now sit at the sea). Measured depth (mean sector rim over the floor), median 1.32 → 4.01, 1.65 → 3.80, 1.34 → 3.90 blocks (all) | `deeptime volcano-census` on `earthlike_quick_16k` seeds 1 to 3, scattered continents, 500 Myr, 2026-10-22: master 10bafdae against the D7 branch, each branch's `deeptime resume` of the same last checkpoint (the bake and the projection again); the simulation byte-identical. The drawn depth is about 4 of the bowl's 6.86 blocks: the game draws the terrain from corners 4 blocks apart and the crater is 7 to 9.5 blocks across; the craters still hidden (one rim sector at or under the floor, median -0.2 blocks) sit on tilted ground and off-centre summits: their median tilt ratio (VOL-082) 1.4 to 1.7 against the visible ones' 1.05 to 1.1, 74 to 77 % of them above 1 (53 to 62 %), their highest column a median 21 to 25 blocks from the vent (6 to 10), so the ground's fall, not the bowl's depth, hides them (a script over the census rows) |
| Active volcanoes at the end, the GameTest planet | 0 on master (every run; 4 hotspot shields before the solvers merge); since the arc fix 5 to 12 (3 to 10 arc stratovolcanoes, 0 to 3 shields) over seeds 11 to 16 | `planet_world_4k`, 150 Myr, 2026-10-19 ([plates.md](plates.md), "Arcs on a lattice coarser than the arc band") |

Stage 2 runs (WP-2, 2026-10-02): `earthlike_quick_16k`, 500 Myr, seed 1, both starts (scattered
continents / supercontinent), `deeptime stage volcanoes`, two threads, on the test VM, built from
master 29d493f plus a report of cone heights (commit 360b2ed; nothing else in the simulation
differs from master, flexure on as in the preset). Command: `tools/review/p1-17-hyps.sh 500 "1"
"scattered_continents supercontinent" base "h:volcanoes.arc_height_km=[0.6,2.0]"
"sub0:volcanoes.island_subsidence_m_myr=0"`. Heights of cones are the 10th, 50th and 90th
percentiles at the end of the run; "standing" cones are live or dead ones still at a quarter of
their peak and at least 300 m. One run per start, and single runs scatter widely: four runs of
the same settings in the flexure build log (2026-10-02) gave land above 2 km from 0.092 to 0.149,
so only large differences mean anything.

| Quantity | Earth | Master defaults | Arc cones 0.6–2.0 km (radius unchanged) | No extra island subsidence |
|---|---|---|---|---|
| Land share | 0.289 | 0.252 / 0.284 | 0.246 / 0.266 | 0.285 / 0.280 |
| Median land height, m | 427 | 569 / 650 | 547 / 787 | 694 / 872 |
| Land above 2 km | 0.102 | 0.092 / 0.149 | 0.079 / 0.156 | 0.091 / 0.134 |
| Land above 3 km | | 0.018 / 0.042 | 0.011 / 0.025 | 0.014 / 0.018 |
| Active stratovolcanoes at the end | | 115 / 168 | 8 / 128 | 46 / 203 |
| Their heights, p10 / p50 / p90, m | median 1,020 (all stratovolcanoes, Grosse et al. 2014) | 1,345 / 1,869 / 2,814; 1,437 / 1,831 / 2,908 | 979 / 1,433 / 1,900; 826 / 1,226 / 1,890 | 1,274 / 1,947 / 2,789; 1,389 / 1,995 / 2,834 |
| Standing stratovolcanoes | | 974 / 920 | 482 / 732 | 741 / 1,382 |
| Their heights, p10 / p50 / p90, m | median 1,020, mean 1,130 | 634 / 1,471 / 2,338; 823 / 1,524 / 2,588 | 514 / 929 / 1,682; 582 / 914 / 1,540 | 668 / 1,318 / 2,079; 853 / 1,486 / 2,325 |
| Standing shields (heights p50, m) | | 5 (2,824) / 14 (2,669) | 8 (2,755) / 3 (3,130) | 7 (1,976) / 11 (3,465) |
| Calderas formed (and summit calderas) | | 545 (22) / 524 (26) | 552 (23) / 512 (20) | 492 (15) / 510 (16) |
| Tallest volcano, m | 4,570 (subaerial, Grosse et al. 2014) | 4,360 / 4,461 | 4,424 / 5,694 | 4,348 / 4,661 |
| Relief built by volcanoes, km³ (block metric) | | 2.4 × 10⁷ / 2.5 × 10⁷ | 2.2 × 10⁷ / 1.8 × 10⁷ | 2.1 × 10⁷ / 2.7 × 10⁷ |

What the runs show:

- **The cones are taller than Earth's.** At the defaults, standing stratovolcanoes have a median
  height of 1.5 km and active ones 1.8–1.9 km, against Earth's 1.02 km. With the range lowered to
  0.6–2.0 km the standing cones' median is 0.91–0.93 km, Earth's figure.
- **Lower cones did not measurably lower the land above 2 km** in this pair of runs (0.092 →
  0.079 on one start, 0.149 → 0.156 on the other): the change is inside the run-to-run scatter.
  The build log's earlier figure (land above 2 km 0.307 with volcanoes, 0.170 without, before the
  hypsometry recalibration and before flexure) is no longer the state of master: with flexure the
  defaults give 0.092 and 0.149.
- **The extra island subsidence makes no measurable difference** to the hypsometry or to how many
  shields stand at the end.

Runs for the volcano decisions (2026-10-02): the same preset, length and tools, seeds 1 to 3 and both
starts, six runs per variant, built from the commit that carries the new defaults; the old values
are set by `--set` (`volcanoes.arc_height_km=[1.2,3.0]`, `arc_radius_km=14`,
`island_subsidence_m_myr=25`, `hotspot_caldera_rate_per_myr=0.12`), which reproduces the WP-2 master
numbers of seed 1 exactly (land 0.252 / 0.284, standing cones 1,471 / 1,524 m, 545 / 524 calderas).
Command: `tools/review/p1-17-hyps.sh 500 "1 2 3" "scattered_continents supercontinent" new
"old:volcanoes.arc_height_km=[1.2,3.0];..."`. Mean of the six runs, with the range in brackets.

| Quantity | Earth | Old defaults (master) | New defaults |
|---|---|---|---|
| Land share | 0.289 | 0.295 (0.252-0.327) | 0.266 (0.185-0.336) |
| Median land height, m | 427 | 687 (569-832) | 559 (478-625) |
| Land above 2 km | 0.102 | 0.124 (0.092-0.149) | 0.077 (0.062-0.097) |
| Standing stratovolcanoes' median height, m | 1,020 | 1,484 (1,389-1,627) | 869 (764-994) |
| Active stratovolcanoes' median height, m | 1,020 | 1,875 (1,634-2,146) | 1,044 (835-1,252) |
| Standing stratovolcanoes at the end | | 836 (670-974) | 625 (499-788) |
| Relief built by volcanoes, km³ (block metric) | | 2.4 × 10⁷ (2.2-2.6) | 1.2 × 10⁷ (1.1-1.4) |
| Calderas formed in all | | 512 (415-562) | 496 (384-568) |
| of them at continental hotspot centres | | 42 (25-68) | 62 (18-102) |
| of them elsewhere (arc cones) | | 469 (390-526) | 433 (366-493) |
| Summit calderas on shields | | 22 (17-26) | 16 (9-22) |
| Silicic hotspot centres born / share that collapsed | | 64 / 66 % (253 of 382) | 63 / 98 % (374 of 380) |

What these runs show:

- **The standing cones' median is 869 m, 85 % of Earth's 1,020 m**, inside one standard deviation
  (Earth's s.d. is 550 m); the cones still active, whose heights are not yet lowered by erosion, are
  at 1,044 m. With the old range they were at 1,484 m and 1,875 m.
- **The land above 2 km fell below Earth's**: 0.077 against 0.124 (Earth 0.102), in all six
  pairs of runs (by 0.004 to 0.072; the smallest is seed 3, supercontinent: 0.090 against 0.094). The median land height fell from 687 m
  to 559 m (Earth 427 m). The two figures now straddle Earth's instead of both standing above it;
  the hypsometry is recalibrated in a separate task, with the plate work (decisions log 2026-10-01).
  These runs change the cone heights, the island subsidence and the hotspot caldera rate together,
  and the WP-2 runs that isolated the first two found no measurable effect from either alone, so
  most of the fall is probably the lower cones; this was not separated again.
- **Nearly every continental hotspot centre now collapses** (98 % against 66 %), because a centre
  stays active for several Myr over a slow plate, so even the old 0.12 per Myr collapsed two in
  three; WP-2's estimate of a 10-30 % chance had assumed a life of 1-3 Myr. The hotspot calderas
  rose from 42 to 62 per run (about 1.5 times) rather than tenfold, because the centre dies at its
  collapse and a new one forms only after the plate has carried the old one away. Arc calderas
  fell by 8 %, with the shorter cones (a cone under 300 m cannot collapse).
- **Standing shields**: 4.5 on average (0-9) against 7.2 (4-14), not separable from run-to-run
  scatter and from the changed hypsometry.

## Simplifications and limits

- **Gullies are drawn, not eroded.** The flank gullies (VOL-100 to VOL-105) are a pattern sized
  by O'Hara et al. 2024's eroded volume against age, not a drainage network: no channel follows
  the ground's flow, the basins are even round the cone, and the eroded rock goes nowhere (no
  fans at the foot). Only stratovolcanoes get them (O'Hara's sample): shields and silicic centres
  are left as they were. They are at least 8 blocks (2.4 km) apart, since the game draws the
  terrain from corners 4 blocks apart; Earth's young cones carry channels about 1 km apart
  (drainage density about 1 km⁻¹), which the game cannot show. A Lambert world (no projection
  pass) gets none.
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
  speed of subduction. On a lattice coarser than the arc band (the 4k test planet's 5,000 samples)
  the flag is one sample per slab strip, about 320 km wide: seed 11 of the 4k planet ends with 3
  active arc cones against 19 on the same planet at 20,000 samples (3 to 10 over seeds 11 to 16).
- **Lengths do not shrink with the planet.** A volcano's radius is in real kilometres on a planet
  an eighth of Earth's size, so volcanoes are large for their world. Flood basalt provinces, by
  contrast, are scaled to the planet (about 110 km in radius at 16k).
- **Magma chambers are one fixed shape** at one depth, whatever the volcano's size, age or kind (the
  chamber's size follows the volcano's radius; the crust-deep column below it follows kind, radius and
  activity).
- **The crust-deep column is a picture, not a model.** Edmonds et al. (2019, read as a fetched page,
  not the full text) say arc systems are vertically extensive, stacked, and dominated by mush, with
  thin sills (their model: under 10 m); the game draws lenses 40 to 100 m thick because a block is
  20 m tall, spaces them by a setting, and makes them lava where the papers describe mostly
  crystal-rich mush with 5 to 15 percent melt. The count, spacing, widths and the widening with depth
  are the game's, set so that a volcano is a place with a great deal of lava below it. Nothing in the
  column is simulated: no melt fraction, no heat, no eruption, no cooling.
- **Neighbouring columns join.** Each active volcano's column is drawn on its own, and where two
  stand close their lenses and shells overlap: on the test planet of 2026-10-19 the shields 46 and
  48, 12 km apart, overlap so far that about 650,000 lava blocks in the box of one are lenses of the
  other. The pass never lets a shell or the altered rock replace lava, so the overlap is one body
  of lava sealed by the outer shells (the GameTest `magmaUnderActiveVolcanoes` counts it apart as
  `otherBody`); nothing models a shared plumbing system.
- **The volcanoes and the share of high land.** With the old, taller cones the volcanoes pushed the
  share of high land above Earth's (land above 2 km 0.124 on average, Earth 0.102); with the cones
  at Earth's heights they build half the relief (1.2 × 10⁷ against 2.4 × 10⁷ km³ in block metric)
  and the share falls below Earth's (0.077). Neither is calibrated: Ben kept volcano density at 1
  (decisions log 2026-10-01) and the hypsometry is recalibrated later with the plate work.
- **Activity saturates.** Most arc volcanoes have the highest activity value, so which of them get
  a crater lake is decided by their catalogue number, not by anything physical.
- **Hardwired couplings.** The hotspot strength scale (VOL-008) mirrors the plate model's random
  plume strength range in another file, and the ceiling's normalisation (VOL-034) repeats the plate
  model's default crust strength. If either changes alone, volcano heights shift silently.

## Game choices

| What | Why | Who and when |
|---|---|---|
| Which kinds exist (arc stratovolcanoes, hotspot shields and chains, calderas, flood basalts) | scope for version 1 | Ben, decisions log 2026-09-29 |
| Stratovolcano height 0.6–2.0 km, radius 10 km at the top (VOL-019, VOL-020) | Earth's median stratovolcano is 1.02 km (Grosse et al. 2014); the proportions of the old defaults are kept | Ben, decisions log 2026-10-02 ("Lower to Earth's range") |
| Extra island subsidence 0 by default (VOL-048) | flexure and the plate model's ocean floor already sink old islands; the setting stays | Ben, decisions log 2026-10-02 ("Set it to 0, keep the setting") |
| Continental hotspot centres collapse at about 1 per Myr (VOL-074) | Yellowstone had three caldera-forming eruptions in about 1.5 Myr (Christiansen 2001); the rate is a new setting | Ben, decisions log 2026-10-02 ("About 1 per Myr") |
| Flank steepness 0.5: every volcano twice as wide as its physical profile (VOL-024) | at 15 times vertical exaggeration a physical cone is a spike | Ben, decisions log 2026-09-30 ("Setting, gentler default") |
| Young volcanoes are smooth; old ones regain rugged ground (VOL-068) | looks | Ben, decisions log 2026-09-30; for cone flanks changed 2026-10-22 (next row) |
| A smooth fresh summit with radially gullied older flanks (VOL-100 to VOL-105) | looks: a cone reads as a volcano its age, as O'Hara et al. 2024 measured them | Ben, decisions log 2026-10-22 (D2); the drawing rules, the 1 kyr last eruption and the stratovolcanoes-only scope the agent's |
| Small arc segments counted on their subduction event (VOL-015) | event volume | Ben, decisions log 2026-09-30 ("Count small ones") |
| Volcano density 1 (VOL-012) | kept although it raises the land | Ben, decisions log 2026-10-01 |
| Lava only at active volcanoes; random lava lakes and springs removed (MAG-001, MAG-034) | "only at active volcanoes, ridges and hotspots" is where lava is on Earth | Ben, decisions log 2026-10-01 |
| The lava floor at the bottom of the world (MAG-027) | a Minecraft convention with no counterpart on Earth; kept as a setting until decided | open; Ben, decisions log 2026-10-01 |
| A crust-deep column of stacked lava lenses in altered hot rock under every active volcano (MAG-043 to MAG-056) | "the amount of lava inside of them is truly pathetic": the old system stopped 150 blocks down; real arc systems are vertically extensive and stacked (Edmonds et al. 2019; Cashman et al. 2017, not opened) | Ben, decisions log 2026-10-02 ("Crust-deep magma system (Recommended)") |
| One chamber shape at one depth, 3 km (MAG-002 to MAG-005) | a body a player can find under every active volcano; 3 km lies inside Earth's range | agent, build log 2026-10-01, under Ben's "Volcanoes is a go ahead" |
| A quarter of able volcanoes have an open lava lake (MAG-019), though persistent lava lakes are rare on Earth | "a player should be able to find one" | agent, build log 2026-10-01 |
| Vanilla's cave carvers are kept 6 blocks from the underwater face of a sea cliff (MAG-057) | the carvers cut arches into the underwater base of the sea cliffs beside a volcano; the arches are an artefact of the carvers, not geology | Ben's report, 2026-10-02 (the setting is `sea_cliff_skin_blocks`; 0 gives vanilla's carving) |
| Lava is sealed in smooth basalt and never touches water (MAG-010, MAG-011, MAG-041) | lava pools were found overlapping the ocean | agent, build log 2026-10-01, after Ben's report |
| Arc volcanoes at least 60 km apart (VOL-039) | keeps the doubled cones apart; one cone stands for a large long-lived centre | agent, build log 2026-09-30; reviewed WP-2 2026-10-02 |
| Summit crater at least 2 km across (VOL-091) | a typical real crater (0.5 km, Pike & Clow 1981) would be under two blocks wide; the drawn one is of Grosse et al. 2014's size class, so it takes their median depth, 240 m (VOL-026) | agent, build log 2026-09-30; the depth Ben, 2026-10-22 |
| A cone is laid on the ground from its vent: the vent's ground level inside 0.4 of its radius, fading to its foot (VOL-080, VOL-081) | a cone is built by eruptions from its vent, so its summit is the vent; how far a real cone's onlap reaches down a slope was not read in a source | agent, coordinator's call, 2026-10-22, after Ben's volcano 27476 |
| Stratovolcano lifetime 2 to 8 Myr (VOL-038) | one step is 2 Myr | agent, build log 2026-09-30 |
| Eight flows in a flood basalt pile (VOL-052), terraces 80 m high (VOL-036) | visible layering and steps | agent, build log 2026-09-30 |
| Implementation rules: the volcanic front, the flux threshold, merging, the blind step, retirement, rendering and fading (VOL-002, VOL-004, VOL-008 to VOL-011, VOL-027, VOL-028, VOL-040, VOL-042, VOL-043, VOL-047, VOL-049 to VOL-051, VOL-053 to VOL-055, VOL-061 to VOL-073) | how the stage turns plate data into cones and cones into blocks; no Earth counterpart | agent, build log 2026-09-30 and 2026-10-01; labelled WP-2 2026-10-02 |

## Decisions of 2026-10-02

Three questions of the stage-2 package were put to Ben and answered on 2026-10-02 (decisions log; items 4 and 5 are later answers of the same day):

1. **Stratovolcano heights** (VOL-019, VOL-020). "Lower to Earth's range (Recommended)": the
   range is 0.6 to 2.0 km instead of 1.2 to 3.0 km and `arc_radius_km` 10 km instead of 14, so the
   proportions stay. Measured: standing cones' median 869 m (Earth 1,020 m), old 1,484 m; see the
   runs above for the side effect on the land above 2 km.
2. **Island subsidence counted twice** (VOL-048). "Set it to 0, keep the setting (Recommended)":
   `island_subsidence_m_myr` defaults to 0.
3. **Continental hotspot calderas** (VOL-074). "About 1 per Myr (Recommended)": a new setting,
   `hotspot_caldera_rate_per_myr`, 1.0, replaces the factor 3 on `caldera_rate_per_myr`, which keeps
   its 0.04 for arc cones and shield summits. In the code's terms it is a chance per active centre
   per step (1 − e^(−rate·Δt)); a collapse ends the centre and the hotspot builds a new one once
   the old has drifted 70 km, so a hotspot track makes about one caldera per centre-life. Yellowstone's
   own pace was 0.8 and 0.66 Myr between its three (Christiansen 2001), 1.4 to 2 per Myr.

4. **Crust-deep magma system** (MAG-043 to MAG-057; Ben looking at a 3D preview of an active volcano:
   "the amount of lava inside of them is truly pathetic. It just ends after ~100 blocks down???";
   answer "Crust-deep magma system (Recommended)"). Before: a plus-shaped conduit from six blocks
   under the crater to one chamber 3 km (150 blocks) down, and nothing below. Now: stacked lava lenses
   in altered hot rock down to the lava floor (see "What you see in the world"). Lava blocks measured
   in generated worlds (16k quick planet, 1,000 Myr, MIC set, one seed): an arc stratovolcano under
   the sea, 4,769 before, 21,196 after (4.4 times); the shield with a crater lake, 43,060 before,
   423,171 after (9.8 times), one connected body in each, no face on water, the lake's 13 faces on the
   air the only ones open. Over all 121 volcanoes with magma, from the builder's own volumes: the
   median arc volcano 3,680 blocks before (chamber and conduit), 18,419 after (5.0 times), 6 to 8 lenses;
   the median shield 43,357 before, 344,409 after (7.9 times), 8 lenses; the planet 0.95 million before
   and 5.7 million after (6 times). Lens width 1.0 gave a median arc 2.8 times: too little for the
   decision's "several times", so the default is 1.5.
5. **Arches in underwater cliffs** (MAG-057; Ben's report with run 1a0fb48c797). Found by generating the
   same 255 x 255 box with one seed under four configurations: caves that open onto the open sea
   were 157,646 blocks in 318 bodies; with vanilla's carvers switched off 111,023 blocks remained, so
   the carvers made about 30 percent and the density function's own noise caves (cheese, spaghetti,
   entrances) the rest. The guard (`sea_cliff_skin_blocks`, 6) keeps a wall of rock between any cave
   and the water at a cliff of 45 degrees or more, under the sea: 31,148 blocks in 218 bodies after
   (a fifth); caves on gentle sea floors are untouched (vanilla's ocean caves). Open space in the whole
   box (air and water under ground) went from 320,379 to 280,039 blocks (13 percent less).

## Open questions

Not yet settled, for a later package or a geologist:

4. **Arc spacing is regular, Earth's is random** (VOL-039). A random spacing with the same mean
   would put some cones close together and leave long gaps, as in the Cascades. Not proposed
   now: with cones twice their physical width, close pairs would merge into ridges.
5. **The cone's profile** (VOL-016). Real stratovolcanoes are steepest a little below a flatter
   summit region (at 0.79 of the height, Grosse et al. 2014); the code's cone is steepest at the
   summit. The amplifier and the crater already flatten the top, so the difference in blocks is
   small; not measured.
6. **Crater and summit caldera sizes scale with the doubled radius** (VOL-091, VOL-031): a 3-km
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
    Courtillot & Renne list is taken at face value. That is the plate model's doing: it keeps about
    3 live hotspots against Earth's catalogues of 26 to 69, and half of its few births bring a
    plume head ([plates.md](plates.md), "Also found, not changed: the hotspot population",
    2026-10-19). With so few plumes most planets end with 0 to 3 active shields.
14. **Where the lava floor goes** is undecided, and so is whether a median 658 blocks to the
    nearest volcano is acceptable for players (STATUS.md, pending questions).

## Sources

Keys refer to [bibliography.md](bibliography.md).

| Key | Used for | How it has been read |
|---|---|---|
| Grosse2014 | stratovolcano and shield heights, widths, slopes; crater and caldera sizes (VOL-016 to VOL-020, VOL-026, VOL-029, VOL-030, VOL-091) | in full (repository copy of the submitted version), WP-2; p. 8 again 2026-10-22 |
| PikeClow1981 | typical stratocone craters: depth, width, depth over the cone's height (VOL-026, VOL-090, VOL-091) | table 2 read by a script from the library text, 2026-10-22 |
| Hildreth2007 | arc volcano spacing, the volcanic front, lifespans (VOL-004, VOL-038, VOL-039) | in full, WP-2 |
| ClagueSherrod2014 | Hawaiian growth, subsidence, shield slopes, reservoir depths (VOL-007, VOL-009, VOL-017, VOL-041, VOL-048, MAG-002) | in full, WP-2 |
| Christiansen2001 | Yellowstone: plateau height, caldera size, caldera-forming eruptions (VOL-006, VOL-023, VOL-029, VOL-045, VOL-046) | the summary, physiography and chronology sections, WP-2 |
| Poland2014 | Kilauea reservoir depths, size and shape (MAG-002 to MAG-004) | the reservoir sections, WP-2 |
| Bremond1995 | arc volcano spacing (VOL-039) | abstract only; the full text is free to read but refused scripted access |
| Edmonds2019 | trans-crustal, stacked, mushy magma systems; thin sills; hot lower-crustal zone (MAG-043, MAG-044, MAG-047, MAG-054) | a page fetch of the open PMC copy (PMC6335485), summarised by the fetch tool, not the PDF; WP of 2026-10-02 |
| Cashman2017, Sparks2019 | the same picture (Science 355:eaag3055; Phil. Trans. R. Soc. A 377) | named by the decision, not opened: no open copy was reachable from the test machine |
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
