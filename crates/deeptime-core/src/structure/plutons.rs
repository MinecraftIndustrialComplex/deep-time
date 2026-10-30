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
//!   `collision_pluton_factor` on continental crust in a collision's melt window (crustal-melt
//!   granites: from `tectonics.collision_melt_after_myr` after the orogeny's onset to
//!   `tectonics.collision_melt_end_myr` after its last stamp, the plate layer
//!   `plates/collision_melt`; until 2026-10-30 the collision regime's 30 Myr after each stamp,
//!   which ended about where the sources start them).
//! - **What.** Continental arcs: granodiorite (0.7), granite (0.2), diorite (0.1); island arcs:
//!   diorite (0.6), granodiorite (0.4, the class holds tonalite); collision zones: granite. The
//!   shares are game choices (ledger RCK-064, RCK-065). The qualitative picture is right: the
//!   central Sierra Nevada is tonalite and granodiorite with granite less common and under 3 %
//!   mafic rock (Bateman 1992; Ducea 2001), and the Peruvian Coastal Batholith is two-thirds
//!   Santa Rosa tonalite; no source read gives percentages like these.
//! - **Size.** Length L (the long axis) lognormal (`pluton_length_km` median and bounds,
//!   `pluton_length_sigma`), width 0.5–1 × L, a random strike. Thickness is the power law
//!   T = 0.6·D^0.6 km of Cruden & McCaffrey (2001; read in Cruden 2008, SKB R-08-138, eq. 1, fitted
//!   to 66 plutons) with D the diameter of the circle of the pluton's area, √(L·width), as in the
//!   fit ([`thickness_km`]; ledger RCK-061). The roof is emplaced `pluton_roof_km` below the
//!   surface of the time.
//! - **Contact aureole.** Each intrusion bakes the host rock around it: a shell outside the
//!   intrusion `aureole_share` × its smallest dimension wide (a pluton's thickness; a stock's
//!   width when it is a column rooted on a pluton, [`stock_size_m`]), × `aureole_carbonate_factor`
//!   where the host at its roof is a carbonate ([`aureole_width_m`]). It becomes a body of its own
//!   at the end of the run (role `aureole`, [`Aureole`]). The aureoles Annen (2017, Frontiers in
//!   Earth Science 5:82; abstract, introduction and its table of aureoles read) collects are a
//!   share of their intrusion's size: the western Adamello's 1.7 km round a 5.5-km-thick body is
//!   0.3 of it; the Alta stock's ~1 km round 2 to 3 km is 0.3 to 0.5; but an incrementally
//!   assembled body such as Manaslu has ~100 m round several km. `aureole_share` 0.3 is the
//!   single-pulse middle of that range (ledger RCK-053, game choice). The class follows the host by depth band, read from the column at the
//!   intrusion's centre at the end of the run ([`aureole_bands`]): skarn where the host is a
//!   carbonate, hornfels where it is any other non-crystalline rock, none in crystalline rock
//!   (RCK-054; skarn is calc-silicate rock from carbonate, Meinert 1992; the fine-grained rock
//!   baked from other hosts is hornfels, Nelson, Tulane). Only rock older than the intrusion is
//!   baked, by the structural events' own test (a horizon's youngest key) on the column as the
//!   strata pass paints it ([`HostColumns`]: a pile of one rock laid over an unconformity is one
//!   horizon, cover as a whole): the cover laid on an unroofed pluton is not. A stock lies in the rock older than
//!   itself: under younger cover its top goes down to the cover's base ([`cover_m`]). The
//!   plutons' body ids come before their
//!   aureoles', and where bodies overlap the first by id keeps the block (the strata pass and the
//!   column evaluators), so a pluton keeps its own rock inside its aureole (until 2026-10-24 the
//!   aureoles came first and every pluton was hornfels or skarn inside); a stock's aureole (`deform`'s
//!   stocks are structural events, not bodies) leaves the stock's own volume alone instead
//!   ([`Aureole::hollow`]). Bodies are not moved by the faults younger than them, so an aureole
//!   stays where its intrusion was emplaced.
//! - **Motion.** Each step a pluton turns with the Ω of the plate it sat on (the plate model's
//!   finite rotation, as `surface::advect`), takes the plate now under it (terrane transfer), or is
//!   lost when oceanic crust carrying it went under another plate. Its roof depth follows the
//!   surface's net material change at its cell (`surface/material_m`); a pluton eroded through its
//!   whole height is gone.
//! - **Round on the planet.** Its plan, a_m × c_m at its strike, and its aureole's, that plan grown
//!   by the aureole's width on every side, are ellipses on the planet: the outcrop is drawn with
//!   the planet's offsets from its centre ([`for_each_cell_grown`]) and the bodies carry the
//!   ellipse the Lambert blocks draw of it ([`Pluton::plan_blocks`]), which the world's projection
//!   carries back to the planet's (ledger SCL-235; until 2026-10-23 both were drawn in Lambert
//!   blocks, so the world stretched every pluton north–south by 1 / cos² of its latitude). A
//!   stock's plan (a structural event's window, `deform::intrude`) is drawn the same way.

use std::collections::HashMap;

use deeptime_format::bodies::NO_CLIP;
use deeptime_format::strata::{ColumnHorizon, StackScratch, StoredHorizon, stack_at_with};
use deeptime_format::structure::{Shape as StructShape, StructEvent};
use deeptime_format::{Body, GridDesc, Shape, Zone};
use glam::DVec3;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::deposits::fields::cell_area_km2;
use crate::deposits::random::Rng;
#[cfg(test)]
use crate::params::ScaleParams;
use crate::plates::kinematics::rotation;
use crate::plates::state::{ARC, CONTINENTAL};
use crate::sphere::{Around, BlockFrame};
use crate::surface::lithology::{index, table};
use crate::surface::volcanoes::for_cells;
use crate::synth::EVERY_CLASS;

use super::bake_horizons;
use super::columns::{self, Tables};
use super::params::StructureParams;

/// Thickness (km) of a tabular pluton whose map outline is `length_km` long and `width_km` wide:
/// T = b·D^a with b = 0.6, a = 0.6 and D, the circle-equivalent diameter √(length × width), in km.
/// Cruden & McCaffrey (2001) fitted this to 66 plutons, 4.5 to 600 km across (b ±0.15, a ±0.1;
/// read in Cruden 2008, SKB R-08-138, section 2.1.7, eq. 1); the scatter is large (the 56-km
/// Lucerne granite is 4.5 km thick against 6.7 km predicted). The Cruden (1998) paper that the
/// code used to cite gives no equation, only "about 3 km" for plutons 10 to 100 km wide.
// ledger: RCK-061
pub fn thickness_km(length_km: f64, width_km: f64) -> f64 {
    0.6 * (length_km * width_km).sqrt().powf(0.6)
}

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
    /// Strike on the planet, radians from east towards south (+x towards +z).
    pub yaw: f32,
    pub class: u16,
    /// Width of its contact aureole outside it, m (0: none): [`aureole_width_m`] of the host at
    /// its roof when it was emplaced.
    pub aureole_m: f32,
    /// Emplacement time, Myr.
    pub t_myr: f32,
}

impl Pluton {
    /// Its plan grown by `grow_m` on every side as the ellipse the Lambert blocks of a planet
    /// `circumference` blocks round draw of it at its centre ([`Around::ellipse_blocks`]):
    /// (semi-axis nearest its strike, the other, yaw), blocks and radians from +x towards +z. The
    /// world's projection draws that ellipse as the planet's a_m × c_m (ledger: SCL-235).
    pub fn plan_blocks(&self, circumference: u32, m_per_block_h: f64, grow_m: f64) -> [f64; 3] {
        let g = grow_m.max(0.0);
        let (a, c, yaw) = Around::of(circumference, m_per_block_h, self.x, self.z).ellipse_blocks(
            f64::from(self.a_m) + g,
            f64::from(self.c_m) + g,
            f64::from(self.yaw),
        );
        [a, c, yaw]
    }
}

/// Counters for the run summary.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Counts {
    pub emplaced: u64,
    pub eroded: u64,
    pub subducted: u64,
    /// Not emplaced because drawn past a pole, in the grid row that straddles it (RCK-097).
    #[serde(default)]
    pub past_pole: u64,
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
    pub plate_id: &'a [f32],
    /// Whether the cell's crust melts after a collision (`plates/collision_melt`).
    pub melt: &'a [bool],
}

/// The emplacement intensity of cell `c` (0–1). The 80 mm/yr convergence scale and the 0.7 for
/// island arcs are game choices (ledger RCK-059).
pub fn intensity(x: &ArcInputs<'_>, c: usize, p: &StructureParams) -> f64 {
    let cont = x.crust_type[c] as u8 == CONTINENTAL;
    let flags = x.flags[c] as u32 as u8;
    let arc = if flags & ARC != 0 {
        f64::from((x.convergence[c] / 80.0).clamp(0.0, 1.0)) * if cont { 1.0 } else { 0.7 }
    } else {
        0.0
    };
    let collision = if cont && x.melt[c] {
        p.collision_pluton_factor
    } else {
        0.0
    };
    arc.max(collision)
}

/// Width of the contact aureole of an intrusion whose smallest dimension (a pluton's thickness)
/// is `size_m` and whose roof's host is `host`, m: `aureole_share` of the size, ×
/// `aureole_carbonate_factor` in a carbonate host (ledger RCK-053, RCK-091, game choices; the
/// module docs).
pub fn aureole_width_m(p: &StructureParams, t: &Tables, size_m: f64, host: Option<u16>) -> f64 {
    let carbonate = host.is_some_and(|h| t.carbonate.get(usize::from(h)) == Some(&true));
    p.aureole_share
        * size_m
        * if carbonate {
            p.aureole_carbonate_factor
        } else {
            1.0
        }
}

/// Draws this step's new plutons. `host_at(c, depth)` gives the host class at depth `depth`
/// (m) below the surface of cell `c` (for the aureole's width). A pluton drawn past a pole (the
/// grid's rows run on past it, and the row that straddles it is drawn from over its whole cell)
/// is not emplaced, after all its random draws, so the stream and every other pluton's place and
/// size stay as they were (ids after it shift down by one); it is counted in `counts.past_pole`
/// (ledger: RCK-097).
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
    counts: &mut Counts,
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
    let (granodiorite, granite, diorite) =
        (index("granodiorite"), index("granite"), index("diorite"));
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
        // ledger: RCK-064, RCK-065 (game choice)
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
        // Width 0.5 to 1 of the length (RCK-062, game choice): the large Sierra plutons are
        // narrower, Mount Givens 80 x 17 km and Lamarck over 60 x 10 km (Bateman 1992), while small
        // leucocratic plutons are "subcircular".
        let a_m = 500.0 * length_km;
        let c_m = a_m * (0.5 + 0.5 * rng.uniform());
        // a_m and c_m are the half length and half width in metres: 500 m per km of outline.
        let thick_km = thickness_km(a_m / 500.0, c_m / 500.0);
        let b_m = 500.0 * thick_km;
        let roof_km =
            p.pluton_roof_km[0] + rng.uniform() * (p.pluton_roof_km[1] - p.pluton_roof_km[0]);
        let yaw = rng.uniform() * std::f64::consts::PI;
        if !frame.projection().inside_poles(bz) {
            // Drawn past the pole: no ground there, and the cube bake's Jacobian, singular at the
            // pole, turns a body centred at or past it into a window of 10^7 to 10^10 blocks (a
            // 6 by 4 km pluton comes out 1.4e7 by 5e8 blocks; one centred 0.1 block inside the
            // pole bakes to 9 by 28). The cell's weight already holds the share of it on the
            // planet, so this thins the straddling row a second time by that share (under one
            // row of 2R / r at each pole).
            // ledger: RCK-097
            counts.past_pole += 1;
            continue;
        }
        let aureole_m = aureole_width_m(p, t, 2.0 * b_m, host_at(c, roof_km * 1000.0));
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
            aureole_m: aureole_m as f32,
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

/// Calls `f(cell, r2)` for every L1 cell whose centre lies inside the pluton's horizontal
/// ellipse, with r2 = (u/a)² + (w/c)² in its own axes.
pub fn for_each_cell(
    pl: &Pluton,
    grid: &GridDesc,
    m_per_block_h: f64,
    mut f: impl FnMut(usize, f64),
) {
    for_each_cell_grown(pl, grid, m_per_block_h, 0.0, |c, r2, _| {
        if r2 <= 1.0 {
            f(c, r2);
        }
    });
}

/// Calls `f(cell, r2, r2_grown)` for every L1 cell whose centre lies inside the pluton's
/// horizontal ellipse grown by `grow_m` on every side (its aureole's), with r2 in the pluton's
/// own axes and `r2_grown` in the grown ones. The ellipses are the planet's: a cell's offset is
/// its great-circle distance from the centre along its bearing (ledger: SCL-235; the Lambert
/// grid's block offsets until 2026-10-23).
pub fn for_each_cell_grown(
    pl: &Pluton,
    grid: &GridDesc,
    m_per_block_h: f64,
    grow_m: f64,
    mut f: impl FnMut(usize, f64, f64),
) {
    let (a, c) = (f64::from(pl.a_m), f64::from(pl.c_m));
    let (ga, gc) = (a + grow_m.max(0.0), c + grow_m.max(0.0));
    let (sy, cy) = f64::from(pl.yaw).sin_cos();
    for_cells(grid, m_per_block_h, pl.x, pl.z, ga.max(gc), |cell, e, s| {
        let u = cy * e + sy * s;
        let v = -sy * e + cy * s;
        let r2_grown = (u / ga).powi(2) + (v / gc).powi(2);
        if r2_grown <= 1.0 {
            let r2 = (u / a).powi(2) + (v / c).powi(2);
            f(cell, r2, r2_grown);
        }
    });
}

/// Where the pluton sits in the column at a point of its footprint with r2: the depths of its top
/// and bottom below the surface, m.
pub fn span_at(pl: &Pluton, r2: f64) -> (f64, f64) {
    let b = f64::from(pl.b_m);
    let h = b * (1.0 - r2).max(0.0).sqrt();
    let centre = f64::from(pl.roof_m) + b;
    (centre - h, centre + h)
}

/// The aureole class a host rock gives: skarn from a carbonate, none from crystalline rock,
/// hornfels from any other (ledger RCK-054).
pub fn aureole_class(t: &Tables, host: u16) -> Option<u16> {
    let h = usize::from(host);
    if t.carbonate.get(h) == Some(&true) {
        Some(t.skarn)
    } else if t.crystalline.get(h).is_none_or(|&c| c) {
        None
    } else {
        Some(t.hornfels)
    }
}

/// The class the pluton shows at the surface over a point of its footprint with r2 (`r2_grown`
/// in its aureole's footprint): its own; its aureole's where the aureole reaches the surface and
/// `host`, the rock at the surface there when it is older than the pluton, can be baked; else
/// `None`.
pub fn outcrop_at(
    pl: &Pluton,
    t: &Tables,
    r2: f64,
    r2_grown: f64,
    host: Option<u16>,
) -> Option<u16> {
    let b = f64::from(pl.b_m);
    let centre = f64::from(pl.roof_m) + b;
    if r2 + (centre / b).powi(2) <= 1.0 {
        return Some(pl.class);
    }
    let gb = b + f64::from(pl.aureole_m);
    if pl.aureole_m > 0.0 && r2_grown + (centre / gb).powi(2) <= 1.0 {
        host.and_then(|h| aureole_class(t, h))
    } else {
        None
    }
}

/// The pluton as a body: centre y from the surface elevation `surface_y` (blocks) at its cell,
/// its age the time from its emplacement to `now_myr`, the end of the history, its plan the
/// planet's ellipse in the Lambert blocks of a planet `circumference` blocks round
/// ([`Pluton::plan_blocks`]). `None` when it lies wholly below `min_y`.
#[allow(clippy::too_many_arguments)]
pub fn to_body(
    pl: &Pluton,
    id: u32,
    surface_y: f64,
    circumference: u32,
    m_per_block_h: f64,
    m_per_block_v: f64,
    min_y: i32,
    now_myr: f64,
) -> Option<Body> {
    let b_blocks = f64::from(pl.b_m) / m_per_block_v;
    let cy = centre_y(pl, surface_y, m_per_block_v);
    if cy + b_blocks < f64::from(min_y) {
        return None;
    }
    let [a, c, yaw] = pl.plan_blocks(circumference, m_per_block_h, 0.0);
    let axes = [
        a.max(0.5) as f32,
        b_blocks.max(0.5) as f32,
        c.max(0.5) as f32,
    ];
    let mut body = Body::new(
        id,
        None,
        Shape::Ellipsoid,
        "pluton",
        [pl.x, cy, pl.z],
        axes,
        yaw as f32,
        0.0,
        NO_CLIP,
    );
    body.rock_class = Some(EVERY_CLASS[usize::from(pl.class)].to_owned());
    body.age_ma = Some((now_myr - f64::from(pl.t_myr)).max(0.0) as f32);
    Some(body)
}

/// The y of the pluton's centre, blocks, under ground at `surface_y`.
pub fn centre_y(pl: &Pluton, surface_y: f64, m_per_block_v: f64) -> f64 {
    surface_y - (f64::from(pl.roof_m) + f64::from(pl.b_m)) / m_per_block_v
}

/// The host class at `depth_m` below the surface of a column (for the aureole).
pub fn host_class(units: &[columns::Unit], depth_m: f64) -> Option<u16> {
    columns::class_at(units, depth_m)
}

/// One depth band of a contact aureole: from `top_m` to `base_m` below the ground, the rock it
/// bakes there becomes `class`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub top_m: f64,
    pub base_m: f64,
    pub class: u16,
}

/// The aureole's bands in a column (`units`, top down from the ground, as the strata pass
/// paints them: [`HostColumns`]) between the depths
/// `from_m` and `to_m`: each unit older than the intrusion takes its host's aureole class
/// ([`aureole_class`]). Older is the structural events' own test: a unit whose top key (its
/// youngest part) is at least `key`, the intrusion's stratigraphic key. A unit whose keys
/// straddle `key` is younger cover as a whole, as it is to the stock it would wrap (the column
/// evaluator replaces only the rock under a horizon whose key is at least the event's).
/// Neighbouring bands of one class merge.
pub fn aureole_bands(
    units: &[columns::Unit],
    t: &Tables,
    from_m: f64,
    to_m: f64,
    key: u16,
) -> Vec<Band> {
    let mut out: Vec<Band> = Vec::new();
    let mut top = 0.0;
    for u in units {
        if top >= to_m {
            break;
        }
        let th = f64::from(u.thickness_m);
        let bottom = top + th;
        let older_from = if u.key[0] >= key { top } else { bottom };
        let (lo, hi) = (older_from.max(from_m), bottom.min(to_m));
        if lo < hi
            && let Some(class) = aureole_class(t, u.class)
        {
            match out.last_mut() {
                Some(l) if l.class == class && (l.base_m - lo).abs() < 1e-6 => l.base_m = hi,
                _ => out.push(Band {
                    top_m: lo,
                    base_m: hi,
                    class,
                }),
            }
        }
        top = bottom;
    }
    out
}

/// How deep the cover younger than an intrusion of stratigraphic key `key` reaches in a column
/// (`units`, top down from the ground, as the strata pass paints them: [`HostColumns`]), m: the
/// depth of
/// the first unit older than it by the
/// structural events' test ([`aureole_bands`]), or the whole of `units` when none is. A stock
/// placed in that cover would replace nothing (the column evaluator leaves younger rock alone)
/// and its aureole would bake rock laid after it, so its top goes down to there.
pub fn cover_m(units: &[columns::Unit], key: u16) -> f64 {
    let mut top = 0.0;
    for u in units {
        if u.key[0] >= key {
            return top;
        }
        top += f64::from(u.thickness_m);
    }
    top
}

/// Puts into an aureole's centre column (`units`, as [`HostColumns::units`] gives them under the
/// ground at `a.surface_y`) the rock of the other intrusion events that reach the inner half of
/// its intrusion's plan (any of five probes: the centre and the points half a semi-axis out along
/// and across): a stock emplaced inside or beside an older stock, or a stock emplaced through it
/// later. The aureole is a body and paints over the structural events, so its bands must not
/// bake the older stock's crystalline rock nor paint over the younger one's (it is younger than
/// the aureole). The events are applied as the column rule does at that column (oldest first;
/// each replaces, inside its ellipsoid, the rock from the first unit older than it down, and
/// takes its own class and key), as thick as at the probe it holds deepest, unmoved by later
/// folds and faults. Bands are depth layers across the whole aureole, so an event that holds
/// part of the plan leaves the rest of the aureole unbaked at its depths: the price of no
/// hornfels inside another stock's granite (REV-2, the 2.png line at 220 km). One that holds
/// only the rim is left out. `cands` are the
/// [`StructIndex`](deeptime_format::structure::StructIndex) candidates at the centre (youngest
/// first), `own` the aureole's own event, `c` the planet's circumference, blocks.
pub fn mask_intrusions(
    units: &mut Vec<columns::Unit>,
    a: &Aureole,
    events: &[StructEvent],
    cands: &[u32],
    own: Option<usize>,
    m_per_block_v: f64,
    c: f64,
) {
    let (x, z) = (a.centre[0], a.centre[2]);
    let (s, co) = f64::from(a.yaw).sin_cos();
    let (ha, hc) = (0.5 * a.axes[0], 0.5 * a.axes[2]);
    let probes = [
        (x, z),
        (x + ha * co, z + ha * s),
        (x - ha * co, z - ha * s),
        (x - hc * s, z + hc * co),
        (x + hc * s, z - hc * co),
    ];
    for &e in cands.iter().rev() {
        if Some(e as usize) == own {
            continue;
        }
        let ev = &events[e as usize];
        let StructShape::Intrusion {
            y,
            half_height,
            class,
            ..
        } = ev.shape
        else {
            continue;
        };
        let w = &ev.window;
        let r2 = |(px, pz): (f64, f64)| {
            let (u, v) = w.local(px, pz, c);
            (u / w.half_along).powi(2) + (v / w.half_across).powi(2)
        };
        let held = probes.iter().map(|&p| r2(p)).fold(f64::INFINITY, f64::min);
        if held >= 1.0 {
            continue;
        }
        let hh = half_height * (1.0 - held).sqrt();
        let from_m = (a.surface_y - (y + hh)) * m_per_block_v;
        let to_m = (a.surface_y - (y - hh)) * m_per_block_v;
        overlay(units, from_m, to_m, u16::from(class), ev.key);
    }
}

/// `units` with the depths `from_m` to `to_m` made of `class` (key `key`) from the first unit
/// whose top key is at least `key` down: an intrusion event as the column rule applies it.
fn overlay(units: &mut Vec<columns::Unit>, from_m: f64, to_m: f64, class: u16, key: u16) {
    if to_m <= from_m {
        return;
    }
    let mut out = Vec::with_capacity(units.len() + 2);
    let (mut top, mut reached) = (0.0, false);
    for u in units.iter() {
        let bottom = top + f64::from(u.thickness_m);
        reached |= u.key[0] >= key;
        let (lo, hi) = (from_m.max(top), to_m.min(bottom));
        if reached && lo < hi {
            let part = |th: f64| columns::Unit {
                thickness_m: th as f32,
                ..*u
            };
            if lo > top {
                out.push(part(lo - top));
            }
            out.push(columns::Unit {
                class,
                thickness_m: (hi - lo) as f32,
                source: u.source,
                key: [key, key],
            });
            if bottom > hi {
                out.push(part(bottom - hi));
            }
        } else {
            out.push(*u);
        }
        top = bottom;
    }
    *units = out;
}

/// The columns the intrusions sit in as the strata pass will paint them: the column rule
/// ([`deeptime_format::strata::stack_at`]) over the horizons the structure stage keeps for the
/// bake ([`bake_horizons::Horizons`]), at each intrusion's centre, so that a depth reads as the
/// blocks it lies at, as the bodies' depths do. The rule smooths the cells' stacks into the
/// column and joins equal rocks in a row into one horizon keyed by its top, and the structural
/// events' test reads that key, so a pile of one rock laid over an unconformity (young shale on
/// old shale) is younger cover as a whole: [`aureole_bands`] and [`cover_m`] read this, not the
/// strata log's units. Taken while the planet's horizons are at hand, for these columns only.
#[derive(Default)]
pub struct HostColumns(HashMap<(i64, i64), Vec<ColumnHorizon>>);

impl HostColumns {
    /// The columns of `hz` at the blocks `at` (block x, z).
    pub fn of(hz: &bake_horizons::Horizons, at: impl IntoIterator<Item = (f64, f64)>) -> Self {
        let desc = hz.desc(deeptime_format::Codec::Raw);
        let (k, w) = (hz.k as usize, hz.grid.w as usize);
        let horizon = |i: u32, j: u32, h: u32| {
            let n = (j as usize * w + i as usize) * k + h as usize;
            StoredHorizon {
                top: hz.tops[n],
                class: hz.classes[n],
                key: hz.keys[n],
            }
        };
        let mut blocks: Vec<(i64, i64)> = at
            .into_iter()
            .map(|(x, z)| (x.floor() as i64, z.floor() as i64))
            .collect();
        blocks.sort_unstable();
        blocks.dedup();
        let columns = blocks
            .par_iter()
            .map_init(StackScratch::default, |scratch, &(x, z)| {
                let mut col = Vec::new();
                stack_at_with(scratch, &desc, hz.min_y, horizon, x, z, &mut col);
                ((x, z), col)
            })
            .collect();
        Self(columns)
    }

    /// The column at block (x, z) under the ground at `surface_y` into `out`, as units of their
    /// horizons' class and key whose thicknesses are the metres of `m_per_block_v` they span (a
    /// horizon above the ground has none, the last reaches `min_y`); empty where none was taken.
    pub fn units(
        &self,
        x: f64,
        z: f64,
        surface_y: f64,
        min_y: i32,
        m_per_block_v: f64,
        out: &mut Vec<columns::Unit>,
    ) {
        out.clear();
        let Some(col) = self.0.get(&(x.floor() as i64, z.floor() as i64)) else {
            return;
        };
        let mut top = surface_y;
        for (h, &(_, class, key)) in col.iter().enumerate() {
            // A horizon fills from its top block down to the block over the next one's top.
            let bottom = col
                .get(h + 1)
                .map_or(f64::from(min_y), |n| f64::from(n.0) + 1.0)
                .min(top);
            out.push(columns::Unit {
                class: u16::from(class),
                thickness_m: ((top - bottom) * m_per_block_v) as f32,
                source: columns::Source::Log,
                key: [key, key],
            });
            top = bottom;
        }
    }
}

/// How many slabs a hollow aureole's zones follow the intrusion's wall in. Game choice: enough
/// that the gap they leave between the wall and the aureole is a block or two.
const HOLLOW_SLABS: usize = 8;

/// A contact aureole on its way to the world data ([`Aureole::to_body`]): an ellipsoid shell
/// `width_m` wide round an intrusion, its zones the host's aureole classes by depth.
#[derive(Clone, Debug, PartialEq)]
pub struct Aureole {
    /// Block centre of the intrusion.
    pub centre: [f64; 3],
    /// The intrusion's semi-axes, blocks: along its long axis, vertical, across.
    pub axes: [f64; 3],
    /// The long axis, radians from +x towards +z.
    pub yaw: f32,
    /// How far it reaches out from the intrusion, m.
    pub width_m: f64,
    /// Its own plan when it is drawn on the planet (a pluton's: the pluton's plan grown by the
    /// width there, [`Pluton::plan_blocks`]): semi-axes along and across, blocks, and the yaw,
    /// radians from +x towards +z. `None`: the intrusion's axes grown by the width in blocks, at
    /// its yaw (a stock's, whose plan is Lambert's).
    pub plan: Option<[f64; 3]>,
    /// The ground's y at the centre, blocks (the bands' depths are below it).
    pub surface_y: f64,
    pub bands: Vec<Band>,
    /// Its zones leave the intrusion's own volume alone (a stock, which is a structural event
    /// that no body paints over); a pluton's aureole is solid, and the pluton, a body with a
    /// lower id, keeps its own volume.
    pub hollow: bool,
    /// Myr before the end of the history (its intrusion's age).
    pub age_ma: Option<f32>,
}

impl Aureole {
    /// The aureole of `pl` under ground at `surface_y`, without its bands yet, on a planet
    /// `circumference` blocks round: the pluton's plan and the plan grown by its width are the
    /// planet's ellipses ([`Pluton::plan_blocks`]).
    pub fn of_pluton(
        pl: &Pluton,
        surface_y: f64,
        circumference: u32,
        m_per_block_h: f64,
        m_per_block_v: f64,
        now_myr: f64,
    ) -> Aureole {
        let [a, c, yaw] = pl.plan_blocks(circumference, m_per_block_h, 0.0);
        let width_m = f64::from(pl.aureole_m);
        Aureole {
            centre: [pl.x, centre_y(pl, surface_y, m_per_block_v), pl.z],
            axes: [a, f64::from(pl.b_m) / m_per_block_v, c],
            yaw: yaw as f32,
            width_m,
            plan: Some(pl.plan_blocks(circumference, m_per_block_h, width_m)),
            surface_y,
            bands: Vec::new(),
            hollow: false,
            age_ma: Some((now_myr - f64::from(pl.t_myr)).max(0.0) as f32),
        }
    }

    /// Its semi-axes, blocks: the intrusion's grown by its width (its own [`plan`](Self::plan)
    /// in plan when it has one).
    pub fn grown(&self, m_per_block_h: f64, m_per_block_v: f64) -> [f64; 3] {
        let up = self.axes[1] + self.width_m / m_per_block_v;
        match self.plan {
            Some([a, c, _]) => [a, up, c],
            None => [
                self.axes[0] + self.width_m / m_per_block_h,
                up,
                self.axes[2] + self.width_m / m_per_block_h,
            ],
        }
    }

    /// The yaw of its body: its plan's, or the intrusion's.
    pub fn body_yaw(&self) -> f32 {
        self.plan.map_or(self.yaw, |p| p[2] as f32)
    }

    /// The depths below the ground at the centre it spans, m: [top, base].
    pub fn depths_m(&self, m_per_block_h: f64, m_per_block_v: f64) -> [f64; 2] {
        let gb = self.grown(m_per_block_h, m_per_block_v)[1];
        [
            (self.surface_y - (self.centre[1] + gb)) * m_per_block_v,
            (self.surface_y - (self.centre[1] - gb)) * m_per_block_v,
        ]
    }

    /// The aureole as a body (role `aureole`, no rock class of its own, one zone per band, or per
    /// band and slab when hollow), or `None` when it is empty or wholly below `min_y`.
    pub fn to_body(
        &self,
        id: u32,
        m_per_block_h: f64,
        m_per_block_v: f64,
        min_y: i32,
    ) -> Option<Body> {
        if self.width_m <= 0.0 || self.bands.is_empty() {
            return None;
        }
        let g = self.grown(m_per_block_h, m_per_block_v);
        if self.centre[1] + g[1] < f64::from(min_y) {
            return None;
        }
        let zones = self.zones(g, m_per_block_v);
        if zones.is_empty() {
            return None;
        }
        let mut body = Body::new(
            id,
            None,
            Shape::Ellipsoid,
            "aureole",
            self.centre,
            g.map(|x| x.max(0.5) as f32),
            self.body_yaw(),
            0.0,
            NO_CLIP,
        );
        body.zones = zones;
        body.age_ma = self.age_ma;
        Some(body)
    }

    /// The zones in the grown ellipsoid `g`. A body's zone is a box in (d, v): v the height
    /// (0 its top, 1 its bottom), d the normalised distance from its centre. A hollow aureole's
    /// zones start, slab by slab of v, at the largest d the intrusion reaches in that slab.
    fn zones(&self, g: [f64; 3], m_per_block_v: f64) -> Vec<Zone> {
        let [a, b, c] = self.axes.map(|x| x.max(1e-6));
        let gb = g[1];
        let v_of = |depth_m: f64| {
            let y = self.surface_y - depth_m / m_per_block_v;
            ((self.centre[1] + gb - y) / (2.0 * gb)).clamp(0.0, 1.0)
        };
        let mut cuts: Vec<(f64, f64, f64)> = Vec::new();
        if self.hollow {
            // On the intrusion's wall at height ly, d² = ρ²·(1 − ly²/b²) + ly²/gb² at most,
            // with ρ the larger of its horizontal axes over the grown ones: linear in ly², so
            // its largest value in a slab is at one end of it or at ly = 0.
            let rho2 = (a / g[0]).max(c / g[2]).powi(2);
            let k = 1.0 / (gb * gb) - rho2 / (b * b);
            let d_at = |ly: f64| (rho2 + ly * ly * k).max(0.0).sqrt();
            let (vt, vb) = ((gb - b) / (2.0 * gb), (gb + b) / (2.0 * gb));
            cuts.push((0.0, vt, 0.0));
            for s in 0..HOLLOW_SLABS {
                let s0 = vt + (vb - vt) * s as f64 / HOLLOW_SLABS as f64;
                let s1 = vt + (vb - vt) * (s + 1) as f64 / HOLLOW_SLABS as f64;
                let (ly0, ly1) = (gb - 2.0 * gb * s0, gb - 2.0 * gb * s1);
                let mut d = d_at(ly0).max(d_at(ly1));
                if ly0 >= 0.0 && ly1 <= 0.0 {
                    d = d.max(d_at(0.0));
                }
                cuts.push((s0, s1, d.min(1.0)));
            }
            cuts.push((vb, 1.0, 0.0));
        } else {
            cuts.push((0.0, 1.0, 0.0));
        }
        let names = table();
        let mut out = Vec::new();
        for band in &self.bands {
            let (v0, v1) = (v_of(band.top_m), v_of(band.base_m));
            for &(s0, s1, d) in &cuts {
                let (lo, hi) = (v0.max(s0), v1.min(s1));
                if hi > lo && d < 1.0 {
                    out.push(Zone {
                        name: "aureole".into(),
                        material: None,
                        rock_class: Some(names[usize::from(band.class)].name.to_owned()),
                        d: [d as f32, 1.0],
                        v: [lo as f32, hi as f32],
                        grade: 0.0,
                        density: 0.0,
                        raw: 0.0,
                    });
                }
            }
        }
        out
    }
}

/// A stock's baked ellipsoid (an intrusion event): its block centre, its semi-axes (along its
/// long axis, vertical, across) and the long axis's yaw (radians from +x towards +z, a body's
/// convention), or `None` for any other shape.
pub fn stock_frame(e: &StructEvent) -> Option<([f64; 3], [f64; 3], f32)> {
    let StructShape::Intrusion { y, half_height, .. } = e.shape else {
        return None;
    };
    let w = &e.window;
    Some((
        [w.centre[0], y, w.centre[1]],
        [w.half_along, half_height, w.half_across],
        w.along[1].atan2(w.along[0]) as f32,
    ))
}

/// The smallest dimension of a stock with semi-axes `axes` (blocks: along, vertical, across), m:
/// what its aureole's width scales with ([`aureole_width_m`]). A stock with its own floor is
/// thinner than it is wide; one rooted on a pluton is a column, and its width counts.
pub fn stock_size_m(axes: [f64; 3], m_per_block_h: f64, m_per_block_v: f64) -> f64 {
    let wide = 2.0 * axes[0].min(axes[2]) * m_per_block_h;
    (2.0 * axes[1] * m_per_block_v).min(wide)
}

/// The part of a stock rooted on a pluton from `from_y` down: the stock's rock written as a body
/// (role `stock`, its id before the aureoles'), since the pluton's aureole, a body, would
/// otherwise paint over the stock where it rises through that aureole (a stock itself is a
/// structural event, under every body).
#[derive(Clone, Debug, PartialEq)]
pub struct StockRoot {
    pub centre: [f64; 3],
    /// Semi-axes, blocks: along, vertical, across.
    pub axes: [f64; 3],
    pub yaw: f32,
    pub from_y: f64,
    pub class: u16,
    pub age_ma: Option<f32>,
}

impl StockRoot {
    /// The body, or `None` when nothing of the stock lies under `from_y` above `min_y`.
    pub fn to_body(&self, id: u32, min_y: i32) -> Option<Body> {
        let b = self.axes[1].max(1e-6);
        let v0 = ((self.centre[1] + b - self.from_y) / (2.0 * b)).clamp(0.0, 1.0);
        if v0 >= 1.0 || self.centre[1] + b < f64::from(min_y) {
            return None;
        }
        let mut body = Body::new(
            id,
            None,
            Shape::Ellipsoid,
            "stock",
            self.centre,
            self.axes.map(|x| x.max(0.5) as f32),
            self.yaw,
            0.0,
            NO_CLIP,
        );
        body.zones = vec![Zone {
            name: "root".into(),
            material: None,
            rock_class: Some(EVERY_CLASS[usize::from(self.class)].to_owned()),
            d: [0.0, 1.0],
            v: [v0 as f32, 1.0],
            grade: 0.0,
            density: 0.0,
            raw: 0.0,
        }];
        body.age_ma = self.age_ma;
        Some(body)
    }
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
            aureole_m: 900.0,
            t_myr: -10.0,
        }
    }

    #[test]
    fn thickness_follows_the_published_power_law_of_the_circle_equivalent_diameter() {
        // Cruden & McCaffrey 2001 as restated in Cruden 2008 (SKB R-08-138): T = 0.6 D^0.6 (km).
        // Checks worked from that equation: a round 4.5-km pluton (their Gasborn granite, observed
        // 1.6 km) comes to 1.48 km, a round 56-km one (Lucerne, observed 4.5 km) to 6.7 km.
        assert!((thickness_km(4.5, 4.5) - 1.48).abs() < 0.01);
        assert!((thickness_km(56.0, 56.0) - 6.7).abs() < 0.05);
        // The fit is to the diameter of the circle of equal area, so an elongate body is as thick
        // as the circle of its area: 100 x 25 km has the area of a circle 50 km across.
        assert!((thickness_km(100.0, 25.0) - thickness_km(50.0, 50.0)).abs() < 1e-9);
        // Doubling the diameter multiplies the thickness by 2^0.6.
        let r = thickness_km(40.0, 40.0) / thickness_km(20.0, 20.0);
        assert!((r - 2.0f64.powf(0.6)).abs() < 1e-9);
    }

    #[test]
    fn emplaced_plutons_are_as_thick_as_their_area_says() {
        let grid = GridDesc::for_planet("L1", 4096, 16).unwrap();
        let n = grid.cells();
        let flags = vec![f32::from(ARC); n];
        let convergence = vec![80.0f32; n];
        let crust = vec![f32::from(CONTINENTAL); n];
        let melt = vec![false; n];
        let plate = vec![0.0f32; n];
        let x = ArcInputs {
            flags: &flags,
            convergence: &convergence,
            crust_type: &crust,
            plate_id: &plate,
            melt: &melt,
        };
        let p = StructureParams {
            pluton_rate_km2_myr: 1.0e-4,
            ..StructureParams::default()
        };
        let t = Tables::get();
        let (mut rng, mut next) = (Rng::new(7), 0u32);
        let out = emplace(
            &grid,
            300.0,
            &x,
            &p,
            &t,
            1.0,
            0.0,
            &mut rng,
            &mut next,
            &mut Counts::default(),
            |_, _| None,
        );
        assert!(out.len() > 20, "{} plutons", out.len());
        for pl in &out {
            // Semi-axes a, c and half-height b are 500 m per km of length, width and thickness.
            let (len, wid, thick) = (
                f64::from(pl.a_m) / 500.0,
                f64::from(pl.c_m) / 500.0,
                f64::from(pl.b_m) / 500.0,
            );
            let want = 0.6 * (len * wid).sqrt().powf(0.6);
            assert!((thick - want).abs() < 1e-3 * want, "{thick} against {want}");
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
        advance(
            &mut pls,
            &grid,
            &omega,
            1.0e6,
            &plate,
            &crust,
            Some(&material),
            &mut counts,
        );
        assert_eq!(pls.len(), 1);
        // Moved one cell in longitude (16 blocks east or west, whichever the frame calls +Ω),
        // and the erosion there took 500 m off its roof.
        let moved = BlockFrame::new(4096).wrap_x(pls[0].x).abs();
        assert!((moved - 16.0).abs() < 0.05, "moved {moved}");
        assert!((pls[0].roof_m - 2500.0).abs() < 1e-3);
        // Eroded through its whole height: gone.
        let mut deep = vec![-10_000.0f32; n];
        deep[0] = 0.0;
        advance(
            &mut pls,
            &grid,
            &omega,
            0.0,
            &plate,
            &crust,
            Some(&deep),
            &mut counts,
        );
        assert!(pls.is_empty());
        assert_eq!(counts.eroded, 1);
        // Oceanic crust carried under another plate: subducted.
        let mut isl = pluton(100.0, 100.0);
        isl.oceanic = true;
        let mut pls = vec![isl];
        let other = vec![3.0f32; n];
        let ocean = vec![0.0f32; n];
        advance(
            &mut pls,
            &grid,
            &omega,
            0.0,
            &other,
            &ocean,
            None,
            &mut counts,
        );
        assert!(pls.is_empty() && counts.subducted == 1);
    }

    #[test]
    fn a_pluton_is_round_on_the_planet_at_high_latitude() {
        // SCL-235: at 30°, 55° and 70° N a round pluton's outcrop reaches as far east as north on
        // the planet, and its body's plan is wide in Lambert x by 1 / cos φ and narrow in z by
        // cos φ, so the world draws it round; so is its aureole's.
        let c = 16_384;
        let grid = GridDesc::for_planet("L1", c, 16).unwrap();
        let frame = BlockFrame::new(c);
        let (mh, mv) = (300.0, 20.0);
        for lat in [30.0f64, 55.0, 70.0] {
            let cos = lat.to_radians().cos();
            let mut pl = pluton(0.0, frame.z_of_lat(lat));
            (pl.a_m, pl.c_m) = (30_000.0, 30_000.0);
            let at = Around::new(frame, mh, pl.x, pl.z);
            let (mut east, mut north) = (0.0f64, 0.0f64);
            for_each_cell(&pl, &grid, mh, |cell, r2| {
                let (i, j) = (cell % grid.w as usize, cell / grid.w as usize);
                let bx = grid.block_x(i as i64) as f64;
                let (e, s) = at.offset(bx, grid.block_z(j as i64) as f64);
                assert!(
                    ((e.hypot(s) / 30_000.0).powi(2) - r2).abs() < 1e-9,
                    "{lat}°: r2 {r2}"
                );
                east = east.max(e.abs());
                north = north.max(s.abs());
            });
            // An L1 cell is 4.8 km of the planet east–west times cos φ, north–south over it.
            assert!(
                east <= 30_000.0 && east > 30_000.0 - 4800.0 * cos,
                "{lat}°: east {east}"
            );
            assert!(
                north <= 30_000.0 && north > 30_000.0 - 4800.0 / cos,
                "{lat}°: north {north}"
            );
            let b = to_body(&pl, 1, 100.0, c, mh, mv, -352, 40.0).unwrap();
            assert!(
                (f64::from(b.axes[0]) * cos * mh / 30_000.0 - 1.0).abs() < 1e-3,
                "{lat}°"
            );
            assert!(
                (f64::from(b.axes[2]) * mh / cos / 30_000.0 - 1.0).abs() < 1e-3,
                "{lat}°"
            );
            let a = Aureole::of_pluton(&pl, 100.0, c, mh, mv, 40.0);
            let g = a.grown(mh, mv);
            let w = 30_000.0 + f64::from(pl.aureole_m);
            assert!(
                (g[0] * cos * mh / w - 1.0).abs() < 1e-3,
                "{lat}°: aureole {g:?}"
            );
            assert!(
                (g[2] * mh / cos / w - 1.0).abs() < 1e-3,
                "{lat}°: aureole {g:?}"
            );
        }
        // A yawed one keeps its outline on the planet: every point of its body's plan, drawn in
        // Lambert blocks, lies on the planet's 6 × 4 km ellipse at its strike.
        let mut pl = pluton(0.0, frame.z_of_lat(55.0));
        pl.yaw = 0.5;
        let [a, b, yaw] = pl.plan_blocks(c, mh, 0.0);
        let at = Around::new(frame, mh, pl.x, pl.z);
        let (sy, cy) = (0.5f64).sin_cos();
        for k in 0..72 {
            let t = f64::from(k) * 5f64.to_radians();
            let (u, v) = (a * t.cos(), b * t.sin());
            let (x, z) = (
                pl.x + yaw.cos() * u - yaw.sin() * v,
                pl.z + yaw.sin() * u + yaw.cos() * v,
            );
            let (e, s) = at.offset(x, z);
            let r2 = ((cy * e + sy * s) / 6000.0).powi(2) + ((-sy * e + cy * s) / 4000.0).powi(2);
            assert!((r2 - 1.0).abs() < 1e-2, "at {k}: r2 {r2}");
        }
    }

    #[test]
    fn a_pluton_becomes_a_body_with_its_age() {
        let pl = pluton(10.0, 20.0);
        let b = to_body(&pl, 7, 100.0, 16_384, 300.0, 20.0, -352, 40.0).unwrap();
        assert_eq!(b.rock_class.as_deref(), Some("granodiorite"));
        assert!(b.zones.is_empty(), "the aureole is a body of its own");
        // Emplaced at -10 Myr, the history ends at 40: 50 Myr old.
        assert_eq!(b.age_ma, Some(50.0));
        b.validate().unwrap();
        // Roof 3 km (150 blocks) below y 100, half-height 75 blocks.
        assert!((b.center[1] - (100.0 - 225.0)).abs() < 1e-9);
        // 20 blocks of 300 m east–west, a hair more at its latitude (0.44°).
        assert!((b.axes[0] - 20.0).abs() < 1e-3 && (b.axes[1] - 75.0).abs() < 1e-5);
        // Buried below the world: no body.
        let mut deep = pl.clone();
        deep.roof_m = 20_000.0;
        assert!(to_body(&deep, 8, 0.0, 16_384, 300.0, 20.0, -352, 40.0).is_none());
        let (top, bottom) = span_at(&pl, 0.0);
        assert!((top - 3000.0).abs() < 1e-9 && (bottom - 6000.0).abs() < 1e-9);
    }

    #[test]
    fn the_aureole_is_in_the_host_rock_and_its_class_follows_the_host() {
        let t = Tables::get();
        let pl = pluton(0.0, 0.0);
        // Buried: nothing at the surface.
        assert_eq!(outcrop_at(&pl, &t, 0.0, 0.0, Some(index("shale"))), None);
        // Unroofed to its middle: the pluton over its footprint, and outside it, where the
        // aureole (900 m wide) reaches the surface, the host baked by its class.
        let mut cut = pl.clone();
        cut.roof_m = -1500.0;
        let shale = Some(index("shale"));
        assert_eq!(
            outcrop_at(&cut, &t, 0.0, 0.0, shale),
            Some(index("granodiorite"))
        );
        // 6.5 km out along the long axis: outside the pluton (a 6 km), inside the aureole (6.9).
        let (r2, r2g) = ((6500.0f64 / 6000.0).powi(2), (6500.0f64 / 6900.0).powi(2));
        assert_eq!(
            outcrop_at(&cut, &t, r2, r2g, shale),
            Some(index("hornfels"))
        );
        assert_eq!(
            outcrop_at(&cut, &t, r2, r2g, Some(index("limestone"))),
            Some(index("skarn"))
        );
        assert_eq!(outcrop_at(&cut, &t, r2, r2g, Some(index("granite"))), None);
        // Younger cover (no older host at the surface): nothing baked.
        assert_eq!(outcrop_at(&cut, &t, r2, r2g, None), None);
        // 7.5 km out: past the aureole.
        let (r2, r2g) = ((7500.0f64 / 6000.0).powi(2), (7500.0f64 / 6900.0).powi(2));
        assert_eq!(outcrop_at(&cut, &t, r2, r2g, shale), None);
        // The width: 0.3 of the thickness, 1.5 times that in a carbonate host.
        let p = StructureParams::default();
        assert!((aureole_width_m(&p, &t, 3000.0, shale) - 900.0).abs() < 1e-9);
        let lime = Some(index("limestone"));
        assert!((aureole_width_m(&p, &t, 3000.0, lime) - 1350.0).abs() < 1e-9);
    }

    fn unit(class: &str, thickness_m: f32, key: [u16; 2]) -> columns::Unit {
        columns::Unit {
            class: index(class),
            thickness_m,
            source: columns::Source::Log,
            key,
        }
    }

    #[test]
    fn only_rock_older_than_the_intrusion_is_baked() {
        let t = Tables::get();
        let units = [
            // Younger than the intrusion (key 8000): sealed over it, not baked.
            unit("sandstone", 500.0, [5000, 6000]),
            // Straddles it: cover as a whole, as the column evaluator takes it.
            unit("shale", 1000.0, [7000, 9000]),
            unit("limestone", 1000.0, [9000, 10000]),
            unit("granite", 5000.0, [40000, 40100]),
        ];
        let bands = aureole_bands(&units, &t, 0.0, 4000.0, 8000);
        let skarn = index("skarn");
        // The limestone only: the granite under it is crystalline.
        assert_eq!(
            bands,
            vec![Band {
                top_m: 1500.0,
                base_m: 2500.0,
                class: skarn
            }]
        );
        // Clipped to the depths asked for.
        let bands = aureole_bands(&units, &t, 1200.0, 2000.0, 8000);
        assert_eq!(bands.len(), 1);
        assert!((bands[0].top_m - 1500.0).abs() < 1e-9 && (bands[0].base_m - 2000.0).abs() < 1e-9);
        // The cover a stock of that key would lie under: the sandstone and the shale.
        assert!((cover_m(&units, 8000) - 1500.0).abs() < 1e-9);
        assert!((cover_m(&units, 4000)).abs() < 1e-9);
        assert!((cover_m(&units, 50000) - 7500.0).abs() < 1e-9);
        // A unit wholly older than the intrusion is baked from its top.
        let older = [unit("shale", 1000.0, [8000, 9000])];
        let bands = aureole_bands(&older, &t, 0.0, 4000.0, 8000);
        assert_eq!(
            bands,
            vec![Band {
                top_m: 0.0,
                base_m: 1000.0,
                class: index("hornfels")
            }]
        );
    }

    #[test]
    fn the_column_rule_joins_one_rock_over_an_unconformity_into_cover() {
        // Young shale (strata log) laid on old shale (a basement pool's key range): the bake
        // keeps two horizons, the column rule joins them into one keyed by the young top, so the
        // structural events and the aureole take both as cover.
        let t = Tables::get();
        let mut s: ScaleParams =
            serde_json::from_value(serde_json::json!({ "circumference_blocks": 16384 })).unwrap();
        s.m_per_block_v = 20.0;
        let units = [
            unit("shale", 200.0, [4096, 5000]),
            unit("shale", 1000.0, [33000, 34000]),
            unit("limestone", 1000.0, [34000, 35000]),
            unit("granite", 30000.0, [40000, 40100]),
        ];
        assert!((cover_m(&units, 8000) - 200.0).abs() < 1e-9);
        let k = 8;
        let (mut tops, mut classes, mut keys) = (vec![0i16; k], vec![0u8; k], vec![[0u16; 2]; k]);
        let n = bake_horizons::bake_cell(
            &units,
            100.0,
            s.min_y,
            s.max_y,
            s.m_per_block_v,
            &t.min_blocks,
            &mut tops,
            &mut classes,
            &mut keys,
        );
        assert_eq!(n, 4, "the bake keeps the two shales apart");
        let hz = bake_horizons::Horizons {
            grid: GridDesc::for_planet("L1", 64, 32).unwrap(),
            k: k as u32,
            min_y: s.min_y,
            tops: tops.repeat(4),
            classes: classes.repeat(4),
            keys: keys.repeat(4),
            tongues: (64, 0.15),
            min_blocks: Vec::new(),
        };
        let hosts = HostColumns::of(&hz, [(3.5, -2.0)]);
        let mut col = Vec::new();
        hosts.units(3.9, -1.5, 100.0, s.min_y, s.m_per_block_v, &mut col);
        assert_eq!(
            col.iter().map(|u| u.class).collect::<Vec<_>>(),
            vec![index("shale"), index("limestone"), index("granite")]
        );
        assert_eq!(col[0].key[0], 4096);
        assert!((cover_m(&col, 8000) - 1200.0).abs() <= 20.0);
        let bands = aureole_bands(&col, &t, 0.0, 4000.0, 8000);
        assert_eq!(bands.len(), 1);
        assert_eq!(bands[0].class, index("skarn"));
        assert!((bands[0].top_m - 1200.0).abs() <= 20.0);
        assert!((bands[0].base_m - 2200.0).abs() <= 20.0);
        // Its depths read in blocks of `m_per_block_v`, down to the world's floor.
        let total: f64 = col.iter().map(|u| f64::from(u.thickness_m)).sum();
        assert!((total - (100.0 - f64::from(s.min_y)) * 20.0).abs() < 1.0);
        // A block not taken has no column.
        hosts.units(40.0, -1.5, 100.0, s.min_y, s.m_per_block_v, &mut col);
        assert!(col.is_empty());
    }

    #[test]
    fn a_plutons_aureole_is_a_body_round_it() {
        let t = Tables::get();
        let pl = pluton(0.0, 0.0);
        let (mh, mv) = (300.0, 20.0);
        let mut a = Aureole::of_pluton(&pl, 100.0, 16_384, mh, mv, 40.0);
        // Its top 900 m above the roof (3 km deep), its base 900 m below the floor (6 km).
        let [top, base] = a.depths_m(mh, mv);
        assert!((top - 2100.0).abs() < 1e-6 && (base - 6900.0).abs() < 1e-6);
        let units = [
            unit("shale", 2500.0, [9000, 9500]),
            unit("limestone", 10000.0, [9500, 12000]),
        ];
        a.bands = aureole_bands(&units, &t, top, base, 8000);
        let b = a.to_body(3, mh, mv, -352).unwrap();
        b.validate().unwrap();
        assert_eq!(b.role, "aureole");
        assert_eq!(b.rock_class, None);
        assert_eq!(b.age_ma, Some(50.0));
        assert!((b.axes[0] - 23.0).abs() < 1e-5 && (b.axes[1] - 120.0).abs() < 1e-5);
        let class_at = |y: i32| -> Option<String> {
            let at = b.locate(4096, 0, y, 0)?;
            b.zones_at(at).last().and_then(|z| z.rock_class.clone())
        };
        // Over the roof: hornfels in the shale (to 2,500 m, y -25), skarn in the limestone.
        assert_eq!(class_at(-10).as_deref(), Some("hornfels"));
        assert_eq!(class_at(-40).as_deref(), Some("skarn"));
        // Above the aureole: nothing.
        assert_eq!(class_at(0), None);
        // No bands, or no width: no body.
        let mut empty = a.clone();
        empty.bands.clear();
        assert!(empty.to_body(4, mh, mv, -352).is_none());
    }

    #[test]
    fn a_hollow_aureole_leaves_its_intrusion_alone() {
        let t = Tables::get();
        let (mh, mv) = (300.0, 35.0);
        let mut a = Aureole {
            centre: [0.0, 0.0, 0.0],
            axes: [15.0, 30.0, 12.0],
            yaw: 0.4,
            width_m: 600.0,
            plan: None,
            surface_y: 60.0,
            bands: Vec::new(),
            hollow: true,
            age_ma: Some(12.0),
        };
        let [top, base] = a.depths_m(mh, mv);
        a.bands = aureole_bands(&[unit("shale", 1.0e5, [9000, 9100])], &t, top, base, 8000);
        let b = a.to_body(1, mh, mv, -512).unwrap();
        b.validate().unwrap();
        let (sy, cy) = 0.4f64.sin_cos();
        let (mut outside, mut baked) = (0, 0);
        for x in -20..=20 {
            for y in -55..=55 {
                for z in -20..=20 {
                    let Some(at) = b.locate(4096, x, y, z) else {
                        continue;
                    };
                    let has = b.zones_at(at).next().is_some();
                    let (dx, dy, dz) = (x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5);
                    let (lx, lz) = (cy * dx + sy * dz, -sy * dx + cy * dz);
                    let inner = (lx / 15.0).powi(2) + (dy / 30.0).powi(2) + (lz / 12.0).powi(2);
                    if inner < 1.0 {
                        assert!(!has, "zone inside the intrusion at ({x}, {y}, {z})");
                    } else {
                        outside += 1;
                        baked += usize::from(has);
                    }
                }
            }
        }
        // Most of the shell outside it is baked (the slabs leave a thin gap at the wall).
        assert!(baked * 10 > outside * 6, "{baked} of {outside}");
        // Straight over its top, through the aureole's whole height.
        for y in 31..47 {
            let at = b.locate(4096, 0, y, 0).unwrap();
            assert!(b.zones_at(at).next().is_some(), "y {y}");
        }
    }

    #[test]
    fn a_stock_inside_or_beside_another_bakes_no_granite_and_paints_over_no_younger_stock() {
        use deeptime_format::structure::Window;
        let t = Tables::get();
        let mv = 20.0;
        // A stock 15 x 12 blocks in plan at the origin, under ground at y 100, key 8000.
        let a = Aureole {
            centre: [0.0, 40.0, 0.0],
            axes: [15.0, 30.0, 12.0],
            yaw: 0.0,
            width_m: 600.0,
            plan: None,
            surface_y: 100.0,
            bands: Vec::new(),
            hollow: true,
            age_ma: Some(12.0),
        };
        let stock = |id: u32, key: u16, c: [f64; 2], h: [f64; 2], y: f64, half: f64| StructEvent {
            id,
            key,
            t_years: -1.0e6,
            window: Window {
                centre: c,
                along: [1.0, 0.0],
                across: [0.0, 1.0],
                half_along: h[0],
                half_across: h[1],
            },
            shape: StructShape::Intrusion {
                y,
                half_height: half,
                class: index("granite") as u8,
            },
        };
        let events = [
            // 0: its own event.
            stock(0, 8000, [0.0, 0.0], [15.0, 12.0], 40.0, 30.0),
            // 1: an older, larger stock round it: y 50 to 70 at the centre (600 to 1,000 m deep).
            stock(1, 9000, [0.0, 0.0], [60.0, 50.0], 60.0, 10.0),
            // 2: an older stock holding only a corner of its plan (no probe): left out.
            stock(2, 9000, [14.0, 10.0], [6.0, 6.0], 20.0, 15.0),
            // 3: a younger, larger stock through it: y 10 to 20 (1,600 to 1,800 m deep).
            stock(3, 6000, [0.0, 0.0], [40.0, 40.0], 15.0, 5.0),
            // 4: an older stock beside it holding the point half a semi-axis out along (7.5, 0),
            // r2 0.36 there, so 8 of its 10 blocks of half height: y 77 to 93 (140 to 460 m).
            stock(4, 9000, [10.5, 0.0], [5.0, 5.0], 85.0, 10.0),
        ];
        let mut units = vec![unit("sandstone", 1.0e4, [9500, 9600])];
        mask_intrusions(
            &mut units,
            &a,
            &events,
            &[3, 0, 4, 2, 1],
            Some(0),
            mv,
            16384.0,
        );
        let layers: Vec<(u16, f64, u16)> = units
            .iter()
            .map(|u| (u.class, f64::from(u.thickness_m), u.key[0]))
            .collect();
        let expect = [
            ("sandstone", 140.0, 9500),
            ("granite", 320.0, 9000),
            ("sandstone", 140.0, 9500),
            ("granite", 400.0, 9000),
            ("sandstone", 600.0, 9500),
            ("granite", 200.0, 6000),
            ("sandstone", 8200.0, 9500),
        ];
        assert_eq!(layers.len(), expect.len(), "{layers:?}");
        for (&(class, th, key), (n, t_m, k)) in layers.iter().zip(expect) {
            assert_eq!((class, key), (index(n), k), "{layers:?}");
            assert!((th - t_m).abs() < 1e-3, "{layers:?}");
        }
        // Its bands: hornfels in the sandstone, none in the older granite nor in the younger stock.
        let bands = aureole_bands(&units, &t, 0.0, 2500.0, 8000);
        let got: Vec<(f64, f64)> = bands.iter().map(|b| (b.top_m, b.base_m)).collect();
        assert_eq!(got.len(), 4, "{got:?}");
        for ((top, base), (et, eb)) in got.iter().zip([
            (0.0, 140.0),
            (460.0, 600.0),
            (1000.0, 1600.0),
            (1800.0, 2500.0),
        ]) {
            assert!(
                (top - et).abs() < 1e-3 && (base - eb).abs() < 1e-3,
                "{got:?}"
            );
        }
        assert!(bands.iter().all(|b| b.class == index("hornfels")));
    }

    /// A pluton drawn past a pole is not emplaced (RCK-097). The grid's rows run on past the pole
    /// R = C / 2π to ±Zo (651.9 and 704 blocks on this 4,096-block planet, rows of 16), the row
    /// that straddles the pole weighs in by its share of the planet (`GridDesc::area_share`), and a
    /// pluton is placed at a random point of its cell, so most of the straddling row's draws lay
    /// past the pole, where the cube bake's Jacobian is singular.
    #[test]
    fn no_pluton_is_emplaced_past_the_poles() {
        let c = 4096;
        let grid = GridDesc::for_planet("L1", c, 16).unwrap();
        let pole = deeptime_format::Projection::new(c).pole_z();
        assert!(
            (0..grid.h).any(|j| {
                let s = grid.area_share(j);
                s > 0.0 && s < 1.0 && grid.block_z(i64::from(j)) as f64 > 0.0
            }) && (0..grid.h).any(|j| grid.area_share(j) == 0.0),
            "the fixture's grid has a row straddling the pole and rows wholly past it"
        );
        let n = grid.cells();
        let flags = vec![f32::from(ARC); n];
        let convergence = vec![80.0f32; n];
        let crust = vec![f32::from(CONTINENTAL); n];
        let melt = vec![false; n];
        let plate = vec![0.0f32; n];
        let x = ArcInputs {
            flags: &flags,
            convergence: &convergence,
            crust_type: &crust,
            plate_id: &plate,
            melt: &melt,
        };
        let p = StructureParams {
            pluton_rate_km2_myr: 1.0e-4,
            ..StructureParams::default()
        };
        let t = Tables::get();
        let (mut total, mut near) = (0usize, 0usize);
        let mut counts = Counts::default();
        for seed in 0..120 {
            let (mut rng, mut next) = (Rng::new(seed), 0u32);
            let out = emplace(
                &grid,
                300.0,
                &x,
                &p,
                &t,
                1.0,
                0.0,
                &mut rng,
                &mut next,
                &mut counts,
                |_, _| None,
            );
            total += out.len();
            for pl in &out {
                near += usize::from(pl.z.abs() > pole - 8.0);
                assert!(
                    pl.z.abs() < pole,
                    "pluton {} at z {} (pole {pole})",
                    pl.id,
                    pl.z
                );
            }
        }
        // Plutons do form in the last 8 blocks before a pole, and the draws past it are counted,
        // so the test is not empty.
        assert!(
            total > 1000 && near > 0 && counts.past_pole > 0,
            "{total} plutons, {near} near a pole, {counts:?}"
        );
    }
}
