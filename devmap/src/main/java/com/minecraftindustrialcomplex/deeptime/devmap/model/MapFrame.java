// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

/**
 * Where the map's pictures (satellite and elevation) sit in block space: pixel column i covers block x in
 * [x0 + i·s, x0 + (i + 1)·s), row j block z in [z0 + j·s, z0 + (j + 1)·s), s blocks a pixel. The
 * image covers the world's whole domain: x over one period from the seam (−period/2), z from the
 * world's north limit (the north pole on a Lambert-frame test level) south. On a cube
 * it covers the net's window, 4s by 3s from (−3s/2, −3s/2) ({@link PlanetGeometry#west}), and the
 * pictures leave the void off the faces transparent ({@link BaseImage#maskVoid}).
 *
 * @param x0          block x of the image's west side (the seam, or a cube window's west side)
 * @param z0          block z of its north side
 * @param blocksPerPx s, blocks a pixel (the same east–west and north–south)
 * @param w           width, pixels
 * @param h           height, pixels
 */
public record MapFrame(double x0, double z0, double blocksPerPx, int w, int h) {

    /** The width a server draws the pictures at, pixels (6.5 blocks a pixel on the play planet). */
    public static final int DEFAULT_WIDTH = 2048;

    public MapFrame {
        if (!(blocksPerPx > 0) || w <= 0 || h <= 0)
            throw new IllegalArgumentException("bad frame " + blocksPerPx + " " + w + "x" + h);
    }

    /** The frame of a {@code width}-pixel image of the world {@code p}. */
    public static MapFrame of(WorldProjection p, int width) {
        double period = p.period();
        double s = period / width;
        double span = p.zMax() - p.zMin();
        int h = Math.max(1, (int) Math.ceil(span / s - 1e-9));
        return new MapFrame(PlanetGeometry.west(p), p.zMin(), s, width, h);
    }

    /** Block x at pixel coordinate {@code px} (continuous: column i spans [i, i + 1)). */
    public double blockX(double px) {
        return x0 + px * blocksPerPx;
    }

    /** Block z at pixel coordinate {@code py}. */
    public double blockZ(double py) {
        return z0 + py * blocksPerPx;
    }

    /** Pixel coordinate of block x (not wrapped). */
    public double pixelX(double x) {
        return (x - x0) / blocksPerPx;
    }

    /** Pixel coordinate of block z. */
    public double pixelZ(double z) {
        return (z - z0) / blocksPerPx;
    }

    /** The column holding block x, wrapped into [0, w) (the image spans one period). */
    public int column(double x) {
        return (int) Math.floorMod((long) Math.floor(pixelX(x)), (long) w);
    }

    /** The row holding block z, clamped into [0, h). */
    public int row(double z) {
        long j = (long) Math.floor(pixelZ(z));
        return (int) Math.max(0, Math.min(h - 1, j));
    }

    /** The pixel of an ABGR image in this frame at block (x, z): x wraps, z clamps. */
    public int sample(int[] abgr, double x, double z) {
        return abgr[row(z) * w + column(x)];
    }
}
