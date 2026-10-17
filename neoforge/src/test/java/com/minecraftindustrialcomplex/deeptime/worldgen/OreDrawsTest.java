// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The ore features' draws ({@link OreDraws}; Ben, 2026-10-02, "Rich Deposits, Thinner Scatter,
 * Also make deposits use some of the vanilla ore vein features"): the raw-ore draw of a rich core
 * and the world's scatter multiplier.
 */
class OreDrawsTest {

    private static final int C = 4096;

    private static Body body(int id, float raw) {
        Body.Zone core = new Body.Zone("potassic_core", "c:copper", null, 0f, 0.45f, 0f, 1f, 0.006f, 0.6f, raw);
        return new Body(id, 1, 1, "ore", new double[]{0, 0, 0}, new float[]{10, 10, 10}, 0f, 0f, Body.NO_CLIP,
                new int[]{-10, -10, -10}, new int[]{10, 10, 10}, null, List.of(core));
    }

    /** About the zone's raw share of its ore blocks are raw blocks; none without a share. */
    @Test
    void rawBlocksTakeTheZonesShare() {
        Body b = body(77, 0.05f);
        Body none = body(77, 0f);
        int raw = 0, n = 0;
        for (int x = -40; x < 40; x++)
            for (int y = -40; y < 40; y++)
                for (int z = 0; z < 8; z++) {
                    n++;
                    if (OreDraws.rawAt(b, b.zones().get(0), C, x, y, z))
                        raw++;
                    assertFalse(OreDraws.rawAt(none, none.zones().get(0), C, x, y, z));
                }
        double share = raw / (double) n;
        assertTrue(Math.abs(share - 0.05) < 0.006, "raw share " + share);
    }

    /** The raw draw is independent of the ore draw: not the same blocks as a low ore draw. */
    @Test
    void theRawDrawIsItsOwn() {
        Body b = body(5, 0.1f);
        int both = 0, raw = 0;
        for (int x = 0; x < 200; x++)
            for (int y = 0; y < 100; y++) {
                boolean r = OreDraws.rawAt(b, b.zones().get(0), C, x, y, 3);
                if (r) {
                    raw++;
                    if (OreDraws.unit(x, y, 3, b.id()) < 0.1)
                        both++;
                }
            }
        // Independent draws: about a tenth of the raw blocks also have an ore draw below 0.1.
        assertTrue(both < raw * 0.2, both + " of " + raw);
        // The same block on the other side of the seam draws the same.
        assertEquals(OreDraws.rawAt(b, b.zones().get(0), C, 10, 4, 3), OreDraws.rawAt(b, b.zones().get(0), C, 10 + C, 4, 3));
    }

    /**
     * The loot seed of a placer's panned block (Ben, 2026-10-16): never 0 (a zero seed rolls from the
     * level's random sequence and the block entity does not save it), a pure function of the world
     * seed and the position, the same on both sides of the seam, and different for neighbours.
     */
    @Test
    void thePanSeedIsAPureNonZeroFunctionOfSeedAndPosition() {
        long seed = 8675309L;
        assertEquals(OreDraws.panSeed(seed, C, 12, 63, -40), OreDraws.panSeed(seed, C, 12, 63, -40));
        assertNotEquals(OreDraws.panSeed(seed, C, 12, 63, -40), OreDraws.panSeed(seed + 1, C, 12, 63, -40));
        assertNotEquals(OreDraws.panSeed(seed, C, 12, 63, -40), OreDraws.panSeed(seed, C, 13, 63, -40));
        assertNotEquals(OreDraws.panSeed(seed, C, 12, 63, -40), OreDraws.panSeed(seed, C, 12, 64, -40));
        assertNotEquals(OreDraws.panSeed(seed, C, 12, 63, -40), OreDraws.panSeed(seed, C, 12, 63, -39));
        // The world wraps in x: a block and its copy one circumference away are one block.
        assertEquals(OreDraws.panSeed(seed, C, 12, 63, -40), OreDraws.panSeed(seed, C, 12 + C, 63, -40));
        assertEquals(OreDraws.panSeed(seed, C, 12, 63, -40), OreDraws.panSeed(seed, C, 12 - 3 * C, 63, -40));
        assertEquals(OreDraws.panSeed(seed, C, C / 2, 5, 5), OreDraws.panSeed(seed, C, -C / 2, 5, 5));
        // 40,000 neighbouring blocks: no zero, no repeated seed.
        Set<Long> seen = new HashSet<>();
        for (int x = -20; x < 20; x++)
            for (int y = 60; y < 70; y++)
                for (int z = -50; z < 50; z++) {
                    long s = OreDraws.panSeed(seed, C, x, y, z);
                    assertNotEquals(0L, s, "seed 0 at " + x + "," + y + "," + z);
                    assertTrue(seen.add(s), "repeated seed at " + x + "," + y + "," + z);
                }
        assertEquals(40_000, seen.size());
    }

    /** The seeds are spread: each of the 64 bits is set in about half of 10,000 neighbouring blocks' seeds. */
    @Test
    void thePanSeedsAreSpreadOverTheBits() {
        int[] ones = new int[64];
        int n = 0;
        for (int x = 0; x < 100; x++)
            for (int z = 0; z < 100; z++) {
                long s = OreDraws.panSeed(42L, C, x, 64, z);
                for (int bit = 0; bit < 64; bit++)
                    if ((s >>> bit & 1L) != 0)
                        ones[bit]++;
                n++;
            }
        for (int bit = 0; bit < 64; bit++)
            assertTrue(Math.abs(ones[bit] / (double) n - 0.5) < 0.03, "bit " + bit + " is set in " + ones[bit] + " of " + n);
    }

    /** The scatter multiplier is the manifest's notes.deposits.scatter_mult, else 1. */
    @Test
    void scatterMultFromTheNotes() {
        assertEquals(0.5, OreDraws.scatterMult(JsonParser.parseString("{\"deposits\": {\"scatter_mult\": 0.5}}").getAsJsonObject()));
        assertEquals(0.0, OreDraws.scatterMult(JsonParser.parseString("{\"deposits\": {\"scatter_mult\": 0}}").getAsJsonObject()));
        assertEquals(1.0, OreDraws.scatterMult(JsonParser.parseString("{\"deposits\": {}}").getAsJsonObject()));
        assertEquals(1.0, OreDraws.scatterMult(JsonParser.parseString("{\"deposits\": {\"scatter_mult\": -2}}").getAsJsonObject()));
        assertEquals(1.0, OreDraws.scatterMult(JsonParser.parseString("{}").getAsJsonObject()));
        assertEquals(1.0, OreDraws.scatterMult(null));
    }
}
