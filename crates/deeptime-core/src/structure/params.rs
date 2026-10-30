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
    // ledger: STR-001 (game choice)
    pub horizons: u32,
    /// Surface temperature of the geotherm where no climate stage wrote `climate/t_mean_c`, °C.
    pub surface_temp_c: f64,
    /// Thermal conductivity of the linear geotherm T(z) = T_s + q·z/k, W/(m·K) (the surface's
    /// burial temperature uses the same 2.5). Dense crystalline rocks at 300 K measure 2.1 to
    /// 3.0 (gneiss across the foliation) up to 4.5 (along it), marble 2.8–2.9, amphibolite 2.9
    /// (Robertson 1988, USGS OFR 88-441, table 2); a crust-average table was not read.
    // ledger: RCK-029, RCK-028 (Fourier's law)
    pub conductivity_w_mk: f64,
    /// Heat flow above this is not used for the geotherm, mW/m² (the plate model's proxy reaches
    /// 300 near ridges and arcs). A clamp, not a physical limit: 120 mW/m² at k = 2.5 is
    /// 48 °C/km, the gradient of rifts and arcs (about 50 K/km, Hacker's lecture notes), while
    /// measured values run above 150 mW/m² (Salton Trough, Sass et al. 1994) and the young
    /// ocean averages 128 (Davies & Davies 2010).
    // ledger: RCK-030 (game choice)
    pub max_heat_flow_mw_m2: f64,
    /// Thickness of normal continental crust, m: the plate model's initial column (35 km,
    /// `plates::pools::continental_column`), the reference of the collision burial factor. The
    /// global mean is 41.1 km (extended crust 30.5, orogens 46.3; Christensen & Mooney 1995,
    /// abstract).
    // ledger: RCK-033
    pub reference_crust_m: f64,
    /// Largest collision burial factor β (depths below the surface scale by β = e/e₀ while the
    /// crust under a collision zone is thickened, homogeneous pure shear). A doubling of 35 km
    /// reaches 70 km; continental crust is "never thicker than ~75 km" (Hacker's lecture notes).
    // ledger: RCK-032, RCK-034 (game choice)
    pub max_burial_factor: f64,
    /// Lithification: the part of a young deposit within this depth of the surface is shown as its
    /// loose equivalent, m. Game choice: tectonics.js lithifies sediment under 2.2 MPa of
    /// overburden (150 m of sediment at 1,500 kg/m³; its comment cites the AAPG wiki page on
    /// sandstone diagenesis for "500 ft", which the page does not state), and the plate model
    /// uses the same threshold (`plates::pools`). Real mud and sand are still 47 to 58 % pore
    /// space at 150 m and stiffen over 1 to 3 km (Sclater & Christie 1980 curves via the
    /// pyBacktrack tables; Bjørlykke et al. 2009), so this is a look-of-the-ground rule, not
    /// diagenesis.
    // ledger: RCK-022 (game choice)
    pub loose_depth_m: f64,
    /// … and "young" means a base age within this many Myr of the present. Game choice:
    /// Neogene clastics are commonly still unconsolidated at shallow depth.
    // ledger: RCK-022
    pub loose_age_myr: f64,
    /// The ore window the host-rock fractions cover, from the surface down, m. Game choice:
    /// porphyry stocks are emplaced within a few km of the palaeosurface and sediment-hosted
    /// Zn–Pb, coal and petroleum reservoirs sit in the top ~2 km of their basins.
    // ledger: STR-041 (game choice)
    pub host_window_m: f64,
    /// Plutons emplaced per km² of full-intensity arc per Myr. Calibrated, not measured. Each
    /// pluton stands for an intrusive suite (P1-17): the increments of a batholith merge at 300 m
    /// per block. P1-07's first rate, ~1e-4 per km² per Myr at a median 10 km, was set from the
    /// Coastal Batholith of Peru: "up to" 1,000 plutons along 1,600 km, active about 100 to
    /// 37 Ma (secondary sources; Haederle & Atherton 2002 and Mukasa 1986 not read; no source
    /// gave the 60-km width assumed here, and 1,000 / (1,600 × 60 km² × 60 Myr) is 1.7e-4, not
    /// 1e-4). The rate is that figure scaled by (10/25)^2.6 = 0.09 (rounded to 1e-5), the volume
    /// ratio of a median 10-km and a median 25-km pluton (V ∝ D²·T ∝ D^2.6, from the 0.6
    /// exponent of `plutons::thickness_km`), so arcs emplace about the same volume in a tenth
    /// as many bodies.
    // ledger: RCK-056 (calibrated), RCK-057
    pub pluton_rate_km2_myr: f64,
    /// Collision (S-type, crustal-melt) granites per km² of collision zone, relative to the arc
    /// rate. Game choice. Himalayan leucogranites are produced at under 10 km³ per km per Myr,
    /// island arcs at 20 to 40 and the Peruvian and Canadian Coastal batholiths at about 200
    /// (Vigneresse & Burg 2003), so 0.3 fits an island arc, not a continental arc.
    // ledger: RCK-058 (game choice)
    pub collision_pluton_factor: f64,
    /// The melt window of a collision ([`crate::structure::collision_melt`]): the collision
    /// granites, the granite stocks and the thickened crust's metamorphism start this many Myr
    /// after the collision orogeny's onset (`plates/orogeny_onset_age_myr`). Thermal models need
    /// "a time period of at least 20 to 30 million yrs between collision and melting" (Hopkinson
    /// et al. 2017, p. 58); Thompson & England (1984, Fig. 8) show a small melt field 20 Myr
    /// after a thrust and none at 0.
    // ledger: RCK-098
    pub collision_melt_after_myr: f64,
    /// … and end this many Myr after the last collision stamp (`plates/orogeny_age_myr`): the
    /// last of Thompson & England's (1984, Fig. 8) panels with a melt field, 80 Myr after the
    /// thrust (none at 120).
    // ledger: RCK-099
    pub collision_melt_end_myr: f64,
    /// Pluton length L (the long horizontal axis), km: [median, min, max] of a lognormal with
    /// `pluton_length_sigma`. Game choice, median 25 km (P1-17; P1-07 had 10): the size of an
    /// intrusive suite such as the Sierra Nevada's Tuolumne, ~1,100 km² (Frontiers in Earth
    /// Science 8:190, 2020; its 45 × 25 km outline was not found in any source read); central
    /// Sierra plutons range from under 1 to over 1,000 km² (Bateman 1992, USGS PP 1483).
    /// Thickness follows `plutons::thickness_km` (Cruden & McCaffrey 2001).
    // ledger: RCK-060 (game choice), RCK-061
    pub pluton_length_km: [f64; 3],
    pub pluton_length_sigma: f64,
    /// Depth of a pluton's roof below the surface when it is emplaced, km [min, max] (epizonal to
    /// mesozonal emplacement, Buddington 1959, not read; secondary sources put the epizone at 0 to
    /// ~5–6 km and the mesozone to ~11–12 km). The central Sierra Nevada was emplaced at 3.5 to
    /// 7 km (Bateman 1992) and the Tuolumne suite at 6 to 10 km, so 2 to 6 km is shallow.
    // ledger: RCK-063
    pub pluton_roof_km: [f64; 2],
    /// Width of an intrusion's contact aureole (hornfels or skarn), the shell of host rock it
    /// bakes outside itself, as a share of the intrusion's thickness (0 turns aureoles off;
    /// `plutons::aureole_width_m`). Game choice. Aureole thickness grows linearly with the
    /// intrusion's thickness, and Annen 2017 (Front. Earth Sci. 5:82) collects measured ratios:
    /// ~0.3 for the western Adamello (1.7 km round 5.5 km), 0.3 to 0.5 for the Alta stock, at
    /// least 0.5 for Mooselookmeguntic, 0.05 to 0.2 for Cupsuctic, 0.05 for Skaergaard's floor,
    /// 0.01 to 0.02 for Manaslu (incremental assembly). 0.3 is the middle of the bodies emplaced
    /// in one pulse.
    // ledger: RCK-053 (game choice)
    pub aureole_share: f64,
    /// The aureole is this many times wider where the host at the intrusion's roof is a
    /// carbonate. Game choice, weakly founded: the Alta stock's aureole in dolomite reaches
    /// further than conduction alone explains (its talc isograd ~1.5 km from the contact), which
    /// is ascribed to heat carried by fluids (Brenner et al. 2021, seen only in search excerpts:
    /// from-memory); no source read compares carbonate and pelite hosts directly.
    // ledger: RCK-091 (game choice)
    pub aureole_carbonate_factor: f64,
    /// Fold-thrust belts reach this far into the foreland of an orogen, km: the Zagros fold-thrust
    /// belt has "an average width of ~300 km" in a 200–300 km wide zone of deformation (Alavi
    /// 2007, Am. J. Sci. 307, read in full); the Subandean belt was not read.
    // ledger: STR-030
    pub fold_belt_km: f64,
    /// Sediment a fold, fault-block or growth-fault closure needs, m. Game choice.
    // ledger: STR-025 (game choice)
    pub trap_min_sediment_m: f64,
    /// Salt domes: halite at least this thick … Game choice; no minimum thickness was found in a
    /// source read.
    // ledger: STR-026 (game choice)
    pub salt_min_m: f64,
    /// … under at least this overburden, m. Game choice. Salt (2,160 kg/m³) rises through
    /// sediment that compacts past it (from 2,000 to 2,500 kg/m³ with burial), but roofs
    /// "of more than several hundred meters" are not deformed by salt without regional extension
    /// or shortening (Hudec & Jackson 2007 as quoted by the EGU TS blog; the review itself was
    /// not read).
    // ledger: STR-026, STR-027 (game choice)
    pub salt_overburden_m: f64,
    /// Growth faults and rollover anticlines in sediment wedges thicker than this, m. Game
    /// choice. The Niger Delta, the model delta, is a 12-km clastic wedge detached on overpressured
    /// shale (Sanuade & Kaka 2017, review of Doust & Omatsola 1990, not read); the 1,500 m
    /// threshold has no source.
    // ledger: STR-028, STR-029 (game choice)
    pub growth_fault_sediment_m: f64,
    /// How far the bake smooths the rock layers sideways: σ of the Gaussian, in L1 cells (0 = each
    /// cell's own stack). Larger values give gentler dips (`bake_horizons::Horizons::smoothed`).
    pub strata_smoothing_cells: f64,
    /// The size of the tongues two rocks interlock in where one changes into the other, as a
    /// multiple of the column rule's default band (64 keys; `deeptime_format::strata`).
    pub strata_tongue_scale: f64,
    /// A rock's share of a layer below which it takes no part in the tongues (0–0.45).
    pub strata_minor_share: f64,
    /// Folds and faults (P2-01, `deform/params.rs`).
    pub deformation: super::deform::DeformationParams,
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
            pluton_rate_km2_myr: 1.0e-5,
            collision_pluton_factor: 0.3,
            collision_melt_after_myr: 20.0,
            collision_melt_end_myr: 80.0,
            pluton_length_km: [25.0, 8.0, 80.0],
            pluton_length_sigma: 0.5,
            pluton_roof_km: [2.0, 6.0],
            aureole_share: 0.3,
            aureole_carbonate_factor: 1.5,
            fold_belt_km: 300.0,
            trap_min_sediment_m: 500.0,
            salt_min_m: 100.0,
            salt_overburden_m: 1000.0,
            growth_fault_sediment_m: 1500.0,
            strata_smoothing_cells: DEFAULT_STRATA_SMOOTHING_CELLS,
            strata_tongue_scale: 1.0,
            strata_minor_share: deeptime_format::strata::MINOR_SHARE,
            deformation: super::deform::DeformationParams::default(),
        }
    }
}

/// The default lateral smoothing of the rock layers, L1 cells (see `strata_smoothing_cells`).
pub const DEFAULT_STRATA_SMOOTHING_CELLS: f64 = 10.0;

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
            (self.collision_melt_after_myr, "collision_melt_after_myr"),
            (self.collision_melt_end_myr, "collision_melt_end_myr"),
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
        check(
            (0.0..=16.0).contains(&self.strata_smoothing_cells),
            "strata_smoothing_cells",
            "must be in [0, 16]",
        )?;
        check(
            (0.25..=16.0).contains(&self.strata_tongue_scale),
            "strata_tongue_scale",
            "must be in [0.25, 16]",
        )?;
        check(
            (0.0..=0.45).contains(&self.strata_minor_share),
            "strata_minor_share",
            "must be in [0, 0.45]",
        )?;
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
            (0.0..=2.0).contains(&self.aureole_share),
            "aureole_share",
            "must be in [0, 2]",
        )?;
        check(
            (1.0..=5.0).contains(&self.aureole_carbonate_factor),
            "aureole_carbonate_factor",
            "must be in [1, 5]",
        )?;
        self.deformation.validate()?;
        Ok(())
    }
}
