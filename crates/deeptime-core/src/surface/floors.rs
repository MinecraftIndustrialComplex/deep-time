// SPDX-License-Identifier: LGPL-3.0-or-later
//! The floor trace: measurement only, off by default (the flexure-faults lane, 2026-10-23,
//! `docs/science/flexure.md` "The held offsets").
//!
//! With `DEEPTIME_FLOOR_TRACE=<m>` set, every surface step prints to stderr each cell that ends
//! the step deeper than `<m>` metres under the sea (at most [`MAX_LINES`] a step, the deepest
//! first): where it is, the plate model's height under it, the surface's offset over that at the
//! step's start (what the carry brought) and at its end, whether the surface holds the cell's
//! offset (a gap between plates, a trench row), the step's change of the ground by stage (the rock
//! moved, the local isostasy, the spreading, the flexure) and, with the gap budget on
//! (`DEEPTIME_GAP_BUDGET`), the flexure's change by what it answers, and the flexure's state at
//! the cell. No rule reads any of it, and the world it makes is the same.

use deeptime_format::GridDesc;

/// Set to a depth under the sea, m, to switch the trace on.
pub const ENV: &str = "DEEPTIME_FLOOR_TRACE";

/// The most cells one step prints.
pub const MAX_LINES: usize = 4;

/// The trace's threshold, m under the sea, when the environment asks for it.
pub fn threshold() -> Option<f64> {
    std::env::var(ENV)
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|x| x.is_finite())
}

/// What one step's lines read, per cell.
pub struct Step<'a> {
    pub t_myr: f64,
    pub grid: &'a GridDesc,
    pub sea: f64,
    /// The ground at the step's start (after the carry), after the rock moved, after the local
    /// isostasy, after the spreading (when known) and at the end, m.
    pub h_start: &'a [f64],
    pub h_eroded: Option<&'a [f64]>,
    pub h_before_spread: Option<&'a [f64]>,
    pub h_spread: Option<&'a [f64]>,
    pub h_end: &'a [f64],
    /// The plate model's height, m.
    pub plate_z: &'a [f32],
    /// The offset over it at the step's start (the carry's), m.
    pub offset_start: &'a [f32],
    /// 1 where the surface holds the offset (`advect::held_cells`), when known.
    pub held: Option<&'a [f32]>,
    pub gap: Option<&'a [f32]>,
    pub flags: &'a [f32],
    pub regime: &'a [f32],
    /// The gap budget's parts of the step's change (name, per cell), when on.
    pub parts: Vec<(&'static str, &'a [f32])>,
    /// The flexure's state after the step: load, deflection, the sea counted in the load (m).
    pub flex: Option<[&'a [f32]; 3]>,
}

/// Prints one step's lines (see the module docs).
pub fn print(x: &Step<'_>) {
    let Some(th) = threshold() else { return };
    let n = x.h_end.len();
    let mut deep: Vec<usize> = (0..n).filter(|&c| x.sea - x.h_end[c] > th).collect();
    if deep.is_empty() {
        return;
    }
    let n_deep = deep.len();
    deep.sort_by(|&a, &b| x.h_end[a].total_cmp(&x.h_end[b]).then(a.cmp(&b)));
    deep.truncate(MAX_LINES);
    let w = x.grid.w as usize;
    let at = |v: Option<&[f64]>, c: usize| v.map_or(f64::NAN, |v| v[c]);
    for (rank, &c) in deep.iter().enumerate() {
        let (i, j) = (c % w, c / w);
        let lat = super::fields::row_lat(x.grid, j);
        let lon = (i as f64 + 0.5) / w as f64 * 360.0;
        let p = f64::from(x.plate_z[c]);
        let he = at(x.h_eroded, c);
        let hb = at(x.h_before_spread, c);
        let hs = at(x.h_spread, c);
        let mut line = format!(
            "floor t={:.1} n_deep={} rank={} c={} lat={:.2} lon={:.2} depth={:.0} p={:.0} \
             o0={:.0} o1={:.0} held={} gap={} regime={} flags={:#04x} d_rock={:.0} d_local={:.0} \
             d_spread={:.0} d_flex={:.0}",
            x.t_myr,
            n_deep,
            rank,
            c,
            lat,
            lon,
            x.sea - x.h_end[c],
            p,
            x.offset_start[c],
            x.h_end[c] - p,
            x.held.map_or(-1, |h| i32::from(h[c] > 0.5)),
            x.gap.map_or(0, |g| i32::from(g[c] > 0.5)),
            x.regime[c],
            x.flags[c] as u32,
            he - x.h_start[c],
            hb - he,
            hs - hb,
            x.h_end[c] - if hs.is_finite() { hs } else { hb },
        );
        if !x.parts.is_empty() {
            let list: Vec<String> = x
                .parts
                .iter()
                .map(|(name, v)| format!("{}:{:.0}", name, v[c]))
                .collect();
            line.push_str(&format!(" parts=[{}]", list.join(",")));
        }
        if let Some([load, defl, water]) = x.flex {
            line.push_str(&format!(
                " flex_load={:.0} flex_w={:.0} flex_water={:.0}",
                load[c], defl[c], water[c]
            ));
        }
        eprintln!("{line}");
    }
}
