// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.ore.ClaimOresBiomeModifier;
import com.minecraftindustrialcomplex.deeptime.ore.ClaimRules;
import net.minecraft.core.registries.Registries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Biomes;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;

/**
 * Ore claiming (P1-15, plan §7.7): {@code deeptime:claim_ores} removes the scatter of the materials
 * Deep Time simulates and (plan Q4, default on) the random rock blobs, only in worlds whose overworld
 * is Deep Time's, and keeps unknown ores. The GameTest server's overworld is not Deep Time's, so
 * there the modifier must change nothing and its rules are checked as a dry run over every
 * registered placed feature (vanilla's, and with {@code -PwithMic} Create's, Create Nuclear's,
 * Ice and Fire's, Destroy's, Create Propulsion's, Born in Chaos's and Terralith's);
 * {@link PlanetClaimGameTests} checks the removal itself on the planet server.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class ClaimGameTests {

    private ClaimGameTests() {}

    static ResourceLocation id(String s) {
        return ResourceLocation.parse(s);
    }

    /** Placed-feature ids of a biome. */
    static Set<String> features(MinecraftServer server, ResourceKey<Biome> key) {
        Biome b = server.registryAccess().registryOrThrow(Registries.BIOME).getOrThrow(key);
        return ClaimOresBiomeModifier.features(b).stream()
                .map(h -> h.unwrapKey().map(k -> k.location().toString()).orElse("direct"))
                .collect(Collectors.toSet());
    }

    /** Vanilla placed features Deep Time claims (their materials are simulated). */
    static final List<String> VANILLA_ORE = List.of("minecraft:ore_coal_upper", "minecraft:ore_coal_lower",
            "minecraft:ore_iron_upper", "minecraft:ore_iron_middle", "minecraft:ore_iron_small", "minecraft:ore_gold",
            "minecraft:ore_gold_lower", "minecraft:ore_copper", "minecraft:ore_copper_large");
    /** Vanilla rock blobs (plan Q4). */
    static final List<String> VANILLA_ROCK = List.of("minecraft:ore_granite_upper", "minecraft:ore_granite_lower",
            "minecraft:ore_diorite_upper", "minecraft:ore_diorite_lower", "minecraft:ore_andesite_upper",
            "minecraft:ore_andesite_lower", "minecraft:ore_tuff");
    /** Vanilla features that stay: unknown ores and loose-rock pockets. */
    static final List<String> VANILLA_KEEP = List.of("minecraft:ore_diamond", "minecraft:ore_lapis", "minecraft:ore_redstone",
            "minecraft:ore_emerald", "minecraft:ore_dirt", "minecraft:ore_gravel", "minecraft:ore_infested");

    /** Expected verdicts of MIC mods' placed features (checked when the mod is loaded). */
    static Map<String, ClaimRules.Verdict> modExpectations(boolean rockTags) {
        Map<String, ClaimRules.Verdict> m = new LinkedHashMap<>();
        m.put("create:zinc_ore", ClaimRules.Verdict.ORE);
        m.put("create:striated_ores_overworld", ClaimRules.Verdict.ROCK);
        m.put("createnuclear:lead_ore", ClaimRules.Verdict.ORE);
        m.put("createnuclear:uranium_ore", ClaimRules.Verdict.KEEP);
        // Rock and uranium ore in one layered feature: an unknown ore keeps it.
        m.put("createnuclear:striated_ores_overworld", ClaimRules.Verdict.KEEP);
        m.put("iceandfire:silver_ore", ClaimRules.Verdict.ORE);
        m.put("iceandfire:sapphire_ore", ClaimRules.Verdict.KEEP);
        m.put("destroy:nickel_ore", ClaimRules.Verdict.KEEP);
        m.put("destroy:fluorite_ore", ClaimRules.Verdict.KEEP);
        m.put("createpropulsion:ore_platinum", ClaimRules.Verdict.KEEP);
        m.put("createmetallurgy:wolframite_ore", ClaimRules.Verdict.KEEP);
        // Black argillite is a rock only through the MIC rock tags (coal's first fallback).
        if (rockTags)
            m.put("born_in_chaos_v1:black_argillite", ClaimRules.Verdict.ROCK);
        m.put("terralith:cliffs/granite/granitifier", ClaimRules.Verdict.ROCK);
        m.put("terralith:yosemite/diorite", ClaimRules.Verdict.ROCK);
        m.put("terralith:cave/fungal/ore_coal_extra", ClaimRules.Verdict.ORE);
        // Terrain features that hold ore or rock among other blocks are not ore-shaped: they stay.
        m.put("terralith:skylands/floating_islands", ClaimRules.Verdict.KEEP);
        m.put("terralith:taiga/mega/boulders", ClaimRules.Verdict.KEEP);
        m.put("terralith:highlands/pillar", ClaimRules.Verdict.KEEP);
        return m;
    }

    /** In a world whose overworld is not Deep Time's, the modifier removes nothing. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void claimOffWhenOverworldNotGeologic(GameTestHelper helper) {
        MinecraftServer server = helper.getLevel().getServer();
        boolean on = ClaimOresBiomeModifier.deepTimeWorld(server);
        Set<String> plains = features(server, Biomes.PLAINS);
        List<String> missing = new ArrayList<>();
        for (String f : concat(VANILLA_ORE.subList(0, 7), VANILLA_ROCK.subList(0, 2), List.of("minecraft:ore_diamond")))
            if (!plains.contains(f))
                missing.add(f);
        DeepTime.LOGGER.info("[gametest] claimOffWhenOverworldNotGeologic deepTimeWorld={} modifiers={} removed={} plainsFeatures={} missing={}",
                on, ClaimRules.modifiers(server).keySet(), ClaimRules.REMOVED.size(), plains.size(), missing);
        helper.assertTrue(!on, "the GameTest overworld counts as a Deep Time world");
        helper.assertTrue(ClaimRules.modifiers(server).containsKey(id("deeptime:claim_ores")), "deeptime:claim_ores is not registered");
        helper.assertTrue(ClaimRules.REMOVED.isEmpty(), "the modifier removed features: " + ClaimRules.REMOVED);
        helper.assertTrue(missing.isEmpty(), "plains lost features outside a Deep Time world: " + missing);
        helper.succeed();
    }

    /**
     * The claim rules, as a dry run over every registered placed feature: simulated materials'
     * ores and rock blobs go, unknown ores and loose-rock pockets stay; MIC's mods' features sort
     * the same way (checked for each mod that is loaded).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 400)
    public static void claimRulesSortFeatures(GameTestHelper helper) {
        MinecraftServer server = helper.getLevel().getServer();
        ClaimRules rules = ClaimRules.dryRun(server);
        Map<ResourceLocation, ClaimRules.Verdict> plan = rules.planAll(server.registryAccess());
        boolean rockTags = GameTestSupport.extraDatapackActive(server);
        List<String> wrong = new ArrayList<>();
        int checked = 0;
        for (String f : VANILLA_ORE)
            checked += expect(plan, f, ClaimRules.Verdict.ORE, wrong, rules);
        for (String f : VANILLA_ROCK)
            checked += expect(plan, f, ClaimRules.Verdict.ROCK, wrong, rules);
        for (String f : VANILLA_KEEP)
            checked += expect(plan, f, ClaimRules.Verdict.KEEP, wrong, rules);
        List<String> modsChecked = new ArrayList<>();
        for (var e : modExpectations(rockTags).entrySet()) {
            String ns = e.getKey().substring(0, e.getKey().indexOf(':'));
            if (!ModList.get().isLoaded(ns) || !plan.containsKey(id(e.getKey())))
                continue;
            checked += expect(plan, e.getKey(), e.getValue(), wrong, rules);
            modsChecked.add(e.getKey());
        }
        var byVerdict = ClaimRules.byVerdict(plan);
        DeepTime.LOGGER.info("[gametest] claimRulesSortFeatures materials={} checked={} wrong={} modFeatures={} ore={} rock={} (of {} placed features) {}",
                rules.materials(), checked, wrong.size(), modsChecked,
                byVerdict.getOrDefault(ClaimRules.Verdict.ORE, List.of()),
                byVerdict.getOrDefault(ClaimRules.Verdict.ROCK, List.of()), plan.size(), wrong);
        helper.assertTrue(rules.materials().containsAll(List.of(id("c:copper"), id("c:iron"), id("c:coal"), id("c:gold"),
                id("c:zinc"), id("c:lead"))), "claimed materials " + rules.materials());
        helper.assertTrue(checked >= VANILLA_ORE.size() + VANILLA_ROCK.size() + VANILLA_KEEP.size(), "checked only " + checked);
        helper.assertTrue(wrong.isEmpty(), wrong.size() + " wrong: " + wrong);
        helper.succeed();
    }

    static int expect(Map<ResourceLocation, ClaimRules.Verdict> plan, String f, ClaimRules.Verdict want, List<String> wrong,
                      ClaimRules rules) {
        ClaimRules.Verdict got = plan.get(id(f));
        if (got == null) {
            wrong.add(f + ": not registered");
            return 0;
        }
        if (got != want)
            wrong.add(f + ": " + got + " (want " + want + ", places " + rules.placedBlocksOf(id(f)) + ")");
        return 1;
    }

    @SafeVarargs
    static List<String> concat(List<String>... lists) {
        List<String> out = new ArrayList<>();
        for (List<String> l : lists)
            out.addAll(l);
        return out;
    }
}
