// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import org.junit.jupiter.api.Test;

import java.nio.file.Path;

/**
 * The cave-entrance shield beside Deep Time's water (HYD-078) on the real test planet
 * ({@code planet4k}): every river, lake and shallow-sea column and the ground within
 * {@link WaterCaveShield#FULL_BLOCKS} of it gets the full shield, so the router opens no cave
 * mouth there and {@link WaterBodies} never builds a bank or a bed up from a pit's floor (Ben,
 * 2026-10-20: "bodies of water just have these dirt walls around them"); ground with no water
 * within {@link WaterCaveShield#FADE_BLOCKS} gets none, so the caves away from water are vanilla's.
 */
class WaterCaveShieldTest {

    private static final int CELL = 4;

    private static WorldData world() {
        return WorldData.open(Path.of(System.getProperty("deeptime.testWorlds")).resolve("planet4k"));
    }

    @Test
    void weightsAreFullNearAndNoneFar() {
        assertEquals(1.0, WaterCaveShield.weight(-3.0));
        assertEquals(1.0, WaterCaveShield.weight(WaterCaveShield.FULL_BLOCKS));
        assertEquals(0.5, WaterCaveShield.weight((WaterCaveShield.FULL_BLOCKS + WaterCaveShield.FADE_BLOCKS) / 2), 1e-12);
        assertEquals(0.0, WaterCaveShield.weight(WaterCaveShield.FADE_BLOCKS));
        assertEquals(0.0, WaterCaveShield.weight(Double.POSITIVE_INFINITY));
        double last = 1.0;
        for (double d = 0; d <= 20; d += 0.25) {
            double w = WaterCaveShield.weight(d);
            assertTrue(w <= last && w >= 0.0 && w <= 1.0, "weight falls monotonically at " + d);
            last = w;
        }
        int top = 62;
        assertEquals(1.0, WaterCaveShield.seaWeight(200, top), "land counts the sea fully");
        assertEquals(1.0, WaterCaveShield.seaWeight(top - WaterCaveShield.SHALLOW_FULL_BLOCKS, top));
        assertEquals(0.0, WaterCaveShield.seaWeight(top - WaterCaveShield.SHALLOW_NONE_BLOCKS, top));
        assertEquals(0.0, WaterCaveShield.seaWeight(-300, top), "the deep sea floor keeps its cave mouths");
        assertEquals(1.0, WaterCaveShield.combine(3.0, Double.POSITIVE_INFINITY, -300, top));
        assertEquals(0.0, WaterCaveShield.combine(Double.POSITIVE_INFINITY, 0.0, -300, top));
        assertEquals(1.0, WaterCaveShield.combine(Double.POSITIVE_INFINITY, 0.0, top, top));
    }

    /** The lift at shield 1 keeps every cave mouth shut: 5 · entrances never falls that far below the band's edge. */
    @Test
    void theLiftClosesTheBandWhereTheShieldIsFull() {
        double band = 1.5625;
        // vanilla's overworld/caves/entrances is min(cave_entrance + 0.37 + gradient, roughness + spaghetti_3d),
        // both above -2 (normal noise in [-1, 1] apart from rare tails, clamps to [-1, 1]); ratio <= 1.
        assertTrue(5.0 * -2.0 + WaterCaveShield.LIFT > band);
    }

    private record Kinds(int river, int lake, int shallowSea, int deepSea) {}

    /**
     * Every water column of a sparse grid over the planet, and every column within
     * {@link WaterCaveShield#FULL_BLOCKS} − 1 of it, has the full shield (the shallow sea's only
     * where that column's ground is shallow too); prints how many of each were seen.
     */
    @Test
    void everyWaterColumnAndItsShoreAreShielded() {
        WorldData wd = world();
        Water.Spec spec = Water.Spec.of(wd, CELL);
        WaterCaveShield shield = new WaterCaveShield(wd, spec);
        int c = wd.circumference(), seaTop = spec.seaLevel() - 1;
        int zHalf = c / 4 - 8;
        int river = 0, lake = 0, shallow = 0, deep = 0, checked = 0;
        int ring = (int) WaterCaveShield.FULL_BLOCKS - 1;
        for (int z = -zHalf; z < zHalf; z += 7) {
            for (int x = -c / 2; x < c / 2; x += 13) {
                Water.Core q = Water.core(wd, spec, x, z);
                switch (q.kind()) {
                    case RIVER -> river++;
                    case LAKE -> lake++;
                    case SEA -> {
                        if (q.ground() >= seaTop - WaterCaveShield.SHALLOW_FULL_BLOCKS)
                            shallow++;
                        else {
                            deep++;
                            continue;
                        }
                    }
                    default -> {
                        continue;
                    }
                }
                // The column and a ring of shore around it (every 3rd block of the disc).
                for (int dz = -ring; dz <= ring; dz += 3) {
                    for (int dx = -ring; dx <= ring; dx += 3) {
                        if (dx * dx + dz * dz > ring * ring)
                            continue;
                        int px = Math.floorMod(x + dx + c / 2, c) - c / 2, pz = z + dz;
                        if (q.kind() == Water.Kind.SEA
                                && wd.height(px, pz) < seaTop - WaterCaveShield.SHALLOW_FULL_BLOCKS)
                            continue;   // deeper ground beside a shallow sea column: the aquifer's
                        double s = shield.at(px, pz);
                        assertEquals(1.0, s, q.kind() + " column (" + x + ", " + z + "): shield at (" + px + ", " + pz + ")");
                        checked++;
                    }
                }
            }
        }
        System.out.printf("[water-shield] columns seen: %d river, %d lake, %d shallow sea, %d deep sea; %d shore columns checked%n",
                river, lake, shallow, deep, checked);
        assertTrue(river > 0 && lake + shallow > 0, "the test planet has rivers and lakes or a shallow sea");
    }

    /**
     * A column the shield touches nearly always has water within {@link WaterCaveShield#FADE_BLOCKS}
     * plus the rules' reach (the rule is conservative: a wet {@code water_level} cell or an
     * {@code elevation} cell whose E − A is under the top need not hold a water column), and much
     * of the planet keeps its cave mouths.
     */
    @Test
    void groundFarFromWaterIsUnshielded() {
        WorldData wd = world();
        Water.Spec spec = Water.Spec.of(wd, CELL);
        WaterCaveShield shield = new WaterCaveShield(wd, spec);
        int c = wd.circumference(), seaTop = spec.seaLevel() - 1;
        int zHalf = c / 4 - 40;
        int zero = 0, some = 0, samples = 0, dry = 0;
        // The brute force: the nearest water core within the fade and the sea's reach.
        int r = (int) Math.ceil(WaterCaveShield.FADE_BLOCKS + WaterCaveShield.SEA_REACH) + 1;
        for (int z = -zHalf; z < zHalf; z += 97) {
            for (int x = -c / 2; x < c / 2; x += 89) {
                double s = shield.at(x, z);
                samples++;
                if (s == 0.0) {
                    zero++;
                    continue;
                }
                some++;
                boolean water = false;
                for (int dz = -r; dz <= r && !water; dz++)
                    for (int dx = -r; dx <= r && !water; dx++) {
                        if (dx * dx + dz * dz > r * r)
                            continue;
                        Water.Kind k = Water.core(wd, spec, x + dx, z + dz).kind();
                        water = k == Water.Kind.RIVER || k == Water.Kind.LAKE || k == Water.Kind.SEA;
                    }
                if (!water)
                    dry++;
            }
        }
        System.out.printf("[water-shield] %d samples: %d unshielded, %d shielded (%.1f %%), %d of them with no water column within %d blocks%n",
                samples, zero, some, 100.0 * some / samples, dry, r);
        assertTrue(zero > samples / 4, "most of the planet keeps its cave mouths: " + zero + " of " + samples);
        assertTrue(dry <= Math.max(2, some / 10), dry + " of " + some + " shielded samples have no water within " + r + " blocks");
        assertTrue(seaTop > 0);
    }

    /** The cost per column, as the router's flat cache asks it (25 columns a chunk). */
    @Test
    void costPerColumn() {
        WorldData wd = world();
        WaterCaveShield shield = new WaterCaveShield(wd, Water.Spec.of(wd, CELL));
        int c = wd.circumference();
        double sink = 0;
        long best = Long.MAX_VALUE;
        int n = 0;
        for (int pass = 0; pass < 3; pass++) {
            n = 0;
            long t0 = System.nanoTime();
            for (int z = -c / 4 + 16; z < c / 4 - 16; z += 4 * 9) {
                for (int x = -c / 2; x < c / 2; x += 4 * 5) {
                    sink += shield.at(x, z);
                    n++;
                }
            }
            best = Math.min(best, System.nanoTime() - t0);
        }
        double us = best / 1000.0 / n;
        System.out.printf("[water-shield] %.2f µs a column over %d columns (%.2f ms a chunk at 25 columns; sum %.1f)%n",
                us, n, us * 25 / 1000.0, sink);
        assertTrue(us < 500.0, "the shield costs " + us + " µs a column");
    }
}
