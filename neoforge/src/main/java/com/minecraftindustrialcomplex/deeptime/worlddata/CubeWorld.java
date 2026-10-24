// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * A cube world: the net ({@link CubeLayout}), the map table ({@link CubeMap}), the turn that places
 * the cube on the planet ({@link CubeTurn}) and the planet's size. Latitude, the poles and every
 * metres-per-block figure come from here, never from the net alone (coordinator, 2026-10-22: the
 * world's turn is free). The twin of Rust's {@code deeptime_format::cube_map::CubeWorld}, bit for
 * bit. It is not a {@link WorldProjection} yet (phase 2 of docs/plans/cube-world-plan.md adapts it).
 */
public final class CubeWorld {

    /**
     * The planet's metres per block at a point of the cube world ({@link #jacobian}).
     *
     * @param m     {@code [[east, north] per block of net +x, [east, north] per block of net +z]},
     *              planet metres; for the conformal map
     *              a scalar times a rotation and a reflection (net +z is south-ish)
     * @param sigma √|det m|: planet metres per block (the conformal map's one scale; for another map
     *              the geometric mean of its two)
     * @param angle the direction of net +x on the planet, radians from east toward north
     *              (atan2(m[0][1], m[0][0]), the portable atan2)
     */
    public record WorldJacobian(double[][] m, double sigma, double angle) {}

    private final CubeLayout cube;
    private final CubeMap map;
    private final CubeTurn turn;
    private final double radiusM;

    /**
     * The world of net {@code cube}, map {@code map} and turn {@code turn}, at {@code mPerBlockH}
     * nominal metres per block: the planet's radius in metres is mPerBlockH × {@link CubeLayout#radius}
     * (the six faces' area is the sphere's).
     */
    public CubeWorld(CubeLayout cube, CubeMap map, CubeTurn turn, double mPerBlockH) {
        this.cube = cube;
        this.map = map;
        this.turn = turn;
        this.radiusM = mPerBlockH * cube.radius(); // ledger: SCL-107
    }

    public CubeLayout cube() {
        return cube;
    }

    public CubeMap map() {
        return map;
    }

    public CubeTurn turn() {
        return turn;
    }

    /** The planet's radius, metres. */
    public double radiusM() {
        return radiusM;
    }

    /** The planet unit vector of a face's local point. */
    public double[] unitOnFace(CubeLayout.Face face, double u, double v) {
        return turn.toPlanet(map.forward(face, u, v));
    }

    /**
     * The planet unit vector the net point (x, z) shows (through {@link CubeLayout#foldPoint}: x
     * wraps, a point beyond a net edge shows the point it is glued to); null past the net's corners.
     */
    public double[] unit(double x, double z) {
        CubeLayout.PointOnFace p = cube.foldPoint(x, z);
        if (p == null)
            return null;
        return unitOnFace(p.face(), p.u(), p.v());
    }

    /** The face and local point of the planet unit vector {@code p}. */
    public CubeLayout.LocalPoint localOfUnit(double[] p) {
        return map.inverse(turn.toCube(p));
    }

    /** The net point (in the window) of the planet unit vector {@code p}: {x, z}. */
    public double[] pointOfUnit(double[] p) {
        CubeLayout.LocalPoint l = localOfUnit(p);
        return cube.pointOfLocal(l.face(), l.u(), l.v());
    }

    /** {latitude, longitude} in degrees of the net point (x, z), longitude in [0, 360); null past the net's corners. */
    public double[] latLon(double x, double z) {
        double[] v = unit(x, z);
        return v == null ? null : WorldProjection.latLonOfUnit(v);
    }

    /** The net point of (latitude, longitude), degrees: {x, z}. */
    public double[] pointOfLatLon(double latDeg, double lonDeg) {
        return pointOfUnit(unitOfLatLon(latDeg, lonDeg));
    }

    /** Metres per block at a face's local point ({@link WorldJacobian}). */
    public WorldJacobian jacobianOnFace(CubeLayout.Face face, double u, double v) {
        CubeMap.FaceJacobian j = map.jacobian(face, u, v);
        double[] p = turn.toPlanet(j.p());
        double[] du = turn.toPlanet(j.du()), dv = turn.toPlanet(j.dv());
        double[][] fr = WorldProjection.localFrame(p);
        double[] east = fr[0], north = fr[1];
        // du/dx = dv/dz = 2/s.
        double k = radiusM / ((double) cube.side() / 2.0);
        double[][] m = {
                {dot(du, east) * k, dot(du, north) * k},
                {dot(dv, east) * k, dot(dv, north) * k}};
        return worldJacobian(m);
    }

    /**
     * Metres per block at the net point (x, z), through the fold (beyond a net edge the motion's turn
     * is applied, so the rows stay per block of the queried point's +x and +z); null past the net's
     * corners.
     */
    public WorldJacobian jacobian(double x, double z) {
        CubeLayout.PointOnFace p = cube.foldPoint(x, z);
        if (p == null)
            return null;
        WorldJacobian j = jacobianOnFace(p.face(), p.u(), p.v());
        if (p.iso().k() == 0)
            return j;
        double[][] jm = j.m();
        double[] r0 = row(jm, p.iso(), 1.0, 0.0);
        double[] r1 = row(jm, p.iso(), 0.0, 1.0);
        return worldJacobian(new double[][]{r0, r1});
    }

    private static double[] row(double[][] jm, CubeLayout.Iso iso, double dx, double dz) {
        double[] v = iso.vector(dx, dz);
        return new double[]{
                jm[0][0] * v[0] + jm[1][0] * v[1],
                jm[0][1] * v[0] + jm[1][1] * v[1]};
    }

    /** The unit vector of north in block space at the net point (x, z), {dx, dz}; null past the net's corners. */
    public double[] north(double x, double z) {
        WorldJacobian w = jacobian(x, z);
        if (w == null)
            return null;
        double[][] j = w.m();
        // Solve J·(dx, dz) = (0, 1): dx·j[0] + dz·j[1] = (east 0, north 1).
        double det = j[0][0] * j[1][1] - j[1][0] * j[0][1];
        if (det == 0.0 || !Double.isFinite(det))
            return new double[]{0.0, -1.0};
        double dx = -j[1][0] / det, dz = j[0][0] / det;
        double n = Math.sqrt(dx * dx + dz * dz);
        return new double[]{dx / n, dz / n};
    }

    /** The north pole's face and local point, found through the turn. */
    public CubeLayout.LocalPoint northPole() {
        return localOfUnit(new double[]{0.0, 0.0, 1.0});
    }

    /** The south pole's face and local point, found through the turn. */
    public CubeLayout.LocalPoint southPole() {
        return localOfUnit(new double[]{0.0, 0.0, -1.0});
    }

    /** The planet unit vector of a cube corner ({@link CubeLayout.Corner#cube}). */
    public double[] cornerUnit(int[] cubeCorner) {
        double r = Math.sqrt(3.0);
        return turn.toPlanet(new double[]{
                (double) cubeCorner[0] / r,
                (double) cubeCorner[1] / r,
                (double) cubeCorner[2] / r});
    }

    /** {latitude, longitude} in degrees of a cube corner. */
    public double[] cornerLatLon(int[] cubeCorner) {
        return WorldProjection.latLonOfUnit(cornerUnit(cubeCorner));
    }

    private static double dot(double[] p, double[] q) {
        return (p[0] * q[0] + p[1] * q[1]) + p[2] * q[2];
    }

    private static WorldJacobian worldJacobian(double[][] m) {
        double det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
        return new WorldJacobian(m, Math.sqrt(Math.abs(det)), PortableMath.atan2(m[0][1], m[0][0]));
    }

    /** The unit vector of (latitude, longitude) in degrees, on the portable functions. */
    public static double[] unitOfLatLon(double latDeg, double lonDeg) {
        double phi = PortableMath.toRadians(latDeg), lam = PortableMath.toRadians(lonDeg);
        double sp = PortableMath.sin(phi), cp = PortableMath.cos(phi);
        return new double[]{cp * PortableMath.cos(lam), cp * PortableMath.sin(lam), sp};
    }
}
