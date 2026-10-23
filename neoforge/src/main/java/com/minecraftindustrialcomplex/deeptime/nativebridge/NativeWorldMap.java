// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.file.Path;

/**
 * A world-data folder drawn as the workbench's "Final world" draws it: the Rust renderer behind
 * that view ({@code deeptime-tiles}' {@code BakeMap}, the core's satellite look fed with the bake's
 * climate, ice, surface rock, lakes and rivers) over an equirectangular frame of block space,
 * through the native {@code renderWorldMap}. One look in every viewer, so the game asks the natives
 * rather than drawing a second satellite picture in Java. A view of finished world data: it reads
 * the folder and writes nothing. The dev map ({@code deeptime_devmap}) shows it as its satellite
 * picture.
 */
public final class NativeWorldMap {

    /** The satellite look (true colour). */
    public static final String SATELLITE = "sat";
    /** The workbench's elevation ramp under its hill shade. */
    public static final String ELEVATION = "elev";

    /**
     * A drawn frame.
     *
     * @param abgr  {@code w × h} pixels row by row from the north, ABGR ints (the RGBA bytes read
     *              little-endian: {@code NativeImage}'s order)
     * @param info  the native's JSON {@code {"layer","w","h","zoom","climate","rivers","lakes","ms"}}
     */
    public record Picture(int w, int h, int[] abgr, String info) {}

    private NativeWorldMap() {}

    /**
     * Draws {@code layer} ({@link #SATELLITE} or {@link #ELEVATION}) of {@code worldDataDir} over the
     * frame of {@code w × h} pixels whose pixel (i, j) is block (x0 + (i + ½)·s, z0 + (j + ½)·s), with
     * {@code threads} threads (at least one). Blocks the caller until done.
     *
     * @throws IllegalStateException    the native library is not linked
     * @throws IllegalArgumentException a bad layer, size or frame
     * @throws DeepTimeIoException      the folder is not world data the renderer can read
     */
    public static Picture render(Path worldDataDir, String layer, double x0, double z0, double blocksPerPx, int w, int h,
                                 int threads) {
        NativeLoader.requireLinked();
        if (w < 0 || h < 0 || (long) w * h * 4 > Integer.MAX_VALUE)
            throw new IllegalArgumentException("bad frame size " + w + " x " + h);
        ByteBuffer rgba = ByteBuffer.allocateDirect(w * h * 4);
        String info = DeepTimeNative.renderWorldMap(worldDataDir.toString(), layer, x0, z0, blocksPerPx, w, h,
                Math.max(1, threads), rgba);
        int[] px = new int[w * h];
        rgba.order(ByteOrder.LITTLE_ENDIAN).asIntBuffer().get(px);
        return new Picture(w, h, px, info);
    }
}
