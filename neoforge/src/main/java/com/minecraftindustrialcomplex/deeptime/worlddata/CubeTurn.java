// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * Where the cube sits on the planet: a rotation, planet frame to cube frame (the manifest's
 * {@code projection.rotation}, 3 × 3, row-major). The planet frame is the format's: +x the point
 * (0°, 0°), +y (0°, 90° E), +z the north pole. The world's turn is free (the corner search); the
 * simulation's cube takes the identity (its poles at the centres of N and S, λ0 = 0). The twin of
 * Rust's {@code deeptime_format::cube::CubeTurn}: the products keep its order and grouping
 * ({@code (m0·p0 + m1·p1) + m2·p2}), so the bits agree.
 */
public final class CubeTurn {

    /** The identity: the polar-axis cube. */
    public static final CubeTurn IDENTITY = new CubeTurn(new double[][]{{1.0, 0.0, 0.0}, {0.0, 1.0, 0.0}, {0.0, 0.0, 1.0}});

    private final double[][] m;

    private CubeTurn(double[][] m) {
        this.m = m;
    }

    /**
     * A turn from its rows (planet to cube), checked to be a rotation: orthonormal to 1e-12 and of
     * determinant +1 (a negative determinant is a reflection and is refused; the rows are copied).
     *
     * @throws WorldDataException when the rows are not a rotation
     */
    public static CubeTurn fromRows(double[][] rows) {
        if (rows == null || rows.length != 3)
            throw new WorldDataException("cube rotation needs 3 rows");
        double[][] m = new double[3][];
        for (int i = 0; i < 3; i++) {
            if (rows[i] == null || rows[i].length != 3)
                throw new WorldDataException("cube rotation row " + i + " needs 3 numbers");
            m[i] = rows[i].clone();
        }
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                double[] a = m[i], b = m[j];
                double d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
                double want = i == j ? 1.0 : 0.0;
                if (Math.abs(d - want) > 1e-12 || !Double.isFinite(d))
                    throw new WorldDataException("cube rotation rows " + i + ", " + j + ": dot " + d + ", want " + want);
            }
        }
        double det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
                - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
                + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
        if (det < 0.0)
            throw new WorldDataException("cube rotation has determinant " + det + ": a reflection");
        return new CubeTurn(m);
    }

    /** The polar-axis turn by λ0 (the old layout's): the planet point (0°, λ0) at F0's centre, the north pole at N's. */
    public static CubeTurn polar(double lambda0Deg) {
        double l = PortableMath.toRadians(lambda0Deg);
        double s = PortableMath.sin(l), c = PortableMath.cos(l);
        return new CubeTurn(new double[][]{{c, s, 0.0}, {-s, c, 0.0}, {0.0, 0.0, 1.0}});
    }

    /** The rows, planet to cube (a copy). */
    public double[][] rows() {
        return new double[][]{m[0].clone(), m[1].clone(), m[2].clone()};
    }

    /** A planet-frame vector in the cube frame (Q·p). */
    public double[] toCube(double[] p) {
        return new double[]{
                (m[0][0] * p[0] + m[0][1] * p[1]) + m[0][2] * p[2],
                (m[1][0] * p[0] + m[1][1] * p[1]) + m[1][2] * p[2],
                (m[2][0] * p[0] + m[2][1] * p[1]) + m[2][2] * p[2]};
    }

    /** A cube-frame vector in the planet frame (Qᵀ·c). */
    public double[] toPlanet(double[] c) {
        return new double[]{
                (m[0][0] * c[0] + m[1][0] * c[1]) + m[2][0] * c[2],
                (m[0][1] * c[0] + m[1][1] * c[1]) + m[2][1] * c[2],
                (m[0][2] * c[0] + m[1][2] * c[1]) + m[2][2] * c[2]};
    }
}
