// SPDX-License-Identifier: LGPL-3.0-or-later
//! Volcano shapes at the amplifier's grids (P1-21, `surface/volcanoes`).
//!
//! The deep-time surface carries each edifice at L1 (4.8 km cells at 16k quick), where a 12-km
//! cone is a smooth bump of a few cells, eroded like the land around it. From the catalogue the
//! volcano stage leaves (`WorldState::volcanoes`), the amplifier puts the shape back at the finer
//! grids, scaled by the height the surface preserved (its measured relief):
//!
//! 1. **Before the erosion passes**, on the young landscape's grid (L2): the coarse bump the
//!    young landscape holds of the edifice (the body blurred by the deep-time grid, about a tent
//!    1.5 L1 cells wide, scaled to the relief measured over its foot, never more than the volcano
//!    built) is taken away and the true profile put in its place, scaled by the share of that
//!    relief kept; the amplifier's erosion then cuts radial gullies into a real cone and its
//!    thermal pass slumps it to the rock's angle. The cone is laid on the ground from its vent
//!    (`onlap`, VOL-080): near the vent it buries the ground up to the vent's level plus its
//!    profile, so on a slope its summit is still the vent, blending back to lying on the ground
//!    towards its foot.
//! 2. **After the last flood**, on the finest grid (L3, in the sink the tiles pass through): the
//!    part of the profile L2 cannot hold (the sharp summit, the body less its L2 blur,
//!    `shape::blurred`) and the depression — a summit crater, or a caldera's flat floor inside a
//!    slightly irregular steep rim — which the flood would have filled; and the flood-basalt
//!    terraces: where a pile crops out
//!    (`volcanoes/flood_basalt`, written by the volcano stage), elevations are stepped into flat
//!    treads and steep risers of `flood_basalt_step_m`.
//!
//! A dead volcano's detail fades: its crater and sharp summit with `fade_myr`, its cone's shape
//! three times slower (the finest landforms erode first; the L1 surface erodes the rest). The
//! thinned preview the stage keeps gets the same last-grid shaping, so the keyframes show it.
//!
//! The mountain character (`character.rs`) leaves the edifices alone where it would fight them:
//! no fold ridges over a volcano's body (`fold_keep`), and on an active or young one no unsharp
//! mask — no young-peak sharpening, no rounding — and soft gullies (`youth_keep`; Ben,
//! 2026-09-30, "Smooth young, rugged old"), so cones stay clean and shields domed; it returns on
//! the crater's clock as a dead volcano erodes.

use deeptime_format::GridDesc;

use super::resample::Level;
use super::tiles::{TileOut, TileSink};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::surface::volcanoes::shape::{self, Radial};
use crate::surface::volcanoes::{self as vol, VolcanoState};
use crate::world::WorldState;

/// One volcano's residuals.
#[derive(Clone, Debug)]
struct Item {
    x: f64,
    z: f64,
    radius_m: f64,
    /// Metres at ρ, for the young grid (before erosion): the cone put in (`tp`) and the young
    /// landscape's coarse bump of it taken out (`rem`) …
    tp: Radial,
    rem: Radial,
    /// … the relief kept (m: the cone's height, the most its onlap fills) and the ground at the
    /// vent without the volcano (m; read from the young landscape when there is one) …
    at: f64,
    vent_ground: Option<f64>,
    /// … and for the finest grid (after the flood).
    l3: Radial,
    /// A caldera: its rim radius (ρ), the level of its flat floor (m) and the phases of its rim's
    /// irregularity.
    caldera: Option<(f64, f64, f64, f64)>,
    /// How young it is for the mountain character: 1 while active, fading with `fade_myr` once
    /// dead (the crater's clock).
    young: f64,
}

/// The shapes a run's volcanoes add to the amplifier's grids.
#[derive(Clone, Debug, Default)]
pub struct Shaper {
    items: Vec<Item>,
    /// The flood-basalt outcrop on L1 (0–1) and its grid; the terrace step, m.
    terraces: Option<(GridDesc, Vec<f32>)>,
    step_m: f64,
    sea: f64,
    mh: f64,
    circumference: u32,
}

impl Shaper {
    /// The world's volcanoes as seen by a preset's grids; `None` without a volcano stage.
    pub fn from_world(world: &WorldState, p: &PlanetPreset) -> Option<Shaper> {
        let st: &VolcanoState = world.volcanoes.as_ref()?;
        let params = p.volcanoes.clone().unwrap_or_default();
        let mh = p.scale.m_per_block_h;
        let dx1 = f64::from(p.resolution.l1_r) * mh;
        let dx2 = f64::from(p.resolution.l2_r) * mh;
        let fade = st.fade_myr.max(1e-3);
        let young = world.fields.get(crate::surface::young::field::ELEVATION_M);
        let frame = crate::sphere::BlockFrame::new(p.scale.circumference_blocks);
        let items = st
            .volcanoes
            .iter()
            // ledger: VOL-064
            .filter(|v| v.height_m >= 30.0)
            .map(|v| {
                let age = v.extinct_myr.map_or(0.0, |e| (st.t_myr - e).max(0.0));
                let sharp = (-age / (3.0 * fade)).exp(); // ledger: VOL-065
                let dep = (-age / fade).exp();
                // What the volcano stage says the edifice stands (never more than it was built
                // to), and the shape it has.
                let a = v.height_m.min(1.25 * v.target_m); // ledger: VOL-064
                let s = v.shape();
                let body = |r: f64| s.body(r);
                let r = v.radius_m;
                // The bump the young landscape holds of it: the body blurred by the deep-time
                // grid (cell-averaged rendering, bilinear advection, the interpolation to L2:
                // about a tent 1.5 L1 cells wide), and its measured relief there.
                let coarse = shape::blurred(&body, 1.5 * dx1 / r); // ledger: VOL-063
                let e0 = coarse.at(0.0).max(1e-3);
                let expected = a * e0;
                let top = young.map(|f| sample(f, v.x, v.z));
                let held = young.zip(top).map_or(expected, |(f, top)| {
                    let rr = 1.1 * r / mh;
                    let ring: f64 = (0..24)
                        .map(|k| {
                            let th = std::f64::consts::TAU * f64::from(k) / 24.0;
                            sample(f, frame.wrap_x(v.x + rr * th.cos()), v.z + rr * th.sin())
                        })
                        .sum::<f64>()
                        / 24.0;
                    top - ring
                });
                let removed = held.clamp(0.0, expected);
                // The ground the vent stands on: the young landscape there less the bump it holds
                // of the edifice.
                let vent_ground = top.map(|t| t - removed);
                // The cone it becomes: the relief kept, the true profile (a dead one keeps more of
                // the blurred shape as it ages).
                let at = a * (removed / expected);
                let fine = shape::blurred(&body, dx2 / r);
                let reach = coarse.reach().max(fine.reach()).max(1.05);
                let tp = table(reach, |rho| {
                    at * (sharp * body(rho) + (1.0 - sharp) * coarse.at(rho))
                });
                let rem = table(reach, |rho| removed * coarse.at(rho) / e0);
                let is_caldera = s.depression.is_some_and(|d| d.caldera);
                let l3 = table(reach, |rho| {
                    let bowl = if is_caldera { 0.0 } else { s.bowl(rho) };
                    at * sharp * (body(rho) - fine.at(rho)) - bowl * at * dep
                });
                // A caldera's floor: flat, its depth below the mean level of its rim, the rim read
                // from the young landscape with the young-grid change (the same for every tile).
                let caldera = s.depression.filter(|d| d.caldera).and_then(|d| {
                    let f = young?;
                    let rim_m = d.rho * r;
                    let mut sum = 0.0;
                    for k in 0..32 {
                        let th = std::f64::consts::TAU * f64::from(k) / 32.0;
                        let bx = frame.wrap_x(v.x + rim_m * th.cos() / mh);
                        let z = sample(f, bx, v.z + rim_m * th.sin() / mh);
                        sum += onlap(z, d.rho, tp.at(d.rho), rem.at(d.rho), at, vent_ground);
                    }
                    let rim = sum / 32.0;
                    let h = (v.id as f64 * 0.618_033_988_7).fract() * std::f64::consts::TAU;
                    Some((d.rho, rim - d.depth * at * sharp, h, 2.3 * h + 1.0))
                });
                Item {
                    x: v.x,
                    z: v.z,
                    radius_m: r,
                    tp,
                    rem,
                    at,
                    vent_ground,
                    l3,
                    caldera,
                    young: dep,
                }
            })
            .collect();
        let terraces = world
            .fields
            .get(vol::field::FLOOD_BASALT)
            .filter(|f| f.data.iter().any(|&x| x > 0.0))
            .map(|f| (f.grid.clone(), f.data.clone()));
        Some(Shaper {
            items,
            terraces,
            step_m: params.flood_basalt_step_m,
            sea: st.sea_level_m,
            mh,
            circumference: p.scale.circumference_blocks,
        })
    }

    /// Volcanoes shaped.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Per L1 cell of `grid` (row-major), how much of the mountain character's fold ridges to keep:
    /// 0 over a shaped volcano's body, rising to 1 at 1.3 radii.
    pub fn fold_keep(&self, grid: &GridDesc) -> Vec<f32> {
        let mut keep = vec![1.0f32; grid.cells()];
        for it in &self.items {
            vol::for_cells(grid, self.mh, it.x, it.z, 1.3 * it.radius_m, |c, dx, dz| {
                let rho = dx.hypot(dz) / it.radius_m;
                let k = smooth(0.9, 1.3, rho) as f32; // ledger: VOL-067
                keep[c] = keep[c].min(k);
            });
        }
        keep
    }

    /// Per L1 cell of `grid` (row-major), how much of the mountain character's unsharp mask (its
    /// young-peak sharpening) and gully youth to keep: 1 − the volcano's youth over its body,
    /// rising back to 1 at 1.3 radii; an active volcano keeps none, a dead one regains it on the
    /// crater's clock.
    pub fn youth_keep(&self, grid: &GridDesc) -> Vec<f32> {
        let mut keep = vec![1.0f32; grid.cells()];
        for it in self.items.iter().filter(|it| it.young > 0.01) {
            vol::for_cells(grid, self.mh, it.x, it.z, 1.3 * it.radius_m, |c, dx, dz| {
                let inside = 1.0 - smooth(0.9, 1.3, dx.hypot(dz) / it.radius_m); // ledger: VOL-068
                let k = (1.0 - it.young * inside) as f32;
                keep[c] = keep[c].min(k);
            });
        }
        keep
    }

    /// Puts the cones into the ladder's first level (the L2 terrain), on the ground as [`onlap`]
    /// lays them. Returns the cells changed.
    pub fn sharpen_level(&self, level: &mut Level, grid: &GridDesc) -> usize {
        // Every vent's ground is read before any cone goes in, so where two overlap the result
        // does not depend on their order.
        let vents: Vec<f64> = self
            .items
            .iter()
            .map(|it| {
                it.vent_ground.unwrap_or_else(|| {
                    let r = f64::from(grid.r);
                    let fi = (it.x - grid.origin_block_x as f64) / r;
                    let fj = (it.z - grid.origin_block_z as f64) / r;
                    let z = crate::surface::advect::bilinear(level.w, level.h, &level.z, fi, fj);
                    f64::from(z) - it.rem.at(0.0)
                })
            })
            .collect();
        let mut changed = 0;
        for (it, &gv) in self.items.iter().zip(&vents) {
            vol::for_cells(
                grid,
                self.mh,
                it.x,
                it.z,
                it.tp.reach() * it.radius_m,
                |c, dx, dz| {
                    let rho = dx.hypot(dz) / it.radius_m;
                    let z = f64::from(level.z[c]);
                    let v = onlap(z, rho, it.tp.at(rho), it.rem.at(rho), it.at, Some(gv)) as f32;
                    if v != level.z[c] {
                        level.z[c] = v;
                        changed += 1;
                    }
                },
            );
        }
        changed
    }

    /// The last-grid change at block (bx, bz) of a cell of terrain `z` (m): the volcanoes' sharp
    /// summits and depressions, then the flood-basalt steps.
    fn fine_at(&self, near: &[&Item], bx: f64, bz: f64, z: f32) -> f32 {
        let frame = crate::sphere::BlockFrame::new(self.circumference);
        let mut out = f64::from(z);
        for it in near {
            let dx = frame.wrap_x(bx - it.x) * self.mh;
            let dz = (bz - it.z) * self.mh;
            let rho = dx.hypot(dz) / it.radius_m;
            out += it.l3.at(rho);
        }
        // Calderas: a flat floor inside a slightly irregular, steep-walled rim (only ever lowers).
        for it in near {
            let Some((rc, floor, p2, p3)) = it.caldera else {
                continue;
            };
            let dx = frame.wrap_x(bx - it.x) * self.mh;
            let dz = (bz - it.z) * self.mh;
            let th = dz.atan2(dx);
            let rim = rc * (1.0 + 0.1 * (2.0 * th + p2).sin() + 0.05 * (3.0 * th + p3).sin()); // ledger: VOL-066
            let x = dx.hypot(dz) / it.radius_m / rim;
            if x < 1.0 {
                let inside = 1.0 - smooth(0.78, 1.0, x); // ledger: VOL-066
                out = out.min(out + (floor - out) * inside);
            }
        }
        if let Some((g, w)) = &self.terraces
            && self.step_m > 0.0
        {
            let r = f64::from(g.r);
            let fi = (bx - g.origin_block_x as f64) / r;
            let fj = (bz - g.origin_block_z as f64) / r;
            let wgt = f64::from(crate::surface::advect::bilinear(
                g.w as usize,
                g.h as usize,
                w,
                fi,
                fj,
            ));
            if wgt > 0.0 && out > self.sea + self.step_m {
                let q = (out - self.sea) / self.step_m;
                let k = q.floor();
                let f = q - k;
                // Treads over 70 % of each step, a steep riser over the rest.
                let stepped = self.sea + self.step_m * (k + smooth(0.7, 1.0, f)); // ledger: VOL-036
                out += wgt.min(1.0) * (stepped - out);
            }
        }
        out as f32
    }

    /// The items whose reach overlaps the block rectangle [x0, x1] × [z0, z1] (x wraps).
    fn near(&self, x0: f64, x1: f64, z0: f64, z1: f64) -> Vec<&Item> {
        let frame = crate::sphere::BlockFrame::new(self.circumference);
        let (cx, hw) = (0.5 * (x0 + x1), 0.5 * (x1 - x0));
        let (cz, hh) = (0.5 * (z0 + z1), 0.5 * (z1 - z0));
        self.items
            .iter()
            .filter(|it| {
                let reach = it.l3.reach().max(1.2) * it.radius_m / self.mh;
                frame.wrap_x(it.x - cx).abs() <= hw + reach && (it.z - cz).abs() <= hh + reach
            })
            .collect()
    }

    /// Applies the last-grid shaping to a row-major block of cells of `grid` starting at cell
    /// (x0, y0), `w` wide, every `stride`-th cell of the grid.
    pub fn shape_block(
        &self,
        grid: &GridDesc,
        x0: usize,
        y0: usize,
        w: usize,
        stride: usize,
        z: &mut [f32],
    ) {
        if self.items.is_empty() && self.terraces.is_none() {
            return;
        }
        let r = f64::from(grid.r);
        let bx = |i: usize| grid.origin_block_x as f64 + ((x0 + i * stride) as f64) * r;
        let bz = |j: usize| grid.origin_block_z as f64 + ((y0 + j * stride) as f64) * r;
        // Bands of rows in parallel; within a band, blocks of columns each with the volcanoes
        // that reach them.
        const B: usize = 64;
        use rayon::prelude::*;
        z.par_chunks_mut(w * B)
            .enumerate()
            .for_each(|(band, rows)| {
                let j0 = band * B;
                let nrows = rows.len() / w;
                for i0 in (0..w).step_by(B) {
                    let i1 = (i0 + B).min(w);
                    let near = self.near(bx(i0), bx(i1 - 1), bz(j0), bz(j0 + nrows - 1));
                    if near.is_empty() && self.terraces.is_none() {
                        continue;
                    }
                    for dj in 0..nrows {
                        let zb = bz(j0 + dj);
                        for i in i0..i1 {
                            let v = &mut rows[dj * w + i];
                            *v = self.fine_at(&near, bx(i), zb, *v);
                        }
                    }
                }
            });
    }
}

/// Bilinear value of a field at block (x, z).
fn sample(f: &crate::world::Field, x: f64, z: f64) -> f64 {
    let g = &f.grid;
    let fi = (x - g.origin_block_x as f64) / f64::from(g.r);
    let fj = (z - g.origin_block_z as f64) / f64::from(g.r);
    f64::from(crate::surface::advect::bilinear(
        g.w as usize,
        g.h as usize,
        &f.data,
        fi,
        fj,
    ))
}

/// Inside this ρ a cone lies wholly on its vent's ground level; from here to its foot it blends
/// back to lying on the ground beneath it.
const ONLAP_INNER: f64 = 0.4; // ledger: VOL-080 (game-choice)

/// The young-grid terrain at ρ after a cone is put in: `z` is the young landscape there (m),
/// holding `rem` of the edifice as a coarse bump; `tp` the cone's profile there (m); `at` its
/// height; `vent` the ground at its vent without it (m; `None`: the cone simply added, as before
/// VOL-080).
///
/// The cone is built from its vent, so near the vent it buries the ground up to the vent's
/// ground level plus its profile, and its summit is the vent wherever it stands: on a slope it
/// fills the downhill side and the uphill ground shows through above its flank. Towards the foot
/// it blends back to lying on the ground (no wall where the ground falls away), and the fill is
/// never more than the cone's own height (a cone on a cliff edge does not pave the sea). On level
/// ground it is the cone added to the ground, as before.
fn onlap(z: f64, rho: f64, tp: f64, rem: f64, at: f64, vent: Option<f64>) -> f64 {
    let g = z - rem;
    let Some(gv) = vent else {
        return g + tp;
    };
    let w = 1.0 - smooth(ONLAP_INNER, 1.0, rho); // ledger: VOL-080 (game-choice)
    let fill = (w * (gv - g)).min(at); // ledger: VOL-081 (game-choice)
    g + (tp + fill).max(0.0)
}

/// A radial table of `f` out to ρ = `reach`.
fn table(reach: f64, f: impl Fn(f64) -> f64) -> Radial {
    const N: usize = 128;
    let step = reach / (N - 1) as f64;
    Radial {
        step,
        v: (0..N).map(|k| f(k as f64 * step) as f32).collect(),
    }
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A sink that adds the last-grid shaping to every tile before passing it on.
pub struct Sink<'a> {
    pub inner: &'a mut dyn TileSink,
    pub shaper: &'a Shaper,
    pub grid: GridDesc,
}

impl TileSink for Sink<'_> {
    fn put(&mut self, t: &TileOut<'_>) -> Result<(), CoreError> {
        let (x0, y0, w, _) = t.rect;
        let mut z = t.z_m.to_vec();
        self.shaper.shape_block(&self.grid, x0, y0, w, 1, &mut z);
        self.inner.put(&TileOut {
            rect: t.rect,
            z_m: &z,
            amp_blocks: t.amp_blocks,
            level: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::volcanoes::{Depression, Kind, Volcano};
    use crate::world::Field;

    fn strato(x: f64, z: f64, extinct: Option<f64>) -> Volcano {
        Volcano {
            id: 0,
            kind: Kind::Strato,
            x,
            z,
            plate: 0,
            oceanic: false,
            silicic: false,
            hotspot: None,
            chain: 0,
            born_myr: -5.0,
            dies_myr: 1.0,
            extinct_myr: extinct,
            radius_m: 14_000.0,
            height_m: 2500.0,
            peak_m: 2500.0,
            built_m: 2500.0,
            sunk_m: 0.0,
            kept_m: 2500.0,
            target_m: 2500.0,
            flux_last: 1.0,
            missed: 0,
            flux_myr: 0.0,
            active_myr: 0.0,
            depression: Some(Depression {
                rho: 0.05,
                depth: 0.1,
                caldera: false,
            }),
            caldera: None,
            cause: None,
            segment: 0,
            gone: None,
        }
    }

    fn world_with(vs: Vec<Volcano>) -> (WorldState, PlanetPreset) {
        let json = crate::preset::with_timeline(
            crate::params::planet_builtin("pipeline_quick_16k").unwrap(),
            Some("scattered_continents"),
            Some(10.0),
        )
        .unwrap();
        let preset = match crate::preset::Preset::from_json(&json).unwrap().body {
            crate::preset::PresetBody::Planet(p) => (*p).clone(),
            _ => unreachable!(),
        };
        let mut w = WorldState::new(1, 16_384, 0.0);
        w.volcanoes = Some(VolcanoState {
            volcanoes: vs,
            t_myr: 0.0,
            fade_myr: 5.0,
            sea_level_m: 0.0,
            ..VolcanoState::default()
        });
        (w, preset)
    }

    #[test]
    fn a_cone_is_sharpened_and_its_crater_cut() {
        let (world, preset) = world_with(vec![strato(0.0, 0.0, None)]);
        let sh = Shaper::from_world(&world, &preset).unwrap();
        assert_eq!(sh.len(), 1);
        // L2 (8 blocks, 2.4 km): level ground at 100 m holding the deep-time edifice (as the
        // young landscape does); the band raises the summit and lowers the skirt a little; far
        // away nothing changes; the volume stays about the same.
        let g2 = GridDesc::for_planet("L2", 16_384, 8).unwrap();
        let n = g2.cells();
        let mut level = Level {
            w: g2.w as usize,
            h: g2.h as usize,
            dx: 2400.0,
            z: vec![100.0; n],
            area: vec![1.0; n],
            wgt: vec![1.0; n],
            kfac: vec![1.0; n],
            lake: vec![0; n],
            zs: Vec::new(),
        };
        let it = &sh.items[0];
        let mh = preset.scale.m_per_block_h;
        vol::for_cells(
            &g2,
            mh,
            it.x,
            it.z,
            it.rem.reach() * it.radius_m,
            |c, dx, dz| {
                level.z[c] += it.rem.at(dx.hypot(dz) / it.radius_m) as f32;
            },
        );
        let before = level.z.clone();
        assert!(sh.sharpen_level(&mut level, &g2) > 0);
        let at = |g: &GridDesc, z: &[f32], bx: f64, bz: f64| {
            z[crate::surface::volcanoes::cell_of(g, bx, bz)]
        };
        let top = at(&g2, &level.z, 0.0, 0.0);
        assert!(top > at(&g2, &before, 0.0, 0.0) + 50.0, "{top}");
        assert_eq!(at(&g2, &level.z, 1000.0, 0.0), 100.0);
        let dv: f64 = level
            .z
            .iter()
            .zip(&before)
            .map(|(&z, &b)| f64::from(z) - f64::from(b))
            .sum();
        assert!(dv.abs() < 0.2 * 2500.0, "volume change {dv} m·cells");
        // L3 (2 blocks, 600 m): the crater: the centre below the rim; the summit sharper.
        let g3 = GridDesc::for_planet("L3", 16_384, 2).unwrap();
        let (w3, x0, y0) = (64usize, (g3.w / 2 - 32) as usize, (g3.h / 2 - 32) as usize);
        let mut z = vec![0.0f32; w3 * w3];
        sh.shape_block(&g3, x0, y0, w3, 1, &mut z);
        let cell = |bx: f64, bz: f64| {
            let i = ((bx - g3.origin_block_x as f64) / 2.0).round() as usize - x0;
            let j = ((bz - g3.origin_block_z as f64) / 2.0).round() as usize - y0;
            z[j * w3 + i]
        };
        let centre = cell(0.0, 0.0);
        let rim = cell(2.0, 0.0).max(cell(4.0, 0.0));
        assert!(
            centre < rim,
            "crater floor {centre} not below the rim {rim}"
        );
        assert!(rim > 0.0, "the sharp summit adds height");
    }

    #[test]
    fn onlap_is_the_old_sum_on_level_ground_and_lets_high_ground_show() {
        // Level ground: the cone added, at every ρ.
        for rho in [0.0f64, 0.3, 0.7, 1.0, 1.2] {
            let (tp, rem) = (1000.0 * (1.0 - rho).max(0.0), 50.0);
            let added = 300.0 - rem + tp;
            let v = onlap(300.0, rho, tp, rem, 2000.0, Some(250.0));
            assert!((v - added).abs() < 1e-9, "ρ {rho}: {v} vs {added}");
            assert_eq!(onlap(300.0, rho, tp, rem, 2000.0, None), added);
        }
        // Near the vent the cone stands on the vent's ground: 400 m lower ground is filled …
        assert!((onlap(-400.0, 0.2, 900.0, 0.0, 2000.0, Some(0.0)) - 900.0).abs() < 1e-9);
        // … higher ground than its envelope shows through …
        assert_eq!(onlap(1500.0, 0.3, 700.0, 0.0, 2000.0, Some(0.0)), 1500.0);
        // … and the fill is never more than the cone's height.
        assert_eq!(onlap(-5000.0, 0.0, 1000.0, 0.0, 1000.0, Some(0.0)), -3000.0);
        // At the foot it lies on the ground again.
        assert_eq!(onlap(-400.0, 1.0, 0.0, 0.0, 2000.0, Some(0.0)), -400.0);
    }

    #[test]
    fn a_cone_on_a_steep_slope_has_its_summit_at_the_vent() {
        // A cone 2.5 km high, 14 km in radius, on ground rising 1.5 times its height across its
        // radius (east is up), as Ben's volcano 27476 stands on its range's flank.
        let v = strato(0.0, 0.0, None);
        let (h, r) = (v.height_m, v.radius_m);
        let slope = 1.5 * h / r;
        let (mut world, preset) = world_with(vec![v.clone()]);
        let mh = preset.scale.m_per_block_h;
        let g2 = GridDesc::for_planet("L2", 16_384, preset.resolution.l2_r).unwrap();
        // The young landscape holds the plane and the edifice blurred by the deep-time grid.
        let dx1 = f64::from(preset.resolution.l1_r) * mh;
        let s = v.shape();
        let coarse = shape::blurred(&|q| s.body(q), 1.5 * dx1 / r);
        let frame = crate::sphere::BlockFrame::new(16_384);
        let rr = f64::from(g2.r);
        let mut young = Field::filled(g2.clone(), 0.0);
        let mut plane = vec![0.0f32; g2.cells()];
        for j in 0..g2.h as usize {
            for i in 0..g2.w as usize {
                let bx = frame.wrap_x(g2.origin_block_x as f64 + i as f64 * rr);
                let bz = g2.origin_block_z as f64 + j as f64 * rr;
                let (xm, zm) = (bx * mh, bz * mh);
                let base = 500.0 + slope * xm.clamp(-3.0 * r, 3.0 * r);
                let c = j * g2.w as usize + i;
                plane[c] = base as f32;
                young.data[c] = (base + h * coarse.at(xm.hypot(zm) / r)) as f32;
            }
        }
        world.fields.insert(
            crate::surface::young::field::ELEVATION_M.to_owned(),
            young.clone(),
        );
        let sh = Shaper::from_world(&world, &preset).unwrap();
        let n = g2.cells();
        let mut level = Level {
            w: g2.w as usize,
            h: g2.h as usize,
            dx: rr * mh,
            z: young.data.clone(),
            area: vec![1.0; n],
            wgt: vec![1.0; n],
            kfac: vec![1.0; n],
            lake: vec![0; n],
            zs: Vec::new(),
        };
        sh.sharpen_level(&mut level, &g2);
        let vent = crate::surface::volcanoes::cell_of(&g2, 0.0, 0.0);
        // The highest cell within 0.4 radii is the vent's (the old sum put it up the slope).
        let mut best = (f32::MIN, usize::MAX, 0.0);
        let mut old_best = (f64::MIN, 0.0);
        let it = &sh.items[0];
        vol::for_cells(&g2, mh, 0.0, 0.0, 0.4 * r, |c, dx, dz| {
            let rho = dx.hypot(dz) / r;
            if level.z[c] > best.0 {
                best = (level.z[c], c, rho);
            }
            let old = onlap(
                f64::from(young.data[c]),
                rho,
                it.tp.at(rho),
                it.rem.at(rho),
                it.at,
                None,
            );
            if old > old_best.0 {
                old_best = (old, rho);
            }
        });
        assert_eq!(best.1, vent, "summit at ρ {:.2}, not the vent", best.2);
        assert!(
            old_best.1 > 0.2,
            "the old sum's summit at ρ {:.2}",
            old_best.1
        );
        // The summit stands the cone's kept height above the vent's ground.
        let top = f64::from(level.z[vent]) - f64::from(plane[vent]);
        assert!(
            top > 0.5 * h && top < 1.05 * h,
            "summit {top} m above the ground"
        );
        // Uphill beyond the envelope the ground is the plane (no cone on it, and of the coarse
        // bump only the share the ring at 1.1 radii holds, which the held edifice leaves out:
        // about half a percent of the height here); far away nothing changes; the fill is never
        // more than the cone's height.
        let up = crate::surface::volcanoes::cell_of(&g2, 0.7 * r / mh, 0.0);
        let left = f64::from(level.z[up] - plane[up]);
        assert!(left.abs() < 0.01 * h, "{} vs {}", level.z[up], plane[up]);
        let far = crate::surface::volcanoes::cell_of(&g2, 3.0 * r / mh, 0.0);
        assert_eq!(level.z[far], young.data[far]);
        assert_eq!(level.z.len(), n);
        for (&z, &p) in level.z.iter().zip(&plane) {
            assert!(f64::from(z - p) <= 2.0 * h + 1.0);
        }
    }

    #[test]
    fn young_volcanoes_get_no_young_peak_sharpening_and_old_ones_regain_it() {
        // An active cone, one dead 5 Myr (one fade time) and one dead 30 Myr, far apart.
        let (world, preset) = world_with(vec![
            strato(0.0, 0.0, None),
            strato(2000.0, 0.0, Some(-5.0)),
            strato(4000.0, 0.0, Some(-30.0)),
        ]);
        let sh = Shaper::from_world(&world, &preset).unwrap();
        let g1 = GridDesc::for_planet("L1", 16_384, 16).unwrap();
        let keep = sh.youth_keep(&g1);
        let at = |bx: f64| keep[crate::surface::volcanoes::cell_of(&g1, bx, 0.0)];
        assert!(at(0.0) < 0.01, "active: {}", at(0.0));
        let dead5 = at(2000.0);
        assert!(
            (dead5 - (1.0 - (-1.0f32).exp())).abs() < 0.05,
            "5 Myr dead: {dead5}"
        );
        assert!(at(4000.0) > 0.99, "30 Myr dead: {}", at(4000.0));
        assert_eq!(at(1000.0), 1.0, "between the cones");
    }

    #[test]
    fn a_dead_volcano_loses_its_crater_first_and_basalts_step() {
        let (mut world, preset) = world_with(vec![strato(0.0, 0.0, Some(-30.0))]);
        let sh = Shaper::from_world(&world, &preset).unwrap();
        let g3 = GridDesc::for_planet("L3", 16_384, 2).unwrap();
        let (w3, x0, y0) = (16usize, (g3.w / 2 - 8) as usize, (g3.h / 2 - 8) as usize);
        let mut z = vec![0.0f32; w3 * w3];
        sh.shape_block(&g3, x0, y0, w3, 1, &mut z);
        let centre = z[8 * w3 + 8];
        let rim = z[8 * w3 + 9];
        // Its crater has faded (e^-6 of it left): a few metres of the flat rim top's curvature.
        assert!(
            rim - centre < 10.0,
            "a 30-Myr-dead crater: {centre} vs {rim}"
        );
        // Flood-basalt terraces: a ramp becomes treads and risers.
        let g1 = GridDesc::for_planet("L1", 16_384, 16).unwrap();
        world
            .fields
            .insert(vol::field::FLOOD_BASALT.to_owned(), Field::filled(g1, 1.0));
        world.volcanoes.as_mut().unwrap().volcanoes.clear();
        let sh = Shaper::from_world(&world, &preset).unwrap();
        let mut ramp: Vec<f32> = (0..64).map(|i| 200.0 + 10.0 * i as f32).collect();
        sh.shape_block(&g3, 0, 100, 64, 1, &mut ramp);
        let flats = ramp
            .windows(2)
            .filter(|p| (p[1] - p[0]).abs() < 0.5)
            .count();
        let steep = ramp.windows(2).filter(|p| p[1] - p[0] > 15.0).count();
        assert!(
            flats >= 30 && steep >= 5,
            "flats {flats} steep {steep}: {ramp:?}"
        );
        assert!(ramp.windows(2).all(|p| p[1] >= p[0] - 1e-3), "monotone");
    }
}
