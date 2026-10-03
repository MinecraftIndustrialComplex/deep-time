// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `impacts` section (P2-08): every key optional, the defaults below. A planet preset
//! with a `surface` section and this section runs the impact stage (`{}` = the defaults; `flux` 0
//! runs it with no impacts). Every key is a setting (decisions log 2026-09-29: world-shaping knobs
//! are settings); the editor shows `flux` under Terrain and the rest under Advanced · Impacts.
//!
//! Diameters are block-metric kilometres (the world's own, 300 m per block at the built-in
//! scale), like the volcanoes': a crater is a physical landform whose size does not shrink with
//! the planet, and the rate is per square kilometre of the planet's own surface.
//!
//! ```json
//! "impacts": {"flux": 1.0}
//! ```

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The impacts section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ImpactParams {
    /// How often impactors arrive, as a multiple of Earth's rate over the last few hundred
    /// million years: 5.6 × 10⁻¹⁵ craters of 20 km or more per km² per year (Grieve & Shoemaker
    /// 1994, as quoted by Hughes 2000; the other published estimates are 2.6 to 5.4), and the
    /// observed record's shallower rise towards smaller craters below 20 km (`scaling.rs`).
    /// 0 = no impacts. Ledger IMP-001, IMP-006 (game choice: Ben, decisions log 2026-09-29).
    pub flux: f64,
    /// The cumulative size-frequency slope b of N(≥ D) ∝ D⁻ᵇ for craters of 20 km and more (1.8:
    /// Grieve 1987, section on the terrestrial record, "for large diameters the distribution approximates the relation
    /// N ∝ D⁻¹·⁸"). A steeper slope means more small craters for each large one. Below 20 km the
    /// record is flatter and the production function follows it, whatever this is (ledger
    /// IMP-003, IMP-004).
    pub size_slope: f64,
    /// The smallest crater simulated, km across (as it would form on Earth's land). The default
    /// 2 km is seven blocks, three cells of the finest terrain grid: a smaller bowl cannot be
    /// drawn. Game choice (ledger IMP-013; the coding agent, build log 2026-10-01). Earth's
    /// atmosphere also breaks up the stony bodies that would make craters under a few hundred
    /// metres across (Bland & Artemieva 2006), which is why the rate falls off there.
    pub min_crater_km: f64,
    /// The largest crater a run may make, km across (as on Earth's land). Game choice (ledger
    /// IMP-015; the coding agent, build log 2026-10-01; Ben has not been asked): larger impacts
    /// melt and resurface a large share of a planet this size, which the stage does not model.
    /// 300 km is the upper estimate for Vredefort (Pike 1985 lists 150 ± 10 km; other estimates
    /// reach 300), the largest known structure on Earth.
    pub max_crater_km: f64,
    /// Impact speed, km/s. Asteroids strike Earth at 12–20 km/s (Collins, Melosh & Marcus 2005,
    /// p. 819); the mean of fireball and asteroid data is about 18 (Bland & Artemieva 2006 use
    /// 18 and cite estimates from 17.3 to 20.2; Davison & Collins 2007 use 18). 20 is the top of
    /// that range and the paper's worked examples' speed (ledger IMP-020).
    pub speed_km_s: f64,
    /// Impactor density, kg/m³ (stony asteroids 2,000–3,000, iron about 8,000, comets 500–1,500:
    /// Collins, Melosh & Marcus 2005, p. 819; the defaults take 3,000 and 7,800 for iron).
    pub impactor_density_kg_m3: f64,
    /// How deep craters are, as a share of the real depth: every vertical measure of a crater
    /// (floor depth, rim, central peak, breccia lens) is its physical one times this. Heights are
    /// stretched `m_per_block_h / m_per_block_v` times against widths (8.6 at the default 35 m
    /// high against 300 m wide per block, 15 at the earlier 20 m), so a simple bowl of real
    /// proportions (1, a fifth as deep as wide) is 1.7 times deeper than wide in blocks (3 times
    /// at 20 m); the default 0.5 follows the volcanoes' `flank_steepness` (Ben, 2026-09-30:
    /// "Setting, gentler default") and draws it 0.86 times as deep as wide (1.5 at 20 m). Kept at
    /// 0.5 with the 35 m scale (2026-10-02): craters the same in metres, shallower in blocks.
    /// Game choice (ledger IMP-130): the agent reused Ben's answer for volcanoes; the question
    /// for craters is open.
    pub depth_scale: f64,
    /// How far the ejecta blanket reaches, in rim radii from the centre. The continuous blanket
    /// of lunar craters ends 2.35 radii out (Moore, Hodges & Scott 1974, eq. 1: R_ce = 2.348 R,
    /// craters of 0.65–218 km). The cubic law gives 1/27 of the rim thickness at 3 radii, but the
    /// drawn blanket is tapered to zero over its outer 30 %, so it ends at this reach. The 3 is a
    /// game choice (ledger IMP-056, IMP-057).
    pub ejecta_radii: f64,
    /// Water deeper than this many impactor diameters leaves no crater in the sea floor: 8
    /// (Davison & Collins 2007, "at water thicknesses in excess of ~8 L, no crater forms"; see
    /// `scaling::sea_floor_factor` for the rest of their rule). Ledger IMP-066.
    pub sea_cutoff_ratio: f64,
    /// Hillslope diffusivity that rounds a crater's rim and fills its bowl between the grid
    /// cells of the deep-time surface, m²/yr, multiplied by the rock's `diffusivity`. Not sourced
    /// (ledger IMP-074, `uncited`): the coding agent chose it from the range it remembered for
    /// soil-mantled hillslopes (0.003–0.05 m²/yr, attributed to Fernandes & Dietrich 1997 and
    /// Roering et al. 1999, neither read). The one diffusivity read is lunar and about 1,800
    /// times smaller (Fassett & Thomson 2014: 5.5 m²/Myr, abstract). The deep-time surface's own
    /// erosion and deposition wear a crater down as well, whatever this is.
    pub diffusivity_m2_yr: f64,
    /// Multiplies how fast the deep-time surface's erosion and deposition and the diffusion
    /// above wear a crater down (0 = craters never fade).
    pub degradation: f64,
}

impl Default for ImpactParams {
    fn default() -> Self {
        ImpactParams {
            flux: 3.0, // ledger: IMP-006 (a game choice; Earth's own rate is 1)
            size_slope: 1.8,
            min_crater_km: 2.0,
            max_crater_km: 300.0,
            speed_km_s: 20.0,
            impactor_density_kg_m3: 3000.0,
            depth_scale: 0.5,
            ejecta_radii: 3.0,
            sea_cutoff_ratio: super::scaling::SEA_CUTOFF_RATIO,
            diffusivity_m2_yr: 0.01,
            degradation: 1.0,
        }
    }
}

impl ImpactParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("$.impacts.{k}"), why);
        let range = |v: f64, lo: f64, hi: f64, k: &str| {
            if v.is_finite() && v >= lo && v <= hi {
                Ok(())
            } else {
                Err(bad(k, &format!("must be in [{lo}, {hi}]")))
            }
        };
        range(self.flux, 0.0, 1000.0, "flux")?;
        range(self.size_slope, 1.0, 3.0, "size_slope")?;
        range(self.min_crater_km, 0.5, 50.0, "min_crater_km")?;
        range(self.max_crater_km, 10.0, 1000.0, "max_crater_km")?;
        if self.max_crater_km <= self.min_crater_km {
            return Err(bad("max_crater_km", "must be larger than min_crater_km"));
        }
        range(self.speed_km_s, 5.0, 72.0, "speed_km_s")?;
        range(
            self.impactor_density_kg_m3,
            500.0,
            8000.0,
            "impactor_density_kg_m3",
        )?;
        range(self.depth_scale, 0.1, 1.0, "depth_scale")?;
        range(self.ejecta_radii, 1.5, 6.0, "ejecta_radii")?;
        range(self.sea_cutoff_ratio, 1.0, 20.0, "sea_cutoff_ratio")?;
        range(self.diffusivity_m2_yr, 0.0, 1.0, "diffusivity_m2_yr")?;
        range(self.degradation, 0.0, 10.0, "degradation")?;
        Ok(())
    }
}
