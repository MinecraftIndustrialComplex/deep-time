// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.RegionCensus;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * TerraBlender's regions follow the geologic provinces (P1-10, plan §7.3). The assertions apply
 * with {@code -PwithTerraBlender} (TerraBlender and Biomes O' Plenty in the run); in the other
 * profiles the test checks that Deep Time leaves the biome source alone and passes.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class TerraBlenderGameTests {

    private TerraBlenderGameTests() {}

    /**
     * On the real baked planet ({@code test_planet_real}): over a 96 × 48 grid of surface columns,
     * every column of a province has the one region the world's table gives it (the file's region
     * name resolved to TerraBlender's index), and the biome the level's biome source places there
     * is one that region offers (or vanilla's, TerraBlender's deferred placeholder). TerraBlender's
     * own tessellation would put Biomes O' Plenty biomes in provinces whose region is vanilla's.
     * The planet must have several provinces on at least two different regions, and some land under
     * a mod region, or the check proves nothing.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void regionsFollowProvinces(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        RealPlanetGameTests.terrain(helper, level);
        boolean loaded = ModList.get().isLoaded("terrablender");
        if (!loaded) {
            helper.assertTrue(RegionCensus.sourceOf(level).isEmpty(),
                    "TerraBlender is not installed but the level carries a province → region source");
            DeepTime.LOGGER.info("[gametest] regionsFollowProvinces loaded=false (no TerraBlender: nothing to follow)");
            helper.succeed();
            return;
        }
        helper.assertTrue(RegionCensus.sourceOf(level).isPresent(),
                "TerraBlender is installed but the real planet has no province → region source (climate_mapping.json regions)");
        TerraBlenderChecks.Result r = TerraBlenderChecks.run(level, 96, 48);
        DeepTime.LOGGER.info("[gametest] regionsFollowProvinces samples={} provinces={} regions={} provincesByRegion={} "
                        + "landUnderModRegions={} landByNamespace={} outside={} mismatch={}",
                r.samples(), r.provincesSeen(), r.regionsSeen(), r.provincesByRegion(), r.landSamplesInModRegions(),
                r.landByNamespace(), r.outsideItsRegion(), r.perProvinceMismatch());
        DeepTime.LOGGER.info("[gametest] regionsFollowProvinces generated chunks: {} sampled, {} with a mod biome, {} outside their region",
                r.chunkSamples(), r.chunkModBiomes(), r.chunkOutside());
        helper.assertTrue(r.chunkOutside() == 0, "a generated chunk's biome is outside its province's region: " + r.firstProblem());
        helper.assertTrue(r.perProvinceMismatch() == 0, "regions differ inside a province: " + r.firstProblem());
        helper.assertTrue(r.outsideItsRegion() == 0, "a biome outside its province's region: " + r.firstProblem());
        helper.assertTrue(r.provincesSeen() >= 3, "only " + r.provincesSeen() + " provinces sampled");
        helper.assertTrue(r.regionsSeen() >= 2, "every sampled province has the same region: " + r.provincesByRegion());
        helper.assertTrue(r.landSamplesInModRegions() > 0, "no land under a mod region: " + r.provincesByRegion());
        helper.succeed();
    }
}
