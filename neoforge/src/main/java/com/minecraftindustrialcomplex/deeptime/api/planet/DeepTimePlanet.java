// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.planet;

import net.minecraft.world.level.Level;

import java.util.Optional;

/**
 * A Deep Time planet as a sphere, for other mods: latitude and longitude of a block and back, the
 * local frame (where north is, how long a block is), and great-circle distance, bearing and
 * destination, all correct across every seam, edge, corner and wrap of whatever map lays the planet
 * out in blocks. The package documentation is the contract; read it once.
 *
 * <pre>{@code
 * DeepTimePlanet.of(level).ifPresent(planet -> {
 *     double lat = planet.latitudeDeg(pos.getX() + 0.5, pos.getZ() + 0.5);          // NaN off the planet
 *     double km = planet.distanceMetres(ax, az, bx, bz) / 1000.0;
 *     planet.frame(x, z).ifPresent(f -> pointCompass(f.northBearingDeg()));
 * });
 * }</pre>
 */
public interface DeepTimePlanet {

    /** This API's version: 1 = the first (2026-10-24). Read it reflectively; javac inlines it. */
    int API_VERSION = 1;

    /**
     * The planet {@code level} is, or empty for a level that is not a Deep Time planet (and on a
     * client before the server's planet info for it arrived). The same object for the same world, no
     * allocation after the first call. Safe from any thread.
     */
    static Optional<DeepTimePlanet> of(Level level) {
        return PlanetLookup.of(level);
    }

    /** The sphere's radius in metres, fixed for the life of the world. */
    double radiusMetres();

    /** Whether the position (x, z) shows a planet point; false off the planet. */
    boolean isOnPlanet(double x, double z);

    /** The latitude and longitude the position (x, z) shows; empty off the planet. */
    Optional<GeoPoint> geoPoint(double x, double z);

    /** {@link #geoPoint}'s latitude without the objects: degrees in [−90, 90], NaN off the planet. */
    double latitudeDeg(double x, double z);

    /** {@link #geoPoint}'s longitude without the objects: degrees in [0, 360), 0 at a pole, NaN off the planet. */
    double longitudeDeg(double x, double z);

    /** The canonical position of a latitude and longitude: a position on the planet that shows it. */
    BlockXZ blockPoint(GeoPoint point);

    /**
     * The canonical position of the planet point (x, z) shows: the same for every position showing
     * that point, exact and a block centre for a block centre. Empty off the planet.
     */
    Optional<BlockXZ> canonical(double x, double z);

    /** The local frame of the block frame at (x, z); empty off the planet and where the map has no derivative. */
    Optional<LocalFrame> frame(double x, double z);

    /** The great-circle distance in metres between the points two positions show, in [0, πR]; NaN if either is off the planet. */
    double distanceMetres(double x0, double z0, double x1, double z1);

    /** The great-circle distance in metres between two planet points, in [0, πR]. */
    double distanceMetres(GeoPoint from, GeoPoint to);

    /**
     * The geographic bearing in degrees [0, 360) at the point (x0, z0) shows of the great circle to the
     * point (x1, z1) shows; NaN if either is off the planet, or the points are the same or antipodal.
     */
    double initialBearingDeg(double x0, double z0, double x1, double z1);

    /** The geographic bearing in degrees [0, 360) at {@code from} of the great circle to {@code to}; NaN when they are the same or antipodal. */
    double initialBearingDeg(GeoPoint from, GeoPoint to);

    /**
     * The canonical position reached by going {@code metres} along the great circle that leaves the
     * point (x, z) shows at geographic bearing {@code bearingDeg} (a negative distance goes the other
     * way); empty if (x, z) is off the planet or an argument is not finite.
     */
    Optional<BlockXZ> destination(double x, double z, double bearingDeg, double metres);

    /**
     * The planet point reached by going {@code metres} along the great circle that leaves {@code from}
     * at geographic bearing {@code bearingDeg} (a negative distance goes the other way).
     *
     * @throws IllegalArgumentException when the bearing or the distance is not finite
     */
    GeoPoint destination(GeoPoint from, double bearingDeg, double metres);
}
