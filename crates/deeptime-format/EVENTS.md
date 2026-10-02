<!-- SPDX-License-Identifier: LGPL-3.0-or-later -->
# Deep Time event log (format 0.0)

The shared event log of plan §5.5: one append-only record of notable happenings that feeds the
timelapse ticker, "Why here?" and a later legends view. Several **writers** append to it without
coordinating: geology is writer 1; the civ module (or anything else) registers its own id and adds
its own segments without touching geology's. Code: `crates/deeptime-format/src/events.rs`
(`Event`, `EventWriter`, `EventLog`). All integers are little-endian.

## Ids and writers

An event id is a `u64`: `writer << 48 | sequence`.

- `writer` (16 bits) names who wrote the event. `0` is reserved and never valid.
- `sequence` (48 bits) counts that writer's events from 0. A writer continues after the highest
  sequence number it has on disk (`EventWriter::open` reads `last_seq` from the index), so two
  writers never collide and one writer never reuses a number.
- Text form: `<writer>:<sequence>` (`1:42`). In JSON the id is a plain number; JavaScript reads it
  exactly while `writer ≤ 31` (2^53).

| Writer | Name in `index.json` | Who |
|---|---|---|
| 0 | — | reserved |
| 1 | `deeptime:geology` | the simulation core (every stage) |
| 2 … 65535 | a namespaced id, e.g. `civ:history` | later modules; register by opening an `EventWriter` |

A geology re-run (`finalizeWrite` of a new session into the same folder) replaces writer 1's
segments and keeps every other writer's (`EventWriter::clear`). Appends to one folder must not run
at the same time: `index.json` is rewritten whole (geology finishes before any other writer starts).

## Folder

```text
events/index.json            the writers and their segments
events/<writer>-<segment>.dtev   immutable segments, <segment> counting from 0 per writer
```

A folder without `events/index.json` has an empty log. Every file is written as `<name>.tmp`,
fsynced and renamed; a segment is written before the index that lists it.

### `events/index.json`

```json
{
  "format_version": "0.0",
  "writers": [{"id": 1, "name": "deeptime:geology"}],
  "segments": [
    {"path": "events/1-0.dtev", "writer": 1, "segment": 0, "events": 13,
     "first_seq": 0, "last_seq": 12, "t_min": -360000000.0, "t_max": 0.0,
     "bytes": 2312, "crc32": 305419896}
  ]
}
```

`format_version` is checked before anything else. `t_min` is the smallest `t_start`, `t_max` the
largest `t_end` (years). `bytes` and `crc32` (CRC-32/ISO-HDLC) cover the whole segment file; a
reader checks them before decoding. Every segment's writer must be registered.

## Segment (`.dtev`)

A sectioned file (`FORMAT.md`, "Sectioned files") with magic `DTEV`, 32 bytes of fixed fields and
five sections:

```text
fixed fields (offset from byte 16)
0    2   u16 writer
2    2   u16 reserved, 0
4    4   u32 segment number (the <segment> of the file name)
8    4   u32 N, events
12   4   u32 S, strings
16   8   f64 t_min (smallest t_start)
24   8   f64 t_max (largest t_end)
sections
0    strings     S × (u32 byte length, UTF-8 bytes), back to back: kinds, sources, param keys and
                 string param values, deduplicated, in order of first use
1    records     N × 64 bytes, sorted by (t_start, id)
2    causes      u64 event ids, referenced by the records
3    params      16-byte entries, referenced by the records
4    time index  ceil(N / 64) × 24 bytes
```

### Record (64 bytes)

```text
0    8   u64 id
8    8   f64 t_start, years relative to world creation (negative = geologic past)
16   8   f64 t_end (= t_start for an instantaneous event; t_end ≥ t_start)
24   4   u32 kind (string index)
28   4   u32 source (string index)
32   4   f32 magnitude (kind-defined, for ranking and filtering)
36   1   u8 place tag
37   3   reserved, 0
40   4   u32 place a
44   4   u32 place b
48   4   u32 first cause (index into the causes section, in entries)
52   4   u32 cause count
56   4   u32 first param (index into the params section, in entries)
60   4   u32 param count
```

| Place tag | Meaning | a | b |
|---|---|---|---|
| 0 | global | 0 | 0 |
| 1 | point | f32 latitude (degrees, north +) bits | f32 longitude (degrees in [0, 360), east of the seam) bits |
| 2 | plate | plate id | 0 |
| 3 | region | L1 mask id (reserved: no writer stores masks yet; the plates' events use points) | 0 |
| 4 | site | site id (`sites/sites.dtsr`) | 0 |
| 5 | path | polyline id (reserved: no writer stores paths yet) | 0 |

Longitude is plan §5.2's 360°·(x + C/2)/C, so the seam is 0° and a point is at block
x = lon/360 · C − C/2, z = −lat/90 · C/4 (north is −z; `FORMAT.md`, "Grids").

### Param entry (16 bytes)

```text
0    4   u32 key (string index)
4    1   u8 type: 0 i64, 1 f64, 2 string, 3 id
5    3   reserved, 0
8    8   value: i64 | f64 bits | u32 string index (zero-extended) | u64 id
```

An event's params are sorted by key and keys are unique within an event. An `id` is an event,
site, plate, … id; the kind's entry below says which.

### Time index entry (24 bytes)

One per block of 64 records (the last block may be shorter):

```text
0    8   f64 t_start of the block's first record
8    8   f64 largest t_end in the block
16   4   u32 first record
20   4   u32 record count
```

A reader answering "what overlapped [t0, t1]" skips blocks whose largest `t_end` is below `t0`
and stops at the first block that starts after `t1`.

## Rules every writer keeps

- `t_start` and `t_end` are finite, `t_end ≥ t_start`.
- `kind` and `source` are namespaced ids `ns:path` (lower-case letters, digits, `_ - .`, and `/`
  in the path).
- Every id in a segment belongs to the segment's writer; no event causes itself.
- Causes may name any writer's events, including ones whose segments are absent (readers skip
  unknown causes).

## Reading: the merged log and its indexes

`EventLog::load(<world>)` reads every listed segment, checks sizes, CRCs, writers and that no id
appears twice, and merges all writers by (t_start, id). It keeps:

- **by time**: the merged order; `between(t0, t1)` returns events overlapping [t0, t1];
- **by kind**: `of_kind(kind)`;
- **by cause**: `effects_of(id)`, the reverse links, built on load; `cause_chain(id, depth)` walks
  the causes breadth first ("Why here?");
- **by L1 cell**: `cell_index(grid, site_lat_lon)` for point and site places ("what happened here").

## JSON (`eventsSince`, the viewer's `events.json`)

```json
{"id": 281474976710665, "writer": 1, "seq": 9, "t_start": -3.6e8, "t_end": -3.6e8,
 "kind": "deeptime:erosion_pulse", "source": "deeptime:dummy",
 "place": {"type": "point", "lat": 12.5, "lon": 320.0}, "magnitude": 0.6,
 "causes": [281474976710664],
 "params": {"stage": "dummy_drift", "step": 35, "peak_y": 363.0, "parent": {"id": 281474976710600}}}
```

- `place.type` is `global`, `point` (`lat`, `lon`), `plate` (`plate`), `region` (`mask`), `site`
  (`site`) or `path` (`path`).
- Params: an i64 or f64 is a JSON number, a string a string, an id `{"id": <number>}`. A
  non-finite f64 (and magnitude) is `null`.
- `eventsSince(cursor, max)` returns at most `max` events in **emission order** starting at index
  `cursor`; pass the number of events already received.

## Kinds

Geology's v1 kinds (plan §5.5). Each stage documents its params here when it starts emitting the
kind; until then a kind is reserved with its intended meaning. Source ids name the stage module
(`deeptime:plates`, `deeptime:surface`, `deeptime:climate`, `deeptime:deposits`, …).

| Kind | Meaning | Emitted by | Params |
|---|---|---|---|
| `deeptime:plate_birth` | a plate appears (rifting splits one) | plates (P1-03) | see "The plate model's events" |
| `deeptime:plate_death` | a plate is consumed | plates (P1-03) | see below |
| `deeptime:rift_onset` | continental rifting starts | plates (P1-03); dummy | see below; dummy: see below |
| `deeptime:ridge_birth` | a spreading ridge forms | plates (P1-03) | see below |
| `deeptime:subduction_onset` | a subduction zone starts | plates (P1-03) | see below |
| `deeptime:collision` | continents collide | plates (P1-03); dummy | see below; dummy: see below |
| `deeptime:terrane_accretion` | a terrane docks | plates (P1-03) | see below |
| `deeptime:orogeny_phase` | mountain building, with a duration | plates (P1-03); dummy | see below; dummy: see below |
| `deeptime:hotspot_track` | a hotspot's track over a plate | plates (P1-03) | see below |
| `deeptime:large_igneous_province` | a LIP eruption | plates (P1-03) | see below |
| `deeptime:volcano_arc` | a volcanic arc segment, with its cones | volcanoes (P1-21) | see "The volcanoes' events" |
| `deeptime:volcano_hotspot_chain` | a hotspot's chain of volcanoes, with its members | volcanoes (P1-21) | see "The volcanoes' events" |
| `deeptime:volcano_stratovolcano` | an arc stratovolcano, from its birth to its death | volcanoes (P1-21) | see "The volcanoes' events" |
| `deeptime:volcano_shield` | a hotspot's shield volcano (an island or seamount of its chain) | volcanoes (P1-21) | see "The volcanoes' events" |
| `deeptime:volcano_caldera_complex` | a continental hotspot's silicic centre | volcanoes (P1-21) | see "The volcanoes' events" |
| `deeptime:volcano_caldera` | a caldera collapse | volcanoes (P1-21) | see "The volcanoes' events" |
| `deeptime:volcano_flood_basalt` | a flood-basalt plateau (a large igneous province's lava pile) | volcanoes (P1-21) | see "The volcanoes' events" |
| `deeptime:impact_crater` | a meteor impact that left a crater | impacts (P2-08) | see "The impacts' events" |
| `deeptime:impact_ocean` | a large meteor impact into deep water that left no crater | impacts (P2-08) | see "The impacts' events" |
| `deeptime:basin_subsidence` | a sedimentary basin subsides | surface (P1-04) | see "The surface's events" |
| `deeptime:marine_transgression` | the sea advances over land | surface (P1-04) | see "The surface's events" |
| `deeptime:marine_regression` | the sea retreats | surface (P1-04) | see "The surface's events" |
| `deeptime:glaciation` | an ice age | climate (P1-05) | see "The climate's events" |
| `deeptime:snowball` | a snowball Earth: the ocean frozen over | climate (deep-time climate) | see "The climate's events" |
| `deeptime:hothouse` | a hothouse: global mean above `climate.hothouse_c` | climate (deep-time climate) | see "The climate's events" |
| `deeptime:erosion_pulse` | a burst of erosion | surface (P1-04); dummy | see "The surface's events"; dummy: see below |
| `deeptime:deposit_formation` | a deposit forms (place: its site) | deposits (P1-14); dummy | below |
| `deeptime:deposit_exhumation` | erosion exposes (or destroys) a deposit | deposits (P1-14) | below |
| `deeptime:placer_formation` | a placer forms from an eroded source | deposits (P1-14) | below |
| `deeptime:supergene_enrichment` | a supergene blanket forms | deposits (P1-14) | below |
| `deeptime:petroleum_generation` | a source rock enters the oil window | deposits (P1-14) | below |
| `deeptime:trap_charge` | a trap fills with petroleum | deposits (P1-14) | below |
| `deeptime:guaranteed_occurrence` | a deposit added by the playability pass (plan §7.8) | deposits (P1-14) | below |

### The plate model's events (source `deeptime:plates`, P1-03)

Written by `crates/deeptime-core/src/plates/` (plan §8.1). Every place is a **point** (the
feature's centre on the map); plate ids are params (`i64`, indices that are never reused).
Areas (`*_km2`) and lengths (`*_km`) are on the real planet; speeds (`*_mm_yr`) are
Earth-normalized, as the plates move in the model (decisions file, "Tectonics scaling"). Times are
the step at which it happened unless stated.

| Kind | Params | Magnitude | Causes |
|---|---|---|---|
| `plate_birth` | `plate`, `parent`, `origin` (`rift` or `subduction_initiation`), `area_km2`, `speed_mm_yr`, `continental_frac` (near 0 for an initiation: it takes old oceanic crust and at most slivers stranded in it); initiations add `mean_ocean_age_myr` and `coast_split_km` (continental margin the split cut off its ocean, km) | area / average initial plate area | its `rift_onset` (rifts) |
| `plate_death` | `plate`, `fate` (`consumed`: no crust left; `absorbed`: too small, merged; `sutured`: a continental collision welded it whole to `absorbed_by`), `absorbed_by` | 0 | the plate's last `subduction_onset` or the `collision` that took its crust, when known |
| `rift_onset` | `plate` (the parent), `fragments`, `continental_frac`, `area_km2`, `coast_split_km` (continental margin the rift cut off its ocean, km) | continental fraction | the `large_igneous_province` under the plate, when one boosted it |
| `ridge_birth` | `plate_a`, `plate_b`, `spreading_mm_yr`, `new_samples` (L0 samples of new crust between them) | spreading / 100 mm/yr | the `rift_onset` that separated them, if any |
| `subduction_onset` | `subducting_plate`, `overriding_plate`, `setting` (`ocean_ocean`, `ocean_continent`, `continent_continent`), `convergence_mm_yr`, `slab_age_myr` | convergence / 100 mm/yr | — |
| `collision` | `plate_a` (the terrane's former plate), `plate_b` (the receiving plate), `terrane_area_km2`, `uplift_m`, `radius_km`, `convergence_mm_yr`, `sutured` (1: plate_a welded whole to plate_b, which then dies `sutured`), `coast_split_km` (the terrane's margin left facing plate_a's ocean, km; 0 with a suture) | uplift, km | the contact's `subduction_onset` |
| `terrane_accretion` | `from_plate`, `to_plate`, `area_km2`, `samples`; without a collision also `uplift_m`, `sutured`, `coast_split_km` (as for `collision`) | area / average initial plate area | its `collision`, or the contact's `subduction_onset` |
| `orogeny_phase` | `orogeny` (`andean` or `himalayan`), `plate`; Andean: `subducting_plate`, `max_uplift_mm_yr`, `ongoing` (1 when the run ended first); Himalayan: `uplift_m`, `uplifted_samples` | uplift / u_0 (Andean), uplift in km (Himalayan) | `subduction_onset` (Andean), `collision` (Himalayan) |
| `hotspot_track` | `hotspot`, `plates` (the plates it sat under, e.g. `"3,7,12"`), `track_km`, `uplift_mm_yr`, `ongoing` | strength relative to the preset's | its `large_igneous_province`, if it had a plume head |
| `large_igneous_province` | `area_km2`, `volume_km3`, `thickness_m`, `plate` | lava pile, km | — |

- `subduction_onset` is written once a contact has lasted 3 steps while converging at ≥ 10 mm/yr
  over ≥ max(3, n/10,000) samples, and dated to when the contact began; flickers at transform
  boundaries never get one. A polarity reversal closes the contact and later writes a new onset.
- `orogeny_phase` spans `t_start`–`t_end`: an Andean one from its subduction onset to when the
  contact closed (only when it uplifted continental crust for ≥ 10 Myr), a Himalayan one
  `collision_orogeny_myr` from its collision.
- `hotspot_track` spans the hotspot's life (from the run's start for hotspots older than it).
- Terranes below `terrane_event_min_frac` of the average initial plate area accrete without an
  event; below `collision_min_frac` they make a `terrane_accretion` only.
- The L1 layer `plates/regime` uses these codes: 0 ocean basin, 1 ridge, 2 trench, 3 arc,
  4 back-arc, 5 collision, 6 rift, 7 hotspot, 8 craton, 9 passive margin.

### The surface's events (source `deeptime:surface`, P1-04)

Written by `crates/deeptime-core/src/surface/events.rs` (plan §8.2) once per frame of the deposit
engine's history (`surface.frame_every_myr`, 10 Myr), from the frame-mean rates. Regions are
8-connected groups of L1 cells of at least `surface.event_min_area_km2` (true area); a region whose
centroid lies within 250 km (on the planet) of a tracked one continues it, and a region is written
**when it ends** (or, still active, at the end of the run with `ongoing` = 1), spanning its life.
Places are points: the region's centroid in its strongest frame. Areas and volumes are true
(cos φ-weighted) km² and km³.

| Kind | Region | Params | Magnitude | Causes |
|---|---|---|---|---|
| `erosion_pulse` | land eroding ≥ `erosion_pulse_m_myr` (200 m/Myr) | `peak_rate_m_myr`, `area_km2` (largest), `volume_km3` (eroded over its life), `ongoing` | peak rate / (4 × threshold), ≤ 1 | the nearest `orogeny_phase`, `collision` or `subduction_onset` active within 400 km and from 20 Myr before its start |
| `basin_subsidence` | cells aggrading ≥ `basin_m_myr` (80 m/Myr), sea floor deeper than 2 km excluded | as above (`volume_km3` = deposited) plus `setting` (the plate regime covering most of it: `rift`, `passive_margin`, `back_arc` = foreland, `craton` = intracratonic, `trench`, …) | peak rate / (4 × threshold), ≤ 1 | the nearest `rift_onset`, `collision`, `orogeny_phase`, `subduction_onset` or `ridge_birth`, as above |
| `marine_transgression`, `marine_regression` | the share f of continental crust (by area) under the sea, followed as a zigzag: a move of at least `transgression_frac` (0.04) that then reverses by as much is one event from its start to its turning point (and a last one, `ongoing` = 1, at the end of the run) | `flooded_from`, `flooded_to` (f), `sea_level_from_m`, `sea_level_to_m` (the preset's eustatic curve), `area_km2` (the land the sea gained or lost), `ongoing` | \|Δf\| / 0.3, ≤ 1 | — |

Place of a transgression or regression: the centroid of the continental cells flooded at its end and
not at its start (regression: the reverse); global when there are none.

### The impacts' events (source `deeptime:impacts`, P2-08)

Written by `crates/deeptime-core/src/surface/impacts/` at the end of the coupled group (after the
last plate step, like the volcanoes'), in time order: one event for every impact that left a crater
at least 5 km across (`EVENT_MIN_KM`), whatever became of the crater, and one for every smaller
crater that still shows at the end (at least 40 m of relief). Most small craters are gone within a
few tens of Myr; they are counted in `notes.impacts` and not listed. The event of a crater still in
the catalogue is placed where its plate has carried it (where it struck is in `impact_lat`,
`impact_lon`). Impacts have no cause in the log.

| Kind | Time | Place | Params | Magnitude |
|---|---|---|---|---|
| `impact_crater` | the impact (an instant inside its step) | the crater now; where it struck for a crater that is gone | `crater` (catalogue id), `diameter_km` (rim to rim), `radius_km`, `morphology` (`simple`, `complex`, `peak_ring`), `depth_m` and `rim_m` (fresh, in game metres: the physical ones × `impacts.depth_scale`), `impactor_m`, `energy_mt` (megatons of TNT), `target` (`land`, `sea_floor`), `water_depth_m`, `melt_km3`, `melt_sheet_m` (0 without a sheet), `impact_lat`, `impact_lon`, `plate`, `fate` (`fresh`, `degraded`, `buried`, `eroded_to_roots` for a crater in the catalogue; `eroded`, `subducted`, `overprinted` for one that left it), `relief_m` (what is left of `depth_m`), `cover_m` (sediment over its floor), `standing` (1 in the catalogue, 0 gone) | diameter / 100 km, at most 1 |
| `impact_ocean` | the impact | where it struck | `impactor_m`, `energy_mt`, `water_depth_m`, `crater_on_land_km` (the crater the same body would have made on land, at least 20 km) | that diameter / 100 km, at most 1 |

The deposit engine reads `impact_crater` as a cause of the impact deposit types (`impact_ni_cu_pge`,
`impact_hydrothermal_cu_zn_au`, `impact_diamond`, `impact_basin_u`) and of petroleum traps in impact
structures.

### The volcanoes' events (source `deeptime:volcanoes`, P1-21)

Written by `crates/deeptime-core/src/surface/volcanoes/` at the end of the coupled group (after the
plate model has closed its hotspot tracks), so every cause exists when they are written. Grouped (Ben,
2026-09-30, "Group by arc/chain"): one event per **arc segment** — the stratovolcanoes one subduction
contact fed (keyed by its onset event; by plate where the contact never had one) — and one per
**hotspot chain**, each with its standing cones as a detail; on their own only the notable ones: each
group's tallest standing cone (a stratovolcano at least 1.5 km × `volcanoes.size`), the calderas whose
volcano still stands or that are at least 6 km across (× size), and the flood basalts. Heights are relief
above the ground round the edifice, m; `summit_m` its summit above the sea at the end; radii and areas
block-metric (the world's own), km and km². A **cone list** (`cones`) is `lat,lon,radius_km,height_m`
per standing cone, `;`-separated, tallest first, at most 80; `inspect-column --why` finds a column's
cone in it. An arc segment that built fewer than 3 cones (Ben, 2026-09-30, "Count small ones") has no event of
its own: it is counted on its `subduction_onset` event, which gains `volcano_cones_built`,
`volcano_cones_standing` and `volcano_cones` (the same list format), added at the end of the coupled
group; its notable cone, if any, is caused by that onset.

| Kind | t_start (t_end) | Place | Params | Magnitude | Causes |
|---|---|---|---|---|---|
| `volcano_arc` | its first cone's birth (its last live step) | the centroid of its standing cones, else of where its cones were born | `setting` (`continental_arc`, `island_arc`: where most were born), `cones_built`, `live`, `standing`, `tallest_m`, `peak_m` (the tallest any stood), `calderas`, `plate`, `ongoing`, `cones` | cones built / 50, ≤ 1 | its subduction onset |
| `volcano_hotspot_chain` | its first volcano's birth (its last death, or the end) | its youngest volcano | `hotspot`, `volcanoes_built`, `standing`, `islands`, `seamounts`, `subducted`, `tallest_m`, `ongoing`, `cones` | volcanoes / 20, ≤ 1 | the hotspot's `hotspot_track`, else its `large_igneous_province` |
| `volcano_stratovolcano` | its birth (its death, or the end while active) | its summit at the end | `volcano` (catalogue id), `setting`, `magma` (`andesitic`, `silicic`), `peak_height_m`, `height_m`, `radius_km`, `summit_m`, `arc_flux` (mean while active, 0–1), `plate`, `fate` (`active`, `extinct`), `ongoing` | peak height / 3 km | its `volcano_arc`, or the `subduction_onset` of a small segment |
| `volcano_shield` | its birth over the plume (its death) | as above | as above plus `hotspot`, `chain_index`; `magma` `basaltic`; `fate` (`active`, `island`, `seamount`) | as above | its `volcano_hotspot_chain` |
| `volcano_caldera_complex` | as a shield (a continental hotspot) | as above | as `volcano_shield`, `magma` = `silicic` | as above | its `volcano_hotspot_chain` |
| `volcano_caldera` | the collapse | where its volcano stood then | `volcano`, `radius_km`, `depth_m`, `erupted_km3`, `type` (`silicic`: the volcano collapsed and died; `summit`: a shield's summit caldera) | erupted volume / 1,000 km³, ≤ 1 | its volcano's event, else its arc segment or chain |
| `volcano_flood_basalt` | the eruption | the province's centre at the end | `area_km2`, `thickness_m` (the pile at its centre), `flows`, `preserved_frac` (share of its area where the pile still crops out within 150 m of the surface), `pile_frac` (share where any of it is left), `plate` | pile / 1 km | the plate model's `large_igneous_province` |

### The climate's events (source `deeptime:climate`, P1-05)

Written by `crates/deeptime-core/src/climate/` (plan §8.3 item 6; `glaciers.rs`, `clock.rs`,
`mod.rs`). **Glaciations** are decided at the end of every plate step (since the deep-time climate;
at each 10-Myr snapshot before): a cell is glaciated when land ice stands on it
(`climate/ice_thickness_m` ≥ 1 m: its land above its equilibrium line, from the summer temperature
and Ohmura et al. 1992's precipitation relation, the line moving with the fine clock's climate). A
glaciation starts at the first step where ice covers at least `glaciation_on_frac` (0.02) of the
land (true area) and ends at the first where it covers less than `glaciation_off_frac` (0.01);
still open at the end of the run it is written with `ongoing` = 1. Place: the ice's area-weighted
centroid at its largest.

**Snowballs and hothouses** come from the fine clock (`clock.rs`: carbon, EBM and ice every
100 kyr, every 2 kyr in the recent ice-age window), not from a script: a snowball starts when sea
ice covers `climate.snowball_on_frac` (0.95) of the ocean (annual mean) — the ice-albedo runaway
takes the ice edge from mid latitudes to the equator — and ends when the cover falls below
`snowball_off_frac` (0.5), written at its end (or at the end of the run with `ongoing` = 1). A
hothouse is a stretch with the EBM's global mean above `climate.hothouse_c` (25 °C), ending once
it is 1 °C below. Both are global (place `Global`).

| Kind | Params | Magnitude | Causes |
|---|---|---|---|
| `glaciation` | `peak_land_frac` (glaciated share of the land at its largest), `peak_area_km2`, `peak_t_years`, `ongoing` | peak share / 0.2, ≤ 1 | — |
| `snowball` | `duration_myr`; CO₂ (ppm) `co2_before_ppm` (the most in the 10 Myr before the onset), `co2_onset_ppm`, `co2_escape_ppm`, `co2_max_ppm`, `co2_drawdown_factor` (the most / the least CO₂ of the 10 Myr before the onset), `co2_rise_factor` (escape / onset); `tropical_land_share` and `tropical_weathering_share` (within 30° of the equator, just before the onset); `degassing_onset_rel`, `degassing_escape_rel` (× Earth's today); `sun_rel`; `t_global_min_c`; `cap_carbonate_m` (the drawdown carbonate the escape's plate step laid on the sea floor above the compensation depth, ≤ 4 km: weathering beyond degassing precipitates as carbonate — the cap carbonate); `onset_cause` (`weathering_drawdown` when CO₂ fell by ≥ 1.5× over the 10 Myr before, else `slow_cooling`); `escape_cause` (`degassing_buildup` when CO₂ rose by ≥ 2× while frozen, else `warming`; `none` while ongoing); `ongoing` | duration / 50 Myr, ≤ 1 | — |
| `hothouse` | `peak_t_global_c`, `peak_t_years`, `co2_peak_ppm`, `after_snowball` (1 when it began within 5 Myr of a snowball's end), `ongoing` | (peak − `hothouse_c`) / 25 °C, ≤ 1 | the `snowball` it followed |

### The dummy preset's events (source `deeptime:dummy`, until P1-03)

The dummy stages (`crates/deeptime-core/src/dummy.rs`) emit, every `event_every_steps` steps, one
of `rift_onset`, `collision`, `orogeny_phase` (lasting three steps), `erosion_pulse` in rotation,
at the drifting continent's centre (a point place), each caused by the one before it. Magnitude is
the continent's peak height / 8,000 m. Params: `stage` (str, the stage name), `step` (i64, the run's
step index), `peak_y` (f64, the peak's block y). At the end of the last stage one
`deposit_formation` per site, caused by the last chain event, with a site place.

### Deposit events (source `deeptime:deposits`, P1-14)

The deposit stage (`crates/deeptime-core/src/deposits/`) writes one forming event per site, whose
`causes` are the "Why here?" chain: per mineral-system component the event of the kinds the deposit
type lists (`causes.source`, `.driver`, `.pathway`, `.trap`) nearest the site in space (within the
type's `cause_radius_km`) and time (active at the formation or ended at most 100 Myr before; any
time for a type dated by its cause); a deposit whose components find nothing cites the nearest
earlier geology event. Scores are 0–1. Every forming event's place is its site, except for deposits
that erosion destroyed (point place, `site` −1).

| Kind | t_start (t_end) | Magnitude | Causes | Params |
|---|---|---|---|---|
| `deposit_formation` | the formation time: the cell's most favourable frame (jittered inside it), or the earliest cause for types dated by their cause (inherited BIF) | the tonnage's quantile in its distribution | component events | `deposit_type` str, `site` i64 (−1 = destroyed), `favourability` f64 (the cell's integrated F; for petroleum the charged trap × seal × reservoir); rule-based types add `score_source`, `score_driver`, `score_pathway`, `score_trap`, `score_host`, `score_clock` f64 (at the best frame), `limiting` str (the site's weakest component), `neighbours_limited_by` str (the component that most often limits the barren cells within 3 L1 cells), `host_class` str (the host class that scored best; empty without host fields); `tonnage_mt` f64 (as drawn), `tonnage_quantile` f64, `formation_depth_m` f64, `province_endowment` f64, `fate` str (`buried`, `truncated`, `eroded`), `fraction_left` f64; coal adds `rank` str and `tti` f64 |
| `deposit_exhumation` | when erosion reached the body's top (truncated) or its base (destroyed) | 0 | the deposit's formation event, the nearest `causes.preservation` event (an erosion pulse) | `deposit_type` str, `site` i64 (−1 = destroyed), `eroded_m` f64 (since formation), `fraction_left` f64, `destroyed` i64 (0/1) |
| `placer_formation` | the parent's destruction | the placer's tonnage quantile | the parent's `deposit_exhumation` (+ preservation events) | `deposit_type`, `site`, `parent_type` str, `transport_km` f64, `resistate_lost_t` f64 (resistate metal the parent lost), `tonnage_mt` f64 |
| `supergene_enrichment` | start of the stable window (to the present) | k / 3 | the parent's formation and exhumation events (+ preservation events) | `deposit_type`, `site`, `parent` i64 (site), `grade_mult` f64 (k), `stable_myr` f64, `water_table_m` f64, `leached_t` f64 (parent ore leached into the blanket) |
| `petroleum_generation` (point place, a kitchen's centre) | the kitchen's earliest oil-window onset (to its last generating frame) | min(TTI_max / TTI_peak, 4) / 4 | the kitchen's `causes.source` events (subsidence, rifting) | `deposit_type` str, `kitchen` i64, `tti_max` f64, `area_km2` f64, `toc_mean` f64 |
| `trap_charge` (a reservoir's forming event) | the charge time: max(trap formation, kitchen onset) | tonnage quantile | the kitchen's `petroleum_generation` + component events | as `deposit_formation`, plus `kitchen` i64, `migration_km` f64, `trap_formed_years` f64 |
| `guaranteed_occurrence` | the cell's most favourable frame | 0 | component events as for a natural deposit | `deposit_type`, `site`, `material` str, `region` str (`col,row` of the essentials grid), `favourability` f64, `limiting` str, `score` f64, `reason` str |

### The synthetic history's events (source `deeptime:synthetic`, P1-14)

`deposits_synthetic` presets stand in for P1-03…P1-07 with an invented planet
(`crates/deeptime-core/src/deposits/synthetic.rs`); it emits the tectonic events a real history
would, at point places along its features: `subduction_onset`, `terrane_accretion`,
`orogeny_phase`, `erosion_pulse`, `collision`, `marine_transgression`, `basin_subsidence` and
`rift_onset`, each with the param `feature` (str, e.g. `A margin`, `C rift`).
