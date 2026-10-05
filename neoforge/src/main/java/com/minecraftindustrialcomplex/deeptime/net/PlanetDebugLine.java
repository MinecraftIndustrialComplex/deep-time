// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;

import java.util.Locale;

/**
 * The F3 line (plan §7.15): {@code Deep Time: 12.345° N 67.890° W · <planet> (C 16384, 300 m/block)},
 * plus {@code · polar storm 62%} inside the cap band. Common code, so a GameTest can check the text;
 * the client-only overlay ({@code client/DeepTimeClient}) adds it to the debug screen's left column.
 */
public final class PlanetDebugLine {

    private PlanetDebugLine() {}

    /** The line for a camera at (x, z) with the given drawn storm, or null when {@code info} is not a planet. */
    public static String format(WorldInfoPayload info, double x, double z, float storm) {
        if (info == null || !info.planet() || info.circumference() <= 0)
            return null;
        int c = info.circumference();
        double lat, lon;
        if (info.petroffGuyou()) {
            // The seamless world: latitude and longitude depend on x and z (FORMAT.md "The world's
            // layout"); the longitude in the simulation's frame, shown in [−180°, 180°).
            double[] ll = info.worldProjection().latLon(x, z);
            lat = ll[0];
            lon = ll[1] >= 180.0 ? ll[1] - 360.0 : ll[1];
        } else {
            lat = PoleCapCurve.latitudeDeg(c, z);
            lon = PoleCapCurve.longitudeDeg(c, x);
        }
        StringBuilder s = new StringBuilder(String.format(Locale.ROOT, "Deep Time: %.3f° %s %.3f° %s · %s (C %d, %.0f m/block)",
                Math.abs(lat), lat >= 0 ? "N" : "S", Math.abs(lon), lon >= 0 ? "E" : "W",
                info.planetName(), c, info.mPerBlockH()));
        if (storm > 0.005f)
            s.append(String.format(Locale.ROOT, " · polar storm %d%%", Math.round(storm * 100)));
        return s.toString();
    }
}
