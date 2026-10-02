# Volcanoes, magma and lava

Status of this page: `stage 1 (written from the ledger, nothing verified)`, 2026-10-02.

Ledger ids on this page: `VOL-001` to `VOL-073` ([ledger/VOL.md](ledger/VOL.md)) and `MAG-001` to
`MAG-042` ([ledger/MAG.md](ledger/MAG.md)). Of these 115 rows, 63 are `uncited`, 47 are
`game-choice`, 4 are `cited-unverified` and 1 is `calibrated`. **This feature rests on one cited
paper, and that paper, as far as its abstract shows, does not say what it is cited for.** Every
volcano size in the code is a design value backed by a comment naming a real volcano, with no
reference. Read the page with that in mind.

Scale used throughout: one block is 300 m across and 20 m up, so 1 km is 3.3 blocks across and
50 blocks up.

## What you see in the world

**Arc volcanoes.** Lines of cones stand along the landward side of ocean trenches, where one plate
sinks under another, at least about 60 km (200 blocks) apart. They occur on continents and as island
chains. A cone is 1.2 to 3 km tall (60 to 150 blocks) with a concave profile that steepens toward
the top, and at the default setting it is 13 to 28 km in radius (45 to 93 blocks), twice as wide as
its physical shape, so that it does not read as a spike. It has a summit crater at least 2 km
(7 blocks) across. Its rock is andesite over tuff (volcanic ash turned to rock), or rhyolite over
tuff where the crust is thick.

**Shield volcanoes and island chains.** Over a hotspot (a fixed plume of hot mantle) under ocean
floor, a broad, gently sloping basalt shield rises 3.5 to 6.5 km above the sea floor, 49 to 80 km in
radius. Because the plate moves over the plume, a new shield starts about every 70 km and the older
ones go extinct, erode and sink, leaving a chain of islands and seamounts (drowned volcanoes) that
gets older away from the plume. Most shields do not reach the surface; in the reviewed 1,000 Myr run
only 3 islands and 8 seamounts stood at the end.

**Calderas.** Volcanoes on thick continental crust, and hotspot centres under continents, can
collapse after a large eruption into a caldera: a flat-floored basin with steep walls, 6 to 18 km
(20 to 60 blocks) across and roughly 400 to 850 m (20 to 42 blocks) deep, ringed by a sheet of tuff
up to 250 m thick that thins outward.

**Flood basalts.** A large igneous province becomes a basalt plateau 1,500 m (75 blocks) thick at
its centre, built of eight flows with thin ash beds between. Where the pile is at the surface, the
terrain is cut into steps 80 m (4 blocks) high.

**Magma.** Under each volcano that is still active at the present there is a chamber of lava, its
top 3 km (150 blocks) below the crater floor. In blocks it is a tall lens (for example 19 across and
57 high under a large arc cone) because of the vertical stretch. A narrow column of lava rises from
it to just under the crater, capped by magma blocks. A quarter of the active volcanoes on dry land
have an open lava lake in the crater instead, 4 to 20 blocks across. Everything molten is walled in
by smooth basalt, so lava never touches water or an open cave. Extinct volcanoes have no magma.

**Lava elsewhere.** The random lava lakes and springs of vanilla Minecraft and of Terralith are
removed. The only other lava is a layer at the very bottom of the world (below y -342 in the default
world). On the measured planet a point on land was a median 658 blocks from the nearest active
volcano and 1,178 blocks from the nearest open lava lake.

## The model in plain words

The volcano stage runs once per plate step (about 2 Myr), after the surface stage, and keeps a
catalogue of individual volcanoes that ride on their plates.

1. **Where arcs are.** The plate model flags cells above a subducting plate as "arc". The stage
   turns that into an arc magma flux between 0 and 1: the convergence speed divided by 80 mm/yr,
   capped at 1, and multiplied by 0.7 on oceanic crust (VOL-003). It then thins the flagged band to
   its centre line, the volcanic front (VOL-004).
2. **Birth of arc volcanoes.** On the front, wherever the flux is at least 0.25, a new
   stratovolcano (a steep cone of lava and ash) is placed if no other cone is within 60 km times a
   random factor of 0.8 to 1.2 (VOL-002, VOL-039). Its planned height runs from 1.2 km at the
   threshold flux to 3 km at full flux (VOL-019); its radius follows height to the power 0.8
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
   rhyolite or basalt (VOL-056 to VOL-059).
5. **Erosion.** The surface stage erodes a cone like any other land. The stage measures that
   change over the cone's footprint and lowers the volcano's recorded height to match (VOL-051).
6. **Death.** A stratovolcano goes extinct when its planned life ends or the arc flux under it has
   been too low for two steps (VOL-010). An extinct volcano on ocean crust sinks at 25 m/Myr, up to
   1.5 km (VOL-048). It leaves the catalogue when worn below a tenth of its peak height, or after
   250 Myr (VOL-049), or when its plate carries it into a trench (VOL-050).
7. **Calderas.** A silicic cone collapses into a caldera with a chance of 0.04 per Myr; a
   continental hotspot centre with 0.12 per Myr (VOL-045). The radius is drawn between 3 and 9 km
   (VOL-029) and the depth is half the cone's height, capped at 400 m plus 5 % of the radius
   (VOL-030). Half of the removed rock is laid around it as a tuff sheet (VOL-046).
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

Shapes. `ρ` is the distance from the centre divided by the basal radius; each profile is the
fraction of the full height at that distance.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| VOL-016 | Stratovolcano profile | (1 − ρ)^1.5 | – | none; comment names Fuji | `uncited` |
| VOL-017 | Shield profile | (1 − ρ²)^1.5 | – | none | `uncited` |
| VOL-018 | Silicic centre profile | (1 − ρ²)² | – | none | `uncited` |
| VOL-019 | Stratovolcano height | 1.2 to 3.0, linear in arc flux | km | none; comment names Fuji, Cotopaxi, Shasta | `uncited` |
| VOL-020 | Stratovolcano radius | 14 × (height / 3 km)^0.8 ÷ flank steepness | km | none; comment names Fuji, Rainier, Shasta | `uncited` |
| VOL-021 | Shield height above the sea floor | 3.5 to 6.5, by plume strength | km | none; comment names Hawaii, Réunion, Galápagos | `uncited` |
| VOL-022 | Shield radius | 40 × (height / 6.5 km)^0.8 ÷ flank steepness | km | none; comment names Mauna Loa, Mauna Kea | `uncited` |
| VOL-023 | Continental hotspot centre | 0.9 tall, 35 radius | km | none; comment names Yellowstone | `uncited` |
| VOL-024 | Flank steepness | 0.5 (radius doubled) | – | Ben, decisions log 2026-09-30 | `game-choice` |
| VOL-026 | Summit crater | radius max(1 km, 0.07 R); depth 0.06 of height | km | none | `uncited` |
| VOL-029 | Caldera radius | uniform 3 to 9, at most 0.6 R | km | none; comment names Crater Lake, Long Valley, Yellowstone | `uncited` |
| VOL-030 | Caldera depth | min(0.5 × height, 400 m + 0.05 × radius) | m | none | `uncited` |
| VOL-032 | Summit ceiling | 6.8 above the sea; × 0.65 for shields; scaled by crust strength over gravity | km | none; comment names Ojos del Salado, Mauna Kea | `uncited` |

Rules and rates.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| VOL-003 | Arc magma flux | clamp(convergence / 80, 0, 1), × 0.7 on oceanic crust | – (convergence in mm/yr) | none | `uncited` |
| VOL-002 | Flux needed for a volcano | 0.25 | – | none | `uncited` |
| VOL-039 | Spacing of arc volcanoes | 60 × random(0.8 to 1.2) | km | de Bremond d'Ars, Jaupart & Sparks 1995 | `cited-unverified` |
| VOL-005 | Crust thickness for silicic volcanoes | 42 | km | none; comment names the Central Andes | `uncited` |
| VOL-007 | Spacing along a hotspot chain | 70 | km | none; comment names Hawaii | `uncited` |
| VOL-038 | Stratovolcano lifetime | 2 to 8 | Myr | agent, build log 2026-09-30 | `game-choice` |
| VOL-041 | Growth time constant | 1 | Myr | none | `uncited` |
| VOL-045 | Caldera eruption rate | 0.04 (silicic cone), 0.12 (continental hotspot) | per Myr | none | `uncited` |
| VOL-048 | Subsidence of extinct ocean volcanoes | 25, up to 1,500 m in all | m/Myr | none; comment names Hawaii | `uncited` |
| VOL-052 | Flood basalt pile | 8 flows; thickness and radius from the plate model (1,500 m; 900 km on an Earth-sized planet) | – | none; comment names the Deccan | `uncited` |
| VOL-056 to VOL-060 | Rock laid down | arc: 35 % tuff, 65 % andesite; silicic: 45 % tuff, 55 % rhyolite; shield: basalt; flood basalt: 88 % basalt, 12 % tuff | share | none | `uncited` |
| VOL-065 | Fading of an extinct volcano's fine detail | e^(−age / 5 Myr) for the crater, three times slower for the cone | Myr | none | `uncited` |

Magma and lava.

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| MAG-002 | Depth of the chamber's top below the crater floor | 3.0 | km | none; comment names Kīlauea, Mount St Helens, Vesuvius | `uncited` |
| MAG-003 | Chamber radius | 0.1 × volcano radius, kept within 1.2 to 4.5 | km | none | `uncited` |
| MAG-004 | Chamber half-height | 0.2 × its radius | – | none | `uncited` |
| MAG-019 | Share of able volcanoes with a crater lava lake | 0.25, most active first | – | agent, build log 2026-10-01 ("a player should be able to find one") | `game-choice` |
| MAG-020 | Earth's persistent lava lakes | "about eight of some 1,400 active volcanoes" | count | none | `uncited` |
| MAG-027 | Lava floor | open space below world floor + 10 blocks is lava | blocks | vanilla Minecraft's rule, moved to the world's floor; Ben, decisions log 2026-10-01 (kept as a setting, not yet decided) | `game-choice` |
| MAG-034 | Random lava lakes and springs | removed | – | Ben, decisions log 2026-10-01 | `game-choice` |

The full tables, with file and symbol for each row, are in the ledger files.

## Earth against the mod

The build log holds no systematic comparison of the mod's volcanoes with Earth's. What exists is
below. The Earth figures in the second table were looked up in stage 1; most are from Wikipedia
and are a starting point for stage 2, not a verification.

Measured in the mod:

| Quantity | The mod | Run |
|---|---|---|
| Volcanoes at the end of a run | 250 in the catalogue; 203 active stratovolcanoes, 2 active shields | `earthlike_quick_16k`, scattered continents, 500 Myr, 2026-09-30 (seed not stated; the preset's is 1) |
| Stratovolcanoes born in a run | 24,017 (most erode away) | same |
| Calderas | 295, and 27 summit calderas on shields | same |
| Flood basalt provinces | 9 in 500 Myr; 17 in 1,000 Myr | same; and run review, `earthlike_quick_16k`, supercontinent, 1,000 Myr, seed 1, 2026-10-01 |
| Tallest volcano | 4,643 m of relief | same 500 Myr run |
| A caldera 2 Myr old | 14.6 km across, floor 764 m deep (the depth cap, not a physical law) | same |
| Arcs | 713 arcs, 61 % continental, 39 % island arcs | run review, 1,000 Myr |
| Hotspot tracks | 35 tracks, median 4,750 km (scaled to Earth's size) over 78 Myr | run review, 1,000 Myr |
| Active volcanoes at the present | 123: 119 arc, 3 caldera complexes, 1 shield; 100 on land; 16 with a crater lava lake | `earthlike_quick_16k`, supercontinent, 500 Myr, 2026-10-01 |
| Distance from a land point to the nearest active volcano on land | median 658 blocks (10th percentile 169, 90th 1,309) | same; 4,000 points |
| Effect on land height | with volcanoes off, land above 2 km falls from 0.307 to 0.170 of the land (Earth: 0.102) | hypsometry runs, `earthlike_quick_16k`, 500 Myr, 2026-10-01 |

Earth figures looked up in stage 1, against what the code assumes:

| Quantity | Earth | Source and how it was read | The code |
|---|---|---|---|
| Spacing of arc volcanoes | "Volcano spacing follows a random distribution"; "no characteristic spacing" | de Bremond d'Ars and others 1995, abstract only | a minimum of 60 km |
| Upper flank of Mount Fuji | 31 to 35 degrees | Wikipedia (secondary) | the profile gives about 15 degrees at 3 km over 14 km before the flank setting, about 8 after; the comment says about 20 |
| Steepest slope of Mauna Loa | about 12 degrees | Wikipedia (secondary) | about 14 degrees for the tallest shield before the flank setting, about 7 after; the comment says "under about 10" |
| Summit craters of stratovolcanoes | Fuji 780 m across; Cotopaxi 800 by 550 m | Wikipedia (secondary) | at least 2 km across |
| Caldera widths | Crater Lake 8.0 by 9.7 km; Long Valley 16 by 32 km; Yellowstone 55 by 72 km | Wikipedia; USGS page for Long Valley (fetched) | 6 to 18 km across: fits Crater Lake only |
| Highest volcano | Ojos del Salado, 6,893 m | Wikipedia, citing the Global Volcanism Program (secondary) | ceiling 6.8 km |
| Mauna Loa above the sea floor | about 9 km | USGS page (fetched), Wikipedia | shields 3.5 to 6.5 km, capped at 4.4 km above the sea |
| Hawaii–Emperor chain | 6,200 km; oldest dated seamount 81 Myr | Wikipedia, citing Sharp & Clague 2006 (secondary) | median track 4,750 km over 78 Myr |
| Sinking of the island of Hawaii | 2.7 mm/yr at present | Szabo & Moore 1986, abstract only | 25 m/Myr (0.025 mm/yr) for extinct volcanoes; see open questions |
| Shallow magma storage | Kīlauea 1.2 and 3.5 km; Vesuvius 3 to 4 km; Mount St Helens 5 to 12 km | Przeor and others 2022 and Scaillet and others 2008, abstracts; Wikipedia | 3 km for every volcano |
| Persistent lava lakes | eight volcanoes listed; about 1,350 potentially active volcanoes | Wikipedia list (secondary); USGS page (fetched) | "about eight of some 1,400" |
| Flood basalt provinces | 16 over about 345 Myr, so about one per 22 Myr | Courtillot & Renne 2003, abstract; the division is ours | one per 59 Myr in the 1,000 Myr run |
| Deccan Traps | originally about 1.5 million km², more than 2 km thick | Wikipedia (secondary) | about 2.5 million km² (Earth-equivalent), 1.5 km thick |

## Simplifications and limits

- **Three kinds only.** There are no cinder cones, no volcanoes at mid-ocean ridges or in rifts,
  and no fissure eruptions. The code's own comment notes that ridges and rifts are where lava is
  common on Earth (MAG-038); they are not exported to the game at all.
- **A volcano is a centre, not an eruption.** With steps of about 2 Myr, one "stratovolcano" stands
  for a volcanic centre that rebuilds itself in one place. Hence lifetimes of 2 to 8 Myr against
  the fraction of a million years a single real cone lasts.
- **No eruptions, magma supply or chemistry.** Height grows toward a target on a fixed clock.
  Eruption rate, erupted volume and magma composition are not modelled. "Silicic" is a yes or no set
  by crust thickness.
- **Volcano positions follow the plate model's arc flag**, which covers broad areas; the stage
  reduces it to a line itself. When the plate model changes (see [plates.md](plates.md)), so will
  where volcanoes stand.
- **Lengths do not shrink with the planet.** A volcano's radius is in real kilometres on a planet
  an eighth of Earth's size, so volcanoes are large for their world. Flood basalt provinces, by
  contrast, are scaled to the planet (about 110 km in radius at 16k).
- **Magma chambers are one fixed shape** at one depth, whatever the volcano's size, age or kind.
- **Too many volcanoes for the land.** Cones add about a fifth of everything the surface erodes in
  a run and push the share of high land well above Earth's (see the table). This was kept as it is
  (Ben, decisions log 2026-10-01: volcano density stays 1).
- **Activity saturates.** Most arc volcanoes have the highest activity value, so which of them get
  a crater lake is decided by their catalogue number, not by anything physical.

## Game choices

| What | Why | Who and when |
|---|---|---|
| Which kinds exist (arc stratovolcanoes, hotspot shields and chains, calderas, flood basalts) | scope for version 1 | Ben, decisions log 2026-09-29 |
| Flank steepness 0.5: every volcano twice as wide as its physical profile (VOL-024) | at 15 times vertical stretch a physical cone is a spike | Ben, decisions log 2026-09-30 ("Setting, gentler default") |
| Young volcanoes are smooth; old ones regain rugged ground (VOL-068) | looks | Ben, decisions log 2026-09-30 |
| Volcano density 1 (VOL-012) | kept although it raises the land | Ben, decisions log 2026-10-01 |
| Lava only at active volcanoes; random lava lakes and springs removed (MAG-001, MAG-034) | "only at active volcanoes, ridges and hotspots" is where lava is on Earth | Ben, decisions log 2026-10-01 |
| The lava floor at the bottom of the world (MAG-027) | a Minecraft convention with no counterpart on Earth; kept as a setting until decided | open; Ben, decisions log 2026-10-01 |
| A quarter of able volcanoes have an open lava lake (MAG-019), though persistent lava lakes are rare on Earth | "a player should be able to find one" | agent, build log 2026-10-01 |
| Lava is sealed in smooth basalt and never touches water (MAG-010, MAG-011, MAG-041) | lava pools were found overlapping the ocean | agent, build log 2026-10-01, after Ben's report |
| Summit crater at least 2 km across (VOL-026) | a real crater would be under three blocks wide | agent, build log 2026-09-30 |
| Stratovolcano lifetime 2 to 8 Myr (VOL-038) | one step is 2 Myr | agent, build log 2026-09-30 |

## Open questions

Things stage 2 must settle. None has been changed in the code.

1. **The one citation does not appear to support its value.** The 60 km spacing (VOL-039) cites
   de Bremond d'Ars, Jaupart & Sparks 1995. That paper's abstract says volcano spacing is random
   with no characteristic value. The full text may give mean spacings; it has not been read.
2. **Comments that disagree with the code**, found by the extraction agent's arithmetic:
   - the stratovolcano comment says upper flanks of about 20 degrees; the formula gives about 15
     (VOL-016);
   - the caldera comment gives its examples "across" where the numbers are radii; the coded range
     excludes Long Valley and Yellowstone (VOL-029);
   - the crater comment says real craters are 0.3 to 1 km while the code makes them at least 2 km
     (VOL-026);
   - the subsidence comment says "about 2 km over about 30 Myr", which is 67 m/Myr; the code uses
     25 m/Myr (VOL-048).
3. **Island subsidence.** No source was found for the long-term sinking of extinct ocean
   volcanoes. The one rate read (2.7 mm/yr for the island of Hawaii today) is for a young island
   still loading the plate and is a hundred times the coded rate; it is not the right comparison,
   and the right one is still to be found.
4. **Every size range needs a source.** Heights, radii and the height-to-radius power of 0.8 are
   design values. Stage 2 should set them against a morphometric database of volcanoes, not against
   single named examples. (A lead written from memory and not checked: Grosse and others published
   morphometry of composite volcanoes around 2009 to 2014.)
5. **Caldera rate and size** (VOL-029, VOL-045) have no source.
6. **Chamber depth** (MAG-002): the comment's figures for Kīlauea, Mount St Helens and Vesuvius
   are near, but not equal to, what the abstracts read in stage 1 give.
7. **The volcano ceiling** (VOL-032, VOL-034) takes a crustal strength of 140 MPa from the plate
   model; see [isostasy-and-height-limit.md](isostasy-and-height-limit.md).
8. **Flood basalt frequency**: the 1,000 Myr run made 17, about a third of Earth's rate if the
   Courtillot & Renne list is taken at face value. That is the plate model's doing.
9. **Where the lava floor goes** is undecided, and so is whether a median 658 blocks to the nearest
   volcano is acceptable for players (STATUS.md, pending questions).

## Sources

Keys refer to [bibliography.md](bibliography.md).

| Key | Used for | How it has been read |
|---|---|---|
| Bremond1995 | arc volcano spacing (VOL-039) | abstract only (stage 1); not stated before |
| Szabo1986 | subsidence of Hawaii | abstract only (stage 1); not cited by the code |
| Przeor2022, Scaillet2008 | magma storage depths | abstracts only (stage 1); not cited by the code |
| Courtillot2003 | flood basalt ages | abstract only (stage 1); not cited by the code |
| USGS volcano pages; Wikipedia pages on named volcanoes | sizes of named volcanoes | fetched (stage 1); secondary |

Session note: [`../research/notes/session-lava-realism.md`](../research/notes/session-lava-realism.md).
Extraction notes, with 21 flags: [extraction-notes/VOL.md](extraction-notes/VOL.md).
Build log: STATUS.md, "P1-21 volcanoes as landforms" (2026-09-30), "Magma under active volcanoes"
and "Lava floods" (2026-10-01).
