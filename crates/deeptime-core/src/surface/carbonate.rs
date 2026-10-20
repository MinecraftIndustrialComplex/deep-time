// SPDX-License-Identifier: LGPL-3.0-or-later
//! Carbonate production: the light-limited growth law of Bosscher & Schlager 1992 ("Computer
//! simulation of reef growth", Sedimentology 39(3):503–512), G = G_m·tanh(I₀·e^(−kz) / I_k),
//! ported from Landlab's `CarbonateProducer` (https://github.com/landlab/landlab,
//! `src/landlab/components/carbonate/carbonate_producer.py`, MIT; the equations and defaults:
//! G_m 10 mm/yr, k 0.1 m⁻¹, I₀ 2,000 and I_k 400 µE/m²/s, the depth floored at −2·tidal range and a
//! logistic "smooth Heaviside" of width = tidal range across sea level).
//!
//! **Changes:** Landlab's rate applies everywhere the depth is positive; carbonate factories are
//! tropical and choke on mud, so here it is scaled by a water-temperature factor (0 below
//! `min_temp_c` − 6 °C, 1 above `min_temp_c`; the 18 °C of today's reef belt by default) and by
//! exp(−clastic / `clastic_m`) with the clastic sediment the cell received this step. Growth
//! stops at sea level: a step adds at most the water depth (Landlab's explicit rate times Δt would
//! overshoot at deep-time steps).
//!
//! **Pelagic carbonate** ([`pelagic`], 2026-10-19). Since the Mesozoic, planktic calcifiers
//! (coccolithophores, planktic foraminifera) rain calcareous ooze on the open sea floor, and about
//! as much carbonate accumulates there as on reefs, banks and shelves (Milliman and Droxler 1995,
//! after Milliman 1993: deep sea 11 of 32 × 10¹² mol/yr, slopes 6, the neritic rest 14.5). The ooze
//! survives only above the carbonate compensation depth (the CCD of the column builder,
//! `structure::columns::CCD_M`, RCK-015); between the lysocline and the CCD it dissolves more and
//! more. Before the calcifiers' clock (`deposits::clocks::Epoch::pelagic_calcifiers_at`) the deep
//! sea gets none (the Neritan ocean of Ridgwell 2005).

/// Pelagic carbonate above the lysocline once the calcifiers are in full swing, m/Myr of chalk:
/// the deep sea's 11 × 10¹² mol CaCO₃/yr over 290 × 10⁶ km² (Milliman and Droxler 1995, table 1)
/// is 1.4 m/Myr of calcite (2,710 kg/m³) over the whole deep floor, 3.1 m/Myr over the ~45 % of it
/// that is calcareous (Dutkiewicz et al. 2015, supplement table DR1; Diesing 2020: 37 %), so
/// 4.4 m/Myr of chalk at 30 % porosity (RCK-001: dry chalk 1,940 kg/m³, 28.8 %); rounded down
/// because the taper above the CCD thins it further (calcareous ooze 0.3–5 cm/kyr wet).
pub const PELAGIC_M_MYR: f64 = 4.0; // ledger: ERO-191
/// The lysocline lies this far above the CCD: the ooze's dissolution grows linearly from none at
/// the lysocline to all of it at the CCD.
pub const LYSOCLINE_ABOVE_CCD_M: f64 = 1000.0; // ledger: ERO-192

/// The share of the pelagic rain that survives at water depth `z` (m) over a CCD at `ccd_m`.
pub fn pelagic_share(z: f64, ccd_m: f64) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    ((ccd_m - z) / LYSOCLINE_ABOVE_CCD_M).clamp(0.0, 1.0)
}

/// Chalk laid over `dt` years at water depth `z` (m) when the calcifiers are at `calcifiers`
/// (0–1) of their full rain, m (never more than the depth).
pub fn pelagic(z: f64, calcifiers: f64, dt: f64, ccd_m: f64) -> f64 {
    (PELAGIC_M_MYR * 1.0e-6 * dt * calcifiers.clamp(0.0, 1.0) * pelagic_share(z, ccd_m))
        .min(z.max(0.0))
}

/// A cell's outgoing share below this is taken as none: the tracer stops spreading from it.
const EXPORT_MIN_SHARE: f64 = 0.01; // ledger: ERO-195
/// An exchange smaller than this (m of the giving cell over the step) does not carry the tracer
/// on: below the wave base the marine diffusivity falls by e every 60 m, so the deep floor's
/// exchanges are millimetres.
const EXPORT_MIN_FLUX_M: f64 = 1.0e-3; // ledger: ERO-195

#[derive(Clone, Copy, PartialEq)]
struct Higher {
    h: f64,
    c: u32,
}

impl Eq for Higher {}

impl PartialOrd for Higher {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Higher {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.h.total_cmp(&other.h).then(other.c.cmp(&self.c))
    }
}

/// **Periplatform carbonate** (ledger ERO-195): the carbonate in what the marine diffusion and the
/// slope failure laid in each sea cell over a step, m, as (cell, metres) for the cells that got
/// any.
///
/// Platforms shed their carbonate: the mud and sand the factory makes is swept off the platform
/// top and laid on its slope and the basin floor around it (periplatform ooze and
/// calciturbidites: the Holocene aragonite-mud wedge on the western slope of Great Bahama Bank,
/// up to 90 m thick, holds 40 to 80 % as much sediment as the bank top kept since it flooded,
/// Wilber, Milliman & Halley 1990). The diffusion moves it as it moves mud, so without this the
/// carbonate a platform exported was laid down as shale.
///
/// The implicit diffusion step's exchange across each face follows from the solved heights `h`:
/// a cell gives c·(h_i − h_j) m to a lower neighbour j, c = ½(κ_i + κ_j)·Δt/Δx² (the solver's
/// coupling, `diffusion.rs`; the faces of the five-point operator, x wrapping when the grid does).
/// Every exchange runs downhill, so taking the cells from the highest down handles each cell's
/// givers before it. What a cell passes on is a well-mixed blend of what it received and what it
/// lost itself (`loss`, m, with `loss_carb` m of carbonate in it); what it kept of its gain
/// (`gain`, m) has that blend's carbonate share. Slope failure's moves are not on the diffusion's
/// faces; its losses and gains are in `loss` and `gain` and take the blend of the diffusion
/// network around them. Only sea cells (`sea`) carry the tracer; land around them dilutes it.
#[allow(clippy::too_many_arguments)]
pub fn exported(
    d: &super::grid::Dims,
    kd: &[f64],
    dt: f64,
    h: &[f64],
    sea: &[bool],
    loss: &[f64],
    loss_carb: &[f64],
    gain: &[f64],
) -> Vec<(u32, f64)> {
    use std::collections::BinaryHeap;
    let (w, rows) = (d.w, d.h);
    let n = w * rows;
    if n == 0 || d.dx <= 0.0 {
        return Vec::new();
    }
    let k = dt / (d.dx * d.dx);
    let wrap = d.wrap_x && w > 1;
    let faces = |c: usize| -> [Option<usize>; 4] {
        let (i, j) = (c % w, c / w);
        let east = if i + 1 < w {
            Some(c + 1)
        } else if wrap {
            Some(c + 1 - w)
        } else {
            None
        };
        let west = if i > 0 {
            Some(c - 1)
        } else if wrap {
            Some(c + w - 1)
        } else {
            None
        };
        [
            east,
            west,
            (j > 0).then(|| c - w),
            (j + 1 < rows).then(|| c + w),
        ]
    };
    // 0: untouched, 1: queued, 2: done.
    let mut state = vec![0u8; n];
    let mut share = vec![0.0f32; n];
    let mut heap = BinaryHeap::new();
    for c in 0..n {
        if sea[c] && loss_carb[c] > 0.0 {
            state[c] = 1;
            heap.push(Higher {
                h: h[c],
                c: c as u32,
            });
        }
    }
    let mut out = Vec::new();
    while let Some(Higher { c, .. }) = heap.pop() {
        let c = c as usize;
        let near = faces(c);
        let (mut inflow, mut inflow_carb) = (0.0f64, 0.0f64);
        for m in near.into_iter().flatten() {
            if h[m] > h[c] {
                let f = 0.5 * (kd[m] + kd[c]) * k * (h[m] - h[c]);
                inflow += f;
                if state[m] == 2 {
                    inflow_carb += f * f64::from(share[m]);
                }
            }
        }
        let total = inflow + loss[c];
        let s = if total > 0.0 {
            ((inflow_carb + loss_carb[c]) / total).clamp(0.0, 1.0)
        } else {
            0.0
        };
        share[c] = s as f32;
        state[c] = 2;
        if gain[c] > 0.0 && s > 0.0 {
            out.push((c as u32, gain[c] * s));
        }
        if s < EXPORT_MIN_SHARE {
            continue;
        }
        for m in near.into_iter().flatten() {
            if state[m] == 0
                && sea[m]
                && h[m] < h[c]
                && 0.5 * (kd[m] + kd[c]) * k * (h[c] - h[m]) > EXPORT_MIN_FLUX_M
            {
                state[m] = 1;
                heap.push(Higher {
                    h: h[m],
                    c: m as u32,
                });
            }
        }
    }
    out.sort_unstable_by_key(|&(c, _)| c);
    out
}

/// The photic zone the factory's limits are measured over ([`limits`]), m: Bosscher and
/// Schlager's rate at 100 m is 2 × 10⁻⁴ of its surface value.
pub const PHOTIC_M: f64 = 100.0; // ledger: ERO-193

/// Whether a cell at depth `z` is in the photic zone, and its warmth and warmth × clarity factors
/// (`produce`'s), each 0 outside it (measurement only).
pub fn limits(z: f64, t_c: f64, clastic: f64, p: &CarbonateParams) -> [f64; 3] {
    if !(z > 0.0 && z <= PHOTIC_M) {
        return [0.0; 3];
    }
    let (warm, clear) = factors(t_c, clastic, p);
    [1.0, warm, warm * clear]
}

/// `produce`'s warmth and clarity factors.
fn factors(t_c: f64, clastic: f64, p: &CarbonateParams) -> (f64, f64) {
    let warm = ((t_c - (p.min_temp_c - 6.0)) / 6.0).clamp(0.0, 1.0);
    let clear = if p.clastic_m > 0.0 {
        (-clastic.max(0.0) / p.clastic_m).exp()
    } else {
        1.0
    };
    (warm, clear)
}

/// Parameters of the producer.
#[derive(Clone, Copy, Debug)]
pub struct CarbonateParams {
    /// G_m, m/yr.
    pub max_rate_m_yr: f64,
    /// k, 1/m.
    pub extinction_per_m: f64,
    pub surface_light: f64,
    pub saturating_light: f64,
    pub tidal_range_m: f64,
    pub min_temp_c: f64,
    pub clastic_m: f64,
}

/// Landlab's `smooth_heaviside`: 1 / (1 + e^(−2x/width)), a step when width is 0.
pub fn smooth_heaviside(x: f64, width: f64) -> f64 {
    if width > 0.0 {
        1.0 / (1.0 + (-2.0 / width * x).exp())
    } else if x > 0.0 {
        1.0
    } else if x < 0.0 {
        0.0
    } else {
        0.5
    }
}

/// Bosscher & Schlager's rate at water depth `z` (m, positive below sea level), m/yr.
pub fn rate(z: f64, p: &CarbonateParams) -> f64 {
    let z = z.max(-2.0 * p.tidal_range_m - 1.0e-6);
    let g = p.max_rate_m_yr
        * (p.surface_light * (-p.extinction_per_m * z).exp() / p.saturating_light).tanh();
    g * smooth_heaviside(z, p.tidal_range_m)
}

/// Thickness produced over `dt` years at depth `z` in water of `t_c` °C that received `clastic` m
/// of land-derived sediment this step, m (never more than the depth).
pub fn produce(z: f64, t_c: f64, clastic: f64, dt: f64, p: &CarbonateParams) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    let (warm, clear) = factors(t_c, clastic, p);
    if warm <= 0.0 {
        return 0.0;
    }
    (rate(z, p) * warm * clear * dt).min(z)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> CarbonateParams {
        CarbonateParams {
            max_rate_m_yr: 0.01,
            extinction_per_m: 0.1,
            surface_light: 2000.0,
            saturating_light: 400.0,
            tidal_range_m: 0.0,
            min_temp_c: 18.0,
            clastic_m: 20.0,
        }
    }

    #[test]
    fn bosscher_schlager_matches_its_formula() {
        let p = p();
        // tanh(2000 e^(−0.1·10) / 400) · 0.01
        let expect = 0.01 * (2000.0 * (-1.0f64).exp() / 400.0).tanh();
        assert!((rate(10.0, &p) - expect).abs() < 1e-12);
        assert!(rate(80.0, &p) < rate(10.0, &p));
        assert_eq!(rate(-5.0, &p), 0.0);
        // A warm clear lagoon 5 m deep fills to sea level in 2 Myr; cold or muddy water does not.
        assert!((produce(5.0, 26.0, 0.0, 2.0e6, &p) - 5.0).abs() < 1e-9);
        assert_eq!(produce(5.0, 5.0, 0.0, 2.0e6, &p), 0.0);
        assert!(produce(150.0, 26.0, 200.0, 1.0e4, &p) < 1e-3);
    }

    #[test]
    fn exported_carbonate_follows_the_diffusion_downhill() {
        use crate::surface::grid::Dims;
        // Two rows of three, faces of 1 (κ 1, Δt 1, Δx 1). Cell 0 sheds 1 m, all carbonate;
        // cells 1 and 2 lie below it, 1 and 2 m down. Cell 4 stands at cell 0's height.
        let d = Dims {
            w: 3,
            h: 2,
            dx: 1.0,
            wrap_x: false,
        };
        let kd = [1.0; 6];
        let sea = [true; 6];
        let loss = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let loss_carb = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let gain = [0.0, 0.5, 0.5, 0.0, 0.0, 0.0];
        // All the slope's supply is carbonate: everything it keeps is carbonate.
        let h = [0.0, -1.0, -2.0, -10.0, -10.0, -10.0];
        let out = exported(&d, &kd, 1.0, &h, &sea, &loss, &loss_carb, &gain);
        assert_eq!(out, vec![(1, 0.5), (2, 0.5)]);
        // Cell 4 (no carbonate) gives cell 1 as much as cell 0 does: half of it is carbonate,
        // and half of what cell 1 passes on to cell 2.
        let h = [0.0, -1.0, -2.0, -10.0, 0.0, -10.0];
        let out = exported(&d, &kd, 1.0, &h, &sea, &loss, &loss_carb, &gain);
        assert_eq!(out, vec![(1, 0.25), (2, 0.25)]);
        // On land the tracer stops; without carbonate there is none.
        let land = [true, false, true, true, true, true];
        assert!(exported(&d, &kd, 1.0, &h, &land, &loss, &loss_carb, &gain).is_empty());
        let none = [0.0; 6];
        assert!(exported(&d, &kd, 1.0, &h, &sea, &loss, &none, &gain).is_empty());
    }

    #[test]
    fn pelagic_ooze_dissolves_between_the_lysocline_and_the_ccd() {
        let ccd = 4500.0;
        // Above the lysocline: the full rain, 4 m/Myr over 2 Myr.
        assert!((pelagic(3000.0, 1.0, 2.0e6, ccd) - 8.0).abs() < 1e-9);
        // Half way between the lysocline (3,500 m) and the CCD: half of it.
        assert!((pelagic(4000.0, 1.0, 2.0e6, ccd) - 4.0).abs() < 1e-9);
        // Below the CCD, on land, or before the calcifiers: none.
        assert_eq!(pelagic(5000.0, 1.0, 2.0e6, ccd), 0.0);
        assert_eq!(pelagic(-10.0, 1.0, 2.0e6, ccd), 0.0);
        assert_eq!(pelagic(3000.0, 0.0, 2.0e6, ccd), 0.0);
        // Never above the sea.
        assert!((pelagic(3.0, 1.0, 2.0e6, ccd) - 3.0).abs() < 1e-9);
    }
}
