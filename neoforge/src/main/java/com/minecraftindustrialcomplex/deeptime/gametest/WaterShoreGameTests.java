// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterCaveShield;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * The shores of rivers, lakes and the shallow sea with the noise caves on ({@code
 * deeptime:test_planet_caves}: the full router, the real planet's world data, the bare biome, no
 * carvers or features). Ben, 2026-10-20: "bodies of water just have these dirt walls around them,
 * extremely unprofessional". The router's near-surface cave entrances opened pits to the sky beside
 * the water, the aquifer left them dry, and the water pass built every bank and bed up from the pits'
 * floors in dirt: walls of dirt 10 to 27 blocks tall lining the channels. Every other water test
 * runs without caves ({@code planet_nocaves}), so none saw it.
 *
 * <p>Over boxes on the planet's rivers, lake mouths and outlets and shallow coasts, for every column
 * within {@link WaterCaveShield#FULL_BLOCKS} − 1 of a river, lake or shallow-sea column:
 * <ul>
 *   <li>{@code pits}: the generated ground lies more than {@value #PIT_TOLERANCE} blocks under
 *       the rule's ({@link Water}'s ground, a bank's raised or lowered ground, a channel's bed), or
 *       its top block hangs over air within {@value #PIT_TOLERANCE} blocks (a sea floor of one dirt
 *       block over a pit);</li>
 *   <li>{@code dirtFaces}: a column shows more than {@value #MAX_DIRT_FACE} blocks of dirt in a
 *       row to open air beside it (the surface rules put at most about 7 blocks of dirt under
 *       grass, a bank raised by up to {@code WaterBodies.SOIL_BLOCKS} adds its soil, and a slope
 *       shows less of it).</li>
 * </ul>
 * No dirt face may remain; pits are allowed in at most one column in a hundred (the shield closes
 * the band's entrance term and, since 2026-10-21, vanilla's noodle caves within the band, HYD-078;
 * a cave of the deep stack under the band can still reach up to the surface now and then). The
 * gate batch on master f43d1e1a found 173 of 11,086 shore columns in a pit with only the entrances
 * shut: noodle tubes 1 to 4 blocks under shallow sea floors, and an 8-block void under a river;
 * with the noodles shut too, none.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class WaterShoreGameTests {

    private WaterShoreGameTests() {}

    /** Half the side of each box, blocks. */
    static final int HALF = 20;
    /** How far under the rule's ground the generated ground may lie, blocks (the density's ground between noise-cell corners strays by a block or two). */
    static final int PIT_TOLERANCE = 3;
    /** The longest run of dirt a column may show to the air beside it, blocks. */
    static final int MAX_DIRT_FACE = 10;
    /** The most shallow-coast boxes. */
    static final int COAST_SITES = 4;

    /** Water the shield counts: a river or lake column, or a sea column whose ground is shallow. */
    static boolean shieldedWater(Water.Column c, int seaTop) {
        return switch (c.kind()) {
            case RIVER, LAKE -> true;
            case SEA -> c.ground() >= seaTop - WaterCaveShield.SHALLOW_FULL_BLOCKS;
            default -> false;
        };
    }

    /** Boxes on shallow coasts: a sea column with shallow ground beside land, spread over the planet. */
    static List<RiverGameTests.Site> coastSites(WorldData wd, Water.Spec spec, List<RiverGameTests.Site> have) {
        List<RiverGameTests.Site> out = new ArrayList<>();
        int c = wd.circumference(), seaTop = spec.seaLevel() - 1;
        int zHalf = c / 4 - 2 * HALF;
        // Rows across the planet so the boxes spread out; the first fitting column of each row.
        for (int z = -zHalf; z < zHalf && out.size() < COAST_SITES; z += Math.max(64, 2 * zHalf / (2 * COAST_SITES))) {
            for (int x = -c / 2; x < c / 2; x += 8) {
                Water.Core q = Water.core(wd, spec, x, z);
                if (q.kind() != Water.Kind.SEA || q.ground() < seaTop - 3)
                    continue;
                if (Water.core(wd, spec, x + 8, z).kind() != Water.Kind.NONE && Water.core(wd, spec, x - 8, z).kind() != Water.Kind.NONE)
                    continue;
                boolean near = false;
                for (RiverGameTests.Site s : have)
                    near |= Math.abs(s.x() - x) < 3 * HALF && Math.abs(s.z() - z) < 3 * HALF;
                for (RiverGameTests.Site s : out)
                    near |= Math.abs(s.x() - x) < 3 * HALF && Math.abs(s.z() - z) < 3 * HALF;
                if (!near) {
                    out.add(new RiverGameTests.Site("shallow coast", x, z));
                    break;
                }
            }
        }
        return out;
    }

    /** The ground the water pass leaves at a column: a channel's or lake's bed, a bank's ground, the rule's ground elsewhere. */
    static int expectedGround(Water.Column c) {
        if (c.kind() == Water.Kind.RIVER || c.kind() == Water.Kind.LAKE)
            return c.bed();
        int g = c.ground();
        if (c.maxGround() != null)
            g = Math.min(g, c.maxGround());
        if (c.minGround() != null)
            g = Math.max(g, c.minGround());
        return g;
    }

    /** Not ground: air, a fluid, ice or snow on top. */
    static boolean overGround(BlockState s) {
        return s.isAir() || !s.getFluidState().isEmpty() || s.is(BlockTags.ICE) || s.is(BlockTags.SNOW) || s.is(Blocks.SNOW_BLOCK);
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 3600)
    public static void shoresHaveNoDirtWalls(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_CAVES.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)) {
            helper.fail("test_planet_caves is not generating from world data");
            return;
        }
        WorldData wd = w.data();
        Rivers r = wd.rivers();
        if (r == null || r.chainCount() == 0) {
            helper.fail("the real test planet has no river network");
            return;
        }
        Water.Spec spec = Water.Spec.of(wd, g.waterReach());
        int sea = g.generatorSettings().value().seaLevel(), seaTop = sea - 1;
        List<RiverGameTests.Site> sites = new ArrayList<>(RiverGameTests.sites(r));
        sites.addAll(RiverGameTests.highSites(r, sea, sites));
        sites.addAll(coastSites(wd, spec, sites));
        List<LevelChunk> chunks = new ArrayList<>();
        for (RiverGameTests.Site s : sites)
            for (int cx = (s.x() - HALF) >> 4; cx <= (s.x() + HALF) >> 4; cx++)
                for (int cz = (s.z() - HALF) >> 4; cz <= (s.z() + HALF) >> 4; cz++) {
                    level.setChunkForced(cx, cz, true);
                    chunks.add(level.getChunk(cx, cz));
                }
        int ring = (int) WaterCaveShield.FULL_BLOCKS - 1;
        long checked = 0, pits = 0, dirtFaces = 0, waterColumns = 0;
        StringBuilder detail = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int minY = level.getMinBuildHeight();
        for (RiverGameTests.Site s : sites) {
            int x0 = s.x() - HALF + 1, z0 = s.z() - HALF + 1, n = 2 * HALF - 2, m = n + 2 * ring;
            Water.Column[] cols = new Water.Column[m * m];
            for (int j = 0; j < m; j++)
                for (int i = 0; i < m; i++)
                    cols[j * m + i] = Water.column(wd, spec, x0 - ring + i, z0 - ring + j);
            for (int lz = 0; lz < n; lz++) {
                for (int lx = 0; lx < n; lx++) {
                    int x = x0 + lx, z = z0 + lz;
                    Water.Column col = cols[(lz + ring) * m + lx + ring];
                    if (shieldedWater(col, seaTop))
                        waterColumns++;
                    boolean near = false;
                    for (int dz = -ring; dz <= ring && !near; dz++)
                        for (int dx = -ring; dx <= ring && !near; dx++)
                            near = dx * dx + dz * dz <= ring * ring && shieldedWater(cols[(lz + ring + dz) * m + lx + ring + dx], seaTop);
                    if (!near)
                        continue;
                    checked++;
                    LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
                    int y = chunk.getHeight(Heightmap.Types.WORLD_SURFACE, x & 15, z & 15);
                    while (y > minY && overGround(chunk.getBlockState(pos.set(x, y, z))))
                        y--;
                    int want = expectedGround(col);
                    boolean hangs = false;
                    for (int k = 1; k <= PIT_TOLERANCE && !hangs; k++)
                        hangs = chunk.getBlockState(pos.set(x, y - k, z)).isAir();
                    if (y < want - PIT_TOLERANCE || hangs) {
                        pits++;
                        if (detail.length() < 2400)
                            detail.append(" pit(").append(x).append(',').append(y).append(',').append(z).append(" want ").append(want)
                                    .append(' ').append(col.kind().id).append(hangs ? " hangs" : "").append(')');
                    }
                    // The longest run of dirt with open air beside it, from the top down 40 blocks.
                    int run = 0, longest = 0, at = 0;
                    for (int yy = y; yy > y - 40 && yy > minY; yy--) {
                        boolean dirt = chunk.getBlockState(pos.set(x, yy, z)).is(Blocks.DIRT);
                        boolean exposed = false;
                        if (dirt)
                            for (int[] d : Water.NEIGHBOURS)
                                exposed |= level.getBlockState(pos.set(x + d[0], yy, z + d[1])).isAir();
                        run = dirt && exposed ? run + 1 : 0;
                        if (run > longest) {
                            longest = run;
                            at = yy;
                        }
                    }
                    if (longest > MAX_DIRT_FACE) {
                        dirtFaces++;
                        if (detail.length() < 2400)
                            detail.append(" dirt(").append(x).append(',').append(at).append(',').append(z).append(" run ").append(longest)
                                    .append(' ').append(col.kind().id).append(')');
                    }
                }
            }
        }
        for (LevelChunk c : chunks)
            level.setChunkForced(c.getPos().x, c.getPos().z, false);
        DeepTime.LOGGER.info("[gametest] shoresHaveNoDirtWalls sites={} chunks={} waterColumns={} shoreColumns={} pits={} dirtFaces={}",
                sites.size(), chunks.size(), waterColumns, checked, pits, dirtFaces);
        DeepTime.LOGGER.info("[gametest] shoresHaveNoDirtWalls sites: {}", sites);
        if (!detail.isEmpty())
            DeepTime.LOGGER.info("[gametest] shoresHaveNoDirtWalls detail:{}", detail);
        helper.assertTrue(waterColumns > 100, "only " + waterColumns + " river, lake or shallow-sea columns in the boxes");
        helper.assertTrue(dirtFaces == 0, dirtFaces + " shore columns show a wall of dirt over " + MAX_DIRT_FACE + " blocks tall");
        helper.assertTrue(pits * 100 <= checked, pits + " of " + checked + " shore columns lie in a pit under the rule's ground");
        helper.succeed();
    }
}
