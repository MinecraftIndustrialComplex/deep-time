// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Ported from Noddy (Loop3D/noddyverse, commit 6a21b23, MIT License, Copyright (c) 2021 Loop3D):
// `src/matrix.c`, (c) Ewen Christopher 1994, (c) Monash University 1994. See NOTICE.md.
//! Homogeneous 4 × 4 transformation matrices: rotations, scaling and translation in one matrix
//! (`matrix.c`). The original builds its display and import transforms with them; here they
//! compose the placement of an event or a model window (a local east-north-up frame cut out of a
//! larger world) and apply it to points.
//!
//! Column vectors: `p' = M · p`, the translation in the last column, as in the original.

use crate::V3;

/// A 4 × 4 matrix, `m[row][column]`.
pub type M4 = [[f64; 4]; 4];

/// `setIdentity`.
pub fn identity() -> M4 {
    let mut m = [[0.0; 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    m
}

/// `multiplyMatrix`: `a · b`.
pub fn multiply(a: &M4, b: &M4) -> M4 {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            let mut s = 0.0;
            for k in 0..4 {
                s += a[i][k] * b[k][j];
            }
            r[i][j] = s;
        }
    }
    r
}

/// `rotateX`: a rotation of `d` radians about the x axis. (The original assigns `[1][1]` twice
/// and never `[1][2]`, leaving a matrix that is not a rotation; this is the rotation its sibling
/// functions imply.)
pub fn rotate_x(d: f64) -> M4 {
    let (s, c) = d.sin_cos();
    let mut m = identity();
    m[1][1] = c;
    m[1][2] = -s;
    m[2][1] = s;
    m[2][2] = c;
    m
}

/// `rotateY`.
pub fn rotate_y(d: f64) -> M4 {
    let (s, c) = d.sin_cos();
    let mut m = identity();
    m[0][0] = c;
    m[0][2] = s;
    m[2][0] = -s;
    m[2][2] = c;
    m
}

/// `rotateZ`.
pub fn rotate_z(d: f64) -> M4 {
    let (s, c) = d.sin_cos();
    let mut m = identity();
    m[0][0] = c;
    m[0][1] = -s;
    m[1][0] = s;
    m[1][1] = c;
    m
}

/// `translateBy`: a point at the origin moves to (x, y, z).
pub fn translate_by(x: f64, y: f64, z: f64) -> M4 {
    let mut m = identity();
    m[0][3] = x;
    m[1][3] = y;
    m[2][3] = z;
    m
}

/// `scaleBy`.
pub fn scale_by(x: f64, y: f64, z: f64) -> M4 {
    let mut m = identity();
    m[0][0] = x;
    m[1][1] = y;
    m[2][2] = z;
    m
}

/// Applies a transformation to a point (w = 1).
pub fn transform(m: &M4, p: V3) -> V3 {
    let mut r = [0.0; 3];
    for (i, out) in r.iter_mut().enumerate() {
        *out = m[i][0] * p[0] + m[i][1] * p[1] + m[i][2] * p[2] + m[i][3];
    }
    r
}

/// The inverse of a rigid transformation (rotation and translation only): `Rᵀ`, `−Rᵀ·t`.
pub fn invert_rigid(m: &M4) -> M4 {
    let mut r = identity();
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = m[j][i];
        }
    }
    for i in 0..3 {
        r[i][3] = -(r[i][0] * m[0][3] + r[i][1] * m[1][3] + r[i][2] * m[2][3]);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: V3, b: V3) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() < 1e-12)
    }

    #[test]
    fn rotations_turn_the_axes_the_right_way() {
        let q = std::f64::consts::FRAC_PI_2;
        assert!(close(
            transform(&rotate_z(q), [1.0, 0.0, 0.0]),
            [0.0, 1.0, 0.0]
        ));
        assert!(close(
            transform(&rotate_x(q), [0.0, 1.0, 0.0]),
            [0.0, 0.0, 1.0]
        ));
        assert!(close(
            transform(&rotate_y(q), [0.0, 0.0, 1.0]),
            [1.0, 0.0, 0.0]
        ));
    }

    #[test]
    fn a_composed_placement_inverts() {
        let m = multiply(
            &translate_by(100.0, -40.0, 7.0),
            &multiply(&rotate_z(0.7), &rotate_x(-0.2)),
        );
        let p = [3.0, 4.0, 5.0];
        let back = transform(&invert_rigid(&m), transform(&m, p));
        assert!(close(back, p));
        assert!(close(transform(&identity(), p), p));
        assert!(close(
            transform(&scale_by(2.0, 3.0, 4.0), p),
            [6.0, 12.0, 20.0]
        ));
    }
}
