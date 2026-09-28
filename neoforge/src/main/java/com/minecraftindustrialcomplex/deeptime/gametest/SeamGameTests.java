// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * The seam in a real Toroidal World-shaped Deep Time overworld (P0-06). Planet namespace: these
 * run only on the dev planet server (./gametest-planet.sh), whose overworld is the planet; the
 * GameTest server's flat overworld has no seam. The harness positions them next to x = C/2.
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class SeamGameTests {

    private SeamGameTests() {}

    static int circumference(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)) {
            helper.fail("the test level is not a Deep Time planet: " + level.getChunkSource().getGenerator().getClass().getName());
            throw new IllegalStateException();
        }
        return g.dataRef().circumferenceBlocks();
    }

    /**
     * Across the seam the world continues. Positions past the edge are folded through TW's API
     * first (TW's rule: fold a position before using it; a raw read at chunk C/32 can stall in TW's
     * seam-generation queue): x = C/2 + k folds to −C/2 + k, and the step from the last canonical
     * column (C/2 − 1) to the first (−C/2) is no larger than the terrain's own steps beside it.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void columnsContinuous(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        var shape = com.toroidalworld.api.v1.ToroidalWorldApi.shapeOf(level).orElse(null);
        if (shape == null) {
            helper.fail("the level has no Toroidal World shape");
            return;
        }
        int badFolds = 0, jumps = 0, rows = 0, unloadedRows = 0;
        StringBuilder detail = new StringBuilder();
        // Rows inside the strip the harness forceloads around the seam (z ± 32).
        for (int i = 0; i < 16; i++) {
            int z = (i - 8) * 4 + 1;
            for (int k = 0; k < 4; k++) {
                var folded = shape.fold(new net.minecraft.core.BlockPos(c / 2 + k, 64, z));
                if (folded.getX() != -c / 2 + k || folded.getZ() != z) {
                    badFolds++;
                    detail.append(" fold(").append(c / 2 + k).append(")=").append(folded.getX());
                }
            }
            // Only chunks the harness's forceload already holds: a synchronous load of a seam chunk
            // can wait in TW's seam-generation queue (P0-06 finding), so the test never forces one.
            var east = level.getChunkSource().getChunkNow((c / 2 - 1) >> 4, z >> 4);
            var west = level.getChunkSource().getChunkNow((-c / 2) >> 4, z >> 4);
            if (east == null || west == null) {
                unloadedRows++;
                continue;
            }
            rows++;
            int a = east.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2 - 2, z);
            int b = east.getHeight(Heightmap.Types.OCEAN_FLOOR, c / 2 - 1, z);
            int across = west.getHeight(Heightmap.Types.OCEAN_FLOOR, -c / 2, z);
            int after = west.getHeight(Heightmap.Types.OCEAN_FLOOR, -c / 2 + 1, z);
            if (Math.abs(across - b) > Math.max(Math.max(Math.abs(b - a), Math.abs(after - across)), 2)) {
                jumps++;
                detail.append(" jump(z=").append(z).append("):").append(b).append("->").append(across);
            }
        }
        DeepTime.LOGGER.info("[gametest] columnsContinuous C={} rows={} unloadedRows={} badFolds={} seamJumps={}{}", c, rows,
                unloadedRows, badFolds, jumps, detail);
        helper.assertTrue(rows >= 8, "only " + rows + " seam rows loaded");
        helper.assertTrue(badFolds == 0 && jumps == 0, "seam discontinuities:" + detail);
        helper.succeed();
    }

    /**
     * An entity that moves itself across the seam is still one entity, now inside the canonical
     * span. TW folds a non-player entity at the end of its own server tick, so the entity must move
     * under its own motion (a gravity-free snowball flying east), not be pushed from outside.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void entityCrossesSeam(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = circumference(helper);
        var ball = net.minecraft.world.entity.EntityType.SNOWBALL.create(level);
        if (ball == null) {
            helper.fail("could not create a snowball");
            return;
        }
        double y = 300;   // clear air, far above any terrain near the seam
        ball.setNoGravity(true);
        ball.moveTo(c / 2.0 - 3.5, y, 0.5, 0, 0);
        ball.setDeltaMovement(0.25, 0, 0);   // 5 blocks a second eastward, across x = C/2
        level.addFreshEntity(ball);
        var id = ball.getUUID();
        helper.runAfterDelay(40, () -> {
            var found = level.getEntity(id);
            double x = found == null ? Double.NaN : found.getX();
            boolean inSpan = found != null && x >= -c / 2.0 && x < c / 2.0;
            boolean crossed = found != null && x < 0;   // it now flies on from the west edge
            DeepTime.LOGGER.info("[gametest] entityCrossesSeam C={} found={} x={} inCanonicalSpan={} crossed={}", c,
                    found != null, x, inSpan, crossed);
            if (found != null)
                found.discard();
            helper.assertTrue(found != null, "the entity vanished at the seam");
            helper.assertTrue(inSpan && crossed, "entity at x=" + x + ": expected just east of " + (-c / 2) + " after crossing");
            helper.succeed();
        });
    }
}
