// SPDX-License-Identifier: LGPL-3.0-or-later
//! Jacobi elliptic functions and Carlson's R_F, as far as the Peirce square needs them.
//!
//! Written from the textbook definitions, not ported from any code:
//! - real sn, cn, dn by the arithmetic–geometric mean (Abramowitz & Stegun 16.4);
//! - complex sn, cn, dn from the real ones by the addition formulas (A&S 16.21.2–4);
//! - the complete integral K(m) = π / (2·AGM(1, √(1 − m))) (A&S 17.6);
//! - Carlson's symmetric R_F by duplication (Carlson 1995; DLMF 19.36.1), complex arguments;
//! - the inverse of cn as arccn(ζ) = √(1 − ζ²) · R_F(ζ², 1 − m + m ζ², 1) (DLMF 22.15.13 with
//!   19.25.5), continued analytically off the real segment.

use crate::complex::C;

/// The parameter m = k² of the Peirce quincuncial projection: k = 1/√2, so m = m₁ = ½ and the
/// period lattice is square.
pub const M_HALF: f64 = 0.5;

/// K(m), the complete elliptic integral of the first kind, for 0 ≤ m < 1.
pub fn complete_k(m: f64) -> f64 {
    let (mut a, mut b) = (1.0f64, (1.0 - m).sqrt());
    for _ in 0..40 {
        if (a - b).abs() <= 1e-16 * a {
            break;
        }
        let an = 0.5 * (a + b);
        b = (a * b).sqrt();
        a = an;
    }
    std::f64::consts::PI / (2.0 * a)
}

/// Real (sn, cn, dn)(u | m) for 0 ≤ m < 1 by the AGM / descending Landen scheme (A&S 16.4).
pub fn sncndn(u: f64, m: f64) -> (f64, f64, f64) {
    if m <= 0.0 {
        return (u.sin(), u.cos(), 1.0);
    }
    // Reduce by the real period 4K: the AGM's amplitude doubles each level, so a small argument
    // keeps the last bits.
    let k = complete_k(m);
    let u = u - 4.0 * k * (u / (4.0 * k)).round();
    let mut a = [0.0f64; 24];
    let mut c = [0.0f64; 24];
    a[0] = 1.0;
    c[0] = m.sqrt();
    let mut b = (1.0 - m).sqrt();
    let mut n = 0;
    while c[n].abs() > 1e-17 * a[n] && n < 22 {
        let an = a[n];
        a[n + 1] = 0.5 * (an + b);
        c[n + 1] = 0.5 * (an - b);
        b = (an * b).sqrt();
        n += 1;
    }
    let mut phi = (1u64 << n) as f64 * a[n] * u;
    for j in (1..=n).rev() {
        phi = 0.5 * (phi + (c[j] / a[j] * phi.sin()).asin());
    }
    let (s, cn) = phi.sin_cos();
    let dn = (1.0 - m * s * s).max(0.0).sqrt();
    (s, cn, dn)
}

/// Complex (sn, cn, dn)(w | m) by the addition formulas over the real and imaginary parts
/// (A&S 16.21.2–4). The denominator vanishes at the poles of the functions (w = iK′ and its
/// translates); there the values are infinite or NaN and the caller must treat the point as a pole.
pub fn sncndn_c(w: C, m: f64) -> (C, C, C) {
    let m1 = 1.0 - m;
    let (s, c, d) = sncndn(w.re, m);
    let (s1, c1, d1) = sncndn(w.im, m1);
    let den = c1 * c1 + m * s * s * s1 * s1;
    let sn = C::new(s * d1, c * d * s1 * c1).scale(1.0 / den);
    let cn = C::new(c * c1, -s * d * s1 * d1).scale(1.0 / den);
    let dn = C::new(d * c1 * d1, -m * s * c * s1).scale(1.0 / den);
    (sn, cn, dn)
}

/// The denominator of [`sncndn_c`]: zero exactly at the poles of sn, cn and dn.
pub fn pole_distance_proxy(w: C, m: f64) -> f64 {
    let (s, _, _) = sncndn(w.re, m);
    let (s1, c1, _) = sncndn(w.im, 1.0 - m);
    c1 * c1 + m * s * s * s1 * s1
}

/// Carlson's R_F(x, y, z) for complex arguments off the closed negative real axis, at most one of
/// them zero (DLMF 19.36.1, duplication then the fifth-order series).
pub fn carlson_rf(x: C, y: C, z: C) -> C {
    let (mut x, mut y, mut z) = (x, y, z);
    let third = 1.0 / 3.0;
    for _ in 0..60 {
        let a = (x + y + z).scale(third);
        let dev = ((a - x) / a)
            .abs()
            .max(((a - y) / a).abs())
            .max(((a - z) / a).abs());
        if dev < 1e-4 {
            break;
        }
        let (sx, sy, sz) = (x.sqrt(), y.sqrt(), z.sqrt());
        let lam = sx * sy + sy * sz + sz * sx;
        x = (x + lam).scale(0.25);
        y = (y + lam).scale(0.25);
        z = (z + lam).scale(0.25);
    }
    let a = (x + y + z).scale(third);
    let xx = (a - x) / a;
    let yy = (a - y) / a;
    let zz = -(xx + yy);
    let e2 = xx * yy - zz * zz;
    let e3 = xx * yy * zz;
    let series = C::ONE - e2.scale(0.1) + e3.scale(1.0 / 14.0) + (e2 * e2).scale(1.0 / 24.0)
        - (e2 * e3).scale(3.0 / 44.0);
    series / a.sqrt()
}

/// A preimage of ζ under cn(· | m): w with cn(w) = ζ. For ζ in the sector |arg ζ| ≤ π/4 (the one the
/// Peirce square uses, see `peirce`) the arguments of R_F stay in the right half-plane, so the
/// principal branches are continuous there except across the real ray ζ > 1, where the two results
/// are w and −w (cn is even): the glued halves of the square's west edge.
pub fn arccn(zeta: C, m: f64) -> C {
    let z2 = zeta * zeta;
    let s = (C::ONE - z2).sqrt();
    let y = C::real(1.0 - m) + z2.scale(m);
    s * carlson_rf(z2, y, C::ONE)
}

#[cfg(test)]
mod tests {
    use super::*;

    // K(1/2) = Γ(¼)² / (4√π) (from memory, not read from a table); the AGM must reproduce it.
    const K_HALF: f64 = 1.854_074_677_301_372;

    #[test]
    fn complete_k_of_one_half_matches_the_table() {
        assert!((complete_k(M_HALF) - K_HALF).abs() < 1e-14);
        assert!((complete_k(0.0) - std::f64::consts::FRAC_PI_2).abs() < 1e-15);
    }

    #[test]
    fn real_functions_satisfy_their_identities_and_special_values() {
        let k = complete_k(M_HALF);
        let (s, c, d) = sncndn(k, M_HALF);
        assert!((s - 1.0).abs() < 1e-14 && c.abs() < 1e-14 && (d - M_HALF.sqrt()).abs() < 1e-14);
        for i in 0..200 {
            let u = -9.0 + 0.09 * i as f64;
            let (s, c, d) = sncndn(u, M_HALF);
            assert!((s * s + c * c - 1.0).abs() < 1e-14);
            assert!((d * d + M_HALF * s * s - 1.0).abs() < 1e-14);
            // Derivative check: d sn / du = cn dn.
            let h = 1e-6;
            let ds = (sncndn(u + h, M_HALF).0 - sncndn(u - h, M_HALF).0) / (2.0 * h);
            assert!((ds - c * d).abs() < 1e-8, "u {u}");
        }
    }

    #[test]
    fn complex_cn_is_holomorphic_and_has_the_square_lattice_symmetries() {
        let k = complete_k(M_HALF);
        let kk = C::real(k);
        for i in 0..40 {
            for j in 0..40 {
                let w = C::new(-3.0 + 0.15 * i as f64 + 0.013, -3.0 + 0.15 * j as f64 + 0.007);
                if pole_distance_proxy(w, M_HALF) < 1e-3 {
                    continue;
                }
                let (sn, cn, dn) = sncndn_c(w, M_HALF);
                // cn² + sn² = 1 and dn² + m sn² = 1 for complex arguments too.
                assert!((cn * cn + sn * sn - C::ONE).abs() < 1e-10);
                assert!((dn * dn + sn.scale(M_HALF) * sn - C::ONE).abs() < 1e-10);
                // Cauchy–Riemann through the derivative: d cn / dw = −sn dn along both axes.
                let h = 1e-6;
                let dx = (sncndn_c(w + C::real(h), M_HALF).1 - sncndn_c(w - C::real(h), M_HALF).1)
                    .scale(0.5 / h);
                let dy = (sncndn_c(w + C::new(0.0, h), M_HALF).1
                    - sncndn_c(w - C::new(0.0, h), M_HALF).1)
                    .scale(0.5 / h);
                let want = -(sn * dn);
                assert!((dx - want).abs() < 1e-6 * (1.0 + want.abs()));
                assert!((dy - C::I * want).abs() < 1e-6 * (1.0 + want.abs()));
                // Quarter turn about K: cn(K + i(w − K)) = i cn(w) (the square's 90° symmetry).
                let turned = sncndn_c(kk + C::I * (w - kk), M_HALF).1;
                assert!((turned - C::I * cn).abs() < 1e-9 * (1.0 + cn.abs()), "{w:?}");
            }
        }
    }

    #[test]
    fn carlson_rf_matches_known_values() {
        // R_F(0, 1/2, 1) = K(1/2) and R_F(x, x, x) = x^(-1/2).
        let v = carlson_rf(C::ZERO, C::real(0.5), C::ONE);
        assert!((v.re - K_HALF).abs() < 1e-14 && v.im.abs() < 1e-15);
        let v = carlson_rf(C::real(4.0), C::real(4.0), C::real(4.0));
        assert!((v.re - 0.5).abs() < 1e-15);
        // DLMF 19.20.1: R_F(x, y, y) = R_C(x, y); R_C(0, 1/4) = π.
        let v = carlson_rf(C::ZERO, C::real(0.25), C::real(0.25));
        assert!((v.re - std::f64::consts::PI).abs() < 1e-13);
    }

    #[test]
    fn arccn_inverts_cn_on_the_sector() {
        for i in 0..60 {
            for j in 0..60 {
                let r = 0.02 + 0.08 * i as f64; // up to |ζ| = 4.7, deep in the south
                let a = -std::f64::consts::FRAC_PI_4 + std::f64::consts::FRAC_PI_2 * j as f64 / 59.0;
                let zeta = C::cis(a).scale(r);
                let w = arccn(zeta, M_HALF);
                let back = sncndn_c(w, M_HALF).1;
                assert!((back - zeta).abs() < 1e-11 * (1.0 + r), "{zeta:?} -> {w:?} -> {back:?}");
            }
        }
    }
}
