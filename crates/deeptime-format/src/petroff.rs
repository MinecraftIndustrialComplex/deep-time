// SPDX-License-Identifier: LGPL-3.0-or-later
//! Petroff's square equal-area map projection (Petroff 2021, "A square equal-area map projection
//! with low angular distortion, minimal cusps, and closed-form solutions", ACM TSAS 7(4):18,
//! arXiv:2008.13670): forward and inverse, closed form.
//!
//! **Port.** A scalar port of `new_projection` and `new_projection_inverse` from Petroff's
//! `new_projection.py` (Zenodo 10.5281/zenodo.4661351, released under CC0 1.0 by its author; see
//! NOTICE.md), on [`crate::portable_math`] so the Java (`worlddata/Petroff.java`) and JS
//! (`projection.js`) twins give the same bits. Changes from the original:
//! - the inverse's quadrant rule gives each boundary ray to one side (the original put the ray
//!   y = 0, x < 0 in a quadrant 4 and x = 0, y < 0 in quadrant 1; found by the world-shape page's
//!   tear check, 2026-10-04, and fixed in `tools/research/projection-survey/survey.py`);
//! - 0/0 cases the original masks with array arithmetic are explicit branches, and arguments of
//!   `acos`, `asin` and `sqrt` that rounding pushes past their domain are clamped.
//!
//! **Frame.** The square is [−1, 1]² with the north pole at its centre and the equator the diamond
//! |x| + |y| = 1 (the southern hemisphere in the four corners); longitude λ in [0, 2π) runs counter-
//! clockwise from the +x, −y diagonal's quadrant (λ ∈ [0, π/2) is the quadrant x > 0, y < 0). The
//! map is equal-area: every part of the square covers the same share of the sphere (ledger:
//! SCL-083 for the constants, SCL-086 for the tested equal-area property).

use crate::portable_math as pm;
use pm::PI;

/// Petroff's constants, computed once (with the same code in every twin).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Petroff {
    sin_phi0: f64,
    cos_phi0: f64,
    hp: f64,
    xip: f64,
    psi0: f64,
    psi1: f64,
    p0p: f64,
    p1p: f64,
    p2p: f64,
    c_a: f64,
    c_b: f64,
    f0: f64,
    f1: f64,
    a1p: f64,
    c01p: f64,
    c2p: f64,
    b1: f64,
    b2: f64,
    tan_b: [f64; 3],
}

impl Default for Petroff {
    fn default() -> Self {
        Petroff::new()
    }
}

/// The sign of `x`: −1, 0 or 1 (numpy's `sign`).
#[inline]
fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// √x for x ≥ 0 (rounding below zero clamps to 0).
#[inline]
fn sqrt0(x: f64) -> f64 {
    if x > 0.0 { x.sqrt() } else { 0.0 }
}

impl Petroff {
    /// The projection's constants.
    pub fn new() -> Petroff {
        let phi0 = 3.0 * PI / 8.0;
        let (sin_phi0, cos_phi0) = (pm::sin(phi0), pm::cos(phi0));
        let a1 = pm::asin(1.0 / (2.0 - cos_phi0 * cos_phi0).sqrt());
        let a2 = pm::asin(2.0 * sin_phi0 / (3.0 - pm::cos(2.0 * phi0)).sqrt());
        let hp = 12.0 / PI * (a1 + a2 - PI / 2.0);
        let s3 = 3.0f64.sqrt();
        let xip = pm::atan(
            (PI * (hp - 3.0) * (hp - 3.0))
                / (s3 * (PI * (hp * hp - 2.0 * hp + 45.0) - 96.0 * a1 - 48.0 * a2)),
        );
        let psi0 = a1;
        let psi1 = PI - 2.0 * psi0;
        let p0p = pm::atan(s3 / hp);
        let p1p = 7.0 * PI / 6.0 - p0p - xip;
        let p2p = xip - PI / 6.0;
        let c_a = pm::acos(cos_phi0 / 2.0f64.sqrt());
        let c_b = PI / 2.0 - phi0;
        let a1p = (hp * hp + 3.0).sqrt() * pm::sin(PI / 3.0 - pm::atan(hp / s3)) / pm::sin(xip);
        let b1 = pm::atan(2.0f64.sqrt() * pm::tan(phi0));
        let b2 = PI / 2.0 - b1;
        Petroff {
            sin_phi0,
            cos_phi0,
            hp,
            xip,
            psi0,
            psi1,
            p0p,
            p1p,
            p2p,
            c_a,
            c_b,
            f0: a2,
            f1: PI / 2.0 - a2,
            a1p,
            c01p: (hp * hp + 3.0).sqrt(),
            c2p: 3.0 - hp,
            b1,
            b2,
            tan_b: [pm::tan(PI / 4.0), pm::tan(b1), pm::tan(b2)],
        }
    }

    /// Longitude λ (radians, any value; folded into [0, 2π)) and latitude φ (radians) to the
    /// square: (x, y) in [−1, 1]².
    pub fn forward(&self, lambda: f64, phi: f64) -> (f64, f64) {
        let two_pi = 2.0 * PI;
        let mut lam = lambda - two_pi * (lambda / two_pi).floor();
        if lam >= two_pi || lam < 0.0 {
            lam = 0.0;
        }
        let phi_c = phi.abs();
        let lambda_c = lam - PI / 4.0;
        let mut quadrant = (2.0 * lam / PI).floor();
        if quadrant > 3.0 {
            quadrant = 3.0;
        }
        let lambda0 = quadrant * PI / 2.0;
        let dl = lambda_c - lambda0;
        let (sin_pc, cos_pc) = (pm::sin(phi_c), pm::cos(phi_c));
        let (sin_dl, cos_dl) = (pm::sin(dl), pm::cos(dl));
        let theta = pm::atan2(
            cos_pc * sin_dl,
            self.sin_phi0 * cos_pc * cos_dl - self.cos_phi0 * sin_pc,
        )
        .abs();
        let r = pm::acos(self.sin_phi0 * sin_pc + self.cos_phi0 * cos_pc * cos_dl);
        let s = if theta <= self.psi0 {
            0
        } else if theta <= self.psi0 + self.psi1 {
            1
        } else {
            2
        };
        let beta = match s {
            0 => self.psi0 - theta,
            1 => theta - self.psi0,
            _ => PI - theta,
        };
        let c = if s <= 1 { self.c_a } else { self.c_b };
        let g = if s == 1 { self.psi1 } else { self.psi0 };
        let gp = [self.p0p, self.p1p, self.p2p][s];
        let f = [self.f0, self.f1, PI / 4.0][s];
        let ap = if s == 0 { self.hp } else { self.a1p };
        let cp = if s <= 1 { self.c01p } else { self.c2p };

        let (sin_r, cos_r) = (pm::sin(r), pm::cos(r));
        let (sin_c, cos_c) = (pm::sin(c), pm::cos(c));
        let x = pm::acos(cos_r * cos_c + sin_r * sin_c * pm::cos(beta));
        let gamma = if x > 0.0 {
            pm::asin(pm::sin(beta) * sin_r / pm::sin(x))
        } else {
            0.0
        };
        let (sin_g, cos_g) = (pm::sin(g), pm::cos(g));
        let epsilon = pm::acos(sin_g * pm::sin(gamma) * cos_c - cos_g * pm::cos(gamma));
        let upupvp = (gamma + g + epsilon - PI) / (f + g - PI / 2.0);
        let sin_eps = pm::sin(epsilon);
        let cos_xy = if sin_eps != 0.0 {
            let q = sin_g * sin_c / sin_eps;
            sqrt0(1.0 - q * q)
        } else {
            0.0
        };
        let den = 1.0 - cos_xy;
        let xpxpyp = if den > 0.0 {
            sqrt0((1.0 - pm::cos(x)) / den)
        } else {
            0.0
        };
        let up = ap * upupvp;
        let xpyp = sqrt0(up * up + cp * cp - 2.0 * up * cp * pm::cos(gp));
        let cos_gammap = if xpyp > 0.0 {
            let q = up * pm::sin(gp) / xpyp;
            sqrt0(1.0 - q * q)
        } else {
            1.0
        };
        let xp = xpyp * xpxpyp;
        let yp = xpyp - xp;
        let rp = sqrt0(xp * xp + cp * cp - 2.0 * xp * cp * cos_gammap);
        let alphap = if up * rp <= 0.0 {
            self.xip - PI / 6.0
        } else {
            let q = (yp * yp - up * up - rp * rp) / (-2.0 * up * rp);
            if q < 1.0 { pm::acos(q) } else { 0.0 }
        };
        let thetap = match s {
            0 => alphap,
            1 => PI - (self.xip - PI / 6.0) - alphap,
            _ => PI - (self.xip - PI / 6.0) + alphap,
        };
        let side = if dl < 0.0 { -1.0 } else { 1.0 };
        let x_c = side * rp * pm::sin(thetap);
        let y_c = self.hp - rp * pm::cos(thetap);
        let y_h = y_c * sign(phi) - 3.0;
        let zeta = PI / 4.0 + quadrant * PI / 2.0;
        let (sin_z, cos_z) = (pm::sin(zeta), pm::cos(zeta));
        let s3 = 3.0f64.sqrt();
        let k = s3 / (3.0 * 2.0f64.sqrt());
        (
            (x_c * cos_z - y_h * sin_z / s3) * k,
            (x_c * sin_z + y_h * cos_z / s3) * k,
        )
    }

    /// The square's (x, y) to (λ, φ), radians; λ in [π/4 − π, 2π + π/4) before folding, 0 at the
    /// poles. Fold λ with [`crate::projection::wrap_deg`] (in degrees) where a range matters.
    pub fn inverse(&self, x_m: f64, y_m: f64) -> (f64, f64) {
        let quadrant = if x_m <= 0.0 && y_m >= 0.0 {
            2.0
        } else if y_m >= 0.0 {
            1.0
        } else if x_m >= 0.0 {
            0.0
        } else {
            3.0
        };
        let zeta = PI / 4.0 + quadrant * PI / 2.0;
        let (sin_z, cos_z) = (pm::sin(zeta), pm::cos(zeta));
        let x_c = 6.0f64.sqrt() * (x_m * cos_z + y_m * sin_z);
        let y_c = 3.0 * 2.0f64.sqrt() * (y_m * cos_z - x_m * sin_z);
        let (sgnphi, y_h) = if y_c < -3.0 {
            (-1.0, -6.0 - y_c)
        } else {
            (1.0, y_c)
        };
        let hp = self.hp;
        let dy = hp - 3.0 - y_h;
        let rp = (x_c * x_c + dy * dy).sqrt();
        let thetap = pm::atan2(x_c, dy).abs();
        let s = if thetap <= self.p0p {
            0
        } else if thetap <= self.p0p + self.p1p {
            1
        } else {
            2
        };
        let alphap = match s {
            0 => thetap,
            1 => PI - self.p2p - thetap,
            _ => thetap + self.p2p - PI,
        };
        let c = if s <= 1 { self.c_a } else { self.c_b };
        let g = if s == 1 { self.psi1 } else { self.psi0 };
        let gp = [self.p0p, self.p1p, self.p2p][s];
        let f = [self.f0, self.f1, PI / 4.0][s];
        let ap = if s == 0 { hp } else { self.a1p };
        let cp = if s <= 1 { self.c01p } else { self.c2p };
        let b = [PI / 4.0, self.b1, self.b2][s];
        let tan_b = self.tan_b[s];

        let betap = gp - alphap;
        let xp = sqrt0(rp * rp + cp * cp - 2.0 * rp * cp * pm::cos(betap));
        let gammap = if xp > 0.0 {
            pm::asin(rp * pm::sin(betap) / xp)
        } else {
            0.0
        };
        let epsp = PI - gp - gammap;
        let yp = if epsp > 0.0 {
            rp * pm::sin(alphap) / pm::sin(epsp)
        } else {
            0.0
        };
        let xy = xp + yp;
        let up = (cp * cp + xy * xy - 2.0 * cp * xy * pm::cos(gammap))
            .abs()
            .sqrt();
        let vp = ap - up;
        let w = vp * (f + g - PI / 2.0) / ap;
        let num = -pm::sin(w);
        let den = pm::cos(b) - pm::cos(w);
        let delta = if den == 0.0 && num == 0.0 {
            0.0
        } else {
            pm::atan(num / den)
        };
        let gamma = f - delta;
        let tq = tan_b / pm::cos(delta);
        let cos_xy = 1.0 / (1.0 + tq * tq).sqrt();
        let x = if xy > 0.0 {
            let q = xp / xy;
            pm::acos(1.0 - q * q * (1.0 - cos_xy))
        } else {
            0.0
        };
        let (sin_x, cos_x) = (pm::sin(x), pm::cos(x));
        let (sin_c, cos_c) = (pm::sin(c), pm::cos(c));
        let r = pm::acos(cos_x * cos_c + sin_x * sin_c * pm::cos(gamma));
        let sin_r = pm::sin(r);
        let beta = if sin_r != 0.0 {
            pm::asin(sin_x * pm::sin(gamma) / sin_r)
        } else {
            0.0
        };
        let alpha = match s {
            0 => self.psi0 - beta,
            1 => self.psi0 + beta,
            _ => PI - beta,
        };
        let cos_r = pm::cos(r);
        let phi_h = pm::asin(self.sin_phi0 * cos_r - self.cos_phi0 * sin_r * pm::cos(alpha));
        let lambda0 = PI / 4.0 + quadrant * PI / 2.0;
        let phi = sgnphi * phi_h;
        let num = pm::sin(alpha) * sin_r * self.cos_phi0;
        let den = cos_r - self.sin_phi0 * pm::sin(phi_h);
        let dl = if den == 0.0 && num == 0.0 {
            0.0
        } else {
            pm::atan(num / den)
        };
        let mut lambda = lambda0 + sign(x_c) * dl;
        if phi.abs() == PI / 2.0 {
            lambda = 0.0;
        }
        (lambda, phi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn f(&mut self) -> f64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn unit(lambda: f64, phi: f64) -> [f64; 3] {
        let (sp, cp) = (phi.sin(), phi.cos());
        [cp * lambda.cos(), cp * lambda.sin(), sp]
    }

    fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
        let c = [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ];
        let s = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        s.atan2(a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
    }

    #[test]
    fn the_square_and_its_equator() {
        let p = Petroff::new();
        // North pole at the centre, the equator on the diamond, the south pole at the corners.
        let (x, y) = p.forward(0.3, PI / 2.0);
        assert!(x.abs() < 1e-12 && y.abs() < 1e-12, "{x} {y}");
        for k in 0..64 {
            let lam = 2.0 * PI * (f64::from(k) + 0.5) / 64.0;
            let (x, y) = p.forward(lam, 0.0);
            assert!((x.abs() + y.abs() - 1.0).abs() < 1e-12, "λ {lam}: {x} {y}");
            let (x, y) = p.forward(lam, -PI / 2.0 + 1e-12);
            assert!(
                (x.abs() - 1.0).abs() < 1e-5 && (y.abs() - 1.0).abs() < 1e-5,
                "{x} {y}"
            );
        }
        let (lam, phi) = p.inverse(0.0, 0.0);
        assert!((phi - PI / 2.0).abs() < 1e-12 && lam == 0.0, "{lam} {phi}");
    }

    #[test]
    fn round_trips_both_ways() {
        let p = Petroff::new();
        let mut g = Rng(0x9E7);
        let mut worst: f64 = 0.0;
        for _ in 0..200_000 {
            let lam = 2.0 * PI * g.f();
            let phi = (2.0 * g.f() - 1.0).asin();
            let (x, y) = p.forward(lam, phi);
            assert!(x.abs() <= 1.0 + 1e-12 && y.abs() <= 1.0 + 1e-12);
            let (l2, p2) = p.inverse(x, y);
            let d = angle(unit(lam, phi), unit(l2, p2));
            worst = worst.max(d);
            assert!(
                d < 1e-9,
                "({lam}, {phi}) -> ({x}, {y}) -> ({l2}, {p2}): {d}"
            );
            let (x, y) = (2.0 * g.f() - 1.0, 2.0 * g.f() - 1.0);
            let (l, f) = p.inverse(x, y);
            let (x2, y2) = p.forward(l, f);
            assert!(
                (x2 - x).abs() < 1e-9 && (y2 - y).abs() < 1e-9,
                "({x}, {y}) -> ({l}, {f}) -> ({x2}, {y2})"
            );
        }
        assert!(worst < 1e-9, "{worst}");
    }

    #[test]
    fn the_boundary_rays_are_continuous() {
        // The quadrant boundaries (the axes) and the diagonals: points just either side map to
        // nearby sphere points (the original's quadrant rule put the rays y = 0, x < 0 and
        // x = 0, y < 0 in the wrong quadrant).
        let p = Petroff::new();
        for k in 1..200 {
            let t = f64::from(k) / 200.0;
            for (x, y, dx, dy) in [
                (-t, 0.0, 0.0, 1e-9),
                (t, 0.0, 0.0, 1e-9),
                (0.0, -t, 1e-9, 0.0),
                (0.0, t, 1e-9, 0.0),
                (t * 0.5, t * 0.5, 1e-9, -1e-9),
            ] {
                let on = p.inverse(x, y);
                let a = p.inverse(x + dx, y + dy);
                let b = p.inverse(x - dx, y - dy);
                let (u0, ua, ub) = (unit(on.0, on.1), unit(a.0, a.1), unit(b.0, b.1));
                assert!(
                    angle(u0, ua) < 1e-7 && angle(u0, ub) < 1e-7,
                    "({x}, {y}): {on:?} {a:?} {b:?}"
                );
            }
        }
    }
}
