// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * Where a Deep Time world has lava underground, in plain Java (no Minecraft types, so JUnit can
 * test it). Both rules are vanilla's, measured from the world's floor instead of from y = 0.
 *
 * <p><b>Why.</b> Vanilla hard-codes two heights for a world whose floor is y −64: every open block
 * below y −54 is lava ({@code NoiseBasedChunkGenerator.createFluidPicker}), and a cave lake whose
 * surface is at y −10 or lower may be lava ({@code Aquifer.computeFluidType}). Those are
 * floor + 10 and floor + 54. A Deep Time world's floor is y −352, so left alone the first rule
 * turned the lowest 298 blocks of every cave, and of every stretch of sea deeper than 117 blocks,
 * into lava (Ben's preview of 2026-10-01: 2.15 million lava blocks in a 256 × 256 box, 4,020
 * columns of sea standing on lava). {@link LavaFluids} and the {@code deeptime:lava_lakes} density
 * function apply the two rules from this class instead.
 *
 * @param floorBlocks open space less than this many blocks above the world's floor is lava
 *                    (preset key {@code underground.lava_floor_blocks}; vanilla 10; 0: none)
 * @param lakeBlocks  cave lakes may be lava where their 40-block level cell is centred no more than
 *                    this many blocks above the floor ({@code underground.lava_lake_blocks};
 *                    vanilla 54; 0: none)
 */
public record LavaRules(int floorBlocks, int lakeBlocks) {

    /** Vanilla's two heights above the floor: −54 and −10 in a world whose floor is y −64. */
    public static final LavaRules DEFAULT = new LavaRules(10, 54);

    /**
     * Blocks of rock kept between a sea bed and the lava under it: where the bed lies within reach
     * of the lava floor, the lava's surface in that column drops to this far below the bed, so the
     * sea is water all the way down.
     */
    public static final int SEA_COVER = 8;

    /** The height of the aquifer's fluid-level cells, blocks ({@code Aquifer.computeFluidType}). */
    public static final int LAKE_CELL = 40;

    /** The largest values accepted (the presets' range checks agree). */
    public static final int MAX_BLOCKS = 992;

    public LavaRules {
        floorBlocks = Math.clamp(floorBlocks, 0, MAX_BLOCKS);
        lakeBlocks = Math.clamp(lakeBlocks, 0, MAX_BLOCKS);
    }

    /**
     * The world's rules: {@code notes.bake.underground.params}; {@link #DEFAULT} for a key that is
     * absent or not a number (a world baked before the settings existed, a synthetic world).
     */
    public static LavaRules of(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "underground", "params"}) {
            if (e == null || !e.isJsonObject())
                return DEFAULT;
            e = e.getAsJsonObject().get(key);
        }
        if (e == null || !e.isJsonObject())
            return DEFAULT;
        JsonObject p = e.getAsJsonObject();
        return new LavaRules(number(p, "lava_floor_blocks", DEFAULT.floorBlocks),
                number(p, "lava_lake_blocks", DEFAULT.lakeBlocks));
    }

    private static int number(JsonObject p, String key, int fallback) {
        JsonElement v = p.get(key);
        if (v == null || !v.isJsonPrimitive() || !v.getAsJsonPrimitive().isNumber())
            return fallback;
        return (int) Math.round(v.getAsDouble());
    }

    /** Open space with y below this is lava: {@code minY + floorBlocks}, never above the sea. */
    public int floodY(int minY, int seaLevel) {
        return Math.min(minY + floorBlocks, seaLevel);
    }

    /**
     * {@link #floodY} in a column whose terrain height is {@code height}: lowered to
     * {@link #SEA_COVER} blocks below the ground where that is lower, so lava never stands in open
     * water or within a few blocks of a sea bed.
     */
    public int columnFloodY(int minY, int seaLevel, double height) {
        return Math.min(floodY(minY, seaLevel), (int) Math.floor(height) - SEA_COVER);
    }

    /**
     * Whether a cave lake in fluid-level cell {@code cellY} (blocks {@code 40·cellY} to
     * {@code 40·cellY + 39}, the y the aquifer samples its lava noise at) may be lava. A lake's
     * surface lies within about ten blocks of its cell's centre.
     */
    public boolean lakeCell(int minY, int cellY) {
        return lakeBlocks > 0 && cellY * LAKE_CELL + LAKE_CELL / 2 <= minY + lakeBlocks;
    }

    /** The highest y a lava lake's surface can have: its cell's centre plus the level spread. */
    public int highestLakeY(int minY) {
        if (lakeBlocks <= 0)
            return Integer.MIN_VALUE;
        int cell = Math.floorDiv(minY + lakeBlocks - LAKE_CELL / 2, LAKE_CELL);
        return cell * LAKE_CELL + LAKE_CELL / 2 + LAKE_SPREAD;
    }

    /**
     * The most a lake's surface rises above its cell's centre: the spread noise × 10, quantised to
     * 3 ({@code Aquifer.computeRandomizedFluidSurfaceLevel}); the noise stays within ±1.2.
     */
    public static final int LAKE_SPREAD = 12;
}
