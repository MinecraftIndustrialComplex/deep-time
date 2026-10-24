// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

/**
 * The timelapse's globe view (plan §7.12): an orthographic projection of a keyframe (a map
 * picture of the planet's projection, {@link com.minecraftindustrialcomplex.deeptime.worlddata.Projection}), drawn on the CPU into a square
 * image (no shaders on the Create World screen). Keyframe pixel (px, py) is centred on longitude
 * 360°·px/w (the seam is 0°) and the latitude {@link com.minecraftindustrialcomplex.deeptime.worlddata.Projection#pictureLatitudeDeg} gives row py
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
                int px = (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureXOfLongitude(ll[1], w)) % w;
                int py = (int) Math.round(com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureYOfLatitude(ll[0], h));
                py = Math.max(0, Math.min(h - 1, py));
                int c = src[py * w + px];
                double z = Math.sqrt(Math.max(0, 1 - u * u - v * v));
                dst[at] = shade(c, 0.35 + 0.65 * z);
            }
        }
    }

    /** The border's edges on the globe, ABGR (orange-red). */
    public static final int BORDER_ABGR = 0xff2050ff;
    /** The cube's other edges, ABGR (white, drawn at half strength over the planet). */
    public static final int EDGE_ABGR = 0xffffffff;
    /** A corner a player can reach, ABGR (white), and one behind the border (orange-red). */
    public static final int CORNER_ABGR = 0xffffffff, CORNER_BLOCKED_ABGR = BORDER_ABGR;

    /**
     * Draws a cube world's edges and corners ({@link CubeOutline}) over a globe {@link #render}ed into
     * {@code dst} ({@code size × size}, ABGR), on the near side only: the border's edges two pixels
     * wide in {@link #BORDER_ABGR}, the others one pixel wide in white at half strength, and each
     * visible corner as a dot ringed in black, white when a player can reach it, orange-red when the
     * border keeps them off it.
     */
    public void drawOutline(CubeOutline outline, int[] dst, int size) {
        double[] uv = new double[2];
        double half = size / 2.0;
        for (CubeOutline.Edge e : outline.edges()) {
            boolean border = e.kind() == CubeOutline.Kind.BORDER;
            double px = Double.NaN, py = Double.NaN;
            for (double[] ll : e.latLon()) {
                if (forward(ll[0], ll[1], uv) == null) {
                    px = Double.NaN;
                    continue;
                }
                double x = half + uv[0] * half - 0.5, y = half - uv[1] * half - 0.5;
                if (!Double.isNaN(px)) {
                    int n = (int) Math.ceil(Math.max(Math.abs(x - px), Math.abs(y - py)));
                    for (int k = 0; k <= n; k++) {
                        double t = n == 0 ? 1.0 : (double) k / n;
                        int ix = (int) Math.round(px + (x - px) * t), iy = (int) Math.round(py + (y - py) * t);
                        if (border) {
                            for (int dy = 0; dy <= 1; dy++)
                                for (int dx = 0; dx <= 1; dx++)
                                    put(dst, size, ix + dx, iy + dy, BORDER_ABGR);
                        } else {
                            blend(dst, size, ix, iy, EDGE_ABGR);
                        }
                    }
                }
                px = x;
                py = y;
            }
        }
        for (CubeOutline.Corner c : outline.corners()) {
            if (forward(c.lat(), c.lon(), uv) == null)
                continue;
            int cx = (int) Math.round(half + uv[0] * half - 0.5), cy = (int) Math.round(half - uv[1] * half - 0.5);
            for (int dy = -2; dy <= 2; dy++)
                for (int dx = -2; dx <= 2; dx++)
                    put(dst, size, cx + dx, cy + dy, Math.abs(dx) == 2 || Math.abs(dy) == 2 ? 0xff000000
                            : c.reachable() ? CORNER_ABGR : CORNER_BLOCKED_ABGR);
        }
    }

    private static void put(int[] dst, int size, int x, int y, int abgr) {
        if (x >= 0 && y >= 0 && x < size && y < size && dst[y * size + x] != 0)
            dst[y * size + x] = abgr;
    }

    /** Half {@code abgr} over the pixel (on the disk only). */
    private static void blend(int[] dst, int size, int x, int y, int abgr) {
        if (x < 0 || y < 0 || x >= size || y >= size)
            return;
        int at = y * size + x, c = dst[at];
        if (c == 0)
            return;
        int b = (((c >> 16) & 0xff) + ((abgr >> 16) & 0xff)) / 2, g = (((c >> 8) & 0xff) + ((abgr >> 8) & 0xff)) / 2,
                r = ((c & 0xff) + (abgr & 0xff)) / 2;
        dst[at] = 0xff000000 | b << 16 | g << 8 | r;
    }

    /** Scales an ABGR colour's RGB by {@code f}; alpha kept (nodata stays transparent). */
    static int shade(int abgr, double f) {
        int a = abgr >>> 24;
        if (a == 0)
            return 0;
        int b = (int) (((abgr >> 16) & 0xff) * f), g = (int) (((abgr >> 8) & 0xff) * f), r = (int) ((abgr & 0xff) * f);
        return a << 24 | b << 16 | g << 8 | r;
    }

    /** Map pixel of (lat, lon) in a {@code w × h} map picture, as fractional coordinates of its centre. */
    public static double[] mapPixel(double latDeg, double lonDeg, int w, int h, double[] out) {
        out[0] = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureXOfLongitude(wrapLon(lonDeg), w) + 0.5;
        out[1] = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.pictureYOfLatitude(latDeg, h) + 0.5;
        return out;
    }

    /** Block (x, z) → (lat, lon) on a planet of circumference {@code c} (the planet's projection; {@code EVENTS.md}). */
    public static double[] latLon(double x, double z, int c, double[] out) {
        out[0] = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.latitudeDeg(c, z);
        out[1] = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.longitudeDeg(c, x);
        return out;
    }

    /** (lat, lon) → block (x, z) in the canonical span. */
    public static double[] block(double latDeg, double lonDeg, int c, double[] out) {
        out[0] = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.xOfLongitude(c, lonDeg);
        out[1] = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.zOfLatitude(c, latDeg);
        return out;
    }
}
