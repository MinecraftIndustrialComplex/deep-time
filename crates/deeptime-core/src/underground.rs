// SPDX-License-Identifier: LGPL-3.0-or-later
//! The `underground` section of a preset: where the game has lava. The bake records the section in
//! `notes.bake.underground.params` and writes the active volcanoes to `volcanoes/active.json`; the
//! game reads both when it generates chunks (Java `worldgen/LavaRules`, `worldgen/MagmaPass`).
//!
//! **The lava floor and lava lakes.** Vanilla fills every open block below y −54 with lava and
//! lets a cave lake at y −10 or lower be lava. Both heights are hard-coded for a world whose floor
//! is y −64: they are floor + 10 and floor + 54. In a Deep Time world (floor y −352 by default)
//! they would flood the lowest 298 blocks of every cave and of every stretch of sea deeper than
//! 117 blocks, so the two distances are settings here and the game measures them from the world's
//! floor.
//!
//! **Lava where the simulation has it** (Ben, decisions log 2026-10-01, "Volcanoes is a go
//! ahead"). Every volcano the volcano stage leaves active at the present gets a magma chamber
//! under it, a conduit up to its crater and, at the most active ones standing on land, a crater
//! lava lake. A dead volcano gets nothing: its chamber froze into the plutons the structure stage
//! already drew. The game builds the bodies from [`active_volcanoes`]'s list and these settings;
//! FORMAT.md ("Lava") has the geometry. The random lava lakes and lava springs of vanilla's and
//! the pack's biomes are removed from Deep Time worlds (`remove_lava_features`).
//!
//! **The crust-deep column** (Ben, decisions log 2026-10-02, "Crust-deep magma system"). Under the
//! chamber each active volcano gets stacked, sill-like lava lenses joined by a conduit, set in a body
//! of altered hot rock, down to a basal lens on the lava floor, so the system does not end a few
//! hundred blocks down. Four settings shape it: `magma_lens_count`, `magma_lens_spacing_blocks`,
//! `magma_lens_width` and `magma_mush_width`; the game builds it (Java `worldgen/Magma`).
//!
//! **Geothermal features** (P2-14) sit around the same list: hot springs and terraces where the
//! baked climate has groundwater, fumaroles and altered ground where it is dry or frozen. Four
//! settings here shape them; the game places them (Java `worldgen/Geothermal`).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::CoreError;
use crate::surface::volcanoes::VolcanoState;

/// The largest distance accepted, blocks (the default build range is 992 blocks tall).
pub const MAX_BLOCKS: u32 = 992;

/// The most lenses a volcano's crust-deep column may hold (`magma_lens_count`).
pub const MAX_LENSES: u32 = 64;

/// The world-data file of the active volcanoes.
pub const VOLCANOES_PATH: &str = "volcanoes/active.json";
/// The version of that file.
pub const VOLCANOES_VERSION: u32 = 1;

/// The `underground` section of a preset (every key optional).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct UndergroundParams {
    /// Caves and any other open space less than this many blocks above the world's floor are
    /// filled with lava (vanilla: 10). 0: no lava floor. Under a sea bed that deep the lava stays
    /// eight blocks below the bed, so the sea is water all the way down.
    pub lava_floor_blocks: u32,
    /// Cave lakes may be lava where their 40-block level cell is centred no more than this many
    /// blocks above the world's floor (vanilla: 54); higher lakes are always water. 0: no lava
    /// lakes.
    pub lava_lake_blocks: u32,
    /// Depth of the top of an active volcano's magma chamber below its crater floor, km. Real
    /// shallow reservoirs lie 1–7 km down under Hawaiian shields (Clague & Sherrod 2014, USGS PP
    /// 1801 ch. 3), 1–2 km under Kīlauea's caldera and 2–4 km under its south part (Poland et al.
    /// 2014, PP 1801 ch. 5); under arc volcanoes geophysics finds them at 6 ± 3 km (Rasmussen et
    /// al. 2022, abstract). At 20 m per block 3 km is 150 blocks. The game raises a chamber that
    /// would reach the lava floor. Ledger MAG-002.
    pub magma_chamber_depth_km: f64,
    /// Multiplies every magma chamber's size (its radius is a tenth of the volcano's basal
    /// radius, between 1.2 and 4.5 km, and its half-height a fifth of that radius). 0: no magma
    /// under volcanoes at all (no chambers, conduits or crater lakes). Ledger MAG-006.
    pub magma_chamber_size: f64,
    /// The share of the active volcanoes standing on land that hold a crater lava lake, the most
    /// active first. Persistent lava lakes are rare on Earth: a handful of volcanoes (eight in a
    /// secondary list) against about 1,350 potentially active ones (USGS); the default is far
    /// above that, because a player should be able to find one. Ledger MAG-019, MAG-020.
    pub crater_lake_share: f64,
    /// The most stacked lava lenses a volcano's crust-deep column holds below its chamber (Ben,
    /// decisions log 2026-10-02): current volcanology describes a vertically extensive system of
    /// crystal mush with melt lenses from the shallow chamber down through the crust (Cashman,
    /// Sparks and Blundy 2017; Sparks et al. 2019), and the game draws it down to the lava floor.
    /// The lenses are spread evenly between the chamber and the floor, `magma_lens_spacing_blocks`
    /// apart; fewer than this are drawn where the ground is thin, and where the count would stop the
    /// column short of the floor the spacing widens instead. 0: no column (the chamber and its
    /// conduit only). Ledger MAG-043.
    pub magma_lens_count: u32,
    /// Vertical distance between the lenses of the crust-deep column, blocks (20 m each by default,
    /// so 28 is 560 m). Ledger MAG-043.
    pub magma_lens_spacing_blocks: f64,
    /// Multiplies every lens's radius (the top lens is about as wide as the chamber, the lowest 2.2
    /// times wider; shields' are wider, quiet volcanoes' narrower). 0: no column. Ledger MAG-045.
    pub magma_lens_width: f64,
    /// Multiplies the radius of the body of altered hot rock (the mush) around the column; 0: none,
    /// the lenses sit in the ordinary rock. Ledger MAG-054.
    pub magma_mush_width: f64,
    /// Whether the random lava lakes and lava springs of vanilla's and the pack's biomes are taken
    /// out of a Deep Time world, so lava is found at the volcanoes and at the world's floor only.
    pub remove_lava_features: bool,
    /// How many geothermal fields (hot springs, sinter and travertine terraces, fumaroles,
    /// altered ground) each active volcano gets, as a multiple of the default (P2-14): the
    /// count is floor(density × (0.7 + 2.3 activity) × clamp(sqrt(R / 45), 0.6, 2.2) + u), from none
    /// at a small, quiet volcano to seven at a large, active one, more at the more active and
    /// larger volcanoes (ledger: GEO-001, GEO-014). 0: no geothermal features anywhere.
    pub geothermal_density: f64,
    /// How far from a volcano's summit its fields may lie, in basal radii. The fields cluster
    /// toward the summit and the lower flanks; 1 keeps them on the edifice.
    pub geothermal_reach: f64,
    /// Multiplies the size of every geothermal feature (a pool's radius, the terrace around it,
    /// a fumarole's field of altered ground).
    pub geothermal_size: f64,
    /// A field is a field of hot springs only where the year's rain (mm, from the baked
    /// climate) reaches this much and the mean temperature is at least 3 °C; drier or colder ground gets
    /// fumaroles and bare altered ground instead. 400 mm is a game choice (ledger: GEO-007).
    pub geothermal_spring_min_precip_mm: f64,
}

impl Default for UndergroundParams {
    fn default() -> Self {
        UndergroundParams {
            lava_floor_blocks: 10,                  // ledger: MAG-027
            lava_lake_blocks: 54,                   // ledger: MAG-030
            magma_chamber_depth_km: 3.0,            // ledger: MAG-002
            magma_chamber_size: 1.0,                // ledger: MAG-006
            crater_lake_share: 0.25,                // ledger: MAG-019
            magma_lens_count: 24,                   // ledger: MAG-043
            magma_lens_spacing_blocks: 28.0,        // ledger: MAG-043
            magma_lens_width: 1.0,                  // ledger: MAG-045
            magma_mush_width: 1.0,                  // ledger: MAG-054
            remove_lava_features: true,             // ledger: MAG-034
            geothermal_density: 1.0,                // ledger: GEO-001 (game-choice)
            geothermal_reach: 1.0,                  // ledger: GEO-003 (game-choice)
            geothermal_size: 1.0,                   // ledger: GEO-005 (game-choice)
            geothermal_spring_min_precip_mm: 400.0, // ledger: GEO-007 (game-choice)
        }
    }
}

impl UndergroundParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.magma_lens_count > MAX_LENSES {
            return Err(CoreError::preset(
                "$.underground.magma_lens_count".to_string(),
                format!("must be in [0, {MAX_LENSES}]"),
            ));
        }
        for (v, key) in [
            (self.lava_floor_blocks, "lava_floor_blocks"),
            (self.lava_lake_blocks, "lava_lake_blocks"),
        ] {
            if v > MAX_BLOCKS {
                return Err(CoreError::preset(
                    format!("$.underground.{key}"),
                    format!("must be in [0, {MAX_BLOCKS}]"),
                ));
            }
        }
        for (v, lo, hi, key) in [
            (
                self.magma_chamber_depth_km,
                0.5,
                15.0,
                "magma_chamber_depth_km",
            ),
            (self.magma_chamber_size, 0.0, 4.0, "magma_chamber_size"),
            (self.crater_lake_share, 0.0, 1.0, "crater_lake_share"),
            (
                self.magma_lens_spacing_blocks,
                8.0,
                200.0,
                "magma_lens_spacing_blocks",
            ),
            (self.magma_lens_width, 0.0, 4.0, "magma_lens_width"),
            (self.magma_mush_width, 0.0, 4.0, "magma_mush_width"),
            (self.geothermal_density, 0.0, 4.0, "geothermal_density"),
            (self.geothermal_reach, 0.25, 3.0, "geothermal_reach"),
            (self.geothermal_size, 0.5, 3.0, "geothermal_size"),
            (
                self.geothermal_spring_min_precip_mm,
                0.0,
                4000.0,
                "geothermal_spring_min_precip_mm",
            ),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                return Err(CoreError::preset(
                    format!("$.underground.{key}"),
                    format!("must be in [{lo}, {hi}]"),
                ));
            }
        }
        Ok(())
    }
}

/// A volcano's activity, 0–1: the arc magma flux of an arc volcano's last step, the mean plume
/// proximity of a hotspot's.
fn activity(v: &crate::surface::volcanoes::Volcano) -> f64 {
    let a = if v.hotspot.is_some() {
        if v.active_myr > 0.0 {
            v.flux_myr / v.active_myr
        } else {
            0.5
        }
    } else {
        v.flux_last
    };
    if a.is_finite() {
        a.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// `volcanoes/active.json`: the volcanoes active at the present, in id order (FORMAT.md, "Lava").
/// Dead volcanoes and those that left the catalogue are not listed.
pub fn active_volcanoes(st: &VolcanoState) -> Value {
    let r3 = |v: f64| (v * 1000.0).round() / 1000.0;
    let mut list: Vec<&crate::surface::volcanoes::Volcano> = st
        .volcanoes
        .iter()
        .filter(|v| v.active() && v.gone.is_none() && v.radius_m > 0.0)
        .collect();
    list.sort_by_key(|v| v.id);
    let volcanoes: Vec<Value> = list
        .iter()
        .map(|v| {
            let crater = v.depression.map(|d| {
                json!({
                    "radius_m": (d.rho * v.radius_m).round(),
                    "depth_m": (d.depth * v.height_m).round(),
                    "caldera": d.caldera,
                })
            });
            json!({
                "id": v.id,
                "kind": v.kind.name(),
                "x": r3(v.x),
                "z": r3(v.z),
                "radius_m": v.radius_m.round(),
                "height_m": v.height_m.round(),
                "activity": r3(activity(v)),
                "oceanic": v.oceanic,
                "silicic": v.silicic,
                "age_myr": r3((st.t_myr - v.born_myr).max(0.0)),
                "crater": crater,
            })
        })
        .collect();
    json!({
        "version": VOLCANOES_VERSION,
        "t_myr": st.t_myr,
        "volcanoes": volcanoes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::volcanoes::shape::{Depression, Kind};
    use crate::surface::volcanoes::{Gone, Volcano};

    #[test]
    fn defaults_are_vanillas_heights_above_the_floor() {
        // Vanilla: floor −64, lava below −54, lava lakes at −10 and lower.
        let d = UndergroundParams::default();
        assert_eq!(-64 + d.lava_floor_blocks as i32, -54);
        assert_eq!(-64 + d.lava_lake_blocks as i32, -10);
        d.validate().unwrap();
    }

    fn volcano(id: u32, kind: Kind) -> Volcano {
        Volcano {
            id,
            kind,
            x: -100.5,
            z: 42.25,
            plate: 1,
            oceanic: kind == Kind::Shield,
            silicic: false,
            hotspot: (kind != Kind::Strato).then_some(3),
            chain: 0,
            born_myr: -4.0,
            dies_myr: f64::INFINITY,
            extinct_myr: None,
            radius_m: 20_000.0,
            height_m: 2_000.0,
            peak_m: 2_000.0,
            built_m: 2_000.0,
            sunk_m: 0.0,
            kept_m: 2_000.0,
            target_m: 2_000.0,
            flux_last: 0.6,
            missed: 0,
            flux_myr: 3.0,
            active_myr: 4.0,
            depression: Some(Depression {
                rho: 0.07,
                depth: 0.06,
                caldera: false,
            }),
            caldera: None,
            cause: None,
            segment: 0,
            gone: None,
        }
    }

    #[test]
    fn only_the_volcanoes_active_at_the_present_are_listed() {
        let mut dead = volcano(2, Kind::Strato);
        dead.extinct_myr = Some(-1.0);
        let mut eroded = volcano(3, Kind::Strato);
        eroded.gone = Some(Gone::Eroded);
        let mut plain = volcano(1, Kind::Shield);
        plain.depression = None;
        let st = VolcanoState {
            volcanoes: vec![volcano(7, Kind::Strato), dead, eroded, plain],
            t_myr: 0.0,
            ..VolcanoState::default()
        };
        let v = active_volcanoes(&st);
        assert_eq!(v["version"], VOLCANOES_VERSION);
        let list = v["volcanoes"].as_array().unwrap();
        let ids: Vec<u64> = list.iter().map(|e| e["id"].as_u64().unwrap()).collect();
        assert_eq!(ids, [1, 7], "in id order, the dead and the eroded left out");
        // The shield: a hotspot's activity is its mean plume proximity (3 / 4), and it has no crater.
        assert_eq!(list[0]["kind"], "shield");
        assert_eq!(list[0]["activity"], 0.75);
        assert!(list[0]["crater"].is_null());
        assert_eq!(list[0]["oceanic"], true);
        // The arc volcano: its last flux, and its crater in metres (7 % of 20 km, 6 % of 2 km).
        assert_eq!(list[1]["kind"], "stratovolcano");
        assert_eq!(list[1]["activity"], 0.6);
        assert_eq!(list[1]["x"], -100.5);
        assert_eq!(list[1]["crater"]["radius_m"], 1400.0);
        assert_eq!(list[1]["crater"]["depth_m"], 120.0);
        assert_eq!(list[1]["age_myr"], 4.0);
    }

    #[test]
    fn a_section_may_set_one_key_and_rejects_unknown_ones() {
        let p: UndergroundParams = serde_json::from_str(r#"{"lava_lake_blocks": 0}"#).unwrap();
        assert_eq!(p.lava_floor_blocks, 10);
        assert_eq!(p.lava_lake_blocks, 0);
        assert!(p.remove_lava_features);
        assert!(serde_json::from_str::<UndergroundParams>(r#"{"lava": 1}"#).is_err());
        assert!(serde_json::from_str::<UndergroundParams>(r#"{"lava_floor_blocks": -1}"#).is_err());
        let e = UndergroundParams {
            lava_floor_blocks: 993,
            ..UndergroundParams::default()
        }
        .validate()
        .unwrap_err();
        assert!(
            e.to_string().contains("$.underground.lava_floor_blocks"),
            "{e}"
        );
        let e = UndergroundParams {
            geothermal_density: -0.5,
            ..UndergroundParams::default()
        }
        .validate()
        .unwrap_err();
        assert!(
            e.to_string().contains("$.underground.geothermal_density"),
            "{e}"
        );
        let none: UndergroundParams = serde_json::from_str(r#"{"geothermal_density": 0}"#).unwrap();
        none.validate().unwrap();
        assert_eq!(none.geothermal_reach, 1.0);
        let e = UndergroundParams {
            crater_lake_share: 1.5,
            ..UndergroundParams::default()
        }
        .validate()
        .unwrap_err();
        assert!(
            e.to_string().contains("$.underground.crater_lake_share"),
            "{e}"
        );
    }

    #[test]
    fn the_crust_deep_column_has_defaults_and_ranges() {
        let d = UndergroundParams::default();
        assert_eq!(d.magma_lens_count, 24);
        assert_eq!(d.magma_lens_spacing_blocks, 28.0);
        assert_eq!(d.magma_lens_width, 1.0);
        assert_eq!(d.magma_mush_width, 1.0);
        // A section that names none of them keeps the column.
        let p: UndergroundParams = serde_json::from_str(r#"{"crater_lake_share": 0.5}"#).unwrap();
        assert_eq!(p.magma_lens_count, 24);
        // Zero is "no column" / "no altered rock", not an error.
        let none: UndergroundParams =
            serde_json::from_str(r#"{"magma_lens_count": 0, "magma_mush_width": 0}"#).unwrap();
        none.validate().unwrap();
        for (bad, key) in [
            (
                UndergroundParams {
                    magma_lens_count: MAX_LENSES + 1,
                    ..d
                },
                "magma_lens_count",
            ),
            (
                UndergroundParams {
                    magma_lens_spacing_blocks: 7.0,
                    ..d
                },
                "magma_lens_spacing_blocks",
            ),
            (
                UndergroundParams {
                    magma_lens_width: 4.5,
                    ..d
                },
                "magma_lens_width",
            ),
            (
                UndergroundParams {
                    magma_mush_width: -1.0,
                    ..d
                },
                "magma_mush_width",
            ),
        ] {
            let e = bad.validate().unwrap_err();
            assert!(
                e.to_string().contains(&format!("$.underground.{key}")),
                "{e}"
            );
        }
    }
}
