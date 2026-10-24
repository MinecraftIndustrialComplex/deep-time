// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * Meanders on the meander-test world ({@code deeptime:test_meander},
 * {@code deeptime synth --kind meander-test}; V1 gap 5, HYD-155 to HYD-159).
 *
 * <p>The bake redraws a river's reaches on a floodplain as a sine-generated curve about the old
 * course, at 10.9 channel widths a wavelength and as wide as the plain's room allows up to a
 * sinuosity of 2. The world here is a straight river on a plain run through that same pass: the file
 * holds the meandered chain, and the game carves it as it carves every chain. Where the new channel
 * swings away from the old course it holds the river's water a block under the plain's top block,
 * and the old course between the bends is dry plain like the rest.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class MeanderGameTests {

    /** The share of checked channel columns whose water must be right. */
    static final double WITHIN_SHARE = 0.95;
    /** The x range the column checks cover (the plain's middle, away from both ends' tapers). */
    static final int FROM_X = -250, TO_X = 250;

    private MeanderGameTests() {}

    private record World(ServerLevel level, WorldData wd, JsonObject note) {}

    private static World world(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_MEANDER);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_MEANDER.location() + " does not exist");
            return null;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_meander is not generating from world data");
            return null;
        }
        WorldData wd = t.data();
        JsonObject note = wd.manifest().notes().getAsJsonObject("meander");
        if (note == null || wd.rivers() == null) {
            helper.fail("world data " + wd.folder() + " has no notes.meander or no rivers (not a meander-test world?)");
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
     * The river file holds the meandered river: one chain with more vertices than the straight one,
     * the same ends, swinging at least 15 blocks off its old course and no further than the plain
     * less its half-width, at a sinuosity of at least 1.5 over the plain's middle (HYD-156, HYD-157).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void riverMeandersOnThePlain(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        JsonObject river = wld.note().getAsJsonObject("river");
        double half = wld.note().getAsJsonObject("floodplain").get("half_width").getAsDouble();
        double hw = river.get("half_width").getAsDouble();
        int straight = wld.note().get("straight_vertices").getAsInt();
        helper.assertTrue(r.chainCount() == 1, "chains: " + r.chainCount());
        int first = r.chainFirst(0), n = r.chainLength(0);
        helper.assertTrue(n > straight, n + " vertices, the straight river has " + straight);
        helper.assertTrue(r.vertexX(first) == river.get("from_x").getAsFloat() && r.vertexZ(first) == 0f,
                "the river starts at (" + r.vertexX(first) + ", " + r.vertexZ(first) + ")");
        helper.assertTrue(r.vertexX(first + n - 1) == river.get("to_x").getAsFloat() && r.vertexZ(first + n - 1) == 0f,
                "the river ends at (" + r.vertexX(first + n - 1) + ", " + r.vertexZ(first + n - 1) + ")");
        double off = 0, path = 0;
        for (int i = first; i < first + n; i++) {
            off = Math.max(off, Math.abs(r.vertexZ(i)));
            if (i + 1 < first + n && r.vertexX(i) >= FROM_X && r.vertexX(i + 1) <= TO_X)
                path += Math.hypot(r.vertexX(i + 1) - r.vertexX(i), r.vertexZ(i + 1) - r.vertexZ(i));
        }
        double k = path / (TO_X - FROM_X);
        DeepTime.LOGGER.info("[gametest] riverMeandersOnThePlain vertices={} straight={} maxOffset={} sinuosity={} stats={}",
                n, straight, off, k, wld.note().get("stats"));
        helper.assertTrue(off >= 15 && off <= half - hw, "the river swings " + off + " blocks off its course (the plain's half-width " + half + ")");
        helper.assertTrue(k >= 1.5, "sinuosity " + k + " over x " + FROM_X + " to " + TO_X);
        helper.succeed();
    }

    /**
     * Where the new channel runs at least 10 blocks off the old course, its centre column's water
     * stands a block under the plain's top block (the river's top water block, the plain laid a block
     * over it, HYD-151) and is within a block of the channel's depth. At least {@value #WITHIN_SHARE}
     * of the columns hold it.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void meanderChannelHoldsTheRiversWater(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        double depth = wld.note().getAsJsonObject("river").get("depth").getAsDouble();
        int plainTop = (int) Math.ceil(wld.note().get("plain_y").getAsDouble()) - 1;
        int columns = 0, ok = 0, dry = 0;
        StringBuilder bad = new StringBuilder();
        int first = r.chainFirst(0), n = r.chainLength(0);
        for (int i = first; i + 1 < first + n; i++) {
            double ax = r.vertexX(i), az = r.vertexZ(i), bx = r.vertexX(i + 1), bz = r.vertexZ(i + 1);
            if (Math.min(ax, bx) < FROM_X || Math.max(ax, bx) > TO_X || Math.min(Math.abs(az), Math.abs(bz)) < 10)
                continue;
            int steps = (int) Math.ceil(Math.hypot(bx - ax, bz - az));
            for (int s = 0; s < steps; s++) {
                double t = (double) s / steps;
                int x = (int) Math.floor(ax + t * (bx - ax)), z = (int) Math.floor(az + t * (bz - az));
                double w = r.vertexW(i) + t * (r.vertexW(i + 1) - r.vertexW(i));
                int top = (int) Math.ceil(w) - 1;
                int[] got = water(wld.level(), x, z, plainTop + 4, top - (int) Math.ceil(depth) - 6);
                columns++;
                if (got[1] == 0)
                    dry++;
                boolean good = got[0] == top && top == plainTop - 1 && Math.abs(got[1] - depth) <= 1.0 + 1e-9;
                if (good)
                    ok++;
                else if (bad.length() < 600)
                    bad.append(String.format(" (%d,%d) top %d (w %.2f) holds %d;", x, z, got[0], w, got[1]));
            }
        }
        DeepTime.LOGGER.info("[gametest] meanderChannelHoldsTheRiversWater columns={} ok={} dry={} plainTop={} depth={}{}",
                columns, ok, dry, plainTop, depth, bad.length() == 0 ? "" : " outside:" + bad);
        helper.assertTrue(columns >= 300, "only " + columns + " channel columns checked");
        helper.assertTrue(ok >= WITHIN_SHARE * columns, ok + " of " + columns + " channel columns hold water a block under the plain's top "
                + plainTop + " within a block of depth " + depth + ":" + bad);
        helper.succeed();
    }

    /**
     * The plain across the meander belt, the old course included, is dry land at the plain's ground
     * block wherever it is clear of the new channel and its bank zone: the old course is not left as
     * a trench or a pond. Checks every other column of the belt; at least 80 of them on the old course.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void oldCourseIsDryPlain(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        int ground = (int) Math.ceil(wld.note().get("plain_y").getAsDouble()) - 1;
        int belt = (int) Math.floor(wld.note().getAsJsonObject("floodplain").get("half_width").getAsDouble()) - 10;
        int columns = 0, course = 0, wetCount = 0, off = 0;
        StringBuilder bad = new StringBuilder();
        boolean[] wet = new boolean[1];
        for (int x = FROM_X; x <= TO_X; x += 2) {
            for (int z = -belt; z <= belt; z += 2) {
                if (clearance(r, x + 0.5, z + 0.5) <= 0)
                    continue;
                columns++;
                if (z == 0)
                    course++;
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
        DeepTime.LOGGER.info("[gametest] oldCourseIsDryPlain columns={} onOldCourse={} wet={} offGround={} ground={}{}",
                columns, course, wetCount, off, ground, bad.length() == 0 ? "" : " detail:" + bad);
        helper.assertTrue(course >= 80, "only " + course + " columns of the old course clear of the new channel");
        helper.assertTrue(columns >= 2000, "only " + columns + " plain columns checked");
        helper.assertTrue(wetCount == 0, wetCount + " of " + columns + " plain columns hold water:" + bad);
        helper.assertTrue(off == 0, off + " of " + columns + " plain columns are not at the plain's ground block " + ground + ":" + bad);
        helper.succeed();
    }
}
