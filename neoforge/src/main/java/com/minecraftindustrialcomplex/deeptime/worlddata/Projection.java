// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * The map projection between block space and the sphere: the one place the mod converts between
 * block coordinates, map pictures and latitude/longitude ({@code docs/design/equal-area-projection.md}
 * §2). The Java twin of the Rust {@code deeptime_format::projection}, with the same formulas in the
 * same order; {@code ColumnParityTest} checks that both give the same latitudes, longitudes, z of
 * latitudes, extent and index frame for the parity fixture's points.
 *
 * <p>The projection is <b>Lambert's cylindrical equal-area projection</b>, true on the equator (Ben,
 * decisions log 2026-10-02: "Lambert for now"; ledger: SCL-049, the equal-area property SCL-052):
 * <ul>
 *   <li>A planet of circumference C blocks has radius R = C / 2π blocks.</li>
 *   <li>Longitude λ = 360°·(x + C/2)/C in [0°, 360°): the seam x = −C/2 is 0°, x wraps.</li>
 *   <li>Latitude φ = asin(−z / R), north (−z) positive; the poles are at z = ∓R (∓2,607.6 at
 *       C = 16,384) and φ clamps to ±90° beyond them.</li>
 *   <li>Every block covers the same area of the planet; east–west a block is cos φ blocks of planet,
 *       north–south 1 / cos φ (true shape only on the equator).</li>
 *   <li>Grids run from z = −Zo to Zo ({@link #gridHalfExtent}: the smallest multiple of 64 at or past
 *       the pole on every real planet); so do the spatial indexes (river tiles, structure buckets,
 *       site index).</li>
 *   <li>A map picture of the whole planet ({@code w × h}, π : 1) has x linear in longitude and y
 *       linear in sin φ, the north pole at y = 0 and the south pole at y = h: z = −R + 2R·y/h.</li>
 * </ul>
 */
public final class Projection {

    /** The largest quantum of the grids' z extent, blocks (Rust {@code QUANTUM_MAX}). */
    public static final int QUANTUM_MAX = 64;

    private Projection() {}

    /** The planet's radius R = C / 2π, blocks (true scale on the equator). */
    public static double radius(double circumference) {
        return circumference / (2 * Math.PI);
    }

    /** |z| of the poles, blocks: R. The planet's z range is [−poleZ, poleZ]. */
    public static double poleZ(double circumference) {
        return radius(circumference);
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

    /** Latitude in degrees of block z, north (−z) positive, clamped to ±90° past the poles. */
    public static double latitudeDeg(double circumference, double z) {
        return Math.toDegrees(Math.asin(sinLat(circumference, z)));
    }

    /** The z of a latitude (north positive, clamped to ±90°), in [−R, R]. */
    public static double zOfLatitude(double circumference, double latDeg) {
        return -radius(circumference) * Math.sin(Math.toRadians(Math.max(-90.0, Math.min(90.0, latDeg))));
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

    /** Whether block z is on the planet: between the poles (x wraps, so every x is). */
    public static boolean onPlanet(double circumference, double z) {
        return Math.abs(z) <= poleZ(circumference);
    }

    /** Blocks of planet per block east–west at block z: cos φ (the metric's x scale; 0 at the poles). */
    public static double scaleEw(double circumference, double z) {
        return cosLat(circumference, z);
    }

    /** Blocks of planet per block north–south at block z: 1 / cos φ (infinite at the poles). */
    public static double scaleNs(double circumference, double z) {
        return 1.0 / cosLat(circumference, z);
    }

    /** Area of planet per block of block space: 1 on the planet (the projection keeps areas), 0 past the poles. */
    public static double areaScale(double circumference, double z) {
        return onPlanet(circumference, z) ? 1.0 : 0.0;
    }

    /** The raster's topology in x: wraps a column index into [0, w). */
    public static int wrapColumn(long i, int w) {
        return (int) Math.floorMod(i, (long) w);
    }

    /** The raster's topology in z: clamps a row index into [0, h − 1] (the rows stop at the poles). */
    public static int clampRow(long j, int h) {
        return (int) Math.max(0, Math.min(h - 1L, j));
    }

    /**
     * The quantum the grids' z extent is a multiple of: the largest power of two up to
     * {@link #QUANTUM_MAX} that divides C/2 and is at most R/2 (64 for every real planet).
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
    public static long gridHalfExtent(double circumference, int r) {
        double q = quantum(circumference);
        return (long) (Math.ceil(radius(circumference) / q) * q);
    }

    /**
     * Block x of the first cell of the world data's spatial indexes (river tiles, structure buckets,
     * site index): −C/2, the seam. Lookup frames, not geography; Rust {@code Projection::index_origin}.
     */
    public static double indexOriginX(double circumference) {
        return (double) indexOriginBlockX((long) circumference);
    }

    /** Block z of the first cell of the spatial indexes: −Zo, the grids' first row. */
    public static double indexOriginZ(double circumference) {
        return (double) indexOriginBlockZ((long) circumference);
    }

    /** Blocks the spatial indexes span north–south: 2·Zo. */
    public static double indexSpanZ(double circumference) {
        return (double) (2 * gridHalfExtent(circumference, 1));
    }

    /** {@link #indexOriginX} in whole blocks. */
    public static long indexOriginBlockX(long circumference) {
        return -(circumference / 2);
    }

    /** {@link #indexOriginZ} in whole blocks. */
    public static long indexOriginBlockZ(long circumference) {
        return -gridHalfExtent(circumference, 1);
    }

    /** Block z at row coordinate y of a map picture h rows high (y = 0 the north pole, y = h the south). */
    public static double pictureZ(double circumference, double y, int h) {
        double r = radius(circumference);
        return -r + 2.0 * r * y / Math.max(1, h);
    }

    /** The row coordinate of block z in a map picture h rows high (the inverse of {@link #pictureZ}). */
    public static double pictureY(double circumference, double z, int h) {
        double r = radius(circumference);
        return (z + r) / (2.0 * r) * Math.max(1, h);
    }

    /** Longitude, degrees (not wrapped), at column coordinate x of a map picture w columns wide (x = 0 the seam). */
    public static double pictureLongitudeDeg(double x, int w) {
        return 360.0 * x / w;
    }

    /** Column coordinate of a longitude in [0°, 360°) in a map picture w columns wide. */
    public static double pictureXOfLongitude(double lonDeg, int w) {
        return lonDeg / 360.0 * w;
    }

    /** Latitude, degrees, at row coordinate y of a map picture h rows high: rows are uniform in sin φ. */
    public static double pictureLatitudeDeg(double y, int h) {
        return Math.toDegrees(Math.asin(Math.max(-1.0, Math.min(1.0, 1.0 - 2.0 * y / Math.max(1, h)))));
    }

    /** Row coordinate of a latitude in a map picture h rows high. */
    public static double pictureYOfLatitude(double latDeg, int h) {
        return (1.0 - Math.sin(Math.toRadians(Math.max(-90.0, Math.min(90.0, latDeg))))) / 2.0 * h;
    }

    /** The height of a map picture w pixels wide at the projection's aspect (π : 1). */
    public static int pictureHeight(int w) {
        return Math.max(1, (int) Math.round(w / Math.PI));
    }
}
