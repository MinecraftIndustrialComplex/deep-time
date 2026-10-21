// SPDX-License-Identifier: LGPL-3.0-or-later
//! The playability pass's deposits in the world as generated (DEP-221, ORE-280).
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
//! horizon, where the bake and the projection keep it (ORE-271): moving it would take it out of
//! its host. A placer is left where `placer_channels` settled it (in a channel or on the ground).
//!
//! The guaranteed sites are the ones whose forming event is a `deeptime:guaranteed_occurrence`
//! (the event log; the run passes them from memory), so the pass needs no flag in the world data.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::time::Instant;

use deeptime_format::bodies::{BODIES_PATH, NO_CLIP, read_bodies, write_bodies};
use deeptime_format::events::{EventId, EventLog, ParamValue};
use deeptime_format::sites::{INDEX_PATH, SITES_PATH, read_sites, write_sites_in};
use deeptime_format::{Body, ColumnWorld, FileEntry, Manifest, Site, SiteKind};
use serde_json::{Value, json};

use super::SOIL_BLOCKS;
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
fn bears_ore(b: &Body) -> bool {
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

/// How far (whole blocks, up positive) a guaranteed site's bodies move: the top of its ore to
/// `rock_top` for an exposed essential, else down to it at most.
pub fn anchor_shift(bodies: &[&Body], rock_top: i32, exposed: bool) -> i32 {
    let ore: Vec<&&Body> = bodies.iter().filter(|b| bears_ore(b)).collect();
    let top = if ore.is_empty() {
        bodies.iter().map(|b| body_top(b)).max()
    } else {
        ore.iter().map(|b| body_top(b)).max()
    };
    let Some(top) = top else { return 0 };
    let dy = rock_top - top;
    if exposed { dy } else { dy.min(0) }
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
            .any(|y| b.locate(c, x, y, z).is_some())
    })
}

/// What [`anchor_sites`] did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Anchored {
    pub guaranteed: usize,
    /// Sites whose bodies moved, and the largest move (blocks).
    pub moved: usize,
    pub largest_move: i32,
    pub moved_up: usize,
    pub moved_down: usize,
    /// In-stratum sites left in their host horizon, and placers left where they settled.
    pub in_stratum_kept: usize,
    pub placers_kept: usize,
    /// Guaranteed sites whose centre column holds ore in rock after the pass, and those whose
    /// centre lies under water.
    pub centre_holds_ore: usize,
    pub under_water: usize,
}

/// Anchors the guaranteed `sites` (forming event in `guaranteed`, material → whether its
/// essential is `exposed`) in `world`, moving their `bodies`; `in_stratum` the in-stratum sites.
pub fn anchor_sites(
    world: &ColumnWorld,
    sites: &mut [Site],
    bodies: &mut [Body],
    guaranteed: &BTreeMap<EventId, String>,
    exposed: &dyn Fn(&str) -> bool,
    in_stratum: &HashSet<u32>,
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
        } else if in_stratum.contains(&s.id) {
            out.in_stratum_kept += 1;
        } else {
            let mine: Vec<&Body> = idx.iter().map(|&i| &bodies[i]).collect();
            let dy = anchor_shift(&mine, top, exposed(material));
            if dy != 0 {
                for &i in &idx {
                    shift_body(&mut bodies[i], dy);
                }
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

/// Anchors the guaranteed deposits of the finished world folder `dir` (its manifest `m`, written
/// to disk) in the world as generated; `guaranteed` the run's guaranteed forming events (read
/// from the folder's event log when None). Rewrites the bodies and sites when any moved; returns
/// the pass's record (`notes.bake.guaranteed`).
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
    let loaded;
    let guaranteed = match guaranteed {
        Some(g) => g,
        None => {
            loaded = match EventLog::load(dir) {
                Ok(log) => guaranteed_of(log.of_kind(GUARANTEED_KIND)),
                Err(_) => BTreeMap::new(),
            };
            &loaded
        }
    };
    if !sites
        .iter()
        .any(|s| guaranteed.contains_key(&s.forming_event))
    {
        return Ok(json!({"guaranteed": 0}));
    }
    let Some(mut bodies) = read_bodies(dir)? else {
        return Ok(json!({"guaranteed": 0, "why": "no bodies"}));
    };
    let world = ColumnWorld::open(dir)?;
    let types = super::types::builtin()?;
    let in_stratum: HashSet<u32> = sites
        .iter()
        .filter(|s| {
            types
                .iter()
                .any(|t| t.id == s.deposit_type && t.hosts.in_stratum)
        })
        .map(|s| s.id)
        .collect();
    let essentials = p
        .deposits
        .as_ref()
        .map(|d| d.essentials.clone())
        .unwrap_or_default();
    let exposed = |material: &str| {
        essentials
            .iter()
            .any(|e| e.material == material && e.exposed)
    };
    let a = anchor_sites(
        &world,
        &mut sites,
        &mut bodies,
        guaranteed,
        &exposed,
        &in_stratum,
    );
    if a.moved > 0 {
        let (bytes, crc32) = write_bodies(dir, &bodies)?;
        let w = write_sites_in(
            dir,
            m.scale.circumference_blocks,
            m.world_projection().index_frame(),
            &sites,
        )?;
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
    }
    Ok(json!({
        "guaranteed": a.guaranteed,
        "moved": a.moved,
        "moved_up": a.moved_up,
        "moved_down": a.moved_down,
        "largest_move_blocks": a.largest_move,
        "in_stratum_kept": a.in_stratum_kept,
        "placers_kept": a.placers_kept,
        "centre_holds_ore": a.centre_holds_ore,
        "under_water": a.under_water,
        "millis": started.elapsed().as_secs_f64() * 1000.0,
    }))
}
