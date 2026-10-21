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
//! horizon, where the bake and the projection keep it (ORE-271), while its ore lies in rock in
//! its centre column; otherwise (its horizon left in the soil, where the projection's clip under
//! the drawn ground empties it) it is read unclipped, goes down into the nearest block of a host
//! class under the top rock block and is clipped there ([`settle_in_stratum`]). A placer is left
//! where `placer_channels` settled it (in a channel or on the ground).
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

/// How far (whole blocks, never up) a guaranteed in-stratum body moves. It stays in its host
/// horizon, where the bake and the projection keep it (ORE-271), while the top of its ore in the
/// centre column lies at or under `rock_top`. Where the bake left the horizon in the soil or
/// above the ground (a quick planet's guaranteed MVT lay 2 blocks under its ground, in the
/// soil), the ore's top goes to the highest block of a host class in `rock` (the centre column's
/// rock runs, top first: (top, bottom, class)) at or under `rock_top` and within
/// [`HOST_SEARCH_BLOCKS`] of it, and to `rock_top` itself when there is none. Returns the move and
/// whether the body ended in a host class.
pub fn in_stratum_shift(
    bodies: &[&Body],
    c: u32,
    x: i32,
    z: i32,
    rock_top: i32,
    rock: &[(i32, i32, Option<String>)],
    hosts: &HashSet<String>,
) -> (i32, bool) {
    let dy = anchor_shift(bodies, c, x, z, rock_top, false);
    if dy == 0 {
        return (0, true);
    }
    let floor = rock_top - HOST_SEARCH_BLOCKS;
    let host_top = rock
        .iter()
        .filter(|(_, _, k)| k.as_ref().is_some_and(|k| hosts.contains(k)))
        .filter(|&&(top, bottom, _)| bottom <= rock_top && top >= floor)
        .map(|&(top, _, _)| top.min(rock_top))
        .max();
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
/// A site whose ore tops out in rock in its centre column is kept as it is. Any other is read
/// unclipped and moved by [`in_stratum_shift`], and every body is then clipped at `rock_top`, so
/// nothing of it lies in the soil.
pub fn settle_in_stratum(
    bodies: &mut [Body],
    c: u32,
    x: i32,
    z: i32,
    rock_top: i32,
    rock: &[(i32, i32, Option<String>)],
    hosts: &HashSet<String>,
) -> InStratum {
    let refs: Vec<&Body> = bodies.iter().collect();
    if ore_top_in_column(&refs, c, x, z).is_some_and(|t| t <= rock_top) {
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
    let (dy, in_host) = in_stratum_shift(&refs, c, x, z, rock_top, rock, hosts);
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
                    let r = settle_in_stratum(&mut own, c, cx, cz, top, &rock, hosts);
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
    // The guaranteed in-stratum sites and their type's host classes (weight above 0).
    let in_stratum: BTreeMap<u32, HashSet<String>> = sites
        .iter()
        .filter(|s| guaranteed.contains_key(&s.forming_event))
        .filter_map(|s| {
            let t = types
                .iter()
                .find(|t| t.id == s.deposit_type && t.hosts.in_stratum)?;
            let hosts = t.hosts.weights().unwrap_or_default();
            Some((
                s.id,
                hosts
                    .into_iter()
                    .filter(|&(_, w)| w > 0.0)
                    .map(|(k, _)| k)
                    .collect(),
            ))
        })
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
        "in_stratum_to_host": a.in_stratum_to_host,
        "in_stratum_out_of_host": a.in_stratum_out_of_host,
        "placers_kept": a.placers_kept,
        "centre_holds_ore": a.centre_holds_ore,
        "under_water": a.under_water,
        "millis": started.elapsed().as_secs_f64() * 1000.0,
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
            in_stratum_shift(&[&ore], C, 0, 0, rock_top, rock, &hosts)
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
            let r = settle_in_stratum(&mut bs, C, 0, 0, 70, rock, &hosts);
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
}
