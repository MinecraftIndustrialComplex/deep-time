// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.LavaFeatures;
import com.minecraftindustrialcomplex.deeptime.worldgen.LavaRules;
import com.minecraftindustrialcomplex.deeptime.worldgen.Magma;
import com.minecraftindustrialcomplex.deeptime.worldgen.SeaCliffGuard;
import com.minecraftindustrialcomplex.deeptime.worldgen.SeaCliffPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Where the generator puts lava (Ben's preview of 2026-10-01, "huge bodies of lava"): on
 * {@code deeptime:test_planet_caves}, the real test planet with the full router (noise caves and
 * aquifers, {@code deeptime:planet}) under a biome with no carvers and no features, so every fluid
 * block read back is the aquifer's or the global fluid picker's.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class LavaGameTests {

    private LavaGameTests() {}

    /** A sample site: a column and why it was chosen. */
    private record Site(String what, int x, int z, double h) {}

    /** Chunks per side generated around each site. */
    private static final int SIDE = 2;

    /**
     * Lava is measured from the world's floor, not from y 0 ({@link LavaRules}). Vanilla's two
     * hard-coded heights (lava below y −54, lava lakes at y −10 and lower) put lava in every cave
     * and every stretch of sea below y −54 of a world whose floor is y −352. Chunks are generated
     * around the planet's deepest sea, a sea about 150 blocks deep, a coast and high ground, and
     * every block is read back:
     * <ul>
     *   <li>{@code lavaAboveLakes}: no lava above the highest level a lava lake can have
     *       ({@link LavaRules#highestLakeY}, y −288 by default), the magma of active volcanoes
     *       ({@link Magma}, read back by {@link #magmaUnderActiveVolcanoes}) counted apart. The old
     *       rule put it up to y −55;</li>
     *   <li>{@code seaOnLava}: no sea column whose water stands on lava (the first block under the
     *       water is never lava). The old rule turned the sea below y −54 into lava;</li>
     *   <li>{@code floorNotLava}: under ground that stands clear of it, open space below the lava
     *       floor ({@link LavaRules#floodY}, y −342) is lava, as in vanilla's bottom ten blocks;</li>
     *   <li>the premises: a sea bed below y −60 was sampled ({@code deepSeaColumns}), and open space
     *       exists between the lava lakes and y −54 ({@code openDeep}: caves the old rule flooded).</li>
     * </ul>
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void lavaStaysAtTheFloor(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_CAVES.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_caves is not generating from world data");
            return;
        }
        var settings = g.generatorSettings().value();
        helper.assertTrue(settings.aquifersEnabled(), "test_planet_caves must run the full router (aquifers on)");
        int minY = settings.noiseSettings().minY(), sea = settings.seaLevel();
        LavaRules rules = t.lava();
        int flood = rules.floodY(minY, sea), lakeTop = Math.max(rules.highestLakeY(minY), flood - 1);
        // The magma under active volcanoes (chambers, conduits, the crust-deep column's lenses) is lava at
        // any depth by design and is read back by magmaUnderActiveVolcanoes; it is counted apart here.
        Magma magma = t.magma(g.waterReach(), minY, sea);
        helper.assertTrue(flood < -54 - 100, "the lava floor (y " + flood + ") must follow the world's floor (y " + minY + ")");

        // Sites from the world data: the deepest sea, a sea about 150 blocks deep, a coast, high ground.
        int c = t.circumference();
        Site deepest = null, deep = null, coast = null, high = null;
        int zp = (int) t.zHalf();   // the poles (Lambert) or the turning edges (Petroff–Guyou)
        for (int z = -zp + 64; z < zp - 64; z += 16)
            for (int x = -c / 2; x < c / 2; x += 16) {
                double h = t.height(x, z);
                if (deepest == null || h < deepest.h)
                    deepest = new Site("deepest sea", x, z, h);
                if (h < -60 && (deep == null || Math.abs(h + 90) < Math.abs(deep.h + 90)))
                    deep = new Site("deep sea", x, z, h);
                if (coast == null || Math.abs(h - sea) < Math.abs(coast.h - sea))
                    coast = new Site("coast", x, z, h);
                if (high == null || h > high.h)
                    high = new Site("high ground", x, z, h);
            }
        List<Site> sites = new ArrayList<>(List.of(deepest, coast, high));
        if (deep != null)
            sites.add(deep);
        Site deepestSite = deepest;

        long[] n = new long[9];   // lava, lavaAboveLakes, seaOnLava, floorOpen, floorNotLava, openDeep, deepSeaColumns, columns, magmaLava
        int[] topLava = {Integer.MIN_VALUE};
        StringBuilder detail = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        ChunkSampling.forEach(helper, sites.size() * SIDE * SIDE, i -> {
            Site s = sites.get(i / (SIDE * SIDE));
            int k = i % (SIDE * SIDE);
            int cx = (s.x >> 4) + k % SIDE, cz = (s.z >> 4) + k / SIDE;
            ChunkAccess chunk = level.getChunkSource().getChunk(cx, cz, ChunkStatus.NOISE, true);
            for (int dx = 0; dx < 16; dx++)
                for (int dz = 0; dz < 16; dz++) {
                    int x = (cx << 4) + dx, z = (cz << 4) + dz;
                    double h = t.height(x, z);
                    n[7]++;
                    if (h < -60)
                        n[6]++;
                    boolean clear = rules.columnFloodY(minY, sea, h) == flood && h > flood + 32;
                    boolean inSea = false;   // walking down through the sea's open water
                    for (int y = sea - 1; y >= minY; y--) {
                        BlockState b = chunk.getBlockState(pos.set(x, y, z));
                        boolean lava = b.is(Blocks.LAVA), water = b.is(Blocks.WATER);
                        if (y == sea - 1)
                            inSea = water && h < sea;
                        else if (inSea && !water) {
                            if (lava) {
                                n[2]++;
                                if (detail.length() < 400)
                                    detail.append(" seaOnLava(").append(x).append(',').append(y).append(',').append(z).append(')');
                            }
                            inSea = false;
                        }
                        if (lava && y > lakeTop && magma.at(x, y, z).molten()) {
                            n[8]++;
                        } else if (lava) {
                            n[0]++;
                            topLava[0] = Math.max(topLava[0], y);
                            if (y > lakeTop) {
                                n[1]++;
                                if (detail.length() < 400)
                                    detail.append(" lava(").append(x).append(',').append(y).append(',').append(z).append(')');
                            }
                        }
                        boolean open = b.isAir() || !b.getFluidState().isEmpty();
                        if (open && y < -54 && y > lakeTop && !inSea)
                            n[5]++;
                        if (open && y < flood && clear) {
                            n[3]++;
                            if (!lava)
                                n[4]++;
                        }
                    }
                }
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] lavaStaysAtTheFloor rules={} minY={} lavaFloorY={} highestLakeY={} sites={} columns={} "
                            + "deepSeaColumns={} (deepest bed y {}) lava={} topLavaY={} lavaAboveLakes={} seaOnLava={} openDeep={} "
                            + "floorOpen={} floorNotLava={} magmaLava={}{}",
                    rules, minY, flood, lakeTop, sites, n[7], n[6], (int) Math.floor(deepestSite.h), n[0],
                    topLava[0] == Integer.MIN_VALUE ? "-" : topLava[0], n[1], n[2], n[5], n[3], n[4], n[8], detail);
            helper.assertTrue(n[6] > 0, "no sea bed below y -60 was sampled (deepest: y " + deepestSite.h
                    + "): the planet has no sea the old lava level would have reached");
            helper.assertTrue(n[5] > 0, "no open space between y " + lakeTop + " and y -54 in the sampled chunks: nothing the old lava level would have flooded");
            helper.assertTrue(n[1] == 0, n[1] + " lava blocks above y " + lakeTop + " (the highest a lava lake may stand):" + detail);
            helper.assertTrue(n[2] == 0, n[2] + " sea columns stand on lava:" + detail);
            helper.assertTrue(n[4] == 0, n[4] + " of " + n[3] + " open blocks below the lava floor (y " + flood + ") are not lava");
            helper.succeed();
        });
    }

    /** What was read back around one body. */
    private static final class Tally {
        long expected, lava, stray, wetFaces, airFaces, skippedFaces, lensExpected, lensLava, mushBlocks, mushLava, mushOpen;
        final StringBuilder detail = new StringBuilder();
    }

    private static final int[][] N6 = {{1, 0, 0}, {-1, 0, 0}, {0, 0, 1}, {0, 0, -1}, {0, 1, 0}, {0, -1, 0}};

    /** One chunk to read for one body. */
    private record Job(Magma.Body body, int cx, int cz) {}

    /**
     * Lava is where the simulation has it (Ben, decisions log 2026-10-01, "Volcanoes is a go ahead"):
     * under and in the planet's active volcanoes ({@link Magma}, {@code MagmaPass}), on the level
     * with caves and aquifers. Up to four bodies are generated to the surface step, whichever the
     * planet has of: one with a crater lake, one under water, one on land with a plug, and the
     * largest. Every block of their boxes is read back:
     * <ul>
     *   <li>{@code lava}: the chamber, conduit and lake hold lava (at least half of the blocks
     *       {@link Magma} calls molten; the rest is the cover kept under low ground);</li>
     *   <li>{@code stray}: no lava above the lava-lake zone that is not part of a body;</li>
     *   <li>{@code wetFaces}, {@code airFaces}: no lava block of a body touches water, and none
     *       touches air except a lake's surface: the bodies are sealed, in a sea, a lake, a river
     *       or a cave alike;</li>
     *   <li>a body with a crater lake has lava at the lake's surface under open air.</li>
     * </ul>
     * A planet with no active volcano passes with a line saying so: the geometry itself is
     * {@code MagmaTest}'s, on cones built in closed form.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void magmaUnderActiveVolcanoes(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_CAVES.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_caves is not generating from world data");
            return;
        }
        var settings = g.generatorSettings().value();
        int minY = settings.noiseSettings().minY(), sea = settings.seaLevel();
        Magma magma = t.magma(g.waterReach(), minY, sea);
        int lakeZoneTop = Math.max(t.lava().highestLakeY(minY), t.lava().floodY(minY, sea));
        List<Magma.Body> all = magma.bodies().stream().filter(b -> b.chamberRh() > 0).toList();
        long lakes = all.stream().filter(Magma.Body::lake).count(), submerged = all.stream().filter(Magma.Body::submerged).count();
        if (all.isEmpty()) {
            DeepTime.LOGGER.info("[gametest] magmaUnderActiveVolcanoes activeVolcanoes={} withMagma=0: this planet has no active "
                    + "volcano with room for a chamber; nothing to read back", t.data().volcanoes().list().size());
            helper.succeed();
            return;
        }
        List<Magma.Body> picked = new ArrayList<>();
        all.stream().filter(Magma.Body::lake).findFirst().ifPresent(picked::add);
        all.stream().filter(Magma.Body::submerged).findFirst().ifPresent(picked::add);
        all.stream().filter(b -> !b.lake() && !b.submerged()).findFirst().ifPresent(picked::add);
        all.stream().filter(b -> "shield".equals(b.kind()) && !picked.contains(b)).findFirst().ifPresent(picked::add);
        all.stream().max(java.util.Comparator.comparingInt(Magma.Body::reach)).filter(b -> !picked.contains(b)).ifPresent(picked::add);
        List<Job> jobs = new ArrayList<>();
        java.util.Map<Magma.Body, Tally> tallies = new java.util.LinkedHashMap<>();
        for (Magma.Body b : picked) {
            tallies.put(b, new Tally());
            for (int cx = (b.x() - b.reach() - 1) >> 4; cx <= (b.x() + b.reach() + 1) >> 4; cx++)
                for (int cz = (b.z() - b.reach() - 1) >> 4; cz <= (b.z() + b.reach() + 1) >> 4; cz++)
                    jobs.add(new Job(b, cx, cz));
        }
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int lo0 = level.getMinBuildHeight(), hi0 = level.getMaxBuildHeight() - 1;
        ChunkSampling.forEach(helper, jobs.size(), i -> {
            Job job = jobs.get(i);
            Magma.Body b = job.body();
            Tally n = tallies.get(b);
            ChunkAccess chunk = level.getChunkSource().getChunk(job.cx(), job.cz(), ChunkStatus.SURFACE, true);
            int x0 = job.cx() << 4, z0 = job.cz() << 4;
            int lo = Math.max(lo0, b.minY() - 1), hi = Math.min(hi0, b.maxY() + 1);
            for (int lx = 0; lx < 16; lx++)
                for (int lz = 0; lz < 16; lz++) {
                    int x = x0 + lx, z = z0 + lz, dx = magma.dx(x, b), dz = z - b.z();
                    if (Math.abs(dx) > b.reach() + 1 || Math.abs(dz) > b.reach() + 1)
                        continue;
                    for (int y = lo; y <= hi; y++) {
                        Magma.Part part = b.at(dx, dz, y);
                        boolean lava = chunk.getBlockState(pos.set(x, y, z)).is(Blocks.LAVA);
                        if (part.molten())
                            n.expected++;
                        if (part == Magma.Part.LENS)
                            n.lensExpected++;
                        if (part == Magma.Part.NONE && b.mush(dx, dz, y)) {
                            BlockState ms = chunk.getBlockState(pos);
                            if (lava && y > lakeZoneTop && !magma.at(x, y, z).molten())
                                n.mushLava++;       // lava in the altered rock outside a lens: stray
                            else if (ms.isAir() || !ms.getFluidState().isEmpty())
                                n.mushOpen++;       // the mush leaves no cave or water inside it
                            else
                                n.mushBlocks++;
                        }
                        if (part == Magma.Part.LENS && lava)
                            n.lensLava++;
                        if (!lava)
                            continue;
                        if (!part.molten()) {
                            if (y > lakeZoneTop && !magma.at(x, y, z).molten()) {
                                n.stray++;
                                if (n.detail.length() < 300)
                                    n.detail.append(" stray(").append(x).append(',').append(y).append(',').append(z).append(')');
                            }
                            continue;
                        }
                        n.lava++;
                        for (int[] d : N6) {
                            int nx = lx + d[0], ny = y + d[1], nz = lz + d[2];
                            if (nx < 0 || nx > 15 || nz < 0 || nz > 15 || ny < lo0 || ny > hi0) {
                                n.skippedFaces++;
                                continue;
                            }
                            BlockState s = chunk.getBlockState(pos.set(x0 + nx, ny, z0 + nz));
                            if (!s.getFluidState().isEmpty() && !s.is(Blocks.LAVA)) {
                                n.wetFaces++;
                                if (n.detail.length() < 300)
                                    n.detail.append(" wet(").append(x).append(',').append(y).append(',').append(z).append(')');
                            } else if (s.isAir() && !(part == Magma.Part.LAKE && d[1] == 1)) {
                                n.airFaces++;
                                if (n.detail.length() < 600) {
                                    int ax = x0 + nx, az = z0 + nz;
                                    n.detail.append(" air(").append(x).append(',').append(y).append(',').append(z).append(" ").append(part)
                                            .append(" by ").append(b.at(magma.dx(ax, b), az - b.z(), ny)).append(b.mush(magma.dx(ax, b), az - b.z(), ny) ? "/mush" : "")
                                            .append(' ').append(chunk.getPersistedStatus()).append(')');
                                }
                            }
                        }
                    }
                }
            return true;
        }, () -> {
            StringBuilder line = new StringBuilder();
            String failure = null;
            for (var e : tallies.entrySet()) {
                Magma.Body b = e.getKey();
                Tally n = e.getValue();
                String lake = "-";
                if (b.lake()) {
                    ChunkAccess c = level.getChunkSource().getChunk(b.x() >> 4, b.z() >> 4, ChunkStatus.SURFACE, true);
                    boolean top = c.getBlockState(pos.set(b.x(), b.lakeTop(), b.z())).is(Blocks.LAVA);
                    boolean sky = true;
                    for (int y = b.lakeTop() + 1; y <= Math.min(hi0, b.lakeTop() + Magma.LAKE_CLEAR); y++)
                        sky &= c.getBlockState(pos.set(b.x(), y, b.z())).isAir();
                    lake = "y" + b.lakeTop() + (top ? " lava" : " NOT LAVA") + (sky ? " open" : " COVERED");
                    if (!top || !sky)
                        failure = "volcano " + b.id() + ": its crater lake is " + lake;
                }
                line.append(String.format(" [id=%d %s at (%d, %d) floorY=%d chamber y=%d rh=%d rv=%d %s lake=%s molten=%d lava=%d stray=%d "
                                + "wetFaces=%d airFaces=%d lenses=%d lensMolten=%d lensLava=%d mush=%d mushOpen=%d mushLava=%d column y %d..%d%s]",
                        b.id(), b.kind(), b.x(), b.z(), b.floorY(), b.chamberY(), b.chamberRh(),
                        b.chamberRv(), b.submerged() ? "under water" : "on land", lake, n.expected, n.lava, n.stray, n.wetFaces,
                        n.airFaces, b.lensCount(), n.lensExpected, n.lensLava, n.mushBlocks, n.mushOpen, n.mushLava, b.minY(), b.maxY(),
                        n.detail));
                if (failure == null && b.lensCount() > 0 && n.lensExpected > 0 && n.lensLava * 2 < n.lensExpected)
                    failure = "volcano " + b.id() + ": only " + n.lensLava + " of " + n.lensExpected + " lens blocks are lava";
                if (failure == null && b.lensCount() > 0 && n.mushBlocks == 0 && b.mush(0, 0, b.chamberY() - b.chamberRv() - 10))
                    failure = "volcano " + b.id() + ": no altered hot rock was placed around its column";
                if (failure == null && n.mushLava > 0)
                    failure = "volcano " + b.id() + ": " + n.mushLava + " lava blocks in its altered rock outside any lens";
                if (failure == null && n.lava * 2 < n.expected)
                    failure = "volcano " + b.id() + ": only " + n.lava + " of " + n.expected + " molten blocks are lava";
                if (failure == null && n.stray > 0)
                    failure = "volcano " + b.id() + ": " + n.stray + " lava blocks outside its body above y " + lakeZoneTop + ":" + n.detail;
                if (failure == null && n.wetFaces + n.airFaces > 0)
                    failure = "volcano " + b.id() + ": lava touches water on " + n.wetFaces + " faces and air on " + n.airFaces + ":" + n.detail;
            }
            DeepTime.LOGGER.info("[gametest] magmaUnderActiveVolcanoes rules={} activeVolcanoes={} withMagma={} craterLakes={} "
                            + "underWater={} read={} chunks={}{}", t.magmaRules(), t.data().volcanoes().list().size(), all.size(), lakes,
                    submerged, picked.size(), jobs.size(), line);
            if (failure != null)
                helper.fail(failure);
            helper.succeed();
        });
    }

    /**
     * No cave of any kind reaches the underwater face of a sea cliff ({@link SeaCliffGuard}, Ben's
     * report of 2026-10-02: arches cut into the underwater base of sea cliffs). On the level with
     * noise caves and aquifers, the planet is scanned for chunks where the ground drops by the guard's
     * skin or more within the skin, beside water under the sea level; up to six, far apart, are
     * generated through the carving step and read back: in every guarded block under the ground's top
     * block there is no air and no water (lava is the lava floor's and the volcanoes'). A planet with
     * no such cliff passes with a line saying so.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void seaCliffsHaveNoCaveMouths(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_CAVES.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_caves is not generating from world data");
            return;
        }
        int sea = g.generatorSettings().value().seaLevel();
        int skin = t.cliffRules().skin();
        if (skin <= 0) {
            helper.succeed();
            return;
        }
        int minY = level.getMinBuildHeight() + 1, c = t.circumference();
        // Pre-screen on a coarse grid: a column that stands skin blocks above a neighbour skin columns away
        // that lies under the sea.
        List<int[]> chunks = new ArrayList<>();
        int[][] ring = {{skin, 0}, {-skin, 0}, {0, skin}, {0, -skin}, {skin, skin}, {-skin, -skin}, {skin, -skin}, {-skin, skin}};
        int pole = (int) t.zHalf();   // the poles (Lambert) or the turning edges (Petroff–Guyou)
        for (int z = -pole; z < pole && chunks.size() < 6; z += 16)
            for (int x = -c / 2; x < c / 2 && chunks.size() < 6; x += 16) {
                int own = (int) Math.ceil(t.height(x, z)) - 1;
                int low = own;
                for (int[] d : ring)
                    low = Math.min(low, (int) Math.ceil(t.height(x + d[0], z + d[1])) - 1);
                if (own - low < skin || low >= sea - 2)
                    continue;
                int cx = x >> 4, cz = z >> 4;
                boolean far = true;
                for (int[] o : chunks)
                    far &= Math.abs(o[0] - cx) + Math.abs(o[1] - cz) > 8;
                if (far)
                    chunks.add(new int[]{cx, cz});
            }
        if (chunks.isEmpty()) {
            DeepTime.LOGGER.info("[gametest] seaCliffsHaveNoCaveMouths skin={} no sea cliff on this planet; nothing to read back", skin);
            helper.succeed();
            return;
        }
        long[] n = {0, 0, 0};   // guarded blocks read, open ones, chunks read
        StringBuilder detail = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        ChunkSampling.forEach(helper, chunks.size(), i -> {
            int cx = chunks.get(i)[0], cz = chunks.get(i)[1];
            ChunkAccess chunk = level.getChunkSource().getChunk(cx, cz, ChunkStatus.CARVERS, true);
            int[][] zone = SeaCliffGuard.zone(t::height, cx << 4, cz << 4, sea, minY, skin);
            n[2]++;
            for (int lz = 0; lz < 16; lz++)
                for (int lx = 0; lx < 16; lx++) {
                    int[] r = zone[lz * 16 + lx];
                    if (r == null)
                        continue;
                    int own = (int) Math.ceil(t.height((cx << 4) + lx, (cz << 4) + lz)) - 1;
                    for (int y = r[0]; y <= Math.min(r[1], own - 1); y++) {
                        BlockState b = chunk.getBlockState(pos.set((cx << 4) + lx, y, (cz << 4) + lz));
                        n[0]++;
                        if (b.isAir() || (!b.getFluidState().isEmpty() && !b.is(Blocks.LAVA))) {
                            n[1]++;
                            if (detail.length() < 300)
                                detail.append(" open(").append(pos.getX()).append(',').append(y).append(',').append(pos.getZ()).append(')');
                        }
                    }
                }
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] seaCliffsHaveNoCaveMouths skin={} chunks={} guardedBlocks={} open={} filledSoFar={}{}", skin, n[2],
                    n[0], n[1], SeaCliffPass.filledTotal(), detail);
            if (n[1] > 0)
                helper.fail(n[1] + " of " + n[0] + " guarded blocks under sea cliffs are open (air or water):" + detail);
            helper.succeed();
        });
    }

    /**
     * The biomes' random lava stays out of a Deep Time level ({@link LavaFeatures},
     * {@code underground.remove_lava_features}): vanilla's lava lakes and lava springs are refused
     * for the level's generator and placed as ever for any other; water springs, ores and trees are
     * not touched. Every placed feature the server knows is asked, and the ones refused are logged:
     * with a pack, that line is the list of what it would have scattered.
     */
    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void lavaFeaturesStayOut(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null || !(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)) {
            helper.fail("test_planet_caves is not a Deep Time level");
            return;
        }
        helper.assertTrue(g.removesLavaFeatures(), "the test planet keeps the default (remove_lava_features on)");
        var registry = level.registryAccess().registryOrThrow(net.minecraft.core.registries.Registries.PLACED_FEATURE);
        var here = new net.minecraft.world.level.levelgen.placement.PlacementContext(level, g, java.util.Optional.empty());
        ServerLevel overworld = helper.getLevel().getServer().overworld();
        var elsewhere = new net.minecraft.world.level.levelgen.placement.PlacementContext(overworld,
                overworld.getChunkSource().getGenerator(), java.util.Optional.empty());
        List<String> refused = new ArrayList<>();
        registry.entrySet().forEach(e -> {
            if (LavaFeatures.removed(e.getValue(), here))
                refused.add(e.getKey().location().toString());
        });
        java.util.Collections.sort(refused);
        DeepTime.LOGGER.info("[gametest] lavaFeaturesStayOut placedFeatures={} refused={} {}", registry.size(), refused.size(), refused);
        for (String id : List.of("minecraft:lake_lava_surface", "minecraft:lake_lava_underground", "minecraft:spring_lava",
                "minecraft:spring_lava_frozen")) {
            helper.assertTrue(refused.contains(id), id + " is still placed in a Deep Time level");
            var f = registry.get(net.minecraft.resources.ResourceLocation.parse(id));
            helper.assertTrue(f != null && !LavaFeatures.removed(f, elsewhere), id + " is refused in a level that is not Deep Time's");
        }
        for (String id : List.of("minecraft:spring_water", "minecraft:ore_dirt", "minecraft:trees_plains", "minecraft:freeze_top_layer"))
            helper.assertTrue(registry.containsKey(net.minecraft.resources.ResourceLocation.parse(id)) && !refused.contains(id),
                    id + " is refused, but places no lava");
        if (net.neoforged.fml.ModList.get().isLoaded("terralith"))
            for (String id : List.of("terralith:volcano/lava_lakes", "terralith:volcano/spring_lava", "terralith:cave/mantle/lava_drip",
                    "terralith:cave/frostfire/lava_remover"))
                helper.assertTrue(refused.contains(id), id + " is still placed in a Deep Time level");
        helper.succeed();
    }
}
