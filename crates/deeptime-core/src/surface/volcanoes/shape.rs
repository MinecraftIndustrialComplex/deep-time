// SPDX-License-Identifier: LGPL-3.0-or-later
//! Volcano shapes (P1-21): the radial profiles of the edifices, their rendering onto a planet grid
//! and the band-pass residuals the young landscape and the amplifier add at finer grids.
//!
//! Distances are block-metric (the game's flat world: block offsets × `m_per_block_h`), so a
//! volcano is round in the world whatever its latitude, like the plutons (`structure/plutons.rs`).
//!
//! **Profiles** (ρ = r / R, R the basal radius; the body is 1 at the summit and 0 at the foot):
//! - stratovolcano: (1 − ρ)^1.8, a concave cone (upper flanks ~25° at 2.5 km over R = 12 km);
//! - shield: (1 − ρ²)^1.5, broad and convex with flanks under ~10°;
//! - silicic centre (a continental hotspot): (1 − ρ²)², low and broad.
//!
//! A **depression** (a summit crater, or a caldera once the edifice collapsed) truncates the body
//! at the rim level (the body becomes flat-topped inside the rim) and is subtracted as a bowl:
//! steep-sided for a crater, flat-floored with steep walls for a caldera. The deep-time surface on
//! L1 sees only the body; the bowl is cut at the finest grid (`amplify/volcano.rs`), after the
//! amplifier's flood, which would fill it.
//!
//! **Residuals.** A grid of cell size dx represents a shape S about as S convolved with a tent of
//! half-width dx (point samples, then linear interpolation, averaged over where the shape falls
//! in a cell): `blurred(dx)`. The deep-time surface holds the body at L1; the young landscape's
//! grid can hold `blurred(dx2) − blurred(dx1)` more of it and the last grid `S − blurred(dx2)`.
//! Both are radial tables per volcano, scaled by the height the surface preserved.

use serde::{Deserialize, Serialize};

/// What made the volcano.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// An arc stratovolcano.
    Strato,
    /// An oceanic hotspot shield volcano.
    Shield,
    /// A continental hotspot's silicic centre (a caldera complex).
    Silicic,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Strato => "stratovolcano",
            Kind::Shield => "shield",
            Kind::Silicic => "caldera_complex",
        }
    }
}

/// A volcano's depression.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Depression {
    /// Rim radius / basal radius.
    pub rho: f64,
    /// Depth below the rim, as a share of the edifice's height.
    pub depth: f64,
    /// A caldera (flat floor), else a crater (a bowl).
    pub caldera: bool,
}

/// A volcano's normalised shape.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub kind: Kind,
    pub depression: Option<Depression>,
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Shape {
    /// The untruncated profile at ρ.
    pub fn profile(kind: Kind, rho: f64) -> f64 {
        if !(0.0..1.0).contains(&rho) {
            return if rho < 0.0 { 1.0 } else { 0.0 };
        }
        match kind {
            Kind::Strato => (1.0 - rho).powf(1.8),
            Kind::Shield => (1.0 - rho * rho).powf(1.5),
            Kind::Silicic => (1.0 - rho * rho).powi(2),
        }
    }

    /// The body at ρ: the profile, truncated at the rim of a depression.
    pub fn body(&self, rho: f64) -> f64 {
        let f = Self::profile(self.kind, rho);
        match self.depression {
            Some(d) => f.min(Self::profile(self.kind, d.rho)),
            None => f,
        }
    }

    /// The depression's depth below the body at ρ (≥ 0), as a share of the height.
    pub fn bowl(&self, rho: f64) -> f64 {
        let Some(d) = self.depression else {
            return 0.0;
        };
        if d.rho <= 0.0 || rho >= d.rho {
            return 0.0;
        }
        let x = rho / d.rho;
        if d.caldera {
            d.depth * (1.0 - smoothstep(0.78, 1.0, x))
        } else {
            d.depth * (1.0 - x * x).sqrt()
        }
    }

    /// The full shape at ρ: body less the bowl.
    pub fn full(&self, rho: f64) -> f64 {
        self.body(rho) - self.bowl(rho)
    }
}

/// A radial table: values at ρ = k · step, zero beyond the last.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Radial {
    pub step: f64,
    pub v: Vec<f32>,
}

impl Radial {
    /// Linear interpolation at ρ (0 beyond the table).
    pub fn at(&self, rho: f64) -> f64 {
        if self.v.is_empty() || !(rho >= 0.0) {
            return 0.0;
        }
        let q = rho / self.step;
        let k = q.floor() as usize;
        if k + 1 >= self.v.len() {
            return 0.0;
        }
        let f = q - k as f64;
        f64::from(self.v[k]) * (1.0 - f) + f64::from(self.v[k + 1]) * f
    }

    /// The last ρ the table covers.
    pub fn reach(&self) -> f64 {
        self.step * self.v.len().saturating_sub(1) as f64
    }
}

/// Taps per side of the tent kernel.
const TAPS: i32 = 4;
/// Entries of a radial table.
const ENTRIES: usize = 96;

/// `f` convolved with a separable tent of half-width `h` (in ρ units), sampled along a radius.
/// `h` ≤ 0 returns `f` itself.
pub fn blurred(f: &dyn Fn(f64) -> f64, h: f64) -> Radial {
    let reach = 1.0 + 2.0 * h.max(0.0) + 0.05;
    let step = reach / (ENTRIES - 1) as f64;
    let mut wts = Vec::new();
    let mut total = 0.0;
    for i in -TAPS..=TAPS {
        let wi = 1.0 - f64::from(i.abs()) / f64::from(TAPS + 1);
        for j in -TAPS..=TAPS {
            let wj = 1.0 - f64::from(j.abs()) / f64::from(TAPS + 1);
            let (u, v) = (
                h * f64::from(i) / f64::from(TAPS + 1),
                h * f64::from(j) / f64::from(TAPS + 1),
            );
            wts.push((u, v, wi * wj));
            total += wi * wj;
        }
    }
    let v = (0..ENTRIES)
        .map(|k| {
            let rho = k as f64 * step;
            if h <= 0.0 {
                return f(rho) as f32;
            }
            let s: f64 = wts
                .iter()
                .map(|&(u, w, wt)| wt * f((rho + u).hypot(w)))
                .sum();
            (s / total) as f32
        })
        .collect();
    Radial { step, v }
}

/// The band-pass residuals of a shape between two grids: `fine − coarse`, where each is the shape
/// blurred at its grid (half-width in ρ units; `fine` ≤ 0 is the shape itself).
pub fn band(f: &dyn Fn(f64) -> f64, fine: f64, coarse: f64) -> Radial {
    let a = blurred(f, fine);
    let b = blurred(f, coarse);
    let reach = a.reach().max(b.reach());
    let step = reach / (ENTRIES - 1) as f64;
    let v = (0..ENTRIES)
        .map(|k| {
            let rho = k as f64 * step;
            let fa = if fine <= 0.0 { f(rho) } else { a.at(rho) };
            (fa - b.at(rho)) as f32
        })
        .collect();
    Radial { step, v }
}

/// The cell-averaged weight of a normalised radial function over the cell centred on
/// (`dx_m`, `dz_m`) (metres from the volcano's centre) of side `cell_m`, for basal radius `r_m`:
/// the mean of 4 × 4 samples.
pub fn cell_weight(f: &dyn Fn(f64) -> f64, dx_m: f64, dz_m: f64, cell_m: f64, r_m: f64) -> f64 {
    let mut s = 0.0;
    for a in 0..4 {
        for b in 0..4 {
            let u = dx_m + cell_m * ((f64::from(a) + 0.5) / 4.0 - 0.5);
            let v = dz_m + cell_m * ((f64::from(b) + 0.5) / 4.0 - 0.5);
            s += f(u.hypot(v) / r_m);
        }
    }
    s / 16.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_have_a_summit_and_a_foot() {
        for k in [Kind::Strato, Kind::Shield, Kind::Silicic] {
            assert!((Shape::profile(k, 0.0) - 1.0).abs() < 1e-12);
            assert_eq!(Shape::profile(k, 1.0), 0.0);
            // Monotone.
            let mut last = 2.0;
            for i in 0..=100 {
                let v = Shape::profile(k, f64::from(i) / 100.0);
                assert!(v <= last + 1e-12, "{k:?}");
                last = v;
            }
        }
    }

    #[test]
    fn a_caldera_truncates_and_floors() {
        let s = Shape {
            kind: Kind::Strato,
            depression: Some(Depression {
                rho: 0.3,
                depth: 0.2,
                caldera: true,
            }),
        };
        let rim = Shape::profile(Kind::Strato, 0.3);
        assert!((s.body(0.0) - rim).abs() < 1e-12);
        assert!((s.full(0.0) - (rim - 0.2)).abs() < 1e-12);
        assert!((s.full(0.1) - (rim - 0.2)).abs() < 1e-12, "flat floor");
        assert!((s.full(0.3) - rim).abs() < 1e-12, "the rim");
        assert!(s.full(0.5) < rim);
    }

    #[test]
    fn blurring_keeps_volume_and_lowers_the_peak() {
        let f = |r: f64| Shape::profile(Kind::Strato, r);
        let b = blurred(&f, 0.4);
        assert!(b.at(0.0) < 0.9 && b.at(0.0) > 0.3, "peak {}", b.at(0.0));
        // Volume (∫ f 2πρ dρ) is kept by a normalised kernel.
        let vol = |g: &dyn Fn(f64) -> f64| {
            let n = 4000;
            (0..n)
                .map(|i| {
                    let r = (f64::from(i) + 0.5) / f64::from(n) * 2.0;
                    g(r) * r * 2.0 / f64::from(n)
                })
                .sum::<f64>()
        };
        let v0 = vol(&f);
        let v1 = vol(&|r| b.at(r));
        assert!((v1 / v0 - 1.0).abs() < 0.03, "{v0} vs {v1}");
        // The band between a grid and itself is empty; the sharpening residual sums to ~0.
        let z = band(&f, 0.3, 0.3);
        assert!(z.v.iter().all(|x| x.abs() < 1e-6));
        let r = band(&f, 0.0, 0.4);
        assert!(r.at(0.0) > 0.1);
        assert!(vol(&|x| r.at(x)).abs() < 0.03 * v0);
    }

    #[test]
    fn cell_weights_average_the_profile() {
        let f = |r: f64| Shape::profile(Kind::Shield, r);
        let w = cell_weight(&f, 0.0, 0.0, 100.0, 10_000.0);
        assert!(w > 0.99 && w <= 1.0);
        assert_eq!(cell_weight(&f, 20_000.0, 0.0, 100.0, 10_000.0), 0.0);
    }
}
