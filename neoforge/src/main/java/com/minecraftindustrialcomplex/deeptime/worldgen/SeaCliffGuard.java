// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * Keeps arches out of the underwater face of a sea cliff (Ben, 2026-10-02: arches cut into the
 * underwater base of the sea cliffs beside a volcano). Measured on the run's cliff (preview of
 * 2026-10-02, seed fixed, MIC set): of 157,639 blocks of caves that open onto the open sea, vanilla's
 * carvers (cave, cave_extra_underground, canyon) made about 46,000 and the density function's own
 * noise caves (cheese, spaghetti and entrance caves) the rest; with the carvers switched off 111,023
 * remained. So the guard has two halves, both on the same rule:
 * <ol>
 *   <li><b>The carvers are kept out</b>: the guarded blocks go in the chunk's carving masks, which
 *       the carvers skip (as for {@link WaterBodies} and {@link MagmaPass}).</li>
 *   <li><b>The noise caves are filled</b>: an open block (air, or cave water) in the guarded zone
 *       becomes rock again (stone, deepslate below y 0, which the strata pass then turns into the
 *       local rock), so no cave of any kind reaches the face. Lava and the sea above the ground are
 *       never touched.</li>
 * </ol>
 *
 * <p>The rule is geometry: a block under the sea level, inside the ground, is guarded where the
 * terrain of some column within {@code skin} blocks to the side is lower than the block (open water
 * lies that close beside it, at its height) <i>and</i> the ground drops by at least {@code skin}
 * blocks within that distance (a slope of 45 degrees or more: a cliff, not a sea floor, where cave
 * mouths are vanilla's and wanted). The guarded zone is a wall {@code skin} blocks thick behind the
 * face; caves may still run through the cliff but keep that wall towards the water, and open on
 * gentle ground.
 *
 * <p>Setting {@code underground.sea_cliff_skin_blocks} (6; 0 turns the guard off). Ledger MAG-057.
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
     * the water, in a column where the ground drops by at least {@code skin} blocks within {@code skin}
     * columns. Null when there are none.
     */
    static int[] guarded(int ownTop, int lowestAround, int seaLevel, int minY, int skin) {
        if (ownTop - lowestAround < skin)
            return null;   // not a cliff: a sea floor's cave mouths are left alone
        int hi = Math.min(ownTop, seaLevel - 1);
        int lo = Math.max(lowestAround + 1, minY);
        return lo > hi ? null : new int[]{lo, hi};
    }

    /**
     * The guarded blocks of the 16 x 16 columns of the chunk at (x0, z0): for each column, index
     * {@code lz * 16 + lx}, the inclusive y range {@code {lo, hi}} of blocks the guard holds (the
     * ground's top block is only masked, never filled, but is inside the range), or null.
     */
    public static int[][] zone(Magma.Heights heights, int x0, int z0, int seaLevel, int minY, int skin) {
        int[][] out = new int[256][];
        if (skin <= 0)
            return out;
        int[] around = lowestAround(heights, x0, z0, skin);
        for (int lz = 0; lz < 16; lz++)
            for (int lx = 0; lx < 16; lx++) {
                int own = Magma.top(heights.at(x0 + lx, z0 + lz));
                out[lz * 16 + lx] = guarded(own, around[lz * 16 + lx], seaLevel, minY, skin);
            }
        return out;
    }
}
