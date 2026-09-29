// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

/**
 * The timelapse's globe view (plan §7.12): an orthographic projection of an equirectangular
 * keyframe, drawn on the CPU into a square image (no shaders on the Create World screen). Keyframe
 * pixel (px, py) is centred on longitude 360°·px/w (the seam is 0°) and latitude 90° − 180°·py/h
 * ({@code FORMAT.md}, "Keyframes"); the globe looks at ({@link #centreLat}, {@link #centreLon}).
 *
 * <p>Screen coordinates are u (right) and v (up) in [−1, 1]; the disk is u² + v² ≤ 1.
 */
public final class GlobeProjection {

    private final double centreLat, centreLon;
    private final double sinLat0, cosLat0;

    public GlobeProjection(double centreLatDeg, double centreLonDeg) {
        this.centreLat = centreLatDeg;
        this.centreLon = wrapLon(centreLonDeg);
        double p = Math.toRadians(centreLatDeg);
        this.sinLat0 = Math.sin(p);
        this.cosLat0 = Math.cos(p);
    }

    public double centreLat() {
        return centreLat;
    }

    public double centreLon() {
        return centreLon;
    }

    static double wrapLon(double lon) {
        double l = lon % 360.0;
        return l < 0 ? l + 360.0 : l;
    }

    /**
     * The (lat, lon) in degrees seen at screen point (u, v), with lon in [0, 360); null outside the
     * disk. Writes into {@code out} (length 2) to stay allocation-free.
     */
    public double[] inverse(double u, double v, double[] out) {
        double r2 = u * u + v * v;
        if (r2 > 1.0)
            return null;
        double z = Math.sqrt(1.0 - r2);
        double lat = Math.asin(Math.max(-1, Math.min(1, z * sinLat0 + v * cosLat0)));
        double lon = Math.atan2(u, z * cosLat0 - v * sinLat0);
        out[0] = Math.toDegrees(lat);
        out[1] = wrapLon(centreLon + Math.toDegrees(lon));
        return out;
    }

    /**
     * The screen point (u, v) of (lat, lon), or null when it is on the far side. Writes into
     * {@code out} (length 2).
     */
    public double[] forward(double latDeg, double lonDeg, double[] out) {
        double p = Math.toRadians(latDeg), dl = Math.toRadians(lonDeg - centreLon);
        double cosP = Math.cos(p), sinP = Math.sin(p);
        double cosC = sinLat0 * sinP + cosLat0 * cosP * Math.cos(dl);
        if (cosC < 0)
            return null;
        out[0] = cosP * Math.sin(dl);
        out[1] = cosLat0 * sinP - sinLat0 * cosP * Math.cos(dl);
        return out;
    }

    /**
     * Renders {@code src} (ABGR, {@code w × h}, equirectangular) onto a {@code size × size} globe in
     * {@code dst} (ABGR), nearest-pixel, shaded by the cosine of the view angle; outside the disk
     * transparent.
     */
    public void render(int[] src, int w, int h, int[] dst, int size) {
        double[] ll = new double[2];
        double half = size / 2.0;
        for (int y = 0; y < size; y++) {
            double v = (half - (y + 0.5)) / half;
            for (int x = 0; x < size; x++) {
                double u = ((x + 0.5) - half) / half;
                int at = y * size + x;
                if (inverse(u, v, ll) == null) {
                    dst[at] = 0;
                    continue;
                }
                int px = (int) Math.round(ll[1] / 360.0 * w) % w;
                int py = (int) Math.round((90.0 - ll[0]) / 180.0 * h);
                py = Math.max(0, Math.min(h - 1, py));
                int c = src[py * w + px];
                double z = Math.sqrt(Math.max(0, 1 - u * u - v * v));
                dst[at] = shade(c, 0.35 + 0.65 * z);
            }
        }
    }

    /** Scales an ABGR colour's RGB by {@code f}; alpha kept (nodata stays transparent). */
    static int shade(int abgr, double f) {
        int a = abgr >>> 24;
        if (a == 0)
            return 0;
        int b = (int) (((abgr >> 16) & 0xff) * f), g = (int) (((abgr >> 8) & 0xff) * f), r = (int) ((abgr & 0xff) * f);
        return a << 24 | b << 16 | g << 8 | r;
    }

    /** Map pixel of (lat, lon) in a {@code w × h} equirectangular image, as fractional coordinates of its centre. */
    public static double[] mapPixel(double latDeg, double lonDeg, int w, int h, double[] out) {
        out[0] = wrapLon(lonDeg) / 360.0 * w + 0.5;
        out[1] = (90.0 - latDeg) / 180.0 * h + 0.5;
        return out;
    }

    /** Block (x, z) → (lat, lon) on a planet of circumference {@code c} (plan §5.2; {@code EVENTS.md}). */
    public static double[] latLon(double x, double z, int c, double[] out) {
        out[0] = -z / (c / 4.0) * 90.0;
        out[1] = wrapLon((x + c / 2.0) / c * 360.0);
        return out;
    }

    /** (lat, lon) → block (x, z) in the canonical span. */
    public static double[] block(double latDeg, double lonDeg, int c, double[] out) {
        out[0] = wrapLon(lonDeg) / 360.0 * c - c / 2.0;
        out[1] = -latDeg / 90.0 * (c / 4.0);
        return out;
    }
}
