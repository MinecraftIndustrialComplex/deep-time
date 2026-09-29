// SPDX-License-Identifier: LGPL-3.0-or-later
//! Deposit types (plan §5.4.3): the datapack JSON of `data/<ns>/deeptime/deposit_type/<name>.json`,
//! parsed and checked. The JSON Schema is `crates/deeptime-format/schema/deposit_type.schema.json`;
//! this module is its executable form (unknown keys are refused, so the two cannot drift silently —
//! a test compares the schema's property names with these structs).
//!
//! The nine field groups of `ore_genesis_modeling.md` Q7 with `deeptime:` field names: identity and
//! products; genesis `components` (source, driver, pathway, trap as piecewise-linear membership
//! curves over named fields, combined so a missing process zeroes the score) plus `clock_gates`;
//! `hosts` with the `host_bias` that makes the Create stones preferred hosts; `sizing` (the USGS
//! grade–tonnage values, deposit density and same-deposit separation, copied from
//! `crates/deeptime-core/data/usgs/<grade_tonnage>.json`, which a test checks); `geometry` and
//! zoning with children; `preservation`; the child rules (`weathering` for supergene blankets,
//! `placer`, `petroleum`, `coal`); `minecraft` (`coe_vein`, `cdg_reservoir`, `ore_block_policy`);
//! and `balance`.

use std::collections::{BTreeMap, BTreeSet};

use deeptime_format::Shape;
use deeptime_format::events::is_namespaced;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Where Deep Time's own deposit types live in the mod jar (and in this repository).
pub const DATAPACK_DIR: &str = "neoforge/src/main/resources/data/deeptime/deeptime/deposit_type";

/// The built-in types, in the order the engine runs them (parents before children).
pub const BUILTIN: &[(&str, &str)] = &[
    (
        "deeptime:porphyry_cu_mo_au",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/porphyry_cu_mo_au.json"
        ),
    ),
    (
        "deeptime:supergene_cu_blanket",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/supergene_cu_blanket.json"
        ),
    ),
    (
        "deeptime:placer_au",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/placer_au.json"
        ),
    ),
    (
        "deeptime:bif_fe",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/bif_fe.json"
        ),
    ),
    (
        "deeptime:coal_seam",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/coal_seam.json"
        ),
    ),
    (
        "deeptime:mvt_zn_pb",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/mvt_zn_pb.json"
        ),
    ),
    (
        "deeptime:sedex_zn_pb",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/sedex_zn_pb.json"
        ),
    ),
    (
        "deeptime:petroleum_trap",
        include_str!(
            "../../../../neoforge/src/main/resources/data/deeptime/deeptime/deposit_type/petroleum_trap.json"
        ),
    ),
];

/// How a deposit's formation time is chosen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormationTime {
    #[default]
    Peak,
    Cause,
}

/// What a type's sites are (`deeptime_format::SiteKind` names).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteKindName {
    Deposit,
    Reservoir,
    Placer,
    Supergene,
    Fantasy,
}

impl SiteKindName {
    pub fn site_kind(self) -> deeptime_format::SiteKind {
        use deeptime_format::SiteKind as K;
        match self {
            SiteKindName::Deposit => K::Deposit,
            SiteKindName::Reservoir => K::Reservoir,
            SiteKindName::Placer => K::Placer,
            SiteKindName::Supergene => K::Supergene,
            SiteKindName::Fantasy => K::Fantasy,
        }
    }
}

/// One product of a type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductDef {
    /// `c:<material>`, resolved through `#c:ores/<material>` (plan §7.6).
    pub material: String,
    /// A key of `sizing.grades` (`cu_pct`, `au_gpt`, …), or none for a product with no grade
    /// (coal, oil).
    #[serde(default)]
    pub grade_key: Option<String>,
    /// `primary`, `coproduct`, `byproduct`.
    pub role: String,
}

/// How a component's terms combine and how components combine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Combine {
    /// The product (the default): any missing process zeroes the score.
    #[default]
    Product,
    /// The fuzzy minimum.
    Min,
    /// The geometric mean (a softer AND).
    GeometricMean,
}

/// One membership term: `curve(field)^weight`, or the fuzzy OR (maximum) of `any_of`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term {
    /// A field (`deeptime:arc_magma_flux`) or clock (`deeptime:clock/land_plants`) name.
    #[serde(default)]
    pub field: Option<String>,
    /// Piecewise-linear [x, membership] points, x strictly increasing, clamped at the ends.
    #[serde(default)]
    pub curve: Vec<[f64; 2]>,
    /// Exponent on the membership (1 by default; 0.5 softens, 2 sharpens).
    #[serde(default = "one")]
    pub weight: f64,
    /// Alternatives: the term is the maximum of these.
    #[serde(default)]
    pub any_of: Vec<Term>,
    /// Why this curve (a citation or "design").
    #[serde(default)]
    pub note: Option<String>,
}

fn one() -> f64 {
    1.0
}

/// The four mineral-system components (Skirrow et al. 2019; McCuaig et al. 2010).
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Components {
    #[serde(default)]
    pub combine: Combine,
    #[serde(default)]
    pub source: Vec<Term>,
    #[serde(default)]
    pub driver: Vec<Term>,
    #[serde(default)]
    pub pathway: Vec<Term>,
    #[serde(default)]
    pub trap: Vec<Term>,
}

/// The component names, in the order the engine records them (the limiting-component index).
pub const COMPONENTS: [&str; 6] = ["source", "driver", "pathway", "trap", "host", "clock"];

impl Components {
    pub fn list(&self) -> [&[Term]; 4] {
        [&self.source, &self.driver, &self.pathway, &self.trap]
    }
}

/// Host lithology.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hosts {
    /// Rock classes (`granodiorite`) and groups (`#deeptime:rock/carbonate`).
    #[serde(default)]
    pub any_of: Vec<String>,
    /// Class → weight. A weight above 1 prefers that class (the Create stones, decisions file:
    /// crimsite/BIF → iron, ochrum/ironstone → gold, veridium/serpentinite → copper,
    /// asurine/dolostone → zinc); a class listed only here becomes a host with that weight.
    #[serde(default)]
    pub host_bias: BTreeMap<String, f64>,
}

/// Event kinds that count as causes of each component ("Why here?").
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CauseKinds {
    #[serde(default)]
    pub source: Vec<String>,
    #[serde(default)]
    pub driver: Vec<String>,
    #[serde(default)]
    pub pathway: Vec<String>,
    #[serde(default)]
    pub trap: Vec<String>,
    /// Causes of exposure (exhumation, placers, supergene blankets).
    #[serde(default)]
    pub preservation: Vec<String>,
}

impl CauseKinds {
    pub fn components(&self) -> [(&'static str, &[String]); 4] {
        [
            ("source", &self.source),
            ("driver", &self.driver),
            ("pathway", &self.pathway),
            ("trap", &self.trap),
        ]
    }
}

/// USGS percentiles: p90 is exceeded by 90% of deposits (the small end), p10 by 10%.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Percentiles {
    pub p90: f64,
    pub p50: f64,
    pub p10: f64,
}

/// A grade distribution.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GradeDist {
    /// The USGS percentiles over all deposits, unreported grades counted as zero (so a
    /// by-product's p90, even p50, may be 0); `sizing.rs` fits the distribution among deposits
    /// that carry the metal from the positive ones and `presence`.
    #[serde(default)]
    pub p90: Option<f64>,
    pub p50: f64,
    pub p10: f64,
    /// Fraction of deposits that carry the metal at all (USGS `n_reported / n`).
    #[serde(default = "one")]
    pub presence: f64,
}

/// Clustering of positions (plan §8.6 step 3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clustering {
    /// Only `thomas` in v1.
    pub model: String,
    /// Minimum spacing of camp centres (km, block metric).
    pub camp_spacing_km: f64,
    /// Standard deviation of a deposit's offset from its camp centre (km).
    pub cluster_radius_km: f64,
}

/// Sizing (plan §8.6 steps 2–4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sizing {
    /// The USGS table these values come from (`data/usgs/<id>.json`).
    pub grade_tonnage: String,
    pub tonnage_mt: Percentiles,
    #[serde(default)]
    pub grades: BTreeMap<String, GradeDist>,
    /// `[a, b, r]` Pearson correlations of the logs (`tonnage_mt` or a grade key).
    #[serde(default)]
    pub correlations: Vec<(String, String, f64)>,
    pub ore_density_t_m3: f64,
    /// Deposits per 100,000 km² of permissive area (the count model uses p50).
    pub density_per_1e5km2: Percentiles,
    /// USGS same-deposit rule, metres.
    pub min_separation_m: f64,
    /// Favourability above which a cell is permissive.
    pub tract_threshold: f64,
    pub clustering: Clustering,
}

/// A zone of a body, relative to the type's grades.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneDef {
    pub zone: String,
    #[serde(default = "unit_range")]
    pub d: [f32; 2],
    #[serde(default = "unit_range")]
    pub v: [f32; 2],
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub rock_class: Option<String>,
    /// Multiplies the site's grade of `material` in this zone.
    #[serde(default = "one")]
    pub grade_mult: f64,
    /// Probability that a host block in the zone becomes ore.
    #[serde(default)]
    pub density: f64,
}

fn unit_range() -> [f32; 2] {
    [0.0, 1.0]
}

/// How a child relates to its parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    /// A supergene blanket at the (paleo)water table above an exposed parent.
    AboveWatertable,
    /// A placer downstream of an eroded parent.
    Downstream,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChildDef {
    pub type_id: String,
    pub relation: Relation,
}

/// When an extra body exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum When {
    /// The part above the ore body is not eroded (a lithocap).
    Preserved,
    /// The ore body reaches the surface (a gossan).
    Exposed,
}

/// A body beside the ore body (a lithocap above a porphyry, a gossan on an exposed one).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraBody {
    pub role: String,
    pub shape: String,
    pub when: When,
    /// Semi-axes relative to the ore body's horizontal semi-axis (x, z) and absolute thickness
    /// in metres (y).
    pub scale_h: f64,
    pub thickness_m: f64,
    #[serde(default)]
    pub rock_class: Option<String>,
    #[serde(default)]
    pub zones: Vec<ZoneDef>,
}

/// Geometry and zoning (plan §8.6 step 5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    /// A `deeptime_format::Shape` name.
    pub shape: String,
    /// Relative semi-axes in metres (x, vertical, z) before scaling to the tonnage.
    pub aspect: [f64; 3],
    /// Dip range in degrees (a random dip in it; 0 = flat or vertical as the shape is).
    #[serde(default)]
    pub dip_deg: [f64; 2],
    /// Rock class of the whole ore body's host (a BIF sheet is `banded_iron_formation`).
    #[serde(default)]
    pub rock_class: Option<String>,
    #[serde(default)]
    pub zoning: Vec<ZoneDef>,
    #[serde(default)]
    pub extra_bodies: Vec<ExtraBody>,
    #[serde(default)]
    pub children: Vec<ChildDef>,
}

/// What erosion does (plan §8.6 step 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErosionBehavior {
    /// Partly eroded bodies are cut at the surface; fully eroded ones vanish.
    Truncate,
    /// As truncate, and a fully eroded body spawns its downstream placer children.
    SpawnPlacer,
    /// Any erosion into the body destroys it (coal, oil: breached).
    Destroy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preservation {
    /// Depth of the body's centre below the surface when it formed (m).
    pub formation_depth_m: f64,
    /// Vertical extent of the body (m); informative, the geometry decides.
    pub vertical_extent_m: f64,
    pub erosion_behavior: ErosionBehavior,
    /// Products that survive erosion into placers.
    #[serde(default)]
    pub resistates: Vec<String>,
}

/// Supergene enrichment rules (Sillitoe 2005), for a child type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weathering {
    /// The parent's top must lie within this depth of the present surface (m).
    pub max_top_depth_m: f64,
    /// The water table must be at least this deep (m).
    pub min_water_table_m: f64,
    /// Eligible precipitation (mm/yr).
    pub precip_mm: [f64; 2],
    /// Minimum uninterrupted time with slow erosion (Myr).
    pub min_stable_myr: f64,
    /// Erosion counts as slow below this rate (m/Myr).
    pub max_erosion_m_myr: f64,
    /// Enrichment factor range over hypogene grade.
    pub grade_mult: [f64; 2],
    /// Depth of the leached column above the water table the blanket collects from (m).
    pub leach_depth_m: f64,
}

/// Placer rules, for a child type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacerRules {
    /// Maximum transport from the source (km).
    pub transport_km: f64,
    /// Fraction of the eroded resistate metal a placer may hold at most.
    pub capture_fraction: f64,
    /// Where the channel gradient falls below this (m/km), gold drops out.
    pub flatten_gradient_m_km: f64,
}

/// Petroleum-system rules (Lopatin TTI; Waples 1980).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PetroleumRules {
    pub tti_onset: f64,
    pub tti_peak: f64,
    pub tti_end: f64,
    /// Lateral migration range from a kitchen (km).
    pub migration_km: f64,
    /// Field names.
    pub source_field: String,
    pub temperature_field: String,
    pub trap_field: String,
    pub seal_field: String,
    pub reservoir_field: String,
    /// Thresholds on the trap, seal and reservoir fields.
    pub trap_threshold: f64,
    pub seal_threshold: f64,
    pub reservoir_threshold: f64,
    /// Minimum source-rock value (TOC wt%) for a kitchen.
    pub min_source: f64,
}

/// Coal rules: rank from the maximum TTI the seam reached.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoalRules {
    pub temperature_field: String,
    /// `[rank, TTI upper bound]`, ascending.
    pub rank_tti: Vec<(String, f64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinecraftMapping {
    /// A COE vein recipe id (`createoreexcavation:ore_vein_type/copper`) or null.
    #[serde(default)]
    pub coe_vein: Option<String>,
    /// Whether sites feed Create: Diesel Generators' oil (P1-16).
    #[serde(default)]
    pub cdg_reservoir: bool,
    /// `ev_then_vanilla` (decisions file: Excavated Variants, else stone/deepslate ores) or
    /// `none` (fluids).
    pub ore_block_policy: String,
}

/// Regional specialisation strength (decisions file: configurable, default strong).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Province {
    Off,
    Weak,
    Medium,
    #[default]
    Strong,
}

impl Province {
    pub fn from_name(s: &str) -> Option<Province> {
        match s {
            "off" => Some(Province::Off),
            "weak" => Some(Province::Weak),
            "medium" => Some(Province::Medium),
            "strong" => Some(Province::Strong),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Province::Off => "off",
            Province::Weak => "weak",
            Province::Medium => "medium",
            Province::Strong => "strong",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Balance {
    #[serde(default = "one")]
    pub count_mult: f64,
    #[serde(default = "one")]
    pub size_mult_h: f64,
    #[serde(default = "one")]
    pub size_mult_v: f64,
    #[serde(default = "one")]
    pub richness_mult: f64,
    /// Overrides the preset's province strength for this type.
    #[serde(default)]
    pub province: Option<Province>,
}

impl Default for Balance {
    fn default() -> Self {
        Balance {
            count_mult: 1.0,
            size_mult_h: 1.0,
            size_mult_v: 1.0,
            richness_mult: 1.0,
            province: None,
        }
    }
}

/// A fantasy field a type needs, with a fallback (plan §5.4.3; Phase 3 computes them).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldFallback {
    pub provider: String,
    #[serde(default)]
    pub fallback: serde_json::Value,
}

/// One deposit type (plan §5.4.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepositType {
    pub id: String,
    pub family: String,
    pub site_kind: SiteKindName,
    /// Children exist only when a parent spawns them.
    #[serde(default)]
    pub child_only: bool,
    #[serde(default)]
    pub description: String,
    pub products: Vec<ProductDef>,
    #[serde(default)]
    pub components: Components,
    /// Clocks (`deeptime:clock/oxygenated_oceans`) the whole type is multiplied by.
    #[serde(default)]
    pub clock_gates: Vec<String>,
    /// The time an ideal system needs (Myr): favourability integrates as
    /// `1 − Π (1 − F_t)^(Δt/τ)` over the history.
    #[serde(default = "ten")]
    pub integration_myr: f64,
    /// When a deposit formed: at its cell's most favourable frame (`peak`), or with its earliest
    /// cause (`cause`: inherited deposits older than the timeline, such as BIF in a craton).
    #[serde(default)]
    pub formation_time: FormationTime,
    #[serde(default)]
    pub hosts: Hosts,
    #[serde(default)]
    pub causes: CauseKinds,
    /// How far from a site a causing event may lie (km, block metric).
    #[serde(default = "cause_radius")]
    pub cause_radius_km: f64,
    pub sizing: Sizing,
    pub geometry: Geometry,
    pub preservation: Preservation,
    #[serde(default)]
    pub weathering: Option<Weathering>,
    #[serde(default)]
    pub placer: Option<PlacerRules>,
    #[serde(default)]
    pub petroleum: Option<PetroleumRules>,
    #[serde(default)]
    pub coal: Option<CoalRules>,
    pub minecraft: MinecraftMapping,
    #[serde(default)]
    pub balance: Balance,
    #[serde(default)]
    pub requires_fields: BTreeMap<String, FieldFallback>,
    /// Sources and design notes (free text).
    #[serde(default)]
    pub notes: Vec<String>,
}

fn ten() -> f64 {
    10.0
}

fn cause_radius() -> f64 {
    600.0
}

/// The rock groups of the rock-tag vocabulary (`deeptime-mic-rock-tags/README.md`, "Groups").
pub fn rock_group(name: &str) -> Option<&'static [&'static str]> {
    Some(match name {
        "deeptime:rock/intrusive" => &[
            "granite",
            "granodiorite",
            "diorite",
            "gabbro",
            "anorthosite",
            "peridotite",
        ],
        "deeptime:rock/volcanic" => &[
            "rhyolite", "andesite", "basalt", "tuff", "obsidian", "scoria", "pumice",
        ],
        "deeptime:rock/clastic" => &[
            "conglomerate",
            "sandstone",
            "red_bed",
            "mudstone",
            "shale",
            "black_shale",
        ],
        "deeptime:rock/carbonate" => &["limestone", "dolostone", "chalk", "travertine", "marble"],
        "deeptime:rock/evaporite" => &["halite", "gypsum", "anhydrite"],
        "deeptime:rock/iron_formation" => &["ironstone", "banded_iron_formation"],
        "deeptime:rock/felsic" => &["granite", "granodiorite", "rhyolite", "pegmatite", "greisen"],
        "deeptime:rock/intermediate" => &["diorite", "andesite"],
        "deeptime:rock/mafic" => &[
            "gabbro",
            "anorthosite",
            "basalt",
            "scoria",
            "amphibolite",
            "greenstone",
        ],
        "deeptime:rock/ultramafic" => &["peridotite", "serpentinite", "kimberlite"],
        "deeptime:rock/unconsolidated" => &[
            "soil",
            "saprolite",
            "laterite",
            "bauxite",
            "gravel",
            "sand",
            "clay",
            "mud",
            "till",
            "loess",
            "volcanic_ash",
        ],
        _ => return None,
    })
}

impl Hosts {
    /// Class → weight: every class of `any_of` (groups expanded) at 1, then `host_bias`
    /// multiplying listed classes or adding unlisted ones at their weight.
    pub fn weights(&self) -> Result<BTreeMap<String, f64>, String> {
        let mut w = BTreeMap::new();
        for h in &self.any_of {
            if let Some(g) = h.strip_prefix('#') {
                let members = rock_group(g).ok_or_else(|| format!("unknown rock group #{g}"))?;
                for m in members {
                    w.insert((*m).to_owned(), 1.0);
                }
            } else {
                w.insert(h.clone(), 1.0);
            }
        }
        for (class, bias) in &self.host_bias {
            if !(bias.is_finite() && *bias >= 0.0) {
                return Err(format!("host_bias {class}: weight must be ≥ 0"));
            }
            match w.get_mut(class) {
                Some(v) => *v *= bias,
                None => {
                    w.insert(class.clone(), *bias);
                }
            }
        }
        Ok(w)
    }
}

impl Term {
    /// The membership curve at `x`.
    pub fn curve_at(&self, x: f64) -> f64 {
        let c = &self.curve;
        if c.is_empty() || x.is_nan() {
            return 0.0;
        }
        if x <= c[0][0] {
            return c[0][1];
        }
        for w in c.windows(2) {
            if x <= w[1][0] {
                let t = (x - w[0][0]) / (w[1][0] - w[0][0]);
                return w[0][1] + (w[1][1] - w[0][1]) * t;
            }
        }
        c[c.len() - 1][1]
    }

    /// Every field this term reads.
    pub fn fields(&self, out: &mut BTreeSet<String>) {
        if let Some(f) = &self.field {
            out.insert(f.clone());
        }
        for t in &self.any_of {
            t.fields(out);
        }
    }

    fn validate(&self, at: &str) -> Result<(), String> {
        match (&self.field, self.any_of.is_empty()) {
            (Some(f), true) => {
                if !is_namespaced(f) {
                    return Err(format!("{at}: field {f:?} is not namespaced"));
                }
                if self.curve.len() < 2 {
                    return Err(format!("{at}: a curve needs at least two points"));
                }
                if self.curve.windows(2).any(|w| w[1][0] <= w[0][0]) {
                    return Err(format!("{at}: curve x values must increase"));
                }
                if self
                    .curve
                    .iter()
                    .any(|p| !p[0].is_finite() || !(0.0..=1.0).contains(&p[1]))
                {
                    return Err(format!("{at}: memberships must lie in [0, 1]"));
                }
            }
            (None, false) => {
                for (k, t) in self.any_of.iter().enumerate() {
                    t.validate(&format!("{at}.any_of[{k}]"))?;
                }
            }
            _ => return Err(format!("{at}: give either field + curve or any_of")),
        }
        if !(self.weight > 0.0 && self.weight.is_finite()) {
            return Err(format!("{at}: weight must be > 0"));
        }
        Ok(())
    }
}

impl DepositType {
    /// Parses one datapack file.
    pub fn from_json(text: &str) -> Result<DepositType, CoreError> {
        let t: DepositType = serde_json::from_str(text)
            .map_err(|e| CoreError::preset("deposit_type", format!("{e}")))?;
        t.validate()
            .map_err(|e| CoreError::preset(format!("deposit_type {}", t.id), e))?;
        Ok(t)
    }

    /// The path part of the id (`porphyry_cu_mo_au`).
    pub fn short(&self) -> &str {
        self.id.split_once(':').map_or(&self.id, |(_, p)| p)
    }

    pub fn shape(&self) -> Shape {
        Shape::from_name(&self.geometry.shape).expect("validated")
    }

    pub fn province(&self, preset: Province) -> Province {
        self.balance.province.unwrap_or(preset)
    }

    /// Every field and clock the type's rules read.
    pub fn fields(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for comp in self.components.list() {
            for t in comp {
                t.fields(&mut out);
            }
        }
        out.extend(self.clock_gates.iter().cloned());
        if let Some(p) = &self.petroleum {
            for f in [
                &p.source_field,
                &p.temperature_field,
                &p.trap_field,
                &p.seal_field,
                &p.reservoir_field,
            ] {
                out.insert(f.clone());
            }
        }
        if let Some(c) = &self.coal {
            out.insert(c.temperature_field.clone());
        }
        out
    }

    /// The type's own checks (cross-type checks are in [`validate_set`]).
    pub fn validate(&self) -> Result<(), String> {
        if !is_namespaced(&self.id) {
            return Err(format!("id {:?} is not namespaced", self.id));
        }
        if self.products.is_empty() {
            return Err("no products".into());
        }
        for (k, p) in self.products.iter().enumerate() {
            if !is_namespaced(&p.material) {
                return Err(format!("products[{k}]: material {:?} is not namespaced", p.material));
            }
            if let Some(g) = &p.grade_key
                && !self.sizing.grades.contains_key(g)
            {
                return Err(format!("products[{k}]: grade {g} is not in sizing.grades"));
            }
        }
        for (name, comp) in ["source", "driver", "pathway", "trap"]
            .iter()
            .zip(self.components.list())
        {
            for (k, t) in comp.iter().enumerate() {
                t.validate(&format!("components.{name}[{k}]"))?;
            }
        }
        if !self.child_only
            && self.petroleum.is_none()
            && self.components.list().iter().all(|c| c.is_empty())
        {
            return Err("a type that is not child_only needs components".into());
        }
        for g in &self.clock_gates {
            if !is_namespaced(g) {
                return Err(format!("clock gate {g:?} is not namespaced"));
            }
        }
        if self.integration_myr.is_nan() || self.integration_myr <= 0.0 {
            return Err("integration_myr must be > 0".into());
        }
        self.hosts.weights()?;
        let s = &self.sizing;
        let ordered = |p: &Percentiles| p.p90 > 0.0 && p.p90 <= p.p50 && p.p50 <= p.p10;
        if !ordered(&s.tonnage_mt) {
            return Err("sizing.tonnage_mt must satisfy 0 < p90 ≤ p50 ≤ p10".into());
        }
        if !ordered(&s.density_per_1e5km2) {
            return Err("sizing.density_per_1e5km2 must satisfy 0 < p90 ≤ p50 ≤ p10".into());
        }
        for (k, g) in &s.grades {
            let p90_ok = g.p90.is_none_or(|v| v >= 0.0 && v <= g.p50);
            if !(g.p10 > 0.0 && g.p50 >= 0.0 && g.p50 <= g.p10 && p90_ok)
                || !(0.0..=1.0).contains(&g.presence)
            {
                return Err(format!(
                    "sizing.grades.{k} must satisfy 0 ≤ p90 ≤ p50 ≤ p10, p10 > 0, presence in [0, 1]"
                ));
            }
        }
        for (a, b, r) in &s.correlations {
            for key in [a, b] {
                if key != "tonnage_mt" && !s.grades.contains_key(key) {
                    return Err(format!("correlation names unknown key {key}"));
                }
            }
            if r.is_nan() || r.abs() >= 1.0 || a == b {
                return Err(format!("correlation {a}–{b} must have |r| < 1"));
            }
        }
        if !(s.ore_density_t_m3 > 0.0 && s.min_separation_m >= 0.0) {
            return Err("sizing.ore_density_t_m3 must be > 0, min_separation_m ≥ 0".into());
        }
        if !(0.0..1.0).contains(&s.tract_threshold) {
            return Err("sizing.tract_threshold must lie in [0, 1)".into());
        }
        if s.clustering.model != "thomas"
            || !(s.clustering.camp_spacing_km >= 0.0 && s.clustering.cluster_radius_km > 0.0)
        {
            return Err("sizing.clustering: model thomas with radii ≥ 0".into());
        }
        if Shape::from_name(&self.geometry.shape).is_none() {
            return Err(format!("geometry.shape {:?} is unknown", self.geometry.shape));
        }
        if !self.geometry.aspect.iter().all(|v| *v > 0.0) {
            return Err("geometry.aspect must be > 0".into());
        }
        let zones = self
            .geometry
            .zoning
            .iter()
            .chain(self.geometry.extra_bodies.iter().flat_map(|e| &e.zones));
        for z in zones {
            if let Some(m) = &z.material
                && !self.products.iter().any(|p| &p.material == m)
            {
                return Err(format!("zone {}: material {m} is not a product", z.zone));
            }
            if !(0.0..=1.0).contains(&z.density) || z.d[0] > z.d[1] || z.v[0] > z.v[1] {
                return Err(format!("zone {}: bad range or density", z.zone));
            }
        }
        for e in &self.geometry.extra_bodies {
            if Shape::from_name(&e.shape).is_none() || !(e.scale_h > 0.0 && e.thickness_m > 0.0) {
                return Err(format!("extra body {}: bad shape or size", e.role));
            }
        }
        let b = &self.balance;
        if ![b.count_mult, b.size_mult_h, b.size_mult_v, b.richness_mult]
            .iter()
            .all(|v| *v >= 0.0 && v.is_finite())
        {
            return Err("balance multipliers must be ≥ 0".into());
        }
        if !["ev_then_vanilla", "none"].contains(&self.minecraft.ore_block_policy.as_str()) {
            return Err("minecraft.ore_block_policy must be ev_then_vanilla or none".into());
        }
        if self.site_kind == SiteKindName::Supergene && self.weathering.is_none() {
            return Err("a supergene type needs weathering rules".into());
        }
        if self.site_kind == SiteKindName::Placer && self.placer.is_none() {
            return Err("a placer type needs placer rules".into());
        }
        Ok(())
    }
}

/// Checks a set of types together: unique ids, children defined and `child_only`, relations
/// that fit the child's kind.
pub fn validate_set(types: &[DepositType]) -> Result<(), CoreError> {
    let mut seen = BTreeSet::new();
    for t in types {
        if !seen.insert(t.id.as_str()) {
            return Err(CoreError::preset("deposit_types", format!("{} twice", t.id)));
        }
    }
    for t in types {
        for c in &t.geometry.children {
            let child = types.iter().find(|x| x.id == c.type_id).ok_or_else(|| {
                CoreError::preset(
                    format!("deposit_type {}", t.id),
                    format!("child {} is not loaded", c.type_id),
                )
            })?;
            let fits = match c.relation {
                Relation::AboveWatertable => child.site_kind == SiteKindName::Supergene,
                Relation::Downstream => child.site_kind == SiteKindName::Placer,
            };
            if !child.child_only || !fits {
                return Err(CoreError::preset(
                    format!("deposit_type {}", t.id),
                    format!("child {} must be child_only and fit {:?}", c.type_id, c.relation),
                ));
            }
        }
    }
    Ok(())
}

/// The built-in types, parsed and checked.
pub fn builtin() -> Result<Vec<DepositType>, CoreError> {
    let types = BUILTIN
        .iter()
        .map(|(id, text)| {
            let t = DepositType::from_json(text)?;
            if t.id != *id {
                return Err(CoreError::preset(
                    "deposit_type",
                    format!("{id}: the file says {}", t.id),
                ));
            }
            Ok(t)
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_set(&types)?;
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_types_parse_and_validate() {
        let types = builtin().unwrap();
        assert_eq!(types.len(), BUILTIN.len());
        let porphyry = &types[0];
        assert_eq!(porphyry.short(), "porphyry_cu_mo_au");
        assert!(porphyry.fields().contains("deeptime:arc_magma_flux"));
    }

    #[test]
    fn curves_interpolate_and_clamp() {
        let t = Term {
            field: Some("deeptime:x".into()),
            curve: vec![[0.0, 0.2], [10.0, 1.0], [20.0, 0.0]],
            weight: 1.0,
            any_of: vec![],
            note: None,
        };
        assert_eq!(t.curve_at(-5.0), 0.2);
        assert!((t.curve_at(5.0) - 0.6).abs() < 1e-12);
        assert_eq!(t.curve_at(15.0), 0.5);
        assert_eq!(t.curve_at(99.0), 0.0);
        assert_eq!(t.curve_at(f64::NAN), 0.0);
    }

    #[test]
    fn host_weights_expand_groups_and_bias() {
        let h = Hosts {
            any_of: vec!["#deeptime:rock/carbonate".into(), "black_shale".into()],
            host_bias: [("dolostone".to_string(), 2.0), ("serpentinite".to_string(), 0.5)]
                .into_iter()
                .collect(),
        };
        let w = h.weights().unwrap();
        assert_eq!(w["limestone"], 1.0);
        assert_eq!(w["dolostone"], 2.0);
        assert_eq!(w["serpentinite"], 0.5);
        assert_eq!(w["black_shale"], 1.0);
        assert!(
            Hosts {
                any_of: vec!["#deeptime:rock/nope".into()],
                host_bias: BTreeMap::new()
            }
            .weights()
            .is_err()
        );
    }

    #[test]
    fn unknown_keys_are_refused() {
        let text = BUILTIN[0].1.replacen('{', "{\"surprise\": 1,", 1);
        assert!(DepositType::from_json(&text).is_err());
    }

    /// The JSON Schema and the structs name the same top-level properties.
    #[test]
    fn schema_matches_structs() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../../deeptime-format/schema/deposit_type.schema.json"
        ))
        .unwrap();
        let props: BTreeSet<String> = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        let t = builtin().unwrap().remove(0);
        let v = serde_json::to_value(&t).unwrap();
        let fields: BTreeSet<String> = v.as_object().unwrap().keys().cloned().collect();
        assert_eq!(props, fields);
        let required: BTreeSet<String> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_owned())
            .collect();
        for r in &required {
            assert!(fields.contains(r), "{r}");
        }
    }
}
