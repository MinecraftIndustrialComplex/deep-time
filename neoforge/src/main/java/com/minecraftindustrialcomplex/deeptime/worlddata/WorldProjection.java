// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * The projection a world's data is laid out in (manifest {@code projection}, FORMAT.md "The world's
 * layout"): the cube world ({@link Cube}, format 0.7, the only world the game plays) or Lambert's
 * cylinder ({@link Lambert}: the simulation's frame, in which the synthetic test worlds are laid out).
 * The twin of Rust's {@code deeptime_format::projection::WorldProjection}. Every reader that turns a
 * block into a latitude and longitude, or the other way, goes through this.
 */
public interface WorldProjection {

    /** {@code projection.kind}. */
    enum Kind {
        LAMBERT("lambert"),
        /**
         * The cube world (docs/plans/cube-world-plan.md, format 0.7): six faces in a net, {@link CubeLayout},
         * {@link CubeMap}, {@link CubeWorld}; the projection {@link Cube}.
         */
        CUBE("cube");

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

    /** The world's z range: the poles (Lambert) or the net's window (cube). */
    double zMin();

    double zMax();

    /** x wrapped into [−period/2, period/2). */
    double wrapX(double x);

    /** The canonical point {x, z} of any point (x wrapped; Lambert clamps z to the poles, the cube folds it). */
    double[] canonical(double x, double z);

    /** Whether the point's canonical image is turned half round (a half-turn edge; never on Lambert). */
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
     * The unit vector of geographic north in block space at (x, z), {dx, dz}: from the local Jacobian
     * over half a block (each world overrides it with its exact answer).
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

    /**
     * The bearing of geographic north at (x, z) in the block frame ({@link #north}), degrees clockwise
     * from the block frame's north (−z) seen from above, in [0, 360): 0 where north is −z (everywhere on
     * a Lambert world), 90 where it is +x, 180 where it is +z. NaN off the planet.
     */
    default double northBearingDeg(double x, double z) {
        double[] n = north(x, z);
        return bearingDeg(n[0], n[1]);
    }

    /**
     * The local linear scale at (x, z): blocks of the equal-area sphere ({@link #radius}) per world
     * block, √|det J| of the map's Jacobian. 1 on the equal-area Lambert world (a block's area is the
     * same everywhere there, though it is longer one way and shorter the other); a cube world's conformal map has one scale in every direction, which varies across each
     * face. Times the manifest's {@code m_per_block_h} it is metres per block. NaN off the planet.
     */
    default double scale(double x, double z) {
        return 1.0;
    }

    /**
     * The great-circle distance between (x0, z0) and (x1, z1) on the planet, in blocks of the
     * equal-area sphere ({@link #radius}; times {@code m_per_block_h} it is metres, SCL-107): the angle
     * between their unit vectors, atan2(|a × b|, a · b), which keeps near and far points exact. NaN
     * when either point is off the planet.
     */
    default double greatCircleBlocks(double x0, double z0, double x1, double z1) {
        double[] a = unit(x0, z0), b = unit(x1, z1);
        if (Double.isNaN(a[0]) || Double.isNaN(b[0]))
            return Double.NaN;
        double cx = a[1] * b[2] - a[2] * b[1], cy = a[2] * b[0] - a[0] * b[2], cz = a[0] * b[1] - a[1] * b[0];
        return Math.atan2(Math.sqrt(cx * cx + cy * cy + cz * cz), a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) * radius();
    }

    /** The bearing in degrees [0, 360) of the block-frame direction (dx, dz), clockwise from −z seen from above. */
    static double bearingDeg(double dx, double dz) {
        double b = Math.toDegrees(Math.atan2(dx, -dz));
        if (b < 0.0)
            b += 360.0;
        return b >= 360.0 ? 0.0 : b;
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

    /**
     * The projection a manifest's layout block describes; null (no block) is Lambert of circumference {@code c}.
     *
     * @throws WorldDataException on a cube layout whose map this jar does not ship (by name and sha256) or whose
     *                            rotation is not one (Rust {@code WorldProjection::try_from_spec})
     */
    static WorldProjection of(WorldLayout layout, int c) {
        if (layout != null && layout.kind() == Kind.CUBE)
            return Cube.of(layout);
        return new Lambert(c);
    }

    /**
     * A cube world (format 0.7, FORMAT.md "The cube world's layout"): the net {@link CubeLayout} of face
     * side s, drawn on the planet through {@link CubeWorld} at one block a block (the radius in blocks of
     * the equal-area sphere, as Rust's {@code WorldProjection::Cube}, so a Jacobian is planet blocks per
     * net block). x wraps with period 4s into the window [−3s/2, 5s/2), z spans [−3s/2, 3s/2]; a point
     * off the faces is folded through the nearest net edge, and past the net's corners every answer is
     * NaN.
     */
    record Cube(CubeWorld world) implements WorldProjection {

        /**
         * The cube world a layout block describes (Rust {@code WorldProjection::try_from_spec}).
         *
         * @throws WorldDataException when the map table is not one this jar ships, by name and sha256,
         *                            or the rotation is not a rotation
         */
        public static Cube of(WorldLayout layout) {
            WorldLayout.CubeSpec spec = layout.cube();
            if (spec == null)
                throw new WorldDataException("cube layout without face_blocks");
            return new Cube(new CubeWorld(CubeLayout.shared(spec.faceBlocks()), mapNamed(spec.mapName(), spec.mapSha256()),
                    CubeTurn.fromRows(spec.rotation()), 1.0));
        }

        /**
         * The shipped cube map {@code name}, checked against {@code sha256} (lowercase hex). The refusal is
         * FORMAT.md's and Rust's {@code cube_map_named}: "world data needs cube map NAME (sha256 HASH); this
         * Deep Time has NAMES".
         */
        public static CubeMap mapNamed(String name, String sha256) {
            CubeMap m = CubeMap.builtin(name);
            if (m != null && m.sha256Hex().equals(sha256.toLowerCase(java.util.Locale.ROOT)))
                return m;
            StringBuilder has = new StringBuilder();
            for (String n : CubeMap.builtinNames()) {
                if (!has.isEmpty())
                    has.append(", ");
                has.append(n).append(" (sha256 ").append(CubeMap.builtin(n).sha256Hex()).append(')');
            }
            throw new WorldDataException("world data needs cube map " + name + " (sha256 " + sha256 + "); this Deep Time has " + has);
        }

        /** The net. */
        public CubeLayout cube() {
            return world.cube();
        }

        /** s, the face side in blocks. */
        public long side() {
            return world.cube().side();
        }

        @Override
        public Kind kind() {
            return Kind.CUBE;
        }

        @Override
        public double period() {
            return (double) cube().period();
        }

        @Override
        public double radius() {
            return cube().radius();
        }

        @Override
        public double zMin() {
            return cube().window()[2];
        }

        @Override
        public double zMax() {
            return cube().window()[3];
        }

        /** x wrapped into the window [−3s/2, 5s/2) (not the Lambert frame's [−period/2, period/2)). */
        @Override
        public double wrapX(double x) {
            return cube().wrapXPoint(x);
        }

        @Override
        public double[] canonical(double x, double z) {
            CubeLayout.PointOnFace p = cube().foldPoint(x, z);
            return p == null ? new double[]{Double.NaN, Double.NaN} : new double[]{p.x(), p.z()};
        }

        /** Whether the fold turns the point half round (a half-turn edge); a quarter turn is not a flip. */
        @Override
        public boolean flipped(double x, double z) {
            CubeLayout.PointOnFace p = cube().foldPoint(x, z);
            return p != null && p.iso().k() == 2;
        }

        /**
         * The block brought onto the planet ({@link CubeLayout#fold}): {x, z, quarter turns} (on a cube the
         * third number counts quarter turns, not half-turns); a block past the net's corners is returned
         * x-wrapped with 0.
         */
        @Override
        public long[] foldCell(long bx, long bz) {
            CubeLayout.Fold f = cube().fold(bx, bz);
            if (f instanceof CubeLayout.Outside o)
                return new long[]{o.x(), o.z(), o.iso().k()};
            return new long[]{cube().wrapX(bx), bz, 0};
        }

        @Override
        public double[] unit(double x, double z) {
            double[] v = world.unit(x, z);
            return v == null ? new double[]{Double.NaN, Double.NaN, Double.NaN} : v;
        }

        @Override
        public double[] latLon(double x, double z) {
            double[] ll = world.latLon(x, z);
            return ll == null ? new double[]{Double.NaN, Double.NaN} : ll;
        }

        @Override
        public double[] blockOf(double latDeg, double lonDeg) {
            return world.pointOfLatLon(latDeg, lonDeg);
        }

        @Override
        public GridDesc grid(String name, int r) {
            return GridDesc.forCube(name, side(), r);
        }

        @Override
        public long indexOriginBlockX() {
            return -3 * side() / 2;
        }

        @Override
        public long indexOriginBlockZ() {
            return -3 * side() / 2;
        }

        @Override
        public long indexSpanBlockZ() {
            return 3 * side();
        }

        /** Geographic north through the map's Jacobian and the fold's turn ({@link CubeWorld#north}); (0, −1) past the corners. */
        @Override
        public double[] north(double x, double z) {
            double[] n = world.north(x, z);
            return n == null ? new double[]{0.0, -1.0} : n;
        }

        /**
         * North's bearing through the fold's turn: smooth from a face on into the copy past its edge, where
         * the same planet point on its own face reads it turned by the edge's motion. NaN past the corners.
         */
        @Override
        public double northBearingDeg(double x, double z) {
            double[] n = world.north(x, z);
            return n == null ? Double.NaN : WorldProjection.bearingDeg(n[0], n[1]);
        }

        /** The conformal map's local scale through the fold ({@link CubeWorld.WorldJacobian#sigma}); NaN past the corners. */
        @Override
        public double scale(double x, double z) {
            CubeWorld.WorldJacobian j = world.jacobian(x, z);
            return j == null ? Double.NaN : j.sigma();
        }
    }

    /**
     * Lambert's cylindrical equal-area world of circumference C: the simulation's frame (decisions log
     * 2026-10-02, "Lambert for now"; ledger: SCL-049, the equal-area property SCL-052), in which the
     * synthetic test worlds are laid out. The game plays only the cube world (decisions log 2026-10-24,
     * "Cube only"); this frame goes with the Lambert simulation grid (cube plan §7.4 item 2).
     * <ul>
     *   <li>A planet of circumference C blocks has radius R = C / 2π blocks.</li>
     *   <li>Longitude λ = 360°·(x + C/2)/C in [0°, 360°): the seam x = −C/2 is 0°, x wraps.</li>
     *   <li>Latitude φ = asin(−z / R), north (−z) positive; the poles are at z = ∓R and φ clamps to
     *       ±90° beyond them.</li>
     *   <li>Grids run from z = −Zo to Zo ({@link #gridHalfExtent}: the smallest multiple of the
     *       quantum at or past the pole); so do the spatial indexes.</li>
     * </ul>
     * The formulas are Rust's {@code deeptime_format::Projection}'s, in the same order;
     * {@code ColumnParityTest} checks both give the same latitudes, z of latitudes and pole.
     */
    record Lambert(int circumference) implements WorldProjection {

        /** The largest quantum of the grids' z extent, blocks (Rust {@code QUANTUM_MAX}). */
        public static final int QUANTUM_MAX = 64;

        /** The planet's radius R = C / 2π, blocks (true scale on the equator). */
        public static double radius(double circumference) {
            return circumference / (2 * Math.PI);
        }

        /** sin φ of block z, clamped to [−1, 1] (north is −z). */
        public static double sinLat(double circumference, double z) {
            return Math.max(-1.0, Math.min(1.0, -z / radius(circumference)));
        }

        /** cos φ of block z (0 at and beyond the poles). */
        public static double cosLat(double circumference, double z) {
            double s = sinLat(circumference, z);
            return Math.sqrt(Math.max(0.0, 1.0 - s * s));
        }

        /**
         * The quantum the grids' z extent is a multiple of: the largest power of two up to
         * {@link #QUANTUM_MAX} that divides C/2 and is at most R/2 (64 for every planet size).
         */
        public static int quantum(double circumference) {
            long half = Math.round(circumference / 2.0);
            double r = radius(circumference);
            int q = QUANTUM_MAX;
            while (q > 1 && (half % q != 0 || q > r / 2.0))
                q /= 2;
            return q;
        }

        /**
         * The half-extent Zo of the planet's grids: the smallest multiple of the quantum at or past the
         * pole, the same for every grid (Rust {@code Projection::grid_half_extent}).
         */
        public static long gridHalfExtent(double circumference) {
            double q = quantum(circumference);
            return (long) (Math.ceil(radius(circumference) / q) * q);
        }

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
            return radius(circumference);
        }

        /** The pole z = −R. */
        @Override
        public double zMin() {
            return -radius(circumference);
        }

        /** The pole z = R. */
        @Override
        public double zMax() {
            return radius(circumference);
        }

        /** x wrapped into [−C/2, C/2). */
        @Override
        public double wrapX(double x) {
            double c = circumference;
            double t = ((x + 0.5 * c) % c + c) % c;
            return (t >= c ? 0.0 : t) - 0.5 * c;
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

        /**
         * Latitude asin(−z / R), clamped to ±90° past the poles; longitude 360°·(x + C/2)/C in [0, 360),
         * the seam x = −C/2 at 0°.
         */
        @Override
        public double[] latLon(double x, double z) {
            double lat = Math.toDegrees(Math.asin(sinLat(circumference, z)));
            double c = circumference;
            double lon = 360.0 * (wrapX(x) + 0.5 * c) / c;
            double w = ((lon % 360.0) + 360.0) % 360.0;
            return new double[]{lat, w >= 360.0 ? 0.0 : w};
        }

        /** x of the longitude east of the seam, in [−C/2, C/2); z = −R sin φ, φ clamped to ±90°. */
        @Override
        public double[] blockOf(double latDeg, double lonDeg) {
            double c = circumference;
            double l = ((lonDeg % 360.0) + 360.0) % 360.0;
            double x = wrapX(l / 360.0 * c - 0.5 * c);
            double z = -radius(c) * Math.sin(Math.toRadians(Math.max(-90.0, Math.min(90.0, latDeg))));
            return new double[]{x, z};
        }

        @Override
        public GridDesc grid(String name, int r) {
            long zo = gridHalfExtent(circumference);
            long oz = -zo;
            return new GridDesc(name, circumference / r, (int) ((2 * zo + r - 1) / r), r, true, -(circumference / 2), oz, GridDesc.ZEdges.CLAMP);
        }

        /** −C/2, the seam (lookup frames, not geography; Rust {@code Projection::index_origin}). */
        @Override
        public long indexOriginBlockX() {
            return -(circumference / 2);
        }

        /** −Zo, the grids' first row. */
        @Override
        public long indexOriginBlockZ() {
            return -gridHalfExtent(circumference);
        }

        /** 2·Zo. */
        @Override
        public long indexSpanBlockZ() {
            return 2 * gridHalfExtent(circumference);
        }

        @Override
        public double[] north(double x, double z) {
            return new double[]{0.0, -1.0};
        }
    }
}
