// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterLevels;
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
 * generator's passes touch the basin. West of it lie two closed bowls below sea level
 * ({@code notes.lake.closed_bowls}, the {@code closed_basin} layer; HYD-100), one dry and one with
 * a lake at y 50: {@link #closedBowlsHoldNoSea} checks that no sea stands in either.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class WaterGameTests {

    /** Ticks the forceloaded chunks run between the two counts. */
    static final int WAIT_TICKS = 400;
    /**
     * Blocks scanned for escaped water around the footprint. Escaping water must cross the rim, and
     * flowing water stays connected to its source, so any leak shows within 16 blocks. (It was 48,
     * past the rim's outer slope at d = 36; that forceloaded ~2× the chunks and ran the 160-mod MIC
     * GameTest server out of heap intermittently, P1-12.)
     */
    static final int MARGIN = 16;
    /** Blocks scanned below the plain and above the rim crest. */
    static final int Y_MARGIN = 16;

    private WaterGameTests() {}

    /** The scanned box, from the manifest's {@code notes.lake} geometry. */
    record Box(int minX, int maxX, int minZ, int maxZ, int minY, int maxY) {}

    /** One count over the box. */
    record Count(long expected, long sources, long outside, int columns, int shoreColumns, long analytic,
                 int floorCornerMaxDiff, int cornerColumns, List<BlockPos> water, String detail,
                 java.util.Map<Long, Integer> floors, int washable) {}

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
        int reach = g.waterReach();
        Count before = count(level, t, reach, box);

        // Wake the whole lake: a leak anywhere in the basin would now flow.
        long now = level.getGameTime();
        for (BlockPos p : before.water())
            level.getChunkAt(p).getFluidTicks().schedule(new ScheduledTick<>(Fluids.WATER, p, now + 1, level.nextSubTickCount()));

        helper.runAfterDelay(WAIT_TICKS, () -> {
            Count after = count(level, t, reach, box);
            int pendingAfter = pendingFluidTicks(chunks);
            int[] centre = centreColumn(level, lake);
            for (LevelChunk c : chunks)
                level.setChunkForced(c.getPos().x, c.getPos().z, false);

            DeepTime.LOGGER.info("[gametest] lakeHolds before={} after={} expected={} outside={} outsideAtGeneration={} "
                            + "columns={} shoreColumns={} reach={} woken={} ticks={} generatedFluidTicks={} pendingFluidTicksAfter={} "
                            + "analytic={} floorCornerMaxDiff={} cornerColumns={} chunks={} washable={}",
                    before.sources(), after.sources(), before.expected(), after.outside(), before.outside(),
                    before.columns(), before.shoreColumns(), reach, before.water().size(), WAIT_TICKS, generatedTicks,
                    pendingAfter, before.analytic(), before.floorCornerMaxDiff(), before.cornerColumns(), chunks.size(),
                    before.washable());
            // Columns whose floor moved while the lake was awake (what washed away, or filled).
            StringBuilder moved = new StringBuilder();
            for (var e : after.floors().entrySet()) {
                Integer f0 = before.floors().get(e.getKey());
                if (!e.getValue().equals(f0) && moved.length() < 800) {
                    BlockPos c = BlockPos.of(e.getKey());
                    moved.append(String.format(" (%d,%d) floor %s -> %d, block %s;", c.getX(), c.getZ(), f0, e.getValue(),
                            f0 == null ? "?" : level.getBlockState(new BlockPos(c.getX(), f0, c.getZ()))));
                }
            }
            if (moved.length() > 0)
                DeepTime.LOGGER.info("[gametest] lakeHolds floors moved:{}", moved);
            if (!before.detail().isEmpty() || !after.detail().isEmpty())
                DeepTime.LOGGER.info("[gametest] lakeHolds escaped water: atGeneration{} after{}", before.detail(), after.detail());
            double waterLevel = lake.get("water_level_y").getAsDouble();
            int wantTop = WaterLevels.topWaterBlock(waterLevel) + 1;
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
            helper.assertTrue(after.sources() >= before.expected() && after.sources() <= before.expected() + before.washable(),
                    "after " + WAIT_TICKS + " ticks " + after.sources() + " sources of " + before.expected() + " expected (+ up to "
                            + before.washable() + " washable plants at the waterline)");
            helper.assertTrue(after.outside() == 0, after.outside() + " water blocks outside the lake after " + WAIT_TICKS + " ticks");
            helper.assertTrue(centre[2] == wantTop, "water top at the lake centre is y " + centre[2] + ", want " + wantTop);
            helper.succeed();
        });
    }

    /**
     * No sea in a closed basin below sea level (HYD-100), on the lake-test world's two closed bowls, without the
     * aquifer ({@code test_lake}, {@code planet_nocaves}: the fluid picker alone) and with it ({@code test_lake_caves},
     * {@code planet}: noise caves and vanilla's aquifer). In every column of a bowl the first block under the open air
     * is not water in the dry bowl (with caves: not water from {@value #SEA_BAND_ABOVE_FLOOR} blocks over its floor up,
     * since a cave mouth may hold the aquifer's own water) and no higher than the lake's top block in the other; on
     * each bowl's floor no water
     * stands from {@value #SEA_BAND_ABOVE_FLOOR} blocks over the dry floor (over the lake's top) up to the sea's top
     * block, where the sea would stand (the aquifer's own water tables lie under the lowest ground around); the lake's
     * water top at its centre is its level's.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void closedBowlsHoldNoSea(GameTestHelper helper) {
        closedBowls(helper, GameTestSupport.TEST_LAKE);
    }

    /** {@link #closedBowlsHoldNoSea} with noise caves and vanilla's aquifer (the full router). */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void closedBowlsHoldNoSeaWithTheAquifer(GameTestHelper helper) {
        closedBowls(helper, GameTestSupport.TEST_LAKE_CAVES);
    }

    /** Blocks over a dry bowl's floor from which no water may stand (the sea's band, see {@link #closedBowlsHoldNoSea}). */
    static final int SEA_BAND_ABOVE_FLOOR = 10;

    static void closedBowls(GameTestHelper helper, net.minecraft.resources.ResourceKey<net.minecraft.world.level.Level> key) {
        ServerLevel level = helper.getLevel().getServer().getLevel(key);
        if (level == null) {
            helper.fail("level " + key.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail(key.location() + " is not generating from world data");
            return;
        }
        WorldData wd = t.data();
        JsonObject lake = wd.manifest().notes().getAsJsonObject("lake");
        if (lake == null || !lake.has("closed_bowls")) {
            helper.fail("world data " + wd.folder() + " has no notes.lake.closed_bowls (an old lake-test world?)");
            return;
        }
        helper.assertTrue(wd.has(WorldData.CLOSED_BASIN), "the lake-test world has no closed_basin layer");
        int seaTop = g.getSeaLevel() - 1;
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        StringBuilder detail = new StringBuilder();
        int bowls = 0;
        boolean caves = key != GameTestSupport.TEST_LAKE;
        for (var e : lake.getAsJsonArray("closed_bowls")) {
            JsonObject b = e.getAsJsonObject();
            int cx = (int) Math.round(b.getAsJsonArray("centre_block").get(0).getAsDouble());
            int cz = (int) Math.round(b.getAsJsonArray("centre_block").get(1).getAsDouble());
            double floorY = b.get("floor_y").getAsDouble();
            double floorR = b.get("floor_radius").getAsDouble();
            double outer = floorR + b.get("slope_blocks").getAsDouble() - 2;
            boolean wet = b.has("lake_level_y") && !b.get("lake_level_y").isJsonNull();
            int lakeTop = wet ? WaterLevels.topWaterBlock(b.get("lake_level_y").getAsDouble()) : Integer.MIN_VALUE;
            int bandFrom = wet ? lakeTop + 1 : (int) Math.ceil(floorY) - 1 + SEA_BAND_ABOVE_FLOOR;
            helper.assertTrue(wd.closedBasin(cx, cz), "bowl at (" + cx + ", " + cz + ") is not in the closed_basin layer");
            int columns = 0, surfaceWater = 0, overLake = 0, seaBand = 0, r = (int) Math.ceil(outer);
            for (int x = cx - r; x <= cx + r; x++)
                for (int z = cz - r; z <= cz + r; z++) {
                    double d = Math.hypot(x - cx, z - cz);
                    if (d > outer)
                        continue;
                    columns++;
                    LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
                    int y = (int) Math.ceil(lake.get("plain_y").getAsDouble()) + 8;
                    while (y > level.getMinBuildHeight() && chunk.getBlockState(pos.set(x, y, z)).isAir())
                        y--;
                    if (chunk.getBlockState(pos.set(x, y, z)).getFluidState().is(FluidTags.WATER)) {
                        boolean bad = wet ? y > lakeTop : !caves || y >= bandFrom;
                        if (bad && wet)
                            overLake++;
                        else if (bad)
                            surfaceWater++;
                        if (bad && detail.length() < 600)
                            detail.append(String.format(" open water at (%d,%d,%d);", x, y, z));
                    }
                    if (d <= floorR)
                        for (int yy = bandFrom; yy <= seaTop; yy++)
                            if (chunk.getBlockState(pos.set(x, yy, z)).getFluidState().is(FluidTags.WATER)) {
                                seaBand++;
                                if (detail.length() < 600)
                                    detail.append(String.format(" sea band water at (%d,%d,%d);", x, yy, z));
                            }
                }
            int[] centre = centreColumn(level, b);
            DeepTime.LOGGER.info("[gametest] closedBowlsHoldNoSea level={} bowl=({}, {}) floor_y={} lake_top_block={} columns={} "
                            + "surfaceWater={} overLake={} seaBand={} (y {} to {}) centre water_top_y={} floor_y={}",
                    key.location(), cx, cz, floorY, wet ? lakeTop : "-", columns, surfaceWater, overLake, seaBand, bandFrom, seaTop,
                    centre[2] == Integer.MIN_VALUE ? "-" : centre[2], centre[3]);
            helper.assertTrue(columns > 0, "no columns in the bowl at (" + cx + ", " + cz + ")");
            helper.assertTrue(surfaceWater == 0, surfaceWater + " columns of the dry bowl at (" + cx + ", " + cz
                    + ") hold open water (the sea would stand to y " + seaTop + "):" + detail);
            helper.assertTrue(overLake == 0, overLake + " columns of the lake bowl at (" + cx + ", " + cz
                    + ") hold water above the lake's top block y " + lakeTop + ":" + detail);
            helper.assertTrue(seaBand == 0, seaBand + " water blocks on the floor of the bowl at (" + cx + ", " + cz
                    + ") between y " + bandFrom + " and the sea's top block " + seaTop + ":" + detail);
            if (wet)
                helper.assertTrue(centre[2] == lakeTop + 1, "the lake below sea level at (" + cx + ", " + cz
                        + ") has its water top at y " + centre[2] + ", want " + (lakeTop + 1));
            else
                helper.assertTrue(centre[2] == Integer.MIN_VALUE || (caves && centre[2] <= bandFrom),
                        "the dry bowl's centre holds water to y " + centre[2]);
            bowls++;
        }
        helper.assertTrue(bowls == 2, bowls + " closed bowls in notes.lake, want 2");
        helper.succeed();
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
     * A footprint column is one that has a water level by WaterPass's own rule ({@link WaterLevels#levelAt}:
     * its nearest cell, else a wet cell within {@code reach}; a shore column is the latter kind).
     * In it, the floor is the first block below the water surface that is neither air nor water;
     * every block between the floor and the surface should be a water source. Anything else that
     * holds water in the box is outside.
     */
    static Count count(ServerLevel level, WorldDataTerrain t, int reach, Box box) {
        RasterSampler levels = t.data().layer(WorldData.WATER_LEVEL);
        long expected = 0, sources = 0, outside = 0, analytic = 0;
        int columns = 0, shore = 0, cornerMax = 0, corners = 0;
        List<BlockPos> water = new ArrayList<>();
        StringBuilder detail = new StringBuilder();
        java.util.Map<Long, Integer> floors = new java.util.HashMap<>();
        int washable = 0;
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (int x = box.minX(); x <= box.maxX(); x++) {
            for (int z = box.minZ(); z <= box.maxZ(); z++) {
                LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
                double wl = WaterLevels.levelAt(levels, x, z, reach);
                int top = Integer.MIN_VALUE, floor = Integer.MIN_VALUE;
                if (!Double.isNaN(wl)) {
                    top = WaterLevels.topWaterBlock(wl);
                    // A plant or snow layer at the waterline is not a wall: flowing water washes it
                    // away and the block becomes a source (not a leak).
                    BlockState atTop = chunk.getBlockState(pos.set(x, top, z));
                    if (!open(atTop) && atTop.canBeReplaced(net.minecraft.world.level.material.Fluids.WATER))
                        washable++;
                    floor = top;
                    while (floor > level.getMinBuildHeight() && open(chunk.getBlockState(pos.set(x, floor, z))))
                        floor--;
                    if (top > floor) {
                        floors.put(BlockPos.asLong(x, 0, z), floor);
                        columns++;
                        if (Double.isNaN(levels.nearest(x, z)))
                            shore++;
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
        return new Count(expected, sources, outside, columns, shore, analytic, cornerMax, corners, water, detail.toString(),
                floors, washable);
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
