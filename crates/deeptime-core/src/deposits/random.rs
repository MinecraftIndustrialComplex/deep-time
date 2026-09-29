// SPDX-License-Identifier: LGPL-3.0-or-later
//! The deposit engine's random numbers: xoshiro256++ (Blackman & Vigna 2021, "Scrambled linear
//! pseudorandom number generators", ACM TOMS 47(4); public-domain reference code, reimplemented)
//! seeded through SplitMix64 as its authors recommend, plus the few distributions the engine
//! draws from: standard normal (Marsaglia polar method), Poisson (multiplication method below
//! λ = 10, Hörmann's PTRS transformed rejection above; Hörmann 1993, "The transformed rejection
//! method for generating Poisson random variables", Insurance: Mathematics and Economics 12:39),
//! and the inverse normal CDF (Acklam's rational approximation).
//!
//! Every draw is deterministic given the seed; independent streams come from [`Rng::stream`]
//! (a hash of the parent seed and a label), so adding a deposit type does not shift the draws of
//! another one.

use crate::sphere::SplitMix64;

/// xoshiro256++.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
    spare_normal: Option<f64>,
}

/// A 64-bit FNV-1a hash (stream labels, type ids).
pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut sm = SplitMix64::new(seed);
        let s = [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()];
        Rng {
            s,
            spare_normal: None,
        }
    }

    /// An independent generator for `label` under `seed`.
    pub fn stream(seed: u64, label: &str) -> Rng {
        Rng::new(seed ^ hash_str(label).rotate_left(17))
    }

    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// Uniform in [0, 1), 53 bits.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in (0, 1).
    pub fn uniform_open(&mut self) -> f64 {
        loop {
            let u = self.uniform();
            if u > 0.0 {
                return u;
            }
        }
    }

    /// Uniform in [0, n) for n > 0.
    pub fn below(&mut self, n: u64) -> u64 {
        ((u128::from(self.next_u64()) * u128::from(n)) >> 64) as u64
    }

    /// A standard normal draw (Marsaglia polar method).
    pub fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare_normal.take() {
            return z;
        }
        loop {
            let u = 2.0 * self.uniform() - 1.0;
            let v = 2.0 * self.uniform() - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let f = (-2.0 * s.ln() / s).sqrt();
                self.spare_normal = Some(v * f);
                return u * f;
            }
        }
    }

    /// A Poisson draw with mean `lambda` (0 for λ ≤ 0).
    pub fn poisson(&mut self, lambda: f64) -> u64 {
        if lambda.is_nan() || lambda <= 0.0 {
            return 0;
        }
        if lambda < 10.0 {
            let l = (-lambda).exp();
            let mut k = 0u64;
            let mut p = self.uniform_open();
            while p > l {
                k += 1;
                p *= self.uniform_open();
            }
            return k;
        }
        // PTRS (Hörmann 1993), the constants of the paper.
        let slam = lambda.sqrt();
        let loglam = lambda.ln();
        let b = 0.931 + 2.53 * slam;
        let a = -0.059 + 0.02483 * b;
        let invalpha = 1.1239 + 1.1328 / (b - 3.4);
        let vr = 0.9277 - 3.6224 / (b - 2.0);
        loop {
            let u = self.uniform() - 0.5;
            let v = self.uniform_open();
            let us = 0.5 - u.abs();
            let k = ((2.0 * a / us + b) * u + lambda + 0.43).floor();
            if us >= 0.07 && v <= vr {
                return k as u64;
            }
            if k < 0.0 || (us < 0.013 && v > us) {
                continue;
            }
            if v.ln() + invalpha.ln() - (a / (us * us) + b).ln()
                <= -lambda + k * loglam - ln_factorial(k as u64)
            {
                return k as u64;
            }
        }
    }

    /// Picks an index with probability proportional to `weights` (non-negative); `None` when
    /// they sum to zero.
    pub fn weighted(&mut self, weights: &[f64]) -> Option<usize> {
        let total: f64 = weights.iter().filter(|w| **w > 0.0).sum();
        if total.is_nan() || total <= 0.0 {
            return None;
        }
        let mut r = self.uniform() * total;
        for (i, w) in weights.iter().enumerate() {
            if *w > 0.0 {
                if r < *w {
                    return Some(i);
                }
                r -= w;
            }
        }
        weights.iter().rposition(|w| *w > 0.0)
    }
}

/// ln(k!) — exact sums below 30, Stirling's series above.
pub fn ln_factorial(k: u64) -> f64 {
    if k < 30 {
        return (2..=k).map(|i| (i as f64).ln()).sum();
    }
    let x = k as f64 + 1.0;
    // ln Γ(x) by Stirling's series to 1/x⁵.
    (x - 0.5) * x.ln() - x + 0.5 * (2.0 * std::f64::consts::PI).ln() + 1.0 / (12.0 * x)
        - 1.0 / (360.0 * x.powi(3))
        + 1.0 / (1260.0 * x.powi(5))
}

/// The standard normal CDF Φ(x) (Abramowitz & Stegun 7.1.26 through erf; |error| < 1.5e-7).
pub fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let s = x.signum();
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-x * x).exp();
    s * y
}

/// Φ⁻¹(p) for p in (0, 1): Acklam's rational approximation (relative error < 1.2e-9).
pub fn normal_quantile(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    let p = p.clamp(1e-300, 1.0 - 1e-16);
    let plow = 0.024_25;
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= 1.0 - plow {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_streams_differ() {
        let mut a = Rng::new(9);
        let mut b = Rng::new(9);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut s1 = Rng::stream(9, "porphyry");
        let mut s2 = Rng::stream(9, "mvt");
        assert_ne!(s1.next_u64(), s2.next_u64());
    }

    #[test]
    fn normal_moments() {
        let mut r = Rng::new(1);
        let n = 200_000;
        let (mut m, mut v) = (0.0, 0.0);
        for _ in 0..n {
            let z = r.normal();
            m += z;
            v += z * z;
        }
        m /= n as f64;
        v = v / n as f64 - m * m;
        assert!(m.abs() < 0.01, "mean {m}");
        assert!((v - 1.0).abs() < 0.015, "variance {v}");
    }

    /// Mean and variance of Poisson draws on both algorithms' sides of λ = 10.
    #[test]
    fn poisson_moments() {
        for &lambda in &[0.3, 3.7, 9.9, 10.0, 25.0, 180.0] {
            let mut r = Rng::stream(5, &format!("{lambda}"));
            let n = 100_000;
            let (mut m, mut v) = (0.0f64, 0.0f64);
            for _ in 0..n {
                let k = r.poisson(lambda) as f64;
                m += k;
                v += k * k;
            }
            m /= n as f64;
            v = v / n as f64 - m * m;
            let se = (lambda / n as f64).sqrt();
            assert!((m - lambda).abs() < 5.0 * se, "λ={lambda}: mean {m}");
            assert!((v / lambda - 1.0).abs() < 0.03, "λ={lambda}: variance {v}");
        }
        assert_eq!(Rng::new(1).poisson(0.0), 0);
    }

    #[test]
    fn quantile_inverts_cdf() {
        for &p in &[1e-6, 0.01, 0.1, 0.25, 0.5, 0.9, 0.975, 0.999_99] {
            let x = normal_quantile(p);
            assert!((normal_cdf(x) - p).abs() < 2e-7, "p={p} x={x}");
        }
        assert!((normal_quantile(0.9) - 1.281_551_565_5).abs() < 1e-8);
    }

    #[test]
    fn ln_factorial_matches_sums() {
        let exact: f64 = (2..=40u64).map(|i| (i as f64).ln()).sum();
        assert!((ln_factorial(40) - exact).abs() < 1e-9);
        assert_eq!(ln_factorial(0), 0.0);
    }

    #[test]
    fn weighted_picks() {
        let mut r = Rng::new(3);
        assert_eq!(r.weighted(&[0.0, 0.0]), None);
        let mut counts = [0u32; 3];
        for _ in 0..30_000 {
            counts[r.weighted(&[1.0, 0.0, 3.0]).unwrap()] += 1;
        }
        assert_eq!(counts[1], 0);
        let ratio = f64::from(counts[2]) / f64::from(counts[0]);
        assert!((ratio - 3.0).abs() < 0.2, "{ratio}");
    }
}
