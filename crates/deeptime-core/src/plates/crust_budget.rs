// SPDX-License-Identifier: LGPL-3.0-or-later
//! The continental crust budget: measurement only, no rule reads it (build log 2026-10-19,
//! "mountains eat the world": where the continents' area and volume go, and why their land
//! drowns).
//!
//! # What it measures
//! The stock of continental crust over a run: its area (visible continental samples, as a share
//! of the planet) and its volume (Σ thickness, visible and hidden crust apart, km³
//! Earth-normalized); the land (visible samples above the plate datum, 0 m, and above the
//! climate's sea level); the dry share of the continental crust; its thickness distribution, each
//! band with its elevation (the freeboard) and its Airy height ([`super::collapse::iso_height`]);
//! the hypsometry (the land's share above 2 km) and the median distance from land to the coast.
//!
//! # How
//! Every process of a plate step is bracketed as for the gap budget (`gap_budget.rs`, the same
//! [`Part`]s): after it, the change of each stock goes to that process, and the changes of state
//! of single samples are counted (ocean floor made continental and back, with the elevation and
//! thickness of crust made continental; continental crust hidden under another plate or
//! surfacing; dry continental crust drowned, wet continental crust emerging). The surface stage's
//! feedback goes to the `erosion` part (all of its change: erosion, deposition, isostasy,
//! flexure, the sea), a resampling to `resample_blend`. A snapshot every
//! [`super::GAP_SNAPSHOT_MYR`] and at the end. With [`MAPS_ENV`] set to a folder, every snapshot
//! also writes the visible samples there (`t<Myr>.bin`, little-endian f32 records of latitude and
//! longitude in degrees, elevation (m), thickness (m), kind (0 oceanic, 1 continental)) for
//! `tools/review/continent-budget.py`.
//!
//! Off unless [`ON_ENV`] is set; it changes nothing the model does.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use glam::DVec3;
use rayon::prelude::*;

use super::collapse::iso_height;
use super::gap_budget::{NAMES, PARTS, Part};
use super::geometry::Tree;
use super::state::Sample;

/// Set to switch the budget on.
pub const ON_ENV: &str = "DEEPTIME_CRUST_BUDGET";
/// Set to a folder to have every snapshot write the visible samples there.
pub const MAPS_ENV: &str = "DEEPTIME_CRUST_BUDGET_MAPS";

/// Whether the environment asks for the budget (`PlatesStage::new` reads it).
pub fn enabled() -> bool {
    std::env::var_os(ON_ENV).is_some()
}

/// The stocks, in order: visible continental samples, hidden continental samples, their volumes
/// (Σ thickness, m), visible samples above 0 m, visible continental samples above 0 m, Σ
/// elevation of the visible continental samples (m), visible samples.
const STOCKS: [&str; 8] = [
    "cont",
    "cont_hidden",
    "vol",
    "vol_hidden",
    "land",
    "dry",
    "z_cont",
    "vis",
];
const NT: usize = STOCKS.len();
const CONT: usize = 0;
const CONT_HIDDEN: usize = 1;
const VOL: usize = 2;
const VOL_HIDDEN: usize = 3;
const LAND: usize = 4;
const DRY: usize = 5;

/// The changes of state counted per sample: ocean floor made continental (count, Σ elevation,
/// Σ thickness, m), continental crust made oceanic, hidden, surfacing, drowned, emerging.
const MOVES: [&str; 8] = [
    "o2c", "o2c_z", "o2c_t", "c2o", "hide", "surface", "drown", "emerge",
];
const NX: usize = MOVES.len();

const BIT_CONT: u8 = 1;
const BIT_HIDDEN: u8 = 2;
const BIT_DRY: u8 = 4;

/// The thickness bands of the snapshots, km.
// ledger: PLT-293 (game-choice: reporting bands, read by no rule; agent, 2026-10-19).
pub const EDGES_KM: [f64; 10] = [
    0.0,
    20.0,
    25.0,
    30.0,
    35.0,
    40.0,
    45.0,
    55.0,
    70.0,
    f64::INFINITY,
];

/// The globals each snapshot copies (cumulative over the run, km³ or km² Earth-normalized).
const GLOBALS: [(&str, &str); 20] = [
    ("foundered", super::collapse::FOUNDERED_GLOBAL),
    ("rate_lost", super::rate::LOST_GLOBAL),
    ("crossed", super::belts::CROSSED_GLOBAL),
    ("kept", super::belts::KEPT_GLOBAL),
    ("accreted", super::ridges::ACCRETED_GLOBAL),
    ("stacked", super::ridges::STACKED_CONSUMED_GLOBAL),
    (
        "accreted_continent",
        super::ridges::ACCRETED_HIDDEN_CONTINENT_GLOBAL,
    ),
    ("dropped", super::ridges::DROPPED_GLOBAL),
    ("mafic_dropped", super::ridges::MAFIC_DROPPED_GLOBAL),
    ("collapse_moved", super::collapse::MOVED_GLOBAL),
    ("collapse_gained_km2", super::collapse::GAINED_GLOBAL),
    ("collapse_held", super::collapse::HELD_GLOBAL),
    ("collapse_unplaced", super::collapse::UNPLACED_GLOBAL),
    ("channel_flow", super::flow::MOVED_GLOBAL),
    ("settle_rise", super::settle::RISE_GLOBAL),
    ("settle_fall", super::settle::FALL_GLOBAL),
    ("arc_c", super::subduction::ARC_CONTINENTAL_GLOBAL),
    ("arc_o", super::subduction::ARC_OCEANIC_GLOBAL),
    ("cordillera", super::subduction::CORDILLERA_GLOBAL),
    ("cone_felsic", super::CONE_FELSIC_GLOBAL),
];

/// Earth's radius, km: the distances to the coast are Earth-normalized.
const EARTH_KM: f64 = 6371.0;

fn bits(s: &Sample) -> u8 {
    let mut b = 0;
    if s.is_continental() {
        b |= BIT_CONT;
    }
    if s.is_hidden() {
        b |= BIT_HIDDEN;
    }
    if s.elevation_m > 0.0 {
        b |= BIT_DRY;
    }
    b
}

fn stocks(samples: &[Sample]) -> [f64; NT] {
    samples
        .par_iter()
        .fold(
            || [0.0f64; NT],
            |mut a, s| {
                let t = f64::from(s.thickness_m);
                let dry = s.elevation_m > 0.0;
                match (s.is_continental(), s.is_hidden()) {
                    (true, false) => {
                        a[CONT] += 1.0;
                        a[VOL] += t;
                        a[DRY] += f64::from(u8::from(dry));
                        a[6] += f64::from(s.elevation_m);
                    }
                    (true, true) => {
                        a[CONT_HIDDEN] += 1.0;
                        a[VOL_HIDDEN] += t;
                    }
                    _ => {}
                }
                if !s.is_hidden() {
                    a[LAND] += f64::from(u8::from(dry));
                    a[7] += 1.0;
                }
                a
            },
        )
        .reduce(
            || [0.0; NT],
            |mut a, b| {
                for (x, y) in a.iter_mut().zip(b) {
                    *x += y;
                }
                a
            },
        )
}

/// The budget (`PlatesStage`, while [`ON_ENV`] is set).
#[derive(Clone, Debug)]
pub struct CrustBudget {
    z_0: f64,
    gravity: f64,
    /// The stocks at the last booking.
    last: [f64; NT],
    /// Every sample's state bits at the last booking.
    bits: Vec<u8>,
    /// Per part, the change of each stock over the run.
    parts: Vec<[f64; NT]>,
    /// Per part, the changes of state over the run.
    moves: Vec<[f64; NX]>,
    /// The snapshots' lines.
    lines: Vec<String>,
    maps: Option<PathBuf>,
}

impl CrustBudget {
    /// Starts the budget on the run's first samples.
    pub fn start(samples: &[Sample], z_0: f64, gravity: f64) -> CrustBudget {
        CrustBudget {
            z_0,
            gravity,
            last: stocks(samples),
            bits: samples.par_iter().map(bits).collect(),
            parts: vec![[0.0; NT]; PARTS],
            moves: vec![[0.0; NX]; PARTS],
            lines: Vec::new(),
            maps: std::env::var_os(MAPS_ENV).map(PathBuf::from),
        }
    }

    /// Whether no snapshot was taken yet.
    pub fn fresh(&self) -> bool {
        self.lines.is_empty()
    }

    /// Books every change since the last booking to `part`.
    pub fn book(&mut self, samples: &[Sample], part: Part) {
        let now = stocks(samples);
        let p = part as usize;
        for (k, x) in self.parts[p].iter_mut().enumerate() {
            *x += now[k] - self.last[k];
        }
        self.last = now;
        if samples.len() != self.bits.len() {
            // A resampling: the samples are new, their changes of state are not single moves.
            self.bits = samples.par_iter().map(bits).collect();
            return;
        }
        let moved = self
            .bits
            .par_iter_mut()
            .zip(samples.par_iter())
            .fold(
                || [0.0f64; NX],
                |mut m, (b, s)| {
                    let (was, now) = (*b, bits(s));
                    if was == now {
                        return m;
                    }
                    *b = now;
                    let (c0, c1) = (was & BIT_CONT != 0, now & BIT_CONT != 0);
                    let (h0, h1) = (was & BIT_HIDDEN != 0, now & BIT_HIDDEN != 0);
                    let (d0, d1) = (was & BIT_DRY != 0, now & BIT_DRY != 0);
                    if !c0 && c1 {
                        m[0] += 1.0;
                        m[1] += f64::from(s.elevation_m);
                        m[2] += f64::from(s.thickness_m);
                    }
                    if c0 && !c1 {
                        m[3] += 1.0;
                    }
                    if c0 && c1 {
                        if !h0 && h1 {
                            m[4] += 1.0;
                        }
                        if h0 && !h1 {
                            m[5] += 1.0;
                        }
                        if !h0 && !h1 {
                            if d0 && !d1 {
                                m[6] += 1.0;
                            }
                            if !d0 && d1 {
                                m[7] += 1.0;
                            }
                        }
                    }
                    m
                },
            )
            .reduce(
                || [0.0; NX],
                |mut a, b| {
                    for (x, y) in a.iter_mut().zip(b) {
                        *x += y;
                    }
                    a
                },
            );
        for (x, y) in self.moves[p].iter_mut().zip(moved) {
            *x += y;
        }
    }

    /// Records the budget at `t_myr`: the samples at `pos`, a sample's area `area_km2`
    /// (Earth-normalized), the climate's sea level `sea_m`, the world's globals.
    pub fn snapshot(
        &mut self,
        samples: &[Sample],
        pos: &[DVec3],
        t_myr: f64,
        area_km2: f64,
        sea_m: f64,
        globals: &BTreeMap<String, f64>,
    ) {
        let (z_0, g) = (self.z_0, self.gravity);
        let s = stocks(samples);
        let vis = s[7].max(1.0);
        let sea = sea_m as f32;
        let visible = |i: usize| !samples[i].is_hidden();
        let land = |i: usize| visible(i) && samples[i].elevation_m > sea;
        let n_land_sl = (0..samples.len()).filter(|&i| land(i)).count() as f64;
        let above_2km = (0..samples.len())
            .filter(|&i| land(i) && samples[i].elevation_m > sea + 2000.0)
            .count() as f64;
        let ocean_land = (0..samples.len())
            .filter(|&i| land(i) && !samples[i].is_continental())
            .count() as f64;
        // The distance from each piece of land to the nearest visible sea floor.
        let coast_p50 = if pos.len() == samples.len() && n_land_sl > 0.0 {
            let sea_tree = Tree::over(pos, |i| visible(i) && samples[i].elevation_m <= sea);
            let mut d: Vec<f64> = (0..samples.len())
                .into_par_iter()
                .filter(|&i| land(i))
                .filter_map(|i| sea_tree.nearest(pos[i]))
                .map(|(_, c2)| 2.0 * (0.5 * c2.sqrt()).min(1.0).asin() * EARTH_KM)
                .collect();
            d.sort_by(f64::total_cmp);
            d.get(d.len() / 2).copied().unwrap_or(f64::NAN)
        } else {
            f64::NAN
        };
        let mut line = format!(
            "\ncb_t t={t_myr:.0} vis={:.0} cont={:.4} cont_hidden={:.4} vol_km3={:.4e} \
             vol_hidden_km3={:.4e} land0={:.4} land={:.4} sea_m={sea_m:.0} dry={:.4} \
             above_2km={:.4} ocean_land={:.4} coast_p50_km={coast_p50:.0} mean_t_km={:.2}",
            s[7],
            s[CONT] / vis,
            s[CONT_HIDDEN] / vis,
            s[VOL] * area_km2 / 1000.0,
            s[VOL_HIDDEN] * area_km2 / 1000.0,
            s[LAND] / vis,
            n_land_sl / vis,
            s[DRY] / s[CONT].max(1.0),
            above_2km / n_land_sl.max(1.0),
            ocean_land / n_land_sl.max(1.0),
            s[VOL] / s[CONT].max(1.0) / 1000.0,
        );
        // The thickness bands of the visible continental crust.
        let mut bands: Vec<Vec<(f32, f32, f32)>> = vec![Vec::new(); EDGES_KM.len() - 1];
        for x in samples
            .iter()
            .filter(|x| x.is_continental() && !x.is_hidden())
        {
            let t = f64::from(x.thickness_m) / 1000.0;
            let b = (0..bands.len())
                .find(|&b| t < EDGES_KM[b + 1])
                .unwrap_or(bands.len() - 1);
            bands[b].push((
                x.elevation_m,
                x.thickness_m,
                iso_height(&x.pools, z_0, g) as f32,
            ));
        }
        let _ = write!(line, "\ncb_bins t={t_myr:.0}");
        for (b, v) in bands.iter_mut().enumerate() {
            v.sort_by(|a, b| a.0.total_cmp(&b.0));
            let n = v.len() as f64;
            let q = |f: f64| {
                v.get(((v.len().max(1) - 1) as f64 * f).round() as usize)
                    .map_or(f64::NAN, |x| f64::from(x.0))
            };
            let vol: f64 = v.iter().map(|x| f64::from(x.1)).sum();
            let iso = v.iter().map(|x| f64::from(x.2)).sum::<f64>() / n.max(1.0);
            let dry = v.iter().filter(|x| x.0 > sea).count() as f64 / n.max(1.0);
            let _ = write!(
                line,
                " [{}-{}km a={:.4} v={:.4} z10/50/90={:.0}/{:.0}/{:.0} iso={:.0} dry={:.3}]",
                EDGES_KM[b],
                EDGES_KM[b + 1].min(999.0),
                n / s[CONT].max(1.0),
                vol / s[VOL].max(1.0),
                q(0.1),
                q(0.5),
                q(0.9),
                iso,
                dry,
            );
        }
        // The parts' changes over the run so far: area as a share of the planet, volume in
        // 10^6 km³, land and dry continental crust as shares of the planet.
        let _ = write!(line, "\ncb_parts t={t_myr:.0}");
        for (name, k, scale) in [
            ("cont", CONT, 1.0 / vis),
            ("cont_hidden", CONT_HIDDEN, 1.0 / vis),
            ("vol", VOL, area_km2 / 1000.0 / 1.0e6),
            ("vol_hidden", VOL_HIDDEN, area_km2 / 1000.0 / 1.0e6),
            ("land", LAND, 1.0 / vis),
            ("dry", DRY, 1.0 / vis),
        ] {
            let row: Vec<String> = (0..PARTS)
                .filter(|&p| self.parts[p][k] != 0.0)
                .map(|p| format!("{}:{:.5}", NAMES[p], self.parts[p][k] * scale))
                .collect();
            let _ = write!(line, " {name}=[{}]", row.join(","));
        }
        let _ = write!(line, "\ncb_moves t={t_myr:.0}");
        for p in 0..PARTS {
            let m = &self.moves[p];
            if m.iter().all(|&x| x == 0.0) {
                continue;
            }
            let _ = write!(
                line,
                " {}=[o2c={:.5} o2c_z={:.0} o2c_t_km={:.1} c2o={:.5} hide={:.5} surface={:.5} drown={:.5} emerge={:.5}]",
                NAMES[p],
                m[0] / vis,
                m[1] / m[0].max(1.0),
                m[2] / m[0].max(1.0) / 1000.0,
                m[3] / vis,
                m[4] / vis,
                m[5] / vis,
                m[6] / vis,
                m[7] / vis,
            );
        }
        let _ = write!(line, "\ncb_globals t={t_myr:.0}");
        for (name, key) in GLOBALS {
            let _ = write!(
                line,
                " {name}={:.4e}",
                globals.get(key).copied().unwrap_or(0.0)
            );
        }
        self.lines.push(line);
        if let Some(dir) = &self.maps {
            write_map(dir, samples, pos, t_myr);
        }
    }

    /// The summary lines for `deeptime run` and `stage`.
    pub fn lines(&self) -> String {
        let mut out = format!(
            "\ncrust_budget z0_m={:.0} stocks={} moves={} parts={}",
            self.z_0,
            STOCKS.join(","),
            MOVES.join(","),
            NAMES.join(",")
        );
        for l in &self.lines {
            out += l;
        }
        out
    }
}

/// Writes the visible samples to `<dir>/t<Myr>.bin` (measurement only; a failure is reported and
/// ignored).
fn write_map(dir: &std::path::Path, samples: &[Sample], pos: &[DVec3], t_myr: f64) {
    if pos.len() != samples.len() {
        return;
    }
    let mut buf: Vec<u8> = Vec::with_capacity(samples.len() * 20);
    for (s, p) in samples.iter().zip(pos) {
        if s.is_hidden() {
            continue;
        }
        let p = p.normalize_or_zero();
        let lat = p.z.clamp(-1.0, 1.0).asin().to_degrees();
        let lon = p.y.atan2(p.x).to_degrees();
        for x in [
            lat as f32,
            lon as f32,
            s.elevation_m,
            s.thickness_m,
            f32::from(u8::from(s.is_continental())),
        ] {
            buf.extend_from_slice(&x.to_le_bytes());
        }
    }
    let path = dir.join(format!("t{:04.0}.bin", t_myr.round()));
    if let Err(e) = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&path, &buf)) {
        eprintln!("crust_budget: cannot write {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::pools::continental_column;
    use crate::plates::state::{CONTINENTAL, HIDDEN, OCEANIC};

    fn column(kind: u8, z: f32) -> Sample {
        let mut s = Sample::blank(0);
        s.kind = kind;
        s.pools = continental_column();
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = z;
        s
    }

    /// Each change lands in the part it was booked to, and the moves are counted once.
    #[test]
    fn changes_land_in_their_parts() {
        let mut samples = vec![
            column(CONTINENTAL, 500.0),
            column(CONTINENTAL, 300.0),
            column(OCEANIC, -3000.0),
        ];
        let mut b = CrustBudget::start(&samples, 525.0, 9.81);
        samples[1].elevation_m = -100.0;
        b.book(&samples, Part::Erosion);
        samples[0].flags |= HIDDEN;
        b.book(&samples, Part::Subduction);
        samples[2].kind = CONTINENTAL;
        b.book(&samples, Part::Collapse);
        assert_eq!(b.moves[Part::Erosion as usize][6], 1.0, "drowned");
        assert_eq!(b.moves[Part::Subduction as usize][4], 1.0, "hidden");
        assert_eq!(b.moves[Part::Collapse as usize][0], 1.0, "made continental");
        assert_eq!(b.moves[Part::Collapse as usize][1], -3000.0);
        assert_eq!(b.parts[Part::Subduction as usize][CONT], -1.0);
        assert_eq!(b.parts[Part::Subduction as usize][CONT_HIDDEN], 1.0);
        assert_eq!(b.parts[Part::Erosion as usize][DRY], -1.0);
        let pos = vec![DVec3::X, DVec3::Y, DVec3::Z];
        b.snapshot(&samples, &pos, 2.0, 1.0, 0.0, &BTreeMap::new());
        let out = b.lines();
        assert!(out.contains("cb_t t=2"), "{out}");
        assert!(out.contains("collapse=[o2c="), "{out}");
    }
}
