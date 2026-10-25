// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.api.climate.DeepTimeClimate;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrain;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrainCheck;
import com.minecraftindustrialcomplex.deeptime.net.PlanetDebugLine;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.server.DeepTimeCommands;
import com.minecraftindustrialcomplex.deeptime.server.SiteCommands;
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
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.status.ChunkStatus;
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
 * 12 turning edges, folds a deep arrival onto its partner and lets a spectator pass; the probe
 * commands give the poles, the fold, north, the scale and the distances (cube plan G11).
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

        // The planet info: the client builds the same projection from it.
        WorldInfoPayload info = WorldInfoPayload.forLevel(level);
        helper.assertTrue(info.cube() && info.circumference() == 4096 && info.worldProjection() != null,
                "the planet info: " + info);
        WorldProjection.Cube wp = (WorldProjection.Cube) wd.projection();
        WorldProjection.Cube cp = (WorldProjection.Cube) info.worldProjection();
        for (CubeLayout.Face f : CubeLayout.Face.all()) {
            double x = cube.faceCentreX(f) + 0.5, z = cube.faceCentreZ(f) + 0.5;
            double[] a = wp.latLon(x, z), b = cp.latLon(x, z);
            String line = PlanetDebugLine.format(info, x, z);
            DeepTime.LOGGER.info("[gametest] cubeWorldLayout face {} centre ({}, {}): {}", f, x, z, line);
            helper.assertTrue(!Double.isNaN(a[0]) && Math.abs(a[0] - b[0]) < 1e-9 && Math.abs(a[1] - b[1]) < 1e-9,
                    "face " + f + " centre: the server's and the client's lat/lon agree: " + a[0] + "," + a[1] + " vs " + b[0] + "," + b[1]);
            helper.assertTrue(line != null && line.contains("°") && line.contains("face " + f.name() + " ") && !line.contains("mismatch"),
                    "the debug line at face " + f + ": " + line);
        }
        ChunkPos ocean = oceanChunk(cube);
        helper.assertTrue(ocean != null, "a chunk past the net's corners");
        String off = PlanetDebugLine.format(info, ocean.getMiddleBlockX() + 0.5, ocean.getMiddleBlockZ() + 0.5);
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
                        int pqx = (int) Math.floorDiv(px, 4), pqz = (int) Math.floorDiv(pz, 4);
                        Holder<Biome> want = partner.getNoiseBiome(pqx, qy, pqz);
                        compared++;
                        if (!got.equals(want)) {
                            mismatched++;
                            String why = climateTie(g.getBiomeSource(), level.getChunkSource().randomState().sampler(), pqx, qy, pqz, got, want);
                            DeepTime.LOGGER.info("[gametest] cubeFillBiomesAreThePartners {} quart ({}, {}, {}) {} against partner quart ({}, {}, {}) {}{}",
                                    entry.getKey(), qx, qy, qz, got.getRegisteredName(), pqx, qy, pqz, want.getRegisteredName(), why);
                            if (first == null)
                                first = entry.getKey() + " quart (" + qx + ", " + qy + ", " + qz + "): " + got.getRegisteredName()
                                        + "; its partner's " + want.getRegisteredName() + why;
                        }
                    }
                }
        }
        DeepTime.LOGGER.info("[gametest] cubeFillBiomesAreThePartners {} chunks, {} quarts compared, {} mismatches{}",
                chunks.size(), compared, mismatched, first == null ? "" : "; first " + first);
        helper.assertTrue(compared > 0 && mismatched == 0, mismatched + " of " + compared + " quarts differ; first " + first);
        helper.succeed();
    }

    /**
     * Why quart (qx, qy, qz) can hold either of two biomes: the climate point's fitness to the nearest
     * entry of the parameter list and to the nearest entry of each biome ("a tie" when both are the
     * nearest), and the biome a search that starts cold gives ({@code climate.ColdBiomeSearch}).
     */
    static String climateTie(net.minecraft.world.level.biome.BiomeSource source, net.minecraft.world.level.biome.Climate.Sampler sampler,
                             int qx, int qy, int qz, Holder<Biome> a, Holder<Biome> b) {
        if (!(source instanceof net.minecraft.world.level.biome.MultiNoiseBiomeSource m))
            return "";
        net.minecraft.world.level.biome.Climate.TargetPoint t = sampler.sample(qx, qy, qz);
        long best = Long.MAX_VALUE, fa = Long.MAX_VALUE, fb = Long.MAX_VALUE;
        int atBest = 0;
        for (var e : m.parameters().values()) {
            long f = fitness(e.getFirst(), t);
            if (f < best) {
                best = f;
                atBest = 0;
            }
            if (f == best)
                atBest++;
            if (e.getSecond().equals(a))
                fa = Math.min(fa, f);
            if (e.getSecond().equals(b))
                fb = Math.min(fb, f);
        }
        Holder<Biome> cold;
        com.minecraftindustrialcomplex.deeptime.climate.ColdBiomeSearch.begin();
        try {
            cold = source.getNoiseBiome(qx, qy, qz, sampler);
        } finally {
            com.minecraftindustrialcomplex.deeptime.climate.ColdBiomeSearch.end();
        }
        return " (climate point " + t + ": nearest entry " + best + " (" + atBest + " entries at it), " + a.getRegisteredName() + " " + fa
                + ", " + b.getRegisteredName() + " " + fb + (fa == best && fb == best ? ", a tie" : "") + "; a cold search gives "
                + cold.getRegisteredName() + ")";
    }

    /** {@code Climate.ParameterPoint.fitness}, which is package private. */
    static long fitness(net.minecraft.world.level.biome.Climate.ParameterPoint p, net.minecraft.world.level.biome.Climate.TargetPoint t) {
        return sq(p.temperature().distance(t.temperature())) + sq(p.humidity().distance(t.humidity()))
                + sq(p.continentalness().distance(t.continentalness())) + sq(p.erosion().distance(t.erosion()))
                + sq(p.depth().distance(t.depth())) + sq(p.weirdness().distance(t.weirdness())) + sq(p.offset());
    }

    private static long sq(long v) {
        return v * v;
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

    /**
     * The commands on a cube world (cube plan G11), through the JSON they print.
     * <ul>
     *   <li>{@code probe pole} finds both poles inside N and S, at the projection's pole blocks, where
     *       the border lets a player reach them. From anywhere, the planet distances to the two poles
     *       add up to half the planet's circumference, and the walk under the border is never shorter
     *       than the straight line in the net.</li>
     *   <li>{@code probe column} gives the face and cell, the turning edge a block lies past and the
     *       planet block it shows (the layout's fold), or the ocean past the corners, with north and the
     *       metres a block spans as the climate API gives them.</li>
     *   <li>{@code probe tw} gives the period 4s and the face side.</li>
     *   <li>{@code sites near} gives each site's planet distance and its walk, never shorter than its
     *       distance in the net.</li>
     * </ul>
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void cubeCommandProbes(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = generator(helper, level);
        CubeLayout cube = g.cubeLayout();
        if (!(g.terrain() instanceof WorldDataTerrain t && t.data().projection() instanceof WorldProjection.Cube wp)) {
            helper.fail("test_planet_cube's projection is not the cube");
            return;
        }
        double mpb = t.data().manifest().mPerBlockH(), h = cube.side() / 2.0, period = cube.period();
        CubeBorder border = CubeBorder.of(cube.side());
        double halfKm = Math.PI * wp.radius() * mpb / 1000.0;   // ledger: SCL-107
        StringBuilder log = new StringBuilder();

        // probe pole from F0's, N's and S's centres and from F2.
        double[] np = wp.blockOf(90, 0), sp = wp.blockOf(-90, 0);
        helper.assertTrue(np != null && sp != null, "the poles have blocks");
        for (double[] from : new double[][]{{0.5, 0.5}, {0.5, -2 * h + 0.5}, {0.5, 2 * h + 0.5}, {4 * h + 0.5, 100.5}}) {
            JsonObject o = DeepTimeCommands.poleJson(level, from[0], from[1]);
            String where = "probe pole at (" + from[0] + ", " + from[1] + "): " + o;
            helper.assertTrue("cube".equals(o.get("projection").getAsString()) && o.get("cubeBorder").getAsBoolean()
                    && o.get("borderZone").getAsInt() == 0, "a cube world inside its border: " + where);
            helper.assertTrue(Math.abs(o.get("latitude").getAsDouble() - DeepTimeClimate.latitudeDeg(level, from[0], from[1])) < 1e-9
                            && Math.abs(o.get("northBearing").getAsDouble() - DeepTimeClimate.northBearingDeg(level, from[0], from[1])) < 1e-9
                            && Math.abs(o.get("metresPerBlock").getAsDouble() - DeepTimeClimate.metresPerBlock(level, from[0], from[1])) < 1e-9,
                    "the source's latitude, north and scale are the climate API's: " + where);
            double km = 0;
            for (boolean north : new boolean[]{true, false}) {
                JsonObject p = o.getAsJsonObject(north ? "northPole" : "southPole");
                double[] want = north ? np : sp;
                double px = p.get("x").getAsDouble(), pz = p.get("z").getAsDouble();
                double dx = px - want[0];
                dx -= period * Math.rint(dx / period);
                helper.assertTrue(p.get("face").getAsString().equals(north ? "N" : "S")
                                && Math.abs(p.get("latitude").getAsDouble()) > 89.999 && p.get("latitude").getAsDouble() * (north ? 1 : -1) > 0
                                && Math.hypot(dx, pz - want[1]) < 1e-3 && px >= -2 * cube.side() && px < 2 * cube.side()
                                && p.get("reachable").getAsBoolean(),
                        (north ? "the north" : "the south") + " pole, in " + (north ? "N" : "S") + " at ("
                                + want[0] + ", " + want[1] + ") in Toroidal World's frame, reachable: " + where);
                double sx = from[0] - px;
                sx -= period * Math.rint(sx / period);
                double straight = Math.hypot(sx, from[1] - pz), walk = p.get("walkBlocks").getAsDouble();
                helper.assertTrue(walk >= straight - 1e-6 && walk < straight + 4 * h,
                        "the walk to the pole is no shorter than the straight line in the net (" + straight + "): " + where);
                km += p.get("distanceKm").getAsDouble();
            }
            helper.assertTrue(Math.abs(km - halfKm) < 1e-6 * halfKm, "the distances to the two poles add up to "
                    + halfKm + " km: " + km + ", " + where);
            log.append(String.format(java.util.Locale.ROOT, " pole(%.0f, %.0f: N %.0f km walk %.0f, S %.0f km walk %.0f)", from[0], from[1],
                    o.getAsJsonObject("northPole").get("distanceKm").getAsDouble(), o.getAsJsonObject("northPole").get("walkBlocks").getAsDouble(),
                    o.getAsJsonObject("southPole").get("distanceKm").getAsDouble(), o.getAsJsonObject("southPole").get("walkBlocks").getAsDouble()));
        }
        // Past the corners: off the planet, no walk.
        ChunkPos ocean = oceanChunk(cube);
        helper.assertTrue(ocean != null, "a chunk past the net's corners");
        JsonObject off = DeepTimeCommands.poleJson(level, ocean.getMiddleBlockX() + 0.5, ocean.getMiddleBlockZ() + 0.5);
        helper.assertTrue(off.get("borderZone").getAsInt() == 2 && Double.isNaN(off.get("latitude").getAsDouble())
                        && Double.isNaN(off.getAsJsonObject("northPole").get("walkBlocks").getAsDouble())
                        && Double.isNaN(off.getAsJsonObject("northPole").get("distanceKm").getAsDouble()),
                "probe pole past the corners: off the planet, no distance and no walk: " + off);

        // probe column: on a face, past each turning edge, past the corners.
        JsonObject centre = DeepTimeCommands.columnJson(level, 0, 0);
        JsonObject cf = centre.getAsJsonObject("fold");
        helper.assertTrue(centre.get("faceBlocks").getAsLong() == cube.side() && "face".equals(cf.get("kind").getAsString())
                        && "F0".equals(cf.get("face").getAsString())
                        && cube.fold(0, 0) instanceof CubeLayout.OnFace on && on.i() == cf.get("i").getAsLong() && on.j() == cf.get("j").getAsLong(),
                "probe column at F0's centre: " + centre);
        List<CubeLayout.NetEdge> edges = cube.edges();
        for (int k = 2; k < edges.size(); k++) {
            CubeLayout.NetEdge e = edges.get(k);
            long[] b = beyond(e, 64, (e.spanFrom() + e.spanTo()) / 2);
            JsonObject o = DeepTimeCommands.columnJson(level, (int) b[0], (int) b[1]);
            JsonObject f = o.getAsJsonObject("fold");
            double bx = b[0] + 0.5, bz = b[1] + 0.5;
            boolean folds = cube.fold(b[0], b[1]) instanceof CubeLayout.Outside out && "outside".equals(f.get("kind").getAsString())
                    && e.name().equals(f.get("edge").getAsString()) && out.x() == f.get("x").getAsLong() && out.z() == f.get("z").getAsLong()
                    && out.iso().k() == f.get("quarterTurns").getAsInt();
            double north = o.get("northBearing").getAsDouble(), m = o.get("metresPerBlock").getAsDouble();
            helper.assertTrue(folds && north == DeepTimeClimate.northBearingDeg(level, bx, bz) && m == DeepTimeClimate.metresPerBlock(level, bx, bz)
                            && !Double.isNaN(north) && m > 0.9 * mpb && m < 1.5 * mpb,
                    "probe column 64 past " + e.name() + " folds onto its partner, north and scale the API's: " + o);
            if (k == 2)
                log.append(" column(").append(e.name()).append(" +64: ").append(f).append(", north ")
                        .append(String.format(java.util.Locale.ROOT, "%.1f, %.1f m", north, m)).append(')');
        }
        JsonObject oc = DeepTimeCommands.columnJson(level, ocean.getMiddleBlockX(), ocean.getMiddleBlockZ());
        helper.assertTrue("ocean".equals(oc.getAsJsonObject("fold").get("kind").getAsString())
                        && Double.isNaN(oc.get("northBearing").getAsDouble()) && Double.isNaN(oc.get("metresPerBlock").getAsDouble()),
                "probe column past the corners: the ocean, no north, no scale: " + oc);

        // probe tw: the period and the face side.
        JsonObject tw = DeepTimeCommands.twJson(level);
        helper.assertTrue("cube".equals(tw.get("projection").getAsString()) && tw.get("periodBlocks").getAsLong() == cube.period()
                        && tw.get("faceBlocks").getAsLong() == cube.side()
                        && (!tw.has("widthIsPeriod") || tw.get("widthIsPeriod").getAsBoolean()),
                "probe tw: period 4s, face side s, Toroidal World's width the period: " + tw);
        log.append(" tw(").append(tw).append(')');

        // sites near F0's centre: the planet distance and the walk to each.
        JsonObject sites = SiteCommands.sitesNearJson(level, 0, 0, (int) (3 * h));
        int n = 0;
        for (var el : sites.getAsJsonArray("sites")) {
            JsonObject s = el.getAsJsonObject();
            double d = s.get("distance").getAsDouble(), walk = s.get("walk").getAsDouble(), km = s.get("distance_km").getAsDouble();
            helper.assertTrue(walk >= d - 0.11 && km >= 0 && km < halfKm, "sites near: the walk is no shorter than the net's distance: " + s);
            n++;
        }
        log.append(" sites(").append(n).append(" within ").append((int) (3 * h)).append(')');
        DeepTime.LOGGER.info("[gametest] cubeCommandProbes{}", log);
        helper.succeed();
    }

    /** One far-terrain sample: its group, where it is, and the block (x in Toroidal World's frame). */
    private record FarSample(String group, String where, int x, int z) {}

    /** 1 when far terrain shows dry ground at (x, z) above the build floor's band, else 0 (far terrain alone: no chunk). */
    private static int dry(FarTerrain far, int x, int z, int minY) {
        FarTerrain.Column c = far.column(x, z);
        return !c.water() && c.ground() >= minY + 32 ? 1 : 0;
    }

    /**
     * Far terrain on the cube (cube plan G6) against the generated chunks where the cube differs from
     * a polar world: beside each of the 12 turning edges on the face (the column rule's rasters fold
     * there) and just past it (the fill: the partner's columns turned, {@code CubeFill}), on both sides
     * of the slide (the net's F2 to F3 edge, x = −3s/2 in Toroidal World's frame) and of Toroidal
     * World's own seam inside F2 (x = ±2s, where Distant Horizons' areas wrap), and past the net's
     * corners (deep ocean). Each column is read as the column rule, the fill and the carvers left it,
     * before any feature, as {@code FarTerrainGameTests}' seam test reads them (no features are placed
     * off the faces yet, coordinator 2026-10-23), and each group must meet the planet-wide test's
     * properties ({@link FarTerrainGameTests#assertAgrees}) and the seam test's on land
     * ({@link FarTerrainGameTests#assertLand}; places with dry ground are searched by far terrain
     * alone, as that test's land rows are, since the 4k cube's first run of this test met 23 dry
     * columns of 288 by the edges and none at the two crossings). Also the sphere Distant Horizons
     * is told the planet curves as: the cube's nominal radius s √(1.5 / π), not its period over 2π.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void farTerrainMatchesTheCube(GameTestHelper helper) {
        ServerLevel level = level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = generator(helper, level);
        CubeLayout cube = g.cubeLayout();
        FarTerrain far = FarTerrain.of(level).orElse(null);
        helper.assertTrue(far != null, "test_planet_cube has far terrain");
        WorldInfoPayload info = WorldInfoPayload.forLevel(level);
        double r = info.worldProjection().radius();
        helper.assertTrue(Math.abs(r - cube.side() * Math.sqrt(1.5 / Math.PI)) < 1e-9,
                "the client's sphere is the cube's nominal one: R " + r + " for s " + cube.side());

        long s = cube.side(), h = s / 2;
        int sea = g.getSeaLevel(), minY = level.getMinBuildHeight();
        List<FarSample> samples = new ArrayList<>();
        // Beside and past every turning edge, at three places along it (clear of the other tests' chunks),
        // and at a fourth where far terrain shows the most dry ground by the edge, so the fold is
        // checked on land wherever the planet has any there (the 4k cube's edges are mostly sea).
        List<CubeLayout.NetEdge> edges = cube.edges();
        long[] inside = {-1, -6, -11, -16}, outside = {0, 5, 10, 15};
        List<String> landAt = new ArrayList<>();
        for (int k = 2; k < edges.size(); k++) {
            CubeLayout.NetEdge e = edges.get(k);
            long span = e.spanTo() - e.spanFrom();
            long[] fixed = {e.spanFrom() + span / 4 + 24, e.spanFrom() + span / 2 + 104, e.spanFrom() + 3 * span / 4 + 8};
            long best = Long.MIN_VALUE;
            int bestDry = 0;
            for (long along = e.spanFrom() + 72; along < e.spanTo() - 64; along += 32) {
                int dry = 0;
                for (long[] ds : new long[][]{inside, outside})
                    for (long d : ds) {
                        long[] b = beyond(e, d, along);
                        dry += dry(far, (int) CubeFill.twX(cube, b[0]), (int) b[1], minY);
                    }
                if (dry > bestDry) {
                    bestDry = dry;
                    best = along;
                }
            }
            long[] alongs = best == Long.MIN_VALUE ? fixed : new long[]{fixed[0], fixed[1], fixed[2], best};
            if (best != Long.MIN_VALUE)
                landAt.add(e.name() + "@" + best + ":" + bestDry);
            for (long along : alongs) {
                for (long d : inside) {
                    long[] b = beyond(e, d, along);
                    samples.add(new FarSample("beside the turning edges", e.name() + " " + d, (int) CubeFill.twX(cube, b[0]), (int) b[1]));
                }
                for (long d : outside) {
                    long[] b = beyond(e, d, along);
                    samples.add(new FarSample("past the turning edges", e.name() + " +" + d, (int) CubeFill.twX(cube, b[0]), (int) b[1]));
                }
            }
        }
        // The slide and Toroidal World's seam, on five rows of the band, then on up to three rows each
        // where far terrain shows the most dry ground across it: 64 blocks each side, a column every 4.
        long[] rows = {-h + 40, -h / 2 + 7, 9, h / 2 + 3, h - 37};
        long slide = -3 * h, seam = 2 * s;
        // {group, side label, x of the first column west, x of the first column east}
        Object[][] crossings = {{"the slide", "F2", "F3", slide - 1, slide}, {"Toroidal World's seam", "west", "east", seam - 1, -seam}};
        for (long z : rows)
            for (long d : outside)
                for (Object[] c : crossings) {
                    samples.add(new FarSample((String) c[0], c[1] + " " + d, (int) ((long) c[3] - d), (int) z));
                    samples.add(new FarSample((String) c[0], c[2] + " " + d, (int) ((long) c[4] + d), (int) z));
                }
        for (Object[] c : crossings) {
            List<long[]> byDry = new ArrayList<>();   // {z, dry}
            for (long z = -h + 9; z < h; z += 16) {
                int dry = 0;
                for (long d = 0; d < 64; d += 4)
                    dry += dry(far, (int) ((long) c[3] - d), (int) z, minY) + dry(far, (int) ((long) c[4] + d), (int) z, minY);
                if (dry > 0)
                    byDry.add(new long[]{z, dry});
            }
            byDry.sort((a, b) -> Long.compare(b[1], a[1]));
            for (long[] row : byDry.subList(0, Math.min(3, byDry.size()))) {
                landAt.add(c[0] + "@z" + row[0] + ":" + row[1]);
                for (long d = 0; d < 64; d += 4) {
                    samples.add(new FarSample((String) c[0], c[1] + " " + d + " land row", (int) ((long) c[3] - d), (int) row[0]));
                    samples.add(new FarSample((String) c[0], c[2] + " " + d + " land row", (int) ((long) c[4] + d), (int) row[0]));
                }
            }
        }
        // 24 chunks further out than the other cube tests' ocean chunk, which they take to FULL (beyond
        // N-top's corner, x in [h, 3h): the net's x and Toroidal World's agree there), and out of the
        // view of cubeBorderStopsAndFolds' player past the corners at (h + 64, -3h - 64): 4 chunks out,
        // a run that put that test first (-PwithMic) found the chunk taken to FULL and compared none.
        ChunkPos first = oceanChunk(cube);
        helper.assertTrue(first != null, "a chunk past the net's corners");
        ChunkPos ocean = new ChunkPos(first.x, first.z - 24);
        for (int lx = 0; lx < 16; lx += 15)
            for (int lz = 0; lz < 16; lz += 15)
                helper.assertTrue(cube.fold(ocean.getMinBlockX() + lx, ocean.getMinBlockZ() + lz) instanceof CubeLayout.Ocean,
                        "the sampled chunk " + ocean + " lies past the corners");
        for (int lx = 3; lx < 16; lx += 4)
            samples.add(new FarSample("past the corners", "ocean " + lx, ocean.getMinBlockX() + lx, ocean.getMiddleBlockZ()));

        Map<String, FarTerrainCheck.Tally> tallies = new LinkedHashMap<>();
        Map<String, int[]> farDry = new LinkedHashMap<>();   // dry columns far terrain shows among those compared (no carver's cut)
        for (FarSample p : samples) {
            tallies.computeIfAbsent(p.group(), k -> new FarTerrainCheck.Tally());
            farDry.computeIfAbsent(p.group(), k -> new int[1]);
        }
        int[] taken = {0}, logged = {0};
        ChunkSampling.forEach(helper, samples.size(), i -> {
            FarSample p = samples.get(i);
            FarTerrain.Column column = far.column(p.x(), p.z());
            if (column.ground() < minY + 32)
                return false;   // the floor band: the router's bottom slide, not the bake
            ChunkAccess chunk = level.getChunkSource().getChunk(p.x() >> 4, p.z() >> 4, ChunkStatus.CARVERS, true);
            if (chunk.getPersistedStatus() != ChunkStatus.CARVERS) {
                taken[0]++;   // something else took it on to its features: not counted
                return true;
            }
            FarTerrainCheck.RealColumn real = FarTerrainCheck.real(chunk, p.x(), p.z());
            FarTerrainCheck.Tally into = tallies.get(p.group());
            int carved = into.carved;
            into.add(column, real, chunk);
            if (!column.water() && into.carved == carved)
                farDry.get(p.group())[0]++;
            if ((Math.abs(column.ground() - real.ground()) > 2 || column.water() != real.water()
                    || column.groundBlock().getBlock() != real.groundBlock().getBlock() || !column.biome().equals(real.biome()))
                    && logged[0]++ < 40)
                DeepTime.LOGGER.info("[gametest] farTerrainMatchesTheCube diag {} {}: far biome {} {}", p.group(), p.where(),
                        column.biome().getRegisteredName(), FarTerrainGameTests.diag(chunk, column, real, p.x(), p.z()));
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] farTerrainMatchesTheCube {} samples, {} chunks taken further already, R {}; land searched {}; "
                            + "far dry {}: {}", samples.size(), taken[0], r, landAt,
                    farDry.entrySet().stream().map(e -> e.getKey() + "=" + e.getValue()[0]).toList(), tallies);
            for (String group : new String[]{"beside the turning edges", "past the turning edges", "the slide", "Toroidal World's seam"}) {
                FarTerrainGameTests.assertAgrees(helper, tallies.get(group), group.contains("turning") ? 96 : 24, group + ": ");
                FarTerrainGameTests.assertLand(helper, tallies.get(group), farDry.get(group)[0], group + ": ");
            }
            FarTerrainCheck.Tally o = tallies.get("past the corners");
            helper.assertTrue(o.columns == 4 && o.heightExact == 4 && o.waterTopExact == 4 && o.biomeSame == 4 && o.groundBlockSame == 4,
                    "past the corners far terrain is the fill's deep ocean: " + o.toJson());
            FarTerrain.Column c = far.column(ocean.getMinBlockX() + 7, ocean.getMinBlockZ() + 7);
            helper.assertTrue(c.water() && c.waterTop() == sea - 1 && c.ground() == sea - CubeFill.OCEAN_DEPTH - 1
                            && c.biome().is(Biomes.DEEP_OCEAN),
                    "the ocean column: ground " + c.ground() + ", water top " + c.waterTop() + ", " + c.biome().getRegisteredName());
            helper.succeed();
        });
    }
}
