// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * The map projection between block space and the sphere: the one place the mod converts between
 * block coordinates, map pictures and latitude/longitude ({@code docs/design/equal-area-projection.md}
 * §2). The Java twin of the Rust {@code deeptime_format::projection}; {@code ColumnParityTest} checks
 * that both give the same latitudes, longitudes and z of latitudes for the parity fixture's points.
 *
 * <p>The projection is <b>equirectangular</b> (plan §5.2): longitude λ = 360°·(x + C/2)/C in
 * [0°, 360°) (the seam x = −C/2 is 0°), latitude φ = −360°·z/C (north is −z), the poles at
 * z = ∓C/4, one block 360°/C both ways; φ clamps to ±90° past the poles. A map picture of the whole
 * planet is 2 : 1, x linear in longitude from the seam, y linear in latitude from the north pole.
 * Ben reopened the choice on 2026-10-02; whichever projection he picks replaces these bodies.
 */
public final class Projection {

    private Projection() {}

    /** The planet's radius R = C / 2π, blocks (true scale on the equator). */
    public static double radius(double circumference) {
        return circumference / (2 * Math.PI);
    }

    /** |z| of the poles, blocks: C/4. The world's playable z range is [−poleZ, poleZ]. */
    public static double poleZ(double circumference) {
        return circumference / 4.0;
    }

    /** Latitude in degrees of block z, north (−z) positive, clamped to ±90° past the poles. */
    public static double latitudeDeg(double circumference, double z) {
        return Math.max(-90.0, Math.min(90.0, -z * 360.0 / circumference));
    }

    /** The z of a latitude (north positive), in [−C/4, C/4]. */
    public static double zOfLatitude(double circumference, double latDeg) {
        return -Math.max(-90.0, Math.min(90.0, latDeg)) * circumference / 360.0;
    }

    /** Wraps block x into [−C/2, C/2). */
    public static double wrapX(double circumference, double x) {
        double t = ((x + 0.5 * circumference) % circumference + circumference) % circumference;
        return (t >= circumference ? 0.0 : t) - 0.5 * circumference;
    }

    /** Longitude of block x, degrees in [0, 360) east of the seam (the seam x = −C/2 is 0°). */
    public static double longitudeDeg(double circumference, double x) {
        double lon = 360.0 * (wrapX(circumference, x) + 0.5 * circumference) / circumference;
        double w = ((lon % 360.0) + 360.0) % 360.0;
        return w >= 360.0 ? 0.0 : w;
    }

    /** Longitude of block x as the F3 line shows it: degrees east of x = 0, folded to [−180°, 180°). */
    public static double longitudeEastDeg(double circumference, double x) {
        double lon = x * 360.0 / circumference;
        return ((lon + 180.0) % 360.0 + 360.0) % 360.0 - 180.0;
    }

    /** Block x of a longitude east of the seam, in [−C/2, C/2). */
    public static double xOfLongitude(double circumference, double lonDeg) {
        double l = ((lonDeg % 360.0) + 360.0) % 360.0;
        return wrapX(circumference, l / 360.0 * circumference - 0.5 * circumference);
    }

    /** Latitude, degrees, at row coordinate y of a map picture h rows high (y = 0 the north pole). */
    public static double pictureLatitudeDeg(double y, int h) {
        return 90.0 - y / h * 180.0;
    }

    /** Row coordinate of a latitude in a map picture h rows high. */
    public static double pictureYOfLatitude(double latDeg, int h) {
        return (90.0 - latDeg) / 180.0 * h;
    }

    /** The height of a map picture w pixels wide at the projection's aspect (2 : 1). */
    public static int pictureHeight(int w) {
        return Math.max(1, w / 2);
    }
}
