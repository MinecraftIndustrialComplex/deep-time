// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.Geothermal;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeothermalPalette;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeothermalPalette.Role;
import com.minecraftindustrialcomplex.deeptime.worldgen.Magma;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * Hot springs, terraces and fumaroles around the planet's active volcanoes (P2-14), on
 * {@code deeptime:test_planet_caves}: the real test planet with the full router, under a biome with
 * no carvers and no features, so every block read back at the surface step is the pass's own.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class GeothermalGameTests {

    private GeothermalGameTests() {}

    /** What was read back around one site. */
    private static final class Tally {
        long poolBlocks, poolWater, rimColumns, rimSolid, bedColumns, bedSolid, uncut, uncutOpen, leaks, wetNextToMagma, lavaNear,
                groundColumns, groundRight, ventColumns, ventRight, skipped;
        final StringBuilder detail = new StringBuilder();

        void note(String what, int x, int y, int z) {
            if (detail.length() < 300)
                detail.append(' ').append(what).append('(').append(x).append(',').append(y).append(',').append(z).append(')');
        }
    }

    private record Job(Geothermal.Site site, int cx, int cz) {}

    private static boolean solid(BlockState s) {
        return !s.isAir() && s.getFluidState().isEmpty();
    }

    /**
     * Hot springs where groundwater is and fumaroles where it is not (the setting decides), read back
     * block by block for up to four of the planet's sites: a pool, a vent, a patch of altered ground,
     * and the widest. Properties asserted:
     * <ul>
     *   <li>{@code poolWater}: the pool's columns hold water from the site's level down, over a bed
     *       that is not air or water; the ground above the level is cut away;</li>
     *   <li>{@code leaks}: no water block has air beside it (every side is water or a block): the rim
     *       holds; {@code lavaNear}: no lava in a site's pool, rim or vent columns;</li>
     *   <li>{@code ground}, {@code vents}: the top ground block of a terrace, altered-ground and sulfur
     *       column is the palette's block for it (columns with a fluid or magma on top are left alone),
     *       and a vent's heart is the vent block.</li>
     * </ul>
     * The test planet's preset puts fields on its hot continental ground (heat flow of 50 mW/m2 or
     * more), so it always has sites: a planet with none fails (Ben's rule, decisions log 2026-10-01: a
     * test that reads nothing is not a pass). Whether its active volcanoes have sites of their own
     * depends on dry land beside them, so their count is logged, not asserted.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void geothermalAroundActiveVolcanoes(GameTestHelper helper) {
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
        Geothermal geo = t.geothermal(g.waterReach(), minY, sea);
        Magma magma = t.magma(g.waterReach(), minY, sea);
        int[] counts = geo.counts();
        DeepTime.LOGGER.info("[gametest] geothermalAroundActiveVolcanoes filters: activeVolcanoes={} sites={} ofVolcanoes={} "
                        + "fromHeatFlow={} pools={} vents={} patches={}", t.data().volcanoes().list().size(), geo.sites().size(),
                geo.sites().size() - geo.heatFlowSites(), geo.heatFlowSites(), counts[0], counts[1], counts[2]);
        if (geo.sites().isEmpty()) {
            helper.fail("the test planet has no geothermal site (" + t.geothermalRules() + ", " + t.data().volcanoes().list().size()
                    + " active volcanoes): its preset's heat-flow fields should always place some");
            return;
        }
        List<Geothermal.Site> picked = new ArrayList<>();
        for (Geothermal.Kind k : Geothermal.Kind.values())
            geo.sites().stream().filter(s -> s.kind() == k).findFirst().ifPresent(picked::add);
        geo.sites().stream().max(java.util.Comparator.comparingInt(Geothermal.Site::reach)).filter(s -> !picked.contains(s))
                .ifPresent(picked::add);
        // The test planet's preset lowers the heat-flow threshold to 50 (every continental cell; GEO-061), so
        // the fields of hot ground are on it: read one back as well (2026-10-02, "Export heat flow").
        helper.assertTrue(geo.heatFlowSites() > 0, "the test planet has no geothermal site from the baked heat flow ("
                + t.geothermalRules() + ", " + geo.sites().size() + " sites): the layer meta/heat_flow_mw_m2 or the setting is missing");
        geo.sites().stream().filter(s -> s.volcano() == Geothermal.HEAT_FLOW_SOURCE).findFirst().filter(s -> !picked.contains(s))
                .ifPresent(picked::add);
        GeothermalPalette palette = GeothermalPalette.get();
        Set<BlockState> terrace = new java.util.HashSet<>(palette.blocks(Role.TRAVERTINE));
        terrace.addAll(palette.blocks(Role.SINTER));
        List<Job> jobs = new ArrayList<>();
        Map<Geothermal.Site, Tally> tallies = new LinkedHashMap<>();
        for (Geothermal.Site s : picked) {
            tallies.put(s, new Tally());
            for (int cx = (s.x() - s.reach() - 1) >> 4; cx <= (s.x() + s.reach() + 1) >> 4; cx++)
                for (int cz = (s.z() - s.reach() - 1) >> 4; cz <= (s.z() + s.reach() + 1) >> 4; cz++)
                    jobs.add(new Job(s, cx, cz));
        }
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int lo0 = level.getMinBuildHeight(), hi0 = level.getMaxBuildHeight() - 1;
        ChunkSampling.forEach(helper, jobs.size(), i -> {
            Job job = jobs.get(i);
            Geothermal.Site s = job.site();
            Tally n = tallies.get(s);
            ChunkAccess chunk = level.getChunkSource().getChunk(job.cx(), job.cz(), ChunkStatus.SURFACE, true);
            int x0 = job.cx() << 4, z0 = job.cz() << 4;
            for (int lx = 0; lx < 16; lx++)
                for (int lz = 0; lz < 16; lz++) {
                    int x = x0 + lx, z = z0 + lz;
                    Geothermal.Site claim = geo.claim(x, z);
                    if (claim != s)
                        continue;   // another site's column, or none
                    int dx = geo.dx(x, s), dz = z - s.z();
                    Geothermal.Part part = s.part(dx, dz);
                    int top = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, lx, lz);
                    int surface = chunk.getHeight(Heightmap.Types.WORLD_SURFACE_WG, lx, lz);
                    if (part.pool()) {
                        int depth = part.depth();
                        for (int y = s.level(); y > s.level() - depth; y--) {
                            n.poolBlocks++;
                            BlockState b = chunk.getBlockState(pos.set(x, y, z));
                            if (b.is(Blocks.WATER)) {
                                n.poolWater++;
                                for (int[] d : new int[][]{{1, 0}, {-1, 0}, {0, 1}, {0, -1}}) {
                                    int nx = lx + d[0], nz = lz + d[1];
                                    if (nx < 0 || nx > 15 || nz < 0 || nz > 15) {
                                        n.skipped++;
                                        continue;
                                    }
                                    BlockState nb = chunk.getBlockState(pos.set(x0 + nx, y, z0 + nz));
                                    if (nb.isAir()) {
                                        n.leaks++;
                                        n.note("leak", x, y, z);
                                    }
                                }
                                if (chunk.getBlockState(pos.set(x, y + 1, z)).is(Blocks.LAVA) || chunk.getBlockState(pos.set(x, y - 1, z)).is(Blocks.LAVA))
                                    n.lavaNear++;
                            }
                        }
                        n.bedColumns++;
                        BlockState bed = chunk.getBlockState(pos.set(x, s.level() - depth, z));
                        if (solid(bed))
                            n.bedSolid++;
                        // The ground above the level is cut away.
                        n.uncut++;
                        if (!chunk.getBlockState(pos.set(x, s.level() + 1, z)).isAir()) {
                            n.uncutOpen++;
                            n.note("uncut", x, s.level() + 1, z);
                        }
                    } else if (part == Geothermal.Part.RIM) {
                        n.rimColumns++;
                        if (solid(chunk.getBlockState(pos.set(x, s.level(), z))) && solid(chunk.getBlockState(pos.set(x, s.level() - 1, z))))
                            n.rimSolid++;
                        else
                            n.note("rim", x, s.level(), z);
                    } else if (part != Geothermal.Part.NONE && surface <= top && top >= lo0 && top <= hi0) {
                        BlockState b = chunk.getBlockState(pos.set(x, top, z));
                        List<BlockState> want = switch (part) {
                            case VENT -> palette.blocks(Role.VENT);
                            case SULFUR -> palette.blocks(Role.SULFUR);
                            case STAIN, APRON_STAIN -> palette.blocks(Role.STAIN);
                            case CINNABAR -> palette.blocks(Role.CINNABAR);
                            case ALTERED -> palette.blocks(Role.ALTERED);
                            default -> List.of();
                        };
                        boolean ok = part == Geothermal.Part.APRON ? terrace.contains(b) : want.contains(b);
                        n.groundColumns++;
                        if (ok)
                            n.groundRight++;
                        if (part == Geothermal.Part.VENT) {
                            n.ventColumns++;
                            if (ok)
                                n.ventRight++;
                            if (chunk.getBlockState(pos.set(x, top + 1, z)).is(Blocks.WATER))
                                n.wetNextToMagma++;
                        }
                    }
                    if (part.pool() || part == Geothermal.Part.RIM || part == Geothermal.Part.VENT) {
                        for (int y = s.groundY() - 8; y <= s.groundY() + 8; y++)
                            if (y >= lo0 && y <= hi0 && chunk.getBlockState(pos.set(x, y, z)).is(Blocks.LAVA)) {
                                n.lavaNear++;
                                n.note("lava", x, y, z);
                            }
                    }
                }
            return true;
        }, () -> {
            StringBuilder line = new StringBuilder();
            String failure = null;
            for (var e : tallies.entrySet()) {
                Geothermal.Site s = e.getKey();
                Tally n = e.getValue();
                line.append(String.format(" [id=%d %s of volcano %d at (%d, %d) r=%d apron=%d level=%d %s poolWater=%d/%d bed=%d/%d rim=%d/%d "
                                + "uncutOpen=%d/%d leaks=%d lavaNear=%d ground=%d/%d vents=%d/%d wetNextToVent=%d skippedFaces=%d%s]", s.id(),
                        s.kind().name().toLowerCase(), s.volcano(), s.x(), s.z(), s.r(), s.apron(), s.level(),
                        s.kind() == Geothermal.Kind.POOL ? (s.sinter() ? "sinter" : "travertine") : "-", n.poolWater, n.poolBlocks,
                        n.bedSolid, n.bedColumns, n.rimSolid, n.rimColumns, n.uncutOpen, n.uncut, n.leaks, n.lavaNear, n.groundRight,
                        n.groundColumns, n.ventRight, n.ventColumns, n.wetNextToMagma, n.skipped, n.detail));
                if (failure == null && n.poolWater < n.poolBlocks)
                    failure = "site " + s.id() + ": only " + n.poolWater + " of " + n.poolBlocks + " pool blocks are water" + n.detail;
                if (failure == null && n.bedSolid < n.bedColumns)
                    failure = "site " + s.id() + ": " + (n.bedColumns - n.bedSolid) + " pool columns have no bed" + n.detail;
                if (failure == null && n.rimSolid < n.rimColumns)
                    failure = "site " + s.id() + ": " + (n.rimColumns - n.rimSolid) + " rim columns are not solid" + n.detail;
                if (failure == null && n.leaks > 0)
                    failure = "site " + s.id() + ": water has air beside it on " + n.leaks + " faces" + n.detail;
                if (failure == null && n.uncutOpen > 0)
                    failure = "site " + s.id() + ": " + n.uncutOpen + " pool columns still have ground over the water" + n.detail;
                if (failure == null && n.lavaNear > 0)
                    failure = "site " + s.id() + ": lava in or beside the site: " + n.lavaNear + n.detail;
                if (failure == null && n.wetNextToMagma > 0)
                    failure = "site " + s.id() + ": water over a vent's magma block" + n.detail;
                if (failure == null && n.groundColumns > 0 && n.groundRight * 10 < n.groundColumns * 9)
                    failure = "site " + s.id() + ": only " + n.groundRight + " of " + n.groundColumns + " ground columns have their block";
            }
            DeepTime.LOGGER.info("[gametest] geothermalAroundActiveVolcanoes rules={} activeVolcanoes={} sites={} heatFlowSites={} pools={} vents={} patches={} "
                            + "read={} chunks={} palette={}{}", t.geothermalRules(), t.data().volcanoes().list().size(), geo.sites().size(),
                    geo.heatFlowSites(), counts[0], counts[1], counts[2], picked.size(), jobs.size(), palette.toJson(), line);
            if (failure != null)
                helper.fail(failure);
            helper.succeed();
        });
    }
}
