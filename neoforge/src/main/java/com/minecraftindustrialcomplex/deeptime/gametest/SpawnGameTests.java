// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.server.SiteCommands;
import com.minecraftindustrialcomplex.deeptime.server.SpawnHandler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.nio.file.Files;

/**
 * Spawn follows the standard rules (Ben, 2026-10-05, "I don't believe we need spawn selectoin since
 * we have some passive iron gen like vanilla"; before, P1-15 ranked the candidates by the
 * essentials). On the synthetic deposit world the candidates are dry flat land listed nearest
 * (0, 0) first, with no ranking by the essentials, and the spawn Deep Time picks is the first of
 * them that is dry land once generated: above sea level, on ground that is not fluid, with no
 * fluid in the spawn block.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class SpawnGameTests {

    private SpawnGameTests() {}

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void spawnFollowsTheStandardRules(GameTestHelper helper) {
        ServerLevel level = DepositGameTests.level(helper, GameTestSupport.TEST_DEPOSITS);
        WorldData wd = DepositGameTests.data(helper, level);
        JsonObject file;
        try {
            file = JsonParser.parseString(Files.readString(wd.folder().resolve("spawn/candidates.json"))).getAsJsonObject();
        } catch (Exception e) {
            helper.fail("cannot read spawn/candidates.json: " + e);
            return;
        }
        helper.assertTrue(!file.has("ranked_by"), "the spawn candidates are still ranked by essentials");
        var candidates = file.getAsJsonArray("candidates");
        helper.assertTrue(!candidates.isEmpty(), "no spawn candidates");
        int c = wd.circumference();
        double last = -1;
        for (var e : candidates) {
            var cand = e.getAsJsonObject();
            helper.assertTrue(!cand.has("unmet") && !cand.has("essentials"), "a candidate carries essentials: " + cand);
            double d = SiteCommands.distance(c, 0, 0, cand.get("x").getAsInt(), cand.get("z").getAsInt());
            helper.assertTrue(d >= last, "candidates are not nearest (0, 0) first: " + cand);
            last = d;
        }
        var choice = SpawnHandler.choose(level, wd);
        helper.assertTrue(choice.isPresent(), "no spawn chosen");
        BlockPos spawn = choice.get().pos();
        int sea = level.getChunkSource().getGenerator().getSeaLevel();
        helper.assertTrue(spawn.getY() > sea, "spawn " + spawn.toShortString() + " is not above sea level " + sea);
        helper.assertTrue(level.getBlockState(spawn.below()).getFluidState().isEmpty(), "spawn stands in or on a fluid at " + spawn.toShortString());
        helper.assertTrue(level.getBlockState(spawn).getFluidState().isEmpty(), "spawn block holds a fluid at " + spawn.toShortString());
        DeepTime.LOGGER.info("[gametest] spawnFollowsTheStandardRules spawn={} candidates={} nearest candidate {}",
                spawn.toShortString(), candidates.size(), candidates.get(0));
        helper.succeed();
    }
}
