// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterBodies;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * Delta plains and their distributaries on the delta-test world ({@code deeptime:test_delta},
 * {@code deeptime synth --kind delta-test}; V1 gap 5, HYD-136 to HYD-140).
 *
 * <p>The bake raises a delta plain to a height in (sea level, sea level + 1], so its top block is a
 * block over the sea's top water block, and splits the river at the apex into chains that leave it
 * ({@code from} in the river file). The world here is that result written by hand: a plain at y 63.5,
 * a river that reaches it and two distributaries that leave the river's vertex at the apex, every
 * channel's surface the sea's. The game carves the distributaries as it carves every chain; the
 * plain between them stays dry land. Upstream, a floodplain layer over the river's reach on the
 * high plain gives its banks the levee's sand (HYD-153), and the slope below it keeps soil.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class DeltaGameTests {

    /** The share of checked channel columns whose water must be right. */
    static final double WITHIN_SHARE = 0.95;

    private DeltaGameTests() {}

    private record World(ServerLevel level, WorldData wd, JsonObject note) {}

    private static World world(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_DELTA);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_DELTA.location() + " does not exist");
            return null;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_delta is not generating from world data");
            return null;
        }
        WorldData wd = t.data();
        JsonObject note = wd.manifest().notes().getAsJsonObject("delta");
        if (note == null || wd.rivers() == null) {
            helper.fail("world data " + wd.folder() + " has no notes.delta or no rivers (not a delta-test world?)");
            return null;
        }
        return new World(level, wd, note);
    }

    /** The water in column (x, z) from {@code fromY} down: its top block and how many blocks it holds (0: none). */
    private static int[] water(ServerLevel level, int x, int z, int fromY, int toY) {
        LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int y = fromY;
        while (y >= toY && !chunk.getBlockState(pos.set(x, y, z)).getFluidState().is(FluidTags.WATER))
            y--;
        if (y < toY)
            return new int[] {Integer.MIN_VALUE, 0};
        int top = y;
        while (y >= toY && chunk.getBlockState(pos.set(x, y, z)).getFluidState().is(FluidTags.WATER))
            y--;
        return new int[] {top, top - y};
    }

    /** The highest block in column (x, z) that is not air, and whether it is water. */
    private static int surface(ServerLevel level, int x, int z, int fromY, boolean[] wet) {
        LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int y = fromY;
        BlockState s;
        while ((s = chunk.getBlockState(pos.set(x, y, z))).isAir() && y > level.getMinBuildHeight())
            y--;
        wet[0] = s.getFluidState().is(FluidTags.WATER);
        return y;
    }

    /** The distance from (x, z) to the nearest segment of any chain, less that segment's reach (its half-width and bank zone). */
    private static double clearance(Rivers r, double x, double z) {
        double best = Double.POSITIVE_INFINITY;
        for (int k = 0; k < r.chainCount(); k++) {
            int first = r.chainFirst(k), n = r.chainLength(k);
            for (int i = first; i + 1 < first + n; i++) {
                double ax = r.vertexX(i), az = r.vertexZ(i), bx = r.vertexX(i + 1), bz = r.vertexZ(i + 1);
                double dx = bx - ax, dz = bz - az, len2 = dx * dx + dz * dz;
                double t = len2 > 0 ? Math.max(0, Math.min(1, ((x - ax) * dx + (z - az) * dz) / len2)) : 0;
                double d = Math.hypot(x - ax - t * dx, z - az - t * dz);
                double hw = Math.max(r.vertexHalfWidth(i), r.vertexHalfWidth(i + 1));
                best = Math.min(best, d - 1.5 * hw - 3.0);
            }
        }
        return best;
    }

    /**
     * The river file holds the split: chains 1 and 2 leave chain 0 ({@code from} 0) at its apex
     * vertex, and chain 0 leaves nothing.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void distributariesLeaveTheRiverAtTheApex(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        helper.assertTrue(r.chainCount() == 3, "chains: " + r.chainCount());
        helper.assertTrue(r.chainFrom(0) == Rivers.NO_CHAIN, "the river leaves chain " + r.chainFrom(0));
        int apex = wld.note().getAsJsonObject("apex").get("vertex_of_chain_0").getAsInt();
        for (int k = 1; k <= 2; k++) {
            helper.assertTrue(r.chainFrom(k) == 0, "chain " + k + " leaves chain " + r.chainFrom(k));
            int v = r.chainFirst(k), p = r.chainFirst(0) + apex;
            helper.assertTrue(r.vertexX(v) == r.vertexX(p) && r.vertexZ(v) == r.vertexZ(p),
                    "chain " + k + " starts at (" + r.vertexX(v) + ", " + r.vertexZ(v) + "), the apex is (" + r.vertexX(p) + ", " + r.vertexZ(p) + ")");
        }
        helper.succeed();
    }

    /**
     * Along the river below the apex and both distributaries across the plain: the centre column's
     * water stands at the sea's top water block and is within a block of the channel's depth
     * (HYD-120, HYD-136). At least {@value #WITHIN_SHARE} of the columns hold it.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void distributariesAreCutAtTheSeaLevel(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        JsonObject delta = wld.note().getAsJsonObject("delta");
        double seaW = wld.note().get("sea_w").getAsDouble();
        int seaTop = (int) Math.ceil(seaW) - 1;
        double depth = wld.note().getAsJsonObject("branch").get("depth").getAsDouble();
        int toX = (int) Math.round(delta.get("to_x").getAsDouble()) - 10;
        int columns = 0, ok = 0, dry = 0;
        StringBuilder bad = new StringBuilder();
        for (int k = 0; k < r.chainCount(); k++) {
            int first = r.chainFirst(k), n = r.chainLength(k);
            for (int i = first; i + 1 < first + n; i++) {
                double ax = r.vertexX(i), az = r.vertexZ(i), bx = r.vertexX(i + 1), bz = r.vertexZ(i + 1);
                // Below the apex (x > 8, away from the split's own overlap) and on the plain.
                if (Math.min(ax, bx) < 8 || Math.max(ax, bx) > toX)
                    continue;
                int steps = (int) Math.ceil(Math.hypot(bx - ax, bz - az));
                for (int s = 0; s < steps; s++) {
                    double t = (double) s / steps;
                    int x = (int) Math.floor(ax + t * (bx - ax)), z = (int) Math.floor(az + t * (bz - az));
                    int[] got = water(wld.level(), x, z, seaTop + 4, seaTop - (int) Math.ceil(depth) - 6);
                    columns++;
                    if (got[1] == 0)
                        dry++;
                    boolean good = got[0] == seaTop && Math.abs(got[1] - depth) <= 1.0 + 1e-9;
                    if (good)
                        ok++;
                    else if (bad.length() < 600)
                        bad.append(String.format(" chain %d (%d,%d) top %d holds %d;", k, x, z, got[0], got[1]));
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] distributariesAreCutAtTheSeaLevel columns={} ok={} dry={} seaTop={} depth={}{}",
                columns, ok, dry, seaTop, depth, bad.length() == 0 ? "" : " outside:" + bad);
        helper.assertTrue(columns >= 300, "only " + columns + " channel columns checked");
        helper.assertTrue(ok >= WITHIN_SHARE * columns, ok + " of " + columns + " channel columns hold water at y " + seaTop
                + " within a block of depth " + depth + ":" + bad);
        helper.succeed();
    }

    /**
     * The plain between and beside the channels is dry land whose top block is the plain's ground
     * block, a block over the sea's top water block (HYD-136): no water on it and nothing raised.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void deltaPlainIsDryLandAboveTheSea(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        JsonObject delta = wld.note().getAsJsonObject("delta");
        int ground = (int) Math.ceil(delta.get("y").getAsDouble()) - 1;
        int seaTop = (int) Math.ceil(wld.note().get("sea_w").getAsDouble()) - 1;
        int fromX = (int) Math.round(delta.get("from_x").getAsDouble()) + 20;
        int toX = (int) Math.round(delta.get("to_x").getAsDouble()) - 15;
        int columns = 0, wetCount = 0, off = 0;
        StringBuilder bad = new StringBuilder();
        boolean[] wet = new boolean[1];
        for (int x = fromX; x <= toX; x += 2) {
            for (int z = -120; z <= 120; z += 2) {
                if (clearance(r, x + 0.5, z + 0.5) <= 0)
                    continue;
                columns++;
                int y = surface(wld.level(), x, z, ground + 8, wet);
                if (wet[0]) {
                    wetCount++;
                    if (bad.length() < 600)
                        bad.append(String.format(" wet(%d,%d) y %d;", x, z, y));
                } else if (y != ground) {
                    off++;
                    if (bad.length() < 600)
                        bad.append(String.format(" (%d,%d) y %d;", x, z, y));
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] deltaPlainIsDryLandAboveTheSea columns={} wet={} offGround={} ground={} seaTop={}{}",
                columns, wetCount, off, ground, seaTop, bad.length() == 0 ? "" : " detail:" + bad);
        helper.assertTrue(ground == seaTop + 1, "the fixture's plain ground " + ground + " is not a block over the sea's water " + seaTop);
        helper.assertTrue(columns >= 1000, "only " + columns + " plain columns checked");
        helper.assertTrue(wetCount == 0, wetCount + " of " + columns + " plain columns hold water:" + bad);
        helper.assertTrue(off == 0, off + " of " + columns + " plain columns are not at the plain's ground block " + ground + ":" + bad);
        helper.succeed();
    }

    /** The highest ground block of column (x, z) under {@code fromY} (plants, logs and leaves skipped), or null where water tops it. */
    private static BlockState ground(ServerLevel level, int x, int z, int fromY) {
        LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (int y = fromY; y > level.getMinBuildHeight(); y--) {
            BlockState s = chunk.getBlockState(pos.set(x, y, z));
            if (!s.getFluidState().isEmpty())
                return null;
            if (s.isSolid() && !s.is(BlockTags.LOGS) && !s.is(BlockTags.LEAVES))
                return s;
        }
        return null;
    }

    /**
     * The levee (HYD-153): the river's first dry columns beside its water on the floodplain written over
     * the upstream reach ({@code notes.delta.floodplain}) are sand; on the slope below it, off the plain,
     * the banks keep their soil. Counts each side's first two dry columns within the bank zone.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void leveesAreSandOnTheFloodplainOnly(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        JsonObject plain = wld.note().getAsJsonObject("floodplain");
        if (plain == null || !wld.wd().has(WorldData.FLOODPLAIN)) {
            helper.fail("the delta-test world has no floodplain layer or no notes.delta.floodplain");
            return;
        }
        double hw = wld.note().getAsJsonObject("river").get("half_width").getAsDouble();
        int fromY = (int) Math.ceil(wld.note().get("plain_y").getAsDouble()) + 12;
        int reach = (int) Math.ceil(hw + 1.0 + 0.5 * hw) + 2;   // the bank zone (Rivers) and a margin
        int[] onPlain = new int[3], offPlain = new int[3];   // columns, sand tops, soil tops
        StringBuilder bad = new StringBuilder();
        int plainFrom = (int) Math.round(plain.get("from_x").getAsDouble()) + 15;
        int plainTo = (int) Math.round(plain.get("to_x").getAsDouble()) - 15;
        int slopeFrom = (int) Math.round(wld.note().getAsJsonObject("delta").get("from_x").getAsDouble()) - 185;
        int slopeTo = (int) Math.round(wld.note().getAsJsonObject("delta").get("from_x").getAsDouble()) - 45;
        for (int pass = 0; pass < 2; pass++) {
            int[] tally = pass == 0 ? onPlain : offPlain;
            int x0 = pass == 0 ? plainFrom : slopeFrom, x1 = pass == 0 ? plainTo : slopeTo;
            for (int x = x0; x <= x1; x++) {
                for (int side : new int[] {1, -1}) {
                    int found = 0;
                    for (int d = 1; d <= reach && found < 2; d++) {
                        int z = side * d;
                        BlockState top = ground(wld.level(), x, z, fromY);
                        if (top == null)
                            continue;   // the channel's water
                        found++;
                        tally[0]++;
                        if (top.is(Blocks.SAND))
                            tally[1]++;
                        else if (WaterBodies.levees(top))
                            tally[2]++;
                        if (pass == 0 && !top.is(Blocks.SAND) && bad.length() < 600)
                            bad.append(String.format(" (%d,%d) %s;", x, z, top.getBlock().getDescriptionId()));
                    }
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] leveesAreSandOnTheFloodplainOnly plain columns={} sand={} soil={}; slope columns={} sand={} soil={}{}",
                onPlain[0], onPlain[1], onPlain[2], offPlain[0], offPlain[1], offPlain[2], bad.length() == 0 ? "" : " not sand on the plain:" + bad);
        helper.assertTrue(onPlain[0] >= 300, "only " + onPlain[0] + " bank columns checked on the plain");
        helper.assertTrue(offPlain[0] >= 300, "only " + offPlain[0] + " bank columns checked on the slope");
        helper.assertTrue(onPlain[2] == 0, onPlain[2] + " of " + onPlain[0] + " bank columns on the floodplain keep a soil top:" + bad);
        helper.assertTrue(onPlain[1] >= WITHIN_SHARE * onPlain[0], onPlain[1] + " of " + onPlain[0] + " bank columns on the floodplain are sand:" + bad);
        helper.assertTrue(offPlain[2] >= 0.5 * offPlain[0], "only " + offPlain[2] + " of " + offPlain[0]
                + " bank columns off the floodplain keep a soil top (" + offPlain[1] + " sand)");
        helper.succeed();
    }
}
