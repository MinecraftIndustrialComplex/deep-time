// SPDX-License-Identifier: LGPL-3.0-or-later
//! The preset's `volcanoes` section (P1-21): every key optional, the defaults below. A planet
//! preset with a `surface` section and this section runs the volcano stage (`{}` = the defaults).
//! Every key is a setting (decisions log 2026-09-29: world-shaping knobs are settings); the
//! editor shows `density` and `size` under Terrain and the rest under Advanced · Volcanoes.
//!
//! Lengths are block-metric kilometres (the world's own: 300 m per block at the built-in scale),
//! not the plate model's Earth-normalized ones: a volcano is a physical landform whose slopes do
//! not shrink with the planet. Heights are metres of relief above the ground around the edifice.
//!
//! ```json
//! "volcanoes": {"density": 1.0, "size": 1.0}
//! ```

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// The volcanoes section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct VolcanoParams {
    /// How many volcanoes: divides the spacing along arcs and hotspot tracks (2 = twice as many).
    pub density: f64,
    /// How big: multiplies every edifice's height and radius (and a caldera's).
    pub size: f64,
    /// Spacing of stratovolcanoes along an arc, km. Real volcanic fronts space their centres
    /// 50–70 km apart (de Bremond d'Ars, Jaupart & Sparks 1995, JGR 100(B10):20421; the Andes,
    /// Cascades, Aleutians, Japan).
    pub arc_spacing_km: f64,
    /// Arc magma flux (the deposit engine's `arc_magma_flux`, 0–1) below which no stratovolcano
    /// forms.
    pub arc_min_flux: f64,
    /// Relief of a stratovolcano at the lowest and the highest arc flux, km. Design range: most
    /// arc cones stand 1–3 km above their surroundings (Fuji 3.8 km a.s.l. on a low base;
    /// Cotopaxi ~2.9 km above the Andean plateau; Shasta ~3 km above its base).
    pub arc_height_km: [f64; 2],
    /// Basal radius of a stratovolcano at the top of the height range, km (Fuji ~18, Rainier ~10,
    /// Shasta ~14); a lower cone is narrower (R ∝ H^0.8).
    pub arc_radius_km: f64,
    /// How long a stratovolcano stays active, Myr [min, max] (uniform). Longer than a real cone's
    /// 0.1–1 Myr: a centre here stands for a volcanic complex that rebuilds itself in one place
    /// over a 2-Myr step.
    pub arc_lifetime_myr: [f64; 2],
    /// A new volcano forms over a hotspot once its last one has drifted this far, km (Hawaiian
    /// volcanoes line up ~40–80 km apart along the chain).
    pub hotspot_spacing_km: f64,
    /// Relief of a shield volcano at the weakest and strongest plume, km above the sea floor
    /// around it (Hawaii's shields 5–9 km above the sea floor, Réunion ~7, Galápagos 3–4).
    pub shield_height_km: [f64; 2],
    /// Basal radius of a shield at the top of its range, km (Mauna Loa ~60, Mauna Kea ~40).
    pub shield_radius_km: f64,
    /// Relief and basal radius of a continental hotspot's silicic centre, km (Yellowstone's
    /// plateau stands ~1 km above the Snake River Plain over ~70 km).
    pub silicic_height_km: f64,
    pub silicic_radius_km: f64,
    /// Extinct volcanoes on oceanic crust subside by this much per Myr (lithospheric flexure and
    /// cooling; Hawaiian islands sink ~2 km over ~30 Myr beyond the swell the plate model makes),
    /// m/Myr, …
    pub island_subsidence_m_myr: f64,
    /// … up to this much in all, m.
    pub island_subsidence_max_m: f64,
    /// Caldera-forming eruptions per Myr of a silicic volcano (a stratovolcano on continental
    /// crust at least `silicic_crust_km` thick); continental hotspot centres collapse three times
    /// as often, and a shield grown past 4 km opens a summit caldera at this rate.
    pub caldera_rate_per_myr: f64,
    /// Caldera radius, km [min, max] (Crater Lake 4–5, Long Valley 8–16, Yellowstone 30–45 across).
    pub caldera_radius_km: [f64; 2],
    /// Continental crust at least this thick makes silicic (dacite–rhyolite) magma, km (the
    /// Central Andes' ignimbrite flare-up sits on 60–70 km crust; normal crust is ~35).
    pub silicic_crust_km: f64,
    /// Lava flows of a flood-basalt pile, each over a thin tuff bed (the red boles between
    /// Deccan flows): the steps a plateau erodes into.
    pub flood_basalt_flows: u32,
    /// Height of the steps the finished terrain shows where flood basalts crop out, m (Deccan
    /// flows are 10–100 m thick; a step of 80 m is 4 blocks at 20 m per block).
    pub flood_basalt_step_m: f64,
    /// A volcano builds toward its target height with this time constant, Myr.
    pub build_myr: f64,
    /// After a volcano dies, its crater and the sharpness of its summit fade with this time
    /// constant (the finest landforms erode first); the cone's shape fades three times slower,
    /// Myr.
    pub fade_myr: f64,
    /// No summit rises above this elevation (above the sea of the time), km (Ojos del Salado
    /// 6.9 km; Mauna Kea 4.2).
    pub max_elevation_km: f64,
}

impl Default for VolcanoParams {
    fn default() -> Self {
        VolcanoParams {
            density: 1.0,
            size: 1.0,
            arc_spacing_km: 60.0,
            arc_min_flux: 0.25,
            arc_height_km: [1.2, 3.0],
            arc_radius_km: 14.0,
            arc_lifetime_myr: [2.0, 8.0],
            hotspot_spacing_km: 70.0,
            shield_height_km: [3.5, 6.5],
            shield_radius_km: 40.0,
            silicic_height_km: 0.9,
            silicic_radius_km: 35.0,
            island_subsidence_m_myr: 25.0,
            island_subsidence_max_m: 1500.0,
            caldera_rate_per_myr: 0.04,
            caldera_radius_km: [3.0, 9.0],
            silicic_crust_km: 42.0,
            flood_basalt_flows: 8,
            flood_basalt_step_m: 80.0,
            build_myr: 1.0,
            fade_myr: 5.0,
            max_elevation_km: 6.8,
        }
    }
}

impl VolcanoParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("$.volcanoes.{k}"), why);
        let range = |v: f64, lo: f64, hi: f64, k: &str| {
            if v.is_finite() && v >= lo && v <= hi {
                Ok(())
            } else {
                Err(bad(k, &format!("must be in [{lo}, {hi}]")))
            }
        };
        let pair = |v: [f64; 2], lo: f64, hi: f64, k: &str| {
            if v.iter().all(|x| x.is_finite() && *x >= lo && *x <= hi) && v[0] <= v[1] {
                Ok(())
            } else {
                Err(bad(k, &format!("must be [min, max] with lo ≤ min ≤ max ≤ hi = [{lo}, {hi}]")))
            }
        };
        range(self.density, 0.0, 5.0, "density")?;
        range(self.size, 0.2, 3.0, "size")?;
        range(self.arc_spacing_km, 10.0, 500.0, "arc_spacing_km")?;
        range(self.arc_min_flux, 0.0, 1.0, "arc_min_flux")?;
        pair(self.arc_height_km, 0.1, 6.0, "arc_height_km")?;
        range(self.arc_radius_km, 2.0, 60.0, "arc_radius_km")?;
        pair(self.arc_lifetime_myr, 0.5, 100.0, "arc_lifetime_myr")?;
        range(self.hotspot_spacing_km, 10.0, 500.0, "hotspot_spacing_km")?;
        pair(self.shield_height_km, 0.1, 10.0, "shield_height_km")?;
        range(self.shield_radius_km, 5.0, 150.0, "shield_radius_km")?;
        range(self.silicic_height_km, 0.0, 5.0, "silicic_height_km")?;
        range(self.silicic_radius_km, 5.0, 150.0, "silicic_radius_km")?;
        range(
            self.island_subsidence_m_myr,
            0.0,
            1000.0,
            "island_subsidence_m_myr",
        )?;
        range(
            self.island_subsidence_max_m,
            0.0,
            10_000.0,
            "island_subsidence_max_m",
        )?;
        range(self.caldera_rate_per_myr, 0.0, 10.0, "caldera_rate_per_myr")?;
        pair(self.caldera_radius_km, 0.5, 60.0, "caldera_radius_km")?;
        range(self.silicic_crust_km, 0.0, 100.0, "silicic_crust_km")?;
        if self.flood_basalt_flows == 0 || self.flood_basalt_flows > 12 {
            return Err(bad("flood_basalt_flows", "must be in [1, 12]"));
        }
        range(self.flood_basalt_step_m, 0.0, 500.0, "flood_basalt_step_m")?;
        range(self.build_myr, 0.01, 100.0, "build_myr")?;
        range(self.fade_myr, 0.01, 1000.0, "fade_myr")?;
        range(self.max_elevation_km, 0.5, 12.0, "max_elevation_km")?;
        Ok(())
    }
}
