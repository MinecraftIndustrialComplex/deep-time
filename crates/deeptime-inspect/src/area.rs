// SPDX-License-Identifier: LGPL-3.0-or-later
//! `export-area` (P1-20 stage 2, the browser workbench's 3D preview): one box of a generated
//! world as a compact voxel file, with what Deep Time says about it, for
//! `tools/review/pages/area3d/`.
//!
//! **The box** is `width × depth` block columns from (x0, z0) and the blocks y0 ..= y1. x wraps:
//! x0 is folded into the canonical span [−C/2, C/2) first, and the box then runs east from it, so
//! a box may straddle the seam at x = C/2 (≡ −C/2). Column i holds the world's column
//! `fold(x0 + i)`; `x0 + i` is its unwrapped x (past C/2 on the far side of the seam), the x a
//! walk east from the box origin would show.
//!
//! **The file** (`.dtva`, normally gzipped as `.dtva.gz`; the viewer inflates either):
//!
//! ```text
//! 0    4  magic "DTVA"
//! 4    4  u32 LE format version (1)
//! 8    4  u32 LE header length L
//! 12   L  header, UTF-8 JSON (below)
//! …       zero padding to a multiple of 4, then the blob: the sections the header lists,
//!         each at its `offset` (from the blob's start, 4-aligned), little-endian
//! ```
//!
//! Sections: `run_counts` (u16, one per column: runs in it), `run_values` and `run_lengths` (u16,
//! one per run: its entry and its length in blocks), and with a strata layer `strata_tops` (i16,
//! K per column, top y of each horizon, `i16::MIN` unused) and `strata_classes` (u8, K per
//! column, index into `strata.classes`, 255 unused). Columns are in z-major order,
//! `col = (z − z0)·width + i`; a column's runs go bottom-up from y0 and sum to y1 − y0 + 1.
//!
//! **Entries** are (block, tag) pairs: `blocks[b]` is a block id (properties dropped) with its
//! colour, category and ore material; `tags[t]` lists the deposit bodies (and the zones of each)
//! whose SDF holds the block centre (tag 0: none). Air carries no tag. Entry 0 is air.
//!
//! **Explanations** (header): `bodies` in the box, `sites` owning them or overlapping the box
//! with their "why here" chain (from the server's `/deeptime` probes when the runner recorded
//! them, else from the world data's sites and event log), `palette` (rock class → blocks, from
//! `/deeptime probe palette`), `claims` (`/deeptime probe claims`), and `stats`, which include
//! how many rock blocks match the strata pass's prediction and whether `probe site` answers
//! agree with the body tags.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Read, Write};
use std::path::Path;

use deeptime_format::{Body, EventLog, Site};
use serde_json::{Value, json};

use crate::blocks::{self, Registry};
use crate::colors::ColorTable;
use crate::error::{Error, Result};
use crate::image::hex;
use crate::world::World;
use crate::worlddata::WorldData;

/// File magic.
pub const MAGIC: [u8; 4] = *b"DTVA";
/// Format version.
pub const VERSION: u32 = 1;
/// The largest box side accepted without `--max-side` (the workbench's bound).
pub const MAX_SIDE: i32 = 256;

/// x folded into the canonical span [−C/2, C/2); unchanged when `c` ≤ 0 (no wrap known).
pub fn fold(x: i64, c: i64) -> i64 {
    if c <= 0 {
        x
    } else {
        (x + c / 2).rem_euclid(c) - c / 2
    }
}

/// Whether unwrapped column `x` lies in the inclusive x range [lo, hi] on a planet of
/// circumference `c` (ranges may leave the canonical span; `c` ≤ 0: no wrap).
pub fn x_in_range(x: i64, lo: i64, hi: i64, c: i64) -> bool {
    if c <= 0 {
        return (lo..=hi).contains(&x);
    }
    if hi - lo + 1 >= c {
        return true;
    }
    (x - lo).rem_euclid(c) <= hi - lo
}

/// The exported box (see the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AreaBox {
    /// Canonical after [`AreaBox::normalised`].
    pub x0: i32,
    pub z0: i32,
    pub width: i32,
    pub depth: i32,
    pub y0: i32,
    /// Inclusive.
    pub y1: i32,
}

impl AreaBox {
    /// Parses `x0,z0,x1,z1` (x1, z1 exclusive, as `--area`); y0, y1 are set by the caller.
    pub fn parse_xz(s: &str) -> std::result::Result<(i32, i32, i32, i32), String> {
        let v: Vec<i32> = s
            .split(',')
            .map(|p| p.trim().parse::<i32>().map_err(|e| format!("'{p}': {e}")))
            .collect::<std::result::Result<_, _>>()?;
        if v.len() != 4 || v[2] <= v[0] || v[3] <= v[1] {
            return Err(format!(
                "box '{s}' must be x0,z0,x1,z1 with x1 > x0 and z1 > z0"
            ));
        }
        Ok((v[0], v[1], v[2], v[3]))
    }

    /// Parses `lo,hi` (inclusive).
    pub fn parse_y(s: &str) -> std::result::Result<(i32, i32), String> {
        let v: Vec<i32> = s
            .split(',')
            .map(|p| p.trim().parse::<i32>().map_err(|e| format!("'{p}': {e}")))
            .collect::<std::result::Result<_, _>>()?;
        if v.len() != 2 || v[1] < v[0] {
            return Err(format!("y range '{s}' must be lo,hi with hi ≥ lo"));
        }
        Ok((v[0], v[1]))
    }

    pub fn new(x0: i32, z0: i32, x1: i32, z1: i32, y0: i32, y1: i32) -> AreaBox {
        AreaBox {
            x0,
            z0,
            width: x1 - x0,
            depth: z1 - z0,
            y0,
            y1,
        }
    }

    /// The same box with x0 folded into the canonical span.
    pub fn normalised(self, c: i64) -> AreaBox {
        AreaBox {
            x0: fold(i64::from(self.x0), c) as i32,
            ..self
        }
    }

    pub fn height(&self) -> i32 {
        self.y1 - self.y0 + 1
    }

    pub fn columns(&self) -> usize {
        self.width as usize * self.depth as usize
    }

    /// The world column (folded x) of box column i.
    pub fn world_x(&self, i: i32, c: i64) -> i32 {
        fold(i64::from(self.x0) + i64::from(i), c) as i32
    }

    /// The chunks the box's columns lie in (folded).
    pub fn chunks(&self, c: i64) -> BTreeSet<(i32, i32)> {
        let mut out = BTreeSet::new();
        for i in 0..self.width {
            let cx = self.world_x(i, c) >> 4;
            for cz in (self.z0 >> 4)..=((self.z0 + self.depth - 1) >> 4) {
                out.insert((cx, cz));
            }
        }
        out
    }

    /// The box column of unwrapped-or-folded world x, if the box holds it.
    pub fn column_of(&self, x: i64, c: i64) -> Option<i32> {
        let d = if c > 0 {
            (x - i64::from(self.x0)).rem_euclid(c)
        } else {
            x - i64::from(self.x0)
        };
        (0..i64::from(self.width)).contains(&d).then_some(d as i32)
    }

    /// Whether the box straddles the seam, and the first column past it.
    pub fn seam_column(&self, c: i64) -> Option<i32> {
        if c <= 0 {
            return None;
        }
        let past = c / 2 - i64::from(self.x0);
        (past > 0 && past < i64::from(self.width)).then_some(past as i32)
    }
}

/// A block category, for the viewer's toggles: `air`, `water`, `lava`, `ore`, `plant` (anything
/// [`blocks::is_feature`] names: vegetation, tree parts, snow layers, sheet ice), `soil` (surface
/// material, [`blocks::is_surface_material`]), `crust` (`minecraft:stone` and `deepslate`, the
/// unclassified crust the strata pass paints over) and `rock` (every other block).
pub fn category(name: &str, flags: u16) -> &'static str {
    if flags & blocks::AIR != 0 {
        "air"
    } else if flags & blocks::LAVA != 0 {
        "lava"
    } else if flags & blocks::FLUID != 0 {
        "water"
    } else if flags & blocks::ORE != 0 {
        "ore"
    } else if flags & blocks::FEATURE != 0 {
        "plant"
    } else if flags & blocks::SURFACE != 0 {
        "soil"
    } else if matches!(name, "minecraft:stone" | "minecraft:deepslate") {
        "crust"
    } else {
        "rock"
    }
}

/// What the runner recorded from the server (`probes.jsonl`, one `{"command", "probe", "reply"}`
/// per line as `tools/smoke/run.py` writes them; a bare probe object per line works too).
#[derive(Default, Debug)]
pub struct Probes {
    pub palette: Option<Value>,
    pub claims: Option<Value>,
    /// Every `sites near` answer.
    pub sites_near: Vec<Value>,
    /// `why <id>` answers by site id, with the readable lines of the RCON reply.
    pub why: BTreeMap<u64, (Value, Vec<String>)>,
    /// `probe site x y z` answers.
    pub site_probes: Vec<Value>,
    /// Everything else (`probe tw`, `probe column`, …), kept as recorded.
    pub other: Vec<Value>,
}

impl Probes {
    pub fn read(path: &Path) -> Result<Probes> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        Probes::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Probes> {
        let mut p = Probes::default();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let v: Value = serde_json::from_str(line)?;
            let (probe, reply) = match v.get("probe") {
                Some(Value::Object(_)) => (
                    v["probe"].clone(),
                    v.get("reply").and_then(Value::as_str).unwrap_or(""),
                ),
                Some(Value::String(_)) => (v.clone(), ""),
                _ => continue,
            };
            match probe.get("probe").and_then(Value::as_str).unwrap_or("") {
                "palette" => p.palette = Some(probe),
                "claims" => p.claims = Some(probe),
                "sites-near" => p.sites_near.push(probe),
                "why" => {
                    let id = probe
                        .get("site")
                        .and_then(Value::as_u64)
                        .unwrap_or(u64::MAX);
                    let lines = reply
                        .lines()
                        .map(str::trim_end)
                        .filter(|l| {
                            l.starts_with("Site ")
                                || l.trim_start().starts_with("formed:")
                                || l.trim_start().starts_with("<-")
                        })
                        .map(str::to_owned)
                        .collect();
                    p.why.insert(id, (probe, lines));
                }
                "site" => p.site_probes.push(probe),
                _ => p
                    .other
                    .push(json!({"command": v.get("command"), "probe": probe})),
            }
        }
        Ok(p)
    }

    /// Rock class → its blocks (ids, properties dropped), in the palette's order.
    pub fn class_members(&self) -> BTreeMap<String, Vec<String>> {
        let mut out = BTreeMap::new();
        if let Some(classes) = self
            .palette
            .as_ref()
            .and_then(|p| p.get("classes"))
            .and_then(Value::as_object)
        {
            for (class, states) in classes {
                let ids: Vec<String> = states
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(|s| s.split('[').next().unwrap_or(s).to_owned())
                            .collect()
                    })
                    .unwrap_or_default();
                out.insert(class.clone(), ids);
            }
        }
        out
    }

    /// Claimed ore materials by their path (`c:iron` → `iron`).
    pub fn claimed(&self) -> BTreeSet<String> {
        self.claims
            .as_ref()
            .and_then(|c| c.get("materials"))
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(|m| blocks::path_of(m).to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Everything besides the region files that an export reads.
pub struct Context<'a> {
    /// The world's circumference (0: unknown, no wrap).
    pub circumference: i64,
    pub world_data: Option<&'a WorldData>,
    pub bodies: &'a [Body],
    /// The world data's sites (the fallback when no probes were recorded).
    pub sites: &'a [Site],
    pub events: Option<&'a EventLog>,
    pub probes: &'a Probes,
    pub colors: &'a ColorTable,
    /// Free-form job description the runner passes through (`--meta`).
    pub meta: Value,
}

/// An export in memory: the header and the sections, before encoding.
pub struct Export {
    pub header: Value,
    pub run_counts: Vec<u16>,
    pub run_values: Vec<u16>,
    pub run_lengths: Vec<u16>,
    /// K per column, or empty.
    pub strata_tops: Vec<i16>,
    pub strata_classes: Vec<u8>,
}

/// One body hit in a column: (y, body index into `ctx.bodies`, zone bit mask).
type Hit = (i32, u32, u64);

/// Reads the chunks a box needs.
pub fn load_world(dir: &Path, bx: &AreaBox, c: i64) -> Result<World> {
    let want = bx.chunks(c);
    World::load(dir, |cx, cz| want.contains(&(cx, cz)))
}

/// Body voxels in the box, per box column.
fn body_hits(bx: &AreaBox, ctx: &Context) -> (Vec<Vec<Hit>>, Vec<usize>) {
    let c = ctx.circumference;
    let mut hits: Vec<Vec<Hit>> = vec![Vec::new(); bx.columns()];
    let mut per_body = vec![0usize; ctx.bodies.len()];
    for (bi, b) in ctx.bodies.iter().enumerate() {
        let [lo, hi] = b.aabb;
        let (ya, yb) = (lo[1].max(bx.y0), hi[1].min(bx.y1));
        let (za, zb) = (lo[2].max(bx.z0), hi[2].min(bx.z0 + bx.depth - 1));
        if ya > yb || za > zb {
            continue;
        }
        for i in 0..bx.width {
            let ux = i64::from(bx.x0) + i64::from(i);
            if !x_in_range(ux, i64::from(lo[0]), i64::from(hi[0]), c) {
                continue;
            }
            let wx = bx.world_x(i, c);
            for z in za..=zb {
                let col = (z - bx.z0) as usize * bx.width as usize + i as usize;
                for y in ya..=yb {
                    let Some(at) = b.locate(c.max(1) as u32, wx, y, z) else {
                        continue;
                    };
                    // `zones_at` yields references into `b.zones`: their positions are the bits.
                    let mut mask = 0u64;
                    for z2 in b.zones_at(at) {
                        if let Some(zi) = b.zones.iter().position(|z| std::ptr::eq(z, z2))
                            && zi < 64
                        {
                            mask |= 1 << zi;
                        }
                    }
                    hits[col].push((y, bi as u32, mask));
                    per_body[bi] += 1;
                }
            }
        }
    }
    for h in &mut hits {
        h.sort_unstable();
    }
    (hits, per_body)
}

fn body_json(b: &Body) -> Value {
    json!({
        "id": b.id,
        "site": b.site,
        "role": b.role,
        "shape": b.shape.name(),
        "center": b.center,
        "axes": b.axes,
        "yaw": b.yaw,
        "dip": b.dip,
        "clip_top_y": (b.clip_top_y != deeptime_format::bodies::NO_CLIP).then_some(b.clip_top_y),
        "aabb": b.aabb,
        "rock_class": b.rock_class,
        "zones": b.zones.iter().map(|z| json!({
            "name": z.name, "material": z.material, "rock_class": z.rock_class,
            "d": z.d, "v": z.v, "grade": z.grade, "density": z.density,
        })).collect::<Vec<_>>(),
    })
}

/// A world-data site in the shape of the server's `siteJson` (`SiteCommands`).
fn site_json(s: &Site) -> Value {
    json!({
        "id": s.id,
        "kind": s.kind.name(),
        "type": s.deposit_type,
        "center": s.center,
        "bounds": [s.aabb[0][0], s.aabb[0][1], s.aabb[0][2], s.aabb[1][0], s.aabb[1][1], s.aabb[1][2]],
        "products": s.products.iter().map(|p| json!({
            "material": p.material, "grade": p.grade, "tonnage_t": p.tonnage_t, "role": p.role,
        })).collect::<Vec<_>>(),
        "forming_event": s.forming_event.0,
        "parent": s.parent,
        "coe_vein": s.extraction.coe_vein,
        "cdg_charge_mb": s.extraction.cdg_charge_mb,
        "source": "world-data",
    })
}

/// Whether a site JSON's `bounds` [x0 y0 z0 x1 y1 z1] (inclusive) overlap the box.
fn bounds_overlap(site: &Value, bx: &AreaBox, c: i64) -> bool {
    let Some(b) = site.get("bounds").and_then(Value::as_array) else {
        return false;
    };
    let v: Vec<i64> = b.iter().filter_map(Value::as_i64).collect();
    if v.len() != 6 {
        return false;
    }
    let zs = v[2] <= i64::from(bx.z0 + bx.depth - 1) && v[5] >= i64::from(bx.z0);
    let ys = v[1] <= i64::from(bx.y1) && v[4] >= i64::from(bx.y0);
    let xs = (0..bx.width).any(|i| x_in_range(i64::from(bx.x0) + i64::from(i), v[0], v[3], c));
    zs && ys && xs
}

/// The sites of the export: every site that owns a body in the box or whose bounds overlap it,
/// with its "why here" chain.
fn sites_of(bx: &AreaBox, ctx: &Context, body_sites: &BTreeSet<u64>) -> Vec<Value> {
    let c = ctx.circumference;
    let mut by_id: BTreeMap<u64, Value> = BTreeMap::new();
    for near in &ctx.probes.sites_near {
        for s in near
            .get("sites")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(id) = s.get("id").and_then(Value::as_u64) {
                let mut s = s.clone();
                s["source"] = json!("probe");
                by_id.entry(id).or_insert(s);
            }
        }
    }
    for (id, (why, _)) in &ctx.probes.why {
        if let Some(d) = why.get("detail") {
            let mut d = d.clone();
            d["source"] = json!("probe");
            by_id.entry(*id).or_insert(d);
        }
    }
    if by_id.is_empty() {
        for s in ctx.sites {
            by_id.insert(u64::from(s.id), site_json(s));
        }
    }
    let mut out = Vec::new();
    for (id, mut s) in by_id {
        if !body_sites.contains(&id) && !bounds_overlap(&s, bx, c) {
            continue;
        }
        if let Some((why, lines)) = ctx.probes.why.get(&id) {
            s["why"] = json!({
                "source": "probe",
                "forming_event": why.get("forming_event"),
                "chain": why.get("chain"),
                "error": why.get("error"),
                "lines": lines,
            });
        } else if let Some(log) = ctx.events {
            let fe = s.get("forming_event").and_then(Value::as_u64);
            let formed = fe.and_then(|f| log.get(deeptime_format::events::EventId(f)));
            s["why"] = match formed {
                Some(e) => json!({
                    "source": "world-data",
                    "forming_event": e.to_json(),
                    "chain": log.cause_chain(e.id, 4).iter().map(|c| c.to_json()).collect::<Vec<_>>(),
                }),
                None => {
                    json!({"source": "world-data", "error": "the forming event is not in the event log"})
                }
            };
        }
        out.push(s);
    }
    out
}

/// Builds the export of `bx` (already normalised) from `world`.
pub fn build(world: &World, bx: &AreaBox, ctx: &Context) -> Result<Export> {
    let c = ctx.circumference;
    if bx.width <= 0 || bx.depth <= 0 || bx.height() <= 0 {
        return Err(Error::new("empty box"));
    }
    if bx.height() > i32::from(u16::MAX) {
        return Err(Error::new("y range too tall for u16 runs"));
    }
    let reg: &Registry = &world.reg;

    // Block ids (properties dropped), in first-seen order after air.
    let mut block_names: Vec<String> = vec!["minecraft:air".into()];
    let mut block_of_name: HashMap<String, u32> = HashMap::from([("minecraft:air".into(), 0)]);
    let state_block: Vec<u32> = reg
        .states
        .iter()
        .map(|s| {
            let n = block_names.len() as u32;
            *block_of_name.entry(s.name.clone()).or_insert_with(|| {
                block_names.push(s.name.clone());
                n
            })
        })
        .collect();
    let block_flags: Vec<u16> = block_names.iter().map(|n| blocks::classify(n)).collect();

    let members = ctx.probes.class_members();
    let mut classes_of_block: HashMap<&str, Vec<&str>> = HashMap::new();
    for (class, ids) in &members {
        for id in ids {
            classes_of_block
                .entry(id.as_str())
                .or_default()
                .push(class.as_str());
        }
    }
    let claimed = ctx.probes.claimed();
    // member[class index][block]: the block is one of the class's blocks (strata classes by
    // their index in the world data's list; body classes by name through `members`).
    let wd_classes: &[String] = ctx.world_data.map_or(&[], |wd| wd.classes.as_slice());
    let member: Vec<Vec<bool>> = wd_classes
        .iter()
        .map(|cl| {
            let ids = members.get(cl);
            block_names
                .iter()
                .map(|n| ids.is_some_and(|m| m.iter().any(|b| b == n)))
                .collect()
        })
        .collect();
    let is_member_by_name = |cl: &str, block: u32| {
        members
            .get(cl)
            .is_some_and(|m| m.iter().any(|b| *b == block_names[block as usize]))
    };
    let block_cat: Vec<&'static str> = block_names
        .iter()
        .zip(&block_flags)
        .map(|(n, &f)| category(n, f))
        .collect();

    let (hits, per_body) = body_hits(bx, ctx);

    // Tags: sorted (body index, zone mask) lists.
    let mut tags: Vec<Vec<(u32, u64)>> = vec![Vec::new()];
    let mut tag_of: HashMap<Vec<(u32, u64)>, u32> = HashMap::from([(Vec::new(), 0)]);
    // Entries: (block, tag).
    let mut entries: Vec<(u32, u32)> = vec![(0, 0)];
    let mut entry_of: HashMap<(u32, u32), u16> = HashMap::from([((0, 0), 0)]);

    let (w, d, h) = (bx.width as usize, bx.depth as usize, bx.height() as usize);
    let k = ctx.world_data.map_or(0, WorldData::stack_depth);
    let mut run_counts = Vec::with_capacity(w * d);
    let mut run_values = Vec::new();
    let mut run_lengths = Vec::new();
    let mut strata_tops = vec![i16::MIN; if k > 0 { w * d * k } else { 0 }];
    let mut strata_classes = vec![255u8; strata_tops.len()];

    let mut block_counts = vec![0u64; block_names.len()];
    // Entries of untagged blocks, the common case, without hashing.
    let mut plain_entry = vec![u16::MAX; block_names.len()];
    plain_entry[0] = 0;
    let mut ore_by_material: BTreeMap<String, [u64; 3]> = BTreeMap::new(); // in body, scatter, unclaimed
    let mut ore_per_body = vec![0u64; ctx.bodies.len()];
    let mut strata_check = [0u64; 4]; // checked, painted (member), crust, other
    let mut missing_chunks: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut not_full: BTreeMap<(i32, i32), String> = BTreeMap::new();

    let body_class = |tag: &[(u32, u64)]| -> Option<&str> {
        let mut cls = None;
        for &(bi, mask) in tag {
            let b = &ctx.bodies[bi as usize];
            let mut here = b.rock_class.as_deref();
            for (zi, z) in b.zones.iter().enumerate().take(64) {
                if mask & (1 << zi) != 0 && z.rock_class.is_some() {
                    here = z.rock_class.as_deref();
                }
            }
            if here.is_some() {
                cls = here;
            }
        }
        cls
    };

    let mut column = vec![0u16; h];
    for zk in 0..bx.depth {
        let z = bx.z0 + zk;
        for i in 0..bx.width {
            let col = zk as usize * w + i as usize;
            let wx = bx.world_x(i, c);
            let chunk = world.chunks.get(&(wx >> 4, z >> 4));
            match chunk {
                None => {
                    missing_chunks.insert((wx >> 4, z >> 4));
                }
                Some(ch) if !ch.is_full() => {
                    not_full.insert((ch.cx, ch.cz), ch.status.clone());
                }
                _ => {}
            }
            let strata = ctx
                .world_data
                .map(|wd| wd.strata_column(wx, z))
                .unwrap_or_default();
            for (hh, &(top, cls)) in strata.iter().enumerate().take(k) {
                strata_tops[col * k + hh] =
                    top.clamp(i32::from(i16::MIN) + 1, i32::from(i16::MAX)) as i16;
                strata_classes[col * k + hh] = cls;
            }
            let col_hits = &hits[col];
            let mut hp = 0usize;
            let mut hz = 0usize; // the horizon holding y, walking down from the top
            let (lx, lz) = ((wx & 15) as usize, (z & 15) as usize);
            let mut tag_key: Vec<(u32, u64)> = Vec::new();
            for (yi, slot) in column.iter_mut().enumerate() {
                let y = bx.y0 + yi as i32;
                let sid = chunk.map_or(0, |ch| ch.get(lx, y, lz));
                let block = state_block[sid as usize];
                let flags = block_flags[block as usize];
                block_counts[block as usize] += 1;
                // The body hits at this y (none for air).
                while hp < col_hits.len() && col_hits[hp].0 < y {
                    hp += 1;
                }
                tag_key.clear();
                if flags & blocks::AIR == 0 {
                    let mut q = hp;
                    while q < col_hits.len() && col_hits[q].0 == y {
                        tag_key.push((col_hits[q].1, col_hits[q].2));
                        q += 1;
                    }
                }
                let cat = block_cat[block as usize];
                if cat == "ore" {
                    let m = blocks::ore_material(&block_names[block as usize]).unwrap_or_default();
                    let e = ore_by_material.entry(m.clone()).or_default();
                    if !tag_key.is_empty() {
                        e[0] += 1;
                        for &(bi, _) in &tag_key {
                            ore_per_body[bi as usize] += 1;
                        }
                    } else if claimed.contains(&m) {
                        e[1] += 1;
                    } else {
                        e[2] += 1;
                    }
                } else if (cat == "rock" || cat == "crust")
                    && !members.is_empty()
                    && !strata.is_empty()
                    && y <= strata[0].0
                {
                    // Horizon h fills tops[h+1] < y ≤ tops[h]; y climbs, so h only falls.
                    while hz > 0 && y > strata[hz].0 {
                        hz -= 1;
                    }
                    if hz == 0 {
                        hz = strata.iter().rposition(|&(t, _)| y <= t).unwrap_or(0);
                    }
                    let cls = usize::from(strata[hz].1);
                    let horizon = member.get(cls).is_some_and(|m| m[block as usize]);
                    let body = !tag_key.is_empty()
                        && body_class(&tag_key).is_some_and(|cl| is_member_by_name(cl, block));
                    strata_check[0] += 1;
                    if horizon || body {
                        strata_check[1] += 1;
                    } else if cat == "crust" {
                        strata_check[2] += 1;
                    } else {
                        strata_check[3] += 1;
                    }
                }
                if tag_key.is_empty() && plain_entry[block as usize] != u16::MAX {
                    *slot = plain_entry[block as usize];
                    continue;
                }
                let tag = if tag_key.is_empty() {
                    0
                } else if let Some(&t) = tag_of.get(&tag_key) {
                    t
                } else {
                    let n = tags.len() as u32;
                    tags.push(tag_key.clone());
                    tag_of.insert(tag_key.clone(), n);
                    n
                };
                let n = entries.len();
                let e = *entry_of.entry((block, tag)).or_insert_with(|| {
                    entries.push((block, tag));
                    n as u16
                });
                if entries.len() > usize::from(u16::MAX) {
                    return Err(Error::new(
                        "more than 65,535 (block, tag) entries in the box",
                    ));
                }
                if tag == 0 {
                    plain_entry[block as usize] = e;
                }
                *slot = e;
            }
            // Run-length encode bottom-up.
            let mut runs = 0u16;
            let mut yi = 0;
            while yi < h {
                let v = column[yi];
                let mut len = 1;
                while yi + len < h && column[yi + len] == v {
                    len += 1;
                }
                run_values.push(v);
                run_lengths.push(len as u16);
                runs += 1;
                yi += len;
            }
            run_counts.push(runs);
        }
    }

    // Header tables.
    let blocks_json: Vec<Value> = block_names
        .iter()
        .enumerate()
        .map(|(bi, n)| {
            let f = block_flags[bi];
            json!({
                "id": n,
                "color": hex(ctx.colors.color(n)),
                "cat": block_cat[bi],
                "count": block_counts[bi],
                "ore": blocks::ore_material(n).filter(|_| f & blocks::ORE != 0),
                "classes": classes_of_block.get(n.as_str()).cloned().unwrap_or_default(),
            })
        })
        .collect();
    let tags_json: Vec<Value> = tags
        .iter()
        .map(|t| {
            Value::Array(
                t.iter()
                    .map(|&(bi, mask)| {
                        let b = &ctx.bodies[bi as usize];
                        json!({
                            "body": b.id,
                            "zones": (0..b.zones.len().min(64)).filter(|zi| mask & (1 << zi) != 0).collect::<Vec<_>>(),
                        })
                    })
                    .collect(),
            )
        })
        .collect();
    let used_bodies: Vec<usize> = (0..ctx.bodies.len())
        .filter(|&bi| per_body[bi] > 0)
        .collect();
    let body_sites: BTreeSet<u64> = used_bodies
        .iter()
        .filter_map(|&bi| ctx.bodies[bi].site.map(u64::from))
        .collect();
    let bodies_json: Vec<Value> = used_bodies
        .iter()
        .map(|&bi| {
            let mut v = body_json(&ctx.bodies[bi]);
            v["voxels_in_box"] = json!(per_body[bi]);
            v["ore_blocks_in_box"] = json!(ore_per_body[bi]);
            v
        })
        .collect();
    let sites = sites_of(bx, ctx, &body_sites);

    // `probe site` answers against the tags and the blocks.
    let mut site_checks = Vec::new();
    for p in &ctx.probes.site_probes {
        let (Some(x), Some(y), Some(z)) = (
            p.get("x").and_then(Value::as_i64),
            p.get("y").and_then(Value::as_i64),
            p.get("z").and_then(Value::as_i64),
        ) else {
            continue;
        };
        let Some(i) = bx.column_of(x, c) else {
            continue;
        };
        let (zk, yk) = (z - i64::from(bx.z0), y - i64::from(bx.y0));
        if !(0..i64::from(bx.depth)).contains(&zk) || !(0..i64::from(bx.height())).contains(&yk) {
            continue;
        }
        let col = zk as usize * w + i as usize;
        let first = run_counts[..col]
            .iter()
            .map(|&n| usize::from(n))
            .sum::<usize>();
        let mut acc = 0usize;
        let mut entry = 0u16;
        for ri in first..first + usize::from(run_counts[col]) {
            let len = usize::from(run_lengths[ri]);
            if (yk as usize) < acc + len {
                entry = run_values[ri];
                break;
            }
            acc += len;
        }
        let (block, tag) = entries[usize::from(entry)];
        let ours: BTreeSet<(u64, String)> = tags[tag as usize]
            .iter()
            .flat_map(|&(bi, mask)| {
                let b = &ctx.bodies[bi as usize];
                b.zones
                    .iter()
                    .enumerate()
                    .filter(move |(zi, _)| *zi < 64 && mask & (1 << zi) != 0)
                    .map(move |(_, z)| (u64::from(b.id), z.name.clone()))
                    .chain(std::iter::once((u64::from(b.id), String::new())))
            })
            .collect();
        let theirs: BTreeSet<(u64, String)> = p
            .get("bodies")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .flat_map(|b| {
                let id = b.get("body").and_then(Value::as_u64).unwrap_or(u64::MAX);
                b.get("zones")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(move |z| (id, z.split(" (").next().unwrap_or(z).to_owned()))
                    .chain(std::iter::once((id, String::new())))
                    .collect::<Vec<_>>()
            })
            .collect();
        let server_block = p
            .get("block")
            .and_then(Value::as_str)
            .map(|s| s.split('[').next().unwrap_or(s).to_owned());
        let our_block = block_names[block as usize].clone();
        site_checks.push(json!({
            "at": [x, y, z],
            "bodies_agree": ours == theirs,
            "block_agrees": server_block.as_ref().map(|b| *b == our_block),
            "server_block": server_block,
            "export_block": our_block,
        }));
    }

    let mut cat_counts: BTreeMap<&str, u64> = BTreeMap::new();
    for (b, &n) in block_counts.iter().enumerate() {
        if n > 0 {
            *cat_counts.entry(block_cat[b]).or_default() += n;
        }
    }
    let non_air: u64 = cat_counts
        .iter()
        .filter(|(k, _)| **k != "air")
        .map(|(_, v)| *v)
        .sum();
    let stats = json!({
        "columns": w * d,
        "voxels": (w * d * h) as u64,
        "non_air": non_air,
        "runs": run_values.len(),
        "blocks": block_names.len(),
        "tags": tags.len(),
        "entries": entries.len(),
        "categories": cat_counts,
        "ore": ore_by_material.iter().map(|(m, v)| (m.clone(), json!({
            "in_bodies": v[0], "outside_claimed": v[1], "outside_unclaimed": v[2],
        }))).collect::<serde_json::Map<_, _>>(),
        "strata": if strata_check[0] > 0 { json!({
            "checked": strata_check[0],
            "painted": strata_check[1],
            "crust": strata_check[2],
            "other": strata_check[3],
            "definition": "rock and crust blocks at or below the column's first horizon top: painted = the block is a member of the horizon's class (or of the class a deposit body gives it there), crust = minecraft:stone/deepslate, other = anything else (surface-rule and feature rock)",
        }) } else { Value::Null },
        "missing_chunks": missing_chunks.iter().map(|&(x, z)| [x, z]).collect::<Vec<_>>(),
        "chunks_not_full": not_full.iter().map(|(&(x, z), s)| json!([x, z, s])).collect::<Vec<_>>(),
        "chunk_errors": world.errors.iter().take(20).collect::<Vec<_>>(),
        "probe_site_checks": site_checks,
    });

    let header = json!({
        "format": "deeptime-area",
        "version": VERSION,
        "box": {"x0": bx.x0, "z0": bx.z0, "x1": bx.x0 + bx.width, "z1": bx.z0 + bx.depth},
        "width": bx.width,
        "depth": bx.depth,
        "y0": bx.y0,
        "y1": bx.y1,
        "height": bx.height(),
        "circumference": c,
        "seam": {"straddles": bx.seam_column(c).is_some(), "column": bx.seam_column(c)},
        "column_order": "col = (z - z0) * width + i; runs bottom-up from y0",
        "blocks": blocks_json,
        "tags": tags_json,
        "entries": entries.iter().map(|&(b, t)| [b, t]).collect::<Vec<_>>(),
        "strata": if k > 0 { json!({"k": k, "classes": ctx.world_data.map(|wd| wd.classes.clone())}) } else { Value::Null },
        "bodies": bodies_json,
        "sites": sites,
        "palette": ctx.probes.palette.as_ref().map(|p| json!({
            "classes": p.get("classes"), "source": p.get("source"),
        })),
        "claims": ctx.probes.claims.as_ref().map(|p| json!({
            "active": p.get("active"), "materials": p.get("materials"), "removed": p.get("removed"),
        })),
        "probes": ctx.probes.other,
        "stats": stats,
        "meta": ctx.meta,
    });
    Ok(Export {
        header,
        run_counts,
        run_values,
        run_lengths,
        strata_tops,
        strata_classes,
    })
}

fn pad4(v: &mut Vec<u8>) {
    while !v.len().is_multiple_of(4) {
        v.push(0);
    }
}

/// Encodes an export as the `.dtva` byte layout (not gzipped).
pub fn encode(e: &Export) -> Result<Vec<u8>> {
    let mut blob = Vec::new();
    let mut sections = serde_json::Map::new();
    let mut put = |name: &str, ty: &str, count: usize, bytes: Vec<u8>| {
        pad4(&mut blob);
        sections.insert(
            name.into(),
            json!({"offset": blob.len(), "count": count, "type": ty}),
        );
        blob.extend_from_slice(&bytes);
    };
    let le16 = |v: &[u16]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
    put("run_counts", "u16", e.run_counts.len(), le16(&e.run_counts));
    put("run_values", "u16", e.run_values.len(), le16(&e.run_values));
    put(
        "run_lengths",
        "u16",
        e.run_lengths.len(),
        le16(&e.run_lengths),
    );
    if !e.strata_tops.is_empty() {
        put(
            "strata_tops",
            "i16",
            e.strata_tops.len(),
            e.strata_tops.iter().flat_map(|x| x.to_le_bytes()).collect(),
        );
        put(
            "strata_classes",
            "u8",
            e.strata_classes.len(),
            e.strata_classes.clone(),
        );
    }
    let mut header = e.header.clone();
    header["sections"] = Value::Object(sections);
    let h = serde_json::to_vec(&header)?;
    let mut out = Vec::with_capacity(12 + h.len() + 4 + blob.len());
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(h.len() as u32).to_le_bytes());
    out.extend_from_slice(&h);
    pad4(&mut out);
    out.extend_from_slice(&blob);
    Ok(out)
}

/// Writes the export, gzipped when the path ends in `.gz`. Returns the bytes written.
pub fn write(path: &Path, e: &Export) -> Result<u64> {
    let raw = encode(e)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|er| Error::io(dir, er))?;
    }
    let bytes = if path.extension().is_some_and(|x| x == "gz") {
        let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
        enc.write_all(&raw)?;
        enc.finish()?
    } else {
        raw
    };
    std::fs::write(path, &bytes).map_err(|er| Error::io(path, er))?;
    Ok(bytes.len() as u64)
}

/// A decoded file: the header and dense voxels (entry per block, index
/// `(col · height) + (y − y0)`), for tests and `--verify`.
pub struct Decoded {
    pub header: Value,
    pub width: usize,
    pub depth: usize,
    pub height: usize,
    pub voxels: Vec<u16>,
    pub strata_tops: Vec<i16>,
    pub strata_classes: Vec<u8>,
}

impl Decoded {
    /// The entry at box column i, row zk (z − z0) and y.
    pub fn entry(&self, i: usize, zk: usize, y: i32) -> u16 {
        let y0 = self.header["y0"].as_i64().unwrap_or(0) as i32;
        self.voxels[(zk * self.width + i) * self.height + (y - y0) as usize]
    }

    /// The block id at (i, zk, y).
    pub fn block(&self, i: usize, zk: usize, y: i32) -> &str {
        let e = usize::from(self.entry(i, zk, y));
        let b = self.header["entries"][e][0].as_u64().unwrap_or(0) as usize;
        self.header["blocks"][b]["id"].as_str().unwrap_or("?")
    }

    /// The tag index at (i, zk, y).
    pub fn tag(&self, i: usize, zk: usize, y: i32) -> u64 {
        let e = usize::from(self.entry(i, zk, y));
        self.header["entries"][e][1].as_u64().unwrap_or(0)
    }
}

/// Reads a `.dtva` (gzipped or not) back.
pub fn decode(bytes: &[u8]) -> Result<Decoded> {
    let raw = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut v = Vec::new();
        flate2::read::GzDecoder::new(bytes).read_to_end(&mut v)?;
        v
    } else {
        bytes.to_vec()
    };
    if raw.len() < 12 || raw[..4] != MAGIC {
        return Err(Error::new("not a DTVA file"));
    }
    let u32_at = |o: usize| u32::from_le_bytes([raw[o], raw[o + 1], raw[o + 2], raw[o + 3]]);
    let version = u32_at(4);
    if version != VERSION {
        return Err(Error(format!("DTVA version {version}, expected {VERSION}")));
    }
    let hl = u32_at(8) as usize;
    let header: Value = serde_json::from_slice(&raw[12..12 + hl])?;
    let blob_at = (12 + hl).div_ceil(4) * 4;
    let blob = &raw[blob_at..];
    let section = |name: &str| -> Result<(&[u8], usize)> {
        let s = &header["sections"][name];
        let (Some(off), Some(n), Some(ty)) = (
            s["offset"].as_u64(),
            s["count"].as_u64(),
            s["type"].as_str(),
        ) else {
            return Err(Error(format!("DTVA: no section {name}")));
        };
        let size = match ty {
            "u8" => 1,
            "u16" | "i16" => 2,
            _ => 4,
        };
        let (off, n) = (off as usize, n as usize);
        if off + n * size > blob.len() {
            return Err(Error(format!("DTVA: section {name} runs past the end")));
        }
        Ok((&blob[off..off + n * size], n))
    };
    let u16s = |b: &[u8]| {
        b.as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect::<Vec<_>>()
    };
    let (counts, _) = section("run_counts")?;
    let (values, _) = section("run_values")?;
    let (lengths, _) = section("run_lengths")?;
    let (counts, values, lengths) = (u16s(counts), u16s(values), u16s(lengths));
    let width = header["width"].as_u64().unwrap_or(0) as usize;
    let depth = header["depth"].as_u64().unwrap_or(0) as usize;
    let height = header["height"].as_u64().unwrap_or(0) as usize;
    if counts.len() != width * depth {
        return Err(Error::new("DTVA: run_counts does not match width × depth"));
    }
    let mut voxels = Vec::with_capacity(width * depth * height);
    let mut r = 0usize;
    for &n in &counts {
        let start = voxels.len();
        for _ in 0..n {
            let (Some(&v), Some(&l)) = (values.get(r), lengths.get(r)) else {
                return Err(Error::new("DTVA: fewer runs than run_counts says"));
            };
            voxels.extend(std::iter::repeat_n(v, usize::from(l)));
            r += 1;
        }
        if voxels.len() - start != height {
            return Err(Error(format!(
                "DTVA: a column's runs sum to {} blocks, not {height}",
                voxels.len() - start
            )));
        }
    }
    let (strata_tops, strata_classes) = if header["sections"].get("strata_tops").is_some() {
        let (t, _) = section("strata_tops")?;
        let (cl, _) = section("strata_classes")?;
        (
            t.as_chunks::<2>()
                .0
                .iter()
                .map(|c| i16::from_le_bytes(*c))
                .collect(),
            cl.to_vec(),
        )
    } else {
        (Vec::new(), Vec::new())
    };
    Ok(Decoded {
        header,
        width,
        depth,
        height,
        voxels,
        strata_tops,
        strata_classes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::{Chunk, pack};
    use crate::nbt::Tag;
    use std::collections::HashMap as Map;

    /// A full chunk (cx, cz) with sections y 0 and 1 (blocks 0..32): `block(x, y, z)` names each.
    fn chunk(
        reg: &mut Registry,
        cx: i32,
        cz: i32,
        block: impl Fn(i32, i32, i32) -> &'static str,
    ) -> Chunk {
        let mut sections = Vec::new();
        for sy in 0..2i8 {
            let mut palette: Vec<&str> = Vec::new();
            let mut idx = Vec::with_capacity(4096);
            for i in 0..4096usize {
                let (x, y, z) = ((i & 15) as i32, (i >> 8) as i32, ((i >> 4) & 15) as i32);
                let n = block(cx * 16 + x, i32::from(sy) * 16 + y, cz * 16 + z);
                let p = palette.iter().position(|q| *q == n).unwrap_or_else(|| {
                    palette.push(n);
                    palette.len() - 1
                });
                idx.push(p as u32);
            }
            let bits = crate::chunk::ceil_log2(palette.len()).max(4);
            let mut bs = vec![(
                "palette".to_string(),
                Tag::List(
                    palette
                        .iter()
                        .map(|n| Tag::Compound(vec![("Name".into(), Tag::String((*n).into()))]))
                        .collect(),
                ),
            )];
            if palette.len() > 1 {
                bs.push(("data".into(), Tag::LongArray(pack(&idx, bits))));
            }
            sections.push(Tag::Compound(vec![
                ("Y".into(), Tag::Byte(sy)),
                ("block_states".into(), Tag::Compound(bs)),
            ]));
        }
        let root = Tag::Compound(vec![
            ("xPos".into(), Tag::Int(cx)),
            ("zPos".into(), Tag::Int(cz)),
            ("yPos".into(), Tag::Int(0)),
            ("Status".into(), Tag::String("minecraft:full".into())),
            ("sections".into(), Tag::List(sections)),
        ]);
        Chunk::parse(&root, reg).unwrap()
    }

    fn world(chunks: &[(i32, i32)], block: impl Fn(i32, i32, i32) -> &'static str + Copy) -> World {
        let mut reg = Registry::new();
        let mut map = Map::new();
        for &(cx, cz) in chunks {
            map.insert((cx, cz), chunk(&mut reg, cx, cz, block));
        }
        World {
            dir: std::path::PathBuf::new(),
            reg,
            chunks: map,
            errors: Vec::new(),
            regions: 0,
        }
    }

    fn ctx<'a>(
        c: i64,
        probes: &'a Probes,
        colors: &'a ColorTable,
        bodies: &'a [Body],
    ) -> Context<'a> {
        Context {
            circumference: c,
            world_data: None,
            bodies,
            sites: &[],
            events: None,
            probes,
            colors,
            meta: Value::Null,
        }
    }

    /// Stone below y 10, dirt 10..12, grass at 12, air above; an iron ore at (3, 5, 4) of every
    /// chunk; water at y 13..15 where x is odd.
    fn layered(x: i32, y: i32, z: i32) -> &'static str {
        if (x & 15, y, z & 15) == (3, 5, 4) {
            "minecraft:iron_ore"
        } else if y < 10 {
            "minecraft:stone"
        } else if y < 12 {
            "minecraft:dirt"
        } else if y == 12 {
            "minecraft:grass_block"
        } else if y < 15 && x & 1 == 1 {
            "minecraft:water"
        } else {
            "minecraft:air"
        }
    }

    #[test]
    fn fold_and_ranges() {
        assert_eq!(fold(8191, 16384), 8191);
        assert_eq!(fold(8192, 16384), -8192);
        assert_eq!(fold(-8193, 16384), 8191);
        assert_eq!(fold(20000, 16384), 20000 - 16384);
        assert_eq!(fold(5, 0), 5);
        assert!(x_in_range(8195, 8190, 8200, 16384));
        assert!(x_in_range(-8189, 8190, 8200, 16384)); // −8189 ≡ 8195
        assert!(!x_in_range(-8100, 8190, 8200, 16384));
        assert!(x_in_range(-8189, -8194, -8180, 16384));
    }

    #[test]
    fn round_trip_and_rle() {
        let w = world(&[(0, 0), (1, 0), (0, 1), (1, 1)], layered);
        let bx = AreaBox::new(4, 6, 28, 30, 0, 31); // 24 × 24, 32 high
        let probes = Probes::default();
        let colors = ColorTable::default();
        let e = build(&w, &bx, &ctx(0, &probes, &colors, &[])).unwrap();
        // Every column's runs sum to the height, and each run is maximal.
        let mut r = 0usize;
        for &n in &e.run_counts {
            let (mut sum, mut prev) = (0usize, None);
            for _ in 0..n {
                sum += usize::from(e.run_lengths[r]);
                assert_ne!(prev, Some(e.run_values[r]), "adjacent runs must differ");
                prev = Some(e.run_values[r]);
                r += 1;
            }
            assert_eq!(sum, 32);
        }
        // A plain column: stone 0..10, dirt 10..12, grass 12, air 13..32 = 4 runs; with water 5.
        let col = |i: usize, zk: usize| zk * 24 + i;
        assert_eq!(e.run_counts[col(0, 0)], 4); // x = 4 (even): no water
        assert_eq!(e.run_counts[col(1, 0)], 5); // x = 5 (odd): water 13..15
        let bytes = encode(&e).unwrap();
        let d = decode(&bytes).unwrap();
        assert_eq!((d.width, d.depth, d.height), (24, 24, 32));
        for zk in 0..24 {
            for i in 0..24 {
                for y in 0..32 {
                    let (x, z) = (4 + i as i32, 6 + zk as i32);
                    assert_eq!(d.block(i, zk, y), layered(x, y, z), "at {x} {y} {z}");
                }
            }
        }
        // The ore at (19, 5, 20): its material and category.
        let blocks = d.header["blocks"].as_array().unwrap();
        let ore = blocks
            .iter()
            .find(|b| b["id"] == "minecraft:iron_ore")
            .unwrap();
        assert_eq!(
            (ore["cat"].as_str(), ore["ore"].as_str()),
            (Some("ore"), Some("iron"))
        );
        assert_eq!(d.header["stats"]["ore"]["iron"]["outside_unclaimed"], 1);
        assert_eq!(
            d.header["stats"]["missing_chunks"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        // Gzipped through a file.
        let dir = std::env::temp_dir().join(format!("dt-area-{}", std::process::id()));
        let p = dir.join("a.dtva.gz");
        let n = write(&p, &e).unwrap();
        let back = std::fs::read(&p).unwrap();
        assert_eq!(back.len() as u64, n);
        assert!(back.starts_with(&[0x1f, 0x8b]));
        let d2 = decode(&back).unwrap();
        assert_eq!(d2.voxels, d.voxels);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn long_runs_and_many_entries() {
        // A column taller than a section boundary with one block: one run of the full height.
        let w = world(&[(0, 0)], |_, _, _| "minecraft:stone");
        let bx = AreaBox::new(0, 0, 16, 16, 0, 31);
        let (probes, colors) = (Probes::default(), ColorTable::default());
        let e = build(&w, &bx, &ctx(0, &probes, &colors, &[])).unwrap();
        assert!(e.run_counts.iter().all(|&n| n == 1));
        assert!(e.run_lengths.iter().all(|&n| n == 32));
        // Above the stored sections is air: y 32..40 adds one air run.
        let bx2 = AreaBox::new(0, 0, 16, 16, 0, 39);
        let e2 = build(&w, &bx2, &ctx(0, &probes, &colors, &[])).unwrap();
        assert!(e2.run_counts.iter().all(|&n| n == 2));
        let d = decode(&encode(&e2).unwrap()).unwrap();
        assert_eq!(d.block(3, 3, 35), "minecraft:air");
        assert_eq!(d.block(3, 3, 31), "minecraft:stone");
    }

    /// C = 128 (chunks −4..=3). A box from x 56 to 72 straddles the seam at 64 ≡ −64: columns
    /// 0..8 are world x 56..63 (chunk 3), columns 8..16 world x −64..−57 (chunk −4).
    #[test]
    fn box_across_the_seam() {
        let c = 128i64;
        let block = |x: i32, y: i32, _z: i32| {
            if y > 3 {
                "minecraft:air"
            } else if x >= 0 {
                "minecraft:granite"
            } else {
                "minecraft:diorite"
            }
        };
        let w = world(&[(3, 0), (-4, 0), (2, 0), (-3, 0)], block);
        // Given in the unwrapped frame past the edge, and in the canonical frame: the same box.
        for (x0, x1) in [(56, 72), (56 - 128, 72 - 128), (56 + 128, 72 + 128)] {
            let bx = AreaBox::new(x0, 0, x1, 16, 0, 7).normalised(c);
            assert_eq!(bx.x0, 56);
            assert_eq!(bx.seam_column(c), Some(8));
            assert_eq!(bx.world_x(7, c), 63);
            assert_eq!(bx.world_x(8, c), -64);
            assert_eq!(
                bx.chunks(c)
                    .into_iter()
                    .map(|(x, _)| x)
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([-4, 3])
            );
            let (probes, colors) = (Probes::default(), ColorTable::default());
            let e = build(&w, &bx, &ctx(c, &probes, &colors, &[])).unwrap();
            let d = decode(&encode(&e).unwrap()).unwrap();
            for i in 0..16 {
                let want = if i < 8 {
                    "minecraft:granite"
                } else {
                    "minecraft:diorite"
                };
                assert_eq!(d.block(i, 5, 2), want, "column {i}");
            }
            assert_eq!(d.header["seam"]["column"], 8);
            assert_eq!(d.header["box"]["x1"], 72);
            assert_eq!(
                d.header["stats"]["missing_chunks"]
                    .as_array()
                    .unwrap()
                    .len(),
                0
            );
            assert_eq!(bx.column_of(-60, c), Some(12));
            assert_eq!(bx.column_of(68, c), Some(12));
            assert_eq!(bx.column_of(40, c), None);
        }
    }

    /// A body across the seam tags the voxels on both sides; its zone split shows in the tags.
    #[test]
    fn body_tags_across_the_seam() {
        let c = 128i64;
        let w = world(&[(3, 0), (-4, 0)], |_, y, _| {
            if y < 20 {
                "minecraft:stone"
            } else {
                "minecraft:air"
            }
        });
        let mut b = Body::new(
            7,
            Some(3),
            deeptime_format::Shape::Ellipsoid,
            "ore",
            [63.5, 8.5, 8.5],
            [4.0, 4.0, 4.0],
            0.0,
            0.0,
            deeptime_format::bodies::NO_CLIP,
        );
        b.zones.push(deeptime_format::Zone {
            name: "core".into(),
            material: Some("c:copper".into()),
            rock_class: None,
            d: [0.0, 0.5],
            v: [0.0, 1.0],
            grade: 0.01,
            density: 0.5,
        });
        let bodies = vec![b];
        let bx = AreaBox::new(56, 0, 72, 16, 0, 31).normalised(c);
        let (probes, colors) = (Probes::default(), ColorTable::default());
        let e = build(&w, &bx, &ctx(c, &probes, &colors, &bodies)).unwrap();
        let d = decode(&encode(&e).unwrap()).unwrap();
        // Block centre (63.5, 8.5, 8.5) is the body's centre: column 7, z 8; −64 (column 8) is
        // 1 block east across the seam and inside too.
        let t_core = d.tag(7, 8, 8);
        let t_east = d.tag(8, 8, 8);
        assert_ne!(t_core, 0);
        assert_ne!(t_east, 0);
        assert_eq!(d.header["tags"][t_core as usize][0]["body"], 7);
        assert_eq!(d.header["tags"][t_core as usize][0]["zones"], json!([0]));
        // 3 blocks out: inside the body (d = 0.75), outside the core zone.
        let t_shell = d.tag(10, 8, 8);
        assert_eq!(d.header["tags"][t_shell as usize][0]["zones"], json!([]));
        // Outside: no tag; air in the body carries none either.
        assert_eq!(d.tag(0, 8, 8), 0);
        assert_eq!(d.header["bodies"][0]["id"], 7);
        assert!(d.header["bodies"][0]["voxels_in_box"].as_u64().unwrap() > 200);
    }

    #[test]
    fn probes_parse() {
        let text = r#"{"command": "deeptime probe palette", "probe": {"probe": "palette", "classes": {"granite": ["minecraft:granite"], "loess": ["projectatmosphere:dust[layers=8]"]}}, "reply": ""}
{"command": "deeptime probe claims", "probe": {"probe": "claims", "active": true, "materials": ["c:iron", "c:coal"], "removed": []}, "reply": ""}
{"command": "deeptime why 2", "probe": {"probe": "why", "site": 2, "detail": {"id": 2, "bounds": [0,0,0,1,1,1]}, "chain": []}, "reply": "Site 2: x\n  formed: deposit formation\n  <- subduction onset\nother"}
{"command": "deeptime probe tw", "probe": {"probe": "tw", "stamped": true}, "reply": ""}
{"command": "list", "probe": null, "reply": "x"}"#;
        let p = Probes::parse(text).unwrap();
        assert_eq!(p.class_members()["loess"], vec!["projectatmosphere:dust"]);
        assert_eq!(
            p.claimed(),
            BTreeSet::from(["iron".to_string(), "coal".to_string()])
        );
        assert_eq!(p.why[&2].1.len(), 3);
        assert_eq!(p.other.len(), 1);
    }

    #[test]
    fn categories() {
        let cat = |n: &str| category(n, blocks::classify(n));
        assert_eq!(cat("minecraft:air"), "air");
        assert_eq!(cat("minecraft:water"), "water");
        assert_eq!(cat("minecraft:lava"), "lava");
        assert_eq!(cat("minecraft:deepslate_iron_ore"), "ore");
        assert_eq!(cat("minecraft:oak_leaves"), "plant");
        assert_eq!(cat("minecraft:grass_block"), "soil");
        assert_eq!(cat("minecraft:stone"), "crust");
        assert_eq!(cat("minecraft:deepslate"), "crust");
        assert_eq!(cat("create:asurine"), "rock");
        assert_eq!(cat("minecraft:granite"), "rock");
    }
}
