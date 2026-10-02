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
    /// 1994). 0 = no impacts.
    pub flux: f64,
    /// The cumulative size-frequency slope b of N(≥ D) ∝ D⁻ᵇ (1.8: Grieve & Shoemaker 1994). A
    /// steeper slope means more small craters for each large one.
    pub size_slope: f64,
    /// The smallest crater simulated, km across (as it would form on Earth's land). The default
    /// 2 km is seven blocks, three cells of the finest terrain grid: a smaller bowl cannot be
    /// drawn, and Earth's atmosphere breaks up most of the stony bodies that would make one
    /// (Bland & Artemieva 2006, Meteoritics & Planet. Sci. 41:607–631).
    pub min_crater_km: f64,
    /// The largest crater a run may make, km across (as on Earth's land). 300 km is the largest
    /// known on Earth (Vredefort, 2.02 Gyr). Larger impacts melt and resurface a large share of a
    /// planet this size, which the stage does not model.
    pub max_crater_km: f64,
    /// Impact speed, km/s (asteroids strike Earth at about 20 km/s on average: Collins, Melosh &
    /// Marcus 2005; Le Feuvre & Wieczorek 2011, Icarus 214:1–20).
    pub speed_km_s: f64,
    /// Impactor density, kg/m³ (stony asteroids: 3,000; iron 7,800; comets ~1,000).
    pub impactor_density_kg_m3: f64,
    /// How deep craters are, as a share of the real depth: every vertical measure of a crater
    /// (floor depth, rim, central peak, breccia lens) is its physical one times this. Heights are stretched 15 times
    /// against widths (20 m high against 300 m wide per block), so a bowl of real proportions (1)
    /// is a shaft three times deeper than wide in blocks; the default 0.5 follows the volcanoes'
    /// `flank_steepness` (Ben, 2026-09-30: "Setting, gentler default").
    pub depth_scale: f64,
    /// How far the ejecta blanket reaches, in rim radii from the centre. The continuous blanket
    /// of lunar craters ends about 2.3 radii out (Moore, Hodges & Scott 1974, Proc. Lunar Sci.
    /// Conf. 5:71–100). The cubic law gives 1/27 of the rim thickness at 3 radii, but the drawn
    /// blanket is tapered to zero over its outer 30 %, so it ends at this reach.
    pub ejecta_radii: f64,
    /// Water deeper than this many impactor diameters stops the impactor before the sea floor:
    /// no crater (6.3, fitted to Davison & Collins 2007; `scaling::sea_floor_factor`).
    pub sea_cutoff_ratio: f64,
    /// Hillslope diffusivity that rounds a crater's rim and fills its bowl between the grid
    /// cells of the deep-time surface, m²/yr, multiplied by the rock's `diffusivity`. Soil-mantled
    /// hillslopes measure 0.003–0.05 m²/yr (Fernandes & Dietrich 1997, Water Resour. Res.
    /// 33:1307–1318; Roering, Kirchner & Dietrich 1999). The deep-time surface's own erosion and
    /// deposition wear a crater down as well, whatever this is.
    pub diffusivity_m2_yr: f64,
    /// Multiplies how fast the deep-time surface's erosion and deposition and the diffusion
    /// above wear a crater down (0 = craters never fade).
    pub degradation: f64,
}

impl Default for ImpactParams {
    fn default() -> Self {
        ImpactParams {
            flux: 1.0,
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
