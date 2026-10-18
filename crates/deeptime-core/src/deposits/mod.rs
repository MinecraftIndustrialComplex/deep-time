// SPDX-License-Identifier: LGPL-3.0-or-later
//! Deposits — `core::deposits` (plan §8.6; P1-14): the mineral-systems rule engine over the
//! history's named fields, the USGS three-part method for counts and sizes, a preservation
//! ledger, the four Phase 1 families and the playability pass, writing sites, bodies and events.
//!
//! No code exists to reuse (plan §13); this is Deep Time's own engine, from the papers cited in
//! each module:
//!
//! | Step | Module |
//! |---|---|
//! | the named-field interface P1-03…P1-07 publish into | [`fields`] |
//! | deposit-type datapack JSON (plan §5.4.3) | [`types`] |
//! | public-domain USGS grade–tonnage tables | [`usgs`] |
//! | favourability, limiting component, time integration | [`rules`] |
//! | permissive tracts | [`tracts`] |
//! | Poisson counts, metallogenic provinces | [`count`] |
//! | the survivor formula: formation from the density of survivors | [`survival`] |
//! | regional fields (basin brines, regional head) derived from each frame | [`regional`] |
//! | Thomas-cluster positions, minimum separation | [`cluster`] |
//! | lognormal tonnage, Gaussian-copula grades | [`sizing`] |
//! | zoned bodies | [`geometry`] |
//! | burial and erosion since formation | [`ledger`] |
//! | supergene blankets, placers | [`children`] |
//! | Lopatin TTI, kitchens, charge | [`petroleum`] |
//! | regional minimums | [`playability`] |
//! | "Why here?" causes | [`sites`] |
//! | a synthetic history for tests and the CLI | [`synthetic`] |
//!
//! The stage reads a [`fields::FieldHistory`] in two passes: pass 1 steps through every frame
//! (one frame per `step`) folding favourability and maturity into per-cell accumulators; the last
//! step finds the permissive tracts, replays the frames at every tract cell (pass 2: each cell's
//! favourability hazard frame by frame and its burial and erosion, from which [`survival`] gives
//! the chance that a deposit formed there survives to the present), sets how many deposits form
//! so that the survivors match the USGS density, places them, runs the ledger and the supergene
//! windows at their cells, builds bodies and sites and emits the events. Every frame read gets the
//! [`regional`] fields first.
//!
//! **Where the history comes from.** Two sources feed the same engine. The `deposits_synthetic`
//! preset builds a synthetic history from a seed ([`synthetic`]; the engine's own tests and the
//! deposit worlds of P1-15). A planet preset with a `deposits` section reads the history the
//! plates and surface stages recorded into `WorldState::history` (P1-04's
//! [`recorded::RecordedHistory`]: 16-bit and 8-bit LZ4 frames every `surface.frame_every_myr`, plus
//! the present-day drainage receivers) — the stage takes it out of the world for the placement step
//! and puts it back, so a checkpoint carries it and a resumed run replays the same frames.
//!
//! **Missing fields degrade, deterministically.** The field names are read from the history's own
//! metadata ([`fields::FieldHistory::names`]), never from a list here. A deposit type whose rules
//! read a name the history lacks is *skipped*: it accumulates nothing, places nothing, and the log
//! and the census (`skipped`) name it and the fields it lacks (today the petroleum trap, until
//! P1-07 publishes `deeptime:trap_structure`). A history with no `deeptime:host/<class>` field at
//! all is a documented fallback rather than a skip: the host score is 1 everywhere ("no lithology
//! opinion", [`rules`]), and the census says so (`fallbacks`). When a field appears in the history
//! the type comes back with no other change.

pub mod bif_basins;
pub mod children;
pub mod clocks;
pub mod cluster;
pub mod count;
pub mod fields;
pub mod geometry;
pub mod ledger;
pub mod petroleum;
pub mod placer_channels;
pub mod playability;
pub mod random;
pub mod recorded;
pub mod regional;
pub mod rules;
pub mod sites;
pub mod sizing;
pub mod spill;
pub mod survival;
pub mod synthetic;
pub mod tracts;
pub mod types;
pub mod usgs;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, OnceLock};

use deeptime_format::{Body, Event, EventId, Extraction, GridDesc, Place, Product, Site, SiteId};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::CoreError;
use crate::stage::{Stage, StageContext, StageEstimate, from_state, to_state};
use crate::world::{Field, WorldState};

use children::ColumnSample;
use cluster::{Point, Spacing, TractSpace};
use fields::{FieldFrame, FieldHistory, NO_ORIGIN};
use geometry::{Grades, Placement, Scales};
use ledger::{Fate, Ledger};
use petroleum::CellPet;
use playability::{EXPOSED_BLOCKS, Essential, Regions};
use random::Rng;
use rules::{CellAcc, Compiled, FrameEval};
use sites::{CauseIndex, lat_lon_of};
use sizing::{SizeDraw, SizeModel, mass_fraction};
use synthetic::{SyntheticHistory, SyntheticParams};
use types::{COMPONENTS, DepositType, ErosionBehavior, FormationTime, Province, Relation};

/// The stage's name in a pipeline (`deeptime stage deposits`).
pub const STAGE_NAME: &str = "deposits";
/// The source of the stage's events.
pub const SOURCE: &str = "deeptime:deposits";
/// Keyframe layer of the sites map.
pub const SITES_LAYER: &str = "deposits/sites";
/// Crude oil density used to turn oil tonnes into CDG millibuckets (t/m³; a design value, crude
/// oils run about 0.8–0.9).
pub const OIL_T_M3: f64 = 0.85;

fn one() -> f64 {
    1.0
}

fn province_km() -> f64 {
    800.0
}

/// USGS SIR 2010-5070-A (the MVT deposit model): MVT-permissive tracts are platform carbonates
/// "less than ~600 km inboard of Phanerozoic orogenic belts".
fn brine_flow_km() -> f64 {
    600.0
}

fn max_formed_per_survivor() -> f64 {
    20.0
}

/// 5 % of the stable interiors: a game choice (Ben, 2026-10-02, "5 % of stable interiors"), so a
/// 16k planet, which has about 1/44 of Earth's area, holds a handful of iron-formation sites.
/// Earth's own 2.6–1.8 Ga iron-formation basins cover about 1.4 % of its shields and cratons (0.75
/// million km² of 53.5 million: `bif_basins`), the reference value. ledger: ORE-261
fn inherited_bif_share() -> f64 {
    0.05
}

/// The median of Earth's iron-formation basin areas (Hamersley about 100,000 km²; `bif_basins`).
/// ledger: ORE-263
fn bif_basin_km2() -> f64 {
    100_000.0
}

/// Background scatter at half the scatter rules' amounts (Ben, 2026-10-02, "Rich Deposits,
/// Thinner Scatter"; the targets in docs/science/deposits.md). game-choice. ledger: ORE-265
pub const SCATTER_MULT: f64 = 0.5;

fn scatter_mult() -> f64 {
    SCATTER_MULT
}

/// The preset's `deposits` section (plan §5.1 `DepositParams`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepositParams {
    /// Global count multiplier (§8.6 step 2; 1 = real densities of surviving deposits).
    #[serde(default = "one")]
    pub count_mult: f64,
    /// Global tonnage multiplier (the editor's "Deposit size"): every type's grade–tonnage
    /// percentiles are scaled by it before the draw, so each deposit holds this many times the
    /// ore and its body grows with it (its volume follows the tonnage); the counts and the
    /// grades stay. 1 = the USGS models.
    #[serde(default = "one")]
    pub size_mult: f64,
    /// Global richness multiplier (the editor's "Deposit richness"): multiplies every type's
    /// `balance.richness_mult`, which scales the zones' ore density and raw-ore share in the
    /// world (`geometry::zone`, capped there), the zone grades and the metal each site holds
    /// (grades stay at most 1). 1 = the types' own densities. ledger: ORE-262
    #[serde(default = "one")]
    pub richness_mult: f64,
    /// Multiplies the background scatter of every scatter rule (the game reads it from the
    /// manifest's `notes.deposits.scatter_mult`; `BackgroundScatterFeature`). ledger: ORE-265
    #[serde(default = "scatter_mult")]
    pub scatter_mult: f64,
    /// At most this many deposits form per expected survivor (the survivor formula's cap,
    /// [`survival`]): a type the planet barely preserves forms this multiple of its target and
    /// falls short, rather than forming without bound.
    #[serde(default = "max_formed_per_survivor")]
    pub max_formed_per_survivor: f64,
    /// Reach of a basin-scale groundwater flow system (km): how far basinal brines and an
    /// orogen's head act ([`regional`]; MVT's driver).
    #[serde(default = "brine_flow_km")]
    pub brine_flow_km: f64,
    /// Inherited banded iron formation (`deposits::clocks`, `bif_basins`, plan §8.6): the share of
    /// the stable continental interiors that are iron-formation basins inherited from before the
    /// run, when the run starts after the 2.6–1.85 Ga window (0–1; Ben, 2026-10-02, "Earth-like
    /// patch field"). 0 = none; 1 = every stable interior (412 BIF sites on a 1,000 Myr quick 16k
    /// planet). ledger: ORE-261
    #[serde(default = "inherited_bif_share")]
    pub inherited_bif_share: f64,
    /// The typical area of one inherited iron-formation basin, km² (`bif_basins`). ledger: ORE-263
    #[serde(default = "bif_basin_km2")]
    pub bif_basin_km2: f64,
    /// Regional specialisation (decisions file: default strong).
    #[serde(default)]
    pub province: Province,
    /// Province size (km across).
    #[serde(default = "province_km")]
    pub province_km: f64,
    /// Deposit types to run (ids); empty = every built-in type.
    #[serde(default)]
    pub types: Vec<String>,
    /// The essentials list (plan §7.8).
    #[serde(default)]
    pub essentials: Vec<Essential>,
}

impl Default for DepositParams {
    fn default() -> Self {
        DepositParams {
            count_mult: 1.0,
            size_mult: 1.0,
            richness_mult: 1.0,
            scatter_mult: scatter_mult(),
            max_formed_per_survivor: max_formed_per_survivor(),
            brine_flow_km: brine_flow_km(),
            inherited_bif_share: inherited_bif_share(),
            bif_basin_km2: bif_basin_km2(),
            province: Province::Strong,
            province_km: province_km(),
            types: Vec::new(),
            essentials: Vec::new(),
        }
    }
}

/// The `deposits_synthetic` preset: a synthetic history and the deposit stage on it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticPreset {
    pub seed: u64,
    pub circumference: u32,
    pub grid_r: u32,
    pub t_start_years: f64,
    #[serde(default)]
    pub t_end_years: f64,
    pub snapshot_every_years: f64,
    pub keyframe_every_years: f64,
    pub keyframe_size: [u32; 2],
    #[serde(default)]
    pub deposits: DepositParams,
    /// P1-15: bake a playable terrain into the run's world data at `finalize_write`: the
    /// history's present elevation as the ground (the surface the bodies were placed against,
    /// [`synthetic::HistoryTerrain`]), the synthetic climate and horizons, and spawn candidates
    /// (dry flat land nearest (0, 0)). Without it the world data holds sites, bodies and
    /// events only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<TerrainBake>,
}

/// The `deposits_synthetic.terrain` section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerrainBake {
    /// The horizon stacks: `placeholder` (default), `every-class` or `basin` (`crate::synth::Strata`).
    #[serde(default = "placeholder_strata")]
    pub strata: String,
}

fn placeholder_strata() -> String {
    crate::synth::Strata::Placeholder.name().to_owned()
}

impl TerrainBake {
    pub fn strata(&self) -> crate::synth::Strata {
        crate::synth::Strata::from_name(&self.strata).expect("checked when the preset was parsed")
    }
}

impl SyntheticPreset {
    pub fn from_json(v: &Value) -> Result<SyntheticPreset, CoreError> {
        let p: SyntheticPreset = serde_json::from_value(v.clone())
            .map_err(|e| CoreError::preset("$.deposits_synthetic", e.to_string()))?;
        GridDesc::for_planet("L1", p.circumference, p.grid_r)
            .map_err(|e| CoreError::preset("$.deposits_synthetic.grid_r", e.to_string()))?;
        let span = p.t_end_years - p.t_start_years;
        if !(span > 0.0 && p.snapshot_every_years > 0.0 && p.keyframe_every_years > 0.0) {
            return Err(CoreError::preset(
                "$.deposits_synthetic",
                "t_start_years < t_end_years and positive intervals are required",
            ));
        }
        if span / p.snapshot_every_years > 10_000.0 {
            return Err(CoreError::preset(
                "$.deposits_synthetic.snapshot_every_years",
                "more than 10,000 frames",
            ));
        }
        p.deposits.check("$.deposits_synthetic.deposits")?;
        if p.keyframe_size[0] == 0 || p.keyframe_size[1] == 0 {
            return Err(CoreError::preset(
                "$.deposits_synthetic.keyframe_size",
                "must be positive",
            ));
        }
        if let Some(t) = &p.terrain {
            if crate::synth::Strata::from_name(&t.strata).is_none() {
                return Err(CoreError::preset(
                    "$.deposits_synthetic.terrain.strata",
                    format!("{:?} is not placeholder or every-class", t.strata),
                ));
            }
            crate::synth::check_circumference(p.circumference)
                .map_err(|e| CoreError::preset("$.deposits_synthetic.terrain", e))?;
        }
        Ok(p)
    }

    pub fn synthetic(&self) -> SyntheticParams {
        SyntheticParams {
            seed: self.seed,
            circumference: self.circumference,
            grid_r: self.grid_r,
            t_start_years: self.t_start_years,
            t_end_years: self.t_end_years,
            snapshot_every_years: self.snapshot_every_years,
        }
    }

    /// The pipeline: the deposit stage on the synthetic history.
    pub fn stages(&self) -> Result<Vec<Box<dyn Stage>>, CoreError> {
        Ok(vec![Box::new(DepositStage::synthetic(
            self.synthetic(),
            self.deposits.clone(),
        )?)])
    }

    pub fn initial_world(&self) -> WorldState {
        WorldState::new(self.seed, self.circumference, self.t_start_years)
    }
}

/// Interns a keyframe layer name (the `Stage` trait hands out `&'static str`s).
fn intern(s: String) -> &'static str {
    static NAMES: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();
    let names = NAMES.get_or_init(Default::default);
    let mut g = names.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(x) = g.get(s.as_str()) {
        return x;
    }
    let leaked: &'static str = Box::leak(s.into_boxed_str());
    g.insert(leaked);
    leaked
}

/// The inputs shown as keyframe layers (`input/<name>`), when the history has them.
const INPUT_LAYERS: &[(&str, &str)] = &[
    ("deeptime:elevation_m", "input/elevation_m"),
    ("deeptime:arc_magma_flux", "input/arc_magma_flux"),
    ("deeptime:carbonate_platform", "input/carbonate_platform"),
    ("deeptime:euxinic_basin", "input/euxinic_basin"),
    ("deeptime:coal_facies", "input/coal_facies"),
    ("deeptime:erosion_rate_m_myr", "input/erosion_rate_m_myr"),
    (regional::BASIN_BRINE, "input/basin_brine"),
    (regional::HEAD_DROP, "input/regional_head_drop_m_km"),
];

/// What a checkpoint keeps of the stage.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct State {
    /// 0 = set-up, 1..=frames = pass 1, frames + 1 = placement; then done.
    step: u64,
    /// The number of frames of the history, once set-up read it (a recorded history's count is
    /// only known then; 0 before).
    frames: usize,
    enabled: Vec<bool>,
    missing: Vec<Vec<String>>,
    /// Documented fallbacks the history's missing fields forced (the census lists them).
    fallbacks: Vec<String>,
    /// The names the history carried at set-up (fields and clocks), for the census.
    field_names: Vec<String>,
    acc: Vec<Vec<CellAcc>>,
    pet: Vec<CellPet>,
    frame_times: Vec<(f64, f64)>,
}

/// Where a stage's field history comes from.
#[derive(Clone, Debug)]
enum Source {
    /// The synthetic history of a `deposits_synthetic` preset.
    Synthetic(SyntheticParams),
    /// The history the plates and surface stages recorded into the world (P1-04): the L1 grid it
    /// was recorded on, and the number of frames the preset's timeline leads to (corrected from
    /// the world's own history in set-up).
    Recorded { circumference: u32, grid_r: u32 },
}

/// The deposit stage.
pub struct DepositStage {
    params: DepositParams,
    seed: u64,
    scales: Scales,
    source: Source,
    types: Vec<DepositType>,
    history: Option<Box<dyn FieldHistory>>,
    synth_events: Vec<synthetic::SynthEvent>,
    frames: usize,
    state: State,
    layers: Vec<&'static str>,
    /// The planet preset of a recorded history (the sedimentary cover a coal stack is held in is
    /// built from it and the world, `structure::sedimentary_cover_m`); None for a synthetic one.
    planet: Option<Box<crate::params::PlanetPreset>>,
}

/// The types a run uses: every built-in type, or the requested ones and the children they spawn.
/// The deposit types a run uses: the built-ins (or the requested ones and their children) with the
/// preset's global multipliers folded into each type's own: `size_mult` scales the tonnage
/// percentiles, `richness_mult` the balance's, and `feature` (`scale.feature_scale.deposits`) the
/// balance's horizontal and vertical body size. Every multiplier at 1 leaves the types as they are.
fn select_types(params: &DepositParams, feature: f64) -> Result<Vec<DepositType>, CoreError> {
    let mut types = types::builtin()?;
    if !params.types.is_empty() {
        for id in &params.types {
            if !types.iter().any(|t| &t.id == id) {
                return Err(CoreError::preset(
                    "$.deposits.types",
                    format!("unknown deposit type {id}"),
                ));
            }
        }
        // Keep the requested types and the children they spawn.
        let mut keep: BTreeSet<String> = params.types.iter().cloned().collect();
        for t in &types {
            if keep.contains(&t.id) {
                keep.extend(t.geometry.children.iter().map(|c| c.type_id.clone()));
            }
        }
        types.retain(|t| keep.contains(&t.id));
    }
    for t in &mut types {
        let s = params.size_mult;
        t.sizing.tonnage_mt.p10 *= s;
        t.sizing.tonnage_mt.p50 *= s;
        t.sizing.tonnage_mt.p90 *= s;
        t.balance.richness_mult *= params.richness_mult;
        t.balance.size_mult_h *= feature;
        t.balance.size_mult_v *= feature;
    }
    Ok(types)
}

impl DepositParams {
    /// Range checks and the type list, each naming its key under `path` (`$.deposits` in a planet
    /// preset).
    pub fn validate(&self, path: &str) -> Result<(), CoreError> {
        self.check(path)?;
        select_types(self, 1.0).map(|_| ())
    }

    fn check(&self, path: &str) -> Result<(), CoreError> {
        if !(self.count_mult >= 0.0
            && self.province_km > 0.0
            && self.max_formed_per_survivor >= 1.0
            && self.brine_flow_km > 0.0)
        {
            return Err(CoreError::preset(
                path,
                "count_mult ≥ 0, province_km > 0, max_formed_per_survivor ≥ 1 and brine_flow_km > 0 \
                 are required",
            ));
        }
        if !(0.0..=1.0).contains(&self.inherited_bif_share) {
            return Err(CoreError::preset(
                format!("{path}.inherited_bif_share"),
                "must be in [0, 1]",
            ));
        }
        if !(self.bif_basin_km2.is_finite()
            && (1000.0..=10_000_000.0).contains(&self.bif_basin_km2))
        {
            return Err(CoreError::preset(
                format!("{path}.bif_basin_km2"),
                "must be in [1000, 1e7]",
            ));
        }
        if !(self.scatter_mult.is_finite() && (0.0..=10.0).contains(&self.scatter_mult)) {
            return Err(CoreError::preset(
                format!("{path}.scatter_mult"),
                "must be in [0, 10]",
            ));
        }
        for (v, key) in [
            (self.size_mult, "size_mult"),
            (self.richness_mult, "richness_mult"),
        ] {
            if !(v.is_finite() && v > 0.0 && v <= 100.0) {
                return Err(CoreError::preset(
                    format!("{path}.{key}"),
                    "must be in (0, 100]",
                ));
            }
        }
        Ok(())
    }
}

impl DepositStage {
    /// The stage on a synthetic history.
    pub fn synthetic(
        sp: SyntheticParams,
        params: DepositParams,
    ) -> Result<DepositStage, CoreError> {
        let types = select_types(&params, 1.0)?;
        let span = sp.t_end_years - sp.t_start_years;
        let frames = (span / sp.snapshot_every_years).ceil().max(1.0) as usize;
        Ok(DepositStage::build(
            sp.seed,
            params,
            Scales::default(),
            Source::Synthetic(sp),
            types,
            frames,
        ))
    }

    /// The stage on the history the plates and surface stages record into the world (a planet
    /// preset with `surface` and `deposits` sections; the preset's scales, grid and seed).
    pub fn recorded(preset: &crate::params::PlanetPreset) -> Result<DepositStage, CoreError> {
        let params = preset.deposits.clone().unwrap_or_default();
        let types = select_types(&params, preset.scale.feature_scale.deposits)?;
        let every = preset.surface.as_ref().map_or(10.0, |s| s.frame_every_myr);
        // The surface stage's frame count is known only from its history; this is the estimate
        // the stage reports until set-up reads the real one.
        let frames = (preset.run_myr() / every).ceil().max(1.0) as usize;
        let sc = &preset.scale;
        let scales = Scales {
            m_per_block_h: sc.m_per_block_h,
            m_per_block_v: sc.m_per_block_v,
            min_y: sc.min_y,
            max_y: sc.max_y,
            sea_level_y: sc.sea_level_y,
        };
        let mut stage = DepositStage::build(
            preset.seed,
            params,
            scales,
            Source::Recorded {
                circumference: sc.circumference_blocks,
                grid_r: preset.resolution.l1_r,
            },
            types,
            frames,
        );
        stage.planet = Some(Box::new(preset.clone()));
        Ok(stage)
    }

    /// The same stage running `types` instead (tests: a type written for the test).
    #[cfg(test)]
    fn with_types(self, types: Vec<DepositType>) -> DepositStage {
        let mut stage = DepositStage::build(
            self.seed,
            self.params,
            self.scales,
            self.source,
            types,
            self.frames,
        );
        stage.planet = self.planet;
        stage
    }

    fn build(
        seed: u64,
        params: DepositParams,
        scales: Scales,
        source: Source,
        types: Vec<DepositType>,
        frames: usize,
    ) -> DepositStage {
        let mut layers: Vec<&'static str> = types
            .iter()
            .filter(|t| !t.child_only)
            .map(|t| intern(format!("deposits/favourability/{}", t.short())))
            .collect();
        layers.push(SITES_LAYER);
        layers.extend(INPUT_LAYERS.iter().map(|(_, l)| *l));
        DepositStage {
            seed,
            params,
            scales,
            source,
            types,
            history: None,
            synth_events: Vec::new(),
            frames,
            state: State::default(),
            layers,
            planet: None,
        }
    }

    fn grid(&self) -> GridDesc {
        let (c, r) = match &self.source {
            Source::Synthetic(sp) => (sp.circumference, sp.grid_r),
            Source::Recorded {
                circumference,
                grid_r,
            } => (*circumference, *grid_r),
        };
        GridDesc::for_planet("L1", c, r).expect("checked when the preset was parsed")
    }

    /// The synthetic history, built on first use (on the session's pool); a fresh one also hands
    /// over its tectonic events for [`DepositStage::setup`] to emit.
    fn history(&mut self) -> Result<&dyn FieldHistory, CoreError> {
        if self.history.is_none() {
            let Source::Synthetic(sp) = &self.source else {
                return Err(CoreError::InvalidState(
                    "a recorded history lives in the world, not the stage".into(),
                ));
            };
            let h = SyntheticHistory::new(sp.clone())?;
            self.synth_events = h.events();
            self.history = Some(Box::new(h));
        }
        Ok(self.history.as_deref().expect("just built"))
    }

    /// The world's recorded history.
    fn recorded_of(world: &WorldState) -> Result<&recorded::RecordedHistory, CoreError> {
        world.history.as_ref().ok_or_else(|| {
            CoreError::InvalidState(
                "the deposit stage needs the field history the surface stage records: the world \
                 has none (run the plates and surface stages first, with a `surface` section)"
                    .into(),
            )
        })
    }

    /// Frame `k` of whichever history this stage reads, with the regional fields.
    fn read_frame(&mut self, k: usize, world: &WorldState) -> Result<FieldFrame, CoreError> {
        let frame = if matches!(self.source, Source::Synthetic(_)) {
            self.history()?.frame(k)?
        } else {
            DepositStage::recorded_of(world)?.frame(k)?
        };
        Ok(self.with_regional(frame))
    }

    /// The regional fields' parameters on this stage's grid.
    fn regional(&self) -> regional::Regional {
        regional::Regional::new(
            &self.grid(),
            self.scales.m_per_block_h,
            self.params.brine_flow_km,
        )
    }

    /// `frame` with the [`regional`] fields that an enabled type reads.
    fn with_regional(&self, mut frame: FieldFrame) -> FieldFrame {
        let want: Vec<String> = self
            .types
            .iter()
            .zip(&self.state.enabled)
            .filter(|(_, ok)| **ok)
            .flat_map(|(t, _)| t.fields())
            .filter(|f| regional::DERIVED.contains(&f.as_str()))
            .collect();
        if !want.is_empty() {
            regional::add(&mut frame, &self.grid(), &self.regional(), &want);
        }
        frame
    }

    /// Frame `k` of `hist` with the regional fields (the placement step's reads).
    fn frame_of(&self, hist: &dyn FieldHistory, k: usize) -> Result<FieldFrame, CoreError> {
        Ok(self.with_regional(hist.frame(k)?))
    }

    /// Index of the petroleum type, if loaded and enabled.
    fn petroleum_type(&self) -> Option<usize> {
        self.types.iter().enumerate().position(|(k, t)| {
            t.petroleum.is_some() && self.state.enabled.get(k).copied().unwrap_or(false)
        })
    }

    /// The field whose TTI the petroleum and coal rules read.
    fn burial_field(&self) -> Option<String> {
        self.types
            .iter()
            .find_map(|t| t.petroleum.as_ref().map(|p| p.temperature_field.clone()))
            .or_else(|| {
                self.types
                    .iter()
                    .find_map(|t| t.coal.as_ref().map(|c| c.temperature_field.clone()))
            })
    }

    fn setup(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let grid = self.grid();
        let n = grid.cells();
        // The names, and (recorded) the frames the surface stage really wrote.
        let (names, frame_times): (Vec<String>, Vec<(f64, f64)>) =
            if matches!(self.source, Source::Synthetic(_)) {
                let names = ctx.pool.install(|| self.history().map(|h| h.names()))?;
                let times = (0..self.frames).map(|k| self.synthetic_time(k)).collect();
                (names, times)
            } else {
                let h = DepositStage::recorded_of(world)?;
                if h.grid != grid {
                    return Err(CoreError::InvalidState(format!(
                        "the recorded history is on grid {:?}, the deposit stage expects {:?}",
                        h.grid, grid
                    )));
                }
                let times = h.frame_times();
                if times.is_empty() {
                    return Err(CoreError::InvalidState(
                        "the recorded history has no frames (the surface stage has not finished)"
                            .into(),
                    ));
                }
                (h.names(), times)
            };
        // The regional fields every frame gets on reading.
        let mut names = names;
        names.extend(regional::derivable(&names));
        self.frames = frame_times.len();
        self.state.frames = self.frames;
        let bytes = (self.types.len() as u64 * n as u64 * std::mem::size_of::<CellAcc>() as u64)
            + n as u64 * std::mem::size_of::<CellPet>() as u64;
        ctx.reserve(bytes)?;
        let mut enabled = Vec::new();
        let mut missing = Vec::new();
        let mut acc = Vec::new();
        for t in &self.types {
            let mut want: Vec<String> = rules::missing_names(&Compiled::new(t), &names);
            for f in t.fields() {
                if !names.contains(&f) && !want.contains(&f) {
                    want.push(f);
                }
            }
            let ok = want.is_empty();
            if !ok {
                ctx.log.push(
                    crate::log::Level::Warn,
                    format!(
                        "deposit type {} skipped: the history lacks {want:?}; it accumulates and \
                         places nothing until a stage publishes them",
                        t.id
                    ),
                );
            }
            enabled.push(ok);
            missing.push(want);
            let needs_acc = ok && !t.child_only && t.petroleum.is_none();
            acc.push(if needs_acc {
                vec![CellAcc::fresh(); n]
            } else {
                Vec::new()
            });
        }
        // The documented fallback for absent host-rock fractions: no lithology opinion.
        let mut fallbacks = Vec::new();
        if !names.iter().any(|n| n.starts_with(fields::HOST_PREFIX)) {
            let hosted: Vec<&str> = self
                .types
                .iter()
                .zip(&enabled)
                .filter(|(t, ok)| **ok && t.hosts.weights().is_ok_and(|w| !w.is_empty()))
                .map(|(t, _)| t.id.as_str())
                .collect();
            if !hosted.is_empty() {
                let note = format!(
                    "the history has no {}<class> fields: host-rock scores are 1 (no lithology \
                     opinion) for {}",
                    fields::HOST_PREFIX,
                    hosted.join(", ")
                );
                ctx.log
                    .push(crate::log::Level::Warn, format!("deposits: {note}"));
                fallbacks.push(note);
            }
        }
        let skipped: Vec<&str> = self
            .types
            .iter()
            .zip(&enabled)
            .filter(|(_, ok)| !**ok)
            .map(|(t, _)| t.id.as_str())
            .collect();
        let clocks = names
            .iter()
            .filter(|n| n.starts_with(fields::CLOCK_PREFIX))
            .count();
        ctx.log.push(
            crate::log::Level::Info,
            format!(
                "deposits: {} of {} types run on {} frames ({} fields, {clocks} clocks){}",
                self.types.len() - skipped.len(),
                self.types.len(),
                self.frames,
                names.len() - clocks,
                if skipped.is_empty() {
                    String::new()
                } else {
                    format!("; skipped for missing fields: {}", skipped.join(", "))
                }
            ),
        );
        self.state.enabled = enabled;
        self.state.missing = missing;
        self.state.fallbacks = fallbacks;
        self.state.field_names = names;
        self.state.acc = acc;
        self.state.pet = if self.burial_field().is_some() {
            vec![CellPet::default(); n]
        } else {
            Vec::new()
        };
        self.state.frame_times = frame_times;
        // The synthetic history's own tectonic events (what the plates and surface stages emit
        // for real).
        for e in std::mem::take(&mut self.synth_events) {
            let ev = world
                .event(e.t_start, e.kind, synthetic::SOURCE)
                .until(e.t_end)
                .at(Place::Point {
                    lat: e.lat,
                    lon: e.lon,
                })
                .magnitude(e.magnitude)
                .param("feature", e.feature);
            world.emit(ev)?;
        }
        Ok(())
    }

    /// Bytes of one frame unpacked (every name the history carries as a full field).
    fn frame_bytes(&self) -> u64 {
        self.grid().cells() as u64 * 4 * self.state.field_names.len().max(1) as u64
    }

    fn synthetic_time(&self, k: usize) -> (f64, f64) {
        let Source::Synthetic(sp) = &self.source else {
            unreachable!("only a synthetic history is timed by the preset");
        };
        let dt = (sp.t_end_years - sp.t_start_years) / self.frames as f64;
        (sp.t_start_years + (k as f64 + 1.0) * dt, dt)
    }

    fn ingest(
        &mut self,
        k: usize,
        world: &mut WorldState,
        ctx: &StageContext<'_>,
    ) -> Result<(), CoreError> {
        let grid = self.grid();
        // The frame unpacked, the crust's origins, one carried accumulator and the favourability
        // layer being replaced (P1-17: on a long run the recorded history's frames give way to
        // them).
        let per_cell = 12 + 4 + std::mem::size_of::<CellAcc>() + std::mem::size_of::<CellPet>();
        ctx.reserve(self.frame_bytes() + grid.cells() as u64 * per_cell as u64)?;
        let burial = self.burial_field();
        let pet_rules = self.types.iter().find_map(|t| t.petroleum.clone());
        let t0 = std::time::Instant::now();
        let frame = ctx.pool.install(|| self.read_frame(k, world))?;
        let read_s = t0.elapsed().as_secs_f64();
        // The crust moves: carry what each cell has accumulated so far with it (a porphyry formed
        // on an arc travels with its plate), then fold this frame in at the new positions.
        let origin = if matches!(self.source, Source::Synthetic(_)) {
            None
        } else {
            ctx.pool
                .install(|| DepositStage::recorded_of(world)?.origins(k))?
        };
        let types = &self.types;
        let state = &mut self.state;
        ctx.pool.install(|| {
            if let Some(o) = &origin {
                for a in &mut state.acc {
                    if !a.is_empty() {
                        carry(a, o, CellAcc::fresh());
                    }
                }
                if !state.pet.is_empty() {
                    carry(&mut state.pet, o, CellPet::default());
                }
            }
            for (i, t) in types.iter().enumerate() {
                if state.acc[i].is_empty() {
                    continue;
                }
                let rules = Compiled::new(t);
                if let Some(eval) = FrameEval::new(&rules, &frame) {
                    rules::accumulate(&mut state.acc[i], &eval, k as u16, frame.dt_myr());
                }
            }
            if let Some(temp) = &burial
                && !state.pet.is_empty()
            {
                petroleum::accumulate(&mut state.pet, &frame, temp, pet_rules.as_ref());
            }
        });
        // Keyframe layers: favourability so far and the inputs.
        for (i, t) in self.types.iter().enumerate() {
            if self.state.acc[i].is_empty() {
                continue;
            }
            // Cell by cell, so in parallel (P1-17).
            let acc = &self.state.acc[i];
            let data: Vec<f32> = ctx
                .pool
                .install(|| acc.par_iter().map(|a| a.favourability() as f32).collect());
            world.fields.insert(
                format!("deposits/favourability/{}", t.short()),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        }
        for (name, layer) in INPUT_LAYERS {
            if let Some(v) = frame.fields.get(*name) {
                world.fields.insert(
                    (*layer).to_owned(),
                    Field {
                        grid: grid.clone(),
                        data: v.clone(),
                    },
                );
            }
        }
        // The last frame's favourability is drawn a year before its end, so the stage's closing
        // keyframe (sites placed, at the frame's end) is a frame of its own. (A recorded history
        // reaches this stage at the end of time already, and the session tells the two keyframes
        // apart by their stage.)
        let t = if k + 1 == self.frames {
            frame.t_years - 1.0
        } else {
            frame.t_years
        };
        world.t_years = world.t_years.max(t);
        ctx.log.push(
            crate::log::Level::Debug,
            format!(
                "deposits: frame {k} read in {read_s:.3} s, folded in {:.3} s",
                t0.elapsed().as_secs_f64() - read_s
            ),
        );
        Ok(())
    }
}

/// One deposit chosen by placement, before the ledger.
#[derive(Clone, Debug)]
struct Candidate {
    ty: usize,
    point: Point,
    size: SizeDraw,
    endowment: f64,
    /// The favourability at its cell (the charged-trap score for petroleum).
    fav: f64,
    /// Petroleum: (kitchen, charge time, migration km).
    charge: Option<(usize, f64, f64)>,
    /// The favourability hazard of its cell, frame by frame (types that form at a frame of the
    /// history): its formation frame is drawn ∝ this ([`survival`]).
    hazard: Option<Vec<f32>>,
    /// p(c) of its cell: the chance the survivor formula gave it.
    p_survive: f64,
}

/// When a type's deposits form (what [`survival::preservation`] integrates over).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FormationKind {
    /// At a frame drawn ∝ the cell's favourability hazard (`formation_time: peak`).
    Hazard,
    /// Before the history (`formation_time: cause`, inherited deposits).
    Start,
    /// At the trap's charge (petroleum).
    Charge,
}

/// One type's permissive tracts, before the counts.
struct TypeTracts {
    ti: usize,
    fv: Vec<f64>,
    tracts: Vec<tracts::Tract>,
    endow: Vec<f64>,
    strength: Province,
    /// Every cell of its tracts, ascending.
    cells: Vec<u32>,
    formation: FormationKind,
}

/// Pass 2: every tract cell's column through the history, `[i · frames + k]` for cell `cells[i]`.
struct Columns {
    cells: Vec<u32>,
    frames: usize,
    /// Whether the crust now at the cell existed in frame k.
    alive: Vec<bool>,
    erosion: Vec<f32>,
    deposition: Vec<f32>,
    water_table: Vec<f32>,
    precip: Vec<f32>,
    /// Where the crust now at `cells[i]` was at the end of every frame.
    paths: Vec<Vec<u32>>,
    /// Per [`TypeTracts`] (same order), for `FormationKind::Hazard` types: the hazard
    /// −(Δt/τ)·ln(1 − F_t) of each of its cells, `[j · frames + k]` for its `cells[j]`.
    hazard: Vec<Vec<f32>>,
}

impl Columns {
    fn index(&self, cell: u32) -> Option<usize> {
        self.cells.binary_search(&cell).ok()
    }

    fn hazard_of(&self, plan: usize, j: usize) -> &[f32] {
        &self.hazard[plan][j * self.frames..(j + 1) * self.frames]
    }

    /// The column's net burial per frame (m).
    fn net(&self, i: usize, times: &[(f64, f64)]) -> survival::ColumnNet {
        let at = i * self.frames;
        survival::ColumnNet::new(
            (0..self.frames)
                .map(|k| {
                    if self.alive[at + k] {
                        (f64::from(self.deposition[at + k]) - f64::from(self.erosion[at + k]))
                            * times[k].1
                            / 1.0e6
                    } else {
                        0.0
                    }
                })
                .collect(),
        )
    }

    /// The column's samples, the frames its crust existed in (what the ledger and the supergene
    /// windows read).
    fn series(&self, i: usize, times: &[(f64, f64)]) -> Vec<ColumnSample> {
        let at = i * self.frames;
        (0..self.frames)
            .filter(|k| self.alive[at + k])
            .map(|k| ColumnSample {
                t_years: times[k].0,
                dt_years: times[k].1,
                erosion_m_myr: f64::from(self.erosion[at + k]),
                deposition_m_myr: f64::from(self.deposition[at + k]),
                water_table_m: f64::from(self.water_table[at + k]),
                precip_mm: f64::from(self.precip[at + k]),
            })
            .collect()
    }
}

/// Per-type census numbers.
#[derive(Clone, Debug, Default, Serialize)]
struct TypeCensus {
    id: String,
    enabled: bool,
    missing: Vec<String>,
    tracts: usize,
    permissive_km2: f64,
    mean_f: f64,
    /// N* = count_mult · ρ/10⁵ · Σ A: the survivors within the assessment depth the USGS density
    /// asks for (plan §8.6 as built 2026-09-29, [`survival`]).
    target: f64,
    /// P̄: the share of what forms that the planet preserves (formation-weighted).
    preservation: f64,
    /// Deposits formed per expected survivor, 1 / max(P̄, 1 / cap).
    formation_mult: f64,
    /// The cap stopped formation short of the target.
    capped: bool,
    /// Σ λ: the deposits expected to form.
    expected_formed: f64,
    /// Σ λ·p̄₁: the survivors within the assessment depth expected (= `target` unless capped).
    expected: f64,
    /// The density aimed at (per 100,000 km²): the mean of a measured distribution, P50 of an
    /// estimate.
    density: f64,
    /// Σ λ·p̄: all survivors expected (sites, the deeper ones included).
    expected_sites: f64,
    /// Sites whose top lies within the assessment depth (what the density counts).
    counted: usize,
    /// Deposits formed (drawn).
    drawn: u64,
    /// Σ p(c) over the deposits formed, at the cells they were placed in: the survivors the
    /// model expects given where they landed (a check on the placement against the weights).
    expected_at_positions: f64,
    placed: usize,
    dropped_separation: usize,
    sites: usize,
    buried: usize,
    truncated: usize,
    eroded: usize,
    too_deep: usize,
    children: usize,
    guaranteed: usize,
    /// Sites whose centre is above sea level at the end of the history (a sanity check on where
    /// the geology put them).
    on_land: usize,
    /// Stacked types: seams over all sites, the most in one site, and the coal measures' total
    /// thickness (m) over all sites (their means are the stack's numbers).
    seams: usize,
    seams_max: usize,
    measures_m: f64,
    /// Stacked types: deposits dropped because the site's sedimentary cover holds none of their
    /// seams (basement at the surface, or the stack wholly under the cover), and stacks the cover
    /// cut short.
    no_cover: usize,
    cover_capped: usize,
    /// The most favourable cell of the planet: where, how favourable, and the component that
    /// limits it (the answer to "why so few?" for a type with an empty map).
    best_cell: Option<Value>,
}

impl Stage for DepositStage {
    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn estimate(&self) -> StageEstimate {
        let n = self.grid().cells() as u64;
        // Accumulators, one frame of ~45 f32 fields and the favourability layers.
        let bytes = n * (self.types.len() as u64 * 24 + 24 + 45 * 4 + 16 * 4);
        StageEstimate {
            bytes,
            seconds: 2.0e-8 * n as f64 * self.frames as f64 * (self.types.len() as f64 + 30.0),
            history_bytes: 0,
        }
    }

    fn total_steps(&self) -> u64 {
        self.frames as u64 + 2
    }

    fn steps_done(&self) -> u64 {
        self.state.step
    }

    fn step(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        let s = self.state.step;
        if s == 0 {
            self.setup(world, ctx)?;
        } else if s as usize <= self.frames {
            self.ingest(s as usize - 1, world, ctx)?;
        } else {
            self.finish(world, ctx)?;
        }
        self.state.step += 1;
        Ok(())
    }

    fn keyframe_layers(&self) -> &[&'static str] {
        &self.layers
    }

    fn save_state(&self) -> Result<Vec<u8>, CoreError> {
        to_state(&self.state)
    }

    fn load_state(&mut self, bytes: &[u8]) -> Result<(), CoreError> {
        self.state = from_state(bytes)?;
        // A recorded history's frame count is read in set-up; a resumed stage takes it from there.
        if self.state.frames > 0 {
            self.frames = self.state.frames;
        }
        Ok(())
    }
}

/// Everything the placement step shares.
struct Finish {
    grid: GridDesc,
    c: f64,
    sc: Scales,
    present: FieldFrame,
    elev: Vec<f32>,
    causes: CauseIndex,
    census: Vec<TypeCensus>,
    /// Placed deposits per type (separation checks across tracts and children).
    placed: Vec<Vec<Point>>,
    /// Where the crust now at a cell was at the end of every frame ([`trace`]), for the cells that
    /// matter (kitchen centres, candidate sites): the place a deposit's forming events are
    /// looked up at.
    paths: BTreeMap<u32, Vec<u32>>,
    /// The sedimentary cover a coal stack must lie in, as the world will draw it; None without a
    /// planet's columns (a synthetic history).
    cover: Option<crate::structure::Cover>,
}

/// Where the crust now at each of `cells` was at the end of every frame: `[i][k]`, [`NO_ORIGIN`]
/// before it existed. A history whose crust does not move gives the cell itself every time.
fn trace(
    hist: &dyn FieldHistory,
    frames: usize,
    cells: &[u32],
) -> Result<Vec<Vec<u32>>, CoreError> {
    let mut out = vec![vec![NO_ORIGIN; frames]; cells.len()];
    let mut cur: Vec<u32> = cells.to_vec();
    for k in (0..frames).rev() {
        for (o, c) in out.iter_mut().zip(&cur) {
            o[k] = *c;
        }
        if let Some(map) = hist.origins(k)? {
            for c in &mut cur {
                if *c != NO_ORIGIN {
                    *c = map[*c as usize];
                }
            }
        }
    }
    Ok(out)
}

/// Carries per-cell accumulators with the crust into a frame: the cell `c` takes what was at
/// `origin[c]` (or a fresh value for crust born in the frame).
fn carry<T: Copy + Send + Sync>(v: &mut Vec<T>, origin: &[u32], fresh: T) {
    let old = std::mem::take(v);
    *v = origin
        .par_iter()
        .map(|&q| {
            if q == NO_ORIGIN {
                fresh
            } else {
                old[q as usize]
            }
        })
        .collect();
}

/// The block y of the present surface at elevation `elev_m` (plan §8.8: y = sea_level_y + h /
/// m_per_block_v; sea floors stay where they are, deposits there are rare).
pub fn surface_y(elev_m: f32, sc: &Scales) -> f64 {
    (f64::from(sc.sea_level_y) + f64::from(elev_m) / sc.m_per_block_v)
        .clamp(f64::from(sc.min_y + 8), f64::from(sc.max_y))
}

fn favourability_of(acc: &[CellAcc]) -> Vec<f64> {
    acc.iter().map(CellAcc::favourability).collect()
}

/// Bilinear sample of per-cell `values` on `grid` at block (x, z), with FORMAT.md's convention
/// ("Grids": point samples, x wraps, z clamps).
pub fn bilinear_cells(grid: &GridDesc, values: &[f32], x: f64, z: f64) -> f64 {
    let (w, h) = (i64::from(grid.w), i64::from(grid.h));
    let r = f64::from(grid.r);
    let u = (x - grid.origin_block_x as f64) / r;
    let v = (z - grid.origin_block_z as f64) / r;
    let (i0, j0) = (u.floor() as i64, v.floor() as i64);
    let (fu, fv) = (u - i0 as f64, v - j0 as f64);
    let at = |i: i64, j: i64| {
        let i = i.rem_euclid(w);
        let j = j.clamp(0, h - 1);
        f64::from(values[(j * w + i) as usize])
    };
    let a = at(i0, j0) + (at(i0 + 1, j0) - at(i0, j0)) * fu;
    let b = at(i0, j0 + 1) + (at(i0 + 1, j0 + 1) - at(i0, j0 + 1)) * fu;
    a + (b - a) * fv
}

impl Finish {
    /// The present surface (block y) at block (x, z): the L1 elevation sampled bilinearly, the
    /// same surface a synthetic deposit world's baked terrain follows
    /// ([`synthetic::HistoryTerrain`]), so a body's clip and depth match the ground it is in.
    fn surface_at(&self, x: f64, z: f64) -> f64 {
        surface_y(
            bilinear_cells(&self.grid, &self.elev, x, z) as f32,
            &self.sc,
        )
    }

    fn radius_blocks(&self, t: &DepositType) -> f64 {
        t.cause_radius_km * 1000.0 / self.sc.m_per_block_h
    }

    /// The causes of a deposit of type `t` at (x, z) formed at `t_form`.
    fn cause_ids(&self, t: &DepositType, x: f64, z: f64, t_form: f64, slack: f64) -> Vec<EventId> {
        let any_time = t.formation_time == FormationTime::Cause;
        let r = self.radius_blocks(t);
        let mut out = Vec::new();
        for (_, kinds) in t.causes.components() {
            if kinds.is_empty() {
                continue;
            }
            if let Some(id) = self.causes.find(kinds, x, z, t_form, r, slack, any_time)
                && !out.contains(&id)
            {
                out.push(id);
            }
        }
        if out.is_empty()
            && let Some(id) = self.causes.nearest_before(x, z, t_form + slack, 3.0 * r)
        {
            out.push(id);
        }
        out
    }

    fn preservation_causes(&self, t: &DepositType, x: f64, z: f64, t_ev: f64) -> Vec<EventId> {
        if t.causes.preservation.is_empty() {
            return Vec::new();
        }
        self.causes
            .find(
                &t.causes.preservation,
                x,
                z,
                t_ev,
                self.radius_blocks(t),
                1.0e7,
                false,
            )
            .into_iter()
            .collect()
    }

    fn emit(
        &mut self,
        world: &mut WorldState,
        e: Event,
        x: f64,
        z: f64,
    ) -> Result<EventId, CoreError> {
        self.causes.push(&e, x, z);
        world.emit(e)
    }
}

/// The frame drawn ∝ `hazard` by a uniform `u` ∈ [0, 1) (`None` when the hazard is zero).
fn draw_frame(hazard: &[f32], u: f64) -> Option<usize> {
    let total: f64 = hazard.iter().map(|h| f64::from(*h)).sum();
    if total.is_nan() || total <= 0.0 {
        return None;
    }
    let mut left = u * total;
    let mut last = None;
    for (k, h) in hazard.iter().enumerate() {
        let h = f64::from(*h);
        if h <= 0.0 {
            continue;
        }
        if left < h {
            return Some(k);
        }
        left -= h;
        last = Some(k);
    }
    last
}

/// Erosion and burial at a column after `t_form` (partial overlap of the formation frame).
fn since(series: &[ColumnSample], t_form: f64) -> (f64, f64) {
    let mut e = 0.0;
    let mut b = 0.0;
    for s in series {
        let lo = s.t_years - s.dt_years;
        let overlap = (s.t_years - lo.max(t_form)).clamp(0.0, s.dt_years) / 1.0e6;
        e += s.erosion_m_myr * overlap;
        b += s.deposition_m_myr * overlap;
    }
    (e, b)
}

/// The most net erosion (erosion − deposition) at a column at any time after `t_form` (m, ≥ 0),
/// frame by frame as [`survival::ColumnNet`] reads it: how far the surface came down towards a
/// deposit formed then, whatever burial came after.
fn exhumed_max(series: &[ColumnSample], t_form: f64) -> f64 {
    let mut net = 0.0f64;
    let mut most = 0.0f64;
    for s in series {
        let lo = s.t_years - s.dt_years;
        let overlap = (s.t_years - lo.max(t_form)).clamp(0.0, s.dt_years) / 1.0e6;
        net += (s.erosion_m_myr - s.deposition_m_myr) * overlap;
        most = most.max(net);
    }
    most
}

/// When the net erosion after `t_form` first reached `depth_m` (years), if it did.
fn time_eroded_to(series: &[ColumnSample], t_form: f64, depth_m: f64) -> Option<f64> {
    let mut e = 0.0;
    for s in series {
        let lo = (s.t_years - s.dt_years).max(t_form);
        if s.t_years <= lo {
            continue;
        }
        let rate = s.erosion_m_myr - s.deposition_m_myr;
        let add = rate * (s.t_years - lo) / 1.0e6;
        if rate > 0.0 && e + add >= depth_m {
            return Some(lo + (depth_m - e) / rate * 1.0e6);
        }
        e += add;
    }
    None
}

impl DepositStage {
    /// The placement step: takes the history out of where it lives (the stage for a synthetic
    /// one, the world for a recorded one, whose placement then needs `&mut world`) and puts it
    /// back, so a checkpoint taken afterwards still carries it.
    fn finish(&mut self, world: &mut WorldState, ctx: &StageContext<'_>) -> Result<(), CoreError> {
        if matches!(self.source, Source::Synthetic(_)) {
            // Built in set-up, or again after a resume.
            ctx.pool.install(|| self.history().map(|_| ()))?;
            let h = self.history.take().expect("built above");
            let done = self.finish_with(world, ctx, h.as_ref());
            self.history = Some(h);
            done
        } else {
            let h = world.history.take().ok_or_else(|| {
                CoreError::InvalidState("the recorded history vanished from the world".into())
            })?;
            let done = self.finish_with(world, ctx, &h);
            world.history = Some(h);
            done
        }
    }

    #[allow(clippy::too_many_lines)]
    fn finish_with(
        &self,
        world: &mut WorldState,
        ctx: &StageContext<'_>,
        hist: &dyn FieldHistory,
    ) -> Result<(), CoreError> {
        let grid = self.grid();
        let n = grid.cells();
        let c = grid.circumference() as f64;
        let sc = self.scales;
        let last = self.frames - 1;
        if let Some((t_end, _)) = self.state.frame_times.last() {
            world.t_years = world.t_years.max(*t_end);
        }
        // The present frame, kept, and pass 2's frames one at a time.
        ctx.reserve(2 * self.frame_bytes())?;
        let present = ctx.pool.install(|| hist.frame(last))?;
        let elev = present
            .fields
            .get(fields::ELEVATION)
            .cloned()
            .unwrap_or_else(|| vec![0.0; n]);
        let radius_km = c * sc.m_per_block_h / 1000.0 / (2.0 * std::f64::consts::PI);
        let provinces = ctx.pool.install(|| {
            count::Provinces::new(&grid, self.seed, self.params.province_km, radius_km)
        });
        let receivers: Option<Vec<u32>> = hist.receivers().map(<[u32]>::to_vec);
        let mut f = Finish {
            grid: grid.clone(),
            c,
            sc,
            elev,
            causes: CauseIndex::new(world.circumference, &world.events, &world.sites),
            census: self
                .types
                .iter()
                .enumerate()
                .map(|(k, t)| TypeCensus {
                    id: t.id.clone(),
                    enabled: self.state.enabled[k],
                    missing: self.state.missing[k].clone(),
                    ..TypeCensus::default()
                })
                .collect(),
            placed: vec![Vec::new(); self.types.len()],
            present,
            paths: BTreeMap::new(),
            cover: self.planet.as_deref().and_then(|p| {
                // In the session's pool: the horizon bake is parallel, and outside an `install`
                // it ran on Rayon's global pool, every core of the machine whatever the session's
                // thread count (P1-17, `tests/session_pool.rs`).
                let cover = ctx
                    .pool
                    .install(|| crate::structure::Cover::new(&*world, p))
                    .ok()?;
                ctx.reserve(cover.bytes()).ok()?;
                Some(cover)
            }),
        };

        // ---- petroleum kitchens (their events come first so traps can cite them) -------------
        let pet_ty = self.petroleum_type();
        let mut kitchen_events: Vec<EventId> = Vec::new();
        let mut kitchens = Vec::new();
        let mut petroleum_note: Option<Value> = None;
        let mut pet_f: Vec<f64> = Vec::new();
        if let Some(pi) = pet_ty {
            let t = &self.types[pi];
            let r = t.petroleum.as_ref().expect("petroleum type");
            let (kept, small) = petroleum::kitchens(&grid, &self.state.pet, r, sc.m_per_block_h);
            kitchens = kept;
            let centres: Vec<u32> = kitchens.iter().map(|k| k.centre).collect();
            for (cell, path) in centres
                .iter()
                .zip(ctx.pool.install(|| trace(hist, self.frames, &centres))?)
            {
                f.paths.insert(*cell, path);
            }
            let mut areas: Vec<f64> = kitchens.iter().map(|k| k.tract.area_km2).collect();
            areas.sort_by(f64::total_cmp);
            let quantile = |q: f64| {
                areas
                    .get(((areas.len() as f64 - 1.0) * q).round() as usize)
                    .copied()
                    .unwrap_or(0.0)
            };
            petroleum_note = Some(json!({
                "kitchen_km2_p50": quantile(0.5),
                "kitchen_km2_p90": quantile(0.9),
                "kitchen_km2_max": quantile(1.0),
                "kitchens": kitchens.len(),
                "dropped_small": small,
                "min_kitchen_km2": r.min_kitchen_km2,
                "kitchen_km2_total": kitchens.iter().map(|k| k.tract.area_km2).sum::<f64>(),
            }));
            ctx.log.push(
                crate::log::Level::Info,
                format!(
                    "deposits petroleum: {} kitchens of at least {} km² (area p50 {:.0}, p90 {:.0}, max {:.0} km²; {} smaller regions of mature source rock dropped)",
                    kitchens.len(), r.min_kitchen_km2, quantile(0.5), quantile(0.9), quantile(1.0), small
                ),
            );
            for (k, kit) in kitchens.iter().enumerate() {
                let (x, z) = cluster::cell_xz(&grid, kit.centre);
                let ll = lat_lon_of(c, x, z);
                let (xf, zf) = self.xz_at(&f, kit.centre, (x, z), kit.t_onset);
                let causes = f.cause_ids(t, xf, zf, kit.t_onset, 1.0e7);
                let end = if kit.t_last_gen.is_finite() {
                    kit.t_last_gen.max(kit.t_onset)
                } else {
                    kit.t_onset
                };
                let mut e = world
                    .event(kit.t_onset, "deeptime:petroleum_generation", SOURCE)
                    .until(end)
                    .at(Place::Point {
                        lat: ll[0],
                        lon: ll[1],
                    })
                    .magnitude((kit.tti_max / r.tti_peak).min(4.0) as f32 / 4.0)
                    .param("kitchen", k as i64)
                    .param("tti_max", kit.tti_max)
                    .param("area_km2", kit.tract.area_km2)
                    .param("toc_mean", kit.toc_mean)
                    .param("deposit_type", t.id.as_str());
                for cse in causes {
                    e = e.caused_by(cse);
                }
                kitchen_events.push(f.emit(world, e, x, z)?);
            }
            // Charged-trap favourability: charge × trap × seal × reservoir (present values).
            let get = |name: &str| {
                f.present
                    .fields
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| vec![0.0; n])
            };
            let (trap, seal, res) = (
                get(&r.trap_field),
                get(&r.seal_field),
                get(&r.reservoir_field),
            );
            pet_f = (0..n)
                .map(|cell| {
                    let a = &self.state.pet[cell];
                    if a.t_trap.is_nan()
                        || f64::from(seal[cell]) < r.seal_threshold
                        || f64::from(res[cell]) < r.reservoir_threshold
                    {
                        return 0.0;
                    }
                    match petroleum::charge(
                        &grid,
                        &kitchens,
                        cell as u32,
                        f64::from(a.t_trap),
                        r,
                        sc.m_per_block_h,
                    ) {
                        Some(_) => f64::from(trap[cell] * seal[cell].min(1.0) * res[cell].min(1.0))
                            .min(1.0),
                        None => 0.0,
                    }
                })
                .collect();
            world.fields.insert(
                format!("deposits/favourability/{}", t.short()),
                Field {
                    grid: grid.clone(),
                    data: pet_f.iter().map(|v| *v as f32).collect(),
                },
            );
        }

        // ---- tracts ---------------------------------------------------------------------------
        let mut plans: Vec<TypeTracts> = Vec::new();
        for (ti, t) in self.types.iter().enumerate() {
            if !self.state.enabled[ti] || t.child_only {
                continue;
            }
            let fv = if Some(ti) == pet_ty {
                pet_f.clone()
            } else {
                favourability_of(&self.state.acc[ti])
            };
            if fv.is_empty() {
                continue;
            }
            let strength = t.province(self.params.province);
            let endow = provinces.endowment(&t.id, strength);
            let tracts =
                tracts::find_tracts(&grid, &fv, t.sizing.tract_threshold, sc.m_per_block_h);
            f.census[ti].tracts = tracts.len();
            if let Some((cell, &fmax)) = fv
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
                && let Some(a) = self.state.acc[ti].get(cell)
            {
                let (x, z) = cluster::cell_xz(&grid, cell as u32);
                let ll = lat_lon_of(c, x, z);
                f.census[ti].best_cell = Some(json!({
                    "lat": ll[0],
                    "lon": ll[1],
                    "favourability": fmax,
                    "limiting": COMPONENTS[a.limiting()],
                    "scores": COMPONENTS
                        .iter()
                        .zip(a.scores)
                        .map(|(n, q)| (*n, rules::dq8(q)))
                        .collect::<BTreeMap<_, _>>(),
                }));
            }
            let mut cells: Vec<u32> = tracts
                .iter()
                .flat_map(|tr| tr.cells.iter().copied())
                .collect();
            cells.sort_unstable();
            let formation = if Some(ti) == pet_ty {
                FormationKind::Charge
            } else if t.formation_time == FormationTime::Cause {
                FormationKind::Start
            } else {
                FormationKind::Hazard
            };
            plans.push(TypeTracts {
                ti,
                fv,
                tracts,
                endow,
                strength,
                cells,
                formation,
            });
        }

        // ---- pass 2: every tract cell's column through the history -------------------------
        let t_pass = std::time::Instant::now();
        let cols = self.column_pass(&plans, hist, ctx)?;
        ctx.log.push(
            crate::log::Level::Debug,
            format!(
                "deposits: pass 2 over {} tract cells in {:.2} s",
                cols.cells.len(),
                t_pass.elapsed().as_secs_f64()
            ),
        );
        let t_counts = std::time::Instant::now();

        // ---- counts (the survivor formula), sizes, positions --------------------------------
        let mut candidates: Vec<Candidate> = Vec::new();
        for (pi, plan) in plans.iter().enumerate() {
            let ti = plan.ti;
            let t = &self.types[ti];
            let (fv, endow, tracts) = (&plan.fv, &plan.endow, &plan.tracts);
            // A stacked type (coal) lies in the sedimentary cover (`geometry::fit_stack`): per tract
            // cell the room the cover leaves above its base (m, in the stack's vertical scale).
            // Fields form only where it holds a seam, and a field survives only with its top in
            // it, so the survivor formula counts what the placement will keep.
            let room_m: Option<Vec<f64>> = match (&t.geometry.stack, &f.cover) {
                (Some(_), Some(cover)) => Some(ctx.pool.install(|| {
                    plan.cells
                        .par_iter()
                        .map(|&cell| {
                            let (x, z) = cluster::cell_xz(&grid, cell);
                            let surf = surface_y(f.elev[cell as usize], &sc);
                            (cover.blocks_at(x, z, surf) - geometry::COVER_MARGIN_BLOCKS)
                                * sc.m_per_block_v
                        })
                        .collect()
                })),
                _ => None,
            };
            // The cover factor of tract cell j: 1 where the cover holds at least a block of seams.
            let holds = |j: usize| -> f64 {
                room_m
                    .as_ref()
                    .map_or(1.0, |r| f64::from(u8::from(r[j] >= sc.m_per_block_v)))
            };
            let holds_cell = |cell: u32| plan.cells.binary_search(&cell).map_or(0.0, holds);
            // p(c): the chance a deposit formed in each tract cell survives to the present.
            let model = survival::FateModel::new(t);
            let pet_rules = t.petroleum.as_ref();
            let p_cell: Vec<(f64, f64)> = ctx.pool.install(|| {
                plan.cells
                    .par_iter()
                    .enumerate()
                    .map(|(j, &cell)| {
                        let Some(i) = cols.index(cell) else {
                            return (0.0, 0.0);
                        };
                        if holds(j) == 0.0 {
                            return (0.0, 0.0);
                        }
                        let net = cols.net(i, &self.state.frame_times);
                        let mut limit = survival::too_deep_limit_m(
                            surface_y(f.elev[cell as usize], &sc),
                            sc.min_y,
                            sc.m_per_block_v,
                        );
                        if let Some(r) = &room_m {
                            // A stack's top seam must lie in the cover.
                            limit = limit.min(r[j]);
                        }
                        let when = match plan.formation {
                            FormationKind::Hazard => {
                                survival::Formation::Hazard(cols.hazard_of(pi, j))
                            }
                            FormationKind::Start => survival::Formation::Start,
                            FormationKind::Charge => {
                                let Some(r) = pet_rules else {
                                    return (0.0, 0.0);
                                };
                                let t_trap = f64::from(self.state.pet[cell as usize].t_trap);
                                let Some(ch) = petroleum::charge(
                                    &grid,
                                    &kitchens,
                                    cell,
                                    t_trap,
                                    r,
                                    sc.m_per_block_h,
                                ) else {
                                    return (0.0, 0.0);
                                };
                                let k = self.frame_at(ch.t_charge);
                                let (te, dt) = self.state.frame_times[k];
                                survival::Formation::At {
                                    k,
                                    u: if dt > 0.0 {
                                        (te - ch.t_charge) / dt
                                    } else {
                                        0.0
                                    },
                                }
                            }
                        };
                        (
                            survival::preservation(&model, &net, when, limit, false),
                            survival::preservation(&model, &net, when, limit, true),
                        )
                    })
                    .collect()
            });
            // p: survives as a site; p₁: survives with its top within the assessment depth (what
            // the density counts).
            let p_of = |cell: u32| plan.cells.binary_search(&cell).map_or(0.0, |j| p_cell[j].0);
            let p1_of = |cell: u32| plan.cells.binary_search(&cell).map_or(0.0, |j| p_cell[j].1);
            // Per tract: its endowment E_i, its formation weight φ_i = A·F̄·E and its preservation
            // p̄_i (p averaged with the position weights F·E); for a stacked type F carries the
            // cover factor (F̄ the tract's mean of F·cover).
            let mut areas = Vec::with_capacity(tracts.len());
            let mut phis = Vec::with_capacity(tracts.len());
            let mut pbars = Vec::with_capacity(tracts.len());
            let mut pbars_site = Vec::with_capacity(tracts.len());
            for tr in tracts {
                let e = tr
                    .cells
                    .iter()
                    .map(|c| endow[provinces.of_cell[*c as usize] as usize])
                    .sum::<f64>()
                    / tr.cells.len() as f64;
                let (mut num, mut num_s, mut den, mut plain, mut plain_s) =
                    (0.0, 0.0, 0.0, 0.0, 0.0);
                let mut f_held = 0.0;
                for &c in &tr.cells {
                    let g = holds_cell(c);
                    f_held += fv[c as usize] * g;
                    let w = fv[c as usize] * g * endow[provinces.of_cell[c as usize] as usize];
                    let (p1, p) = (p1_of(c), p_of(c));
                    num += w * p1;
                    num_s += w * p;
                    den += w;
                    plain += p1;
                    plain_s += p;
                }
                let cells = tr.cells.len() as f64;
                pbars.push(if den > 0.0 { num / den } else { plain / cells });
                pbars_site.push(if den > 0.0 {
                    num_s / den
                } else {
                    plain_s / cells
                });
                areas.push(tr.area_km2);
                let mean_f = if room_m.is_some() {
                    f_held / cells
                } else {
                    tr.mean_f
                };
                phis.push(tr.area_km2 * mean_f * e);
            }
            let count_mult = self.params.count_mult * t.balance.count_mult;
            let fp = survival::plan(
                t.sizing.density_target(),
                count_mult,
                &areas,
                &phis,
                &pbars,
                self.params.max_formed_per_survivor,
            );
            {
                let cen = &mut f.census[ti];
                cen.permissive_km2 = areas.iter().sum::<f64>() + 0.0;
                cen.mean_f = if cen.permissive_km2 > 0.0 {
                    tracts.iter().map(|tr| tr.mean_f * tr.area_km2).sum::<f64>()
                        / cen.permissive_km2
                } else {
                    0.0
                };
                cen.target = fp.target;
                cen.preservation = fp.preservation;
                cen.formation_mult = fp.formation_mult;
                cen.capped = fp.capped;
                // (+ 0.0: an empty sum is −0.0.)
                cen.expected_formed = fp.formed.iter().sum::<f64>() + 0.0;
                cen.expected = fp.expected_survivors;
                cen.density = t.sizing.density_target();
                cen.expected_sites = fp
                    .formed
                    .iter()
                    .zip(&pbars_site)
                    .map(|(l, p)| l * p)
                    .sum::<f64>()
                    + 0.0;
            }
            if fp.capped {
                ctx.log.push(
                    crate::log::Level::Warn,
                    format!(
                        "deposits {}: the planet preserves {:.1} % of what forms; formation stops \
                         at {:.0} per survivor, so about {:.2} of the {:.2} survivors the density \
                         asks for are expected",
                        t.id,
                        100.0 * fp.preservation,
                        fp.formation_mult,
                        fp.expected_survivors,
                        fp.target
                    ),
                );
            }
            let model_size = SizeModel::new(&t.sizing);
            let mut rng = Rng::stream(self.seed, &format!("deposits/{}", t.id));
            let per_camp = match plan.strength {
                Province::Off => 1.0,
                Province::Weak => 2.0,
                Province::Medium => 3.0,
                Province::Strong => 4.0,
            };
            let spacing = Spacing {
                camp_spacing: t.sizing.clustering.camp_spacing_km * 1000.0 / sc.m_per_block_h,
                sigma: t.sizing.clustering.cluster_radius_km * 1000.0 / sc.m_per_block_h,
                min_sep: t.sizing.min_separation_m / sc.m_per_block_h,
                per_camp,
                adjoining: t.sizing.clustering.adjoining(),
            };
            for (i, tract) in tracts.iter().enumerate() {
                let lambda = fp.formed[i];
                let k = rng.poisson(lambda);
                f.census[ti].drawn += k;
                if k == 0 {
                    continue;
                }
                let sizes: Vec<SizeDraw> = (0..k).map(|_| model_size.draw(&mut rng)).collect();
                let radii: Vec<f64> = sizes
                    .iter()
                    .map(|s| geometry::radius_blocks(t, s.tonnage_mt * 1.0e6, &sc))
                    .collect();
                let weights: Vec<f64> = tract
                    .cells
                    .iter()
                    .map(|c| {
                        fv[*c as usize]
                            * holds_cell(*c)
                            * endow[provinces.of_cell[*c as usize] as usize]
                    })
                    .collect();
                let set: BTreeSet<u32> = tract.cells.iter().copied().collect();
                let allowed =
                    |c: u32| set.contains(&c) && fv[c as usize] > 0.0 && holds_cell(c) > 0.0;
                let space = TractSpace {
                    grid: &grid,
                    cells: &tract.cells,
                    weights: &weights,
                    allowed: &allowed,
                };
                let footprints: Option<Vec<[f64; 2]>> = spacing.adjoining.then(|| {
                    sizes
                        .iter()
                        .map(|s| geometry::footprint_blocks(t, s.tonnage_mt * 1.0e6, &sc))
                        .collect()
                });
                // A field claims its footprint only if it will survive (its fate decided here the
                // way `materialise` decides it, from the same random streams).
                let mut placed = std::mem::take(&mut f.placed[ti]);
                let t_end = self
                    .state
                    .frame_times
                    .last()
                    .map_or(world.t_years, |(t, _)| *t);
                let mut claims = |k: usize, pt: &Point| -> bool {
                    if !spacing.adjoining {
                        return true;
                    }
                    let (Some(i), Ok(j)) =
                        (cols.index(pt.cell), plan.cells.binary_search(&pt.cell))
                    else {
                        return true;
                    };
                    let hazard = match plan.formation {
                        FormationKind::Hazard => Some(cols.hazard_of(pi, j)),
                        _ => None,
                    };
                    let col = cols.series(i, &self.state.frame_times);
                    self.survives(&f, ti, pt, &sizes[k], hazard, &col, t_end)
                };
                let (pts, dropped) = cluster::place(
                    &mut rng,
                    &space,
                    &spacing,
                    &radii,
                    footprints.as_deref(),
                    &mut placed,
                    &mut claims,
                );
                f.placed[ti] = placed;
                f.census[ti].dropped_separation += dropped;
                f.census[ti].placed += pts.len();
                for (k, p) in &pts {
                    let size = sizes[*k].clone();
                    let charge = if Some(ti) == pet_ty {
                        let r = t.petroleum.as_ref().expect("petroleum");
                        let t_trap = f64::from(self.state.pet[p.cell as usize].t_trap);
                        petroleum::charge(&grid, &kitchens, p.cell, t_trap, r, sc.m_per_block_h)
                            .map(|ch| (ch.kitchen, ch.t_charge, ch.migration_km))
                    } else {
                        None
                    };
                    let hazard = match (plan.formation, plan.cells.binary_search(&p.cell)) {
                        (FormationKind::Hazard, Ok(j)) => Some(cols.hazard_of(pi, j).to_vec()),
                        _ => None,
                    };
                    f.census[ti].expected_at_positions += p_of(p.cell);
                    candidates.push(Candidate {
                        ty: ti,
                        point: *p,
                        size,
                        endowment: endow[provinces.of_cell[p.cell as usize] as usize],
                        fav: fv[p.cell as usize],
                        charge,
                        hazard,
                        p_survive: p_of(p.cell),
                    });
                }
            }
        }

        ctx.log.push(
            crate::log::Level::Debug,
            format!(
                "deposits: survival and placement in {:.2} s",
                t_counts.elapsed().as_secs_f64()
            ),
        );
        // ---- the chosen cells' columns (the ledger, the supergene windows) ---------------------
        let mut series: BTreeMap<u32, Vec<ColumnSample>> = BTreeMap::new();
        for cand in &candidates {
            let cell = cand.point.cell;
            if series.contains_key(&cell) {
                continue;
            }
            let i = cols
                .index(cell)
                .expect("a candidate lies in a tract, whose cells were all replayed");
            series.insert(cell, cols.series(i, &self.state.frame_times));
            f.paths.insert(cell, cols.paths[i].clone());
        }

        // ---- materialise: ledger, sites, bodies, events, children -------------------------
        for cand in &candidates {
            self.materialise(
                &mut f,
                world,
                cand,
                &series,
                receivers.as_deref(),
                &kitchen_events,
            )?;
        }

        // ---- playability -------------------------------------------------------------------
        self.guarantee(&mut f, world, hist, ctx)?;
        for s in &world.sites {
            if let Some(ti) = self.types.iter().position(|t| t.id == s.deposit_type)
                && bilinear_cells(
                    &f.grid,
                    &f.elev,
                    f64::from(s.center[0]),
                    f64::from(s.center[2]),
                ) > 0.0
            {
                f.census[ti].on_land += 1;
            }
        }

        // ---- the sites layer and the census --------------------------------------------------
        let mut layer = vec![f32::NAN; n];
        for s in &world.sites {
            let ti = self
                .types
                .iter()
                .position(|t| t.id == s.deposit_type)
                .unwrap_or(0);
            let [lo, hi] = s.aabb;
            let r = f64::from(grid.r);
            let mut x = f64::from(lo[0]);
            while x <= f64::from(hi[0]) + r {
                let mut z = f64::from(lo[2]);
                while z <= f64::from(hi[2]) + r {
                    let cell =
                        cluster::cell_of(&grid, x.min(f64::from(hi[0])), z.min(f64::from(hi[2])));
                    layer[cell as usize] = (ti + 1) as f32;
                    z += r;
                }
                x += r;
            }
        }
        world.fields.insert(
            SITES_LAYER.to_owned(),
            Field {
                grid: grid.clone(),
                data: layer,
            },
        );
        let skipped: Vec<Value> = f
            .census
            .iter()
            .filter(|tc| !tc.enabled)
            .map(|tc| json!({ "type": tc.id, "missing": tc.missing }))
            .collect();
        let census = json!({
            "stage": STAGE_NAME,
            "history": {
                "source": match self.source {
                    Source::Synthetic(_) => "synthetic",
                    Source::Recorded { .. } => "recorded",
                },
                "frames": self.frames,
                "names": self.state.field_names,
            },
            "skipped": skipped,
            "fallbacks": self.state.fallbacks,
            "petroleum": petroleum_note,
            "count_mult": self.params.count_mult,
            "richness_mult": self.params.richness_mult,
            // The game's background scatter multiplier (`BackgroundScatterFeature.scatterMult`).
            "scatter_mult": self.params.scatter_mult,
            "inherited_bif_share": self.params.inherited_bif_share,
            "bif_basin_km2": self.params.bif_basin_km2,
            // What the surface stage dealt (`bif_basins`): basins, and their share of the stable
            // continental interiors at the first step.
            "bif_basins": world.globals.get(crate::surface::BIF_BASINS_GLOBAL),
            "bif_basin_share": world.globals.get(crate::surface::BIF_BASIN_SHARE_GLOBAL),
            "province": self.params.province.name(),
            "province_km": self.params.province_km,
            "provinces": provinces.count,
            "types": f.census,
            "sites": world.sites.len(),
            "bodies": world.bodies.len(),
            "max_formed_per_survivor": self.params.max_formed_per_survivor,
            "brine_flow_km": self.params.brine_flow_km,
            "formula": "survivors: target = count_mult · ρ/1e5 · Σ_tracts A_km2, ρ the mean of a measured USGS density distribution or the P50 of an estimate (plan §8.6 as built 2026-09-29: USGS densities count survivors whose top lies within the assessment depth, 1 km for hard-rock types); formed λ_i = target · φ_i / Σ φ_j p̄₁_j with φ = A · mean_F · endowment and p̄₁ the tract's modelled chance of surviving within the assessment depth, capped at max_formed_per_survivor; expected = Σ λ_i p̄₁_i; expected_sites = Σ λ_i p̄_i counts the deeper survivors too",
        });
        world
            .notes
            .insert(STAGE_NAME.to_owned(), census.to_string());
        for tc in &f.census {
            ctx.log.push(
                crate::log::Level::Info,
                format!(
                    "deposits {}: sites={} counted={} target={:.2} expected={:.2} expected_sites={:.2} at_positions={:.2} preservation={:.3} formed_expected={:.2} drawn={} tracts={} permissive_km2={:.0} mean_F={:.3} eroded={} truncated={} too_deep={} children={} guaranteed={}{}",
                    tc.id, tc.sites, tc.counted, tc.target, tc.expected, tc.expected_sites, tc.expected_at_positions, tc.preservation, tc.expected_formed, tc.drawn, tc.tracts, tc.permissive_km2,
                    tc.mean_f, tc.eroded, tc.truncated, tc.too_deep, tc.children, tc.guaranteed,
                    if tc.capped { " CAPPED" } else { "" }
                ),
            );
        }
        Ok(())
    }
}

impl DepositStage {
    /// Pass 2 (see the module doc): replays every frame at every tract cell, where its crust was
    /// then, recording the column's erosion, deposition, water table and precipitation and each
    /// hazard type's favourability hazard.
    #[allow(clippy::needless_range_loop)] // frame k indexes every path and every array
    fn column_pass(
        &self,
        plans: &[TypeTracts],
        hist: &dyn FieldHistory,
        ctx: &StageContext<'_>,
    ) -> Result<Columns, CoreError> {
        let cells: Vec<u32> = plans
            .iter()
            .flat_map(|p| p.cells.iter().copied())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let nf = self.frames;
        let hazard_cells: usize = plans
            .iter()
            .filter(|p| p.formation == FormationKind::Hazard)
            .map(|p| p.cells.len())
            .sum();
        ctx.reserve(((cells.len() * (4 * 4 + 1 + 4) + hazard_cells * 4) * nf) as u64)?;
        let paths = ctx.pool.install(|| trace(hist, nf, &cells))?;
        let n = cells.len() * nf;
        let mut cols = Columns {
            frames: nf,
            alive: vec![false; n],
            erosion: vec![0.0; n],
            deposition: vec![0.0; n],
            water_table: vec![0.0; n],
            precip: vec![0.0; n],
            hazard: plans
                .iter()
                .map(|p| {
                    if p.formation == FormationKind::Hazard {
                        vec![0.0; p.cells.len() * nf]
                    } else {
                        Vec::new()
                    }
                })
                .collect(),
            paths: Vec::new(),
            cells,
        };
        // Each type's cells as positions in the union.
        let pos: Vec<Vec<usize>> = plans
            .iter()
            .map(|p| {
                p.cells
                    .iter()
                    .map(|c| cols.index(*c).expect("in the union"))
                    .collect()
            })
            .collect();
        let rules: Vec<Option<Compiled>> = plans
            .iter()
            .map(|p| {
                (p.formation == FormationKind::Hazard).then(|| Compiled::new(&self.types[p.ti]))
            })
            .collect();
        for k in 0..nf {
            let frame = ctx.pool.install(|| self.frame_of(hist, k))?;
            let live: Vec<usize> = (0..cols.cells.len())
                .filter(|&i| paths[i][k] != NO_ORIGIN)
                .collect();
            let at: Vec<u32> = live.iter().map(|&i| paths[i][k]).collect();
            for (&i, smp) in live.iter().zip(children::sample(&frame, &at)) {
                let o = i * nf + k;
                cols.alive[o] = true;
                cols.erosion[o] = smp.erosion_m_myr as f32;
                cols.deposition[o] = smp.deposition_m_myr as f32;
                cols.water_table[o] = smp.water_table_m as f32;
                cols.precip[o] = smp.precip_mm as f32;
            }
            for (pi, r) in rules.iter().enumerate() {
                let Some(r) = r else { continue };
                let Some(eval) = FrameEval::new(r, &frame) else {
                    continue;
                };
                let scale = frame.dt_myr() / r.tau_myr;
                let h: Vec<f32> = ctx.pool.install(|| {
                    pos[pi]
                        .par_iter()
                        .map(|&i| {
                            let q = paths[i][k];
                            if q == NO_ORIGIN {
                                return 0.0;
                            }
                            // The same hazard pass 1 folds into the favourability.
                            let f = eval.at(q as usize).f.min(0.999);
                            (-(scale * (1.0 - f).ln())) as f32
                        })
                        .collect()
                });
                for (j, v) in h.into_iter().enumerate() {
                    cols.hazard[pi][j * nf + k] = v;
                }
            }
        }
        cols.paths = paths;
        Ok(cols)
    }

    /// The frame whose interval contains year `t` (the first one ending at or after it).
    fn frame_at(&self, t: f64) -> usize {
        let times = &self.state.frame_times;
        times
            .iter()
            .position(|(te, _)| t <= *te)
            .unwrap_or(times.len().saturating_sub(1))
    }

    /// Where a deposit now at (`at`, in `cell`) was at year `t`: the point shifted by how far the
    /// crust's cell has moved since (block metric; a history that does not move gives `at`). The
    /// events that made a deposit are looked up here, not where its crust is today.
    fn xz_at(&self, f: &Finish, cell: u32, at: (f64, f64), t: f64) -> (f64, f64) {
        let Some(path) = f.paths.get(&cell) else {
            return at;
        };
        let k = self.frame_at(t).min(path.len().saturating_sub(1));
        let Some(&then) = path
            .get(k..)
            .and_then(|p| p.iter().find(|c| **c != NO_ORIGIN))
        else {
            return at;
        };
        let (fx, fz) = cluster::cell_xz(&f.grid, then);
        let (px, pz) = cluster::cell_xz(&f.grid, cell);
        let mut dx = fx - px;
        if dx > f.c / 2.0 {
            dx -= f.c;
        } else if dx < -f.c / 2.0 {
            dx += f.c;
        }
        (at.0 + dx, at.1 + (fz - pz))
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    /// A deposit's formation time, and the random stream its materialisation goes on with: a
    /// charged trap at its charge; a type that forms during the history at a frame drawn ∝ its
    /// cell's favourability hazard, uniform within the frame (the distribution the survivor
    /// formula integrated over, [`survival`]).
    fn formation_draw(
        &self,
        ti: usize,
        cell: u32,
        x: f64,
        hazard: Option<&[f32]>,
        charge_t: Option<f64>,
        t_end: f64,
    ) -> (f64, Rng) {
        let t = &self.types[ti];
        let mut rng = Rng::stream(self.seed, &format!("materialise/{}/{cell}/{x:.3}", t.id));
        let acc = self.state.acc[ti].get(cell as usize).copied();
        let hazard_frame = hazard.and_then(|h| draw_frame(h, rng.uniform()));
        let t_form = match (charge_t, hazard_frame, acc) {
            (Some(tc), _, _) => tc,
            (None, Some(k), _) => {
                let (te, dt) = self.state.frame_times[k];
                te - dt * rng.uniform()
            }
            (None, None, Some(a)) if a.k_best != u16::MAX => {
                let (te, dt) = self.state.frame_times[a.k_best as usize];
                te - dt * rng.uniform()
            }
            _ => t_end,
        };
        (t_form, rng)
    }

    /// A stacked deposit's seams and where its centre lies (coal measures: their number from the
    /// sediment the history laid down under the site while it was coal-forming; the top seam
    /// half the measures above the centre), held in the site's sedimentary cover
    /// (`geometry::fit_stack`: the seams under the cover's base are not there, none where there
    /// is no cover). (None, the ledger's depth, None) for a type without a stack.
    #[allow(clippy::too_many_arguments)]
    fn stack_plan(
        &self,
        f: &Finish,
        ti: usize,
        (cell, x, z): (u32, f64, f64),
        surf: f64,
        centre_depth: f64,
        half_m: f64,
        hazard: Option<&[f32]>,
        col: &[ColumnSample],
    ) -> (Option<geometry::SeamPlan>, f64, Option<geometry::StackFit>) {
        let t = &self.types[ti];
        let Some(k) = &t.geometry.stack else {
            return (None, centre_depth, None);
        };
        let frame_years: Vec<f64> = self.state.frame_times.iter().map(|(te, _)| *te).collect();
        let measures = geometry::measures_thickness_m(
            hazard.unwrap_or(&[]),
            t.integration_myr,
            &frame_years,
            col,
        );
        let stream = format!("stack/{}/{cell}/{x:.3}", t.id);
        let plan = geometry::seam_plan(
            k,
            measures,
            2.0 * half_m,
            &mut Rng::stream(self.seed, &stream),
        );
        let Some(cover) = &f.cover else {
            return (Some(plan), centre_depth, None);
        };
        let fit = geometry::fit_stack(k, &plan, centre_depth, cover.blocks_at(x, z, surf), &f.sc);
        match fit {
            geometry::StackFit::Capped {
                measures_m,
                centre_depth_m,
            } => {
                let plan = if measures_m < plan.measures_m {
                    geometry::seam_plan(
                        k,
                        measures_m,
                        2.0 * half_m,
                        &mut Rng::stream(self.seed, &stream),
                    )
                } else {
                    plan
                };
                (Some(plan), centre_depth_m, Some(fit))
            }
            _ => (Some(plan), centre_depth, Some(fit)),
        }
    }

    /// Whether a field of type `ti` and `size` placed at `pt` becomes a site: its fate decided
    /// exactly as [`Self::materialise`] decides it (the same formation draw, ledger, stack in the
    /// cover, build-range check and seams below the ground), without touching the world. The
    /// placement asks it so that only surviving fields reserve their footprint.
    #[allow(clippy::too_many_arguments)]
    fn survives(
        &self,
        f: &Finish,
        ti: usize,
        pt: &Point,
        size: &SizeDraw,
        hazard: Option<&[f32]>,
        col: &[ColumnSample],
        t_end: f64,
    ) -> bool {
        let t = &self.types[ti];
        if t.formation_time == FormationTime::Cause {
            // Its formation time follows its causes; the placement does not replay those.
            return true;
        }
        let sc = f.sc;
        let (x, z, cell) = (pt.x, pt.z, pt.cell);
        let (t_form, mut rng) = self.formation_draw(ti, cell, x, hazard, None, t_end);
        let tonnage_t = size.tonnage_mt * 1.0e6;
        let half_m = geometry::half_height_m(t, tonnage_t);
        let (eroded_m, buried_m) = since(col, t_form);
        let ledger = Ledger {
            formation_depth_m: t.preservation.formation_depth_m,
            half_height_m: half_m,
            eroded_m,
            buried_m,
            exhumed_max_m: exhumed_max(col, t_form),
        };
        let fate = ledger.apply(t.preservation.erosion_behavior, t.shape().metric());
        if matches!(fate, Fate::Eroded) {
            return false;
        }
        let surf = f.surface_at(x, z);
        let (seams, centre_depth, fit) = self.stack_plan(
            f,
            ti,
            (cell, x, z),
            surf,
            ledger.centre_depth_m(),
            half_m,
            hazard,
            col,
        );
        if fit == Some(geometry::StackFit::NoCover) {
            return false;
        }
        let centre_y = surf - centre_depth / sc.m_per_block_v;
        let top_y = seams.as_ref().map_or(centre_y, |p| {
            surf - (centre_depth - p.measures_m / 2.0) / sc.m_per_block_v
        });
        let half_blocks = half_m / sc.m_per_block_v * t.balance.size_mult_v;
        if top_y + half_blocks < f64::from(sc.min_y + 4) {
            return false;
        }
        let (Some(k), Some(plan)) = (&t.geometry.stack, &seams) else {
            return true;
        };
        // The same draws `materialise` makes next (dip, then yaw) for the stack's bodies.
        let dip = if t.geometry.dip_deg[1] > t.geometry.dip_deg[0] {
            (t.geometry.dip_deg[0]
                + rng.uniform() * (t.geometry.dip_deg[1] - t.geometry.dip_deg[0]))
                .to_radians()
        } else {
            t.geometry.dip_deg[0].to_radians()
        };
        let yaw_draw = rng.uniform() * std::f64::consts::PI;
        let placement = Placement {
            x,
            z,
            // A truncated body buried again is placed under the surface that cut it.
            surface_y: surf - fate.cut_depth_m() / sc.m_per_block_v,
            centre_depth_m: centre_depth - fate.cut_depth_m(),
            yaw: pt.ellipse.map_or(yaw_draw, |e| e[2]) as f32,
            dip: dip as f32,
            truncated: matches!(fate, Fate::Truncated { .. }),
            water_table_m: children::present_water_table_m(col),
        };
        let mut n = 0;
        !geometry::stack_bodies(
            t,
            k,
            0,
            tonnage_t,
            &Grades::new(),
            &placement,
            plan,
            &sc,
            &mut || {
                n += 1;
                n
            },
        )
        .bodies
        .is_empty()
    }

    fn materialise(
        &self,
        f: &mut Finish,
        world: &mut WorldState,
        cand: &Candidate,
        series: &BTreeMap<u32, Vec<ColumnSample>>,
        receivers: Option<&[u32]>,
        kitchen_events: &[EventId],
    ) -> Result<(), CoreError> {
        let t = &self.types[cand.ty];
        let sc = f.sc;
        let c = f.c;
        let (x, z) = (cand.point.x, cand.point.z);
        let cell = cand.point.cell;
        let col = &series[&cell];
        let acc = self.state.acc[cand.ty].get(cell as usize).copied();
        let slack = self.state.frame_times.first().map_or(1.0e7, |(_, dt)| *dt);
        let t_end = self
            .state
            .frame_times
            .last()
            .map_or(world.t_years, |(t, _)| *t);
        let (mut t_form, mut rng) = self.formation_draw(
            cand.ty,
            cell,
            x,
            cand.hazard.as_deref(),
            cand.charge.map(|c| c.1),
            t_end,
        );
        let (xf, zf) = self.xz_at(f, cell, (x, z), t_form);
        let mut causes = f.cause_ids(t, xf, zf, t_form, slack);
        if t.formation_time == FormationTime::Cause
            && let Some(first) = causes
                .iter()
                .filter_map(|id| f.causes.start_of(*id))
                .reduce(f64::min)
        {
            t_form = first;
        }
        if let Some((k, _, _)) = cand.charge
            && let Some(id) = kitchen_events.get(k)
        {
            causes.insert(0, *id);
        }
        // Ledger.
        let tonnage_t = cand.size.tonnage_mt * 1.0e6;
        let half_m = geometry::half_height_m(t, tonnage_t);
        let (eroded_m, buried_m) = since(col, t_form);
        let ledger = Ledger {
            formation_depth_m: t.preservation.formation_depth_m,
            half_height_m: half_m,
            eroded_m,
            buried_m,
            exhumed_max_m: exhumed_max(col, t_form),
        };
        let fate = ledger.apply(t.preservation.erosion_behavior, t.shape().metric());
        let surf = f.surface_at(x, z);
        let ll = lat_lon_of(c, x, z);
        let scores = acc.map(|a| a.scores.map(rules::dq8)).unwrap_or([1.0; 6]);
        let limiting = acc.map_or("none", |a| COMPONENTS[a.limiting()]);
        let host_class = acc
            .and_then(|a| {
                let r = Compiled::new(t);
                r.host_classes.get(a.host as usize).cloned()
            })
            .unwrap_or_default();
        let favourability = cand.fav;
        let neighbours = self.neighbours_limited_by(cand.ty, cell, &f.grid);
        let base_event = |world: &mut WorldState, kind: &str, t0: f64| {
            let mut e = world
                .event(t0, kind, SOURCE)
                .param("deposit_type", t.id.as_str())
                .param("favourability", favourability);
            // Rule-based types record their component scores; petroleum's favourability is the
            // charged trap × seal × reservoir and has none.
            if acc.is_some() {
                e = e
                    .param("score_source", scores[0])
                    .param("score_driver", scores[1])
                    .param("score_pathway", scores[2])
                    .param("score_trap", scores[3])
                    .param("score_host", scores[4])
                    .param("score_clock", scores[5])
                    .param("limiting", limiting)
                    .param("neighbours_limited_by", neighbours.as_str())
                    .param("host_class", host_class.as_str());
            }
            let mut e = e
                .param("tonnage_mt", cand.size.tonnage_mt)
                .param("tonnage_quantile", cand.size.tonnage_q)
                .param("p_survive", cand.p_survive)
                .param("formation_depth_m", t.preservation.formation_depth_m)
                // The ledger: erosion and burial of the column since formation.
                .param("eroded_m", eroded_m)
                .param("buried_m", buried_m)
                .param("exhumed_max_m", ledger.exhumed_max_m)
                .param("province_endowment", cand.endowment)
                .magnitude(cand.size.tonnage_q as f32);
            for id in &causes {
                e = e.caused_by(*id);
            }
            e
        };
        let ti = cand.ty;
        match fate {
            Fate::Eroded => {
                f.census[ti].eroded += 1;
                // The deposit existed; its formation and destruction stay in the history.
                let e = base_event(world, "deeptime:deposit_formation", t_form)
                    .at(Place::Point {
                        lat: ll[0],
                        lon: ll[1],
                    })
                    .param("site", -1i64)
                    .param("fate", "eroded");
                let formed = f.emit(world, e, x, z)?;
                let t_gone = time_eroded_to(col, t_form, t.preservation.formation_depth_m + half_m)
                    .unwrap_or(t_form);
                let mut ex = world
                    .event(t_gone, "deeptime:deposit_exhumation", SOURCE)
                    .at(Place::Point {
                        lat: ll[0],
                        lon: ll[1],
                    })
                    .param("deposit_type", t.id.as_str())
                    .param("site", -1i64)
                    .param("eroded_m", eroded_m)
                    .param("fraction_left", 0.0)
                    .param("destroyed", 1i64)
                    .caused_by(formed);
                let (xg, zg) = self.xz_at(f, cell, (x, z), t_gone);
                for id in f.preservation_causes(t, xg, zg, t_gone) {
                    ex = ex.caused_by(id);
                }
                let gone = f.emit(world, ex, x, z)?;
                if t.preservation.erosion_behavior == ErosionBehavior::SpawnPlacer {
                    for child in t
                        .geometry
                        .children
                        .iter()
                        .filter(|c| c.relation == Relation::Downstream)
                    {
                        self.spawn_placer(
                            f,
                            world,
                            cand,
                            &child.type_id,
                            gone,
                            t_gone,
                            receivers,
                            Shed {
                                eroded_fraction: 1.0,
                                parent_site: None,
                            },
                            &mut rng,
                        )?;
                    }
                }
                return Ok(());
            }
            Fate::Buried { .. } | Fate::Truncated { .. } => {}
        }
        let (seams, centre_depth, fit) = self.stack_plan(
            f,
            cand.ty,
            (cell, x, z),
            surf,
            ledger.centre_depth_m(),
            half_m,
            cand.hazard.as_deref(),
            col,
        );
        match fit {
            None | Some(geometry::StackFit::Fits) => {}
            Some(geometry::StackFit::NoCover) => {
                f.census[ti].no_cover += 1;
                return Ok(());
            }
            Some(geometry::StackFit::Capped { .. }) => f.census[ti].cover_capped += 1,
        }
        let centre_y = surf - centre_depth / sc.m_per_block_v;
        let top_y = seams.as_ref().map_or(centre_y, |p| {
            surf - (centre_depth - p.measures_m / 2.0) / sc.m_per_block_v
        });
        let half_blocks = half_m / sc.m_per_block_v * t.balance.size_mult_v;
        if top_y + half_blocks < f64::from(sc.min_y + 4) {
            f.census[ti].too_deep += 1;
            return Ok(());
        }
        let truncated = matches!(fate, Fate::Truncated { .. });
        if truncated {
            f.census[ti].truncated += 1;
        } else {
            f.census[ti].buried += 1;
        }
        if centre_depth - half_m <= t.preservation.assessment_depth_m {
            f.census[ti].counted += 1;
        }
        let dip = if t.geometry.dip_deg[1] > t.geometry.dip_deg[0] {
            (t.geometry.dip_deg[0]
                + rng.uniform() * (t.geometry.dip_deg[1] - t.geometry.dip_deg[0]))
                .to_radians()
        } else {
            t.geometry.dip_deg[0].to_radians()
        };
        let yaw_draw = rng.uniform() * std::f64::consts::PI;
        let placement = Placement {
            x,
            z,
            // A truncated body buried again is placed under the surface that cut it.
            surface_y: surf - fate.cut_depth_m() / sc.m_per_block_v,
            centre_depth_m: centre_depth - fate.cut_depth_m(),
            // An adjoining deposit's footprint was placed with its yaw.
            yaw: cand.point.ellipse.map_or(yaw_draw, |e| e[2]) as f32,
            dip: dip as f32,
            truncated,
            water_table_m: children::present_water_table_m(col),
        };
        let mut left = fate.fraction_left();
        let mut grades = Grades::new();
        for p in &t.products {
            match &p.grade_key {
                Some(k) => {
                    if let Some(g) = cand.size.grades.get(k) {
                        grades.insert(p.material.clone(), mass_fraction(k, *g));
                    }
                }
                None => {
                    grades.insert(p.material.clone(), 1.0);
                }
            }
        }
        // A stack is built before the site takes its id, so a deposit whose seams are all gone
        // leaves no gap in the ids (the seams take their ids below).
        let mut stacked = None;
        if let (Some(k), Some(plan)) = (&t.geometry.stack, &seams) {
            let mut n = 0;
            let st = geometry::stack_bodies(
                t,
                k,
                0,
                tonnage_t,
                &grades,
                &placement,
                plan,
                &sc,
                &mut || {
                    n += 1;
                    n
                },
            );
            if st.bodies.is_empty() {
                // Every seam is above the ground or below the world: the deposit is gone.
                let c = &mut f.census[ti];
                if truncated {
                    c.truncated -= 1;
                } else {
                    c.buried -= 1;
                }
                if centre_depth - half_m <= t.preservation.assessment_depth_m {
                    c.counted -= 1;
                }
                c.eroded += 1;
                return Ok(());
            }
            left = st.kept;
            stacked = Some(st.bodies);
        }
        if let Some(p) = &seams {
            let c = &mut f.census[ti];
            c.seams += stacked.as_ref().map_or(0, Vec::len);
            c.seams_max = c.seams_max.max(stacked.as_ref().map_or(0, Vec::len));
            c.measures_m += p.measures_m;
        }
        let site_id = world.next_site_id();
        let bodies = match stacked {
            Some(mut bodies) => {
                for b in &mut bodies {
                    b.site = Some(site_id);
                    b.id = world.next_body_id();
                }
                bodies
            }
            None => {
                geometry::bodies_for(t, site_id, tonnage_t, &grades, &placement, &sc, &mut || {
                    world.next_body_id()
                })
            }
        };
        let mut remaining_t = tonnage_t * left;
        // The forming event.
        let kind = if cand.charge.is_some() {
            "deeptime:trap_charge"
        } else {
            "deeptime:deposit_formation"
        };
        let mut e = base_event(world, kind, t_form)
            .at(Place::Site { site: site_id })
            .param("site", i64::from(site_id))
            .param("fate", fate.name())
            .param("fraction_left", left);
        if let Some((k, _, km)) = cand.charge {
            e = e
                .param("kitchen", k as i64)
                .param("migration_km", km)
                .param(
                    "trap_formed_years",
                    f64::from(self.state.pet[cell as usize].t_trap),
                );
        }
        if let Some(plan) = &seams {
            e = e
                .param("seams", plan.seams() as i64)
                .param("coal_measures_m", plan.measures_m);
        }
        if let Some(coal) = &t.coal
            && let Some(p) = self.state.pet.get(cell as usize)
        {
            let tti = f64::from(p.tti);
            let rank = coal
                .rank_tti
                .iter()
                .find(|(_, bound)| tti < *bound)
                .or(coal.rank_tti.last())
                .map_or("unknown", |(name, _)| name.as_str());
            e = e.param("rank", rank).param("tti", tti);
        }
        let forming = f.emit(world, e, x, z)?;
        let mut exhumation = None;
        let mut t_exhumed = t_form;
        if truncated {
            // When erosion first reached the body's top.
            let t_exp = time_eroded_to(col, t_form, t.preservation.formation_depth_m - half_m)
                .unwrap_or(t_form);
            let mut ex = world
                .event(t_exp, "deeptime:deposit_exhumation", SOURCE)
                .at(Place::Site { site: site_id })
                .param("deposit_type", t.id.as_str())
                .param("site", i64::from(site_id))
                .param("eroded_m", eroded_m)
                .param("fraction_left", left)
                .param("destroyed", 0i64)
                .caused_by(forming);
            let (xe, ze) = self.xz_at(f, cell, (x, z), t_exp);
            for id in f.preservation_causes(t, xe, ze, t_exp) {
                ex = ex.caused_by(id);
            }
            exhumation = Some(f.emit(world, ex, x, z)?);
            t_exhumed = t_exp;
        }
        // Supergene child (the parent's copper is conserved: the leached part becomes cap).
        let mut child_site = None;
        for child in t
            .geometry
            .children
            .iter()
            .filter(|c| c.relation == Relation::AboveWatertable)
        {
            let Some(ct) = self.types.iter().position(|x| x.id == child.type_id) else {
                continue;
            };
            if !self.state.enabled[ct] {
                continue;
            }
            let rules = self.types[ct].weathering.as_ref().expect("validated");
            let top_depth = (centre_depth - half_m).max(0.0);
            let Some((t0, stable)) = children::supergene_window(rules, top_depth, col) else {
                continue;
            };
            let Some(cu) = grades.get("c:copper").copied() else {
                continue;
            };
            let k =
                rules.grade_mult[0] + rng.uniform() * (rules.grade_mult[1] - rules.grade_mult[0]);
            let metric = t.shape().metric();
            let wt = col
                .last()
                .map_or(rules.min_water_table_m, |s| s.water_table_m);
            // Leaching reaches down to the water table (at most the rules' leached column); the
            // protore below it stays hypogene.
            let leach_to = rules.leach_depth_m.min(wt.max(top_depth));
            let u_s = (centre_depth / half_m).clamp(-1.0, 1.0);
            let u_l = ((centre_depth - leach_to) / half_m).clamp(-1.0, 1.0);
            let leached_frac =
                ledger::fraction_below(metric, u_s) - ledger::fraction_below(metric, u_l);
            let (bt, bg, leached) = children::blanket(tonnage_t, cu, leached_frac, k);
            if bt <= 0.0 {
                continue;
            }
            remaining_t = (remaining_t - leached).max(0.0);
            child_site = Some((ct, bt, bg, k, stable, t0, wt));
        }
        // Register the parent site.
        let products = self.products(t, &cand.size, remaining_t);
        let aabb = union_aabb(&bodies);
        let site = Site {
            id: site_id,
            kind: t.site_kind.site_kind(),
            deposit_type: t.id.clone(),
            center: [
                x.round() as i32,
                centre_y
                    .round()
                    .clamp(f64::from(sc.min_y), f64::from(sc.max_y)) as i32,
                z.round() as i32,
            ],
            lat_lon: ll,
            aabb,
            body: bodies.first().map(|b| b.id),
            products,
            forming_event: forming,
            parent: None,
            extraction: self.extraction(t, remaining_t),
        };
        world.add_site(site)?;
        for b in bodies {
            world.add_body(b)?;
        }
        f.census[ti].sites += 1;
        if let Some((ct, bt, bg, k, stable, t0, wt)) = child_site {
            let ctype = &self.types[ct];
            let child_id = world.next_site_id();
            // The blanket sits at the water table, below the parent's (eroded) top and within
            // the leached column, and never pokes out of the ground.
            let rules = ctype.weathering.as_ref().expect("validated");
            let top_depth = (centre_depth - half_m).max(0.0);
            let child_half = geometry::half_height_m(ctype, bt) * ctype.balance.size_mult_v;
            let depth = wt
                .clamp(top_depth, top_depth + rules.leach_depth_m)
                .max(child_half);
            let place = Placement {
                x,
                z,
                surface_y: surf,
                centre_depth_m: depth,
                yaw: placement.yaw,
                dip: 0.0,
                truncated: false,
                water_table_m: 0.0,
            };
            let cgrades: Grades = [("c:copper".to_string(), bg)].into_iter().collect();
            let cbodies =
                geometry::bodies_for(ctype, child_id, bt, &cgrades, &place, &sc, &mut || {
                    world.next_body_id()
                });
            let mut ce = world
                .event(t0, "deeptime:supergene_enrichment", SOURCE)
                .until(self.state.frame_times.last().map_or(t0, |(t, _)| t.max(t0)))
                .at(Place::Site { site: child_id })
                .param("deposit_type", ctype.id.as_str())
                .param("site", i64::from(child_id))
                .param("parent", i64::from(site_id))
                .param("grade_mult", k)
                .param("stable_myr", stable)
                .param("water_table_m", wt)
                .param("leached_t", bt * k)
                .magnitude((k / 3.0) as f32)
                .caused_by(forming);
            if let Some(ex) = exhumation {
                ce = ce.caused_by(ex);
            }
            let (xs, zs) = self.xz_at(f, cell, (x, z), t0);
            for id in f.preservation_causes(ctype, xs, zs, t0) {
                ce = ce.caused_by(id);
            }
            let cev = f.emit(world, ce, x, z)?;
            let cu_role = ctype
                .products
                .first()
                .map_or("primary".to_string(), |p| p.role.clone());
            let site = Site {
                id: child_id,
                kind: ctype.site_kind.site_kind(),
                deposit_type: ctype.id.clone(),
                center: [
                    x.round() as i32,
                    (surf - depth / sc.m_per_block_v).round() as i32,
                    z.round() as i32,
                ],
                lat_lon: ll,
                aabb: union_aabb(&cbodies),
                body: cbodies.first().map(|b| b.id),
                products: vec![Product {
                    material: "c:copper".into(),
                    grade: bg as f32,
                    tonnage_t: bt,
                    role: cu_role,
                }],
                forming_event: cev,
                parent: Some(site_id),
                extraction: self.extraction(ctype, bt),
            };
            world.add_site(site)?;
            for b in cbodies {
                world.add_body(b)?;
            }
            f.census[ct].sites += 1;
            f.census[ti].children += 1;
        }
        // A porphyry cut by the surface sheds placer gold in proportion to the share of it that
        // erosion has taken (Ben, 2026-10-05, "Reword check + partial shedding").
        if let (true, Some(ex), ErosionBehavior::SpawnPlacer) =
            (truncated, exhumation, t.preservation.erosion_behavior)
        {
            let eroded_fraction = 1.0 - fate.fraction_left();
            for child in t
                .geometry
                .children
                .iter()
                .filter(|c| c.relation == Relation::Downstream)
            {
                self.spawn_placer(
                    f,
                    world,
                    cand,
                    &child.type_id,
                    ex,
                    t_exhumed,
                    receivers,
                    Shed {
                        eroded_fraction,
                        parent_site: Some(site_id),
                    },
                    &mut rng,
                )?;
            }
        }
        Ok(())
    }

    /// A placer below an eroded parent.
    #[allow(clippy::too_many_arguments)]
    fn spawn_placer(
        &self,
        f: &mut Finish,
        world: &mut WorldState,
        cand: &Candidate,
        child_id: &str,
        gone_event: EventId,
        t_gone: f64,
        receivers: Option<&[u32]>,
        shed: Shed,
        rng: &mut Rng,
    ) -> Result<(), CoreError> {
        let parent = &self.types[cand.ty];
        let Some(ct) = self.types.iter().position(|t| t.id == child_id) else {
            return Ok(());
        };
        if !self.state.enabled[ct] {
            return Ok(());
        }
        let child = &self.types[ct];
        let rules = child.placer.as_ref().expect("validated");
        // The resistate metal the parent held (the part in what eroded is taken in
        // `shed_tonnage_mt`).
        let mut lost_t = 0.0;
        for p in &parent.products {
            if !parent.preservation.resistates.contains(&p.material) {
                continue;
            }
            if let Some(k) = &p.grade_key
                && let Some(g) = cand.size.grades.get(k)
            {
                lost_t += cand.size.tonnage_mt * 1.0e6 * mass_fraction(k, *g);
            }
        }
        if lost_t <= 0.0 {
            return Ok(());
        }
        let sc = f.sc;
        let (cell, km) = children::placer_cell(
            &f.grid,
            receivers,
            &f.elev,
            cand.point.cell,
            rules,
            sc.m_per_block_h,
        );
        let (cx, cz) = cluster::cell_xz(&f.grid, cell);
        let r = f64::from(f.grid.r);
        let (x, z) = (
            cluster::wrap_x(f.c, cx + (rng.uniform() - 0.5) * r),
            cz + (rng.uniform() - 0.5) * r,
        );
        let model = SizeModel::new(&child.sizing);
        let mut size = model.draw(rng);
        // The gravel scales with the share of the parent that eroded, then the placer's gold is
        // capped at the captured share of what the parent lost (so the whole is proportional).
        let key = child
            .products
            .iter()
            .find_map(|p| p.grade_key.clone())
            .unwrap_or_default();
        let grade = size
            .grades
            .get(&key)
            .map_or(0.0, |g| mass_fraction(&key, *g));
        size.tonnage_mt = shed_tonnage_mt(
            size.tonnage_mt,
            grade,
            lost_t,
            rules.capture_fraction,
            shed.eroded_fraction,
        );
        // A trickle below the smallest recorded placer is not a deposit (a parent the surface
        // only grazes; a fully eroded one draws from the whole curve as before).
        if shed.eroded_fraction < 1.0 && size.tonnage_mt < child.sizing.tonnage_mt.p90 {
            return Ok(());
        }
        let tonnage_t = size.tonnage_mt * 1.0e6;
        let radius = geometry::radius_blocks(child, tonnage_t, &sc);
        let min_sep = (child.sizing.min_separation_m / sc.m_per_block_h).max(0.0);
        if f.placed[ct]
            .iter()
            .any(|q| cluster::distance(f.c, (x, z), (q.x, q.z)) < min_sep.max(radius + q.radius))
        {
            f.census[ct].dropped_separation += 1;
            return Ok(());
        }
        f.placed[ct].push(Point {
            x,
            z,
            cell,
            radius,
            ellipse: None,
        });
        let surf = f.surface_at(x, z);
        let half_m = geometry::half_height_m(child, tonnage_t);
        // Follow the drainage direction.
        let next = receivers.map_or(cell, |rec| rec[cell as usize]);
        let (nx, nz) = cluster::cell_xz(&f.grid, next);
        let yaw = if next == cell {
            rng.uniform() * std::f64::consts::PI
        } else {
            cluster::wrap_x(f.c, nx - cx)
                .atan2(nz - cz)
                .mul_add(-1.0, std::f64::consts::FRAC_PI_2)
        };
        let placement = Placement {
            x,
            z,
            surface_y: surf,
            centre_depth_m: half_m * child.balance.size_mult_v,
            yaw: yaw as f32,
            dip: 0.0,
            truncated: false,
            water_table_m: 0.0,
        };
        let mut grades = Grades::new();
        for p in &child.products {
            if let Some(k) = &p.grade_key
                && let Some(g) = size.grades.get(k)
            {
                grades.insert(p.material.clone(), mass_fraction(k, *g));
            }
        }
        let site_id = world.next_site_id();
        let bodies = geometry::bodies_for(
            child,
            site_id,
            tonnage_t,
            &grades,
            &placement,
            &sc,
            &mut || world.next_body_id(),
        );
        let ll = lat_lon_of(f.c, x, z);
        let mut e = world
            .event(t_gone, "deeptime:placer_formation", SOURCE)
            .at(Place::Site { site: site_id })
            .param("deposit_type", child.id.as_str())
            .param("site", i64::from(site_id))
            .param("parent_type", parent.id.as_str())
            .param("transport_km", km)
            .param("resistate_lost_t", lost_t)
            .param("tonnage_mt", size.tonnage_mt)
            .magnitude(size.tonnage_q as f32)
            .caused_by(gone_event);
        let (xp, zp) = self.xz_at(f, cand.point.cell, (cand.point.x, cand.point.z), t_gone);
        for id in f.preservation_causes(child, xp, zp, t_gone) {
            e = e.caused_by(id);
        }
        let ev = f.emit(world, e, x, z)?;
        let site = Site {
            id: site_id,
            kind: child.site_kind.site_kind(),
            deposit_type: child.id.clone(),
            center: [
                x.round() as i32,
                (surf - half_m * child.balance.size_mult_v / sc.m_per_block_v).round() as i32,
                z.round() as i32,
            ],
            lat_lon: ll,
            aabb: union_aabb(&bodies),
            body: bodies.first().map(|b| b.id),
            products: self.products(child, &size, tonnage_t),
            forming_event: ev,
            parent: shed.parent_site,
            extraction: self.extraction(child, tonnage_t),
        };
        world.add_site(site)?;
        for b in bodies {
            world.add_body(b)?;
        }
        f.census[ct].sites += 1;
        f.census[cand.ty].children += 1;
        Ok(())
    }

    /// The site's products; none when no ore is left (a porphyry whose preserved roots were all
    /// leached into its supergene blanket keeps its cap and gossan bodies but no hypogene ore).
    fn products(&self, t: &DepositType, size: &SizeDraw, tonnage_t: f64) -> Vec<Product> {
        if tonnage_t < 1.0 {
            return Vec::new();
        }
        t.products
            .iter()
            .filter_map(|p| {
                let grade = match &p.grade_key {
                    Some(k) => mass_fraction(k, *size.grades.get(k)?) * t.balance.richness_mult,
                    None => 1.0,
                };
                Some(Product {
                    material: p.material.clone(),
                    grade: grade.min(1.0) as f32,
                    tonnage_t,
                    role: p.role.clone(),
                })
            })
            .collect()
    }

    fn extraction(&self, t: &DepositType, tonnage_t: f64) -> Extraction {
        let some = tonnage_t >= 1.0;
        Extraction {
            finite: true,
            amount_base: tonnage_t.max(0.0),
            coe_vein: t.minecraft.coe_vein.clone().filter(|_| some),
            cdg_charge_mb: t
                .minecraft
                .cdg_reservoir
                .then(|| (tonnage_t.max(0.0) / OIL_T_M3 * 1000.0).round() as u64)
                .filter(|_| some),
        }
    }

    /// The component that most often limits the barren cells around `cell` (radius 3 cells).
    fn neighbours_limited_by(&self, ty: usize, cell: u32, grid: &GridDesc) -> String {
        let acc = &self.state.acc[ty];
        if acc.is_empty() {
            return "none".into();
        }
        let (w, h) = (i64::from(grid.w), i64::from(grid.h));
        let (i0, j0) = (i64::from(cell % grid.w), i64::from(cell / grid.w));
        let mut counts = [0u32; 6];
        for dj in -3..=3 {
            for di in -3..=3 {
                let j = j0 + dj;
                if j < 0 || j >= h {
                    continue;
                }
                let a = &acc[(j * w + (i0 + di).rem_euclid(w)) as usize];
                if a.f_max <= 0.0 {
                    counts[a.limiting()] += 1;
                }
            }
        }
        match counts.iter().enumerate().max_by_key(|(_, n)| **n) {
            Some((k, n)) if *n > 0 => COMPONENTS[k].to_string(),
            _ => "none".into(),
        }
    }

    /// The playability pass (plan §7.8).
    fn guarantee(
        &self,
        f: &mut Finish,
        world: &mut WorldState,
        hist: &dyn FieldHistory,
        ctx: &StageContext<'_>,
    ) -> Result<(), CoreError> {
        let sc = f.sc;
        for ess in &self.params.essentials {
            let regions = Regions::new(world.circumference, ess.region_blocks);
            let mut have = vec![0u32; regions.count()];
            for s in &world.sites {
                if !ess.counts(&s.products) {
                    continue;
                }
                if ess.exposed {
                    let surf = f.surface_at(f64::from(s.center[0]), f64::from(s.center[2]));
                    if f64::from(s.aabb[1][1]) < surf - EXPOSED_BLOCKS {
                        continue;
                    }
                }
                have[regions.index(f64::from(s.center[0]), f64::from(s.center[2]))] += 1;
            }
            // Candidate types: primary producers of the material with favourability maps.
            let producers: Vec<usize> = self
                .types
                .iter()
                .enumerate()
                .filter(|(k, t)| {
                    self.state.enabled[*k]
                        && !t.child_only
                        && !self.state.acc[*k].is_empty()
                        && t.products
                            .iter()
                            .any(|p| p.material == ess.material && p.role == "primary")
                })
                .map(|(k, _)| k)
                .collect();
            if producers.is_empty() {
                continue;
            }
            for (region, &count) in have.iter().enumerate() {
                let mut short = ess.min_sites_per_region.saturating_sub(count);
                while short > 0 {
                    // The best (type, land cell) in the region.
                    let mut best: Option<(f64, usize, u32)> = None;
                    for &ti in &producers {
                        let t = &self.types[ti];
                        let min_sep = t.sizing.min_separation_m / sc.m_per_block_h;
                        for (cell, a) in self.state.acc[ti].iter().enumerate() {
                            if f.elev[cell] < 0.0 {
                                continue;
                            }
                            let (x, z) = cluster::cell_xz(&f.grid, cell as u32);
                            if regions.index(x, z) != region {
                                continue;
                            }
                            let score = a.favourability() + 1.0e-3 * f64::from(a.f_max);
                            if best.is_some_and(|b| b.0 >= score) {
                                continue;
                            }
                            let r = geometry::radius_blocks(
                                t,
                                SizeModel::new(&t.sizing).smallest_plausible().tonnage_mt * 1.0e6,
                                &sc,
                            );
                            if f.placed[ti].iter().any(|q| {
                                cluster::distance(f.c, (x, z), (q.x, q.z))
                                    < min_sep.max(r + q.radius)
                            }) {
                                continue;
                            }
                            best = Some((score, ti, cell as u32));
                        }
                    }
                    let Some((score, ti, cell)) = best else { break };
                    let path = ctx.pool.install(|| trace(hist, self.frames, &[cell]))?;
                    f.paths
                        .insert(cell, path.into_iter().next().unwrap_or_default());
                    self.place_guaranteed(f, world, ti, cell, score, ess, region, &regions)?;
                    short -= 1;
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn place_guaranteed(
        &self,
        f: &mut Finish,
        world: &mut WorldState,
        ti: usize,
        cell: u32,
        score: f64,
        ess: &Essential,
        region: usize,
        regions: &Regions,
    ) -> Result<(), CoreError> {
        let t = &self.types[ti];
        let sc = f.sc;
        let size = SizeModel::new(&t.sizing).smallest_plausible();
        let tonnage_t = size.tonnage_mt * 1.0e6;
        let (x, z) = cluster::cell_xz(&f.grid, cell);
        let radius = geometry::radius_blocks(t, tonnage_t, &sc);
        f.placed[ti].push(Point {
            x,
            z,
            cell,
            radius,
            ellipse: None,
        });
        let half_m = geometry::half_height_m(t, tonnage_t);
        let surf = f.surface_at(x, z);
        let depth = if ess.exposed {
            // Top at the surface: the body's vertical half-height in blocks is scaled by
            // size_mult_v about its centre.
            half_m * t.balance.size_mult_v
        } else {
            t.preservation
                .formation_depth_m
                .min((surf - f64::from(sc.min_y + 16)) * sc.m_per_block_v)
        };
        let acc = self.state.acc[ti][cell as usize];
        let t_ev = if acc.k_best != u16::MAX {
            self.state.frame_times[acc.k_best as usize].0
        } else {
            self.state
                .frame_times
                .last()
                .map_or(world.t_years, |(t, _)| *t)
        };
        let slack = self.state.frame_times.first().map_or(1.0e7, |(_, dt)| *dt);
        let (xf, zf) = self.xz_at(f, cell, (x, z), t_ev);
        let causes = f.cause_ids(t, xf, zf, t_ev, slack);
        let mut grades = Grades::new();
        for p in &t.products {
            match &p.grade_key {
                Some(k) => {
                    if let Some(g) = size.grades.get(k) {
                        grades.insert(p.material.clone(), mass_fraction(k, *g));
                    }
                }
                None => {
                    grades.insert(p.material.clone(), 1.0);
                }
            }
        }
        let site_id = world.next_site_id();
        let placement = Placement {
            x,
            z,
            surface_y: surf,
            centre_depth_m: depth,
            yaw: 0.0,
            dip: 0.0,
            truncated: false,
            water_table_m: 0.0,
        };
        let stacked = if let Some(k) = &t.geometry.stack {
            // No column history here: a typical stack, six seams' worth of measures, its top
            // seam at the surface when exposed.
            let stream = format!("stack/guaranteed/{}/{cell}", t.id);
            let mut rng = Rng::stream(self.seed, &stream);
            let mut plan = geometry::seam_plan(k, 6.0 * k.seam_spacing_m, 2.0 * half_m, &mut rng);
            let mut centre_depth_m = if ess.exposed {
                plan.measures_m / 2.0
            } else {
                depth
            };
            // Held in the site's sedimentary cover like any stack; a guaranteed site keeps one
            // seam at the surface where there is none.
            if let Some(cover) = &f.cover {
                let cover_blocks = cover.blocks_at(x, z, surf);
                let refit = match geometry::fit_stack(k, &plan, centre_depth_m, cover_blocks, &sc) {
                    geometry::StackFit::Fits => None,
                    geometry::StackFit::NoCover => Some((k.seam_spacing_m, k.seam_spacing_m / 2.0)),
                    geometry::StackFit::Capped {
                        measures_m,
                        centre_depth_m,
                    } => Some((measures_m, centre_depth_m)),
                };
                if let Some((measures_m, d)) = refit {
                    let mut rng = Rng::stream(self.seed, &stream);
                    if measures_m < plan.measures_m {
                        plan = geometry::seam_plan(k, measures_m, 2.0 * half_m, &mut rng);
                    }
                    centre_depth_m = d;
                }
            }
            let placement = Placement {
                centre_depth_m,
                ..placement
            };
            geometry::stack_bodies(
                t,
                k,
                site_id,
                tonnage_t,
                &grades,
                &placement,
                &plan,
                &sc,
                &mut || world.next_body_id(),
            )
            .bodies
        } else {
            Vec::new()
        };
        let bodies = if stacked.is_empty() {
            geometry::bodies_for(t, site_id, tonnage_t, &grades, &placement, &sc, &mut || {
                world.next_body_id()
            })
        } else {
            stacked
        };
        let (col, row) = (region as u32 % regions.cols, region as u32 / regions.cols);
        let mut e = world
            .event(t_ev, "deeptime:guaranteed_occurrence", SOURCE)
            .at(Place::Site { site: site_id })
            .param("deposit_type", t.id.as_str())
            .param("site", i64::from(site_id))
            .param("material", ess.material.as_str())
            .param("region", format!("{col},{row}"))
            .param("favourability", acc.favourability())
            .param("limiting", COMPONENTS[acc.limiting()])
            .param("score", score)
            .param("reason", "playability minimum (plan §7.8)")
            .magnitude(0.0);
        for id in causes {
            e = e.caused_by(id);
        }
        let ev = f.emit(world, e, x, z)?;
        let site = Site {
            id: site_id,
            kind: t.site_kind.site_kind(),
            deposit_type: t.id.clone(),
            center: [
                x.round() as i32,
                (surf - depth / sc.m_per_block_v).round() as i32,
                z.round() as i32,
            ],
            lat_lon: lat_lon_of(f.c, x, z),
            aabb: union_aabb(&bodies),
            body: bodies.first().map(|b| b.id),
            products: self.products(t, &size, tonnage_t),
            forming_event: ev,
            parent: None,
            extraction: self.extraction(t, tonnage_t),
        };
        world.add_site(site)?;
        for b in bodies {
            world.add_body(b)?;
        }
        f.census[ti].guaranteed += 1;
        f.census[ti].sites += 1;
        Ok(())
    }
}

fn union_aabb(bodies: &[Body]) -> [[i32; 3]; 2] {
    let mut lo = [i32::MAX; 3];
    let mut hi = [i32::MIN; 3];
    for b in bodies {
        for k in 0..3 {
            lo[k] = lo[k].min(b.aabb[0][k]);
            hi[k] = hi[k].max(b.aabb[1][k]);
        }
    }
    if bodies.is_empty() {
        return [[0; 3]; 2];
    }
    [lo, hi]
}

/// The sites of a type in a world (tests and the CLI).
pub fn sites_of<'a>(
    world: &'a WorldState,
    type_id: &'a str,
) -> impl Iterator<Item = &'a Site> + 'a {
    world
        .sites
        .iter()
        .filter(move |s| s.deposit_type == type_id)
}

/// The tonnage (Mt of gravel) of a placer shed from a parent that erosion has taken `eroded_fraction`
/// of (by volume; 1 for one eroded away): the drawn tonnage times that fraction, held to the
/// captured share of the resistate metal in the eroded part (the gold spread through the body, so
/// `parent_metal_t` times the fraction). Both the draw and the cap scale with the fraction, so a
/// half-eroded parent sheds half the placer of a wholly eroded one, capped or not.
/// `grade` is the placer's gold as a mass fraction. Game choice, ledger DEP-207.
fn shed_tonnage_mt(
    drawn_mt: f64,
    grade: f64,
    parent_metal_t: f64,
    capture_fraction: f64,
    eroded_fraction: f64,
) -> f64 {
    let f = eroded_fraction.clamp(0.0, 1.0);
    let mut mt = drawn_mt * f;
    let held = mt * 1.0e6 * grade;
    let cap = capture_fraction * parent_metal_t * f;
    if held > cap && held > 0.0 {
        mt *= cap / held;
    }
    mt
}

/// How much of a parent a placer is shed from.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Shed {
    /// The share of the parent (by volume) erosion has taken: 1 for a parent eroded away.
    eroded_fraction: f64,
    /// The parent's site when it survives (a cut porphyry), the placer's `parent` link.
    parent_site: Option<SiteId>,
}

#[cfg(test)]
mod recorded_tests;

#[cfg(test)]
mod spill_tests;

#[cfg(test)]
mod tests {
    use super::types::DepositType;

    /// Partial shedding (Ben, 2026-10-05): a placer's size follows the share of its porphyry that
    /// erosion took. Half eroded gives half the tonnage of fully eroded, whether or not the gold
    /// cap binds; an uneroded one gives none.
    #[test]
    fn a_placer_is_shed_in_proportion_to_the_eroded_fraction() {
        let types = super::types::builtin().unwrap();
        let porphyry = types
            .iter()
            .find(|t| t.id == "deeptime:porphyry_cu_mo_au")
            .unwrap();
        assert_eq!(
            porphyry.preservation.erosion_behavior,
            ErosionBehavior::SpawnPlacer
        );
        let rules = types
            .iter()
            .find(|t| t.id == "deeptime:placer_au")
            .unwrap()
            .placer
            .clone()
            .unwrap();
        let cut = |left: f64, drawn_mt: f64, metal_t: f64| {
            // The eroded share the way `materialise` takes it from the ledger's fate.
            let fate = Fate::Truncated {
                centre_depth_m: 0.0,
                fraction_left: left,
                cut_depth_m: 0.0,
            };
            shed_tonnage_mt(
                drawn_mt,
                0.2e-6,
                metal_t,
                rules.capture_fraction,
                1.0 - fate.fraction_left(),
            )
        };
        // Uncapped: 1.1 Mt of gravel at 0.2 g/t holds 220 kg; the parent held 30 t of gold.
        let full = cut(0.0, 1.1, 30.0);
        let half = cut(0.5, 1.1, 30.0);
        assert!((full - 1.1).abs() < 1e-12);
        assert!((half / full - 0.5).abs() < 1e-9, "{half} of {full}");
        // Capped: a parent that held only 0.2 t of gold caps the placer at 0.1 t.
        let full = cut(0.0, 1.1, 0.2);
        let half = cut(0.5, 1.1, 0.2);
        assert!(
            full < 1.1 && (half / full - 0.5).abs() < 1e-9,
            "{half} {full}"
        );
        let held_t = full * 1.0e6 * 0.2e-6;
        assert!(
            (held_t - rules.capture_fraction * 0.2).abs() < 1e-9,
            "{held_t}"
        );
        // A tenth eroded sheds a tenth.
        assert!((cut(0.9, 1.1, 30.0) / 1.1 - 0.1).abs() < 1e-9);
        // Uneroded: nothing.
        assert_eq!(cut(1.0, 1.1, 30.0), 0.0);
        assert_eq!(
            Fate::Buried {
                centre_depth_m: 900.0
            }
            .fraction_left(),
            1.0,
            "a buried body is not cut, so it sheds nothing"
        );
    }

    /// The surface's block y follows the world's sea level and vertical scale (2026-10-02: it read
    /// sea level 63 whatever the world said).
    #[test]
    fn surface_y_reads_the_world_sea_level_and_scale() {
        let sc = Scales {
            sea_level_y: 100,
            m_per_block_v: 35.0,
            ..Scales::default()
        };
        assert_eq!(surface_y(700.0, &sc), 120.0);
        assert_eq!(surface_y(-350.0, &sc), 90.0);
        let sc20 = Scales {
            m_per_block_v: 20.0,
            ..Scales::default()
        };
        assert_eq!(surface_y(700.0, &sc20), 98.0);
    }

    fn base_types() -> Vec<DepositType> {
        select_types(&DepositParams::default(), 1.0).unwrap()
    }

    /// The editor's Multipliers (the settings audit) fold into the types before anything is
    /// drawn: `size_mult` the tonnage percentiles, `richness_mult` the grade multiplier,
    /// `feature` the body size in blocks; all at 1 leave every type as it was.
    #[test]
    fn the_global_multipliers_fold_into_the_types() {
        let base = base_types();
        assert_eq!(
            select_types(
                &DepositParams {
                    size_mult: 1.0,
                    richness_mult: 1.0,
                    ..DepositParams::default()
                },
                1.0
            )
            .unwrap(),
            base
        );
        let p = DepositParams {
            size_mult: 2.5,
            richness_mult: 1.5,
            ..DepositParams::default()
        };
        let tuned = select_types(&p, 2.0).unwrap();
        assert_eq!(tuned.len(), base.len());
        for (b, t) in base.iter().zip(&tuned) {
            assert_eq!(
                t.sizing.tonnage_mt.p50,
                b.sizing.tonnage_mt.p50 * 2.5,
                "{}",
                b.id
            );
            assert_eq!(t.sizing.tonnage_mt.p10, b.sizing.tonnage_mt.p10 * 2.5);
            assert_eq!(t.sizing.tonnage_mt.p90, b.sizing.tonnage_mt.p90 * 2.5);
            assert_eq!(t.balance.richness_mult, b.balance.richness_mult * 1.5);
            assert_eq!(t.balance.size_mult_h, b.balance.size_mult_h * 2.0);
            assert_eq!(t.balance.size_mult_v, b.balance.size_mult_v * 2.0);
            // Counts, densities and clustering are not touched.
            assert_eq!(t.balance.count_mult, b.balance.count_mult);
            assert_eq!(t.sizing.density_per_1e5km2, b.sizing.density_per_1e5km2);
        }
        for bad in [0.0, -1.0, 101.0, f64::NAN] {
            for p in [
                DepositParams {
                    size_mult: bad,
                    ..DepositParams::default()
                },
                DepositParams {
                    richness_mult: bad,
                    ..DepositParams::default()
                },
            ] {
                assert!(p.validate("$.deposits").is_err(), "{bad}");
            }
        }
        // A bigger body from a bigger deposit: the volume follows the tonnage.
        let t0 = &base[0];
        let sc = Scales::default();
        let semi = |m: f64| {
            let vol = 240.0e6 * m / t0.sizing.ore_density_t_m3;
            geometry::to_blocks(
                geometry::semi_axes_m(t0.shape(), t0.geometry.aspect, vol),
                t0,
                &sc,
            )
        };
        let (a, b) = (semi(1.0), semi(8.0));
        assert!(
            (f64::from(b[0]) / f64::from(a[0]) - 2.0).abs() < 1e-3,
            "8x the ore, 2x the width"
        );
    }

    use std::sync::Arc;
    use std::time::Duration;

    use deeptime_format::ParamValue;

    use super::*;
    use crate::session::{AdvanceStatus, NoGauge, Session, SessionOptions};

    const PRESET_4K: &str = include_str!("../../tests/presets/deposits_synthetic_4k.json");

    fn run(preset: &str, name: &str) -> (WorldState, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("dt-deposits-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let s = Session::create(
            preset,
            "",
            SessionOptions {
                out_dir: dir.clone(),
                threads: 2,
                mem_cap: 0,
            },
            Arc::new(NoGauge),
        )
        .unwrap();
        loop {
            match s.advance(Duration::from_secs(60)).unwrap() {
                AdvanceStatus::Done => break,
                AdvanceStatus::Cancelled => panic!("cancelled"),
                _ => {}
            }
        }
        let ck = s.checkpoint().unwrap();
        s.finalize_write().unwrap();
        (ck.world, dir)
    }

    /// Acceptance: a synthetic run places sites of several families, every site has a forming
    /// event with causes, bodies belong to their sites, the minimum separation holds, and the
    /// world data carries bodies and the census.
    #[test]
    fn synthetic_run_places_sites_with_causes() {
        let (w, dir) = run(PRESET_4K, "run");
        eprintln!("census: {}", w.notes[STAGE_NAME]);
        assert!(!w.sites.is_empty());
        let types: BTreeSet<&str> = w.sites.iter().map(|s| s.deposit_type.as_str()).collect();
        assert!(types.len() >= 4, "types with sites: {types:?}");
        for s in &w.sites {
            let e = w
                .events
                .iter()
                .find(|e| e.id == s.forming_event)
                .expect("every site has its forming event in the log");
            assert!(
                !e.causes.is_empty(),
                "site {} ({}) has no causes",
                s.id,
                s.deposit_type
            );
            for c in &e.causes {
                assert!(
                    w.events.iter().any(|x| x.id == *c),
                    "cause {c} is in the log"
                );
            }
            assert_eq!(
                e.params.get("site"),
                Some(&ParamValue::Int(i64::from(s.id)))
            );
            assert_eq!(e.place, Place::Site { site: s.id });
            let first = s.body.expect("a site has a body");
            let body = w.bodies.iter().find(|b| b.id == first).expect("its body");
            assert_eq!(body.site, Some(s.id));
            assert!(
                s.products
                    .iter()
                    .all(|p| (0.0..=1.0).contains(&p.grade) && p.tonnage_t >= 0.0)
            );
        }
        for b in &w.bodies {
            b.validate().unwrap();
            assert!(w.sites.iter().any(|s| Some(s.id) == b.site));
        }
        // Minimum separation between sites of one type (centres rounded to blocks).
        let all = types::builtin().unwrap();
        let c = f64::from(w.circumference);
        for t in &all {
            let sites: Vec<&Site> = sites_of(&w, &t.id).collect();
            let min_sep = t.sizing.min_separation_m / 300.0;
            for (a, s) in sites.iter().enumerate() {
                for q in &sites[a + 1..] {
                    let d = cluster::distance(
                        c,
                        (f64::from(s.center[0]), f64::from(s.center[2])),
                        (f64::from(q.center[0]), f64::from(q.center[2])),
                    );
                    assert!(
                        d >= min_sep - 1.5,
                        "{}: sites {} and {} are {d} blocks apart",
                        t.id,
                        s.id,
                        q.id
                    );
                }
            }
        }
        let census: Value = serde_json::from_str(&w.notes[STAGE_NAME]).unwrap();
        assert_eq!(census["sites"], w.sites.len());
        assert!(dir.join("bodies/bodies.dtb").is_file());
        let back = deeptime_format::bodies::read_bodies(&dir).unwrap().unwrap();
        assert_eq!(back.len(), w.bodies.len());
        let manifest = deeptime_format::Manifest::read(&dir).unwrap();
        assert!(manifest.notes.contains_key(STAGE_NAME));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A run resumed from a mid-stage checkpoint ends in the same world.
    #[test]
    fn resumed_run_matches() {
        let (full, dir) = run(PRESET_4K, "full");
        std::fs::remove_dir_all(&dir).unwrap();
        let dir = std::env::temp_dir().join(format!("dt-deposits-resume-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let opts = || SessionOptions {
            out_dir: dir.clone(),
            threads: 2,
            mem_cap: 0,
        };
        let s = Session::create(PRESET_4K, "", opts(), Arc::new(NoGauge)).unwrap();
        // Set-up and a few frames, then a checkpoint mid-stage.
        while s.progress().stage_permille < 300 {
            s.advance(Duration::from_millis(1)).unwrap();
        }
        let ck = s.checkpoint().unwrap();
        drop(s);
        let s = Session::resume(ck, opts(), Arc::new(NoGauge)).unwrap();
        while s.advance(Duration::from_secs(60)).unwrap() != AdvanceStatus::Done {}
        let resumed = s.checkpoint().unwrap().world;
        assert_eq!(resumed.sites, full.sites);
        assert_eq!(resumed.bodies, full.bodies);
        assert_eq!(resumed.events.len(), full.events.len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// P1-15: a `terrain` preset bakes a playable world whose ground is the surface the bodies
    /// were placed against, with the standard spawn candidates (dry, flat land nearest (0, 0)); the
    /// content bundle's essentials replace the preset's.
    #[test]
    fn world_preset_bakes_terrain_under_the_bodies() {
        use deeptime_format::{Manifest, RasterRead, TilePackReader};
        const WORLD_4K: &str = include_str!("../../tests/presets/deposits_world_4k.json");
        let (w, dir) = run(WORLD_4K, "world");
        let m = Manifest::read(&dir).unwrap();
        m.verify(&dir).unwrap();
        assert_eq!(m.notes["synthetic"]["kind"], "deposit-history");
        let names: Vec<&str> = m.layers.iter().map(|l| l.name.as_str()).collect();
        for l in [
            "elevation",
            "water_level",
            "climate/temperature",
            "strata/horizons",
        ] {
            assert!(names.contains(&l), "{l} missing from {names:?}");
        }
        for f in ["sites/sites.dtsr", "bodies/bodies.dtb", "events/index.json"] {
            assert!(dir.join(f).is_file(), "{f}");
        }
        let elev = TilePackReader::open(&dir.join("rasters/elevation.dtr"))
            .unwrap()
            .load()
            .unwrap();
        // A truncated body is clipped at the ground it sits in: its clip is the history's surface
        // at the body's centre, and the baked terrain (L3, quantised to 1/16) follows it closely.
        let mut clipped = 0;
        for b in w.bodies.iter().filter(|b| b.role == "ore") {
            if b.clip_top_y == deeptime_format::bodies::NO_CLIP {
                continue;
            }
            clipped += 1;
            let ground = elev.bilinear(b.center[0], b.center[2]);
            assert!(
                (f64::from(b.clip_top_y) - ground).abs() <= 3.0,
                "body {} clip {} vs ground {ground:.2}",
                b.id,
                b.clip_top_y
            );
        }
        assert!(clipped > 0, "no truncated ore body in the 4k world");
        let spawn: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("spawn/candidates.json")).unwrap(),
        )
        .unwrap();
        // Spawn follows the standard rules (Ben, 2026-10-05): no ranking by the essentials.
        assert!(spawn.get("ranked_by").is_none(), "{spawn}");
        let first = &spawn["candidates"][0];
        assert!(first.get("unmet").is_none(), "{first}");
        assert!(first["surface_y"].as_f64().unwrap() >= 65.0, "{first}");
        let _ = std::fs::remove_dir_all(&dir);

        // The content bundle's essentials win over the preset's.
        let p = crate::session::Preset::from_json(WORLD_4K)
            .unwrap()
            .with_content(r#"{"essentials": [{"material": "c:gold", "min_sites_per_region": 1, "region_blocks": 4096}]}"#)
            .unwrap();
        match &p.body {
            crate::preset::PresetBody::DepositsSynthetic(d) => {
                assert_eq!(d.deposits.essentials.len(), 1);
                assert_eq!(d.deposits.essentials[0].material, "c:gold");
            }
            other => panic!("{other:?}"),
        }
        assert!(
            crate::session::Preset::from_json(WORLD_4K)
                .unwrap()
                .with_content(r#"{"essentials": [{"material": "c:gold"}]}"#)
                .is_err()
        );
    }

    #[test]
    fn presets_parse_and_refuse_unknown_types() {
        let p = crate::session::Preset::from_json(PRESET_4K).unwrap();
        assert_eq!(p.name(), "deposits_synthetic");
        let back = crate::session::Preset::from_json(&p.to_json().to_string()).unwrap();
        assert_eq!(back, p);
        let bad = PRESET_4K.replace(
            "\"province\": \"weak\",",
            "\"province\": \"weak\", \"types\": [\"deeptime:nope\"],",
        );
        assert_ne!(bad, PRESET_4K);
        assert!(crate::session::Preset::from_json(&bad).is_err());
    }
}
