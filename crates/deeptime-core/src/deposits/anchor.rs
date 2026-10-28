// SPDX-License-Identifier: LGPL-3.0-or-later
//! The playability pass's deposits in the world as generated (DEP-220, DEP-221, ORE-280).
//!
//! The playability pass (`playability`, plan §7.8) places a guaranteed deposit against the deposit
//! stage's ground: the present L1 surface, an exposed one with its top just under the soil
//! (`guaranteed_depth`, ORE-076). Everything after it moves the ground and the body apart. The
//! young landscape and the amplifier redraw the ground; the bake lifts the strata, and the bodies
//! with them, where the final terrain stands above the history's surface (STR-052, STR-053,
//! STR-056); the projection moves each body with the change of the drawn height between its
//! Lambert column and its world one. On master 5dc06974, 20 of 46 guaranteed iron bodies held ore,
//! and with the soil floor 3 of 16: most ended above the world's ground.
//!
//! [`anchor_folder`] runs last, on the finished folder, read as the game reads it
//! (`ColumnWorld`): every guaranteed site's bodies move together, by whole blocks, so the top of
//! its ore lies at the top rock block over its centre (the world's ground, the bed under water,
//! less the [`SOIL_BLOCKS`] of soil on dry land) for an exposed essential, and no higher than it
//! for the others (a body the history buried deeper stays there). Natural deposits keep their
//! physics and are not touched. An in-stratum body (MVT, `hosts.in_stratum`) is left in its host
//! horizon, where the bake and the projection keep it (ORE-271), while its ore lies in rock in
//! its centre column; otherwise (its horizon left in the soil, where the projection's clip under
//! the drawn ground empties it) it is read unclipped, goes into a run of a host class under the
//! top rock block that holds it and is clipped there ([`settle_in_stratum`]). A placer is left
//! where `placer_channels` settled it (in a channel or on the ground).
//!
//! The deposit stage counted each essential's regions on its forecast of the world's λc; the
//! projection chooses its own on the baked land. [`top_up`] then counts the world's own land
//! regions against the sources that hold ore and have a dry centre ([`centre_is_dry`]: a source
//! whose centre lies under the sea is a dive and counts for no region), and gives each one short
//! a copy of a template source, anchored the same way, with an event of its own.
//!
//! The guaranteed sites are the ones whose forming event is a `deeptime:guaranteed_occurrence`
//! (the event log; the run passes them from memory), so the pass needs no flag in the world data.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::time::Instant;

use deeptime_format::bodies::{BODIES_PATH, NO_CLIP, read_bodies, write_bodies};
use deeptime_format::events::{
    Event, EventId, EventIndex, EventLog, EventWriter, GEOLOGY_WRITER, MAX_SEQ, ParamValue, Place,
    decode_segment,
};
use deeptime_format::projection::WorldProjection;
use deeptime_format::sites::{INDEX_PATH, SITES_PATH, read_sites, write_sites_in};
use deeptime_format::{Body, ColumnWorld, FileEntry, Manifest, Site, SiteKind};
use rayon::prelude::*;
use serde_json::{Value, json};

use super::SOIL_BLOCKS;
use super::playability::{EXPOSED_BLOCKS, Essential, Regions};
use crate::error::CoreError;
use crate::params::PlanetPreset;

/// The kind of the playability pass's events.
pub const GUARANTEED_KIND: &str = "deeptime:guaranteed_occurrence";

/// The guaranteed deposits of an event list: forming event → the essential's material.
pub fn guaranteed_of<'a>(
    events: impl IntoIterator<Item = &'a deeptime_format::Event>,
) -> BTreeMap<EventId, String> {
    events
        .into_iter()
        .filter(|e| e.kind == GUARANTEED_KIND)
        .map(|e| {
            let material = match e.params.get("material") {
                Some(ParamValue::Str(s)) => s.clone(),
                _ => String::new(),
            };
            (e.id, material)
        })
        .collect()
}

/// The top rock block of a column: its ground (the bed under water) less the soil on dry land.
pub fn rock_top(hydro: &deeptime_format::WaterColumn) -> i32 {
    if hydro.kind.is_water() {
        hydro.bed.unwrap_or(hydro.ground)
    } else {
        hydro.ground - SOIL_BLOCKS
    }
}

/// Whether a body places ore (a zone with a material).
pub fn bears_ore(b: &Body) -> bool {
    b.zones
        .iter()
        .any(|z| z.material.is_some() && z.density > 0.0)
}

/// The highest block a body can fill: its box's top, or its clip below that.
fn body_top(b: &Body) -> i32 {
    if b.clip_top_y == NO_CLIP {
        b.aabb[1][1]
    } else {
        b.aabb[1][1].min(b.clip_top_y)
    }
}

/// Whether `b` places ore at block (x, y, z): a zone with a material applies at the block's
/// centre (the game draws each such block as ore with the zone's density).
fn places_ore(b: &Body, c: u32, x: i32, y: i32, z: i32) -> bool {
    b.locate(c, x, y, z).is_some_and(|at| {
        b.zones_at(at)
            .any(|zn| zn.material.is_some() && zn.density > 0.0)
    })
}

/// The highest block of column (x, z) where one of `bodies` places ore, or None when none
/// does there. A body's box is padded past its ore (a block or two over a sheet's top), so the
/// box's top is not the ore's.
pub fn ore_top_in_column(bodies: &[&Body], c: u32, x: i32, z: i32) -> Option<i32> {
    bodies
        .iter()
        .filter(|b| bears_ore(b))
        .filter_map(|b| {
            (b.aabb[0][1]..=body_top(b))
                .rev()
                .find(|&y| places_ore(b, c, x, y, z))
        })
        .max()
}

/// How far (whole blocks, up positive) a guaranteed site's bodies move: the top of its ore in
/// the centre column (x, z) to `rock_top` for an exposed essential, else down to it at most.
/// When that column holds none of its ore, the ore-bearing bodies' tops stand in (every body's
/// when none bears ore).
pub fn anchor_shift(bodies: &[&Body], c: u32, x: i32, z: i32, rock_top: i32, exposed: bool) -> i32 {
    let ore: Vec<&&Body> = bodies.iter().filter(|b| bears_ore(b)).collect();
    let top = ore_top_in_column(bodies, c, x, z).or_else(|| {
        if ore.is_empty() {
            bodies.iter().map(|b| body_top(b)).max()
        } else {
            ore.iter().map(|b| body_top(b)).max()
        }
    });
    let Some(top) = top else { return 0 };
    let dy = rock_top - top;
    if exposed { dy } else { dy.min(0) }
}

/// How far below the top rock block a guaranteed in-stratum body looks for its host class when
/// its ore lies above that block (ledger: ORE-280, game-choice: a reach a player can dig to, the
/// coordinator's call; no source read for it): 64 blocks, about 2.2 km at 35 m a block. Past it
/// the body goes to the top rock block in whatever rock is there.
pub const HOST_SEARCH_BLOCKS: i32 = 64;

/// The highest and lowest blocks of column (x, z) where one of `bodies` places ore.
pub fn ore_span_in_column(bodies: &[&Body], c: u32, x: i32, z: i32) -> Option<(i32, i32)> {
    let mut span: Option<(i32, i32)> = None;
    for b in bodies.iter().filter(|b| bears_ore(b)) {
        for y in b.aabb[0][1]..=body_top(b) {
            if places_ore(b, c, x, y, z) {
                span = Some(span.map_or((y, y), |(t, lo)| (t.max(y), lo.min(y))));
            }
        }
    }
    span
}

/// How far (whole blocks) a guaranteed in-stratum body moves. It stays in its host horizon,
/// where the bake and the projection keep it (ORE-271), while the top of its ore in the centre
/// column lies at or under `rock_top` (at it, for an `exposed` essential). Otherwise (a quick
/// planet's guaranteed MVT lay 2 blocks under its ground, in the soil) the ore's top goes to the
/// top of a run of a host class in `rock` (the centre column's rock runs, top first: (top,
/// bottom, class), as the game paints them, other bodies' classes included) at or under
/// `rock_top` and within [`HOST_SEARCH_BLOCKS`] of it ([`EXPOSED_BLOCKS`] when `exposed`): the
/// highest run as thick as the ore's span in the column, else the thickest (ORE-280); and to
/// `rock_top` itself when there is none. Returns the move and whether the body ended in a host
/// class.
#[allow(clippy::too_many_arguments)]
pub fn in_stratum_shift(
    bodies: &[&Body],
    c: u32,
    x: i32,
    z: i32,
    rock_top: i32,
    rock: &[(i32, i32, Option<String>)],
    hosts: &HashSet<String>,
    exposed: bool,
) -> (i32, bool) {
    let dy = anchor_shift(bodies, c, x, z, rock_top, exposed);
    if dy == 0 {
        return (0, true);
    }
    let need = ore_span_in_column(bodies, c, x, z).map_or(1, |(t, lo)| t - lo + 1);
    let reach = if exposed {
        EXPOSED_BLOCKS as i32
    } else {
        HOST_SEARCH_BLOCKS
    };
    let floor = rock_top - reach;
    // The host runs within reach: (top under the rock top, thickness under it).
    let runs: Vec<(i32, i32)> = rock
        .iter()
        .filter(|(_, _, k)| k.as_ref().is_some_and(|k| hosts.contains(k)))
        .filter(|&&(top, bottom, _)| bottom <= rock_top && top >= floor)
        .map(|&(top, bottom, _)| {
            let t = top.min(rock_top);
            (t, t - bottom + 1)
        })
        .collect();
    let host_top = runs
        .iter()
        .filter(|r| r.1 >= need)
        .map(|r| r.0)
        .max()
        .or_else(|| {
            runs.iter()
                .max_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)))
                .map(|r| r.0)
        });
    match host_top {
        Some(t) => (dy - (rock_top - t), true),
        None => (dy, false),
    }
}

/// What [`settle_in_stratum`] did to a guaranteed in-stratum site, with its move (blocks).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InStratum {
    /// Its ore lies in rock in its centre column: left in its horizon.
    Kept,
    /// Moved into a host class.
    ToHost(i32),
    /// No host class within reach: moved into whatever rock is there.
    OutOfHost(i32),
}

/// Settles a guaranteed in-stratum site's `bodies` (changed in place) over centre column
/// (x, z). The projection clips such a body [`SOIL_BLOCKS`] under the world's drawn ground
/// (ORE-271), so one whose horizon the bake left in the soil band comes out of it empty (a quick
/// planet's guaranteed MVT, 2026-10-21: no block at all), and a rigid move would keep it empty.
/// A site whose ore tops out in rock in its centre column (at the top rock block, for an
/// `exposed` essential) is kept as it is. Any other is read unclipped and moved by
/// [`in_stratum_shift`], up as well as down when `exposed`, and every body is then clipped at
/// `rock_top`, so nothing of it lies in the soil.
#[allow(clippy::too_many_arguments)]
pub fn settle_in_stratum(
    bodies: &mut [Body],
    c: u32,
    x: i32,
    z: i32,
    rock_top: i32,
    rock: &[(i32, i32, Option<String>)],
    hosts: &HashSet<String>,
    exposed: bool,
) -> InStratum {
    let refs: Vec<&Body> = bodies.iter().collect();
    if ore_top_in_column(&refs, c, x, z)
        .is_some_and(|t| t <= rock_top && (!exposed || t == rock_top))
    {
        return InStratum::Kept;
    }
    let unclipped: Vec<Body> = bodies
        .iter()
        .map(|b| {
            let mut b = b.clone();
            b.clip_top_y = NO_CLIP;
            b.aabb = b.computed_aabb();
            b
        })
        .collect();
    let refs: Vec<&Body> = unclipped.iter().collect();
    let (dy, in_host) = in_stratum_shift(&refs, c, x, z, rock_top, rock, hosts, exposed);
    for (b, mut u) in bodies.iter_mut().zip(unclipped) {
        shift_body(&mut u, dy);
        // ledger: ORE-271 (no ore in the soil)
        u.clip_top_y = rock_top;
        u.aabb = u.computed_aabb();
        *b = u;
    }
    if in_host {
        InStratum::ToHost(dy)
    } else {
        InStratum::OutOfHost(dy)
    }
}

/// Moves a body up by `dy` whole blocks (centre, clip, box).
fn shift_body(b: &mut Body, dy: i32) {
    if dy == 0 {
        return;
    }
    b.center[1] += f64::from(dy);
    if b.clip_top_y != NO_CLIP {
        b.clip_top_y += dy;
    }
    b.aabb = b.computed_aabb();
}

/// Whether a block of `bodies` that places ore lies in rock in column (x, z): at or under
/// `rock_top`, scanning `reach` blocks down.
fn ore_in_column(bodies: &[&Body], c: u32, x: i32, z: i32, rock_top: i32, reach: i32) -> bool {
    bodies.iter().filter(|b| bears_ore(b)).any(|b| {
        let lo = b.aabb[0][1].max(rock_top - reach);
        (lo..=rock_top.min(body_top(b)))
            .rev()
            .any(|y| places_ore(b, c, x, y, z))
    })
}

/// What [`anchor_sites`] did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Anchored {
    pub guaranteed: usize,
    /// Sites whose bodies moved (an in-stratum one also when only re-clipped), and the largest
    /// move (blocks).
    pub moved: usize,
    pub largest_move: i32,
    pub moved_up: usize,
    pub moved_down: usize,
    /// In-stratum sites left in their host horizon; those moved under the top rock block, into a
    /// host class or (none within reach) into whatever rock is there; placers left where they
    /// settled.
    pub in_stratum_kept: usize,
    pub in_stratum_to_host: usize,
    pub in_stratum_out_of_host: usize,
    pub placers_kept: usize,
    /// Guaranteed sites whose centre column holds ore in rock after the pass, and those whose
    /// centre lies under water.
    pub centre_holds_ore: usize,
    pub under_water: usize,
}

/// Anchors the guaranteed `sites` (forming event in `guaranteed`, material → whether its
/// essential is `exposed`) in `world`, moving their `bodies`; `in_stratum` the in-stratum sites
/// and their type's host classes.
pub fn anchor_sites(
    world: &ColumnWorld,
    sites: &mut [Site],
    bodies: &mut [Body],
    guaranteed: &BTreeMap<EventId, String>,
    exposed: &dyn Fn(&str) -> bool,
    in_stratum: &BTreeMap<u32, HashSet<String>>,
) -> Anchored {
    let mut out = Anchored::default();
    let c = world.c as u32;
    let mut of_site: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (i, b) in bodies.iter().enumerate() {
        if let Some(s) = b.site {
            of_site.entry(s).or_default().push(i);
        }
    }
    for s in sites.iter_mut() {
        let Some(material) = guaranteed.get(&s.forming_event) else {
            continue;
        };
        out.guaranteed += 1;
        let (cx, cz) = (s.center[0], s.center[2]);
        let hydro = world.hydro(i64::from(cx), i64::from(cz));
        let top = rock_top(&hydro);
        out.under_water += usize::from(hydro.kind.is_water());
        let idx = of_site.get(&s.id).cloned().unwrap_or_default();
        if s.kind == SiteKind::Placer {
            out.placers_kept += 1;
        } else {
            // The move, None when the site's bodies are left as they are.
            let moved_by = match in_stratum.get(&s.id) {
                Some(hosts) => {
                    let rock = world.sample(i64::from(cx), i64::from(cz)).rock;
                    let mut own: Vec<Body> = idx.iter().map(|&i| bodies[i].clone()).collect();
                    let r = settle_in_stratum(
                        &mut own,
                        c,
                        cx,
                        cz,
                        top,
                        &rock,
                        hosts,
                        exposed(material),
                    );
                    let dy = match r {
                        InStratum::Kept => None,
                        InStratum::ToHost(dy) | InStratum::OutOfHost(dy) => Some(dy),
                    };
                    match r {
                        InStratum::Kept => out.in_stratum_kept += 1,
                        InStratum::ToHost(_) => out.in_stratum_to_host += 1,
                        InStratum::OutOfHost(_) => out.in_stratum_out_of_host += 1,
                    }
                    if dy.is_some() {
                        for (&i, b) in idx.iter().zip(own) {
                            bodies[i] = b;
                        }
                    }
                    dy
                }
                None => {
                    let mine: Vec<&Body> = idx.iter().map(|&i| &bodies[i]).collect();
                    let dy = anchor_shift(&mine, c, cx, cz, top, exposed(material));
                    if dy != 0 {
                        for &i in &idx {
                            shift_body(&mut bodies[i], dy);
                        }
                    }
                    (dy != 0).then_some(dy)
                }
            };
            if let Some(dy) = moved_by {
                out.moved += 1;
                out.moved_up += usize::from(dy > 0);
                out.moved_down += usize::from(dy < 0);
                out.largest_move = out.largest_move.max(dy.abs());
                // The site's centre is its ore's (the first body's) centre block; its box its
                // bodies'.
                if let Some(first) = s.body.and_then(|id| bodies.iter().find(|b| b.id == id)) {
                    s.center[1] = first.center[1].floor() as i32;
                } else {
                    s.center[1] += dy;
                }
                if idx.is_empty() {
                    s.aabb[0][1] += dy;
                    s.aabb[1][1] += dy;
                } else {
                    s.aabb[0][1] = idx.iter().map(|&i| bodies[i].aabb[0][1]).min().unwrap_or(0);
                    s.aabb[1][1] = idx.iter().map(|&i| bodies[i].aabb[1][1]).max().unwrap_or(0);
                }
            }
        }
        let mine: Vec<&Body> = idx.iter().map(|&i| &bodies[i]).collect();
        out.centre_holds_ore += usize::from(ore_in_column(&mine, c, cx, cz, top, 256));
    }
    out
}

/// What [`settle_weathering`] did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Weathered {
    /// Weathering profiles moved down to the world's top rock block, their bodies, and the
    /// largest move (blocks).
    pub moved: usize,
    pub bodies_moved: usize,
    pub largest_move: i32,
    /// Profiles already at or under it.
    pub kept: usize,
}

// ledger: ORE-285
/// The weathering profiles of the natural sites under the world's own ground. A gossan (a body of
/// role `gossan`: the leached, iron-oxide cap over an exposed sulphide body) and a supergene
/// site's blanket (`SiteKind::Supergene`: it forms at the top of the water table, USGS SIR
/// 2010-5070-B p. 120) are made by today's weathering, at today's surface. The deposit stage
/// placed them against its own ground, and the young landscape and the amplifier ran after it and
/// cut the world's ground lower over many of them: on the Phase 1 run's planet of 2026-10-28 a
/// porphyry's gossan and blanket stood 16 and 17 blocks above the top rock block, in the air.
/// Each profile, a natural site's gossans with its supergene children, moves down by whole blocks
/// so the gossan's clip (the stage's ground) lies at the top rock block over the gossan's centre
/// ([`rock_top`]); a supergene site with no gossan in its family goes by its own clip. The
/// hypogene bodies stay where the history put them, truncated where the ground has come down
/// through them, as erosion truncates them. A profile never moves up: where the world's ground
/// stands higher the bake lifted it with the strata (STR-056) and it lies buried, as an old
/// blanket can. Guaranteed sites, and the families of guaranteed parents, are [`anchor_sites`]'s.
pub fn settle_weathering(
    world: &dyn Columns,
    sites: &mut [Site],
    bodies: &mut [Body],
    guaranteed: &BTreeMap<EventId, String>,
) -> Weathered {
    let mut out = Weathered::default();
    let held = |s: &Site| guaranteed.contains_key(&s.forming_event);
    let by_id: BTreeMap<u32, &Site> = sites.iter().map(|s| (s.id, s)).collect();
    // Each profile under its root: the site itself, or a supergene child's parent.
    let mut profiles: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (i, b) in bodies.iter().enumerate() {
        let Some(site) = b.site.and_then(|id| by_id.get(&id)) else {
            continue;
        };
        let root = match (site.kind, site.parent.and_then(|p| by_id.get(&p))) {
            (SiteKind::Supergene, Some(parent)) => *parent,
            (SiteKind::Supergene, None) => *site,
            _ if b.role == "gossan" => *site,
            _ => continue,
        };
        if held(root) || held(site) {
            continue;
        }
        profiles.entry(root.id).or_default().push(i);
    }
    drop(by_id);
    let mut touched: HashSet<u32> = HashSet::new();
    for idx in profiles.values() {
        // The gossan's clip and centre; else the first clipped body's.
        let lead = idx
            .iter()
            .copied()
            .filter(|&i| bodies[i].clip_top_y != NO_CLIP)
            .min_by_key(|&i| (bodies[i].role != "gossan", bodies[i].id));
        let Some(lead) = lead else {
            continue;
        };
        let b = &bodies[lead];
        let (top, _) = world.rock_top(b.center[0].floor() as i64, b.center[2].floor() as i64);
        let dy = top - b.clip_top_y;
        if dy >= 0 {
            out.kept += 1;
            continue;
        }
        for &i in idx {
            shift_body(&mut bodies[i], dy);
            touched.extend(bodies[i].site);
        }
        out.moved += 1;
        out.bodies_moved += idx.len();
        out.largest_move = out.largest_move.max(-dy);
    }
    // Each site a profile moved: its centre is its first body's centre block, its box its bodies'
    // (as `anchor_sites` keeps them).
    for s in sites.iter_mut().filter(|s| touched.contains(&s.id)) {
        let own: Vec<&Body> = bodies.iter().filter(|b| b.site == Some(s.id)).collect();
        if let Some(first) = s.body.and_then(|id| own.iter().find(|b| b.id == id)) {
            s.center[1] = first.center[1].floor() as i32;
        }
        if let (Some(lo), Some(hi)) = (
            own.iter().map(|b| b.aabb[0][1]).min(),
            own.iter().map(|b| b.aabb[1][1]).max(),
        ) {
            s.aabb[0][1] = lo;
            s.aabb[1][1] = hi;
        }
    }
    out
}

/// The world as the top-up reads it: which columns are dry land, their top rock block and rock.
/// [`ColumnWorld`] is the one the run uses; tests give a small one.
pub trait Columns: Sync {
    /// The x period (blocks) and the z range of the world.
    fn period(&self) -> i64;
    fn z_range(&self) -> (i64, i64);
    /// Whether the terrain at (x, z) stands at or above sea level.
    fn dry(&self, x: i64, z: i64) -> bool;
    /// The top rock block of (x, z) ([`rock_top`]) and whether water covers it.
    fn rock_top(&self, x: i64, z: i64) -> (i32, bool);
    /// The rock runs of (x, z), top first: (top, bottom, class).
    fn rock(&self, x: i64, z: i64) -> Vec<(i32, i32, Option<String>)>;
}

impl Columns for ColumnWorld {
    fn period(&self) -> i64 {
        self.c
    }
    fn z_range(&self) -> (i64, i64) {
        let (lo, hi) = self.manifest.world_projection().z_range();
        (lo.ceil() as i64, hi.floor() as i64)
    }
    fn dry(&self, x: i64, z: i64) -> bool {
        self.height(x, z) >= f64::from(self.manifest.scale.sea_level_y)
    }
    fn rock_top(&self, x: i64, z: i64) -> (i32, bool) {
        let h = self.hydro(x, z);
        (rock_top(&h), h.kind.is_water())
    }
    fn rock(&self, x: i64, z: i64) -> Vec<(i32, i32, Option<String>)> {
        self.sample(x, z).rock
    }
}

/// The spacing (blocks) of the lattice the top-up reads the world's land on and places from
/// (ledger: DEP-220, game-choice: four chunks; a region of 4,096 blocks holds 4,096 points, and
/// the 128-block inland margin is two steps).
pub const TOP_UP_STEP_BLOCKS: i64 = 64;

/// How many inland points the top-up tries for an in-stratum copy before it takes the first one
/// whose column holds no host within reach (game-choice, DEP-220).
pub const TOP_UP_HOST_TRIES: usize = 48;

/// What [`top_up`] found and did for one essential.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TopUp {
    /// The world's land regions of the essential's size, and those that held a source with ore
    /// before the top-up.
    pub land_regions: usize,
    pub with_source: usize,
    /// The sources with ore that counted (a dry centre), the ones with ore left out because
    /// their centre lies under the sea, and the land regions that held enough with those
    /// included but too few without them (each of them got a dry copy).
    pub counted: usize,
    pub sub_sea: usize,
    pub short_by_sea: usize,
    /// Copies placed, of them at an inland point, and land regions still short (no template, or
    /// no dry point clear of the guaranteed bodies).
    pub added: usize,
    pub inland: usize,
    pub short: usize,
}

/// The world's guaranteed sources of a run, for [`top_up`]: the sites, their bodies, which
/// sites' forming events are guaranteed occurrences, and the in-stratum types' host classes.
pub struct Deposits<'a> {
    pub sites: &'a mut Vec<Site>,
    pub bodies: &'a mut Vec<Body>,
    pub guaranteed: &'a mut BTreeMap<EventId, String>,
    /// Deposit type → its host classes, for the in-stratum types.
    pub in_stratum: &'a BTreeMap<String, HashSet<String>>,
}

/// Whether a site holds ore of its own in rock in its centre column as the game draws it: within
/// [`EXPOSED_BLOCKS`] of the top rock block for an `exposed` essential, anywhere under it for
/// another; a placer anywhere in the column (its gravel lies on the ground or in a channel).
fn site_holds_ore(world: &dyn Columns, s: &Site, own: &[&Body], c: u32, exposed: bool) -> bool {
    let (cx, cz) = (s.center[0], s.center[2]);
    if s.kind == SiteKind::Placer {
        return ore_top_in_column(own, c, cx, cz).is_some();
    }
    let (top, _) = world.rock_top(i64::from(cx), i64::from(cz));
    let reach = if exposed {
        EXPOSED_BLOCKS as i32
    } else {
        i32::MAX / 4
    };
    ore_in_column(own, c, cx, cz, top, reach)
}

/// Whether a site's centre column is dry land as the game draws it: the ground there stands at or
/// above sea level ([`Columns::dry`], the test the top-up picks its lattice's land by) with no
/// water over its top rock block ([`Columns::rock_top`]'s flag, the test it picks a copy's point
/// by). A source whose centre is not dry asks the player to dive (Ben's rule of 2026-10-16,
/// decisions log 2026-10-21), so the top-up does not count it for its region.
pub fn centre_is_dry(world: &dyn Columns, s: &Site) -> bool {
    let (x, z) = (i64::from(s.center[0]), i64::from(s.center[2]));
    world.dry(x, z) && !world.rock_top(x, z).1
}

/// The top-up (DEP-220): the deposit stage counted each essential's regions on its forecast of
/// the world, and the projection chooses λc on the baked land, so a region of the world as
/// generated can lack the source the stage placed in its own (and the CLI's `project` runs the
/// stage on the Lambert frame altogether). For every essential, each land region of the world
/// (a point of a [`TOP_UP_STEP_BLOCKS`] lattice on dry land) that holds fewer than
/// `min_sites_per_region` sources with ore and a dry centre ([`Essential::counts`],
/// [`site_holds_ore`], [`centre_is_dry`]: one under the sea counts for no region) gets a copy of
/// a template source: the essential's first guaranteed site with ore, else its smallest natural
/// source with ore (never a placer), a source whose centre is under the sea skipped while another
/// exists. The copy goes to the region's dry lattice point
/// nearest `spawn` (the world's spawn) whose lattice neighbours within 128 blocks are dry land
/// too, any dry point when none is, clear of every guaranteed body; it is anchored as
/// [`anchor_sites`] anchors (an in-stratum copy where its host lies within reach, tried at up to
/// [`TOP_UP_HOST_TRIES`] points). Each copy gets its own `deeptime:guaranteed_occurrence`
/// (`event_id`), at its template's forming time (`times`), caused by the template's event.
/// Returns each essential's record and the new events.
#[allow(clippy::too_many_arguments)]
pub fn top_up(
    world: &dyn Columns,
    wp: &WorldProjection,
    deposits: &mut Deposits<'_>,
    essentials: &[Essential],
    spawn: (f64, f64),
    times: &mut dyn FnMut(EventId) -> (f64, f64),
    event_id: &mut dyn FnMut() -> EventId,
) -> (BTreeMap<String, TopUp>, Vec<Event>) {
    let mut out = BTreeMap::new();
    let mut events = Vec::new();
    if essentials.is_empty() {
        return (out, events);
    }
    let c = world.period();
    let cu = c as u32;
    let (zlo, zhi) = world.z_range();
    let step = TOP_UP_STEP_BLOCKS;
    // The lattice: x from the canonical span's start, z from the world's top edge. A cube
    // world's span starts at its window's west edge, and its points in the void (past the net's
    // faces) are never land (SCL-129).
    let (x0, cube) = match wp {
        WorldProjection::Cube(view) => (wp.index_frame().origin.0, Some(view.cube)),
        _ => (-c / 2, None),
    };
    let nx = (c / step).max(1) as usize;
    let nz = ((zhi - zlo) / step + 1).max(1) as usize;
    let at = |i: usize, j: usize| (x0 + i as i64 * step, zlo + j as i64 * step);
    let dry: Vec<bool> = (0..nx * nz)
        .into_par_iter()
        .map(|k| {
            let (x, z) = at(k % nx, k / nx);
            cube.is_none_or(|cb| cb.face_of_block(x, z).is_some()) && world.dry(x, z)
        })
        .collect();
    // Inland: every lattice point within two steps (128 blocks) is dry land too (x wrapping;
    // past the world's top and bottom edges nothing counts against it).
    let reach = INLAND_TOP_UP_BLOCKS / step;
    let inland: Vec<bool> = (0..nx * nz)
        .map(|k| {
            let (i, j) = ((k % nx) as i64, (k / nx) as i64);
            dry[k]
                && (-reach..=reach).all(|dj| {
                    let jj = j + dj;
                    jj < 0
                        || jj >= nz as i64
                        || (-reach..=reach).all(|di| {
                            dry[jj as usize * nx + (i + di).rem_euclid(nx as i64) as usize]
                        })
                })
        })
        .collect();
    for ess in essentials {
        let regions = Regions::in_frame(wp.index_frame(), ess.region_blocks);
        let mut land = vec![false; regions.count()];
        for (k, &d) in dry.iter().enumerate() {
            if d {
                let (x, z) = at(k % nx, k / nx);
                land[regions.index(x as f64, z as f64)] = true;
            }
        }
        let of_site = |bodies: &[Body]| {
            let mut m: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
            for (i, b) in bodies.iter().enumerate() {
                if let Some(s) = b.site {
                    m.entry(s).or_default().push(i);
                }
            }
            m
        };
        let idx = of_site(deposits.bodies);
        let own = |s: &Site| -> Vec<&Body> {
            idx.get(&s.id)
                .map(|v| v.iter().map(|&i| &deposits.bodies[i]).collect())
                .unwrap_or_default()
        };
        // The sources with ore, by region: those with a dry centre count; one under the sea is
        // kept as a template candidate only.
        let mut have = vec![0u32; regions.count()];
        let mut wet = vec![0u32; regions.count()];
        let mut holding: Vec<usize> = Vec::new();
        let mut dry_holding: Vec<usize> = Vec::new();
        for (k, s) in deposits.sites.iter().enumerate() {
            if ess.counts(&s.products) && site_holds_ore(world, s, &own(s), cu, ess.exposed) {
                let r = regions.index(f64::from(s.center[0]), f64::from(s.center[2]));
                holding.push(k);
                if centre_is_dry(world, s) {
                    have[r] += 1;
                    dry_holding.push(k);
                } else {
                    wet[r] += 1;
                }
            }
        }
        let need_one = ess.min_sites_per_region.max(1);
        let mut rec = TopUp {
            land_regions: land.iter().filter(|&&l| l).count(),
            with_source: (0..regions.count())
                .filter(|&r| land[r] && have[r] >= need_one)
                .count(),
            counted: dry_holding.len(),
            sub_sea: holding.len() - dry_holding.len(),
            short_by_sea: (0..regions.count())
                .filter(|&r| land[r] && have[r] < need_one && have[r] + wet[r] >= need_one)
                .count(),
            ..TopUp::default()
        };
        let short: Vec<(usize, u32)> = (0..regions.count())
            .filter(|&r| land[r] && have[r] < ess.min_sites_per_region)
            .map(|r| (r, ess.min_sites_per_region - have[r]))
            .collect();
        if short.is_empty() {
            out.insert(ess.material.clone(), rec);
            continue;
        }
        // The template: the first guaranteed source of this essential with ore, else the
        // natural source with ore of the smallest footprint.
        let usable = |s: &Site| s.kind != SiteKind::Placer && s.parent.is_none();
        let footprint = |s: &Site| {
            i64::from(s.aabb[1][0] - s.aabb[0][0] + 1) * i64::from(s.aabb[1][2] - s.aabb[0][2] + 1)
        };
        // A template whose centre is under the sea is skipped while another exists.
        let usable_dry = || {
            dry_holding
                .iter()
                .copied()
                .filter(|&k| usable(&deposits.sites[k]))
        };
        let candidates: Vec<usize> = if usable_dry().next().is_some() {
            usable_dry().collect()
        } else {
            holding
                .iter()
                .copied()
                .filter(|&k| usable(&deposits.sites[k]))
                .collect()
        };
        let template = candidates
            .iter()
            .copied()
            .filter(|&k| {
                deposits
                    .guaranteed
                    .get(&deposits.sites[k].forming_event)
                    .is_some_and(|m| *m == ess.material)
            })
            .min_by_key(|&k| deposits.sites[k].id)
            .or_else(|| {
                candidates
                    .iter()
                    .copied()
                    .min_by_key(|&k| (footprint(&deposits.sites[k]), deposits.sites[k].id))
            });
        let Some(tk) = template else {
            rec.short = short.iter().map(|&(_, n)| n as usize).sum();
            out.insert(ess.material.clone(), rec);
            continue;
        };
        let tsite = deposits.sites[tk].clone();
        let tbodies: Vec<Body> = own(&tsite).into_iter().cloned().collect();
        let hosts = deposits.in_stratum.get(&tsite.deposit_type).cloned();
        let (t0, t1) = times(tsite.forming_event);
        let (template_top, _) =
            world.rock_top(i64::from(tsite.center[0]), i64::from(tsite.center[2]));
        // The template's half footprint about its centre, blocks.
        let half = [
            (tsite.aabb[1][0] - tsite.aabb[0][0] + 1) / 2,
            (tsite.aabb[1][2] - tsite.aabb[0][2] + 1) / 2,
        ];
        for (region, mut need) in short {
            // The region's dry points, inland ones first, each nearest the spawn first.
            let mut points: Vec<(bool, f64, i64, i64)> = (0..nx * nz)
                .filter(|&k| dry[k])
                .filter_map(|k| {
                    let (x, z) = at(k % nx, k / nx);
                    (regions.index(x as f64, z as f64) == region).then(|| {
                        let d = super::cluster::distance(c as f64, (x as f64, z as f64), spawn);
                        (!inland[k], d, z, x)
                    })
                })
                .collect();
            points.sort_by(|a, b| {
                a.0.cmp(&b.0)
                    .then(a.1.total_cmp(&b.1))
                    .then(a.2.cmp(&b.2))
                    .then(a.3.cmp(&b.3))
            });
            let mut tries = 0usize;
            let mut fallback: Option<(Vec<Body>, i64, i64, bool)> = None;
            let mut placed: Vec<(Vec<Body>, i64, i64, bool)> = Vec::new();
            for &(coastal, _, z, x) in &points {
                if need == 0 {
                    break;
                }
                let (top, water) = world.rock_top(x, z);
                if water {
                    continue;
                }
                // Clear of every guaranteed body (DEP-222), the copies already placed included.
                let clear = deposits.sites.iter().all(|s| {
                    !deposits.guaranteed.contains_key(&s.forming_event)
                        || !overlaps(c, (x, z), half, s.aabb)
                }) && placed
                    .iter()
                    .all(|(_, px, pz, _)| !overlaps_point(c, (x, z), (*px, *pz), half));
                if !clear {
                    continue;
                }
                let mut copy: Vec<Body> = tbodies
                    .iter()
                    .map(|b| {
                        let mut b = b.clone();
                        let dx = (x - i64::from(tsite.center[0])) as f64;
                        let dz = (z - i64::from(tsite.center[2])) as f64;
                        b.center[0] = super::cluster::wrap_x(c as f64, b.center[0] + dx);
                        b.center[2] += dz;
                        b.aabb = b.computed_aabb();
                        b
                    })
                    .collect();
                let (xi, zi) = (x as i32, z as i32);
                let in_host = match &hosts {
                    Some(h) => {
                        // Read unclipped, its ore's top just over the top rock block, then
                        // settled down into the highest host run that holds it (ORE-280).
                        tries += 1;
                        for b in &mut copy {
                            b.clip_top_y = NO_CLIP;
                            b.aabb = b.computed_aabb();
                        }
                        let refs: Vec<&Body> = copy.iter().collect();
                        let dy = anchor_shift(&refs, cu, xi, zi, top + 1, true);
                        for b in &mut copy {
                            shift_body(b, dy);
                        }
                        let rock = world.rock(x, z);
                        !matches!(
                            settle_in_stratum(&mut copy, cu, xi, zi, top, &rock, h, ess.exposed),
                            InStratum::OutOfHost(_)
                        )
                    }
                    None => {
                        // As deep under the top rock block as the template, then anchored.
                        for b in &mut copy {
                            shift_body(b, top - template_top);
                        }
                        let refs: Vec<&Body> = copy.iter().collect();
                        let dy = anchor_shift(&refs, cu, xi, zi, top, ess.exposed);
                        for b in &mut copy {
                            shift_body(b, dy);
                        }
                        true
                    }
                };
                let refs: Vec<&Body> = copy.iter().collect();
                if !ore_in_column(&refs, cu, xi, zi, top, i32::MAX / 4) {
                    continue;
                }
                if !in_host {
                    if fallback.is_none() {
                        fallback = Some((copy, x, z, !coastal));
                    }
                    if tries >= TOP_UP_HOST_TRIES {
                        break;
                    }
                    continue;
                }
                placed.push((copy, x, z, !coastal));
                need -= 1;
            }
            if need > 0
                && let Some(f) = fallback
            {
                placed.push(f);
                need -= 1;
            }
            rec.short += need as usize;
            for (copy, x, z, inland_point) in placed {
                let site_id = deposits
                    .sites
                    .iter()
                    .map(|s| s.id)
                    .max()
                    .map_or(0, |m| m + 1);
                let next_body = deposits
                    .bodies
                    .iter()
                    .map(|b| b.id)
                    .max()
                    .map_or(0, |m| m + 1);
                let ev = event_id();
                let mut first_body = None;
                let mut lo = [i32::MAX; 3];
                let mut hi = [i32::MIN; 3];
                for (id, mut b) in (next_body..).zip(copy) {
                    b.id = id;
                    b.site = Some(site_id);
                    first_body.get_or_insert((b.id, b.center[1]));
                    for k in 0..3 {
                        lo[k] = lo[k].min(b.aabb[0][k]);
                        hi[k] = hi[k].max(b.aabb[1][k]);
                    }
                    deposits.bodies.push(b);
                }
                let (lat, lon) = wp.lat_lon(x as f64, z as f64);
                let (col, row) = regions.of(x as f64, z as f64);
                let y = first_body.map_or(tsite.center[1], |(_, y)| y.floor() as i32);
                deposits.sites.push(Site {
                    id: site_id,
                    center: [x as i32, y, z as i32],
                    lat_lon: [
                        lat as f32,
                        deeptime_format::projection::wrap_deg(lon) as f32,
                    ],
                    aabb: if first_body.is_some() {
                        [lo, hi]
                    } else {
                        tsite.aabb
                    },
                    body: first_body.map(|(id, _)| id),
                    forming_event: ev,
                    parent: None,
                    ..tsite.clone()
                });
                deposits.guaranteed.insert(ev, ess.material.clone());
                events.push(
                    Event::new(ev, t0, GUARANTEED_KIND, super::SOURCE)
                        .until(t1)
                        .at(Place::Site { site: site_id })
                        .param("deposit_type", tsite.deposit_type.as_str())
                        .param("site", i64::from(site_id))
                        .param("material", ess.material.as_str())
                        .param("region", format!("{col},{row}"))
                        .param("copy_of", i64::from(tsite.id))
                        .param(
                            "reason",
                            "playability minimum in the world as generated (DEP-220)",
                        )
                        .magnitude(0.0)
                        .caused_by(tsite.forming_event),
                );
                rec.added += 1;
                rec.inland += usize::from(inland_point);
            }
        }
        out.insert(ess.material.clone(), rec);
    }
    (out, events)
}

/// How far inland (world blocks) the top-up looks first: the deposit stage's margin
/// (`INLAND_MARGIN_BLOCKS`, DEP-222), read on the world's own lattice.
const INLAND_TOP_UP_BLOCKS: i64 = super::INLAND_MARGIN_BLOCKS as i64;

/// Whether a footprint of half size `half` about (x, z) overlaps box `aabb`'s footprint (x
/// wrapping with period `c`).
fn overlaps(c: i64, (x, z): (i64, i64), half: [i32; 2], aabb: [[i32; 3]; 2]) -> bool {
    let (h0, h2) = (i64::from(half[0]), i64::from(half[1]));
    let (lo_z, hi_z) = (i64::from(aabb[0][2]), i64::from(aabb[1][2]));
    if z + h2 < lo_z || z - h2 > hi_z {
        return false;
    }
    let (lo_x, hi_x) = (i64::from(aabb[0][0]), i64::from(aabb[1][0]));
    let span = hi_x - lo_x;
    if span + 2 * h0 >= c - 1 {
        return true;
    }
    (x + h0 - lo_x).rem_euclid(c) <= span + 2 * h0
}

/// Whether two footprints of half size `half` about `a` and `b` overlap (x wrapping).
fn overlaps_point(c: i64, a: (i64, i64), b: (i64, i64), half: [i32; 2]) -> bool {
    let dx = (a.0 - b.0 + c / 2).rem_euclid(c) - c / 2;
    dx.abs() <= 2 * i64::from(half[0]) && (a.1 - b.1).abs() <= 2 * i64::from(half[1])
}

/// The time span of event `id` in the folder `dir`'s log, read from its one segment (the top-up
/// dates a copy with its template's event); (0, 0) when the log does not hold it.
fn event_times(dir: &Path, id: EventId) -> (f64, f64) {
    let found = || -> Option<(f64, f64)> {
        let index = EventIndex::read(dir).ok()?;
        let seg = index
            .segments
            .iter()
            .find(|s| s.writer == id.writer() && (s.first_seq..=s.last_seq).contains(&id.seq()))?;
        let path = dir.join(&seg.path);
        let bytes = std::fs::read(&path).ok()?;
        let e = decode_segment(&path, &bytes)
            .ok()?
            .events
            .into_iter()
            .find(|e| e.id == id)?;
        Some((e.t_start, e.t_end))
    };
    found().unwrap_or((0.0, 0.0))
}

/// The world's spawn (`spawn/candidates.json`, the first candidate), (0, 0) without one.
fn spawn_of(dir: &Path) -> (f64, f64) {
    std::fs::read_to_string(dir.join("spawn/candidates.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| {
            let p = v["candidates"].get(0)?.clone();
            Some((p["x"].as_f64()?, p["z"].as_f64()?))
        })
        .unwrap_or((0.0, 0.0))
}

/// Records `path` (its size and CRC32) in the manifest's file list.
fn set_file(m: &mut Manifest, path: &str, bytes: u64, crc32: u32) {
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

/// Anchors the guaranteed deposits of the finished world folder `dir` (its manifest `m`, written
/// to disk) in the world as generated, and tops up the essentials' regions of that world
/// ([`top_up`]); `guaranteed` the run's guaranteed forming events (read from the folder's event
/// log when None). Rewrites the event log (a new segment, when it added a copy), the bodies and
/// the sites when anything changed, and records them in `m`; the caller writes the manifest
/// last. Returns the pass's record (`notes.bake.guaranteed`).
pub fn anchor_folder(
    dir: &Path,
    p: &PlanetPreset,
    m: &mut Manifest,
    guaranteed: Option<&BTreeMap<EventId, String>>,
) -> Result<Value, CoreError> {
    let started = Instant::now();
    let Some((mut sites, _)) = read_sites(dir)? else {
        return Ok(json!({"guaranteed": 0, "why": "no sites"}));
    };
    let mut guaranteed: BTreeMap<EventId, String> = match guaranteed {
        Some(g) => g.clone(),
        None => match EventLog::load(dir) {
            Ok(log) => guaranteed_of(log.of_kind(GUARANTEED_KIND)),
            Err(_) => BTreeMap::new(),
        },
    };
    let essentials = p
        .deposits
        .as_ref()
        .map(|d| d.essentials.clone())
        .unwrap_or_default();
    let Some(mut bodies) = read_bodies(dir)? else {
        return Ok(json!({"guaranteed": 0, "why": "no bodies"}));
    };
    let world = ColumnWorld::open(dir)?;
    // The natural sites' weathering profiles under the world's ground (ORE-285).
    let w = settle_weathering(&world, &mut sites, &mut bodies, &guaranteed);
    let weathering = json!({
        "moved": w.moved,
        "bodies_moved": w.bodies_moved,
        "largest_move_blocks": w.largest_move,
        "kept": w.kept,
    });
    if essentials.is_empty()
        && !sites
            .iter()
            .any(|s| guaranteed.contains_key(&s.forming_event))
    {
        if w.moved > 0 {
            let (bytes, crc32) = write_bodies(dir, &bodies)?;
            set_file(m, BODIES_PATH, bytes, crc32);
            let ws = write_sites_in(
                dir,
                m.scale.circumference_blocks,
                m.world_projection().index_frame(),
                &sites,
            )?;
            set_file(m, SITES_PATH, ws.sites_bytes, ws.sites_crc32);
            set_file(m, INDEX_PATH, ws.index_bytes, ws.index_crc32);
        }
        return Ok(json!({"guaranteed": 0, "weathering": weathering}));
    }
    let types = super::types::builtin()?;
    // The in-stratum types' host classes (weight above 0), and the guaranteed in-stratum sites'.
    let type_hosts: BTreeMap<String, HashSet<String>> = types
        .iter()
        .filter(|t| t.hosts.in_stratum)
        .map(|t| {
            let hosts = t.hosts.weights().unwrap_or_default();
            (
                t.id.clone(),
                hosts
                    .into_iter()
                    .filter(|&(_, w)| w > 0.0)
                    .map(|(k, _)| k)
                    .collect(),
            )
        })
        .collect();
    let in_stratum: BTreeMap<u32, HashSet<String>> = sites
        .iter()
        .filter(|s| guaranteed.contains_key(&s.forming_event))
        .filter_map(|s| Some((s.id, type_hosts.get(&s.deposit_type)?.clone())))
        .collect();
    let exposed = |material: &str| {
        essentials
            .iter()
            .any(|e| e.material == material && e.exposed)
    };
    let a = anchor_sites(
        &world,
        &mut sites,
        &mut bodies,
        &guaranteed,
        &exposed,
        &in_stratum,
    );
    let anchor_ms = started.elapsed().as_secs_f64() * 1000.0;
    // The top-up, in the world's own regions (DEP-220).
    let t = Instant::now();
    let wp = m.world_projection();
    let mut writer: Option<EventWriter> = None;
    let mut open_error: Option<CoreError> = None;
    let (topped, events) = {
        let mut deposits = Deposits {
            sites: &mut sites,
            bodies: &mut bodies,
            guaranteed: &mut guaranteed,
            in_stratum: &type_hosts,
        };
        let mut times = |id: EventId| event_times(dir, id);
        let mut next_id = || -> EventId {
            if writer.is_none() {
                match EventWriter::open(dir, GEOLOGY_WRITER, crate::session::GEOLOGY_WRITER_NAME) {
                    Ok(w) => writer = Some(w),
                    Err(e) => {
                        open_error.get_or_insert(e.into());
                        return EventId::new(GEOLOGY_WRITER, MAX_SEQ);
                    }
                }
            }
            writer
                .as_mut()
                .map_or(EventId::new(GEOLOGY_WRITER, MAX_SEQ), |w| w.next_id())
        };
        top_up(
            &world,
            &wp,
            &mut deposits,
            &essentials,
            spawn_of(dir),
            &mut times,
            &mut next_id,
        )
    };
    if let Some(e) = open_error {
        return Err(e);
    }
    let added: usize = topped.values().map(|r| r.added).sum();
    // The new events first, then the bodies and sites that name them; the manifest (the
    // caller's) last.
    if let Some(w) = writer.as_mut()
        && let Some(seg) = w.append(&events)?
    {
        set_file(m, &seg.path, seg.bytes, seg.crc32);
        let index = FileEntry::of(dir, deeptime_format::events::INDEX_PATH)?;
        set_file(m, &index.path, index.bytes, index.crc32);
    }
    if a.moved > 0 || added > 0 || w.moved > 0 {
        let (bytes, crc32) = write_bodies(dir, &bodies)?;
        set_file(m, BODIES_PATH, bytes, crc32);
        let w = write_sites_in(dir, m.scale.circumference_blocks, wp.index_frame(), &sites)?;
        set_file(m, SITES_PATH, w.sites_bytes, w.sites_crc32);
        set_file(m, INDEX_PATH, w.index_bytes, w.index_crc32);
    }
    let regions: serde_json::Map<String, Value> = topped
        .iter()
        .map(|(k, r)| {
            (
                k.clone(),
                json!({
                    "land_regions": r.land_regions,
                    "with_source": r.with_source,
                    "counted": r.counted,
                    "sub_sea": r.sub_sea,
                    "short_by_sea": r.short_by_sea,
                    "added": r.added,
                    "added_inland": r.inland,
                    "short": r.short,
                }),
            )
        })
        .collect();
    Ok(json!({
        "weathering": weathering,
        "guaranteed": a.guaranteed,
        "moved": a.moved,
        "moved_up": a.moved_up,
        "moved_down": a.moved_down,
        "largest_move_blocks": a.largest_move,
        "in_stratum_kept": a.in_stratum_kept,
        "in_stratum_to_host": a.in_stratum_to_host,
        "in_stratum_out_of_host": a.in_stratum_out_of_host,
        "placers_kept": a.placers_kept,
        "centre_holds_ore": a.centre_holds_ore,
        "under_water": a.under_water,
        "topped_up": added,
        "top_up": regions,
        "millis": anchor_ms,
        "top_up_millis": t.elapsed().as_secs_f64() * 1000.0,
    }))
}

#[cfg(test)]
mod tests {
    use deeptime_format::Zone;
    use deeptime_format::bodies::Shape;

    use super::*;

    fn body(id: u32, y: f64, half: f32, ore: bool, clip: i32) -> Body {
        let mut b = Body::new(
            id,
            Some(1),
            Shape::Sheet,
            if ore { "ore" } else { "halo" },
            [0.5, y, 0.5],
            [12.0, half, 12.0],
            0.0,
            0.0,
            clip,
        );
        b.zones.push(Zone {
            name: "core".into(),
            material: ore.then(|| "c:iron".to_owned()),
            rock_class: None,
            d: [0.0, 1.0],
            v: [0.0, 1.0],
            grade: 0.3,
            density: if ore { 0.5 } else { 0.0 },
            raw: 0.0,
        });
        b
    }

    const C: u32 = 4096;

    /// An exposed essential's ore tops out at the top rock block, from above or below; another
    /// only moves down, so a body the history buried deeper stays there. The top is the ore's
    /// highest block in the centre column, not its padded box's nor a barren halo's, and a clip
    /// lowers it.
    #[test]
    fn a_guaranteed_body_moves_its_ore_top_to_the_top_rock_block() {
        let ore = body(1, 80.0, 3.0, true, NO_CLIP);
        let halo = body(2, 90.0, 3.0, false, NO_CLIP);
        let shift =
            |bs: &[&Body], rock: i32, exposed: bool| anchor_shift(bs, C, 0, 0, rock, exposed);
        // The ore's top block: its centre (y + 0.5) within 3 blocks of 80, under its box's 83.
        let top = ore_top_in_column(&[&ore, &halo], C, 0, 0).unwrap();
        assert_eq!(top, 82);
        assert_eq!(body_top(&ore), 83);
        // Above the rock: down for either kind.
        assert_eq!(shift(&[&ore, &halo], 60, true), 60 - top);
        assert_eq!(shift(&[&ore, &halo], 60, false), 60 - top);
        // Below it: up only when exposed.
        assert_eq!(shift(&[&ore], top + 9, true), 9);
        assert_eq!(shift(&[&ore], top + 9, false), 0);
        // A barren body alone goes by its box's top; a clip lowers the ore's top.
        assert_eq!(shift(&[&halo], 60, true), 60 - body_top(&halo));
        let clipped = body(3, 80.0, 3.0, true, 78);
        assert_eq!(ore_top_in_column(&[&clipped], C, 0, 0), Some(78));
        assert_eq!(shift(&[&clipped], 70, true), 70 - 78);
        // A centre column the ore misses: the box's top stands in.
        assert_eq!(ore_top_in_column(&[&ore], C, 40, 0), None);
        assert_eq!(
            anchor_shift(&[&ore], C, 40, 0, 60, true),
            60 - body_top(&ore)
        );
        assert_eq!(shift(&[], 70, true), 0);
    }

    /// An in-stratum body stays where its ore is in rock; above the top rock block it goes down
    /// into the nearest host class under that block within reach, and to the block itself when
    /// there is none (a host run in the soil does not count).
    #[test]
    fn an_in_stratum_body_goes_down_into_its_host() {
        let ore = body(1, 80.0, 3.0, true, NO_CLIP);
        let top = ore_top_in_column(&[&ore], C, 0, 0).unwrap();
        let hosts: HashSet<String> = ["limestone", "dolostone"].map(String::from).into();
        let run = |t: i32, b: i32, k: &str| (t, b, Some(k.to_owned()));
        let shift = |rock: &[(i32, i32, Option<String>)], rock_top: i32| {
            in_stratum_shift(&[&ore], C, 0, 0, rock_top, rock, &hosts, false)
        };
        let column = [
            run(74, 71, "sandstone"),
            run(70, 66, "shale"),
            run(65, 50, "limestone"),
            run(49, 0, "sandstone"),
        ];
        assert_eq!(shift(&column, 70), (65 - top, true));
        // A horizon across the top rock block: to that block.
        assert_eq!(shift(&[run(72, 60, "dolostone")], 70), (70 - top, true));
        // Its ore already in rock: kept.
        assert_eq!(shift(&column, top + 3), (0, true));
        // Host only in the soil band, or deeper than the reach: the top rock block, out of host.
        assert_eq!(shift(&[run(74, 71, "limestone")], 70), (70 - top, false));
        let deep = 70 - HOST_SEARCH_BLOCKS - 1;
        assert_eq!(
            shift(&[run(70, deep + 1, "shale"), run(deep, 0, "limestone")], 70),
            (70 - top, false)
        );
        // The moved ore tops out at the host's top block.
        let mut b = ore.clone();
        shift_body(&mut b, shift(&column, 70).0);
        assert_eq!(ore_top_in_column(&[&b], C, 0, 0), Some(65));
    }

    /// The host run an in-stratum body goes into is the highest as thick as the ore's span in its
    /// centre column, else the thickest within reach (ORE-280); an exposed essential's buried body
    /// comes up to its host's top at the top rock block, another stays where it lies.
    #[test]
    fn an_in_stratum_body_takes_a_host_that_holds_it() {
        let hosts: HashSet<String> = ["limestone"].map(String::from).into();
        let run = |t: i32, b: i32, k: &str| (t, b, Some(k.to_owned()));
        let ore = body(1, 80.0, 3.0, true, NO_CLIP);
        let (top, lo) = ore_span_in_column(&[&ore], C, 0, 0).unwrap();
        assert_eq!((top, lo), (82, 77));
        let shift = |b: &Body, rock: &[(i32, i32, Option<String>)], exposed: bool| {
            in_stratum_shift(&[b], C, 0, 0, 70, rock, &hosts, exposed)
        };
        // A run too thin above a thick one: the thick one.
        let column = [
            run(70, 66, "limestone"),
            run(65, 56, "shale"),
            run(55, 20, "limestone"),
        ];
        assert_eq!(shift(&ore, &column, false), (55 - top, true));
        // Only thin runs: the thickest.
        let thin = [
            run(70, 69, "limestone"),
            run(68, 61, "shale"),
            run(60, 57, "limestone"),
        ];
        assert_eq!(shift(&ore, &thin, false), (60 - top, true));
        // Buried under the top rock block, in its host.
        let deep = body(1, 40.0, 3.0, true, NO_CLIP);
        let dtop = ore_top_in_column(&[&deep], C, 0, 0).unwrap();
        let lime = [run(70, 30, "limestone")];
        assert_eq!(shift(&deep, &lime, true), (70 - dtop, true));
        assert_eq!(shift(&deep, &lime, false), (0, true));
        let mut bs = vec![deep.clone()];
        let r = settle_in_stratum(&mut bs, C, 0, 0, 70, &lime, &hosts, true);
        assert_eq!(r, InStratum::ToHost(70 - dtop));
        assert_eq!(ore_top_in_column(&[&bs[0]], C, 0, 0), Some(70));
        let mut bs = vec![deep.clone()];
        let r = settle_in_stratum(&mut bs, C, 0, 0, 70, &lime, &hosts, false);
        assert_eq!((r, &bs[0]), (InStratum::Kept, &deep));
    }

    /// The projection clips an in-stratum body under the drawn ground (ORE-271), which empties
    /// one whose horizon lies in the soil band. Settled, it is read unclipped, moved into its host
    /// under the top rock block and clipped there, and holds ore; one whose ore is in rock is
    /// kept as it is.
    #[test]
    fn an_emptied_in_stratum_body_is_settled_into_rock() {
        let hosts: HashSet<String> = ["limestone"].map(String::from).into();
        let open = body(1, 72.0, 1.0, true, NO_CLIP);
        let top = ore_top_in_column(&[&open], C, 0, 0).unwrap();
        assert!(top > 70, "{top}");
        let emptied = body(1, 72.0, 1.0, true, 70);
        assert_eq!(ore_top_in_column(&[&emptied], C, 0, 0), None);
        let settle = |b: &Body, rock: &[(i32, i32, Option<String>)]| {
            let mut bs = vec![b.clone()];
            let r = settle_in_stratum(&mut bs, C, 0, 0, 70, rock, &hosts, false);
            (r, bs.remove(0))
        };
        let lime = |t: i32, b: i32| (t, b, Some("limestone".to_owned()));
        let sand = |t: i32, b: i32| (t, b, Some("sandstone".to_owned()));
        let (r, b) = settle(&emptied, &[lime(74, 40)]);
        assert_eq!(r, InStratum::ToHost(70 - top));
        assert_eq!(b.clip_top_y, 70);
        assert_eq!(ore_top_in_column(&[&b], C, 0, 0), Some(70));
        let (r, b) = settle(&emptied, &[sand(74, 69), lime(68, 40)]);
        assert_eq!(r, InStratum::ToHost(68 - top));
        assert_eq!(ore_top_in_column(&[&b], C, 0, 0), Some(68));
        let (r, b) = settle(&emptied, &[sand(74, 0)]);
        assert_eq!(r, InStratum::OutOfHost(70 - top));
        assert_eq!(ore_top_in_column(&[&b], C, 0, 0), Some(70));
        // Its ore in rock already: kept, clip and all.
        let deep = body(1, 60.0, 1.0, true, 70);
        let (r, b) = settle(&deep, &[lime(74, 40)]);
        assert_eq!(r, InStratum::Kept);
        assert_eq!(b, deep);
    }

    /// The move is rigid and whole: centre, clip and box go together, and the moved body's ore
    /// tops out exactly at the top rock block in its centre column.
    #[test]
    fn a_moved_body_holds_ore_at_the_top_rock_block() {
        for (y, half, clip) in [
            (100.4, 2.0, 101),
            (80.0, 3.0, NO_CLIP),
            (91.7, 1.4, NO_CLIP),
        ] {
            let mut b = body(1, y, half, true, clip);
            let before = (b.center[1], b.clip_top_y, b.aabb);
            let rock = 64;
            let dy = anchor_shift(&[&b], C, 0, 0, rock, true);
            assert!(!ore_in_column(&[&b], C, 0, 0, rock, 256));
            shift_body(&mut b, dy);
            assert_eq!(b.center[1], before.0 + f64::from(dy));
            if clip != NO_CLIP {
                assert_eq!(b.clip_top_y, before.1 + dy);
            }
            assert_eq!(b.aabb[1][1], before.2[1][1] + dy);
            assert_eq!(b.aabb[0][0], before.2[0][0], "x and z stay");
            assert_eq!(ore_top_in_column(&[&b], C, 0, 0), Some(rock), "{y} {half}");
            assert!(ore_in_column(&[&b], C, 0, 0, rock, 256));
            // The block above the top rock holds none of it.
            assert!(!places_ore(&b, C, 0, rock + 1, 0));
        }
    }

    /// A natural porphyry's gossan and its supergene child's blanket move down together to the
    /// world's top rock block (ORE-285), its hypogene ore staying where it was; a profile already
    /// under the rock top stays, and a guaranteed family is left to the guaranteed anchor.
    #[test]
    fn a_natural_weathering_profile_comes_down_to_the_worlds_ground() {
        let wp = WorldProjection::Lambert(deeptime_format::Projection::new(2048));
        let world = HalfLand { wp };
        let site = |id: u32, kind: SiteKind, parent: Option<u32>, ev: u64| Site {
            id,
            kind,
            deposit_type: "deeptime:porphyry_cu_mo_au".into(),
            center: [256, 137, 0],
            lat_lon: [0.0, 0.0],
            aabb: [[240, 116, -16], [272, 140, 16]],
            body: None,
            products: Vec::new(),
            forming_event: EventId::new(1, ev),
            parent,
            extraction: deeptime_format::sites::Extraction::default(),
        };
        let placed = |id: u32, site: u32, role: &str, y: f64, half: f32, clip: i32| {
            let mut b = body(id, y, half, true, clip);
            b.site = Some(site);
            b.role = role.into();
            b.center[0] = 256.5;
            b.aabb = b.computed_aabb();
            b
        };
        let fresh = || {
            vec![
                placed(1, 24, "ore", 128.0, 12.0, 140),
                placed(2, 24, "gossan", 138.5, 1.5, 140),
                placed(3, 25, "ore", 137.5, 1.5, 139),
            ]
        };
        let all_sites = || {
            let mut porphyry = site(24, SiteKind::Deposit, None, 1);
            porphyry.body = Some(1);
            let mut blanket = site(25, SiteKind::Supergene, Some(24), 2);
            blanket.body = Some(3);
            vec![porphyry, blanket]
        };
        let mut sites = all_sites();
        // The rock top on land is 66: the profile comes down 140 - 66 = 74 blocks.
        let mut bodies = fresh();
        let w = settle_weathering(&world, &mut sites, &mut bodies, &BTreeMap::new());
        assert_eq!(
            (w.moved, w.bodies_moved, w.largest_move, w.kept),
            (1, 2, 74, 0)
        );
        assert_eq!(bodies[0], fresh()[0], "the hypogene ore stays");
        assert_eq!((bodies[1].clip_top_y, bodies[2].clip_top_y), (66, 65));
        assert_eq!(bodies[1].center[1], 138.5 - 74.0);
        assert_eq!(bodies[2].aabb[1][1], fresh()[2].aabb[1][1] - 74);
        // The sites follow their bodies: the porphyry's centre is its ore's and its box spans the
        // ore and the moved gossan; the blanket's centre and box came down with it.
        let span = |ids: &[usize]| {
            (
                ids.iter().map(|&i| bodies[i].aabb[0][1]).min().unwrap(),
                ids.iter().map(|&i| bodies[i].aabb[1][1]).max().unwrap(),
            )
        };
        assert_eq!(sites[0].center[1], 128);
        assert_eq!((sites[0].aabb[0][1], sites[0].aabb[1][1]), span(&[0, 1]));
        assert_eq!(sites[1].center[1], (137.5f64 - 74.0).floor() as i32);
        assert_eq!((sites[1].aabb[0][1], sites[1].aabb[1][1]), span(&[2]));
        // Under the rock top already: kept.
        let mut low = fresh();
        for b in &mut low {
            shift_body(b, -80);
        }
        let before = low.clone();
        let mut sites = all_sites();
        let w = settle_weathering(&world, &mut sites, &mut low, &BTreeMap::new());
        assert_eq!((w.moved, w.kept), (0, 1));
        assert_eq!(low, before);
        // A guaranteed parent: its family is the guaranteed anchor's.
        let mut held = fresh();
        let g: BTreeMap<EventId, String> = [(EventId::new(1, 1), "c:copper".to_owned())].into();
        let mut sites = all_sites();
        let w = settle_weathering(&world, &mut sites, &mut held, &g);
        assert_eq!((w.moved, w.kept), (0, 0));
        assert_eq!(held, fresh());
        assert_eq!(sites, all_sites());
    }

    /// A Lambert world of 2,048 blocks: land east of x = 0 (ground 70, rock top 66, limestone
    /// from 60 down), sea to the west.
    struct HalfLand {
        wp: WorldProjection,
    }

    impl Columns for HalfLand {
        fn period(&self) -> i64 {
            2048
        }
        fn z_range(&self) -> (i64, i64) {
            let (lo, hi) = self.wp.z_range();
            (lo.ceil() as i64, hi.floor() as i64)
        }
        fn dry(&self, x: i64, _z: i64) -> bool {
            (x + 1024).rem_euclid(2048) >= 1024
        }
        fn rock_top(&self, x: i64, z: i64) -> (i32, bool) {
            if self.dry(x, z) {
                (66, false)
            } else {
                (30, true)
            }
        }
        fn rock(&self, _x: i64, _z: i64) -> Vec<(i32, i32, Option<String>)> {
            vec![
                (66, 61, Some("sandstone".into())),
                (60, 0, Some("limestone".into())),
            ]
        }
    }

    /// The top-up (DEP-220) gives every land region of the world that lacks a source with ore a
    /// copy of the essential's guaranteed site, at the region's inland point nearest the spawn,
    /// anchored with its ore at the top rock block, with its own guaranteed-occurrence event; sea
    /// regions get none, and a second top-up adds nothing.
    #[test]
    fn the_top_up_fills_every_land_region_of_the_world() {
        let wp = WorldProjection::Lambert(deeptime_format::Projection::new(2048));
        let world = HalfLand { wp };
        let ess: Essential = serde_json::from_str(
            r#"{"material": "c:iron", "min_sites_per_region": 1, "region_blocks": 512, "exposed": true}"#,
        )
        .unwrap();
        let mut b = body(7, 66.0, 2.0, true, NO_CLIP);
        b.center[0] = 256.5;
        b.center[2] = -100.5;
        b.aabb = b.computed_aabb();
        b.site = Some(3);
        let refs = [&b];
        let dy = anchor_shift(&refs, 2048, 256, -101, 66, true);
        shift_body(&mut b, dy);
        let e1 = EventId::new(1, 5);
        let site = Site {
            id: 3,
            kind: SiteKind::Deposit,
            deposit_type: "deeptime:bif".into(),
            center: [256, 66, -101],
            lat_lon: [0.0, 0.0],
            aabb: b.aabb,
            body: Some(7),
            products: vec![deeptime_format::Product {
                material: "c:iron".into(),
                grade: 0.3,
                tonnage_t: 1.0e6,
                role: "primary".into(),
            }],
            forming_event: e1,
            parent: None,
            extraction: deeptime_format::sites::Extraction::default(),
        };
        let mut sites = vec![site];
        let mut bodies = vec![b];
        let mut guaranteed: BTreeMap<EventId, String> = [(e1, "c:iron".to_owned())].into();
        let in_stratum = BTreeMap::new();
        let mut seq = 100;
        let spawn = (700.0, 0.0);
        let regions = Regions::in_frame(wp.index_frame(), 512);
        let mut run =
            |sites: &mut Vec<Site>, bodies: &mut Vec<Body>, g: &mut BTreeMap<EventId, String>| {
                let mut d = Deposits {
                    sites,
                    bodies,
                    guaranteed: g,
                    in_stratum: &in_stratum,
                };
                top_up(
                    &world,
                    &wp,
                    &mut d,
                    std::slice::from_ref(&ess),
                    spawn,
                    &mut |_| (-1.0e9, -1.0e9),
                    &mut || {
                        seq += 1;
                        EventId::new(1, seq)
                    },
                )
            };
        let (rec, events) = run(&mut sites, &mut bodies, &mut guaranteed);
        let r = &rec["c:iron"];
        // Land in two of the four columns of regions, on every row.
        assert!(r.land_regions >= 2, "{r:?}");
        assert_eq!(r.with_source, 1);
        assert_eq!(r.added, r.land_regions - 1, "{r:?}");
        assert_eq!((r.inland, r.short), (r.added, 0), "{r:?}");
        assert_eq!(events.len(), r.added);
        let mut filled = vec![regions.index(256.0, -101.0)];
        for s in &sites[1..] {
            let (x, z) = (s.center[0], s.center[2]);
            assert!((128..=1024 - 128).contains(&x), "inland: {x}");
            let region = regions.index(f64::from(x), f64::from(z));
            assert!(!filled.contains(&region), "one copy a region");
            filled.push(region);
            let mine: Vec<&Body> = bodies.iter().filter(|b| b.site == Some(s.id)).collect();
            assert_eq!(mine.len(), 1);
            assert_eq!(s.body, Some(mine[0].id));
            assert_eq!(
                ore_top_in_column(&mine, 2048, x, z),
                Some(66),
                "site {}",
                s.id
            );
            let e = events.iter().find(|e| e.id == s.forming_event).unwrap();
            assert_eq!(e.kind, GUARANTEED_KIND);
            assert_eq!(e.causes, vec![e1]);
            assert_eq!(e.t_start, -1.0e9);
            assert_eq!(guaranteed[&s.forming_event], "c:iron");
        }
        // The spawn's own region gets the inland point nearest it.
        let near = sites[1..]
            .iter()
            .map(|s| {
                super::super::cluster::distance(
                    2048.0,
                    (f64::from(s.center[0]), f64::from(s.center[2])),
                    spawn,
                )
            })
            .fold(f64::INFINITY, f64::min);
        assert!(near < 46.0, "{near}");
        let ids: HashSet<u32> = bodies.iter().map(|b| b.id).collect();
        assert_eq!(ids.len(), bodies.len(), "body ids stay unique");
        // Settled.
        let (again, events) = run(&mut sites, &mut bodies, &mut guaranteed);
        assert_eq!((again["c:iron"].added, events.len()), (0, 0));
    }

    /// A Lambert world of 2,048 blocks with land east of x = `edge` (rock top 66, limestone from
    /// 60 down) and the sea to its west (the bed's rock top 30, water over it); the columns of
    /// `lake` stand above the sea with water over them (rock top 60).
    struct Coast {
        wp: WorldProjection,
        edge: i64,
        lake: std::ops::Range<i64>,
    }

    impl Columns for Coast {
        fn period(&self) -> i64 {
            2048
        }
        fn z_range(&self) -> (i64, i64) {
            let (lo, hi) = self.wp.z_range();
            (lo.ceil() as i64, hi.floor() as i64)
        }
        fn dry(&self, x: i64, _z: i64) -> bool {
            (x + 1024).rem_euclid(2048) - 1024 >= self.edge
        }
        fn rock_top(&self, x: i64, z: i64) -> (i32, bool) {
            if self.lake.contains(&x) {
                (60, true)
            } else if self.dry(x, z) {
                (66, false)
            } else {
                (30, true)
            }
        }
        fn rock(&self, _x: i64, _z: i64) -> Vec<(i32, i32, Option<String>)> {
            vec![(60, 0, Some("limestone".into()))]
        }
    }

    fn coast(edge: i64, lake: std::ops::Range<i64>) -> Coast {
        Coast {
            wp: WorldProjection::Lambert(deeptime_format::Projection::new(2048)),
            edge,
            lake,
        }
    }

    /// A guaranteed iron site with one body, anchored with its ore at the top rock block over
    /// (x, z), as the playability pass leaves it.
    fn iron_site(world: &Coast, id: u32, forming: EventId, x: i32, z: i32) -> (Site, Body) {
        let (top, _) = world.rock_top(i64::from(x), i64::from(z));
        let mut b = body(100 + id, f64::from(top), 2.0, true, NO_CLIP);
        b.center[0] = f64::from(x) + 0.5;
        b.center[2] = f64::from(z) + 0.5;
        b.aabb = b.computed_aabb();
        b.site = Some(id);
        let dy = anchor_shift(&[&b], 2048, x, z, top, true);
        shift_body(&mut b, dy);
        let site = Site {
            id,
            kind: SiteKind::Deposit,
            deposit_type: "deeptime:bif".into(),
            center: [x, top, z],
            lat_lon: [0.0, 0.0],
            aabb: b.aabb,
            body: Some(b.id),
            products: vec![deeptime_format::Product {
                material: "c:iron".into(),
                grade: 0.3,
                tonnage_t: 1.0e6,
                role: "primary".into(),
            }],
            forming_event: forming,
            parent: None,
            extraction: deeptime_format::sites::Extraction::default(),
        };
        (site, b)
    }

    fn iron() -> Essential {
        serde_json::from_str(
            r#"{"material": "c:iron", "min_sites_per_region": 1, "region_blocks": 512, "exposed": true}"#,
        )
        .unwrap()
    }

    /// One top-up of `world` over `sites` (ids 1.. of writer 1 are theirs), the spawn at (700, 0).
    fn run_top_up(
        world: &Coast,
        sites: &mut Vec<Site>,
        bodies: &mut Vec<Body>,
        guaranteed: &mut BTreeMap<EventId, String>,
        seq: &mut u64,
    ) -> (TopUp, Vec<Event>) {
        let in_stratum = BTreeMap::new();
        let mut d = Deposits {
            sites,
            bodies,
            guaranteed,
            in_stratum: &in_stratum,
        };
        let (mut rec, events) = top_up(
            world,
            &world.wp,
            &mut d,
            &[iron()],
            (700.0, 0.0),
            &mut |_| (-1.0e9, -1.0e9),
            &mut || {
                *seq += 1;
                EventId::new(1, *seq)
            },
        );
        (rec.remove("c:iron").unwrap(), events)
    }

    /// A source whose centre is under the sea counts for no region (Ben's rule of 2026-10-16): a
    /// land region whose only source with ore lies in the sea gets a copy at a dry point (ore at
    /// the top rock block), one with a dry source gets none, and the copy's template is the dry
    /// source though the sea one forms first. The copies are dry, so a second top-up adds none.
    #[test]
    fn the_top_up_does_not_count_a_source_under_the_sea() {
        // Land from x = 300: the region of columns [0, 512) is land in its east part, its
        // source at x = 100 lies under the sea.
        let world = coast(300, 0..0);
        let regions = Regions::in_frame(world.wp.index_frame(), 512);
        let (wet_ev, dry_ev) = (EventId::new(1, 5), EventId::new(1, 6));
        let (wet, wet_body) = iron_site(&world, 3, wet_ev, 100, -100);
        let (dry, dry_body) = iron_site(&world, 4, dry_ev, 700, -100);
        let (wet_region, dry_region) = (regions.index(100.0, -100.0), regions.index(700.0, -100.0));
        assert_ne!(wet_region, dry_region);
        assert!(!centre_is_dry(&world, &wet) && centre_is_dry(&world, &dry));
        let mut sites = vec![wet, dry];
        let mut bodies = vec![wet_body, dry_body];
        let mut guaranteed: BTreeMap<EventId, String> =
            [(wet_ev, "c:iron".to_owned()), (dry_ev, "c:iron".to_owned())].into();
        let mut seq = 100;
        let (rec, events) = run_top_up(&world, &mut sites, &mut bodies, &mut guaranteed, &mut seq);
        // One counted, one left out, and the region it alone covered is now short for the sea.
        assert_eq!(
            (rec.counted, rec.sub_sea, rec.short_by_sea),
            (1, 1, 1),
            "{rec:?}"
        );
        assert_eq!(rec.short, 0, "{rec:?}");
        assert_eq!(events.len(), rec.added);
        let copies = &sites[2..];
        let in_region = |r: usize| {
            copies
                .iter()
                .filter(|s| regions.index(f64::from(s.center[0]), f64::from(s.center[2])) == r)
                .count()
        };
        assert_eq!(
            in_region(wet_region),
            1,
            "the sea source's region gets one copy"
        );
        assert_eq!(
            in_region(dry_region),
            0,
            "a region with a dry source gets none"
        );
        for s in copies {
            let (x, z) = (i64::from(s.center[0]), i64::from(s.center[2]));
            assert!(world.dry(x, z) && !world.rock_top(x, z).1, "{x} {z}");
            let mine: Vec<&Body> = bodies.iter().filter(|b| b.site == Some(s.id)).collect();
            assert_eq!(
                ore_top_in_column(&mine, 2048, s.center[0], s.center[2]),
                Some(66)
            );
            // The template is the dry source (id 4), not the sea one (id 3, the lower id).
            let e = events.iter().find(|e| e.id == s.forming_event).unwrap();
            assert_eq!(e.causes, vec![dry_ev], "site {}", s.id);
        }
        // Settled: every land region now holds a counted source.
        let (again, events) =
            run_top_up(&world, &mut sites, &mut bodies, &mut guaranteed, &mut seq);
        assert_eq!(
            (again.added, events.len(), again.short_by_sea),
            (0, 0, 0),
            "{again:?}"
        );
        assert_eq!((again.counted, again.sub_sea), (1 + rec.added, 1));
    }

    /// With no source of a dry centre anywhere the sea one is the template (a template under the
    /// sea is skipped only while another exists), every land region gets a dry copy of it, and the
    /// sea source itself still counts for nothing.
    #[test]
    fn a_sea_source_is_the_template_when_it_is_the_only_one() {
        let world = coast(300, 0..0);
        let regions = Regions::in_frame(world.wp.index_frame(), 512);
        let ev = EventId::new(1, 5);
        let (wet, wet_body) = iron_site(&world, 3, ev, 100, -100);
        let mut sites = vec![wet];
        let mut bodies = vec![wet_body];
        let mut guaranteed: BTreeMap<EventId, String> = [(ev, "c:iron".to_owned())].into();
        let mut seq = 100;
        let (rec, events) = run_top_up(&world, &mut sites, &mut bodies, &mut guaranteed, &mut seq);
        assert_eq!(
            (rec.counted, rec.sub_sea, rec.with_source),
            (0, 1, 0),
            "{rec:?}"
        );
        assert_eq!(rec.added, rec.land_regions, "{rec:?}");
        assert_eq!(rec.short, 0, "{rec:?}");
        assert!(sites[1..].iter().all(|s| centre_is_dry(&world, s)));
        assert!(events.iter().all(|e| e.causes == vec![ev]));
        // The sea source's own region (land in its east part) got its copy there.
        assert!(sites[1..].iter().any(|s| {
            regions.index(f64::from(s.center[0]), f64::from(s.center[2]))
                == regions.index(100.0, -100.0)
        }));
    }

    /// A centre is dry when the ground stands at or above sea level with no water over it: the
    /// sea's bed and a lake above the sea both fail it.
    #[test]
    fn a_dry_centre_is_ground_at_sea_level_with_no_water_over_it() {
        let world = coast(300, 500..600);
        let at = |x: i32| iron_site(&world, 1, EventId::new(1, 1), x, 0).0;
        assert!(centre_is_dry(&world, &at(320)), "land");
        assert!(!centre_is_dry(&world, &at(100)), "the sea's bed");
        assert!(!centre_is_dry(&world, &at(550)), "a lake above sea level");
        assert!(centre_is_dry(&world, &at(700)), "land past the lake");
    }
}
