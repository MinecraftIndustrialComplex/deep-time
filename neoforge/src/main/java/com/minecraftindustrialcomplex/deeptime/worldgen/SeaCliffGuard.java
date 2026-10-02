// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import net.minecraft.world.level.chunk.CarvingMask;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.levelgen.GenerationStep;

/**
 * Keeps vanilla's cave and canyon carvers from cutting arches into the underwater face of a sea
 * cliff (Ben, 2026-10-02: the carvers cut arches into the underwater base of the sea cliffs beside
 * a volcano). A tunnel whose centre line runs a few blocks behind a steep face breaks out through
 * it: the cut is filled with the sea (or, in an aquifer cell that is dry, left as air), and the
 * cliff reads as an arch.
 *
 * <p>The rule is one line of geometry: a block under the sea level, inside the ground, is out of
 * the carvers' reach when the terrain of some column within {@code skin} blocks to the side is
 * lower than the block, which is to say open water lies that close beside it at its height. The
 * block is put in the chunk's carving masks, which the carvers skip (as for {@link WaterBodies} and
 * {@link MagmaPass}). Caves may still run through the cliff and open on gentle ground; they just
 * keep a wall {@code skin} blocks thick towards open water. On a sea floor that slopes by a block
 * in ten the rule protects a band under the floor about as thick as the slope times the skin, one
 * block or less, and nothing at all on level ground.
 *
 * <p>Setting {@code underground.sea_cliff_skin_blocks} (6; 0 turns the guard off). Ledger MAG-057.
 * Noise caves (the density function's) are not carvers and are not touched.
 */
public final class SeaCliffGuard {

    private SeaCliffGuard() {}

    /** The settings: the width of the wall kept towards open water, blocks; 0 is off. */
    public record Rules(int skin) {
        public static final Rules DEFAULT = new Rules(6);

        public Rules {
            skin = Math.clamp(skin, 0, 32);
        }

        public static Rules of(JsonObject notes) {
            JsonElement e = notes;
            for (String key : new String[]{"bake", "underground", "params"}) {
                if (e == null || !e.isJsonObject())
                    return DEFAULT;
                e = e.getAsJsonObject().get(key);
            }
            if (e == null || !e.isJsonObject())
                return DEFAULT;
            JsonElement v = e.getAsJsonObject().get("sea_cliff_skin_blocks");
            if (v == null || !v.isJsonPrimitive() || !v.getAsJsonPrimitive().isNumber())
                return DEFAULT;
            double d = v.getAsDouble();
            return Double.isFinite(d) ? new Rules((int) Math.round(d)) : DEFAULT;
        }
    }

    /**
     * For the 16 x 16 columns of the chunk at (x0, z0), index {@code lz * 16 + lx}: the lowest top
     * block of the ground within {@code skin} columns (a square window) of the column.
     */
    static int[] lowestAround(Magma.Heights heights, int x0, int z0, int skin) {
        int w = 16 + 2 * skin;
        int[] tops = new int[w * w];
        for (int j = 0; j < w; j++)
            for (int i = 0; i < w; i++)
                tops[j * w + i] = Magma.top(heights.at(x0 - skin + i, z0 - skin + j));
        // Separable minimum: along x, then along z.
        int[] alongX = new int[w * 16];
        for (int j = 0; j < w; j++)
            for (int lx = 0; lx < 16; lx++) {
                int m = Integer.MAX_VALUE;
                for (int i = lx; i <= lx + 2 * skin; i++)
                    m = Math.min(m, tops[j * w + i]);
                alongX[j * 16 + lx] = m;
            }
        int[] out = new int[256];
        for (int lz = 0; lz < 16; lz++)
            for (int lx = 0; lx < 16; lx++) {
                int m = Integer.MAX_VALUE;
                for (int j = lz; j <= lz + 2 * skin; j++)
                    m = Math.min(m, alongX[j * 16 + lx]);
                out[lz * 16 + lx] = m;
            }
        return out;
    }

    /**
     * The y range, inclusive, of the blocks of a column that the carvers must leave alone: those
     * above the lowest ground to the side and no higher than the column's own ground or the top of
     * the water. Null when there are none.
     */
    static int[] guarded(int ownTop, int lowestAround, int seaLevel, int minY) {
        int hi = Math.min(ownTop, seaLevel - 1);
        int lo = Math.max(lowestAround + 1, minY);
        return lo > hi ? null : new int[]{lo, hi};
    }

    /** Puts the guarded blocks of the chunk in its carving masks; returns how many. */
    public static int apply(ChunkAccess chunk, Magma.Heights heights, int seaLevel, Rules rules) {
        if (rules.skin() <= 0 || !(chunk instanceof ProtoChunk proto))
            return 0;
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        int minY = chunk.getMinBuildHeight();
        int[] around = lowestAround(heights, x0, z0, rules.skin());
        CarvingMask air = proto.getOrCreateCarvingMask(GenerationStep.Carving.AIR);
        CarvingMask liquid = proto.getOrCreateCarvingMask(GenerationStep.Carving.LIQUID);
        int n = 0;
        for (int lz = 0; lz < 16; lz++)
            for (int lx = 0; lx < 16; lx++) {
                int own = Magma.top(heights.at(x0 + lx, z0 + lz));
                int[] r = guarded(own, around[lz * 16 + lx], seaLevel, minY);
                if (r == null)
                    continue;
                for (int y = r[0]; y <= r[1]; y++) {
                    air.set(lx, y, lz);
                    liquid.set(lx, y, lz);
                    n++;
                }
            }
        return n;
    }
}
