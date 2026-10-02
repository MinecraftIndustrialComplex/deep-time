// SPDX-License-Identifier: LGPL-3.0-or-later
//! The shared event log (plan §5.5): one append-only log of notable happenings, written by several
//! independent writers (geology is writer 1; the civ module registers another) without
//! coordination. An event id is `writer << 48 | sequence`, so two writers never collide.
//!
//! On disk: immutable segments `events/<writer>-<segment>.dtev` plus `events/index.json`. The byte
//! layout, the JSON form and every kind's params are specified in `EVENTS.md` next to this crate's
//! `Cargo.toml`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::FORMAT_VERSION;
use crate::container::{self, StringTable, f32_at, f64_at, string_at, u16_at, u32_at, u64_at};
use crate::error::{FormatError, Result};
use crate::fsutil::write_atomic;
use crate::raster::GridDesc;

/// Magic of an event segment.
pub const MAGIC: [u8; 4] = *b"DTEV";
/// Bytes per event record.
pub const RECORD_BYTES: usize = 64;
/// Bytes per params entry.
pub const PARAM_BYTES: usize = 16;
/// Records per time-index block.
pub const TIME_BLOCK: usize = 64;
/// Bytes per time-index entry.
pub const TIME_ENTRY_BYTES: usize = 24;
/// The segment's fixed header fields (after the common 16 bytes).
const FIXED_BYTES: usize = 32;
/// Sections: strings, records, causes, params, time index.
const SECTIONS: usize = 5;

/// The folder of the event log inside a world-data folder.
pub const EVENTS_DIR: &str = "events";
/// The segment list, relative to the world-data folder.
pub const INDEX_PATH: &str = "events/index.json";

/// Writer 1: the geology core (`deeptime-core`). Writer 0 is reserved (never a valid event).
pub const GEOLOGY_WRITER: u16 = 1;
/// The largest sequence number (48 bits).
pub const MAX_SEQ: u64 = (1 << 48) - 1;

/// `writer << 48 | sequence` (plan §5.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl EventId {
    /// The id of `writer`'s event number `seq`. Panics if `seq` does not fit in 48 bits.
    pub fn new(writer: u16, seq: u64) -> EventId {
        assert!(seq <= MAX_SEQ, "event sequence {seq} exceeds 48 bits");
        EventId((u64::from(writer) << 48) | seq)
    }

    pub fn writer(self) -> u16 {
        (self.0 >> 48) as u16
    }

    pub fn seq(self) -> u64 {
        self.0 & MAX_SEQ
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.writer(), self.seq())
    }
}

/// Where an event happened.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Place {
    /// The whole planet (a glaciation, an atmosphere switch).
    Global,
    /// Degrees: latitude north positive; longitude in [0°, 360°) east of the seam, as plan §5.2
    /// (block x = lon/360 · C − C/2, z = −lat/90 · C/4).
    Point {
        lat: f32,
        lon: f32,
    },
    Plate {
        plate: u32,
    },
    /// A stored L1 bitmask (e.g. an orogen's footprint); masks are stored from P1-03 on.
    Region {
        mask: u32,
    },
    Site {
        site: u32,
    },
    /// A stored polyline (a trench, a river); paths are stored from P1-03 on.
    Path {
        path: u32,
    },
}

impl Place {
    /// The on-disk tag (`EVENTS.md`).
    pub fn tag(&self) -> u8 {
        match self {
            Place::Global => 0,
            Place::Point { .. } => 1,
            Place::Plate { .. } => 2,
            Place::Region { .. } => 3,
            Place::Site { .. } => 4,
            Place::Path { .. } => 5,
        }
    }

    pub fn to_json(&self) -> Value {
        match *self {
            Place::Global => json!({ "type": "global" }),
            Place::Point { lat, lon } => json!({ "type": "point", "lat": lat, "lon": lon }),
            Place::Plate { plate } => json!({ "type": "plate", "plate": plate }),
            Place::Region { mask } => json!({ "type": "region", "mask": mask }),
            Place::Site { site } => json!({ "type": "site", "site": site }),
            Place::Path { path } => json!({ "type": "path", "path": path }),
        }
    }
}

/// A kind-defined parameter value (plan §5.5: string → i64 | f64 | str | id).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParamValue {
    Int(i64),
    Float(f64),
    Str(String),
    /// An identifier: an event id, site id, plate id, … (the kind's `EVENTS.md` entry says which).
    Id(u64),
}

impl ParamValue {
    fn tag(&self) -> u8 {
        match self {
            ParamValue::Int(_) => 0,
            ParamValue::Float(_) => 1,
            ParamValue::Str(_) => 2,
            ParamValue::Id(_) => 3,
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            ParamValue::Int(v) => json!(v),
            ParamValue::Float(v) if v.is_finite() => json!(v),
            ParamValue::Float(_) => Value::Null,
            ParamValue::Str(s) => json!(s),
            ParamValue::Id(v) => json!({ "id": v }),
        }
    }
}

impl From<i64> for ParamValue {
    fn from(v: i64) -> Self {
        ParamValue::Int(v)
    }
}
impl From<f64> for ParamValue {
    fn from(v: f64) -> Self {
        ParamValue::Float(v)
    }
}
impl From<&str> for ParamValue {
    fn from(v: &str) -> Self {
        ParamValue::Str(v.to_owned())
    }
}
impl From<String> for ParamValue {
    fn from(v: String) -> Self {
        ParamValue::Str(v)
    }
}
impl From<EventId> for ParamValue {
    fn from(v: EventId) -> Self {
        ParamValue::Id(v.0)
    }
}

/// A kind-defined typed map, ordered by key.
pub type Params = BTreeMap<String, ParamValue>;

/// One event (plan §5.5). Kinds and sources are namespaced ids (`deeptime:rift_onset`,
/// `deeptime:plates`); on disk they are interned in the segment's string table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    /// Years relative to world creation (negative = geologic past).
    pub t_start: f64,
    /// `= t_start` for instantaneous events.
    pub t_end: f64,
    pub kind: String,
    pub source: String,
    pub place: Place,
    /// Kind-defined scalar for ranking and filtering.
    pub magnitude: f32,
    /// Causal links, possibly across writers.
    pub causes: Vec<EventId>,
    pub params: Params,
}

impl Event {
    /// An instantaneous, global event with magnitude 0, no causes and no params.
    pub fn new(id: EventId, t: f64, kind: &str, source: &str) -> Event {
        Event {
            id,
            t_start: t,
            t_end: t,
            kind: kind.to_owned(),
            source: source.to_owned(),
            place: Place::Global,
            magnitude: 0.0,
            causes: Vec::new(),
            params: Params::new(),
        }
    }

    pub fn until(mut self, t_end: f64) -> Event {
        self.t_end = t_end;
        self
    }

    pub fn at(mut self, place: Place) -> Event {
        self.place = place;
        self
    }

    pub fn magnitude(mut self, m: f32) -> Event {
        self.magnitude = m;
        self
    }

    pub fn caused_by(mut self, cause: EventId) -> Event {
        self.causes.push(cause);
        self
    }

    pub fn param(mut self, key: &str, value: impl Into<ParamValue>) -> Event {
        self.params.insert(key.to_owned(), value.into());
        self
    }

    /// Checks the invariants every writer must keep (`EVENTS.md`, "Rules").
    pub fn validate(&self) -> Result<()> {
        let bad = |m: String| Err(FormatError::Invalid(format!("event {}: {m}", self.id)));
        if self.id.writer() == 0 {
            return bad("writer 0 is reserved".into());
        }
        if !self.t_start.is_finite() || !self.t_end.is_finite() || self.t_end < self.t_start {
            return bad(format!(
                "times must be finite with t_end ≥ t_start, got [{}, {}]",
                self.t_start, self.t_end
            ));
        }
        for (what, id) in [("kind", &self.kind), ("source", &self.source)] {
            if !is_namespaced(id) {
                return bad(format!("{what} {id:?} is not a namespaced id (ns:path)"));
            }
        }
        if self.params.keys().any(String::is_empty) {
            return bad("empty param key".into());
        }
        if self.causes.contains(&self.id) {
            return bad("an event cannot cause itself".into());
        }
        Ok(())
    }

    /// The JSON form (`EVENTS.md`, "JSON"): what `eventsSince` returns and the viewer reads.
    pub fn to_json(&self) -> Value {
        let params: serde_json::Map<String, Value> = self
            .params
            .iter()
            .map(|(k, v)| (k.clone(), v.to_json()))
            .collect();
        json!({
            "id": self.id.0,
            "writer": self.id.writer(),
            "seq": self.id.seq(),
            "t_start": self.t_start,
            "t_end": self.t_end,
            "kind": self.kind,
            "source": self.source,
            "place": self.place.to_json(),
            "magnitude": if self.magnitude.is_finite() { json!(self.magnitude) } else { Value::Null },
            "causes": self.causes.iter().map(|c| c.0).collect::<Vec<_>>(),
            "params": params,
        })
    }
}

/// `ns:path`, both parts non-empty, lower-case ASCII letters, digits and `_ - . /`.
pub fn is_namespaced(id: &str) -> bool {
    let Some((ns, path)) = id.split_once(':') else {
        return false;
    };
    let ok = |s: &str, extra: &[char]| {
        !s.is_empty()
            && s.chars().all(|c| {
                c.is_ascii_lowercase()
                    || c.is_ascii_digit()
                    || "_-.".contains(c)
                    || extra.contains(&c)
            })
    };
    ok(ns, &[]) && ok(path, &['/'])
}

/// Sort order inside a segment and of a merged log: by start time, then id.
fn order(a: &Event, b: &Event) -> std::cmp::Ordering {
    a.t_start.total_cmp(&b.t_start).then(a.id.cmp(&b.id))
}

/// A decoded segment.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub writer: u16,
    /// The `<segment>` of the file name.
    pub segment: u32,
    /// Sorted by (t_start, id).
    pub events: Vec<Event>,
}

/// Encodes writer `writer`'s segment number `segment` (`EVENTS.md`, "Segment"). Events are
/// validated and written sorted by (t_start, id); every id must belong to `writer`.
pub fn encode_segment(writer: u16, segment: u32, events: &[Event]) -> Result<Vec<u8>> {
    if events.is_empty() {
        return Err(FormatError::Invalid(
            "an event segment needs at least one event".into(),
        ));
    }
    let mut seen = HashSet::with_capacity(events.len());
    for e in events {
        e.validate()?;
        if e.id.writer() != writer {
            return Err(FormatError::Invalid(format!(
                "event {} does not belong to writer {writer}",
                e.id
            )));
        }
        if !seen.insert(e.id) {
            return Err(FormatError::Invalid(format!(
                "event {} appears twice",
                e.id
            )));
        }
    }
    let n = u32::try_from(events.len())
        .map_err(|_| FormatError::Invalid("too many events in one segment".into()))?;
    let mut sorted: Vec<&Event> = events.iter().collect();
    sorted.sort_by(|a, b| order(a, b));

    let mut strings = StringTable::default();
    let mut records = Vec::with_capacity(sorted.len() * RECORD_BYTES);
    let mut causes = Vec::new();
    let mut params = Vec::new();
    let (mut t_min, mut t_max) = (f64::INFINITY, f64::NEG_INFINITY);
    for e in &sorted {
        t_min = t_min.min(e.t_start);
        t_max = t_max.max(e.t_end);
        let kind = strings.intern(&e.kind);
        let source = strings.intern(&e.source);
        let (a, b) = match e.place {
            Place::Global => (0, 0),
            Place::Point { lat, lon } => (lat.to_bits(), lon.to_bits()),
            Place::Plate { plate } => (plate, 0),
            Place::Region { mask } => (mask, 0),
            Place::Site { site } => (site, 0),
            Place::Path { path } => (path, 0),
        };
        let causes_first = (causes.len() / 8) as u32;
        for c in &e.causes {
            causes.extend_from_slice(&c.0.to_le_bytes());
        }
        let params_first = (params.len() / PARAM_BYTES) as u32;
        for (k, v) in &e.params {
            let key = strings.intern(k);
            params.extend_from_slice(&key.to_le_bytes());
            params.extend_from_slice(&[v.tag(), 0, 0, 0]);
            let raw: u64 = match v {
                ParamValue::Int(i) => *i as u64,
                ParamValue::Float(f) => f.to_bits(),
                ParamValue::Str(s) => u64::from(strings.intern(s)),
                ParamValue::Id(id) => *id,
            };
            params.extend_from_slice(&raw.to_le_bytes());
        }
        records.extend_from_slice(&e.id.0.to_le_bytes());
        records.extend_from_slice(&e.t_start.to_le_bytes());
        records.extend_from_slice(&e.t_end.to_le_bytes());
        records.extend_from_slice(&kind.to_le_bytes());
        records.extend_from_slice(&source.to_le_bytes());
        records.extend_from_slice(&e.magnitude.to_le_bytes());
        records.extend_from_slice(&[e.place.tag(), 0, 0, 0]);
        records.extend_from_slice(&a.to_le_bytes());
        records.extend_from_slice(&b.to_le_bytes());
        records.extend_from_slice(&causes_first.to_le_bytes());
        records.extend_from_slice(&(e.causes.len() as u32).to_le_bytes());
        records.extend_from_slice(&params_first.to_le_bytes());
        records.extend_from_slice(&(e.params.len() as u32).to_le_bytes());
    }
    let mut time = Vec::with_capacity(sorted.len().div_ceil(TIME_BLOCK) * TIME_ENTRY_BYTES);
    for (b, block) in sorted.chunks(TIME_BLOCK).enumerate() {
        let max_end = block
            .iter()
            .map(|e| e.t_end)
            .fold(f64::NEG_INFINITY, f64::max);
        time.extend_from_slice(&block[0].t_start.to_le_bytes());
        time.extend_from_slice(&max_end.to_le_bytes());
        time.extend_from_slice(&((b * TIME_BLOCK) as u32).to_le_bytes());
        time.extend_from_slice(&(block.len() as u32).to_le_bytes());
    }
    let strings_bytes = strings.encode();
    let mut fixed = Vec::with_capacity(FIXED_BYTES);
    fixed.extend_from_slice(&writer.to_le_bytes());
    fixed.extend_from_slice(&0u16.to_le_bytes());
    fixed.extend_from_slice(&segment.to_le_bytes());
    fixed.extend_from_slice(&n.to_le_bytes());
    fixed.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    fixed.extend_from_slice(&t_min.to_le_bytes());
    fixed.extend_from_slice(&t_max.to_le_bytes());
    container::encode(
        MAGIC,
        &fixed,
        &[&strings_bytes, &records, &causes, &params, &time],
    )
}

/// Decodes and checks a segment (`path` only names it in errors).
pub fn decode_segment(path: &Path, b: &[u8]) -> Result<Segment> {
    let p = container::parse(path, b, MAGIC, FIXED_BYTES, |_| SECTIONS)?;
    let f = p.fixed;
    let writer = u16_at(f, 0);
    let segment = u32_at(f, 4);
    let n = u32_at(f, 8) as usize;
    let s = u32_at(f, 12) as usize;
    let [strings_b, records, causes, params, time] = [
        p.sections[0],
        p.sections[1],
        p.sections[2],
        p.sections[3],
        p.sections[4],
    ];
    let corrupt = |m: String| FormatError::corrupt(path, m);
    if records.len() != n * RECORD_BYTES {
        return Err(corrupt(format!(
            "{} record bytes for {n} events",
            records.len()
        )));
    }
    if causes.len() % 8 != 0 || params.len() % PARAM_BYTES != 0 {
        return Err(corrupt(
            "cause or params section not a whole number of entries".into(),
        ));
    }
    if time.len() != n.div_ceil(TIME_BLOCK) * TIME_ENTRY_BYTES {
        return Err(corrupt("time index has the wrong length".into()));
    }
    let strings = container::decode_strings(path, strings_b, s)?;
    let mut events = Vec::with_capacity(n);
    for r in 0..n {
        let at = r * RECORD_BYTES;
        let rec = &records[at..at + RECORD_BYTES];
        let id = EventId(u64_at(rec, 0));
        let a = u32_at(rec, 40);
        let place = match rec[36] {
            0 => Place::Global,
            1 => Place::Point {
                lat: f32::from_bits(a),
                lon: f32_at(rec, 44),
            },
            2 => Place::Plate { plate: a },
            3 => Place::Region { mask: a },
            4 => Place::Site { site: a },
            5 => Place::Path { path: a },
            t => return Err(corrupt(format!("event {id}: place tag {t}"))),
        };
        let (cf, cn) = (u32_at(rec, 48) as usize, u32_at(rec, 52) as usize);
        if (cf + cn) * 8 > causes.len() {
            return Err(corrupt(format!("event {id}: causes past the section")));
        }
        let causes = (cf..cf + cn)
            .map(|i| EventId(u64_at(causes, i * 8)))
            .collect();
        let (pf, pn) = (u32_at(rec, 56) as usize, u32_at(rec, 60) as usize);
        if (pf + pn) * PARAM_BYTES > params.len() {
            return Err(corrupt(format!("event {id}: params past the section")));
        }
        let mut map = Params::new();
        for i in pf..pf + pn {
            let e = &params[i * PARAM_BYTES..(i + 1) * PARAM_BYTES];
            let key = string_at(path, &strings, u32_at(e, 0), "param key")?.to_owned();
            let raw = u64_at(e, 8);
            let value = match e[4] {
                0 => ParamValue::Int(raw as i64),
                1 => ParamValue::Float(f64::from_bits(raw)),
                2 => ParamValue::Str(
                    string_at(path, &strings, raw as u32, "param value")?.to_owned(),
                ),
                3 => ParamValue::Id(raw),
                t => return Err(corrupt(format!("event {id}: param type {t}"))),
            };
            map.insert(key, value);
        }
        let event = Event {
            id,
            t_start: f64_at(rec, 8),
            t_end: f64_at(rec, 16),
            kind: string_at(path, &strings, u32_at(rec, 24), "kind")?.to_owned(),
            source: string_at(path, &strings, u32_at(rec, 28), "source")?.to_owned(),
            place,
            magnitude: f32_at(rec, 32),
            causes,
            params: map,
        };
        if event.id.writer() != writer {
            return Err(corrupt(format!(
                "event {} in a segment of writer {writer}",
                event.id
            )));
        }
        if let Some(prev) = events.last()
            && order(prev, &event) != std::cmp::Ordering::Less
        {
            return Err(corrupt(format!("event {} out of order", event.id)));
        }
        events.push(event);
    }
    for (bi, block) in events.chunks(TIME_BLOCK).enumerate() {
        let e = &time[bi * TIME_ENTRY_BYTES..(bi + 1) * TIME_ENTRY_BYTES];
        let max_end = block
            .iter()
            .map(|e| e.t_end)
            .fold(f64::NEG_INFINITY, f64::max);
        if f64_at(e, 0) != block[0].t_start
            || f64_at(e, 8) != max_end
            || u32_at(e, 16) as usize != bi * TIME_BLOCK
            || u32_at(e, 20) as usize != block.len()
        {
            return Err(corrupt(format!(
                "time index block {bi} does not match the records"
            )));
        }
    }
    Ok(Segment {
        writer,
        segment,
        events,
    })
}

/// A registered writer of the log.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WriterEntry {
    pub id: u16,
    /// A namespaced name: `deeptime:geology`, `civ:history`.
    pub name: String,
}

/// One segment in `events/index.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SegmentEntry {
    /// Relative to the world-data folder: `events/<writer>-<segment>.dtev`.
    pub path: String,
    pub writer: u16,
    pub segment: u32,
    pub events: u32,
    /// The smallest and largest event sequence number in the segment.
    pub first_seq: u64,
    pub last_seq: u64,
    /// Smallest t_start and largest t_end, in years.
    pub t_min: f64,
    pub t_max: f64,
    pub bytes: u64,
    pub crc32: u32,
}

/// `events/index.json`: the writers and their segments.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventIndex {
    pub format_version: String,
    pub writers: Vec<WriterEntry>,
    pub segments: Vec<SegmentEntry>,
}

impl Default for EventIndex {
    fn default() -> Self {
        EventIndex {
            format_version: FORMAT_VERSION.to_owned(),
            writers: Vec::new(),
            segments: Vec::new(),
        }
    }
}

impl EventIndex {
    /// Reads `<world>/events/index.json`; an absent file is an empty index.
    pub fn read(world: &Path) -> Result<EventIndex> {
        let path = world.join(INDEX_PATH);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(EventIndex::default()),
            Err(e) => return Err(FormatError::io(&path, e)),
        };
        let what = || path.display().to_string();
        let value: Value = serde_json::from_slice(&bytes).map_err(|source| FormatError::Json {
            what: what(),
            source,
        })?;
        let found = value
            .get("format_version")
            .and_then(Value::as_str)
            .ok_or_else(|| FormatError::corrupt(&path, "no format_version"))?;
        if found != FORMAT_VERSION {
            return Err(FormatError::Version {
                found: found.to_owned(),
                supported: FORMAT_VERSION.to_owned(),
            });
        }
        serde_json::from_value(value).map_err(|source| FormatError::Json {
            what: what(),
            source,
        })
    }

    /// Writes `<world>/events/index.json` atomically; returns its size and CRC32.
    pub fn write(&self, world: &Path) -> Result<(u64, u32)> {
        let mut json = serde_json::to_vec_pretty(self).map_err(|source| FormatError::Json {
            what: "events/index.json".into(),
            source,
        })?;
        json.push(b'\n');
        let crc = write_atomic(&world.join(INDEX_PATH), &json)?;
        Ok((json.len() as u64, crc))
    }

    pub fn writer_name(&self, id: u16) -> Option<&str> {
        self.writers
            .iter()
            .find(|w| w.id == id)
            .map(|w| w.name.as_str())
    }
}

/// Appends one writer's segments to a world-data folder. Writers never coordinate: each owns its
/// id range. Appends to one folder must not run concurrently (`index.json` is rewritten whole);
/// geology finishes before any other writer starts.
pub struct EventWriter {
    world: PathBuf,
    writer: u16,
    index: EventIndex,
    /// The lowest sequence number this writer has not written yet.
    floor: u64,
    next_seq: u64,
}

impl EventWriter {
    /// Opens `world` for `writer`, registering `name` for it (or checking it matches), and
    /// continues numbering after the writer's highest sequence number already on disk.
    pub fn open(world: &Path, writer: u16, name: &str) -> Result<EventWriter> {
        if writer == 0 {
            return Err(FormatError::Invalid("writer 0 is reserved".into()));
        }
        if !is_namespaced(name) {
            return Err(FormatError::Invalid(format!(
                "writer name {name:?} is not a namespaced id"
            )));
        }
        let mut index = EventIndex::read(world)?;
        match index.writers.iter().find(|w| w.id == writer) {
            Some(w) if w.name != name => {
                return Err(FormatError::Invalid(format!(
                    "writer {writer} is registered as {:?}, not {name:?}",
                    w.name
                )));
            }
            Some(_) => {}
            None => index.writers.push(WriterEntry {
                id: writer,
                name: name.to_owned(),
            }),
        }
        let floor = index
            .segments
            .iter()
            .filter(|s| s.writer == writer)
            .map(|s| s.last_seq + 1)
            .max()
            .unwrap_or(0);
        Ok(EventWriter {
            world: world.to_owned(),
            writer,
            index,
            floor,
            next_seq: floor,
        })
    }

    pub fn writer(&self) -> u16 {
        self.writer
    }

    /// Allocates the next event id of this writer.
    pub fn next_id(&mut self) -> EventId {
        let id = EventId::new(self.writer, self.next_seq);
        self.next_seq += 1;
        id
    }

    /// The index as it stands (the last written state plus this writer's registration).
    pub fn index(&self) -> &EventIndex {
        &self.index
    }

    /// Removes every segment of this writer (files and index entries) and restarts its numbering
    /// at 0: a geology re-run replaces its own history and keeps other writers'.
    pub fn clear(&mut self) -> Result<()> {
        let (mine, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut self.index.segments)
            .into_iter()
            .partition(|s| s.writer == self.writer);
        self.index.segments = keep;
        self.index.write(&self.world)?;
        for s in mine {
            let p = self.world.join(&s.path);
            match std::fs::remove_file(&p) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(FormatError::io(&p, e)),
            }
        }
        self.floor = 0;
        self.next_seq = 0;
        Ok(())
    }

    /// Writes `events` as this writer's next segment and records it in `index.json` (segment
    /// first, index second, both atomically). Every id must be this writer's and above any
    /// sequence number it wrote before. An empty slice writes nothing.
    pub fn append(&mut self, events: &[Event]) -> Result<Option<SegmentEntry>> {
        if events.is_empty() {
            return Ok(None);
        }
        let mut first = u64::MAX;
        let mut last = 0;
        for e in events {
            if e.id.writer() == self.writer && e.id.seq() < self.floor {
                return Err(FormatError::Invalid(format!(
                    "event {} reuses a sequence number below {} already on disk",
                    e.id, self.floor
                )));
            }
            first = first.min(e.id.seq());
            last = last.max(e.id.seq());
        }
        let segment = self
            .index
            .segments
            .iter()
            .filter(|s| s.writer == self.writer)
            .map(|s| s.segment + 1)
            .max()
            .unwrap_or(0);
        let bytes = encode_segment(self.writer, segment, events)?;
        let rel = format!("{EVENTS_DIR}/{}-{segment}.dtev", self.writer);
        let crc32 = write_atomic(&self.world.join(&rel), &bytes)?;
        let t_min = events
            .iter()
            .map(|e| e.t_start)
            .fold(f64::INFINITY, f64::min);
        let t_max = events
            .iter()
            .map(|e| e.t_end)
            .fold(f64::NEG_INFINITY, f64::max);
        let entry = SegmentEntry {
            path: rel,
            writer: self.writer,
            segment,
            events: events.len() as u32,
            first_seq: first,
            last_seq: last,
            t_min,
            t_max,
            bytes: bytes.len() as u64,
            crc32,
        };
        self.index.segments.push(entry.clone());
        self.index.write(&self.world)?;
        self.floor = last + 1;
        self.next_seq = self.next_seq.max(self.floor);
        Ok(Some(entry))
    }
}

/// The merged log of every writer, with the indexes of plan §5.5: by time (sorted), by kind and
/// by cause (reverse links); [`EventLog::cell_index`] builds the L1-cell index.
#[derive(Clone, Debug, Default)]
pub struct EventLog {
    events: Vec<Event>,
    by_id: HashMap<EventId, usize>,
    by_kind: BTreeMap<String, Vec<usize>>,
    effects: HashMap<EventId, Vec<EventId>>,
    max_duration: f64,
    writers: Vec<WriterEntry>,
}

impl EventLog {
    /// Reads every segment listed in `<world>/events/index.json`, checks sizes, CRCs, writers and
    /// id uniqueness, and merges them by (t_start, id). A folder without an index is an empty log.
    pub fn load(world: &Path) -> Result<EventLog> {
        let index = EventIndex::read(world)?;
        let mut all = Vec::new();
        let mut owner: HashMap<EventId, String> = HashMap::new();
        for s in &index.segments {
            let path = world.join(&s.path);
            if index.writer_name(s.writer).is_none() {
                return Err(FormatError::corrupt(
                    world.join(INDEX_PATH),
                    format!("segment {} of unregistered writer {}", s.path, s.writer),
                ));
            }
            let bytes = std::fs::read(&path).map_err(|e| FormatError::io(&path, e))?;
            if bytes.len() as u64 != s.bytes {
                return Err(FormatError::corrupt(
                    &path,
                    format!("{} bytes, index says {}", bytes.len(), s.bytes),
                ));
            }
            let crc = crc32fast::hash(&bytes);
            if crc != s.crc32 {
                return Err(FormatError::Crc {
                    path,
                    what: "segment".into(),
                    expected: s.crc32,
                    actual: crc,
                });
            }
            let seg = decode_segment(&path, &bytes)?;
            if seg.writer != s.writer || seg.segment != s.segment {
                return Err(FormatError::corrupt(
                    &path,
                    format!(
                        "holds segment {}-{}, index says {}-{}",
                        seg.writer, seg.segment, s.writer, s.segment
                    ),
                ));
            }
            for e in &seg.events {
                if let Some(other) = owner.insert(e.id, s.path.clone()) {
                    return Err(FormatError::corrupt(
                        &path,
                        format!("event {} also appears in {other}", e.id),
                    ));
                }
            }
            all.extend(seg.events);
        }
        let mut log = EventLog::from_events(all)?;
        log.writers = index.writers;
        Ok(log)
    }

    /// Builds a log (and its indexes) from events of any writers; ids must be unique.
    pub fn from_events(mut events: Vec<Event>) -> Result<EventLog> {
        events.sort_by(order);
        let mut by_id = HashMap::with_capacity(events.len());
        let mut by_kind: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut effects: HashMap<EventId, Vec<EventId>> = HashMap::new();
        let mut max_duration = 0.0f64;
        for (i, e) in events.iter().enumerate() {
            if by_id.insert(e.id, i).is_some() {
                return Err(FormatError::Invalid(format!(
                    "event {} appears twice",
                    e.id
                )));
            }
            by_kind.entry(e.kind.clone()).or_default().push(i);
            for c in &e.causes {
                effects.entry(*c).or_default().push(e.id);
            }
            max_duration = max_duration.max(e.t_end - e.t_start);
        }
        Ok(EventLog {
            events,
            by_id,
            by_kind,
            effects,
            max_duration,
            writers: Vec::new(),
        })
    }

    /// Every event, by (t_start, id).
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// The registered writers (empty for a log built with [`EventLog::from_events`]).
    pub fn writers(&self) -> &[WriterEntry] {
        &self.writers
    }

    pub fn get(&self, id: EventId) -> Option<&Event> {
        self.by_id.get(&id).map(|&i| &self.events[i])
    }

    /// Events of `kind`, by time.
    pub fn of_kind<'a>(&'a self, kind: &str) -> impl Iterator<Item = &'a Event> + 'a {
        self.by_kind
            .get(kind)
            .into_iter()
            .flatten()
            .map(|&i| &self.events[i])
    }

    /// Events that name `id` among their causes (the reverse links).
    pub fn effects_of(&self, id: EventId) -> &[EventId] {
        self.effects.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Events whose interval [t_start, t_end] overlaps [t0, t1], by time.
    pub fn between(&self, t0: f64, t1: f64) -> Vec<&Event> {
        let hi = self.events.partition_point(|e| e.t_start <= t1);
        let lo = self
            .events
            .partition_point(|e| e.t_start < t0 - self.max_duration);
        self.events[lo..hi.max(lo)]
            .iter()
            .filter(|e| e.t_end >= t0)
            .collect()
    }

    /// The causes of `id`, breadth first, up to `max_depth` links away, each once; unknown ids
    /// (a cause in a writer whose segments are absent) are skipped.
    pub fn cause_chain(&self, id: EventId, max_depth: usize) -> Vec<&Event> {
        let mut out = Vec::new();
        let mut seen = HashSet::from([id]);
        let mut frontier = vec![id];
        for _ in 0..max_depth {
            let mut next = Vec::new();
            for f in frontier {
                let Some(e) = self.get(f) else { continue };
                for &c in &e.causes {
                    if seen.insert(c)
                        && let Some(ce) = self.get(c)
                    {
                        out.push(ce);
                        next.push(c);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        out
    }

    /// The by-cell index on `grid` (normally L1): `Point` places by their latitude and longitude,
    /// `Site` places through `site_lat_lon`. Global, plate, region and path places are not
    /// cell-local and are left out (region masks and paths are stored from P1-03 on).
    pub fn cell_index(
        &self,
        grid: &GridDesc,
        site_lat_lon: impl Fn(u32) -> Option<[f32; 2]>,
    ) -> CellIndex {
        let mut cells: HashMap<u32, Vec<EventId>> = HashMap::new();
        for e in &self.events {
            let ll = match e.place {
                Place::Point { lat, lon } => Some([lat, lon]),
                Place::Site { site } => site_lat_lon(site),
                _ => None,
            };
            if let Some([lat, lon]) = ll {
                let (i, j) = cell_of(grid, lat, lon);
                cells.entry(j * grid.w + i).or_default().push(e.id);
            }
        }
        CellIndex { w: grid.w, cells }
    }
}

/// The grid cell (nearest point sample) at a latitude and longitude (the planet's projection,
/// [`crate::projection`]: the seam is longitude 0°), x wrapped, z clamped.
pub fn cell_of(grid: &GridDesc, lat: f32, lon: f32) -> (u32, u32) {
    let (x, z) = grid
        .projection()
        .block_of(f64::from(lat), f64::from(lon));
    let r = f64::from(grid.r);
    let i = ((x - grid.origin_block_x as f64) / r).round() as i64;
    let j = ((z - grid.origin_block_z as f64) / r).round() as i64;
    (grid.wrap_i(i), grid.clamp_j(j))
}

/// Event ids by grid cell ("what happened here").
#[derive(Clone, Debug)]
pub struct CellIndex {
    w: u32,
    cells: HashMap<u32, Vec<EventId>>,
}

impl CellIndex {
    /// Events at cell (i, j), by time.
    pub fn at(&self, i: u32, j: u32) -> &[EventId] {
        self.cells.get(&(j * self.w + i)).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("dt-events-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn sample(writer: u16, seqs: std::ops::Range<u64>, t0: f64) -> Vec<Event> {
        seqs.map(|s| {
            let id = EventId::new(writer, s);
            let t = t0 + s as f64 * 1.0e6;
            let mut e = Event::new(id, t, "deeptime:rift_onset", "deeptime:plates")
                .until(t + (s % 3) as f64 * 2.5e6)
                .at(match s % 6 {
                    0 => Place::Global,
                    1 => Place::Point {
                        lat: 12.5,
                        lon: -170.25,
                    },
                    2 => Place::Plate { plate: 7 },
                    3 => Place::Region { mask: 2 },
                    4 => Place::Site { site: 9 },
                    _ => Place::Path { path: 1 },
                })
                .magnitude(s as f32 * 0.5)
                .param("step", s as i64)
                .param("ratio", 0.25 * s as f64)
                .param("class", "granite")
                .param("plate_id", ParamValue::Id(u64::MAX - s));
            if s > 0 {
                e = e.caused_by(EventId::new(writer, s - 1));
            }
            e
        })
        .collect()
    }

    #[test]
    fn segment_round_trips() {
        // 150 events: three time-index blocks, every place and param type, unsorted input.
        let mut events = sample(1, 0..150, -5.0e8);
        events.reverse();
        events[3].kind = "deeptime:collision".into();
        let bytes = encode_segment(1, 4, &events).unwrap();
        let seg = decode_segment(Path::new("1-4.dtev"), &bytes).unwrap();
        assert_eq!((seg.writer, seg.segment), (1, 4));
        let mut expect = events.clone();
        expect.sort_by(order);
        assert_eq!(seg.events, expect);
        // Deterministic bytes.
        assert_eq!(encode_segment(1, 4, &expect).unwrap(), bytes);
    }

    #[test]
    fn segments_refuse_bad_input_and_detect_corruption() {
        let ev = sample(1, 0..3, 0.0);
        assert!(encode_segment(2, 0, &ev).is_err(), "foreign writer");
        assert!(encode_segment(1, 0, &[]).is_err(), "empty");
        let mut dup = ev.clone();
        dup.push(ev[0].clone());
        assert!(encode_segment(1, 0, &dup).is_err(), "duplicate id");
        let mut bad = ev.clone();
        bad[1].kind = "no_namespace".into();
        assert!(encode_segment(1, 0, &bad).is_err());
        let mut bad = ev.clone();
        bad[1].t_end = bad[1].t_start - 1.0;
        assert!(encode_segment(1, 0, &bad).is_err());
        let mut bad = ev.clone();
        bad[1].t_start = f64::NAN;
        assert!(encode_segment(1, 0, &bad).is_err());

        let bytes = encode_segment(1, 0, &ev).unwrap();
        let mut flipped = bytes.clone();
        let last = flipped.len() - 30;
        flipped[last] ^= 0x55;
        assert!(decode_segment(Path::new("x"), &flipped).is_err());
    }

    #[test]
    fn two_writers_merge_with_no_id_collision() {
        let world = tmp("merge");
        // Geology writes two segments; its numbering continues across them.
        let mut geo = EventWriter::open(&world, GEOLOGY_WRITER, "deeptime:geology").unwrap();
        let first: Vec<Event> = (0..5)
            .map(|k| {
                Event::new(
                    geo.next_id(),
                    -4.0e8 + k as f64 * 1e7,
                    "deeptime:rift_onset",
                    "deeptime:plates",
                )
            })
            .collect();
        let e0 = geo.append(&first).unwrap().unwrap();
        assert_eq!(e0.path, "events/1-0.dtev");
        let second: Vec<Event> = (0..3)
            .map(|k| {
                Event::new(
                    geo.next_id(),
                    -1.0e8 + k as f64,
                    "deeptime:collision",
                    "deeptime:plates",
                )
                .caused_by(first[k].id)
            })
            .collect();
        let e1 = geo.append(&second).unwrap().unwrap();
        assert_eq!(
            (e1.path.as_str(), e1.first_seq, e1.last_seq),
            ("events/1-1.dtev", 5, 7)
        );
        // A reopened writer continues after its highest sequence number on disk.
        let mut geo2 = EventWriter::open(&world, GEOLOGY_WRITER, "deeptime:geology").unwrap();
        assert_eq!(geo2.next_id(), EventId::new(1, 8));
        assert!(
            geo2.append(&[Event::new(
                EventId::new(1, 2),
                0.0,
                "deeptime:x",
                "deeptime:y"
            )])
            .is_err(),
            "reusing a sequence number on disk is refused"
        );
        assert!(EventWriter::open(&world, GEOLOGY_WRITER, "civ:history").is_err());

        // Civ is writer 2: its own numbering starts at 0 and its ids cannot collide.
        let mut civ = EventWriter::open(&world, 2, "civ:history").unwrap();
        let founded: Vec<Event> = (0..4)
            .map(|k| {
                Event::new(
                    civ.next_id(),
                    10.0 + k as f64,
                    "civ:city_founded",
                    "civ:politics",
                )
                .caused_by(second[0].id)
                .at(Place::Point { lat: 1.0, lon: 2.0 })
            })
            .collect();
        assert_eq!(founded[0].id.seq(), 0);
        let c0 = civ.append(&founded).unwrap().unwrap();
        assert_eq!(c0.path, "events/2-0.dtev");

        let log = EventLog::load(&world).unwrap();
        assert_eq!(log.len(), 12);
        assert_eq!(log.writers().len(), 2);
        let ids: HashSet<EventId> = log.events().iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), 12, "no id collision");
        assert!(log.events().windows(2).all(|w| order(&w[0], &w[1]).is_lt()));
        assert_eq!(log.get(founded[2].id).unwrap().kind, "civ:city_founded");
        assert_eq!(
            log.effects_of(second[0].id).len(),
            4,
            "cross-writer reverse links"
        );
        let chain = log.cause_chain(founded[0].id, 5);
        assert_eq!(
            chain.iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![second[0].id, first[0].id]
        );
        assert_eq!(log.of_kind("deeptime:collision").count(), 3);
        assert_eq!(log.between(-1.0e8, -1.0e8 + 1.5).len(), 2);

        // Geology re-runs: its segments are replaced, civ's stay.
        let mut again = EventWriter::open(&world, GEOLOGY_WRITER, "deeptime:geology").unwrap();
        again.clear().unwrap();
        assert!(!world.join("events/1-0.dtev").exists());
        let id = again.next_id();
        assert_eq!(id, EventId::new(1, 0));
        again
            .append(&[Event::new(
                id,
                -3.0,
                "deeptime:rift_onset",
                "deeptime:plates",
            )])
            .unwrap();
        let log = EventLog::load(&world).unwrap();
        assert_eq!(log.len(), 5);

        // A tampered segment is caught by the index CRC.
        let p = world.join("events/2-0.dtev");
        let mut b = std::fs::read(&p).unwrap();
        let n = b.len();
        b[n - 9] ^= 1;
        std::fs::write(&p, &b).unwrap();
        assert!(matches!(
            EventLog::load(&world),
            Err(FormatError::Crc { .. })
        ));
        std::fs::remove_dir_all(&world).unwrap();
    }

    #[test]
    fn cell_index_places_points_and_sites() {
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap(); // 64 × 32
        let log = EventLog::from_events(vec![
            Event::new(EventId::new(1, 0), 0.0, "deeptime:a", "deeptime:s").at(Place::Point {
                lat: 0.0,
                lon: 180.0,
            }),
            Event::new(EventId::new(1, 1), 1.0, "deeptime:a", "deeptime:s")
                .at(Place::Site { site: 3 }),
            Event::new(EventId::new(1, 2), 2.0, "deeptime:a", "deeptime:s"),
        ])
        .unwrap();
        let idx = log.cell_index(&grid, |s| (s == 3).then_some([0.0, 180.0]));
        // lat 0, lon 180° is block (0, 0): cell (32, 16).
        assert_eq!(cell_of(&grid, 0.0, 180.0), (32, 16));
        assert_eq!(idx.at(32, 16), &[EventId::new(1, 0), EventId::new(1, 1)]);
        // The seam is 0° (and 360°): column 0; the poles clamp.
        assert_eq!(cell_of(&grid, 90.0, 0.0), (0, 0));
        assert_eq!(cell_of(&grid, -90.0, 360.0), (0, 31));
    }

    #[test]
    fn ids_and_json() {
        let id = EventId::new(2, 5);
        assert_eq!(
            (id.writer(), id.seq(), id.to_string()),
            (2, 5, "2:5".into())
        );
        let e = Event::new(id, -1.0, "civ:x", "civ:y")
            .param("n", 3i64)
            .param("f", 0.5)
            .param("s", "a")
            .param("e", EventId::new(1, 1));
        let j = e.to_json();
        assert_eq!(j["id"], json!((2u64 << 48) | 5));
        assert_eq!(j["params"]["n"], json!(3));
        assert_eq!(j["params"]["e"], json!({ "id": (1u64 << 48) | 1 }));
        assert_eq!(j["place"]["type"], "global");
        assert!(is_namespaced("deeptime:plates/rift"));
        assert!(!is_namespaced("Deeptime:x"));
        assert!(!is_namespaced(":x"));
    }
}
