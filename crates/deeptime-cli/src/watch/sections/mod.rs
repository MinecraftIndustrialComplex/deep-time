// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's "Rock at depth" and "Sections" (decisions log 2026-10-01, "Which workbench
//! features?"): what the run's bake says about the rock under the map, answered on request.
//!
//! - **Depth slice.** The rock class at a depth below the surface (surface, 20, 50, 100 … blocks,
//!   down to the world floor) over the whole map, as a picture that sits under the map exactly
//!   like a frame's layer ([`slice`]), coloured with the `rock_class` ramp.
//! - **Cross-section.** The strata along a line between two points: rock classes, the surface,
//!   water, and the bodies (deposits, seams, plutons) the line passes through ([`section`]).
//!
//! Both are read from the run's finished bake (`<run>/world`, the column evaluator's data), so they
//! exist only for runs that have one. They add nothing to the simulation or the bake: a request
//! reads the world data and computes, then keeps the answer in `<run>/rock/<stamp>/` (a picture per
//! depth, a JSON per line; the stamp is the bake's, so a world written again starts afresh), so
//! asking again is a file read. The finished bake's strata stay in memory for
//! ten minutes after the last request (one world at a time), so moving the depth slider does not
//! reload them.
//!
//! Routes, all `GET` (only `/api/runs/<id>/rock…` and the page's script):
//! - `/api/runs/<id>/rock`: whether the run has a bake to read (`available`, `why` when not), the
//!   world's scale, the class names with their colours and the depths the page offers;
//! - `/api/runs/<id>/rock/slice.png?depth=<blocks|floor>[&w=<pixels>]`: the slice, RGBA, `w` × `w/2`
//!   (1,024 by default), transparent where nothing is named;
//! - `/api/runs/<id>/rock/section?x0=&z0=&x1=&z1=[&n=<columns>]`: the section from block (x0, z0) to
//!   (x1, z1) along the shorter way round in x, as JSON (see [`section::Section::columns_json`]);
//! - `/sections.js`: the page's script (`crates/deeptime-viewer/assets/sections.js`).

mod section;
mod slice;
#[cfg(test)]
mod tests;
mod world;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, UNIX_EPOCH};

use deeptime_core::synth::EVERY_CLASS;
use deeptime_format::Manifest;
use serde_json::{Value, json};

use super::runs::{Run, Workbench};
use slice::Depth;
use world::RockWorld;

/// A route's answer.
pub struct Reply {
    pub status: u16,
    pub content_type: &'static str,
    pub cache: &'static str,
    pub body: Vec<u8>,
}

impl Reply {
    fn json(status: u16, v: &Value) -> Reply {
        Reply {
            status,
            content_type: "application/json",
            cache: "no-store",
            body: v.to_string().into_bytes(),
        }
    }

    fn error(status: u16, message: impl Into<String>) -> Reply {
        Reply::json(status, &json!({ "error": message.into() }))
    }

    fn unavailable(why: &str) -> Reply {
        Reply::json(
            409,
            &json!({ "error": why, "available": false, "why": why }),
        )
    }
}

/// A finished bake's strata stay in memory this long after the last request.
const KEEP_WORLD: Duration = Duration::from_secs(600);
/// The map's width in pixels: one per L1 cell on a 16k planet, which is all the strata hold.
const DEFAULT_W: u32 = 1024;
const MAX_W: u32 = 2048;
/// The depths the page offers, blocks below the surface (and the floor).
const DEPTHS: [u32; 13] = [0, 10, 20, 35, 50, 75, 100, 150, 200, 300, 400, 600, 800];

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

struct Cached {
    /// The world folder and its manifest's size and time: a re-baked folder is another world.
    key: (PathBuf, u64, Option<std::time::SystemTime>),
    world: Arc<RockWorld>,
    used: Instant,
}

static WORLD: Mutex<Option<Cached>> = Mutex::new(None);
/// One slice or section computes at a time (each already uses every thread it is given).
static COMPUTE: Mutex<()> = Mutex::new(());
static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

fn threads() -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get().min(8))
}

fn world_dir(run: &Run) -> PathBuf {
    run.dir.join("world")
}

/// The folder the answers are kept in: one per bake (the manifest's size and time), so a world
/// written again does not answer from the old one's files.
fn cache_dir(run: &Run) -> PathBuf {
    let stamp = std::fs::metadata(world_dir(run).join("manifest.json")).map_or_else(
        |_| "none".to_owned(),
        |m| {
            let secs = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            format!("{:x}-{secs:x}", m.len())
        },
    );
    run.dir.join("rock").join(stamp)
}

/// Whether the run has a bake to read, and what it is: the manifest's facts, or why not.
fn bake_facts(run: &Run) -> Result<(Manifest, Vec<String>), String> {
    let dir = world_dir(run);
    if !run.is_finished() {
        return Err(
            "The run has not finished: its bake (the world data) is written when it ends.".into(),
        );
    }
    let m = Manifest::read(&dir).map_err(|e| match e {
        deeptime_format::FormatError::Version { found, supported } => format!(
            "The run's world data is format {found}; this workbench reads {supported}. \
             Run it again to get a bake it can read."
        ),
        e => format!("The run has no readable bake: {e}"),
    })?;
    let baked = m.notes.get("bake").is_some_and(|b| b["baked"] != false);
    if !baked {
        return Err("The run's world data has no bake (a stage run, or an older run).".into());
    }
    if m.layer(deeptime_format::column::HORIZONS).is_none() || m.layer("elevation").is_none() {
        return Err("The run's world data has no terrain or strata layer.".into());
    }
    let path = dir.join(deeptime_format::column::CLASSES_PATH);
    let classes: Value = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .ok_or("The run's world data has no strata/classes.json.")?;
    let names = classes["classes"]
        .as_array()
        .ok_or("strata/classes.json has no classes array")?
        .iter()
        .map(|c| c.as_str().unwrap_or("?").to_owned())
        .collect();
    Ok((m, names))
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn classes_json(names: &[String]) -> Value {
    let colours = world::class_colours(names);
    Value::Array(
        names
            .iter()
            .zip(&colours)
            .enumerate()
            .map(|(i, (n, c))| {
                // `k`: the class's index in the `rock_class` ramp (the rock-tag README's order),
                // which the page names its colours by.
                let k = EVERY_CLASS.iter().position(|e| e == n);
                json!({ "i": i, "name": n, "colour": hex(*c), "k": k })
            })
            .collect(),
    )
}

fn info(run: &Run) -> Reply {
    let (m, names) = match bake_facts(run) {
        Ok(f) => f,
        Err(why) => {
            return Reply::json(
                200,
                &json!({ "available": false, "run": run.id, "why": why }),
            );
        }
    };
    let s = &m.scale;
    let mut depths: Vec<Value> = DEPTHS.iter().map(|&d| json!(d)).collect();
    depths.push(json!("floor"));
    Reply::json(
        200,
        &json!({
            "available": true,
            "run": run.id,
            "format_version": m.format_version,
            "source": m.source,
            "placeholder": m.notes.get("run").is_some_and(|r| r["placeholder"] == true),
            "circumference": s.circumference_blocks,
            "min_y": s.min_y, "max_y": s.max_y, "sea_level_y": s.sea_level_y,
            "m_per_block_h": s.m_per_block_h, "m_per_block_v": s.m_per_block_v,
            "depths": depths,
            "classes": classes_json(&names),
        }),
    )
}

/// The world of `run`, opened (or the one kept from the last request).
fn world_for(run: &Run) -> Result<Arc<RockWorld>, String> {
    let dir = world_dir(run);
    let meta = std::fs::metadata(dir.join("manifest.json")).ok();
    let key = (
        dir.clone(),
        meta.as_ref().map_or(0, std::fs::Metadata::len),
        meta.and_then(|m| m.modified().ok()),
    );
    let mut cached = lock(&WORLD);
    if let Some(c) = cached.as_mut() {
        if c.key == key && c.used.elapsed() < KEEP_WORLD {
            c.used = Instant::now();
            return Ok(Arc::clone(&c.world));
        }
        *cached = None;
    }
    let t = Instant::now();
    let world = Arc::new(RockWorld::open(&dir).map_err(|e| format!("the run's bake: {e}"))?);
    println!(
        "deeptime watch: {} rock: opened the bake in {:.2}s",
        run.id,
        t.elapsed().as_secs_f64()
    );
    *cached = Some(Cached {
        key,
        world: Arc::clone(&world),
        used: Instant::now(),
    });
    Ok(world)
}

/// Writes `bytes` to `path` (its folder made) through a temporary file, so a reader never sees a
/// half-written answer; failing to cache is not an error.
fn keep(path: &Path, bytes: &[u8]) {
    let Some(dir) = path.parent() else { return };
    let _ = std::fs::create_dir_all(dir);
    let tmp = path.with_extension(format!("tmp{}", TMP_SEQ.fetch_add(1, Ordering::Relaxed)));
    if std::fs::write(&tmp, bytes).is_ok() && std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v)
}

fn int_param(query: &str, key: &str) -> Result<i64, String> {
    query_param(query, key)
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite() && v.abs() < 1e9)
        .map(|v| v.round() as i64)
        .ok_or_else(|| format!("give {key} as a block coordinate"))
}

fn slice_reply(run: &Run, query: &str) -> Reply {
    let depth = match query_param(query, "depth").map_or(Some(Depth::Below(0)), Depth::parse) {
        Some(d) => d,
        None => return Reply::error(400, "depth: blocks below the surface, or floor"),
    };
    let w = query_param(query, "w")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_W)
        .clamp(64, MAX_W)
        & !1;
    if let Err(why) = bake_facts(run) {
        return Reply::unavailable(&why);
    }
    let file = cache_dir(run).join(format!("slice-d{}-w{w}.png", depth.label()));
    if let Ok(png) = std::fs::read(&file) {
        return Reply {
            status: 200,
            content_type: "image/png",
            cache: "private, max-age=300",
            body: png,
        };
    }
    let _busy = lock(&COMPUTE);
    // Another request may have made it while this one waited.
    if let Ok(png) = std::fs::read(&file) {
        return Reply {
            status: 200,
            content_type: "image/png",
            cache: "private, max-age=300",
            body: png,
        };
    }
    let world = match world_for(run) {
        Ok(w) => w,
        Err(e) => return Reply::error(500, e),
    };
    let t = Instant::now();
    let slice = world.slice(
        depth,
        w,
        slice::picture_height(&world.cw.manifest.world_projection(), w),
        threads(),
    );
    let png = match deeptime_viewer::encode_png(slice.w, slice.h, &slice.rgba(&world.colours)) {
        Ok(p) => p,
        Err(e) => return Reply::error(500, format!("encoding the slice: {e}")),
    };
    println!(
        "deeptime watch: {} rock: slice depth {} ({}×{}) in {:.2}s",
        run.id,
        depth.label(),
        slice.w,
        slice.h,
        t.elapsed().as_secs_f64()
    );
    keep(&file, &png);
    Reply {
        status: 200,
        content_type: "image/png",
        cache: "private, max-age=300",
        body: png,
    }
}

fn section_reply(run: &Run, query: &str) -> Reply {
    let ints = (|| -> Result<[i64; 4], String> {
        Ok([
            int_param(query, "x0")?,
            int_param(query, "z0")?,
            int_param(query, "x1")?,
            int_param(query, "z1")?,
        ])
    })();
    let [x0, z0, x1, z1] = match ints {
        Ok(v) => v,
        Err(e) => return Reply::error(400, e),
    };
    let (m, names) = match bake_facts(run) {
        Ok(f) => f,
        Err(why) => return Reply::unavailable(&why),
    };
    let c = i64::from(m.scale.circumference_blocks);
    let a = (x0, z0);
    let b = section::short_way(c, a, (x1, z1));
    let len = ((b.0 - a.0) as f64).hypot((b.1 - a.1) as f64);
    if len < 1.0 {
        return Reply::error(400, "pick two different points");
    }
    let n = query_param(query, "n")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| len.ceil() as usize + 1)
        .clamp(2, section::MAX_COLUMNS);
    let file = cache_dir(run).join(format!("section-{}_{}_{}_{}-n{n}.json", a.0, a.1, b.0, b.1));
    let cached = || {
        std::fs::read(&file).ok().map(|body| Reply {
            status: 200,
            content_type: "application/json",
            cache: "private, max-age=300",
            body,
        })
    };
    if let Some(r) = cached() {
        return r;
    }
    let _busy = lock(&COMPUTE);
    if let Some(r) = cached() {
        return r;
    }
    let world = match world_for(run) {
        Ok(w) => w,
        Err(e) => return Reply::error(500, e),
    };
    let t = Instant::now();
    let sec = world.section(a, b, n, threads());
    let seconds = t.elapsed().as_secs_f64();
    let s = &m.scale;
    let point = |p: (i64, i64)| {
        let wx = world.cw.wrap_x(p.0);
        let (lat, lon) = world
            .cw
            .manifest
            .world_projection()
            .lat_lon(wx as f64, p.1 as f64);
        let _ = c;
        json!({ "x": p.0, "z": p.1, "lat": lat, "lon": lon })
    };
    // The bodies the section crosses, with what they are.
    let mut ids: Vec<u32> = sec
        .columns
        .iter()
        .flat_map(|col| col.bodies.iter().map(|b| b.0))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let bodies: Vec<Value> = ids
        .iter()
        .filter_map(|id| world.cw.bodies.iter().find(|b| b.id == *id))
        .map(|b| {
            let site = b.site.and_then(|s| world.site(s));
            json!({ "id": b.id, "role": b.role, "site": b.site, "rock_class": b.rock_class,
                    "shape": b.shape.name(),
                    "deposit_type": site.map(|s| s.deposit_type.clone()) })
        })
        .collect();
    let doc = json!({
        "available": true,
        "run": run.id,
        "seconds": seconds,
        "circumference": c,
        "min_y": s.min_y, "max_y": s.max_y, "sea_level_y": s.sea_level_y,
        "m_per_block_h": s.m_per_block_h, "m_per_block_v": s.m_per_block_v,
        "a": point(sec.a), "b": point(sec.b),
        "length_blocks": sec.length_blocks,
        "length_km": sec.length_blocks * s.m_per_block_h / 1000.0,
        "n": sec.columns.len(),
        "classes": classes_json(&names),
        "columns": sec.columns_json(),
        "bodies": bodies,
    });
    println!(
        "deeptime watch: {} rock: section ({}, {}) → ({}, {}), {} columns, in {seconds:.2}s",
        run.id,
        a.0,
        a.1,
        b.0,
        b.1,
        sec.columns.len()
    );
    let body = doc.to_string().into_bytes();
    keep(&file, &body);
    Reply {
        status: 200,
        content_type: "application/json",
        cache: "private, max-age=300",
        body,
    }
}

/// Answers `GET path?query` if it is one of this module's, else `None`.
pub fn route(wb: &Workbench, path: &str, query: &str) -> Option<Reply> {
    if path == "/sections.js" {
        return Some(Reply {
            status: 200,
            content_type: "text/javascript; charset=utf-8",
            cache: "no-cache",
            body: deeptime_viewer::SECTIONS_JS.as_bytes().to_vec(),
        });
    }
    let rest = path.strip_prefix("/api/runs/")?;
    let (id, tail) = rest.split_once('/')?;
    let tail = tail.strip_prefix("rock")?;
    if !matches!(tail, "" | "/slice.png" | "/section") {
        return None;
    }
    let Some(run) = wb.run(id) else {
        return Some(Reply::error(
            404,
            format!("no run {id} (it may have been pruned)"),
        ));
    };
    Some(match tail {
        "" => info(&run),
        "/slice.png" => slice_reply(&run, query),
        _ => section_reply(&run, query),
    })
}
