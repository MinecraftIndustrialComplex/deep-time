// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plutons (plan §8.5, Phase 1: "plutons as simple bodies under arcs"). Each step, arc magmatism
//! emplaces plutons at depth; they ride their plate, are unroofed by the surface's erosion (or
//! buried deeper by its deposits) and, at the end of the run, become ellipsoid bodies in the world
//! data (`bodies/bodies.dtb`, role `pluton`, the body's `rock_class` the pluton's), which the strata
//! pass paints over the horizons (plan §7.5).
//!
//! - **Where and how many.** A Poisson number per step with mean Σ_cells intensity · area ·
//!   `pluton_rate_km2_myr` · Δt, placed with probability ∝ intensity · area. Intensity is the arc
//!   flux of the deposit engine's `arc_magma_flux` (`surface/fields.rs`): 1 where the plate model
//!   put arc magma this step, × min(1, convergence / 80 mm/yr), × 0.7 on oceanic crust; plus
//!   `collision_pluton_factor` on continental crust in the collision regime (crustal-melt
//!   granites).
//! - **What.** Continental arcs: granodiorite (0.7), granite (0.2), diorite (0.1); island arcs:
//!   diorite (0.6), granodiorite (0.4, the class holds tonalite); collision zones: granite. Design
//!   shares after the dominance of tonalite–granodiorite in Cordilleran batholiths.
//! - **Size.** Length L lognormal (`pluton_length_km` median and bounds, `pluton_length_sigma`),
//!   width 0.5–1 × L, thickness T = 0.6·L^0.6 km (Cruden 1998), a random strike. The roof is
//!   emplaced `pluton_roof_km` below the surface of the time.
//! - **Contact aureole.** The outer `aureole_shell` of the body (normalised distance) is a zone
//!   whose rock class is the aureole's: skarn where the roof's host rock is a carbonate, hornfels
//!   where it is any other non-crystalline rock, none in crystalline hosts.
//! - **Motion.** Each step a pluton turns with the Ω of the plate it sat on (the plate model's
//!   finite rotation, as `surface::advect`), takes the plate now under it (terrane transfer), or is
//!   lost when oceanic crust carrying it went under another plate. Its roof depth follows the
//!   surface's net material change at its cell (`surface/material_m`); a pluton eroded through its
//!   whole height is gone.

use deeptime_format::bodies::NO_CLIP;
use deeptime_format::{Body, GridDesc, Shape, Zone};
use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::deposits::fields::cell_area_km2;
use crate::deposits::random::Rng;
use crate::plates::kinematics::rotation;
use crate::plates::state::{ARC, CONTINENTAL, regime};
use crate::sphere::BlockFrame;
use crate::surface::lithology::{index, table};
use crate::synth::EVERY_CLASS;

use super::columns::{self, Tables};
use super::params::StructureParams;

/// No aureole.
pub const NO_AUREOLE: u16 = u16::MAX;

/// One pluton.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pluton {
    pub id: u32,
    /// Block position of its centre (x in [−C/2, C/2)).
    pub x: f64,
    pub z: f64,
    /// The plate it rides.
    pub plate: u32,
    /// Emplaced in oceanic crust.
    pub oceanic: bool,
    /// Depth of its roof below the surface, m (negative once erosion has cut into it).
    pub roof_m: f32,
    /// Horizontal semi-axes (along its strike and across), m, and its vertical half-height, m.
    pub a_m: f32,
    pub c_m: f32,
    pub b_m: f32,
    /// Strike, radians from +x towards +z.
    pub yaw: f32,
    pub class: u16,
    /// The aureole's class, or [`NO_AUREOLE`].
    pub aureole: u16,
    /// Emplacement time, Myr.
    pub t_myr: f32,
}

/// Counters for the run summary.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Counts {
    pub emplaced: u64,
    pub eroded: u64,
    pub subducted: u64,
}

/// The L1 cell nearest block (x, z).
pub fn cell_of(grid: &GridDesc, x: f64, z: f64) -> usize {
    let r = f64::from(grid.r);
    let i = ((x - grid.origin_block_x as f64) / r).round() as i64;
    let j = ((z - grid.origin_block_z as f64) / r).round() as i64;
    grid.clamp_j(j) as usize * grid.w as usize + grid.wrap_i(i) as usize
}

/// What a step's emplacement reads, per L1 cell.
pub struct ArcInputs<'a> {
    pub flags: &'a [f32],
    pub convergence: &'a [f32],
    pub crust_type: &'a [f32],
    pub regime: &'a [f32],
    pub plate_id: &'a [f32],
}

/// The emplacement intensity of cell `c` (0–1).
pub fn intensity(x: &ArcInputs<'_>, c: usize, p: &StructureParams) -> f64 {
    let cont = x.crust_type[c] as u8 == CONTINENTAL;
    let flags = x.flags[c] as u32 as u8;
    let arc = if flags & ARC != 0 {
        f64::from((x.convergence[c] / 80.0).clamp(0.0, 1.0)) * if cont { 1.0 } else { 0.7 }
    } else {
        0.0
    };
    let collision = if cont && x.regime[c] as u8 == regime::COLLISION {
        p.collision_pluton_factor
    } else {
        0.0
    };
    arc.max(collision)
}

/// Draws this step's new plutons. `aureole_at(c, depth)` gives the host class at depth `depth`
/// (m) below the surface of cell `c`.
#[allow(clippy::too_many_arguments)]
pub fn emplace(
    grid: &GridDesc,
    m_per_block_h: f64,
    x: &ArcInputs<'_>,
    p: &StructureParams,
    t: &Tables,
    dt_myr: f64,
    t_myr: f64,
    rng: &mut Rng,
    next_id: &mut u32,
    host_at: impl Fn(usize, f64) -> Option<u16>,
) -> Vec<Pluton> {
    let w = grid.w as usize;
    let n = grid.cells();
    let mut cum = Vec::with_capacity(n);
    let mut total = 0.0f64;
    for c in 0..n {
        let k = intensity(x, c, p);
        if k > 0.0 {
            total += k * cell_area_km2(grid, (c / w) as u32, m_per_block_h);
        }
        cum.push(total);
    }
    let lambda = total * p.pluton_rate_km2_myr * dt_myr;
    let count = rng.poisson(lambda);
    let (granodiorite, granite, diorite) = (index("granodiorite"), index("granite"), index("diorite"));
    let r = f64::from(grid.r);
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let target = rng.uniform() * total;
        let c = cum.partition_point(|&v| v <= target).min(n - 1);
        let (i, j) = ((c % w) as i64, (c / w) as i64);
        let bx = grid.block_x(i) as f64 + (rng.uniform() - 0.5) * r;
        let bz = grid.block_z(j) as f64 + (rng.uniform() - 0.5) * r;
        let frame = BlockFrame::new(grid.circumference() as u32);
        let oceanic = x.crust_type[c] as u8 != CONTINENTAL;
        let collision = !oceanic && x.flags[c] as u32 as u8 & ARC == 0;
        let u = rng.uniform();
        let class = if collision {
            granite
        } else if oceanic {
            if u < 0.6 { diorite } else { granodiorite }
        } else if u < 0.7 {
            granodiorite
        } else if u < 0.9 {
            granite
        } else {
            diorite
        };
        let [median, lo, hi] = p.pluton_length_km;
        let length_km = (median * (p.pluton_length_sigma * rng.normal()).exp()).clamp(lo, hi);
        let thick_km = 0.6 * length_km.powf(0.6);
        let a_m = 500.0 * length_km;
        let c_m = a_m * (0.5 + 0.5 * rng.uniform());
        let b_m = 500.0 * thick_km;
        let roof_km = p.pluton_roof_km[0] + rng.uniform() * (p.pluton_roof_km[1] - p.pluton_roof_km[0]);
        let yaw = rng.uniform() * std::f64::consts::PI;
        let host = host_at(c, roof_km * 1000.0);
        let aureole = match host {
            Some(h) if p.aureole_shell > 0.0 && t.carbonate[usize::from(h)] => t.skarn,
            Some(h) if p.aureole_shell > 0.0 && !t.crystalline[usize::from(h)] => t.hornfels,
            _ => NO_AUREOLE,
        };
        out.push(Pluton {
            id: *next_id,
            x: frame.wrap_x(bx),
            z: bz,
            plate: x.plate_id[c].max(0.0) as u32,
            oceanic,
            roof_m: (roof_km * 1000.0) as f32,
            a_m: a_m as f32,
            c_m: c_m as f32,
            b_m: b_m as f32,
            yaw: yaw as f32,
            class,
            aureole,
            t_myr: t_myr as f32,
        });
        *next_id += 1;
    }
    out
}

/// Moves every pluton with its plate over `dt_yr` (Ω per plate id, rad/yr), then takes the plate
/// now under it and the surface's net material change there; drops the lost and the eroded.
#[allow(clippy::too_many_arguments)]
pub fn advance(
    plutons: &mut Vec<Pluton>,
    grid: &GridDesc,
    omega: &dyn Fn(u32) -> [f64; 3],
    dt_yr: f64,
    plate_id: &[f32],
    crust_type: &[f32],
    material_m: Option<&[f32]>,
    counts: &mut Counts,
) {
    let frame = BlockFrame::new(grid.circumference() as u32);
    plutons.retain_mut(|pl| {
        let om = omega(pl.plate);
        if om != [0.0; 3] {
            let q = frame.unit(pl.x, pl.z);
            let (x, z) = frame.block(rotation(DVec3::from_array(om), dt_yr) * q);
            pl.x = x;
            pl.z = z;
        }
        let c = cell_of(grid, pl.x, pl.z);
        let now = plate_id[c].max(0.0) as u32;
        let continental = crust_type[c] as u8 == CONTINENTAL;
        if now != pl.plate {
            if pl.oceanic && !continental {
                counts.subducted += 1;
                return false;
            }
            pl.plate = now;
        }
        if continental {
            pl.oceanic = false;
        }
        if let Some(m) = material_m {
            pl.roof_m += m[c];
        }
        if pl.roof_m < -2.0 * pl.b_m {
            counts.eroded += 1;
            return false;
        }
        true
    });
}

/// Calls `f(cell, r2, depth_to_centre_m)` for every L1 cell whose centre lies inside the
/// pluton's horizontal ellipse, with r2 = (u/a)² + (w/c)² in its own axes.
pub fn for_each_cell(pl: &Pluton, grid: &GridDesc, m_per_block_h: f64, mut f: impl FnMut(usize, f64)) {
    let r = f64::from(grid.r);
    let ext_blocks = f64::from(pl.a_m.max(pl.c_m)) / m_per_block_h;
    let cells = (ext_blocks / r).ceil() as i64 + 1;
    let c0 = cell_of(grid, pl.x, pl.z);
    let w = grid.w as usize;
    let (i0, j0) = ((c0 % w) as i64, (c0 / w) as i64);
    let frame = BlockFrame::new(grid.circumference() as u32);
    let (sy, cy) = f64::from(pl.yaw).sin_cos();
    for dj in -cells..=cells {
        let j = j0 + dj;
        if j < 0 || j >= i64::from(grid.h) {
            continue;
        }
        for di in -cells..=cells {
            let i = i0 + di;
            let bx = grid.block_x(grid.wrap_i(i) as i64) as f64;
            let bz = grid.block_z(j) as f64;
            let dx = frame.wrap_x(bx - pl.x) * m_per_block_h;
            let dz = (bz - pl.z) * m_per_block_h;
            let u = cy * dx + sy * dz;
            let v = -sy * dx + cy * dz;
            let r2 = (u / f64::from(pl.a_m)).powi(2) + (v / f64::from(pl.c_m)).powi(2);
            if r2 <= 1.0 {
                f(j as usize * w + grid.wrap_i(i) as usize, r2);
            }
        }
    }
}

/// Where the pluton sits in the column at a point of its footprint with r2: the depths of its top
/// and bottom below the surface, m.
pub fn span_at(pl: &Pluton, r2: f64) -> (f64, f64) {
    let b = f64::from(pl.b_m);
    let h = b * (1.0 - r2).max(0.0).sqrt();
    let centre = f64::from(pl.roof_m) + b;
    (centre - h, centre + h)
}

/// The class the pluton shows at the surface over a point of its footprint with r2 (its own or its
/// aureole's), or `None` where it does not reach the surface.
pub fn outcrop_at(pl: &Pluton, r2: f64, shell: f64) -> Option<u16> {
    let b = f64::from(pl.b_m);
    let centre = f64::from(pl.roof_m) + b;
    let d2 = r2 + (centre / b).powi(2);
    if d2 > 1.0 {
        return None;
    }
    if pl.aureole != NO_AUREOLE && d2.sqrt() >= 1.0 - shell {
        Some(pl.aureole)
    } else {
        Some(pl.class)
    }
}

/// The pluton as a body: centre y from the surface elevation `surface_y` (blocks) at its cell.
/// `None` when it lies wholly below `min_y`.
pub fn to_body(
    pl: &Pluton,
    id: u32,
    surface_y: f64,
    m_per_block_h: f64,
    m_per_block_v: f64,
    min_y: i32,
    shell: f64,
) -> Option<Body> {
    let b_blocks = f64::from(pl.b_m) / m_per_block_v;
    let cy = surface_y - (f64::from(pl.roof_m) + f64::from(pl.b_m)) / m_per_block_v;
    if cy + b_blocks < f64::from(min_y) {
        return None;
    }
    let axes = [
        (f64::from(pl.a_m) / m_per_block_h).max(0.5) as f32,
        b_blocks.max(0.5) as f32,
        (f64::from(pl.c_m) / m_per_block_h).max(0.5) as f32,
    ];
    let mut body = Body::new(
        id,
        None,
        Shape::Ellipsoid,
        "pluton",
        [pl.x, cy, pl.z],
        axes,
        pl.yaw,
        0.0,
        NO_CLIP,
    );
    body.rock_class = Some(EVERY_CLASS[usize::from(pl.class)].to_owned());
    if pl.aureole != NO_AUREOLE && shell > 0.0 {
        body.zones.push(Zone {
            name: "aureole".into(),
            material: None,
            rock_class: Some(table()[usize::from(pl.aureole)].name.to_owned()),
            d: [(1.0 - shell) as f32, 1.0],
            v: [0.0, 1.0],
            grade: 0.0,
            density: 0.0,
        });
    }
    Some(body)
}

/// The host class at `depth_m` below the surface of a column (for the aureole).
pub fn host_class(units: &[columns::Unit], depth_m: f64) -> Option<u16> {
    columns::class_at(units, depth_m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pluton(x: f64, z: f64) -> Pluton {
        Pluton {
            id: 0,
            x,
            z,
            plate: 0,
            oceanic: false,
            roof_m: 3000.0,
            a_m: 6000.0,
            c_m: 4000.0,
            b_m: 1500.0,
            yaw: 0.0,
            class: index("granodiorite"),
            aureole: index("hornfels"),
            t_myr: -10.0,
        }
    }

    #[test]
    fn plutons_ride_their_plate_and_are_unroofed() {
        let grid = GridDesc::for_planet("L1", 4096, 16).unwrap();
        let n = grid.cells();
        let w = grid.w as usize;
        // Ω about the pole: one L1 cell of longitude per Myr.
        let dlon = std::f64::consts::TAU / w as f64;
        let omega = move |_: u32| [0.0, 0.0, dlon / 1.0e6];
        let plate = vec![0.0f32; n];
        let crust = vec![f32::from(CONTINENTAL); n];
        let mut material = vec![0.0f32; n];
        let mut pls = vec![pluton(0.0, 0.0)];
        let mut counts = Counts::default();
        let c0 = cell_of(&grid, 0.0, 0.0);
        material[c0 + 1] = -500.0;
        material[c0] = -500.0;
        advance(&mut pls, &grid, &omega, 1.0e6, &plate, &crust, Some(&material), &mut counts);
        assert_eq!(pls.len(), 1);
        // Moved one cell in longitude (16 blocks east or west, whichever the frame calls +Ω),
        // and the erosion there took 500 m off its roof.
        let moved = BlockFrame::new(4096).wrap_x(pls[0].x).abs();
        assert!((moved - 16.0).abs() < 0.05, "moved {moved}");
        assert!((pls[0].roof_m - 2500.0).abs() < 1e-3);
        // Eroded through its whole height: gone.
        let mut deep = vec![-10_000.0f32; n];
        deep[0] = 0.0;
        advance(&mut pls, &grid, &omega, 0.0, &plate, &crust, Some(&deep), &mut counts);
        assert!(pls.is_empty());
        assert_eq!(counts.eroded, 1);
        // Oceanic crust carried under another plate: subducted.
        let mut isl = pluton(100.0, 100.0);
        isl.oceanic = true;
        let mut pls = vec![isl];
        let other = vec![3.0f32; n];
        let ocean = vec![0.0f32; n];
        advance(&mut pls, &grid, &omega, 0.0, &other, &ocean, None, &mut counts);
        assert!(pls.is_empty() && counts.subducted == 1);
    }

    #[test]
    fn a_pluton_becomes_a_body_with_its_aureole() {
        let pl = pluton(10.0, 20.0);
        let b = to_body(&pl, 7, 100.0, 300.0, 20.0, -352, 0.15).unwrap();
        assert_eq!(b.rock_class.as_deref(), Some("granodiorite"));
        assert_eq!(b.zones[0].rock_class.as_deref(), Some("hornfels"));
        b.validate().unwrap();
        // Roof 3 km (150 blocks) below y 100, half-height 75 blocks.
        assert!((b.center[1] - (100.0 - 225.0)).abs() < 1e-9);
        assert!((b.axes[0] - 20.0).abs() < 1e-5 && (b.axes[1] - 75.0).abs() < 1e-5);
        // Buried below the world: no body.
        let mut deep = pl.clone();
        deep.roof_m = 20_000.0;
        assert!(to_body(&deep, 8, 0.0, 300.0, 20.0, -352, 0.15).is_none());
        // Outcrop: only once erosion has cut into it.
        assert_eq!(outcrop_at(&pl, 0.0, 0.15), None);
        let mut cut = pl.clone();
        cut.roof_m = -1500.0;
        assert_eq!(outcrop_at(&cut, 0.0, 0.15), Some(index("granodiorite")));
        assert_eq!(outcrop_at(&cut, 0.95, 0.15), Some(index("hornfels")));
        let (top, bottom) = span_at(&pl, 0.0);
        assert!((top - 3000.0).abs() < 1e-9 && (bottom - 6000.0).abs() < 1e-9);
    }
}
