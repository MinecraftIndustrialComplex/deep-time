// SPDX-License-Identifier: LGPL-3.0-or-later
//! Coherent noise on the sphere for the initial shapes and fracture lines (Cortial §3: "warping the
//! geodetic distances … using a simple noise function"): 3D value noise (lattice values from an
//! integer hash, quintic interpolation) evaluated at unit vectors, and its fractal sum. Written from
//! the textbook definition (Ebert et al., "Texturing and Modeling", ch. 2); no code ported.

use glam::DVec3;

fn hash(x: i64, y: i64, z: i64, seed: u64) -> f64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
    for v in [x, y, z] {
        h ^= (v as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = h.rotate_left(27).wrapping_mul(0x94D0_49BB_1331_11EB);
    }
    h ^= h >> 31;
    // [−1, 1)
    (h >> 11) as f64 * (2.0 / (1u64 << 53) as f64) - 1.0
}

fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Value noise in [−1, 1] at `p` (any point of 3D space), smooth, with unit lattice spacing.
pub fn value(p: DVec3, seed: u64) -> f64 {
    let f = p.floor();
    let (ix, iy, iz) = (f.x as i64, f.y as i64, f.z as i64);
    let (tx, ty, tz) = (fade(p.x - f.x), fade(p.y - f.y), fade(p.z - f.z));
    let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
    let mut c = [0.0; 8];
    for (k, v) in c.iter_mut().enumerate() {
        let (dx, dy, dz) = ((k & 1) as i64, ((k >> 1) & 1) as i64, ((k >> 2) & 1) as i64);
        *v = hash(ix + dx, iy + dy, iz + dz, seed);
    }
    let x00 = lerp(c[0], c[1], tx);
    let x10 = lerp(c[2], c[3], tx);
    let x01 = lerp(c[4], c[5], tx);
    let x11 = lerp(c[6], c[7], tx);
    lerp(lerp(x00, x10, ty), lerp(x01, x11, ty), tz)
}

/// Fractal sum of `octaves` value-noise octaves at unit vector `p`: base frequency `freq`
/// (features about 1/freq radians across), gain 0.5, lacunarity 2; roughly in [−1, 1].
pub fn fbm(p: DVec3, freq: f64, octaves: u32, seed: u64) -> f64 {
    let (mut sum, mut amp, mut f, mut norm) = (0.0, 1.0, freq, 0.0);
    for o in 0..octaves {
        sum += amp * value(p * f, seed.wrapping_add(u64::from(o) * 0x51ED));
        norm += amp;
        amp *= 0.5;
        f *= 2.0;
    }
    sum / norm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_bounded_smooth_and_deterministic() {
        let mut prev = value(DVec3::new(0.1, 0.2, 0.3), 5);
        for k in 1..2000 {
            let p = DVec3::new(0.1 + k as f64 * 0.001, 0.2, 0.3);
            let v = value(p, 5);
            assert!((-1.0..=1.0).contains(&v));
            assert!((v - prev).abs() < 0.02, "jump at {k}");
            prev = v;
        }
        assert_eq!(fbm(DVec3::X, 3.0, 4, 1), fbm(DVec3::X, 3.0, 4, 1));
        assert_ne!(fbm(DVec3::X, 3.0, 4, 1), fbm(DVec3::X, 3.0, 4, 2));
    }
}
