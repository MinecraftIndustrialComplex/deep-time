// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import com.minecraftindustrialcomplex.deeptime.climate.VolcanicCensus;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.Geothermal;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.block.state.BlockState;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.List;

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
     * {@link #MAX_SPECK_SHARE} of their surface cells stand alone). The test planet always has active
     * volcanoes (arcs on its subduction margins) and hot-spring pools (its preset's heat-flow fields), so
     * where the level places volcanic biomes it must place some for the test to read: none fails (Ben's
     * rule, decisions log 2026-10-01). A profile without volcanic biomes (vanilla) has nothing of the
     * kind to place; it passes with a line saying so, and {@link #volcanicBiomesChangeNothingWithoutThem}
     * checks that profile.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void volcanicBiomesOnlyAtVolcanoes(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        VolcanicCensus.Result r = census(helper, level);
        DeepTime.LOGGER.info("[gametest] volcanicBiomesOnlyAtVolcanoes {}", r.json());
        boolean places = placesVolcanicBiomes(level);
        int activeVolcanoes = r.json().get("active_volcanoes").getAsInt();
        DeepTime.LOGGER.info("[gametest] volcanicBiomesOnlyAtVolcanoes filters: activeVolcanoes={} pools={} zonesByRole={} "
                        + "landColumns={} volcanicColumns={} volcanicCells={} levelPlacesThem={}", activeVolcanoes, r.pools(),
                zoneCounts(r), r.now().columns, r.now().volcanic, r.now().volcanicCells, places);
        helper.assertTrue(activeVolcanoes > 0 && r.pools() > 0, "the test planet has " + activeVolcanoes + " active volcanoes and "
                + r.pools() + " hot-spring pools: it should always have both (arcs on its subduction margins, heat-flow fields)");
        if (!places) {
            DeepTime.LOGGER.info("[gametest] volcanicBiomesOnlyAtVolcanoes: this profile has no volcanic biome to place "
                    + "(volcanicBiomesChangeNothingWithoutThem checks it)");
            helper.succeed();
            return;
        }
        helper.assertTrue(r.now().volcanic > 0 && r.now().volcanicCells > 0, "the level places volcanic biomes but none of the "
                + r.now().columns + " land columns sampled is in one (zones " + zoneCounts(r) + ")");
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
     * ({@code #deeptime:volcanic/hot_spring_field}: Terralith's yellowstone, Biomes O' Plenty's hot springs), but a
     * pool within {@link VolcanicBiomes#PLAYA_MARGIN} of a playa, which stands in the flat's biome (BIO-165); and
     * whatever the biome, no pool's water is frozen: the surface block of every pool column is water, not ice, and a
     * cold biome's freezing rule (vanilla's {@code Biome.shouldFreeze} of snowy plains, asked at the surface of each
     * pool column) does not ice it (a hot spring's water stays open: GEO-062; before it, Biomes O' Plenty's hot springs,
     * temperature 0.17, froze a pool over at generation). The test planet always has pools (its preset's heat-flow
     * fields), so a planet without any fails; a pack without such a biome has nothing to check of the first and
     * passes it with a line saying so, and the second runs in every profile.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void hotSpringPoolsSitInTheirBiome(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        VolcanicCensus.Result r = census(helper, level);
        String role = r.roles().get(VolcanicZones.Role.HOT_SPRING_FIELD.id);
        DeepTime.LOGGER.info("[gametest] hotSpringPoolsSitInTheirBiome pools={} byPlaya={} inRole={} (plain search: {}) role={} roles={}", r.pools(),
                r.poolsByPlaya(), r.now().poolsInRole, r.plain().poolsInRole, role, r.roles());
        helper.assertTrue(r.pools() > 0, "the test planet has no hot-spring pool: its preset's heat-flow fields should always "
                + "place some");
        if ("-".equals(role)) {
            DeepTime.LOGGER.info("[gametest] hotSpringPoolsSitInTheirBiome: the pack has no hot-spring biome; no biome to check");
        } else {
            // A pool by a playa stands in the flat's biome (BIO-165: no surface role within PLAYA_MARGIN of a playa).
            int away = r.pools() - r.poolsByPlaya();
            helper.assertTrue(r.now().poolsInRole == away, (away - r.now().poolsInRole) + " of " + away
                    + " hot-spring pools away from playas are not in " + role);
        }
        poolWaterStaysOpen(helper, level);
    }

    /**
     * Reads the surface of every pool's water, a few pools a tick ({@link ChunkSampling}): never ice, and never frozen
     * by a cold biome's rule. Ends the test.
     */
    private static void poolWaterStaysOpen(GameTestHelper helper, ServerLevel level) {
        var t = RealPlanetGameTests.terrain(helper, level);
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        var settings = g.generatorSettings().value();
        Geothermal geo = t.geothermal(g.waterReach(), settings.noiseSettings().minY(), settings.seaLevel());
        List<Geothermal.Site> pools = geo.sites().stream().filter(s -> s.kind() == Geothermal.Kind.POOL).toList();
        Biome cold = level.registryAccess().registryOrThrow(Registries.BIOME).getOrThrow(Biomes.SNOWY_PLAINS);
        int c = t.circumference();
        // 0 columns read; 1 of them water, 2 ice, 3 something else; 4 columns (any of these) in a biome cold enough to snow there; 5, 6 water
        // surfaces the cold biome's freezing rule was asked about and said yes to (must be none); 7, 8 the same rule one block
        // under a surface that is also water, and its yeses (must be some: the probe can say yes).
        int[] n = new int[9];
        StringBuilder detail = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        ChunkSampling.forEach(helper, pools.size(), i -> {
            Geothermal.Site s = pools.get(i);
            for (int dz = -s.reach(); dz <= s.reach(); dz++)
                for (int dx = -s.reach(); dx <= s.reach(); dx++) {
                    int x = Math.floorMod(s.x() + dx + c / 2, c) - c / 2, z = s.z() + dz;
                    if (geo.claim(x, z) != s || !s.part(geo.dx(x, s), z - s.z()).pool())
                        continue;
                    BlockState top = level.getChunk(x >> 4, z >> 4).getBlockState(pos.set(x, s.level(), z));
                    n[0]++;
                    if (level.getBiome(pos).value().coldEnoughToSnow(pos))
                        n[4]++;
                    if (top.is(BlockTags.ICE)) {
                        n[2]++;
                        if (detail.length() < 600)
                            detail.append(" iced(").append(x).append(',').append(s.level()).append(',').append(z).append(' ')
                                    .append(level.getBiome(pos).unwrapKey().map(k -> k.location().toString()).orElse("?")).append(')');
                    } else if (top.getFluidState().is(FluidTags.WATER)) {
                        n[1]++;
                        n[5]++;
                        if (cold.shouldFreeze(level, pos, false))
                            n[6]++;
                        BlockPos under = pos.below();
                        if (level.getFluidState(under).is(FluidTags.WATER)) {
                            n[7]++;
                            if (cold.shouldFreeze(level, under, false))
                                n[8]++;
                        }
                    } else {
                        n[3]++;
                        if (detail.length() < 600)
                            detail.append(" not-water(").append(x).append(',').append(s.level()).append(',').append(z).append(' ')
                                    .append(top.getBlock().getDescriptionId()).append(')');
                    }
                }
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] hotSpringPoolsSitInTheirBiome pool water: pools={} columns={} (inColdBiome={}) water={} iced={} other={} "
                    + "coldBiomeFreezes={} of {} surfaces; control one block under: {} of {}{}", pools.size(), n[0], n[4], n[1], n[2], n[3], n[6], n[5],
                    n[8], n[7], detail);
            helper.assertTrue(n[0] > 0 && n[1] > 0, "no pool water was read: " + n[0] + " pool columns, " + n[1] + " of water");
            helper.assertTrue(n[2] == 0, n[2] + " of " + n[0] + " pool columns have ice on the water:" + detail);
            helper.assertTrue(n[6] == 0, n[6] + " of " + n[5] + " pool water surfaces freeze in a cold biome (snowy plains' rule): a hot "
                    + "spring's water stays open");
            helper.assertTrue(n[8] > 0, "the cold biome froze none of " + n[7] + " control blocks of water, so the probe cannot say yes");
            helper.succeed();
        });
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

    /** Whether the level places volcanic biomes (the pack has some and the level's layer is on). */
    private static boolean placesVolcanicBiomes(ServerLevel level) {
        return level.getChunkSource().randomState().sampler().spawnTarget() instanceof RegionSpawnTarget m && m.volcanic().active();
    }

    /** The census's zones as role=count, for the logs. */
    private static String zoneCounts(VolcanicCensus.Result r) {
        StringBuilder b = new StringBuilder("{");
        r.zones().forEach((role, n) -> b.append(b.length() > 1 ? ", " : "").append(role).append('=').append(n[0]));
        return b.append('}').toString();
    }
}
