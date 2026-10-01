// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeCensus;
import com.minecraftindustrialcomplex.deeptime.climate.OverworldBands;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.IcePass;
import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.OverworldBiomeBuilder;
import net.minecraft.core.BlockPos;
import net.minecraft.core.QuartPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.chunk.ProtoChunk;
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
     * first calibrated run, standalone and MIC; see STATUS.md's P1-09 build log; since 2026-10-01
     * the lowest share over ten planets less 0.1, see below). A family with fewer than
     * {@link #MIN_COLUMNS} columns on the test planet is logged, not asserted.
     */
    // First calibrated runs (2026-09-29, test_planet_real, standalone / MIC): BW 0.765 / 0.824, BS 0.738 / 0.825,
    // C 0.86 / 0.86, D 1.0 / 1.0, Dsub 0.969 / 0.987, E 1.0 / 1.0, sea 1.0 / 1.0, BW:desert — / 0.588,
    // Dfc:taiga — / 0.657, E:snow 1.0 / 1.0; the small planet has no A and only 17 BW columns (logged only).
    // Floors sit about 0.1 under the lower of the two.
    //
    // Re-set 2026-09-30 after the deep-time climate (carbon cycle, brightening sun): the 4k test planet is
    // land-poor (~8 % land), so its weathering balances degassing only at 2,949 ppm and it ends at 23 °C
    // instead of the fixed 280 ppm and 15.6 °C it had. Its Köppen columns moved: E 1,765 → 319, D 1,296 → 1,964,
    // BS 274 → 792, C 279 → 521, BW 25 → 87. The mapping did not change: with the old climate pinned
    // (carbon cycle, sun, ice window, glacio-eustasy, hydrological factor off) the same code passes, standalone
    // and MIC. The lower shares are where the new columns sit: cold coasts whose shore biome is stony shore
    // (ET 17 %, BWh 24 % standalone), Terralith's alpine highlands (26 %) and yellowstone (23 %) on the Dfc
    // uplands that were ET before, moonlight valley and gravel desert on BSk. New runs (standalone / MIC):
    // BW 0.793 / 0.931, BS 0.610 / 0.519, C 0.881 / 0.891, D 1.0 / 1.0, Dsub 0.961 / 0.970, E 1.0 / 0.991,
    // BW:desert 0.690 / 0.816, Dfc:taiga 0.952 / 0.435, E:snow 0.824 / 0.991. The three below their floors
    // take the same rule (0.1 under the lower); the others keep theirs, which the rule would only raise.
    // P1-21 (volcano branch, before this re-set, not combined with it) had measured E:snow 0.899
    // standalone with the volcano stage and EF's barren frozen ground; the lower floor above covers both.
    //
    // Re-set 2026-10-01 over several planets instead of one. The simulation is not bit-identical across
    // platforms (the platform's libm), so the same preset is another planet on x86_64 Linux than on the
    // aarch64 Mac the floors above were read from, and a floor read from one planet is that planet's number,
    // not the mapping's: the Linux planet failed A 0.553 < 0.6 (its Aw and As columns are 63–71 % forest) and
    // Dsub 0.812 < 0.85 with no change to the mapping. Measured on x86_64 Linux with
    // DEEPTIME_TEST_PLANET_SET="seed=N" ./gametest.sh [-PwithMic], seeds 11–16 standalone and 11–14 with MIC
    // (ten censuses; a family counts in a census that has at least MIN_COLUMNS of its columns). Lowest and
    // highest share:
    //   A 0.553–0.944, Af 0.642–0.963, BW 0.675–0.925, BS 0.623–0.744, C 0.849–0.968, D 0.984–1.0,
    //   Dsub 0.659–0.882, E 0.922–1.0, sea 1.0, BW:desert 0.475–0.827, Af:jungle 0.515–0.774,
    //   Dfc:taiga 0.563–0.822, E:snow 0.849–0.998.
    // A floor is 0.1 under the lowest share measured on any planet, the Mac's above included (so BS,
    // Dfc:taiga and E:snow keep theirs); it catches a broken mapping (shares fall to the 0.1–0.3 a random
    // biome gives), not a planet whose climate zones sit elsewhere. MIN_COLUMNS went from 20 to 40: a share
    // of twenty columns moves by 0.05 with one column.
    //
    // Later the same day the planets changed again (the surface moves with its own crust: lower test
    // planets). Seeds 11–16 standalone and 11 with MIC on the new ones: every family with 40 columns is
    // above its floor except Dsub, 0.481 on seed 14 (266 columns; its Dfc:taiga is 0.474), so Dsub goes
    // to 0.38 by the same rule. A floor that needs re-reading whenever the simulation changes is a weak
    // test; it stays as a guard against a broken mapping only.
    static final Map<String, Double> FLOORS = Map.ofEntries(
            Map.entry("A", 0.45), Map.entry("Af", 0.54), Map.entry("BW", 0.57), Map.entry("BS", 0.42), Map.entry("C", 0.75),
            Map.entry("D", 0.88), Map.entry("Dsub", 0.38), Map.entry("E", 0.82), Map.entry("sea", 0.95),
            Map.entry("BW:desert", 0.37), Map.entry("Af:jungle", 0.41), Map.entry("Dfc:taiga", 0.33), Map.entry("E:snow", 0.72));
    static final int MIN_COLUMNS = 40;
    /** The most EF columns that may land on the red badlands or scarlet mountains (the peaks of negative weirdness keep Terralith's). */
    static final double FORBIDDEN_CEILING = 0.10;

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
        // EF is ice or barren frozen ground, never the red badlands or Nether-stone mountains.
        StringBuilder red = new StringBuilder();
        for (var f : BiomeCensus.FORBIDDEN) {
            double share = r.forbiddenShare(f.name());
            DeepTime.LOGGER.info("[gametest] climateConsistent {} {} of {} columns", f.name(), share, r.forbiddenTotal(f.name()));
            if (r.forbiddenTotal(f.name()) >= MIN_COLUMNS && share > FORBIDDEN_CEILING)
                red.append(String.format(java.util.Locale.ROOT, " %s %.3f > %.3f (%d columns)", f.name(), share, FORBIDDEN_CEILING,
                        r.forbiddenTotal(f.name())));
        }
        helper.assertTrue(red.isEmpty(), "families on biomes they must not have:" + red);
        helper.succeed();
    }

    /**
     * Ben, 2026-09-30, "Force snow/ice cover": where the bake's ice cover says so, Deep Time's own
     * surface pass ({@code IcePass}) lays snow, packed ice and blue ice over the top of the ground
     * whatever biome the pack drew. On the real baked planet, columns across its ice cover (one per
     * 16 blocks, about two dozen spread over them, each in its own chunk at the surface status) have
     * the table's blocks from the ground's top down, as many as the cover is thick.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void iceCoverLaidOnGlaciatedGround(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        var t = RealPlanetGameTests.terrain(helper, level);
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        WorldData wd = t.data();
        var cover = wd.iceCover();
        helper.assertTrue(cover.isPresent(), "the real planet has an ice cover (climate/ice_cover)");
        IceCover ice = cover.get();
        // (The 150 Myr test planet may have no ground above its equilibrium line at all: then only the
        // layer and the synthetic chunk of icePassLaysTheCover are checked.)
        int c = t.circumference(), sea = g.getSeaLevel();
        java.util.List<int[]> columns = new java.util.ArrayList<>();
        for (int z = -c / 4 + 8; z < c / 4; z += 16)
            for (int x = -c / 2 + 8; x < c / 2; x += 16) {
                int n = ice.blocks(x, z);
                if (n > 0 && t.height(x, z) > sea + 2)
                    columns.add(new int[] {x, z, n});
            }
        int stride = Math.max(1, columns.size() / 24);
        int checked = 0, wet = 0, thick = 0;
        StringBuilder bad = new StringBuilder();
        java.util.Set<Long> chunks = new java.util.HashSet<>();
        for (int k = 0; k < columns.size() && checked < 24; k += stride) {
            int x = columns.get(k)[0], z = columns.get(k)[1], n = columns.get(k)[2];
            if (!chunks.add(((long) (x >> 4) << 32) ^ (z >> 4)))
                continue;
            var chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
            // The ground's top: the surface heightmap's block, below a snow layer or a plant if features ran.
            int y = chunk.getHeight(Heightmap.Types.WORLD_SURFACE_WG, x, z);
            var pos = new BlockPos.MutableBlockPos();
            var snow = BuiltInRegistries.BLOCK.get(ResourceLocation.parse(ice.spec().blockId(IceCover.Spec.SNOW)));
            for (int up = 0; up < 6 && y > chunk.getMinBuildHeight() && !chunk.getBlockState(pos.set(x, y, z)).is(snow); up++)
                y--;
            if (!chunk.getBlockState(pos.set(x, y, z)).getFluidState().isEmpty()) {
                wet++;
                continue;
            }
            checked++;
            thick = Math.max(thick, n);
            for (int i = 0; i < n; i++) {
                var want = BuiltInRegistries.BLOCK.get(ResourceLocation.parse(ice.spec().blockId(ice.spec().kind(i, n))));
                var have = chunk.getBlockState(pos.set(x, y - i, z));
                if (!have.is(want) && bad.length() < 400)
                    bad.append(String.format(java.util.Locale.ROOT, " (%d,%d) depth %d of %d: %s, want %s;", x, z, i, n, have.getBlock(), want));
            }
        }
        DeepTime.LOGGER.info("[gametest] iceCoverLaidOnGlaciatedGround ice columns={} checked={} underWater={} thickest={}", columns.size(), checked, wet, thick);
        helper.assertTrue(columns.isEmpty() || checked + wet > 0, "no column of ice cover could be checked (of " + columns.size() + ")");
        helper.assertTrue(bad.isEmpty(), "columns without their ice cover:" + bad);
        helper.succeed();
    }

    /**
     * The ice pass on a chunk built for it (any {@link IceCover} will do): columns 4 to 7 have a cover
     * of 1 block, 8 to 11 of 5, 12 to 15 of 12; columns 0 to 3 have none. The top of each is ground
     * under a dirt block; the cover replaces the top {@code n} blocks (so the ground's height is
     * unchanged) with the table's snow on top, packed ice between and blue ice at the base of a
     * thick one; a column under water and one that runs out of ground after two blocks are the edge
     * cases.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void icePassLaysTheCover(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        var pos = new BlockPos.MutableBlockPos();
        var chunk = new ProtoChunk(new net.minecraft.world.level.ChunkPos(4000, 4000), net.minecraft.world.level.chunk.UpgradeData.EMPTY, level,
                level.registryAccess().registryOrThrow(net.minecraft.core.registries.Registries.BIOME), null);
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        int minY = chunk.getMinBuildHeight();
        for (int lx = 0; lx < 16; lx++)
            for (int lz = 0; lz < 16; lz++) {
                int x = x0 + lx, z = z0 + lz;
                if (lz == 15) {   // two blocks of ground at the bottom of the world, then air
                    chunk.setBlockState(pos.set(x, minY, z), Blocks.STONE.defaultBlockState(), false);
                    chunk.setBlockState(pos.set(x, minY + 1, z), Blocks.STONE.defaultBlockState(), false);
                    continue;
                }
                for (int y = minY; y <= 80; y++)
                    chunk.setBlockState(pos.set(x, y, z), Blocks.STONE.defaultBlockState(), false);
                if (lz == 14) {   // under a lake
                    for (int y = 81; y <= 85; y++)
                        chunk.setBlockState(pos.set(x, y, z), Blocks.WATER.defaultBlockState(), false);
                } else {
                    chunk.setBlockState(pos.set(x, 81, z), Blocks.DIRT.defaultBlockState(), false);
                }
            }
        Heightmap.primeHeightmaps(chunk, java.util.EnumSet.of(Heightmap.Types.WORLD_SURFACE_WG, Heightmap.Types.OCEAN_FLOOR_WG));
        IceCover fake = new IceCover() {
            @Override
            public int blocks(int x, int z) {
                int lx = x - x0;
                return lx < 4 ? 0 : lx < 8 ? 1 : lx < 12 ? 5 : 12;
            }

            @Override
            public Spec spec() {
                return new Spec(160.0, 12, 3, 9, 2, "minecraft:snow_block", "minecraft:packed_ice", "minecraft:blue_ice");
            }
        };
        int placed = IcePass.apply(chunk, fake);
        var snow = Blocks.SNOW_BLOCK;
        var packed = Blocks.PACKED_ICE;
        var blue = Blocks.BLUE_ICE;
        String bad = "";
        for (int lx = 0; lx < 16; lx++)
            for (int lz = 0; lz < 16; lz++) {
                int x = x0 + lx, z = z0 + lz, n = fake.blocks(x, z);
                if (lz == 15) {
                    // Two blocks of ground: the cover takes both (snow, the top of any cover) and stops at the air.
                    for (int i = 0; i < Math.min(n, 2); i++)
                        if (!chunk.getBlockState(pos.set(x, minY + 1 - i, z)).is(snow))
                            bad += " thin(" + lx + "," + i + ")";
                    if (n == 0 && !chunk.getBlockState(pos.set(x, minY + 1, z)).is(Blocks.STONE))
                        bad += " thinbare(" + lx + ")";
                    continue;
                }
                if (lz == 14) {
                    if (!chunk.getBlockState(pos.set(x, 80, z)).is(Blocks.STONE) || !chunk.getBlockState(pos.set(x, 85, z)).is(Blocks.WATER))
                        bad += " lake(" + lx + ")";
                    continue;
                }
                if (n == 0) {
                    if (!chunk.getBlockState(pos.set(x, 81, z)).is(Blocks.DIRT))
                        bad += " bare(" + lx + "," + lz + ")";
                    continue;
                }
                for (int i = 0; i < n; i++) {
                    var want = i < 3 ? snow : (n >= 9 && i >= n - 2) ? blue : packed;
                    if (!chunk.getBlockState(pos.set(x, 81 - i, z)).is(want))
                        bad += " (" + lx + "," + lz + ")@" + i + "of" + n;
                }
                // Below the cover the ground is what it was, and nothing was added above it.
                if (!chunk.getBlockState(pos.set(x, 81 - n, z)).is(Blocks.STONE))
                    bad += " below(" + lx + "," + lz + ")";
                if (!chunk.getBlockState(pos.set(x, 82, z)).isAir())
                    bad += " above(" + lx + "," + lz + ")";
            }
        // 14 rows of ground, 4 columns each of 1, 5 and 12 blocks; the thin row: 4 of 1 block and 8 of 2.
        DeepTime.LOGGER.info("[gametest] icePassLaysTheCover placed={} bad={}", placed, bad);
        helper.assertTrue(bad.isEmpty(), "the ice pass left:" + bad);
        helper.assertTrue(placed == 14 * 4 * (1 + 5 + 12) + 4 * 1 + 8 * 2, "blocks placed: " + placed);
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
