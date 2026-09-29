// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeCensus;
import com.minecraftindustrialcomplex.deeptime.climate.OverworldBands;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.OverworldBiomeBuilder;
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

    /**
     * Floors of {@link #climateConsistent}, per Köppen family (P1-09 acceptance: recorded at the
     * first calibrated run, standalone and MIC; see STATUS.md's P1-09 build log). A family with
     * fewer than {@link #MIN_COLUMNS} columns on the test planet is logged, not asserted.
     */
    // First calibrated runs (2026-09-29, test_planet_real, standalone / MIC): BW 0.765 / 0.824, BS 0.738 / 0.825,
    // C 0.86 / 0.86, D 1.0 / 1.0, Dsub 0.969 / 0.987, E 1.0 / 1.0, sea 1.0 / 1.0, BW:desert — / 0.588,
    // Dfc:taiga — / 0.657, E:snow 1.0 / 1.0; the small planet has no A and only 17 BW columns (logged only).
    // Floors sit about 0.1 under the lower of the two.
    static final Map<String, Double> FLOORS = Map.ofEntries(
            Map.entry("A", 0.6), Map.entry("Af", 0.6), Map.entry("BW", 0.65), Map.entry("BS", 0.6), Map.entry("C", 0.75),
            Map.entry("D", 0.9), Map.entry("Dsub", 0.85), Map.entry("E", 0.9), Map.entry("sea", 0.95),
            Map.entry("BW:desert", 0.45), Map.entry("Af:jungle", 0.5), Map.entry("Dfc:taiga", 0.5), Map.entry("E:snow", 0.9));
    static final int MIN_COLUMNS = 20;

    /**
     * P1-09: biomes follow the simulated climate. On the real baked planet ({@code test_planet_real},
     * the pack's biomes through the world's climate mapping), for each Köppen family the share of
     * surface columns whose biome carries a matching NeoForge {@code c:} tag ({@link BiomeCensus#CHECKS})
     * is logged and meets its floor.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void climateConsistent(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        var t = RealPlanetGameTests.terrain(helper, level);
        helper.assertTrue(!t.mapping().isIdentity(), "the real planet has a climate mapping (climate_mapping.json)");
        BiomeCensus.Result r;
        try {
            r = BiomeCensus.run(level, t.circumference() / 256, 32, null);
        } catch (java.io.IOException e) {
            helper.fail(e.toString());
            return;
        }
        DeepTime.LOGGER.info("[gametest] climateConsistent {}", r.json());
        StringBuilder low = new StringBuilder();
        for (var ch : BiomeCensus.CHECKS) {
            double share = r.share(ch.name());
            if (r.total(ch.name()) < MIN_COLUMNS)
                continue;
            double floor = FLOORS.getOrDefault(ch.name(), 0.0);
            if (share < floor)
                low.append(String.format(java.util.Locale.ROOT, " %s %.3f < %.3f (%d columns)", ch.name(), share, floor, r.total(ch.name())));
        }
        helper.assertTrue(low.isEmpty(), "Köppen families below their floor:" + low);
        helper.succeed();
    }

    /**
     * The reference bands the bake places values against ({@code default.json} in the jar) are
     * {@code OverworldBiomeBuilder}'s own (plan §7.2: "the band boundaries come from
     * OverworldBiomeBuilder in the 1.21.1 sources"), read from the class by reflection.
     */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void bandsAreTheBuilders(GameTestHelper helper) {
        OverworldBands jar = OverworldBands.defaults();
        try {
            var b = new OverworldBiomeBuilder();
            double[] t = edges(param(b, "temperatures"));
            double[] h = edges(param(b, "humidities"));
            double[] e = edges(param(b, "erosions"));
            String[] cont = {"mushroomFieldsContinentalness", "deepOceanContinentalness", "oceanContinentalness",
                    "coastContinentalness", "nearInlandContinentalness", "midInlandContinentalness", "farInlandContinentalness"};
            double[] c = new double[cont.length + 1];
            for (int i = 0; i < cont.length; i++) {
                Climate.Parameter p = (Climate.Parameter) field(b, cont[i]);
                c[i] = Climate.unquantizeCoord(p.min());
                c[i + 1] = Climate.unquantizeCoord(p.max());
            }
            double[] w = {0.0, (Float) staticField("VALLEY_SIZE"), (Float) staticField("LOW_START"), OverworldBiomeBuilder.HIGH_START,
                    OverworldBiomeBuilder.PEAK_START, (Float) staticField("PEAK_END")};
            OverworldBands vanilla = new OverworldBands(t, h, c, e, w);
            DeepTime.LOGGER.info("[gametest] bandsAreTheBuilders builder={} jar={}", vanilla.toJson(), jar.toJson());
            for (int i = 0; i < 6; i++) {
                helper.assertTrue(Math.abs(t[i] - jar.temperature()[i]) < 1e-4, "temperature edge " + i);
                helper.assertTrue(Math.abs(h[i] - jar.vegetation()[i]) < 1e-4, "humidity edge " + i);
                helper.assertTrue(Math.abs(w[i] - jar.weirdness()[i]) < 1e-4, "weirdness edge " + i);
            }
            for (int i = 0; i < 8; i++) {
                helper.assertTrue(Math.abs(e[i] - jar.erosion()[i]) < 1e-4, "erosion edge " + i);
                helper.assertTrue(Math.abs(c[i] - jar.continents()[i]) < 1e-4, "continentalness edge " + i);
            }
        } catch (ReflectiveOperationException ex) {
            helper.fail("cannot read OverworldBiomeBuilder: " + ex);
        }
        helper.succeed();
    }

    private static Object field(Object o, String name) throws ReflectiveOperationException {
        var f = OverworldBiomeBuilder.class.getDeclaredField(name);
        f.setAccessible(true);
        return f.get(o);
    }

    private static Object staticField(String name) throws ReflectiveOperationException {
        var f = OverworldBiomeBuilder.class.getDeclaredField(name);
        f.setAccessible(true);
        return f.get(null);
    }

    private static Climate.Parameter[] param(Object b, String name) throws ReflectiveOperationException {
        return (Climate.Parameter[]) field(b, name);
    }

    /** The edges of consecutive spans. */
    private static double[] edges(Climate.Parameter[] ps) {
        double[] out = new double[ps.length + 1];
        for (int i = 0; i < ps.length; i++) {
            out[i] = Climate.unquantizeCoord(ps[i].min());
            out[i + 1] = Climate.unquantizeCoord(ps[i].max());
        }
        return out;
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
