// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.ore.ClaimOresBiomeModifier;
import com.minecraftindustrialcomplex.deeptime.ore.ClaimRules;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.level.biome.Biomes;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

/**
 * Ore claiming on a real Deep Time world (P1-15, plan §7.7; the planet server,
 * {@code ./gametest-planet.sh --run claimoninplanetworld@0,100,0}): the overworld is Deep Time's, so
 * {@code deeptime:claim_ores} removed the claimed ores and the rock blobs from the biomes, and kept
 * the unknown ores.
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class PlanetClaimGameTests {

    private PlanetClaimGameTests() {}

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void claimOnInPlanetWorld(GameTestHelper helper) {
        MinecraftServer server = helper.getLevel().getServer();
        boolean on = ClaimOresBiomeModifier.deepTimeWorld(server);
        Set<String> plains = ClaimGameTests.features(server, Biomes.PLAINS);
        List<String> kept = new ArrayList<>(), lost = new ArrayList<>();
        for (String f : ClaimGameTests.concat(ClaimGameTests.VANILLA_ORE.subList(0, 7), ClaimGameTests.VANILLA_ROCK.subList(0, 2)))
            if (plains.contains(f))
                kept.add(f);
        for (String f : List.of("minecraft:ore_diamond", "minecraft:ore_lapis", "minecraft:ore_redstone", "minecraft:ore_dirt"))
            if (!plains.contains(f))
                lost.add(f);
        if (ModList.get().isLoaded("create") && plains.contains("create:zinc_ore"))
            kept.add("create:zinc_ore");
        if (ModList.get().isLoaded("createnuclear") && !plains.contains("createnuclear:uranium_ore"))
            lost.add("createnuclear:uranium_ore");
        DeepTime.LOGGER.info("[gametest] claimOnInPlanetWorld deepTimeWorld={} removed={} plainsFeatures={} claimedButKept={} unknownButLost={}",
                on, ClaimRules.REMOVED.size(), plains.size(), kept, lost);
        helper.assertTrue(on, "the planet server's overworld is not Deep Time's");
        helper.assertTrue(!ClaimRules.REMOVED.isEmpty(), "the modifier removed nothing");
        helper.assertTrue(kept.isEmpty(), "claimed features still in plains: " + kept);
        helper.assertTrue(lost.isEmpty(), "unknown ores removed from plains: " + lost);
        helper.succeed();
    }
}
