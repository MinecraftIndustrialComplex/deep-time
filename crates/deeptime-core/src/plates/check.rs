// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime plate-check`: the plate dynamics of a finished run (`dynamics.rs`) against the target
//! ranges of `docs/design/plate-collisions-and-welding.md` §6.
//!
//! [`MEASURES`] is the one place the ranges live (the design's table is their source; a calibration
//! task moves them there, or in a file given to the check, [`Targets::apply_json`]). A measure is
//! computed per run ([`measures`]); the check takes the mean over the runs it is given (the design
//! asserts the plate targets on the mean of seeds 1–4) and passes it when it lies inside its range,
//! both ends included. A measure with no range is reported only. A measure the run cannot give
//! (no collision, no plate that died) is *no data*, which fails a ranged measure: a run too short
//! to measure something has not shown it holds.
//!
//! Every share is of the planet's L0 samples unless the measure says "plates" (then of the
//! visible planet, as the design measured them). The definitions follow the throwaway trace the
//! design's numbers came from (`tools/review/plate-dynamics-trace.patch`, now removed), so a
//! baseline on master reproduces the design's "Now" column.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::dynamics::{Dynamics, PlateLife, PlateStep};
use super::stats::Margins;

/// One measure: its key, label, unit, range (inclusive; `None` is open) and how it is taken.
#[derive(Clone, Copy, Debug)]
pub struct MeasureDef {
    pub key: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
    pub lo: Option<f64>,
    pub hi: Option<f64>,
}

const fn m(
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    lo: Option<f64>,
    hi: Option<f64>,
) -> MeasureDef {
    MeasureDef {
        key,
        label,
        unit,
        lo,
        hi,
    }
}

/// The measures in table order: first the ones with the design's §6 ranges, then the ones only
/// reported (the design's §2.2 figures and what a force-balance plate model will be judged on).
pub const MEASURES: &[MeasureDef] = &[
    m(
        "largest_share_p50",
        "Largest plate, median share of the planet",
        "share",
        Some(0.15),
        Some(0.35),
    ),
    m(
        "largest_over_50_pct",
        "Steps with one plate above 50 %",
        "%",
        None,
        Some(10.0),
    ),
    m(
        "plates_over_5pct_p50",
        "Plates above 5 % of the planet, median",
        "plates",
        Some(5.0),
        Some(9.0),
    ),
    m(
        "plate_life_p50_myr",
        "Median plate lifetime (plates that died)",
        "Myr",
        Some(80.0),
        None,
    ),
    m(
        "dead_within_20_pct",
        "Plates dead within 20 Myr",
        "%",
        None,
        Some(15.0),
    ),
    m(
        "rift_alive_50_pct",
        "Rift-born plates alive after 50 Myr",
        "%",
        Some(60.0),
        None,
    ),
    m(
        "rift_welded_back_50_pct",
        "Rift-born plates welded back to their own rift family within 50 Myr",
        "%",
        None,
        Some(10.0),
    ),
    m(
        "collision_land_max_pct",
        "Land raised by one collision in one step, the most",
        "% of planet",
        None,
        Some(0.5),
    ),
    m(
        "land_gain_max_pct",
        "Most land gained in any step (after the first 10 Myr)",
        "% of planet",
        None,
        Some(1.5),
    ),
    m(
        "belt_width_km_p50",
        "Collision belt width, median (ground raised 100 m or more)",
        "km",
        None,
        Some(800.0),
    ),
    m(
        "belt_elongation_p50",
        "Collision belt elongation, median",
        "ratio",
        Some(4.0),
        None,
    ),
    m(
        "collision_length_p50_myr",
        "Collision length, onset to weld or parting, median",
        "Myr",
        Some(15.0),
        Some(50.0),
    ),
    m(
        "collision_length_p90_myr",
        "Collision length, 90th percentile",
        "Myr",
        None,
        Some(120.0),
    ),
    m(
        "uplift_spread",
        "Peak uplift per collision: 90th ÷ 10th percentile",
        "ratio",
        Some(2.0),
        None,
    ),
    m(
        "collisions_per_100myr",
        "Collisions per 100 Myr",
        "per 100 Myr",
        Some(2.0),
        Some(6.0),
    ),
    m(
        "continental_min",
        "Continental crust, share of the planet, lowest step",
        "share",
        Some(0.37),
        None,
    ),
    m(
        "continental_max",
        "Continental crust, share of the planet, highest step",
        "share",
        None,
        Some(0.45),
    ),
    m(
        "active_margin_mean",
        "Active share of the continental margin (after the first 100 Myr)",
        "share",
        Some(0.25),
        Some(0.40),
    ),
    m(
        "ocean_age_p90_myr",
        "Ocean floor, 90th percentile age, end of the run",
        "Myr",
        None,
        Some(230.0),
    ),
    // Reported only: the design's §2.2 figures.
    m(
        "largest_over_30_pct",
        "Steps with one plate above 30 %",
        "%",
        None,
        None,
    ),
    m(
        "largest_share_max",
        "Largest plate, worst step",
        "share",
        None,
        None,
    ),
    m(
        "plates_alive_p50",
        "Plates alive, median",
        "plates",
        None,
        None,
    ),
    m(
        "megaplate_regrow_myr_p50",
        "After a rift splits a plate above half the planet: until one is above half again, median",
        "Myr",
        None,
        None,
    ),
    m(
        "rift_life_p50_myr",
        "Rift-born plates that died: median lifetime",
        "Myr",
        None,
        None,
    ),
    m(
        "deaths_sutured",
        "Plate deaths by welding",
        "per run",
        None,
        None,
    ),
    m(
        "deaths_absorbed",
        "Plate deaths by absorption of a tiny plate",
        "per run",
        None,
        None,
    ),
    m(
        "deaths_consumed",
        "Plate deaths subducted away",
        "per run",
        None,
        None,
    ),
    m(
        "welded_dying_larger_pct",
        "Welds in which the dying plate is the larger one",
        "%",
        None,
        None,
    ),
    m(
        "land_share_p50",
        "Land, median share of the planet",
        "share",
        None,
        None,
    ),
    m(
        "continental_end",
        "Continental crust at the end, share of the planet",
        "share",
        None,
        None,
    ),
    m(
        "collision_lifted10_pct_p50",
        "Ground one collision lifts by more than 10 m: median",
        "% of planet",
        None,
        None,
    ),
    m(
        "collision_lifted10_pct_max",
        "Ground one collision lifts by more than 10 m: the most",
        "% of planet",
        None,
        None,
    ),
    m(
        "belt_length_km_p50",
        "Collision belt length, median (ground raised 100 m or more)",
        "km",
        None,
        None,
    ),
    m(
        "belt_elongation_over_3_pct",
        "Collisions whose belt is at least 3 times longer than wide",
        "%",
        None,
        None,
    ),
    m(
        "contact_age_p50_myr",
        "Contact age at the collision, median",
        "Myr",
        None,
        None,
    ),
    m(
        "contact_age_p90_myr",
        "Contact age at the collision, 90th percentile",
        "Myr",
        None,
        None,
    ),
    m(
        "counted_km_p50",
        "What the collision counter read, median",
        "km",
        None,
        None,
    ),
    m(
        "shortening_km_p50",
        "Real shortening at the collision, median",
        "km",
        None,
        None,
    ),
    m(
        "shortening_km_p90",
        "Real shortening at the collision, 90th percentile",
        "km",
        None,
        None,
    ),
    m(
        "ocean_age_p90_mean_myr",
        "Ocean floor, 90th percentile age, mean after the first 100 Myr",
        "Myr",
        None,
        None,
    ),
    // Reported only: what a force-balance plate model is judged on.
    m(
        "speed_p50_mm_yr",
        "Plate speed, median over plates and steps",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_slab_mm_yr",
        "Speed of plates with a slab on 20 % of their boundary or more",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_noslab_mm_yr",
        "Speed of plates with a slab on less than 20 % of their boundary",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_slab_ratio",
        "Slab-attached ÷ other plate speed (Earth: 3 to 4)",
        "ratio",
        None,
        None,
    ),
    m(
        "speed_slab_corr",
        "Correlation of plate speed with slab share of the boundary",
        "r",
        None,
        None,
    ),
    m(
        "slab_share_mean",
        "Slab share of the boundary, mean over plates and steps",
        "share",
        None,
        None,
    ),
    m(
        "speed_oceanic_mm_yr",
        "Speed of mostly oceanic plates (under half continental)",
        "mm/yr",
        None,
        None,
    ),
    m(
        "speed_continental_mm_yr",
        "Speed of mostly continental plates",
        "mm/yr",
        None,
        None,
    ),
];

/// Target ranges by measure key: the built-in ones, with any overrides applied.
#[derive(Clone, Debug)]
pub struct Targets {
    ranges: BTreeMap<&'static str, (Option<f64>, Option<f64>)>,
}

impl Targets {
    /// The design's §6 ranges ([`MEASURES`]).
    pub fn builtin() -> Targets {
        Targets {
            ranges: MEASURES.iter().map(|d| (d.key, (d.lo, d.hi))).collect(),
        }
    }

    /// The range of `key`.
    pub fn range(&self, key: &str) -> (Option<f64>, Option<f64>) {
        self.ranges.get(key).copied().unwrap_or((None, None))
    }

    /// Overrides from a JSON object `{"key": [lo, hi], …}`; `null` leaves an end open (and
    /// `"key": null` turns the measure into a reported one). Unknown keys are an error.
    pub fn apply_json(&mut self, text: &str) -> Result<(), String> {
        let v: Value = serde_json::from_str(text).map_err(|e| format!("targets file: {e}"))?;
        let obj = v
            .as_object()
            .ok_or("targets file: expected an object of measure -> [lo, hi]")?;
        for (key, range) in obj {
            let Some(slot) = self.ranges.get_mut(key.as_str()) else {
                return Err(format!(
                    "targets file: no measure {key:?} (keys: {})",
                    MEASURES
                        .iter()
                        .map(|d| d.key)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            };
            *slot = match range {
                Value::Null => (None, None),
                Value::Array(a) if a.len() == 2 => {
                    let end = |x: &Value| -> Result<Option<f64>, String> {
                        match x {
                            Value::Null => Ok(None),
                            _ => x
                                .as_f64()
                                .map(Some)
                                .ok_or_else(|| format!("targets file: {key}: {x} is not a number")),
                        }
                    };
                    (end(&a[0])?, end(&a[1])?)
                }
                _ => return Err(format!("targets file: {key}: expected [lo, hi] or null")),
            };
        }
        Ok(())
    }
}

fn median(xs: &[f64]) -> f64 {
    pct(xs, 0.5)
}

/// The `q` quantile (linear between the two nearest values), NaN of nothing.
fn pct(xs: &[f64], q: f64) -> f64 {
    let mut v: Vec<f64> = xs.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let k = (v.len() - 1) as f64 * q;
    let (lo, hi) = (k.floor() as usize, k.ceil() as usize);
    v[lo] + (v[hi] - v[lo]) * (k - lo as f64)
}

fn mean(xs: impl IntoIterator<Item = f64>) -> f64 {
    let (s, n) = xs
        .into_iter()
        .filter(|x| x.is_finite())
        .fold((0.0, 0u32), |(s, n), x| (s + x, n + 1));
    if n == 0 { f64::NAN } else { s / f64::from(n) }
}

/// Shares of the visible planet of one step's plates, largest first.
fn shares(plates: &[PlateStep]) -> Vec<f64> {
    let total: u32 = plates.iter().map(|p| p.visible).sum();
    let mut s: Vec<f64> = plates
        .iter()
        .map(|p| f64::from(p.visible) / f64::from(total.max(1)))
        .collect();
    s.sort_by(|a, b| b.total_cmp(a));
    s
}

fn percent_of_steps(d: &Dynamics, f: impl Fn(&[f64]) -> bool) -> f64 {
    if d.steps.is_empty() {
        return f64::NAN;
    }
    let n = d.steps.iter().filter(|s| f(&shares(&s.plates))).count();
    100.0 * n as f64 / d.steps.len() as f64
}

/// The plate's life in Myr, for plates that died.
fn lives(d: &Dynamics) -> Vec<(&PlateLife, f64)> {
    d.plates
        .iter()
        .filter_map(|p| p.died_myr.map(|t| (p, t - p.born_myr)))
        .collect()
}

/// Rift-born plates welded back into the plate family they left (the plate holding their
/// parent's crust, or a sibling of the same rift) within `window_myr` of birth, among those born
/// at least `window_myr` before the end of the run (so none is cut off by it), as a percentage.
fn rift_welded_back(d: &Dynamics, window_myr: f64, end_myr: f64) -> f64 {
    // Plate ids change hands at every weld: follow them to the plate that holds the crust now.
    let mut succ: BTreeMap<u32, u32> = BTreeMap::new();
    fn holder(succ: &BTreeMap<u32, u32>, mut p: u32) -> u32 {
        let mut hops = 0;
        while let Some(&n) = succ.get(&p) {
            p = n;
            hops += 1;
            if hops > 100_000 {
                break;
            }
        }
        p
    }
    let mut dead: Vec<&PlateLife> = d.plates.iter().filter(|p| p.died_myr.is_some()).collect();
    dead.sort_by(|a, b| {
        a.died_myr
            .partial_cmp(&b.died_myr)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.id.cmp(&b.id))
    });
    let mut back: BTreeSet<u32> = BTreeSet::new();
    for p in dead {
        let died = p.died_myr.unwrap_or(f64::NAN);
        if p.origin == "rift" && p.fate.as_deref() == Some("sutured") {
            let siblings: BTreeSet<u32> = d
                .plates
                .iter()
                .filter(|q| {
                    q.id != p.id
                        && q.origin == "rift"
                        && q.born_myr == p.born_myr
                        && q.parent == p.parent
                })
                .map(|q| holder(&succ, q.id))
                .collect();
            if let (Some(into), Some(parent)) = (p.into, p.parent)
                && (into == holder(&succ, parent) || siblings.contains(&into))
                && died - p.born_myr <= window_myr
            {
                back.insert(p.id);
            }
        }
        if let Some(into) = p.into {
            succ.insert(p.id, into);
        }
    }
    let old: Vec<&PlateLife> = d
        .plates
        .iter()
        .filter(|p| p.origin == "rift" && p.born_myr <= end_myr - window_myr)
        .collect();
    if old.is_empty() {
        return f64::NAN;
    }
    100.0 * old.iter().filter(|p| back.contains(&p.id)).count() as f64 / old.len() as f64
}

/// The active share of the continental margin, mean over the records after the first 90 Myr of
/// the record (P1-27; all of them when the run is shorter): the margins line of `deeptime run`.
pub fn active_margin_mean(margins: &[Margins]) -> f64 {
    let Some(first) = margins.first() else {
        return f64::NAN;
    };
    let after: Vec<&Margins> = margins
        .iter()
        .filter(|m| m.t_myr - first.t_myr >= 90.0)
        .collect();
    let pool: Vec<&Margins> = if after.is_empty() {
        margins.iter().collect()
    } else {
        after
    };
    mean(pool.iter().map(|m| m.active_share()))
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len().min(y.len());
    if n < 3 {
        return f64::NAN;
    }
    let (mx, my) = (mean(x.iter().copied()), mean(y.iter().copied()));
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx).powi(2);
        syy += (b - my).powi(2);
    }
    if sxx <= 0.0 || syy <= 0.0 {
        f64::NAN
    } else {
        sxy / (sxx * syy).sqrt()
    }
}

/// Every measure of one run, by key (NaN where the run cannot give it).
pub fn measures(d: &Dynamics, margins: &[Margins]) -> BTreeMap<&'static str, f64> {
    let mut out: BTreeMap<&'static str, f64> = BTreeMap::new();
    let mut put = |k: &'static str, v: f64| {
        out.insert(k, v);
    };
    let n = f64::from(d.samples.max(1));
    let first_t = d.steps.first().map_or(d.t0_myr, |s| s.t);
    let end_t = d.steps.last().map_or(d.t0_myr, |s| s.t);
    let span = end_t - d.t0_myr;

    // Plate sizes.
    let largest: Vec<f64> = d
        .steps
        .iter()
        .map(|s| shares(&s.plates).first().copied().unwrap_or(0.0))
        .collect();
    put("largest_share_p50", median(&largest));
    put(
        "largest_share_max",
        largest.iter().copied().fold(f64::NAN, f64::max),
    );
    put(
        "largest_over_50_pct",
        percent_of_steps(d, |s| s.first().is_some_and(|&x| x > 0.5)),
    );
    put(
        "largest_over_30_pct",
        percent_of_steps(d, |s| s.first().is_some_and(|&x| x > 0.3)),
    );
    let big: Vec<f64> = d
        .steps
        .iter()
        .map(|s| shares(&s.plates).iter().filter(|&&x| x >= 0.05).count() as f64)
        .collect();
    put("plates_over_5pct_p50", median(&big));
    // A rift that splits a plate above half the planet: how long until one plate is above half.
    let mut regrow = Vec::new();
    for i in 1..largest.len() {
        let rifted = d
            .plates
            .iter()
            .any(|p| p.origin == "rift" && (p.born_myr - d.steps[i].t).abs() < 1e-6);
        if rifted
            && largest[i - 1] > 0.5
            && largest[i] <= 0.5
            && let Some(j) = (i..largest.len()).find(|&j| largest[j] > 0.5)
        {
            regrow.push((j - i) as f64 * d.dt_myr);
        }
    }
    put("megaplate_regrow_myr_p50", median(&regrow));
    let alive: Vec<f64> = d.steps.iter().map(|s| s.plates.len() as f64).collect();
    put("plates_alive_p50", median(&alive));

    // Lifetimes and fates.
    let dead = lives(d);
    let life: Vec<f64> = dead.iter().map(|&(_, l)| l).collect();
    put("plate_life_p50_myr", median(&life));
    put(
        "dead_within_20_pct",
        if life.is_empty() {
            f64::NAN
        } else {
            100.0 * life.iter().filter(|&&l| l <= 20.0).count() as f64 / life.len() as f64
        },
    );
    let rift_life: Vec<f64> = dead
        .iter()
        .filter(|(p, _)| p.origin == "rift")
        .map(|&(_, l)| l)
        .collect();
    put("rift_life_p50_myr", median(&rift_life));
    let old_rift: Vec<&PlateLife> = d
        .plates
        .iter()
        .filter(|p| p.origin == "rift" && p.born_myr <= end_t - 50.0)
        .collect();
    put(
        "rift_alive_50_pct",
        if old_rift.is_empty() {
            f64::NAN
        } else {
            let alive_50 = old_rift
                .iter()
                .filter(|p| p.died_myr.is_none_or(|t| t - p.born_myr > 50.0))
                .count();
            100.0 * alive_50 as f64 / old_rift.len() as f64
        },
    );
    put("rift_welded_back_50_pct", rift_welded_back(d, 50.0, end_t));
    let fate = |f: &str| {
        d.plates
            .iter()
            .filter(|p| p.fate.as_deref() == Some(f))
            .count() as f64
    };
    put("deaths_sutured", fate("sutured"));
    put("deaths_absorbed", fate("absorbed"));
    put("deaths_consumed", fate("consumed"));
    // Of the collisions that welded a plate whole: how often the plate that died was the larger.
    let welds: Vec<&super::dynamics::CollisionRec> =
        d.collisions.iter().filter(|c| c.sutured).collect();
    put(
        "welded_dying_larger_pct",
        if welds.is_empty() {
            f64::NAN
        } else {
            let larger = welds
                .iter()
                .filter(|c| c.under_samples > c.over_samples)
                .count();
            100.0 * larger as f64 / welds.len() as f64
        },
    );

    // Land and continental crust.
    let land: Vec<f64> = d.steps.iter().map(|s| f64::from(s.land) / n).collect();
    put("land_share_p50", median(&land));
    let gains: Vec<f64> = d
        .steps
        .windows(2)
        .filter(|w| w[1].t >= first_t + 10.0)
        .map(|w| (f64::from(w[1].land) - f64::from(w[0].land)) / n)
        .collect();
    put(
        "land_gain_max_pct",
        100.0 * gains.iter().copied().fold(f64::NAN, f64::max),
    );
    let cont: Vec<f64> = d
        .steps
        .iter()
        .map(|s| f64::from(s.continental) / n)
        .collect();
    put(
        "continental_min",
        cont.iter().copied().fold(f64::NAN, f64::min),
    );
    put(
        "continental_max",
        cont.iter().copied().fold(f64::NAN, f64::max),
    );
    put("continental_end", cont.last().copied().unwrap_or(f64::NAN));

    // Collisions: terranes of at least `collision_min_frac` of an average plate.
    let col: Vec<&super::dynamics::CollisionRec> =
        d.collisions.iter().filter(|c| c.collision).collect();
    let pick = |f: &dyn Fn(&super::dynamics::CollisionRec) -> f64| -> Vec<f64> {
        col.iter().map(|c| f(c)).collect()
    };
    let emerged = pick(&|c| 100.0 * f64::from(c.emerged.samples) / n);
    put(
        "collision_land_max_pct",
        emerged.iter().copied().fold(f64::NAN, f64::max),
    );
    let lifted = pick(&|c| 100.0 * f64::from(c.lifted10.samples) / n);
    put("collision_lifted10_pct_p50", median(&lifted));
    put(
        "collision_lifted10_pct_max",
        lifted.iter().copied().fold(f64::NAN, f64::max),
    );
    let shaped: Vec<&super::dynamics::CollisionRec> = col
        .iter()
        .copied()
        .filter(|c| c.lifted100.width_km > 0.0)
        .collect();
    put(
        "belt_width_km_p50",
        median(
            &shaped
                .iter()
                .map(|c| c.lifted100.width_km)
                .collect::<Vec<_>>(),
        ),
    );
    put(
        "belt_length_km_p50",
        median(
            &shaped
                .iter()
                .map(|c| c.lifted100.length_km)
                .collect::<Vec<_>>(),
        ),
    );
    let elong: Vec<f64> = shaped.iter().map(|c| c.lifted100.elongation()).collect();
    put("belt_elongation_p50", median(&elong));
    put(
        "belt_elongation_over_3_pct",
        if elong.is_empty() {
            f64::NAN
        } else {
            100.0 * elong.iter().filter(|&&e| e >= 3.0).count() as f64 / elong.len() as f64
        },
    );
    let length = pick(&|c| c.end_myr - c.start_myr + d.dt_myr);
    put("collision_length_p50_myr", median(&length));
    put("collision_length_p90_myr", pct(&length, 0.9));
    let peak = pick(&|c| c.peak_uplift_m);
    let (p10, p90) = (pct(&peak, 0.1), pct(&peak, 0.9));
    put(
        "uplift_spread",
        if p10 > 0.0 { p90 / p10 } else { f64::NAN },
    );
    put(
        "collisions_per_100myr",
        if span > 0.0 {
            100.0 * col.len() as f64 / span
        } else {
            f64::NAN
        },
    );
    let age = pick(&|c| c.start_myr - c.contact_onset_myr);
    put("contact_age_p50_myr", median(&age));
    put("contact_age_p90_myr", pct(&age, 0.9));
    put("counted_km_p50", median(&pick(&|c| c.counted_km)));
    let short: Vec<f64> = col
        .iter()
        .filter(|c| c.front_km > 0.0)
        .map(|c| c.shortening_km)
        .collect();
    put("shortening_km_p50", median(&short));
    put("shortening_km_p90", pct(&short, 0.9));

    // Margins and the sea floor.
    put("active_margin_mean", active_margin_mean(margins));
    put(
        "ocean_age_p90_myr",
        d.ages.last().map_or(f64::NAN, |a| a.p90_myr),
    );
    let first_age_t = d.ages.first().map_or(0.0, |a| a.t);
    let late: Vec<f64> = d
        .ages
        .iter()
        .filter(|a| a.t - first_age_t >= 90.0)
        .map(|a| a.p90_myr)
        .collect();
    put("ocean_age_p90_mean_myr", mean(late));

    // What a force-balance plate model is judged on: speeds against slabs and crust type.
    let all: Vec<&PlateStep> = d.steps.iter().flat_map(|s| s.plates.iter()).collect();
    let speed = |sel: &dyn Fn(&PlateStep) -> bool| -> f64 {
        mean(all.iter().filter(|p| sel(p)).map(|p| p.speed_mm_yr))
    };
    let slab = speed(&|p| p.slab_share() >= 0.2);
    let noslab = speed(&|p| p.slab_share() < 0.2);
    put(
        "speed_p50_mm_yr",
        median(&all.iter().map(|p| p.speed_mm_yr).collect::<Vec<_>>()),
    );
    put("speed_slab_mm_yr", slab);
    put("speed_noslab_mm_yr", noslab);
    put(
        "speed_slab_ratio",
        if noslab > 0.0 {
            slab / noslab
        } else {
            f64::NAN
        },
    );
    let xs: Vec<f64> = all.iter().map(|p| p.slab_share()).collect();
    let ys: Vec<f64> = all.iter().map(|p| p.speed_mm_yr).collect();
    put("speed_slab_corr", pearson(&xs, &ys));
    put("slab_share_mean", mean(xs.iter().copied()));
    put(
        "speed_oceanic_mm_yr",
        speed(&|p| p.continental_share() < 0.5),
    );
    put(
        "speed_continental_mm_yr",
        speed(&|p| p.continental_share() >= 0.5),
    );
    out
}

/// A run's data for the check.
#[derive(Clone, Debug)]
pub struct RunData {
    pub name: String,
    pub dynamics: Dynamics,
    pub margins: Vec<Margins>,
}

/// The verdict on one measure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    /// The runs could not give the measure, which has a range: counts as a failure.
    NoData,
    /// No range: reported only.
    Reported,
}

impl Status {
    pub fn word(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "FAIL",
            Status::NoData => "NO DATA",
            Status::Reported => "-",
        }
    }

    /// Whether this verdict fails the check.
    pub fn failed(self) -> bool {
        matches!(self, Status::Fail | Status::NoData)
    }
}

/// One line of the check's table.
#[derive(Clone, Debug)]
pub struct Row {
    pub def: MeasureDef,
    pub lo: Option<f64>,
    pub hi: Option<f64>,
    /// The mean over the runs that gave a value (NaN when none did).
    pub value: f64,
    /// Each run's value.
    pub runs: Vec<f64>,
    pub status: Status,
}

/// Judges `value` against a range (both ends included).
pub fn judge(value: f64, lo: Option<f64>, hi: Option<f64>) -> Status {
    if lo.is_none() && hi.is_none() {
        return Status::Reported;
    }
    if !value.is_finite() {
        return Status::NoData;
    }
    if lo.is_some_and(|l| value < l) || hi.is_some_and(|h| value > h) {
        Status::Fail
    } else {
        Status::Pass
    }
}

/// The check: every measure's mean over `runs` against `targets`.
pub fn evaluate(runs: &[RunData], targets: &Targets) -> Vec<Row> {
    let per_run: Vec<BTreeMap<&'static str, f64>> = runs
        .iter()
        .map(|r| measures(&r.dynamics, &r.margins))
        .collect();
    MEASURES
        .iter()
        .map(|def| {
            let values: Vec<f64> = per_run
                .iter()
                .map(|m| m.get(def.key).copied().unwrap_or(f64::NAN))
                .collect();
            let value = mean(values.iter().copied());
            let (lo, hi) = targets.range(def.key);
            Row {
                def: *def,
                lo,
                hi,
                value,
                runs: values,
                status: judge(value, lo, hi),
            }
        })
        .collect()
}

/// Whether any row fails.
pub fn failed(rows: &[Row]) -> bool {
    rows.iter().any(|r| r.status.failed())
}

fn num(x: f64) -> String {
    if !x.is_finite() {
        "n/a".to_owned()
    } else if x != 0.0 && x.abs() < 0.1 {
        format!("{x:.3}")
    } else if x.abs() < 10.0 {
        format!("{x:.2}")
    } else if x.abs() < 100.0 {
        format!("{x:.1}")
    } else {
        format!("{x:.0}")
    }
}

/// The range as the table prints it.
pub fn range_text(lo: Option<f64>, hi: Option<f64>) -> String {
    match (lo, hi) {
        (Some(l), Some(h)) => format!("{}-{}", num(l), num(h)),
        (Some(l), None) => format!("at least {}", num(l)),
        (None, Some(h)) => format!("at most {}", num(h)),
        (None, None) => "-".to_owned(),
    }
}

/// The table: measure, value, target range, pass or fail. With several runs the value is the mean
/// and a column lists the runs.
pub fn render(rows: &[Row]) -> String {
    let many = rows.first().is_some_and(|r| r.runs.len() > 1);
    let w0 = rows
        .iter()
        .map(|r| r.def.label.chars().count())
        .max()
        .unwrap_or(0);
    let mut s = String::new();
    let mut head = format!(
        "{:<w0$}  {:>9}  {:<14}  {:<8}",
        "measure", "value", "target", "result"
    );
    if many {
        head.push_str("  runs");
    }
    s.push_str(&head);
    s.push('\n');
    for r in rows {
        let label = format!("{} ({})", r.def.label, r.def.unit);
        let mut line = format!(
            "{:<w0$}  {:>9}  {:<14}  {:<8}",
            label,
            num(r.value),
            range_text(r.lo, r.hi),
            r.status.word()
        );
        if many {
            line.push_str("  ");
            line.push_str(&r.runs.iter().map(|&x| num(x)).collect::<Vec<_>>().join(" "));
        }
        s.push_str(&line);
        s.push('\n');
    }
    s
}

/// The rows as JSON (the check's `--json` file).
pub fn to_json(rows: &[Row], names: &[String]) -> Value {
    let any_failed = failed(rows);
    let measures: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "key": r.def.key,
                "label": r.def.label,
                "unit": r.def.unit,
                "value": r.value.is_finite().then_some(r.value),
                "lo": r.lo,
                "hi": r.hi,
                "status": r.status.word(),
                "runs": r.runs.iter().map(|&x| x.is_finite().then_some(x)).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "runs": names,
        "failed": any_failed,
        "measures": measures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::dynamics::{AgeRec, CollisionRec, Footprint, StepRec};

    const N: u32 = 1000;

    fn plate(
        id: u32,
        visible: u32,
        continental: u32,
        speed: f64,
        front: f64,
        bnd: f64,
    ) -> PlateStep {
        PlateStep {
            id,
            visible,
            continental,
            speed_mm_yr: speed,
            front_km: front,
            boundary_km: bnd,
        }
    }

    /// A step at `t` with plates of the given visible sample counts.
    fn step(t: f64, land: u32, continental: u32, sizes: &[u32]) -> StepRec {
        let plates: Vec<PlateStep> = sizes
            .iter()
            .enumerate()
            .map(|(i, &v)| plate(i as u32, v, 0, 40.0, 0.0, 1000.0))
            .collect();
        StepRec {
            t,
            visible: sizes.iter().sum(),
            land,
            continental,
            plates,
            ..StepRec::default()
        }
    }

    fn dynamics(steps: Vec<StepRec>) -> Dynamics {
        Dynamics {
            version: 1,
            t0_myr: steps.first().map_or(0.0, |s| s.t - 2.0),
            dt_myr: 2.0,
            samples: N,
            steps,
            ..Dynamics::default()
        }
    }

    fn life(id: u32, parent: Option<u32>, origin: &str, born: f64) -> PlateLife {
        PlateLife {
            id,
            parent,
            origin: origin.to_owned(),
            born_myr: born,
            ..PlateLife::default()
        }
    }

    fn dies(mut l: PlateLife, t: f64, fate: &str, into: Option<u32>) -> PlateLife {
        l.died_myr = Some(t);
        l.fate = Some(fate.to_owned());
        l.into = into;
        l
    }

    fn fp(samples: u32, length: f64, width: f64) -> Footprint {
        Footprint {
            samples,
            length_km: length,
            width_km: width,
        }
    }

    fn collision(start: f64, end: f64, peak: f64) -> CollisionRec {
        CollisionRec {
            collision: true,
            start_myr: start,
            end_myr: end,
            contact_onset_myr: start - 8.0,
            peak_uplift_m: peak,
            frac: 0.3,
            terrane_samples: 30,
            under_samples: 100,
            over_samples: 200,
            ..CollisionRec::default()
        }
    }

    fn get(m: &BTreeMap<&'static str, f64>, k: &str) -> f64 {
        *m.get(k).unwrap_or_else(|| panic!("no measure {k}"))
    }

    #[test]
    fn every_measure_is_computed_and_an_empty_run_fails() {
        let keys: BTreeSet<&str> = MEASURES.iter().map(|d| d.key).collect();
        assert_eq!(keys.len(), MEASURES.len(), "duplicate keys");
        let m = measures(&Dynamics::default(), &[]);
        for k in m.keys() {
            assert!(keys.contains(k), "{k} is computed but not in MEASURES");
        }
        for d in MEASURES {
            assert!(m.contains_key(d.key), "{} is never computed", d.key);
        }
        // Nothing recorded: a ranged measure has no data and fails; counts are zero.
        let rows = evaluate(
            &[RunData {
                name: "empty".into(),
                dynamics: Dynamics::default(),
                margins: vec![],
            }],
            &Targets::builtin(),
        );
        assert!(failed(&rows));
        assert!(rows.iter().all(|r| r.status != Status::Pass));
    }

    #[test]
    fn plate_sizes() {
        let d = dynamics(vec![
            step(2.0, 0, 0, &[600, 300, 100]),
            step(4.0, 0, 0, &[400, 400, 200]),
            step(6.0, 0, 0, &[560, 440]),
            step(8.0, 0, 0, &[800, 100, 50, 50]),
        ]);
        let m = measures(&d, &[]);
        // Largest shares 0.6, 0.4, 0.56, 0.8: median 0.58; three of four steps above 50 %, all
        // above 30 %.
        assert!((get(&m, "largest_share_p50") - 0.58).abs() < 1e-9);
        assert!((get(&m, "largest_share_max") - 0.8).abs() < 1e-9);
        assert_eq!(get(&m, "largest_over_50_pct"), 75.0);
        assert_eq!(get(&m, "largest_over_30_pct"), 100.0);
        // Plates of at least 5 %: 3, 3, 2, 4 (50 of 1000 is 5 %).
        assert_eq!(get(&m, "plates_over_5pct_p50"), 3.0);
        assert_eq!(get(&m, "plates_alive_p50"), 3.0);
    }

    #[test]
    fn a_split_megaplate_grows_back() {
        // Above half until 6, a rift at 8 splits it, above half again at 14 (6 Myr later) and
        // split once more at 20 (never above half again, which is not counted).
        let mut d = dynamics(vec![
            step(2.0, 0, 0, &[800, 200]),
            step(4.0, 0, 0, &[600, 400]),
            step(6.0, 0, 0, &[600, 400]),
            step(8.0, 0, 0, &[400, 300, 300]),
            step(10.0, 0, 0, &[450, 300, 250]),
            step(12.0, 0, 0, &[480, 300, 220]),
            step(14.0, 0, 0, &[520, 300, 180]),
            step(16.0, 0, 0, &[600, 400]),
            step(18.0, 0, 0, &[600, 400]),
            step(20.0, 0, 0, &[400, 300, 300]),
        ]);
        d.plates = vec![
            life(2, Some(0), "rift", 8.0),
            life(3, Some(0), "rift", 20.0),
            // A weld that happens to drop the largest share is not a rift.
            life(4, Some(0), "subduction_initiation", 12.0),
        ];
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "megaplate_regrow_myr_p50"), 6.0);
    }

    #[test]
    fn lifetimes_and_fates() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[500, 500])]);
        d.plates = vec![
            dies(life(0, None, "initial", 0.0), 100.0, "sutured", Some(1)),
            dies(life(2, Some(1), "rift", 10.0), 20.0, "consumed", None),
            dies(life(3, Some(1), "rift", 30.0), 40.0, "absorbed", Some(1)),
            dies(life(4, Some(1), "rift", 10.0), 200.0, "sutured", Some(1)),
            life(1, None, "initial", 0.0),
        ];
        let m = measures(&d, &[]);
        // Lives of the four that died: 100, 10, 10, 190.
        assert_eq!(get(&m, "plate_life_p50_myr"), 55.0);
        assert_eq!(get(&m, "dead_within_20_pct"), 50.0);
        assert_eq!(get(&m, "rift_life_p50_myr"), 10.0);
        assert_eq!(get(&m, "deaths_sutured"), 2.0);
        assert_eq!(get(&m, "deaths_absorbed"), 1.0);
        assert_eq!(get(&m, "deaths_consumed"), 1.0);
    }

    #[test]
    fn rift_born_plates_that_survive_or_are_welded_back() {
        let mut d = dynamics(vec![
            step(2.0, 0, 0, &[500, 500]),
            step(300.0, 0, 0, &[500, 500]),
        ]);
        d.plates = vec![
            life(0, None, "initial", 0.0),
            // A rift of plate 0 at 10 Myr: 1 goes back to its parent (after 15 Myr), 2 to a
            // stranger.
            dies(life(1, Some(0), "rift", 10.0), 25.0, "sutured", Some(0)),
            dies(life(2, Some(0), "rift", 10.0), 30.0, "sutured", Some(9)),
            // A rift at 50 Myr: 5 is welded to its sibling 6 after 20 Myr.
            dies(life(5, Some(0), "rift", 50.0), 70.0, "sutured", Some(6)),
            life(6, Some(0), "rift", 50.0),
            // One that lives on; one too young to judge (born within 50 Myr of the end).
            life(4, Some(0), "rift", 20.0),
            life(7, Some(0), "rift", 290.0),
            // Welded back to its parent, but only after 80 Myr: not "within 50 Myr".
            dies(life(8, Some(0), "rift", 100.0), 180.0, "sutured", Some(0)),
            life(9, None, "initial", 0.0),
        ];
        let m = measures(&d, &[]);
        // Judged (born at least 50 Myr before 300): 1, 2, 4, 5, 6, 8. Alive after 50 Myr: 4, 6, 8
        // (8 lived 80).
        assert!((get(&m, "rift_alive_50_pct") - 50.0).abs() < 1e-9);
        // Welded back within 50 Myr: 1 (parent) and 5 (sibling).
        assert!((get(&m, "rift_welded_back_50_pct") - 100.0 * 2.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn a_weld_follows_the_plate_that_holds_the_parents_crust() {
        // Plate 0 is welded into 3; plate 1, rifted from 0, is then welded into 3: back to the
        // plate that now holds its parent's crust.
        let mut d = dynamics(vec![step(300.0, 0, 0, &[500, 500])]);
        d.plates = vec![
            dies(life(0, None, "initial", 0.0), 5.0, "sutured", Some(3)),
            dies(life(1, Some(0), "rift", 0.0), 30.0, "sutured", Some(3)),
            life(3, None, "initial", 0.0),
        ];
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "rift_welded_back_50_pct"), 100.0);
    }

    #[test]
    fn land_gain_leaves_out_the_start_up() {
        let d = dynamics(vec![
            step(2.0, 100, 400, &[1000]),
            // The start-up: +50 samples (5 % of the planet) in the first 10 Myr.
            step(4.0, 150, 400, &[1000]),
            step(14.0, 150, 380, &[1000]),
            // After it: +12 samples = 1.2 %, then a loss.
            step(16.0, 162, 390, &[1000]),
            step(18.0, 120, 440, &[1000]),
        ]);
        let m = measures(&d, &[]);
        assert!((get(&m, "land_gain_max_pct") - 1.2).abs() < 1e-9);
        assert!((get(&m, "continental_min") - 0.38).abs() < 1e-9);
        assert!((get(&m, "continental_max") - 0.44).abs() < 1e-9);
        assert!((get(&m, "continental_end") - 0.44).abs() < 1e-9);
        assert!((get(&m, "land_share_p50") - 0.15).abs() < 1e-9);
    }

    #[test]
    fn collisions() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[1000]), step(1000.0, 0, 0, &[1000])]);
        d.t0_myr = 0.0;
        let mut a = collision(100.0, 100.0, 1400.0);
        a.emerged = fp(20, 500.0, 300.0);
        a.lifted10 = fp(250, 4000.0, 2000.0);
        a.lifted100 = fp(100, 3000.0, 1000.0);
        a.counted_km = 350.0;
        a.front_km = 600.0;
        a.shortening_km = 90.0;
        let mut b = collision(300.0, 330.0, 2800.0);
        b.emerged = fp(3, 100.0, 50.0);
        b.lifted10 = fp(80, 4000.0, 800.0);
        b.lifted100 = fp(40, 4000.0, 500.0);
        b.front_km = 1000.0;
        b.shortening_km = 200.0;
        let mut c = collision(600.0, 680.0, 700.0);
        c.lifted100 = fp(60, 5000.0, 800.0);
        c.lifted10 = fp(90, 5000.0, 1000.0);
        let mut transfer = collision(700.0, 700.0, 1400.0);
        transfer.collision = false;
        transfer.emerged = fp(400, 1000.0, 1000.0);
        d.collisions = vec![a, b, c, transfer];
        let m = measures(&d, &[]);
        // Only collisions count, not terrane transfers: 20 samples of 1000 = 2 %.
        assert!((get(&m, "collision_land_max_pct") - 2.0).abs() < 1e-9);
        assert!((get(&m, "collision_lifted10_pct_max") - 25.0).abs() < 1e-9);
        assert!((get(&m, "collision_lifted10_pct_p50") - 9.0).abs() < 1e-9);
        assert_eq!(get(&m, "belt_width_km_p50"), 800.0);
        assert_eq!(get(&m, "belt_length_km_p50"), 4000.0);
        // Elongations 3, 8, 6.25.
        assert_eq!(get(&m, "belt_elongation_p50"), 6.25);
        assert!((get(&m, "belt_elongation_over_3_pct") - 100.0).abs() < 1e-9);
        // Lengths: a one-step collision is one step (2 Myr); 32 and 82.
        assert_eq!(get(&m, "collision_length_p50_myr"), 32.0);
        assert!((get(&m, "collision_length_p90_myr") - 72.0).abs() < 1e-9);
        // Peak uplifts 700, 1400, 2800: 90th ÷ 10th percentile.
        assert!((get(&m, "uplift_spread") - 2520.0 / 840.0).abs() < 1e-9);
        // Three collisions in 1000 Myr.
        assert!((get(&m, "collisions_per_100myr") - 0.3).abs() < 1e-9);
        // Contact age 8 for all; the shortening of the two that have a front.
        assert_eq!(get(&m, "contact_age_p50_myr"), 8.0);
        assert_eq!(get(&m, "shortening_km_p50"), 145.0);
        assert_eq!(get(&m, "counted_km_p50"), 0.0);
    }

    #[test]
    fn welds_report_which_plate_was_larger() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[1000])]);
        let mut big = collision(10.0, 10.0, 1.0);
        big.sutured = true;
        let small = big.clone();
        d.collisions = vec![big, small];
        // The plate that dies has 100 samples against the receiver's 200 in both.
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "welded_dying_larger_pct"), 0.0);
        d.collisions[0].under_samples = 900;
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "welded_dying_larger_pct"), 50.0);
    }

    #[test]
    fn the_sea_floor_and_the_margins() {
        let mut d = dynamics(vec![step(2.0, 0, 0, &[1000])]);
        let age = |t: f64, p90: f64| AgeRec {
            t,
            samples: 100,
            p90_myr: p90,
            ..AgeRec::default()
        };
        d.ages = vec![age(10.0, 100.0), age(120.0, 180.0), age(200.0, 220.0)];
        let margin = |t: f64, subduction: u32, collision: u32| Margins {
            t_myr: t,
            coast: 100,
            subduction,
            collision,
            ..Margins::default()
        };
        // The records less than 90 Myr after the first are left out: 10, 50 | 100, 150.
        let margins = [
            margin(10.0, 60, 0),
            margin(50.0, 50, 0),
            margin(100.0, 20, 5),
            margin(150.0, 30, 0),
        ];
        let m = measures(&d, &margins);
        assert_eq!(get(&m, "ocean_age_p90_myr"), 220.0);
        assert_eq!(get(&m, "ocean_age_p90_mean_myr"), 200.0);
        assert!((get(&m, "active_margin_mean") - 0.275).abs() < 1e-9);
        // A short run uses all its records.
        assert!((active_margin_mean(&margins[..2]) - 0.55).abs() < 1e-9);
        assert!(active_margin_mean(&[]).is_nan());
    }

    #[test]
    fn plate_motion_against_slabs_and_crust() {
        // Slab-attached plates fast, free plates slow, the continental ones slowest.
        let mk = |t: f64| StepRec {
            t,
            visible: 1000,
            plates: vec![
                plate(0, 400, 0, 80.0, 500.0, 1000.0),
                plate(1, 300, 0, 70.0, 400.0, 1000.0),
                plate(2, 200, 150, 30.0, 0.0, 1000.0),
                plate(3, 100, 80, 20.0, 0.0, 500.0),
            ],
            ..StepRec::default()
        };
        let d = dynamics(vec![mk(2.0), mk(4.0), mk(6.0)]);
        let m = measures(&d, &[]);
        assert_eq!(get(&m, "speed_slab_mm_yr"), 75.0);
        assert_eq!(get(&m, "speed_noslab_mm_yr"), 25.0);
        assert_eq!(get(&m, "speed_slab_ratio"), 3.0);
        assert!(get(&m, "speed_slab_corr") > 0.9);
        assert!((get(&m, "slab_share_mean") - 0.225).abs() < 1e-9);
        assert_eq!(get(&m, "speed_oceanic_mm_yr"), 75.0);
        assert_eq!(get(&m, "speed_continental_mm_yr"), 25.0);
        assert_eq!(get(&m, "speed_p50_mm_yr"), 50.0);
    }

    #[test]
    fn judging_is_inclusive_and_a_missing_value_fails() {
        assert_eq!(judge(5.0, Some(5.0), Some(9.0)), Status::Pass);
        assert_eq!(judge(9.0, Some(5.0), Some(9.0)), Status::Pass);
        assert_eq!(judge(4.99, Some(5.0), Some(9.0)), Status::Fail);
        assert_eq!(judge(9.01, Some(5.0), Some(9.0)), Status::Fail);
        assert_eq!(judge(100.0, Some(5.0), None), Status::Pass);
        assert_eq!(judge(100.0, None, Some(5.0)), Status::Fail);
        assert_eq!(judge(f64::NAN, Some(5.0), None), Status::NoData);
        assert!(Status::NoData.failed() && Status::Fail.failed());
        assert!(!Status::Pass.failed() && !Status::Reported.failed());
        assert_eq!(judge(f64::NAN, None, None), Status::Reported);
    }

    fn run_with(sizes: &[u32]) -> RunData {
        RunData {
            name: "r".into(),
            dynamics: dynamics(vec![step(2.0, 0, 0, sizes), step(4.0, 0, 0, sizes)]),
            margins: vec![],
        }
    }

    fn row<'a>(rows: &'a [Row], key: &str) -> &'a Row {
        rows.iter().find(|r| r.def.key == key).unwrap()
    }

    #[test]
    fn the_check_takes_the_mean_of_the_runs_and_fails_outside_the_range() {
        // Largest shares 0.2 and 0.4: mean 0.3.
        let rows = evaluate(
            &[
                run_with(&[200, 200, 200, 200, 200]),
                run_with(&[400, 300, 300]),
            ],
            &Targets::builtin(),
        );
        let r = row(&rows, "largest_share_p50");
        assert_eq!(r.runs.len(), 2);
        assert!((r.value - 0.3).abs() < 1e-9, "{}", r.value);
        assert_eq!(r.status, Status::Pass);
        assert!(failed(&rows), "the other measures have no data here");
        // 0.6 alone is outside.
        let rows = evaluate(&[run_with(&[600, 400])], &Targets::builtin());
        assert_eq!(row(&rows, "largest_share_p50").status, Status::Fail);
        let text = render(&rows);
        assert!(text.contains("FAIL") && text.contains("0.15-0.35"));
        assert_eq!(text.lines().count(), MEASURES.len() + 1);
        let v = to_json(&rows, &["r".to_owned()]);
        assert_eq!(v["failed"], true);
        assert_eq!(v["measures"].as_array().unwrap().len(), MEASURES.len());
    }

    #[test]
    fn targets_can_be_moved_from_a_file() {
        let mut t = Targets::builtin();
        assert_eq!(t.range("largest_share_p50"), (Some(0.15), Some(0.35)));
        t.apply_json(r#"{"largest_share_p50": [0.2, null], "plate_life_p50_myr": null}"#)
            .unwrap();
        assert_eq!(t.range("largest_share_p50"), (Some(0.2), None));
        assert_eq!(t.range("plate_life_p50_myr"), (None, None));
        assert!(t.apply_json(r#"{"no_such_measure": [1, 2]}"#).is_err());
        assert!(t.apply_json(r#"{"largest_share_p50": [1]}"#).is_err());
        assert!(t.apply_json("[]").is_err());
        // The moved range is the one the check uses.
        let rows = evaluate(&[run_with(&[600, 400])], &t);
        let r = row(&rows, "largest_share_p50");
        assert_eq!((r.lo, r.hi), (Some(0.2), None));
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn percentiles_interpolate() {
        assert_eq!(pct(&[1.0, 2.0, 3.0, 4.0], 0.5), 2.5);
        assert_eq!(pct(&[1.0, 2.0, 3.0, 4.0], 0.0), 1.0);
        assert_eq!(pct(&[1.0, 2.0, 3.0, 4.0], 1.0), 4.0);
        assert!(pct(&[], 0.5).is_nan());
        assert!(pct(&[f64::NAN], 0.5).is_nan());
        assert!(pearson(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]).is_nan());
        assert!((pearson(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]) - 1.0).abs() < 1e-12);
    }
}
