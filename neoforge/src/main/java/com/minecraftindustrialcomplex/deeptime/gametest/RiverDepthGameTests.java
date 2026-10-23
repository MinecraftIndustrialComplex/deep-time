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
 * Rivers cut to the bake's depth and incised, not curbed, on the river-test world
 * ({@code deeptime:test_river_depth}, {@code deeptime synth --kind river-test}; HYD-120 to HYD-124).
 *
 * <p>Ben's single-player planet (2026-10-22) had its largest river a block deep where the bake put
 * 8 to 10 blocks, and a one-block lip along both banks of every stream. The world here has the same
 * shapes: a river of half-width 10 and centre depth 8 whose segment capsules overlap every column
 * several times, run down a slope and across sea-level flats and a block of sea into the sea, and a
 * stream whose surface lies in the plain's ground block.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class RiverDepthGameTests {

    /** Offsets from the river's centre line checked at every x of its lower reach, blocks. */
    static final int[] ACROSS = {-6, -3, 0, 3, 6};
    /**
     * The share of checked columns whose water must be within a block of the bake's depth (a column
     * where the surface steps down to the next block holds one block more, the step's flowing water).
     */
    static final double WITHIN_SHARE = 0.95;
    /** How far from the stream's centre line its banks and the plain beyond are read, blocks. */
    static final int STREAM_SIDE = 6;

    private RiverDepthGameTests() {}

    /** The level, its world data and its {@code notes.river}, or null after failing the test. */
    private record World(ServerLevel level, WorldData wd, JsonObject note) {}

    private static World world(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_RIVER_DEPTH);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_RIVER_DEPTH.location() + " does not exist");
            return null;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_river_depth is not generating from world data");
            return null;
        }
        WorldData wd = t.data();
        JsonObject note = wd.manifest().notes().getAsJsonObject("river");
        if (note == null || wd.rivers() == null) {
            helper.fail("world data " + wd.folder() + " has no notes.river or no rivers (not a river-test world?)");
            return null;
        }
        return new World(level, wd, note);
    }

    /** The surface of chain {@code k} at block x, from its vertices (the river and the stream are straight). */
    private static double chainW(Rivers r, int k, double along, boolean byX) {
        int first = r.chainFirst(k), n = r.chainLength(k);
        for (int i = first; i + 1 < first + n; i++) {
            double a = byX ? r.vertexX(i) : r.vertexZ(i), b = byX ? r.vertexX(i + 1) : r.vertexZ(i + 1);
            if (along >= Math.min(a, b) && along <= Math.max(a, b))
                return a == b ? r.vertexW(i) : r.vertexW(i) + (r.vertexW(i + 1) - r.vertexW(i)) * (along - a) / (b - a);
        }
        return Double.NaN;
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

    /**
     * Along the river's lower reach (from the slope above the flats, across the flats at the sea's
     * level and the shallows of the sea, to the shore), at five offsets across it: the water the
     * generator put in each column against the bake's depth there, the parabola of the channel the
     * column lies deepest in, max(1, depth · (1 − (d / hw)²)) (HYD-022, HYD-120). At least
     * {@value #WITHIN_SHARE} of the columns, and of those where the river's surface is the sea's,
     * must hold water within a block of it.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void riverBedsReachTheBakeDepth(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        JsonObject river = wld.note().getAsJsonObject("river");
        JsonObject reach = river.getAsJsonObject("lower_reach");
        double hw = river.get("half_width").getAsDouble(), depth = river.get("depth").getAsDouble();
        double seaW = wld.note().get("sea_w").getAsDouble();
        int z0 = (int) Math.round(river.get("z").getAsDouble());
        int chain = river.get("chain").getAsInt();
        int x0 = (int) Math.round(reach.get("from_x").getAsDouble()), x1 = (int) Math.round(reach.get("to_x").getAsDouble());

        int columns = 0, within = 0, seaColumns = 0, seaWithin = 0, dry = 0;
        double gotSum = 0, wantSum = 0, seaGotSum = 0, seaWantSum = 0;
        int[] centreHist = new int[16];
        StringBuilder worst = new StringBuilder();
        int worstShown = 0;
        for (int x = x0; x <= x1; x++) {
            double w = chainW(r, chain, x, true);
            if (Double.isNaN(w)) {
                helper.fail("the river has no vertex pair around x " + x);
                return;
            }
            boolean sea = w <= seaW;
            int topBlock = (int) Math.ceil(w) - 1;
            for (int dz : ACROSS) {
                double rr = dz / hw;
                double want = Math.max(1.0, depth * (1.0 - rr * rr));
                int[] got = water(wld.level(), x, z0 + dz, topBlock + 3, topBlock - (int) Math.ceil(depth) - 12);
                if (got[1] == 0)
                    dry++;
                boolean ok = Math.abs(got[1] - want) <= 1.0 + 1e-9;
                columns++;
                gotSum += got[1];
                wantSum += want;
                if (ok)
                    within++;
                if (sea) {
                    seaColumns++;
                    seaGotSum += got[1];
                    seaWantSum += want;
                    if (ok)
                        seaWithin++;
                }
                if (dz == 0)
                    centreHist[Math.min(15, got[1])]++;
                if (!ok && worstShown < 12) {
                    worstShown++;
                    worst.append(String.format(" (%d,%d) w %.2f top %d holds %d want %.2f;", x, z0 + dz, w,
                            got[0], got[1], want));
                }
            }
        }
        StringBuilder hist = new StringBuilder();
        for (int i = 0; i < centreHist.length; i++)
            if (centreHist[i] > 0)
                hist.append(' ').append(i).append(':').append(centreHist[i]);
        DeepTime.LOGGER.info("[gametest] riverBedsReachTheBakeDepth columns={} within1={} ({}) seaLevelColumns={} seaLevelWithin1={} ({}) "
                        + "meanWater={} meanBake={} seaLevelMeanWater={} seaLevelMeanBake={} dry={} centreLineWaterBlocks={}{}",
                columns, within, share(within, columns), seaColumns, seaWithin, share(seaWithin, seaColumns),
                mean(gotSum, columns), mean(wantSum, columns), mean(seaGotSum, seaColumns), mean(seaWantSum, seaColumns), dry, hist,
                worst.length() == 0 ? "" : " outside:" + worst);
        helper.assertTrue(columns >= 1000, "only " + columns + " columns checked");
        helper.assertTrue(seaColumns >= 500, "only " + seaColumns + " columns where the river is at the sea's level");
        helper.assertTrue(within >= WITHIN_SHARE * columns, within + " of " + columns
                + " columns hold water within a block of the bake's depth (mean " + mean(gotSum, columns) + " against " + mean(wantSum, columns) + ")" + worst);
        helper.assertTrue(seaWithin >= WITHIN_SHARE * seaColumns, seaWithin + " of " + seaColumns
                + " columns at the sea's level hold water within a block of the bake's depth (mean " + mean(seaGotSum, seaColumns)
                + " against " + mean(seaWantSum, seaColumns) + ")");
        helper.succeed();
    }

    /**
     * Along the flush stream on the plain (away from its source and from the river it joins): the
     * stream's water lies under the plain's ground block, and no column of its banks or of the plain
     * beside it stands above the plain's ground (HYD-123, HYD-124).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void flushStreamHasNoCurb(GameTestHelper helper) {
        World wld = world(helper);
        if (wld == null)
            return;
        Rivers r = wld.wd().rivers();
        JsonObject stream = wld.note().getAsJsonObject("stream");
        int chain = stream.get("chain").getAsInt();
        int sx = (int) Math.round(stream.get("x").getAsDouble());
        int fromZ = (int) Math.round(stream.get("from_z").getAsDouble()) + 6;
        int toZ = (int) Math.round(stream.get("to_z").getAsDouble()) - 16;
        double hw = stream.get("half_width").getAsDouble();
        int plainGround = (int) Math.ceil(wld.note().get("plain_y").getAsDouble()) - 1;

        int streamColumns = 0, perched = 0, dry = 0, sideColumns = 0, raised = 0, maxRaise = 0;
        StringBuilder detail = new StringBuilder();
        boolean[] wet = new boolean[1];
        for (int z = fromZ; z <= toZ; z++) {
            double w = chainW(r, chain, z, false);
            int from = (int) Math.ceil(w) + 4;
            for (int dx = -STREAM_SIDE; dx <= STREAM_SIDE; dx++) {
                int x = sx + dx;
                if (Math.abs(dx) <= hw) {
                    streamColumns++;
                    int[] got = water(wld.level(), x, z, from, plainGround - 8);
                    if (got[1] == 0) {
                        dry++;
                        if (detail.length() < 600)
                            detail.append(String.format(" dry(%d,%d);", x, z));
                    } else if (got[0] >= plainGround) {
                        perched++;
                        if (detail.length() < 600)
                            detail.append(String.format(" perched(%d,%d) top %d;", x, z, got[0]));
                    }
                } else {
                    sideColumns++;
                    int y = surface(wld.level(), x, z, from, wet);
                    if (wet[0] || y > plainGround) {
                        raised++;
                        maxRaise = Math.max(maxRaise, y - plainGround);
                        if (detail.length() < 600)
                            detail.append(String.format(" %s(%d,%d) y %d;", wet[0] ? "wet" : "raised", x, z, y));
                    }
                }
            }
        }
        DeepTime.LOGGER.info("[gametest] flushStreamHasNoCurb streamColumns={} perched={} dry={} sideColumns={} raised={} ({}) maxRaise={} plainGround={}{}",
                streamColumns, perched, dry, sideColumns, raised, share(raised, sideColumns), maxRaise, plainGround,
                detail.length() == 0 ? "" : " detail:" + detail);
        helper.assertTrue(streamColumns > 0 && sideColumns > 0, "no stream columns checked");
        helper.assertTrue(dry == 0, dry + " of " + streamColumns + " stream columns hold no water:" + detail);
        helper.assertTrue(perched == 0, perched + " of " + streamColumns + " stream columns hold water at or over the plain's ground block " + plainGround + ":" + detail);
        helper.assertTrue(raised == 0, raised + " of " + sideColumns + " columns beside the stream stand over the plain's ground block " + plainGround + " (a curb):" + detail);
        helper.succeed();
    }

    private static String share(int n, int of) {
        return of == 0 ? "-" : String.format("%.3f", (double) n / of);
    }

    private static String mean(double sum, int n) {
        return n == 0 ? "-" : String.format("%.2f", sum / n);
    }
}
