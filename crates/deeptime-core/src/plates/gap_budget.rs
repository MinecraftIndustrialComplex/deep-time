// SPDX-License-Identifier: LGPL-3.0-or-later
//! The land's gap budget: measurement only, no rule reads it (STATUS follow-up of 2026-10-17,
//! "which process lowers a continental column's surface faster than it thins it").
//!
//! # What it measures
//! Every sample's gap g = z − z_A: its elevation over the Airy height of its column,
//! [`super::collapse::iso_height`] (z_0, the middle of `continent_elevation_m`, for the normal
//! column, plus (T − T_n) / 7.0, less the eclogite root's pull). On a planet where height kept
//! step with thickness the continental gap would stay near zero; on `earthlike_quick_16k` at
//! 500 Myr the 30 to 55 km crust stands 0.5 to 3 km below its Airy height (build log
//! 2026-10-16). The budget says which process made that gap.
//!
//! # How
//! Each sample keeps its gap split into parts that add up to it. Every process of a plate step
//! is bracketed: after it, each sample's change of gap goes to that process's part (to the
//! sample's ocean share as well while it is oceanic crust). The surface stage's feedback,
//! applied at the start of the step, is split by what moved the surface: the rock moved
//! (erosion and deposition, which change the column's thickness too), its local isostatic
//! rebound, the rest of the flexure, the plateau spreading, and the carry (whatever else the
//! feedback holds: the semi-Lagrangian advection of the surface on L1, the L0/L1 sampling and the
//! clamps). The surface stage writes the per-cell parts as fields ([`field`]) while the budget is
//! on. A resampling blends the parts with the same weights as the column it blends, books the
//! gap its blending adds (the gap is not linear in the pools) and the gap of the crust it accretes
//! (thickness with no height). A sample born later (new sea floor) starts with its gap in the
//! `birth` part.
//!
//! The budget is off unless [`ON_ENV`] is set; it changes nothing the model does.

use rayon::prelude::*;

use super::collapse::iso_height;
use super::state::{CONTINENTAL, Sample, pool};

/// Set to switch the budget on.
pub const ON_ENV: &str = "DEEPTIME_GAP_BUDGET";

/// Whether the environment asks for the budget (`PlatesStage::new` reads it; the surface stage
/// writes its fields while the plate state holds a budget).
pub fn enabled() -> bool {
    std::env::var_os(ON_ENV).is_some()
}

/// The surface stage's parts of the feedback, per L1 cell, m (written only while the plate state
/// holds a budget; the plate model removes them when it applies the feedback).
pub mod field {
    /// The local isostatic response to the step's erosion and deposition: the local isostasy's
    /// share and the flexure's local (Airy) answer to the same load.
    pub const REBOUND: &str = "surface/gap_rebound_m";
    /// The rest of the flexure's change: regional bending, the plate model's roots, water, ice,
    /// the unsolved residual.
    pub const FLEXURE: &str = "surface/gap_flexure_m";
    /// The plateau spreading's change (`spread.rs`).
    pub const SPREAD: &str = "surface/gap_spread_m";
}

/// What changed a gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The gap when the sample was made: the start of the run, or new sea floor.
    Birth = 0,
    /// Surface: the rock erosion removed and deposition laid down (the surface and the column's
    /// thickness change together).
    Erosion,
    /// Surface: the local isostatic rebound of that rock.
    Rebound,
    /// Surface: the rest of the flexure.
    Flexure,
    /// Surface: the plateau spreading.
    Spread,
    /// Surface: the rest of the feedback (advection on L1, sampling, clamps).
    Carry,
    /// `subduction::resolve`.
    Subduction,
    /// `shortening::advance`.
    Shortening,
    /// `belts::stack`.
    Belts,
    /// `subduction::fronts_and_uplift`: arcs and Andean margins.
    Arcs,
    /// `collapse::founder`.
    Founder,
    /// `collapse::step`.
    Collapse,
    /// `transition::step`.
    Transition,
    /// `collision::collide`.
    Collision,
    /// `rifting::heal`, `maybe_rift`, `maybe_initiate_subduction`.
    Rifting,
    /// `hotspots::step`.
    Hotspots,
    /// The plate model's own surface step (`pools::surface_step`).
    PlateSurface,
    /// `finish_samples`.
    Finish,
    /// `subduction::close_contacts`, `ridges::bury`.
    Bury,
    /// A resampling's blend of the old samples.
    ResampleBlend,
    /// A resampling's accretion of the crust under the top cover.
    ResampleAccrete,
    /// Anything else (the motion).
    Other,
}

/// The number of parts.
pub const PARTS: usize = 22;

/// The parts' names, in order.
pub const NAMES: [&str; PARTS] = [
    "birth",
    "erosion",
    "rebound",
    "flexure",
    "spread",
    "carry",
    "subduction",
    "shortening",
    "belts",
    "arcs",
    "founder",
    "collapse",
    "transition",
    "collision",
    "rifting",
    "hotspots",
    "plate_surface",
    "finish",
    "bury",
    "resample_blend",
    "resample_accrete",
    "other",
];

/// The thickness bands, km (the crust bins of `deeptime run`, the two thickest merged).
pub const EDGES_KM: [f64; 6] = [0.0, 30.0, 40.0, 55.0, 70.0, f64::INFINITY];

/// The number of bands.
pub const BANDS: usize = EDGES_KM.len() - 1;

/// The band of a thickness in m.
pub fn band(thickness_m: f32) -> usize {
    let t = f64::from(thickness_m) / 1000.0;
    (0..BANDS).find(|&b| t < EDGES_KM[b + 1]).unwrap_or(BANDS - 1)
}

/// One sample's account.
#[derive(Clone, Debug, PartialEq)]
struct Track {
    parts: [f32; PARTS],
    /// The part of the gap made while the sample was oceanic crust (its birth included).
    ocean: f32,
    z: f32,
    iso: f32,
    continental: bool,
}

/// How a resampling made a sample: the top cover's old corners and weights, and the pools it
/// blended before any accretion. `None`: new crust.
#[derive(Clone, Copy, Debug)]
pub struct Trace {
    pub tri: [u32; 3],
    pub w: [f64; 3],
    pub pools: [f32; pool::COUNT],
}

/// One band's state at a snapshot: visible continental samples.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BandStock {
    pub n: u64,
    /// Means, m.
    pub z: f64,
    pub iso: f64,
    pub gap: f64,
    /// The median gap, m.
    pub gap_p50: f64,
    /// The mean ocean share, m.
    pub ocean: f64,
    /// The mean of each part, m.
    pub parts: [f64; PARTS],
    /// Since the last snapshot: each process's change of the gap per column of the band, m/Myr
    /// (summed over the samples in the band after the process, over the samples in the band at
    /// the steps' ends).
    pub rate: [f64; PARTS],
}

/// The budget at one time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub t_myr: f64,
    /// The share of the visible continental samples in each band.
    pub bands: Vec<BandStock>,
}

/// The budget (`PlateState::gap_budget`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GapBudget {
    z_0: f64,
    gravity: f64,
    track: Vec<Track>,
    /// Since the last snapshot: Σ change of gap per band and part, m (one sample's change once
    /// per process), and the samples per band at the steps' ends.
    flux: Vec<[f64; PARTS]>,
    sample_steps: Vec<f64>,
    /// Times the sample count changed outside a resampling (the accounts were started again).
    pub restarts: u32,
    pub snapshots: Vec<Snapshot>,
}

fn gap_of(s: &Sample, z_0: f64, gravity: f64) -> (f32, f32) {
    (
        s.elevation_m,
        iso_height(&s.pools, z_0, gravity) as f32,
    )
}

impl GapBudget {
    /// Starts the accounts: every sample's gap is its birth.
    pub fn start(samples: &[Sample], z_0: f64, gravity: f64) -> GapBudget {
        let mut b = GapBudget {
            z_0,
            gravity,
            track: Vec::new(),
            flux: vec![[0.0; PARTS]; BANDS],
            sample_steps: vec![0.0; BANDS],
            restarts: 0,
            snapshots: Vec::new(),
        };
        b.begin(samples);
        b
    }

    fn begin(&mut self, samples: &[Sample]) {
        let (z_0, g) = (self.z_0, self.gravity);
        self.track = samples
            .par_iter()
            .map(|s| {
                let (z, iso) = gap_of(s, z_0, g);
                let mut parts = [0.0; PARTS];
                parts[Part::Birth as usize] = z - iso;
                Track {
                    parts,
                    ocean: if s.kind == CONTINENTAL { 0.0 } else { z - iso },
                    z,
                    iso,
                    continental: s.kind == CONTINENTAL,
                }
            })
            .collect();
    }

    /// Brings every sample's account up to date, `split` dividing its change (dz, the elevation's;
    /// diso, the Airy height's) among the parts; the band sums take the same.
    fn update<F>(&mut self, samples: &[Sample], split: F)
    where
        F: Fn(usize, f64, f64, &mut [f64; PARTS]) + Sync,
    {
        if samples.len() != self.track.len() {
            self.restarts += 1;
            self.begin(samples);
            return;
        }
        let (z_0, g) = (self.z_0, self.gravity);
        let sums = self
            .track
            .par_iter_mut()
            .zip(samples.par_iter())
            .enumerate()
            .fold(
                || vec![[0.0f64; PARTS]; BANDS],
                |mut acc, (v, (t, s))| {
                    let (z, iso) = gap_of(s, z_0, g);
                    let (dz, diso) = (f64::from(z - t.z), f64::from(iso - t.iso));
                    if dz == 0.0 && diso == 0.0 && t.continental == (s.kind == CONTINENTAL) {
                        return acc;
                    }
                    let mut d = [0.0f64; PARTS];
                    split(v, dz, diso, &mut d);
                    let total: f64 = d.iter().sum();
                    for (p, x) in t.parts.iter_mut().zip(&d) {
                        *p += *x as f32;
                    }
                    if !t.continental {
                        t.ocean += total as f32;
                    }
                    if s.kind == CONTINENTAL && !s.is_hidden() {
                        let row = &mut acc[band(s.thickness_m)];
                        for (a, x) in row.iter_mut().zip(&d) {
                            *a += x;
                        }
                    }
                    t.z = z;
                    t.iso = iso;
                    t.continental = s.kind == CONTINENTAL;
                    acc
                },
            )
            .reduce(
                || vec![[0.0f64; PARTS]; BANDS],
                |mut a, b| {
                    for (ra, rb) in a.iter_mut().zip(&b) {
                        for (x, y) in ra.iter_mut().zip(rb) {
                            *x += y;
                        }
                    }
                    a
                },
            );
        for (ra, rb) in self.flux.iter_mut().zip(&sums) {
            for (x, y) in ra.iter_mut().zip(rb) {
                *x += y;
            }
        }
    }

    /// Books every sample's change of gap since the last booking to `part`.
    pub fn book(&mut self, samples: &[Sample], part: Part) {
        self.update(samples, |_, dz, diso, d| d[part as usize] = dz - diso);
    }

    /// Books the surface feedback just applied: per sample, the material it took (m, + laid
    /// down) and the rebound, flexure and spreading parts of its elevation change (m); the rest of
    /// the elevation change is the carry.
    pub fn book_surface(
        &mut self,
        samples: &[Sample],
        material: &[f64],
        rebound: &[f64],
        flexure: &[f64],
        spread: &[f64],
    ) {
        self.update(samples, |v, dz, diso, d| {
            let (m, r, f, s) = (material[v], rebound[v], flexure[v], spread[v]);
            d[Part::Erosion as usize] = m - diso;
            d[Part::Rebound as usize] = r;
            d[Part::Flexure as usize] = f;
            d[Part::Spread as usize] = s;
            d[Part::Carry as usize] = dz - m - r - f - s;
        });
    }

    /// After a resampling: the new samples' accounts from the old ones (`traces`, one per new
    /// sample).
    pub fn resampled(&mut self, samples: &[Sample], traces: &[Option<Trace>]) {
        let (z_0, g) = (self.z_0, self.gravity);
        let old = std::mem::take(&mut self.track);
        let rows: Vec<(Track, [f64; PARTS])> = samples
            .par_iter()
            .zip(traces.par_iter())
            .map(|(s, tr)| {
                let (z, iso) = gap_of(s, z_0, g);
                let continental = s.kind == CONTINENTAL;
                let mut d = [0.0f64; PARTS];
                let Some(tr) = tr.filter(|tr| tr.tri.iter().all(|&i| (i as usize) < old.len()))
                else {
                    let mut parts = [0.0; PARTS];
                    parts[Part::Birth as usize] = z - iso;
                    let t = Track {
                        parts,
                        ocean: if continental { 0.0 } else { z - iso },
                        z,
                        iso,
                        continental,
                    };
                    return (t, d);
                };
                let mut parts = [0.0f64; PARTS];
                let mut ocean = 0.0f64;
                let mut was = 0.0f64;
                for k in 0..3 {
                    let o = &old[tr.tri[k] as usize];
                    for (p, x) in parts.iter_mut().zip(&o.parts) {
                        *p += tr.w[k] * f64::from(*x);
                    }
                    ocean += tr.w[k] * f64::from(o.ocean);
                    was += tr.w[k] * f64::from(o.z - o.iso);
                }
                let blended = f64::from(z) - iso_height(&tr.pools, z_0, g);
                d[Part::ResampleBlend as usize] = blended - was;
                d[Part::ResampleAccrete as usize] = f64::from(z - iso) - blended;
                for (p, x) in parts.iter_mut().zip(&d) {
                    *p += x;
                }
                if !continental {
                    ocean += d.iter().sum::<f64>();
                }
                let t = Track {
                    parts: parts.map(|x| x as f32),
                    ocean: ocean as f32,
                    z,
                    iso,
                    continental,
                };
                (t, d)
            })
            .collect();
        for ((_, d), s) in rows.iter().zip(samples) {
            if s.kind == CONTINENTAL && !s.is_hidden() {
                for (a, x) in self.flux[band(s.thickness_m)].iter_mut().zip(d) {
                    *a += x;
                }
            }
        }
        self.track = rows.into_iter().map(|(t, _)| t).collect();
    }

    /// The end of a step: counts the samples in each band (the denominator of the rates).
    pub fn end_step(&mut self, samples: &[Sample]) {
        for s in samples {
            if s.kind == CONTINENTAL && !s.is_hidden() {
                self.sample_steps[band(s.thickness_m)] += 1.0;
            }
        }
    }

    /// Records the budget at `t_myr` (steps of `dt_myr`), and starts the next window of rates.
    pub fn snapshot(&mut self, samples: &[Sample], t_myr: f64, dt_myr: f64) {
        let mut bands = vec![BandStock::default(); BANDS];
        let mut gaps: Vec<Vec<f64>> = vec![Vec::new(); BANDS];
        if samples.len() == self.track.len() {
            for (s, t) in samples.iter().zip(&self.track) {
                if s.kind != CONTINENTAL || s.is_hidden() {
                    continue;
                }
                let b = band(s.thickness_m);
                let st = &mut bands[b];
                st.n += 1;
                st.z += f64::from(t.z);
                st.iso += f64::from(t.iso);
                st.gap += f64::from(t.z - t.iso);
                st.ocean += f64::from(t.ocean);
                for (a, x) in st.parts.iter_mut().zip(&t.parts) {
                    *a += f64::from(*x);
                }
                gaps[b].push(f64::from(t.z - t.iso));
            }
        }
        for (b, st) in bands.iter_mut().enumerate() {
            let n = st.n.max(1) as f64;
            st.z /= n;
            st.iso /= n;
            st.gap /= n;
            st.ocean /= n;
            st.parts.iter_mut().for_each(|x| *x /= n);
            let g = &mut gaps[b];
            g.sort_by(f64::total_cmp);
            st.gap_p50 = g.get(g.len() / 2).copied().unwrap_or(f64::NAN);
            let per = self.sample_steps[b].max(1.0) * dt_myr;
            for (r, f) in st.rate.iter_mut().zip(&self.flux[b]) {
                *r = f / per;
            }
        }
        self.snapshots.push(Snapshot { t_myr, bands });
        self.flux = vec![[0.0; PARTS]; BANDS];
        self.sample_steps = vec![0.0; BANDS];
    }

    /// The largest difference between a sample's parts and its gap, m (a check: the parts add up).
    pub fn worst_residual_m(&self) -> f64 {
        self.track
            .iter()
            .map(|t| {
                let sum: f64 = t.parts.iter().map(|&x| f64::from(x)).sum();
                (sum - f64::from(t.z - t.iso)).abs()
            })
            .fold(0.0, f64::max)
    }

    /// The summary lines for `deeptime run` and `stage`: the part names, then per snapshot and
    /// band (visible continental crust) the stock (means, m) and the window's rates (m/Myr).
    pub fn lines(&self) -> String {
        let mut out = format!(
            "\ngap_budget z0_m={:.0} restarts={} worst_residual_m={:.1} parts={}",
            self.z_0,
            self.restarts,
            self.worst_residual_m(),
            NAMES.join(",")
        );
        for snap in &self.snapshots {
            let total: u64 = snap.bands.iter().map(|b| b.n).sum();
            for (b, st) in snap.bands.iter().enumerate() {
                let fmt = |v: &[f64; PARTS], prec: usize| {
                    v.iter()
                        .map(|x| format!("{x:.prec$}"))
                        .collect::<Vec<_>>()
                        .join("/")
                };
                out += &format!(
                    "\ngap_stock t={:.0} band={}-{}km n={:.3} z={:.0} iso={:.0} gap={:.0} gap_p50={:.0} ocean={:.0} parts={}",
                    snap.t_myr,
                    EDGES_KM[b],
                    EDGES_KM[b + 1].min(999.0),
                    st.n as f64 / total.max(1) as f64,
                    st.z,
                    st.iso,
                    st.gap,
                    st.gap_p50,
                    st.ocean,
                    fmt(&st.parts, 0),
                );
                out += &format!(
                    "\ngap_rate t={:.0} band={}-{}km m_per_myr={}",
                    snap.t_myr,
                    EDGES_KM[b],
                    EDGES_KM[b + 1].min(999.0),
                    fmt(&st.rate, 2),
                );
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::pools::{COLUMN_ISOSTATIC_FACTOR as F, continental_column};
    use crate::plates::state::OCEANIC;

    fn column(z: f32, extra_m: f32) -> Sample {
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.pools = continental_column();
        s.pools[pool::METAMORPHIC] += extra_m;
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = z;
        s
    }

    /// The parts add up to the gap through bookings, a surface split and a resampling, and each
    /// change lands in the part it was booked to.
    #[test]
    fn the_parts_add_up_to_the_gap() {
        let g = 9.81;
        let mut samples = vec![column(525.0, 0.0), column(1000.0, 7000.0), column(0.0, 0.0)];
        samples[2].kind = OCEANIC;
        let mut b = GapBudget::start(&samples, 525.0, g);
        assert!(b.track[0].parts[Part::Birth as usize].abs() < 1.0);
        // Thickening with no height: the gap falls by T / 7.0.
        samples[0].pools[pool::METAMORPHIC] += 7000.0;
        samples[0].thickness_m += 7000.0;
        b.book(&samples, Part::ResampleAccrete);
        let accreted = b.track[0].parts[Part::ResampleAccrete as usize];
        assert!((f64::from(accreted) + 7000.0 / F).abs() < 1.0, "{accreted}");
        // Erosion of 1 km with a rebound of 800 m: the surface falls 200 m, the Airy height
        // 1000 / 7.0 m.
        samples[1].pools[pool::METAMORPHIC] -= 1000.0;
        samples[1].thickness_m -= 1000.0;
        samples[1].elevation_m -= 200.0;
        b.book_surface(
            &samples,
            &[0.0, -1000.0, 0.0],
            &[0.0, 800.0, 0.0],
            &[0.0; 3],
            &[0.0; 3],
        );
        let t = &b.track[1];
        let erosion = t.parts[Part::Erosion as usize] + t.parts[Part::Rebound as usize];
        assert!((f64::from(erosion) - (-200.0 + 1000.0 / F)).abs() < 1.0, "{erosion}");
        assert!(t.parts[Part::Carry as usize].abs() < 0.5);
        // The oceanic sample's change goes to its ocean share as well.
        samples[2].elevation_m -= 100.0;
        b.book(&samples, Part::PlateSurface);
        assert!((b.track[2].ocean - (b.track[2].z - b.track[2].iso)).abs() < 1.0);
        assert!(b.worst_residual_m() < 1.0);
        // A resampling onto two samples: one blended half and half from the first two, one new.
        let mut blended = samples[0];
        let pools: [f32; pool::COUNT] =
            std::array::from_fn(|k| 0.5 * (samples[0].pools[k] + samples[1].pools[k]));
        blended.pools = pools;
        blended.pools[pool::SEDIMENT] += 500.0;
        blended.thickness_m = blended.pools.iter().sum();
        blended.elevation_m = 0.5 * (samples[0].elevation_m + samples[1].elevation_m);
        let mut fresh = samples[2];
        fresh.elevation_m = -2500.0;
        let new = vec![blended, fresh];
        b.resampled(
            &new,
            &[
                Some(Trace {
                    tri: [0, 1, 2],
                    w: [0.5, 0.5, 0.0],
                    pools,
                }),
                None,
            ],
        );
        assert!(b.worst_residual_m() < 1.0, "{}", b.worst_residual_m());
        let acc = b.track[0].parts[Part::ResampleAccrete as usize];
        let expect = 0.5 * f64::from(accreted) - 500.0 / F;
        assert!((f64::from(acc) - expect).abs() < 1.0, "{acc} vs {expect}");
        b.end_step(&new);
        b.snapshot(&new, 2.0, 2.0);
        assert_eq!(b.snapshots[0].bands.iter().map(|s| s.n).sum::<u64>(), 1);
        assert!(b.lines().contains("gap_stock t=2"));
    }
}
