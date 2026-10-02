// SPDX-License-Identifier: LGPL-3.0-or-later
//! Meteor impacts (work item P2-08; Ben, decisions log 2026-09-29, "Yes, in full … not in v1, but
//! before phase 3"): impactors arrive at an Earth-like, size-dependent rate, dig simple and complex
//! craters, lay breccia and impact melt, and the craters then age with the land they sit on:
//! eroded, buried, or carried down a subduction zone with their crust.
//!
//! # Where it runs
//! A stage coupled to the plates (`Stage::coupled_to`), after the volcanoes and before the
//! structure stage: `[plates, climate, surface, volcanoes, impacts, structure]`, one step per
//! plate step. The craters are a catalogue of discrete landforms (block position, size, state)
//! that ride their plates like the volcanoes and the plutons; the world keeps it
//! (`WorldState::impacts`).
//!
//! # One step
//! 1. **Motion.** Every crater turns with the Ω of its plate; oceanic crust carried under another
//!    plate takes its craters with it (subducted).
//! 2. **Degradation**, by what the surface stage just did at the crater
//!    (`surface/material_m`, its net erosion and deposition of the step, averaged inside the
//!    rim): deposition lays a **cover** over the floor, erosion first strips that cover and then
//!    **wears** the crater itself, rim first. A crater keeps the share
//!    1 − (cover + worn) / depth of its relief. Between the grid cells, hillslope diffusion
//!    rounds it (`shape.rs`: Σ κ·t accumulates on land and in water shallower than the wave
//!    base). A crater worn through its whole depth and its breccia lens is gone (eroded).
//! 3. **Impacts.** The number of impactors of the step is a Poisson draw with mean
//!    rate(≥ `min_crater_km`) × `flux` × the planet's area × Δt; each strikes a point drawn
//!    uniformly on the sphere, with a size drawn from the production function N(≥ D) ∝ D^−b
//!    between `min_crater_km` and `max_crater_km` (the crater it would make on Earth's land,
//!    which fixes the impactor: `scaling.rs`). The crater it makes here follows from this planet's
//!    gravity and the depth of the sea where it lands: deep water stops a small impactor (no
//!    crater at all) and shrinks the crater of a larger one. A new crater destroys the smaller
//!    ones inside it.
//! 4. **Landform.** A crater at least two L1 cells across is cut into the deep-time surface
//!    (the surface's elevation and offset, and the plate feedback, as the volcanoes do), so rivers,
//!    lakes and sediment see it and the surface stage erodes and fills it like any other basin;
//!    the amplifier adds only what L1 cannot hold. A smaller crater is too small for L1: the
//!    amplifier draws all of it (`amplify/impact.rs`), from this catalogue, with the relief the
//!    accounting of step 2 left it.
//! 5. **Rock.** In the strata log of a crater cut into L1: the target rock is removed down to
//!    the base of the breccia lens, the lens laid as `impact_breccia`, the melt sheet (complex
//!    craters with at least 10 m of melt) as `impact_melt` on the floor, and the ejecta blanket
//!    as `impact_breccia` outside the rim. A smaller crater's lens and melt become a body at the
//!    end (`WorldState::bodies`, role `impact_breccia`, with an `impact_melt` zone).
//! 6. **Deposit fields.** Whenever the surface has closed a frame of the recorded history, the
//!    stage adds the four impact fields to it (`FIELD_*`; `deposits/fields.rs`), which the
//!    impact deposit types read, and it keeps `impacts/trap` for the structure stage's
//!    `trap_structure` (rim anticlines and central uplifts are petroleum traps).
//!
//! Events (`deeptime:impact_crater`, `deeptime:impact_ocean`; `EVENTS.md`) are written when the
//! impact happens, for craters of at least [`EVENT_MIN_KM`], and completed at the end with what
//! became of each (and where its crust has carried it); a smaller crater that still shows at the
//! end gets its event then. Every draw comes from a stream seeded by the preset seed and the step,
//! and every loop runs in catalogue order, so a run is the same on any thread count and after a
//! resume.

pub mod params;
pub mod scaling;
pub mod shape;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_format::bodies::NO_CLIP;
use deeptime_format::{Body, EventId, GridDesc, ParamValue, Place, Shape, Zone};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use params::ImpactParams;
pub use scaling::Morphology;
pub use shape::Profile;

use crate::deposits::random::Rng;
use crate::error::CoreError;
use crate::params::PlanetPreset;
use crate::plates::kinematics::rotation;
use crate::plates::state::CONTINENTAL;
use crate::sphere::BlockFrame;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::surface::lithology::index;
use crate::surface::strata_log::IN_SITU;
use crate::surface::volcanoes::shape::cell_weight;
use crate::surface::volcanoes::{cell_of, for_cells};
use crate::world::{Field, WorldState};

/// The stage's name in a pipeline (`deeptime stage impacts`).
pub const STAGE_NAME: &str = "impacts";

/// The source of every impact event.
pub const SOURCE: &str = "deeptime:impacts";

/// Event kinds (`crates/deeptime-format/EVENTS.md`, "The impacts' events").
pub mod kind {
    /// An impact that left a crater.
    pub const CRATER: &str = "deeptime:impact_crater";
    /// A large impact into deep water that left no crater in the sea floor.
    pub const OCEAN: &str = "deeptime:impact_ocean";
}

/// Fields the stage writes.
pub mod field {
    /// Where the craters are (codes, NaN elsewhere): 1 a fresh crater, 2 a degraded one, 3 a
    /// buried one, 4 the eroded roots of one.
    pub const MAP: &str = "impacts/map";
    /// 0–1: structural closure at craters (rim anticlines, central uplifts), which the structure
    /// stage folds into `deeptime:trap_structure`.
    pub const TRAP: &str = "impacts/trap";
}

/// The deposit engine's impact fields (`deposits/fields.rs`), added to every frame.
pub const FIELD_MELT: &str = "deeptime:impact_melt_m";
pub const FIELD_HYDROTHERMAL: &str = "deeptime:impact_hydrothermal";
pub const FIELD_SHOCK: &str = "deeptime:impact_shock";
pub const FIELD_BASIN: &str = "deeptime:impact_basin";

/// Keyframe layers.
pub const KEYFRAME_LAYERS: [&str; 1] = [field::MAP];

/// Codes of the map layer.
pub mod code {
    pub const FRESH: f32 = 1.0;
    pub const DEGRADED: f32 = 2.0;
    pub const BURIED: f32 = 3.0;
    pub const ROOTS: f32 = 4.0;
}

/// Craters at least this wide get an event when they form, km.
pub const EVENT_MIN_KM: f64 = 5.0;
/// An impactor the sea stopped gets an event when its crater on land would have been at least
/// this wide, km.
pub const OCEAN_EVENT_MIN_KM: f64 = 20.0;
/// A crater shows in the finished terrain while its relief is at least this (two blocks at 20 m
/// per block), m.
pub const VISIBLE_RELIEF_M: f64 = 40.0;
/// Diffusion works above this depth below the sea (the surface stage's wave base), m.
pub const WAVE_BASE_M: f64 = 60.0;
/// Target (crust) density of the crater scaling, kg/m³ (2,500–2,750 for sedimentary to
/// crystalline rock: Collins, Melosh & Marcus 2005).
pub const TARGET_DENSITY_KG_M3: f64 = 2700.0;
/// Lower edges of the size classes of the summary, km (final diameter); the first class is
/// everything smaller than the second edge.
pub const SIZE_EDGES_KM: [f64; 6] = [0.0, 5.0, 10.0, 20.0, 50.0, 100.0];
/// Their labels.
pub const SIZE_LABELS: [&str; 6] = [
    "under_5_km",
    "5_to_10_km",
    "10_to_20_km",
    "20_to_50_km",
    "50_to_100_km",
    "over_100_km",
];

/// The size class of a diameter.
pub fn size_class(diameter_km: f64) -> usize {
    SIZE_EDGES_KM
        .iter()
        .rposition(|&e| diameter_km >= e)
        .unwrap_or(0)
}

/// What became of a crater that left the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gone {
    /// Worn away through its breccia lens.
    Eroded,
    /// Carried down a subduction zone with its crust.
    Subducted,
    /// Destroyed by a larger, later crater.
    Overprinted,
}

/// What a crater is at the end of a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fate {
    /// Its bowl stands nearly as it formed.
    Fresh,
    /// Its bowl still shows, worn or part filled.
    Degraded,
    /// Sediment covers it.
    Buried,
    /// Erosion has taken its relief; its breccia remains.
    Roots,
}

impl Fate {
    pub fn name(self) -> &'static str {
        match self {
            Fate::Fresh => "fresh",
            Fate::Degraded => "degraded",
            Fate::Buried => "buried",
            Fate::Roots => "eroded_to_roots",
        }
    }
}

/// One crater.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crater {
    pub id: u32,
    /// Block position of its centre now (x in [−C/2, C/2)).
    pub x: f64,
    pub z: f64,
    /// The plate it rides.
    pub plate: u32,
    pub oceanic: bool,
    pub t_myr: f64,
    /// Where it struck, degrees.
    pub lat0: f32,
    pub lon0: f32,
    /// Depth of the sea where it struck, m (0 on land).
    pub water_m: f64,
    /// The impactor's diameter, m, and its energy, J.
    pub impactor_m: f64,
    pub energy_j: f64,
    /// Rim-to-rim diameter and transient diameter, m.
    pub diameter_m: f64,
    pub transient_m: f64,
    /// Its fresh shape in game metres (vertical measures × `depth_scale`).
    pub profile: Profile,
    /// Thickness of the breccia lens and of the melt sheet, game metres.
    pub breccia_m: f64,
    pub melt_m: f64,
    /// The melt sheet's real thickness, m, and the melt's volume, km³.
    pub melt_real_m: f64,
    pub melt_km3: f64,
    /// Cut into the deep-time surface on L1 (at least two cells across).
    pub imprinted: bool,
    /// Sediment over its floor, m.
    pub cover_m: f64,
    /// Erosion of the crater itself, m.
    pub worn_m: f64,
    /// Σ κ·t of hillslope diffusion, m².
    pub kt_m2: f64,
    /// Its relief and state as of the last time they were worked out (every step for the map,
    /// and at the end).
    pub relief_m: f64,
    pub fate: Fate,
    pub event: Option<EventId>,
    /// Set when it left the catalogue.
    pub gone: Option<Gone>,
}

impl Crater {
    pub fn radius_m(&self) -> f64 {
        self.profile.radius_m
    }

    pub fn extent_m(&self) -> f64 {
        self.profile.extent_m()
    }

    /// The share of its relief the surface's erosion and deposition have left it, 0–1.
    pub fn kept(&self) -> f64 {
        (1.0 - (self.cover_m + self.worn_m) / self.profile.depth_m.max(1e-6)).clamp(0.0, 1.0)
    }

    /// The half-width of the tent its diffusion amounts to, m.
    pub fn spread_m(&self) -> f64 {
        shape::spread_m(self.kt_m2)
    }

    /// Its relief now (rim crest to floor), m, and its state.
    pub fn state(&self) -> (f64, Fate) {
        let rim_rho = 1.0 / self.profile.ejecta_radii;
        let aged = self.profile.aged(self.spread_m());
        let relief = self.kept() * shape::relief_m(&aged, rim_rho);
        let fate = if relief >= VISIBLE_RELIEF_M {
            if relief >= 0.75 * self.profile.depth_m {
                Fate::Fresh
            } else {
                Fate::Degraded
            }
        } else if self.cover_m > 0.0 && self.cover_m >= self.worn_m {
            Fate::Buried
        } else {
            Fate::Roots
        };
        (relief, fate)
    }

    /// Depth of the original floor (the top of the breccia lens) below the crater's present
    /// floor, m: the cover, or negative by how far erosion has cut into the lens.
    pub fn lens_top_below_m(&self) -> f64 {
        self.cover_m - (self.worn_m - self.profile.depth_m).max(0.0)
    }

    /// The lens and melt worn through.
    fn worn_out(&self) -> bool {
        self.worn_m > self.profile.depth_m + self.breccia_m
    }
}

/// Counters for the summary, by size class ([`SIZE_LABELS`]).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Counts {
    /// Impactors drawn.
    pub impactors: u64,
    /// Craters formed, on land and in the sea floor.
    pub formed: [u64; 6],
    pub formed_in_sea: [u64; 6],
    /// Impactors the sea stopped, by the crater they would have made on land.
    pub stopped_by_sea: [u64; 6],
    pub eroded: [u64; 6],
    pub subducted: [u64; 6],
    pub overprinted: [u64; 6],
    /// Craters cut into the deep-time surface.
    pub imprinted: u64,
    /// The largest crater formed, km.
    pub largest_km: f64,
}

/// The catalogue, kept in `WorldState::impacts` (checkpoints carry it; the amplifier reads it).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImpactState {
    /// Craters in the catalogue, in id order.
    pub craters: Vec<Crater>,
    /// Craters with an event that left the catalogue (their events are completed at the end).
    pub gone: Vec<Crater>,
    pub next_id: u32,
    /// Every plate's Ω at the end of the last step.
    pub omega: Vec<[f64; 3]>,
    pub counts: Counts,
    pub steps: u64,
    /// Time of the last step, Myr.
    pub t_myr: f64,
    /// Sea level of the last step, m.
    pub sea_level_m: f64,
    /// Frames of the recorded history the stage has published into, and the time of the last.
    pub frames_seen: u64,
    pub frame_t_myr: f64,
    /// Events written, and bodies made at the end.
    pub events_emitted: u64,
    pub bodies: u64,
}

/// Estimate: the catalogue is small; a few L1 arrays per step.
pub fn estimate(p: &PlanetPreset) -> StageEstimate {
    let c = u64::from(p.scale.circumference_blocks);
    let r = u64::from(p.resolution.l1_r);
    let cells = (c / r) * (c / r / 2);
    StageEstimate {
        bytes: cells * 24 + (1 << 20),
        seconds: cells as f64 * p.plate_steps() as f64 * 0.005e-6,
        history_bytes: 0,
    }
}

/// The scaling's setting for a preset.
pub fn setting(preset: &PlanetPreset, p: &ImpactParams) -> scaling::Setting {
    scaling::Setting {
        gravity: preset.planet.gravity_m_s2,
        speed: p.speed_km_s * 1000.0,
        impactor_density: p.impactor_density_kg_m3,
        target_density: TARGET_DENSITY_KG_M3,
        sea_cutoff: p.sea_cutoff_ratio,
    }
}

/// The planet's area, km² (a sphere of the preset's radius).
pub fn planet_area_km2(preset: &PlanetPreset) -> f64 {
    let r = preset.scale.radius_m() / 1000.0;
    4.0 * std::f64::consts::PI * r * r
}

/// Impactors expected per Myr: the production rate at `min_crater_km` × `flux` × the area.
pub fn impactors_per_myr(preset: &PlanetPreset, p: &ImpactParams) -> f64 {
    scaling::earth_rate_per_km2_yr(p.min_crater_km * 1000.0, p.size_slope)
        * p.flux
        * planet_area_km2(preset)
        * 1.0e6
}

/// A crater's game-metre shape and rock from its physical dimensions.
pub fn build(c: &scaling::Crater, p: &ImpactParams) -> (Profile, f64, f64) {
    let s = p.depth_scale;
    let profile = Profile {
        morphology: c.morphology,
        radius_m: 0.5 * c.diameter_m,
        ejecta_radii: p.ejecta_radii,
        depth_m: c.depth_m * s,
        rim_m: c.rim_m * s,
        peak_m: c.peak_m * s,
    };
    (profile, c.breccia_m * s, c.melt_m * s)
}

#[derive(Serialize, Deserialize)]
struct Saved {
    done: u64,
}

/// The impact stage.
pub struct ImpactStage {
    preset: PlanetPreset,
    p: ImpactParams,
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

/// Bilinear value of an L1 array at block (x, z).
pub fn sample_at(grid: &GridDesc, v: &[f32], x: f64, z: f64) -> f64 {
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

/// The mean of an L1 array inside a crater's rim (the value at its centre for a crater smaller
/// than a cell).
fn mean_inside(grid: &GridDesc, mh: f64, cell_m: f64, v: &[f32], c: &Crater) -> f64 {
    let r = c.radius_m();
    if r < cell_m {
        return sample_at(grid, v, c.x, c.z);
    }
    let (mut s, mut n) = (0.0, 0u32);
    for_cells(grid, mh, c.x, c.z, r, |cell, _, _| {
        s += f64::from(v[cell]);
        n += 1;
    });
    if n == 0 {
        sample_at(grid, v, c.x, c.z)
    } else {
        s / f64::from(n)
    }
}

/// Calls `f(cell)` for every L1 cell within `reach_m` of the crater's centre, and for the cell
/// under the centre whatever the reach.
fn footprint(grid: &GridDesc, mh: f64, c: &Crater, reach_m: f64, mut f: impl FnMut(usize)) {
    let centre = cell_of(grid, c.x, c.z);
    f(centre);
    for_cells(grid, mh, c.x, c.z, reach_m, |cell, _, _| {
        if cell != centre {
            f(cell);
        }
    });
}

/// The four deposit fields of a frame: the melt, the hydrothermal systems and the shock of the
/// craters formed after `since_myr`, and the basins of every crater standing.
///
/// - `impact_melt_m`: the melt sheet's real thickness, m, over the crater's floor (inside
///   0.55 R). Sudbury's sheet was 2.5–3 km thick and differentiated into the norite whose base
///   holds the Ni-Cu-PGE sulphides (Naldrett 2004, *Magmatic Sulfide Deposits*; Grieve, Stöffler
///   & Deutsch 1991, JGR 96:22753–22764).
/// - `impact_hydrothermal`: 0–1 inside the rim, (D − 4 km) / 36 km: the melt and the central
///   uplift drive hot-water circulation for 10⁴–10⁶ years, longer in larger craters (Abramov &
///   Kring 2004, JGR 109:E10007; Osinski et al. 2013, Icarus 224:347–363).
/// - `impact_shock`: 0–1 inside 0.7 R, D / 100 km: shock pressures over ~35 GPa turn graphite in
///   the target into diamond (Popigai: Masaitis 1998, Meteoritics & Planet. Sci. 33:349–359).
/// - `impact_basin`: 1 inside the rim of every crater at least 5 km across whose structure
///   remains (a basin that traps sediment, and with it uranium and petroleum).
pub fn frame_fields(
    grid: &GridDesc,
    mh: f64,
    craters: &[Crater],
    since_myr: f64,
) -> BTreeMap<String, Vec<f32>> {
    let n = grid.cells();
    let (mut melt, mut hydro, mut shock, mut basin) = (
        vec![0.0f32; n],
        vec![0.0f32; n],
        vec![0.0f32; n],
        vec![0.0f32; n],
    );
    for c in craters {
        let r = c.radius_m();
        let d_km = c.diameter_m / 1000.0;
        if d_km >= EVENT_MIN_KM && c.worn_m < c.profile.depth_m {
            footprint(grid, mh, c, r, |cell| basin[cell] = 1.0);
        }
        if c.t_myr <= since_myr {
            continue;
        }
        if c.melt_real_m > 0.0 {
            let m = c.melt_real_m as f32;
            footprint(grid, mh, c, shape::FLOOR_RHO * r, |cell| {
                melt[cell] = melt[cell].max(m);
            });
        }
        let h = ((d_km - 4.0) / 36.0).clamp(0.0, 1.0) as f32;
        if h > 0.0 {
            footprint(grid, mh, c, r, |cell| hydro[cell] = hydro[cell].max(h));
        }
        let s = (d_km / 100.0).clamp(0.0, 1.0) as f32;
        footprint(grid, mh, c, 0.7 * r, |cell| shock[cell] = shock[cell].max(s));
    }
    BTreeMap::from([
        (FIELD_MELT.to_owned(), melt),
        (FIELD_HYDROTHERMAL.to_owned(), hydro),
        (FIELD_SHOCK.to_owned(), shock),
        (FIELD_BASIN.to_owned(), basin),
    ])
}

/// The closure craters add to `trap_structure` (`impacts/trap`): 0.7 inside the rim of a complex
/// crater whose structure remains (Ames, Red Wing Creek and the Cantarell field's breccia are
/// impact-structure oil fields: Donofrio 1998, Oil & Gas J. 96(19):69–83; Grieve 2005, Geol. Soc.
/// Spec. Publ. 248:1–29).
pub fn trap_field(grid: &GridDesc, mh: f64, craters: &[Crater]) -> Vec<f32> {
    let mut out = vec![0.0f32; grid.cells()];
    for c in craters {
        if c.profile.morphology != Morphology::Simple && c.worn_m < c.profile.depth_m {
            footprint(grid, mh, c, c.radius_m(), |cell| out[cell] = 0.7);
        }
    }
    out
}

/// The map layer: every crater's footprint inside its rim by state, larger craters first so a
/// small one inside a large one shows.
pub fn map(grid: &GridDesc, mh: f64, craters: &[Crater]) -> Vec<f32> {
    let mut out = vec![f32::NAN; grid.cells()];
    let mut order: Vec<&Crater> = craters.iter().collect();
    order.sort_by(|a, b| b.diameter_m.total_cmp(&a.diameter_m).then(a.id.cmp(&b.id)));
    for c in order {
        let k = match c.fate {
            Fate::Fresh => code::FRESH,
            Fate::Degraded => code::DEGRADED,
            Fate::Buried => code::BURIED,
            Fate::Roots => code::ROOTS,
        };
        footprint(grid, mh, c, c.radius_m(), |cell| out[cell] = k);
    }
    out
}

/// A crater's breccia lens (and melt sheet) as a body: an ellipsoid about the original floor,
/// `floor_y` (blocks), so its lower half is the lens under the floor and its upper half lines
/// the walls. `None` when it is thinner than a block or lies below the world.
pub fn to_body(
    c: &Crater,
    id: u32,
    floor_y: f64,
    m_per_block_h: f64,
    m_per_block_v: f64,
    min_y: i32,
) -> Option<Body> {
    let b = c.breccia_m / m_per_block_v;
    if b < 1.0 || floor_y + b < f64::from(min_y) {
        return None;
    }
    let a = (0.95 * c.radius_m() / m_per_block_h).max(1.0);
    let mut body = Body::new(
        id,
        None,
        Shape::Lens,
        "impact_breccia",
        [c.x, floor_y, c.z],
        [a as f32, b as f32, a as f32],
        0.0,
        0.0,
        NO_CLIP,
    );
    body.rock_class = Some("impact_breccia".into());
    let melt = c.melt_m / m_per_block_v;
    if melt >= 0.5 {
        // The sheet lies on the floor: from the body's middle (v = 0.5) down.
        body.zones.push(Zone {
            name: "melt_sheet".into(),
            material: None,
            rock_class: Some("impact_melt".into()),
            d: [0.0, shape::FLOOR_RHO as f32],
            v: [0.5, (0.5 + melt / (2.0 * b)).min(1.0) as f32],
            grade: 0.0,
            density: 0.0,
        });
    }
    Some(body)
}

/// `impacts/craters.json` (FORMAT.md, "Impact craters"): the craters of the catalogue at the
/// present, in id order.
pub const CRATERS_PATH: &str = "impacts/craters.json";
/// Its version.
pub const CRATERS_VERSION: u32 = 1;

/// The contents of [`CRATERS_PATH`].
pub fn present_craters(st: &ImpactState) -> Value {
    let craters: Vec<Value> = st
        .craters
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "x": (c.x * 10.0).round() / 10.0,
                "z": (c.z * 10.0).round() / 10.0,
                "diameter_m": c.diameter_m.round(),
                "morphology": c.profile.morphology.name(),
                "age_myr": ((st.t_myr - c.t_myr) * 100.0).round() / 100.0,
                "depth_m": c.profile.depth_m.round(),
                "rim_m": c.profile.rim_m.round(),
                "relief_m": c.relief_m.round(),
                "fate": c.fate.name(),
                "breccia_m": c.breccia_m.round(),
                "melt_m": c.melt_m.round(),
                "cover_m": c.cover_m.round(),
                "in_surface": c.imprinted,
            })
        })
        .collect();
    json!({
        "version": CRATERS_VERSION,
        "t_myr": st.t_myr,
        "craters": craters,
    })
}

impl ImpactStage {
    pub fn new(preset: PlanetPreset) -> ImpactStage {
        let p = preset.impacts.clone().unwrap_or_default();
        let total = preset.plate_steps();
        ImpactStage {
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
            CoreError::InvalidState("the impact stage runs after a plate step".into())
        })?;
        let t = plates.t_myr;
        let dt = self.preset.length_myr() / self.total.max(1) as f64;
        let omega_now: Vec<[f64; 3]> = plates
            .plates
            .iter()
            .map(|pl| if pl.alive { pl.omega } else { [0.0; 3] })
            .collect();
        let parents: Vec<Option<u32>> = plates.plates.iter().map(|pl| pl.parent).collect();
        let last = self.done + 1 >= self.total;
        let (breccia, melt_class) = (index("impact_breccia"), index("impact_melt"));
        let mut rng = Rng::stream(
            self.preset.seed ^ (self.done.wrapping_mul(0x9e37_79b9_7f4a_7c15)),
            "impacts",
        );
        let get = |fields: &BTreeMap<String, Field>, name: &str| -> Result<Vec<f32>, CoreError> {
            fields
                .get(name)
                .filter(|f| f.data.len() == n)
                .map(|f| f.data.clone())
                .ok_or_else(|| {
                    CoreError::InvalidState(format!(
                        "the impact stage needs the L1 field {name} (a plate step with \
                         l1_every_steps = 1)"
                    ))
                })
        };
        let plate_id = get(&world.fields, "plates/plate_id")?;
        let crust = get(&world.fields, "plates/crust_type")?;
        // The surface stage's net change of this step (erosion −, deposition +), per cell.
        let material = world
            .fields
            .get(crate::surface::field::MATERIAL)
            .filter(|f| f.data.len() == n)
            .map(|f| f.data.clone());
        let mut st = world.impacts.take().unwrap_or_else(|| ImpactState {
            omega: omega_now.clone(),
            frame_t_myr: f64::NEG_INFINITY,
            ..ImpactState::default()
        });
        let surface = world.surface.as_ref().ok_or_else(|| {
            CoreError::InvalidState("the impact stage runs after the surface stage".into())
        })?;
        let sea = surface.sea_level_m;
        let elev = surface.elevation_m.clone();
        let diffusivity = crate::surface::lithology::diffusivity();
        // The hillslope diffusivity of the rock at the surface of every cell (× the setting).
        let rock_d = |c: usize| -> f64 {
            surface
                .log
                .top(c)
                .and_then(|l| diffusivity.get(usize::from(l.class)).copied())
                .unwrap_or(1.0)
        };
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
        let mut gone_now: Vec<Crater> = Vec::new();
        let mut kept = Vec::with_capacity(st.craters.len());
        for mut c in std::mem::take(&mut st.craters) {
            let om = omega_of(c.plate);
            if om != [0.0; 3] {
                let q = frame.unit(c.x, c.z);
                let (nx, nz) = frame.block(rotation(DVec3::from_array(om), dt_yr) * q);
                c.x = nx;
                c.z = nz;
            }
            let cell = cell_of(&grid, c.x, c.z);
            let now = plate_id[cell].max(0.0) as u32;
            let continental = crust[cell] as u8 == CONTINENTAL;
            if now != c.plate {
                if c.oceanic && !continental {
                    c.gone = Some(Gone::Subducted);
                    st.counts.subducted[size_class(c.diameter_m / 1000.0)] += 1;
                    gone_now.push(c);
                    continue;
                }
                c.plate = now;
            }
            if continental {
                c.oceanic = false;
            }
            kept.push(c);
        }
        st.craters = kept;
        self.time("motion", started);

        // 2. Degradation by the surface stage's erosion and deposition of this step, and by
        // diffusion between the cells.
        let started = Instant::now();
        let mut kept = Vec::with_capacity(st.craters.len());
        for mut c in std::mem::take(&mut st.craters) {
            if let Some(m) = &material {
                let change = mean_inside(&grid, mh, cell_m, m, &c) * p.degradation;
                if change > 0.0 {
                    c.cover_m += change;
                } else {
                    let strip = (-change).min(c.cover_m);
                    c.cover_m -= strip;
                    c.worn_m += -change - strip;
                }
            }
            let cell = cell_of(&grid, c.x, c.z);
            if f64::from(elev[cell]) >= sea - WAVE_BASE_M {
                c.kt_m2 += p.diffusivity_m2_yr * rock_d(cell) * p.degradation * dt_yr;
            }
            if c.worn_out() {
                c.gone = Some(Gone::Eroded);
                st.counts.eroded[size_class(c.diameter_m / 1000.0)] += 1;
                gone_now.push(c);
            } else {
                kept.push(c);
            }
        }
        st.craters = kept;
        self.time("degrade", started);

        // 3–5. The step's impacts.
        let started = Instant::now();
        let set = setting(&self.preset, &p);
        let expected = impactors_per_myr(&self.preset, &p) * dt;
        let count = rng.poisson(expected);
        let mut delta = vec![0.0f32; n];
        // Log changes: (cell, thickness (negative = remove), class) in order.
        let mut log_ops: Vec<(usize, f32, u16)> = Vec::new();
        // Events to write: (index into the catalogue or None for an ocean impact, event).
        let mut ocean_events = Vec::new();
        for _ in 0..count {
            st.counts.impactors += 1;
            let d_earth = scaling::draw_diameter_m(
                rng.uniform(),
                p.min_crater_km * 1000.0,
                p.max_crater_km * 1000.0,
                p.size_slope,
            );
            let l = scaling::impactor_of_earth_crater_m(&set, d_earth);
            // A point uniform on the sphere, and a moment within the step.
            let lat = (2.0 * rng.uniform() - 1.0).asin().to_degrees();
            let lon = 360.0 * rng.uniform();
            let when = t - dt * rng.uniform();
            let (x, z) = (frame.x_of_lon(lon), frame.z_of_lat(lat));
            let here = sample_at(&grid, &elev, x, z) + sample_at(&grid, &delta, x, z);
            let water = (sea - here).max(0.0);
            let made = scaling::crater(&set, l, water)
                .filter(|c| c.diameter_m >= 0.5 * p.min_crater_km * 1000.0);
            let Some(made) = made else {
                st.counts.stopped_by_sea[size_class(d_earth / 1000.0)] += 1;
                if d_earth >= OCEAN_EVENT_MIN_KM * 1000.0 {
                    ocean_events.push((when, lat, lon, l, d_earth, water));
                }
                continue;
            };
            let class = size_class(made.diameter_m / 1000.0);
            st.counts.formed[class] += 1;
            if water > 0.0 {
                st.counts.formed_in_sea[class] += 1;
            }
            st.counts.largest_km = st.counts.largest_km.max(made.diameter_m / 1000.0);
            let (profile, breccia_m, melt_m) = build(&made, &p);
            let cell = cell_of(&grid, x, z);
            let continental = crust[cell] as u8 == CONTINENTAL;
            let imprinted = made.diameter_m >= 2.0 * cell_m;
            let mut c = Crater {
                id: st.next_id,
                x,
                z,
                plate: plate_id[cell].max(0.0) as u32,
                oceanic: !continental,
                t_myr: when,
                lat0: lat as f32,
                lon0: lon as f32,
                water_m: water,
                impactor_m: l,
                energy_j: made.energy_j,
                diameter_m: made.diameter_m,
                transient_m: made.transient_m,
                profile,
                breccia_m,
                melt_m,
                melt_real_m: made.melt_m,
                melt_km3: made.melt_m3 / 1.0e9,
                imprinted,
                cover_m: 0.0,
                worn_m: 0.0,
                kt_m2: 0.0,
                relief_m: profile.depth_m,
                fate: Fate::Fresh,
                event: None,
                gone: None,
            };
            st.next_id += 1;
            // A new crater destroys the smaller ones inside it.
            let r_new = c.radius_m();
            let mut survivors = Vec::with_capacity(st.craters.len());
            for mut old in std::mem::take(&mut st.craters) {
                let d = (frame.wrap_x(old.x - x) * mh).hypot((old.z - z) * mh);
                if old.diameter_m < c.diameter_m && d < 0.8 * r_new {
                    old.gone = Some(Gone::Overprinted);
                    st.counts.overprinted[size_class(old.diameter_m / 1000.0)] += 1;
                    gone_now.push(old);
                } else {
                    survivors.push(old);
                }
            }
            st.craters = survivors;
            if imprinted {
                st.counts.imprinted += 1;
                let extent = c.extent_m();
                let prof = |rho: f64| profile.at(rho);
                for_cells(&grid, mh, x, z, extent + cell_m, |cc, dx, dz| {
                    delta[cc] += cell_weight(&prof, dx, dz, cell_m, extent) as f32;
                    let xr = dx.hypot(dz) / r_new;
                    if xr < 1.0 {
                        // The target is gone down to the base of the lens; the lens (thinning to
                        // the rim) and the melt sheet on the floor take its place.
                        let lens = breccia_m
                            * match profile.morphology {
                                Morphology::Simple => 1.0 - xr * xr,
                                _ => 1.0 - smooth(0.7, 1.0, xr),
                            };
                        let dug = (-profile.at_rim_radii(xr)).max(0.0) + lens;
                        if dug > 0.0 {
                            log_ops.push((cc, -(dug as f32), 0));
                        }
                        if lens >= 0.5 {
                            log_ops.push((cc, lens as f32, breccia));
                        }
                        if melt_m >= 0.5 && xr < shape::FLOOR_RHO {
                            log_ops.push((cc, melt_m as f32, melt_class));
                        }
                    } else {
                        // Half of the rise outside the rim is ejecta (the rest is the uplifted
                        // target: Melosh 1989, §6.3).
                        let th = 0.5 * profile.at_rim_radii(xr);
                        if th >= 1.0 {
                            log_ops.push((cc, th as f32, breccia));
                        }
                    }
                });
            }
            if made.diameter_m >= EVENT_MIN_KM * 1000.0 {
                c.event = Some(self.crater_event(world, &c)?);
                st.events_emitted += 1;
            }
            st.craters.push(c);
        }
        for (when, lat, lon, l, d_earth, water) in ocean_events {
            let e = world
                .event(when * 1.0e6, kind::OCEAN, SOURCE)
                .at(Place::Point {
                    lat: lat as f32,
                    lon: lon as f32,
                })
                .magnitude((d_earth / 100_000.0).min(1.0) as f32)
                .param("impactor_m", l.round())
                .param("energy_mt", megatons(set.energy_j(l)))
                .param("water_depth_m", water.round())
                .param("crater_on_land_km", (d_earth / 100.0).round() / 10.0);
            world.emit(e)?;
            st.events_emitted += 1;
        }
        self.time("impacts", started);

        // Apply: the surface (offset, elevation, its fields) and the plate feedback; the log.
        let started = Instant::now();
        {
            let WorldState {
                fields, surface, ..
            } = world;
            let surf = surface.as_mut().expect("checked above");
            let mv = s.m_per_block_v;
            if !log_ops.is_empty() || delta.iter().any(|&d| d != 0.0) {
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
                        for (x, &d) in f.data.iter_mut().zip(&delta) {
                            *x += d * scale;
                        }
                    }
                };
                add(fields, crate::surface::field::FEEDBACK, 1.0);
                add(fields, crate::surface::field::ELEVATION_M, 1.0);
                add(fields, "elevation", (1.0 / mv) as f32);
            }
            let age = (t - 0.5 * dt) as f32;
            for &(c, th, class) in &log_ops {
                if th < 0.0 {
                    let left = f64::from(-th);
                    let take = f64::from(surf.regolith_m[c]).min(left);
                    surf.regolith_m[c] -= take as f32;
                    surf.log.erode(c, left - take);
                } else if th > 0.0 {
                    surf.log.deposit(c, age, th, class, IN_SITU);
                }
            }
        }
        self.time("apply", started);

        // The craters' state, the map layer and the closure the structure stage reads.
        let started = Instant::now();
        for c in &mut st.craters {
            let (relief, fate) = c.state();
            c.relief_m = relief;
            c.fate = fate;
        }
        for (name, data) in [
            (field::MAP, map(&grid, mh, &st.craters)),
            (field::TRAP, trap_field(&grid, mh, &st.craters)),
        ] {
            world.fields.insert(
                name.to_owned(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        }
        self.time("map", started);

        // 6. The deposit engine's fields for frames the surface closed this step.
        let started = Instant::now();
        if let Some(h) = world.history.as_mut() {
            let frames = h.frame_count() as u64;
            if frames > st.frames_seen {
                ctx.reserve((n * 4 * 4) as u64)?;
                let fields = frame_fields(&grid, mh, &st.craters, st.frame_t_myr);
                for k in st.frames_seen..frames {
                    h.add_fields(k as usize, &fields)?;
                }
                st.frames_seen = frames;
                st.frame_t_myr = t;
            }
        }
        self.time("frame_fields", started);

        for c in gone_now {
            if c.event.is_some() {
                st.gone.push(c);
            }
        }
        st.omega = omega_now;
        st.steps += 1;
        st.t_myr = t;
        st.sea_level_m = sea;
        if last {
            let started = Instant::now();
            self.finish(world, &mut st, &grid)?;
            self.time("finish", started);
            let note = self.note(&st);
            ctx.log.push(crate::log::Level::Info, summary_line(&note));
            world.notes.insert(STAGE_NAME.into(), note.to_string());
        }
        world.impacts = Some(st);
        Ok(())
    }

    /// The event of a crater as it forms.
    fn crater_event(
        &self,
        world: &mut WorldState,
        c: &Crater,
    ) -> Result<EventId, CoreError> {
        let e = world
            .event(c.t_myr * 1.0e6, kind::CRATER, SOURCE)
            .at(Place::Point {
                lat: c.lat0,
                lon: c.lon0,
            })
            .magnitude((c.diameter_m / 100_000.0).min(1.0) as f32)
            .param("crater", i64::from(c.id))
            .param("diameter_km", (c.diameter_m / 100.0).round() / 10.0)
            .param("morphology", c.profile.morphology.name())
            .param("depth_m", c.profile.depth_m.round())
            .param("rim_m", c.profile.rim_m.round())
            .param("impactor_m", c.impactor_m.round())
            .param("energy_mt", megatons(c.energy_j))
            .param(
                "target",
                if c.water_m > 0.0 {
                    "sea_floor"
                } else {
                    "land"
                },
            )
            .param("water_depth_m", c.water_m.round())
            .param("melt_km3", (c.melt_km3 * 10.0).round() / 10.0)
            .param("melt_sheet_m", c.melt_real_m.round())
            .param("impact_lat", f64::from(c.lat0))
            .param("impact_lon", f64::from(c.lon0))
            .param("plate", i64::from(c.plate))
            .param("fate", Fate::Fresh.name())
            .param("relief_m", c.profile.depth_m.round());
        world.emit(e)
    }

    /// The end of the run: every event completed with what became of its crater, an event for
    /// each small crater that still shows, and the bodies of the craters L1 does not hold.
    fn finish(
        &self,
        world: &mut WorldState,
        st: &mut ImpactState,
        grid: &GridDesc,
    ) -> Result<(), CoreError> {
        let s = &self.preset.scale;
        let frame = BlockFrame::new(s.circumference_blocks);
        // Small craters that still show get their event now.
        for k in 0..st.craters.len() {
            if st.craters[k].event.is_none() && st.craters[k].relief_m >= VISIBLE_RELIEF_M {
                let id = self.crater_event(world, &st.craters[k])?;
                st.craters[k].event = Some(id);
                st.events_emitted += 1;
            }
        }
        let by_id: BTreeMap<EventId, usize> = world
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.source == SOURCE)
            .map(|(i, e)| (e.id, i))
            .collect();
        let set = |world: &mut WorldState, c: &Crater, fate: &str, standing: bool| {
            let Some(&ix) = c.event.and_then(|id| by_id.get(&id)) else {
                return;
            };
            let e = &mut world.events[ix];
            e.params
                .insert("fate".into(), ParamValue::Str(fate.to_owned()));
            e.params
                .insert("relief_m".into(), ParamValue::Float(c.relief_m.round()));
            e.params
                .insert("cover_m".into(), ParamValue::Float(c.cover_m.round()));
            e.params
                .insert("standing".into(), ParamValue::Int(i64::from(standing)));
            if standing {
                // Where its crust has carried it.
                e.place = Place::Point {
                    lat: frame.lat_deg(c.z) as f32,
                    lon: frame.lon_deg(c.x) as f32,
                };
            }
        };
        for c in &st.craters {
            set(world, c, c.fate.name(), true);
        }
        for c in &st.gone {
            let fate = match c.gone {
                Some(Gone::Subducted) => "subducted",
                Some(Gone::Overprinted) => "overprinted",
                _ => "eroded",
            };
            let mut c = c.clone();
            c.relief_m = 0.0;
            set(world, &c, fate, false);
        }
        // Bodies: the lens and melt of the craters too small for the strata log's cells.
        let surf = world.surface.as_ref().expect("checked by the caller");
        let sea = st.sea_level_m;
        let mv = s.m_per_block_v;
        let mut bodies = Vec::new();
        for c in st.craters.iter().filter(|c| !c.imprinted) {
            // The crater's present floor: the deep-time surface there plus what the amplifier
            // will draw of the crater; the original floor lies under its cover.
            let ground = sample_at(grid, &surf.elevation_m, c.x, c.z);
            let centre = c.kept() * c.profile.aged(c.spread_m()).at(0.0);
            let floor_m = ground + centre - c.lens_top_below_m();
            let floor_y = f64::from(s.sea_level_y) + (floor_m - sea) / mv;
            bodies.push((c.clone(), floor_y));
        }
        for (c, floor_y) in bodies {
            let id = world.next_body_id();
            if let Some(b) = to_body(&c, id, floor_y, s.m_per_block_h, mv, s.min_y) {
                world.add_body(b)?;
                st.bodies += 1;
            }
        }
        Ok(())
    }

    /// The run summary (`notes.impacts`).
    fn note(&self, st: &ImpactState) -> Value {
        let mut standing = BTreeMap::new();
        for fate in [Fate::Fresh, Fate::Degraded, Fate::Buried, Fate::Roots] {
            let mut by_size = [0u64; 6];
            for c in st.craters.iter().filter(|c| c.fate == fate) {
                by_size[size_class(c.diameter_m / 1000.0)] += 1;
            }
            standing.insert(fate.name(), by_size);
        }
        let visible = st
            .craters
            .iter()
            .filter(|c| c.relief_m >= VISIBLE_RELIEF_M)
            .count();
        let on_land = st
            .craters
            .iter()
            .filter(|c| c.relief_m >= VISIBLE_RELIEF_M && !c.oceanic)
            .count();
        let oldest_visible = st
            .craters
            .iter()
            .filter(|c| c.relief_m >= VISIBLE_RELIEF_M)
            .map(|c| st.t_myr - c.t_myr)
            .fold(0.0f64, f64::max);
        json!({
            "steps": st.steps,
            "size_classes": SIZE_LABELS,
            "counts": st.counts,
            "formed": st.counts.formed.iter().sum::<u64>(),
            "catalogue": st.craters.len(),
            "present": standing,
            "visible": visible,
            "visible_on_continents": on_land,
            "oldest_visible_myr": (oldest_visible * 10.0).round() / 10.0,
            "expected_per_myr": impactors_per_myr(&self.preset, &self.p),
            "area_km2": planet_area_km2(&self.preset).round(),
            "transition_km": setting(&self.preset, &self.p).transition_m() / 1000.0,
            "peak_ring_km": setting(&self.preset, &self.p).peak_ring_m() / 1000.0,
            "events": st.events_emitted,
            "bodies": st.bodies,
            "params": self.p,
            "millis": self.timings,
        })
    }
}

/// Megatons of TNT (4.184 × 10¹⁵ J), rounded to three significant figures.
fn megatons(joules: f64) -> f64 {
    let mt = joules / 4.184e15;
    if mt <= 0.0 {
        return 0.0;
    }
    let scale = 10f64.powf(2.0 - mt.log10().floor());
    (mt * scale).round() / scale
}

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `impacts_final=<steps> impactors=… formed=… …` (the CLI's line), from `notes.impacts`.
pub fn summary_line(v: &Value) -> String {
    let c = &v["counts"];
    let sum = |x: &Value| -> u64 {
        x.as_array()
            .map(|a| a.iter().filter_map(Value::as_u64).sum())
            .unwrap_or(0)
    };
    let list = |x: &Value| -> String {
        x.as_array()
            .map(|a| {
                a.iter()
                    .map(|n| n.as_u64().unwrap_or(0).to_string())
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .unwrap_or_default()
    };
    format!(
        "impacts_final={} impactors={} formed={} formed_by_size={} formed_in_sea={} \
         stopped_by_sea={} eroded={} subducted={} overprinted={} catalogue={} visible={} \
         visible_on_continents={} fresh={} degraded={} buried={} roots={} largest_km={:.1} \
         oldest_visible_myr={} in_surface={} bodies={} events={}",
        v["steps"],
        c["impactors"],
        v["formed"],
        list(&c["formed"]),
        sum(&c["formed_in_sea"]),
        sum(&c["stopped_by_sea"]),
        sum(&c["eroded"]),
        sum(&c["subducted"]),
        sum(&c["overprinted"]),
        v["catalogue"],
        v["visible"],
        v["visible_on_continents"],
        sum(&v["present"]["fresh"]),
        sum(&v["present"]["degraded"]),
        sum(&v["present"]["buried"]),
        sum(&v["present"]["eroded_to_roots"]),
        c["largest_km"].as_f64().unwrap_or(0.0),
        v["oldest_visible_myr"],
        c["imprinted"],
        v["bodies"],
        v["events"],
    )
}

impl Stage for ImpactStage {
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
