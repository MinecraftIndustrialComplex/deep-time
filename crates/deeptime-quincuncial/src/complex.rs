// SPDX-License-Identifier: LGPL-3.0-or-later
//! A minimal complex number: just what the elliptic functions need (no dependency).

use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

impl C {
    pub const ZERO: C = C { re: 0.0, im: 0.0 };
    pub const ONE: C = C { re: 1.0, im: 0.0 };
    pub const I: C = C { re: 0.0, im: 1.0 };

    pub const fn new(re: f64, im: f64) -> C {
        C { re, im }
    }

    pub fn real(re: f64) -> C {
        C { re, im: 0.0 }
    }

    /// e^{iθ}.
    pub fn cis(theta: f64) -> C {
        C::new(theta.cos(), theta.sin())
    }

    pub fn conj(self) -> C {
        C::new(self.re, -self.im)
    }

    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn scale(self, k: f64) -> C {
        C::new(self.re * k, self.im * k)
    }

    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }

    /// The principal square root (branch cut along the negative real axis; Re ≥ 0).
    pub fn sqrt(self) -> C {
        let r = self.abs();
        if r == 0.0 {
            return C::ZERO;
        }
        let t = ((r + self.re.abs()) * 0.5).sqrt();
        if self.re >= 0.0 {
            C::new(t, self.im / (2.0 * t))
        } else {
            C::new(self.im.abs() / (2.0 * t), t.copysign(self.im))
        }
    }

    /// i^q for any integer q.
    pub fn i_pow(q: i64) -> C {
        match q.rem_euclid(4) {
            0 => C::ONE,
            1 => C::I,
            2 => C::real(-1.0),
            _ => C::new(0.0, -1.0),
        }
    }
}

impl Add for C {
    type Output = C;
    fn add(self, o: C) -> C {
        C::new(self.re + o.re, self.im + o.im)
    }
}

impl Sub for C {
    type Output = C;
    fn sub(self, o: C) -> C {
        C::new(self.re - o.re, self.im - o.im)
    }
}

impl Mul for C {
    type Output = C;
    fn mul(self, o: C) -> C {
        C::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
}

impl Div for C {
    type Output = C;
    fn div(self, o: C) -> C {
        let d = o.norm_sqr();
        C::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }
}

impl Neg for C {
    type Output = C;
    fn neg(self) -> C {
        C::new(-self.re, -self.im)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principal_sqrt_squares_back_and_has_nonnegative_real_part() {
        for k in 0..400 {
            let z = C::new(((k * 37) % 41) as f64 - 20.0, ((k * 53) % 43) as f64 - 21.0);
            let s = z.sqrt();
            let back = s * s;
            assert!((back - z).abs() < 1e-12 * (1.0 + z.abs()), "{z:?}");
            assert!(s.re >= 0.0, "{z:?} -> {s:?}");
        }
    }
}
