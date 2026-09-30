// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BiomeTags;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.material.Fluids;
import net.minecraft.world.ticks.ScheduledTick;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Rivers and lakes hold (P1-22, plan §8.4 and §7.9): on the real test planet's world data, generated
 * with a bare biome and no carvers, features or caves ({@code deeptime:test_rivers}), boxes around a
 * big river, a small stream, a river mouth in a lake and a lake outlet. At generation no water block
 * floats (air under it) and none leaks (air beside it over ground or over air); every river and
 * lake column's water top is the one {@link Water} decides; then every water block is woken with a
 * fluid tick and after {@value #WAIT_TICKS} ticks no water has reached a column that should be dry.
 * {@link #riverBiomes} checks that wide channels are river biomes on the real planet's biomes.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class RiverGameTests {

    static final int WAIT_TICKS = 300;
    /** Half the side of each box, blocks. */
    static final int HALF = 20;

    private RiverGameTests() {}

    /** A box to check: its centre and why it was picked. */
    record Site(String what, int x, int z) {}

    /** The boxes: the largest discharge, a stream near its spring, lake mouths and outlets. */
    static List<Site> sites(Rivers r) {
        List<Site> out = new ArrayList<>();
        int big = -1, small = -1;
        for (int i = 0; i < r.vertexCount(); i++) {
            if (big < 0 || r.vertexHalfWidth(i) > r.vertexHalfWidth(big))
                big = i;
        }
        for (int k = 0; k < r.chainCount() && small < 0; k++)
            if (r.chainLength(k) > 12)
                small = r.chainFirst(k) + 6;
        if (big >= 0)
            out.add(new Site("big river", Math.round(r.vertexX(big)), Math.round(r.vertexZ(big))));
        if (small >= 0)
            out.add(new Site("stream", Math.round(r.vertexX(small)), Math.round(r.vertexZ(small))));
        int mouths = 0, outlets = 0;
        for (int k = 0; k < r.chainCount(); k++) {
            int last = r.chainFirst(k) + r.chainLength(k) - 1;
            if (r.chainMouth(k) == Rivers.MOUTH_LAKE && mouths < 2) {
                out.add(new Site("lake mouth", Math.round(r.vertexX(last)), Math.round(r.vertexZ(last))));
                mouths++;
            }
        }
        // A lake outlet: its first vertex is in the lake, at the lake's level, and the next is the
        // spill cell a freeboard lower.
        for (int k = 0; k < r.chainCount() && outlets < 1; k++) {
            int first = r.chainFirst(k);
            if (r.vertexW(first) >= r.vertexW(first + 1) + 0.5f && r.chainLength(k) > 4) {
                out.add(new Site("lake outlet", Math.round(r.vertexX(first)), Math.round(r.vertexZ(first))));
                outlets++;
            }
        }
        return out;
    }

    /** Counts over the boxes. */
    static final class Tally {
        long waterBlocks, floating, leaks, steps, topMismatch, waterColumns, outside;
        final List<BlockPos> water = new ArrayList<>();
        final StringBuilder detail = new StringBuilder();

        void note(String what, int x, int y, int z) {
            if (detail.length() < 800)
                detail.append(' ').append(what).append('(').append(x).append(',').append(y).append(',').append(z).append(')');
        }
    }

    static boolean isWater(BlockState s) {
        return s.getFluidState().is(FluidTags.WATER);
    }

    static Tally count(ServerLevel level, WorldData wd, Water.Spec spec, List<Site> sites) {
        Tally t = new Tally();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (Site s : sites) {
            for (int x = s.x() - HALF + 1; x < s.x() + HALF - 1; x++) {
                for (int z = s.z() - HALF + 1; z < s.z() + HALF - 1; z++) {
                    Water.Column col = Water.column(wd, spec, x, z);
                    LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
                    int top = chunk.getHeight(net.minecraft.world.level.levelgen.Heightmap.Types.WORLD_SURFACE, x & 15, z & 15);
                    int highestWater = Integer.MIN_VALUE;
                    for (int y = top; y > level.getMinBuildHeight() && y > top - 48; y--) {
                        BlockState b = chunk.getBlockState(pos.set(x, y, z));
                        if (!isWater(b))
                            continue;
                        if (highestWater == Integer.MIN_VALUE)
                            highestWater = y;
                        t.waterBlocks++;
                        if (b.getFluidState().isSource())
                            t.water.add(pos.immutable());
                        BlockState below = level.getBlockState(pos.set(x, y - 1, z));
                        if (below.isAir()) {
                            t.floating++;
                            t.note("float", x, y, z);
                        }
                        // Flowing water over water does not spread sideways (vanilla's "water
                        // hole"): the ring of a step and a waterfall's curtain are at rest.
                        if (!b.getFluidState().isSource() && isWater(below))
                            continue;
                        for (int[] d : Water.NEIGHBOURS) {
                            BlockState n = level.getBlockState(pos.set(x + d[0], y, z + d[1]));
                            if (!n.isAir())
                                continue;
                            BlockState under = level.getBlockState(pos.set(x + d[0], y - 1, z + d[1]));
                            if (isWater(under)) {
                                t.steps++;
                            } else {
                                t.leaks++;
                                t.note("leak", x, y, z);
                            }
                        }
                    }
                    boolean wet = col.kind() == Water.Kind.RIVER || col.kind() == Water.Kind.LAKE;
                    if (wet) {
                        t.waterColumns++;
                        if (highestWater < col.top()) {
                            t.topMismatch++;
                            t.note("top" + col.top() + "/" + highestWater, x, highestWater, z);
                        }
                    } else if (!col.kind().isWater() && highestWater != Integer.MIN_VALUE && highestWater > col.ground() + 1) {
                        // Water above the ground of a column that should be dry (a step's flowing
                        // block sits in a water column, never here).
                        t.outside++;
                        t.note("outside:" + col.kind().id, x, highestWater, z);
                    }
                }
            }
        }
        return t;
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = WAIT_TICKS + 2400)
    public static void riversHold(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_RIVERS);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_RIVERS.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)) {
            helper.fail("test_rivers is not generating from world data");
            return;
        }
        WorldData wd = w.data();
        Rivers r = wd.rivers();
        if (r == null || r.chainCount() == 0) {
            helper.fail("the real test planet has no river network");
            return;
        }
        Water.Spec spec = Water.Spec.of(wd, g.waterReach());
        List<Site> sites = sites(r);
        List<LevelChunk> chunks = new ArrayList<>();
        for (Site s : sites)
            for (int cx = (s.x() - HALF) >> 4; cx <= (s.x() + HALF) >> 4; cx++)
                for (int cz = (s.z() - HALF) >> 4; cz <= (s.z() + HALF) >> 4; cz++) {
                    level.setChunkForced(cx, cz, true);
                    chunks.add(level.getChunk(cx, cz));
                }
        int ticksAtGeneration = WaterGameTests.pendingFluidTicks(chunks);
        Tally before = count(level, wd, spec, sites);
        long now = level.getGameTime();
        for (BlockPos p : before.water)
            level.getChunkAt(p).getFluidTicks().schedule(new ScheduledTick<>(Fluids.WATER, p, now + 1, level.nextSubTickCount()));
        helper.runAfterDelay(WAIT_TICKS, () -> {
            Tally after = count(level, wd, spec, sites);
            for (LevelChunk c : chunks)
                level.setChunkForced(c.getPos().x, c.getPos().z, false);
            DeepTime.LOGGER.info("[gametest] riversHold sites={} chunks={} waterColumns={} waterBlocks={}/{} floating={}/{} leaks={}/{} "
                            + "steps={}/{} topMismatch={} outside={}/{} generatedFluidTicks={} chains={} vertices={}",
                    sites.size(), chunks.size(), before.waterColumns, before.waterBlocks, after.waterBlocks, before.floating, after.floating,
                    before.leaks, after.leaks, before.steps, after.steps, before.topMismatch, before.outside, after.outside,
                    ticksAtGeneration, r.chainCount(), r.vertexCount());
            DeepTime.LOGGER.info("[gametest] riversHold sites: {}", sites);
            if (!before.detail.isEmpty() || !after.detail.isEmpty())
                DeepTime.LOGGER.info("[gametest] riversHold detail: atGeneration{} after{}", before.detail, after.detail);
            helper.assertTrue(ticksAtGeneration == 0, "generation left " + ticksAtGeneration + " fluid ticks scheduled");
            helper.assertTrue(before.waterColumns > 50, "only " + before.waterColumns + " river or lake columns in the boxes");
            helper.assertTrue(before.floating == 0, before.floating + " water blocks over air after generation");
            helper.assertTrue(before.leaks == 0, before.leaks + " water blocks beside open air after generation");
            helper.assertTrue(before.topMismatch == 0, before.topMismatch + " river or lake columns below their water top");
            helper.assertTrue(before.outside == 0, before.outside + " dry columns hold water after generation");
            helper.assertTrue(after.outside == 0, after.outside + " dry columns hold water after " + WAIT_TICKS + " ticks");
            helper.assertTrue(after.floating == 0, after.floating + " water blocks over air after " + WAIT_TICKS + " ticks");
            helper.succeed();
        });
    }

    /**
     * Channels wide enough for a river biome ({@code biome_min_width}) are river biomes: the biome
     * source at their centre, at the water surface, carries {@code #minecraft:is_river} on the real
     * planet's own biomes. Logged per sample; a floor, since vanilla's parameter list keeps rivers
     * out of mid- and far-inland mountains (erosion E0–E1).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void riverBiomes(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        WorldDataTerrain t = RealPlanetGameTests.terrain(helper, level);
        Rivers r = t.data().rivers();
        if (r == null) {
            helper.fail("the real test planet has no river network");
            return;
        }
        var sampler = level.getChunkSource().randomState().sampler();
        var g = level.getChunkSource().getGenerator();
        int sampled = 0, river = 0;
        StringBuilder seen = new StringBuilder();
        for (int i = 0; i < r.vertexCount() && sampled < 200; i += 3) {
            int x = Math.round(r.vertexX(i)), z = Math.round(r.vertexZ(i));
            // At sea level the channel is the river's mouth, over the sea's own biomes.
            if (!r.sample(x, z).biome() || r.vertexW(i) <= t.data().manifest().seaLevelY() + 0.5f)
                continue;
            int y = (int) Math.floor(r.vertexW(i)) - 1;
            Holder<Biome> b = g.getBiomeSource().getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y), QuartPos.fromBlock(z), sampler);
            sampled++;
            if (b.is(BiomeTags.IS_RIVER))
                river++;
            else if (seen.length() < 400)
                seen.append(' ').append(b.getRegisteredName());
        }
        DeepTime.LOGGER.info("[gametest] riverBiomes sampled={} river={} share={} others:{}", sampled, river,
                sampled == 0 ? "n/a" : String.format("%.3f", river / (double) sampled), seen);
        if (sampled > 0)
            helper.assertTrue(river >= sampled / 2, "only " + river + " of " + sampled + " wide-channel samples are river biomes");
        helper.succeed();
    }
}
