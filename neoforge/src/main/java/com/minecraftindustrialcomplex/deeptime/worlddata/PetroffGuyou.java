// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * Petroff's equal-area square in the Guyou 2:1 layout: the seamless world (Ben, decisions log
 * 2026-10-04: "We'll proceed w/ Petroff–Guyou"; FORMAT.md "The world's layout"). The twin of
 * {@code deeptime_format::projection::PetroffGuyou} and of the pages' {@code DTPetroffGuyou}: the
 * same operations on {@link PortableMath}, so a block's latitude and longitude have the same bits in
 * the bake, the game and the workbench ({@code ProjectionParityTest}).
 *
 * <p>Two hemisphere squares of side b: x ∈ [−b, b), z ∈ [−b/2, b/2]. West and east slide onto each
 * other (period 2b, Toroidal World); the north edge z = −b/2 and the south edge z = b/2 are each
 * glued onto themselves by a half-turn about their midpoint (0, ∓b/2). The poles lie on the long
 * edges at x = ∓b/2, the four cone points at the edges' midpoints and the corners. The equator is
 * the line z = 0, north is −z along it.
 */
public final class PetroffGuyou implements WorldProjection {

    /** The hemisphere side is a multiple of this (Rust {@code HEMISPHERE_QUANTUM}; ledger: SCL-084, game-choice). */
    public static final int HEMISPHERE_QUANTUM = 256; // ledger: SCL-084 (game-choice)

    private final long b;
    private final double lambdaCDeg;
    private final Petroff pet = new Petroff();
    private final double[] a, e, f, a2, e2, f2;

    public PetroffGuyou(long b, double lambdaCDeg) {
        if (b <= 0 || b % 2 != 0)
            throw new IllegalArgumentException("hemisphere_blocks must be positive and even, got " + b);
        this.b = b;
        this.lambdaCDeg = lambdaCDeg;
        double le = pet.inverse(0.5, 0.5)[0];
        e = new double[]{PortableMath.cos(le), PortableMath.sin(le), 0.0};
        a = new double[]{0.0, 0.0, 1.0};
        f = new double[]{-e[1], e[0], 0.0};
        double lc = PortableMath.toRadians(lambdaCDeg);
        a2 = new double[]{PortableMath.cos(lc), PortableMath.sin(lc), 0.0};
        e2 = new double[]{0.0, 0.0, 1.0};
        f2 = new double[]{a2[1], -a2[0], 0.0};
    }

    /** b for a simulation circumference C at today's blocks per km²: C/√(2π) rounded to {@link #HEMISPHERE_QUANTUM}. */
    public static int hemisphereFor(int circumference) {
        double exact = circumference / Math.sqrt(2.0 * Math.PI);
        double q = HEMISPHERE_QUANTUM;
        return (int) (Math.max(1.0, Math.floor(exact / q + 0.5)) * q);   // Rust's round: half away from zero
    }

    @Override
    public Kind kind() {
        return Kind.PETROFF_GUYOU;
    }

    /** b, blocks. */
    public long hemisphere() {
        return b;
    }

    public double lambdaCDeg() {
        return lambdaCDeg;
    }

    @Override
    public double period() {
        return 2.0 * b;
    }

    @Override
    public double radius() {
        return b / Math.sqrt(2.0 * Math.PI);
    }

    @Override
    public double zMin() {
        return -b / 2.0;
    }

    @Override
    public double zMax() {
        return b / 2.0;
    }

    @Override
    public double wrapX(double x) {
        double p = period();
        double t = PortableMath.remEuclid(x + 0.5 * p, p);
        return (t >= p ? 0.0 : t) - 0.5 * p;
    }

    /**
     * Brings any point into the domain through the edges: x wraps; past the north or south edge the
     * half-turn about its midpoint. Returns {x, z, half-turns taken}.
     */
    public double[] fold(double x, double z) {
        double half = b / 2.0, bb = (double) b;
        int n = 0;
        for (int k = 0; k < 64; k++) {
            if (z < -half) {
                x = -x;
                z = -bb - z;
                n++;
            } else if (z > half) {
                x = -x;
                z = bb - z;
                n++;
            } else {
                break;
            }
        }
        return new double[]{wrapX(x), Math.max(-half, Math.min(half, z)), n};
    }

    @Override
    public double[] canonical(double x, double z) {
        double[] p = fold(x, z);
        return new double[]{p[0], p[1]};
    }

    @Override
    public boolean flipped(double x, double z) {
        return ((int) fold(x, z)[2] & 1) == 1;
    }

    /**
     * The block cell (bx, bz) brought into the domain, the wrap engine's cell arithmetic: x wraps into
     * [−b, b); a cell north of z = −b/2 is (−bx − 1, −b − bz − 1), south of z = b/2 (−bx − 1, b − bz − 1).
     * Returns {x, z, half-turns}.
     */
    @Override
    public long[] foldCell(long bx, long bz) {
        long half = b / 2;
        long x = bx, z = bz, n = 0;
        for (int k = 0; k < 64; k++) {
            if (z < -half) {
                x = -x - 1;
                z = -b - z - 1;
                n++;
            } else if (z >= half) {
                x = -x - 1;
                z = b - z - 1;
                n++;
            } else {
                break;
            }
        }
        long p = 2 * b;
        return new long[]{Math.floorMod(x + b, p) - b, z, n};
    }

    private static double dot3(double[] p, double[] q) {
        return p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
    }

    private static double[] unitPm(double lambda, double phi) {
        double sp = PortableMath.sin(phi), cp = PortableMath.cos(phi);
        return new double[]{cp * PortableMath.cos(lambda), cp * PortableMath.sin(lambda), sp};
    }

    private double[] polarToWorld(double[] v) {
        double pa = dot3(a, v), pe = dot3(e, v), pf = dot3(f, v);
        return new double[]{
                a2[0] * pa + e2[0] * pe + f2[0] * pf,
                a2[1] * pa + e2[1] * pe + f2[1] * pf,
                a2[2] * pa + e2[2] * pe + f2[2] * pf};
    }

    private double[] worldToPolar(double[] v) {
        double pa = dot3(a2, v), pe = dot3(e2, v), pf = dot3(f2, v);
        return new double[]{
                a[0] * pa + e[0] * pe + f[0] * pf,
                a[1] * pa + e[1] * pe + f[1] * pf,
                a[2] * pa + e[2] * pe + f[2] * pf};
    }

    /** The unit vector of any point (folded into the domain first). */
    @Override
    public double[] unit(double x, double z) {
        double[] p = fold(x, z);
        return unitInTile(p[0], p[1]);
    }

    /** The unit vector of a point of the closed domain drawn from its own tile, x not wrapped (the seam tests). */
    public double[] unitUnfolded(double x, double z) {
        double bb = (double) b, h = b / 2.0;
        return unitInTile(Math.max(-bb, Math.min(bb, x)), Math.max(-h, Math.min(h, z)));
    }

    private double[] unitInTile(double x, double z) {
        double bb = (double) b;
        double tx = 2.0 * x / bb + 1.0;
        double ty = -2.0 * z / bb;
        double u, v;
        boolean south;
        if (tx <= 1.0) {
            u = tx;
            v = ty;
            south = false;
        } else {
            u = 2.0 - tx;
            v = ty;
            south = true;
        }
        double[] ll = pet.inverse(0.5 * (u + v), 0.5 * (v - u));
        double phi = south ? -ll[1] : ll[1];
        return polarToWorld(unitPm(ll[0], phi));
    }

    /** The canonical point {x, z} of a unit vector (any length > 0). */
    public double[] pointOfUnit(double[] vec) {
        double[] q = worldToPolar(vec);
        double phi = PortableMath.atan2(q[2], Math.sqrt(q[0] * q[0] + q[1] * q[1]));
        double lam = PortableMath.atan2(q[1], q[0]);
        boolean south = phi < 0.0;
        double[] xy = pet.forward(lam, south ? -phi : phi);
        double u = xy[0] - xy[1], v = xy[0] + xy[1];
        double tx = south ? 2.0 - u : u;
        double bb = (double) b;
        double x = (tx - 1.0) * bb / 2.0;
        double z = -v * bb / 2.0;
        return new double[]{wrapX(x), Math.max(-bb / 2.0, Math.min(bb / 2.0, z))};
    }

    @Override
    public double[] latLon(double x, double z) {
        return WorldProjection.latLonOfUnit(unit(x, z));
    }

    @Override
    public double[] blockOf(double latDeg, double lonDeg) {
        double lat = PortableMath.toRadians(Math.max(-90.0, Math.min(90.0, latDeg)));
        double lon = PortableMath.toRadians(lonDeg);
        return pointOfUnit(unitPm(lon, lat));
    }

    @Override
    public GridDesc grid(String name, int r) {
        if (b % r != 0)
            throw new IllegalArgumentException("hemisphere " + b + " is not a multiple of r = " + r);
        return new GridDesc(name, (int) (2 * b / r), (int) (b / r + 1), r, true, -b, -b / 2, GridDesc.ZEdges.HALF_TURN);
    }

    @Override
    public long indexOriginBlockX() {
        return -b;
    }

    @Override
    public long indexOriginBlockZ() {
        return -b / 2;
    }

    @Override
    public long indexSpanBlockZ() {
        return b + 1;
    }

    @Override
    public String toString() {
        return "Petroff–Guyou world b = " + b + ", λc = " + lambdaCDeg + "°";
    }
}
