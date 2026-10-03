// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import com.minecraftindustrialcomplex.deeptime.climate.VolcanicCensus;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * The pack's volcanic biomes keep to real volcanoes (Ben, decisions log 2026-10-03, "Tie them to real
 * volcanoes (Recommended)"), on the real baked planet {@code deeptime:test_planet_real} with the profile's
 * biomes: Terralith's with -PwithMic, Biomes O' Plenty's with -PwithTerraBlender, vanilla's alone. Each test
 * logs {@link VolcanicCensus}'s numbers on a {@code [gametest]} line, the plain climate search's (master's
 * placement) beside the level's.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class VolcanicBiomeGameTests {

    private VolcanicBiomeGameTests() {}

    /** A specky placement: more than this share of its volcanic surface cells have no neighbour of their biome. */
    static final double MAX_SPECK_SHARE = 0.05; // ledger: BIO-114 (game-choice)

    private static VolcanicCensus.Result census(GameTestHelper helper, ServerLevel level) {
        var t = RealPlanetGameTests.terrain(helper, level);
        return VolcanicCensus.run(level, Math.max(4, t.circumference() / 512), 32);
    }

    /**
     * No volcanic biome lies more than {@link VolcanicCensus#FAR} blocks from an active volcano's summit or a
     * hot-spring pool, nor more than {@link VolcanicCensus#OUTSIDE} blocks outside the zones the volcanoes
     * give them; the biomes that take the kept-out biomes' places are land biomes (no ocean, river or cave
     * biome at the surface of land); and the placed biomes are areas, not specks (at most
     * {@link #MAX_SPECK_SHARE} of their surface cells stand alone). On every profile; without volcanic
     * biomes (vanilla) the counts are all 0 and it passes with a line saying so.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void volcanicBiomesOnlyAtVolcanoes(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        VolcanicCensus.Result r = census(helper, level);
        DeepTime.LOGGER.info("[gametest] volcanicBiomesOnlyAtVolcanoes {}", r.json());
        helper.assertTrue(r.now().far == 0, r.now().far + " of " + r.now().volcanic + " volcanic-biome columns lie more than "
                + VolcanicCensus.FAR + " blocks from every active volcano and hot-spring pool:" + r.now().examples);
        helper.assertTrue(r.now().outside == 0, r.now().outside + " of " + r.now().volcanic + " volcanic-biome columns lie more than "
                + VolcanicCensus.OUTSIDE + " blocks outside every volcanic zone");
        helper.assertTrue(r.holes() == 0, r.holes() + " columns that lost a volcanic biome took an ocean, river or cave biome: " + r.replaced());
        int cells = r.now().volcanicCells, specks = r.now().specks;
        helper.assertTrue(specks <= Math.max(2, MAX_SPECK_SHARE * cells), specks + " of " + cells
                + " volcanic surface cells stand alone (more than " + MAX_SPECK_SHARE + ")");
        helper.succeed();
    }

    /**
     * Every hot-spring pool's centre lies in the hot-spring field's biome when the pack has one
     * ({@code #deeptime:volcanic/hot_spring_field}: Terralith's yellowstone, Biomes O' Plenty's hot springs). A
     * planet without pools, or a pack without such a biome, passes with a line saying so.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void hotSpringPoolsSitInTheirBiome(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        VolcanicCensus.Result r = census(helper, level);
        String role = r.roles().get(VolcanicZones.Role.HOT_SPRING_FIELD.id);
        DeepTime.LOGGER.info("[gametest] hotSpringPoolsSitInTheirBiome pools={} inRole={} (plain search: {}) role={} roles={}", r.pools(),
                r.now().poolsInRole, r.plain().poolsInRole, role, r.roles());
        if ("-".equals(role) || r.pools() == 0) {
            DeepTime.LOGGER.info("[gametest] hotSpringPoolsSitInTheirBiome: {}; nothing to check",
                    r.pools() == 0 ? "this planet has no hot-spring pool" : "the pack has no hot-spring biome");
            helper.succeed();
            return;
        }
        helper.assertTrue(r.now().poolsInRole == r.pools(), (r.pools() - r.now().poolsInRole) + " of " + r.pools()
                + " hot-spring pools are not in " + role);
        helper.succeed();
    }

    /**
     * Without volcanic biomes (vanilla: nothing in {@code #deeptime:volcanic} or its role tags can be placed), the
     * level answers every biome query as before: every sampled column's biome is the plain search's and the level
     * says it has nothing to do. With them, logged only.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void volcanicBiomesChangeNothingWithoutThem(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        var registry = level.registryAccess().registryOrThrow(Registries.BIOME);
        boolean any = false;
        for (Holder<Biome> b : g.getBiomeSource().possibleBiomes())
            any |= b.is(VolcanicBiomes.KEEP_OUT);
        for (VolcanicZones.Role role : VolcanicZones.Role.values())
            for (Holder<Biome> b : registry.getTag(VolcanicBiomes.tag(role)).map(s -> (Iterable<Holder<Biome>>) s).orElse(java.util.List.of()))
                any |= g.getBiomeSource().possibleBiomes().contains(b);
        VolcanicCensus.Result r = census(helper, level);
        boolean active = level.getChunkSource().randomState().sampler().spawnTarget() instanceof RegionSpawnTarget m && m.volcanic().active();
        DeepTime.LOGGER.info("[gametest] volcanicBiomesChangeNothingWithoutThem packHasVolcanicBiomes={} active={} columns={} changed={}",
                any, active, r.now().columns, r.differ());
        if (any) {
            helper.assertTrue(active, "the pack has volcanic biomes but the level does not place them");
            helper.succeed();
            return;
        }
        helper.assertTrue(!active, "the level's volcanic biome layer is active without a volcanic biome in the pack");
        helper.assertTrue(r.differ() == 0, r.differ() + " of " + r.now().columns + " columns changed biome without volcanic biomes");
        helper.assertTrue(r.now().columns > 100, "only " + r.now().columns + " land columns sampled");
        helper.succeed();
    }
}
