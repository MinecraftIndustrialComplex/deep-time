// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhFarTerrain;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrain;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrainCheck;
import com.minecraftindustrialcomplex.deeptime.worlddata.FarColumn;
import net.minecraft.core.Holder;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Far terrain against generated chunks (plan §10.5 P3-04), on the real baked planet
 * ({@code deeptime:test_planet_real}: every pipeline stage, the pack's biomes, surface rules and
 * features). Far terrain is built without a chunk; these tests then generate the chunk and compare
 * the same columns. Distant Horizons is not needed here and no GameTest-server run loads it (it cannot run on the
 * GameTest server); what Distant Horizons stores of these columns is the smoke check's
 * ({@code tools/smoke/run.py --dh}).
 *
 * <p>The thresholds are properties any planet must meet, not one planet's numbers: the ground and
 * the water come from the same column rule the chunk is built by, so they agree but for what
 * features and the pack's surface rules do on top; the top block comes from a table, so its
 * agreement is logged and only a floor is asserted.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class FarTerrainGameTests {

    private FarTerrainGameTests() {}

    static FarTerrain far(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        return FarTerrain.of(level).orElseGet(() -> {
            helper.fail("test_planet_real has no far terrain (it is not generating from world data)");
            throw new IllegalStateException();
        });
    }

    /** A spread of columns over the planet, clear of the pole edge and of the build floor. */
    static int[] column(int i, int c) {
        int x = -c / 2 + 11 + i * (c / 131);
        int z = ((i * 53) % 113 - 56) * (c / 4 / 64) + 7;
        return new int[]{x, z};
    }

    /**
     * Heights, water, the biome and the ground block of far terrain's columns against the generated
     * chunk (features included: the comparison looks under trees and plants).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void farTerrainMatchesGeneratedChunks(GameTestHelper helper) {
        FarTerrain far = far(helper);
        ServerLevel level = far.level();
        int c = far.data().circumference();
        FarTerrainCheck.Tally tally = new FarTerrainCheck.Tally();
        ChunkSampling.forEach(helper, 128, i -> {
            int[] p = column(i, c);
            FarTerrain.Column column = far.column(p[0], p[1]);
            if (column.ground() < level.getMinBuildHeight() + 32)
                return false;   // the floor band: the router's bottom slide, not the bake
            ChunkAccess chunk = level.getChunkSource().getChunk(p[0] >> 4, p[1] >> 4, ChunkStatus.FEATURES, true);
            tally.add(column, FarTerrainCheck.real(chunk, p[0], p[1]));
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] farTerrainMatchesGeneratedChunks {}", tally);
            assertAgrees(helper, tally, 64, "");
            helper.succeed();
        });
    }

    /** The properties far terrain must meet against generated chunks (see the class comment). */
    static void assertAgrees(GameTestHelper helper, FarTerrainCheck.Tally tally, int minColumns, String where) {
        helper.assertTrue(tally.columns >= minColumns, where + "only " + tally.columns + " columns compared");
        helper.assertTrue(tally.share(tally.heightWithin1) >= 0.90,
                where + "the ground is within a block in only " + tally.heightWithin1 + " of " + tally.columns + " columns; worst " + tally.worstHeight);
        helper.assertTrue(tally.share(tally.heightWithin2) >= 0.95,
                where + "the ground is within two blocks in only " + tally.heightWithin2 + " of " + tally.columns + " columns; worst " + tally.worstHeight);
        helper.assertTrue(tally.share(tally.waterAgree) >= 0.95,
                where + "water or dry agrees in only " + tally.waterAgree + " of " + tally.columns + " columns");
        helper.assertTrue(tally.waterBoth == 0 || tally.waterTopExact >= 0.9 * tally.waterBoth,
                where + "the water's top agrees in only " + tally.waterTopExact + " of " + tally.waterBoth + " water columns");
        helper.assertTrue(tally.share(tally.biomeSame) >= 0.95,
                where + "the biome agrees in only " + tally.biomeSame + " of " + tally.columns + " columns: " + tally.toJson().get("biome_differences"));
        helper.assertTrue(tally.share(tally.groundBlockSame) >= 0.4,
                where + "the ground block agrees in only " + tally.groundBlockSame + " of " + tally.columns + " columns: "
                        + tally.toJson().get("ground_block_differences"));
    }

    /**
     * Far terrain against generated chunks on both sides of the seam (Ben, 2026-10-02, with the patch
     * that lets this generator through Toroidal World's guard): the 64 blocks west of x = C/2 and the
     * 64 blocks east of it (x = −C/2 on, the planet's westernmost columns), on four rows from the
     * equator to mid-latitudes. Each side must meet the planet-wide test's properties on its own.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void farTerrainMatchesGeneratedChunksAcrossTheSeam(GameTestHelper helper) {   // ledger: BAK-042
        FarTerrain far = far(helper);
        ServerLevel level = far.level();
        int c = far.data().circumference();
        int[] rows = {7, c / 16 + 7, -c / 16 + 7, c / 8 + 7};
        FarTerrainCheck.Tally west = new FarTerrainCheck.Tally(), east = new FarTerrainCheck.Tally();
        ChunkSampling.forEach(helper, 2 * 16 * rows.length, i -> {
            boolean eastSide = (i & 1) == 1;
            int k = (i >> 1) % 16, row = rows[(i >> 1) / 16];
            int x = eastSide ? -c / 2 + 4 * k + 1 : c / 2 - 64 + 4 * k + 2;
            FarTerrain.Column column = far.column(x, row);
            if (column.ground() < level.getMinBuildHeight() + 32)
                return false;
            ChunkAccess chunk = level.getChunkSource().getChunk(x >> 4, row >> 4, ChunkStatus.FEATURES, true);
            (eastSide ? east : west).add(column, FarTerrainCheck.real(chunk, x, row));
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] farTerrainMatchesGeneratedChunksAcrossTheSeam C={} west {} east {}", c, west, east);
            assertAgrees(helper, west, 32, "west of the seam: ");
            assertAgrees(helper, east, 32, "east of the seam: ");
            helper.succeed();
        });
    }

    /**
     * Distant Horizons is loaded when the run's mods include it ({@code -Ddeeptime.dh.expected},
     * set by neoforge/build.gradle from the server set and its GameTest exclusions), and then far
     * terrain is bound to it: the API resolves and the real test planet's override registered. A
     * run that expects it and lacks it fails here, rather than every far-terrain test passing
     * without Distant Horizons. Today no GameTest-server run has it: Distant Horizons 3.3.1 (as
     * 3.3.0) casts the server to {@code DedicatedServer} at start, which NeoForge's GameTest server is
     * not (tools/smoke/profiles/mic.gametest-excluded), so its path is the smoke check's
     * ({@code tools/smoke/run.py --dh}) on a dedicated server.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void farTerrainHasDistantHorizonsWhenExpected(GameTestHelper helper) {
        boolean expected = Boolean.getBoolean("deeptime.dh.expected");
        boolean installed = DhFarTerrain.installed();
        DeepTime.LOGGER.info("[gametest] farTerrainHasDistantHorizonsWhenExpected expected={} installed={} twGuardPatch={}",
                expected, installed, System.getProperty("deeptime.twDhGuard", "not configured"));
        helper.assertTrue(installed || !expected,
                "this run's mods include Distant Horizons (-Ddeeptime.dh.expected=true) but it is not loaded");
        if (installed && DhFarTerrain.enabled()) {
            helper.assertTrue(DhFarTerrain.api().isPresent(), "Distant Horizons is loaded but its API did not bind: " + DhFarTerrain.unavailable());
            ServerLevel level = RealPlanetGameTests.level(helper);
            DhFarTerrain.Bound bound = DhFarTerrain.bound(level.dimension());
            helper.assertTrue(bound != null && bound.registered(), "far terrain is not registered with Distant Horizons for "
                    + level.dimension().location() + (bound == null ? "" : ": " + bound.message()));
        }
        helper.succeed();
    }

    private record Layer(int bottom, int top, BlockState block, int light) {}

    private static List<Layer> layers(FarTerrain far, int x, int z, int step, List<Holder<Biome>> biome) {
        List<Layer> out = new ArrayList<>();
        far.build(x, z, step, 1, (relX, relZ, count, bottom, top, block, material, light, b) -> {
            biome.add(b);
            for (int k = 0; k < count; k++)
                out.add(new Layer(bottom[k], top[k], block[k], light[k]));
        });
        return out;
    }

    /**
     * The seam and the build range: a column one lap east or west is the same column, biome and
     * blocks included, at full and at coarse detail; every column's layers run from the level's
     * bottom to its top without a gap; and a whole section builds across the seam.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void farTerrainWrapsAndFillsTheColumn(GameTestHelper helper) {
        FarTerrain far = far(helper);
        ServerLevel level = far.level();
        int c = far.data().circumference();
        int minY = level.getMinBuildHeight(), maxY = level.getMaxBuildHeight();
        int checked = 0;
        for (int step : new int[]{1, 8}) {
            for (int i = 0; i < 48; i++) {
                int[] p = column(i * 2, c);
                List<Holder<Biome>> b0 = new ArrayList<>(), b1 = new ArrayList<>(), b2 = new ArrayList<>();
                List<Layer> here = layers(far, p[0], p[1], step, b0);
                helper.assertTrue(here.equals(layers(far, p[0] + c, p[1], step, b1)) && here.equals(layers(far, p[0] - 2 * c, p[1], step, b2)),
                        "column " + p[0] + ", " + p[1] + " at step " + step + " differs one lap on");
                helper.assertTrue(b0.equals(b1) && b0.equals(b2), "the biome of column " + p[0] + ", " + p[1] + " differs one lap on");
                helper.assertTrue(here.get(0).bottom() == minY && here.get(here.size() - 1).top() == maxY,
                        "column " + p[0] + ", " + p[1] + " does not span the build range");
                for (int k = 0; k + 1 < here.size(); k++)
                    helper.assertTrue(here.get(k).top() == here.get(k + 1).bottom(), "a gap in column " + p[0] + ", " + p[1]);
                checked++;
            }
        }
        // A section across the seam: its columns east of the seam are the planet's westernmost.
        int[] columns = {0};
        int[] ground = new int[64];
        far.build(c / 2 - 32, 0, 1, 64, (relX, relZ, count, bottom, top, block, material, light, biome) -> {
            columns[0]++;
            if (relZ == 20) {
                int g = minY - 1;
                for (int k = 0; k < count; k++)
                    if (material[k] != FarColumn.AIR && material[k] != FarColumn.WATER && material[k] != FarColumn.WATER_ICE
                            && material[k] != FarColumn.SNOW_LAYER)
                        g = top[k] - 1;
                ground[relX] = g;
            }
        });
        helper.assertTrue(columns[0] == 4096, "a section has " + columns[0] + " columns");
        for (int i = 0; i < 64; i++) {
            int x = c / 2 - 32 + i, canonical = i < 32 ? x : x - c;
            int expected = far.column(canonical, 20).ground();
            helper.assertTrue(ground[i] == expected, "across the seam, column " + x + " has ground " + ground[i]
                    + ", the planet's column " + canonical + " has " + expected);
        }
        DeepTime.LOGGER.info("[gametest] farTerrainWrapsAndFillsTheColumn columns={} seamSectionColumns={}", checked, columns[0]);
        helper.succeed();
    }

    /**
     * A section is cheap and threads agree: sections built on four threads at once equal the ones
     * built alone, and the time a section takes is logged ({@code ms/section}) for full and coarse
     * detail. The only time asserted is a generous ceiling a real chunk generator could not meet
     * (a section is 16 chunks).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void farTerrainSectionsAreCheapAndThreadSafe(GameTestHelper helper) {
        FarTerrain far = far(helper);
        try {
            sectionsAreCheapAndThreadSafe(helper, far);
        } catch (java.util.concurrent.ExecutionException | java.util.concurrent.TimeoutException | InterruptedException e) {
            DeepTime.LOGGER.error("[gametest] farTerrainSectionsAreCheapAndThreadSafe threw", e);
            helper.fail("building sections on other threads failed: " + e);
        }
    }

    private static void sectionsAreCheapAndThreadSafe(GameTestHelper helper, FarTerrain far)
            throws java.util.concurrent.ExecutionException, java.util.concurrent.TimeoutException, InterruptedException {
        int c = far.data().circumference();
        int[][] jobs = new int[24][];
        for (int k = 0; k < jobs.length; k++)
            jobs[k] = new int[]{-c / 2 + (k * 7 % 24) * (c / 24) / 64 * 64, ((k * 5) % 12 - 6) * (c / 4 / 8) / 64 * 64, k % 3 == 0 ? 8 : 1};
        long[] alone = new long[jobs.length];
        for (int k = 0; k < jobs.length; k++)
            alone[k] = hash(far, jobs[k]);
        java.util.concurrent.ExecutorService pool = java.util.concurrent.Executors.newFixedThreadPool(4);
        try {
            List<java.util.concurrent.Future<long[]>> runs = new ArrayList<>();
            for (int t = 0; t < 4; t++) {
                int offset = t;
                runs.add(pool.submit(() -> {
                    long[] out = new long[jobs.length];
                    for (int n = 0; n < jobs.length; n++) {
                        int k = (n + offset * 7) % jobs.length;
                        out[k] = hash(far, jobs[k]);
                    }
                    return out;
                }));
            }
            for (var run : runs) {
                long[] got = run.get(120, java.util.concurrent.TimeUnit.SECONDS);
                for (int k = 0; k < jobs.length; k++)
                    helper.assertTrue(got[k] == alone[k], "section " + java.util.Arrays.toString(jobs[k]) + " differs on another thread");
            }
        } finally {
            pool.shutdownNow();
        }
        double[] ms = new double[2];
        for (int d = 0; d < 2; d++) {
            int step = d == 0 ? 1 : 8;
            double best = Double.MAX_VALUE;
            for (int pass = 0; pass < 3; pass++) {
                long t0 = System.nanoTime();
                for (int[] job : jobs)
                    hash(far, new int[]{job[0], job[1], step});
                best = Math.min(best, (System.nanoTime() - t0) / 1e6 / jobs.length);
            }
            ms[d] = best;
        }
        DeepTime.LOGGER.info("[gametest] farTerrainSectionsAreCheapAndThreadSafe sections={} threads=4 ms/section: detail 0 {} detail 3 {}",
                jobs.length, String.format(java.util.Locale.ROOT, "%.3f", ms[0]), String.format(java.util.Locale.ROOT, "%.3f", ms[1]));
        helper.assertTrue(ms[0] < 250.0 && ms[1] < 250.0, "a section takes " + ms[0] + " ms at full detail, " + ms[1] + " ms at detail 3");
        helper.succeed();
    }

    private static long hash(FarTerrain far, int[] job) {
        long[] h = {1125899906842597L};
        far.build(job[0], job[1], job[2], 64, (relX, relZ, count, bottom, top, block, material, light, biome) -> {
            long v = h[0];
            for (int k = 0; k < count; k++)
                v = 31 * (31 * (31 * (31 * v + bottom[k]) + top[k]) + block[k].hashCode()) + light[k];
            h[0] = 31 * v + biome.hashCode();
        });
        return h[0];
    }
}
