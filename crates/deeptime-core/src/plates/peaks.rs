// SPDX-License-Identifier: LGPL-3.0-or-later
//! The peak trace: measurement only, off by default (the polar-peaks lane, 2026-10-23,
//! `docs/science/plates.md` "The polar peaks").
//!
//! With `DEEPTIME_PEAK_TRACE=<m>` set, every plate step prints to stderr each visible sample that
//! ends the step higher than `<m>` metres (at most [`MAX_LINES`] a step, the highest first): where
//! it is, what it is, its column, its Airy height and the settling's target, the surface's
//! flexure under it, where it stood before the surface's feedback and after it, and with the gap
//! budget on (`DEEPTIME_GAP_BUDGET`) the step's change of its elevation and thickness by process
//! and the largest parts of its gap. No rule reads any of it, and the world it makes is the same.

use super::Env;
use super::collapse::iso_height;
use super::gap_budget::{GapBudget, NAMES, PARTS};
use super::geometry::{self, Geo};
use super::settle::equilibrium_m;
use super::state::PlateState;

/// Set to a height, m, to switch the trace on.
pub const ENV: &str = "DEEPTIME_PEAK_TRACE";

/// The most samples one step prints.
pub const MAX_LINES: usize = 6;

/// The trace's threshold, m, when the environment asks for it.
pub fn threshold() -> Option<f64> {
    std::env::var(ENV)
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|x| x.is_finite())
}

/// What the step's start knew, per sample (empty when the trace is off).
#[derive(Clone, Debug, Default)]
pub struct Begin {
    /// The elevation before the surface's feedback and after it, m.
    pub z_pre: Vec<f32>,
    pub z0: Vec<f32>,
    /// The surface's flexure under each sample, m (NaN where no cell sees it).
    pub flexure: Vec<f64>,
}

/// One step's lines (see the module docs). `resampled`: the samples are new, so the step's start
/// no longer matches them.
pub fn print(st: &PlateState, geo: &Geo, env: &Env, sea: f64, begin: &Begin, resampled: bool) {
    let Some(th) = threshold() else { return };
    let k = env.k;
    let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
    let mut high: Vec<usize> = (0..st.samples.len())
        .filter(|&i| {
            let s = &st.samples[i];
            !s.is_hidden() && f64::from(s.elevation_m) > th
        })
        .collect();
    if high.is_empty() {
        return;
    }
    let n_high = high.len();
    high.sort_by(|&a, &b| {
        st.samples[b]
            .elevation_m
            .total_cmp(&st.samples[a].elevation_m)
            .then(a.cmp(&b))
    });
    high.truncate(MAX_LINES);
    let pos = geometry::positions(st, geo);
    let same = !resampled && begin.z0.len() == st.samples.len();
    let gb: Option<&GapBudget> = st.gap_budget.as_ref();
    for (rank, &i) in high.iter().enumerate() {
        let s = &st.samples[i];
        let (lat, lon) = crate::sphere::lat_lon_deg(pos[i]);
        let iso = iso_height(&s.pools, z_0, env.gravity);
        let (zp, z0, f) = if same {
            (
                f64::from(begin.z_pre[i]),
                f64::from(begin.z0[i]),
                begin.flexure.get(i).copied().unwrap_or(f64::NAN),
            )
        } else {
            (f64::NAN, f64::NAN, f64::NAN)
        };
        let target = equilibrium_m(iso, sea) + if f.is_finite() { f } else { 0.0 };
        let mut line = format!(
            "peak t={:.1} n_high={} rank={} v={} lat={:.2} lon={:.2} plate={} kind={} regime={} \
             flags={:#04x} orogeny={} orogeny_myr={:.1} T_km={:.2} z_pre={:.0} z0={:.0} z={:.0} \
             iso={:.0} F={:.0} target={:.0} uplift_mm_yr={:.3} crust_uplift_mm_yr={:.3} \
             conv_mm_yr={:.1} coll_mm_yr={:.1} resampled={}",
            env.t_myr,
            n_high,
            rank,
            i,
            lat,
            lon,
            st.plate_of(i),
            s.kind,
            s.regime,
            s.flags,
            s.orogeny,
            s.orogeny_myr,
            f64::from(s.thickness_m) / 1000.0,
            zp,
            z0,
            s.elevation_m,
            iso,
            f,
            target,
            s.uplift_mm_yr,
            s.crust_uplift_mm_yr,
            s.convergence_mm_yr,
            s.collision_mm_yr,
            u8::from(resampled),
        );
        if let Some((parts, step_dz, step_dt)) = gb.and_then(|g| g.sample_parts(i)) {
            let list = |v: &[f32; PARTS], min: f32, top: usize| {
                let mut idx: Vec<usize> = (0..PARTS).filter(|&p| v[p].abs() >= min).collect();
                idx.sort_by(|&a, &b| v[b].abs().total_cmp(&v[a].abs()));
                idx.truncate(top);
                idx.iter()
                    .map(|&p| format!("{}:{:.0}", NAMES[p], v[p]))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            line.push_str(&format!(
                " step_dz=[{}] step_dT=[{}] gap_parts=[{}]",
                list(step_dz, 20.0, 8),
                list(step_dt, 20.0, 6),
                list(parts, 200.0, 8)
            ));
        }
        eprintln!("{line}");
    }
}
