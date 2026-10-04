// SPDX-License-Identifier: LGPL-3.0-or-later
//! Trigonometry that gives the **same bits** in Rust, Java and JS: the seamless world's projection
//! ([`crate::petroff`], [`crate::projection::PetroffGuyou`]) runs on it in all three languages, so the
//! bake, the game and the workbench agree on every block's latitude and longitude bit for bit
//! (Java twin `worlddata/PortableMath.java`, JS twin `DTMath` in
//! `deeptime-viewer/assets/projection.js`).
//!
//! Library `sin`, `atan` and the rest are not specified bit for bit (each platform rounds its own
//! way), so each function here is written with IEEE-754 double `+ − × /`, `sqrt` (correctly rounded
//! everywhere) and comparisons only, in an order every twin repeats exactly. None of the three
//! languages contracts `a·b + c` into a fused multiply-add on its own, so the twins round alike.
//!
//! - **sin, cos**: k = round(2x/π); r = (x − k·P1) − k·P1T with π/2 = P1 + P1T split so k·P1 is
//!   exact (P1 has 33 significant bits); then Taylor series on |r| ≤ π/4 to the 23rd (sin) and 24th
//!   (cos) power, whose next terms are below 10⁻²⁷. Accurate to about one unit in the last place for
//!   |x| < 10⁵ (the projection never passes more than a few turns).
//! - **atan**: |x| > 1 goes to π/2 − atan(1/|x|); the half-angle identity
//!   atan t = 2·atan(t / (1 + √(1 + t²))) applied three times takes t ∈ [0, 1] below 0.1, then the
//!   Taylor series to the 31st power. Accurate to a few units in the last place.
//! - **atan2** from atan of the smaller over the larger magnitude, then the quadrant.
//! - **asin, acos** from atan2 and √((1 − x)(1 + x)).
//!
//! Accuracy is a tested property (against the platform's own functions, `tests` below); identity
//! across the languages is tested by the parity fixture (`ColumnParityTest`, the pages' check).

/// π/2's high part: 33 significant bits, so k·P1 is exact for |k| < 2²⁰ (fdlibm's `pio2_1`).
const P1: f64 = 1.570_796_326_734_125_614_17;
/// π/2 − P1 (fdlibm's `pio2_1t`).
const P1T: f64 = 6.077_100_506_506_192_249_32e-11;
/// π/2 rounded to double.
pub const HALF_PI: f64 = 1.570_796_326_794_896_558;
/// π/2 − HALF_PI.
const HALF_PI_LO: f64 = 6.123_233_995_736_766_036e-17;
/// π rounded to double.
pub const PI: f64 = 3.141_592_653_589_793_116;
/// π − PI.
const PI_LO: f64 = 1.224_646_799_147_353_207_2e-16;

/// sin r on |r| ≤ π/4 (about): Taylor series to r²³.
#[inline]
fn kernel_sin(r: f64) -> f64 {
    let z = r * r;
    let p = -1.0 / 6.0
        + z * (1.0 / 120.0
            + z * (-1.0 / 5_040.0
                + z * (1.0 / 362_880.0
                    + z * (-1.0 / 39_916_800.0
                        + z * (1.0 / 6_227_020_800.0
                            + z * (-1.0 / 1_307_674_368_000.0
                                + z * (1.0 / 355_687_428_096_000.0
                                    + z * (-1.0 / 121_645_100_408_832_000.0
                                        + z * (1.0 / 51_090_942_171_709_440_000.0
                                            + z * (-1.0 / 25_852_016_738_884_976_640_000.0))))))))));
    r + r * (z * p)
}

/// cos r on |r| ≤ π/4 (about): Taylor series to r²⁴, summed as fdlibm's kernel does
/// (1 − z/2 split so the leading terms keep their bits).
#[inline]
fn kernel_cos(r: f64) -> f64 {
    let z = r * r;
    let q = 1.0 / 24.0
        + z * (-1.0 / 720.0
            + z * (1.0 / 40_320.0
                + z * (-1.0 / 3_628_800.0
                    + z * (1.0 / 479_001_600.0
                        + z * (-1.0 / 87_178_291_200.0
                            + z * (1.0 / 20_922_789_888_000.0
                                + z * (-1.0 / 6_402_373_705_728_000.0
                                    + z * (1.0 / 2_432_902_008_176_640_000.0
                                        + z * (-1.0 / 1_124_000_727_777_607_680_000.0
                                            + z * (1.0
                                                / 620_448_401_733_239_439_360_000.0))))))))));
    let hz = 0.5 * z;
    let w = 1.0 - hz;
    w + (((1.0 - w) - hz) + (z * z) * q)
}

/// Rounds half away from zero, as every twin does (no library `round`, whose ties differ).
#[inline]
fn round_half_away(x: f64) -> f64 {
    if x >= 0.0 {
        (x + 0.5).floor()
    } else {
        -((-x + 0.5).floor())
    }
}

/// (k mod 4, r): x = k·π/2 + r with |r| ≤ π/4 (about).
#[inline]
fn reduce(x: f64) -> (i64, f64) {
    let k = round_half_away(x * (2.0 / PI));
    let r = (x - k * P1) - k * P1T;
    ((k as i64).rem_euclid(4), r)
}

/// sin x.
pub fn sin(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let (q, r) = reduce(x);
    match q {
        0 => kernel_sin(r),
        1 => kernel_cos(r),
        2 => -kernel_sin(r),
        _ => -kernel_cos(r),
    }
}

/// cos x.
pub fn cos(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let (q, r) = reduce(x);
    match q {
        0 => kernel_cos(r),
        1 => -kernel_sin(r),
        2 => -kernel_cos(r),
        _ => kernel_sin(r),
    }
}

/// tan x = sin x / cos x.
pub fn tan(x: f64) -> f64 {
    sin(x) / cos(x)
}

/// atan t for t in [0, 1]: three half-angle steps, then the Taylor series.
#[inline]
fn atan_unit(t: f64) -> f64 {
    let mut s = t;
    // atan t = 2·atan(t / (1 + √(1 + t²))), three times: t ≤ 1 → ≤ 0.414 → ≤ 0.199 → ≤ 0.0985.
    s = s / (1.0 + (1.0 + s * s).sqrt());
    s = s / (1.0 + (1.0 + s * s).sqrt());
    s = s / (1.0 + (1.0 + s * s).sqrt());
    let z = s * s;
    let p = -1.0 / 3.0
        + z * (1.0 / 5.0
            + z * (-1.0 / 7.0
                + z * (1.0 / 9.0
                    + z * (-1.0 / 11.0
                        + z * (1.0 / 13.0
                            + z * (-1.0 / 15.0
                                + z * (1.0 / 17.0
                                    + z * (-1.0 / 19.0
                                        + z * (1.0 / 21.0
                                            + z * (-1.0 / 23.0
                                                + z * (1.0 / 25.0
                                                    + z * (-1.0 / 27.0
                                                        + z * (1.0 / 29.0
                                                            + z * (-1.0 / 31.0))))))))))))));
    8.0 * (s + s * (z * p))
}

/// atan x, in [−π/2, π/2].
pub fn atan(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let a = x.abs();
    let r = if a <= 1.0 {
        atan_unit(a)
    } else if a.is_infinite() {
        HALF_PI
    } else {
        HALF_PI - (atan_unit(1.0 / a) - HALF_PI_LO)
    };
    if x < 0.0 { -r } else { r }
}

/// atan2(y, x), in [−π, π]: the angle of the point (x, y). atan2(±0, x ≥ +0) = ±0 and
/// atan2(0, 0) = 0; atan2(y, x < 0) for y = ±0 is ±π.
pub fn atan2(y: f64, x: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    let (ax, ay) = (x.abs(), y.abs());
    if ax == 0.0 && ay == 0.0 {
        // The sign of the zero decides, as the platforms do: (±0, +0) → ±0, (±0, −0) → ±π.
        let neg_x = x.is_sign_negative();
        let a = if neg_x { PI } else { 0.0 };
        return if y.is_sign_negative() { -a } else { a };
    }
    let mut a = if ay <= ax {
        if ax.is_infinite() {
            if ay.is_infinite() { atan_unit(1.0) } else { 0.0 }
        } else {
            atan_unit(ay / ax)
        }
    } else if ay.is_infinite() {
        HALF_PI
    } else {
        HALF_PI - (atan_unit(ax / ay) - HALF_PI_LO)
    };
    if x < 0.0 {
        a = PI - (a - PI_LO);
    }
    if y < 0.0 || (y == 0.0 && y.is_sign_negative()) {
        -a
    } else {
        a
    }
}

/// asin x for x in [−1, 1] (clamped), in [−π/2, π/2].
pub fn asin(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let x = x.clamp(-1.0, 1.0);
    atan2(x, ((1.0 - x) * (1.0 + x)).sqrt())
}

/// acos x for x in [−1, 1] (clamped), in [0, π].
pub fn acos(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let x = x.clamp(-1.0, 1.0);
    atan2(((1.0 - x) * (1.0 + x)).sqrt(), x)
}

/// Degrees to radians, as the twins write it (`x · (π / 180)` with the quotient rounded once).
#[inline]
pub fn to_radians(deg: f64) -> f64 {
    deg * (PI / 180.0)
}

/// Radians to degrees (`x · (180 / π)`).
#[inline]
pub fn to_degrees(rad: f64) -> f64 {
    rad * (180.0 / PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SplitMix64, for property tests without a dependency.
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

    fn ulps(a: f64, b: f64) -> f64 {
        let scale = a.abs().max(b.abs()).max(f64::MIN_POSITIVE);
        (a - b).abs() / (scale * f64::EPSILON)
    }

    #[test]
    fn constants_are_the_splits_of_pi() {
        assert_eq!(HALF_PI, std::f64::consts::FRAC_PI_2);
        assert_eq!(PI, std::f64::consts::PI);
        assert!((P1 + P1T - std::f64::consts::FRAC_PI_2).abs() < 1e-16);
        assert_eq!(P1.to_bits() & 0xF_FFFF, 0, "P1 has 33 significant bits");
        // π/2 − HALF_PI and π − PI, to the double nearest each.
        assert!((HALF_PI_LO * 2.0 - PI_LO).abs() < 1e-31);
    }

    #[test]
    fn sin_cos_tan_match_the_platform_within_two_ulps_or_1e_16() {
        let mut g = Rng(0x51);
        for _ in 0..200_000 {
            let x = (g.f() - 0.5) * 40.0;
            for (mine, theirs) in [(sin(x), x.sin()), (cos(x), x.cos())] {
                assert!(
                    (mine - theirs).abs() <= 1e-16 || ulps(mine, theirs) <= 2.0,
                    "x {x}: {mine} vs {theirs}"
                );
            }
            let t = tan(x);
            assert!(ulps(t, x.tan()) <= 8.0 || (t - x.tan()).abs() < 1e-15, "tan {x}");
        }
        assert_eq!(sin(0.0), 0.0);
        assert_eq!(cos(0.0), 1.0);
        assert!(sin(f64::NAN).is_nan() && cos(f64::INFINITY).is_nan());
    }

    #[test]
    fn inverse_functions_match_the_platform_within_four_ulps_or_1e_16() {
        let mut g = Rng(0xA7A);
        for _ in 0..200_000 {
            let x = (g.f() - 0.5) * 2.0;
            let big = (g.f() - 0.5) * 1e6;
            let y = (g.f() - 0.5) * 2.0;
            for (mine, theirs, what) in [
                (atan(x), x.atan(), "atan"),
                (atan(big), big.atan(), "atan big"),
                (asin(x), x.asin(), "asin"),
                (acos(x), x.acos(), "acos"),
                (atan2(y, x), y.atan2(x), "atan2"),
                (atan2(big, x), big.atan2(x), "atan2 big"),
            ] {
                assert!(
                    (mine - theirs).abs() <= 1e-16 || ulps(mine, theirs) <= 4.0,
                    "{what} {x} {y} {big}: {mine} vs {theirs}"
                );
            }
        }
        for (y, x) in [
            (0.0, 0.0),
            (-0.0, 0.0),
            (0.0, -0.0),
            (-0.0, -0.0),
            (1.0, 0.0),
            (-1.0, 0.0),
            (0.0, -1.0),
            (-0.0, -1.0),
            (f64::INFINITY, 1.0),
            (1.0, f64::NEG_INFINITY),
        ] {
            let (m, t) = (atan2(y, x), y.atan2(x));
            assert!(
                m.to_bits() == t.to_bits() || (m - t).abs() < 1e-15,
                "atan2({y}, {x}) = {m}, platform {t}"
            );
        }
        assert_eq!(asin(1.0), HALF_PI);
        assert_eq!(acos(1.0), 0.0);
        assert_eq!(acos(-1.0), PI);
        assert_eq!(asin(2.0), HALF_PI, "clamped");
    }
}
