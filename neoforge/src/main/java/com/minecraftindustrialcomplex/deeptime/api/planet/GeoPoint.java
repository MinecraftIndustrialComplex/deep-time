// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.planet;

/**
 * A point of the planet: latitude in degrees [−90, 90], north positive, and longitude in degrees
 * [0, 360), east positive, in the planet's own frame (see the package's contract). The constructor
 * brings any finite longitude into [0, 360) and sets it to 0 at a pole, so two equal points are
 * {@code equals}.
 *
 * @throws IllegalArgumentException when the latitude is not in [−90, 90] or the longitude is not finite
 */
public record GeoPoint(double latitudeDeg, double longitudeDeg) {

    public GeoPoint {
        if (!(latitudeDeg >= -90.0 && latitudeDeg <= 90.0))
            throw new IllegalArgumentException("latitude " + latitudeDeg + " is not in [-90, 90]");
        if (!Double.isFinite(longitudeDeg))
            throw new IllegalArgumentException("longitude " + longitudeDeg + " is not finite");
        if (Math.abs(latitudeDeg) == 90.0) {
            longitudeDeg = 0.0;
        } else {
            double l = longitudeDeg % 360.0;
            if (l < 0.0)
                l += 360.0;
            longitudeDeg = l >= 360.0 ? 0.0 : l + 0.0;   // + 0.0 turns −0.0 into 0.0
        }
        latitudeDeg += 0.0;
    }
}
