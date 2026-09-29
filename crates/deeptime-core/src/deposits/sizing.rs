// SPDX-License-Identifier: LGPL-3.0-or-later
//! Sizes and grades (plan §8.6 step 4): lognormal tonnage fitted to the USGS percentiles,
//! μ = ln P50, σ = (ln P10 − ln P90)/2.563 (2.563 = 2 · Φ⁻¹(0.9); USGS convention: P90 is the
//! value 90 % of deposits exceed), and grades as lognormals tied to tonnage and to each other by a
//! Gaussian copula on the logs with the published correlations (porphyry Au vs tonnage
//! r = −0.28, USGS OFR 2008-1155). A grade whose P90 is zero (fewer than 90 % of deposits report
//! it) is fitted to P50 and P10 and present with the type's `presence` probability.
//!
//! The lognormal is **split** at the median: σ_lo = (ln P50 − ln P90)/Φ⁻¹(0.9) below it and
//! σ_hi = (ln P10 − ln P50)/Φ⁻¹(0.9) above it, so all three published percentiles are reproduced
//! exactly (USGS P50s are not the geometric mean of P90 and P10: porphyry's 240 Mt against
//! √(33 · 1,500) = 222 Mt). The mean of the two is the plan's σ, 1.49 for porphyry.

use std::collections::BTreeMap;

use super::random::{Rng, normal_cdf, normal_quantile};
use super::types::{GradeDist, Percentiles, Sizing};

/// Φ⁻¹(0.9).
pub const Z90: f64 = 1.281_551_565_544_600_5;

/// σ of a grade lognormal that has only one positive published percentile (a placeholder, not a
/// USGS value).
pub const DEFAULT_SIGMA: f64 = 1.0;

/// A split lognormal distribution (see the module doc).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lognormal {
    pub mu: f64,
    pub sigma_lo: f64,
    pub sigma_hi: f64,
}

impl Lognormal {
    /// From USGS P90/P50/P10.
    pub fn from_percentiles(p: &Percentiles) -> Lognormal {
        Lognormal {
            mu: p.p50.ln(),
            sigma_lo: (p.p50.ln() - p.p90.ln()) / Z90,
            sigma_hi: (p.p10.ln() - p.p50.ln()) / Z90,
        }
    }

    /// The distribution of a grade **among deposits that carry the metal**. USGS by-product
    /// percentiles count unreported grades as zero (OFR 2008-1155, p. 5), so with a fraction
    /// `presence` reporting, the overall level L (0.1 for P90, 0.5 for P50, 0.9 for P10) is the
    /// conditional level (L − (1 − presence)) / presence. Three positive percentiles with every
    /// deposit reporting give the split lognormal; two give one σ through them; one gives
    /// [`DEFAULT_SIGMA`] (a placeholder, marked in the census).
    pub fn from_grade(g: &GradeDist) -> Lognormal {
        let p = g.presence.clamp(1.0e-6, 1.0);
        if p >= 1.0
            && let Some(p90) = g.p90.filter(|v| *v > 0.0)
            && g.p50 > 0.0
        {
            return Lognormal::from_percentiles(&Percentiles {
                p90,
                p50: g.p50,
                p10: g.p10,
            });
        }
        let pts: Vec<(f64, f64)> = [(0.1, g.p90.unwrap_or(0.0)), (0.5, g.p50), (0.9, g.p10)]
            .into_iter()
            .filter(|(_, v)| *v > 0.0)
            .filter_map(|(level, v)| {
                let q = (level - (1.0 - p)) / p;
                (q > 0.0 && q < 1.0).then(|| (normal_quantile(q), v.ln()))
            })
            .collect();
        let (mu, s) = match pts.as_slice() {
            [] => (g.p10.max(1e-12).ln(), DEFAULT_SIGMA),
            [(z, y)] => (y - DEFAULT_SIGMA * z, DEFAULT_SIGMA),
            [first, .., last] => {
                let s = (last.1 - first.1) / (last.0 - first.0);
                (first.1 - s * first.0, s)
            }
        };
        Lognormal {
            mu,
            sigma_lo: s,
            sigma_hi: s,
        }
    }

    /// True when [`Lognormal::from_grade`] had fewer than two positive percentiles to fit.
    pub fn grade_needs_default_sigma(g: &GradeDist) -> bool {
        let p = g.presence.clamp(1.0e-6, 1.0);
        [(0.1, g.p90.unwrap_or(0.0)), (0.5, g.p50), (0.9, g.p10)]
            .iter()
            .filter(|(level, v)| {
                let q = (level - (1.0 - p)) / p;
                *v > 0.0 && q > 0.0 && q < 1.0
            })
            .count()
            < 2
    }

    /// The plan's σ = (ln P10 − ln P90)/2.563, the mean of the two halves.
    pub fn sigma(&self) -> f64 {
        0.5 * (self.sigma_lo + self.sigma_hi)
    }

    pub fn at(&self, z: f64) -> f64 {
        let s = if z < 0.0 {
            self.sigma_lo
        } else {
            self.sigma_hi
        };
        (self.mu + s * z).exp()
    }

    /// The mean: e^μ · (e^(σ_lo²/2)·Φ(−σ_lo) + e^(σ_hi²/2)·Φ(σ_hi)).
    pub fn mean(&self) -> f64 {
        let half = |s: f64| (0.5 * s * s).exp();
        self.mu.exp()
            * (half(self.sigma_lo) * normal_cdf(-self.sigma_lo)
                + half(self.sigma_hi) * normal_cdf(self.sigma_hi))
    }

    /// The value a fraction `q` of deposits is below.
    pub fn quantile(&self, q: f64) -> f64 {
        self.at(normal_quantile(q))
    }

    /// The fraction of deposits below `x`.
    pub fn cdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        let d = x.ln() - self.mu;
        let s = if d < 0.0 {
            self.sigma_lo
        } else {
            self.sigma_hi
        };
        if s <= 0.0 {
            return if d < 0.0 { 0.0 } else { 1.0 };
        }
        normal_cdf(d / s)
    }
}

/// Tonnage and grades drawn together.
#[derive(Clone, Debug, PartialEq)]
pub struct SizeDraw {
    pub tonnage_mt: f64,
    /// The tonnage's quantile within its lognormal (COE's `randomMul`, plan §7.14).
    pub tonnage_q: f64,
    /// Grade key → value in the key's unit (`_pct`, `_gpt`); absent metals are left out.
    pub grades: BTreeMap<String, f64>,
}

/// The joint model of one type.
pub struct SizeModel {
    pub tonnage: Lognormal,
    keys: Vec<String>,
    grades: Vec<(Lognormal, f64)>,
    /// Lower-triangular Cholesky factor of the correlation matrix over [tonnage, grades…].
    chol: Vec<Vec<f64>>,
    /// Correlations actually used (after any shrinkage to keep the matrix positive definite).
    pub shrink: f64,
}

fn cholesky(m: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = m.len();
    let mut l = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let s: f64 = (0..j).map(|k| l[i][k] * l[j][k]).sum();
            if i == j {
                let d = m[i][i] - s;
                if d <= 1e-12 {
                    return None;
                }
                l[i][j] = d.sqrt();
            } else {
                l[i][j] = (m[i][j] - s) / l[j][j];
            }
        }
    }
    Some(l)
}

impl SizeModel {
    pub fn new(s: &Sizing) -> SizeModel {
        let keys: Vec<String> = s.grades.keys().cloned().collect();
        let grades = keys
            .iter()
            .map(|k| (Lognormal::from_grade(&s.grades[k]), s.grades[k].presence))
            .collect();
        let index = |k: &str| -> Option<usize> {
            if k == "tonnage_mt" {
                Some(0)
            } else {
                keys.iter().position(|x| x == k).map(|p| p + 1)
            }
        };
        let n = keys.len() + 1;
        let mut shrink = 1.0;
        let chol = loop {
            let mut m = vec![vec![0.0; n]; n];
            for (i, row) in m.iter_mut().enumerate() {
                row[i] = 1.0;
            }
            for (a, b, r) in &s.correlations {
                if let (Some(i), Some(j)) = (index(a), index(b)) {
                    m[i][j] = r * shrink;
                    m[j][i] = r * shrink;
                }
            }
            if let Some(l) = cholesky(&m) {
                break l;
            }
            shrink *= 0.9;
        };
        SizeModel {
            tonnage: Lognormal::from_percentiles(&s.tonnage_mt),
            keys,
            grades,
            chol,
            shrink,
        }
    }

    /// One deposit's tonnage and grades.
    pub fn draw(&self, rng: &mut Rng) -> SizeDraw {
        let n = self.chol.len();
        let e: Vec<f64> = (0..n).map(|_| rng.normal()).collect();
        let z: Vec<f64> = (0..n)
            .map(|i| (0..=i).map(|k| self.chol[i][k] * e[k]).sum())
            .collect();
        let mut grades = BTreeMap::new();
        for (k, key) in self.keys.iter().enumerate() {
            let (dist, presence) = self.grades[k];
            if rng.uniform() < presence {
                grades.insert(key.clone(), dist.at(z[k + 1]));
            }
        }
        SizeDraw {
            tonnage_mt: self.tonnage.at(z[0]),
            tonnage_q: normal_cdf(z[0]),
            grades,
        }
    }

    /// The smallest plausible deposit (plan §7.8's playability minimum): P90 tonnage, median
    /// grades, every metal present.
    pub fn smallest_plausible(&self) -> SizeDraw {
        SizeDraw {
            tonnage_mt: self.tonnage.quantile(0.1),
            tonnage_q: 0.1,
            grades: self
                .keys
                .iter()
                .enumerate()
                .map(|(k, key)| (key.clone(), self.grades[k].0.at(0.0)))
                .collect(),
        }
    }
}

/// A grade key's value as a mass fraction: `_pct` / 100, `_gpt` · 10⁻⁶, anything else as is.
pub fn mass_fraction(key: &str, value: f64) -> f64 {
    if key.ends_with("_pct") {
        value / 100.0
    } else if key.ends_with("_gpt") {
        value * 1.0e-6
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::types::Clustering;

    fn porphyry_like() -> Sizing {
        Sizing {
            grade_tonnage: "porphyry_cu".into(),
            tonnage_mt: Percentiles {
                p90: 33.0,
                p50: 240.0,
                p10: 1500.0,
            },
            grades: [
                (
                    "cu_pct".to_string(),
                    GradeDist {
                        p90: Some(0.24),
                        p50: 0.44,
                        p10: 0.8,
                        presence: 1.0,
                    },
                ),
                (
                    "au_gpt".to_string(),
                    GradeDist {
                        p90: None,
                        p50: 0.2,
                        p10: 0.6,
                        presence: 0.6,
                    },
                ),
            ]
            .into_iter()
            .collect(),
            correlations: vec![("tonnage_mt".into(), "au_gpt".into(), -0.28)],
            ore_density_t_m3: 2.6,
            density_per_1e5km2: Percentiles {
                p90: 4.0,
                p50: 15.0,
                p10: 35.0,
            },
            density_basis: Default::default(),
            min_separation_m: 2000.0,
            tract_threshold: 0.2,
            clustering: Clustering {
                model: "thomas".into(),
                camp_spacing_km: 40.0,
                cluster_radius_km: 15.0,
            },
        }
    }

    /// Acceptance: porphyry σ = 1.49 ± 0.01 from 33 / 240 / 1,500 Mt.
    /// The split lognormal's mean: porphyry density 4 / 15 / 35 per 100,000 km² (Singer et al.
    /// 2005) has mean ≈ 17.8; a symmetric one e^(μ+σ²/2); and it matches a numerical integral.
    #[test]
    fn split_lognormal_mean() {
        let l = Lognormal::from_percentiles(&Percentiles {
            p90: 4.0,
            p50: 15.0,
            p10: 35.0,
        });
        let n = 200_000;
        let num: f64 = (0..n)
            .map(|k| l.quantile((f64::from(k) + 0.5) / f64::from(n)))
            .sum::<f64>()
            / f64::from(n);
        assert!((l.mean() - num).abs() / num < 2e-3, "{} vs {num}", l.mean());
        assert!((l.mean() - 17.8).abs() < 0.1, "{}", l.mean());
        let sym = Lognormal {
            mu: 1.0,
            sigma_lo: 0.5,
            sigma_hi: 0.5,
        };
        assert!((sym.mean() - (1.0f64 + 0.125).exp()).abs() < 1e-9);
    }

    #[test]
    fn porphyry_sigma() {
        let l = Lognormal::from_percentiles(&porphyry_like().tonnage_mt);
        assert!((l.sigma() - 1.49).abs() < 0.01, "{}", l.sigma());
        assert!((l.quantile(0.1) - 33.0).abs() < 1e-4);
        assert!((l.quantile(0.9) - 1500.0).abs() < 1e-3);
        assert!((l.quantile(0.5) - 240.0).abs() < 1e-6);
        assert!((l.cdf(1500.0) - 0.9).abs() < 1e-6);
    }

    /// Acceptance: drawn tonnages reproduce P90/P50/P10 within sampling error, and the copula
    /// carries the Au–tonnage correlation.
    #[test]
    fn draws_reproduce_the_percentiles_and_correlation() {
        let s = porphyry_like();
        let model = SizeModel::new(&s);
        let mut rng = Rng::new(11);
        let n = 20_000;
        let draws: Vec<SizeDraw> = (0..n).map(|_| model.draw(&mut rng)).collect();
        let mut t: Vec<f64> = draws.iter().map(|d| d.tonnage_mt).collect();
        t.sort_by(f64::total_cmp);
        // The p-th sample quantile of a lognormal has SE ≈ σ·√(p(1−p)/n)/φ(z_p) in log space.
        let sigma = model.tonnage.sigma_lo.max(model.tonnage.sigma_hi);
        for (q, want) in [(0.1, 33.0), (0.5, 240.0), (0.9, 1500.0)] {
            let got: f64 = t[(q * n as f64) as usize];
            let z = normal_quantile(q);
            let phi = (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt();
            let se = sigma * (q * (1.0 - q) / n as f64).sqrt() / phi;
            assert!(
                (got.ln() - f64::ln(want)).abs() < 4.0 * se,
                "q={q}: {got} vs {want} (se {se})"
            );
        }
        let au: Vec<(f64, f64)> = draws
            .iter()
            .filter_map(|d| d.grades.get("au_gpt").map(|g| (d.tonnage_mt.ln(), g.ln())))
            .collect();
        let frac = au.len() as f64 / n as f64;
        assert!((frac - 0.6).abs() < 0.02, "presence {frac}");
        let (mx, my) = au.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
        let (mx, my) = (mx / au.len() as f64, my / au.len() as f64);
        let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
        for (x, y) in &au {
            sxy += (x - mx) * (y - my);
            sxx += (x - mx) * (x - mx);
            syy += (y - my) * (y - my);
        }
        let r = sxy / (sxx * syy).sqrt();
        assert!((r + 0.28).abs() < 0.03, "r = {r}");
        assert!(draws.iter().all(|d| (0.0..=1.0).contains(&d.tonnage_q)));
        assert_eq!(model.shrink, 1.0);
        let small = model.smallest_plausible();
        assert!((small.tonnage_mt - 33.0).abs() < 0.01);
    }

    #[test]
    fn inconsistent_correlations_are_shrunk() {
        let mut s = porphyry_like();
        s.correlations = vec![
            ("tonnage_mt".into(), "cu_pct".into(), 0.95),
            ("tonnage_mt".into(), "au_gpt".into(), 0.95),
            ("cu_pct".into(), "au_gpt".into(), -0.95),
        ];
        let m = SizeModel::new(&s);
        assert!(m.shrink < 1.0);
        let _ = m.draw(&mut Rng::new(1));
    }

    #[test]
    fn fractions() {
        assert_eq!(mass_fraction("cu_pct", 0.5), 0.005);
        assert!((mass_fraction("au_gpt", 2.0) - 2.0e-6).abs() < 1e-18);
    }
}
