// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.api.DeepTimeSites;
import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.ore.OreData;
import com.minecraftindustrialcomplex.deeptime.server.SiteCommands;
import com.minecraftindustrialcomplex.deeptime.server.SpawnHandler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;

/**
 * Spawn chosen for its essentials (P1-15, plan §7.8): on the synthetic deposit world, whose spawn
 * candidates the core ranked by the preset's essentials, the spawn Deep Time picks is dry land and
 * every essential that sets {@code max_blocks_from_spawn} has a site of its material within that
 * reach, measured from the site registry through {@link DeepTimeSites}.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class SpawnGameTests {

    private SpawnGameTests() {}

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void essentialsWithinReach(GameTestHelper helper) {
        ServerLevel level = DepositGameTests.level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = DepositGameTests.data(helper, level);
        JsonObject file;
        try {
            file = JsonParser.parseString(Files.readString(wd.folder().resolve("spawn/candidates.json"))).getAsJsonObject();
        } catch (Exception e) {
            helper.fail("cannot read spawn/candidates.json: " + e);
            return;
        }
        helper.assertTrue(file.has("ranked_by"), "the deposit world's spawn candidates are not ranked by essentials");
        var choice = SpawnHandler.choose(level, wd);
        helper.assertTrue(choice.isPresent(), "no spawn chosen");
        var spawn = choice.get().pos();
        int sea = level.getChunkSource().getGenerator().getSeaLevel();
        DeepTimeSites sites = DeepTimeSites.of(level);
        int c = wd.circumference();
        List<String> report = new ArrayList<>(), far = new ArrayList<>();
        for (var e : file.getAsJsonArray("ranked_by")) {
            var r = e.getAsJsonObject();
            ResourceLocation m = ResourceLocation.parse(r.get("material").getAsString());
            double max = r.get("max_blocks_from_spawn").getAsDouble();
            double best = Double.POSITIVE_INFINITY;
            int bestId = -1;
            for (Site s : sites.all()) {
                if (!s.produces(m))
                    continue;
                double d = SiteCommands.distance(c, spawn.getX(), spawn.getZ(), s.center().getX(), s.center().getZ());
                if (d < best) {
                    best = d;
                    bestId = s.id();
                }
            }
            report.add(String.format("%s nearest site %d at %.0f of %.0f", m, bestId, best, max));
            if (!(best <= max))
                far.add(m + " (" + Math.round(best) + " > " + Math.round(max) + ")");
        }
        List<String> registry = OreData.entries(level.registryAccess(), OreData.ESSENTIALS).stream()
                .map(en -> en.getValue().material() + en.getValue().maxBlocksFromSpawn().map(v -> "≤" + v).orElse("")).toList();
        DeepTime.LOGGER.info("[gametest] essentialsWithinReach spawn={} unmet={} {} (the pack's deeptime:essentials registry: {})",
                spawn.toShortString(), choice.get().candidate().get("unmet"), report, registry);
        helper.assertTrue(spawn.getY() > sea, "spawn " + spawn.toShortString() + " is not above sea level " + sea);
        helper.assertTrue(far.isEmpty(), "essentials out of reach of the spawn: " + far);
        helper.assertTrue(!registry.isEmpty(), "the deeptime:essentials registry is empty");
        helper.succeed();
    }
}
