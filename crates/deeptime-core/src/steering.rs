// SPDX-License-Identifier: LGPL-3.0-or-later
//! End-weighted nudges ("steering"; docs/design/playability-steering.md §7; Ben, decisions log
//! 2026-10-05: "some end weighted nudges that players can choose. (Default for earth-like to avoid
//! bug reports of players getting a snowball earth hot earth and not being able to play the
//! game.)").
//!
//! Over the last `tail_fraction` of a run a weight rises from 0 to 1 and two controllers act on
//! physical controls the model already has, never on its outputs:
//!
//! - **ocean water volume → sea level → land share**: an offset of the eustatic sea level the
//!   climate stage computes (`climate::SEA_LEVEL_GLOBAL`), as water the deep water cycle takes
//!   into the mantle or gives back. Its volume is `offset · A_ocean / (1 − ρ_w / ρ_m)` (Airy
//!   hydro-isostasy: the floor sinks under added water; [`HYDRO_ISOSTASY`]).
//! - **volcanic outgassing → CO₂ → temperature**: a factor `e^x` on the carbon cycle's degassing
//!   (`climate::carbon::degassing`); the silicate-weathering feedback moves CO₂ and the climate
//!   follows.
//!
//! Each is an integrating controller with a dead band (the target band: nothing happens inside
//! it) and a saturating rate (no step larger than `rate · dt`). Before the tail the weight is 0
//! and nothing runs, so the run before its tail is the unsteered run; a preset without the
//! section, or with `enabled: false`, is the unsteered run bit for bit.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::CoreError;

/// The share of the sea's thickness of added water that the sea gains against the continents
/// when the ocean floor sinks under it (Airy hydro-isostasy, 1 − ρ_w / ρ_m with sea water
/// 1,030 kg/m³ and mantle 3,300 kg/m³).
pub const HYDRO_ISOSTASY: f64 = 1.0 - 1030.0 / 3300.0; // ledger: STE-008

/// Earth's ocean volume, km³ (the steered volume is reported as a share of it): 1.332 billion km³
/// (Charette & Smith 2010, Oceanography 23(2): 112, as their institution's release states it).
pub const EARTH_OCEAN_KM3: f64 = 1.332e9; // ledger: STE-009

/// Land-share error (outside the band) at which the sea controller runs at its full rate.
pub const LAND_FULL_RATE: f64 = 0.05; // ledger: STE-010 (game-choice)

/// Temperature error (outside the band), K, at which the CO₂ controller runs at its full rate.
pub const TEMP_FULL_RATE_K: f64 = 5.0; // ledger: STE-010 (game-choice)

/// The named target bands (`steering.target`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// Land 25–35 %, global mean 10–20 °C (Ben's example, "temperate, ~25–35 % land").
    #[default]
    Temperate,
    /// Land 25–35 %, 18–26 °C: a greenhouse world without a hothouse.
    Warm,
    /// Land 25–35 %, 6–13 °C: an icehouse world without a snowball.
    Cool,
    /// The section's `land_min`, `land_max`, `temp_min_c` and `temp_max_c`.
    Custom,
}

impl Target {
    pub fn name(self) -> &'static str {
        match self {
            Target::Temperate => "temperate",
            Target::Warm => "warm",
            Target::Cool => "cool",
            Target::Custom => "custom",
        }
    }
}

/// How the weight rises over the tail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    /// w = x.
    Linear,
    /// w = x²(3 − 2x): no kink where the tail starts or ends.
    #[default]
    Smoothstep,
    /// w = x²: most of the push near the end.
    Late,
}

impl Shape {
    pub fn name(self) -> &'static str {
        match self {
            Shape::Linear => "linear",
            Shape::Smoothstep => "smoothstep",
            Shape::Late => "late",
        }
    }

    /// The weight at x ∈ [0, 1] of the tail.
    pub fn at(self, x: f64) -> f64 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Shape::Linear => x,
            Shape::Smoothstep => x * x * (3.0 - 2.0 * x),
            Shape::Late => x * x,
        }
    }
}

/// The `steering` section (every key optional; a section without `enabled` is on).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SteeringParams {
    /// Off: the run is the unsteered run, bit for bit.
    pub enabled: bool,
    pub target: Target,
    /// The last share of the run over which the weight rises.
    pub tail_fraction: f64,
    pub shape: Shape,
    /// The custom bands: land share (0–1) and global mean temperature, °C.
    pub land_min: f64,
    pub land_max: f64,
    pub temp_min_c: f64,
    pub temp_max_c: f64,
    /// The sea controller's rate at full weight and full error, m of sea level per Myr.
    pub sea_rate_m_myr: f64,
    /// The largest sea-level offset, m (either way).
    pub sea_max_m: f64,
    /// The CO₂ controller's rate at full weight and full error: ln of the outgassing factor per
    /// Myr.
    pub co2_rate_per_myr: f64,
    /// The largest outgassing factor (and its inverse, the smallest).
    pub co2_factor_max: f64,
}

impl Default for SteeringParams {
    fn default() -> Self {
        SteeringParams {
            enabled: true,
            target: Target::Temperate,
            tail_fraction: 0.1,       // ledger: STE-002 (game-choice)
            shape: Shape::Smoothstep, // ledger: STE-003 (game-choice)
            land_min: 0.25,           // ledger: STE-004 (game-choice)
            land_max: 0.35,           // ledger: STE-004 (game-choice)
            temp_min_c: 10.0,         // ledger: STE-005 (game-choice)
            temp_max_c: 20.0,         // ledger: STE-005 (game-choice)
            sea_rate_m_myr: 2.0,      // ledger: STE-006 (game-choice)
            sea_max_m: 200.0,         // ledger: STE-006 (game-choice)
            co2_rate_per_myr: 0.2,    // ledger: STE-007 (game-choice)
            co2_factor_max: 2.0,      // ledger: STE-007 (game-choice)
        }
    }
}

fn check(ok: bool, key: &str, reason: &str) -> Result<(), CoreError> {
    if ok {
        Ok(())
    } else {
        Err(CoreError::preset(format!("$.steering.{key}"), reason))
    }
}

impl SteeringParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let fin = |v: f64| v.is_finite();
        check(
            fin(self.tail_fraction) && (0.02..=0.5).contains(&self.tail_fraction),
            "tail_fraction",
            "must be in [0.02, 0.5]",
        )?;
        // Each bound is valid on its own (the editor sets one at a time); the band is the two in
        // either order.
        for (v, key) in [(self.land_min, "land_min"), (self.land_max, "land_max")] {
            check(fin(v) && (0.0..=1.0).contains(&v), key, "must be in [0, 1]")?;
        }
        check(
            fin(self.temp_min_c) && (-50.0..=60.0).contains(&self.temp_min_c),
            "temp_min_c",
            "must be in [-50, 60]",
        )?;
        check(
            fin(self.temp_max_c) && (-50.0..=60.0).contains(&self.temp_max_c),
            "temp_max_c",
            "must be in [-50, 60]",
        )?;
        check(
            fin(self.sea_rate_m_myr) && (0.0..=20.0).contains(&self.sea_rate_m_myr),
            "sea_rate_m_myr",
            "must be in [0, 20]",
        )?;
        check(
            fin(self.sea_max_m) && (0.0..=1000.0).contains(&self.sea_max_m),
            "sea_max_m",
            "must be in [0, 1000]",
        )?;
        check(
            fin(self.co2_rate_per_myr) && (0.0..=1.0).contains(&self.co2_rate_per_myr),
            "co2_rate_per_myr",
            "must be in [0, 1]",
        )?;
        check(
            fin(self.co2_factor_max) && (1.0..=10.0).contains(&self.co2_factor_max),
            "co2_factor_max",
            "must be in [1, 10]",
        )?;
        Ok(())
    }

    /// The bands the target names: (land min, land max, temperature min, temperature max).
    pub fn bands(&self) -> (f64, f64, f64, f64) {
        match self.target {
            // ledger: STE-004, STE-005 (game-choice)
            Target::Temperate => (0.25, 0.35, 10.0, 20.0),
            Target::Warm => (0.25, 0.35, 18.0, 26.0),
            Target::Cool => (0.25, 0.35, 6.0, 13.0),
            Target::Custom => (
                self.land_min.min(self.land_max),
                self.land_min.max(self.land_max),
                self.temp_min_c.min(self.temp_max_c),
                self.temp_min_c.max(self.temp_max_c),
            ),
        }
    }

    /// The tail's length, years, of a run `run_years` long (0 when off).
    pub fn tail_years(&self, run_years: f64) -> f64 {
        if self.enabled {
            self.tail_fraction * run_years
        } else {
            0.0
        }
    }

    /// The weight at world time `t` of a run ending at `end` after `run_years`: 0 before the
    /// tail and when off, rising to 1 at the end.
    pub fn weight(&self, t: f64, end: f64, run_years: f64) -> f64 {
        let tail = self.tail_years(run_years);
        if tail <= 0.0 {
            return 0.0;
        }
        let x = (t - (end - tail)) / tail;
        if x <= 0.0 { 0.0 } else { self.shape.at(x) }
    }
}

/// The controllers' state, carried in the climate stage's saved state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SteerState {
    /// The sea-level offset, m (positive: water added).
    pub sea_m: f64,
    /// ln of the outgassing factor.
    pub ln_outgas: f64,
    /// The first step with a weight above 0: (world time, land share, global mean °C).
    pub started: Option<(f64, f64, f64)>,
    /// The last step's measurements: (world time, weight, land share, global mean °C).
    pub last: Option<(f64, f64, f64, f64)>,
    /// The largest change of the sea offset in one step, m, and of ln(outgassing).
    pub max_sea_step_m: f64,
    pub max_ln_step: f64,
    /// Reversals of each control's direction (a sign of flip-flopping), and the last direction.
    pub sea_reversals: u32,
    pub co2_reversals: u32,
    sea_dir: i8,
    co2_dir: i8,
    /// One row per steered step: t_myr, weight, land share, global mean °C, sea offset m,
    /// ln(outgassing).
    pub rows: Vec<[f32; 6]>,
    /// The ocean's area at the last steered step, km² scaled to an Earth-sized planet (the
    /// climate's areas): the offset's water volume is reported on that scale.
    pub ocean_km2: f64,
}

/// What one plate step's controllers see.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measured {
    /// World time at the step's end, years.
    pub t_years: f64,
    /// The weight there.
    pub w: f64,
    /// The step's length, Myr.
    pub dt_myr: f64,
    /// The land share (area) and the global mean temperature at the step's end (NaN: unknown,
    /// no action).
    pub land: f64,
    pub temp_c: f64,
    /// False leaves the outgassing alone (a climate without the carbon cycle).
    pub co2: bool,
}

/// One step of both controllers.
pub fn step(p: &SteeringParams, st: &mut SteerState, m: Measured) {
    let Measured {
        t_years,
        w,
        dt_myr,
        land,
        temp_c,
        co2,
    } = m;
    if w <= 0.0 {
        return;
    }
    let (l_lo, l_hi, t_lo, t_hi) = p.bands();
    if st.started.is_none() {
        st.started = Some((t_years, land, temp_c));
    }
    let band_err = |v: f64, lo: f64, hi: f64| -> f64 {
        if !v.is_finite() {
            0.0
        } else if v < lo {
            lo - v
        } else if v > hi {
            hi - v
        } else {
            0.0
        }
    };
    let dir = |d: f64| -> i8 {
        if d > 0.0 {
            1
        } else if d < 0.0 {
            -1
        } else {
            0
        }
    };
    // Too little land: lower the sea (take water out); too much: raise it.
    let e_l = band_err(land, l_lo, l_hi);
    let d_sea = -w * p.sea_rate_m_myr * dt_myr * (e_l / LAND_FULL_RATE).clamp(-1.0, 1.0);
    let sea = (st.sea_m + d_sea).clamp(-p.sea_max_m, p.sea_max_m);
    let d_sea = sea - st.sea_m;
    st.sea_m = sea;
    st.max_sea_step_m = st.max_sea_step_m.max(d_sea.abs());
    let ds = dir(d_sea);
    if ds != 0 {
        if st.sea_dir != 0 && ds != st.sea_dir {
            st.sea_reversals += 1;
        }
        st.sea_dir = ds;
    }
    // Too cold: more outgassing; too warm: less.
    if co2 {
        let e_t = band_err(temp_c, t_lo, t_hi);
        let lim = p.co2_factor_max.ln();
        let d_x = w * p.co2_rate_per_myr * dt_myr * (e_t / TEMP_FULL_RATE_K).clamp(-1.0, 1.0);
        let x = (st.ln_outgas + d_x).clamp(-lim, lim);
        let d_x = x - st.ln_outgas;
        st.ln_outgas = x;
        st.max_ln_step = st.max_ln_step.max(d_x.abs());
        let dc = dir(d_x);
        if dc != 0 {
            if st.co2_dir != 0 && dc != st.co2_dir {
                st.co2_reversals += 1;
            }
            st.co2_dir = dc;
        }
    }
    st.last = Some((t_years, w, land, temp_c));
    st.rows.push([
        (t_years / 1.0e6) as f32,
        w as f32,
        land as f32,
        temp_c as f32,
        st.sea_m as f32,
        st.ln_outgas as f32,
    ]);
}

/// Columns of [`SteerState::rows`].
pub const ROW_COLUMNS: [&str; 6] = [
    "t_myr",
    "weight",
    "land_frac",
    "t_global_c",
    "sea_offset_m",
    "ln_outgassing",
];

/// The run's report (`manifest.json` `notes.climate.steering`): the settings, where the tail
/// began, the controls' final values, the targets met, the artefact counters and the rows.
pub fn report(p: &SteeringParams, st: &SteerState) -> Value {
    let (l_lo, l_hi, t_lo, t_hi) = p.bands();
    let within = |v: f64, lo: f64, hi: f64| v.is_finite() && v >= lo - 1e-9 && v <= hi + 1e-9;
    // The water the offset stands for on an Earth-sized planet: the sea gains only
    // `HYDRO_ISOSTASY` of the water's thickness against the continents.
    let volume = st.sea_m * st.ocean_km2 / 1000.0 / HYDRO_ISOSTASY;
    let (land_end, temp_end) = st.last.map_or((f64::NAN, f64::NAN), |l| (l.2, l.3));
    json!({
        "enabled": p.enabled,
        "target": p.target.name(),
        "bands": {"land": [l_lo, l_hi], "t_global_c": [t_lo, t_hi]},
        "tail_fraction": p.tail_fraction,
        "shape": p.shape.name(),
        "tail_start": st.started.map(|s| json!({
            "t_myr": s.0 / 1.0e6, "land_frac": s.1, "t_global_c": s.2,
        })),
        "end": {"land_frac": land_end, "t_global_c": temp_end},
        "land_met": within(land_end, l_lo, l_hi),
        "temperature_met": within(temp_end, t_lo, t_hi),
        "sea_offset_m": st.sea_m,
        "earth_equivalent_volume_km3": volume,
        "share_of_earths_ocean": volume / EARTH_OCEAN_KM3,
        "outgassing_factor": st.ln_outgas.exp(),
        "max_sea_step_m": st.max_sea_step_m,
        "max_ln_outgassing_step": st.max_ln_step,
        "sea_reversals": st.sea_reversals,
        "co2_reversals": st.co2_reversals,
        "rows": {"columns": ROW_COLUMNS, "rows": st.rows},
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(t_years: f64, w: f64, dt_myr: f64, land: f64, temp_c: f64, co2: bool) -> Measured {
        Measured {
            t_years,
            w,
            dt_myr,
            land,
            temp_c,
            co2,
        }
    }

    #[test]
    fn the_weight_is_zero_before_the_tail_and_one_at_the_end() {
        let p = SteeringParams::default();
        let run = 500.0e6;
        assert_eq!(p.weight(-60.0e6, 0.0, run), 0.0);
        assert_eq!(p.weight(-50.0e6, 0.0, run), 0.0, "the tail starts at 0");
        assert!((p.weight(-25.0e6, 0.0, run) - 0.5).abs() < 1e-12);
        assert_eq!(p.weight(0.0, 0.0, run), 1.0);
        let off = SteeringParams {
            enabled: false,
            ..p
        };
        assert_eq!(off.weight(0.0, 0.0, run), 0.0);
        assert_eq!(off.tail_years(run), 0.0);
        for s in [Shape::Linear, Shape::Smoothstep, Shape::Late] {
            assert_eq!(s.at(0.0), 0.0);
            assert_eq!(s.at(1.0), 1.0);
            let mut prev = 0.0;
            for k in 1..=100 {
                let w = s.at(k as f64 / 100.0);
                assert!(w >= prev, "{s:?} rises");
                prev = w;
            }
        }
    }

    #[test]
    fn the_controllers_act_only_outside_their_bands_and_never_jump() {
        let p = SteeringParams::default();
        let mut st = SteerState::default();
        // Inside both bands: nothing moves.
        step(&p, &mut st, m(-1.0e6, 1.0, 2.0, 0.30, 15.0, true));
        assert_eq!((st.sea_m, st.ln_outgas), (0.0, 0.0));
        // Too little land and too cold: the sea falls, outgassing rises, each by at most its rate.
        step(&p, &mut st, m(0.0, 1.0, 2.0, 0.10, -30.0, true));
        assert_eq!(st.sea_m, -p.sea_rate_m_myr * 2.0);
        assert!((st.ln_outgas - p.co2_rate_per_myr * 2.0).abs() < 1e-12);
        // A small error: a proportional step.
        let mut s2 = SteerState::default();
        step(&p, &mut s2, m(0.0, 0.5, 2.0, 0.24, 21.0, true));
        assert!((s2.sea_m - -(0.5 * 2.0 * 2.0 * 0.2)).abs() < 1e-12);
        assert!((s2.ln_outgas - -(0.5 * 0.2 * 2.0 * 0.2)).abs() < 1e-12);
        // Bounded.
        let mut s3 = SteerState::default();
        for k in 0..10_000 {
            step(&p, &mut s3, m(k as f64, 1.0, 2.0, 0.0, 60.0, true));
        }
        assert_eq!(s3.sea_m, -p.sea_max_m);
        assert!((s3.ln_outgas + p.co2_factor_max.ln()).abs() < 1e-12);
        assert_eq!((s3.sea_reversals, s3.co2_reversals), (0, 0));
        // Weight 0: no step, no row.
        let mut s4 = SteerState::default();
        step(&p, &mut s4, m(0.0, 0.0, 2.0, 0.0, -40.0, true));
        assert_eq!(s4, SteerState::default());
        // Without the carbon cycle the outgassing stays.
        let mut s5 = SteerState::default();
        step(&p, &mut s5, m(0.0, 1.0, 2.0, 0.3, -40.0, false));
        assert_eq!(s5.ln_outgas, 0.0);
    }

    #[test]
    fn named_targets_ignore_the_custom_numbers() {
        let p = SteeringParams {
            land_min: 0.5,
            land_max: 0.6,
            ..SteeringParams::default()
        };
        assert_eq!(p.bands(), (0.25, 0.35, 10.0, 20.0));
        let c = SteeringParams {
            target: Target::Custom,
            ..p
        };
        assert_eq!(c.bands(), (0.5, 0.6, 10.0, 20.0));
        let swapped = SteeringParams { land_min: 0.7, ..c };
        assert!(swapped.validate().is_ok());
        assert_eq!(swapped.bands(), (0.6, 0.7, 10.0, 20.0), "either order");
        let bad = SteeringParams { land_min: 1.5, ..c };
        assert!(bad.validate().is_err());
        assert!(SteeringParams::default().validate().is_ok());
    }
}
