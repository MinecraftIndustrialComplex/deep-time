// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapPrototype;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityType;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * The pole soft cap (plan §7.11) on the dev planet server. P0-06 holds only the PROTOTYPE
 * ({@link PoleCapPrototype}); P1-18 replaces it with {@code entityPushedBack},
 * {@code sableHullPushedBack} and {@code hardStopAt80}.
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class PoleCapGameTests {

    /** Latitudes (south, +z) of the probes; the first is below the cap and must not move. */
    private static final double[] LATITUDES = {74.5, 76.0, 77.5, 79.0};
    private static final int RUN_TICKS = 40;

    private PoleCapGameTests() {}

    /**
     * PROTOTYPE pushback: gravity-free probes (snowballs tagged {@link PoleCapPrototype#PROBE_TAG},
     * which move only under their own velocity) at 74.5°, 76°, 77.5° and 79°, and an untagged
     * minecart (a vehicle) at 79°, start at rest. After two seconds the probe below 75° has not
     * moved, the others have moved toward the equator, further the higher they started, and the
     * minecart has moved toward the equator too. The harness forceloads a z strip wide enough for
     * all of them: run it at x,y,z(77.5°) with {@code ~16,160} at C = 16,384.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void prototypePushback(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = SeamGameTests.circumference(helper);
        BlockPos spot = SeamGameTests.surfaceSpot(helper);
        double y = spot.getY() + 10.5;
        List<BlockPos> starts = new ArrayList<>();
        for (double lat : LATITUDES)
            starts.add(BlockPos.containing(spot.getX() + 0.5, y, -PoleCapPrototype.zOfLatitude(c, lat) + 0.5));
        List<Entity> probes = new ArrayList<>();
        long[] spawnedAt = {-1};
        helper.onEachTick(() -> {
            if (spawnedAt[0] < 0) {
                // Wait until the forceloaded strip is generated and ticking entities.
                for (BlockPos p : starts)
                    if (!level.isPositionEntityTicking(p))
                        return;
                for (BlockPos p : starts) {
                    var ball = EntityType.SNOWBALL.create(level);
                    ball.setNoGravity(true);
                    ball.addTag(PoleCapPrototype.PROBE_TAG);
                    ball.moveTo(p.getX() + 0.5, y, p.getZ() + 0.5, 0, 0);
                    ball.setDeltaMovement(0, 0, 0);
                    level.addFreshEntity(ball);
                    probes.add(ball);
                }
                var cart = EntityType.MINECART.create(level);
                BlockPos last = starts.get(starts.size() - 1);
                cart.setNoGravity(true);
                cart.moveTo(last.getX() + 3.5, y, last.getZ() + 0.5, 0, 0);
                level.addFreshEntity(cart);
                probes.add(cart);
                spawnedAt[0] = helper.getTick();
                return;
            }
            if (helper.getTick() - spawnedAt[0] < RUN_TICKS)
                return;
            double[] dz = new double[probes.size()];
            StringBuilder detail = new StringBuilder();
            for (int i = 0; i < probes.size(); i++) {
                Entity e = probes.get(i);
                double z0 = (i < starts.size() ? starts.get(i) : starts.get(starts.size() - 1)).getZ() + 0.5;
                dz[i] = e.getZ() - z0;
                double lat = Math.abs(PoleCapPrototype.latitudeDeg(c, z0));
                detail.append(String.format(" %s@%.2f°:dz=%.3f", i < starts.size() ? "probe" : "minecart", lat, dz[i]));
                e.discard();
            }
            DeepTime.LOGGER.info("[gametest] prototypePushback (PROTOTYPE, P1-18 builds the cap) C={} ticks={} maxNudge={}{}",
                    c, RUN_TICKS, PoleCapPrototype.MAX_NUDGE, detail);
            // z > 0 is the southern hemisphere: equatorward is -z.
            helper.assertTrue(Math.abs(dz[0]) < 1e-3, "the probe below 75° moved: dz=" + dz[0]);
            for (int i = 1; i < LATITUDES.length; i++) {
                helper.assertTrue(dz[i] < -0.1, "the probe at " + LATITUDES[i] + "° did not move equatorward: dz=" + dz[i]);
                if (i > 1)
                    helper.assertTrue(dz[i] < dz[i - 1], "pushback does not grow with latitude:" + detail);
            }
            helper.assertTrue(dz[LATITUDES.length] < -0.1, "the minecart at 79° did not move equatorward:" + detail);
            helper.succeed();
        });
    }
}
