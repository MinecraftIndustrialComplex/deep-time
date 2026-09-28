// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.sable.SableTestOps;
import com.toroidalworld.api.v1.ToroidalShape;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.phys.Vec3;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * Sable ships at the seam (P0-06). Planet namespace, dev planet server only (Sable is loaded
 * there from the standalone set). Every Sable call goes through {@link SableTestOps}: no Sable
 * type may appear in this class's method signatures (cold-read review 2, gap 2).
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class SableSeamGameTests {

    /** East speed of the hull, blocks per second: an Aeronautics jet at full speed or more. */
    static final double SPEED = 40.0;
    /** How far west of x = C/2 the hull is assembled (its anchor column). */
    static final int START_WEST = 12;
    static final int MAX_FLIGHT_TICKS = 60;

    private SableSeamGameTests() {}

    /**
     * A small hull (3 × 3 deck plus a 1-block mast: 10 blocks) assembled 12 blocks west of the
     * seam and pushed east at 40 blocks/s crosses x = C/2. After crossing it must still exist,
     * carry the same number of blocks, and sit folded into the canonical span just east of −C/2
     * (TW reseats a body whose centre passes the edge). Twenty attempts, all required: vanilla
     * reruns the test itself and reports one PASS only after 20 passing attempts; each attempt is
     * also logged as {@code [gametest-attempt]} and counted in the summary.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200, attempts = 20, requiredSuccesses = 20)
    public static void jetCrossesSeam(GameTestHelper helper) {
        ServerLevel level = helper.getLevel();
        int c = SeamGameTests.circumference(helper);
        ToroidalShape shape = SeamGameTests.shape(helper);
        if (!ModList.get().isLoaded(SableTestOps.MOD_ID)) {
            helper.fail("Sable is not loaded on this server");
            return;
        }
        BlockPos spot = helper.absolutePos(GameTestSupport.CENTRE);
        int y = spot.getY() + 30, z = spot.getZ();
        BlockPos anchor = new BlockPos(c / 2 - START_WEST, y, z);
        List<BlockPos> hull = new ArrayList<>();
        for (int dx = -1; dx <= 1; dx++)
            for (int dz = -1; dz <= 1; dz++)
                hull.add(anchor.offset(dx, 0, dz));
        hull.add(anchor.above());
        for (BlockPos p : hull)
            level.setBlock(p, Blocks.OAK_PLANKS.defaultBlockState(), 3);
        UUID id = SableTestOps.assemble(level, anchor, hull);
        int blocks0 = SableTestOps.blockCount(level, id);
        Vec3 start = SableTestOps.position(level, id);
        SableTestOps.addVelocity(level, id, new Vec3(SPEED, 0, 0));
        long t0 = helper.getTick();
        long[] crossedAt = {-1};
        int[] pushes = {1};
        helper.onEachTick(() -> {
            long t = helper.getTick() - t0;
            Vec3 pos = SableTestOps.position(level, id);
            if (pos == null || !SableTestOps.exists(level, id)) {
                finish(helper, level, id, false, String.format("hull %s vanished after %d ticks", id, t));
                return;
            }
            // If the body joined the physics pipeline only after the first push, push again (once).
            Vec3 v0 = SableTestOps.velocity(level, id);
            if (crossedAt[0] < 0 && t >= 1 && pushes[0] < 2 && v0 != null && v0.x < SPEED / 2) {
                SableTestOps.addVelocity(level, id, new Vec3(SPEED - v0.x, 0, 0));
                pushes[0]++;
            }
            if (crossedAt[0] < 0) {
                if (pos.x < 0)
                    crossedAt[0] = t;   // folded: it now flies on from the west edge
                else if (t > MAX_FLIGHT_TICKS)
                    finish(helper, level, id, false, String.format("no fold after %d ticks: x=%.2f (edge %d) vx=%.2f pushes=%d",
                            t, pos.x, c / 2, v0 == null ? Double.NaN : v0.x, pushes[0]));
                return;
            }
            if (t - crossedAt[0] < 2)
                return;   // two ticks on the far side
            int blocks = SableTestOps.blockCount(level, id);
            Vec3 v = SableTestOps.velocity(level, id);
            boolean inSpan = pos.x >= -c / 2.0 && pos.x < c / 2.0;
            boolean nearWestEdge = pos.x < -c / 2.0 + 40;
            boolean folded = shape.fold(BlockPos.containing(pos)).getX() == BlockPos.containing(pos).getX();
            boolean ok = blocks0 == hull.size() && blocks == blocks0 && inSpan && nearWestEdge && folded;
            finish(helper, level, id, ok, String.format(
                    "C=%d start.x=%.2f crossedAtTick=%d x=%.2f y=%.2f blocks=%d/%d assembled=%d vx=%.2f pushes=%d inSpan=%s",
                    c, start == null ? Double.NaN : start.x, crossedAt[0], pos.x, pos.y, blocks, blocks0, hull.size(),
                    v == null ? Double.NaN : v.x, pushes[0], inSpan));
        });
    }

    private static void finish(GameTestHelper helper, ServerLevel level, UUID id, boolean ok, String detail) {
        DeepTimeTestReporter.INSTANCE.attempt("jetCrossesSeam", ok, detail);
        DeepTime.LOGGER.info("[gametest] jetCrossesSeam {} {}", ok ? "ok" : "FAILED", detail);
        SableTestOps.remove(level, id);
        if (!ok)
            helper.fail("jetCrossesSeam: " + detail);
        helper.succeed();
    }
}
