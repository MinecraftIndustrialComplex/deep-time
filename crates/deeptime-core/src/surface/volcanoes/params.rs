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
    /// Ledger VOL-012 (game choice, Ben).
    pub density: f64,
    /// How big: multiplies every edifice's height and radius (and a caldera's). Ledger VOL-025.
    pub size: f64,
    /// How steep volcano flanks are, as a share of the real profile: every edifice's basal radius is
    /// its physical one divided by this, the height unchanged. Heights are drawn 15 times larger
    /// than widths (a block is 300 m across and 20 m tall), so a cone of real proportions (1) is a
    /// steep spire in blocks; the default 0.5 doubles the radius (Ben, 2026-09-30: "Setting,
    /// gentler default"), broad enough to read as a volcano while cones 60 km apart along an arc
    /// still stand apart. Ledger VOL-024 (game choice, Ben).
    pub flank_steepness: f64,
    /// The least distance between stratovolcanoes along an arc, km: a new cone keeps this far
    /// (× 0.8–1.2) from the cones standing near it. A game choice, not a measured spacing: arcs
    /// have no characteristic spacing (de Bremond d'Ars, Jaupart & Sparks 1995, JGR
    /// 100(B10):20421, abstract: spacings follow a Gamma distribution, as random points do). The
    /// mean spacing of large centres is 27 km on the Alaska Peninsula and 42 km in the Cascades
    /// and the Andean Southern Volcanic Zone, and 87 km between the 17 largest Cascade centres
    /// (Hildreth 2007, USGS PP 1744, tables 4 and 10). 60 km keeps the doubled cones of
    /// `flank_steepness` apart. Ledger VOL-039.
    pub arc_spacing_km: f64,
    /// Arc magma flux (the deposit engine's `arc_magma_flux`, 0–1) below which no stratovolcano
    /// forms. Ledger VOL-002.
    pub arc_min_flux: f64,
    /// Relief of a stratovolcano at the lowest and the highest arc flux, km. A design range, taller
    /// than most of Earth's: the 512 subaerial stratovolcanoes measured by Grosse et al. (2014,
    /// Bull. Volcanol. 76:784, table 1) stand 0.17–4.57 km above their bases, median 1.02 km,
    /// mean 1.13 ± 0.55 km. Ledger VOL-019.
    pub arc_height_km: [f64; 2],
    /// Basal radius of a stratovolcano at the top of the height range, km; a lower cone is narrower
    /// (R ∝ H^0.8, uncited). At 3 km over 14 km the height is 0.11 of the basal width (Earth's
    /// stratovolcanoes: median 0.13, range 0.02–0.30; Grosse et al. 2014, table 1). Ledger VOL-020.
    pub arc_radius_km: f64,
    /// How long a stratovolcano stays active, Myr [min, max] (uniform). Longer than a real cone's:
    /// most large Cascade centres lived 0.1–1 Myr (Hildreth 2007, USGS PP 1744, p. 64). A centre
    /// here stands for a volcanic complex that rebuilds itself in one place over a 2-Myr step.
    /// Ledger VOL-038 (game choice).
    pub arc_lifetime_myr: [f64; 2],
    /// A new volcano forms over a hotspot once its last one has drifted this far, km. Uncited:
    /// Hawaiian volcanoes are unevenly spaced (Clague & Sherrod 2014, USGS PP 1801 ch. 3, p. 99).
    /// Ledger VOL-007.
    pub hotspot_spacing_km: f64,
    /// Relief of a shield volcano at the weakest and strongest plume, km above the sea floor
    /// around it. Mauna Loa stands about 9 km above the sea floor (4.17 km above the sea and 5 km
    /// of submarine flank: USGS); the range is lower than Hawaii, the largest. Ledger VOL-021.
    pub shield_height_km: [f64; 2],
    /// Basal radius of a shield at the top of its range, km. Uncited. Ledger VOL-022.
    pub shield_radius_km: f64,
    /// Relief and basal radius of a continental hotspot's silicic centre, km. Yellowstone's
    /// plateau averages about 2,400 m against 1,600–1,900 m for Island Park and the eastern Snake
    /// River Plain beside it (Christiansen 2001, USGS PP 729-G, p. G6). Ledger VOL-023.
    pub silicic_height_km: f64,
    pub silicic_radius_km: f64,
    /// Extinct volcanoes on oceanic crust subside by this much per Myr, m/Myr, … Not a measured
    /// rate: Hawaiian volcanoes sink more than 1 km in about 1 Myr under their own load while they
    /// grow, and then about 10 m/Myr as the plate under them cools (Clague & Sherrod 2014, USGS PP
    /// 1801 ch. 3, p. 130). The load part is now the surface stage's flexure (cones are loads) and
    /// the cooling part the plate model's ocean floor, so this counts both twice; a question for
    /// Ben. Ledger VOL-048.
    pub island_subsidence_m_myr: f64,
    /// … up to this much in all, m. Ledger VOL-048.
    pub island_subsidence_max_m: f64,
    /// Caldera-forming eruptions per Myr of a silicic volcano (a stratovolcano on continental
    /// crust at least `silicic_crust_km` thick); continental hotspot centres collapse three times
    /// as often, and a shield grown past 4 km opens a summit caldera at this rate. Uncited;
    /// Yellowstone had three caldera-forming eruptions between 2.1 and 0.64 Ma (Christiansen
    /// 2001). Ledger VOL-045.
    pub caldera_rate_per_myr: f64,
    /// Caldera radius, km [min, max]: Crater Lake's is 4–5 (8 × 10 km across), Long Valley's
    /// 8–16 (16 × 32 km: USGS), Yellowstone's 22–43 (45 × 85 km: Christiansen 2001). The default
    /// spans Crater Lake to Long Valley's short axis and leaves out the largest. Ledger VOL-029.
    pub caldera_radius_km: [f64; 2],
    /// Continental crust at least this thick makes silicic (dacite–rhyolite) magma, km. Uncited;
    /// the Central Andes' ignimbrite flare-up sits on 60–70 km crust (secondary). Ledger VOL-005.
    pub silicic_crust_km: f64,
    /// Lava flows of a flood-basalt pile, each over a thin tuff bed (the red boles between
    /// Deccan flows): the steps a plateau erodes into. Ledger VOL-052 (game choice).
    pub flood_basalt_flows: u32,
    /// Height of the steps the finished terrain shows where flood basalts crop out, m (a step of
    /// 80 m is 4 blocks at 20 m per block; real flow thicknesses not checked). Ledger VOL-036.
    pub flood_basalt_step_m: f64,
    /// A volcano builds toward its target height with this time constant, Myr (Hawaiian shields
    /// do most of their growth in their first 1–2 Myr: Clague & Sherrod 2014). Ledger VOL-041.
    pub build_myr: f64,
    /// After a volcano dies, its crater and the sharpness of its summit fade with this time
    /// constant (the finest landforms erode first); the cone's shape fades three times slower,
    /// Myr. Ledger VOL-065 (game choice).
    pub fade_myr: f64,
    /// No summit rises above this elevation (above the sea of the time), km; shields to 0.65 of
    /// it. Earth's highest volcano, Ojos del Salado, is about 6.9 km (secondary); Mauna Loa's
    /// summit is 4.17 km (USGS). Ledger VOL-032.
    pub max_elevation_km: f64,
}

impl Default for VolcanoParams {
    fn default() -> Self {
        VolcanoParams {
            density: 1.0,                    // ledger: VOL-012
            size: 1.0,                       // ledger: VOL-025
            flank_steepness: 0.5,            // ledger: VOL-024
            arc_spacing_km: 60.0,            // ledger: VOL-039
            arc_min_flux: 0.25,              // ledger: VOL-002
            arc_height_km: [1.2, 3.0],       // ledger: VOL-019
            arc_radius_km: 14.0,             // ledger: VOL-020
            arc_lifetime_myr: [2.0, 8.0],    // ledger: VOL-038
            hotspot_spacing_km: 70.0,        // ledger: VOL-007
            shield_height_km: [3.5, 6.5],    // ledger: VOL-021
            shield_radius_km: 40.0,          // ledger: VOL-022
            silicic_height_km: 0.9,          // ledger: VOL-023
            silicic_radius_km: 35.0,         // ledger: VOL-023
            island_subsidence_m_myr: 25.0,   // ledger: VOL-048
            island_subsidence_max_m: 1500.0, // ledger: VOL-048
            caldera_rate_per_myr: 0.04,      // ledger: VOL-045
            caldera_radius_km: [3.0, 9.0],   // ledger: VOL-029
            silicic_crust_km: 42.0,          // ledger: VOL-005
            flood_basalt_flows: 8,           // ledger: VOL-052
            flood_basalt_step_m: 80.0,       // ledger: VOL-036
            build_myr: 1.0,                  // ledger: VOL-041
            fade_myr: 5.0,                   // ledger: VOL-065
            max_elevation_km: 6.8,           // ledger: VOL-032
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
                Err(bad(
                    k,
                    &format!("must be [min, max] with lo ≤ min ≤ max ≤ hi = [{lo}, {hi}]"),
                ))
            }
        };
        range(self.density, 0.0, 5.0, "density")?;
        range(self.size, 0.2, 3.0, "size")?;
        range(self.flank_steepness, 0.1, 1.0, "flank_steepness")?;
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
