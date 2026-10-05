// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * The projection a world's data is laid out in (manifest {@code projection}, FORMAT.md "The world's
 * layout"): Lambert's cylinder ({@link Lambert}, the formulas of {@link Projection}) or the seamless
 * Petroff–Guyou world ({@link PetroffGuyou}, the default from format 0.5). The twin of Rust's
 * {@code deeptime_format::projection::WorldProjection}. Every reader that turns a block into a
 * latitude and longitude, or the other way, goes through this.
 */
public interface WorldProjection {

    /** {@code projection.kind}. */
    enum Kind {
        LAMBERT("lambert"),
        PETROFF_GUYOU("petroff_guyou");

        private final String id;

        Kind(String id) {
            this.id = id;
        }

        /** The setting's text. */
        public String id() {
            return id;
        }

        /** The kind named {@code s}, or null. */
        public static Kind parse(String s) {
            for (Kind k : values())
                if (k.id.equals(s))
                    return k;
            return null;
        }
    }

    Kind kind();

    /** The world's east–west period, blocks ({@code scale.circumference_blocks}; Toroidal World's loop). */
    double period();

    /** The radius of the equal-area sphere the world's blocks stand for, blocks. */
    double radius();

    /** The world's z range: the poles (Lambert) or the turning edges (Petroff–Guyou). */
    double zMin();

    double zMax();

    /** x wrapped into [−period/2, period/2). */
    double wrapX(double x);

    /** The canonical point {x, z} of any point (x wrapped; Lambert clamps z to the poles, Petroff–Guyou folds it). */
    double[] canonical(double x, double z);

    /** Whether the point's canonical image is turned half round (an odd number of half-turns; never on Lambert). */
    boolean flipped(double x, double z);

    /** The block cell brought into the domain, {x, z, half-turns} (Lambert: x wrapped, z unchanged). */
    long[] foldCell(long bx, long bz);

    /** The unit vector of the point (x, z), in the simulation's frame. */
    double[] unit(double x, double z);

    /** {latitude, longitude} of the point (x, z), degrees; longitude in [0, 360) in the simulation's frame. */
    double[] latLon(double x, double z);

    /** The canonical point {x, z} of a latitude and longitude, degrees. */
    double[] blockOf(double latDeg, double lonDeg);

    /** The world grid of {@code r} blocks a cell. */
    GridDesc grid(String name, int r);

    /** Block x of the spatial indexes' first cell (river tiles, structure buckets, site index). */
    long indexOriginBlockX();

    /** Block z of the spatial indexes' first cell. */
    long indexOriginBlockZ();

    /** Blocks the spatial indexes span north–south (east–west they span the period). */
    long indexSpanBlockZ();

    /** Whether the point is in the world's z range (x wraps). */
    default boolean onWorld(double z) {
        return z >= zMin() && z <= zMax();
    }

    /** Latitude, degrees, of the point (x, z). */
    default double latitudeDeg(double x, double z) {
        return latLon(x, z)[0];
    }

    /** Longitude, degrees in [0, 360), of the point (x, z). */
    default double longitudeDeg(double x, double z) {
        return latLon(x, z)[1];
    }

    /**
     * The unit vector of geographic north in block space at (x, z), {dx, dz} (Rust
     * {@code PetroffGuyou::north}: from the local Jacobian over half a block, through the fold).
     */
    default double[] north(double x, double z) {
        double h = 0.5;
        double[] v0 = unit(x, z);
        double[][] fr = localFrame(v0);
        double[] east = fr[0], north = fr[1];
        double rr = radius();
        double[] jx = jacobianColumn(x, z, h, 0.0, east, north, rr);
        double[] jz = jacobianColumn(x, z, 0.0, h, east, north, rr);
        double det = jx[0] * jz[1] - jz[0] * jx[1];
        if (det == 0.0 || !Double.isFinite(det))
            return new double[]{0.0, -1.0};
        double dx = -jz[0] / det;
        double dz = jx[0] / det;
        double n = Math.hypot(dx, dz);
        return new double[]{dx / n, dz / n};
    }

    private double[] jacobianColumn(double x, double z, double dx, double dz, double[] east, double[] north, double rr) {
        double h = dx != 0 ? dx : dz;
        double[] p = unit(x + dx, z + dz);
        double[] m = unit(x - dx, z - dz);
        double[] dv = {p[0] - m[0], p[1] - m[1], p[2] - m[2]};
        return new double[]{
                (dv[0] * east[0] + dv[1] * east[1] + dv[2] * east[2]) * rr / (2.0 * h),
                (dv[0] * north[0] + dv[1] * north[1] + dv[2] * north[2]) * rr / (2.0 * h)};
    }

    /** {east, north} unit vectors at the unit vector v (east (0, 1, 0) at the poles). */
    static double[][] localFrame(double[] v) {
        double h = Math.sqrt(v[0] * v[0] + v[1] * v[1]);
        double[] east = h > 1e-12 ? new double[]{-v[1] / h, v[0] / h, 0.0} : new double[]{0.0, 1.0, 0.0};
        double[] north = {
                v[1] * east[2] - v[2] * east[1],
                v[2] * east[0] - v[0] * east[2],
                v[0] * east[1] - v[1] * east[0]};
        return new double[][]{east, north};
    }

    /** {latitude, longitude} of a unit vector, degrees, on the portable functions; longitude in [0, 360), 0 at the poles. */
    static double[] latLonOfUnit(double[] v) {
        double h = Math.sqrt(v[0] * v[0] + v[1] * v[1]);
        double lat = PortableMath.toDegrees(PortableMath.atan2(v[2], h));
        double lon = h == 0.0 ? 0.0 : PortableMath.wrapDeg(PortableMath.toDegrees(PortableMath.atan2(v[1], v[0])));
        return new double[]{lat, lon};
    }

    /** The projection a manifest's layout block describes; null (no block) is Lambert of circumference {@code c}. */
    static WorldProjection of(WorldLayout layout, int c) {
        if (layout != null && layout.kind() == Kind.PETROFF_GUYOU)
            return new PetroffGuyou(layout.hemisphereBlocks() > 0 ? layout.hemisphereBlocks() : c / 2, layout.lambdaCDeg());
        return new Lambert(c);
    }

    /** Lambert's cylindrical equal-area world of circumference C ({@link Projection}'s formulas). */
    record Lambert(int circumference) implements WorldProjection {

        @Override
        public Kind kind() {
            return Kind.LAMBERT;
        }

        @Override
        public double period() {
            return circumference;
        }

        @Override
        public double radius() {
            return Projection.radius(circumference);
        }

        @Override
        public double zMin() {
            return -Projection.poleZ(circumference);
        }

        @Override
        public double zMax() {
            return Projection.poleZ(circumference);
        }

        @Override
        public double wrapX(double x) {
            return Projection.wrapX(circumference, x);
        }

        @Override
        public double[] canonical(double x, double z) {
            return new double[]{wrapX(x), Math.max(zMin(), Math.min(zMax(), z))};
        }

        @Override
        public boolean flipped(double x, double z) {
            return false;
        }

        @Override
        public long[] foldCell(long bx, long bz) {
            long half = circumference / 2;
            return new long[]{Math.floorMod(bx + half, (long) circumference) - half, bz, 0};
        }

        @Override
        public double[] unit(double x, double z) {
            double[] ll = latLon(x, z);
            double lat = Math.toRadians(ll[0]), lon = Math.toRadians(ll[1]);
            return new double[]{Math.cos(lat) * Math.cos(lon), Math.cos(lat) * Math.sin(lon), Math.sin(lat)};
        }

        @Override
        public double[] latLon(double x, double z) {
            return new double[]{Projection.latitudeDeg(circumference, z), Projection.longitudeDeg(circumference, x)};
        }

        @Override
        public double[] blockOf(double latDeg, double lonDeg) {
            return new double[]{Projection.xOfLongitude(circumference, lonDeg), Projection.zOfLatitude(circumference, latDeg)};
        }

        @Override
        public GridDesc grid(String name, int r) {
            long zo = Projection.gridHalfExtent(circumference, r);
            long oz = -zo;
            return new GridDesc(name, circumference / r, (int) ((2 * zo + r - 1) / r), r, true, -(circumference / 2), oz, GridDesc.ZEdges.CLAMP);
        }

        @Override
        public long indexOriginBlockX() {
            return Projection.indexOriginBlockX(circumference);
        }

        @Override
        public long indexOriginBlockZ() {
            return Projection.indexOriginBlockZ(circumference);
        }

        @Override
        public long indexSpanBlockZ() {
            return 2 * Projection.gridHalfExtent(circumference, 1);
        }

        @Override
        public double[] north(double x, double z) {
            return new double[]{0.0, -1.0};
        }
    }
}
