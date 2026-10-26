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
//! **Pelagic carbonate** (2026-10-19; since 2026-10-25 in `surface::pelagic`). The chalk is the
//! calcifiers' share of the carbonate the dissolved load returns to the sea floor, not a rate of
//! its own; the carbonate compensation depth decides where it survives.

/// A cell's outgoing share below this is taken as none: the tracer stops spreading from it.
const EXPORT_MIN_SHARE: f64 = 0.01; // ledger: ERO-195
/// An exchange smaller than this (m of the giving cell over [`EXPORT_REF_YEARS`]) does not carry
/// the tracer on: below the wave base the marine diffusivity falls by e every 60 m, so the deep
/// floor's exchanges are millimetres.
const EXPORT_MIN_FLUX_M: f64 = 1.0e-3; // ledger: ERO-195
/// The time [`EXPORT_MIN_FLUX_M`] is over (the default plate step of one substep): a substep of
/// another length scales it, so the cut-off is a rate and the step size does not move it.
const EXPORT_REF_YEARS: f64 = 2.0e6; // ledger: ERO-195

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

/// **Periplatform carbonate** (ledger ERO-195): the carbonate in what the marine diffusion laid in
/// each sea cell over a substep, m, as (cell, metres) for the cells that got any.
///
/// Platforms shed their carbonate: the mud and sand the factory makes is swept off the platform
/// top and laid on its slope and the basin floor around it (periplatform ooze and
/// calciturbidites: the Holocene aragonite-mud wedge on the western slope of Great Bahama Bank,
/// up to 90 m thick, holds 40 to 80 % as much sediment as the bank top kept since it flooded,
/// Wilber, Milliman & Halley 1990). The diffusion moves it as it moves mud, so without this the
/// carbonate a platform exported was laid down as shale.
///
/// `loss` and `gain` are the diffusion's own change (the solve's heights less the heights before
/// it; read per cell, so the caller need not hold them as arrays), so a cell keeps no more than it
/// was given and the carbonate laid is at most the carbonate lost. Slope failure is not on the diffusion's faces: its debris is laid as clastic
/// ([`StepTop::substep`]).
///
/// The implicit diffusion step's exchange across each face follows from the solved heights `h`:
/// a cell gives c·(h_i − h_j) m to a lower neighbour j, c = ½(κ_i + κ_j)·Δt/Δx² (the solver's
/// coupling, `diffusion.rs`; the faces of the five-point operator, x wrapping when the grid does).
/// Every exchange runs downhill, so taking the cells from the highest down handles each cell's
/// givers before it. What a cell passes on is a well-mixed blend of what it received and what it
/// lost itself (`loss`, m, with `loss_carb` m of carbonate in it); what it kept of its gain
/// (`gain`, m) has that blend's carbonate share. Only sea cells (`sea`) carry the tracer; land
/// around them dilutes it.
#[allow(clippy::too_many_arguments)]
pub fn exported(
    d: &super::grid::Dims,
    kd: &[f64],
    dt: f64,
    h: &[f64],
    sea: &[bool],
    loss: impl Fn(usize) -> f64,
    loss_carb: &[f64],
    gain: impl Fn(usize) -> f64,
) -> Vec<(u32, f64)> {
    use std::collections::BinaryHeap;
    let (w, rows) = (d.w, d.h);
    let n = w * rows;
    if n == 0 || d.dx <= 0.0 {
        return Vec::new();
    }
    let k = dt / (d.dx * d.dx);
    let min_flux = EXPORT_MIN_FLUX_M * dt / EXPORT_REF_YEARS;
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
        let total = inflow + loss(c);
        let s = if total > 0.0 {
            ((inflow_carb + loss_carb[c]) / total).clamp(0.0, 1.0)
        } else {
            0.0
        };
        share[c] = s as f32;
        state[c] = 2;
        let gain = gain(c);
        if gain > 0.0 && s > 0.0 {
            out.push((c as u32, gain * s));
        }
        if s < EXPORT_MIN_SHARE {
            continue;
        }
        for m in near.into_iter().flatten() {
            if state[m] == 0
                && sea[m]
                && h[m] < h[c]
                && 0.5 * (kd[m] + kd[c]) * k * (h[c] - h[m]) > min_flux
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

/// One cell's deposit of the plate step so far, for the periplatform tracer's peel (ledger
/// ERO-195): what a substep's diffusion takes off a cell is read from the top of the column as it
/// stands then, not as it stood at the step's start.
///
/// The column's top at a substep's diffusion, top-down: the substep's own carbonate (`fresh`),
/// the rest it laid before the diffusion (`new_other`: river, sea-fill, glacial and evaporite
/// sediment), the step's earlier substeps' deposit (`carb` and `other`, taken well mixed), then
/// the column the step began with (its regolith and strata log) from `taken_old` m down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepTop {
    /// Carbonate of the step's earlier substeps still in the cell, m.
    pub carb: f32,
    /// The rest of the step's earlier substeps' deposit still in the cell, m.
    pub other: f32,
    /// What the step has taken off the column it began with, m.
    pub taken_old: f32,
}

/// A cell's changes over one substep (m), in their order: what the substep's erosion took before
/// it laid anything (`pre_erosion`), what it laid (`new_other`, then `fresh` carbonate on top),
/// the diffusion's change, then slope failure's.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SubstepCell {
    pub pre_erosion: f64,
    pub new_other: f64,
    pub fresh: f64,
    pub diffusion: f64,
    pub failure: f64,
}

impl StepTop {
    /// Takes `m` metres off the top: the step's earlier deposit, well mixed, then the column the
    /// step began with (`old_carb(from, m)`: the carbonate in its `m` metres from `from` m down).
    /// Returns the carbonate taken.
    pub fn take(&mut self, m: f64, old_carb: &impl Fn(f64, f64) -> f64) -> f64 {
        if m <= 0.0 {
            return 0.0;
        }
        let (carb, other) = (f64::from(self.carb), f64::from(self.other));
        let pool = carb + other;
        let t = m.min(pool);
        let carb_taken = if pool > 0.0 { carb * t / pool } else { 0.0 };
        self.carb = (carb - carb_taken).max(0.0) as f32;
        self.other = (other - (t - carb_taken)).max(0.0) as f32;
        let rest = m - t;
        if rest <= 0.0 {
            return carb_taken;
        }
        let from = f64::from(self.taken_old);
        self.taken_old = (from + rest) as f32;
        carb_taken + old_carb(from, rest)
    }

    /// The carbonate in what the substep's diffusion takes off the cell (`s.diffusion` < 0), m,
    /// leaving the record as it is.
    pub fn diffusion_carbonate(&self, s: &SubstepCell, old_carb: &impl Fn(f64, f64) -> f64) -> f64 {
        let mut copy = *self;
        copy.substep(s, 0.0, old_carb).0
    }

    /// Runs the substep `s` on the record, the diffusion's gain holding `laid` m of carbonate
    /// ([`exported`]). Slope failure takes its loss off the top (the diffusion's gain first, well
    /// mixed) and lays its debris as clastic. Returns (the carbonate the diffusion took, the
    /// periplatform carbonate the cell keeps).
    pub fn substep(
        &mut self,
        s: &SubstepCell,
        laid: f64,
        old_carb: &impl Fn(f64, f64) -> f64,
    ) -> (f64, f64) {
        self.take(s.pre_erosion, old_carb);
        let (mut fresh, mut new_other) = (s.fresh.max(0.0), s.new_other.max(0.0));
        let mut carb_lost = 0.0;
        if s.diffusion < 0.0 {
            let mut left = -s.diffusion;
            let a = fresh.min(left);
            fresh -= a;
            left -= a;
            carb_lost += a;
            let b = new_other.min(left);
            new_other -= b;
            left -= b;
            carb_lost += self.take(left, old_carb);
        }
        // What is left of the substep's own deposit joins the step's.
        self.carb = (f64::from(self.carb) + fresh) as f32;
        self.other = (f64::from(self.other) + new_other) as f32;
        let gain = s.diffusion.max(0.0);
        let laid = laid.clamp(0.0, gain);
        let (mut d_carb, mut d_other) = (laid, gain - laid);
        if s.failure < 0.0 {
            let mut left = -s.failure;
            let d = d_carb + d_other;
            if d > 0.0 {
                let f = (left / d).min(1.0);
                d_carb *= 1.0 - f;
                d_other *= 1.0 - f;
                left -= f * d;
            }
            self.take(left, old_carb);
        } else {
            d_other += s.failure;
        }
        self.carb = (f64::from(self.carb) + d_carb) as f32;
        self.other = (f64::from(self.other) + d_other) as f32;
        (carb_lost, d_carb)
    }
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
        let out = exported(&d, &kd, 1.0, &h, &sea, |c| loss[c], &loss_carb, |c| gain[c]);
        assert_eq!(out, vec![(1, 0.5), (2, 0.5)]);
        // Cell 4 (no carbonate) gives cell 1 as much as cell 0 does: half of it is carbonate,
        // and half of what cell 1 passes on to cell 2.
        let h = [0.0, -1.0, -2.0, -10.0, 0.0, -10.0];
        let out = exported(&d, &kd, 1.0, &h, &sea, |c| loss[c], &loss_carb, |c| gain[c]);
        assert_eq!(out, vec![(1, 0.25), (2, 0.25)]);
        // On land the tracer stops; without carbonate there is none.
        let land = [true, false, true, true, true, true];
        assert!(
            exported(
                &d,
                &kd,
                1.0,
                &h,
                &land,
                |c| loss[c],
                &loss_carb,
                |c| gain[c]
            )
            .is_empty()
        );
        let none = [0.0; 6];
        assert!(exported(&d, &kd, 1.0, &h, &sea, |c| loss[c], &none, |c| gain[c]).is_empty());
    }

    /// The step's own deposit is peeled from the top as it stands at each substep: carbonate an
    /// earlier substep laid and the step's earlier losses count, so the second substep does not
    /// read the column as it stood at the step's start.
    #[test]
    fn the_step_top_is_peeled_as_it_stands_at_each_substep() {
        // The column the step began with: 2 m of regolith, then 10 m of limestone.
        let old = |from: f64, m: f64| -> f64 {
            let (lo, hi) = (from, from + m);
            (hi.min(12.0) - lo.max(2.0)).max(0.0)
        };
        // Substep 1 lays 3 m of clastic and 1 m of carbonate on top and loses nothing.
        let mut top = StepTop::default();
        let s1 = SubstepCell {
            new_other: 3.0,
            fresh: 1.0,
            ..SubstepCell::default()
        };
        assert_eq!(top.substep(&s1, 0.0, &old), (0.0, 0.0));
        assert_eq!((top.carb, top.other, top.taken_old), (1.0, 3.0, 0.0));
        // Substep 2 lays 2 m of carbonate, then its diffusion takes 8 m: its own 2 m of
        // carbonate, the step's 4 m (a quarter carbonate), the 2 m of regolith.
        let s2 = SubstepCell {
            fresh: 2.0,
            diffusion: -8.0,
            ..SubstepCell::default()
        };
        let lost = top.diffusion_carbonate(&s2, &old);
        assert!((lost - 3.0).abs() < 1e-9, "{lost}");
        assert_eq!(
            (top.carb, top.other),
            (1.0, 3.0),
            "a read leaves the record"
        );
        let (l, kept) = top.substep(&s2, 0.0, &old);
        assert!((l - 3.0).abs() < 1e-9 && kept == 0.0);
        assert_eq!((top.carb, top.other, top.taken_old), (0.0, 0.0, 2.0));
        // Substep 3 takes 3 m more: the limestone under the regolith, not the regolith again.
        let s3 = SubstepCell {
            diffusion: -3.0,
            ..SubstepCell::default()
        };
        let (l, _) = top.substep(&s3, 0.0, &old);
        assert!((l - 3.0).abs() < 1e-9, "{l}");
        assert_eq!(top.taken_old, 5.0);
        // Erosion before the substep's deposit comes off the step's earlier deposit first.
        let mut t = StepTop {
            carb: 2.0,
            other: 2.0,
            taken_old: 0.0,
        };
        let s = SubstepCell {
            pre_erosion: 2.0,
            diffusion: -2.0,
            ..SubstepCell::default()
        };
        let (l, _) = t.substep(&s, 0.0, &old);
        assert!((l - 1.0).abs() < 1e-9, "{l}");
    }

    /// Slope failure's debris is clastic, and what fails off a cell takes the carbonate the
    /// diffusion laid there with it: a cell never keeps more periplatform carbonate than the
    /// diffusion gave it, nor more than its net gain.
    #[test]
    fn slope_failure_neither_makes_nor_keeps_periplatform_carbonate() {
        let old = |_: f64, _: f64| 0.0;
        // A toe: 0.5 m of diffusion gain, all carbonate, and 20 m of failure debris on top.
        let mut t = StepTop::default();
        let s = SubstepCell {
            diffusion: 0.5,
            failure: 20.0,
            ..SubstepCell::default()
        };
        let (_, kept) = t.substep(&s, 0.5, &old);
        assert_eq!(kept, 0.5);
        assert!((f64::from(t.other) - 20.0).abs() < 1e-6);
        // The same gain with 0.4 m failing off: a fifth of the carbonate is left.
        let mut t = StepTop::default();
        let s = SubstepCell {
            diffusion: 0.5,
            failure: -0.4,
            ..SubstepCell::default()
        };
        let (_, kept) = t.substep(&s, 0.5, &old);
        assert!((kept - 0.1).abs() < 1e-9, "{kept}");
        // `laid` past the gain is cut to it.
        let mut t = StepTop::default();
        let s = SubstepCell {
            diffusion: 0.5,
            ..SubstepCell::default()
        };
        assert_eq!(t.substep(&s, 3.0, &old).1, 0.5);
    }

    /// The tracer's cut-off is a rate: a substep half as long halves the least exchange.
    #[test]
    fn the_tracer_cut_off_scales_with_the_substep() {
        use crate::surface::grid::Dims;
        let d = Dims {
            w: 2,
            h: 1,
            dx: 1.0,
            wrap_x: false,
        };
        let sea = [true; 2];
        let loss = [1.0, 0.0];
        let loss_carb = [1.0, 0.0];
        let gain = [0.0, 0.5];
        let h = [0.0, -1.0];
        // The exchange is κ·Δt (Δh and Δx 1): 0.8 mm over 2 Myr is under the cut-off of 1 mm...
        let kd = [0.8e-3 / 2.0e6; 2];
        assert!(
            exported(
                &d,
                &kd,
                2.0e6,
                &h,
                &sea,
                |c| loss[c],
                &loss_carb,
                |c| gain[c]
            )
            .is_empty()
        );
        // ...and 0.6 mm over 1 Myr is over the cut-off of 0.5 mm then.
        let kd = [0.6e-3 / 1.0e6; 2];
        assert_eq!(
            exported(
                &d,
                &kd,
                1.0e6,
                &h,
                &sea,
                |c| loss[c],
                &loss_carb,
                |c| gain[c]
            ),
            vec![(1, 0.5)]
        );
    }

    /// The tracer conserves carbonate: on the exact face exchanges of an implicit step (the
    /// heights before it rebuilt from the solved ones), what it lays is the carbonate the losers
    /// shed when every cell is sea and carries the tracer, and at most that with land around.
    #[test]
    fn the_tracer_lays_no_more_carbonate_than_the_diffusion_took() {
        use crate::surface::grid::Dims;
        let (w, rows) = (9, 7);
        let n = w * rows;
        let d = Dims {
            w,
            h: rows,
            dx: 1.0,
            wrap_x: true,
        };
        let mut seed = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let h: Vec<f64> = (0..n).map(|_| -50.0 * next()).collect();
        let kd: Vec<f64> = (0..n).map(|_| 0.01 + 0.04 * next()).collect();
        // The implicit step: h − before = Σ_j c_ij (h_j − h_i), the faces as `exported` takes
        // them (x wraps).
        let face = |c: usize, m: usize| 0.5 * (kd[c] + kd[m]) * (h[m] - h[c]);
        let change: Vec<f64> = (0..n)
            .map(|c| {
                let (i, j) = (c % w, c / w);
                let mut s = face(c, j * w + (i + 1) % w) + face(c, j * w + (i + w - 1) % w);
                if j > 0 {
                    s += face(c, c - w);
                }
                if j + 1 < rows {
                    s += face(c, c + w);
                }
                s
            })
            .collect();
        let loss: Vec<f64> = change.iter().map(|&x| (-x).max(0.0)).collect();
        let gain: Vec<f64> = change.iter().map(|&x| x.max(0.0)).collect();
        // Every loser's loss at least half carbonate, so no share falls under the cut-off.
        let loss_carb: Vec<f64> = loss.iter().map(|&l| l * (0.5 + 0.5 * next())).collect();
        let lost: f64 = loss_carb.iter().sum();
        assert!(lost > 0.0);
        let laid = |sea: &[bool]| -> f64 {
            exported(&d, &kd, 1.0, &h, sea, |c| loss[c], &loss_carb, |c| gain[c])
                .iter()
                .map(|&(_, m)| m)
                .sum()
        };
        let all = laid(&vec![true; n]);
        assert!(
            (all - lost).abs() <= 1.0e-5 * lost,
            "laid {all} of {lost} m"
        );
        let some: Vec<bool> = (0..n).map(|c| c % 5 != 2).collect();
        let part = laid(&some);
        assert!(part <= lost * (1.0 + 1.0e-5), "laid {part} of {lost} m");
        assert!(part < all, "land dilutes the tracer");
    }
}
