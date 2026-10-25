// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeCensus;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeRuleSet;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeRules;
import com.minecraftindustrialcomplex.deeptime.climate.RuleCensus;
import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.OverworldOverride;
import com.minecraftindustrialcomplex.deeptime.worldgen.PlanetGenerators;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones;
import net.minecraft.core.Holder;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;
import net.minecraft.world.level.levelgen.presets.WorldPresets;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

/**
 * Expanded Ecosphere as a first-class target beside Terralith (Ben, decisions log 2026-10-04: "Build out
 * support for both so if I install EE it works or if I go w/ terralith it also works"), and Deep Time's biome
 * rules ({@code climate.BiomeRules}) on every profile. The Expanded Ecosphere tests run their checks on the
 * {@code -PwithEe} profile ({@code -Ddeeptime.ee.expected}, set by neoforge/build.gradle from the server set,
 * makes them fail when the mod should be there and is not); on the others they pass with a line saying so.
 * Each logs its numbers on a {@code [gametest]} line.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class BiomeRuleGameTests {

    private BiomeRuleGameTests() {}

    static final String EE = "expanded_ecosphere";
    static final ResourceLocation EE_RULES = ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "expanded_ecosphere");

    /** Fewest cold-wetland columns on the test planet before the peatland share is asserted. */
    static final int MIN_COLD_WETLAND = 20; // ledger: BIO-130 (game-choice)
    /** Least share of cold wetland on a peatland biome where the pack has one. */
    static final double MIN_PEAT_SHARE = 0.9; // ledger: BIO-130 (game-choice)

    /** Whether this run should have Expanded Ecosphere; fails the test when it should and has not. */
    private static boolean ee(GameTestHelper helper, String test) {
        boolean expected = Boolean.getBoolean("deeptime.ee.expected");
        boolean loaded = ModList.get().isLoaded(EE);
        helper.assertTrue(loaded || !expected, "this run's mods include Expanded Ecosphere (-Ddeeptime.ee.expected=true) but it is not loaded");
        if (!loaded)
            DeepTime.LOGGER.info("[gametest] {}: Expanded Ecosphere is not installed in this run; nothing to check", test);
        return loaded;
    }

    /**
     * A datapack's overworld never replaces Deep Time's generator in a Deep Time world, and a vanilla-preset world
     * keeps the datapack's overworld ({@link OverworldOverride}). The datapacks' own overworld is the override where
     * one exists (Expanded Ecosphere's on {@code -PwithEe}, recorded by the GameTest server's dimension loading);
     * elsewhere vanilla's Normal overworld stands in for one. With Expanded Ecosphere the Deep Time overworld must
     * take its biome list. On every profile.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void overworldOverrideKeepsDeepTimesGenerator(GameTestHelper helper) {
        RegistryAccess access = helper.getLevel().getServer().registryAccess();
        WorldDimensions normal = access.registryOrThrow(Registries.WORLD_PRESET).getOrThrow(WorldPresets.NORMAL).createWorldDimensions();
        LevelStem recorded = GameTestSupport.datapackOverworld();
        LevelStem datapackOverworld = recorded != null ? recorded : normal.dimensions().get(LevelStem.OVERWORLD);
        helper.assertTrue(datapackOverworld != null && !(datapackOverworld.generator() instanceof GeologicChunkGenerator),
                "the datapack overworld is not a vanilla-style generator: " + datapackOverworld);
        net.minecraft.core.MappedRegistry<LevelStem> dp = new net.minecraft.core.MappedRegistry<>(Registries.LEVEL_STEM,
                com.mojang.serialization.Lifecycle.stable());
        dp.register(LevelStem.OVERWORLD, datapackOverworld, net.minecraft.core.RegistrationInfo.BUILT_IN);
        Registry<LevelStem> stems = dp.freeze();

        WorldDimensions planet = access.registryOrThrow(Registries.WORLD_PRESET).getOrThrow(PlanetGenerators.PLANET_CUBE).createWorldDimensions();
        OverworldOverride.Result r = OverworldOverride.apply(planet, stems);
        LevelStem ow = r.dims().dimensions().get(LevelStem.OVERWORLD);
        helper.assertTrue(ow != null && ow.generator() instanceof GeologicChunkGenerator,
                "a Deep Time world's overworld lost Deep Time's generator to the datapack's: " + (ow == null ? null : ow.generator()));
        helper.assertTrue(r.stems().get(LevelStem.OVERWORLD) == null, "the datapack's overworld was not set aside in a Deep Time world");
        Set<Holder<Biome>> want = datapackOverworld.generator().getBiomeSource().possibleBiomes();
        Set<Holder<Biome>> got = ow.generator().getBiomeSource().possibleBiomes();
        helper.assertTrue(got.equals(want), "the Deep Time overworld has " + got.size() + " biomes, the datapack's overworld " + want.size());

        OverworldOverride.Result v = OverworldOverride.apply(normal, stems);
        helper.assertTrue(v.dims().dimensions().get(LevelStem.OVERWORLD) == normal.dimensions().get(LevelStem.OVERWORLD),
                "a vanilla-preset world's own overworld was changed");
        helper.assertTrue(v.stems().get(LevelStem.OVERWORLD) == datapackOverworld, "a vanilla-preset world lost the datapack's overworld");

        boolean ee = ee(helper, "overworldOverrideKeepsDeepTimesGenerator");
        long wythers = got.stream().filter(b -> b.unwrapKey().map(k -> k.location().getNamespace().equals("wythers")).orElse(false)).count();
        DeepTime.LOGGER.info("[gametest] overworldOverrideKeepsDeepTimesGenerator datapackOverworld={} ({}) deepTimeBiomes={} wythers={}",
                datapackOverworld.generator().getClass().getSimpleName(), recorded != null ? "the datapacks'" : "vanilla's Normal standing in",
                got.size(), wythers);
        if (ee) {
            helper.assertTrue(recorded != null, "Expanded Ecosphere is loaded but no datapack defines minecraft:overworld");
            helper.assertTrue(wythers >= 150, "with Expanded Ecosphere the Deep Time overworld has only " + wythers + " of its biomes");
        }
        helper.succeed();
    }

    /**
     * Every Deep Time level that the datapacks define on vanilla's overworld list (the real test planet) takes the
     * overriding overworld's biomes: with Expanded Ecosphere, its 189.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void deepTimeLevelsTakeTheOverridingBiomes(GameTestHelper helper) {
        if (!ee(helper, "deepTimeLevelsTakeTheOverridingBiomes")) {
            helper.succeed();
            return;
        }
        ServerLevel level = RealPlanetGameTests.level(helper);
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        Registry<Biome> registry = level.registryAccess().registryOrThrow(Registries.BIOME);
        var iceCap = registry.getHolder(ResourceKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath("wythers", "ice_cap")));
        DeepTime.LOGGER.info("[gametest] deepTimeLevelsTakeTheOverridingBiomes biomes={} iceCapRegistered={}",
                g.getBiomeSource().possibleBiomes().size(), iceCap.isPresent());
        helper.assertTrue(iceCap.isPresent() && g.getBiomeSource().possibleBiomes().contains(iceCap.get()),
                "test_planet_real cannot place Expanded Ecosphere's ice cap: its biome source was not the overriding overworld's");
        helper.succeed();
    }

    /**
     * Deep Time's Expanded Ecosphere rules are all live: the file parses, every biome id it names is registered
     * and the real test planet's source can place it (a typo, or a biome a newer version dropped, would be a
     * mapped biome that can never appear), every rule is bound, and the volcanic roles resolve to Expanded
     * Ecosphere's biomes.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 400)
    public static void expandedEcosphereRulesAreLive(GameTestHelper helper) throws Exception {
        if (!ee(helper, "expandedEcosphereRulesAreLive")) {
            helper.succeed();
            return;
        }
        ServerLevel level = RealPlanetGameTests.level(helper);
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        Registry<Biome> registry = level.registryAccess().registryOrThrow(Registries.BIOME);
        List<BiomeRuleSet.File> files = BiomeRules.load(level.getServer().getResourceManager());
        BiomeRuleSet.File file = files.stream().filter(f -> f.id().equals(EE_RULES.toString())).findFirst().orElse(null);
        helper.assertTrue(file != null, "no rule file " + EE_RULES + " (loaded: " + files.stream().map(BiomeRuleSet.File::id).toList() + ")");
        Set<Holder<Biome>> possible = g.getBiomeSource().possibleBiomes();
        List<String> missing = new ArrayList<>();
        int named = 0;
        for (BiomeRuleSet.Rule rule : file.rules())
            for (String id : rule.biomes()) {
                if (id.startsWith("#"))
                    continue;
                named++;
                var h = registry.getHolder(ResourceKey.create(Registries.BIOME, ResourceLocation.parse(id)));
                if (h.isEmpty() || !possible.contains(h.get()))
                    missing.add(rule.id() + ":" + id);
            }
        // Bind as the level does (its first biome query has run by now: the census below and the other tests).
        sample(level, 0, 0);
        var marker = (RegionSpawnTarget) level.getChunkSource().randomState().sampler().spawnTarget();
        BiomeRules.Set_ bound = marker.volcanic().rules();
        long eeRules = bound.ordinary().stream().filter(b -> b.file().equals(file.id())).count()
                + bound.byRole().values().stream().flatMap(List::stream).filter(b -> b.file().equals(file.id())).count();
        String cone = marker.volcanic().roleBiome(VolcanicZones.Role.CONE).flatMap(Holder::unwrapKey).map(k -> k.location().toString()).orElse("-");
        String spring = marker.volcanic().roleBiome(VolcanicZones.Role.HOT_SPRING_FIELD).flatMap(Holder::unwrapKey)
                .map(k -> k.location().toString()).orElse("-");
        DeepTime.LOGGER.info("[gametest] expandedEcosphereRulesAreLive named={} missing={} rules={} bound={} files={} cone={} hotSpring={}",
                named, missing, file.rules().size(), eeRules, bound.files(), cone, spring);
        helper.assertTrue(missing.isEmpty(), "rule biomes the source cannot place: " + missing);
        helper.assertTrue(eeRules == file.rules().size(), "only " + eeRules + " of " + file.rules().size() + " Expanded Ecosphere rules are bound");
        helper.assertTrue(cone.startsWith("wythers:") && spring.startsWith("wythers:"),
                "the volcanic roles do not resolve to Expanded Ecosphere's biomes: cone " + cone + ", hot-spring field " + spring);
        helper.succeed();
    }

    /**
     * No candidate of a rule is shadowed by its rule's other candidates: at the centre of each of its own
     * parameter boxes, the search over the rule's candidates answers that biome for at least one box. (A
     * single-candidate rule answers its biome wherever it holds.) Together with {@link #expandedEcosphereRulesAreLive}
     * this is "no mapped biome is unreachable"; the census of a 16k planet (tools/review/ee-biomes.sh) lists the
     * ones a given planet's climate did not reach.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 400)
    public static void mappedBiomesAreReachable(GameTestHelper helper) {
        if (!ee(helper, "mappedBiomesAreReachable")) {
            helper.succeed();
            return;
        }
        ServerLevel level = RealPlanetGameTests.level(helper);
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        sample(level, 0, 0);
        var marker = (RegionSpawnTarget) level.getChunkSource().randomState().sampler().spawnTarget();
        BiomeRules.Set_ bound = marker.volcanic().rules();
        var source = new VolcanicBiomes.ListSource(g.getBiomeSource(), ((MultiNoiseBiomeSource) g.getBiomeSource()).parameters());
        List<BiomeRules.Bound> all = new ArrayList<>(bound.ordinary());
        bound.byRole().values().forEach(all::addAll);
        List<String> shadowed = new ArrayList<>();
        int checked = 0;
        for (BiomeRules.Bound rule : all) {
            if (rule.candidates().size() < 2)
                continue;
            for (Holder<Biome> c : rule.candidates()) {
                checked++;
                boolean wins = false;
                for (var p : source.list().values()) {
                    if (p.getSecond() != c)
                        continue;
                    Climate.ParameterPoint q = p.getFirst();
                    Climate.TargetPoint t = new Climate.TargetPoint(mid(q.temperature()), mid(q.humidity()), mid(q.continentalness()),
                            mid(q.erosion()), mid(q.depth()), mid(q.weirdness()));
                    if (BiomeRules.search(source, t, rule) == c) {
                        wins = true;
                        break;
                    }
                }
                if (!wins)
                    shadowed.add(rule.rule().id() + ":" + c.unwrapKey().map(k -> k.location().toString()).orElse("?"));
            }
        }
        DeepTime.LOGGER.info("[gametest] mappedBiomesAreReachable checked={} shadowed={}", checked, shadowed);
        helper.assertTrue(shadowed.isEmpty(), "rule candidates no climate point picks: " + shadowed);
        helper.succeed();
    }

    private static long mid(Climate.Parameter p) {
        return (p.min() + p.max()) / 2;
    }

    /**
     * On the real test planet with Expanded Ecosphere's biomes: the rules decide a share of the land; a column a
     * rule decided holds one of that rule's candidates in one of its Köppen classes (as the census reads the
     * class); an exclusive candidate (ice cap, salt lakes, tsingy, black beach, glacial cliffs, Tibesti) never
     * comes from the climate search; volcanic roles answer with Expanded Ecosphere's biomes; and where the planet
     * has cold wetland the rules decide, at least {@link #MIN_PEAT_SHARE} of it is on a peatland biome. That is the
     * flag where the biome query reads it and where the ordinary rules run (on land at the quart's centre), outside
     * the columns a volcano's zone decided, which outranks the peatland rule by design (BIO-113; set apart and
     * logged, ledger BIO-189). The census (rules section) is logged in full.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void expandedEcosphereBiomesFollowTheirRules(GameTestHelper helper) throws Exception {
        if (!ee(helper, "expandedEcosphereBiomesFollowTheirRules")) {
            helper.succeed();
            return;
        }
        ServerLevel level = RealPlanetGameTests.level(helper);
        var t = RealPlanetGameTests.terrain(helper, level);
        BiomeCensus.Result r = BiomeCensus.run(level, Math.max(4, t.circumference() / 512), 32, null);
        RuleCensus rc = r.rules();
        helper.assertTrue(rc != null, "no biome rule is active on the real test planet with Expanded Ecosphere");
        DeepTime.LOGGER.info("[gametest] expandedEcosphereBiomesFollowTheirRules {}", rc.json());
        helper.assertTrue(rc.ruleColumns() > 0, "no land column of " + rc.columns() + " was placed by a biome rule");
        helper.assertTrue(rc.notCandidate() == 0, rc.notCandidate() + " columns hold a biome their rule does not name: " + rc.examples());
        helper.assertTrue(rc.wrongClass() == 0, rc.wrongClass() + " columns placed by a rule lie outside its Köppen classes: " + rc.examples());
        helper.assertTrue(rc.exclusiveLeaked() == 0, rc.exclusiveLeaked() + " columns hold an exclusive biome the climate search placed: "
                + rc.examples());
        for (var e : rc.decisions().entrySet()) {
            if (!e.getKey().startsWith("role:") || e.getKey().equals("role:thermal_caves"))
                continue;
            for (String b : rc.biomes(e.getKey()).keySet())
                helper.assertTrue(b.startsWith("wythers:"), "volcanic role " + e.getKey() + " placed " + b + ", not an Expanded Ecosphere biome");
        }
        // Cold wetland a volcano's zone decided is set apart, not counted (BIO-189; 20 of the 29 cold-wetland columns of
        // the 2026-10-24 test planet: an oceanic shield's coastal flat inside its cone zone, beaches in lava fields).
        // Cold wetland on a beach is set apart too (BIO-190; 13 of the 37 on master cc27c9f9's planet, black beaches on basalt).
        DeepTime.LOGGER.info("[gametest] expandedEcosphereBiomesFollowTheirRules: {} cold-wetland columns the rules decide, {} on a peatland"
                + " biome; {} more a volcano's zone decided and {} on a beach, set apart", rc.coldWetland(), rc.coldWetlandPeat(),
                rc.coldWetlandVolcanic(), rc.coldWetlandShore());
        if (rc.coldWetland() >= MIN_COLD_WETLAND) {
            double share = rc.coldWetlandPeat() / (double) rc.coldWetland();
            helper.assertTrue(share >= MIN_PEAT_SHARE, "only " + rc.coldWetlandPeat() + " of " + rc.coldWetland()
                    + " cold-wetland columns are on a peatland biome: " + rc.coldWetlandBy());
        } else {
            DeepTime.LOGGER.info("[gametest] expandedEcosphereBiomesFollowTheirRules: {} cold-wetland columns (fewer than {}): peatland share logged only",
                    rc.coldWetland(), MIN_COLD_WETLAND);
        }
        helper.succeed();
    }

    /** One biome query at the surface of block (x, z), so the level binds its rules. */
    private static Holder<Biome> sample(ServerLevel level, int x, int z) {
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        var sampler = level.getChunkSource().randomState().sampler();
        int y = (int) Math.floor(((com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain) g.terrain()).height(x, z));
        return g.getBiomeSource().getNoiseBiome(x >> 2, y >> 2, z >> 2, sampler);
    }
}
