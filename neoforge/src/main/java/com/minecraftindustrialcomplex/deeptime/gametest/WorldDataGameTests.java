// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.server.SpawnHandler;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.StrataPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * The generator on world data (P0-10): the {@code deeptime:test_planet_data} level reads the
 * synthetic wrap-test world ({@code deeptime synth --kind wrap-test --size 1024}) through the
 * pure-Java reader. Heights are read at NOISE status and strata at SURFACE status, before carvers.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class WorldDataGameTests {

    private WorldDataGameTests() {}

    static ServerLevel level(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_DATA);
        if (level == null)
            helper.fail("level " + GameTestSupport.TEST_PLANET_DATA.location() + " does not exist");
        return level;
    }

    static WorldDataTerrain terrain(GameTestHelper helper, ServerLevel level) {
        if (level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)
            return w;
        helper.fail("test_planet_data is not generating from world data");
        throw new IllegalStateException();
    }

    /** Same tolerances as the analytic test: corner columns (x ≡ z ≡ 0 mod 4) exact, others within 1. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void heightMatchesRaster(GameTestHelper helper) {
        ServerLevel level = level(helper);
        WorldDataTerrain t = terrain(helper, level);
        int c = t.circumference();
        int corners = 0, cornerMax = 0, maxDiff = 0, worst = 0;
        String worstAt = "";
        for (int i = 0; i < 128; i++) {
            boolean corner = i % 2 == 0;
            int x = (-c / 2 + 8 + i * (c / 130)) & ~3;
            int z = (((i * 37) % 97 - 48) * 5) & ~3;
            if (!corner) {
                x += 1 + (i % 3);
                z += 2;
            }
            int expected = (int) Math.ceil(t.height(x, z)) - 1;
            int actual = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.NOISE, true)
                    .getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
            int diff = Math.abs(actual - expected);
            if (corner) {
                corners++;
                cornerMax = Math.max(cornerMax, diff);
            }
            maxDiff = Math.max(maxDiff, diff);
            if (diff > worst) {
                worst = diff;
                worstAt = "(" + x + ", " + z + ") expected " + expected + " got " + actual + " H=" + t.height(x, z);
            }
        }
        DeepTime.LOGGER.info("[gametest] heightMatchesRaster columns=128 cornerColumns={} cornerMaxAbsDiff={} maxAbsDiff={} worst={}",
                corners, cornerMax, maxDiff, worstAt);
        helper.assertTrue(cornerMax == 0, "corner columns differ by up to " + cornerMax + ": " + worstAt);
        helper.assertTrue(maxDiff <= 1, "columns differ by up to " + maxDiff + ": " + worstAt);
        helper.succeed();
    }

    /** Below the soil, every sampled block is the block of the class the horizon stack names there. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void strataClassesResolve(GameTestHelper helper) {
        ServerLevel level = level(helper);
        var wd = terrain(helper, level).data();
        int checked = 0, wrong = 0;
        StringBuilder detail = new StringBuilder();
        for (int i = 0; i < 16; i++) {
            int x = -400 + i * 53, z = (i % 4 - 2) * 70 + 3;
            var chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
            int surface = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
            for (int depth : new int[]{20, 80}) {
                int y = surface - depth;
                var cls = StrataPass.classAt(wd, x, y, z);
                if (cls.isEmpty())
                    continue;
                var expected = RockPalette.resolve(cls.get());
                var actual = chunk.getBlockState(new BlockPos(x, y, z));
                checked++;
                if (expected.isEmpty() || actual != expected.get()) {
                    wrong++;
                    if (detail.length() < 400)
                        detail.append(" (").append(x).append(',').append(y).append(',').append(z).append(") ").append(cls.get())
                                .append(": ").append(actual.getBlock()).append(';');
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] strataClassesResolve checked={} wrong={} classes={}{}", checked, wrong,
                StrataPass.classes(wd), detail);
        helper.assertTrue(checked >= 16, "only " + checked + " strata samples");
        helper.assertTrue(wrong == 0, wrong + " blocks are not their class's rock:" + detail);
        helper.succeed();
    }

    /** The world data's spawn candidates give a spawn on dry land. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void spawnOnLand(GameTestHelper helper) {
        ServerLevel level = level(helper);
        var pos = SpawnHandler.chooseSpawn(level, terrain(helper, level).data());
        int sea = level.getChunkSource().getGenerator().getSeaLevel();
        DeepTime.LOGGER.info("[gametest] spawnOnLand spawn={} seaLevel={}", pos.orElse(null), sea);
        helper.assertTrue(pos.isPresent() && pos.get().getY() > sea, "no dry-land spawn: " + pos);
        helper.succeed();
    }
}
