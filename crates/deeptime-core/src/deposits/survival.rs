// SPDX-License-Identifier: LGPL-3.0-or-later
//! The survivor formula (plan §8.6 step 2, as built 2026-09-29; Ben: "treat the USGS densities as
//! deposits that survive to the present … and let the engine form extra so the survivors match;
//! no per-type magic multipliers").
//!
//! USGS deposit densities count the deposits known today in well-explored permissive tracts
//! (Singer 1993; Singer, Berger & Menzie 2005): the ones that *survived* burial, uplift and
//! erosion. Deposits form, then diffuse through the crust (Kesler & Wilkinson 2006, 2008 — of
//! about 126,000 Phanerozoic porphyries some 48,000 remain). The engine therefore aims the
//! density at the survivors and derives how many form:
//!
//! 1. **Target.** A type's expected survivors on the planet are N* = count_mult · ρ/10⁵ · Σ A_i
//!    over its permissive tracts (the USGS three-part method: density × permissive area; ρ the
//!    type's density: the mean of a measured distribution, else its P50,
//!    `Sizing::density_target`; count_mult the preset's times the type's `balance.count_mult`,
//!    both 1 by default).
//! 2. **Where deposits form.** Tract i receives formation in proportion to φ_i = A_i · F̄_i · E_i
//!    (area, mean favourability, province endowment: the plan's ρ·A·F·E without ρ), a deposit's
//!    position within the tract ∝ its cell's favourability × endowment (`cluster.rs`), and its
//!    formation time ∝ its cell's favourability hazard −(Δt/τ)·ln(1 − F_t) frame by frame — the
//!    same hazard whose sum is the integrated favourability (`rules.rs`), so a cell favourable for
//!    100 Myr forms deposits throughout those 100 Myr.
//! 3. **Preservation.** A deposit formed at time t at depth z_f with half-height h(T) (T its
//!    tonnage) lies at D = z_f + B − E below today's surface, B and E the burial and erosion its
//!    column saw since (`ledger.rs`). It survives — becomes a site — when the surface never came
//!    down through it: D_min > −h, D_min = z_f + the lowest running net burial since formation
//!    (D_min ≥ h for a type that any erosion into destroys; a deposit eroded away and then buried
//!    again is gone, 2026-10-05) — and its body still reaches the build range (`too_deep`). p(c) is that probability for a deposit formed in cell c: averaged over
//!    the formation-time distribution (the hazard, and uniform within a frame, as the engine
//!    draws it), and over the tonnage distribution at [`TONNAGE_NODES`] quantiles. Within a frame
//!    D is linear in the formation instant, and D_min is the smaller of z_f and a linear function
//!    of it, so that part is exact. p̄_i averages p over tract i
//!    with the position weights.
//! 4. **Formation rate.** One number per type: λ_i = N* · φ_i / Σ_j φ_j p̄_j, so the expected
//!    survivors Σ λ_i p̄_i are exactly N*. Formation concentrates nowhere; survival decides where
//!    today's deposits are (an old, deeply eroded arc formed as many as a young one and kept few,
//!    as on Earth).
//! 5. **Cap.** If the planet's formation-weighted preservation P̄ = Σ φ p̄ / Σ φ is below
//!    1 / `max_formed_per_survivor` (preset, default 20), formation stops at that multiple and the
//!    survivors fall short of N*; the census says so (`capped`).

use super::types::{DepositType, ErosionBehavior};

/// Tonnage quantiles the survival is averaged over (midpoints of equal-probability bins).
pub const TONNAGE_NODES: usize = 32;

/// What decides a deposit's fate, for one type.
#[derive(Clone, Debug, PartialEq)]
pub struct FateModel {
    /// Depth of the body's centre when it formed (m).
    pub formation_depth_m: f64,
    /// Half-heights of the bodies at the tonnage nodes (m).
    pub half_heights_m: Vec<f64>,
    /// `balance.size_mult_v` (the build-range check scales the body's half-height).
    pub size_mult_v: f64,
    pub behavior: ErosionBehavior,
    /// A survivor counts toward the density when its top lies at most this deep (m;
    /// `preservation.assessment_depth_m`, the depth USGS assessments count deposits to).
    pub assessment_depth_m: f64,
}

impl FateModel {
    pub fn new(t: &DepositType) -> FateModel {
        let tonnage = super::sizing::SizeModel::new(&t.sizing).tonnage;
        let half_heights_m = (0..TONNAGE_NODES)
            .map(|m| {
                let q = (m as f64 + 0.5) / TONNAGE_NODES as f64;
                super::geometry::half_height_m(t, tonnage.quantile(q) * 1.0e6)
            })
            .collect();
        FateModel {
            formation_depth_m: t.preservation.formation_depth_m,
            half_heights_m,
            size_mult_v: t.balance.size_mult_v,
            behavior: t.preservation.erosion_behavior,
            assessment_depth_m: t.preservation.assessment_depth_m,
        }
    }

    /// Survives a body of half-height `h` whose centre is `d` below the surface now and was
    /// `d_min` below it at its shallowest since it formed, with `limit_m` the depth below which a
    /// body's top misses the build range ([`too_deep_limit_m`])? Mirrors `Ledger::apply` and the
    /// stage's too-deep check. With `counted`, the survivor must also have its top within the
    /// assessment depth to count toward the density.
    pub fn survives(&self, d: f64, d_min: f64, h: f64, limit_m: f64, counted: bool) -> bool {
        let kept = match self.behavior {
            ErosionBehavior::Destroy => d_min - h >= 0.0,
            ErosionBehavior::Truncate | ErosionBehavior::SpawnPlacer => d_min + h > 0.0,
        };
        kept && d - h * self.size_mult_v <= limit_m
            && (!counted || d - h <= self.assessment_depth_m)
    }

    /// The share of tonnage nodes and of the formation instants U ∈ [0, 1] of one frame for which
    /// a deposit survives, its centre depth now being `a + b·U` and at its shallowest
    /// min(z_f, `low` + b·U) (a: today's depth of a deposit formed at the frame's end, `low`: its
    /// shallowest depth since, b: the frame's own net burial, m).
    pub fn survival_in_frame(&self, a: f64, low: f64, b: f64, limit_m: f64, counted: bool) -> f64 {
        let mut s = 0.0;
        for &h in &self.half_heights_m {
            // Kept: the surface never reached the body's base (any of it, for `destroy`).
            let kept = match self.behavior {
                ErosionBehavior::Destroy if self.formation_depth_m < h => None,
                ErosionBehavior::Destroy => u_range(low, b, h, f64::INFINITY),
                ErosionBehavior::Truncate | ErosionBehavior::SpawnPlacer => {
                    u_range(low, b, -h, f64::INFINITY)
                }
            };
            let (lo, hi) = self.window(h, limit_m, counted);
            if let (Some(k), Some(w)) = (kept, u_range(a, b, lo, hi)) {
                s += (k.1.min(w.1) - k.0.max(w.0)).max(0.0);
            }
        }
        s / self.half_heights_m.len().max(1) as f64
    }

    /// The share of tonnage nodes for which a deposit `d` deep now and `d_min` deep at its
    /// shallowest survives.
    pub fn survival_at(&self, d: f64, d_min: f64, limit_m: f64, counted: bool) -> f64 {
        let n = self
            .half_heights_m
            .iter()
            .filter(|&&h| self.survives(d, d_min, h, limit_m, counted))
            .count();
        n as f64 / self.half_heights_m.len().max(1) as f64
    }

    /// The open-closed depth window (lo, hi] a body of half-height `h` survives in.
    fn window(&self, h: f64, limit_m: f64, counted: bool) -> (f64, f64) {
        let lo = match self.behavior {
            // d ≥ h: the window's lower end is closed, which has measure zero.
            ErosionBehavior::Destroy => h,
            ErosionBehavior::Truncate | ErosionBehavior::SpawnPlacer => -h,
        };
        let hi = limit_m + h * self.size_mult_v;
        if counted {
            (lo, hi.min(self.assessment_depth_m + h))
        } else {
            (lo, hi)
        }
    }
}

/// The depth (m) below today's surface a body's centre may reach with its top still in the build
/// range: the stage drops a body whose top lies below `min_y + 4`.
pub fn too_deep_limit_m(surface_y: f64, min_y: i32, m_per_block_v: f64) -> f64 {
    (surface_y - f64::from(min_y + 4)) * m_per_block_v
}

/// The range of U ∈ [0, 1] with lo < a + b·U ≤ hi (its ends; None when empty).
fn u_range(a: f64, b: f64, lo: f64, hi: f64) -> Option<(f64, f64)> {
    if hi <= lo {
        return None;
    }
    if b == 0.0 {
        return (a > lo && a <= hi).then_some((0.0, 1.0));
    }
    let (u1, u2) = ((lo - a) / b, (hi - a) / b);
    let (from, to) = if u1 < u2 { (u1, u2) } else { (u2, u1) };
    let (from, to) = (from.max(0.0), to.min(1.0));
    (to > from).then_some((from, to))
}

/// The measure of U ∈ [0, 1] with lo < a + b·U ≤ hi.
pub fn fraction_between(a: f64, b: f64, lo: f64, hi: f64) -> f64 {
    if hi <= lo {
        return 0.0;
    }
    if b == 0.0 {
        return if a > lo && a <= hi { 1.0 } else { 0.0 };
    }
    let (u1, u2) = ((lo - a) / b, (hi - a) / b);
    let (from, to) = if u1 < u2 { (u1, u2) } else { (u2, u1) };
    (to.min(1.0) - from.max(0.0)).max(0.0)
}

/// A column's history as the survival needs it: per frame, whether the crust existed and its net
/// burial over the frame (deposition − erosion, m).
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnNet {
    /// Net burial of each frame (m; 0 where the crust did not exist).
    pub net_m: Vec<f64>,
    /// Σ of `net_m` over the frames after each frame.
    after_m: Vec<f64>,
    /// The lowest the running net burial after the end of each frame gets, at a frame's end (m,
    /// ≤ 0): how far the surface came down towards what lay under it then, whatever burial
    /// followed.
    low_after_m: Vec<f64>,
}

impl ColumnNet {
    pub fn new(net_m: Vec<f64>) -> ColumnNet {
        let n = net_m.len();
        let mut after_m = vec![0.0; n];
        let mut low_after_m = vec![0.0; n];
        let mut s = 0.0;
        let mut low = 0.0f64;
        for k in (0..n).rev() {
            after_m[k] = s;
            low_after_m[k] = low;
            s += net_m[k];
            low = (net_m[k] + low).min(0.0);
        }
        ColumnNet {
            net_m,
            after_m,
            low_after_m,
        }
    }

    /// Net burial after the end of frame `k`.
    pub fn after(&self, k: usize) -> f64 {
        self.after_m[k]
    }

    /// The lowest running net burial after the end of frame `k` (≤ 0).
    pub fn low_after(&self, k: usize) -> f64 {
        self.low_after_m[k]
    }

    /// The lowest running net burial over the whole history (≤ 0).
    pub fn low_total(&self) -> f64 {
        match self.net_m.first() {
            Some(&n0) => (n0 + self.low_after_m[0]).min(0.0),
            None => 0.0,
        }
    }

    /// Net burial over the whole history.
    pub fn total(&self) -> f64 {
        self.net_m.iter().sum()
    }
}

/// When a type's deposits form.
#[derive(Clone, Copy, Debug)]
pub enum Formation<'a> {
    /// Frame k with probability ∝ `hazard[k]`, uniform within it.
    Hazard(&'a [f32]),
    /// At a known instant inside frame `k`: `u` of the frame's span before its end (a charged
    /// trap: its charge time).
    At { k: usize, u: f64 },
    /// Before the history began (inherited deposits, `formation_time: cause`).
    Start,
}

/// p(c): the probability that a deposit formed in this column survives to the present (with
/// `counted`, survives with its top within the assessment depth).
pub fn preservation(
    model: &FateModel,
    col: &ColumnNet,
    when: Formation<'_>,
    limit_m: f64,
    counted: bool,
) -> f64 {
    let z = model.formation_depth_m;
    match when {
        Formation::Hazard(h) => {
            let mut num = 0.0;
            let mut den = 0.0;
            for (k, &hk) in h.iter().enumerate() {
                let hk = f64::from(hk);
                if hk <= 0.0 {
                    continue;
                }
                den += hk;
                num += hk
                    * model.survival_in_frame(
                        z + col.after(k),
                        z + col.low_after(k),
                        col.net_m[k],
                        limit_m,
                        counted,
                    );
            }
            if den > 0.0 { num / den } else { 0.0 }
        }
        Formation::At { k, u } => {
            let bu = u.clamp(0.0, 1.0) * col.net_m[k];
            let d = z + col.after(k) + bu;
            let d_min = z + (bu + col.low_after(k)).min(0.0);
            model.survival_at(d, d_min, limit_m, counted)
        }
        Formation::Start => {
            model.survival_at(z + col.total(), z + col.low_total(), limit_m, counted)
        }
    }
}

/// The formation plan of one type (steps 1, 4 and 5 of the module doc).
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    /// N*: the survivors the density asks for.
    pub target: f64,
    /// P̄: the formation-weighted preservation of the planet's tracts.
    pub preservation: f64,
    /// The formation multiple applied, 1 / max(P̄, 1 / cap).
    pub formation_mult: f64,
    /// Whether the cap limited it.
    pub capped: bool,
    /// λ_i per tract: the expected number of deposits formed there.
    pub formed: Vec<f64>,
    /// Σ λ_i p̄_i.
    pub expected_survivors: f64,
}

/// λ_i for tracts with areas `area_km2`, formation weights `phi` (A·F̄·E) and preservation
/// `pbar`, for density ρ (per 100,000 km²), `count_mult` and the cap.
pub fn plan(
    rho: f64,
    count_mult: f64,
    area_km2: &[f64],
    phi: &[f64],
    pbar: &[f64],
    max_formed_per_survivor: f64,
) -> Plan {
    let target = count_mult * rho / 1.0e5 * area_km2.iter().sum::<f64>();
    let sphi: f64 = phi.iter().sum();
    let kept: f64 = phi.iter().zip(pbar).map(|(f, p)| f * p).sum();
    if sphi.is_nan() || sphi <= 0.0 || target.is_nan() || target <= 0.0 {
        return Plan {
            target: target.max(0.0),
            preservation: 0.0,
            formation_mult: 1.0,
            capped: false,
            formed: vec![0.0; phi.len()],
            expected_survivors: 0.0,
        };
    }
    let p = kept / sphi;
    let floor = 1.0 / max_formed_per_survivor.max(1.0);
    let capped = p < floor;
    let eff = p.max(floor);
    let formed: Vec<f64> = phi.iter().map(|f| target * f / (eff * sphi)).collect();
    let expected_survivors = formed.iter().zip(pbar).map(|(l, p)| l * p).sum();
    Plan {
        target,
        preservation: p,
        formation_mult: 1.0 / eff,
        capped,
        formed,
        expected_survivors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::ledger::{Fate, Ledger};
    use crate::deposits::types::builtin;

    /// Where a candidate's ledger sends it in the stage: a site unless eroded or too deep.
    fn is_site(fate: Fate, too_deep: bool) -> bool {
        !matches!(fate, Fate::Eroded) && !too_deep
    }

    #[test]
    fn fraction_between_is_the_interval_length() {
        assert_eq!(fraction_between(0.0, 1.0, 0.25, 0.75), 0.5);
        assert_eq!(fraction_between(1.0, -1.0, 0.25, 0.75), 0.5);
        assert_eq!(fraction_between(0.5, 0.0, 0.25, 0.75), 1.0);
        assert_eq!(fraction_between(0.5, 0.0, 0.5, 0.75), 0.0);
        assert_eq!(fraction_between(-2.0, 1.0, 0.0, 10.0), 0.0);
        assert!((fraction_between(-0.5, 2.0, 0.0, 10.0) - 0.75).abs() < 1e-12);
    }

    /// The survival rule is the ledger's: for many depths and heights, `survives` agrees with
    /// `Ledger::apply` plus the stage's too-deep check.
    #[test]
    fn survives_agrees_with_the_ledger() {
        for t in builtin().unwrap().iter().filter(|t| !t.child_only) {
            let m = FateModel::new(t);
            let limit = 3000.0;
            for &h in &m.half_heights_m {
                for d10 in -400..=900 {
                    let d = f64::from(d10) * 10.0;
                    // Formed at 2 km (when that is deeper), exhumed to d, or exhumed to d − 1 km
                    // and buried 1 km again.
                    let z = d.max(2000.0);
                    for (eroded, buried) in [(z - d, 0.0), (z - d + 1000.0, 1000.0)] {
                        let ledger = Ledger {
                            formation_depth_m: z,
                            half_height_m: h,
                            eroded_m: eroded,
                            buried_m: buried,
                            exhumed_max_m: eroded,
                        };
                        let fate = ledger.apply(m.behavior, t.shape().metric());
                        let too_deep = d - h * m.size_mult_v > limit;
                        let d_min = z - eroded;
                        if (d_min + h).abs() < 1e-6 || (d_min - h).abs() < 1e-6 {
                            continue;
                        }
                        assert_eq!(
                            m.survives(d, d_min, h, limit, false),
                            is_site(fate, too_deep),
                            "{} d={d} d_min={d_min} h={h}",
                            t.id
                        );
                    }
                }
            }
        }
    }

    /// Uniform erosion r over a history of length L with formation uniform in time: the survival
    /// is E_T[min(1, (z_f + h(T)) / (r·L))] (Kesler & Wilkinson's tectonic-diffusion picture with
    /// no burial), and the frame-exact integration reproduces it.
    #[test]
    fn uniform_erosion_matches_the_closed_form() {
        let t = builtin().unwrap().remove(0); // porphyry: z_f 1.5 km
        let m = FateModel::new(&t);
        let (frames, dt_myr, r) = (50usize, 10.0, 8.0);
        let col = ColumnNet::new(vec![-r * dt_myr; frames]);
        let hazard = vec![1.0f32; frames];
        let mut m = m;
        m.assessment_depth_m = 1.0e9;
        let p = preservation(&m, &col, Formation::Hazard(&hazard), 1.0e9, true);
        assert_eq!(
            p,
            preservation(&m, &col, Formation::Hazard(&hazard), 1.0e9, false)
        );
        let span = r * dt_myr * frames as f64;
        let want = m
            .half_heights_m
            .iter()
            .map(|h| ((m.formation_depth_m + h) / span).min(1.0))
            .sum::<f64>()
            / m.half_heights_m.len() as f64;
        assert!((p - want).abs() < 1e-9, "{p} vs {want}");
        // Formation only in the last 10 % of the history: everything survives this erosion.
        let mut late = vec![0.0f32; frames];
        late[frames - 5..].iter_mut().for_each(|x| *x = 1.0);
        assert!(preservation(&m, &col, Formation::Hazard(&late), 1.0e9, false) > 0.999);
        // Counted within 1 km: a deposit formed at 1.5 km counts once erosion has brought its top
        // within 1 km, and stops counting when it is gone; the counted share is a part of the
        // surviving one.
        m.assessment_depth_m = 1000.0;
        let counted = preservation(&m, &col, Formation::Hazard(&hazard), 1.0e9, true);
        let want_c = m
            .half_heights_m
            .iter()
            .map(|h| {
                let (z, r) = (m.formation_depth_m, 8.0);
                // Age window in which z − r·age − h ≤ 1 km and z − r·age + h > 0.
                let young = ((z - h - 1000.0) / r).max(0.0);
                let old = ((z + h) / r).min(500.0);
                ((old - young) / 500.0).max(0.0)
            })
            .sum::<f64>()
            / m.half_heights_m.len() as f64;
        assert!((counted - want_c).abs() < 1e-9, "{counted} vs {want_c}");
        assert!(counted < p);
    }

    /// A deposit the surface reached and that was buried again is gone: erosion of 5 km then
    /// burial of 5 km leaves a porphyry formed at 1.5 km "at 1.5 km" by the net, but it was
    /// eroded away in between; formed after the erosion, it survives. The frame integration
    /// agrees with the per-instant rule (the erosion and burial audit, 2026-10-05).
    #[test]
    fn eroded_then_buried_is_gone() {
        let t = builtin().unwrap().remove(0); // porphyry: z_f 1.5 km
        let m = FateModel::new(&t);
        // 20 frames: 10 of 500 m of erosion each, then 10 of 500 m of burial.
        let mut net = vec![-500.0; 10];
        net.extend(vec![500.0; 10]);
        let col = ColumnNet::new(net);
        assert_eq!(col.total(), 0.0);
        assert_eq!(col.low_total(), -5000.0);
        assert_eq!(col.low_after(4), -2500.0);
        assert_eq!(col.low_after(12), 0.0);
        let limit = 1.0e9;
        // Formed before the erosion: the net says 1.5 km down, the path says gone.
        assert_eq!(
            preservation(&m, &col, Formation::Start, limit, false),
            0.0,
            "eroded away, then buried"
        );
        // Formed at the trough (the end of frame 9): buried 5 km since, kept.
        let at_trough = preservation(&m, &col, Formation::At { k: 9, u: 0.0 }, limit, false);
        assert_eq!(at_trough, 1.0);
        // Hazard in frame 5 only: what survives is what formed late enough in it that the
        // remaining 2–2.5 km of erosion left its base under ground; the frame integration equals
        // the mean of the per-instant rule.
        let mut hz = vec![0.0f32; 20];
        hz[5] = 1.0;
        let p = preservation(&m, &col, Formation::Hazard(&hz), limit, false);
        let n = 2000;
        let mean = (0..n)
            .map(|i| {
                let u = (i as f64 + 0.5) / n as f64;
                preservation(&m, &col, Formation::At { k: 5, u }, limit, false)
            })
            .sum::<f64>()
            / n as f64;
        assert!((p - mean).abs() < 2e-3, "{p} vs {mean}");
        assert!(p < 1.0, "{p}");
    }

    #[test]
    fn the_plan_hits_the_target_in_expectation_and_caps() {
        let p = plan(
            15.0,
            1.0,
            &[1.0e4, 3.0e4],
            &[5.0e3, 1.0e4],
            &[0.2, 0.6],
            20.0,
        );
        assert!((p.target - 6.0).abs() < 1e-12);
        assert!((p.expected_survivors - p.target).abs() < 1e-9);
        assert!(!p.capped);
        // Formation ∝ φ.
        assert!((p.formed[1] / p.formed[0] - 2.0).abs() < 1e-12);
        let c = plan(15.0, 1.0, &[1.0e4], &[5.0e3], &[0.01], 20.0);
        assert!(c.capped);
        assert!((c.formation_mult - 20.0).abs() < 1e-12);
        assert!((c.expected_survivors - c.target * 0.2).abs() < 1e-9);
    }
}
