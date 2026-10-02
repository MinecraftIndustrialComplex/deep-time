// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Ported from Noddy (Loop3D/noddyverse, commit 6a21b23, MIT License, Copyright (c) 2021 Loop3D):
// `src/utils.c` (`convrt`, `irotate`, `rotset`, `matmlt`, `paxis`, `plane`, `polcar`). See NOTICE.md.
//! Orientations: from dip, dip direction and pitch to the rotation matrices the events use.
//!
//! Every event works in its own frame: the point is moved to the event's position and multiplied
//! by the event's *forward* matrix, displaced there along one axis, and brought back with the
//! *reverse* matrix. [`rotset`] builds the pair from the attitude of the event's plane and the
//! pitch of a line in it; [`Attitude`] is what the original's `convrt` leaves in its globals
//! (`pole`, `axis`).

use crate::{M3, V3};

/// `utils.c`'s tolerance (0.01; `unEvents.c` has its own, 0.001): a pitch whose cosine is below
/// it is taken as 90°.
pub const PITCH_TOLERANCE: f64 = 0.01;

/// Row vector times matrix, `r[m] = Σ_j p[j] · M[j][m]`, summed in the original's order
/// (`matmlt` with one row).
#[inline]
pub fn vm(p: V3, m: &M3) -> V3 {
    let mut r = [0.0; 3];
    for (c, out) in r.iter_mut().enumerate() {
        let mut s = 0.0;
        for j in 0..3 {
            s += p[j] * m[j][c];
        }
        *out = s;
    }
    r
}

/// `matmlt(matp, matq, matr, 3)`: `R = Q · P` (every row of Q times P).
pub fn mm(p: &M3, q: &M3) -> M3 {
    [vm(q[0], p), vm(q[1], p), vm(q[2], p)]
}

/// Degrees to radians the way the original does it (`atan(1.0) / 45.0`).
#[inline]
pub fn rad_per_deg() -> f64 {
    1.0f64.atan() / 45.0
}

/// `irotate`: the matrix that turns row vectors about the axis through the origin and (x, y, z)
/// by `angle` radians.
pub fn irotate(x: f64, y: f64, z: f64, angle: f64) -> M3 {
    let absval = ((x * x) + (y * y) + (z * z)).sqrt();
    let (x1, y1, z1) = (x / absval, y / absval, z / absval);
    let (cosang, sinang) = (angle.cos(), angle.sin());
    [
        [
            (x1 * x1) + ((1.0 - (x1 * x1)) * cosang),
            (x1 * y1 * (1.0 - cosang)) + (z1 * sinang),
            (x1 * z1 * (1.0 - cosang)) - (y1 * sinang),
        ],
        [
            (x1 * y1 * (1.0 - cosang)) - (z1 * sinang),
            (y1 * y1) + ((1.0 - (y1 * y1)) * cosang),
            (y1 * z1 * (1.0 - cosang)) + (x1 * sinang),
        ],
        [
            (x1 * z1 * (1.0 - cosang)) + (y1 * sinang),
            (y1 * z1 * (1.0 - cosang)) - (x1 * sinang),
            (z1 * z1) + ((1.0 - (z1 * z1)) * cosang),
        ],
    ]
}

/// What `convrt` computes: the direction cosines of the pole to a plane and of the line pitching
/// in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attitude {
    /// The pole to the plane (upward for dips below 90°).
    pub pole: V3,
    /// The line in the plane at the pitch.
    pub axis: V3,
}

/// `convrt(dip, dipdir, pitch, TRUE)`: angles in degrees.
pub fn convrt(dip: f64, dipdir: f64, pitch: f64) -> Attitude {
    let rad = rad_per_deg();
    let halfpi = rad * 90.0;
    let dip = dip * rad;
    let pitch = pitch * rad;
    let dipdir = dipdir * rad;
    let pole = [
        dipdir.sin() * dip.sin(),
        dipdir.cos() * dip.sin(),
        dip.cos(),
    ];
    let matd = [(dipdir - halfpi).sin(), (dipdir - halfpi).cos(), 0.0];
    let matc = irotate(pole[0], pole[1], pole[2], -pitch);
    Attitude {
        pole,
        axis: vm(matd, &matc),
    }
}

/// `plane`: the plane a·x + b·y + c·z + d = 0 through `at` whose normal is the attitude's pole.
pub fn plane(att: &Attitude, at: V3) -> [f64; 4] {
    let [a, b, c] = att.pole;
    [a, b, c, -((a * at[0]) + (b * at[1]) + (c * at[2]))]
}

/// `paxis`: the direction cosines of a line from its plunge and plunge direction, degrees.
pub fn paxis(plunge: f64, plunge_direction: f64) -> V3 {
    let rad = rad_per_deg();
    let theta = plunge * rad;
    let phi = plunge_direction * rad;
    [
        theta.cos() * phi.sin(),
        theta.cos() * phi.cos(),
        -theta.sin(),
    ]
}

/// A forward and reverse rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    pub forward: M3,
    pub reverse: M3,
}

/// `rotset(dip, dipdir, pitch, forward, reverse)` after `convrt` with the same angles (every
/// caller in the original pairs them): the concatenated rotation that brings the pitching line
/// onto the local y axis and the plane's pole onto the local x axis, and its reverse.
///
/// So in an event's frame x is across its plane (positive on the pole's side), y along the line
/// that pitches in the plane and z the third axis, in the plane.
pub fn rotset(dip: f64, dipdir: f64, pitch: f64) -> Rotation {
    let att = convrt(dip, dipdir, pitch);
    // The original's literal, not π: the frame is built with `acos` of values at or next to 1,
    // where the last bit of an intermediate moves the angle by 1e-8, so matching the reference
    // closely needs the same constants.
    const PI: f64 = 3.1415926535;
    let rad = PI / 2.0;
    let rad_deg = rad_per_deg();
    let dip = dip * rad_deg;
    let pitch = pitch * rad_deg;
    let dipdir = dipdir * rad_deg;
    let temp1 = if pitch.cos().abs() < PITCH_TOLERANCE {
        dipdir
    } else {
        let t = pitch.sin() * dip.cos() / pitch.cos();
        if pitch > rad {
            t.atan() + PI + (dipdir - rad)
        } else {
            t.atan() + (dipdir - rad)
        }
    };
    let rot1 = irotate(0.0, 0.0, 1.0, temp1);
    let rot6 = irotate(0.0, 0.0, 1.0, -temp1);
    let mut newfa = vm(att.axis, &rot1);
    let newpo = vm(att.pole, &rot1);
    newfa[1] = newfa[1].clamp(-1.0, 1.0);
    let temp2 = newfa[1].acos();
    let rot2 = irotate(1.0, 0.0, 0.0, temp2);
    let rot5 = irotate(1.0, 0.0, 0.0, -temp2);
    let mut newpo2 = vm(newpo, &rot2);
    // The original clamps the upper side, then tests `newfa[1][1] < -1.0` (the wrong variable)
    // for the lower side, so a pole component rounded just below −1 reaches `acos` and gives
    // NaN. Clamped here on both sides: identical wherever the original is finite.
    newpo2[0] = newpo2[0].clamp(-1.0, 1.0);
    let temp3 = newpo2[0].acos();
    let rot3 = irotate(0.0, 1.0, 0.0, temp3);
    let rot4 = irotate(0.0, 1.0, 0.0, -temp3);
    // matmlt(rot2, rot1, temprt, 3); matmlt(rot3, temprt, conrot, 3)
    let conrot = mm(&rot3, &mm(&rot2, &rot1));
    // matmlt(rot5, rot4, temprt, 3); matmlt(rot6, temprt, revcon, 3)
    let revcon = mm(&rot6, &mm(&rot5, &rot4));
    Rotation {
        forward: conrot,
        reverse: revcon,
    }
}

/// `polcar`'s radius (the angle it also returns is never read by the event code).
#[inline]
pub fn radius(x: f64, y: f64) -> f64 {
    ((x * x) + (y * y)).powf(0.5)
}

/// The inverse of a 3 × 3 matrix by cofactors, or `None` when it is singular (the original calls
/// a NAG-style `f01aaf`).
pub fn invert(m: &M3) -> Option<M3> {
    let c =
        |r1: usize, c1: usize, r2: usize, c2: usize| m[r1][c1] * m[r2][c2] - m[r1][c2] * m[r2][c1];
    let cof = [
        [c(1, 1, 2, 2), -c(1, 0, 2, 2), c(1, 0, 2, 1)],
        [-c(0, 1, 2, 2), c(0, 0, 2, 2), -c(0, 0, 2, 1)],
        [c(0, 1, 1, 2), -c(0, 0, 1, 2), c(0, 0, 1, 1)],
    ];
    let det = m[0][0] * cof[0][0] + m[0][1] * cof[0][1] + m[0][2] * cof[0][2];
    if !det.is_finite() || det.abs() < 1e-300 {
        return None;
    }
    let mut inv = [[0.0; 3]; 3];
    for (r, row) in inv.iter_mut().enumerate() {
        for (k, v) in row.iter_mut().enumerate() {
            *v = cof[k][r] / det;
        }
    }
    Some(inv)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: V3, b: V3, tol: f64) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() <= tol)
    }

    #[test]
    fn forward_and_reverse_are_inverse_rotations() {
        for &(dip, dd, pitch) in &[
            (30.0, 40.0, 0.0),
            (89.999, 270.0, 45.0),
            (60.0, 135.0, 90.0),
            (10.0, 0.0, 120.0),
            (75.0, 359.0, 10.0),
        ] {
            let r = rotset(dip, dd, pitch);
            let p = [3.0, -7.0, 11.0];
            let q = vm(vm(p, &r.forward), &r.reverse);
            assert!(close(p, q, 1e-9), "{dip} {dd} {pitch}: {q:?}");
            // Orthonormal: lengths kept.
            let f = vm(p, &r.forward);
            let (lp, lf) = (
                (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt(),
                (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt(),
            );
            assert!((lp - lf).abs() < 1e-9);
        }
    }

    #[test]
    fn the_frame_puts_the_pole_on_x_and_the_pitch_line_on_y() {
        for &(dip, dd, pitch) in &[(30.0, 40.0, 0.0), (50.0, 200.0, 60.0), (80.0, 90.0, 20.0)] {
            let att = convrt(dip, dd, pitch);
            let r = rotset(dip, dd, pitch);
            let pole = vm(att.pole, &r.forward);
            let axis = vm(att.axis, &r.forward);
            assert!(pole[0].abs() > 1.0 - 1e-9, "{pole:?}");
            assert!(axis[1].abs() > 1.0 - 1e-9, "{axis:?}");
        }
    }

    #[test]
    fn a_pole_is_normal_to_its_plane() {
        // A plane dipping 30° towards the east (dip direction 90°): its pole leans east.
        let att = convrt(30.0, 90.0, 0.0);
        assert!(close(att.pole, [0.5, 0.0, 3.0f64.sqrt() / 2.0], 1e-12));
        // The strike line (pitch 0) is horizontal and in the plane.
        assert!(att.axis[2].abs() < 1e-12);
        let dot: f64 = (0..3).map(|k| att.pole[k] * att.axis[k]).sum();
        assert!(dot.abs() < 1e-12);
        let pl = plane(&att, [10.0, 20.0, 30.0]);
        assert!((pl[0] * 10.0 + pl[1] * 20.0 + pl[2] * 30.0 + pl[3]).abs() < 1e-12);
    }

    #[test]
    fn invert_gives_the_inverse() {
        let m = [[2.0, 0.5, 0.0], [0.1, 1.5, 0.3], [0.0, -0.2, 0.8]];
        let inv = invert(&m).unwrap();
        let id = mm(&inv, &m);
        for (r, row) in id.iter().enumerate() {
            for (c, v) in row.iter().enumerate() {
                let want = if r == c { 1.0 } else { 0.0 };
                assert!((v - want).abs() < 1e-12);
            }
        }
        assert!(invert(&[[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [0.0, 1.0, 0.0]]).is_none());
    }
}
