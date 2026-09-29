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
| 3 | region | L1 mask id (masks are stored from P1-03 on) | 0 |
| 4 | site | site id (`sites/sites.dtsr`) | 0 |
| 5 | path | polyline id (paths are stored from P1-03 on) | 0 |

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
| `deeptime:plate_birth` | a plate appears (rifting splits one) | plates (P1-03) | set by P1-03 |
| `deeptime:plate_death` | a plate is consumed | plates (P1-03) | set by P1-03 |
| `deeptime:rift_onset` | continental rifting starts | plates (P1-03); dummy | dummy: see below |
| `deeptime:ridge_birth` | a spreading ridge forms | plates (P1-03) | set by P1-03 |
| `deeptime:subduction_onset` | a subduction zone starts | plates (P1-03) | set by P1-03 |
| `deeptime:collision` | continents collide | plates (P1-03); dummy | dummy: see below |
| `deeptime:terrane_accretion` | a terrane docks | plates (P1-03) | set by P1-03 |
| `deeptime:orogeny_phase` | mountain building, with a duration | plates (P1-03); dummy | dummy: see below |
| `deeptime:hotspot_track` | a hotspot's track over a plate | plates (P1-03) | set by P1-03 |
| `deeptime:large_igneous_province` | a LIP eruption | plates (P1-03) | set by P1-03 |
| `deeptime:basin_subsidence` | a sedimentary basin subsides | surface (P1-04) | set by P1-04 |
| `deeptime:marine_transgression` | the sea advances over land | surface (P1-04) | set by P1-04 |
| `deeptime:marine_regression` | the sea retreats | surface (P1-04) | set by P1-04 |
| `deeptime:glaciation` | an ice age | climate (P1-05) | set by P1-05 |
| `deeptime:erosion_pulse` | a burst of erosion | surface (P1-04); dummy | dummy: see below |
| `deeptime:deposit_formation` | a deposit forms (place: its site) | deposits (P1-14); dummy | `deposit_type` str, `site` i64; P1-14 adds the limiting mineral-system component |
| `deeptime:deposit_exhumation` | erosion exposes a deposit | deposits (P1-14) | set by P1-14 |
| `deeptime:placer_formation` | a placer forms from an eroded source | deposits (P1-14) | set by P1-14 |
| `deeptime:supergene_enrichment` | a supergene blanket forms | deposits (P1-14) | set by P1-14 |
| `deeptime:petroleum_generation` | a source rock enters the oil window | deposits (P1-14) | set by P1-14 |
| `deeptime:trap_charge` | a trap fills with petroleum | deposits (P1-14) | set by P1-14 |
| `deeptime:guaranteed_occurrence` | a deposit added by the playability pass (plan §7.8) | bake (P1-15) | set by P1-15 |

### The dummy preset's events (source `deeptime:dummy`, until P1-03)

The dummy stages (`crates/deeptime-core/src/dummy.rs`) emit, every `event_every_steps` steps, one
of `rift_onset`, `collision`, `orogeny_phase` (lasting three steps), `erosion_pulse` in rotation,
at the drifting continent's centre (a point place), each caused by the one before it. Magnitude is
the continent's peak height / 8,000 m. Params: `stage` (str, the stage name), `step` (i64, the run's
step index), `peak_y` (f64, the peak's block y). At the end of the last stage one
`deposit_formation` per site, caused by the last chain event, with a site place.
