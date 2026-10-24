// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;

import java.util.Locale;

/**
 * The F3 line (plan §7.15): {@code Deep Time: 12.345° N 67.890° W · <planet> (C 16384, 300 m/block)},
 * plus {@code · polar storm 62%} inside the cap band; on a cube world
 * {@code Deep Time: 12.345° N 67.890° W · face F1 · <planet> (C 16384, 300 m/block)}, with
 * {@code F1 (folded)} past a turning seam. Common code, so a GameTest can check the text;
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
        if (info.cube()) {
            // The cube world: latitude and longitude through the map table (FORMAT.md "The cube
            // world's layout"); off the planet (past the net's corners) or with another table, none.
            var wp = info.worldProjection();
            if (wp == null)
                return String.format(Locale.ROOT, "Deep Time: map mismatch (%s) · %s (C %d, %.0f m/block)", info.cubeMap(),
                        info.planetName(), c, info.mPerBlockH());
            double[] ll = wp.latLon(x, z);
            if (Double.isNaN(ll[0]))
                return String.format(Locale.ROOT, "Deep Time: off the planet · %s (C %d, %.0f m/block)", info.planetName(), c,
                        info.mPerBlockH());
            lat = ll[0];
            lon = ll[1] >= 180.0 ? ll[1] - 360.0 : ll[1];
            // The face the camera stands on (cube plan G10); a block off the faces folds onto one.
            var cube = com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.shared(c / 4);
            var cell = cube.faceOfBlock((long) Math.floor(x), (long) Math.floor(z));
            String face;
            if (cell != null) {
                face = cell.face().name();
            } else {
                var p = cube.foldPoint(x, z);
                face = p == null ? "?" : p.face().name() + " (folded)";
            }
            return String.format(Locale.ROOT, "Deep Time: %.3f° %s %.3f° %s · face %s · %s (C %d, %.0f m/block)",
                    Math.abs(lat), lat >= 0 ? "N" : "S", Math.abs(lon), lon >= 0 ? "E" : "W", face,
                    info.planetName(), c, info.mPerBlockH());
        } else if (info.petroffGuyou()) {
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
