// SPDX-License-Identifier: LGPL-3.0-or-later
//! The climate at a block (FORMAT.md "The climate at a block", P1-25): what Deep Time's public
//! climate API (`com.minecraftindustrialcomplex.deeptime.api.climate`) answers, evaluated from a
//! baked world's L1 climate layers. The Java reader (`worlddata.ClimateLayers`) implements the
//! same arithmetic in the same order; `deeptime parity` writes this evaluator's answers to
//! `parity/climate.json` and `ClimateParityTest` holds the Java side to them.
//!
//! Per block (x, y, z), x folded into [−C/2, C/2) and sampled at (x, z) in block space:
//!
//! - each month m's sea-level temperature `T_sl[m]`: `climate/month/t_sl_c_{m+1}`, bilinear;
//! - the height above the sea `e = max(0, y − sea_level_y) · m_per_block_v` (metres) and the
//!   lapse `L = lapse_c_per_km · e / 1000` (`notes.bake.climate.monthly.lapse_c_per_km`);
//! - `T[m] = T_sl[m] − L`; the annual mean is Σ T[m] / 12 (m ascending), the warmest and coldest
//!   month the first maximum and minimum;
//! - the annual precipitation `P = climate/p_mm`, bilinear (not lapsed: the simulation's
//!   orographic rain is already in it); each month's share `s[m] = climate/month/p_share_{m+1}`,
//!   bilinear, normalised by their sum S to `12 · s[m] / S` (1 each when S = 0); the month's
//!   precipitation `P · s[m] / 12` mm;
//! - the Köppen class `climate/koppen`, nearest, named by `notes.bake.climate.koppen_codes`.

use std::path::Path;

use serde_json::{Value, json};

use crate::error::{FormatError, Result};
use crate::lazy::LazyLayer;
use crate::manifest::Manifest;
use crate::raster::RasterRead;

/// Köppen index of the sea (`notes.bake.climate.koppen_codes[31]`).
pub const SEA: usize = 31;

/// Layer name of month `m`'s (0–11) sea-level temperature.
pub fn t_sl_layer(m: usize) -> String {
    format!("climate/month/t_sl_c_{:02}", m + 1)
}

/// Layer name of month `m`'s (0–11) precipitation share.
pub fn p_share_layer(m: usize) -> String {
    format!("climate/month/p_share_{:02}", m + 1)
}

/// A baked world's climate layers, opened for block queries.
pub struct ClimateWorld {
    c: i64,
    sea_level_y: i32,
    m_per_block_v: f64,
    lapse_c_per_km: f64,
    t_sl: Vec<LazyLayer>,
    p_share: Vec<LazyLayer>,
    p_mm: LazyLayer,
    koppen: LazyLayer,
    codes: Vec<String>,
}

/// The climate at one block.
#[derive(Clone, Debug, PartialEq)]
pub struct ClimateSample {
    pub x: i64,
    pub y: i32,
    pub z: i64,
    /// Height above the sea the lapse rate is applied over, m.
    pub elevation_m: f64,
    /// Each month's mean temperature at the block's height, °C (month 0 ≈ January).
    pub t_month: [f64; 12],
    pub t_mean: f64,
    pub t_warm: f64,
    pub t_cold: f64,
    pub warmest: usize,
    pub coldest: usize,
    /// Annual precipitation, mm.
    pub p_annual: f64,
    /// Each month's precipitation, mm.
    pub p_month: [f64; 12],
    /// Köppen class index and code (`sea` at sea).
    pub koppen: usize,
    pub koppen_code: String,
}

impl ClimateWorld {
    /// Opens the climate layers of `dir`; `Ok(None)` when the world has no twelve months (a
    /// synthetic world, or a run without a climate section).
    pub fn open(dir: &Path) -> Result<Option<ClimateWorld>> {
        let manifest = Manifest::read(dir)?;
        Self::from_manifest(dir, &manifest)
    }

    /// As [`ClimateWorld::open`], with the manifest already read.
    pub fn from_manifest(dir: &Path, manifest: &Manifest) -> Result<Option<ClimateWorld>> {
        let monthly = manifest
            .notes
            .get("bake")
            .and_then(|b| b.get("climate"))
            .and_then(|c| c.get("monthly"))
            .filter(|m| m.is_object());
        let Some(monthly) = monthly else {
            return Ok(None);
        };
        let layer = |name: &str| -> Result<LazyLayer> {
            let e = manifest.layer(name).ok_or_else(|| {
                FormatError::Invalid(format!(
                    "{}: notes.bake.climate.monthly is set but the layer {name} is missing",
                    dir.display()
                ))
            })?;
            LazyLayer::open(&dir.join(&e.path))
        };
        let lapse_c_per_km = monthly["lapse_c_per_km"].as_f64().ok_or_else(|| {
            FormatError::Invalid(format!(
                "{}: notes.bake.climate.monthly.lapse_c_per_km is not a number",
                dir.display()
            ))
        })?;
        let codes = manifest.notes["bake"]["climate"]["koppen_codes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|c| c.as_str().unwrap_or("?").to_owned())
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some(ClimateWorld {
            c: i64::from(manifest.scale.circumference_blocks),
            sea_level_y: manifest.scale.sea_level_y,
            m_per_block_v: manifest.scale.m_per_block_v,
            lapse_c_per_km,
            t_sl: (0..12)
                .map(|m| layer(&t_sl_layer(m)))
                .collect::<Result<_>>()?,
            p_share: (0..12)
                .map(|m| layer(&p_share_layer(m)))
                .collect::<Result<_>>()?,
            p_mm: layer("climate/p_mm")?,
            koppen: layer("climate/koppen")?,
            codes,
        }))
    }

    /// The lapse rate the world was simulated with, °C/km.
    pub fn lapse_c_per_km(&self) -> f64 {
        self.lapse_c_per_km
    }

    /// The climate at block (x, y, z).
    pub fn sample(&self, x: i64, y: i32, z: i64) -> ClimateSample {
        let wx = (x + self.c / 2).rem_euclid(self.c) - self.c / 2;
        let (bx, bz) = (wx as f64, z as f64);
        let elevation_m = f64::from((y - self.sea_level_y).max(0)) * self.m_per_block_v;
        let lapse = self.lapse_c_per_km * elevation_m / 1000.0;
        let mut t_month = [0.0; 12];
        let mut share = [0.0; 12];
        let mut s = 0.0;
        for m in 0..12 {
            t_month[m] = self.t_sl[m].bilinear(bx, bz) - lapse;
            share[m] = self.p_share[m].bilinear(bx, bz);
            s += share[m];
        }
        let p_annual = self.p_mm.bilinear(bx, bz).max(0.0);
        let mut p_month = [0.0; 12];
        for m in 0..12 {
            let sm = if s > 0.0 { 12.0 * share[m] / s } else { 1.0 };
            p_month[m] = p_annual * sm / 12.0;
        }
        let mut sum = 0.0;
        let (mut warmest, mut coldest) = (0, 0);
        for m in 0..12 {
            sum += t_month[m];
            if t_month[m] > t_month[warmest] {
                warmest = m;
            }
            if t_month[m] < t_month[coldest] {
                coldest = m;
            }
        }
        let k = self.koppen.nearest(bx, bz);
        let koppen = if k.is_nan() {
            SEA
        } else {
            (k.round() as i64).clamp(0, SEA as i64) as usize
        };
        let koppen_code = if koppen == SEA {
            "sea".to_owned()
        } else {
            self.codes
                .get(koppen)
                .cloned()
                .unwrap_or_else(|| "?".to_owned())
        };
        ClimateSample {
            x,
            y,
            z,
            elevation_m,
            t_month,
            t_mean: sum / 12.0,
            t_warm: t_month[warmest],
            t_cold: t_month[coldest],
            warmest,
            coldest,
            p_annual,
            p_month,
            koppen,
            koppen_code,
        }
    }
}

impl ClimateSample {
    /// The parity fixture's record (`parity/climate.json`).
    pub fn to_json(&self) -> Value {
        json!({
            "x": self.x,
            "y": self.y,
            "z": self.z,
            "elevation_m": self.elevation_m,
            "t_month": self.t_month,
            "t_mean": self.t_mean,
            "t_warm": self.t_warm,
            "t_cold": self.t_cold,
            "warmest": self.warmest,
            "coldest": self.coldest,
            "p_annual": self.p_annual,
            "p_month": self.p_month,
            "koppen": self.koppen,
            "koppen_code": self.koppen_code,
        })
    }
}
