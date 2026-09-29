// SPDX-License-Identifier: LGPL-3.0-or-later
//! A synthetic field history for the deposit engine, standing in for P1-03…P1-07 until they exist
//! (and afterwards for tests): a small invented planet whose geography is fixed in latitude and
//! longitude, so every circumference shows the same story.
//!
//! - **Continent A** (centred 90° E of the seam, 10° N) with an Andean margin on its west coast:
//!   subduction from 320 Myr ago, arc magmatism in three pulses (≈280, 110 and 25 Myr ago), an
//!   ophiolitic fore-arc terrane accreted 330 Myr ago → porphyry Cu-Mo-Au, supergene blankets
//!   where the climate is dry, placers where porphyries were eroded away.
//! - **Continent B** (205°, 30° N), two blocks that collided 350 Myr ago along a suture at 208°;
//!   east of it a carbonate platform (460–330 Myr ago) with evaporites, later swept by
//!   uplift-driven brines → MVT Zn–Pb; a foreland basin with coal measures 330–285 Myr ago.
//! - **Continent C** (300°, 28° S): an Archean craton whose core carries inherited banded iron
//!   formation (a shelf sea 2.1 Gyr ago) → BIF; a rift on its northern edge 400 Myr ago, right at
//!   the thick–thin lithosphere step, with euxinic black shale → SEDEX Zn–Pb; buried under a sag
//!   basin, inverted 255 Myr ago into anticlines → a petroleum system.
//!
//! Clocks: oxygenated oceans throughout, land plants from 420 Myr ago, no BIF acme (the timeline
//! starts after it). Every feature also emits the tectonic events a real history would (source
//! `deeptime:synthetic`), so "Why here?" chains work end to end. All numbers here are design
//! values for a test planet, not measurements.

use std::f64::consts::PI;

use deeptime_format::GridDesc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::fields::{self, FieldFrame, FieldHistory};
use super::random::hash_str;
use crate::error::CoreError;

/// The source id of every synthetic event.
pub const SOURCE: &str = "deeptime:synthetic";

/// Parameters of the synthetic history.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SyntheticParams {
    pub seed: u64,
    pub circumference: u32,
    pub grid_r: u32,
    pub t_start_years: f64,
    pub t_end_years: f64,
    pub snapshot_every_years: f64,
}

/// An event the synthetic planet's history contains.
#[derive(Clone, Debug, PartialEq)]
pub struct SynthEvent {
    pub kind: &'static str,
    pub t_start: f64,
    pub t_end: f64,
    pub lat: f32,
    pub lon: f32,
    pub magnitude: f32,
    pub feature: &'static str,
}

/// Per-cell geography that does not change with time.
struct Static {
    lat: Vec<f32>,
    land: Vec<f32>,
    relief_noise: Vec<f32>,
    arc: Vec<f32>,
    forearc: Vec<f32>,
    strike: Vec<f32>,
    ox_noise: Vec<f32>,
    obl_noise: Vec<f32>,
    slab_noise: Vec<f32>,
    orogen: Vec<f32>,
    platform: Vec<f32>,
    evaporite: Vec<f32>,
    coal_basin: Vec<f32>,
    craton_core: Vec<f32>,
    step_km: Vec<f32>,
    rift: Vec<f32>,
    anticline: Vec<f32>,
    seal: Vec<f32>,
}

/// The synthetic history.
pub struct SyntheticHistory {
    params: SyntheticParams,
    grid: GridDesc,
    stat: Static,
    receivers: Vec<u32>,
    times: Vec<(f64, f64)>,
    km_per_deg: f64,
}

// ---- small helpers -------------------------------------------------------------------------

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 1 inside [t0, t1], ramping over `ramp` on both sides.
fn window(t: f64, t0: f64, t1: f64, ramp: f64) -> f64 {
    smoothstep(t0 - ramp, t0, t) * (1.0 - smoothstep(t1, t1 + ramp, t))
}

fn gauss(t: f64, c: f64, s: f64) -> f64 {
    (-0.5 * ((t - c) / s).powi(2)).exp()
}

/// Longitude difference a − b wrapped into (−180, 180].
fn dlon(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

/// Normalised elliptical distance in degrees (1 on the ellipse).
fn ellipse(lat: f64, lon: f64, lat0: f64, lon0: f64, a_lon: f64, b_lat: f64) -> f64 {
    ((dlon(lon, lon0) / a_lon).powi(2) + ((lat - lat0) / b_lat).powi(2)).sqrt()
}

/// A lattice hash in [0, 1).
fn hash3(seed: u64, x: i64, y: i64, z: i64) -> f64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
    for v in [x, y, z] {
        h ^= (v as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = h.rotate_left(27).wrapping_mul(0x94D0_49BB_1331_11EB);
    }
    h ^= h >> 31;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Trilinear value noise with smoothstep weights, in [0, 1).
fn value_noise(seed: u64, p: [f64; 3]) -> f64 {
    let i = p.map(|v| v.floor() as i64);
    let f = p.map(|v| {
        let t = v - v.floor();
        t * t * (3.0 - 2.0 * t)
    });
    let mut acc = 0.0;
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                let w = (if dx == 1 { f[0] } else { 1.0 - f[0] })
                    * (if dy == 1 { f[1] } else { 1.0 - f[1] })
                    * (if dz == 1 { f[2] } else { 1.0 - f[2] });
                acc += w * hash3(seed, i[0] + dx, i[1] + dy, i[2] + dz);
            }
        }
    }
    acc
}

/// Three octaves of value noise on the unit sphere at `freq` cycles per radius, in [0, 1].
fn sphere_noise(seed: u64, lat: f64, lon: f64, freq: f64) -> f64 {
    let (la, lo) = (lat.to_radians(), lon.to_radians());
    let p = [la.cos() * lo.cos(), la.sin(), la.cos() * lo.sin()];
    let (mut acc, mut amp, mut norm, mut fr) = (0.0, 1.0, 0.0, freq);
    for o in 0..3u64 {
        acc += amp * value_noise(seed.wrapping_add(o), p.map(|v| v * fr + 100.0));
        norm += amp;
        amp *= 0.5;
        fr *= 2.0;
    }
    acc / norm
}

// ---- the planet --------------------------------------------------------------------------

/// West coast of continent A at latitude `lat` (degrees east of the seam).
fn a_west_coast(lat: f64) -> f64 {
    90.0 - 42.0 * (1.0 - ((lat - 10.0) / 38.0).powi(2)).max(0.0).sqrt()
}

/// Arc activity at time `t_myr` (negative) and along-strike position `s` ∈ [0, 1]: a background
/// once subduction starts 320 Myr ago plus three flare-ups whose timing shifts along strike.
fn arc_activity(t_myr: f64, s: f64) -> f64 {
    if t_myr < -325.0 {
        return 0.0;
    }
    let shift = 12.0 * (s - 0.5);
    let pulses = gauss(t_myr, -280.0 + shift, 14.0)
        + gauss(t_myr, -110.0 - shift, 10.0)
        + 0.9 * gauss(t_myr, -25.0 + shift, 11.0);
    (0.25 * smoothstep(-325.0, -315.0, t_myr) + pulses).min(1.0)
}

fn orogeny(t_myr: f64) -> f64 {
    window(t_myr, -350.0, -305.0, 8.0)
}

impl Static {
    fn build(grid: &GridDesc, seed: u64) -> Static {
        let n = grid.cells();
        let c = grid.circumference() as f64;
        let per_cell: Vec<[f32; 19]> = (0..n)
            .into_par_iter()
            .map(|cell| {
                let i = (cell % grid.w as usize) as i64;
                let j = (cell / grid.w as usize) as i64;
                let x = grid.block_x(i) as f64;
                let z = grid.block_z(j) as f64;
                let lon = 360.0 * (x + c / 2.0) / c;
                let lat = -z / (c / 4.0) * 90.0;
                let n1 = sphere_noise(seed ^ hash_str("coast"), lat, lon, 4.0);
                let n2 = sphere_noise(seed ^ hash_str("relief"), lat, lon, 9.0);
                // Continents: ellipses with a noisy edge.
                let edge = |r: f64| 1.0 - smoothstep(0.92, 1.05, r + 0.18 * (n1 - 0.5));
                let a = edge(ellipse(lat, lon, 10.0, 90.0, 42.0, 38.0));
                let b = edge(ellipse(lat, lon, 30.0, 205.0, 45.0, 22.0));
                let cc = edge(ellipse(lat, lon, -28.0, 300.0, 32.0, 26.0));
                let land = a.max(b).max(cc);
                // Andean arc on A's west coast, 2–9° inland, 25° S – 45° N.
                let din = dlon(lon, a_west_coast(lat));
                let lat_ok = window(lat, -25.0, 45.0, 4.0);
                let arc = (smoothstep(1.5, 3.0, din) * (1.0 - smoothstep(6.5, 9.5, din)))
                    * lat_ok
                    * a;
                let forearc = (smoothstep(-0.5, 0.5, din) * (1.0 - smoothstep(1.5, 2.5, din)))
                    * lat_ok
                    * a;
                let strike = ((lat + 25.0) / 70.0).clamp(0.0, 1.0);
                // B's suture at 208° and the platform and foreland east of it.
                let lat_b = window(lat, 12.0, 46.0, 4.0) * b;
                let orogen = (1.0 - smoothstep(2.0, 6.0, dlon(lon, 208.0).abs())) * lat_b;
                let east = dlon(lon, 208.0);
                let platform = smoothstep(3.0, 5.0, east) * (1.0 - smoothstep(15.0, 18.0, east));
                let platform = platform * window(lat, 8.0, 44.0, 4.0) * b;
                let evaporite = platform * window(lat, 22.0, 40.0, 5.0) * 0.9;
                let coal_basin = smoothstep(4.0, 6.0, east)
                    * (1.0 - smoothstep(16.0, 19.0, east))
                    * window(lat, 30.0, 46.0, 3.0)
                    * b;
                // C's craton and the rift on its northern edge.
                let core = 1.0 - smoothstep(0.7, 1.0, ellipse(lat, lon, -30.0, 300.0, 14.0, 11.0));
                let craton_r = ellipse(lat, lon, -30.0, 300.0, 20.0, 17.0);
                let step_deg = (craton_r - 1.0).abs() * 17.0;
                let rift = window(lat, -12.0, -3.0, 2.5)
                    * window(dlon(lon, 301.0), -15.0, 15.0, 3.0)
                    * cc.max(0.3);
                let anticline = rift * (2.0 * PI * dlon(lon, 285.0) / 4.0).sin().max(0.0).powi(3);
                let seal = rift * smoothstep(0.35, 0.55, n2);
                [
                    lat as f32,
                    lon as f32,
                    land as f32,
                    n2 as f32,
                    arc as f32,
                    forearc as f32,
                    strike as f32,
                    sphere_noise(seed ^ hash_str("ox"), lat, lon, 6.0) as f32,
                    sphere_noise(seed ^ hash_str("obl"), lat, lon, 5.0) as f32,
                    sphere_noise(seed ^ hash_str("slab"), lat, lon, 3.0) as f32,
                    orogen as f32,
                    platform as f32,
                    evaporite as f32,
                    coal_basin as f32,
                    core as f32,
                    step_deg as f32,
                    rift as f32,
                    anticline as f32,
                    seal as f32,
                ]
            })
            .collect();
        let col = |k: usize| per_cell.iter().map(|v| v[k]).collect::<Vec<f32>>();
        Static {
            lat: col(0),
            land: col(2),
            relief_noise: col(3),
            arc: col(4),
            forearc: col(5),
            strike: col(6),
            ox_noise: col(7),
            obl_noise: col(8),
            slab_noise: col(9),
            orogen: col(10),
            platform: col(11),
            evaporite: col(12),
            coal_basin: col(13),
            craton_core: col(14),
            step_km: col(15), // degrees here; scaled to km in `new`
            rift: col(16),
            anticline: col(17),
            seal: col(18),
        }
    }
}

/// Which synthetic fields a frame carries (all of [`fields::FIELDS`] plus host fractions).
const HOSTS: &[&str] = &[
    "granodiorite",
    "andesite",
    "serpentinite",
    "limestone",
    "dolostone",
    "black_shale",
    "banded_iron_formation",
    "sandstone",
    "coal",
    "granite",
];

impl SyntheticHistory {
    pub fn new(params: SyntheticParams) -> Result<SyntheticHistory, CoreError> {
        let grid = GridDesc::for_planet("L1", params.circumference, params.grid_r)
            .map_err(|e| CoreError::preset("$.deposits_synthetic.grid_r", e.to_string()))?;
        let span = params.t_end_years - params.t_start_years;
        if !(span > 0.0 && params.snapshot_every_years > 0.0) {
            return Err(CoreError::preset(
                "$.deposits_synthetic",
                "t_end_years must follow t_start_years and snapshot_every_years be > 0",
            ));
        }
        let n = (span / params.snapshot_every_years).ceil() as usize;
        let dt = span / n as f64;
        let times = (0..n)
            .map(|k| (params.t_start_years + (k as f64 + 1.0) * dt, dt))
            .collect();
        let mut stat = Static::build(&grid, params.seed);
        // 1° of latitude in km: C · 300 m / 360.
        let km_per_deg = f64::from(params.circumference) * 0.3 / 360.0;
        for v in &mut stat.step_km {
            *v = (f64::from(*v) * km_per_deg) as f32;
        }
        let mut h = SyntheticHistory {
            params,
            grid,
            stat,
            receivers: Vec::new(),
            times,
            km_per_deg,
        };
        let elev = h.elevation(h.params.t_end_years / 1.0e6);
        h.receivers = steepest_descent(&h.grid, &elev);
        Ok(h)
    }

    pub fn params(&self) -> &SyntheticParams {
        &self.params
    }

    fn map(&self, f: impl Fn(usize) -> f64 + Sync) -> Vec<f32> {
        (0..self.grid.cells())
            .into_par_iter()
            .map(|c| f(c) as f32)
            .collect()
    }

    fn elevation(&self, t: f64) -> Vec<f32> {
        let s = &self.stat;
        self.map(|c| {
            let land = f64::from(s.land[c]);
            let relief = f64::from(s.relief_noise[c]) - 0.5;
            let base_land = 250.0 + 600.0 * relief;
            let arc = f64::from(s.arc[c]) * (1200.0 + 1500.0 * arc_activity(t, f64::from(s.strike[c])))
                * smoothstep(-330.0, -300.0, t);
            let oro = f64::from(s.orogen[c])
                * (3500.0 * orogeny(t) + 900.0 * smoothstep(-350.0, -330.0, t));
            let ocean = -4200.0 + 800.0 * relief;
            land * (base_land + arc + oro) + (1.0 - land) * ocean
        })
    }

    /// The frame at time `t` (years) standing for `dt` years.
    fn build_frame(&self, t_years: f64, dt_years: f64) -> FieldFrame {
        let t = t_years / 1.0e6;
        let s = &self.stat;
        let mut f = FieldFrame {
            t_years,
            dt_years,
            ..FieldFrame::default()
        };
        let arc_act = |c: usize| arc_activity(t, f64::from(s.strike[c]));
        let subduct = smoothstep(-325.0, -315.0, t);
        let g = |v: &[f32], c: usize| f64::from(v[c]);
        let put = |f: &mut FieldFrame, name: &str, v: Vec<f32>| {
            f.fields.insert(name.to_owned(), v);
        };
        put(
            &mut f,
            "deeptime:arc_magma_flux",
            self.map(|c| g(&s.arc, c) * arc_act(c)),
        );
        put(
            &mut f,
            "deeptime:magma_oxidation",
            self.map(|c| g(&s.arc, c).clamp(0.0, 1.0) * (0.35 + 0.6 * g(&s.ox_noise, c))),
        );
        put(
            &mut f,
            "deeptime:convergence_rate_km_myr",
            self.map(|c| {
                let along = (PI * g(&s.strike, c) + 0.012 * t).sin();
                (g(&s.arc, c) + g(&s.forearc, c)).min(1.0) * subduct * (75.0 + 55.0 * along)
            }),
        );
        put(
            &mut f,
            "deeptime:obliquity_deg",
            self.map(|c| 5.0 + 40.0 * g(&s.obl_noise, c)),
        );
        put(
            &mut f,
            "deeptime:slab_age_myr",
            self.map(|c| 15.0 + 90.0 * g(&s.slab_noise, c) + 10.0 * (0.02 * t).sin()),
        );
        put(
            &mut f,
            "deeptime:crust_thickness_km",
            self.map(|c| {
                let land = g(&s.land, c);
                let arc = g(&s.arc, c) * 22.0 * smoothstep(-320.0, -120.0, t);
                let oro = g(&s.orogen, c) * (25.0 * orogeny(t) + 12.0 * smoothstep(-350.0, -330.0, t));
                land * (34.0 + arc + oro) + (1.0 - land) * 7.0
            }),
        );
        put(
            &mut f,
            "deeptime:fault_density",
            self.map(|c| {
                (0.15 + 0.7 * g(&s.arc, c) * (0.4 + 0.6 * arc_act(c))
                    + 0.7 * g(&s.orogen, c) * orogeny(t)
                    + 0.6 * g(&s.rift, c) * window(t, -400.0, -370.0, 5.0))
                .min(1.0)
            }),
        );
        put(
            &mut f,
            "deeptime:orogenic_uplift_m_myr",
            self.map(|c| {
                g(&s.orogen, c) * 800.0 * orogeny(t) + g(&s.arc, c) * 150.0 * arc_act(c)
            }),
        );
        put(
            &mut f,
            "deeptime:rift_subsidence_m_myr",
            self.map(|c| {
                g(&s.rift, c) * (300.0 * window(t, -400.0, -372.0, 4.0) + 40.0 * window(t, -370.0, -262.0, 6.0))
            }),
        );
        put(
            &mut f,
            "deeptime:heat_flow_mw_m2",
            self.map(|c| {
                55.0 + 45.0 * g(&s.rift, c) * window(t, -400.0, -365.0, 5.0)
                    + 30.0 * g(&s.arc, c) * arc_act(c)
            }),
        );
        put(
            &mut f,
            "deeptime:dist_litho_step_km",
            self.map(|c| g(&s.step_km, c)),
        );
        put(
            &mut f,
            "deeptime:inherited_bif",
            self.map(|c| g(&s.craton_core, c) * (0.35 + 0.65 * g(&s.ox_noise, c))),
        );
        put(
            &mut f,
            "deeptime:shelf",
            self.map(|c| g(&s.platform, c) * window(t, -460.0, -335.0, 8.0)),
        );
        put(&mut f, fields::ELEVATION, self.elevation(t));
        put(
            &mut f,
            fields::EROSION,
            self.map(|c| {
                let land = g(&s.land, c);
                // Cratons erode slowly (≈2 m/Myr), other land ≈6 m/Myr.
                land * (6.0 - 4.0 * g(&s.craton_core, c)
                    + g(&s.arc, c) * (20.0 + 60.0 * arc_act(c))
                    + g(&s.forearc, c) * 40.0 * subduct
                    + g(&s.orogen, c) * (450.0 * orogeny(t) + 60.0 * window(t, -300.0, -200.0, 20.0))
                    + g(&s.rift, c) * 60.0 * window(t, -255.0, -240.0, 3.0))
            }),
        );
        put(
            &mut f,
            fields::DEPOSITION,
            self.map(|c| {
                g(&s.rift, c) * (120.0 * window(t, -395.0, -262.0, 5.0))
                    + g(&s.coal_basin, c) * 90.0 * window(t, -345.0, -250.0, 5.0)
                    + g(&s.platform, c) * 40.0 * window(t, -460.0, -335.0, 5.0)
                    + (1.0 - g(&s.land, c)) * 5.0
            }),
        );
        put(
            &mut f,
            fields::GRADIENT,
            self.map(|c| {
                let platform = g(&s.platform, c)
                    * (22.0 * window(t, -352.0, -318.0, 8.0) + 2.0 * smoothstep(-350.0, -330.0, t));
                platform + g(&s.orogen, c) * 30.0 * orogeny(t) + g(&s.arc, c) * 15.0 * arc_act(c)
            }),
        );
        put(
            &mut f,
            "deeptime:carbonate_platform",
            self.map(|c| g(&s.platform, c) * smoothstep(-462.0, -445.0, t)),
        );
        put(
            &mut f,
            "deeptime:evaporite",
            self.map(|c| g(&s.evaporite, c) * smoothstep(-440.0, -425.0, t)),
        );
        put(
            &mut f,
            "deeptime:euxinic_basin",
            self.map(|c| g(&s.rift, c) * window(t, -396.0, -366.0, 4.0)),
        );
        put(
            &mut f,
            "deeptime:coal_facies",
            self.map(|c| g(&s.coal_basin, c) * window(t, -332.0, -286.0, 5.0)),
        );
        put(
            &mut f,
            "deeptime:source_rock_toc",
            self.map(|c| g(&s.rift, c) * 4.5 * smoothstep(-396.0, -388.0, t)),
        );
        put(
            &mut f,
            fields::BURIAL_TEMP,
            self.map(|c| {
                // The rift fill heats to ≈125 °C by 260 Myr ago, inversion cools it after 255;
                // the coal measures reach ≈110 °C under the foreland fill.
                let rift = 20.0 + 105.0 * smoothstep(-392.0, -262.0, t)
                    - 50.0 * smoothstep(-257.0, -240.0, t);
                let coal = 18.0 + 92.0 * smoothstep(-320.0, -250.0, t)
                    - 25.0 * smoothstep(-200.0, -150.0, t);
                let (r, k) = (g(&s.rift, c), g(&s.coal_basin, c));
                if r >= k {
                    r * rift + (1.0 - r) * 18.0
                } else {
                    k * coal + (1.0 - k) * 18.0
                }
            }),
        );
        put(
            &mut f,
            "deeptime:reservoir_quality",
            self.map(|c| g(&s.rift, c) * 0.8 * smoothstep(-372.0, -360.0, t)),
        );
        put(
            &mut f,
            "deeptime:seal",
            self.map(|c| g(&s.seal, c) * smoothstep(-365.0, -355.0, t)),
        );
        put(
            &mut f,
            "deeptime:trap_structure",
            self.map(|c| g(&s.anticline, c) * smoothstep(-258.0, -250.0, t)),
        );
        let precip = |c: usize| {
            let lat = g(&s.lat, c).abs();
            // Wet equator, dry subtropics at 20–30°, moister mid-latitudes, dry poles.
            let base = 2200.0 * gauss(lat, 0.0, 9.0) + 900.0 * gauss(lat, 48.0, 12.0) + 180.0;
            base * (0.75 + 0.5 * g(&s.relief_noise, c))
        };
        put(&mut f, fields::PRECIP, self.map(precip));
        put(
            &mut f,
            fields::WATER_TABLE,
            self.map(|c| {
                let land = g(&s.land, c);
                let dry = 1.0 - (precip(c) / 1400.0).min(1.0);
                land * (8.0 + 260.0 * dry * (0.4 + 0.6 * g(&s.arc, c).max(g(&s.orogen, c))))
            }),
        );
        put(
            &mut f,
            "deeptime:t_mean_c",
            self.map(|c| 27.0 - 0.45 * g(&s.lat, c).abs()),
        );
        for host in HOSTS {
            let v = self.map(|c| match *host {
                "granodiorite" => 0.55 * g(&s.arc, c),
                "andesite" => 0.3 * g(&s.arc, c),
                "serpentinite" => 0.5 * g(&s.forearc, c) * subduct,
                "limestone" => 0.35 * g(&s.platform, c),
                "dolostone" => 0.55 * g(&s.platform, c),
                "black_shale" => 0.6 * g(&s.rift, c),
                "banded_iron_formation" => 0.7 * g(&s.craton_core, c),
                "sandstone" => 0.4 * g(&s.rift, c).max(g(&s.coal_basin, c)),
                "coal" => 0.15 * g(&s.coal_basin, c) * smoothstep(-332.0, -300.0, t),
                _ => 0.4 * g(&s.land, c),
            });
            put(&mut f, &format!("{}{host}", fields::HOST_PREFIX), v);
        }
        f.clocks
            .insert("deeptime:clock/oxygenated_oceans".into(), 1.0);
        f.clocks.insert(
            "deeptime:clock/land_plants".into(),
            if t >= -420.0 { 1.0 } else { 0.0 },
        );
        f.clocks.insert("deeptime:clock/bif_acme".into(), 0.0);
        f
    }

    /// The tectonic events of this history (Point places, degrees).
    pub fn events(&self) -> Vec<SynthEvent> {
        let mut out = Vec::new();
        let mut ev = |kind, t0: f64, t1: f64, lat: f64, lon: f64, m: f64, feature| {
            out.push(SynthEvent {
                kind,
                t_start: t0 * 1.0e6,
                t_end: t1 * 1.0e6,
                lat: lat as f32,
                lon: lon.rem_euclid(360.0) as f32,
                magnitude: m as f32,
                feature,
            });
        };
        // A's margin: subduction, arc compression phases, fore-arc accretion, exhumation.
        for lat in [-20.0, -5.0, 10.0, 25.0, 40.0] {
            let lon = a_west_coast(lat) + 4.0;
            ev("deeptime:subduction_onset", -322.0, 0.0, lat, lon - 4.0, 0.8, "A margin");
            ev("deeptime:terrane_accretion", -330.0, -326.0, lat, lon - 3.0, 0.5, "A fore-arc");
            for (c, w) in [(-280.0, 20.0), (-110.0, 14.0), (-25.0, 16.0)] {
                ev("deeptime:orogeny_phase", c - w, c + w, lat, lon, 0.7, "A arc flare-up");
                ev("deeptime:erosion_pulse", c - w, c + w + 10.0, lat, lon, 0.5, "A arc exhumation");
            }
        }
        // B: collision, orogeny, platform, foreland basin.
        for lat in [16.0, 29.0, 42.0] {
            ev("deeptime:collision", -352.0, -348.0, lat, 208.0, 0.9, "B suture");
            ev("deeptime:orogeny_phase", -350.0, -305.0, lat, 208.0, 0.9, "B orogen");
            ev("deeptime:erosion_pulse", -305.0, -200.0, lat, 208.0, 0.4, "B orogen");
            ev("deeptime:marine_transgression", -462.0, -335.0, lat, 216.0, 0.5, "B platform");
        }
        for lat in [33.0, 42.0] {
            ev("deeptime:basin_subsidence", -345.0, -250.0, lat, 218.0, 0.6, "B foreland");
        }
        // C: the BIF shelf sea 2.1 Gyr ago, the rift, the sag basin and its inversion.
        ev("deeptime:marine_transgression", -2200.0, -2000.0, -30.0, 300.0, 0.6, "C BIF sea");
        ev("deeptime:marine_transgression", -2200.0, -2000.0, -33.0, 292.0, 0.6, "C BIF sea");
        for lon in [290.0, 301.0, 312.0] {
            ev("deeptime:rift_onset", -400.0, -396.0, -7.5, lon, 0.8, "C rift");
            ev("deeptime:basin_subsidence", -396.0, -262.0, -7.5, lon, 0.7, "C sag basin");
            ev("deeptime:orogeny_phase", -258.0, -245.0, -7.5, lon, 0.4, "C inversion");
        }
        out
    }

    pub fn km_per_deg(&self) -> f64 {
        self.km_per_deg
    }
}

impl FieldHistory for SyntheticHistory {
    fn grid(&self) -> &GridDesc {
        &self.grid
    }

    fn frame_count(&self) -> usize {
        self.times.len()
    }

    fn frame(&self, k: usize) -> Result<FieldFrame, CoreError> {
        let &(t, dt) = self
            .times
            .get(k)
            .ok_or_else(|| CoreError::InvalidArgument(format!("no synthetic frame {k}")))?;
        Ok(self.build_frame(t, dt))
    }

    fn names(&self) -> Vec<String> {
        let f = self.build_frame(self.params.t_end_years, 1.0);
        f.fields.keys().chain(f.clocks.keys()).cloned().collect()
    }

    fn receivers(&self) -> Option<&[u32]> {
        Some(&self.receivers)
    }
}

/// D8 steepest descent on `elev` (x wraps): each cell's lowest lower neighbour, itself for a
/// pit and for sea cells (elevation below 0).
pub fn steepest_descent(grid: &GridDesc, elev: &[f32]) -> Vec<u32> {
    let (w, h) = (i64::from(grid.w), i64::from(grid.h));
    (0..grid.cells())
        .into_par_iter()
        .map(|cell| {
            let (i, j) = (cell as i64 % w, cell as i64 / w);
            if elev[cell] < 0.0 {
                return cell as u32;
            }
            let mut best = (cell, 0.0f32);
            for dj in -1..=1i64 {
                for di in -1..=1i64 {
                    let jj = j + dj;
                    if (di == 0 && dj == 0) || jj < 0 || jj >= h {
                        continue;
                    }
                    let n = (jj * w + (i + di).rem_euclid(w)) as usize;
                    let dist = if di != 0 && dj != 0 { std::f32::consts::SQRT_2 } else { 1.0 };
                    let slope = (elev[cell] - elev[n]) / dist;
                    if slope > best.1 {
                        best = (n, slope);
                    }
                }
            }
            best.0 as u32
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn small() -> SyntheticHistory {
        SyntheticHistory::new(SyntheticParams {
            seed: 7,
            circumference: 4096,
            grid_r: 16,
            t_start_years: -5.0e8,
            t_end_years: 0.0,
            snapshot_every_years: 2.5e7,
        })
        .unwrap()
    }

    #[test]
    fn frames_cover_the_timeline_and_every_field() {
        let h = small();
        assert_eq!(h.frame_count(), 20);
        let last = h.frame(19).unwrap();
        assert_eq!(last.t_years, 0.0);
        assert_eq!(h.frame(0).unwrap().t_years, -4.75e8);
        let names = h.names();
        for s in fields::FIELDS.iter().chain(fields::CLOCKS) {
            assert!(names.iter().any(|n| n == s.name), "missing {}", s.name);
        }
        for (name, v) in &last.fields {
            assert_eq!(v.len(), h.grid().cells(), "{name}");
            assert!(v.iter().all(|x| x.is_finite()), "{name}");
        }
        // The arc is active somewhere at 280 Myr ago and nowhere 400 Myr ago.
        let arc_at = |t: f64| {
            let k = h.times.iter().position(|(tt, _)| *tt >= t).unwrap();
            let f = h.frame(k).unwrap();
            f.fields["deeptime:arc_magma_flux"]
                .iter()
                .fold(0.0f32, |a, b| a.max(*b))
        };
        assert!(arc_at(-2.8e8) > 0.5);
        assert_eq!(arc_at(-4.0e8), 0.0);
    }

    #[test]
    fn receivers_descend() {
        let h = small();
        let elev = h.elevation(0.0);
        for (c, &r) in h.receivers().unwrap().iter().enumerate() {
            assert!(elev[r as usize] <= elev[c]);
        }
    }
}
