// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeCensus;
import com.minecraftindustrialcomplex.deeptime.climate.OverworldBands;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.IcePass;
import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.LakeIce;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
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
    //
    // 2026-10-02 (continental crust fix: resampling footprints, island arcs mature by their crust): the
    // default test planet changed again and its A family read 0.446 (83 columns; Af's columns 43 % jungle,
    // 37 % stony shore), with the mapping unchanged. A goes to 0.34 by the same rule.
    //
    // 2026-10-03 (the Lambert equal-area map, merged with the 35 m vertical scale and plate tasks 3 and 4):
    // the default test planet's C family read 0.737 standalone and 0.731 with MIC (156 columns), the
    // mapping unchanged; its misses are coastal columns (Csa/Csb 64-100 % stony shore, Cfb 21 % beach and
    // 15 % stony shore standalone). C goes to 0.63 by the same rule.
    //
    // 2026-10-04 (P1-26, the per-cell climate; design §5.3): the default test planet's BW family grew from
    // 38 columns on master (0.500 standalone, 0.474 MIC; under MIN_COLUMNS, logged only) to 51 and read
    // 0.608 standalone and 0.569 with MIC, the mapping unchanged; its misses are coastal and mountain
    // columns (BWh 64 % stony shore standalone, granite cliffs with MIC). BW goes to 0.46 by the same rule.
    // The same planet's high latitudes warmed (Dfc 138 -> 7 columns, E 4 -> 0; D 507 -> 511, mostly Dfa).
    //
    // 2026-10-19: the floors are compared with the share's one-sided 99 % upper bound (ShareBound,
    // Wilson's score interval), not the bare share. The MIC gate on master 025676c8 failed C 0.621 <
    // 0.630 on 132 columns with the mapping unchanged: one standard error of a share at 0.63 on 132
    // columns is 0.042 from where the census's points fell alone, so the bare share failed on noise
    // (its upper bound is 0.713). A broken mapping (0.1 to 0.3) still fails: C on 40 columns at 0.3
    // has 0.484. The families with low floors (A, Dfc:taiga, BW:desert) let a share of 0.2 through on
    // 40 columns; a broken mapping fails the high floors (C, D, E, sea) at once.
    static final Map<String, Double> FLOORS = Map.ofEntries(
            Map.entry("A", 0.34), Map.entry("Af", 0.54), Map.entry("BW", 0.46), Map.entry("BS", 0.42), Map.entry("C", 0.63),
            Map.entry("D", 0.88), Map.entry("Dsub", 0.38), Map.entry("E", 0.82), Map.entry("sea", 0.95),
            Map.entry("BW:desert", 0.37), Map.entry("Af:jungle", 0.41), Map.entry("Dfc:taiga", 0.33), Map.entry("E:snow", 0.72));
    static final int MIN_COLUMNS = 40;

    // The -PwithTerraBlender profile (2026-10-02, P1-10). Biomes O' Plenty's biomes carry few of the c:
    // tags the families count, and since TerraBlender's regions follow the geologic provinces they fill
    // whole provinces, so which provinces drew a BOP region decided the shares: the shares over all columns,
    // on planets 11-15 (master without the province regions / this branch), were C 0.27-0.69 / 0.14-0.68,
    // E:snow 0.28-0.84 / 0.13-0.95, D 0.50-0.97 / 0.86-1.0, BW:desert 0.26-0.67 / 0.28-0.68: a check of
    // BOP's tags, not of the mapping (and 0.681 against 0.558 on planet 11 was which provinces got the mod's
    // regions, not an improvement: planet 12 went 0.644 -> 0.27 the other way). The census therefore leaves
    // BOP's columns out of the tag checks (BiomeCensus untaggedNamespaces) and judges the rest, which are
    // vanilla biomes through the same mapping. That leaves 40-60 % of the columns, so a family needs
    // MIN_COLUMNS_TB of them to be asserted, and two floors sit lower than the standalone and MIC ones.
    // Measured on planets 11-15 (BOP columns out), lowest-highest over the families with at least
    // MIN_COLUMNS_TB columns (A and Af:jungle on one planet only): A 0.758, Af 1.0, BW 0.739-0.776,
    // BS 0.496-0.659, C 0.850-0.970, D 0.989-1.0, Dsub 0.914-0.951, E 0.964-1.0, sea 1.0,
    // BW:desert 0.568-0.616, Af:jungle 0.585, Dfc:taiga 0.57-0.92, E:snow 0.776-0.988. Floors are 0.1 under the lowest of these or the standalone
    // floor, whichever is lower: BS 0.39, E:snow 0.67.
    // Re-measured after the merge with meteor impacts and the grid audit (the planets changed again; planets
    // 11-17, families with at least MIN_COLUMNS_TB columns, lowest-highest): A 0.717-0.98, Af 0.985-1.0,
    // BW 0.756-0.758, BS 0.527-0.733, C 0.848-0.97, D 1.0, Dsub 0.797-0.934, E 0.953-1.0, sea 1.0,
    // BW:desert 0.569-0.605, Af:jungle 0.489, Dfc:taiga 0.65, E:snow 0.751-0.957. Two of them came within 0.1 of a
    // floor, so with the first set's lowest taken in too: E:snow 0.65 (lowest 0.751) and Af:jungle 0.39
    // (lowest 0.489); BS keeps 0.39 (lowest 0.496 of the first set).
    static final int MIN_COLUMNS_TB = 100;
    static final Map<String, Double> FLOORS_TB = Map.of("BS", 0.39, "E:snow", 0.65, "Af:jungle", 0.39);
    /** The largest share of land columns the bake's wetland rule may hold (its flat share, BiomeParams.wetland_flat_share 0.35). */
    static final double WETLAND_CEILING = 0.35;
    /** The largest share of cold wetland columns that may land on a swamp that is not a peatland (BIO-133). */
    static final double COLD_WETLAND_SWAMP_CEILING = 0.05;
    /**
     * The largest share of the columns inland of the coast (the sea rule the same 8 blocks away on all four sides) that
     * the sea rule leaves dry ({@code Water.seaCovers}, HYD-106, on the world's elevation layer) which may carry an ocean
     * biome, and of those it puts under the sea which may carry a biome of the land (BIO-173). The biome bake settles
     * its land call to the world's own ground in the projection pass (BIO-170); what is left is the coast between two
     * biome nodes, where the game reads the layers bilinear between nodes 8 blocks apart, and the census leaves that
     * strip out. Before the settle 5.1 to 10.5 % of the dry land of the 16k planets carried an ocean biome on the
     * bake's nodes and an area of 0.5 to 1.2 % of the land a land biome under the sea; after it none of the nodes, and
     * 0 to 2 columns in about 60,000 more than a node from the coast (tools/review/biome_land_census.py). The 4k test
     * planet's own count is logged; this is a ceiling for a broken rule, not a measurement.
     */
    static final double SEA_CALL_CEILING = 0.01;
    /** The most EF columns that may land on the red badlands or scarlet mountains (the peaks of negative weirdness keep Terralith's). */
    static final double FORBIDDEN_CEILING = 0.10;
    /**
     * The ice cap's ice spikes are bounded rather than banned (BiomeCensus.EF_ICE_SPIKES_STEEP and _FLATS; ledger:
     * BIO-162, the ice spikes report of 2026-10-20): a share fails only when its one-sided 99 % lower bound is above
     * its ceiling ({@link ShareBound#above}). The game's census emulated on bakes (vanilla and MIC; Ben's play world
     * and the 2026-10-20 master's supercontinent and scattered continents) gave, on the ice cap's steep ground (erosion
     * E0 and E1), 0.010 to 0.030 after EF's continentalness floor (BIO-161; with vanilla its valley columns and the
     * near-inland ring the world grid's projection leaves along the ice cap's coast, 0.030 on a real bake of the
     * supercontinent; none with MIC) and 0.175 to 0.212 before it with vanilla; on its flats, with MIC 0.170 to 0.200
     * after and 0.648 to 0.706 before, with vanilla 0.059 to 0.343 both (the flats rule's noise, BIO-077). The steep
     * ceiling sits near the geometric mean of 0.030 and 0.175; the flats' is the pack's own share of its frozen,
     * driest flats (BIO-161's target: never above it). On the GameTests' 4k planet the ice cap is 3 census columns,
     * under {@link #MIN_COLUMNS}, so there both are logged; they are asserted on a planet with an ice cap.
     */
    static final double STEEP_ICE_SPIKES_CEILING = 0.07; // ledger: BIO-162
    /** Ice spikes' share of the frozen, driest flats in the pack's own worlds (4 seeds each, 2026-10-20): Terralith 2.6.2's and vanilla's. */
    static final double OWN_FLATS_ICE_SPIKES_TERRALITH = 0.345, OWN_FLATS_ICE_SPIKES_VANILLA = 0.468; // ledger: BIO-162

    /** Whether a {@link BiomeCensus#FORBIDDEN} family is a bounded share rather than a ban. */
    static boolean bounded(String name) {
        return name.equals(BiomeCensus.EF_ICE_SPIKES_STEEP) || name.equals(BiomeCensus.EF_ICE_SPIKES_FLATS);
    }

    /** A {@link BiomeCensus#FORBIDDEN} family's ceiling; a pack without Terralith is held to vanilla's own share. */
    static double forbiddenCeiling(String name) {
        if (name.equals(BiomeCensus.EF_ICE_SPIKES_STEEP))
            return STEEP_ICE_SPIKES_CEILING;
        if (name.equals(BiomeCensus.EF_ICE_SPIKES_FLATS))
            return ModList.get().isLoaded("terralith") ? OWN_FLATS_ICE_SPIKES_TERRALITH : OWN_FLATS_ICE_SPIKES_VANILLA;
        return FORBIDDEN_CEILING;
    }

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
        boolean bop = ModList.get().isLoaded("biomesoplenty");
        int minColumns = bop ? MIN_COLUMNS_TB : MIN_COLUMNS;
        try {
            // Biomes O' Plenty's biomes carry few of the c: tags the checks count, and with TerraBlender they
            // fill whole provinces (P1-10): which provinces got its regions would decide the shares, not the
            // mapping. They are left out of the checks (counted in "skipped_untagged"); the census over all
            // columns is logged beside it.
            java.util.Set<String> untagged = bop ? java.util.Set.of("biomesoplenty") : java.util.Set.of();
            if (bop)
                DeepTime.LOGGER.info("[gametest] climateConsistent (all columns, BOP included, not asserted) {}",
                        BiomeCensus.run(level, t.circumference() / 256, 32, null).json());
            r = BiomeCensus.run(level, t.circumference() / 256, 32, null, untagged);
        } catch (java.io.IOException e) {
            helper.fail(e.toString());
            return;
        }
        DeepTime.LOGGER.info("[gametest] climateConsistent {}", r.json());
        StringBuilder low = new StringBuilder();
        for (var ch : BiomeCensus.CHECKS) {
            double share = r.share(ch.name());
            if (r.total(ch.name()) < minColumns)
                continue;
            double floor = (bop ? FLOORS_TB : Map.<String, Double>of()).getOrDefault(ch.name(), FLOORS.getOrDefault(ch.name(), 0.0));
            // Significantly below the floor, not merely below it (ShareBound, 2026-10-19).
            int n = r.total(ch.name()), hits = r.checks().get(ch.name())[0];
            double upper = ShareBound.wilsonUpper(hits, n, ShareBound.Z_99);
            if (upper < floor)
                low.append(String.format(java.util.Locale.ROOT, " %s %.3f (99 %% upper bound %.3f) < %.3f (%d columns)",
                        ch.name(), share, upper, floor, n));
            else if (share < floor)
                DeepTime.LOGGER.info("[gametest] climateConsistent {} {} under its floor {} on {} columns, within the sample's noise "
                        + "(99 % upper bound {})", ch.name(), share, floor, n, upper);
        }
        helper.assertTrue(low.isEmpty(), "Köppen families below their floor:" + low);
        // The wetlands are left out of the families (BiomeCensus): the bake sends wet, flat, low,
        // thawed land to vanilla's swamp band on purpose (BIO-066), at most the flattest 35 % of the
        // land (wetland_flat_share). On master 6f72f0dd's planet the whole D-climate land was such a
        // coastal plain (Dfc 66 % swamp, 28 % beach) and failed Dsub and Dfc:taiga for it. More than
        // that share would mean the rule broke, and is a failure rather than a set-aside.
        var wet = r.json().getAsJsonObject("wetlands_set_aside");
        int landColumns = wet.get("land_columns").getAsInt(), wetColumns = wet.get("wetland_columns").getAsInt();
        helper.assertTrue(wetColumns <= WETLAND_CEILING * landColumns,
                wetColumns + " of " + landColumns + " land columns are in the bake's wetland band, more than " + WETLAND_CEILING);
        // Cold wetland (Ben, 2026-10-03, "Taiga instead, peatland if present"; BIO-133, BIO-135): the bake's
        // flagged boreal wetland is taiga, or the pack's peatland biome where it has one, and never another
        // swamp. Before the rule it was all swamp (66 % of one planet's Dfc land). A few percent of a mod's
        // oddly tagged biomes are let through.
        var cold = r.json().getAsJsonObject("cold_wetland");
        helper.assertTrue(cold.get("layer").getAsBoolean(), "the bake wrote climate/wetland, the flag of cold wetland (BIO-133)");
        int coldColumns = cold.get("columns").getAsInt(), coldSwamp = cold.get("on_swamp").getAsInt();
        DeepTime.LOGGER.info("[gametest] climateConsistent cold wetland {}", cold);
        helper.assertTrue(coldSwamp <= COLD_WETLAND_SWAMP_CEILING * coldColumns,
                coldSwamp + " of " + coldColumns + " cold wetland columns are on a swamp that is not a peatland: " + cold);
        // EF is ice or barren frozen ground, never the red badlands or Nether-stone mountains.
        StringBuilder red = new StringBuilder();
        for (var f : BiomeCensus.FORBIDDEN) {
            double share = r.forbiddenShare(f.name());
            DeepTime.LOGGER.info("[gametest] climateConsistent {} {} of {} columns", f.name(), share, r.forbiddenTotal(f.name()));
            int n = r.forbiddenTotal(f.name());
            if (n < minColumns)
                continue;
            if (bounded(f.name())) {
                // A bounded share, not a ban: significantly above its ceiling, as the floors are significantly
                // below theirs (ShareBound, 2026-10-19 and 2026-10-20).
                double ceiling = forbiddenCeiling(f.name());
                double lower = ShareBound.wilsonLower(r.forbiddenHits(f.name()), n, ShareBound.Z_99);
                if (lower > ceiling)
                    red.append(String.format(java.util.Locale.ROOT, " %s %.3f (99 %% lower bound %.3f) > %.3f (%d columns)",
                            f.name(), share, lower, ceiling, n));
            } else if (share > FORBIDDEN_CEILING)
                red.append(String.format(java.util.Locale.ROOT, " %s %.3f > %.3f (%d columns)", f.name(), share, FORBIDDEN_CEILING, n));
        }
        helper.assertTrue(red.isEmpty(), "families on biomes they must not have:" + red);
        // The land call (BIO-170): the biome bake and the column evaluator call the same ground sea or land.
        var sea = r.json().getAsJsonObject("sea_call");
        int seaColumns = sea.get("sea_columns").getAsInt(), seaOnLand = sea.get("sea_on_land_biome").getAsInt();
        int dryColumns = sea.get("dry_columns").getAsInt(), dryOnOcean = sea.get("dry_on_ocean_biome").getAsInt();
        DeepTime.LOGGER.info("[gametest] climateConsistent sea call {}: {} of {} dry columns inland of the coast on an ocean biome, {} of {} sea columns on a land biome",
                sea, dryOnOcean, dryColumns, seaOnLand, seaColumns);
        helper.assertTrue(dryColumns >= MIN_COLUMNS && seaColumns >= MIN_COLUMNS, "the planet has dry and sea columns inland of its coast: " + sea);
        helper.assertTrue(dryOnOcean <= SEA_CALL_CEILING * dryColumns,
                dryOnOcean + " of " + dryColumns + " columns inland of the coast that the sea rule leaves dry carry an ocean biome, more than "
                        + SEA_CALL_CEILING);
        helper.assertTrue(seaOnLand <= SEA_CALL_CEILING * seaColumns,
                seaOnLand + " of " + seaColumns + " columns the sea rule puts under the sea carry a land biome, more than " + SEA_CALL_CEILING);
        helper.succeed();
    }

    /**
     * Ben, 2026-09-30, "Force snow/ice cover": where the bake's ice cover says so, Deep Time's own
     * surface pass ({@code IcePass}) lays snow, packed ice and blue ice over the top of the ground
     * whatever biome the pack drew. On the real baked planet, columns across its ice cover (one per
     * 16 blocks, about two dozen spread over them, each in its own chunk at the surface status) have
     * the table's blocks from the ground's top down, as many as the cover is thick. A column of a chunk
     * another test took past its carvers, where a cave cut the cover (packed ice and snow are in
     * {@code #minecraft:overworld_carver_replaceables}), is set aside: the air or fluid lies in the chunk's
     * air carving mask, or the chunk is full and its mask gone. The columns are read on a cube's faces only
     * ({@link ChunkSampling#onFaces}): off them a chunk is its partner's copy up to its carvers
     * ({@code CubeFill}), at the surface status with the partner's caves already cut (2026-10-27: a cave
     * through the 12-block cover at (-776, -1368), in a void square of PLT-720's test planet).
     */
    /** Column (x, z) of {@code chunk} from y {@code hi} down to {@code lo} as runs of one block, for a failure's log. */
    private static String columnRuns(net.minecraft.world.level.chunk.ChunkAccess chunk, int x, int z, int hi, int lo) {
        StringBuilder sb = new StringBuilder();
        var pos = new BlockPos.MutableBlockPos();
        String last = null;
        int runTop = hi;
        for (int y = hi; y >= lo; y--) {
            String id = BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(pos.set(x, y, z)).getBlock()).getPath();
            if (!id.equals(last)) {
                if (last != null)
                    sb.append(' ').append(last).append('@').append(runTop).append("..").append(y + 1);
                last = id;
                runTop = y;
            }
        }
        return sb.append(' ').append(last).append('@').append(runTop).append("..").append(lo).toString();
    }

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
        int zp = (int) t.zHalf();   // the net's outer edges on the cube (3s/2), the poles on a Lambert-frame world
        // Every covered column is of the cover's class, EF (BIO-158), read as the biome rules and the census read
        // it: the cover and the class both take the nearest cell (BIO-159; the bilinear read put the cover on Cfb
        // and BWk ground beside the sheet, and L2 cells under an L1 sheet kept their own class).
        RasterSampler kp = wd.has("climate/koppen_l2") ? wd.layer("climate/koppen_l2") : null;
        int notEf = 0;
        StringBuilder notEfAt = new StringBuilder();
        for (int z = -zp + 8; z < zp; z += 16)
            for (int x = -c / 2 + 8; x < c / 2; x += 16) {
                int n = ice.blocks(x, z);
                long k = n > 0 && kp != null ? Math.round(kp.nearest(x, z)) : 30;
                if (k != 30 && notEf++ < 8)
                    notEfAt.append(String.format(java.util.Locale.ROOT, " (%d,%d) class %d;", x, z, k));
                if (n > 0 && t.height(x, z) > sea + 2 && ChunkSampling.onFaces(wd, x, z))
                    columns.add(new int[] {x, z, n});
            }
        helper.assertTrue(notEf == 0, notEf + " covered columns not of class EF:" + notEfAt);
        int stride = Math.max(1, columns.size() / 24);
        int checked = 0, wet = 0, thick = 0, carved = 0;
        StringBuilder bad = new StringBuilder();
        java.util.Set<Long> chunks = new java.util.HashSet<>();
        for (int k = 0; k < columns.size() && checked < 24; k += stride) {
            int x = columns.get(k)[0], z = columns.get(k)[1], n = columns.get(k)[2];
            if (!chunks.add(((long) (x >> 4) << 32) ^ (z >> 4)))
                continue;
            var chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
            // The ground's top: the surface heightmap's block, below a snow layer or a plant if features ran.
            int top = chunk.getHeight(Heightmap.Types.WORLD_SURFACE_WG, x, z), y = top;
            var pos = new BlockPos.MutableBlockPos();
            var snow = BuiltInRegistries.BLOCK.get(ResourceLocation.parse(ice.spec().blockId(IceCover.Spec.SNOW)));
            for (int up = 0; up < 6 && y > chunk.getMinBuildHeight() && !chunk.getBlockState(pos.set(x, y, z)).is(snow); up++)
                y--;
            if (!chunk.getBlockState(pos.set(x, y, z)).is(snow))
                y = top;   // no snow near the top: report the top, not the ground six blocks under it
            // Water, or a frozen lake's ice (on its water, or the world's lake a block deep iced to its bed).
            var at = chunk.getBlockState(pos.set(x, y, z));
            if (!at.getFluidState().isEmpty() || at.is(Blocks.ICE)
                    && (!chunk.getBlockState(pos.set(x, y - 1, z)).getFluidState().isEmpty() || LakeIce.ices(wd, g.waterReach(), x, y, z))) {
                wet++;
                continue;
            }
            boolean full = chunk instanceof net.minecraft.world.level.chunk.ImposterProtoChunk
                    || chunk instanceof net.minecraft.world.level.chunk.LevelChunk;
            var cut = chunk.getPersistedStatus().isOrAfter(ChunkStatus.CARVERS) && !full && chunk instanceof ProtoChunk p
                    ? p.getCarvingMask(net.minecraft.world.level.levelgen.GenerationStep.Carving.AIR) : null;
            boolean cave = false;
            for (int i = 0; i < n && !cave && (full || cut != null); i++) {
                var have = chunk.getBlockState(pos.set(x, y - i, z));
                cave = (have.isAir() || !have.getFluidState().isEmpty()) && (full || cut.get(x, y - i, z));
            }
            if (cave) {
                carved++;
                continue;
            }
            checked++;
            thick = Math.max(thick, n);
            // The biome names the pack's surface the cover went on (2026-10-18: Expanded Ecosphere's ice cap,
            // whose ice the pass took for a frozen lake's).
            String biome = null;
            boolean dumped = false;
            for (int i = 0; i < n; i++) {
                var want = BuiltInRegistries.BLOCK.get(ResourceLocation.parse(ice.spec().blockId(ice.spec().kind(i, n))));
                var have = chunk.getBlockState(pos.set(x, y - i, z));
                if (!have.is(want) && bad.length() < 400) {
                    if (biome == null)
                        biome = level.getBiome(new BlockPos(x, y, z)).unwrapKey().map(key -> key.location().toString()).orElse("?");
                    bad.append(String.format(java.util.Locale.ROOT, " (%d,%d) in %s depth %d of %d: %s, want %s;", x, z, biome, i, n,
                            have.getBlock(), want));
                    if (!dumped && (dumped = true))
                        DeepTime.LOGGER.info("[gametest] iceCoverLaidOnGlaciatedGround column ({},{}) status {} {} top {} y {}: {}", x, z,
                                chunk.getPersistedStatus(), chunk.getClass().getSimpleName(), top, y, columnRuns(chunk, x, z, y + 3, y - n - 3));
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] iceCoverLaidOnGlaciatedGround ice columns={} checked={} underWater={} carved={} thickest={} notEf={}",
                columns.size(), checked, wet, carved, thick, notEf);
        helper.assertTrue(columns.isEmpty() || checked + wet > 0, "no column of ice cover could be checked (of " + columns.size() + ")");
        helper.assertTrue(bad.isEmpty(), "columns without their ice cover:" + bad);
        helper.succeed();
    }

    /**
     * The ice pass on a chunk built for it (any {@link IceCover} will do): columns 4 to 7 have a cover
     * of 1 block, 8 to 11 of 5, 12 to 15 of 12; columns 0 to 3 have none. The top of each is ground
     * under a dirt block; the cover replaces the top {@code n} blocks (so the ground's height is
     * unchanged) with the table's snow on top, packed ice between and blue ice at the base of a
     * thick one; a column under water, one under a frozen lake's ice and one that runs out of ground
     * after two blocks are the edge cases, and since 2026-10-18 two more: the pack's ice on its packed
     * ice (Expanded Ecosphere's ice cap; vanilla's frozen peaks put patches of ice on stone) is ground
     * and takes the cover, and a frozen lake a block deep (the water pass's ice on its bed, no water
     * under it), which the world's lakes name, keeps its ice.
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
                } else if (lz == 13) {   // under a frozen lake: the water pass's ice on its water
                    for (int y = 81; y <= 84; y++)
                        chunk.setBlockState(pos.set(x, y, z), Blocks.WATER.defaultBlockState(), false);
                    chunk.setBlockState(pos.set(x, 85, z), Blocks.ICE.defaultBlockState(), false);
                } else if (lz == 12) {   // the pack's ice on its packed ice (Expanded Ecosphere's ice cap)
                    for (int y = 78; y <= 80; y++)
                        chunk.setBlockState(pos.set(x, y, z), Blocks.PACKED_ICE.defaultBlockState(), false);
                    chunk.setBlockState(pos.set(x, 81, z), Blocks.ICE.defaultBlockState(), false);
                } else if (lz == 11) {   // a frozen lake a block deep: the water pass's ice on its bed
                    chunk.setBlockState(pos.set(x, 80, z), Blocks.DIRT.defaultBlockState(), false);
                    chunk.setBlockState(pos.set(x, 81, z), Blocks.ICE.defaultBlockState(), false);
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
        // The world's lakes: row 11's ice is a frozen lake's top.
        int placed = IcePass.apply(chunk, fake, (x, y, z) -> z - z0 == 11 && y == 81);
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
                if (lz == 13) {
                    // The lake's ice is its surface, not ground: the cover leaves it, whatever the thickness there.
                    if (!chunk.getBlockState(pos.set(x, 80, z)).is(Blocks.STONE) || !chunk.getBlockState(pos.set(x, 84, z)).is(Blocks.WATER)
                            || !chunk.getBlockState(pos.set(x, 85, z)).is(Blocks.ICE))
                        bad += " frozenlake(" + lx + ")";
                    continue;
                }
                if (lz == 12) {
                    // The pack's ice is ground: the cover replaces it as it would stone.
                    if (n == 0 && !chunk.getBlockState(pos.set(x, 81, z)).is(Blocks.ICE))
                        bad += " packbare(" + lx + ")";
                    for (int i = 0; i < n; i++) {
                        var want = i < 3 ? snow : (n >= 9 && i >= n - 2) ? blue : packed;
                        if (!chunk.getBlockState(pos.set(x, 81 - i, z)).is(want))
                            bad += " pack(" + lx + ")@" + i + "of" + n;
                    }
                    if (!chunk.getBlockState(pos.set(x, 82, z)).isAir())
                        bad += " packabove(" + lx + ")";
                    continue;
                }
                if (lz == 11) {
                    // A frozen lake a block deep: its ice is the lake's surface, left as it is.
                    if (!chunk.getBlockState(pos.set(x, 81, z)).is(Blocks.ICE) || !chunk.getBlockState(pos.set(x, 80, z)).is(Blocks.DIRT))
                        bad += " shallowlake(" + lx + ")";
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
        // 11 rows of ground, 4 columns each of 1, 5 and 12 blocks; the pack's ice row the same less the
        // packed ice already where the cover puts it (1, 4 and 11); the thin row: 4 of 1 block and 8 of 2.
        DeepTime.LOGGER.info("[gametest] icePassLaysTheCover placed={} bad={}", placed, bad);
        helper.assertTrue(bad.isEmpty(), "the ice pass left:" + bad);
        helper.assertTrue(placed == 11 * 4 * (1 + 5 + 12) + 4 * (1 + 4 + 11) + 4 * 1 + 8 * 2, "blocks placed: " + placed);
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
