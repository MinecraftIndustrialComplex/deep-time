// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.material.Fluids;
import net.minecraft.world.ticks.ScheduledTick;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Water above sea level holds (P0-13, plan §7.9). The {@code deeptime:test_lake} level reads the
 * synthetic lake-test world ({@code deeptime synth --kind lake-test --size 1024}): a plain at y 100,
 * a round basin (floor 110, rim crest 125) and a closed channel, with {@code water_level} 120 inside
 * the rim. Its biome, {@code deeptime:test_bare}, has no carvers and no features, so only the
 * generator's passes touch the basin.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class WaterGameTests {

    /** Ticks the forceloaded chunks run between the two counts. */
    static final int WAIT_TICKS = 400;
    /** Blocks scanned for escaped water around the footprint: past the rim's outer slope (d = 36). */
    static final int MARGIN = 48;
    /** Blocks scanned below the plain and above the rim crest. */
    static final int Y_MARGIN = 16;

    private WaterGameTests() {}

    /** The scanned box, from the manifest's {@code notes.lake} geometry. */
    record Box(int minX, int maxX, int minZ, int maxZ, int minY, int maxY) {
        int chunks() {
            return ((maxX >> 4) - (minX >> 4) + 1) * ((maxZ >> 4) - (minZ >> 4) + 1);
        }
    }

    /** One count over the box. */
    record Count(long expected, long sources, long outside, int columns, long analytic, int floorCornerMaxDiff,
                 int cornerColumns, List<BlockPos> water, String detail) {}

    /**
     * Counts the water sources in the basin after generation, wakes every one of them with a fluid
     * tick (as a changed neighbour would), lets the forceloaded chunks run {@value #WAIT_TICKS}
     * ticks and counts again. Expected: every open block (air or water) between a footprint
     * column's floor and its water surface is a source, before and after, and no water anywhere
     * else in the box. Generation itself must leave no fluid tick scheduled.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = WAIT_TICKS + 1200)
    public static void lakeHolds(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_LAKE);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_LAKE.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_lake is not generating from world data");
            return;
        }
        WorldData wd = t.data();
        JsonObject lake = wd.manifest().notes().getAsJsonObject("lake");
        if (lake == null) {
            helper.fail("world data " + wd.folder() + " has no notes.lake (not a lake-test world?)");
            return;
        }
        Box box = box(lake);

        // Forceloading generates each chunk synchronously (ServerLevel.setChunkForced → getChunk).
        List<LevelChunk> chunks = new ArrayList<>();
        for (int cx = box.minX() >> 4; cx <= box.maxX() >> 4; cx++)
            for (int cz = box.minZ() >> 4; cz <= box.maxZ() >> 4; cz++) {
                level.setChunkForced(cx, cz, true);
                chunks.add(level.getChunk(cx, cz));
            }
        int generatedTicks = pendingFluidTicks(chunks);
        Count before = count(level, t, box);

        // Wake the whole lake: a leak anywhere in the basin would now flow.
        long now = level.getGameTime();
        for (BlockPos p : before.water())
            level.getChunkAt(p).getFluidTicks().schedule(new ScheduledTick<>(Fluids.WATER, p, now + 1, level.nextSubTickCount()));

        helper.runAfterDelay(WAIT_TICKS, () -> {
            Count after = count(level, t, box);
            int pendingAfter = pendingFluidTicks(chunks);
            int[] centre = centreColumn(level, lake);
            for (LevelChunk c : chunks)
                level.setChunkForced(c.getPos().x, c.getPos().z, false);

            DeepTime.LOGGER.info("[gametest] lakeHolds before={} after={} expected={} outside={} outsideAtGeneration={} "
                            + "columns={} woken={} ticks={} generatedFluidTicks={} pendingFluidTicksAfter={} analytic={} "
                            + "floorCornerMaxDiff={} cornerColumns={} chunks={}",
                    before.sources(), after.sources(), before.expected(), after.outside(), before.outside(),
                    before.columns(), before.water().size(), WAIT_TICKS, generatedTicks, pendingAfter, before.analytic(),
                    before.floorCornerMaxDiff(), before.cornerColumns(), chunks.size());
            if (!before.detail().isEmpty() || !after.detail().isEmpty())
                DeepTime.LOGGER.info("[gametest] lakeHolds escaped water: atGeneration{} after{}", before.detail(), after.detail());
            double waterLevel = lake.get("water_level_y").getAsDouble();
            int wantTop = WaterPass.topWaterBlock(waterLevel) + 1;
            DeepTime.LOGGER.info("[gametest] lakeHolds water_top_y={} at=({}, {}) top_water_block_y={} floor_y={} water_level={}",
                    centre[2], centre[0], centre[1], centre[2] - 1, centre[3], waterLevel);

            helper.assertTrue(generatedTicks == 0, "generation left " + generatedTicks + " fluid ticks scheduled");
            helper.assertTrue(before.expected() > 0, "no open blocks below the water level: is the basin there?");
            helper.assertTrue(before.cornerColumns() > 0 && before.floorCornerMaxDiff() == 0,
                    "the basin floor differs from the elevation raster by " + before.floorCornerMaxDiff()
                            + " at noise-cell corners (" + before.cornerColumns() + " corner columns)");
            helper.assertTrue(before.sources() == before.expected(),
                    "after generation " + before.sources() + " sources of " + before.expected() + " expected");
            helper.assertTrue(before.outside() == 0, before.outside() + " water blocks outside the lake after generation");
            helper.assertTrue(pendingAfter == 0, pendingAfter + " fluid ticks still pending after " + WAIT_TICKS + " ticks");
            helper.assertTrue(after.sources() == before.expected(),
                    "after " + WAIT_TICKS + " ticks " + after.sources() + " sources of " + before.expected() + " expected");
            helper.assertTrue(after.outside() == 0, after.outside() + " water blocks outside the lake after " + WAIT_TICKS + " ticks");
            helper.assertTrue(centre[2] == wantTop, "water top at the lake centre is y " + centre[2] + ", want " + wantTop);
            helper.succeed();
        });
    }

    static Box box(JsonObject lake) {
        double cx = lake.getAsJsonArray("centre_block").get(0).getAsDouble();
        double cz = lake.getAsJsonArray("centre_block").get(1).getAsDouble();
        double r = lake.get("lake_floor_radius").getAsDouble();
        JsonObject ch = lake.getAsJsonObject("channel");
        double hw = ch.get("floor_half_width").getAsDouble();
        double fx = ch.getAsJsonArray("from_block").get(0).getAsDouble(), fz = ch.getAsJsonArray("from_block").get(1).getAsDouble();
        double tx = ch.getAsJsonArray("to_block").get(0).getAsDouble(), tz = ch.getAsJsonArray("to_block").get(1).getAsDouble();
        double minX = Math.min(cx - r, Math.min(fx, tx) - hw), maxX = Math.max(cx + r, Math.max(fx, tx) + hw);
        double minZ = Math.min(cz - r, Math.min(fz, tz) - hw), maxZ = Math.max(cz + r, Math.max(fz, tz) + hw);
        return new Box((int) Math.floor(minX) - MARGIN, (int) Math.ceil(maxX) + MARGIN,
                (int) Math.floor(minZ) - MARGIN, (int) Math.ceil(maxZ) + MARGIN,
                (int) Math.floor(lake.get("plain_y").getAsDouble()) - Y_MARGIN,
                (int) Math.ceil(lake.get("rim_crest_y").getAsDouble()) + Y_MARGIN);
    }

    /**
     * A footprint column is one whose {@code water_level} cell has a value (WaterPass's own rule).
     * In it, the floor is the first block below the water surface that is neither air nor water;
     * every block between the floor and the surface should be a water source. Anything else that
     * holds water in the box is outside.
     */
    static Count count(ServerLevel level, WorldDataTerrain t, Box box) {
        WorldData wd = t.data();
        long expected = 0, sources = 0, outside = 0, analytic = 0;
        int columns = 0, cornerMax = 0, corners = 0;
        List<BlockPos> water = new ArrayList<>();
        StringBuilder detail = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (int x = box.minX(); x <= box.maxX(); x++) {
            for (int z = box.minZ(); z <= box.maxZ(); z++) {
                LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
                double wl = wd.waterLevel(x, z);
                int top = Integer.MIN_VALUE, floor = Integer.MIN_VALUE;
                if (!Double.isNaN(wl)) {
                    top = WaterPass.topWaterBlock(wl);
                    floor = top;
                    while (floor > level.getMinBuildHeight() && open(chunk.getBlockState(pos.set(x, floor, z))))
                        floor--;
                    if (top > floor) {
                        columns++;
                        expected += top - floor;
                        int rasterFloor = (int) Math.ceil(t.height(x, z)) - 1;
                        analytic += Math.max(0, top - rasterFloor);
                        if ((x & 3) == 0 && (z & 3) == 0) {
                            corners++;
                            cornerMax = Math.max(cornerMax, Math.abs(floor - rasterFloor));
                        }
                        for (int y = floor + 1; y <= top; y++) {
                            BlockState s = chunk.getBlockState(pos.set(x, y, z));
                            if (s.is(Blocks.WATER) && s.getFluidState().isSource()) {
                                sources++;
                                water.add(pos.immutable());
                            }
                        }
                    }
                }
                for (int y = box.minY(); y <= box.maxY(); y++) {
                    if (y > floor && y <= top)
                        continue;
                    BlockState s = chunk.getBlockState(pos.set(x, y, z));
                    if (s.getFluidState().is(FluidTags.WATER)) {
                        outside++;
                        if (detail.length() < 600) {
                            int ground = y;
                            while (ground > level.getMinBuildHeight() && open(chunk.getBlockState(pos.set(x, ground, z))))
                                ground--;
                            detail.append(String.format(" (%d,%d,%d) %s wl=%s ground=%d H=%.2f;", x, y, z,
                                    s.getFluidState().isSource() ? "source" : "flowing" + s.getFluidState().getAmount(),
                                    wl, ground, t.height(x, z)));
                        }
                    }
                }
            }
        }
        return new Count(expected, sources, outside, columns, analytic, cornerMax, corners, water, detail.toString());
    }

    /** Air or a water block (source or flowing): what the lake should fill. */
    static boolean open(BlockState s) {
        return s.isAir() || s.is(Blocks.WATER);
    }

    static int pendingFluidTicks(List<LevelChunk> chunks) {
        int n = 0;
        for (LevelChunk c : chunks)
            n += c.getFluidTicks().count();
        return n;
    }

    /** {x, z, water top y (top face of the highest water block, or MIN_VALUE), floor y} at the lake centre. */
    static int[] centreColumn(ServerLevel level, JsonObject lake) {
        int x = (int) Math.round(lake.getAsJsonArray("centre_block").get(0).getAsDouble());
        int z = (int) Math.round(lake.getAsJsonArray("centre_block").get(1).getAsDouble());
        LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int y = level.getMaxBuildHeight() - 1;
        while (y > level.getMinBuildHeight() && chunk.getBlockState(pos.set(x, y, z)).isAir())
            y--;
        int waterTop = chunk.getBlockState(pos.set(x, y, z)).is(Blocks.WATER) ? y + 1 : Integer.MIN_VALUE;
        while (y > level.getMinBuildHeight() && open(chunk.getBlockState(pos.set(x, y, z))))
            y--;
        return new int[]{x, z, waterTop, y};
    }
}
