// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

/**
 * Climate searches that do not start from the thread's last answer.
 *
 * <p>Vanilla's biome search ({@code Climate.RTree.search}) starts from the entry it found last on
 * the thread and keeps it when another entry is exactly as near (it replaces the best only when a
 * nearer one turns up). So where a climate point is equally near two entries, which biome a query
 * gets depends on what the thread asked before. A chunk's biomes carry that order in them; far
 * terrain, which builds the same area on whichever thread asks and in any order, must not. Between
 * {@link #begin()} and {@link #end()} every search on the thread starts cold
 * ({@code mixin.ClimateRTreeMixin}): a tie goes to the first of the entries in the tree's order, the
 * same on every thread. Nests; nothing changes for searches outside.
 */
public final class ColdBiomeSearch {

    private static final ThreadLocal<int[]> DEPTH = ThreadLocal.withInitial(() -> new int[1]);

    private ColdBiomeSearch() {}

    /** Searches on this thread start cold until the matching {@link #end()}. */
    public static void begin() {
        DEPTH.get()[0]++;
    }

    public static void end() {
        DEPTH.get()[0]--;
    }

    /** Whether this thread's searches start cold now. */
    public static boolean active() {
        return DEPTH.get()[0] > 0;
    }
}
