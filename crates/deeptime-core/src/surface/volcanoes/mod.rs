// SPDX-License-Identifier: LGPL-3.0-or-later
//! Volcanoes as landforms (work item P1-21; Ben, decisions log 2026-09-29, "Volcanoes
//! (Recommended, v1)"): stratovolcanoes along arcs, shield volcanoes and island chains over
//! hotspots, calderas, flood-basalt plateaus from large igneous provinces, driven by the plate
//! model and aged by the surface's erosion.
//!
//! # Where it runs
//! A stage coupled to the plates (`Stage::coupled_to`), after the surface and before the structure
//! stage: `[plates, climate, surface, volcanoes, structure]`, one step per plate step. The
//! volcanoes are a catalogue of discrete edifices (block position, kind, radius, height) that ride
//! their plates like the plutons (`structure/plutons.rs`). Each step a volcano adds the relief it
//! builds to the deep-time surface on L1 — the surface's offset and elevation, and the elevation
//! change the plate model applies to L0 at its next step (`surface/feedback_m`), so the crust
//! carries the edifice like any other relief — and its rock to the strata log. The next surface
//! step erodes it like any other land: active volcanoes rebuild toward their target height, dead
//! ones wear down, and their debris becomes the surface's sediment. The coarse L1 edifice is
//! sharpened at L2 and L3 by the amplify stage (`amplify/volcano.rs`) from this catalogue, which
//! the world keeps (`WorldState::volcanoes`).
//!
//! # One step
//! 1. **Motion.** Every volcano and flood-basalt province turns with the Ω of its plate (the plate
//!    model's finite rotation, as `surface::advect`); oceanic crust carried under another plate
//!    takes its volcanoes with it (subducted).
//! 2. **Erosion.** Each volcano loses what the surface stage just took off it: the surface's net
//!    change of the step over its footprint (`surface/material_m`), fitted as dH × the body's
//!    rendered weight plus a uniform part (erosion of the cone lowers it, sediment filling round it
//!    buries it, a change of the whole area does neither).
//! 3. **Arcs.** The arc flux of each cell is the deposit engine's `arc_magma_flux` (the plate
//!    model's `ARC` flag × min(1, convergence / 80 mm/yr), × 0.7 on oceanic crust). A
//!    stratovolcano is active while its planned life lasts and the flux within a cell of it is at
//!    least `arc_min_flux`. New ones form on arc cells, highest flux first (ties in a seeded random
//!    order), where no stratovolcano that is active or died within the shortest lifetime stands
//!    within `arc_spacing_km / density` (× 0.8–1.2), so they line up along the volcanic front
//!    about that far apart. Target height from the flux (`arc_height_km`), radius ∝ height^0.8;
//!    silicic where the continental crust is at least `silicic_crust_km` thick.
//! 4. **Hotspots.** A new volcano forms at a live plume once all its earlier ones have drifted at
//!    least `hotspot_spacing_km / density` away: a shield on oceanic crust, a silicic centre on
//!    continental crust. It builds while within that distance of the plume, more the closer it
//!    is, toward a height set by the plume's strength (`shield_height_km`), so the chain ages
//!    away from the plume: the youngest volcano over it, older ones extinct, eroding and (on
//!    oceanic crust) subsiding by `island_subsidence_m_myr` until they drown as seamounts.
//! 5. **Build.** An active volcano grows by (target − height)·(1 − e^(−Δt/`build_myr`)), capped
//!    so no summit passes `max_elevation_km` above the sea; the growth is laid on the strata log
//!    as its rock (stratovolcano: tuff under andesite, rhyolite when silicic; shield: basalt;
//!    silicic centre: tuff and rhyolite), burying the regolith as a palaeosol.
//! 6. **Calderas.** A silicic stratovolcano collapses at `caldera_rate_per_myr` (a continental
//!    hotspot's centre at three times it): its summit inside the rim radius is removed and the
//!    erupted volume is laid out as an ignimbrite (tuff) sheet to four rim radii and a tuff fill
//!    in the caldera; it then dies. A shield past 4 km opens a small summit caldera.
//! 7. **Flood basalts.** Each new `large_igneous_province` event of the plate model becomes a
//!    province: `flood_basalt_flows` basalt flows, each capped by a thin tuff bed, over its
//!    radius, and the plate model's dome of a third of the pile flattened into a plateau with
//!    a steep edge. The amplifier steps its outcrop into terraces (`flood_basalt_step_m`).
//! 8. **Retire.** A dead volcano worn to a tenth of its peak for 5 Myr leaves the catalogue.
//!
//! At the end the stage writes the events (`deeptime:volcano_*`, `EVENTS.md`; Ben, 2026-09-30, "Group by
//! arc/chain"): one per arc segment (the cones one subduction contact fed) and one per hotspot chain,
//! each with its standing cones as a detail, and on their own only the notable ones — each group's
//! tallest cone, the calderas still standing or large, the flood basalts. The flood-basalt
//! outcrop layer the amplifier terraces (`volcanoes/flood_basalt`) and its summary
//! (`notes.volcanoes`). Every draw comes from a stream seeded by the preset seed and the step, and
//! every loop runs in catalogue order, so a run is the same on any thread count and after a resume.

pub mod params;
pub mod shape;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::{EventId, GridDesc, ParamValue, Place};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use params::VolcanoParams;
pub use shape::{Depression, Kind, Shape};

use crate::deposits::random::Rng;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::plates::events::{HOTSPOT_TRACK, LARGE_IGNEOUS_PROVINCE};
use crate::plates::kinematics::rotation;
use crate::plates::state::{ARC, CONTINENTAL};
use crate::sphere::BlockFrame;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::surface::lithology::index;
use crate::surface::strata_log::IN_SITU;
use crate::world::{Field, WorldState};

/// The stage's name in a pipeline (`deeptime stage volcanoes`).
pub const STAGE_NAME: &str = "volcanoes";

/// The source of every volcano event.
pub const SOURCE: &str = "deeptime:volcanoes";

/// Event kinds (`crates/deeptime-format/EVENTS.md`, "The volcanoes' events").
pub mod kind {
    /// A volcanic arc segment: the stratovolcanoes one subduction contact fed, with its cones.
    pub const ARC: &str = "deeptime:volcano_arc";
    /// A hotspot's chain of volcanoes, with its members.
    pub const HOTSPOT_CHAIN: &str = "deeptime:volcano_hotspot_chain";
    pub const STRATOVOLCANO: &str = "deeptime:volcano_stratovolcano";
    pub const SHIELD: &str = "deeptime:volcano_shield";
    pub const CALDERA_COMPLEX: &str = "deeptime:volcano_caldera_complex";
    pub const CALDERA: &str = "deeptime:volcano_caldera";
    pub const FLOOD_BASALT: &str = "deeptime:volcano_flood_basalt";
}

/// Fields the stage writes.
pub mod field {
    /// Where the volcanoes are (codes, NaN elsewhere): 1 active stratovolcano, 2 dead
    /// stratovolcano, 3 active shield, 4 dead shield or seamount, 5 silicic centre, 6 caldera,
    /// 7 flood basalt.
    pub const MAP: &str = "volcanoes/map";
    /// Written at the end: 0–1, how much of a flood-basalt pile crops out (within 150 m of the
    /// surface), for the amplifier's terraces.
    pub const FLOOD_BASALT: &str = "volcanoes/flood_basalt";
}

/// Keyframe layers.
pub const KEYFRAME_LAYERS: [&str; 1] = [field::MAP];

/// Codes of the map layer.
pub mod code {
    pub const STRATO_ACTIVE: f32 = 1.0;
    pub const STRATO_DEAD: f32 = 2.0;
    pub const SHIELD_ACTIVE: f32 = 3.0;
    pub const SHIELD_DEAD: f32 = 4.0;
    pub const SILICIC: f32 = 5.0;
    pub const CALDERA: f32 = 6.0;
    pub const FLOOD_BASALT: f32 = 7.0;
}

/// A caldera-forming eruption.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Caldera {
    pub t_myr: f64,
    /// Block position of the volcano when it collapsed.
    pub x: f64,
    pub z: f64,
    pub radius_m: f64,
    pub depth_m: f64,
    /// Volume the collapse removed (erupted), km³ (block metric).
    pub erupted_km3: f64,
}

/// What became of a volcano that left the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gone {
    Eroded,
    Subducted,
}

/// One volcano.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Volcano {
    pub id: u32,
    pub kind: Kind,
    /// Block position of its summit (x in [−C/2, C/2)).
    pub x: f64,
    pub z: f64,
    /// The plate it rides.
    pub plate: u32,
    pub oceanic: bool,
    /// Silicic magma (caldera-prone).
    pub silicic: bool,
    /// The hotspot that fed it.
    pub hotspot: Option<u32>,
    /// Its place in its hotspot's chain (0 the first).
    pub chain: u32,
    pub born_myr: f64,
    /// The end of its planned life (arcs); infinite for a hotspot's.
    pub dies_myr: f64,
    /// When it died.
    pub extinct_myr: Option<f64>,
    pub radius_m: f64,
    /// Height of its body now (as last measured and built), m.
    pub height_m: f64,
    /// The most it stood, m.
    pub peak_m: f64,
    /// Relief it built in all, m (at the summit).
    pub built_m: f64,
    /// Subsidence applied since it died, m.
    pub sunk_m: f64,
    /// The relief left of what it built by the surface's erosion (the accounting of step 2), m.
    pub kept_m: f64,
    /// Target height of the last step it was active, m.
    pub target_m: f64,
    /// Arc flux at its last step with arc information, and steps since it last had enough.
    pub flux_last: f64,
    pub missed: u8,
    /// Σ flux·Δt and Σ Δt while active (the mean arc flux; the plume proximity for a hotspot's).
    pub flux_myr: f64,
    pub active_myr: f64,
    pub depression: Option<Depression>,
    pub caldera: Option<Caldera>,
    /// Its subduction onset (arcs) or large igneous province (hotspots), when known.
    pub cause: Option<EventId>,
    /// Its arc segment ([`segment_key`]; 0 for a hotspot's volcano).
    pub segment: u64,
    /// Set when it left the catalogue.
    pub gone: Option<Gone>,
}

impl Volcano {
    pub fn shape(&self) -> Shape {
        Shape {
            kind: self.kind,
            depression: self.depression,
        }
    }

    pub fn active(&self) -> bool {
        self.extinct_myr.is_none()
    }
}

/// A volcanic arc segment: the stratovolcanoes of one subduction contact (its onset event), or of one
/// plate where the contact never had an onset event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArcSegment {
    pub cause: Option<EventId>,
    pub plate: u32,
    pub first_myr: f64,
    pub last_myr: f64,
    /// Cones born, and how many of them on oceanic crust.
    pub born: u32,
    pub oceanic: u32,
    /// Where its cones were born: a reference x and the sums of offsets from it, blocks.
    pub ref_x: f64,
    pub sum_dx: f64,
    pub sum_z: f64,
    /// The tallest any of its cones stood, m.
    pub peak_m: f64,
    pub calderas: u32,
}

/// The key of an arc segment: its onset event, else its plate.
pub fn segment_key(cause: Option<EventId>, plate: u32) -> u64 {
    cause.map_or((1 << 62) | u64::from(plate), |e| e.0)
}

/// A flood-basalt province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloodBasalt {
    pub id: u32,
    pub x: f64,
    pub z: f64,
    pub plate: u32,
    pub radius_m: f64,
    /// The pile at the centre, m.
    pub thickness_m: f64,
    pub t_myr: f64,
    pub event: EventId,
    pub flows: u32,
}

/// Counters for the summary. (Defaults fill counters a checkpoint predates.)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Counts {
    pub strato_born: u64,
    pub shield_born: u64,
    pub silicic_born: u64,
    pub calderas: u64,
    pub summit_calderas: u64,
    pub flood_basalts: u64,
    pub eroded: u64,
    pub subducted: u64,
    /// Relief built in all, km³ (block metric).
    pub built_km3: f64,
    /// Steps whose arc flags the plate model had cleared (resampling steps).
    pub blind_steps: u64,
    /// Stratovolcanoes that died when plate motion brought a taller one within half the spacing.
    pub merged: u64,
    /// Arc segments too small for an event of their own (counted on their subduction onset), and
    /// the cones they built.
    pub small_arc_segments: u64,
    pub small_arc_cones: u64,
    /// The most L1 cells with the arc flag, with at least `arc_min_flux`, and on the volcanic
    /// front, in one step.
    pub arc_flag_cells_max: u64,
    pub arc_cells_max: u64,
    pub front_cells_max: u64,
}

/// The catalogue, kept in `WorldState::volcanoes` (checkpoints carry it; the amplifier reads it).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VolcanoState {
    /// Volcanoes in the catalogue (alive or standing dead), in id order.
    pub volcanoes: Vec<Volcano>,
    /// Hotspot volcanoes and calderas' volcanoes that left the catalogue (for their events).
    pub gone: Vec<Volcano>,
    pub basalts: Vec<FloodBasalt>,
    /// The arc segments, by [`segment_key`].
    pub arcs: BTreeMap<u64, ArcSegment>,
    pub next_id: u32,
    /// Volcanoes each hotspot made so far.
    pub chains: BTreeMap<u32, u32>,
    /// Events of the world already scanned for large igneous provinces.
    pub events_seen: u64,
    /// Every plate's Ω at the end of the last step.
    pub omega: Vec<[f64; 3]>,
    pub counts: Counts,
    /// Steps done.
    pub steps: u64,
    /// Time of the last step, Myr.
    pub t_myr: f64,
    /// `fade_myr` of the run (the amplifier's fade of dead volcanoes' detail).
    pub fade_myr: f64,
    /// Sea level of the last step, m.
    pub sea_level_m: f64,
    /// Events written at the end, and by kind.
    pub events_emitted: u64,
    pub events_by_kind: BTreeMap<String, u64>,
    /// L1 cells with the arc flag at the last step that had arc information.
    pub last_flagged: u64,
    /// The ground under active continental arc cones, every 10 Myr (P1-27).
    #[serde(default)]
    pub arc_bases: Vec<ArcBase>,
}

/// The ground active stratovolcanoes on continental landmasses stand on at one time (P1-27: cones
/// on a cordillera, not on a coastal plain): the surface at each summit less the relief the cone
/// built, above the sea. Landmasses smaller than
/// `plates::stats::MARGIN_MIN_LANDMASS_KM2` (Earth-normalized) are left out: island arcs stand on
/// the sea floor by nature.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ArcBase {
    pub t_myr: f64,
    pub cones: u32,
    pub base_p25_m: f64,
    pub base_p50_m: f64,
    pub base_p75_m: f64,
    /// Share of them standing on ground below 500 m.
    pub low_share: f64,
}

/// The arc bases are recorded this often, Myr.
const ARC_BASE_EVERY_MYR: f64 = 10.0; // ledger: VOL-071

/// Per L1 cell: continental crust (`plates/crust_type`) in a connected landmass of at least
/// `min_km2` (4-neighbour, x wraps).
fn landmasses(grid: &GridDesc, crust: &[f32], min_km2: f64, cell_m: f64) -> Vec<bool> {
    let (w, h) = (grid.w as usize, grid.h as usize);
    let land = |c: usize| crust[c] >= 0.5;
    let mut out = vec![false; w * h];
    let mut seen = vec![false; w * h];
    let mut stack = Vec::new();
    for start in 0..w * h {
        if seen[start] || !land(start) {
            continue;
        }
        let mut members = Vec::new();
        seen[start] = true;
        stack.push(start);
        let mut area = 0.0;
        while let Some(c) = stack.pop() {
            members.push(c);
            let (i, j) = (c % w, c / w);
            // Cells shrink towards the poles (the grid is equirectangular).
            let lat = std::f64::consts::PI * (0.5 - (j as f64 + 0.5) / h as f64);
            area += cell_m * cell_m * lat.cos() / 1.0e6;
            let mut nb = [usize::MAX; 4];
            nb[0] = j * w + (i + 1) % w;
            nb[1] = j * w + (i + w - 1) % w;
            if j > 0 {
                nb[2] = (j - 1) * w + i;
            }
            if j + 1 < h {
                nb[3] = (j + 1) * w + i;
            }
            for q in nb {
                if q != usize::MAX && !seen[q] && land(q) {
                    seen[q] = true;
                    stack.push(q);
                }
            }
        }
        if area >= min_km2 {
            for c in members {
                out[c] = true;
            }
        }
    }
    out
}

/// Estimate: the catalogue is small; a few L1 arrays per step.
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l1_r);
    let cells = (c / r) * (c / r / 2);
    StageEstimate {
        bytes: cells * 16 + (1 << 20),
        seconds: cells as f64 * p.plate_steps() as f64 * 0.01e-6,
        history_bytes: 0,
    }
}

#[derive(Serialize, Deserialize)]
struct Saved {
    done: u64,
}

/// The volcano stage.
pub struct VolcanoStage {
    preset: PlanetPreset,
    p: VolcanoParams,
    total: u64,
    done: u64,
    timings: BTreeMap<&'static str, f64>,
}

/// The L1 grid of a preset.
fn l1_grid(preset: &PlanetPreset) -> Result<GridDesc, CoreError> {
    Ok(GridDesc::for_planet(
        "L1",
        preset.scale.circumference_blocks,
        preset.resolution.l1_r,
    )?)
}

/// Where cells lie relative to a point, on a grid: calls `f(cell, dx_m, dz_m)` for every cell
/// whose centre is within `reach_m` (block metric) of block (x, z).
pub fn for_cells(
    grid: &GridDesc,
    mh: f64,
    x: f64,
    z: f64,
    reach_m: f64,
    mut f: impl FnMut(usize, f64, f64),
) {
    let r = f64::from(grid.r);
    let frame = BlockFrame::new(grid.circumference() as u32);
    let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
    let reach_b = reach_m / mh;
    let (fi, fj) = ((x - ox) / r, (z - oz) / r);
    let span = (reach_b / r).ceil() as i64 + 1;
    let (ci, cj) = (fi.round() as i64, fj.round() as i64);
    let w = grid.w as usize;
    for j in cj - span..=cj + span {
        if j < 0 || j >= i64::from(grid.h) {
            continue;
        }
        let bz = oz + j as f64 * r;
        let dz = (bz - z) * mh;
        if dz.abs() > reach_m {
            continue;
        }
        for i in ci - span..=ci + span {
            let bx = ox + i as f64 * r;
            let dx = frame.wrap_x(bx - x) * mh;
            if dx.hypot(dz) <= reach_m {
                f(j as usize * w + grid.wrap_i(i) as usize, dx, dz);
            }
        }
    }
}

/// Bilinear value of an L1 array at block (x, z).
fn sample_at(grid: &GridDesc, v: &[f32], x: f64, z: f64) -> f64 {
    let r = f64::from(grid.r);
    let fi = (x - grid.origin_block_x as f64) / r;
    let fj = (z - grid.origin_block_z as f64) / r;
    f64::from(crate::surface::advect::bilinear(
        grid.w as usize,
        grid.h as usize,
        v,
        fi,
        fj,
    ))
}

/// The L1 cell nearest block (x, z).
pub fn cell_of(grid: &GridDesc, x: f64, z: f64) -> usize {
    crate::structure::plutons::cell_of(grid, x, z)
}

/// The arc flux of a cell (the deposit engine's `arc_magma_flux`, `surface/fields.rs`).
pub fn arc_flux(flags: f32, convergence: f32, crust: f32) -> f64 {
    let flags = flags as u32 as u8;
    if flags & ARC == 0 {
        return 0.0;
    }
    let cont = crust as u8 == CONTINENTAL;
    f64::from((convergence / 80.0).clamp(0.0, 1.0)) * if cont { 1.0 } else { 0.7 } // ledger: VOL-003
}

/// Rock classes the volcanoes lay down.
#[derive(Clone, Copy)]
struct Rocks {
    andesite: u16,
    basalt: u16,
    tuff: u16,
    rhyolite: u16,
}

impl Rocks {
    fn get() -> Rocks {
        Rocks {
            andesite: index("andesite"),
            basalt: index("basalt"),
            tuff: index("tuff"),
            rhyolite: index("rhyolite"),
        }
    }

    /// (class, share) pairs, bottom first, of a volcano's growth.
    fn of(&self, v: &Volcano) -> [(u16, f32); 2] {
        match (v.kind, v.silicic) {
            (Kind::Strato, false) => [(self.tuff, 0.35), (self.andesite, 0.65)], // ledger: VOL-056
            (Kind::Strato, true) => [(self.tuff, 0.45), (self.rhyolite, 0.55)],
            (Kind::Shield, _) => [(self.basalt, 1.0), (self.basalt, 0.0)], // ledger: VOL-058
            (Kind::Silicic, _) => [(self.tuff, 0.5), (self.rhyolite, 0.5)], // ledger: VOL-059
        }
    }
}

/// What a step reads and changes on L1.
struct L1<'a> {
    grid: &'a GridDesc,
    mh: f64,
    cell_m: f64,
    /// Elevation change of the step per cell, m (applied at the end of the step).
    delta: Vec<f32>,
    /// The surface's elevation at the start of the step, m.
    elev: &'a [f32],
}

impl L1<'_> {
    /// The rendered weight of a normalised radial function at every cell of its reach, with
    /// `f(cell, weight)`.
    fn render(
        &self,
        x: f64,
        z: f64,
        radius_m: f64,
        reach: f64,
        prof: &dyn Fn(f64) -> f64,
    ) -> Vec<(usize, f64)> {
        let cm = self.cell_m;
        let mut out = Vec::new();
        for_cells(
            self.grid,
            self.mh,
            x,
            z,
            radius_m * reach + cm,
            |c, dx, dz| {
                let w = shape::cell_weight(prof, dx, dz, cm, radius_m);
                if w > 0.0 {
                    out.push((c, w));
                }
            },
        );
        out
    }

    /// The rendered weight of `prof` at the volcano's centre (bilinear over the four cells round
    /// it, as `sample_at` reads the surface there).
    fn centre_weight(&self, x: f64, z: f64, radius_m: f64, prof: &dyn Fn(f64) -> f64) -> f64 {
        let g = self.grid;
        let r = f64::from(g.r);
        let fi = (x - g.origin_block_x as f64) / r;
        let fj = (z - g.origin_block_z as f64) / r;
        let (i0, j0) = (fi.floor(), fj.floor());
        let (ti, tj) = (fi - i0, fj - j0);
        let frame = BlockFrame::new(g.circumference() as u32);
        let mut s = 0.0;
        for (di, dj, w) in [
            (0.0, 0.0, (1.0 - ti) * (1.0 - tj)),
            (1.0, 0.0, ti * (1.0 - tj)),
            (0.0, 1.0, (1.0 - ti) * tj),
            (1.0, 1.0, ti * tj),
        ] {
            let bx = g.origin_block_x as f64 + (i0 + di) * r;
            let bz = g.origin_block_z as f64 + (j0 + dj) * r;
            let dx = frame.wrap_x(bx - x) * self.mh;
            let dz = (bz - z) * self.mh;
            s += w * shape::cell_weight(prof, dx, dz, self.cell_m, radius_m);
        }
        s
    }

    /// The body height the surface shows: its relief at the centre over the foot (`base`), per
    /// the weight the rendered body has there.
    fn relief(&self, v: &Volcano) -> f64 {
        let s = v.shape();
        let prof = |r: f64| s.body(r);
        let base = self.base(v.x, v.z, v.radius_m);
        let centre =
            sample_at(self.grid, self.elev, v.x, v.z) + sample_at(self.grid, &self.delta, v.x, v.z);
        let w = self.centre_weight(v.x, v.z, v.radius_m, &prof);
        if w > 1e-3 {
            ((centre - base) / w).max(0.0)
        } else {
            0.0
        }
    }

    /// The ground around a foot: the lower quartile of the elevation over a ring from 1.05 R to
    /// 1.05 R + 1.5 cells.
    fn base(&self, x: f64, z: f64, radius_m: f64) -> f64 {
        let (inner, outer) = (1.05 * radius_m, 1.05 * radius_m + 1.5 * self.cell_m);
        let mut ring: Vec<f64> = Vec::new();
        for_cells(self.grid, self.mh, x, z, outer, |c, dx, dz| {
            if dx.hypot(dz) >= inner {
                ring.push(f64::from(self.elev[c]) + f64::from(self.delta[c]));
            }
        });
        if ring.is_empty() {
            return sample_at(self.grid, self.elev, x, z);
        }
        // The lower quartile: a neighbour's flank on one side of the ring (shields of a chain
        // overlap) does not lift the ground.
        ring.sort_by(f64::total_cmp);
        ring[ring.len() / 4]
    }
}

impl VolcanoStage {
    pub fn new(preset: PlanetPreset) -> VolcanoStage {
        let p = preset.volcanoes.clone().unwrap_or_default();
        let total = preset.plate_steps();
        VolcanoStage {
            preset,
            p,
            total,
            done: 0,
            timings: BTreeMap::new(),
        }
    }

    fn time(&mut self, what: &'static str, since: Instant) {
        *self.timings.entry(what).or_insert(0.0) += since.elapsed().as_secs_f64() * 1000.0;
    }

    #[allow(clippy::too_many_lines)]
    fn advance(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let started = Instant::now();
        let grid = l1_grid(&self.preset)?;
        let n = grid.cells();
        let s = self.preset.scale.clone();
        let mh = s.m_per_block_h;
        let cell_m = f64::from(grid.r) * mh;
        let frame = BlockFrame::new(s.circumference_blocks);
        let p = self.p.clone();
        let plates = world.plates.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the volcano stage runs after a plate step".into())
        })?;
        let t = plates.t_myr;
        let dt = self.preset.length_myr() / self.total.max(1) as f64;
        let omega_now: Vec<[f64; 3]> = plates
            .plates
            .iter()
            .map(|pl| if pl.alive { pl.omega } else { [0.0; 3] })
            .collect();
        let parents: Vec<Option<u32>> = plates.plates.iter().map(|pl| pl.parent).collect();
        let hotspots: Vec<(u32, [f64; 3], f64, Option<EventId>)> = plates
            .hotspots
            .iter()
            // By its dates, not `closed`: at the last step the plate model has already closed the
            // tracks of the hotspots still alive (`ongoing`).
            .filter(|h| h.born_myr <= t && t < h.dies_myr)
            .map(|h| (h.id, h.pos, h.uplift_mm_yr, h.lip_event))
            .collect();
        // Subduction contacts with an onset event: (overriding plate, centroid, event).
        let contacts: Vec<(u32, DVec3, EventId)> = plates
            .contacts
            .iter()
            .filter_map(|c| {
                c.onset_event
                    .map(|e| (c.over, DVec3::from_array(c.centroid), e))
            })
            .collect();
        let hotspot_ref = self.preset.tectonics.hotspot_uplift_mm_yr.max(1e-9);
        let lip_radius_m = self.preset.tectonics.lip_radius_km
            / self.preset.tectonics.model_radius_km
            * s.radius_m();
        let last = self.done + 1 >= self.total;
        let rocks = Rocks::get();
        let mut rng = Rng::stream(
            self.preset.seed ^ (self.done.wrapping_mul(0x9e37_79b9_7f4a_7c15)),
            "volcanoes",
        );

        let get = |fields: &BTreeMap<String, Field>, name: &str| -> Result<Vec<f32>, CoreError> {
            fields
                .get(name)
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.clone())
                .ok_or_else(|| {
                    CoreError::InvalidState(format!(
                        "the volcano stage needs the L1 field {name} (a plate step with \
                         l1_every_steps = 1)"
                    ))
                })
        };
        let plate_id = get(&world.fields, "plates/plate_id")?;
        let crust = get(&world.fields, "plates/crust_type")?;
        let flags = get(&world.fields, "plates/flags")?;
        let conv = get(&world.fields, "plates/convergence_mm_yr")?;
        let thick = get(&world.fields, "plates/thickness_m")?;
        // The surface stage's net change of this step (erosion −, deposition +), per cell.
        let material = world
            .fields
            .get(crate::surface::field::MATERIAL)
            .filter(|f| f.data.len() == n)
            .map(|f| f.data.clone());
        let mut st = world.volcanoes.take().unwrap_or_else(|| VolcanoState {
            omega: omega_now.clone(),
            fade_myr: p.fade_myr,
            ..VolcanoState::default()
        });
        let surface = world.surface.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the volcano stage runs after the surface stage".into())
        })?;
        let sea = surface.sea_level_m;
        let elev = surface.elevation_m.clone();
        self.time("setup", started);

        // 1. Motion.
        let started = Instant::now();
        let old_omega = st.omega.clone();
        let omega_of = |mut k: u32| -> [f64; 3] {
            for _ in 0..64 {
                if let Some(o) = old_omega.get(k as usize) {
                    return *o;
                }
                match parents.get(k as usize).copied().flatten() {
                    Some(pk) => k = pk,
                    None => break,
                }
            }
            [0.0; 3]
        };
        let dt_yr = dt * 1.0e6;
        let turn = |x: &mut f64, z: &mut f64, plate: u32| {
            let om = omega_of(plate);
            if om != [0.0; 3] {
                let q = frame.unit(*x, *z);
                let (nx, nz) = frame.block(rotation(DVec3::from_array(om), dt_yr) * q);
                *x = nx;
                *z = nz;
            }
        };
        let mut gone_now: Vec<Volcano> = Vec::new();
        let mut kept = Vec::with_capacity(st.volcanoes.len());
        for mut v in std::mem::take(&mut st.volcanoes) {
            turn(&mut v.x, &mut v.z, v.plate);
            let c = cell_of(&grid, v.x, v.z);
            let now = plate_id[c].max(0.0) as u32;
            let continental = crust[c] as u8 == CONTINENTAL;
            if now != v.plate {
                if v.oceanic && !continental {
                    v.gone = Some(Gone::Subducted);
                    v.extinct_myr.get_or_insert(t);
                    st.counts.subducted += 1;
                    gone_now.push(v);
                    continue;
                }
                v.plate = now;
            }
            if continental {
                v.oceanic = false;
            }
            kept.push(v);
        }
        st.volcanoes = kept;
        for b in &mut st.basalts {
            turn(&mut b.x, &mut b.z, b.plate);
            let c = cell_of(&grid, b.x, b.z);
            b.plate = plate_id[c].max(0.0) as u32;
        }
        self.time("motion", started);

        let mut l1 = L1 {
            grid: &grid,
            mh,
            cell_m,
            delta: vec![0.0; n],
            elev: &elev,
        };
        // Log changes: (cell, thickness (negative = erode), class) in order.
        let mut log_ops: Vec<(usize, f32, u16)> = Vec::new();

        // 2. Erosion: each edifice loses what the surface stage just took off it — its net change
        // over the footprint fitted as dH · (the body's weight) + a uniform part, so erosion
        // concentrated on the cone lowers it and sediment filling round it buries it, while a
        // change of the whole area does neither. The relief measured over its foot would also
        // count the ridge or the chain neighbour beside it.
        let started = Instant::now();
        if let Some(m) = &material {
            for v in &mut st.volcanoes {
                let sh = v.shape();
                let prof = |r: f64| sh.body(r);
                let mut cells = Vec::new();
                for_cells(
                    &grid,
                    mh,
                    v.x,
                    v.z,
                    1.3 * v.radius_m + cell_m, // ledger: VOL-033
                    |c, dx, dz| {
                        cells.push((c, shape::cell_weight(&prof, dx, dz, cell_m, v.radius_m)));
                    },
                );
                v.kept_m = (v.kept_m + fit_change(&cells, m)).clamp(0.0, v.peak_m);
            }
        }
        // … and it stands no higher than the surface shows it either (the plate model's
        // resampling and the advection of the offset also flatten a small edifice): the smaller
        // of the two, each kept on its own so that a dip of one step does not stay.
        for v in &mut st.volcanoes {
            v.height_m = v.kept_m.min(l1.relief(v));
        }
        self.time("measure", started);

        // 3–4. Activity and births.
        let started = Instant::now();
        let flux: Vec<f64> = (0..n)
            .map(|c| arc_flux(flags[c], conv[c], crust[c]))
            .collect();
        let flagged = flags
            .iter()
            .filter(|&&f| (f as u32 as u8) & ARC != 0)
            .count() as u64;
        let strong = flux.iter().filter(|&&f| f >= p.arc_min_flux).count() as u64;
        st.counts.arc_flag_cells_max = st.counts.arc_flag_cells_max.max(flagged);
        st.counts.arc_cells_max = st.counts.arc_cells_max.max(strong);
        // The plate model clears its flags on crust it resamples (every 10–60 steps, and at the
        // last step): a step with a fifth of the last one's arc cells or fewer says nothing about
        // the arcs, so the stratovolcanoes keep their state and none are born.
        let flux_valid = flagged > 0 && flagged * 5 > st.last_flagged; // ledger: VOL-011
        if flux_valid || st.last_flagged == 0 {
            st.last_flagged = flagged;
        } else {
            st.counts.blind_steps += 1;
        }
        let w = grid.w as usize;
        let near_flux = |c: usize| -> f64 {
            let (i, j) = ((c % w) as i64, (c / w) as i64);
            let mut m = 0.0f64;
            for dj in -1..=1 {
                let jj = j + dj;
                if jj < 0 || jj >= i64::from(grid.h) {
                    continue;
                }
                for di in -1..=1 {
                    let cc = jj as usize * w + grid.wrap_i(i + di) as usize;
                    m = m.max(flux[cc]);
                }
            }
            m
        };
        let spacing_h = if p.density > 0.0 {
            p.hotspot_spacing_km * 1000.0 / p.density
        } else {
            f64::INFINITY
        };
        let plume_at: BTreeMap<u32, (f64, f64, f64)> = hotspots
            .iter()
            .map(|&(id, pos, uplift, _)| {
                let (x, z) = frame.block(DVec3::from_array(pos));
                (id, (x, z, uplift))
            })
            .collect();
        let dist_m =
            |x0: f64, z0: f64, x1: f64, z1: f64| (frame.wrap_x(x1 - x0) * mh).hypot((z1 - z0) * mh);
        for v in &mut st.volcanoes {
            if !v.active() {
                continue;
            }
            let alive = match v.kind {
                Kind::Strato => {
                    if flux_valid {
                        let f = near_flux(cell_of(&grid, v.x, v.z));
                        if f >= p.arc_min_flux {
                            v.flux_last = f;
                            v.missed = 0;
                        } else {
                            v.missed = v.missed.saturating_add(1);
                        }
                    }
                    // One step without flux is forgiven (the arc band is a few L0 samples wide).
                    if t < v.dies_myr && v.missed < 2 { // ledger: VOL-010
                        v.flux_myr += v.flux_last * dt;
                        v.active_myr += dt;
                        true
                    } else {
                        false
                    }
                }
                Kind::Shield | Kind::Silicic => match v.hotspot.and_then(|h| plume_at.get(&h)) {
                    Some(&(px, pz, _)) => {
                        let d = dist_m(v.x, v.z, px, pz);
                        if d <= spacing_h {
                            v.flux_myr += (1.0 - (d / spacing_h).powi(2)) * dt;
                            v.active_myr += dt;
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                },
            };
            if !alive {
                v.extinct_myr = Some(t);
            }
        }
        // Vents that plate motion brought within half the arc spacing of each other merge: the
        // lower (or, as high, the younger) dies, the other takes the magma.
        if p.density > 0.0 {
            let half = 0.5 * p.arc_spacing_km * 1000.0 / p.density; // ledger: VOL-009
            let live: Vec<usize> = (0..st.volcanoes.len())
                .filter(|&k| st.volcanoes[k].kind == Kind::Strato && st.volcanoes[k].active())
                .collect();
            for a in 0..live.len() {
                for b in a + 1..live.len() {
                    let (va, vb) = (&st.volcanoes[live[a]], &st.volcanoes[live[b]]);
                    if !va.active() || !vb.active() || dist_m(va.x, va.z, vb.x, vb.z) >= half {
                        continue;
                    }
                    let loser = if (va.height_m, -va.born_myr) >= (vb.height_m, -vb.born_myr) {
                        live[b]
                    } else {
                        live[a]
                    };
                    st.volcanoes[loser].extinct_myr = Some(t);
                    st.counts.merged += 1;
                }
            }
        }
        // Hotspot births.
        if spacing_h.is_finite() {
            for &(hid, _, uplift, lip) in &hotspots {
                let (px, pz, _) = plume_at[&hid];
                let near = st
                    .volcanoes
                    .iter()
                    .filter(|v| v.hotspot == Some(hid))
                    .any(|v| dist_m(v.x, v.z, px, pz) < spacing_h);
                if near {
                    continue;
                }
                let c = cell_of(&grid, px, pz);
                let continental = crust[c] as u8 == CONTINENTAL;
                let strength = ((uplift / hotspot_ref - 0.6) / 0.8).clamp(0.0, 1.0); // ledger: VOL-008
                let (kind, height, radius) = if continental {
                    (
                        Kind::Silicic,
                        p.silicic_height_km,
                        p.silicic_radius_km * 1000.0 * p.size / p.flank_steepness,
                    )
                } else {
                    let hk = p.shield_height_km[0]
                        + (p.shield_height_km[1] - p.shield_height_km[0]) * strength;
                    let r = p.shield_radius_km * (hk / p.shield_height_km[1]).powf(0.8); // ledger: VOL-022
                    (Kind::Shield, hk, r * 1000.0 * p.size / p.flank_steepness)
                };
                let chain = st.chains.entry(hid).or_insert(0);
                let v = Volcano {
                    id: st.next_id,
                    kind,
                    x: px,
                    z: pz,
                    plate: plate_id[c].max(0.0) as u32,
                    oceanic: !continental,
                    silicic: continental,
                    hotspot: Some(hid),
                    chain: *chain,
                    born_myr: t,
                    dies_myr: f64::INFINITY,
                    extinct_myr: None,
                    radius_m: radius,
                    height_m: 0.0,
                    peak_m: 0.0,
                    built_m: 0.0,
                    sunk_m: 0.0,
                    kept_m: 0.0,
                    target_m: height * 1000.0 * p.size,
                    flux_last: 0.0,
                    missed: 0,
                    flux_myr: 0.0,
                    active_myr: 0.0,
                    depression: None,
                    caldera: None,
                    cause: lip,
                    segment: 0,
                    gone: None,
                };
                *chain += 1;
                st.next_id += 1;
                match kind {
                    Kind::Silicic => st.counts.silicic_born += 1,
                    _ => st.counts.shield_born += 1,
                }
                st.volcanoes.push(v);
            }
        }
        // Arc births.
        if p.density > 0.0 && flux_valid {
            let spacing = p.arc_spacing_km * 1000.0 / p.density;
            let recent = p.arc_lifetime_myr[0]; // ledger: VOL-040
            let mut occupied: Vec<(f64, f64)> = st
                .volcanoes
                .iter()
                .filter(|v| {
                    // New vents keep clear of live cones, of the just-dead and of dead cones
                    // still standing at half their height.
                    v.kind == Kind::Strato
                        && (v.extinct_myr.is_none_or(|e| t - e < recent)
                            || v.height_m >= 0.5 * v.peak_m) // ledger: VOL-040
                })
                .map(|v| (v.x, v.z))
                .collect();
            let front = volcanic_front(&grid, &flux, &elev, sea, p.arc_min_flux);
            // A front cell takes the strongest flux within a cell of it (a patch's skeleton may
            // run along a gap between the lattice's stripes).
            let front_flux: Vec<f64> = (0..n)
                .map(|c| if front[c] { near_flux(c) } else { 0.0 })
                .collect();
            st.counts.front_cells_max = st
                .counts
                .front_cells_max
                .max(front.iter().filter(|&&f| f).count() as u64);
            let born = place_arcs(
                &grid,
                mh,
                &front_flux,
                p.arc_min_flux,
                spacing,
                &mut occupied,
                &mut rng,
            );
            for (bx, bz, c) in born {
                let f = front_flux[c];
                let s01 = ((f - p.arc_min_flux) / (1.0 - p.arc_min_flux).max(1e-9)).clamp(0.0, 1.0);
                let hk = p.arc_height_km[0] + (p.arc_height_km[1] - p.arc_height_km[0]) * s01;
                let radius =
                    p.arc_radius_km * (hk / p.arc_height_km[1]).powf(0.8) * 1000.0 * p.size // ledger: VOL-020
                        / p.flank_steepness;
                let continental = crust[c] as u8 == CONTINENTAL;
                let silicic = continental && f64::from(thick[c]) >= p.silicic_crust_km * 1000.0;
                let plate = plate_id[c].max(0.0) as u32;
                let here = frame.unit(bx, bz);
                let cause = contacts
                    .iter()
                    .filter(|(over, _, _)| *over == plate)
                    .min_by(|a, b| a.1.angle_between(here).total_cmp(&b.1.angle_between(here)))
                    .map(|&(_, _, e)| e);
                // A summit crater at least 2 km across, about 7 blocks at 300 m per block, so it
                // shows: a game choice (VOL-026). Real summit craters are mostly under 1 km across
                // (Fuji's 780 m); the large craters and calderas Grosse et al. (2014) could
                // measure on composite volcanoes are 0.6–11 km across, median 2.2 km, and 240 m
                // deep (median). The 0.07 of the doubled basal radius gives 4 km at a 3-km cone.
                let crater_rho = (1000.0 * p.size).max(0.07 * radius) / radius; // ledger: VOL-026
                st.volcanoes.push(Volcano {
                    id: st.next_id,
                    kind: Kind::Strato,
                    x: bx,
                    z: bz,
                    plate,
                    oceanic: !continental,
                    silicic,
                    hotspot: None,
                    chain: 0,
                    born_myr: t,
                    dies_myr: t + range(&mut rng, p.arc_lifetime_myr[0], p.arc_lifetime_myr[1]),
                    extinct_myr: None,
                    radius_m: radius,
                    height_m: 0.0,
                    peak_m: 0.0,
                    built_m: 0.0,
                    sunk_m: 0.0,
                    kept_m: 0.0,
                    target_m: hk * 1000.0 * p.size,
                    flux_last: f,
                    missed: 0,
                    flux_myr: 0.0,
                    active_myr: 0.0,
                    depression: Some(Depression {
                        rho: crater_rho,
                        depth: 0.06, // ledger: VOL-026
                        caldera: false,
                    }),
                    caldera: None,
                    cause,
                    segment: segment_key(cause, plate),
                    gone: None,
                });
                st.next_id += 1;
                st.counts.strato_born += 1;
                let seg = st
                    .arcs
                    .entry(segment_key(cause, plate))
                    .or_insert(ArcSegment {
                        cause,
                        plate,
                        first_myr: t,
                        last_myr: t,
                        born: 0,
                        oceanic: 0,
                        ref_x: bx,
                        sum_dx: 0.0,
                        sum_z: 0.0,
                        peak_m: 0.0,
                        calderas: 0,
                    });
                seg.born += 1;
                seg.oceanic += u32::from(!continental);
                seg.sum_dx += frame.wrap_x(bx - seg.ref_x);
                seg.sum_z += bz;
            }
        }
        self.time("births", started);

        // 5. Build.
        let started = Instant::now();
        let grow = 1.0 - (-dt / p.build_myr).exp();
        // The strength height of this planet's crust over Earth's with the default crust (140 MPa
        // at 9.81 m/s²): 1 on an Earth-like preset.
        let strength_scale = self
            .preset
            .tectonics
            .strength_height_m(self.preset.planet.gravity_m_s2)
            / (140.0e6 / (crate::params::CRUST_DENSITY_KG_M3 * 9.81)); // ledger: VOL-034
        let cell_km2 = cell_m * cell_m / 1.0e6;
        let every = (ARC_BASE_EVERY_MYR / dt).round().max(1.0) as u64;
        let record_bases = (self.done + 1).is_multiple_of(every) || last;
        // Continental arcs only: cones on a landmass of at least a continent's size (island
        // arcs on small crust fragments stand on the sea floor by nature).
        let big_land = if record_bases {
            let model_km = self.preset.tectonics.model_radius_km;
            let planet_km = self.preset.scale.radius_m() / 1000.0;
            let min_km2 =
                crate::plates::stats::MARGIN_MIN_LANDMASS_KM2 * (planet_km / model_km).powi(2);
            landmasses(&grid, &crust, min_km2, cell_m)
        } else {
            Vec::new()
        };
        let mut bases: Vec<f64> = Vec::new();
        for v in &mut st.volcanoes {
            if !v.active() {
                continue;
            }
            // New volcanoes are measured now; the others were measured before the births.
            let base = l1.base(v.x, v.z, v.radius_m);
            if record_bases && v.kind == Kind::Strato && big_land[cell_of(&grid, v.x, v.z)] {
                // The ground under the cone: the surface at its summit less the relief it built
                // (a ring round a cone wider than the range it stands on would read the
                // foothills or, at a coast, the sea).
                let c = cell_of(&grid, v.x, v.z);
                let here = f64::from(l1.elev[c]) + f64::from(l1.delta[c]);
                bases.push(here - v.height_m - sea);
            }
            let target = match v.kind {
                Kind::Strato => {
                    let f = v.flux_last;
                    let s01 =
                        ((f - p.arc_min_flux) / (1.0 - p.arc_min_flux).max(1e-9)).clamp(0.0, 1.0);
                    let hk = p.arc_height_km[0] + (p.arc_height_km[1] - p.arc_height_km[0]) * s01;
                    // A cone never outgrows the height (and radius) it was born with.
                    (hk * 1000.0 * p.size).min(v.target_m)
                }
                Kind::Shield | Kind::Silicic => {
                    let prox = match v.hotspot.and_then(|hh| plume_at.get(&hh)) {
                        Some(&(px, pz, _)) => {
                            (1.0 - (dist_m(v.x, v.z, px, pz) / spacing_h).powi(2)).clamp(0.0, 1.0)
                        }
                        None => 0.0,
                    };
                    v.target_m * prox.sqrt()
                }
            };
            // The ceiling is the same comparison as the crust's (`surface::spread`): an edifice's
            // weight ρ·g·h against the strength under it. `max_elevation_km` is the height that
            // strength holds at Earth's gravity with the default crust (140 MPa), so it scales
            // with the strength height: 6.8 km on Earth, 17.9 km at Mars's gravity.
            let ceiling = strength_scale
                * match v.kind {
                    Kind::Shield => 0.65 * p.max_elevation_km * 1000.0, // ledger: VOL-032
                    _ => p.max_elevation_km * 1000.0,
                };
            let target = target.min((sea + ceiling - base).max(0.0));
            let dh = (target - v.height_m).max(0.0) * grow;
            if dh <= 0.0 {
                continue;
            }
            let sh = v.shape();
            let prof = |r: f64| sh.body(r);
            let layers = rocks.of(v);
            let mut vol = 0.0;
            let cells = l1.render(v.x, v.z, v.radius_m, 1.0, &prof);
            // No cell the cone stands on passes the ceiling. The target above is measured from the
            // ground round the cone (`base`), and that alone let the surface under a cone ratchet
            // up: every step the surface stage eroded the cone, isostasy gave part of it back,
            // the cone's height was debited the whole erosion and rebuilt it. With the ground
            // rising and the ring round it not, cones stood on 15–25 km of their own rebuilding
            // (2026-10-01, seed 13 of the test planet).
            let top = sea + ceiling;
            let room = cells
                .iter()
                .filter(|&&(_, wgt)| wgt > 0.0)
                .map(|&(c, wgt)| {
                    let here = f64::from(l1.elev[c]) + f64::from(l1.delta[c]);
                    ((top - here) / (dh * wgt)).clamp(0.0, 1.0)
                })
                .fold(1.0f64, f64::min);
            let dh = dh * room;
            if dh <= 0.0 {
                continue;
            }
            for (c, wgt) in cells {
                let dz = (dh * wgt) as f32;
                l1.delta[c] += dz;
                vol += f64::from(dz);
                for (class, share) in layers {
                    if share > 0.0 {
                        log_ops.push((c, dz * share, class));
                    }
                }
            }
            st.counts.built_km3 += vol * cell_km2 / 1000.0;
            v.height_m += dh;
            v.kept_m = v.kept_m.max(v.height_m);
            v.built_m += dh;
            v.peak_m = v.peak_m.max(v.height_m);
        }
        if record_bases {
            bases.sort_by(f64::total_cmp);
            let q = |f: f64| {
                if bases.is_empty() {
                    f64::NAN
                } else {
                    bases[((bases.len() - 1) as f64 * f).round() as usize]
                }
            };
            st.arc_bases.push(ArcBase {
                t_myr: t,
                cones: bases.len() as u32,
                base_p25_m: q(0.25),
                base_p50_m: q(0.5),
                base_p75_m: q(0.75),
                low_share: bases.iter().filter(|&&b| b < 500.0).count() as f64 // ledger: VOL-071
                    / bases.len().max(1) as f64,
            });
        }
        for v in st
            .volcanoes
            .iter()
            .filter(|v| v.kind == Kind::Strato && v.active())
        {
            if let Some(seg) = st.arcs.get_mut(&v.segment) {
                seg.last_myr = t;
                seg.peak_m = seg.peak_m.max(v.peak_m);
            }
        }
        self.time("build", started);

        // 6. Calderas.
        let started = Instant::now();
        for v in &mut st.volcanoes {
            if !v.active() || v.caldera.is_some() {
                continue;
            }
            let (rate, summit) = match v.kind {
                Kind::Strato if v.silicic => (p.caldera_rate_per_myr, false),
                Kind::Silicic => (3.0 * p.caldera_rate_per_myr, false), // ledger: VOL-045
                Kind::Shield if v.height_m >= 4000.0 * p.size => (p.caldera_rate_per_myr, true), // ledger: VOL-031
                _ => (0.0, false),
            };
            let u = rng.uniform();
            if rate <= 0.0 || u >= 1.0 - (-rate * dt).exp() || v.height_m < 300.0 {
                continue;
            }
            let before = v.shape();
            let (rc, depth_m) = if summit {
                let rc = 0.08 * v.radius_m; // ledger: VOL-031
                (rc, (0.04 * v.height_m).min(250.0)) // ledger: VOL-031
            } else {
                let rc = (range(&mut rng, p.caldera_radius_km[0], p.caldera_radius_km[1])
                    * 1000.0
                    * p.size)
                    .min(0.6 * v.radius_m);
                (rc, (0.5 * v.height_m).min(400.0 + 0.05 * rc)) // ledger: VOL-030
            };
            v.depression = Some(Depression {
                rho: rc / v.radius_m,
                depth: depth_m / v.height_m.max(1.0),
                caldera: true,
            });
            let after = v.shape();
            let h = v.height_m;
            let mut removed = 0.0;
            let diff = |r: f64| before.body(r) - after.body(r);
            for (c, wgt) in l1.render(v.x, v.z, v.radius_m, 1.0, &diff) {
                let dz = (h * wgt) as f32;
                l1.delta[c] -= dz;
                removed += f64::from(dz);
                log_ops.push((c, -dz, 0));
            }
            let removed_m3 = removed * cell_m * cell_m + std::f64::consts::PI * rc * rc * depth_m;
            if !summit {
                // The ignimbrite sheet to four rim radii (half the erupted volume) …
                let big = 4.0 * rc; // ledger: VOL-046
                let span = big - rc;
                let t0 = (0.5 * removed_m3 / (std::f64::consts::PI * span * (span / 3.0 + rc)))
                    .min(250.0);
                let (vx, vz) = (v.x, v.z);
                for_cells(&grid, mh, vx, vz, big, |c, dx, dz| {
                    let r = dx.hypot(dz);
                    if r > rc {
                        let th = (t0 * (1.0 - (r - rc) / span)) as f32;
                        if th > 0.0 {
                            l1.delta[c] += th;
                            log_ops.push((c, th, rocks.tuff));
                        }
                    } else {
                        // … and the fill inside the caldera (the rest; log only: the
                        // depression itself is cut at the finest grid).
                        let fill = (0.5 * removed_m3 / (std::f64::consts::PI * rc * rc))
                            .min(0.8 * depth_m) as f32;
                        log_ops.push((c, fill, rocks.tuff));
                    }
                });
                v.extinct_myr = Some(t);
                st.counts.calderas += 1;
                if let Some(seg) = st.arcs.get_mut(&v.segment) {
                    seg.calderas += 1;
                }
            } else {
                st.counts.summit_calderas += 1;
            }
            v.caldera = Some(Caldera {
                t_myr: t,
                x: v.x,
                z: v.z,
                radius_m: rc,
                depth_m,
                erupted_km3: removed_m3 / 1.0e9,
            });
        }
        self.time("calderas", started);

        // 7. Subsidence of dead oceanic volcanoes.
        let started = Instant::now();
        if p.island_subsidence_m_myr > 0.0 {
            for v in &mut st.volcanoes {
                if v.active() || !v.oceanic || v.sunk_m >= p.island_subsidence_max_m {
                    continue;
                }
                let sink =
                    (p.island_subsidence_m_myr * dt).min(p.island_subsidence_max_m - v.sunk_m);
                let kern = |r: f64| {
                    let x = r / 1.6; // ledger: VOL-048
                    if x >= 1.0 { 0.0 } else { (1.0 - x * x).powi(2) }
                };
                for (c, wgt) in l1.render(v.x, v.z, v.radius_m, 1.6, &kern) {
                    l1.delta[c] -= (sink * wgt) as f32;
                }
                v.sunk_m += sink;
            }
        }
        self.time("subsidence", started);

        // 8. Flood basalts from the plate model's new large igneous provinces.
        let started = Instant::now();
        let from = st.events_seen as usize;
        let mut new_lips: Vec<(EventId, f32, f32, f64)> = Vec::new();
        for e in world.events.iter().skip(from) {
            if e.kind != LARGE_IGNEOUS_PROVINCE {
                continue;
            }
            if let Place::Point { lat, lon } = e.place {
                let th = match e.params.get("thickness_m") {
                    Some(ParamValue::Float(x)) => *x,
                    _ => self.preset.tectonics.lip_thickness_m,
                };
                new_lips.push((e.id, lat, lon, th));
            }
        }
        st.events_seen = world.events.len() as u64;
        for (event, lat, lon, th) in new_lips {
            let x = frame.x_of_lon(f64::from(lon));
            let z = frame.z_of_lat(f64::from(lat));
            let c = cell_of(&grid, x, z);
            let flows = p.flood_basalt_flows;
            let plateau = |r: f64| 1.0 - smooth(0.6, 1.0, r); // ledger: VOL-035
            let dome = |r: f64| if r >= 1.0 { 0.0 } else { (1.0 - r * r).powi(2) };
            for_cells(&grid, mh, x, z, lip_radius_m, |cc, dx, dz| {
                let r = dx.hypot(dz) / lip_radius_m;
                let pile = th * plateau(r);
                if pile < 1.0 {
                    return;
                }
                let per = (pile / f64::from(flows)) as f32;
                for _ in 0..flows {
                    log_ops.push((cc, per * 0.88, rocks.basalt)); // ledger: VOL-060
                    log_ops.push((cc, per * 0.12, rocks.tuff));
                }
                l1.delta[cc] += ((th / 3.0) * (plateau(r) - dome(r))) as f32;
            });
            st.basalts.push(FloodBasalt {
                id: st.next_id,
                x,
                z,
                plate: plate_id[c].max(0.0) as u32,
                radius_m: lip_radius_m,
                thickness_m: th,
                t_myr: t,
                event,
                flows,
            });
            st.next_id += 1;
            st.counts.flood_basalts += 1;
        }
        self.time("flood_basalts", started);

        // 9. Retire the worn-out dead.
        let mut kept = Vec::with_capacity(st.volcanoes.len());
        for mut v in std::mem::take(&mut st.volcanoes) {
            let dead_for = v.extinct_myr.map_or(0.0, |e| t - e);
            if !v.active() && ((dead_for > 5.0 && v.height_m < 0.1 * v.peak_m) || dead_for > 250.0) // ledger: VOL-049
            {
                v.gone = Some(Gone::Eroded);
                st.counts.eroded += 1;
                gone_now.push(v);
            } else {
                kept.push(v);
            }
        }
        st.volcanoes = kept;
        for v in gone_now {
            // Stratovolcanoes that are gone leave no event (they are counted), unless a caldera.
            if v.kind != Kind::Strato || v.caldera.is_some() {
                st.gone.push(v);
            }
        }

        // Apply: the surface (offset, elevation, its fields) and the plate feedback; the log.
        let started = Instant::now();
        {
            let WorldState {
                fields, surface, ..
            } = world;
            let surf = surface.as_mut().expect("checked above");
            let mv = s.m_per_block_v;
            let delta = &l1.delta;
            for (c, &d) in delta.iter().enumerate() {
                if d != 0.0 {
                    surf.offset_m[c] += d;
                    surf.elevation_m[c] += d;
                }
            }
            let add = |fields: &mut BTreeMap<String, Field>, name: &str, scale: f32| {
                if let Some(f) = fields.get_mut(name)
                    && f.data.len() == n
                {
                    for (x, &d) in f.data.iter_mut().zip(delta) {
                        *x += d * scale;
                    }
                }
            };
            add(fields, crate::surface::field::FEEDBACK, 1.0);
            add(fields, crate::surface::field::ELEVATION_M, 1.0);
            add(fields, "elevation", (1.0 / mv) as f32);
            let age = (t - 0.5 * dt) as f32;
            let soil = index("soil");
            let (sap, lat, baux) = (index("saprolite"), index("laterite"), index("bauxite"));
            for &(c, th, class) in &log_ops {
                if th < 0.0 {
                    let left = f64::from(-th);
                    let take = f64::from(surf.regolith_m[c]).min(left);
                    surf.regolith_m[c] -= take as f32;
                    surf.log.erode(c, left - take);
                    continue;
                }
                if th <= 0.0 {
                    continue;
                }
                // Lava over regolith buries it as a palaeosol.
                if surf.regolith_m[c] >= 0.5 && th >= 1.0 { // ledger: VOL-061
                    let rk = match surf.regolith_kind[c] {
                        crate::surface::weathering::SAPROLITE => sap,
                        crate::surface::weathering::LATERITE => lat,
                        crate::surface::weathering::BAUXITE => baux,
                        _ => soil,
                    };
                    surf.log.deposit(c, age, surf.regolith_m[c], rk, IN_SITU);
                    surf.regolith_m[c] = 0.0;
                    surf.regolith_kind[c] = crate::surface::weathering::SOIL;
                }
                surf.log.deposit(c, age, th, class, IN_SITU);
            }
        }
        self.time("apply", started);

        // The map layer.
        let started = Instant::now();
        let map = self.map(
            &st,
            &grid,
            &world.surface.as_ref().expect("checked").elevation_m,
            sea,
        );
        world.fields.insert(
            field::MAP.to_owned(),
            Field {
                grid: grid.clone(),
                data: map,
            },
        );
        self.time("map", started);

        st.omega = omega_now;
        st.steps += 1;
        st.t_myr = t;
        st.sea_level_m = sea;
        st.fade_myr = p.fade_myr;
        if last {
            let started = Instant::now();
            self.finish(world, &mut st, &grid, dt)?;
            self.time("finish", started);
            let note = self.note(&st, world);
            ctx.log.push(crate::log::Level::Info, summary_line(&note));
            world.notes.insert(STAGE_NAME.into(), note.to_string());
        }
        world.volcanoes = Some(st);
        Ok(())
    }

    /// The map layer: flood basalts, then volcano footprints (0.6 R) by kind and state.
    fn map(&self, st: &VolcanoState, grid: &GridDesc, elev: &[f32], sea: f64) -> Vec<f32> {
        let n = grid.cells();
        let mh = self.preset.scale.m_per_block_h;
        let mut out = vec![f32::NAN; n];
        for b in &st.basalts {
            for_cells(grid, mh, b.x, b.z, 0.8 * b.radius_m, |c, _, _| {
                out[c] = code::FLOOD_BASALT;
            });
        }
        for v in &st.volcanoes {
            let summit = sample_at(grid, elev, v.x, v.z) - sea;
            let k = match (v.kind, v.active()) {
                (Kind::Strato | Kind::Silicic, _) if v.caldera.is_some() => code::CALDERA,
                (Kind::Strato, true) => code::STRATO_ACTIVE,
                (Kind::Strato, false) => code::STRATO_DEAD,
                (Kind::Shield, true) if summit > -500.0 => code::SHIELD_ACTIVE,
                (Kind::Shield, _) => code::SHIELD_DEAD,
                (Kind::Silicic, _) => code::SILICIC,
            };
            let reach = (0.6 * v.radius_m).max(0.75 * f64::from(grid.r) * mh);
            for_cells(grid, mh, v.x, v.z, reach, |c, _, _| out[c] = k);
        }
        out
    }

    /// The end of the run: the flood-basalt outcrop layer and the events.
    fn finish(
        &self,
        world: &mut WorldState,
        st: &mut VolcanoState,
        grid: &GridDesc,
        dt: f64,
    ) -> Result<(), CoreError> {
        let n = grid.cells();
        let s = &self.preset.scale;
        let mh = s.m_per_block_h;
        let rocks = Rocks::get();
        let sea = st.sea_level_m;
        let t = st.t_myr;
        // Flood-basalt outcrops: the pile's layers (basalt and tuff laid down at its time) left in
        // the log, where the first of them lies within 150 m of the surface.
        let mut fb = vec![0.0f32; n];
        let mut preserved: Vec<(f64, f64)> = Vec::new();
        let (greenstone, amphibolite) = (index("greenstone"), index("amphibolite"));
        {
            let surf = world.surface.as_ref().expect("checked by the caller");
            let half = (0.5 * dt) as f32 + 1e-3;
            for b in &st.basalts {
                let age = (b.t_myr - 0.5 * dt) as f32;
                let (mut on, mut pile, mut all) = (0usize, 0usize, 0usize);
                for_cells(grid, mh, b.x, b.z, b.radius_m, |c, _, _| {
                    all += 1;
                    let col = surf.log.column(c);
                    let mut depth = f64::from(surf.regolith_m[c]);
                    let mut first = f64::NAN;
                    let mut left = 0.0;
                    for l in col.iter().rev() {
                        // Its basalt and tuff, or the basalt metamorphosed (greenstone, amphibolite).
                        let ours = (l.class == rocks.basalt
                            || l.class == rocks.tuff
                            || l.class == greenstone
                            || l.class == amphibolite)
                            && (l.age_myr - age).abs() <= half;
                        if ours {
                            if first.is_nan() {
                                first = depth;
                            }
                            left += f64::from(l.thickness_m);
                        }
                        depth += f64::from(l.thickness_m);
                    }
                    if first.is_finite() {
                        pile += 1;
                    }
                    if first.is_finite() && first <= 150.0 && left >= 30.0 { // ledger: VOL-062
                        let wv = (left / 300.0).min(1.0) as f32;
                        fb[c] = fb[c].max(wv);
                        on += 1;
                    }
                });
                preserved.push((
                    on as f64 / all.max(1) as f64,
                    pile as f64 / all.max(1) as f64,
                ));
            }
        }
        world.fields.insert(
            field::FLOOD_BASALT.to_owned(),
            Field {
                grid: grid.clone(),
                data: fb,
            },
        );

        // Causes that exist only now: each hotspot's track (the plate model writes it when the
        // hotspot dies or the run ends).
        let mut tracks: BTreeMap<i64, EventId> = BTreeMap::new();
        for e in &world.events {
            if e.kind == HOTSPOT_TRACK
                && let Some(ParamValue::Int(h)) = e.params.get("hotspot")
            {
                tracks.insert(*h, e.id);
            }
        }
        let elev = world
            .surface
            .as_ref()
            .expect("checked by the caller")
            .elevation_m
            .clone();
        let point = |x: f64, z: f64| {
            let frame = BlockFrame::new(s.circumference_blocks);
            Place::Point {
                lat: frame.lat_deg(z) as f32,
                lon: frame.lon_deg(x) as f32,
            }
        };
        let myr = 1.0e6;
        let mut emitted = 0u64;
        let first_event = world.events.len();
        let frame = BlockFrame::new(s.circumference_blocks);
        let size = self.p.size;
        let standing = |v: &Volcano| v.gone.is_none() && v.height_m >= (0.25 * v.peak_m).max(300.0);
        let summit_of = |v: &Volcano| {
            if v.gone.is_none() {
                sample_at(grid, &elev, v.x, v.z) - sea
            } else {
                f64::NAN
            }
        };
        // Its cones as a detail: "lat,lon,radius_km,height_m" of each standing one, tallest first.
        let cones_param = |vs: &mut Vec<&Volcano>| -> String {
            vs.sort_by(|a, b| b.height_m.total_cmp(&a.height_m));
            vs.iter()
                .take(80)
                .map(|v| {
                    format!(
                        "{:.3},{:.3},{:.1},{:.0}",
                        frame.lat_deg(v.z),
                        frame.lon_deg(v.x),
                        v.radius_m / 1000.0,
                        v.height_m
                    )
                })
                .collect::<Vec<_>>()
                .join(";")
        };
        // An individual volcano's event (the notable ones), caused by its arc segment or chain.
        let volcano_event = |world: &mut WorldState, v: &Volcano, cause: Option<EventId>| {
            let summit = summit_of(v);
            let fate = match (v.gone, v.kind) {
                (Some(Gone::Subducted), _) => "subducted",
                (Some(Gone::Eroded), _) => "eroded",
                (None, _) if v.active() => "active",
                (None, Kind::Shield) if summit < 0.0 => "seamount",
                (None, Kind::Shield) => "island",
                (None, _) => "extinct",
            };
            let kind = match v.kind {
                Kind::Strato => kind::STRATOVOLCANO,
                Kind::Shield => kind::SHIELD,
                Kind::Silicic => kind::CALDERA_COMPLEX,
            };
            let end = v.extinct_myr.unwrap_or(t).max(v.born_myr);
            let setting = match (v.kind, v.oceanic) {
                (Kind::Strato, true) => "island_arc",
                (Kind::Strato, false) => "continental_arc",
                (_, true) => "oceanic_hotspot",
                (_, false) => "continental_hotspot",
            };
            let mut e = world
                .event(v.born_myr * myr, kind, SOURCE)
                .until(end * myr)
                .at(point(v.x, v.z))
                .magnitude((v.peak_m / 3000.0) as f32)
                .param("volcano", i64::from(v.id))
                .param("setting", setting)
                .param(
                    "magma",
                    match (v.kind, v.silicic) {
                        (Kind::Shield, _) => "basaltic",
                        (_, true) => "silicic",
                        _ => "andesitic",
                    },
                )
                .param("peak_height_m", v.peak_m.round())
                .param("height_m", v.height_m.round())
                .param("radius_km", (v.radius_m / 100.0).round() / 10.0)
                .param("plate", i64::from(v.plate))
                .param("fate", fate)
                .param("ongoing", i64::from(v.active()));
            if summit.is_finite() {
                e = e.param("summit_m", summit.round());
            }
            if let Some(h) = v.hotspot {
                e = e
                    .param("hotspot", i64::from(h))
                    .param("chain_index", i64::from(v.chain));
            } else if v.active_myr > 0.0 {
                e = e.param(
                    "arc_flux",
                    (v.flux_myr / v.active_myr * 100.0).round() / 100.0,
                );
            }
            if let Some(c) = cause {
                e = e.caused_by(c);
            }
            world.emit(e)
        };
        let all: Vec<Volcano> = st.volcanoes.iter().chain(st.gone.iter()).cloned().collect();
        // Events of the groups and of the notable volcanoes, by volcano id.
        let mut group_event: BTreeMap<u32, EventId> = BTreeMap::new();
        let mut notable: BTreeMap<u32, EventId> = BTreeMap::new();

        // 1. Arc segments: one event each, its standing cones as a detail; its tallest standing
        // cone (at least 1.5 km × size) also on its own. A segment that built fewer than
        // `SMALL_SEGMENT` cones is a count on its subduction onset instead (Ben, 2026-09-30, "Count
        // small ones"), with its standing cones listed there for "why here".
        let (mut small_segments, mut small_cones) = (0u64, 0u64);
        for (key, seg) in &st.arcs {
            let mut members: Vec<&Volcano> = all
                .iter()
                .filter(|v| v.kind == Kind::Strato && v.segment == *key && standing(v))
                .collect();
            if seg.born < SMALL_SEGMENT
                && let Some(onset) = seg.cause
                && let Some(ix) = world.events.iter().position(|e| e.id == onset)
            {
                small_segments += 1;
                small_cones += u64::from(seg.born);
                let cones = cones_param(&mut members);
                let e = &mut world.events[ix];
                let num = |e: &deeptime_format::Event, k: &str| match e.params.get(k) {
                    Some(ParamValue::Int(n)) => *n,
                    _ => 0,
                };
                let built = num(e, "volcano_cones_built") + i64::from(seg.born);
                let standing_n = num(e, "volcano_cones_standing") + members.len() as i64;
                e.params
                    .insert("volcano_cones_built".into(), ParamValue::Int(built));
                e.params
                    .insert("volcano_cones_standing".into(), ParamValue::Int(standing_n));
                if !cones.is_empty() {
                    let all_cones = match e.params.get("volcano_cones") {
                        Some(ParamValue::Str(old)) if !old.is_empty() => format!("{old};{cones}"),
                        _ => cones,
                    };
                    e.params
                        .insert("volcano_cones".into(), ParamValue::Str(all_cones));
                }
                for v in all
                    .iter()
                    .filter(|v| v.kind == Kind::Strato && v.segment == *key)
                {
                    group_event.insert(v.id, onset);
                }
                if let Some(top) = members.first().filter(|v| v.height_m >= 1500.0 * size) {
                    notable.insert(top.id, volcano_event(world, top, Some(onset))?);
                    emitted += 1;
                }
                continue;
            }
            let live = members.iter().filter(|v| v.active()).count();
            let (px, pz) = if members.is_empty() {
                let n = f64::from(seg.born.max(1));
                (frame.wrap_x(seg.ref_x + seg.sum_dx / n), seg.sum_z / n)
            } else {
                let rx = members[0].x;
                let n = members.len() as f64;
                let dx: f64 = members.iter().map(|v| frame.wrap_x(v.x - rx)).sum();
                (
                    frame.wrap_x(rx + dx / n),
                    members.iter().map(|v| v.z).sum::<f64>() / n,
                )
            };
            let tallest = members.iter().map(|v| v.height_m).fold(0.0f64, f64::max);
            let cones = cones_param(&mut members);
            let mut e = world
                .event(seg.first_myr * myr, kind::ARC, SOURCE)
                .until(seg.last_myr.max(seg.first_myr) * myr)
                .at(point(px, pz))
                .magnitude((f64::from(seg.born) / 50.0).min(1.0) as f32)
                .param(
                    "setting",
                    if seg.oceanic * 2 > seg.born {
                        "island_arc"
                    } else {
                        "continental_arc"
                    },
                )
                .param("cones_built", i64::from(seg.born))
                .param("live", live as i64)
                .param("standing", members.len() as i64)
                .param("tallest_m", tallest.round())
                .param("peak_m", seg.peak_m.round())
                .param("calderas", i64::from(seg.calderas))
                .param("plate", i64::from(seg.plate))
                .param("ongoing", i64::from(live > 0))
                .param("cones", cones);
            if let Some(c) = seg.cause {
                e = e.caused_by(c);
            }
            let id = world.emit(e)?;
            emitted += 1;
            for v in all
                .iter()
                .filter(|v| v.kind == Kind::Strato && v.segment == *key)
            {
                group_event.insert(v.id, id);
            }
            if let Some(top) = members.first().filter(|v| v.height_m >= 1500.0 * size) {
                notable.insert(top.id, volcano_event(world, top, Some(id))?);
                emitted += 1;
            }
        }

        // 2. Hotspot chains: one event each, its members as a detail; its tallest standing member
        // also on its own.
        let mut hotspots: BTreeMap<u32, Vec<&Volcano>> = BTreeMap::new();
        for v in all.iter().filter(|v| v.hotspot.is_some()) {
            hotspots.entry(v.hotspot.unwrap_or(0)).or_default().push(v);
        }
        for (h, vs) in &hotspots {
            let first = vs.iter().map(|v| v.born_myr).fold(f64::INFINITY, f64::min);
            let last = vs
                .iter()
                .map(|v| v.extinct_myr.unwrap_or(t))
                .fold(f64::NEG_INFINITY, f64::max);
            let youngest = vs
                .iter()
                .max_by(|a, b| a.born_myr.total_cmp(&b.born_myr))
                .expect("a chain has a volcano");
            let mut members: Vec<&Volcano> = vs.iter().copied().filter(|v| standing(v)).collect();
            let islands = members.iter().filter(|v| summit_of(v) > 0.0).count();
            let subducted = vs
                .iter()
                .filter(|v| v.gone == Some(Gone::Subducted))
                .count();
            let tallest = members.iter().map(|v| v.height_m).fold(0.0f64, f64::max);
            let live = vs.iter().filter(|v| v.gone.is_none() && v.active()).count();
            let cones = cones_param(&mut members);
            let cause = tracks
                .get(&i64::from(*h))
                .copied()
                .or(youngest.cause)
                .or(vs.iter().find_map(|v| v.cause));
            let mut e = world
                .event(first * myr, kind::HOTSPOT_CHAIN, SOURCE)
                .until(last.max(first) * myr)
                .at(point(youngest.x, youngest.z))
                .magnitude((vs.len() as f64 / 20.0).min(1.0) as f32)
                .param("hotspot", i64::from(*h))
                .param(
                    "volcanoes_built",
                    st.chains.get(h).map_or(vs.len() as i64, |&n| i64::from(n)),
                )
                .param("standing", members.len() as i64)
                .param("islands", islands as i64)
                .param("seamounts", (members.len() - islands) as i64)
                .param("subducted", subducted as i64)
                .param("tallest_m", tallest.round())
                .param("ongoing", i64::from(live > 0))
                .param("cones", cones);
            if let Some(c) = cause {
                e = e.caused_by(c);
            }
            let id = world.emit(e)?;
            emitted += 1;
            for v in vs {
                group_event.insert(v.id, id);
            }
            if let Some(top) = members.first() {
                notable.insert(top.id, volcano_event(world, top, Some(id))?);
                emitted += 1;
            }
        }

        // 3. Calderas: those whose volcano still stands, and every large one (≥ 6 km across × size).
        for v in &all {
            let Some(cal) = &v.caldera else {
                continue;
            };
            if !(standing(v) || cal.radius_m >= 3000.0 * size) { // ledger: VOL-014
                continue;
            }
            let cause = notable
                .get(&v.id)
                .or(group_event.get(&v.id))
                .copied()
                .or(v.cause);
            let mut e = world
                .event(cal.t_myr * myr, kind::CALDERA, SOURCE)
                .at(point(cal.x, cal.z))
                .magnitude((cal.erupted_km3 / 1000.0).min(1.0) as f32)
                .param("volcano", i64::from(v.id))
                .param("radius_km", (cal.radius_m / 100.0).round() / 10.0)
                .param("depth_m", cal.depth_m.round())
                .param("erupted_km3", cal.erupted_km3.round())
                .param(
                    "type",
                    if v.kind == Kind::Shield {
                        "summit"
                    } else {
                        "silicic"
                    },
                );
            if let Some(c) = cause {
                e = e.caused_by(c);
            }
            world.emit(e)?;
            emitted += 1;
        }
        for (b, &(kept, pile)) in st.basalts.iter().zip(&preserved) {
            let area = std::f64::consts::PI * (b.radius_m / 1000.0).powi(2);
            let e = world
                .event(b.t_myr * myr, kind::FLOOD_BASALT, SOURCE)
                .at(point(b.x, b.z))
                .magnitude((b.thickness_m / 1000.0) as f32)
                .param("area_km2", area.round())
                .param("thickness_m", b.thickness_m.round())
                .param("flows", i64::from(b.flows))
                .param("preserved_frac", (kept * 1000.0).round() / 1000.0)
                .param("pile_frac", (pile * 1000.0).round() / 1000.0)
                .param("plate", i64::from(b.plate))
                .caused_by(b.event);
            world.emit(e)?;
            emitted += 1;
        }
        st.events_emitted = emitted;
        st.counts.small_arc_segments = small_segments;
        st.counts.small_arc_cones = small_cones;
        st.events_by_kind = world.events[first_event..]
            .iter()
            .filter(|e| e.source == SOURCE)
            .fold(BTreeMap::new(), |mut m, e| {
                *m.entry(e.kind.trim_start_matches("deeptime:").to_owned())
                    .or_insert(0) += 1;
                m
            });
        Ok(())
    }

    /// The run summary (`notes.volcanoes`).
    fn note(&self, st: &VolcanoState, world: &WorldState) -> serde_json::Value {
        let standing = |k: Kind| {
            st.volcanoes
                .iter()
                .filter(|v| v.kind == k && v.height_m >= (0.25 * v.peak_m).max(300.0))
                .count()
        };
        let active = |k: Kind| {
            st.volcanoes
                .iter()
                .filter(|v| v.kind == k && v.active())
                .count()
        };
        let peak = st
            .volcanoes
            .iter()
            .map(|v| v.height_m)
            .fold(0.0f64, f64::max);
        let fb_on = world
            .fields
            .get(field::FLOOD_BASALT)
            .map_or(0, |f| f.data.iter().filter(|&&x| x > 0.0).count());
        // Height distributions for the comparison with Earth's composite volcanoes (science page
        // `volcanoes-and-magma.md`, VOL-019): the 10th, 50th and 90th percentiles, m.
        let pct = |mut xs: Vec<f64>| -> serde_json::Value {
            if xs.is_empty() {
                return json!(null);
            }
            xs.sort_by(f64::total_cmp);
            let q = |f: f64| xs[((xs.len() - 1) as f64 * f).round() as usize].round();
            json!([q(0.1), q(0.5), q(0.9)])
        };
        let heights = |pick: &dyn Fn(&Volcano) -> bool| {
            pct(st
                .volcanoes
                .iter()
                .filter(|v| pick(v))
                .map(|v| v.height_m)
                .collect())
        };
        let is_standing = |v: &Volcano| v.height_m >= (0.25 * v.peak_m).max(300.0);
        json!({
            "steps": st.steps,
            "counts": st.counts,
            "catalogue": st.volcanoes.len(),
            "standing": {
                "stratovolcano": standing(Kind::Strato),
                "shield": standing(Kind::Shield),
                "caldera_complex": standing(Kind::Silicic),
            },
            "active": {
                "stratovolcano": active(Kind::Strato),
                "shield": active(Kind::Shield),
                "caldera_complex": active(Kind::Silicic),
            },
            "tallest_m": peak.round(),
            "height_pcts_m": {
                "active_stratovolcano": heights(&|v| v.kind == Kind::Strato && v.active()),
                "standing_stratovolcano": heights(&|v| v.kind == Kind::Strato && is_standing(v)),
                "standing_shield": heights(&|v| v.kind == Kind::Shield && is_standing(v)),
            },
            "flood_basalt_outcrop_cells": fb_on,
            "events": st.events_emitted,
            "events_by_kind": st.events_by_kind,
            "arc_bases": st.arc_bases,
            "params": self.p,
            "millis": self.timings,
        })
    }
}

/// One step's arc births: the cells whose arc flux is at least `min_flux`, highest flux first
/// (ties in the stream's order), each at a random point of its cell and kept when no point of
/// `occupied` (which it then joins) lies within `spacing_m` × U(0.8, 1.2). Returns (x, z, cell).
pub fn place_arcs(
    grid: &GridDesc,
    mh: f64,
    flux: &[f64],
    min_flux: f64,
    spacing_m: f64,
    occupied: &mut Vec<(f64, f64)>,
    rng: &mut Rng,
) -> Vec<(f64, f64, usize)> {
    let frame = BlockFrame::new(grid.circumference() as u32);
    let w = grid.w as usize;
    let r = f64::from(grid.r);
    let mut cands: Vec<usize> = (0..flux.len()).filter(|&c| flux[c] >= min_flux).collect();
    shuffle(rng, &mut cands);
    cands.sort_by(|&a, &b| flux[b].total_cmp(&flux[a]));
    // A coarse bucket grid of the occupied points (cells of `spacing` blocks), so each candidate
    // looks only at its neighbourhood.
    let bucket = (1.2 * spacing_m / mh).max(r);
    let nbx = (frame.circumference() / bucket).floor().max(1.0) as i64;
    let key = |x: f64, z: f64| -> (i64, i64) {
        let bx = (((x + 0.5 * frame.circumference()) / bucket).floor() as i64).rem_euclid(nbx);
        (bx, (z / bucket).floor() as i64)
    };
    let mut grid_pts: BTreeMap<(i64, i64), Vec<(f64, f64)>> = BTreeMap::new();
    for &(x, z) in occupied.iter() {
        grid_pts.entry(key(x, z)).or_default().push((x, z));
    }
    let mut out = Vec::new();
    for c in cands {
        let (i, j) = ((c % w) as i64, (c / w) as i64);
        let bx = frame.wrap_x(grid.block_x(i) as f64 + (rng.uniform() - 0.5) * r);
        let bz = grid.block_z(j) as f64 + (rng.uniform() - 0.5) * r;
        let need = spacing_m * range(rng, 0.8, 1.2);
        let (kx, kz) = key(bx, bz);
        let mut clash = false;
        'scan: for dz in -1..=1 {
            for dx in -1..=1 {
                if let Some(pts) = grid_pts.get(&((kx + dx).rem_euclid(nbx), kz + dz)) {
                    for &(ox, oz) in pts {
                        let d = (frame.wrap_x(bx - ox) * mh).hypot((bz - oz) * mh);
                        if d < need {
                            clash = true;
                            break 'scan;
                        }
                    }
                }
            }
        }
        if clash {
            continue;
        }
        occupied.push((bx, bz));
        grid_pts.entry((kx, kz)).or_default().push((bx, bz));
        out.push((bx, bz, c));
    }
    out
}

/// The least-squares dH of `m[c] ≈ dH · w + b` over the cells (c, w) of a footprint: the change
/// of an edifice's height from a field of per-cell changes.
pub fn fit_change(cells: &[(usize, f64)], m: &[f32]) -> f64 {
    if cells.len() < 3 {
        return 0.0;
    }
    let k = cells.len() as f64;
    let wm = cells.iter().map(|&(_, w)| w).sum::<f64>() / k;
    let mm = cells.iter().map(|&(c, _)| f64::from(m[c])).sum::<f64>() / k;
    let (mut cov, mut var) = (0.0, 0.0);
    for &(c, w) in cells {
        cov += (w - wm) * (f64::from(m[c]) - mm);
        var += (w - wm) * (w - wm);
    }
    if var > 1e-9 { cov / var } else { 0.0 }
}

/// An arc segment that built fewer cones than this is a count on its subduction onset event, not an
/// event of its own (Ben, 2026-09-30, "Count small ones").
pub const SMALL_SEGMENT: u32 = 3; // ledger: VOL-015

/// Arc crust deeper than this below the sea makes no volcano (a trench, not an arc), m.
pub const FRONT_MAX_DEPTH_M: f64 = 4000.0; // ledger: VOL-004

/// The volcanic front of the arcs: the medial line of the cells with at least `min_flux` (and
/// not deeper than [`FRONT_MAX_DEPTH_M`]). The plate model flags as arc a band a few L0 samples
/// wide along every active margin and broad patches where intra-oceanic arcs overlap, striped by
/// its lattice; real arcs line their volcanoes along one front. The mask is closed (a 3 × 3
/// dilation, then erosion) so the stripes of a patch merge, its distance to the mask's edge is
/// taken (chamfer 3-4, x wrapping), and the front is every masked cell at a local maximum of that
/// distance with a masked cell within one cell of it: the band's centre line, a patch's skeleton.
pub fn volcanic_front(
    grid: &GridDesc,
    flux: &[f64],
    elev: &[f32],
    sea: f64,
    min_flux: f64,
) -> Vec<bool> {
    let (w, h) = (grid.w as usize, grid.h as usize);
    let n = w * h;
    let mask: Vec<bool> = (0..n)
        .map(|c| flux[c] >= min_flux && f64::from(elev[c]) >= sea - FRONT_MAX_DEPTH_M)
        .collect();
    let at = |i: i64, j: i64| -> Option<usize> {
        (j >= 0 && j < h as i64).then(|| j as usize * w + i.rem_euclid(w as i64) as usize)
    };
    let morph = |src: &[bool], any: bool| -> Vec<bool> {
        (0..n)
            .map(|c| {
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                let mut hit = !any;
                for dj in -1..=1 {
                    for di in -1..=1 {
                        if let Some(m) = at(i + di, j + dj) {
                            if any && src[m] {
                                hit = true;
                            } else if !any && !src[m] {
                                hit = false;
                            }
                        }
                    }
                }
                hit
            })
            .collect()
    };
    let closed = morph(&morph(&mask, true), false);
    // Distance to the nearest cell outside the closed mask.
    let inf = f32::INFINITY;
    let mut dist: Vec<f32> = closed.iter().map(|&m| if m { inf } else { 0.0 }).collect();
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    for _ in 0..2 {
        for j in 0..h as i64 {
            for i in 0..w as i64 {
                let c = j as usize * w + i as usize;
                for (di, dj, s) in [(-1, 0, a), (-1, -1, b), (0, -1, a), (1, -1, b)] {
                    if let Some(m) = at(i + di, j + dj) {
                        dist[c] = dist[c].min(dist[m] + s);
                    }
                }
            }
        }
        for j in (0..h as i64).rev() {
            for i in (0..w as i64).rev() {
                let c = j as usize * w + i as usize;
                for (di, dj, s) in [(1, 0, a), (1, 1, b), (0, 1, a), (-1, 1, b)] {
                    if let Some(m) = at(i + di, j + dj) {
                        dist[c] = dist[c].min(dist[m] + s);
                    }
                }
            }
        }
    }
    (0..n)
        .map(|c| {
            if !closed[c] {
                return false;
            }
            let (i, j) = ((c % w) as i64, (c / w) as i64);
            let d = dist[c];
            let near_mask =
                (-1..=1).any(|dj| (-1..=1).any(|di| at(i + di, j + dj).is_some_and(|m| mask[m])));
            near_mask
                && d.is_finite()
                && (-1..=1).all(|dj| {
                    (-1..=1).all(|di| at(i + di, j + dj).is_none_or(|m| dist[m] <= d + 1e-3))
                })
        })
        .collect()
}

/// Fisher–Yates with the stage's stream.
fn shuffle<T>(rng: &mut Rng, v: &mut [T]) {
    for i in (1..v.len()).rev() {
        let j = rng.below(i as u64 + 1) as usize;
        v.swap(i, j);
    }
}

/// Uniform in [lo, hi).
fn range(rng: &mut Rng, lo: f64, hi: f64) -> f64 {
    lo + (hi - lo) * rng.uniform()
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `volcanoes_final=<steps> strato=… shield=… …` (the CLI's line), from `notes.volcanoes`.
pub fn summary_line(v: &serde_json::Value) -> String {
    let c = &v["counts"];
    // The arc bases with cones, after the first 100 Myr (the start has no arcs of its own yet).
    let bases: Vec<&serde_json::Value> = v["arc_bases"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|b| b["cones"].as_u64().unwrap_or(0) > 0)
                .collect()
        })
        .unwrap_or_default();
    let t0 = bases
        .first()
        .and_then(|b| b["t_myr"].as_f64())
        .unwrap_or(0.0);
    let later: Vec<&&serde_json::Value> = bases
        .iter()
        .filter(|b| b["t_myr"].as_f64().unwrap_or(0.0) - t0 >= 100.0)
        .collect();
    let mean_of = |k: &str| {
        let xs: Vec<f64> = later.iter().filter_map(|b| b[k].as_f64()).collect();
        xs.iter().sum::<f64>() / xs.len().max(1) as f64
    };
    let last_of = |k: &str| bases.last().and_then(|b| b[k].as_f64()).unwrap_or(f64::NAN);
    let pcts = |k: &str| {
        v["height_pcts_m"][k].as_array().map_or_else(
            || "-".to_string(),
            |a| {
                a.iter()
                    .map(|x| format!("{}", x.as_f64().unwrap_or(f64::NAN)))
                    .collect::<Vec<_>>()
                    .join("/")
            },
        )
    };
    format!(
        "volcanoes_final={} strato_born={} shield_born={} caldera_complex_born={} \
         standing_strato={} standing_shield={} standing_caldera_complex={} active_strato={} \
         active_shield={} calderas={} summit_calderas={} flood_basalts={} eroded={} subducted={} \
         built_km3={:.3e} tallest_m={} flood_basalt_outcrop_cells={} events={} \
         arc_base_p50_mean_m={:.0} arc_base_low_share_mean={:.3} arc_base_p50_final_m={:.0} \
         height_p10_p50_p90_m: active_strato={} standing_strato={} standing_shield={}",
        v["steps"],
        c["strato_born"],
        c["shield_born"],
        c["silicic_born"],
        v["standing"]["stratovolcano"],
        v["standing"]["shield"],
        v["standing"]["caldera_complex"],
        v["active"]["stratovolcano"],
        v["active"]["shield"],
        c["calderas"],
        c["summit_calderas"],
        c["flood_basalts"],
        c["eroded"],
        c["subducted"],
        c["built_km3"].as_f64().unwrap_or(0.0),
        v["tallest_m"],
        v["flood_basalt_outcrop_cells"],
        v["events"],
        mean_of("base_p50_m"),
        mean_of("low_share"),
        last_of("base_p50_m"),
        pcts("active_stratovolcano"),
        pcts("standing_stratovolcano"),
        pcts("standing_shield"),
    )
}

impl Stage for VolcanoStage {
    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn estimate(&self) -> StageEstimate {
        estimate(&self.preset)
    }

    fn total_steps(&self) -> u64 {
        self.total
    }

    fn steps_done(&self) -> u64 {
        self.done
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if self.done >= self.total {
            return Ok(());
        }
        ctx.pool.install(|| self.advance(world, ctx))?;
        self.done += 1;
        Ok(())
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        &KEYFRAME_LAYERS
    }

    fn coupled_to(&self) -> Option<&'static str> {
        Some(crate::plates::STAGE_NAME)
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&Saved { done: self.done })
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        let s: Saved = from_state(bytes)?;
        self.done = s.done;
        Ok(())
    }
}
