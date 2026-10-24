// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * The lat-lon picture of the keyframes and the maps ({@code FORMAT.md}, "Keyframes"): a picture of
 * the whole planet {@code w × h} pixels, π : 1, with x linear in longitude (x = 0 is the seam, 0°)
 * and y linear in sin φ (the north pole at y = 0, the south pole at y = h). It is the simulation's
 * Lambert frame drawn as a picture (ledger: SCL-049, SCL-052), whatever the shape of the world the
 * game plays: the cube world's map pages draw their nets from these pictures.
 *
 * <p>Kept from {@code Projection} when the Lambert world went (cube plan §7.4 item 1, decisions log
 * 2026-10-24): "A lat-lon picture projection stays for keyframe pictures and maps" (item 2).
 */
public final class MapPicture {

    private MapPicture() {}

    /** Longitude, degrees (not wrapped), at column coordinate x of a picture w columns wide (x = 0 the seam). */
    public static double longitudeDeg(double x, int w) {
        return 360.0 * x / w;
    }

    /** Column coordinate of a longitude in [0°, 360°) in a picture w columns wide. */
    public static double xOfLongitude(double lonDeg, int w) {
        return lonDeg / 360.0 * w;
    }

    /** Latitude, degrees, at row coordinate y of a picture h rows high: rows are uniform in sin φ. */
    public static double latitudeDeg(double y, int h) {
        return Math.toDegrees(Math.asin(Math.max(-1.0, Math.min(1.0, 1.0 - 2.0 * y / Math.max(1, h)))));
    }

    /** Row coordinate of a latitude in a picture h rows high. */
    public static double yOfLatitude(double latDeg, int h) {
        return (1.0 - Math.sin(Math.toRadians(Math.max(-90.0, Math.min(90.0, latDeg))))) / 2.0 * h;
    }

    /** The height of a picture w pixels wide at the picture's aspect (π : 1). */
    public static int height(int w) {
        return Math.max(1, (int) Math.round(w / Math.PI));
    }
}
