// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.LavaRules;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Where the generator puts lava (Ben's preview of 2026-10-01, "huge bodies of lava"): on
 * {@code deeptime:test_planet_caves}, the real test planet with the full router (noise caves and
 * aquifers, {@code deeptime:planet}) under a biome with no carvers and no features, so every fluid
 * block read back is the aquifer's or the global fluid picker's.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class LavaGameTests {

    private LavaGameTests() {}

    /** A sample site: a column and why it was chosen. */
    private record Site(String what, int x, int z, double h) {}

    /** Chunks per side generated around each site. */
    private static final int SIDE = 2;

    /**
     * Lava is measured from the world's floor, not from y 0 ({@link LavaRules}). Vanilla's two
     * hard-coded heights (lava below y −54, lava lakes at y −10 and lower) put lava in every cave
     * and every stretch of sea below y −54 of a world whose floor is y −352. Chunks are generated
     * around the planet's deepest sea, a sea about 150 blocks deep, a coast and high ground, and
     * every block is read back:
     * <ul>
     *   <li>{@code lavaAboveLakes}: no lava above the highest level a lava lake can have
     *       ({@link LavaRules#highestLakeY}, y −288 by default). The old rule put it up to y −55;</li>
     *   <li>{@code seaOnLava}: no sea column whose water stands on lava (the first block under the
     *       water is never lava). The old rule turned the sea below y −54 into lava;</li>
     *   <li>{@code floorNotLava}: under ground that stands clear of it, open space below the lava
     *       floor ({@link LavaRules#floodY}, y −342) is lava, as in vanilla's bottom ten blocks;</li>
     *   <li>the premises: a sea bed below y −60 was sampled ({@code deepSeaColumns}), and open space
     *       exists between the lava lakes and y −54 ({@code openDeep}: caves the old rule flooded).</li>
     * </ul>
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void lavaStaysAtTheFloor(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_CAVES.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_caves is not generating from world data");
            return;
        }
        var settings = g.generatorSettings().value();
        helper.assertTrue(settings.aquifersEnabled(), "test_planet_caves must run the full router (aquifers on)");
        int minY = settings.noiseSettings().minY(), sea = settings.seaLevel();
        LavaRules rules = t.lava();
        int flood = rules.floodY(minY, sea), lakeTop = Math.max(rules.highestLakeY(minY), flood - 1);
        helper.assertTrue(flood < -54 - 100, "the lava floor (y " + flood + ") must follow the world's floor (y " + minY + ")");

        // Sites from the world data: the deepest sea, a sea about 150 blocks deep, a coast, high ground.
        int c = t.circumference();
        Site deepest = null, deep = null, coast = null, high = null;
        for (int z = -c / 4 + 64; z < c / 4 - 64; z += 16)
            for (int x = -c / 2; x < c / 2; x += 16) {
                double h = t.height(x, z);
                if (deepest == null || h < deepest.h)
                    deepest = new Site("deepest sea", x, z, h);
                if (h < -60 && (deep == null || Math.abs(h + 90) < Math.abs(deep.h + 90)))
                    deep = new Site("deep sea", x, z, h);
                if (coast == null || Math.abs(h - sea) < Math.abs(coast.h - sea))
                    coast = new Site("coast", x, z, h);
                if (high == null || h > high.h)
                    high = new Site("high ground", x, z, h);
            }
        List<Site> sites = new ArrayList<>(List.of(deepest, coast, high));
        if (deep != null)
            sites.add(deep);
        Site deepestSite = deepest;

        long[] n = new long[8];   // lava, lavaAboveLakes, seaOnLava, floorOpen, floorNotLava, openDeep, deepSeaColumns, columns
        int[] topLava = {Integer.MIN_VALUE};
        StringBuilder detail = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        ChunkSampling.forEach(helper, sites.size() * SIDE * SIDE, i -> {
            Site s = sites.get(i / (SIDE * SIDE));
            int k = i % (SIDE * SIDE);
            int cx = (s.x >> 4) + k % SIDE, cz = (s.z >> 4) + k / SIDE;
            ChunkAccess chunk = level.getChunkSource().getChunk(cx, cz, ChunkStatus.NOISE, true);
            for (int dx = 0; dx < 16; dx++)
                for (int dz = 0; dz < 16; dz++) {
                    int x = (cx << 4) + dx, z = (cz << 4) + dz;
                    double h = t.height(x, z);
                    n[7]++;
                    if (h < -60)
                        n[6]++;
                    boolean clear = rules.columnFloodY(minY, sea, h) == flood && h > flood + 32;
                    boolean inSea = false;   // walking down through the sea's open water
                    for (int y = sea - 1; y >= minY; y--) {
                        BlockState b = chunk.getBlockState(pos.set(x, y, z));
                        boolean lava = b.is(Blocks.LAVA), water = b.is(Blocks.WATER);
                        if (y == sea - 1)
                            inSea = water && h < sea;
                        else if (inSea && !water) {
                            if (lava) {
                                n[2]++;
                                if (detail.length() < 400)
                                    detail.append(" seaOnLava(").append(x).append(',').append(y).append(',').append(z).append(')');
                            }
                            inSea = false;
                        }
                        if (lava) {
                            n[0]++;
                            topLava[0] = Math.max(topLava[0], y);
                            if (y > lakeTop) {
                                n[1]++;
                                if (detail.length() < 400)
                                    detail.append(" lava(").append(x).append(',').append(y).append(',').append(z).append(')');
                            }
                        }
                        boolean open = b.isAir() || !b.getFluidState().isEmpty();
                        if (open && y < -54 && y > lakeTop && !inSea)
                            n[5]++;
                        if (open && y < flood && clear) {
                            n[3]++;
                            if (!lava)
                                n[4]++;
                        }
                    }
                }
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] lavaStaysAtTheFloor rules={} minY={} lavaFloorY={} highestLakeY={} sites={} columns={} "
                            + "deepSeaColumns={} (deepest bed y {}) lava={} topLavaY={} lavaAboveLakes={} seaOnLava={} openDeep={} "
                            + "floorOpen={} floorNotLava={}{}",
                    rules, minY, flood, lakeTop, sites, n[7], n[6], (int) Math.floor(deepestSite.h), n[0],
                    topLava[0] == Integer.MIN_VALUE ? "-" : topLava[0], n[1], n[2], n[5], n[3], n[4], detail);
            helper.assertTrue(n[6] > 0, "no sea bed below y -60 was sampled (deepest: y " + deepestSite.h
                    + "): the planet has no sea the old lava level would have reached");
            helper.assertTrue(n[5] > 0, "no open space between y " + lakeTop + " and y -54 in the sampled chunks: nothing the old lava level would have flooded");
            helper.assertTrue(n[1] == 0, n[1] + " lava blocks above y " + lakeTop + " (the highest a lava lake may stand):" + detail);
            helper.assertTrue(n[2] == 0, n[2] + " sea columns stand on lava:" + detail);
            helper.assertTrue(n[4] == 0, n[4] + " of " + n[3] + " open blocks below the lava floor (y " + flood + ") are not lava");
            helper.succeed();
        });
    }
}
