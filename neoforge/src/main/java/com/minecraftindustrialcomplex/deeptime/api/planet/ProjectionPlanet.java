// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.planet;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

import java.util.Optional;

/**
 * {@link DeepTimePlanet} over a world's {@link WorldProjection} and its nominal metres per block
 * (the manifest's {@code m_per_block_h}): every answer is the projection's own (its unit vectors,
 * latitude and longitude, canonical points, north and scale) plus sphere arithmetic on unit vectors.
 * This class is where the maps' differences stay: Lambert's cylinder answers past its poles with the
 * pole rather than NaN, so it is told off the planet there; the cube has an exact Jacobian; any
 * other map gets the Jacobian by central differences over half a block, as {@link
 * WorldProjection#north} does.
 */
final class ProjectionPlanet implements DeepTimePlanet {

    /** Half the central-difference step of the Jacobian, blocks: {@link WorldProjection#north}'s. */
    private static final double STEP = 0.5;
    /** Below this sine of the angle between two points, no single great circle joins them (radians). */
    private static final double DEGENERATE = 1e-12;

    private final WorldProjection p;
    private final double mPerBlockH;
    private final double radiusM;
    private final boolean lambert;

    ProjectionPlanet(WorldProjection p, double mPerBlockH) {
        this.p = p;
        this.mPerBlockH = mPerBlockH;
        this.radiusM = p.radius() * mPerBlockH; // ledger: SCL-240
        this.lambert = p instanceof WorldProjection.Lambert;
    }

    /** The projection answered over (tests). */
    WorldProjection projection() {
        return p;
    }

    /** The nominal metres per block (tests). */
    double mPerBlockH() {
        return mPerBlockH;
    }

    @Override
    public double radiusMetres() {
        return radiusM;
    }

    /** False when (x, z) shows no planet point before the projection is asked (it may still answer NaN). */
    private boolean maybeOn(double x, double z) {
        return Double.isFinite(x) && Double.isFinite(z) && (!lambert || p.onWorld(z));
    }

    /** The unit vector (x, z) shows in the simulation's frame, or null off the planet. */
    private double[] unit(double x, double z) {
        if (!maybeOn(x, z))
            return null;
        double[] v = p.unit(x, z);
        return Double.isNaN(v[0]) ? null : v;
    }

    /** {latitude, longitude} (x, z) shows, the longitude 0 at a pole, or null off the planet. */
    private double[] latLon(double x, double z) {
        if (!maybeOn(x, z))
            return null;
        double[] ll = p.latLon(x, z);
        if (Double.isNaN(ll[0]) || Double.isNaN(ll[1]))
            return null;
        return Math.abs(ll[0]) == 90.0 ? new double[]{ll[0], 0.0} : ll;
    }

    @Override
    public boolean isOnPlanet(double x, double z) {
        return maybeOn(x, z) && !Double.isNaN(p.canonical(x, z)[0]);
    }

    @Override
    public Optional<GeoPoint> geoPoint(double x, double z) {
        double[] ll = latLon(x, z);
        return ll == null ? Optional.empty() : Optional.of(new GeoPoint(ll[0], ll[1]));
    }

    @Override
    public double latitudeDeg(double x, double z) {
        double[] ll = latLon(x, z);
        return ll == null ? Double.NaN : ll[0];
    }

    @Override
    public double longitudeDeg(double x, double z) {
        double[] ll = latLon(x, z);
        return ll == null ? Double.NaN : ll[1];
    }

    @Override
    public BlockXZ blockPoint(GeoPoint point) {
        return blockPoint(point.latitudeDeg(), point.longitudeDeg());
    }

    private BlockXZ blockPoint(double latDeg, double lonDeg) {
        double[] b = p.blockOf(latDeg, lonDeg);
        // The projection's inverse already lands in its domain; canonical() settles a point that
        // rounding put a hair past an edge (it shows the same planet point either way).
        double[] c = p.canonical(b[0], b[1]);
        return Double.isNaN(c[0]) ? new BlockXZ(b[0], b[1]) : new BlockXZ(c[0], c[1]);
    }

    @Override
    public Optional<BlockXZ> canonical(double x, double z) {
        if (!maybeOn(x, z))
            return Optional.empty();
        double[] c = p.canonical(x, z);
        return Double.isNaN(c[0]) ? Optional.empty() : Optional.of(new BlockXZ(c[0], c[1]));
    }

    @Override
    public Optional<LocalFrame> frame(double x, double z) {
        double[] v = unit(x, z);
        if (v == null)
            return Optional.empty();
        double[][] j = jacobian(x, z, v);
        if (j == null)
            return Optional.empty();
        double det = j[0][0] * j[1][1] - j[1][0] * j[0][1];
        double bearing = p.northBearingDeg(x, z), scale = p.scale(x, z);
        if (det == 0.0 || !Double.isFinite(det) || !Double.isFinite(bearing) || !(scale > 0.0) || !Double.isFinite(scale))
            return Optional.empty();
        double[] n = p.north(x, z);
        double m = mPerBlockH;
        return Optional.of(new LocalFrame(j[0][0] * m, j[0][1] * m, j[1][0] * m, j[1][1] * m,
                n[0], n[1], bearing, m * scale)); // ledger: SCL-107
    }

    /**
     * The map's Jacobian at (x, z), planet blocks of the equal-area sphere: {{east, north} per block of
     * +x, {east, north} per block of +z}; null where the projection has none.
     */
    private double[][] jacobian(double x, double z, double[] v) {
        if (p instanceof WorldProjection.Cube cube) {
            CubeWorld.WorldJacobian w = cube.world().jacobian(x, z);   // exact, through the fold's turn
            return w == null ? null : w.m();
        }
        if (p instanceof WorldProjection.Lambert l) {
            // x = R λ + const, z = −R sin φ: east R cos φ dλ = cos φ per +x, north R dφ = −1 / cos φ per +z.
            double cos = WorldProjection.Lambert.cosLat(l.circumference(), z);
            return new double[][]{{cos, 0.0}, {0.0, -1.0 / cos}};
        }
        double[][] fr = WorldProjection.localFrame(v);
        double rr = p.radius();
        return new double[][]{column(x, z, STEP, 0.0, fr, rr), column(x, z, 0.0, STEP, fr, rr)};
    }

    /** {east, north} planet blocks per block along (dx, dz) (one of them 0), by central differences in the frame {@code fr}. */
    private double[] column(double x, double z, double dx, double dz, double[][] fr, double rr) {
        double h = dx != 0.0 ? dx : dz;
        double[] a = p.unit(x + dx, z + dz), b = p.unit(x - dx, z - dz);
        double[] d = {a[0] - b[0], a[1] - b[1], a[2] - b[2]};
        return new double[]{dot(d, fr[0]) * rr / (2.0 * h), dot(d, fr[1]) * rr / (2.0 * h)};
    }

    @Override
    public double distanceMetres(double x0, double z0, double x1, double z1) {
        double[] a = unit(x0, z0), b = unit(x1, z1);
        return a == null || b == null ? Double.NaN : angle(a, b) * radiusM;
    }

    @Override
    public double distanceMetres(GeoPoint from, GeoPoint to) {
        return angle(unitOf(from), unitOf(to)) * radiusM;
    }

    @Override
    public double initialBearingDeg(double x0, double z0, double x1, double z1) {
        double[] a = unit(x0, z0), b = unit(x1, z1);
        return a == null || b == null ? Double.NaN : bearing(a, b);
    }

    @Override
    public double initialBearingDeg(GeoPoint from, GeoPoint to) {
        return bearing(unitOf(from), unitOf(to));
    }

    @Override
    public Optional<BlockXZ> destination(double x, double z, double bearingDeg, double metres) {
        double[] a = unit(x, z);
        if (a == null || !Double.isFinite(bearingDeg) || !Double.isFinite(metres))
            return Optional.empty();
        double[] ll = WorldProjection.latLonOfUnit(travel(a, bearingDeg, metres / radiusM));
        return Optional.of(blockPoint(ll[0], ll[1]));
    }

    @Override
    public GeoPoint destination(GeoPoint from, double bearingDeg, double metres) {
        if (!Double.isFinite(bearingDeg) || !Double.isFinite(metres))
            throw new IllegalArgumentException("bearing " + bearingDeg + " and distance " + metres + " must be finite");
        double[] ll = WorldProjection.latLonOfUnit(travel(unitOf(from), bearingDeg, metres / radiusM));
        return new GeoPoint(Math.max(-90.0, Math.min(90.0, ll[0])), ll[1]);
    }

    @Override
    public String toString() {
        return "DeepTimePlanet[R " + radiusM + " m, " + p.kind().id() + "]";
    }

    // ---- the sphere ------------------------------------------------------------------------

    /** The unit vector of a planet point in the simulation's frame (WorldProjection.latLonOfUnit's inverse). */
    private static double[] unitOf(GeoPoint g) {
        double phi = Math.toRadians(g.latitudeDeg()), lam = Math.toRadians(g.longitudeDeg());
        double c = Math.cos(phi);
        return new double[]{c * Math.cos(lam), c * Math.sin(lam), Math.sin(phi)};
    }

    private static double dot(double[] a, double[] b) {
        return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }

    private static double[] cross(double[] a, double[] b) {
        return new double[]{a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]};
    }

    /** The angle between two unit vectors, radians: atan2(|a × b|, a · b), exact near and far. */
    private static double angle(double[] a, double[] b) {
        double[] c = cross(a, b);
        return Math.atan2(Math.sqrt(dot(c, c)), dot(a, b));
    }

    /**
     * The geographic bearing at a of the great circle to b: the direction (a × b) × a in a's east and
     * north ({@link WorldProjection#localFrame}, whose north at a pole is the limit along longitude 0).
     */
    private static double bearing(double[] a, double[] b) {
        double[] c = cross(a, b);
        if (Math.sqrt(dot(c, c)) < DEGENERATE)
            return Double.NaN;
        double[] t = cross(c, a);
        double[][] fr = WorldProjection.localFrame(a);
        double deg = Math.toDegrees(Math.atan2(dot(t, fr[0]), dot(t, fr[1])));
        if (deg < 0.0)
            deg += 360.0;
        return deg >= 360.0 ? 0.0 : deg + 0.0;
    }

    /** The unit vector reached from a by an arc of {@code angle} radians leaving at geographic bearing {@code bearingDeg}. */
    private static double[] travel(double[] a, double bearingDeg, double angle) {
        double[][] fr = WorldProjection.localFrame(a);
        double b = Math.toRadians(bearingDeg), cb = Math.cos(b), sb = Math.sin(b), cd = Math.cos(angle), sd = Math.sin(angle);
        double[] v = new double[3];
        for (int i = 0; i < 3; i++)
            v[i] = cd * a[i] + sd * (cb * fr[1][i] + sb * fr[0][i]);
        double n = Math.sqrt(dot(v, v));
        return new double[]{v[0] / n, v[1] / n, v[2] / n};
    }
}
