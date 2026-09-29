// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `structure` section (P1-07): every key optional, the defaults below. A planet
//! preset with a `surface` section runs the structure stage whether or not it has this section.
//!
//! ```json
//! "structure": {"horizons": 32, "loose_depth_m": 150, "pluton_rate_km2_myr": 1e-4}
//! ```

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The structure section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct StructureParams {
    /// K, horizons per strata-grid cell (plan §5.7.4: 32 in the built-in presets).
    pub horizons: u32,
    /// Surface temperature of the geotherm where no climate stage wrote `climate/t_mean_c`, °C.
    pub surface_temp_c: f64,
    /// Thermal conductivity of the linear geotherm T(z) = T_s + q·z/k, W/(m·K) (the surface's
    /// burial temperature uses the same 2.5, a typical upper-crust value).
    pub conductivity_w_mk: f64,
    /// Heat flow above this is not used for the geotherm, mW/m² (the plate model's proxy reaches
    /// 300 near ridges and arcs, where advection, not conduction, carries the heat; continental
    /// surface heat flow rarely exceeds ~120 outside active volcanic centres).
    pub max_heat_flow_mw_m2: f64,
    /// Thickness of normal continental crust, m: the plate model's initial column (35 km,
    /// `plates::pools::continental_column`), the reference of the collision burial factor.
    pub reference_crust_m: f64,
    /// Largest collision burial factor β (depths below the surface scale by β = e/e₀ while the
    /// crust under a collision zone is thickened, homogeneous pure shear).
    pub max_burial_factor: f64,
    /// Lithification: the part of a young deposit within this depth of the surface is shown as its
    /// loose equivalent, m. Design value: tectonics.js lithifies sediment under 2.2 MPa of
    /// overburden, about 150 m (`plates::pools`, the same threshold the plate model uses).
    pub loose_depth_m: f64,
    /// … and "young" means a base age within this many Myr of the present. Design value:
    /// Neogene clastics are commonly still unconsolidated at shallow depth.
    pub loose_age_myr: f64,
    /// The ore window the host-rock fractions cover, from the surface down, m. Design value:
    /// porphyry stocks are emplaced within a few km of the palaeosurface and sediment-hosted
    /// Zn–Pb, coal and petroleum reservoirs sit in the top ~2 km of their basins.
    pub host_window_m: f64,
    /// Plutons emplaced per km² of full-intensity arc per Myr. Design value of the order of the
    /// Coastal Batholith of Peru (roughly a thousand plutons in a 1,600 × 60 km arc over some
    /// 60 Myr, i.e. ~1e-4 per km² per Myr).
    pub pluton_rate_km2_myr: f64,
    /// Collision (S-type, crustal-melt) granites per km² of collision zone, relative to the arc
    /// rate.
    pub collision_pluton_factor: f64,
    /// Pluton length L (the long horizontal axis), km: [median, min, max] of a lognormal with
    /// `pluton_length_sigma`. Thickness T = 0.6·L^0.6 km (the tabular-pluton scaling of Cruden
    /// 1998, J. Geol. Soc. 155:853–862).
    pub pluton_length_km: [f64; 3],
    pub pluton_length_sigma: f64,
    /// Depth of a pluton's roof below the surface when it is emplaced, km [min, max] (epizonal to
    /// mesozonal emplacement, Buddington 1959).
    pub pluton_roof_km: [f64; 2],
    /// Share of a pluton's normalised radius its contact aureole (hornfels or skarn) takes, as
    /// the outer shell of the body (0 turns aureoles off).
    pub aureole_shell: f64,
    /// Fold-thrust belts reach this far into the foreland of an orogen, km (the Zagros Simply
    /// Folded Belt is ~250 km wide; the Subandean belt 100–200 km).
    pub fold_belt_km: f64,
    /// Sediment a fold, fault-block or growth-fault closure needs, m.
    pub trap_min_sediment_m: f64,
    /// Salt domes: halite at least this thick …
    pub salt_min_m: f64,
    /// … under at least this overburden, m (salt rises once the overburden is dense enough to
    /// drive it: Hudec & Jackson 2007, Earth-Sci. Rev. 82:1–28).
    pub salt_overburden_m: f64,
    /// Growth faults and rollover anticlines in sediment wedges thicker than this, m (deltas such
    /// as the Niger's, Doust & Omatsola 1990, AAPG Memoir 48).
    pub growth_fault_sediment_m: f64,
}

impl Default for StructureParams {
    fn default() -> Self {
        StructureParams {
            horizons: 32,
            surface_temp_c: 10.0,
            conductivity_w_mk: 2.5,
            max_heat_flow_mw_m2: 120.0,
            reference_crust_m: 35_000.0,
            max_burial_factor: 2.0,
            loose_depth_m: 150.0,
            loose_age_myr: 20.0,
            host_window_m: 2000.0,
            pluton_rate_km2_myr: 1.0e-4,
            collision_pluton_factor: 0.3,
            pluton_length_km: [10.0, 3.0, 40.0],
            pluton_length_sigma: 0.5,
            pluton_roof_km: [2.0, 6.0],
            aureole_shell: 0.15,
            fold_belt_km: 300.0,
            trap_min_sediment_m: 500.0,
            salt_min_m: 100.0,
            salt_overburden_m: 1000.0,
            growth_fault_sediment_m: 1500.0,
        }
    }
}

fn check(ok: bool, key: &str, reason: &str) -> Result<(), CoreError> {
    if ok {
        Ok(())
    } else {
        Err(CoreError::preset(format!("$.structure.{key}"), reason))
    }
}

impl StructureParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        check(
            (2..=255).contains(&self.horizons),
            "horizons",
            "must be in [2, 255]",
        )?;
        check(
            self.conductivity_w_mk > 0.0,
            "conductivity_w_mk",
            "must be > 0",
        )?;
        check(
            self.max_heat_flow_mw_m2 > 0.0,
            "max_heat_flow_mw_m2",
            "must be > 0",
        )?;
        check(
            self.reference_crust_m > 0.0,
            "reference_crust_m",
            "must be > 0",
        )?;
        check(
            self.max_burial_factor >= 1.0,
            "max_burial_factor",
            "must be ≥ 1",
        )?;
        for (v, key) in [
            (self.loose_depth_m, "loose_depth_m"),
            (self.loose_age_myr, "loose_age_myr"),
            (self.pluton_rate_km2_myr, "pluton_rate_km2_myr"),
            (self.collision_pluton_factor, "collision_pluton_factor"),
            (self.pluton_length_sigma, "pluton_length_sigma"),
            (self.fold_belt_km, "fold_belt_km"),
            (self.trap_min_sediment_m, "trap_min_sediment_m"),
            (self.salt_min_m, "salt_min_m"),
            (self.salt_overburden_m, "salt_overburden_m"),
            (self.growth_fault_sediment_m, "growth_fault_sediment_m"),
        ] {
            check(v.is_finite() && v >= 0.0, key, "must be ≥ 0")?;
        }
        check(self.host_window_m > 0.0, "host_window_m", "must be > 0")?;
        let [m, lo, hi] = self.pluton_length_km;
        check(
            lo > 0.0 && lo <= m && m <= hi,
            "pluton_length_km",
            "[median, min, max] with 0 < min ≤ median ≤ max",
        )?;
        let [a, b] = self.pluton_roof_km;
        check(
            a >= 0.0 && a <= b,
            "pluton_roof_km",
            "[min, max] with 0 ≤ min ≤ max",
        )?;
        check(
            (0.0..1.0).contains(&self.aureole_shell),
            "aureole_shell",
            "must be in [0, 1)",
        )?;
        Ok(())
    }
}
