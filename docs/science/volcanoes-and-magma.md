# Volcanoes, magma and lava

Status of this page: `stage 2 (constants checked against sources on 2026-10-02, package WP-2)`.

Ledger ids on this page: `VOL-001` to `VOL-074`, `VOL-080` to `VOL-082`, `VOL-090` and `VOL-091`,
`VOL-100` to `VOL-105` and `VOL-140` to `VOL-163` ([ledger/VOL.md](ledger/VOL.md)) and `MAG-001` to
`MAG-057` ([ledger/MAG.md](ledger/MAG.md)). The 24 lava-flow rows of 2026-10-22 (VOL-140 to VOL-163)
are 16 `game-choice`, 3 `verified`, 3 `from-memory` and 2 `abstract-only`; the crater rows of
2026-10-22 (VOL-090, VOL-091) are 1 `verified` and 1 `game-choice` (and VOL-026 became `verified`);
the cone-gully rows of 2026-10-22 (VOL-100 to VOL-105) are 2 `verified` and 4 `game-choice`. Of the 131 rows VOL-001 to VOL-074 and MAG-001 to MAG-057 (the 15 rows MAG-043 to MAG-057 of 2026-10-02 are all `game-choice`; the tallies in the next sentence are those of the first 116), 84 are `game-choice`, 25 `uncited`,
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
Earth's 1 to 2 km, so on the measured planets most cones get 8.

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
lava capped by magma blocks, and the open column above the cap basalt, except under a lava lake:
there the column is lava all the way up into the lake's floor, whatever the cave, walled in by its
basalt sleeve (MAG-018; since 2026-10-17, when a cave entrance in the crater of the test planet's
shield, open to the sky down to y 46, 41 blocks under the lake's floor, left the column an open
shaft with the lake's lava hanging over it). The column is five blocks across and each of its five
columns has its own ground, so until 2026-10-24 a cave that opened one of them deeper than the
others left water beside the others' lava: on the cube test planet a hole in the sea bed over a
submerged shield, 24 to 33 blocks deep over the conduit's columns, put sea water on 10 faces of its conduit's lava.
Extinct volcanoes have no magma.

**Lava flows** (since 2026-10-22, review fix 8; VOL-140 to VOL-163). The ground of each active
volcano carries the lava flows of its recent eruptions, as paint on the top block: lines one block
wide, two on gentle ground, that run downhill from just past the crater's rim (on a shield, half of
them from flank vents on two opposite rift zones), follow gullies, pool briefly in hollows, stop at
water and end in a small lobe. A shield's flows are 1.2 to 51 km long (about 4 to 173 blocks on the
q1 quick planet, 294.6 m a block), an arc or silicic cone's 0.5 to 12 km (about 2 to 41 blocks). A
fresh shield flow is blackstone (aa) or smooth basalt (pahoehoe), a fresh flow of a viscous cone
blackstone (blocky lava); older flows turn to tuff (grey with lichen), then to patches of coarse dirt
(brown, oxidised, with the first soil), then vanish under the biome's own ground. How long that
takes follows the rain: on a desert cone a flow stays black for about 15,000 years and shows for
250,000; on Hawaii's wet side it greys in about 12 years and is under forest in 150. So a dry cone
shows hundreds of flows and a wet one few or none. Measured on the q1 planet: the dry shield 12754
(44 mm of rain a year at its driest) shows 256 flows on 10,224 columns, 5 % of its base; cones at
about 280 mm show 100 to 150 flows on 3 to 6 %. The pack's own dark stripes in the volcanic biomes
(Terralith draws them as contours of a noise) are left out of Deep Time levels, so the dark lines on
a cone are its flows. Ice and snow lie over flows. A flow that reaches a salt flat (a playa in a
closed basin) lies over the salt, as a real flow metres thick stands above the flat's seasonal
flood (VOL-162). The paint changes no block's height.

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
11. **Lava flows in the game** (VOL-140 to VOL-163; the Minecraft side only, from the baked world
    data, nothing in the simulation). For each active volcano the game draws its eruptions back
    from the present as a Poisson process: 0.19 a year at full activity for a shield (Mauna Loa's
    34 eruptions since 1843, VOL-145), 0.01 for an arc or silicic cone (VOL-146, from memory), times
    the volcano's activity. Each eruption leaves one flow whose length is drawn log-uniform between
    bounds read from Earth's flows: basalt 1.2 to 51 km, viscous lava 0.5 to 12 km, so that the
    medians stand in Walker's ratio of 3.2 (VOL-140 to VOL-144). Drawing stops when the flows are
    older than a flow stays visible at the driest point on the cone, when the visible flows cover
    35 % of the basal disc (Mauna Loa's share in 1,000 years, VOL-152), or at 256 flows (VOL-153).
    A flow starts a block past the crest of the crater's rim (VOL-155) or, on a shield, half the
    time at a flank vent (VOL-154), and steps half a block at a time down the steepest descent of
    the height the generator draws, so it follows the gullies and the crater the amplifier carves.
    A smooth per-flow wander turns it up to 35° either side while the way still goes down. Where the
    way down turns back on itself it follows a gully's floor, or fills a closed hollow up to its
    own thickness (5 m for basalt, 40 m for viscous lava, VOL-157) and stops at the brim; it also
    stops at water and when its length is used up (VOL-156). Lengths, slopes and the descent are
    measured on the ground through the world's metric (from the projection's sphere, VOL-160), so
    the flows do not depend on the projection's frame. A long basaltic flow (one fed fast) is aa
    from its vent; a short one is pahoehoe until it crosses ground steeper than 8° (VOL-158, from
    memory). Each painted column then shows the flow's stage by its age against the rain there
    (VOL-147 to VOL-151): fresh, weathered, oxidised in patches, then covered. Younger flows lie
    over older ones. A volcano's flows are traced once, when any column within their reach is
    first asked for, from the volcano list, the drawn height, the water and the rain, so chunks and
    far terrain (Distant Horizons) show the same flows. In the volcanic biomes the pack's noise
    stripes of blackstone and smooth basalt are left out (VOL-163). On a cube world a flow runs on
    across the net's seams onto the next face, turned with it, and a volcano by an edge paints
    both faces; see [cube-world-game.md](cube-world-game.md), item 14.

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

Lava flows (paint). `a` is a flow's age, `f` and `v` the times a flow stays fresh and stays
visible at the column's rain.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| VOL-140 | Longest basaltic flow | 51 | km | Riker et al. 2009 (abstract): Mauna Loa 1859 | `abstract-only` |
| VOL-141 | Longest viscous flow | 12 | km | USGS Medicine Lake page (andesite under 12 km); USGS VHP (rarely over 8 km) | `verified` |
| VOL-142 | Shortest viscous flow | 0.5 | km | agent | `game-choice` |
| VOL-143 | Median basaltic flow length over a viscous one | 3.2 | – | Walker 1973 (abstract) | `abstract-only` |
| VOL-144 | Shortest basaltic flow, lengths log-uniform | 3.2² × 0.5 × 12 / 51 = 1.2 (medians 7.8 and 2.4 km) | km | derived from the rows above | `game-choice` |
| VOL-145 | A shield's flows at activity 1 | 34 / 179 = 0.19 | per yr | USGS Mauna Loa FAQ: 34 eruptions since 1843 | `verified` |
| VOL-146 | A viscous cone's flows at activity 1 | 0.01 | per yr | none read | `from-memory` |
| VOL-147 | Arid anchor (150 mm of rain or less) | f 15,000; v 250,000 | yr | Cima volcanic field, at second hand (Wikipedia; Wells et al. 1985 not read) | `from-memory` |
| VOL-148 | Kona anchor (380 mm) | f 1,500; v 3,000 | yr | HVO Volcano Watch 1996; Kinney et al. 2015 | `game-choice` |
| VOL-149 | Hilo anchor (3,800 mm or more) | f 12; v 150 | yr | HVO Volcano Watch 1996, 1999 | `game-choice` |
| VOL-150 | Between anchors; no climate | ln t linear in ln P; 1,000 mm where the world has no rain layer | – | agent | `game-choice` |
| VOL-151 | Stages | fresh for a < f; weathered for a < √(f v); oxidised, a column kept with chance 1 − 0.75 ln(a / √(f v)) / ln(v / √(f v)); covered for a ≥ v | – | HVO Volcano Watch 2006 (black, grey, brown) | `game-choice` |
| VOL-152 | Cover cap | 0.35 of π R² | share | USGS Mauna Loa FAQ: 35 to 40 % in 1,000 years | `game-choice` |
| VOL-153 | Flows a volcano | at most 256 | – | agent (a cost bound) | `game-choice` |
| VOL-154 | Flank vents of a shield | half its flows, on two opposite rift zones, ±25°, out to 0.5 R | – | agent | `game-choice` |
| VOL-155 | Summit start | 1 block past the crest of the crater's rim | blocks | agent | `game-choice` |
| VOL-156 | Path | half-block steps of steepest descent through the metric; wander ±35° on a 3-block lattice; gully floors followed; closed hollows filled | – | agent; DOWNFLOW (Favalli et al. 2005, Chevrel et al. 2021) compared | `game-choice` |
| VOL-157 | Fill depth of a hollow | basalt 5; viscous 40 | m | USGS FS 144-02 (aa 3 to 5 m); NPS (Trident andesite 25 to 60 m) | `verified` |
| VOL-158 | Pahoehoe to aa | long basaltic flows aa from the vent; others aa after ground over 8° | degrees | Peterson & Tilling 1980 (abstract); Rowland & Walker 1990 (search summary) | `from-memory` |
| VOL-159 | Width | 1 column; 2 on ground under 5°; toe lobe 1.25 blocks in radius | blocks | none found | `game-choice` |
| VOL-160 | Metric | G = JᵀJ from the projection's sphere, at the vent, scales within 4 times the nominal block | – | agent | `game-choice` |
| VOL-161 | Lava of a volcano | shields basaltic; arc and silicic cones viscous | – | agent | `game-choice` |
| VOL-162 | Blocks | aa and blocky blackstone; pahoehoe smooth basalt; weathered tuff; oxidised coarse dirt; a pack's tags may replace them | – | HVO Volcano Watch 2006 for the order of colours | `game-choice` |
| VOL-163 | The pack's stripes | in `#deeptime:volcanic` biomes, noise bands within ±1.5 that paint blackstone or smooth basalt are dropped | – | agent | `game-choice` |

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
| Gullies on stratovolcano flanks | eroded volume about 10 % at a few kyr, 30 to 40 % at 10 to 100 kyr and beyond (5.36 log10(age, yr) − 1.56 %); more than 10 summit basins on most cones; drainage density about 1 km⁻¹ young, 0.5 km⁻¹ old | O'Hara et al. 2024, Fig. 4h, §4.1 and §4.3, full text | 8 to 16 gullies at least 8 blocks (2.4 km) apart; the eroded share O'Hara's for the cone's age, capped at 0.8 of the cone's height on each gully's axis (measured below) |
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
| Longest lava flows | basalt: Mauna Loa 1859, 51 km; Giant Crater (Medicine Lake), 45 km; andesite under 12 km (Medicine Lake), rarely over 8 km (VHP); Trident andesite 2.5 to 4 km | Riker et al. 2009, abstract; USGS and NPS pages, fetched 2026-10-22 | basalt 1.2 to 51 km, viscous 0.5 to 12 km, log-uniform |
| Basaltic against viscous flow lengths | median 3.2 times | Walker 1973, abstract (Malin 1980, abstract: length goes better with volume than with effusion rate) | 3.2 by construction |
| Eruptions of a shield | Mauna Loa 34 since 1843, one in about 5 years | USGS Mauna Loa FAQ, fetched | 0.19 a year at activity 1 (0.12 on the q1 shield, activity 0.63) |
| Share of a shield under young flows | Mauna Loa 35 to 40 % under 1,000 years, 14 % under 150; Kilauea 90 % under 1,100 years | USGS Mauna Loa FAQ; USGS, Volcanoes of the United States; fetched | at most 35 %; measured 5.4 % on the q1 shield over 250,000 years (flows 1 or 2 blocks wide) |
| How fast flows weather | Kona (10 to 20 in a year): flows of 137, about 200 and over 1,500 years still bare and black; Hilo (100 to 200 in): a 115-year flow under large trees, lichen in 12 years, forest in under 150; dry land: flows under 300 years 94 to 96 % barren, tree height peaks at 1.5 to 3 kyr on aa | HVO Volcano Watch 1996 and 1999; Kinney et al. 2015; fetched | fresh 1,500 and visible 3,000 years at 380 mm; 12 and 150 at 3,800 mm |
| Desert flows | Cima (Mojave): Black Tank, 12 to 15 ka, still crisp; older flows mantled, the mantle most extensive at 0.25 to 0.75 Myr | Wikipedia and a search summary of Wells et al. 1985 (not read); Miller & Buesch 2022, abstract | fresh 15,000 and visible 250,000 years at 150 mm |
| Flow thickness | aa 3 to 5 m (Pu'u 'O'o); Trident andesite 25 to 60 m | USGS FS 144-02; NPS; fetched | 5 and 40 m, used only to fill hollows |
| Pahoehoe and aa | nearly all Hawaiian lava erupts as pahoehoe and some turns to aa with shear; aa above about 5 to 10 m³/s; either at any distance from the vent | Peterson & Tilling 1980, abstract; Rowland & Walker 1990, search summary; Oregon State Volcano World | long flows aa from the vent, others after ground over 8° |
| Flow widths | Pu'u 'O'o's channels 5 to 25 m | Heliker et al., search summary only | 1 or 2 blocks, 0.3 to 0.6 km |

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
| Cone gullies (VOL-100 to VOL-105) | standing cones 616, 399 and 558 (seeds 1, 2, 3), of them 7, 6 and 3 not stratovolcanoes; active stratovolcanoes gullied 67 of 240, 42 of 158 and 47 of 214, the rest at sea (153, 93, 154), under 4 blocks over their foot (17, 8, 9) or with no flank to cut (3, 15, 4); 8 gullies each (the median: the fewest, since the cones are small in blocks); the active cones' eroded share a median 0.117, 0.035 and 0.102 against O'Hara's 0.234, 0.242 and 0.234 for their age: 61 of 67, 40 of 42 and 42 of 47 held by the cap (a gully's axis cut at most 0.8 of the cone's height there), on cones a median 17, 22 and 23 blocks over their foot; deepest cut a median 7.0, 6.8 and 10.7 blocks (at most 32.8, 31.9, 29.4); dead stratovolcanoes gullied 113 of 369, 35 of 235 and 95 of 341 but hardly cut (eroded share a median 0.0, 0.0 and 0.003, deepest a median 0.06 to 0.4 blocks: the fade e^(−dead / 5 Myr) has taken them); 111,706, 37,361 and 120,749 cells cut | `earthlike_quick_16k`, scattered continents, 500 Myr, seeds 1 to 3, 2026-10-22, the bake's note `notes.bake.projection.relief.cone_gullies` of the D2 and D3 branch (fd231a0c), `deeptime resume` of each seed's last checkpoint |
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

Lava flows on the q1 quick planet (review fix 8, 2026-10-22). The planet: Petroff-Guyou projection,
circumference 13,312 blocks, 294.6 m a block across and 35 m up, sea at y 63, 181 active volcanoes.
Two previews of five boxes under the MIC set, one level seed (20261022) for both, on the test VM:
`tools/workbench/preview.py run --profile=mic --size=13312 --world-data=<q1> --y=20,280
--seed=20261022 --box=...`, before from master 7787c857 and after from this branch (commit
94f5c445). The flows' own counts are the game's log lines (one a volcano); block counts are the top
block of the ground under plants and snow layers, from the exported areas.

| Volcano | Kind | Flows | Window (driest rain on the cone) | Columns showing | Share of the basal disc | How the flows ended |
|---|---|---|---|---|---|---|
| 12754 | shield, activity 0.63 | 256 | 250,000 yr (44 mm) | 10,224: aa 8,648, pahoehoe 994, weathered 329, oxidised 253 | 5.4 % | length 237, water 11, loop 4, hollow 4 |
| 13650 | stratovolcano | 256 | 250,000 yr (55 mm) | 1,729 | 11.9 % | length 256 |
| 13601 | stratovolcano, under the ice | 256 | 250,000 yr (23 mm) | 1,168 (under snow blocks in the world) | 8.1 % | length 255, hollow 1 |
| 13594 | stratovolcano | 145 | 13,410 yr (277 mm) | 807: blocky 420, weathered 252, oxidised 135 | 5.6 % | length 136, hollow 5, loop 2, water 2 |
| 13617 | stratovolcano | 103 | 12,102 yr (283 mm) | 463: blocky 204, weathered 174, oxidised 85 | 3.2 % | length 101, hollow 2 |
| 13597 | stratovolcano | 53 | 5,746 yr (331 mm) | 243: blocky 115, weathered 77, oxidised 51 | 1.7 % | length 53 |
| 13610, 13609, 13749 | stratovolcanoes | 18, 6, 2 | 1,889, 648, 986 yr (542 to 1,234 mm) | 102, 62, 13 | 0.4 to 0.7 % | |
| 13620, 13699, 13745 | stratovolcanoes | 30, 3, 1 | | 0 | 0 | every vent under water |

| Area (ground blocks) | blackstone | smooth basalt | tuff | coarse dirt | basalt |
|---|---|---|---|---|---|
| Shield 12754, 256 by 256 around the summit, before | 8,922 | 10,379 | 1,108 | 85 | 9,004 |
| the same, after | 4,530 | 1,560 | 1,239 | 255 | 24,016 |
| Stratovolcano 13594, 128 by 128 around the vent, before | 427 | 465 | 0 | 1,419 | 865 |
| the same, after | 483 | 298 | 206 | 1,504 | 994 |
| Stratovolcano 13617, before | 712 | 902 | 575 | 810 | 1,370 |
| the same, after | 291 | 154 | 756 | 877 | 2,302 |
| Stratovolcano 13597, before | 678 | 738 | 0 | 1,642 | 1,079 |
| the same, after | 282 | 157 | 66 | 1,669 | 1,974 |

What the previews show:

- **The stripes are gone and the flows run downhill.** Before, the summit of the shield was a mesh
  of blackstone and smooth basalt contours of Terralith's noise; after, it is the pack's basalt
  ground with dark lines that leave the summit and run down the flanks, bundled along the drainage
  and out onto the plain beyond the cone. On the stratovolcanoes short spokes of blackstone, tuff
  and coarse dirt leave the vent.
- **The paint is a small share of the cone**, 2 to 12 %, because a flow is one or two blocks wide;
  on the dry cones the 256-flow bound ends the count, not the 35 % cover cap.
- **A level start.** Before the flows ran out from the vent on level ground (commit 0f337c3c), 92 of
  the shield's 256 flows stopped at their first step: the elevation raster is quantised, so a gentle
  summit holds level patches.

## Simplifications and limits

- **Gullies are drawn, not eroded.** The flank gullies (VOL-100 to VOL-105) are a pattern sized
  by O'Hara et al. 2024's eroded volume against age, not a drainage network: no channel follows
  the ground's flow, the basins are even round the cone, and the eroded rock goes nowhere (no
  fans at the foot). Only stratovolcanoes get them (O'Hara's sample): shields and silicic centres
  are left as they were. They are at least 8 blocks (2.4 km) apart, since the game draws the
  terrain from corners 4 blocks apart; Earth's young cones carry channels about 1 km apart
  (drainage density about 1 km⁻¹), which the game cannot show. A Lambert world (no projection
  pass) gets none. The cuts fall short of O'Hara's share on most cones: on seeds 1 to 3 the active
  cones' gullies took a median 4 to 12 % of their volume against the 23 to 24 % their ages ask,
  since the cones stand a median 17 to 23 blocks over their foot and the cap (a gully's axis cut
  at most 0.8 of the cone's height there) holds 85 to 95 % of them ("Earth against the mod").
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
- **Lava flows are paint, not eruptions.** They are drawn when the world is generated, from rates
  and lengths; the simulation knows nothing of them, and they change no heights (no levees, no
  flow fronts, no thickness on the ground).
- **Only active volcanoes have flows.** The bake exports only the volcanoes active at the present,
  so a young extinct cone, which on Earth would show flows for thousands of years in a dry climate,
  shows none. Giving it flows needs the bake to export it: a world-data format change.
- **Flows are narrow and cover little.** One or two blocks is 0.3 to 0.6 km at 294.6 m a block,
  broad for a real flow but a thin line in the world, so the flows cover 2 to 12 % of a cone where
  Mauna Loa's young flows cover 35 to 40 % in 1,000 years. On dry cones the bound of 256 flows ends
  the count well before the 35 % cap.
- **Cones are round on the planet.** Since 2026-10-23 the volcano stage, its edifice and caldera
  footprints, the ignimbrite sheet, the flood-basalt provinces, the arc and hotspot spacing and the
  amplifier's cone all measure the great-circle distance from the vent (ledger SCL-230 to SCL-232,
  [round-features.md](round-features.md)): a cone is the ellipse the Lambert grid makes of a circle,
  which the conformal cube world draws round (the equal-area Petroff–Guyou world with its own
  stretch, as every shape), so the cone and its flows (measured
  on the ground through the world's metric, read once at each vent) now agree. Until then a cone
  was round in Lambert blocks, long north–south on the planet by 1 / cos² of its latitude (3.0 to 1
  at 55°), and a flow was the right length on a stretched cone. On the Lambert world preset (a test
  layout) a cone is now wide east–west. Flows stop at the world's z edge.
- **Ice hides flows.** The ice pass runs after the flows, so on an ice-capped cone (13601 on the q1
  planet) they lie under snow blocks.
- **Far away a flow is a broken line.** At a level of detail coarser than a block, far terrain
  takes the paint of the one column it samples in each cell.
- **Every arc cone is viscous.** Etna and Fuji erupt basalt; the catalogue holds no composition, so
  their flows would be drawn short and blocky.
- **Pahoehoe and aa by slope and length.** The real control is the flow rate and the shear in the
  flow (Peterson & Tilling 1980; Rowland & Walker 1990); the game has neither per flow, so a long
  flow is aa from its vent and a short one turns to aa on steep ground.
- **The rain is the bake's mean annual precipitation**, read bilinear at each column; the arid
  anchor rests on second-hand reading, and the time a flow shows is set by the driest point of the
  cone.
- **Only band stripes are recognised.** The filter drops a pack rule in a volcanic biome that paints
  blackstone or smooth basalt on a band of a noise (both ends within ±1.5); a one-sided threshold
  stays, so Terralith's Yellowstone keeps its calcite patches, and a pack that drew stripes some
  other way would keep them.
- **A magma body ends at a cube world's turning edge.** A vent stands on a face, but its lenses (5 to
  90 blocks across), its mush and its lake ring can reach past a turning seam, where the ground is a
  turned copy of the partner face (cube-world-game.md, item 15). The body is cut there and sealed on
  its own face (MAG-059): within two columns of the edge its lava, conduit and lake are basalt shell,
  its plug and mush left out, so a lens ends against a two-column wall of its face's rock. A real
  body would go on under the partner face; folding the body onto it as the lava flows are (item 14)
  would keep it whole, but every lens, the mush and the lake would have to be turned into the partner
  face's frame, and the partner's chunks would have to look for bodies across each edge; the cut keeps
  the rule the seal already states (MAG-041). Its vent's water test and
  its lake's ring read its own face only, and a lake whose ring would cross the edge leaves the
  volcano its plug.

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
| A body reaching past a cube world's turning edge is cut there and sealed with a shell two columns thick, not shrunk, dropped or folded onto the partner face (MAG-059) | MAG-041's rule (every body sealed) at the cut; shrinking or dropping would break the size rules (MAG-003, MAG-048, MAG-054) or take an active volcano's magma away | agent, coordinator's follow-up task, 2026-10-25 |
| Arc volcanoes at least 60 km apart (VOL-039) | keeps the doubled cones apart; one cone stands for a large long-lived centre | agent, build log 2026-09-30; reviewed WP-2 2026-10-02 |
| Summit crater at least 2 km across (VOL-091) | a typical real crater (0.5 km, Pike & Clow 1981) would be under two blocks wide; the drawn one is of Grosse et al. 2014's size class, so it takes their median depth, 240 m (VOL-026) | agent, build log 2026-09-30; the depth Ben, 2026-10-22 |
| A cone is laid on the ground from its vent: the vent's ground level inside 0.4 of its radius, fading to its foot (VOL-080, VOL-081) | a cone is built by eruptions from its vent, so its summit is the vent; how far a real cone's onlap reaches down a slope was not read in a source | agent, coordinator's call, 2026-10-22, after Ben's volcano 27476 |
| Stratovolcano lifetime 2 to 8 Myr (VOL-038) | one step is 2 Myr | agent, build log 2026-09-30 |
| Eight flows in a flood basalt pile (VOL-052), terraces 80 m high (VOL-036) | visible layering and steps | agent, build log 2026-09-30 |
| Implementation rules: the volcanic front, the flux threshold, merging, the blind step, retirement, rendering and fading (VOL-002, VOL-004, VOL-008 to VOL-011, VOL-027, VOL-028, VOL-040, VOL-042, VOL-043, VOL-047, VOL-049 to VOL-051, VOL-053 to VOL-055, VOL-061 to VOL-073) | how the stage turns plate data into cones and cones into blocks; no Earth counterpart | agent, build log 2026-09-30 and 2026-10-01; labelled WP-2 2026-10-02 |
| Lava flows painted at active volcanoes, downhill from the rim, one or two blocks wide, paint only; the pack's noise stripes left out of the volcanic biomes (VOL-140 to VOL-163) | Terralith's stripes are contours of a noise: they neither start at a vent nor run downhill | the review of 2026-10-22 (fix 8 on the volcanoes review page), built by an agent under the coordinator |
| A viscous cone makes a lava flow once a century at full activity (VOL-146) | no source read gives how often an arc cone makes a lava flow | agent, 2026-10-22 (from memory) |
| Lengths log-uniform between Earth's bounds, the basaltic minimum set by Walker's ratio (VOL-142, VOL-144) | the sources give longest flows and a ratio of medians, not a distribution | agent, 2026-10-22 |
| Three rain anchors for the weathering clock (VOL-147 to VOL-151) | the sources give Kona, Hilo and desert examples, not a law | agent, 2026-10-22 |
| At most 256 flows, cover at most 35 % (VOL-152, VOL-153) | a cost bound and Mauna Loa's share | agent, 2026-10-22 |
| Terralith's Yellowstone calcite patches kept (VOL-163) | they are patches, not lines, and Yellowstone is not a lava cone | agent, coordinator's call, 2026-10-22 |

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
15. **Flows on extinct volcanoes** need the bake to export the recently extinct volcanoes, a
    world-data format change and so Ben's to decide. Without it, a cone that died at the last step
    shows bare ground where Earth's young dead cones (Cima, the Chaîne des Puys) show flows.
16. **How often an arc cone makes a lava flow** (VOL-146) is from memory; Siebert et al. 2010
    (Volcanoes of the World) may tabulate it.
17. **The slope at which pahoehoe turns to aa** (VOL-158): no source read gives one; the sources
    give a flow rate (about 5 to 10 m³/s, Rowland & Walker 1990, search summary only).
18. **Flow widths** (VOL-159): no source read gives typical widths; whether a broad flow should be
    drawn wider than two blocks is open.
19. **Basaltic arc cones** (VOL-161): Etna and Fuji make long basaltic flows; the catalogue would
    need a composition per volcano.
20. **The desert anchor** (VOL-147): Wells et al. 1985 was not read, nor the Mojave's rain.

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
| Riker2009, Walker1973, Malin1980 | flow lengths (VOL-140, VOL-143, VOL-144) | abstracts only, 2026-10-22 |
| USGS-MedicineLake, USGS-VHP-LavaFlows, NPS-LavaFlows, OregonState-VolcanoWorld | flow lengths, thicknesses, pahoehoe and aa (VOL-141, VOL-142, VOL-157, VOL-158) | fetched, 2026-10-22 |
| USGS-MaunaLoaFAQ, USGS-VolcanoesUS | Mauna Loa's eruptions and the share of young flows (VOL-145, VOL-152) | fetched, 2026-10-22 |
| USGS-FS144-02 | aa thickness (VOL-157) | fetched, 2026-10-22 |
| PetersonTilling1980, RowlandWalker1990 | pahoehoe and aa (VOL-158) | abstract; search summary only |
| HVO-VW1996, HVO-VW1999, HVO-VW2006, Kinney2015 | weathering and plant cover with age and rain; colours (VOL-148, VOL-149, VOL-151, VOL-162) | fetched, 2026-10-22 |
| Wells1985, MillerBuesch2022 | desert flows (VOL-147) | not read (Wikipedia and a search summary); an abstract |
| Favalli2005, Chevrel2021 | DOWNFLOW, compared with the path (VOL-156) | metadata only; fetched |
| Papale2018, Heliker2003, Siebert2010, Wikipedia-Etna | eruption sizes, channel widths, flank eruptions (VOL-146, VOL-159) | fetched; search summary; not read; secondary |

Session note: [`../research/notes/session-lava-realism.md`](../research/notes/session-lava-realism.md).
Extraction notes, with 21 flags: [extraction-notes/VOL.md](extraction-notes/VOL.md) (flags 1 to 5
are settled in the ledger rows; flag 4 rested on reading radii as widths).
Build log: STATUS.md, "P1-21 volcanoes as landforms" (2026-09-30), "Magma under active volcanoes"
and "Lava floods" (2026-10-01), "Science stage 2, WP-2: volcanoes and magma" (2026-10-02),
"Review fix 8: lava flows" (2026-10-22).
