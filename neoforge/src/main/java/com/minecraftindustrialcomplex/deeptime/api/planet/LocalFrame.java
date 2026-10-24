// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.planet;

/**
 * The local frame at one position (see the package's contract): the first-order link between a
 * step in the block frame there and a step on the ground. Directions in the block frame are unit
 * vectors (dx, dz); geographic bearings are degrees clockwise from north on the ground; block-frame
 * bearings are degrees clockwise from −z seen from above. Exact in the limit of short steps.
 */
public final class LocalFrame {

    /** Ground metres {east, north} that one block of +x and one block of +z move. */
    private final double eastPerX, northPerX, eastPerZ, northPerZ, det;
    private final double northX, northZ, eastX, eastZ, northBearingDeg;
    private final double metresPerBlockEast, metresPerBlockNorth, metresPerBlock;

    /**
     * @param eastPerX        ground metres east per block of +x
     * @param northPerX       ground metres north per block of +x
     * @param eastPerZ        ground metres east per block of +z
     * @param northPerZ       ground metres north per block of +z
     * @param northX          north's unit vector in the block frame, x
     * @param northZ          north's unit vector in the block frame, z
     * @param northBearingDeg north's block-frame bearing
     * @param metresPerBlock  the square root of a block's ground area
     */
    LocalFrame(double eastPerX, double northPerX, double eastPerZ, double northPerZ,
               double northX, double northZ, double northBearingDeg, double metresPerBlock) {
        this.eastPerX = eastPerX;
        this.northPerX = northPerX;
        this.eastPerZ = eastPerZ;
        this.northPerZ = northPerZ;
        this.det = eastPerX * northPerZ - eastPerZ * northPerX;
        this.northX = northX;
        this.northZ = northZ;
        this.northBearingDeg = northBearingDeg;
        this.metresPerBlock = metresPerBlock;
        // One metre east and one metre north on the ground, in blocks (the inverse map's columns).
        double ex = northPerZ / det, ez = -northPerX / det;
        double nx = -eastPerZ / det, nz = eastPerX / det;
        double le = Math.hypot(ex, ez), ln = Math.hypot(nx, nz);
        this.eastX = ex / le;
        this.eastZ = ez / le;
        this.metresPerBlockEast = 1.0 / le;
        this.metresPerBlockNorth = 1.0 / ln;
    }

    /**
     * Where geographic north points in the block frame, as a block-frame bearing in [0, 360): 0 when
     * north is −z, 90 when it is +x, 180 when it is +z, 270 when it is −x.
     */
    public double northBearingDeg() {
        return northBearingDeg;
    }

    /** x of the unit vector in the block frame along which a short step goes due north. */
    public double northX() {
        return northX;
    }

    /** z of the unit vector in the block frame along which a short step goes due north. */
    public double northZ() {
        return northZ;
    }

    /** x of the unit vector in the block frame along which a short step goes due east. */
    public double eastX() {
        return eastX;
    }

    /** z of the unit vector in the block frame along which a short step goes due east. */
    public double eastZ() {
        return eastZ;
    }

    /** Ground metres one block along {@link #eastX()}, {@link #eastZ()} covers (due east). */
    public double metresPerBlockEast() {
        return metresPerBlockEast;
    }

    /** Ground metres one block along {@link #northX()}, {@link #northZ()} covers (due north). */
    public double metresPerBlockNorth() {
        return metresPerBlockNorth;
    }

    /** The square root of one block's ground area, metres: the scale to convert areas and densities with. */
    public double metresPerBlock() {
        return metresPerBlock;
    }

    /** Ground metres the block-frame step (dx, dz) covers. */
    public double metresOfStep(double dx, double dz) {
        return Math.hypot(dx * eastPerX + dz * eastPerZ, dx * northPerX + dz * northPerZ);
    }

    /** The geographic bearing in [0, 360) of the block-frame step (dx, dz); NaN for no step. */
    public double bearingOfStep(double dx, double dz) {
        double e = dx * eastPerX + dz * eastPerZ, n = dx * northPerX + dz * northPerZ;
        if (e == 0.0 && n == 0.0)
            return Double.NaN;
        double b = Math.toDegrees(Math.atan2(e, n));
        if (b < 0.0)
            b += 360.0;
        return b >= 360.0 ? 0.0 : b + 0.0;
    }

    /** The block-frame step that goes {@code metres} on the ground at geographic bearing {@code bearingDeg}. */
    public BlockXZ step(double bearingDeg, double metres) {
        double b = Math.toRadians(bearingDeg);
        double e = metres * Math.sin(b), n = metres * Math.cos(b);
        return new BlockXZ((e * northPerZ - n * eastPerZ) / det, (n * eastPerX - e * northPerX) / det);
    }

    @Override
    public String toString() {
        return String.format(java.util.Locale.ROOT, "LocalFrame[north %.3f° (%.5f, %.5f), east (%.5f, %.5f), %.4f m east, %.4f m north, %.4f m]",
                northBearingDeg, northX, northZ, eastX, eastZ, metresPerBlockEast, metresPerBlockNorth, metresPerBlock);
    }
}
