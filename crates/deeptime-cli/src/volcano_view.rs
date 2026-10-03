// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime volcano-view` (P1-21): review pictures of a run's volcanoes from its last checkpoint
//! (the catalogue, the young landscape, the recorded history) and its world-data folder (the L3
//! terrain the game reads):
//!
//! - `map.png`: the finished terrain (the amplifier's thinned L3 copy), hill-shaded, with every
//!   standing volcano (by kind and state) and flood-basalt province marked;
//! - `settings.png`: the same markers over the tectonic settings that make them — the arc flux of
//!   the last frame of the recorded history with arcs in it (red) and the live hotspots' plumes;
//! - close views of the tallest arc volcano, an island-arc volcano, the longest hotspot chain, a
//!   caldera, a silicic centre and the best-kept flood basalt: the young landscape (L2, the deep-time
//!   edifice interpolated) beside the finished L3 terrain, and an east–west profile through the
//!   summit;
//! - `volcanoes.json` (every catalogue entry with its block position) and `index.html`.

use std::fmt::Write as _;
use std::path::Path;

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::deposits::fields::FieldHistory;
use deeptime_core::surface::amplify::resample;
use deeptime_core::surface::volcanoes::{Kind, Volcano, VolcanoState};
use deeptime_format::{LoadedLayer, Ramps, RasterRead, TilePackReader};
use serde_json::{Value, json};

use crate::run::CliResult;

/// Vertical exaggeration of the hill-shading.
const VE: f64 = 3.0;

struct Ctx<'a> {
    l3: &'a LoadedLayer,
    gw: i64,
    gh: i64,
    /// L3 cell size, blocks and metres.
    r: f64,
    dx: f64,
    c: f64,
    sea_y: f64,
    mv: f64,
    sea_m: f64,
}

impl Ctx<'_> {
    fn y(&self, i: i64, j: i64) -> f32 {
        (self.l3.raw(i.rem_euclid(self.gw), j.clamp(0, self.gh - 1)) / 16.0) as f32
    }

    /// L3 cell of block (x, z).
    fn cell(&self, x: f64, z: f64) -> (i64, i64) {
        (
            ((x + self.c / 2.0) / self.r).round() as i64,
            ((z + deeptime_format::Projection::of(self.c).pole_z()) / self.r).round() as i64,
        )
    }
}

/// Hill-shaded, tinted picture of block heights `y` (w × h), scaled `k` times (nearest).
fn shade(y: &[f32], w: usize, h: usize, k: usize, ctx: &Ctx<'_>, dx: f64) -> Vec<u8> {
    let ramps = Ramps::builtin();
    let (_, ramp) = ramps.for_layer("elevation");
    let light = {
        let (x, yy, z) = (-0.6f64, 0.6f64, 0.7f64);
        let n = (x * x + yy * yy + z * z).sqrt();
        (x / n, yy / n, z / n)
    };
    let at = |i: usize, j: usize| f64::from(y[j.min(h - 1) * w + i.min(w - 1)]);
    let (ow, oh) = (w * k, h * k);
    let mut out = vec![0u8; ow * oh * 4];
    for j in 0..h {
        for i in 0..w {
            let yc = at(i, j);
            let tint = ramp.colour(yc);
            let (mut r, mut g, mut b) =
                (f64::from(tint[0]), f64::from(tint[1]), f64::from(tint[2]));
            {
                // Shaded under the sea too (more softly), so seamounts show.
                let gx = (at(i + 1, j) - at(i.saturating_sub(1), j)) * ctx.mv / (2.0 * dx);
                let gs = (at(i, j + 1) - at(i, j.saturating_sub(1))) * ctx.mv / (2.0 * dx);
                let n = (-VE * gx, VE * gs, 1.0f64);
                let len = (n.0 * n.0 + n.1 * n.1 + n.2 * n.2).sqrt();
                let s = ((n.0 * light.0 + n.1 * light.1 + n.2 * light.2) / len).max(0.0) / light.2;
                let f = if yc > ctx.sea_y {
                    (0.3 + 0.7 * s).clamp(0.2, 1.3)
                } else {
                    (0.55 + 0.6 * s).clamp(0.4, 1.6)
                };
                r = (r * f).min(255.0);
                g = (g * f).min(255.0);
                b = (b * f).min(255.0);
            }
            for dj in 0..k {
                for di in 0..k {
                    let c = ((j * k + dj) * ow + i * k + di) * 4;
                    out[c..c + 4].copy_from_slice(&[r as u8, g as u8, b as u8, 255]);
                }
            }
        }
    }
    out
}

fn put(img: &mut [u8], w: usize, h: usize, x: i64, y: i64, rgb: [u8; 3], a: f64) {
    if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
        return;
    }
    let c = (y as usize * w + x as usize) * 4;
    for k in 0..3 {
        img[c + k] = (f64::from(img[c + k]) * (1.0 - a) + f64::from(rgb[k]) * a) as u8;
    }
}

fn disc(img: &mut [u8], w: usize, h: usize, cx: f64, cy: f64, r: f64, rgb: [u8; 3]) {
    let ri = r.ceil() as i64 + 1;
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            let d = (dx as f64).hypot(dy as f64);
            let (x, y) = (cx.round() as i64 + dx, cy.round() as i64 + dy);
            if d <= r {
                put(img, w, h, x, y, rgb, 1.0);
            } else if d <= r + 1.2 {
                put(img, w, h, x, y, [0, 0, 0], 0.9);
            }
        }
    }
}

fn ring(img: &mut [u8], w: usize, h: usize, cx: f64, cy: f64, r: f64, rgb: [u8; 3]) {
    let n = (r * 8.0).max(32.0) as usize;
    for k in 0..n {
        let a = std::f64::consts::TAU * k as f64 / n as f64;
        for t in [-0.5, 0.0, 0.5] {
            let (x, y) = (cx + (r + t) * a.cos(), cy + (r + t) * a.sin());
            put(img, w, h, x.round() as i64, y.round() as i64, rgb, 0.9);
        }
    }
}

/// The colour of a volcano's marker.
fn colour(v: &Volcano) -> [u8; 3] {
    match (v.kind, v.active(), v.caldera.is_some()) {
        (Kind::Strato | Kind::Silicic, _, true) => [255, 121, 198],
        (Kind::Strato, true, _) => [255, 59, 31],
        (Kind::Strato, false, _) => [176, 114, 74],
        (Kind::Shield, true, _) => [255, 192, 46],
        (Kind::Shield, false, _) => [79, 143, 214],
        (Kind::Silicic, _, _) => [212, 92, 255],
    }
}

fn standing(v: &Volcano) -> bool {
    v.height_m >= (0.25 * v.peak_m).max(300.0)
}

/// A close view: the young landscape upsampled (before) and the L3 terrain (after), block heights.
struct Close {
    id: &'static str,
    title: String,
    note: String,
    before: Vec<f32>,
    after: Vec<f32>,
    side: usize,
    /// Row of the profile (the summit's) in the window.
    row: usize,
}

fn window(ctx: &Ctx<'_>, i0: i64, j0: i64, side: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(side * side);
    for j in 0..side as i64 {
        for i in 0..side as i64 {
            out.push(ctx.y(i0 + i, j0 + j));
        }
    }
    out
}

/// The young landscape (L2, metres) over the same L3 window, by the amplifier's interpolation.
#[allow(clippy::too_many_arguments)]
fn young_window(
    z2: &[f32],
    w2: usize,
    h2: usize,
    scale: i64,
    ctx: &Ctx<'_>,
    i0: i64,
    j0: i64,
    side: usize,
) -> Vec<f32> {
    let m = 4i64;
    let cx0 = i0.div_euclid(scale) - m;
    let cy0 = j0.div_euclid(scale) - m;
    let cw = (side as i64 / scale + 2 * m + 2) as usize;
    let mut win = vec![0.0f32; cw * cw];
    for j in 0..cw {
        let sj = (cy0 + j as i64).clamp(0, h2 as i64 - 1) as usize;
        for i in 0..cw {
            let si = (cx0 + i as i64).rem_euclid(w2 as i64) as usize;
            win[j * cw + i] = z2[sj * w2 + si];
        }
    }
    let (mut cur, mut w, mut h) = (win, cw, cw);
    let mut s = 1i64;
    while s < scale {
        cur = resample::upsample2_z(&cur, w, h);
        w *= 2;
        h *= 2;
        s *= 2;
    }
    let ox = (i0 - cx0 * scale) as usize;
    let oy = (j0 - cy0 * scale) as usize;
    let mut out = Vec::with_capacity(side * side);
    for j in 0..side {
        for i in 0..side {
            let zm = f64::from(cur[(oy + j) * w + ox + i]);
            out.push((ctx.sea_y + (zm - ctx.sea_m) / ctx.mv) as f32);
        }
    }
    out
}

/// An east–west profile chart of two rows of block heights (before grey, after red).
fn profile(before: &[f32], after: &[f32], sea_y: f64) -> Vec<u8> {
    let (w, h) = (before.len() * 3, 180usize);
    let mut img = vec![255u8; w * h * 4];
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &v in before.iter().chain(after) {
        lo = lo.min(f64::from(v));
        hi = hi.max(f64::from(v));
    }
    lo = lo.min(sea_y) - 5.0;
    hi += 5.0;
    let py = |v: f64| ((hi - v) / (hi - lo) * (h - 1) as f64) as i64;
    let sea = py(sea_y);
    for x in 0..w {
        for y in sea.max(0)..h as i64 {
            put(&mut img, w, h, x as i64, y, [200, 225, 255], 1.0);
        }
    }
    for (row, rgb) in [(before, [120u8, 120, 120]), (after, [214, 39, 40])] {
        for i in 0..row.len() {
            let y0 = py(f64::from(row[i]));
            let y1 = py(f64::from(row[(i + 1).min(row.len() - 1)]));
            for x in 0..3 {
                let y = y0 + (y1 - y0) * x / 3;
                for t in -1..=1 {
                    put(&mut img, w, h, (i * 3) as i64 + x, y + t, rgb, 1.0);
                }
            }
        }
    }
    img
}

/// Writes the review pictures into `out`.
pub fn run(checkpoint: &Path, world_data: &Path, out: &Path) -> CliResult<Value> {
    let ck = Checkpoint::read(checkpoint)?;
    let world = &ck.world;
    let st: &VolcanoState = world
        .volcanoes
        .as_ref()
        .ok_or("the checkpoint's world has no volcano catalogue (a run without the stage?)")?;
    // The scale from the run's preset (so a `stage amplify --work` folder, which has no manifest,
    // serves as well as a baked world).
    let preset = deeptime_core::preset::Preset::from_json(&ck.preset_json)?;
    let deeptime_core::preset::PresetBody::Planet(pp) = &preset.body else {
        return Err("not a planet preset".into());
    };
    let s = &pp.scale;
    let layer = TilePackReader::open(&world_data.join("rasters/elevation.dtr"))?.load()?;
    let g = layer.desc.grid.clone();
    let ctx = Ctx {
        l3: &layer,
        gw: i64::from(g.w),
        gh: i64::from(g.h),
        r: f64::from(g.r),
        dx: f64::from(g.r) * s.m_per_block_h,
        c: f64::from(s.circumference_blocks),
        sea_y: f64::from(s.sea_level_y),
        mv: s.m_per_block_v,
        sea_m: st.sea_level_m,
    };
    std::fs::create_dir_all(out)?;

    // The global map, from the amplifier's thinned copy (block y).
    let prev = world
        .fields
        .get("amplify/elevation")
        .ok_or("no amplify/elevation field: run the amplify stage")?;
    let (pw, ph) = (prev.grid.w as usize, prev.grid.h as usize);
    let pr = f64::from(prev.grid.r);
    let mut map = shade(&prev.data, pw, ph, 1, &ctx, pr * s.m_per_block_h);
    let pole = deeptime_format::Projection::of(ctx.c).pole_z();
    let to_px = |x: f64, z: f64| ((x + ctx.c / 2.0) / pr, (z + pole) / pr);
    let mut settings = vec![0u8; pw * ph * 4];
    {
        // Settings: grey land/sea from the map, the arc flux of the last frame with arcs, plumes.
        for (k, px) in settings.chunks_mut(4).enumerate() {
            let land = f64::from(prev.data[k]) > ctx.sea_y;
            let v = if land { 150 } else { 60 };
            px.copy_from_slice(&[v, v, v + 10, 255]);
        }
        if let Some(h) = &world.history {
            let flux = (0..h.frame_count())
                .rev()
                .filter_map(|k| h.frame(k).ok())
                .map(|f| {
                    f.fields
                        .get("deeptime:arc_magma_flux")
                        .cloned()
                        .unwrap_or_default()
                })
                .find(|f| f.iter().any(|&x| x > 0.0));
            if let Some(f) = flux {
                let g1 = h.grid.clone();
                let (w1, r1) = (g1.w as usize, f64::from(g1.r));
                for j in 0..ph {
                    for i in 0..pw {
                        let x = -ctx.c / 2.0 + i as f64 * pr;
                        let z = -pole + j as f64 * pr;
                        let ci = ((x + ctx.c / 2.0) / r1).round() as usize % w1;
                        let cj = (((z + pole) / r1).round() as usize).min(g1.h as usize - 1);
                        let a = f64::from(f[cj * w1 + ci]).min(1.0);
                        if a > 0.0 {
                            put(
                                &mut settings,
                                pw,
                                ph,
                                i as i64,
                                j as i64,
                                [230, 30, 30],
                                0.3 + 0.7 * a,
                            );
                        }
                    }
                }
            }
        }
        if let Some(pl) = &world.plates {
            let frame = deeptime_core::sphere::BlockFrame::new(s.circumference_blocks);
            for hs in pl
                .hotspots
                .iter()
                .filter(|h| !h.closed || h.dies_myr >= st.t_myr)
            {
                let [vx, vy, vz] = hs.pos;
                let lat = vz.atan2(vx.hypot(vy)).to_degrees();
                let lon = vy.atan2(vx).to_degrees().rem_euclid(360.0);
                let (x, z) = (frame.x_of_lon(lon), frame.z_of_lat(lat));
                let (px, py) = to_px(x, z);
                for d in -4i64..=4 {
                    put(
                        &mut settings,
                        pw,
                        ph,
                        px as i64 + d,
                        py as i64,
                        [255, 255, 255],
                        1.0,
                    );
                    put(
                        &mut settings,
                        pw,
                        ph,
                        px as i64,
                        py as i64 + d,
                        [255, 255, 255],
                        1.0,
                    );
                }
            }
        }
    }
    for img in [&mut map, &mut settings] {
        for b in &st.basalts {
            let (px, py) = to_px(b.x, b.z);
            ring(
                img,
                pw,
                ph,
                px,
                py,
                b.radius_m / s.m_per_block_h / pr,
                [154, 138, 122],
            );
        }
        for v in st.volcanoes.iter().filter(|v| standing(v)) {
            let (px, py) = to_px(v.x, v.z);
            let r = (v.radius_m / s.m_per_block_h / pr * 0.35).clamp(1.5, 4.0);
            disc(img, pw, ph, px, py, r, colour(v));
        }
    }
    std::fs::write(
        out.join("map.png"),
        deeptime_viewer::encode_png(pw as u32, ph as u32, &map)?,
    )?;
    std::fs::write(
        out.join("settings.png"),
        deeptime_viewer::encode_png(pw as u32, ph as u32, &settings)?,
    )?;

    // Close views.
    let z2f = world
        .fields
        .get("young/elevation_m")
        .ok_or("no young/elevation_m field: run the young stage")?;
    let (w2, h2) = (z2f.grid.w as usize, z2f.grid.h as usize);
    let scale = i64::from(z2f.grid.r / g.r);
    let live: Vec<&Volcano> = st.volcanoes.iter().filter(|v| standing(v)).collect();
    let mut picks: Vec<(&'static str, String, Vec<&Volcano>, usize)> = Vec::new();
    // The summit's block height on L3 (the highest cell within two of its centre).
    let summit_y = |v: &Volcano| {
        let (ci, cj) = ctx.cell(v.x, v.z);
        let mut top = f32::NEG_INFINITY;
        for dj in -2..=2 {
            for di in -2..=2 {
                top = top.max(ctx.y(ci + di, cj + dj));
            }
        }
        f64::from(top)
    };
    let tallest = |f: &dyn Fn(&Volcano) -> bool| {
        live.iter()
            .copied()
            .filter(|v| f(v))
            .max_by(|a, b| a.height_m.total_cmp(&b.height_m))
    };
    // The highest summit among the live continental-arc cones.
    let highest = live
        .iter()
        .copied()
        .filter(|v| v.kind == Kind::Strato && v.active() && !v.oceanic)
        .max_by(|a, b| summit_y(a).total_cmp(&summit_y(b)));
    if let Some(v) = highest {
        picks.push((
            "arc",
            "The highest active continental-arc stratovolcano".into(),
            vec![v],
            192,
        ));
    }
    if let Some(v) =
        tallest(&|v| v.kind == Kind::Strato && v.active() && v.oceanic && summit_y(v) > ctx.sea_y)
    {
        picks.push((
            "island_arc",
            "The tallest active island-arc stratovolcano that is an island".into(),
            vec![v],
            192,
        ));
    }
    let mut chains: std::collections::BTreeMap<u32, Vec<&Volcano>> = Default::default();
    for v in live.iter().filter(|v| v.kind == Kind::Shield) {
        if let Some(h) = v.hotspot {
            chains.entry(h).or_default().push(v);
        }
    }
    // The chain with the most islands, then the most volcanoes standing.
    let islands = |vs: &Vec<&Volcano>| vs.iter().filter(|v| summit_y(v) > ctx.sea_y).count();
    if let Some((h, vs)) = chains
        .iter_mut()
        .max_by_key(|(_, vs)| (islands(vs), vs.len()))
    {
        // The youngest first: the close view's notes are about it.
        vs.sort_by(|a, b| b.born_myr.total_cmp(&a.born_myr));
        picks.push((
            "chain",
            format!(
                "The hotspot chain with the most islands (hotspot {h}: {} volcanoes standing, {} of them islands)",
                vs.len(),
                islands(vs)
            ),
            vs.clone(),
            384,
        ));
    }
    if let Some(v) = live
        .iter()
        .copied()
        .filter(|v| {
            v.kind != Kind::Shield && v.caldera.as_ref().is_some_and(|c| c.radius_m > 3000.0)
        })
        .max_by(|a, b| {
            // The youngest (a caldera's detail fades), then the largest.
            let key = |v: &Volcano| {
                let c = v
                    .caldera
                    .as_ref()
                    .map_or((0.0, 0.0), |c| (c.t_myr, c.radius_m));
                (c.0, c.1)
            };
            key(a)
                .partial_cmp(&key(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    {
        picks.push((
            "caldera",
            "The youngest large caldera still standing".into(),
            vec![v],
            192,
        ));
    }
    let caldera_id = picks.iter().find(|p| p.0 == "caldera").map(|p| p.2[0].id);
    if let Some(v) = tallest(&|v| v.kind == Kind::Silicic && Some(v.id) != caldera_id) {
        picks.push((
            "silicic",
            "The tallest continental hotspot centre".into(),
            vec![v],
            256,
        ));
    }
    let mut closes = Vec::new();
    for (id, title, vs, side) in picks {
        let (mut sx, mut sz, mut n) = (0.0, 0.0, 0.0);
        let frame = deeptime_core::sphere::BlockFrame::new(s.circumference_blocks);
        let x_ref = vs[0].x;
        // The first volcano and those near enough to share its window.
        let near = 0.35 * side as f64 * ctx.r;
        for v in vs
            .iter()
            .filter(|v| frame.wrap_x(v.x - x_ref).hypot(v.z - vs[0].z) <= near)
        {
            sx += x_ref + frame.wrap_x(v.x - x_ref);
            sz += v.z;
            n += 1.0;
        }
        let (cx, cz) = (sx / n, sz / n);
        let (ci, cj) = ctx.cell(cx, cz);
        let (i0, j0) = (
            ci - side as i64 / 2,
            (cj - side as i64 / 2).clamp(0, ctx.gh - side as i64),
        );
        let summit = ctx.cell(vs[0].x, vs[0].z);
        let v = vs[0];
        let note = format!(
            "{} at x {:.0}, z {:.0}; height {:.0} m (peak {:.0} m), radius {:.1} km, born {:.0} Myr, {}{}; window {:.0} km",
            v.kind.name(),
            v.x,
            v.z,
            v.height_m,
            v.peak_m,
            v.radius_m / 1000.0,
            v.born_myr,
            match v.extinct_myr {
                Some(e) => format!("dead since {e:.0} Myr"),
                None => "active".into(),
            },
            v.caldera
                .as_ref()
                .map(|c| format!(
                    ", caldera {:.1} km across, {:.0} m deep",
                    2.0 * c.radius_m / 1000.0,
                    c.depth_m
                ))
                .unwrap_or_default(),
            side as f64 * ctx.dx / 1000.0
        );
        let before = young_window(&z2f.data, w2, h2, scale, &ctx, i0, j0, side);
        let after = window(&ctx, i0, j0, side);
        // Relief over the ring just outside the foot, both terrains, blocks → m.
        let (si, sj) = (summit.0 - i0, summit.1 - j0);
        let rr = v.radius_m * 1.1 / ctx.dx;
        let relief = |z: &[f32]| {
            let (mut top, mut ring, mut k) = (f64::NEG_INFINITY, 0.0, 0.0);
            for j in 0..side as i64 {
                for i in 0..side as i64 {
                    let d = ((i - si) as f64).hypot((j - sj) as f64);
                    let y = f64::from(z[j as usize * side + i as usize]);
                    if d <= 2.5 {
                        top = top.max(y);
                    } else if (d - rr).abs() < 1.0 {
                        ring += y;
                        k += 1.0;
                    }
                }
            }
            let ring = if k > 0.0 { ring / k } else { f64::NAN };
            ((top - ring) * ctx.mv, (top - ctx.sea_y) * ctx.mv)
        };
        let (rb, _) = relief(&before);
        let (ra, summit_m) = relief(&after);
        // The same on the deep-time surface (L1) and the young landscape (L2), at their own cells.
        let coarse = |f: &deeptime_core::world::Field| {
            let g = &f.grid;
            let r = f64::from(g.r);
            let at = |x: f64, z: f64| {
                let fi = (x - g.origin_block_x as f64) / r;
                let fj = (z - g.origin_block_z as f64) / r;
                let (w, h) = (g.w as i64, g.h as i64);
                let (i, j) = (
                    fi.round() as i64,
                    fj.round().clamp(0.0, (h - 1) as f64) as i64,
                );
                f64::from(f.data[(j * w + i.rem_euclid(w)) as usize])
            };
            let frame = deeptime_core::sphere::BlockFrame::new(s.circumference_blocks);
            let top = at(v.x, v.z);
            let rr = v.radius_m * 1.1 / s.m_per_block_h;
            let ring: f64 = (0..24)
                .map(|k| {
                    let th = std::f64::consts::TAU * f64::from(k) / 24.0;
                    at(frame.wrap_x(v.x + rr * th.cos()), v.z + rr * th.sin())
                })
                .sum::<f64>()
                / 24.0;
            top - ring
        };
        let r1 = world
            .fields
            .get("surface/elevation_m")
            .map_or(f64::NAN, &coarse);
        let r2 = coarse(z2f);
        let note = format!(
            "{note}; relief over its foot: L1 surface {r1:.0} m, young L2 {r2:.0} m, L3 before {rb:.0} m, after {ra:.0} m; summit {summit_m:.0} m above the sea"
        );
        closes.push(Close {
            id,
            title,
            note,
            before,
            after,
            side,
            row: sj.clamp(0, side as i64 - 1) as usize,
        });
    }
    // The best-kept flood basalt.
    if let Some(fb) = world.fields.get("volcanoes/flood_basalt") {
        let g1 = &fb.grid;
        let best = st.basalts.iter().max_by_key(|b| {
            let mut n = 0usize;
            deeptime_core::surface::volcanoes::for_cells(
                g1,
                s.m_per_block_h,
                b.x,
                b.z,
                b.radius_m,
                |c, _, _| {
                    if fb.data[c] > 0.0 {
                        n += 1;
                    }
                },
            );
            n
        });
        if let Some(b) = best {
            let side = 384usize;
            let (ci, cj) = ctx.cell(b.x, b.z);
            let (i0, j0) = (
                ci - side as i64 / 2,
                (cj - side as i64 / 2).clamp(0, ctx.gh - side as i64),
            );
            closes.push(Close {
                id: "flood_basalt",
                title: "The best-kept flood-basalt plateau".into(),
                note: format!(
                    "flood basalt at x {:.0}, z {:.0}: {:.0} m pile in {} flows, radius {:.0} km, erupted {:.0} Myr; window {:.0} km",
                    b.x, b.z, b.thickness_m, b.flows, b.radius_m / 1000.0, b.t_myr, side as f64 * ctx.dx / 1000.0
                ),
                before: young_window(&z2f.data, w2, h2, scale, &ctx, i0, j0, side),
                after: window(&ctx, i0, j0, side),
                side,
                row: side / 2,
            });
        }
    }
    let mut html = String::from(
        "<!doctype html><meta charset=utf-8><title>Deep Time volcanoes</title>\
         <style>body{font:14px system-ui;background:#111;color:#ddd;margin:16px}img{max-width:100%;image-rendering:pixelated}\
         .pair{display:flex;gap:8px;flex-wrap:wrap}.pair div{flex:1;min-width:300px}h2{margin-top:32px}</style>\
         <h1>Volcanoes (P1-21)</h1>",
    );
    let _ = write!(
        html,
        "<p>{}</p><h2>The finished terrain with its volcanoes</h2><img src=map.png>\
         <p>Red: active stratovolcano; brown: dead stratovolcano; yellow: active shield; blue: dead shield or seamount; \
         purple: silicic centre; pink: caldera; grey rings: flood-basalt provinces.</p>\
         <h2>The settings that make them</h2><img src=settings.png>\
         <p>Red: arc magma flux of the last 10 Myr (the recorded history); white crosses: live hotspot plumes; markers as above.</p>",
        world
            .notes
            .get("volcanoes")
            .and_then(|n| serde_json::from_str::<Value>(n).ok())
            .map(|v| deeptime_core::surface::volcanoes::summary_line(&v))
            .unwrap_or_default()
    );
    let mut views = Vec::new();
    for cl in &closes {
        let k = if cl.side <= 192 { 3 } else { 2 };
        let b = shade(&cl.before, cl.side, cl.side, k, &ctx, ctx.dx);
        let a = shade(&cl.after, cl.side, cl.side, k, &ctx, ctx.dx);
        let px = (cl.side * k) as u32;
        std::fs::write(
            out.join(format!("{}-before.png", cl.id)),
            deeptime_viewer::encode_png(px, px, &b)?,
        )?;
        std::fs::write(
            out.join(format!("{}-after.png", cl.id)),
            deeptime_viewer::encode_png(px, px, &a)?,
        )?;
        let row = |v: &[f32]| v[cl.row * cl.side..(cl.row + 1) * cl.side].to_vec();
        let pimg = profile(&row(&cl.before), &row(&cl.after), ctx.sea_y);
        std::fs::write(
            out.join(format!("{}-profile.png", cl.id)),
            deeptime_viewer::encode_png((cl.side * 3) as u32, 180, &pimg)?,
        )?;
        let _ = write!(
            html,
            "<h2>{}</h2><p>{}</p><div class=pair><div><b>Before</b>: the young landscape (L2) interpolated<br>\
             <img src={id}-before.png></div><div><b>After</b>: the finished L3 terrain<br><img src={id}-after.png></div></div>\
             <p>East–west profile through the summit (grey before, red after; blue: the sea):</p><img src={id}-profile.png>",
            cl.title,
            cl.note,
            id = cl.id
        );
        views.push(json!({"id": cl.id, "title": cl.title, "note": cl.note}));
    }
    std::fs::write(out.join("index.html"), html)?;
    let list: Vec<Value> = st
        .volcanoes
        .iter()
        .chain(&st.gone)
        .map(|v| {
            json!({
                "id": v.id, "kind": v.kind.name(), "x": v.x.round(), "z": v.z.round(),
                "active": v.active(), "standing": v.gone.is_none() && standing(v),
                "height_m": v.height_m.round(), "peak_m": v.peak_m.round(),
                "radius_km": (v.radius_m / 100.0).round() / 10.0, "born_myr": v.born_myr,
                "extinct_myr": v.extinct_myr, "oceanic": v.oceanic, "silicic": v.silicic,
                "hotspot": v.hotspot, "chain": v.chain,
                "caldera_km": v.caldera.as_ref().map(|c| (c.radius_m / 100.0).round() / 10.0),
                "gone": v.gone.map(|g| format!("{g:?}")),
            })
        })
        .collect();
    let basalts: Vec<Value> = st
        .basalts
        .iter()
        .map(|b| json!({"id": b.id, "x": b.x.round(), "z": b.z.round(), "radius_km": b.radius_m / 1000.0, "t_myr": b.t_myr}))
        .collect();
    let doc = json!({"volcanoes": list, "flood_basalts": basalts, "views": views});
    std::fs::write(out.join("volcanoes.json"), serde_json::to_vec_pretty(&doc)?)?;
    Ok(json!({"views": views.len(), "volcanoes": st.volcanoes.len(), "gone": st.gone.len()}))
}
