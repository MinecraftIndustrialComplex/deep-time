// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.GeologyCensus;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * The geology gate (Ben, decisions log 2026-10-04, "Deep Time-side fixes (Recommended)"), on the real baked planet
 * {@code deeptime:test_planet_real} with the profile's biomes: with -PwithMic, Terralith's rock-named biomes (Shield,
 * Yosemite, the White, Granite and Basalt Cliffs, the red-bed canyons, Stony Spires) stand only on their ground; in
 * vanilla nothing changes. Each test logs {@link GeologyCensus}'s numbers, the plain climate search's beside the
 * level's.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class GeologyBiomeGameTests {

    private GeologyBiomeGameTests() {}

    private static GeologyCensus.Result census(GameTestHelper helper, ServerLevel level) {
        var t = RealPlanetGameTests.terrain(helper, level);
        return GeologyCensus.run(level, Math.max(4, t.circumference() / 512), 32);
    }

    /**
     * No rock-named biome stands on ground without one of its roles (a shield province, exposed granite, a coast of
     * carbonate, granite or basalt, dry sedimentary rock, humid carbonate), and the columns that lost one took a land
     * biome (no ocean, river or cave biome at the surface of land). Without rock-named biomes (vanilla) the counts are 0
     * and it passes with a line saying so.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void rockNamedBiomesSitOnTheirGround(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        GeologyCensus.Result r = census(helper, level);
        DeepTime.LOGGER.info("[gametest] rockNamedBiomesSitOnTheirGround {}", r.json());
        if (r.gatedBiomes() == 0) {
            DeepTime.LOGGER.info("[gametest] rockNamedBiomesSitOnTheirGround: this pack has no rock-named biome; nothing to check");
            helper.succeed();
            return;
        }
        helper.assertTrue(r.now().offGround == 0, r.now().offGround + " of " + r.now().rockNamed
                + " rock-named biome columns stand off their ground:" + r.now().examples);
        helper.assertTrue(r.holes() == 0, r.holes() + " land columns that lost a rock-named biome took an ocean, river or cave biome: "
                + r.replaced());
        helper.succeed();
    }

    /**
     * Without rock-named biomes in the pack (vanilla) the gate changes nothing: every sampled column's biome is the plain
     * search's but where another biome rule decided it (the snow line binds in vanilla, BIO-180). With them, logged only
     * (the volcanic layer's own test covers that layer's changes).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void geologyGateChangesNothingWithoutThem(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        GeologyCensus.Result r = census(helper, level);
        DeepTime.LOGGER.info("[gametest] geologyGateChangesNothingWithoutThem gated={} columns={} changed={} changedByRules={} rockNamedPlain={}",
                r.gatedBiomes(), r.now().columns, r.differ(), r.differByRules(), r.plain().rockNamed);
        if (r.gatedBiomes() > 0) {
            helper.succeed();
            return;
        }
        helper.assertTrue(r.plain().rockNamed == 0 && r.now().rockNamed == 0, "rock-named biomes counted in a pack without them");
        // The plain search also leaves out the volcanic layer, which acts where the pack has volcanic biomes (Biomes O'
        // Plenty), and the other biome rules (the snow line, peatland): only without the layer, and apart from the rules'
        // columns, is every column the plain search's. Without rock-named biomes no rule of the gate binds.
        boolean volcanic = level.getChunkSource().randomState().sampler().spawnTarget()
                instanceof com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget m && m.volcanic().volcanicActive();
        if (!volcanic)
            helper.assertTrue(r.differ() == r.differByRules(), (r.differ() - r.differByRules()) + " of " + r.now().columns
                    + " columns changed biome without rock-named biomes, other than by another biome rule");
        helper.assertTrue(r.now().columns > 100, "only " + r.now().columns + " land columns sampled");
        helper.succeed();
    }
}
