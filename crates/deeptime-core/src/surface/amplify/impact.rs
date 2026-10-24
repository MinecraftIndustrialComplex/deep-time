// SPDX-License-Identifier: LGPL-3.0-or-later
//! Impact craters at the amplifier's finest grid (P2-08, `surface/impacts`).
//!
//! The deep-time surface holds a crater only when it is at least two L1 cells across (9.6 km at
//! 16k quick), and then only as a smooth dimple that the surface stage erodes and fills like any
//! basin. From the catalogue the impact stage leaves (`WorldState::impacts`), the amplifier adds
//! the rest on the last grid (L3), after the last flood, which would fill a bowl nobody declared
//! a lake (the volcanoes' craters are cut at the same point, `volcano.rs`):
//!
//! - a crater **too small for L1** gets its whole profile: bowl, rim and ejecta;
//! - a crater **cut into L1** gets the part of its profile L1 cannot hold: the profile less the
//!   profile blurred by the deep-time grid (a tent 1.5 L1 cells wide, as for the volcanoes): the
//!   sharp rim crest, the steep walls, the flat floor, the central peak or peak ring.
//!
//! Neither is a fresh bowl stamped on old ground. The profile is the crater's **aged** one
//! (`impacts/shape.rs`: diffusion since the impact has rounded the rim and shallowed the bowl,
//! small craters first), scaled by the share of its relief the surface stage's erosion and
//! deposition at that place have left it (`Crater::kept`): a crater in an eroding range or under
//! a growing delta fades to nothing, and one on a quiet craton keeps a subdued bowl. The thinned
//! preview the stage keeps gets the same shaping, so the keyframes show the craters.
//!
//! A crater's distances are the planet's (`sphere::Around`, ledger SCL-230): its bowl and ejecta
//! are round on the planet, so round in the world, at every latitude (a circle in Lambert blocks
//! until 2026-10-23, which the world drew 3× long north–south at 55°).

use deeptime_format::GridDesc;

use super::tiles::{TileOut, TileSink};
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::sphere::{Around, BlockFrame};
use crate::surface::impacts::{Crater, ImpactState};
use crate::surface::volcanoes::shape::Radial;
use crate::world::WorldState;

/// A change smaller than this everywhere is not drawn, m.
const MIN_CHANGE_M: f64 = 2.0;

/// One crater's change of the last grid.
#[derive(Clone, Debug)]
struct Item {
    /// The planet's metric about its centre.
    at: Around,
    /// The Lambert box its shape reaches: half-width in x and the z range, blocks.
    hw: f64,
    z0: f64,
    z1: f64,
    /// Where its ejecta end, m.
    extent_m: f64,
    /// Metres at ρ = r / extent.
    table: Radial,
}

/// The shapes a run's craters add to the amplifier's last grid.
#[derive(Clone, Debug, Default)]
pub struct Shaper {
    items: Vec<Item>,
    circumference: u32,
}

/// What the last grid adds for one crater, as a table over ρ = r / extent: its aged profile (less
/// what L1 holds of it, when it is cut into L1) times the share of its relief it has kept.
/// `l1_cell_m` is the L1 cell size.
pub fn residual(c: &Crater, l1_cell_m: f64) -> Radial {
    const N: usize = 192;
    let kept = c.kept();
    let spread = c.spread_m();
    let fine = c.profile.aged(spread);
    let coarse = c
        .imprinted
        .then(|| c.profile.aged(spread.hypot(1.5 * l1_cell_m)));
    let reach = coarse
        .as_ref()
        .map_or(fine.reach(), |k| k.reach().max(fine.reach()));
    let step = reach / (N - 1) as f64;
    Radial {
        step,
        v: (0..N)
            .map(|k| {
                let rho = k as f64 * step;
                let held = coarse.as_ref().map_or(0.0, |t| t.at(rho));
                (kept * (fine.at(rho) - held)) as f32
            })
            .collect(),
    }
}

impl Shaper {
    /// The world's craters as seen by a preset's grids; `None` without an impact stage.
    pub fn from_world(world: &WorldState, p: &PlanetPreset) -> Option<Shaper> {
        let st: &ImpactState = world.impacts.as_ref()?;
        let mh = p.scale.m_per_block_h;
        let circumference = p.scale.circumference_blocks;
        let frame = BlockFrame::new(circumference);
        let dx1 = f64::from(p.resolution.l1_r) * mh;
        let items = st
            .craters
            .iter()
            .filter(|c| c.kept() > 0.0)
            .filter_map(|c| {
                let table = residual(c, dx1);
                let most = table.v.iter().fold(0.0f32, |m, v| m.max(v.abs()));
                (f64::from(most) >= MIN_CHANGE_M).then(|| {
                    let at = Around::new(frame, mh, c.x, c.z);
                    let extent_m = c.extent_m();
                    let (hw, z0, z1) = at.bounds(table.reach() * extent_m);
                    Item {
                        at,
                        hw,
                        z0,
                        z1,
                        extent_m,
                        table,
                    }
                })
            })
            .collect();
        Some(Shaper {
            items,
            circumference,
        })
    }

    /// Craters shaped.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The items whose reach overlaps the block rectangle [x0, x1] × [z0, z1] (x wraps).
    fn near(&self, x0: f64, x1: f64, z0: f64, z1: f64) -> Vec<&Item> {
        let frame = BlockFrame::new(self.circumference);
        let (cx, hw) = (0.5 * (x0 + x1), 0.5 * (x1 - x0));
        self.items
            .iter()
            .filter(|it| {
                frame.wrap_x(it.at.centre().0 - cx).abs() <= hw + it.hw
                    && it.z0 <= z1
                    && it.z1 >= z0
            })
            .collect()
    }

    /// Adds the craters to a row-major block of cells of `grid` starting at cell (x0, y0), `w`
    /// wide, every `stride`-th cell of the grid.
    pub fn shape_block(
        &self,
        grid: &GridDesc,
        x0: usize,
        y0: usize,
        w: usize,
        stride: usize,
        z: &mut [f32],
    ) {
        if self.items.is_empty() {
            return;
        }
        let r = f64::from(grid.r);
        let bx = |i: usize| grid.origin_block_x as f64 + ((x0 + i * stride) as f64) * r;
        let bz = |j: usize| grid.origin_block_z as f64 + ((y0 + j * stride) as f64) * r;
        // Bands of rows in parallel; within a band, blocks of columns each with the craters that
        // reach them.
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
                    if near.is_empty() {
                        continue;
                    }
                    let mut row = Vec::with_capacity(near.len());
                    for dj in 0..nrows {
                        let zb = bz(j0 + dj);
                        row.clear();
                        row.extend(near.iter().map(|it| it.at.row(zb)));
                        for i in i0..i1 {
                            let xb = bx(i);
                            let mut add = 0.0;
                            for (it, &r) in near.iter().zip(&row) {
                                let d = it.at.distance_of(r, it.at.column(xb));
                                add += it.table.at(d / it.extent_m);
                            }
                            rows[dj * w + i] += add as f32;
                        }
                    }
                }
            });
    }
}

/// A sink that adds the craters to every tile before passing it on.
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
    use crate::surface::impacts::scaling::Morphology;
    use crate::surface::impacts::{Fate, Profile};

    fn crater(x: f64, radius_m: f64, morphology: Morphology) -> Crater {
        let depth = if morphology == Morphology::Simple {
            0.2 * radius_m
        } else {
            400.0
        };
        Crater {
            id: 0,
            x,
            z: 0.0,
            plate: 0,
            oceanic: false,
            t_myr: -1.0,
            lat0: 0.0,
            lon0: 0.0,
            water_m: 0.0,
            impactor_m: 100.0,
            energy_j: 1.0e18,
            diameter_m: 2.0 * radius_m,
            transient_m: 1.6 * radius_m,
            profile: Profile {
                morphology,
                radius_m,
                ejecta_radii: 3.0,
                depth_m: depth,
                rim_m: 0.15 * depth,
                peak_m: 0.5 * depth,
            },
            breccia_m: 0.5 * depth,
            melt_m: 0.0,
            melt_real_m: 0.0,
            melt_km3: 0.0,
            imprinted: false,
            cover_m: 0.0,
            worn_m: 0.0,
            kt_m2: 0.0,
            relief_m: depth,
            fate: Fate::Fresh,
            event: None,
            gone: None,
        }
    }

    fn world_with(craters: Vec<Crater>) -> (WorldState, PlanetPreset) {
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
        w.impacts = Some(ImpactState {
            craters,
            ..ImpactState::default()
        });
        (w, preset)
    }

    /// The crater's change along +x from its centre on L3 (2 blocks, 600 m), on flat ground.
    fn section(sh: &Shaper, x: f64, cells: usize) -> Vec<f32> {
        let g3 = GridDesc::for_planet("L3", 16_384, 2).unwrap();
        let i0 = ((x - g3.origin_block_x as f64) / 2.0).round() as usize;
        let j0 = ((0.0 - g3.origin_block_z as f64) / 2.0).round() as usize;
        let mut z = vec![0.0f32; cells];
        sh.shape_block(&g3, i0, j0, cells, 1, &mut z);
        z
    }

    #[test]
    fn a_fresh_bowl_has_a_floor_below_and_a_rim_above_the_ground() {
        // A 6-km simple crater (R = 3 km = 5 L3 cells), too small for L1.
        let (world, preset) = world_with(vec![crater(0.0, 3000.0, Morphology::Simple)]);
        let sh = Shaper::from_world(&world, &preset).unwrap();
        assert_eq!(sh.len(), 1);
        let z = section(&sh, 0.0, 40);
        assert!(z[0] < -400.0, "floor {}", z[0]);
        assert!(z[5] > 40.0, "rim {}", z[5]);
        assert!(z[5] > z[4] && z[5] > z[7], "the rim is a crest");
        // Ejecta thin outwards and end at three radii.
        assert!(z[7] > z[10] && z[10] > 0.0);
        assert_eq!(z[16], 0.0);
        assert_eq!(z[39], 0.0);
    }

    #[test]
    fn a_crater_is_round_on_the_planet_at_high_latitude() {
        // A 12-km simple crater at 30°, 55° and 70° N: its rim crest and the end of its ejecta lie
        // as far east as north on the planet (a circle in Lambert blocks, until 2026-10-23, put
        // them 1 / cos² φ farther north: 3× at 55°).
        let g3 = GridDesc::for_planet("L3", 16_384, 2).unwrap();
        let frame = BlockFrame::new(16_384);
        for lat in [30.0f64, 55.0, 70.0] {
            let j = ((frame.z_of_lat(lat) - g3.origin_block_z as f64) / 2.0).round() as usize;
            let i = ((0.0 - g3.origin_block_x as f64) / 2.0).round() as usize;
            let x = g3.origin_block_x as f64 + 2.0 * i as f64;
            let z = g3.origin_block_z as f64 + 2.0 * j as f64;
            let mut c = crater(x, 6000.0, Morphology::Simple);
            c.z = z;
            let (world, preset) = world_with(vec![c]);
            let sh = Shaper::from_world(&world, &preset).unwrap();
            // 18 km east is 88 L3 cells at 70°; north, 31 at 30°.
            const N: usize = 160;
            const S: usize = 48;
            let mut east = vec![0.0f32; N];
            sh.shape_block(&g3, i, j, N, 1, &mut east);
            let mut north = vec![0.0f32; S];
            sh.shape_block(&g3, i, j + 1 - S, 1, 1, &mut north);
            north.reverse();
            let at = Around::of(16_384, preset.scale.m_per_block_h, x, z);
            let d_east = |k: usize| at.distance_m(x + 2.0 * k as f64, z);
            let d_north = |k: usize| at.distance_m(x, z - 2.0 * k as f64);
            let crest =
                |v: &[f32]| (0..v.len()).max_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap();
            let end = |v: &[f32]| (0..v.len()).rev().find(|&k| v[k] != 0.0).unwrap();
            // One L3 cell north–south is 600 / cos φ m of the planet (1.75 km at 70°).
            let cell = 600.0 / lat.to_radians().cos() + 1.0;
            let (ce, cn) = (d_east(crest(&east)), d_north(crest(&north)));
            assert!((ce - 6000.0).abs() < cell && (cn - 6000.0).abs() < cell, "{lat}°: rim {ce} {cn}");
            let (ee, en) = (d_east(end(&east)), d_north(end(&north)));
            assert!((ee - en).abs() < cell, "{lat}°: ejecta end {ee} east, {en} north");
            assert!((ee - 18_000.0).abs() < cell, "{lat}°: ejecta end {ee}");
        }
    }

    #[test]
    fn worn_and_old_craters_are_shallower_and_buried_ones_are_not_drawn() {
        let fresh = crater(0.0, 3000.0, Morphology::Simple);
        let mut worn = crater(2000.0, 3000.0, Morphology::Simple);
        worn.worn_m = 0.5 * worn.profile.depth_m;
        let mut old = crater(4000.0, 3000.0, Morphology::Simple);
        old.kt_m2 = 0.01 * 60.0e6;
        let mut buried = crater(6000.0, 3000.0, Morphology::Simple);
        buried.cover_m = 2.0 * buried.profile.depth_m;
        let (world, preset) = world_with(vec![fresh, worn, old, buried]);
        let sh = Shaper::from_world(&world, &preset).unwrap();
        assert_eq!(sh.len(), 3, "the buried crater leaves no shape");
        let relief = |x: f64| {
            let z = section(&sh, x, 12);
            z.iter().fold(f32::MIN, |m, &v| m.max(v)) - z[0]
        };
        let (f, w, o) = (relief(0.0), relief(2000.0), relief(4000.0));
        assert!((w / f - 0.5).abs() < 0.05, "half worn: {w} of {f}");
        assert!(o < 0.8 * f && o > 0.0, "60 Myr of diffusion: {o} of {f}");
        assert!(section(&sh, 6000.0, 12).iter().all(|&v| v == 0.0));
    }

    #[test]
    fn a_crater_in_the_surface_gets_only_what_l1_cannot_hold() {
        // A 40-km complex crater: L1 (4.8 km cells) holds its broad bowl; the last grid adds
        // the central peak, the flat floor's edges and the rim crest, and little net volume.
        let mut big = crater(0.0, 20_000.0, Morphology::Complex);
        let whole = residual(&big, 4800.0);
        big.imprinted = true;
        let band = residual(&big, 4800.0);
        let floor = f64::from(whole.v[0]);
        assert!(floor < -100.0, "the whole profile's centre: {floor}");
        // On the flat floor the band is a fraction of the whole bowl; its rim crest is positive.
        let rim_rho = 1.0 / 3.0;
        let at = 0.4 * rim_rho;
        assert!(
            band.at(at).abs() < 0.5 * whole.at(at).abs(),
            "{} vs {}",
            band.at(at),
            whole.at(at)
        );
        assert!(band.at(rim_rho) > 0.0);
        let vol = |t: &Radial| -> f64 {
            (0..2000)
                .map(|k| {
                    let rho = (f64::from(k) + 0.5) / 2000.0 * t.reach();
                    t.at(rho) * rho
                })
                .sum::<f64>()
        };
        assert!(
            vol(&band).abs() < 0.25 * vol(&whole).abs(),
            "{} vs {}",
            vol(&band),
            vol(&whole)
        );
    }
}
