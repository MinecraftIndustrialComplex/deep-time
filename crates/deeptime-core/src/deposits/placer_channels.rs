// SPDX-License-Identifier: LGPL-3.0-or-later
//! Placers lie in the river channels the bake made (Ben, 2026-10-01: a gold placer is
//! gold-bearing gravel in a channel at river level, not ore set into a hillside).
//!
//! The deposit stage places a placer before the terrain is finished: downstream of its eroded
//! parent on the coarse simulation grid, as one straight flat ribbon at that grid's ground
//! (`spawn_placer`). The game's terrain and rivers are made later (the amplifier, then the bake's
//! water part, P1-22, on the block grid), so that ribbon ends up in the air or buried: on two
//! planets its centre lay a median 0 to 7 blocks from the real ground, but 77 to 121 blocks under
//! it at the 10th percentile and 37 to 66 over it at the 90th, and more than half of the placers
//! placed no ore at all (STATUS.md, build log "deposit audit"). [`settle`] runs after the bake
//! and puts every placer where the finished world has its gravel:
//!
//! **In a channel.** A placer with a baked river within its type's `placer.river_reach_km` moves
//! into the nearest one and becomes **gravel bars along the channel**: the centre line is walked
//! down- and upstream from the nearest vertex (downstream through confluences into the larger
//! river) until the bars hold the ribbon's volume (at most [`MAX_STRETCH`] times its length, at
//! least [`MIN_LENGTH_BLOCKS`]), cut into straight pieces, and each piece becomes one `channel`
//! body of the site (consecutive ids; a site already owns several bodies, as a coal stack does):
//!
//! - along the piece, a little longer than it ([`OVERLAP`]) so neighbouring bars join;
//! - across, the channel's half-width plus [`BANK_BLOCKS`], so the gravel reaches under the banks;
//! - from the water's top block down to [`MIN_GRAVEL_BLOCKS`] (or the ribbon's own thickness, if
//!   more) below the bed at the channel's centre.
//!
//! **On the ground.** A placer with no channel within reach (an old one, far from today's rivers
//! or under today's sea) keeps its place and direction and becomes a **bench**: the ribbon cut
//! into pieces of at most [`MAX_PIECE_BLOCKS`], each laid with its top at the finished ground (the
//! bed under water) at its centre, at least [`MIN_GRAVEL_BLOCKS`] thick.
//!
//! The bodies keep the rock class (gravel) and the zones (the pay streak in the lower half, at the
//! base of the gravel). In the game the strata pass paints the gravel, the water pass cuts the
//! channel through it and lays its own bed block, and the deposit feature turns pay-streak gravel
//! into ore; nothing in the adapter knows about placers. The site record follows (centre, box,
//! first body); its tonnage, grade and events stay. Everything is computed from the site's record
//! and its type ([`ribbon_of`]), not from the bodies in place, so settling again (`deeptime
//! hydro`, which re-makes the channels) gives the same answer.

use std::path::Path;

use deeptime_format::bodies::{BODIES_PATH, NO_CLIP, read_bodies, write_bodies};
use deeptime_format::rivers::{MIN_HALF_WIDTH, Mouth, Rivers};
use deeptime_format::sites::{INDEX_PATH, SITES_PATH, read_sites, write_sites};
use deeptime_format::{Body, ColumnWorld, FileEntry, Manifest, Shape, Site, SiteKind};
use serde_json::{Value, json};

use super::geometry::{self, Scales};
use super::types::DepositType;
use crate::error::CoreError;
use crate::params::PlanetPreset;

/// Gravel under the bed at the channel's centre (and a bench's thickness), blocks, at least.
pub const MIN_GRAVEL_BLOCKS: f64 = 3.0;
/// How far the gravel reaches beyond the channel's edge, blocks.
pub const BANK_BLOCKS: f64 = 2.0;
/// The shortest run of bars, blocks.
pub const MIN_LENGTH_BLOCKS: f64 = 24.0;
/// The bars run at most this many times the length of the ribbon they replace.
pub const MAX_STRETCH: f64 = 4.0;
/// A piece's bar is this many times its length, so neighbours overlap and the gravel is continuous.
pub const OVERLAP: f64 = 1.2;
/// A piece is straight to within this many blocks, and at most [`MAX_PIECE_BLOCKS`] long.
pub const STRAIGHT_BLOCKS: f64 = 0.75;
pub const MAX_PIECE_BLOCKS: f64 = 24.0;
/// Confluences a run of bars follows downstream at most.
pub const MAX_CONFLUENCES: usize = 8;

/// What [`settle`] made.
pub struct Settled {
    pub sites: Vec<Site>,
    pub bodies: Vec<Body>,
    /// `notes.bake.placers`.
    pub note: Value,
}

/// The ribbon the deposit stage lays for a placer site, in blocks: what the bars replace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ribbon {
    /// How far the site looks for a channel.
    pub reach: f64,
    pub volume: f64,
    pub thickness: f64,
    pub length: f64,
    pub width: f64,
}

/// The [`Ribbon`] of a site from its record and its type: the ore body its tonnage makes
/// ([`geometry`]), so the answer is the same before and after the site has been moved.
pub fn ribbon_of(types: &[DepositType], sc: &Scales, s: &Site) -> Option<Ribbon> {
    let t = types.iter().find(|t| t.id == s.deposit_type)?;
    let rules = t.placer.as_ref()?;
    let tonnage = s.products.first().map_or(0.0, |p| p.tonnage_t);
    let semi = geometry::semi_axes_m(
        t.shape(),
        t.geometry.aspect,
        tonnage / t.sizing.ore_density_t_m3,
    );
    let [a, b, c] = geometry::to_blocks(semi, t, sc).map(f64::from);
    Some(Ribbon {
        reach: rules.river_reach_km * 1000.0 / sc.m_per_block_h,
        volume: std::f64::consts::PI * a * c * 2.0 * b,
        thickness: 2.0 * b,
        length: 2.0 * a,
        width: 2.0 * c,
    })
}

/// The first unused body id.
fn next_id(bodies: &[Body]) -> u32 {
    bodies.iter().map(|b| b.id).max().map_or(0, |m| m + 1)
}

/// x folded into the canonical span.
fn fold(c: f64, x: f64) -> f64 {
    (x + c / 2.0).rem_euclid(c) - c / 2.0
}

/// The nearest vertex to (x, z) within `reach` blocks: (vertex index, distance).
fn nearest(r: &Rivers, x: f64, z: f64, reach: f64) -> Option<(usize, f64)> {
    let c = f64::from(r.c);
    let mut best: Option<(usize, f64)> = None;
    for (i, v) in r.vertices.iter().enumerate() {
        let dz = f64::from(v.z) - z;
        if dz.abs() > reach {
            continue;
        }
        let mut dx = f64::from(v.x) - x;
        dx -= (dx / c).round() * c;
        let d = dx.hypot(dz);
        if d <= reach && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((i, d));
        }
    }
    best
}

/// The chain holding vertex `i`.
fn chain_of(r: &Rivers, i: usize) -> Option<usize> {
    r.chains.iter().position(|ch| {
        let (a, n) = (ch.first as usize, ch.count as usize);
        n > 0 && i >= a && i < a + n
    })
}

/// The centre line through vertex `at`, upstream to its chain's head and downstream to the
/// chain's mouth and on through confluences: vertex indices in flow order, and `at`'s position.
fn path(r: &Rivers, at: usize) -> (Vec<usize>, usize) {
    let Some(mut k) = chain_of(r, at) else {
        return (vec![at], 0);
    };
    let first = r.chains[k].first as usize;
    let mut out: Vec<usize> = (first..first + r.chains[k].count as usize).collect();
    let pos = at - first;
    let c = f64::from(r.c);
    for _ in 0..MAX_CONFLUENCES {
        let ch = r.chains[k];
        if ch.mouth != Mouth::River || ch.into as usize >= r.chains.len() {
            break;
        }
        // The last vertex is a vertex of the receiving chain: continue from that one.
        let last = r.vertices[(ch.first + ch.count - 1) as usize];
        let into = r.chains[ch.into as usize];
        let (a, n) = (into.first as usize, into.count as usize);
        let join = (a..a + n).min_by(|&p, &q| {
            let d = |i: usize| {
                let mut dx = f64::from(r.vertices[i].x - last.x);
                dx -= (dx / c).round() * c;
                dx.hypot(f64::from(r.vertices[i].z - last.z))
            };
            d(p).total_cmp(&d(q))
        });
        let Some(join) = join else { break };
        if join + 1 >= a + n {
            break;
        }
        out.extend(join + 1..a + n);
        k = ch.into as usize;
    }
    (out, pos)
}

/// The bars of one placer along its channel: bodies without ids, class or zones.
struct Bars {
    pieces: Vec<Body>,
    length: f64,
}

fn bars(r: &Rivers, site: u32, at: usize, rb: &Ribbon) -> Bars {
    let (line, pos) = path(r, at);
    let v = &r.vertices;
    let c = f64::from(r.c);
    // Positions along the path with x continuous (a path may cross the seam at a confluence).
    let mut xs = Vec::with_capacity(line.len());
    for (k, &i) in line.iter().enumerate() {
        let x = f64::from(v[i].x);
        xs.push(match k {
            0 => x,
            _ => {
                let prev: f64 = xs[k - 1];
                x - ((x - prev) / c).round() * c
            }
        });
    }
    let zs: Vec<f64> = line.iter().map(|&i| f64::from(v[i].z)).collect();
    let seg_len = |a: usize| (xs[a + 1] - xs[a]).hypot(zs[a + 1] - zs[a]);
    let below = rb.thickness.max(MIN_GRAVEL_BLOCKS);
    let half_width = |a: usize| f64::from(v[line[a]].hw).max(MIN_HALF_WIDTH) + BANK_BLOCKS;
    let depth = |a: usize| f64::from(v[line[a]].depth).max(1.0);
    // The gravel's cross-section at a vertex, blocks².
    let section = |a: usize| 2.0 * half_width(a) * (depth(a) + below);
    // Grow [lo, hi] from the vertex, downstream first, until the bars hold the volume.
    let max_len = (MAX_STRETCH * rb.length).max(MIN_LENGTH_BLOCKS);
    let last = line.len() - 1;
    let (mut lo, mut hi) = (pos, pos);
    let (mut len, mut vol) = (0.0, 0.0);
    let mut down = true;
    while (vol < rb.volume || len < MIN_LENGTH_BLOCKS) && len < max_len && (lo > 0 || hi < last) {
        if (down && hi < last) || lo == 0 {
            let l = seg_len(hi);
            vol += l * 0.5 * (section(hi) + section(hi + 1));
            len += l;
            hi += 1;
        } else {
            let l = seg_len(lo - 1);
            vol += l * 0.5 * (section(lo - 1) + section(lo));
            len += l;
            lo -= 1;
        }
        down = !down;
    }
    // Straight pieces.
    let mut pieces = Vec::new();
    let mut a = lo;
    while a < hi {
        let mut b = a + 1;
        while b < hi {
            let n = b + 1;
            let (ex, ez) = (xs[n] - xs[a], zs[n] - zs[a]);
            let l = ex.hypot(ez);
            if l > MAX_PIECE_BLOCKS {
                break;
            }
            let off = (a + 1..n).any(|k| {
                let (px, pz) = (xs[k] - xs[a], zs[k] - zs[a]);
                l > 0.0 && ((px * ez - pz * ex) / l).abs() > STRAIGHT_BLOCKS
            });
            if off {
                break;
            }
            b = n;
        }
        let l = (xs[b] - xs[a]).hypot(zs[b] - zs[a]);
        let w = |k: usize| f64::from(v[line[k]].w);
        let w_max = (a..=b).map(w).fold(f64::MIN, f64::max);
        let w_min = (a..=b).map(w).fold(f64::MAX, f64::min);
        let deepest = (a..=b).map(depth).fold(0.0, f64::max);
        let widest = (a..=b).map(half_width).fold(0.0, f64::max);
        // Blocks bottom ..= top: from `below` under the lowest bed up to the water's top block.
        let top = w_max.ceil() - 1.0;
        let bottom = (w_min - deepest).floor() - below + 1.0;
        let half = ((top + 1.0 - bottom) / 2.0).max(0.5);
        pieces.push(Body::new(
            0,
            Some(site),
            Shape::Channel,
            "ore",
            [
                fold(c, (xs[a] + xs[b]) / 2.0),
                (top + 1.0 + bottom) / 2.0,
                (zs[a] + zs[b]) / 2.0,
            ],
            [
                (OVERLAP * l / 2.0).max(2.0) as f32,
                half as f32,
                widest as f32,
            ],
            (zs[b] - zs[a]).atan2(xs[b] - xs[a]) as f32,
            0.0,
            NO_CLIP,
        ));
        a = b;
    }
    Bars {
        pieces,
        length: len,
    }
}

/// The bench of a placer without a channel: the ribbon at (x, z) along `yaw`, in pieces laid on
/// the finished ground.
fn bench(
    c: f64,
    site: u32,
    (x, z): (f64, f64),
    yaw: f32,
    rb: &Ribbon,
    ground: &dyn Fn(i64, i64) -> i32,
) -> Vec<Body> {
    let n = (rb.length / MAX_PIECE_BLOCKS).ceil().max(1.0);
    let piece = rb.length / n;
    let (sy, cy) = f64::from(yaw).sin_cos();
    let half = (rb.thickness.max(MIN_GRAVEL_BLOCKS) / 2.0).max(0.5);
    (0..n as usize)
        .map(|i| {
            let off = (i as f64 + 0.5) * piece - rb.length / 2.0;
            let (px, pz) = (fold(c, x + off * cy), z + off * sy);
            let top = f64::from(ground(px.round() as i64, pz.round() as i64));
            Body::new(
                0,
                Some(site),
                Shape::Channel,
                "ore",
                [px, top + 1.0 - half, pz],
                [
                    (OVERLAP * piece / 2.0).max(2.0) as f32,
                    half as f32,
                    (rb.width / 2.0).max(0.5) as f32,
                ],
                yaw,
                0.0,
                NO_CLIP,
            )
        })
        .collect()
}

/// Puts the placers of `sites` where the finished world has their gravel: into the channels of
/// `rivers` as bars, or on the ground as benches (module docs). `ribbon(site)` is the site's
/// [`Ribbon`] (None: not a placer type; the site is left alone); `ground(x, z)` the top solid
/// block of a column (the bed under water). Sites and bodies that are not placers come back
/// unchanged, in their order.
pub fn settle(
    sites: &[Site],
    bodies: &[Body],
    rivers: &Rivers,
    ribbon: &dyn Fn(&Site) -> Option<Ribbon>,
    ground: &dyn Fn(i64, i64) -> i32,
) -> Settled {
    let mut out_sites = sites.to_vec();
    let mut out_bodies = bodies.to_vec();
    let mut id = next_id(bodies);
    let c = f64::from(rivers.c);
    let (mut placers, mut moved, mut benches, mut pieces_total) = (0u32, 0u32, 0u32, 0u32);
    let (mut moves, mut lengths, mut lifts) = (Vec::new(), Vec::new(), Vec::new());
    for s in &mut out_sites {
        if s.kind != SiteKind::Placer {
            continue;
        }
        let Some(rb) = ribbon(s) else {
            continue;
        };
        let own: Vec<&Body> = bodies
            .iter()
            .filter(|b| b.site == Some(s.id) && b.role == "ore" && b.shape == Shape::Channel)
            .collect();
        let Some(model) = own.first().copied() else {
            continue;
        };
        placers += 1;
        let (sx, sz) = (f64::from(s.center[0]), f64::from(s.center[2]));
        let (pieces, centre) = match nearest(rivers, sx, sz, rb.reach) {
            Some((at, d)) => {
                let b = bars(rivers, s.id, at, &rb);
                if b.pieces.is_empty() {
                    continue;
                }
                let v = rivers.vertices[at];
                let bed = (f64::from(v.w) - f64::from(v.depth).max(1.0)).floor();
                moved += 1;
                moves.push(d);
                lengths.push(b.length);
                let x = fold(c, f64::from(v.x));
                (b.pieces, [x.round() as i32, bed as i32, v.z.round() as i32])
            }
            None => {
                let top = ground(s.center[0].into(), s.center[2].into());
                benches += 1;
                lifts.push(f64::from(top - s.center[1]));
                (
                    bench(c, s.id, (sx, sz), model.yaw, &rb, ground),
                    [s.center[0], top, s.center[2]],
                )
            }
        };
        let drop: Vec<u32> = own.iter().map(|b| b.id).collect();
        out_bodies.retain(|b| !drop.contains(&b.id));
        let first = id;
        let (mut lo, mut hi) = ([i32::MAX; 3], [i32::MIN; 3]);
        for mut p in pieces {
            p.id = id;
            id += 1;
            p.rock_class = model.rock_class.clone();
            p.zones = model.zones.clone();
            for k in 0..3 {
                lo[k] = lo[k].min(p.aabb[0][k]);
                hi[k] = hi[k].max(p.aabb[1][k]);
            }
            pieces_total += 1;
            out_bodies.push(p);
        }
        s.center = centre;
        s.lat_lon = super::sites::lat_lon_of(c, f64::from(centre[0]), f64::from(centre[2]));
        s.aabb = [lo, hi];
        s.body = Some(first);
    }
    let median = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v.get(v.len() / 2).copied()
    };
    Settled {
        sites: out_sites,
        bodies: out_bodies,
        note: json!({
            "placers": placers,
            "in_channels": moved,
            "benches": benches,
            "pieces": pieces_total,
            "median_move_blocks": median(&mut moves),
            "median_length_blocks": median(&mut lengths),
            "median_bench_lift_blocks": median(&mut lifts),
            "rule": "a placer lies in the nearest baked river channel within its type's placer.river_reach_km as gravel bars along the channel, else on the finished ground as a bench (deposits::placer_channels)",
        }),
    }
}

/// The deposit types and scales of a planet preset, as its deposit stage uses them.
pub fn planet_ribbons(p: &PlanetPreset) -> Result<(Vec<DepositType>, Scales), CoreError> {
    let params = p.deposits.clone().unwrap_or_default();
    let types = super::select_types(&params, p.scale.feature_scale.deposits)?;
    let s = &p.scale;
    let sc = Scales {
        m_per_block_h: s.m_per_block_h,
        m_per_block_v: s.m_per_block_v,
        min_y: s.min_y,
        max_y: s.max_y,
    };
    Ok((types, sc))
}

/// [`settle`] on a baked folder whose manifest is written: reads its sites, bodies, river network
/// and terrain, rewrites the sites and bodies and their manifest entries (the caller writes the
/// manifest), and records `notes.bake.placers`. The run calls it once the folder is complete, and
/// `deeptime hydro` after it re-made the channels. Does nothing (and says so) in a folder without
/// a river network, sites or bodies.
pub fn settle_folder(dir: &Path, p: &PlanetPreset, m: &mut Manifest) -> Result<Value, CoreError> {
    let (Some(rivers), Some((sites, _)), Some(bodies)) = (
        deeptime_format::rivers::read_rivers(dir)?,
        read_sites(dir)?,
        read_bodies(dir)?,
    ) else {
        return Ok(json!({"placers": 0, "why": "no river network, sites or bodies in the folder"}));
    };
    if !sites.iter().any(|s| s.kind == SiteKind::Placer) {
        return Ok(json!({"placers": 0}));
    }
    let (types, sc) = planet_ribbons(p)?;
    let world = ColumnWorld::open(dir)?;
    let ground = |x: i64, z: i64| {
        let h = world.hydro(x, z);
        h.bed.unwrap_or(h.ground)
    };
    let s = settle(
        &sites,
        &bodies,
        &rivers,
        &|site| ribbon_of(&types, &sc, site),
        &ground,
    );
    let (bytes, crc32) = write_bodies(dir, &s.bodies)?;
    let w = write_sites(dir, rivers.c, &s.sites)?;
    for (path, bytes, crc32) in [
        (BODIES_PATH, bytes, crc32),
        (SITES_PATH, w.sites_bytes, w.sites_crc32),
        (INDEX_PATH, w.index_bytes, w.index_crc32),
    ] {
        match m.files.iter_mut().find(|f| f.path == path) {
            Some(f) => {
                f.bytes = bytes;
                f.crc32 = crc32;
            }
            None => m.files.push(FileEntry {
                path: path.into(),
                bytes,
                crc32,
            }),
        }
    }
    if let Some(b) = m.notes.get_mut(crate::bake::NOTES_KEY)
        && b.is_object()
    {
        b["placers"] = s.note.clone();
    }
    if let Some(run) = m.notes.get_mut("run")
        && run.is_object()
    {
        run["bodies"] = json!(s.bodies.len());
    }
    Ok(s.note)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::bodies::Zone;
    use deeptime_format::rivers::{Chain, RiverVertex};
    use deeptime_format::sites::Extraction;

    /// A tributary running east along z = 100 for 80 blocks into a larger river that runs
    /// south-east from (60, 70) through the confluence at (90, 100).
    fn river() -> Rivers {
        let mut r = Rivers::empty(4096);
        for i in 0..41 {
            r.vertices.push(RiverVertex {
                x: 10.0 + 2.0 * i as f32,
                z: 100.0,
                w: 70.0 - 0.05 * i as f32,
                hw: 2.0,
                depth: 1.5,
                q: 20.0,
            });
        }
        for i in 0..40 {
            r.vertices.push(RiverVertex {
                x: 60.0 + 2.0 * i as f32,
                z: 70.0 + 2.0 * i as f32,
                w: 68.5 - 0.02 * i as f32,
                hw: 2.5,
                depth: 2.0,
                q: 45.0,
            });
        }
        r.chains.push(Chain {
            first: 0,
            count: 41,
            mouth: Mouth::River,
            into: 1,
        });
        r.chains.push(Chain {
            first: 41,
            count: 40,
            mouth: Mouth::Sea,
            into: u32::MAX,
        });
        r.build_index();
        r
    }

    fn placer(id: u32, x: f64, z: f64) -> (Site, Body) {
        let mut b = Body::new(
            7,
            Some(id),
            Shape::Channel,
            "ore",
            [x, 118.7, z],
            [39.7, 1.24, 4.96],
            0.3,
            0.0,
            NO_CLIP,
        );
        b.rock_class = Some("gravel".into());
        b.zones = vec![Zone {
            name: "pay_streak".into(),
            material: Some("c:gold".into()),
            rock_class: None,
            d: [0.0, 1.0],
            v: [0.5, 1.0],
            grade: 1e-7,
            density: 0.08,
            raw: 0.0,
        }];
        let s = Site {
            id,
            kind: SiteKind::Placer,
            deposit_type: "deeptime:placer_au".into(),
            center: [x as i32, 118, z as i32],
            lat_lon: [0.0, 0.0],
            aabb: b.aabb,
            body: Some(7),
            products: Vec::new(),
            forming_event: deeptime_format::EventId(1),
            parent: None,
            extraction: Extraction::default(),
        };
        (s, b)
    }

    /// The test placer's ribbon: 100 blocks of reach, the body `placer` makes.
    fn reach(s: &Site) -> Option<Ribbon> {
        (s.deposit_type == "deeptime:placer_au").then_some(Ribbon {
            reach: 100.0,
            volume: std::f64::consts::PI * 39.7 * 4.96 * 2.0 * 1.24,
            thickness: 2.48,
            length: 79.4,
            width: 9.92,
        })
    }

    fn tiny(s: &Site) -> Option<Ribbon> {
        reach(s).map(|r| Ribbon {
            volume: 2.0,
            thickness: 1.0,
            length: 2.0,
            width: 1.0,
            ..r
        })
    }

    /// A hillside rising to the east: the ground is at y 90 + x/4.
    fn hill(x: i64, _z: i64) -> i32 {
        90 + (x / 4) as i32
    }

    /// A placer near a river becomes bars along it: every bar lies on the channel's centre line,
    /// spans from under the bed to the water's top, keeps the gravel class and the pay streak, and
    /// the site's box and first body follow.
    #[test]
    fn a_placer_moves_into_the_channel() {
        let r = river();
        let (s, b) = placer(3, 50.0, 140.0);
        let other = Body::new(
            2,
            None,
            Shape::Ellipsoid,
            "pluton",
            [0.0; 3],
            [5.0; 3],
            0.0,
            0.0,
            NO_CLIP,
        );
        let out = settle(
            std::slice::from_ref(&s),
            &[other.clone(), b.clone()],
            &r,
            &reach,
            &hill,
        );
        assert_eq!(out.note["in_channels"], 1);
        assert_eq!(out.note["benches"], 0);
        assert!(out.bodies.contains(&other), "other bodies stay");
        assert!(
            !out.bodies.iter().any(|x| x.id == 7),
            "the ribbon is replaced"
        );
        let bars: Vec<&Body> = out.bodies.iter().filter(|x| x.site == Some(3)).collect();
        assert!(bars.len() >= 2, "{}", bars.len());
        let site = &out.sites[0];
        assert_eq!(site.body, Some(bars[0].id));
        for (k, bar) in bars.iter().enumerate() {
            assert_eq!(bar.id, bars[0].id + k as u32, "consecutive ids");
            assert_eq!(bar.rock_class.as_deref(), Some("gravel"));
            assert_eq!(bar.zones, b.zones);
            assert_eq!(bar.shape, Shape::Channel);
            // The bar's centre is on a channel.
            let hit = r.sample(bar.center[0].round() as i64, bar.center[2].round() as i64);
            let h = hit
                .hit
                .unwrap_or_else(|| panic!("bar {k} at {:?} is off the channel", bar.center));
            // It spans the water's top block down to three blocks under the bed.
            let (bottom, top) = (bar.aabb[0][1], bar.aabb[1][1]);
            assert!(
                f64::from(top) >= h.w.ceil() - 1.0,
                "top {top}, water {}",
                h.w
            );
            assert!(
                f64::from(bottom) <= (h.w - h.depth).floor() - (MIN_GRAVEL_BLOCKS - 1.0),
                "bottom {bottom} for a bed at {}",
                (h.w - h.depth).floor()
            );
            for k in 0..3 {
                assert!(site.aabb[0][k] <= bar.aabb[0][k] && site.aabb[1][k] >= bar.aabb[1][k]);
            }
        }
        // The block under the bed at the snapped vertex is in the pay streak's half of a bar.
        let [x, y, z] = site.center;
        let inside = bars.iter().find_map(|bar| bar.locate(4096, x, y - 1, z));
        assert!(inside.is_some_and(|l| l.v >= 0.5), "{inside:?}");
        site.validate().unwrap();
        out.bodies.iter().for_each(|b| b.validate().unwrap());
    }

    /// The bars hold about the ribbon's volume, never run longer than MAX_STRETCH times it, and
    /// follow the river through a confluence when the tributary ends.
    #[test]
    fn the_bars_keep_the_volume_and_follow_confluences() {
        let r = river();
        let (s, b) = placer(3, 60.0, 104.0);
        let out = settle(&[s], std::slice::from_ref(&b), &r, &reach, &hill);
        let len = out.note["median_length_blocks"].as_f64().unwrap();
        assert!(
            (MIN_LENGTH_BLOCKS..=MAX_STRETCH * 79.4).contains(&len),
            "{len}"
        );
        // A tiny placer still gets the shortest run.
        let (s2, b2) = placer(4, 60.0, 104.0);
        let out = settle(&[s2], &[b2], &r, &tiny, &hill);
        let len = out.note["median_length_blocks"].as_f64().unwrap();
        assert!(
            (MIN_LENGTH_BLOCKS..MIN_LENGTH_BLOCKS + 4.0).contains(&len),
            "{len}"
        );
        // A big one at the tributary's mouth runs on down the main river (z > 100 there).
        let big = |s: &Site| {
            reach(s).map(|r| Ribbon {
                volume: 20_000.0,
                length: 200.0,
                ..r
            })
        };
        let (s3, b3) = placer(5, 88.0, 101.0);
        let out = settle(&[s3], &[b3], &r, &big, &hill);
        let south = out
            .bodies
            .iter()
            .filter(|b| b.site == Some(5) && b.center[2] > 104.0)
            .count();
        assert!(south >= 1, "no bar below the confluence");
        for b in out.bodies.iter().filter(|b| b.site == Some(5)) {
            let on = r.sample(b.center[0].round() as i64, b.center[2].round() as i64);
            assert!(
                on.hit.is_some(),
                "bar at {:?} is off the channels",
                b.center
            );
        }
    }

    /// Settling what was settled changes nothing but the pieces' ids: the ribbon comes from the
    /// site's record, not from the bodies in place.
    #[test]
    fn settling_twice_is_stable() {
        let r = river();
        for (x, z) in [(50.0, 140.0), (900.0, 900.0)] {
            let (s, b) = placer(3, x, z);
            let once = settle(&[s], std::slice::from_ref(&b), &r, &reach, &hill);
            let twice = settle(&once.sites, &once.bodies, &r, &reach, &hill);
            assert_eq!(once.sites[0].center, twice.sites[0].center);
            assert_eq!(once.sites[0].aabb, twice.sites[0].aabb);
            assert_eq!(once.bodies.len(), twice.bodies.len());
            for (a, b) in once.bodies.iter().zip(&twice.bodies) {
                assert_eq!((a.center, a.axes, a.yaw), (b.center, b.axes, b.yaw));
            }
        }
    }

    /// The shipped placer type gives a median placer (1.1 Mt) a ribbon of a few hundred blocks
    /// and 100 blocks of reach at the default scales.
    #[test]
    fn the_shipped_type_makes_a_ribbon() {
        let types = crate::deposits::types::builtin().unwrap();
        let (mut s, _) = placer(3, 0.0, 0.0);
        s.products = vec![deeptime_format::Product {
            material: "c:gold".into(),
            grade: 2e-7,
            tonnage_t: 1.1e6,
            role: "primary".into(),
        }];
        let rb = ribbon_of(&types, &Scales::default(), &s).unwrap();
        assert!((rb.reach - 100.0).abs() < 1e-9, "{rb:?}");
        assert!(rb.volume > 100.0 && rb.volume < 1000.0, "{rb:?}");
        assert!(rb.length > 20.0 && rb.width > 1.0, "{rb:?}");
        s.deposit_type = "deeptime:coal_seam".into();
        assert_eq!(ribbon_of(&types, &Scales::default(), &s), None);
    }

    /// No channel within reach: the placer stays where it is and becomes a bench on the finished
    /// ground, every piece's top block the ground at its centre, at least three blocks thick.
    #[test]
    fn a_placer_out_of_reach_becomes_a_bench_on_the_ground() {
        let r = river();
        let (s, b) = placer(3, 900.0, 900.0);
        let out = settle(
            std::slice::from_ref(&s),
            std::slice::from_ref(&b),
            &r,
            &reach,
            &hill,
        );
        assert_eq!(out.note["benches"], 1);
        assert_eq!(out.note["in_channels"], 0);
        let site = &out.sites[0];
        assert_eq!((site.center[0], site.center[2]), (900, 900));
        assert_eq!(site.center[1], hill(900, 900));
        let pieces: Vec<&Body> = out.bodies.iter().filter(|x| x.site == Some(3)).collect();
        assert_eq!(pieces.len(), 4, "79.4 blocks in pieces of at most 24");
        for p in &pieces {
            let g = hill(p.center[0].round() as i64, p.center[2].round() as i64);
            let at = |y: i32| {
                p.locate(
                    4096,
                    p.center[0].floor() as i32,
                    y,
                    p.center[2].floor() as i32,
                )
                .is_some()
            };
            assert!(at(g) && !at(g + 1), "the top block is the ground ({g})");
            assert!(at(g - 2) && !at(g - 3), "three blocks thick");
            assert_eq!(p.yaw, b.yaw);
            assert_eq!(p.zones, b.zones);
        }
        // The pieces lie along the ribbon's direction, on a rising hill: their heights differ.
        let ys: Vec<f64> = pieces.iter().map(|p| p.center[1]).collect();
        assert!(ys.iter().any(|y| *y != ys[0]), "{ys:?}");
    }

    /// Another kind of site, or a type without placer rules: unchanged.
    #[test]
    fn other_sites_are_left_alone() {
        let r = river();
        let (mut s2, b2) = placer(3, 60.0, 104.0);
        s2.deposit_type = "x:other".into();
        let out = settle(
            std::slice::from_ref(&s2),
            std::slice::from_ref(&b2),
            &r,
            &reach,
            &hill,
        );
        assert_eq!(out.sites, vec![s2.clone()]);
        assert_eq!(out.bodies, vec![b2.clone()]);
        s2.deposit_type = "deeptime:placer_au".into();
        s2.kind = SiteKind::Deposit;
        let out = settle(
            std::slice::from_ref(&s2),
            std::slice::from_ref(&b2),
            &r,
            &reach,
            &hill,
        );
        assert_eq!(out.bodies, vec![b2]);
        assert_eq!(out.note["placers"], 0);
    }
}
