// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * The generator on the analytic test terrain (P0-05, plan §9.4). Tests run in the GameTest
 * server's flat overworld and read terrain from the {@code deeptime:test_planet} level, whose
 * router has no noise caves, at NOISE status: only the density fill has run, so carvers,
 * surface rules and features cannot move the surface, and the worldgen heightmaps still exist.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class GeneratorGameTests {

    private GeneratorGameTests() {}

    static ServerLevel testPlanet(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET);
        if (level == null)
            helper.fail("level " + GameTestSupport.TEST_PLANET.location() + " does not exist (GameTestServerMixin not applied?)");
        return level;
    }

    static GeologicChunkGenerator generator(GameTestHelper helper, ServerLevel level) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)) {
            helper.fail("expected a GeologicChunkGenerator, got " + level.getChunkSource().getGenerator().getClass().getName());
            throw new IllegalStateException();
        }
        return g;
    }

    /**
     * The highest block that blocks motion and holds no fluid. {@code getHeight} on a
     * {@code *_WG} heightmap already returns that y (the heightmap stores y + 1 and
     * {@code ChunkAccess.getHeight} subtracts 1); compared directly with ceil(H) − 1.
     */
    static int surfaceAtNoise(ServerLevel level, int x, int z) {
        ChunkAccess chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.NOISE, true);
        return chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void testPlanetDimensionExists(GameTestHelper helper) {
        ServerLevel level = testPlanet(helper);
        GeologicChunkGenerator g = generator(helper, level);
        DeepTime.LOGGER.info("[gametest] testPlanetDimensionExists level={} generator={} C={} minY={} height={}",
                level.dimension().location(), g.getClass().getSimpleName(), g.terrain().circumference(),
                level.getMinBuildHeight(), level.getHeight());
        helper.assertTrue(level.getMinBuildHeight() == -352 && level.getHeight() == 992,
                "build range " + level.getMinBuildHeight() + " + " + level.getHeight() + ", expected -352 + 992");
        helper.succeed();
    }

    /**
     * 128 columns: 64 cell-corner columns (x ≡ z ≡ 0 mod 4), where vanilla's interpolation
     * reproduces H exactly (tolerance 0), and 64 columns between corners (tolerance 1), spread
     * over ocean, lowland, the seam continent and the mountain range.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void heightMatchesReference(GameTestHelper helper) {
        ServerLevel level = testPlanet(helper);
        TerrainSource t = generator(helper, level).terrain();
        int c = t.circumference();
        int corners = 0, cornerMax = 0, maxDiff = 0, worst = 0;
        String worstAt = "";
        for (int i = 0; i < 128; i++) {
            boolean corner = i % 2 == 0;
            // Longitudes sweep the planet; every 8th sample lands on the mountain range.
            int x = (i % 8 == 3) ? c / 8 + (i - 64) * 3 : -c / 2 + 64 + i * (c / 130);
            int z = (i % 8 == 3) ? c / 32 + (i % 5) * 20 : ((i * 37) % 97 - 48) * 60;
            x &= ~3;
            z &= ~3;
            if (!corner) {
                x += 1 + (i % 3);
                z += 2;
            }
            int expected = (int) Math.ceil(t.height(x, z)) - 1;
            int actual = surfaceAtNoise(level, x, z);
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
        DeepTime.LOGGER.info("[gametest] heightMatchesReference columns=128 cornerColumns={} cornerMaxAbsDiff={} maxAbsDiff={} worst={}",
                corners, cornerMax, maxDiff, worstAt);
        helper.assertTrue(cornerMax == 0, "corner columns differ by up to " + cornerMax + ": " + worstAt);
        helper.assertTrue(maxDiff <= 1, "columns differ by up to " + maxDiff + ": " + worstAt);
        helper.succeed();
    }

    /** Columns x = C/2 − 1 and x = −C/2 continue each other (the terrain is periodic in x). */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void periodicAcrossSeam(GameTestHelper helper) {
        ServerLevel level = testPlanet(helper);
        int c = generator(helper, level).terrain().circumference();
        int mismatches = 0;
        StringBuilder detail = new StringBuilder();
        for (int k = 0; k < 16; k++) {
            int z = (k - 8) * 250 + 2;
            int east = surfaceAtNoise(level, c / 2 - 1, z);
            int west = surfaceAtNoise(level, -c / 2, z);
            int eastStep = Math.abs(surfaceAtNoise(level, c / 2 - 2, z) - east);
            int westStep = Math.abs(surfaceAtNoise(level, -c / 2 + 1, z) - west);
            int seamStep = Math.abs(east - west);
            if (seamStep > Math.max(Math.max(eastStep, westStep), 1)) {
                mismatches++;
                detail.append(" z=").append(z).append(": ").append(east).append(" vs ").append(west);
            }
        }
        DeepTime.LOGGER.info("[gametest] periodicAcrossSeam pairs=16 mismatches={}{}", mismatches, detail);
        helper.assertTrue(mismatches == 0, "seam discontinuities:" + detail);
        helper.succeed();
    }

    /** Vanilla's large ore veins are off in Deep Time worlds (decisions file, "Other mods' ores"). */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void oreVeinsDisabled(GameTestHelper helper) {
        ServerLevel level = testPlanet(helper);
        var settings = generator(helper, level).generatorSettings().value();
        var toggle = settings.noiseRouter().veinToggle();
        boolean constantZero = toggle.minValue() == 0.0 && toggle.maxValue() == 0.0;
        DeepTime.LOGGER.info("[gametest] oreVeinsDisabled oreVeinsEnabled={} veinToggleConstantZero={}",
                settings.oreVeinsEnabled(), constantZero);
        helper.assertFalse(settings.oreVeinsEnabled(), "ore veins are enabled");
        helper.assertTrue(constantZero, "vein_toggle is not constant 0: " + toggle);
        helper.succeed();
    }
}
