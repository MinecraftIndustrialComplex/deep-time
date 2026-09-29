// SPDX-License-Identifier: LGPL-3.0-or-later
//! Rigid plate motion (plan §8.1): finite rotations about Euler poles and the surface velocity
//! s(p) = ω w × p (Cortial §3). Standard kinematics (the GPlates concepts), no code ported.
//!
//! A plate's motion is its angular-velocity vector Ω = ω·w (rad/yr) in the mantle frame. Every
//! step turns each of its groups by the rotation of angle |Ω|·δt about Ω/|Ω|. Two slow changes
//! act on Ω between discrete events: slab pull (Cortial §4.1: the axis of a subducting plate turns
//! towards its fronts, w_i += ε Σ c_i × q_k / |c_i × q_k| δt, with ε scaled by front length) and
//! a small random walk standing in for mantle reorganisation. Speeds are capped at v_0.

use glam::{DQuat, DVec3};

use super::normalise::Norm;
use super::state::PlateState;
use crate::sphere::SplitMix64;

/// The quaternion of stored `[x, y, z, w]`.
pub fn quat(a: [f64; 4]) -> DQuat {
    DQuat::from_xyzw(a[0], a[1], a[2], a[3])
}

/// Stored form of a quaternion.
pub fn arr(q: DQuat) -> [f64; 4] {
    [q.x, q.y, q.z, q.w]
}

/// The rotation Ω applies over `dt_yr` years.
pub fn rotation(omega: DVec3, dt_yr: f64) -> DQuat {
    let w = omega.length();
    let angle = w * dt_yr;
    if w <= 0.0 || angle.abs() < 1e-15 {
        DQuat::IDENTITY
    } else {
        DQuat::from_axis_angle(omega / w, angle)
    }
}

/// A vector of a group frame (stored as f32) in the mantle frame.
pub fn to_world(rot: DQuat, v: [f32; 3]) -> DVec3 {
    rot * DVec3::new(f64::from(v[0]), f64::from(v[1]), f64::from(v[2]))
}

/// A mantle-frame vector in a group frame, stored as f32.
pub fn to_group(rot: DQuat, v: DVec3) -> [f32; 3] {
    let r = rot.inverse() * v;
    [r.x as f32, r.y as f32, r.z as f32]
}

/// The tangent part of `v` at the unit point `p`.
pub fn tangent(p: DVec3, v: DVec3) -> DVec3 {
    v - p * v.dot(p)
}

/// Azimuth of the tangent vector `v` at `p`, degrees clockwise from north in [0°, 360°); NaN for
/// a zero vector or at a pole. North is +Z (plan §5.2: v = (cos φ cos λ, cos φ sin λ, sin φ)).
pub fn azimuth_deg(p: DVec3, v: DVec3) -> f64 {
    let east = DVec3::Z.cross(p);
    if east.length_squared() < 1e-18 {
        return f64::NAN;
    }
    let east = east.normalize();
    let north = p.cross(east);
    let (e, n) = (v.dot(east), v.dot(north));
    if e * e + n * n < 1e-30 {
        return f64::NAN;
    }
    e.atan2(n).to_degrees().rem_euclid(360.0)
}

/// Moves every group one step: each group turns by its plate's rotation over `dt_yr`.
pub fn advance(state: &mut PlateState, dt_yr: f64) {
    let rots: Vec<DQuat> = state
        .plates
        .iter()
        .map(|p| rotation(DVec3::from_array(p.omega), dt_yr))
        .collect();
    for g in &mut state.groups {
        let r = rots[g.plate as usize] * quat(g.rot);
        g.rot = arr(r.normalize());
    }
}

/// Caps |Ω| so no point of the plate exceeds `v_max` mm/yr.
pub fn cap_speed(omega: DVec3, norm: &Norm, v_max_mm_yr: f64) -> DVec3 {
    let max = norm.omega_of_speed(v_max_mm_yr);
    let w = omega.length();
    if w > max { omega * (max / w) } else { omega }
}

/// An Euler pole and angular speed giving speed `v_mm_yr` at `centroid` (the pole is random but
/// at least 30° from the centroid, so the plate does not merely spin).
pub fn random_motion(rng: &mut SplitMix64, centroid: DVec3, v_mm_yr: f64, norm: &Norm) -> DVec3 {
    let mut axis = rng.unit_vector();
    for _ in 0..16 {
        if axis.cross(centroid).length() > 0.5 {
            break;
        }
        axis = rng.unit_vector();
    }
    let lever = axis.cross(centroid).length().max(0.3);
    axis * (norm.omega_of_speed(v_mm_yr) / lever)
}

/// Turns the axis of Ω towards `pull` (a sum of c × q_k directions weighted by front length,
/// radians), keeping |Ω|: slab pull.
pub fn pull_axis(omega: DVec3, pull: DVec3) -> DVec3 {
    let w = omega.length();
    if w <= 0.0 || pull.length_squared() == 0.0 {
        return omega;
    }
    let axis = (omega / w + pull).normalize_or_zero();
    if axis == DVec3::ZERO { omega } else { axis * w }
}

/// The random walk of Ω: a Gaussian nudge of relative size `sigma` (per √Myr) over `dt_myr`.
pub fn jitter(rng: &mut SplitMix64, omega: DVec3, sigma: f64, dt_myr: f64) -> DVec3 {
    let w = omega.length();
    if w <= 0.0 || sigma <= 0.0 {
        return omega;
    }
    let s = sigma * dt_myr.sqrt() * w;
    omega + DVec3::new(rng.normal(), rng.normal(), rng.normal()) * s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_moves_points_at_the_plate_speed() {
        let norm = Norm::new(6370.0, 782.0, 1000);
        let omega = DVec3::Z * norm.omega_of_speed(50.0);
        let dt = 2.0e6;
        let q = rotation(omega, dt);
        let p = DVec3::X;
        let moved = q * p;
        // 50 mm/yr for 2 Myr = 100 km on the model sphere.
        let km = norm.km(p.angle_between(moved));
        assert!((km - 100.0).abs() < 1e-6, "{km}");
        // The velocity points along the motion.
        assert!(norm.velocity_mm_yr(omega, p).dot(moved - p) > 0.0);
        assert_eq!(quat(arr(q)), q);
    }

    #[test]
    fn vectors_round_trip_through_group_frames() {
        let q = DQuat::from_axis_angle(DVec3::new(1.0, 2.0, 3.0).normalize(), 0.7);
        let v = DVec3::new(0.1, -0.4, 0.2);
        let back = to_world(q, to_group(q, v));
        assert!((back - v).length() < 1e-6);
        let p = DVec3::X;
        assert!((azimuth_deg(p, DVec3::Z) - 0.0).abs() < 1e-9, "north");
        assert!((azimuth_deg(p, DVec3::Y) - 90.0).abs() < 1e-9, "east");
        assert!(azimuth_deg(DVec3::Z, DVec3::X).is_nan());
    }

    #[test]
    fn speed_cap_and_pull() {
        let norm = Norm::new(6370.0, 782.0, 1000);
        let fast = DVec3::Z * norm.omega_of_speed(300.0);
        let capped = cap_speed(fast, &norm, 100.0);
        assert!((norm.speed_mm_yr(capped, DVec3::X) - 100.0).abs() < 1e-9);
        let pulled = pull_axis(capped, DVec3::X * 0.1);
        assert!((pulled.length() - capped.length()).abs() < 1e-20);
        assert!(pulled.x > 0.0);
        let mut rng = SplitMix64::new(3);
        let m = random_motion(&mut rng, DVec3::Y, 40.0, &norm);
        assert!((norm.speed_mm_yr(m, DVec3::Y) - 40.0).abs() < 1e-6);
    }
}
