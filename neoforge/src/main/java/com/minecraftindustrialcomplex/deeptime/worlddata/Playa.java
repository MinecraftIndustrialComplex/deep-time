// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * The playas of a world (FORMAT.md "Playas"): the baked layer of their salt and the rock class it
 * is. {@code worldgen/PlayaPass} lays it. Plain Java, no Minecraft.
 */
public final class Playa {

    /** The baked layer: blocks of salt per cell, 0 for none. */
    public static final String LAYER = "playa";
    /** The rock class of the salt when the bake's note names none. */
    public static final String DEFAULT_CLASS = "halite";

    private Playa() {}

    /** {@code notes.bake.water.playa.rock_class}, else {@link #DEFAULT_CLASS}. */
    public static String rockClass(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "water", "playa", "rock_class"}) {
            if (e == null || !e.isJsonObject())
                return DEFAULT_CLASS;
            e = e.getAsJsonObject().get(key);
        }
        return e != null && e.isJsonPrimitive() && e.getAsJsonPrimitive().isString() ? e.getAsString() : DEFAULT_CLASS;
    }

    /**
     * Whether a block column within {@code margin} blocks of (x, z), in x and in z, has a playa cell (the layer > 0)
     * as its nearest cell, as {@link RasterSampler#nearest} reads it (x wrapped, z through the grid's edges); false
     * without a layer. The volcanic roles keep this margin off the salt ({@code VolcanicBiomes.PLAYA_MARGIN},
     * BIO-165).
     */
    public static boolean within(RasterSampler layer, int x, int z, int margin) {
        if (layer == null)
            return false;
        var grid = layer.desc().grid();
        // A cube grid: the cells under the window carried onto its centre's face (a cell wider than the nearest cells).
        if (grid.isCube())
            return layer.anyInBox((long) x - margin, (long) z - margin, (long) x + margin, (long) z + margin, v -> v > 0);
        double r = grid.r();
        // The cell nearest a block is floor(u + 0.5) and grows with the block, so the cells nearest the window's
        // blocks are those from its first block's to its last block's.
        long i0 = (long) Math.floor(((double) x - margin - grid.originBlockX()) / r + 0.5);
        long i1 = (long) Math.floor(((double) x + margin - grid.originBlockX()) / r + 0.5);
        long j0 = (long) Math.floor(((double) z - margin - grid.originBlockZ()) / r + 0.5);
        long j1 = (long) Math.floor(((double) z + margin - grid.originBlockZ()) / r + 0.5);
        for (long j = j0; j <= j1; j++)
            for (long i = i0; i <= i1; i++)
                if (layer.get(i, j) > 0)
                    return true;
        return false;
    }
}
