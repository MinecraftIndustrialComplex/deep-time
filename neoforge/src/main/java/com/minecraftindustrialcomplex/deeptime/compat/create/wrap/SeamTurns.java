// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.wrapengine.CellIsometry;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;

import java.util.List;

/**
 * What a seam's turn does to an axial quantity: which axis a block's rotation axis becomes on the
 * other side, and whether a signed speed along that axis keeps its sign (docs/design/wrap-engine.md,
 * "Create across turning seams"). Pure integer code, no Minecraft or Create types, so it is
 * unit-tested on its own (SeamTurnsTest).
 *
 * <p>Create stores a kinetic block's speed as a signed angular velocity about the <em>positive</em>
 * direction of its rotation axis ({@code KineticBlockEntity.convertToDirection} flips the sign for a
 * face on the negative side). An angular velocity is an axial vector, but under a proper rotation (a
 * turn about the vertical, determinant +1) it transforms like an ordinary vector: ω' = R ω. So a speed
 * s about the positive x axis seen through a turn R is s about R(+x). If R(+x) is a negative axis
 * direction the stored speed on the far side is −s. Concretely, with turns clockwise from above:
 * <pre>
 *   turn      x axis       z axis       y axis
 *   0         x, +         z, +         y, +
 *   90 (k=1)  z, +         x, −         y, +
 *   180 (k=2) x, −         z, −         y, +
 *   270 (k=3) z, −         x, +         y, +
 * </pre>
 * A half-turn flips every horizontal speed and keeps every vertical one; a quarter-turn swaps x and z
 * and flips one of them.
 */
public final class SeamTurns {

    /** Axis ids, matching {@code Direction.Axis.ordinal()} (X, Y, Z). */
    public static final int X = 0, Y = 1, Z = 2;

    private SeamTurns() {}

    /** The axis an axis becomes under {@code quarterTurns} clockwise turns about the vertical. */
    public static int turnedAxis(int quarterTurns, int axis) {
        if (axis == Y || (quarterTurns & 1) == 0)
            return axis;
        return axis == X ? Z : X;
    }

    /**
     * +1 when a speed about {@code axis}'s positive direction is a speed about the turned axis's
     * positive direction after {@code quarterTurns} clockwise quarter turns, −1 when its sign flips.
     */
    public static int axisSign(int quarterTurns, int axis) {
        if (axis == Y)
            return 1;
        double[] v = new CellIsometry(quarterTurns, 0, 0).applyVector(axis == X ? 1 : 0, axis == Z ? 1 : 0);
        return v[0] + v[1] > 0 ? 1 : -1;
    }

    /** {@link #axisSign} for a motion. */
    public static int axisSign(CellIsometry iso, int axis) {
        return axisSign(iso.quarterTurns(), axis);
    }

    /**
     * Pure part of {@code SeamFrames.link}: the image {x, z} of stored block (tx, tz) within {@code reach} of
     * (vx, vz), nearest first; null if none. Unit-tested (SeamTurnsTest).
     */
    /**
     * The image {x, z} of stored block (tx, tz) next to (vx, vz) (reach 1), accepted only when the link
     * is the same seen from both ends: the target also sees the viewer through the seam, and the two
     * turns cancel. Next to a cube corner (a 270° cone point) a diagonal seen through the seam can fail
     * that, and is then refused from both ends alike, so Create's propagation cannot flicker. Null when
     * there is no such link.
     */
    public static int[] seamLink(WrapTopology t, int vx, int vz, int tx, int tz) {
        int[] q = nearestImage(t, vx, vz, tx, tz, 1);
        if (q == null)
            return null;
        int[] back = nearestImage(t, tx, tz, vx, vz, 1);
        if (back == null)
            return null;
        WrapTopology.Fold f = t.fold(q[0], q[1]), g = t.fold(back[0], back[1]);
        if (f == null || g == null || (f.iso().quarterTurns() + g.iso().quarterTurns()) % 4 != 0)
            return null;
        return q;
    }

    public static int[] nearestImage(WrapTopology t, int vx, int vz, int tx, int tz, int reach) {
        List<int[]> images = t.images(tx, tz);
        int[] best = null;
        int bestD = Integer.MAX_VALUE;
        for (int[] q : images) {
            int dx = Math.abs(q[0] - vx), dz = Math.abs(q[1] - vz);
            if (Math.max(dx, dz) > reach)
                continue;
            if (dx + dz < bestD) {
                bestD = dx + dz;
                best = q;
            }
        }
        return best;
    }
}
