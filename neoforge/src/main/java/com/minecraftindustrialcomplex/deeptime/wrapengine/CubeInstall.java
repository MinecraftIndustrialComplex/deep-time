// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.toroidalworld.api.v1.ToroidalShape;
import com.toroidalworld.api.v1.ToroidalWorldApi;
import net.minecraft.core.Direction;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;

/**
 * Installs the cube on the overworld behind the engine's flag (docs/design/wrap-engine.md §12.3 and
 * §12.7). Until phase 2's manifest reader exists the layout comes only from
 * {@code -Ddeeptime.wrapengine.cube=<s>} (dev); the band depth from
 * {@code -Ddeeptime.wrapengine.depth=<D>}, default {@link #DEFAULT_DEPTH}.
 *
 * <p>Toroidal World keeps the slide when it loops the level (Ben, 2026-10-03, "TW slides, engine
 * turns"): {@link #decide} picks the topology's mode from the level's Toroidal World shape.
 */
public final class CubeInstall {

    /** The face side s of the dev cube (a multiple of 32); unset: no cube. */
    public static final String CUBE_PROPERTY = "deeptime.wrapengine.cube";
    /** The band depth D (a multiple of 16, at most s/2). */
    public static final String DEPTH_PROPERTY = "deeptime.wrapengine.depth";
    /** Two chunks: what light and collisions need across a seam (ledger: SCL-185, game-choice). */
    public static final int DEFAULT_DEPTH = 32;

    private CubeInstall() {}

    /** How the cube goes on a level. */
    public enum Mode {
        /** Toroidal World loops the level as the cube needs: the engine turns, TW slides. */
        TURNS_ONLY,
        /** No loop on the level: the engine slides too. */
        WITH_SLIDES,
        /** A shape the cube cannot share: nothing is installed. */
        REFUSE
    }

    /** {@link #decide}'s answer and why. */
    public record Decision(Mode mode, String reason) {}

    /**
     * The install decision (design §12.3), a pure function of the level's Toroidal World shape.
     *
     * @param shaped  whether Toroidal World gave the level a shape
     * @param loopsX  whether it loops x
     * @param widthX  its x width, blocks
     * @param minX    its first x, blocks
     * @param loopsZ  whether it loops z
     * @param s       the cube's face side
     */
    public static Decision decide(boolean shaped, boolean loopsX, long widthX, long minX, boolean loopsZ, int s) {
        long period = 4L * s;
        if (!shaped || (!loopsX && !loopsZ))
            return new Decision(Mode.WITH_SLIDES, "the level does not loop: the engine slides east to west by " + period + " itself");
        if (loopsZ)
            return new Decision(Mode.REFUSE, "Toroidal World loops the level north to south; the cube's poles are not a loop");
        if (widthX != period || minX != -2L * s)
            return new Decision(Mode.REFUSE, "Toroidal World loops x over [" + minX + ", " + (minX + widthX) + "); the cube of face " + s
                    + " needs [" + (-2L * s) + ", " + (2L * s) + ")");
        return new Decision(Mode.TURNS_ONLY, "Toroidal World loops x over [" + minX + ", " + (minX + widthX) + "): it slides, the engine turns");
    }

    /** The cube's face side from the property, or 0 when unset. Refuses a size the engine cannot align. */
    static int side() {
        String v = System.getProperty(CUBE_PROPERTY);
        if (v == null || v.isBlank())
            return 0;
        int s = Integer.parseInt(v.trim());
        if (s <= 0 || s % 32 != 0)
            throw new IllegalArgumentException("-D" + CUBE_PROPERTY + "=" + s + ": the face side must be a positive multiple of 32");
        return s;
    }

    /** The band depth from the property, or {@link #DEFAULT_DEPTH}, checked against s. */
    static int depth(int s) {
        int d = Integer.getInteger(DEPTH_PROPERTY, DEFAULT_DEPTH);
        if (d <= 0 || d % 16 != 0 || d > s / 2)
            throw new IllegalArgumentException("-D" + DEPTH_PROPERTY + "=" + d + ": the band depth must be a positive multiple of 16 and at most s/2 = " + s / 2);
        return d;
    }

    /** On server start, with the flag and the property set: the cube on the overworld. */
    static void onServerStarted(MinecraftServer server) {
        int s;
        int d;
        try {
            s = side();
            if (s == 0)
                return;
            d = depth(s);
        } catch (IllegalArgumentException e) {
            DeepTime.LOGGER.error("[wrapengine] no cube installed: {}", e.getMessage());
            return;
        }
        ServerLevel overworld = server.overworld();
        ToroidalShape shape = ToroidalWorldApi.shapeOf(overworld).orElse(null);
        Decision dec = shape == null ? decide(false, false, 0, 0, false, s)
                : decide(true, shape.loops(Direction.Axis.X), shape.widthBlocks(Direction.Axis.X), (long) shape.minChunk(Direction.Axis.X) * 16,
                shape.loops(Direction.Axis.Z), s);
        if (dec.mode() == Mode.REFUSE) {
            DeepTime.LOGGER.error("[wrapengine] no cube installed on {}: {}", overworld.dimension().location(), dec.reason());
            return;
        }
        DeepTime.LOGGER.info("[wrapengine] cube s={} depth {}: {}", s, d, dec.reason());
        WrapEngine.install(overworld, WrapLayouts.cube(CubeLayout.of(s), d, dec.mode() == Mode.WITH_SLIDES));
    }
}
