// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import java.util.ArrayList;
import java.util.List;
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
 * Alluvial fans and braids on the fan-test world ({@code deeptime:test_fan},
 * {@code deeptime synth --kind fan-test}; V1 gap 5, HYD-165 to HYD-175).
 *
 * <p>The bake raises the ground to a cone where a river leaves steep ground for gentler, and divides
 * a river on a fan around alluvial islands 5 channel widths long on alternate sides of its course.
 * The world here is a range with a fan at its foot, a cone round on the planet, and a river down it
 * run through the bake's braid pass: the file holds the course and a thread around each island, and
 * the game carves them as it carves every chain. The fan's ground is the cone, each thread holds the
 * river's water a block or more under it, and each island between the threads is dry ground.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class FanGameTests {

    /** The share of checked thread columns whose water must be right, and of cone columns exactly at the cone's block. */
    static final double WITHIN_SHARE = 0.9, EXACT_SHARE = 0.8;
    /** How far off the course a thread's segment must run for its columns to be checked, blocks. */
    static final double THREAD_OFF = 8.0;

    private FanGameTests() {}

    private record World(ServerLevel level, WorldData wd, JsonObject note) {}

    private static World world(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_FAN);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_FAN.location() + " does not exist");
            return null;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_fan is not generating from world data");
            return null;
        }
        WorldData wd = t.data();
        JsonObject note = wd.manifest().notes().getAsJsonObject("fan");
        if (note == null || wd.rivers() == null) {
            helper.fail("world data " + wd.folder() + " has no notes.fan or no rivers (not a fan-test world?)");
            return null;
        }
        return new World(level, wd, note);
    }

    /** The chains that leave chain 0 (the threads around its islands). */
    private static List<Integer> threads(Rivers r) {
        List<Integer> out = new ArrayList<>();
        for (int k = 1; k < r.chainCount(); k++)
            if (r.chainFrom(k) == 0)
                out.add(k);
        return out;
    }

    /** The vertex of chain k furthest off the course's line z 0. */
    private static int farthest(Rivers r, int k) {
        int first = r.chainFirst(k), best = first;
        for (int i = first; i < first + r.chainLength(k); i++)
            if (Math.abs(r.vertexZ(i)) > Math.abs(r.vertexZ(best)))
                best = i;
        return best;
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
     * The river file holds the braided river: the course from the apex to the sea as drawn, and at
     * least two threads that leave it at one of its vertices and rejoin it at another (mouth river,
     * into the course), all on the fan, standing 10 to 25 blocks off the course at their islands'
     * middles, on alternate sides (HYD-171 to HYD-174).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void riverBraidsOnTheFan(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        JsonObject river = wld.note().getAsJsonObject("river");
        JsonObject fan = wld.note().getAsJsonObject("fan");
        double apexX = fan.get("apex_x").getAsDouble(), toeX = fan.get("toe_x").getAsDouble();
        int straight = wld.note().get("straight_vertices").getAsInt();
        int first = r.chainFirst(0), n = r.chainLength(0);
        helper.assertTrue(r.chainFrom(0) == Rivers.NO_CHAIN && r.chainMouth(0) == Rivers.MOUTH_SEA, "chain 0 is not the course to the sea");
        helper.assertTrue(n == straight, n + " course vertices, the straight river has " + straight);
        helper.assertTrue(r.vertexX(first) == river.get("from_x").getAsFloat() && r.vertexX(first + n - 1) == river.get("to_x").getAsFloat(),
                "the course runs from x " + r.vertexX(first) + " to " + r.vertexX(first + n - 1));
        List<Integer> th = threads(r);
        StringBuilder seen = new StringBuilder();
        double lastSide = 0;
        for (int k : th) {
            int f = r.chainFirst(k), m = r.chainLength(k);
            helper.assertTrue(r.chainMouth(k) == Rivers.MOUTH_RIVER && r.chainInto(k) == 0, "thread " + k + " does not rejoin the course");
            for (int end : new int[] {f, f + m - 1}) {
                boolean on = false;
                for (int i = first; i < first + n && !on; i++)
                    on = r.vertexX(i) == r.vertexX(end) && r.vertexZ(i) == r.vertexZ(end);
                helper.assertTrue(on, "thread " + k + " leaves or rejoins off the course's vertices at (" + r.vertexX(end) + ", " + r.vertexZ(end) + ")");
            }
            for (int i = f; i < f + m; i++)
                helper.assertTrue(r.vertexX(i) >= apexX && r.vertexX(i) <= toeX, "thread " + k + " leaves the fan at x " + r.vertexX(i));
            double off = r.vertexZ(farthest(r, k));
            seen.append(String.format(" %d: x %.0f..%.0f off %.1f;", k, r.vertexX(f), r.vertexX(f + m - 1), off));
            helper.assertTrue(Math.abs(off) >= 10 && Math.abs(off) <= 25, "thread " + k + " stands " + off + " blocks off the course");
            helper.assertTrue(Math.signum(off) != lastSide, "threads on the same side:" + seen);
            lastSide = Math.signum(off);
        }
        DeepTime.LOGGER.info("[gametest] riverBraidsOnTheFan threads={}{} stats={}", th.size(), seen, wld.note().get("stats"));
        helper.assertTrue(th.size() >= 2, "only " + th.size() + " threads");
        helper.succeed();
    }

    /**
     * Where a thread runs at least {@value #THREAD_OFF} blocks off the course, its centre column holds
     * the thread's own water with its top block at the thread's water surface there (at least
     * {@value #WITHIN_SHARE} of the columns), and the island's middle, halfway between the course and
     * the thread where the thread stands furthest off, is dry ground above that water.
     *
     * <p>The surface at a column is the column rule's (FORMAT.md, "the hit is the covering segment
     * with the lowest w"), not the thread's surface at the point walked: on a fan, falling 0.1 a block,
     * the round end of the next segment down covers a centre column with water a few tenths of a
     * block lower (0.31 at (-184, 13): w 82.84 against the walked point's 83.15), so a column whose
     * surface lies just over a whole block holds its top block one lower than the walked point's
     * (the first run of this test, on master 1e27a44f, asked for that one and failed 119 of 144
     * columns, the one it named drawn as the rule says).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void threadsHoldWaterAroundDryIslands(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        double depth = wld.note().getAsJsonObject("river").get("depth").getAsDouble();
        int columns = 0, ok = 0, islands = 0, dryIslands = 0;
        StringBuilder bad = new StringBuilder();
        boolean[] wet = new boolean[1];
        for (int k : threads(r)) {
            int f = r.chainFirst(k), m = r.chainLength(k);
            for (int i = f; i + 1 < f + m; i++) {
                double ax = r.vertexX(i), az = r.vertexZ(i), bx = r.vertexX(i + 1), bz = r.vertexZ(i + 1);
                if (Math.min(Math.abs(az), Math.abs(bz)) < THREAD_OFF)
                    continue;
                int steps = (int) Math.ceil(Math.hypot(bx - ax, bz - az));
                for (int s = 0; s < steps; s++) {
                    double t = (double) s / steps;
                    int x = (int) Math.floor(ax + t * (bx - ax)), z = (int) Math.floor(az + t * (bz - az));
                    // The column rule's surface there: its lowest covering segment's, which must be the thread's own.
                    Rivers.Hit hit = r.sample(x, z).hit();
                    double w = hit == null ? Double.NaN : hit.w();
                    boolean own = hit != null && hit.seg() >= f && hit.seg() < f + m - 1;
                    int top = (int) Math.ceil(r.vertexW(i) + t * (r.vertexW(i + 1) - r.vertexW(i))) - 1;
                    if (own)
                        top = (int) Math.ceil(w) - 1;
                    int[] got = water(wld.level(), x, z, top + 6, top - (int) Math.ceil(depth) - 6);
                    columns++;
                    if (own && got[0] == top && got[1] >= 1)
                        ok++;
                    else if (bad.length() < 600)
                        bad.append(String.format(" (%d,%d) top %d (w %.2f%s) holds %d;", x, z, got[0], w,
                                own ? "" : hit == null ? ", no channel" : ", segment " + hit.seg() + " not the thread's", got[1]));
                }
            }
            int far = farthest(r, k);
            int x = (int) Math.floor(r.vertexX(far)), z = (int) Math.floor(0.5 * r.vertexZ(far));
            int top = (int) Math.ceil(r.vertexW(far)) - 1;
            int y = surface(wld.level(), x, z, top + 30, wet);
            islands++;
            if (!wet[0] && y > top)
                dryIslands++;
            else if (bad.length() < 900)
                bad.append(String.format(" island (%d,%d) y %d%s, water top %d;", x, z, y, wet[0] ? " wet" : "", top));
        }
        DeepTime.LOGGER.info("[gametest] threadsHoldWaterAroundDryIslands columns={} ok={} islands={} dry={}{}",
                columns, ok, islands, dryIslands, bad.length() == 0 ? "" : " outside:" + bad);
        helper.assertTrue(islands >= 2, "only " + islands + " islands");
        helper.assertTrue(columns >= 60, "only " + columns + " thread columns checked");
        helper.assertTrue(ok >= WITHIN_SHARE * columns, ok + " of " + columns + " thread columns hold water at the thread's surface:" + bad);
        helper.assertTrue(dryIslands == islands, dryIslands + " of " + islands + " islands are dry ground over the water:" + bad);
        helper.succeed();
    }

    /**
     * The fan's ground is the cone: on the columns {@code notes.fan.cone_samples} lists (every 10
     * blocks on the fan) clear of every channel and its bank zone, the top block is dry and the
     * cone's block (its height rounded up, less one) within one block everywhere and exactly on at
     * least {@value #EXACT_SHARE} of them. The cone falls with the planet's distance from the apex,
     * so it is round on the planet.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void fanGroundIsTheCone(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        JsonArray samples = wld.note().getAsJsonArray("cone_samples");
        int columns = 0, exact = 0, near = 0, wetCount = 0;
        StringBuilder bad = new StringBuilder();
        boolean[] wet = new boolean[1];
        for (JsonElement e : samples) {
            JsonArray a = e.getAsJsonArray();
            int x = (int) a.get(0).getAsDouble(), z = (int) a.get(1).getAsDouble();
            double h = a.get(2).getAsDouble();
            if (clearance(r, x + 0.5, z + 0.5) <= 0)
                continue;
            int want = (int) Math.ceil(h) - 1;
            int y = surface(wld.level(), x, z, want + 12, wet);
            columns++;
            if (wet[0])
                wetCount++;
            if (y == want)
                exact++;
            if (Math.abs(y - want) <= 1)
                near++;
            if ((wet[0] || y != want) && bad.length() < 600)
                bad.append(String.format(" (%d,%d) y %d%s, cone %.2f;", x, z, y, wet[0] ? " wet" : "", h));
        }
        DeepTime.LOGGER.info("[gametest] fanGroundIsTheCone columns={} exact={} withinOne={} wet={}{}",
                columns, exact, near, wetCount, bad.length() == 0 ? "" : " detail:" + bad);
        helper.assertTrue(columns >= 150, "only " + columns + " cone columns checked");
        helper.assertTrue(wetCount == 0, wetCount + " of " + columns + " cone columns hold water:" + bad);
        helper.assertTrue(near == columns, (columns - near) + " of " + columns + " cone columns are more than a block off the cone:" + bad);
        helper.assertTrue(exact >= EXACT_SHARE * columns, exact + " of " + columns + " cone columns at the cone's block:" + bad);
        helper.succeed();
    }
}
