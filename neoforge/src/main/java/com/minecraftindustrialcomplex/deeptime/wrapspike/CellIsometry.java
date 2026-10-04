// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

/**
 * A rigid motion of the horizontal block grid that keeps blocks on blocks: a turn by a whole number of
 * quarter turns followed by a translation (dev-only wrap-engine spike, docs/design/wrap-engine.md).
 *
 * <p>Turns are clockwise seen from above, Minecraft's sense: one quarter turn takes north (−z) to east
 * (+x), as {@code Rotation.CLOCKWISE_90} does. On block <em>cells</em> (the block at integer (x, z)
 * spans [x, x+1) × [z, z+1)) the turns about the origin are
 * <pre>
 *   k = 0: (x, z) → ( x,      z)
 *   k = 1: (x, z) → (−z − 1,  x)
 *   k = 2: (x, z) → (−x − 1, −z − 1)
 *   k = 3: (x, z) → ( z,     −x − 1)
 * </pre>
 * which are the continuous turns of the cell centres. So a half-turn about the integer point (a, b) is
 * {@code CellIsometry.halfTurnAbout(a, b)}: (x, z) → (2a − x − 1, 2b − z − 1) on cells and
 * (x, z) → (2a − x, 2b − z) on points. Pure integer algebra, no Minecraft types, so it is unit-tested
 * on its own (WrapTopologyTest).
 *
 * @param quarterTurns 0..3, clockwise from above
 * @param tx           translation in x, applied after the turn
 * @param tz           translation in z, applied after the turn
 */
public record CellIsometry(int quarterTurns, int tx, int tz) {

    public static final CellIsometry IDENTITY = new CellIsometry(0, 0, 0);

    public CellIsometry {
        quarterTurns = Math.floorMod(quarterTurns, 4);
    }

    public static CellIsometry translation(int dx, int dz) {
        return new CellIsometry(0, dx, dz);
    }

    /** The half-turn about the integer grid point (a, b) (a block corner). */
    public static CellIsometry halfTurnAbout(int a, int b) {
        return new CellIsometry(2, 2 * a, 2 * b);
    }

    /** The quarter turn (k = 1 clockwise, k = 3 counter-clockwise) about the integer grid point (a, b). */
    public static CellIsometry quarterTurnAbout(int k, int a, int b) {
        // p ↦ R(p − c) + c on points; on cells R carries the −1 offsets, so solve t = c − R_point(c).
        CellIsometry r = new CellIsometry(k, 0, 0);
        double[] rc = r.applyPoint(a, b);
        return new CellIsometry(k, (int) Math.round(a - rc[0]), (int) Math.round(b - rc[1]));
    }

    /** The cell (x, z) moved: {x', z'}. */
    public int[] apply(int x, int z) {
        int rx, rz;
        switch (quarterTurns) {
            case 1 -> { rx = -z - 1; rz = x; }
            case 2 -> { rx = -x - 1; rz = -z - 1; }
            case 3 -> { rx = z; rz = -x - 1; }
            default -> { rx = x; rz = z; }
        }
        return new int[]{rx + tx, rz + tz};
    }

    /** The point (x, z) moved (entities, which live on continuous coordinates): {x', z'}. */
    public double[] applyPoint(double x, double z) {
        double rx, rz;
        switch (quarterTurns) {
            case 1 -> { rx = -z; rz = x; }
            case 2 -> { rx = -x; rz = -z; }
            case 3 -> { rx = z; rz = -x; }
            default -> { rx = x; rz = z; }
        }
        return new double[]{rx + tx, rz + tz};
    }

    /** A direction or velocity (dx, dz) turned (no translation): {dx', dz'}. */
    public double[] applyVector(double dx, double dz) {
        return switch (quarterTurns) {
            case 1 -> new double[]{-dz, dx};
            case 2 -> new double[]{-dx, -dz};
            case 3 -> new double[]{dz, -dx};
            default -> new double[]{dx, dz};
        };
    }

    /** this ∘ other: first {@code other}, then this. */
    public CellIsometry compose(CellIsometry other) {
        // this(other(p)) = R1 (R2 p + t2) + t1 on points = (R1R2) p + (R1 t2 + t1).
        double[] t = applyVector(other.tx, other.tz);
        return new CellIsometry(quarterTurns + other.quarterTurns, (int) Math.round(t[0]) + tx, (int) Math.round(t[1]) + tz);
    }

    public CellIsometry inverse() {
        // p = R⁻¹ (q − t)  ⇒  translation −R⁻¹ t.
        CellIsometry rInv = new CellIsometry(-quarterTurns, 0, 0);
        double[] t = rInv.applyVector(-tx, -tz);
        return new CellIsometry(-quarterTurns, (int) Math.round(t[0]), (int) Math.round(t[1]));
    }

    public boolean isIdentity() {
        return quarterTurns == 0 && tx == 0 && tz == 0;
    }

    /** Yaw change in degrees for an entity carried by this motion (Minecraft yaw grows clockwise from above). */
    public float yawDegrees() {
        return 90f * quarterTurns;
    }
}
