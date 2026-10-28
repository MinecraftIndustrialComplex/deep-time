// SPDX-License-Identifier: LGPL-3.0-or-later
//! Regional fields the deposit engine derives from each frame of the history (as built
//! 2026-09-29, Ben: "Rework its driver" for MVT): basin-scale fluid flow, which no single cell's
//! fields can say.
//!
//! Mississippi Valley-type Zn–Pb ore is deposited in platform carbonates by basinal brines that
//! travelled hundreds of kilometres: topography-driven groundwater flow, recharged in an uplifted
//! orogen and discharging at the far, cratonward margin of a foreland basin, sweeps the basin's
//! brines up-dip into the carbonate platform (Garven & Freeze 1984; Garven 1985 for Pine Point
//! in the Western Canada sedimentary basin; Bethke & Marshak 1990 for brines crossing the North
//! American midcontinent from the Ouachita–Appalachian orogens), mostly during contractional
//! events (Leach et al. 2001; Bradley & Leach 2003; Leach et al. 2005, 2010). Tectonic loading
//! squeezes fluids into the foreland as well (Oliver 1986). The source (a brine-filled basin),
//! the drive (an elevated recharge area) and the host (a carbonate platform) therefore need not
//! share a cell: they must share a *flow system* a few hundred kilometres across — the USGS MVT
//! model (SIR 2010-5070-A) draws MVT-permissive tracts on platform carbonates "less than ~600 km
//! inboard of Phanerozoic orogenic belts", the preset's default `brine_flow_km`. Two fields say
//! so, each within `flow_km` of the cell (block metric, the engine's metric everywhere):
//!
//! - **`deeptime:basin_brine`** (0–1): the share of the surroundings within `flow_km` (a tapered
//!   octagonal window, nearer ground weighing more) taken by a hot sedimentary basin on
//!   continental crust: `b = continental · aggrading · hot`,
//!   `continental = clamp((crust − 20 km) / 10 km)`, `aggrading = clamp(deposition / 10 m/Myr)` (a
//!   subsiding basin filling now) and `hot = clamp((T − 50 °C) / 50 °C)` with T the burial
//!   temperature of the basin's buried organic-rich layer (`burial_temp_c`): MVT ore fluids were
//!   brines at 75–200 °C (Garven 1985: 60–100 °C at Pine Point; Leach et al. 2005: 90–200 °C), so
//!   the basin they came from was hotter, and they cool as they travel. Deep basin formation waters
//!   are saline; the engine does not model salinity apart. A share of 1 is hot basin all around;
//!   the MVT source counts fully at 0.035, about 40,000 km² of hot basin spread evenly within a
//!   600 km reach (less when it lies close). The thresholds are design values.
//! - **`deeptime:regional_head_drop_m_km`** (m/km): the regional hydraulic gradient an active
//!   orogen within `flow_km` gives the cell: `(max H_a within flow_km − H) / flow_km` (the maximum
//!   over an octagon of that radius), with `H` the
//!   water-table head (land elevation above the sea of the time, 0 under the sea) averaged over
//!   50 km so a single peak does not count, and `H_a = H · clamp(U / 100 m/Myr)` the head of ground
//!   that is rising now (U the 50-km mean of `orogenic_uplift_m_myr`): MVT ore formed mainly during
//!   contractional events, in platforms "with some hydrological connection to orogenic belts"
//!   (Leach et al. 2001). A 2 km orogen rising at 0.1 mm/yr or more gives 3.3 m/km across the full
//!   600 km reach — Garven & Freeze 1984's base case, a 1 km water-table drop over 300 km.
//!
//! The windows are octagons (a square and the two diagonals in turn: their Minkowski sum, of the
//! area of the reach's disk, strays from it by 5 % at most) in block metric, x wrapping, clipped at the
//! poles. Both fields are exact under east–west shifts of the grid (window sums in 64-bit fixed
//! point, maxima by a sliding window), so a rigidly drifting crust gives bit-identical favourability
//! (`recorded_tests::deposits_ride_a_rigidly_drifting_crust`). A frame that already carries a
//! field of either name keeps it (a later stage may publish a better one).

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

use deeptime_format::GridDesc;
use rayon::prelude::*;

use super::fields::{BURIAL_TEMP, DEPOSITION, ELEVATION, FieldFrame};

/// Share of a hot sedimentary basin within the flow reach (0–1).
pub const BASIN_BRINE: &str = "deeptime:basin_brine";
/// Regional head drop from an active orogen over the flow reach (m/km).
pub const HEAD_DROP: &str = "deeptime:regional_head_drop_m_km";

const CRUST: &str = "deeptime:crust_thickness_km";
const UPLIFT: &str = "deeptime:orogenic_uplift_m_myr";

/// What each derived field is made from.
const INPUTS: [(&str, &[&str]); 2] = [
    (BASIN_BRINE, &[DEPOSITION, CRUST, BURIAL_TEMP]),
    (HEAD_DROP, &[ELEVATION, UPLIFT]),
];

/// Smoothing of the head and the uplift before the maximum is taken (km; the scale of the
/// surface stage's `topographic_gradient_m_km`).
pub const HEAD_SMOOTH_KM: f64 = 50.0;
/// Deposition rate at which a basin counts as fully aggrading (m/Myr; design).
pub const AGGRADING_M_MYR: f64 = 10.0;
/// Burial temperatures over which a basin's brine goes from cold to hot enough (°C; design, from
/// the 75–200 °C of MVT ore fluids).
pub const BRINE_T_C: [f64; 2] = [50.0, 100.0];
/// Uplift rate at which ground counts as an active orogen (m/Myr = 0.1 mm/yr; design).
pub const ACTIVE_UPLIFT_M_MYR: f64 = 100.0;

/// Window normalisations by (width, height, radius).
type NormCache = BTreeMap<(usize, usize, usize), Arc<Vec<i64>>>;

/// Fixed-point scale of the exact box sums.
const FIXED: f64 = 1_048_576.0;

/// The derived fields' parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Regional {
    /// Reach of a basin-scale flow system (km, block metric).
    pub flow_km: f64,
    /// L1 cell side (km, block metric).
    pub cell_km: f64,
}

impl Regional {
    pub fn new(grid: &GridDesc, m_per_block_h: f64, flow_km: f64) -> Regional {
        Regional {
            flow_km,
            cell_km: f64::from(grid.r) * m_per_block_h / 1000.0,
        }
    }

    fn cells(&self, km: f64) -> usize {
        (km / self.cell_km).round().max(0.0) as usize
    }
}

/// The derived names a history with `names` can have (their inputs are all there).
pub fn derivable(names: &[String]) -> Vec<String> {
    INPUTS
        .iter()
        .filter(|(out, ins)| {
            !names.iter().any(|n| n == out) && ins.iter().all(|i| names.iter().any(|n| n == i))
        })
        .map(|(out, _)| (*out).to_owned())
        .collect()
}

/// The derived names.
pub const DERIVED: [&str; 2] = [BASIN_BRINE, HEAD_DROP];

/// Adds the derived fields among `want` that `frame` can have and does not carry yet.
pub fn add(frame: &mut FieldFrame, grid: &GridDesc, p: &Regional, want: &[String]) {
    let (w, h) = (grid.w as usize, grid.h as usize);
    let n = w * h;
    let have = |f: &FieldFrame, name: &str| f.fields.get(name).is_some_and(|v| v.len() == n);
    let wants = |name: &str| want.iter().any(|x| x == name);
    let reach = p.cells(p.flow_km);
    if wants(BASIN_BRINE) && !have(frame, BASIN_BRINE) && INPUTS[0].1.iter().all(|i| have(frame, i))
    {
        let (dep, crust, temp) = (
            &frame.fields[DEPOSITION],
            &frame.fields[CRUST],
            &frame.fields[BURIAL_TEMP],
        );
        let b: Vec<i64> = (0..n)
            .into_par_iter()
            .map(|c| {
                let cont = ((f64::from(crust[c]) - 20.0) / 10.0).clamp(0.0, 1.0);
                let agg = (f64::from(dep[c]) / AGGRADING_M_MYR).clamp(0.0, 1.0);
                let hot = ((f64::from(temp[c]) - BRINE_T_C[0]) / (BRINE_T_C[1] - BRINE_T_C[0]))
                    .clamp(0.0, 1.0);
                let v = cont * agg * hot;
                if v.is_finite() {
                    (v * FIXED).round() as i64
                } else {
                    0
                }
            })
            .collect();
        let mean = octagon_mean(w, h, &b, reach);
        frame.fields.insert(BASIN_BRINE.to_owned(), mean);
    }
    if wants(HEAD_DROP) && !have(frame, HEAD_DROP) && INPUTS[1].1.iter().all(|i| have(frame, i)) {
        // Fixed point: exact sums whatever the order.
        let fixed = |v: &[f32], lo: f64| -> Vec<i64> {
            v.par_iter()
                .map(|&e| {
                    let v = f64::from(e).max(lo);
                    if v.is_finite() {
                        (v * FIXED).round() as i64
                    } else {
                        0
                    }
                })
                .collect()
        };
        let s = p.cells(HEAD_SMOOTH_KM);
        let smooth = octagon_mean(w, h, &fixed(&frame.fields[ELEVATION], 0.0), s);
        let uplift = octagon_mean(w, h, &fixed(&frame.fields[UPLIFT], 0.0), s);
        let active: Vec<f32> = smooth
            .par_iter()
            .zip(uplift.par_iter())
            .map(|(&hs, &u)| {
                (f64::from(hs) * (f64::from(u) / ACTIVE_UPLIFT_M_MYR).clamp(0.0, 1.0)) as f32
            })
            .collect();
        let top = octagon_max(w, h, &active, reach);
        let span_km = p.flow_km.max(p.cell_km);
        let drop: Vec<f32> = smooth
            .par_iter()
            .zip(top.par_iter())
            .map(|(&hs, &hm)| ((f64::from(hm) - f64::from(hs)).max(0.0) / span_km) as f32)
            .collect();
        frame.fields.insert(HEAD_DROP.to_owned(), drop);
    }
}

/// One family of grid lines a window slides along: rows (x wraps), columns (clipped at the
/// poles) or the two diagonals (i − d·j constant mod w, clipped at the poles).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Line {
    Row,
    Col,
    Diag(i64),
}

impl Line {
    fn count(self, w: usize, h: usize) -> usize {
        match self {
            Line::Row => h,
            Line::Col | Line::Diag(_) => w,
        }
    }
}

/// Window sums of half-length `r` along a sequence (circular or clipped).
fn window_sum(seq: &[i64], r: usize, circular: bool) -> Vec<i64> {
    let len = seq.len();
    if circular && 2 * r + 1 >= len {
        let total: i64 = seq.iter().sum();
        return vec![total; len];
    }
    if circular {
        let mut prefix = vec![0i64; 2 * len + 1];
        for k in 0..2 * len {
            prefix[k + 1] = prefix[k] + seq[k % len];
        }
        (0..len)
            .map(|k| {
                let lo = (k + len - r) % len;
                prefix[lo + 2 * r + 1] - prefix[lo]
            })
            .collect()
    } else {
        let mut prefix = vec![0i64; len + 1];
        for k in 0..len {
            prefix[k + 1] = prefix[k] + seq[k];
        }
        (0..len)
            .map(|k| prefix[(k + r).min(len - 1) + 1] - prefix[k.saturating_sub(r)])
            .collect()
    }
}

/// Sliding-window maximum of half-length `r` along a sequence (circular or clipped).
fn window_max(src: &[f32], r: usize, circular: bool) -> Vec<f32> {
    let len = src.len();
    if circular && 2 * r + 1 >= len {
        let m = src.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        return vec![m; len];
    }
    // Circular: an extended sequence of len + 2r values starting r before the first.
    let (ext, offset): (Vec<f32>, usize) = if circular {
        (
            (0..len + 2 * r).map(|k| src[(k + len - r) % len]).collect(),
            r,
        )
    } else {
        (src.to_vec(), 0)
    };
    let mut out = vec![0.0f32; len];
    let mut dq: VecDeque<usize> = VecDeque::new();
    let mut next = 0usize;
    for (k, o) in out.iter_mut().enumerate() {
        let centre = k + offset;
        let hi = (centre + r).min(ext.len() - 1);
        let lo = centre.saturating_sub(r);
        while next <= hi {
            while dq.back().is_some_and(|&b| ext[b] <= ext[next]) {
                dq.pop_back();
            }
            dq.push_back(next);
            next += 1;
        }
        while dq.front().is_some_and(|&f| f < lo) {
            dq.pop_front();
        }
        *o = ext[*dq.front().expect("the window holds its centre")];
    }
    out
}

/// One pass of `f` along every line of a family, written back cell by cell.
fn pass<T: Copy + Default + Send + Sync>(
    w: usize,
    h: usize,
    v: &[T],
    line: Line,
    f: &(dyn Fn(&[T], bool) -> Vec<T> + Sync),
) -> Vec<T> {
    if line == Line::Row {
        return v.par_chunks(w).flat_map_iter(|row| f(row, true)).collect();
    }
    let index = |k: usize, j: usize| -> usize {
        match line {
            Line::Diag(d) => j * w + (k as i64 + d * j as i64).rem_euclid(w as i64) as usize,
            _ => j * w + k,
        }
    };
    let lines: Vec<Vec<T>> = (0..line.count(w, h))
        .into_par_iter()
        .map(|k| {
            let seq: Vec<T> = (0..h).map(|j| v[index(k, j)]).collect();
            f(&seq, false)
        })
        .collect();
    // Back into rows, rows in parallel (the speed work of 2026-10-28; one thread before): cell
    // (i, j) is line k's j-th value, k = i − d·j round the circle (a column: k = i).
    let d = match line {
        Line::Diag(d) => d,
        _ => 0,
    };
    let mut out = vec![T::default(); w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        for (i, x) in row.iter_mut().enumerate() {
            *x = lines[(i as i64 - d * j as i64).rem_euclid(w as i64) as usize][j];
        }
    });
    out
}

/// The square half-side s and the diagonal half-length q of the octagon standing for a disk of
/// radius `r` cells: the Minkowski sum of the square [−s, s]² and two diagonal segments of
/// half-length q is an octagon with axis edges at s + 2q and diagonal edges at (s + q)·√2; it is
/// regular for s = q·√2, and of the disk's area for an inradius of r·√(π / (8·tan 22.5°)) =
/// 0.974·r (circumradius 1.054·r).
fn octagon(r: usize) -> (usize, usize) {
    use std::f64::consts::{PI, SQRT_2};
    let inradius = r as f64 * (PI / (8.0 * (PI / 8.0).tan())).sqrt();
    let s = inradius / (1.0 + SQRT_2);
    let q = s / SQRT_2;
    (s.round() as usize, q.round() as usize)
}

/// A tapered mean of fixed-point values over an octagon of radius `r` cells around each cell
/// (the square and the two diagonal box filters in turn, which weights nearer cells more), x
/// wrapping, clipped at the poles and renormalised there. Integer sums: the result is the same
/// bits whatever the grid's east–west offset.
pub fn octagon_mean(w: usize, h: usize, v: &[i64], r: usize) -> Vec<f32> {
    let (s, q) = octagon(r);
    let sums = |x: &[i64]| -> Vec<i64> {
        let mut x = x.to_vec();
        for (line, half) in [
            (Line::Row, s),
            (Line::Col, s),
            (Line::Diag(1), q),
            (Line::Diag(-1), q),
        ] {
            x = pass(w, h, &x, line, &|seq: &[i64], circ| {
                window_sum(seq, half, circ)
            });
        }
        x
    };
    let num = sums(v);
    // The normalisation depends on the grid and the radius only: computed once.
    static DEN: OnceLock<Mutex<NormCache>> = OnceLock::new();
    let cache = DEN.get_or_init(Default::default);
    let known = cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&(w, h, r))
        .cloned();
    let den = known.unwrap_or_else(|| {
        let d = Arc::new(sums(&vec![1i64; w * h]));
        cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert((w, h, r), d.clone());
        d
    });
    num.par_iter()
        .zip(den.par_iter())
        .map(|(&a, &b)| (a as f64 / b as f64 / FIXED) as f32)
        .collect()
}

/// The maximum over an octagon of radius `r` cells around each cell (a dilation by the square,
/// then by the two diagonal segments: their Minkowski sum), x wrapping, clipped at the poles.
pub fn octagon_max(w: usize, h: usize, v: &[f32], r: usize) -> Vec<f32> {
    let (s, q) = octagon(r);
    let mut x = v.to_vec();
    for (line, half) in [
        (Line::Row, s),
        (Line::Col, s),
        (Line::Diag(1), q),
        (Line::Diag(-1), q),
    ] {
        x = pass(w, h, &x, line, &|seq: &[f32], circ| {
            window_max(seq, half, circ)
        });
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_match_the_naive_ones() {
        let seq: Vec<i64> = (0..23).map(|k| (k * 7919 % 101) as i64).collect();
        let f: Vec<f32> = seq.iter().map(|&x| x as f32).collect();
        let len = seq.len() as i64;
        for r in [0usize, 1, 3, 11, 30] {
            for circular in [true, false] {
                let (sum, max) = (window_sum(&seq, r, circular), window_max(&f, r, circular));
                for k in 0..len {
                    let idx: Vec<usize> = if circular && 2 * r as i64 + 1 >= len {
                        (0..len as usize).collect()
                    } else if circular {
                        (-(r as i64)..=r as i64)
                            .map(|d| (k + d).rem_euclid(len) as usize)
                            .collect()
                    } else {
                        ((k - r as i64).max(0)..=(k + r as i64).min(len - 1))
                            .map(|i| i as usize)
                            .collect()
                    };
                    let s: i64 = idx.iter().map(|&i| seq[i]).sum();
                    let m = idx.iter().map(|&i| f[i]).fold(f32::NEG_INFINITY, f32::max);
                    assert_eq!(sum[k as usize], s, "sum r={r} circular={circular} k={k}");
                    assert_eq!(max[k as usize], m, "max r={r} circular={circular} k={k}");
                }
            }
        }
    }

    /// The octagon: a spike dilates into a dense octagon of the asked radius (nothing beyond it,
    /// no hole inside the inscribed circle); a constant field keeps its value to the poles; an
    /// east–west shift of the input shifts both outputs bit for bit.
    /// A line pass puts each line's values back where the serial scatter did (the speed work of
    /// 2026-10-28: rows in parallel).
    #[test]
    fn line_passes_put_the_lines_back_where_the_serial_scatter_did() {
        let (w, h) = (37, 23);
        let v: Vec<i64> = (0..w * h).map(|c| (c as i64 * 7919) % 1009 - 500).collect();
        for line in [Line::Col, Line::Diag(1), Line::Diag(-1)] {
            let f = |seq: &[i64], circ: bool| window_sum(seq, 3, circ);
            let got = pass(w, h, &v, line, &f);
            let index = |k: usize, j: usize| -> usize {
                match line {
                    Line::Diag(d) => {
                        j * w + (k as i64 + d * j as i64).rem_euclid(w as i64) as usize
                    }
                    _ => j * w + k,
                }
            };
            let mut want = vec![0i64; w * h];
            for k in 0..line.count(w, h) {
                let seq: Vec<i64> = (0..h).map(|j| v[index(k, j)]).collect();
                for (j, x) in f(&seq, false).into_iter().enumerate() {
                    want[index(k, j)] = x;
                }
            }
            assert_eq!(got, want);
        }
    }

    #[test]
    fn octagons_have_the_radius_and_shift_exactly() {
        let (w, h) = (96usize, 64usize);
        let r = 20usize;
        let (ci, cj) = (40usize, 32usize);
        let mut spike = vec![0.0f32; w * h];
        spike[cj * w + ci] = 1.0;
        let m = octagon_max(w, h, &spike, r);
        for j in 0..h {
            for i in 0..w {
                let (dx, dz) = (i as f64 - ci as f64, j as f64 - cj as f64);
                let d = (dx * dx + dz * dz).sqrt();
                let inside = m[j * w + i] == 1.0;
                // The octagon of the disk's area: inradius 0.974·r, circumradius 1.054·r.
                if d <= r as f64 * 0.974 - 1.0 {
                    assert!(inside, "hole at {dx},{dz}");
                }
                if d > r as f64 * 1.054 + 1.0 {
                    assert!(!inside, "reached {dx},{dz}");
                }
            }
        }
        // Constant field: the tapered mean is the constant everywhere.
        let c = (0.3 * FIXED).round() as i64;
        let flat = octagon_mean(w, h, &vec![c; w * h], r);
        assert!(flat.iter().all(|&x| (f64::from(x) - 0.3).abs() < 1e-6));
        // A spike's mean is positive inside the octagon only, symmetric, and tapers outwards.
        let mut one = vec![0i64; w * h];
        one[cj * w + ci] = FIXED as i64;
        let k = octagon_mean(w, h, &one, r);
        for c in 0..w * h {
            if m[c] == 0.0 {
                assert_eq!(k[c], 0.0, "cell {c}");
            }
        }
        assert_eq!(k[cj * w + ci + 5], k[cj * w + ci - 5]);
        assert!(k[cj * w + ci] > k[cj * w + ci + 15]);
        // Shifts.
        let v: Vec<i64> = (0..w * h)
            .map(|c| ((c * 7919) % 1013) as i64 * 1000)
            .collect();
        let vf: Vec<f32> = v.iter().map(|&x| x as f32).collect();
        let (a, b) = (octagon_mean(w, h, &v, r), octagon_max(w, h, &vf, r));
        let by = 7usize;
        let sh = |c: usize| (c / w) * w + (c % w + w - by) % w;
        let v2: Vec<i64> = (0..w * h).map(|c| v[sh(c)]).collect();
        let vf2: Vec<f32> = (0..w * h).map(|c| vf[sh(c)]).collect();
        let (a2, b2) = (octagon_mean(w, h, &v2, r), octagon_max(w, h, &vf2, r));
        for c in 0..w * h {
            assert_eq!(a2[c].to_bits(), a[sh(c)].to_bits());
            assert_eq!(b2[c].to_bits(), b[sh(c)].to_bits());
        }
    }

    fn all() -> Vec<String> {
        DERIVED.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn a_basin_beside_an_orogen_drives_brine_to_its_margin() {
        // 128 × 64 cells of 4.8 km: a rising orogen (2 km, 300 m/Myr) in columns 0–7, a hot
        // foreland basin (40 m/Myr of fill, 110 °C at depth) in 8–23, a lowland platform beyond.
        let grid = GridDesc::for_planet("L1", 2048, 16).unwrap();
        let (w, h) = (grid.w as usize, grid.h as usize);
        let n = w * h;
        let mut f = FieldFrame {
            t_years: 0.0,
            dt_years: 1.0e7,
            ..FieldFrame::default()
        };
        let col = |c: usize| c % w;
        f.fields.insert(
            ELEVATION.into(),
            (0..n)
                .map(|c| if col(c) < 8 { 2000.0 } else { 100.0 })
                .collect(),
        );
        f.fields.insert(
            DEPOSITION.into(),
            (0..n)
                .map(|c| if (8..24).contains(&col(c)) { 40.0 } else { 0.0 })
                .collect(),
        );
        f.fields.insert(
            UPLIFT.into(),
            (0..n)
                .map(|c| if col(c) < 8 { 300.0 } else { 0.0 })
                .collect(),
        );
        f.fields.insert(
            BURIAL_TEMP.into(),
            (0..n)
                .map(|c| {
                    if (8..24).contains(&col(c)) {
                        110.0
                    } else {
                        20.0
                    }
                })
                .collect(),
        );
        f.fields.insert(CRUST.into(), vec![38.0; n]);
        let p = Regional::new(&grid, 300.0, 100.0);
        add(&mut f, &grid, &p, &all());
        let brine = &f.fields[BASIN_BRINE];
        let drop = &f.fields[HEAD_DROP];
        let at = |v: &[f32], i: usize| v[(h / 2) * w + i];
        // The platform next to the basin sees its brines and the orogen's head; one far away
        // (more than 100 km from both) sees neither.
        assert!(at(brine, 30) > 0.1, "{}", at(brine, 30));
        assert!(at(drop, 30) > 3.0, "{}", at(drop, 30));
        assert!(at(brine, 80) < 1e-6, "{}", at(brine, 80));
        assert!(at(drop, 80) < 1e-6, "{}", at(drop, 80));
        assert_eq!(
            derivable(&[ELEVATION.to_string(), UPLIFT.to_string()]),
            vec![HEAD_DROP.to_string()]
        );
        // The same orogen, no longer rising, drives nothing.
        f.fields.remove(HEAD_DROP);
        f.fields.insert(UPLIFT.into(), vec![0.0; n]);
        add(&mut f, &grid, &p, &all());
        assert!(at(&f.fields[HEAD_DROP], 30) < 1e-6);
    }
}
