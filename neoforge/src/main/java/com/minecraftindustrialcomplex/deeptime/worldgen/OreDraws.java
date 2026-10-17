// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;

/**
 * The draws of the ore features in plain Java (no Minecraft types, so JUnit can test them; the
 * Rust audit {@code deeptime deposit-audit} ports them bit for bit): the block hash
 * ({@link #unit}), the raw-ore draw of a rich deposit core ({@link #rawAt}), the loot seed of a
 * placer's suspicious block ({@link #panSeed}) and the world's background-scatter multiplier
 * ({@link #scatterMult}).
 */
public final class OreDraws {

    private OreDraws() {}

    /** Salt of the raw-ore draw: the body id XOR this (ASCII "RAW!"; FORMAT.md). */
    public static final int RAW_SALT = 0x52415721;

    /** A uniform draw in [0, 1) from a block position and a salt (SplitMix64 finaliser). */
    public static double unit(int x, int y, int z, int salt) {
        long h = mix(0x9E3779B97F4A7C15L * (x + 0x632BE59BD9B4E019L));
        h = mix(h ^ (0xC2B2AE3D27D4EB4FL * (y + 0x2545F4914F6CDD1DL)));
        h = mix(h ^ (0x165667B19E3779F9L * (z + 0x1B873593L)));
        h = mix(h ^ Integer.toUnsignedLong(salt));
        return (h >>> 11) * 0x1.0p-53;
    }

    static long mix(long z) {
        z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
        z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
        return z ^ (z >>> 31);
    }

    /**
     * Whether the ore block {@code zone} places at (x, y, z) of body {@code b} on a planet of
     * circumference c is the material's raw-ore block: a second draw, independent of the ore draw,
     * below the zone's {@code raw} share. Vanilla's copper and iron ore veins carry raw blocks among
     * their ore the same way (Ben, 2026-10-02: "make deposits use some of the vanilla ore vein
     * features").
     */
    public static boolean rawAt(Body b, Body.Zone zone, int c, int x, int y, int z) {
        return zone.raw() > 0 && unit(Math.floorMod(x + c / 2, c) - c / 2, y, z, b.id() ^ RAW_SALT) < zone.raw();
    }

    /** Salt of a panned block's loot seed (ASCII "PAN!"; ledger: ORE-269, game-choice). */
    public static final int PAN_SALT = 0x50414E21;

    /**
     * The loot seed of the suspicious block a placer puts at (x, y, z) in a world of seed
     * {@code worldSeed} and circumference c (Ben, 2026-10-16: placer gold is brushed out of suspicious
     * gravel and sand). A pure function of the world seed and the canonical position (x wraps, so a
     * block and its copy across the seam get the same seed), so regenerating a chunk, or the same
     * world on another machine, gives the same nuggets for the same brush. The nuggets are rolled with
     * it when the block is placed ({@code PlacerPanning.fill}). Never 0: a loot table rolled with seed
     * 0 draws from the level's random sequence instead of the block's own (and the brushable block
     * entity does not save a zero seed).
     */
    public static long panSeed(long worldSeed, int c, int x, int y, int z) {
        int cx = Math.floorMod(x + c / 2, c) - c / 2;
        long h = mix(0x9E3779B97F4A7C15L * (cx + 0x632BE59BD9B4E019L));
        h = mix(h ^ (0xC2B2AE3D27D4EB4FL * (y + 0x2545F4914F6CDD1DL)));
        h = mix(h ^ (0x165667B19E3779F9L * (z + 0x1B873593L)));
        h = mix(h ^ Integer.toUnsignedLong(PAN_SALT));
        h = mix(h ^ worldSeed);
        return h == 0 ? 1 : h;
    }

    /**
     * The world's background-scatter multiplier from the manifest's notes: the preset's
     * {@code deposits.scatter_mult}, which the deposit stage writes to
     * {@code notes.deposits.scatter_mult} (Ben, 2026-10-02, "Rich Deposits, Thinner Scatter"). It
     * multiplies every scatter rule's {@code clusters_per_chunk}. 1 for a world without it (a
     * synthetic or test world) or a value that is not a number ≥ 0.
     */
    public static double scatterMult(JsonObject notes) {
        JsonElement d = notes == null ? null : notes.get("deposits");
        if (d == null || !d.isJsonObject())
            return 1.0;
        JsonElement v = d.getAsJsonObject().get("scatter_mult");
        if (v == null || !v.isJsonPrimitive() || !v.getAsJsonPrimitive().isNumber())
            return 1.0;
        double m = v.getAsDouble();
        return Double.isFinite(m) && m >= 0 ? m : 1.0;
    }
}
