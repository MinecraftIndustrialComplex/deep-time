// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.core.BlockPos;
import net.minecraft.core.QuartPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.Map;
import java.util.TreeMap;

/**
 * The pack's biome machinery survives Deep Time's generator (P0-12): biome mods place their
 * biomes on raster terrain, and the pack's surface rules paint it. Runs on the world-data level
 * {@code deeptime:test_planet_data}; with -PwithMic (Terralith through Lithostitched) or
 * -PwithTerraBlender (Biomes O' Plenty through TerraBlender) the mod-specific assertions apply,
 * otherwise they are logged and pass.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class BiomeGameTests {

    private BiomeGameTests() {}

    /**
     * Surface biomes by namespace over a 64 × 64 grid of land columns (every 8 blocks), sampled from
     * the biome source at the surface, so no chunk has to load.
     */
    static Map<String, Integer> landBiomeNamespaces(ServerLevel level) {
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        var sampler = level.getChunkSource().randomState().sampler();
        var terrain = g.terrain();
        Map<String, Integer> counts = new TreeMap<>();
        int c = terrain.circumference();
        for (int i = 0; i < 64; i++) {
            for (int j = 0; j < 64; j++) {
                int x = -c / 2 + i * (c / 64) + 3;
                int z = (j - 32) * 8 + 1;
                double h = terrain.height(x, z);
                if (h <= g.getSeaLevel() + 2)
                    continue;
                var biome = g.getBiomeSource().getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock((int) h),
                        QuartPos.fromBlock(z), sampler);
                String ns = biome.unwrapKey().map(k -> k.location().getNamespace()).orElse("?");
                counts.merge(ns, 1, Integer::sum);
            }
        }
        return counts;
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void terralithBiomesOnLand(GameTestHelper helper) {
        ServerLevel level = WorldDataGameTests.level(helper);
        var counts = landBiomeNamespaces(level);
        int total = counts.values().stream().mapToInt(Integer::intValue).sum();
        int terralith = counts.getOrDefault("terralith", 0);
        boolean loaded = ModList.get().isLoaded("terralith");
        DeepTime.LOGGER.info("[gametest] terralithBiomesOnLand terralith={} of {} loaded={} byNamespace={}", terralith, total, loaded, counts);
        helper.assertTrue(total > 100, "only " + total + " land samples");
        if (loaded)
            helper.assertTrue(terralith > 0, "Terralith is loaded but none of " + total + " land samples is a Terralith biome: " + counts);
        helper.succeed();
    }

    /**
     * The pack's surface rules run on Deep Time's terrain: land columns get a painted surface (soil,
     * sand, snow, …). Peaks may legitimately stay stone, so at least half of the sampled land columns
     * must be painted; with the rules broken none would be. (On the C = 1,024 test planet the
     * analytic range covers most of the land, so there are few lowlands.)
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void surfaceRulesApplied(GameTestHelper helper) {
        ServerLevel level = WorldDataGameTests.level(helper);
        var terrain = ((GeologicChunkGenerator) level.getChunkSource().getGenerator()).terrain();
        int checked = 0, painted = 0;
        StringBuilder bare = new StringBuilder();
        for (int i = 0; i < 256 && checked < 24; i++) {
            int x = -500 + i * 37 % 1000, z = (i % 11 - 5) * 40 + 5;
            double h = terrain.height(x, z);
            if (h < 66)   // land only
                continue;
            var chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
            int y = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
            var top = chunk.getBlockState(new BlockPos(x, y, z));
            checked++;
            if (!top.is(Blocks.STONE) && !top.is(Blocks.DEEPSLATE) && !top.is(BlockTags.BASE_STONE_OVERWORLD))
                painted++;
            else if (bare.length() < 300)
                bare.append(" (").append(x).append(',').append(y).append(',').append(z).append(") ").append(top.getBlock());
        }
        DeepTime.LOGGER.info("[gametest] surfaceRulesApplied checked={} painted={}{}", checked, painted, bare);
        helper.assertTrue(checked >= 16, "only " + checked + " land columns found");
        helper.assertTrue(painted * 2 >= checked, "only " + painted + " of " + checked + " land columns are painted:" + bare);
        helper.succeed();
    }

    /** Biomes O' Plenty's regions reach Deep Time's terrain through TerraBlender (-PwithTerraBlender). */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void bopBiomesPresent(GameTestHelper helper) {
        ServerLevel level = WorldDataGameTests.level(helper);
        var counts = landBiomeNamespaces(level);
        int bop = counts.getOrDefault("biomesoplenty", 0);
        boolean loaded = ModList.get().isLoaded("biomesoplenty") && ModList.get().isLoaded("terrablender");
        DeepTime.LOGGER.info("[gametest] bopBiomesPresent biomesoplenty={} loaded={} byNamespace={}", bop, loaded, counts);
        if (loaded)
            helper.assertTrue(bop > 0, "TerraBlender and BOP are loaded but no BOP biome on land: " + counts);
        helper.succeed();
    }
}
