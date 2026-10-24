// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.net.PlanetDebugLine;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.server.SpawnHandler;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.minecraftindustrialcomplex.deeptime.worldgen.CubeFill;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import com.minecraftindustrialcomplex.deeptime.wrap.CubeBorder;
import com.minecraftindustrialcomplex.deeptime.wrap.EdgeBorder;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Biomes;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.RotatedPillarBlock;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.StairBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.material.Fluids;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The cube world in the game (format 0.7, cube plan G15): {@code test_planet_cube} is the 4k test
 * planet baked as a cube (face side s = 1,024, period 4s) with the pack's biomes, and
 * {@code test_cube_bare} the same world data under the bare biome. The generator, world data, layout
 * and planet info agree; chunks generate on the faces, at every seam and at the corners; a chunk off
 * the faces is its partner turned, block for block and biome for biome; past the net's corners is
 * deep ocean; block states turn with the fold; the temporary border stops a survival player at the
 * 12 turning edges, folds a deep arrival onto its partner and lets a spectator pass.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class CubeWorldGameTests {

    private CubeWorldGameTests() {}

    static ServerLevel level(GameTestHelper helper, ResourceKey<Level> key) {
        ServerLevel level = helper.getLevel().getServer().getLevel(key);
        if (level == null)
            helper.fail("level " + key.location() + " does not exist");
        return level;
    }

    static GeologicChunkGenerator generator(GameTestHelper helper, ServerLevel level) {
        if (level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.cubeLayout() != null)
            return g;
        helper.fail(level.dimension().location() + " is not a cube world");
        throw new IllegalStateException();
    }

    /** The block {@code d} deep beyond net edge {@code e} (0 touching its line), {@code along} the edge. */
    static long[] beyond(CubeLayout.NetEdge e, long d, long along) {
        long n = e.beyond() == CubeLayout.Side.MAX ? e.coord() + d : e.coord() - 1 - d;
        return e.axis() == CubeLayout.Axis.X ? new long[]{n, along} : new long[]{along, n};
    }

    /** The chunk of block (x, z), x in Toroidal World's frame (where the game generates it). */
    static ChunkPos chunkOf(CubeLayout cube, long x, long z) {
        return new ChunkPos((int) Math.floorDiv(CubeFill.twX(cube, x), 16), (int) Math.floorDiv(z, 16));
    }

    /**
     * The chunks wholly off the faces a test walks: per turning edge, the chunk at the middle of its
     * span touching the line and the one 64 blocks out; per corner of F0, the chunk a void square's
     * diagonal cuts 64 to 79 blocks from both edges (two partners in one chunk).
     */
    static Map<String, ChunkPos> outsideChunks(CubeLayout cube, long[] depths) {
        Map<String, ChunkPos> out = new LinkedHashMap<>();
        List<CubeLayout.NetEdge> edges = cube.edges();
        for (int k = 2; k < edges.size(); k++) {
            CubeLayout.NetEdge e = edges.get(k);
            long along = (e.spanFrom() + e.spanTo()) / 2;
            for (long d : depths) {
                long[] b = beyond(e, d, along);
                out.put(e.name() + "@" + d, chunkOf(cube, b[0], b[1]));
            }
        }
        long h = cube.side() / 2;
        for (long[] c : new long[][]{{h, -h}, {-h, -h}, {h, h}, {-h, h}}) {
            long x = c[0] > 0 ? c[0] + 64 : c[0] - 80, z = c[1] > 0 ? c[1] + 64 : c[1] - 80;
            out.put("diagonal@" + c[0] + "," + c[1], chunkOf(cube, x, z));
        }
        return out;
    }

    /** The first chunk whose every column lies past the net's corners (in no band), searched beyond N-top's corner. */
    static ChunkPos oceanChunk(CubeLayout cube) {
        long h = cube.side() / 2, s = cube.side();
        for (long z = -3 * h - 16; z > -3 * h - s; z -= 16)
            for (long x = h; x < 3 * h; x += 16) {
                boolean all = true;
                for (int lx = 0; lx < 16 && all; lx += 15)
                    for (int lz = 0; lz < 16 && all; lz += 15)
                        all = cube.fold(x + lx, z + lz) instanceof CubeLayout.Ocean;
                if (all)
                    return chunkOf(cube, x, z);
            }
        return null;
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void cubeWorldLayout(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = generator(helper, level);
        if (!(g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_cube is not generating from world data");
            return;
        }
        WorldData wd = t.data();
        CubeLayout cube = g.cubeLayout();
        helper.assertTrue(g.dataRef().cube() && wd.projection() instanceof WorldProjection.Cube,
                "test_planet_cube is a cube world: " + wd.projection());
        helper.assertTrue(cube.side() == 1024 && cube.period() == 4096 && wd.circumference() == 4096
                        && g.dataRef().circumferenceBlocks() == 4096 && g.dataRef().faceBlocks() == 1024,
                "s 1,024, period 4,096: s " + cube.side() + ", C " + wd.circumference());
        WorldLayout layout = wd.layout();
        helper.assertTrue(layout != null && layout.kind() == WorldProjection.Kind.CUBE
                        && layout.layoutVersion() == WorldLayout.CUBE_LAYOUT_VERSION && layout.edges().size() == 14
                        && layout.cube() != null && layout.cube().faceBlocks() == 1024 && layout.cube().faces().size() == 6,
                "the cube layout block: " + layout);
        helper.assertTrue(layout.border() != null && layout.border().temporary(), "a temporary border: " + layout.border());
        EdgeBorder.CubeStop stop = EdgeBorder.cubeStop(level);
        helper.assertTrue(stop != null && stop.border().segmentCount() == 12, "the cube border is in force: " + stop);
        helper.assertTrue(Double.isNaN(EdgeBorder.stopZ(level)), "no Petroff–Guyou stop line on a cube world");

        // The planet info: the client builds the same projection from it.
        WorldInfoPayload info = WorldInfoPayload.forLevel(level);
        helper.assertTrue(info.cube() && info.circumference() == 4096 && info.poleCap() == null && info.worldProjection() != null,
                "the planet info: " + info);
        WorldProjection.Cube wp = (WorldProjection.Cube) wd.projection();
        WorldProjection.Cube cp = (WorldProjection.Cube) info.worldProjection();
        for (CubeLayout.Face f : CubeLayout.Face.all()) {
            double x = cube.faceCentreX(f) + 0.5, z = cube.faceCentreZ(f) + 0.5;
            double[] a = wp.latLon(x, z), b = cp.latLon(x, z);
            String line = PlanetDebugLine.format(info, x, z, 0f);
            DeepTime.LOGGER.info("[gametest] cubeWorldLayout face {} centre ({}, {}): {}", f, x, z, line);
            helper.assertTrue(!Double.isNaN(a[0]) && Math.abs(a[0] - b[0]) < 1e-9 && Math.abs(a[1] - b[1]) < 1e-9,
                    "face " + f + " centre: the server's and the client's lat/lon agree: " + a[0] + "," + a[1] + " vs " + b[0] + "," + b[1]);
            helper.assertTrue(line != null && line.contains("°") && line.contains("face " + f.name() + " ") && !line.contains("mismatch"),
                    "the debug line at face " + f + ": " + line);
        }
        ChunkPos ocean = oceanChunk(cube);
        helper.assertTrue(ocean != null, "a chunk past the net's corners");
        String off = PlanetDebugLine.format(info, ocean.getMiddleBlockX() + 0.5, ocean.getMiddleBlockZ() + 0.5, 0f);
        helper.assertTrue(off != null && off.contains("off the planet"), "the debug line past the corners: " + off);

        // Every seam joins: a point just inside an edge and its neighbour across the seam (the fold of
        // the point just outside) are a block apart on the sphere.
        double r = cube.radius();
        List<CubeLayout.NetEdge> edges = cube.edges();
        double worst = 0;
        for (int k = 2; k < edges.size(); k++) {
            CubeLayout.NetEdge e = edges.get(k);
            for (int q = 1; q <= 3; q++) {
                double along = e.spanFrom() + q * (e.spanTo() - e.spanFrom()) / 4.0 + 0.5;
                double n = e.coord(), in = e.beyond() == CubeLayout.Side.MAX ? n - 0.5 : n + 0.5,
                        outN = e.beyond() == CubeLayout.Side.MAX ? n + 0.5 : n - 0.5;
                double xi = e.axis() == CubeLayout.Axis.X ? in : along, zi = e.axis() == CubeLayout.Axis.X ? along : in;
                double xo = e.axis() == CubeLayout.Axis.X ? outN : along, zo = e.axis() == CubeLayout.Axis.X ? along : outN;
                CubeLayout.PointOnFace p = cube.foldPoint(xo, zo);
                helper.assertTrue(p != null && p.edge() == k && p.face() == edges.get(e.partner()).face(),
                        "the point just past " + e.name() + " folds onto " + edges.get(e.partner()).face() + ": " + p);
                double[] u = wp.unit(xi, zi), v = wp.unit(p.x(), p.z());
                double dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
                double blocks = Math.acos(Math.max(-1, Math.min(1, dot))) * r;
                worst = Math.max(worst, blocks);
                helper.assertTrue(blocks < 3, e.name() + " at " + along + ": across the seam is " + blocks + " blocks on the sphere");
            }
        }
        DeepTime.LOGGER.info("[gametest] cubeWorldLayout every seam joins: worst {} blocks across (R {})", worst, r);

        // Spawn: not at a corner, not at a turning edge, yes at a face centre.
        CubeBorder border = stop.border();
        long h = cube.side() / 2;
        helper.assertTrue(SpawnHandler.cubeSpawnOk(border, 0, 0), "spawn is fine at F0's centre");
        helper.assertTrue(!SpawnHandler.cubeSpawnOk(border, (int) h - 100, (int) -h + 100), "no spawn within 256 of corner n0");
        helper.assertTrue(!SpawnHandler.cubeSpawnOk(border, (int) h - 10, (int) (-2 * h)), "no spawn by N-east");
        helper.assertTrue(!SpawnHandler.cubeSpawnOk(border, (int) h + 10, (int) (-2 * h)), "no spawn off the planet");
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void cubeChunksGenerateAtFacesSeamsAndCorners(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = generator(helper, level);
        CubeLayout cube = g.cubeLayout();
        Map<String, ChunkPos> chunks = new LinkedHashMap<>();
        for (CubeLayout.Face f : CubeLayout.Face.all())
            chunks.put("centre " + f, chunkOf(cube, cube.faceCentreX(f), cube.faceCentreZ(f)));
        List<CubeLayout.NetEdge> edges = cube.edges();
        for (int k = 2; k < edges.size(); k++) {
            CubeLayout.NetEdge e = edges.get(k);
            long along = (e.spanFrom() + e.spanTo()) / 2 + 40;
            chunks.put(e.name() + " inside", chunkOf(cube, beyond(e, -9, along)[0], beyond(e, -9, along)[1]));
            for (long d : new long[]{8, cube.side() / 2}) {
                long[] b = beyond(e, d, along);
                chunks.put(e.name() + "@" + d, chunkOf(cube, b[0], b[1]));
            }
        }
        for (CubeLayout.Corner c : cube.corners())
            for (long[] img : c.images())
                for (int dx = -1; dx <= 0; dx++)
                    for (int dz = -1; dz <= 0; dz++)
                        chunks.put(c.name() + " " + img[0] + "," + img[1] + " " + dx + "," + dz,
                                chunkOf(cube, img[0] + 16L * dx, img[1] + 16L * dz));
        chunks.put("ocean", oceanChunk(cube));
        int sea = g.getSeaLevel(), minY = level.getMinBuildHeight();
        int generated = 0, outside = 0, lowestTop = Integer.MAX_VALUE;
        for (var entry : chunks.entrySet()) {
            ChunkPos p = entry.getValue();
            LevelChunk chunk = level.getChunk(p.x, p.z);
            generated++;
            if (CubeFill.outside(cube, p))
                outside++;
            for (int lx = 0; lx < 16; lx++)
                for (int lz = 0; lz < 16; lz++) {
                    int x = p.getMinBlockX() + lx, z = p.getMinBlockZ() + lz;
                    BlockState floor = chunk.getBlockState(new BlockPos(x, minY + 4, z));
                    helper.assertTrue(!floor.isAir(), entry.getKey() + ": a hole to the floor at " + x + ", " + z);
                    int top = chunk.getHeight(net.minecraft.world.level.levelgen.Heightmap.Types.WORLD_SURFACE, lx, lz);
                    lowestTop = Math.min(lowestTop, top);
                }
        }
        DeepTime.LOGGER.info("[gametest] cubeChunksGenerateAtFacesSeamsAndCorners {} chunks ({} off the faces), lowest top {} (sea {})",
                generated, outside, lowestTop, sea);
        helper.assertTrue(outside >= 24, "the walk reaches off the faces: " + outside);
        helper.succeed();
    }

    /**
     * Compares the outside chunk {@code p} with its partners column by column (each column's own
     * fold), every y: {compared blocks, mismatches, ocean columns}; the first mismatch goes to
     * {@code first[0]}.
     */
    static long[] compareWithPartners(ServerLevel level, CubeLayout cube, ChunkPos p, int sea, String[] first) {
        LevelChunk chunk = level.getChunk(p.x, p.z);
        long compared = 0, mismatched = 0, oceanCols = 0;
        int minY = level.getMinBuildHeight(), maxY = level.getMaxBuildHeight();
        BlockPos.MutableBlockPos a = new BlockPos.MutableBlockPos(), b = new BlockPos.MutableBlockPos();
        for (int lx = 0; lx < 16; lx++)
            for (int lz = 0; lz < 16; lz++) {
                long x = p.getMinBlockX() + lx, z = p.getMinBlockZ() + lz;
                CubeLayout.Fold f = cube.fold(x, z);
                if (!(f instanceof CubeLayout.Outside o)) {
                    oceanCols++;
                    BlockState w = chunk.getBlockState(a.set(x, sea - 1, z)), rock = chunk.getBlockState(a.set(x, sea - CubeFill.OCEAN_DEPTH - 1, z));
                    if (!w.getFluidState().is(Fluids.WATER) || rock.isAir() || !rock.getFluidState().isEmpty()) {
                        mismatched++;
                        if (first[0] == null)
                            first[0] = "ocean column " + x + ", " + z + ": " + w + " at sea−1, " + rock + " at the floor";
                    }
                    continue;
                }
                long px = CubeFill.twX(cube, o.x()), pz = o.z();
                LevelChunk partner = level.getChunk((int) Math.floorDiv(px, 16), (int) Math.floorDiv(pz, 16));
                Rotation turn = CubeFill.rotation(o.iso().k());
                for (int y = minY; y < maxY; y++) {
                    BlockState got = chunk.getBlockState(a.set(x, y, z));
                    BlockState want = partner.getBlockState(b.set(px, y, pz)).rotate(turn);
                    compared++;
                    if (got != want) {
                        mismatched++;
                        if (first[0] == null)
                            first[0] = "(" + x + ", " + y + ", " + z + ") holds " + got + "; its partner (" + px + ", " + y + ", " + pz
                                    + ") turned by k " + o.iso().k() + " is " + want;
                    }
                }
            }
        return new long[]{compared, mismatched, oceanCols};
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void cubeFillIsThePartnerTurned(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_CUBE_BARE);
        GeologicChunkGenerator g = generator(helper, level);
        CubeLayout cube = g.cubeLayout();
        int sea = g.getSeaLevel();
        long compared = 0, mismatched = 0, ocean = 0;
        String[] first = new String[1];
        String firstChunk = null;
        Map<String, ChunkPos> chunks = outsideChunks(cube, new long[]{0, 64});
        chunks.put("ocean", oceanChunk(cube));
        for (var entry : chunks.entrySet()) {
            helper.assertTrue(CubeFill.outside(cube, entry.getValue()), entry.getKey() + " is off the faces: " + entry.getValue());
            String before = first[0];
            long[] r = compareWithPartners(level, cube, entry.getValue(), sea, first);
            compared += r[0];
            mismatched += r[1];
            ocean += r[2];
            if (before == null && first[0] != null)
                firstChunk = entry.getKey();
        }
        DeepTime.LOGGER.info("[gametest] cubeFillIsThePartnerTurned {} chunks, {} blocks compared, {} ocean columns, {} mismatches{}",
                chunks.size(), compared, ocean, mismatched, first[0] == null ? "" : "; first in " + firstChunk + ": " + first[0]);
        helper.assertTrue(compared > 0 && ocean >= 256, "blocks were compared (" + compared + ") and the ocean chunk checked (" + ocean + ")");
        helper.assertTrue(mismatched == 0, mismatched + " blocks differ from their partners; first in " + firstChunk + ": " + first[0]);
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void cubeFillBiomesAreThePartners(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = generator(helper, level);
        CubeLayout cube = g.cubeLayout();
        int minQ = level.getMinBuildHeight() >> 2, maxQ = level.getMaxBuildHeight() >> 2;
        long compared = 0, mismatched = 0;
        String first = null;
        Map<String, ChunkPos> chunks = outsideChunks(cube, new long[]{0});
        for (var entry : chunks.entrySet()) {
            ChunkPos p = entry.getValue();
            LevelChunk chunk = level.getChunk(p.x, p.z);
            for (int qx = p.getMinBlockX() >> 2; qx < (p.getMinBlockX() >> 2) + 4; qx++)
                for (int qz = p.getMinBlockZ() >> 2; qz < (p.getMinBlockZ() >> 2) + 4; qz++) {
                    if (!(cube.fold(4L * qx + 1, 4L * qz + 1) instanceof CubeLayout.Outside o))
                        continue;
                    long px = CubeFill.twX(cube, o.x()), pz = o.z();
                    LevelChunk partner = level.getChunk((int) Math.floorDiv(px, 16), (int) Math.floorDiv(pz, 16));
                    for (int qy = minQ; qy < maxQ; qy += 4) {
                        Holder<Biome> got = chunk.getNoiseBiome(qx, qy, qz);
                        Holder<Biome> want = partner.getNoiseBiome((int) Math.floorDiv(px, 4), qy, (int) Math.floorDiv(pz, 4));
                        compared++;
                        if (!got.equals(want)) {
                            mismatched++;
                            if (first == null)
                                first = entry.getKey() + " quart (" + qx + ", " + qy + ", " + qz + "): " + got.getRegisteredName()
                                        + "; its partner's " + want.getRegisteredName();
                        }
                    }
                }
        }
        DeepTime.LOGGER.info("[gametest] cubeFillBiomesAreThePartners {} chunks, {} quarts compared, {} mismatches{}",
                chunks.size(), compared, mismatched, first == null ? "" : "; first " + first);
        helper.assertTrue(compared > 0 && mismatched == 0, mismatched + " of " + compared + " quarts differ; first " + first);
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void pastTheCornersIsDeepOcean(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = generator(helper, level);
        CubeLayout cube = g.cubeLayout();
        ChunkPos p = oceanChunk(cube);
        helper.assertTrue(p != null && CubeFill.outside(cube, p), "a chunk past the net's corners: " + p);
        LevelChunk chunk = level.getChunk(p.x, p.z);
        int sea = g.getSeaLevel(), floor = sea - CubeFill.OCEAN_DEPTH;
        BlockPos.MutableBlockPos m = new BlockPos.MutableBlockPos();
        for (int lx = 0; lx < 16; lx++)
            for (int lz = 0; lz < 16; lz++) {
                int x = p.getMinBlockX() + lx, z = p.getMinBlockZ() + lz;
                helper.assertTrue(chunk.getBlockState(m.set(x, sea - 1, z)).getFluidState().is(Fluids.WATER)
                                && chunk.getBlockState(m.set(x, floor, z)).getFluidState().is(Fluids.WATER),
                        "water from the floor to sea level at " + x + ", " + z);
                BlockState rock = chunk.getBlockState(m.set(x, floor - 1, z));
                helper.assertTrue(!rock.isAir() && rock.getFluidState().isEmpty(), "rock under the floor at " + x + ", " + z + ": " + rock);
                helper.assertTrue(chunk.getBlockState(m.set(x, sea, z)).isAir(), "air over the sea at " + x + ", " + z);
            }
        Holder<Biome> b = chunk.getNoiseBiome(p.getMinBlockX() >> 2, sea >> 2, p.getMinBlockZ() >> 2);
        helper.assertTrue(b.is(Biomes.DEEP_OCEAN), "the biome past the corners is deep ocean: " + b.getRegisteredName());
        DeepTime.LOGGER.info("[gametest] pastTheCornersIsDeepOcean chunk {}: water {} to {}, {}", p, floor, sea - 1, b.getRegisteredName());
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void blockStatesTurnWithTheFold(GameTestHelper helper) {
        // The fold's motion takes the outside block to its partner: a vector d there is the vector
        // iso.inverse().vector(d) here, so a state facing d in the partner faces that way outside.
        for (int k = 0; k < 4; k++) {
            CubeLayout.Iso back = new CubeLayout.Iso(k, 0, 0).inverse();
            Rotation turn = CubeFill.rotation(k);
            for (Direction d : Direction.Plane.HORIZONTAL) {
                double[] v = back.vector(d.getStepX(), d.getStepZ());
                Direction want = Direction.getNearest(v[0], 0, v[1]);
                BlockState stairs = Blocks.OAK_STAIRS.defaultBlockState().setValue(StairBlock.FACING, d).rotate(turn);
                helper.assertTrue(stairs.getValue(StairBlock.FACING) == want,
                        "k " + k + ": stairs facing " + d + " turn to " + stairs.getValue(StairBlock.FACING) + ", the fold says " + want);
            }
            BlockState log = Blocks.OAK_LOG.defaultBlockState().setValue(RotatedPillarBlock.AXIS, Direction.Axis.X).rotate(turn);
            Direction.Axis axis = k % 2 == 0 ? Direction.Axis.X : Direction.Axis.Z;
            helper.assertTrue(log.getValue(RotatedPillarBlock.AXIS) == axis, "k " + k + ": an x log lies " + log.getValue(RotatedPillarBlock.AXIS));
        }
        // And every turning edge's own motion: its quarter turns are a rotation (no mirror), and the
        // edge's direction outward maps onto the partner edge's direction inward.
        CubeLayout cube = CubeLayout.shared(1024);
        List<CubeLayout.NetEdge> edges = cube.edges();
        for (int k = 2; k < edges.size(); k++) {
            CubeLayout.NetEdge e = edges.get(k), q = edges.get(e.partner());
            double[] out = outward(e), in = outward(q);
            double[] v = e.iso().vector(out[0], out[1]);
            helper.assertTrue(Math.abs(v[0] + in[0]) < 1e-9 && Math.abs(v[1] + in[1]) < 1e-9,
                    e.name() + ": outward " + out[0] + "," + out[1] + " maps to " + v[0] + "," + v[1] + ", not into " + q.name());
        }
        helper.succeed();
    }

    /** The unit vector pointing beyond edge {@code e}. */
    static double[] outward(CubeLayout.NetEdge e) {
        double sign = e.beyond() == CubeLayout.Side.MAX ? 1 : -1;
        return e.axis() == CubeLayout.Axis.X ? new double[]{sign, 0} : new double[]{0, sign};
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void cubeBorderStopsAndFolds(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = generator(helper, level);
        CubeLayout cube = g.cubeLayout();
        EdgeBorder.CubeStop stop = EdgeBorder.cubeStop(level);
        helper.assertTrue(stop != null, "the cube border is in force");
        CubeBorder border = stop.border();
        double h = cube.side() / 2.0, y = level.getMaxBuildHeight() - 20.5;

        // name → {x, z, yaw}
        Map<String, double[]> at = new LinkedHashMap<>();
        at.put("warned", new double[]{h - 10.5, -2 * h + 0.5, 0});
        at.put("N centre", new double[]{0.5, -2 * h + 0.5, 0});
        at.put("S centre", new double[]{0.5, 2 * h + 0.5, 0});
        // The poles: the cube is turned freely, so they lie somewhere in N and S (plan G15: a player
        // teleported to each pole is not pushed).
        if (g.terrain() instanceof WorldDataTerrain t && t.data().projection() instanceof WorldProjection.Cube wp) {
            double[] np = wp.blockOf(90, 0), sp = wp.blockOf(-90, 0);
            helper.assertTrue(np != null && sp != null, "the poles have blocks");
            at.put("north pole", new double[]{CubeFill.twX(cube, (long) Math.floor(np[0])) + 0.5, Math.floor(np[1]) + 0.5, 0});
            at.put("south pole", new double[]{CubeFill.twX(cube, (long) Math.floor(sp[0])) + 0.5, Math.floor(sp[1]) + 0.5, 0});
        } else {
            helper.fail("test_planet_cube's projection is not the cube");
        }
        at.put("void square", new double[]{h + 3.5, -h - 3.5, 0});
        at.put("corner n0", new double[]{h + 0.5, -h - 0.5, 0});
        at.put("deep", new double[]{h + 100.5, -2 * h + 0.5, 30});
        at.put("past the corners", new double[]{h + 64.5, -3 * h - 64.5, 0});
        at.put("spectator", new double[]{h + 3.5, -2 * h + 0.5, 0});
        List<long[]> forced = new ArrayList<>();
        for (double[] p : at.values()) {
            int cx = (int) Math.floor(p[0]) >> 4, cz = (int) Math.floor(p[1]) >> 4;
            level.setChunkForced(cx, cz, true);
            forced.add(new long[]{cx, cz});
        }
        CubeLayout.PointOnFace deepFold = cube.foldPoint(at.get("deep")[0], at.get("deep")[1]);
        helper.assertTrue(deepFold != null && "N-east".equals(cube.edges().get(deepFold.edge()).name())
                        && deepFold.face() == CubeLayout.Face.F1,
                "a point 100 blocks past N-east folds onto F1: " + deepFold);
        helper.assertTrue(cube.foldPoint(at.get("past the corners")[0], at.get("past the corners")[1]) == null,
                "past the corners folds nowhere");

        Map<String, MockPlayers.Mock> mocks = new LinkedHashMap<>();
        List<MockPlayers.Mock> all = new ArrayList<>();
        try {
            for (var e : at.entrySet()) {
                MockPlayers.Mock m = MockPlayers.create(level, "spectator".equals(e.getKey()) ? GameType.SPECTATOR : GameType.SURVIVAL);
                mocks.put(e.getKey(), m);
                all.add(m);
                m.player.teleportTo(level, e.getValue()[0], y, e.getValue()[1], (float) e.getValue()[2], 0);
            }
            for (MockPlayers.Mock m : all)
                m.forgetSent();
            EdgeBorder.tickCube(level, stop);

            for (String still : new String[]{"warned", "N centre", "S centre", "north pole", "south pole", "spectator"}) {
                MockPlayers.Mock m = mocks.get(still);
                double[] p = at.get(still);
                helper.assertTrue(Math.abs(m.player.getX() - p[0]) < 1e-9 && Math.abs(m.player.getZ() - p[1]) < 1e-9,
                        still + " is not moved: " + m.player.getX() + ", " + m.player.getZ());
            }
            helper.assertTrue(mocks.get("warned").sawActionBar(EdgeBorder.KEY_CUBE_NEAR), "the warning band warns");
            helper.assertTrue(!mocks.get("N centre").sawActionBar(EdgeBorder.KEY_CUBE_NEAR)
                    && !mocks.get("S centre").sawActionBar(EdgeBorder.KEY_CUBE_NEAR), "the face centres are quiet");
            DeepTime.LOGGER.info("[gametest] cubeBorderStopsAndFolds poles at ({}, {}) and ({}, {}), not pushed",
                    at.get("north pole")[0], at.get("north pole")[1], at.get("south pole")[0], at.get("south pole")[1]);
            for (String stopped : new String[]{"void square", "corner n0", "past the corners"}) {
                MockPlayers.Mock m = mocks.get(stopped);
                double[] p = at.get(stopped);
                double moved = Math.hypot(m.player.getX() - p[0], m.player.getZ() - p[1]);
                helper.assertTrue(border.inside(m.player.getX(), m.player.getZ(), 0.0) && moved > 0,
                        stopped + " is put back on the planet: " + m.player.getX() + ", " + m.player.getZ() + " (moved " + moved + ")");
                helper.assertTrue(m.sawActionBar(EdgeBorder.KEY_CUBE_STOP), stopped + ": the stop says why");
                DeepTime.LOGGER.info("[gametest] cubeBorderStopsAndFolds {} ({}, {}) stopped at ({}, {}), {} blocks",
                        stopped, p[0], p[1], m.player.getX(), m.player.getZ(), moved);
            }
            MockPlayers.Mock deep = mocks.get("deep");
            double[] want = border.nearest(border.frameX(deepFold.x()), deepFold.z(), stop.inset() + 0.05);
            float yaw = CubeBorder.turnYaw(30f, deepFold.iso());
            helper.assertTrue(Math.abs(deep.player.getX() - want[0]) < 1e-6 && Math.abs(deep.player.getZ() - want[1]) < 1e-6
                            && Math.abs(net.minecraft.util.Mth.wrapDegrees(deep.player.getYRot() - yaw)) < 1e-3,
                    "the deep arrival is folded onto F1 at " + want[0] + ", " + want[1] + " yaw " + yaw + ": "
                            + deep.player.getX() + ", " + deep.player.getZ() + " yaw " + deep.player.getYRot());
            helper.assertTrue(Math.abs(deep.player.getZ() - (-h + 100.5)) < 1.0 && cube.faceOfBlock((long) Math.floor(deep.player.getX()),
                            (long) Math.floor(deep.player.getZ())).face() == CubeLayout.Face.F1,
                    "100 blocks inside F1's top edge: z " + deep.player.getZ());
            DeepTime.LOGGER.info("[gametest] cubeBorderStopsAndFolds deep ({}, {}) yaw 30 folded to ({}, {}) yaw {}",
                    at.get("deep")[0], at.get("deep")[1], deep.player.getX(), deep.player.getZ(), deep.player.getYRot());
            helper.succeed();
        } finally {
            MockPlayers.removeAll(all);
            for (long[] c : forced)
                level.setChunkForced((int) c[0], (int) c[1], false);
        }
    }
}
