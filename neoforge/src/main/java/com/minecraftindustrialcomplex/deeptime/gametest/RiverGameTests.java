// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterBodies;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BiomeTags;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.GenerationStep;
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
 * lake column's water top is the one {@link Water} decides (none is dry shore, see {@link #count}); then every water block is woken with a
 * fluid tick and after {@value #WAIT_TICKS} ticks no water has reached a column that should be dry.
 *
 * <p>The checks are properties of any planet, not one planet's numbers: the simulation is not
 * bit-identical across platforms, so the boxes sit somewhere else on every machine.
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

    /** The most boxes {@link #highSites} adds. */
    static final int HIGH_SITES = 6;
    /** The highest water surface a box of {@link #highSites} is put on: the test carver starts no higher than y 180. */
    static final int HIGH_SITE_TOP = 170;

    /**
     * Boxes on rivers well above sea level, for {@link #carversSpareWater}: only there does a carver
     * that replaces water leave air (below sea level the cut fills with sea water). {@link #sites}
     * picks by shape (the widest channel is usually a river's mouth, at sea level), so on a planet
     * whose lakes and mouths all sit at the coast its boxes hold almost no water above the sea. One
     * box per chain at the middle of the chain's vertices between {@code sea + 4} and
     * {@value #HIGH_SITE_TOP}, then boxes along those runs, each at least three half-boxes from
     * every box already chosen, up to {@link #HIGH_SITES}.
     */
    static List<Site> highSites(Rivers r, int sea, List<Site> have) {
        List<Site> out = new ArrayList<>();
        // First the middle of each chain's high run, then any of its high vertices: the Lambert test
        // planet has few chains above the sea, so one box each was too few for the carvers to reach.
        for (int pass = 0; pass < 2; pass++)
            for (int k = 0; k < r.chainCount() && out.size() < HIGH_SITES; k++) {
                int first = -1, last = -1;
                for (int i = r.chainFirst(k); i < r.chainFirst(k) + r.chainLength(k); i++)
                    if (r.vertexW(i) >= sea + 4 && r.vertexW(i) <= HIGH_SITE_TOP) {
                        if (first < 0)
                            first = i;
                        last = i;
                    }
                if (first < 0)
                    continue;
                int from = pass == 0 ? (first + last) / 2 : first, to = pass == 0 ? (first + last) / 2 : last;
                for (int at = from; at <= to && out.size() < HIGH_SITES; at++) {
                    if (r.vertexW(at) < sea + 4 || r.vertexW(at) > HIGH_SITE_TOP)
                        continue;
                    int x = Math.round(r.vertexX(at)), z = Math.round(r.vertexZ(at));
                    boolean near = false;
                    for (Site s : have)
                        near |= Math.abs(s.x() - x) < 3 * HALF && Math.abs(s.z() - z) < 3 * HALF;
                    for (Site s : out)
                        near |= Math.abs(s.x() - x) < 3 * HALF && Math.abs(s.z() - z) < 3 * HALF;
                    if (!near)
                        out.add(new Site("high river", x, z));
                }
            }
        return out;
    }

    /** Counts over the boxes. */
    static final class Tally {
        long waterBlocks, floating, leaks, steps, topMismatch, waterColumns, outside, dryShore, lakeColumns, iced, wrongIce;
        final List<BlockPos> water = new ArrayList<>();
        final StringBuilder detail = new StringBuilder();

        void note(String what, int x, int y, int z) {
            if (detail.length() < 2400)
                detail.append(' ').append(what).append('(').append(x).append(',').append(y).append(',').append(z).append(')');
        }
    }

    static boolean isWater(BlockState s) {
        return s.getFluidState().is(FluidTags.WATER);
    }

    /**
     * Water the pass placed, liquid or frozen: a frozen lake's top block is ice ({@code WaterBodies}
     * sets it inside the shielded range), and on a cold planet a lake one block deep holds only
     * that ice.
     */
    static boolean isPlacedWater(BlockState s) {
        return isWater(s) || s.is(net.minecraft.world.level.block.Blocks.ICE);
    }

    /**
     * Reads the boxes back. A river column holds water up to its top, always: the pass cuts or fills
     * it to the channel. So does a lake column (FORMAT.md, "Water columns": a lake column is cut to
     * its bed): {@link Water}'s {@code ground} is the terrain bilinear between noise-cell corners,
     * which the generated ground between corners can exceed by a block or two (the density's scale
     * differs from corner to corner), and the pass removes that ground from the water's range. A
     * lake column whose top block is neither water nor ice is {@code dryShore}: ground flush with
     * the water beside it, which the pass must not leave (it was 96 of 898 water columns on the
     * seed-11 test planet of the ice branch before the cut). A lake column whose top block is open
     * and holds no water is a mismatch, as is any river column below its top.
     */
    static Tally count(ServerLevel level, WorldData wd, Water.Spec spec, List<Site> sites) {
        Tally t = new Tally();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        com.minecraftindustrialcomplex.deeptime.worlddata.LakeIce lakeIce = com.minecraftindustrialcomplex.deeptime.worlddata.LakeIce.of(wd);
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
                        BlockState atTop = chunk.getBlockState(pos.set(x, col.top(), z));
                        boolean lake = col.kind() == Water.Kind.LAKE;
                        if (lake)
                            t.lakeColumns++;
                        // A frozen lake's top block is ice (LakeIce), an open lake's never: the
                        // climate's rule, whatever the biome.
                        boolean ice = lake && atTop.is(net.minecraft.world.level.block.Blocks.ICE);
                        boolean frozen = lake && lakeIce != null && lakeIce.frozen(x, z);
                        if (ice)
                            t.iced++;
                        if (lake && (ice || isWater(atTop)) && ice != frozen) {
                            t.wrongIce++;
                            t.note(frozen ? "open-but-frozen" : "iced-but-open", x, col.top(), z);
                        }
                        if (ice) {
                            // Water under the ice, or the ice rests on the bed: nothing to add.
                        } else if (lake && highestWater < col.top() && !atTop.isAir() && atTop.getFluidState().isEmpty()) {
                            t.dryShore++;
                            StringBuilder blocks = new StringBuilder();
                            for (int y = col.top() + 2; y >= col.top() - 3; y--)
                                blocks.append(' ').append(BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(pos.set(x, y, z)).getBlock()).getPath());
                            t.note("dry" + col.top() + "[ground " + col.ground() + " surface " + String.format(java.util.Locale.ROOT, "%.3f", col.surface())
                                    + " level " + col.level() + " bed " + col.bed() + " heightmap " + top + " y" + (col.top() + 2) + "↓" + blocks + "]", x, highestWater, z);
                        } else if (highestWater < col.top()) {
                            t.topMismatch++;
                            StringBuilder blocks = new StringBuilder();
                            for (int y = col.top() + 2; y >= col.top() - 2; y--)
                                blocks.append(' ').append(BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(pos.set(x, y, z)).getBlock()).getPath());
                            t.note("top" + col.top() + "/" + highestWater + "[" + col.kind().id + " ground " + col.ground() + " surface "
                                    + String.format(java.util.Locale.ROOT, "%.3f", col.surface()) + " level " + col.level() + " bed " + col.bed()
                                    + " heightmap " + top + " y" + (col.top() + 2) + "↓" + blocks + "]", x, highestWater, z);
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
                            + "steps={}/{} topMismatch={} dryShore={} of {} lake columns (iced={} wrongIce={}) outside={}/{} generatedFluidTicks={} chains={} vertices={}",
                    sites.size(), chunks.size(), before.waterColumns, before.waterBlocks, after.waterBlocks, before.floating, after.floating,
                    before.leaks, after.leaks, before.steps, after.steps, before.topMismatch, before.dryShore, before.lakeColumns, before.iced, before.wrongIce,
                    before.outside, after.outside, ticksAtGeneration, r.chainCount(), r.vertexCount());
            DeepTime.LOGGER.info("[gametest] riversHold sites: {}", sites);
            if (!before.detail.isEmpty() || !after.detail.isEmpty())
                DeepTime.LOGGER.info("[gametest] riversHold detail: atGeneration{} after{}", before.detail, after.detail);
            helper.assertTrue(ticksAtGeneration == 0, "generation left " + ticksAtGeneration + " fluid ticks scheduled");
            helper.assertTrue(before.waterColumns > 50, "only " + before.waterColumns + " river or lake columns in the boxes");
            helper.assertTrue(before.floating == 0, before.floating + " water blocks over air after generation");
            helper.assertTrue(before.leaks == 0, before.leaks + " water blocks beside open air after generation");
            helper.assertTrue(before.topMismatch == 0, before.topMismatch + " river or lake columns below their water top");
            // The pass cuts a lake column to its bed as it does a river's: no lake column is left with
            // ground at the waterline (22 % of one planet's lake columns were, before the cut).
            helper.assertTrue(before.dryShore == 0, before.dryShore + " of " + before.waterColumns
                    + " water columns are lake columns with ground at the waterline");
            helper.assertTrue(before.wrongIce == 0, before.wrongIce + " lake columns are iced where the climate keeps them open, or open where it freezes them");
            helper.assertTrue(before.outside == 0, before.outside + " dry columns hold water after generation");
            helper.assertTrue(after.outside == 0, after.outside + " dry columns hold water after " + WAIT_TICKS + " ticks");
            helper.assertTrue(after.floating == 0, after.floating + " water blocks over air after " + WAIT_TICKS + " ticks");
            helper.succeed();
        });
    }

    /** Whether a block is air or holds fluid: what a carver's cut leaves (see {@link #carversSpareWater}). */
    static boolean cut(BlockState s) {
        return s.isAir() || !s.getFluidState().isEmpty();
    }

    /**
     * Carvers never replace the water Deep Time places (Ben, decisions log 2026-10-01, "Protect
     * water blocks"; {@code hydro.protect_water_from_carvers}, on by default). On
     * {@code deeptime:test_rivers_carved}, {@code test_rivers}' world data under a biome whose only
     * carver is {@code deeptime:test_wide_caves} (a cave carver that starts in every chunk, with
     * tunnels 1.5 to 2.5 times vanilla's width, at every height from y -32 to 180), the boxes around
     * rivers and lakes are generated, and every river and lake column is read back next to the same
     * column of {@code deeptime:test_rivers}, the same world data generated without carvers:
     * <ul>
     *   <li>no block between its bed and its top that is water (or a frozen lake's ice,
     *       {@link #isPlacedWater}) without carvers and air with them ({@code drainedBlocks}), and
     *       no column whose top water block became air ({@code holes}): the carvers replaced none
     *       of the water. Air where the uncarved level has ground is the
     *       carvers' right ({@code carvedShore}: a lake column's generated ground can stand in the
     *       water's range, see {@link #count}; the ground beside and under the water stays
     *       carvable), so it is counted apart, and {@code waterBlocks} says how much water the
     *       comparison covered;</li>
     *   <li>the carvers did reach it: ground right under the beds is cut ({@code carvedBelow}: the
     *       pass leaves it solid, so air or fluid there is a carver's cut) and some beds opened
     *       ({@code entrances}, the bed block itself is cut): the ground under the water is still
     *       carved, so caves open into river and lake beds.</li>
     *   <li>the carvers cut under water above sea level ({@code highColumns}, {@code carvedBelowHigh}):
     *       this level has no aquifer, so a cut below sea level fills with sea water and a carver
     *       replacing river water there would leave water; only above sea level does it leave air,
     *       so only there can the first check fail. {@link #highSites} adds boxes on rivers above
     *       sea level, so this holds on a planet whose lakes and river mouths all sit at the coast.</li>
     * </ul>
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void carversSpareWater(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_RIVERS_CARVED);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_RIVERS_CARVED.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)) {
            helper.fail("test_rivers_carved is not generating from world data");
            return;
        }
        WorldData wd = w.data();
        Rivers r = wd.rivers();
        if (r == null || r.chainCount() == 0) {
            helper.fail("the real test planet has no river network");
            return;
        }
        helper.assertTrue(WaterBodies.protectsWater(wd), "the test planet's hydro settings do not protect the water");
        ServerLevel plain = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_RIVERS);
        if (plain == null) {
            helper.fail("level " + GameTestSupport.TEST_RIVERS.location() + " (the uncarved twin) does not exist");
            return;
        }
        Water.Spec spec = Water.Spec.of(wd, g.waterReach());
        List<Site> sites = new ArrayList<>(sites(r));
        sites.addAll(highSites(r, spec.seaLevel(), sites));
        List<LevelChunk> chunks = new ArrayList<>();
        for (Site s : sites)
            for (int cx = (s.x() - HALF) >> 4; cx <= (s.x() + HALF) >> 4; cx++)
                for (int cz = (s.z() - HALF) >> 4; cz <= (s.z() + HALF) >> 4; cz++) {
                    level.setChunkForced(cx, cz, true);
                    chunks.add(level.getChunk(cx, cz));
                }
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        long columns = 0, drainedBlocks = 0, drainedColumns = 0, holes = 0, carvedBelow = 0, entrances = 0, highColumns = 0, carvedBelowHigh = 0;
        long carvedShore = 0, waterBlocks = 0;
        StringBuilder detail = new StringBuilder();
        java.util.Map<String, Integer> underBed = new java.util.TreeMap<>();
        for (Site s : sites)
            for (int x = s.x() - HALF + 1; x < s.x() + HALF - 1; x++)
                for (int z = s.z() - HALF + 1; z < s.z() + HALF - 1; z++) {
                    Water.Column col = Water.column(wd, spec, x, z);
                    if ((col.kind() != Water.Kind.RIVER && col.kind() != Water.Kind.LAKE) || col.top() == null || col.bed() == null)
                        continue;
                    int top = col.top(), bed = col.bed();
                    LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
                    LevelChunk uncarved = plain.getChunk(x >> 4, z >> 4);
                    columns++;
                    int air = 0;
                    for (int y = bed + 1; y <= top; y++) {
                        boolean was = isPlacedWater(uncarved.getBlockState(pos.set(x, y, z)));
                        if (was)
                            waterBlocks++;
                        if (chunk.getBlockState(pos).isAir()) {
                            if (was)
                                air++;
                            else
                                carvedShore++;
                        }
                    }
                    if (air > 0) {
                        drainedColumns++;
                        drainedBlocks += air;
                        if (detail.length() < 600)
                            detail.append(" drained(").append(x).append(',').append(top).append(',').append(z).append(")x").append(air);
                    }
                    if (chunk.getBlockState(pos.set(x, top, z)).isAir() && isPlacedWater(uncarved.getBlockState(pos)))
                        holes++;
                    // The pass leaves solid ground under the bed, so a block there that is air or fluid was cut
                    // by a carver. Below sea level a carver's cut fills with water (this level has no aquifer:
                    // the fluid picker puts the sea there), above it the cut is air.
                    if (cut(chunk.getBlockState(pos.set(x, bed, z))))
                        entrances++;
                    boolean high = top >= spec.seaLevel();
                    if (high)
                        highColumns++;
                    for (int y = Math.max(level.getMinBuildHeight(), bed - 8); y < bed; y++)
                        if (cut(chunk.getBlockState(pos.set(x, y, z)))) {
                            carvedBelow++;
                            if (high)
                                carvedBelowHigh++;
                        }
                    for (int y = bed - 1; y >= bed - 2; y--) {
                        BlockState b = chunk.getBlockState(pos.set(x, y, z));
                        underBed.merge(BuiltInRegistries.BLOCK.getKey(b.getBlock()).toString() + (b.is(BlockTags.OVERWORLD_CARVER_REPLACEABLES) ? "+" : "-"), 1, Integer::sum);
                    }
                }
        // Premise diagnostics: is the carver on the level's biome, and did it cut the ground anywhere in the boxes?
        StringBuilder carvers = new StringBuilder();
        for (Holder<Biome> b : g.getBiomeSource().possibleBiomes())
            for (var c : b.value().getGenerationSettings().getCarvers(GenerationStep.Carving.AIR))
                carvers.append(c.unwrapKey().map(k -> k.location().toString()).orElse("?")).append(' ');
        long groundAir = 0, groundCells = 0, groundReplaceable = 0;
        java.util.Map<String, Integer> groundBlocks = new java.util.TreeMap<>();
        for (Site s : sites)
            for (int x = s.x() - HALF + 1; x < s.x() + HALF - 1; x += 2)
                for (int z = s.z() - HALF + 1; z < s.z() + HALF - 1; z += 2) {
                    Water.Column col = Water.column(wd, spec, x, z);
                    if (col.kind().isWater())
                        continue;
                    LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
                    for (int y = col.ground() - 3; y >= Math.max(level.getMinBuildHeight(), col.ground() - 40); y--) {
                        groundCells++;
                        BlockState gb = chunk.getBlockState(pos.set(x, y, z));
                        if (gb.isAir())
                            groundAir++;
                        else if (gb.is(BlockTags.OVERWORLD_CARVER_REPLACEABLES))
                            groundReplaceable++;
                        groundBlocks.merge(BuiltInRegistries.BLOCK.getKey(gb.getBlock()).toString(), 1, Integer::sum);
                    }
                }
        DeepTime.LOGGER.info("[gametest] carversSpareWater premise: biome carvers [{}] groundAir={}/{} replaceableByVanillaTag={} underBed={} ground={}",
                carvers.toString().trim(), groundAir, groundCells, groundReplaceable, underBed, groundBlocks);
        for (LevelChunk c : chunks)
            level.setChunkForced(c.getPos().x, c.getPos().z, false);
        DeepTime.LOGGER.info("[gametest] carversSpareWater boxes: {}", sites);
        DeepTime.LOGGER.info("[gametest] carversSpareWater sites={} chunks={} columns={} waterBlocks={} drainedBlocks={} drainedColumns={} holes={} "
                        + "carvedShore={} carvedBelow={} entrances={} highColumns={} carvedBelowHigh={}{}",
                sites.size(), chunks.size(), columns, waterBlocks, drainedBlocks, drainedColumns, holes, carvedShore, carvedBelow, entrances,
                highColumns, carvedBelowHigh, detail.length() == 0 ? "" : " " + detail);
        helper.assertTrue(columns > 50, "only " + columns + " river or lake columns in the boxes");
        helper.assertTrue(waterBlocks >= columns / 2, "the uncarved level holds only " + waterBlocks + " water or lake-ice blocks in " + columns
                + " river or lake columns: nothing to compare with");
        helper.assertTrue(carvedBelow >= 20, "the carvers cut only " + carvedBelow + " blocks under the beds: they did not reach the water, so the check proves nothing");
        helper.assertTrue(highColumns >= 20 && carvedBelowHigh >= 20, "only " + highColumns + " columns hold water above sea level, with " + carvedBelowHigh
                + " blocks cut under them: only there does a carver leave air, so the check proves nothing");
        helper.assertTrue(entrances > 0, "no bed was opened by a cave: the ground under the water must stay carvable");
        helper.assertTrue(drainedBlocks == 0, drainedBlocks + " water blocks in " + drainedColumns + " columns were replaced by air:" + detail);
        helper.assertTrue(holes == 0, holes + " river or lake columns lost their top water block to air");
        helper.succeed();
    }

    /**
     * Channels wide enough for a river biome ({@code biome_min_width}) are river biomes: the biome
     * source at their centre, at the water surface, carries {@code #minecraft:is_river} on the real
     * planet's own biomes, or swamp where vanilla's valley slice is swamp (any biome of
     * {@code #c:is_swamp}, so a pack's own swamp there counts). Logged per sample; a floor, since
     * vanilla's parameter list keeps rivers out of mid- and far-inland mountains (erosion E0–E1).
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
        int sampled = 0, river = 0, swamp = 0;
        StringBuilder seen = new StringBuilder();
        for (int i = 0; i < r.vertexCount() && sampled < 200; i += 3) {
            int x = Math.round(r.vertexX(i)), z = Math.round(r.vertexZ(i));
            // At sea level the channel is the river's mouth, over the sea's own biomes.
            if (!r.sample(x, z).biome() || r.vertexW(i) <= t.data().manifest().seaLevelY() + 0.5f)
                continue;
            int y = (int) Math.floor(r.vertexW(i)) - 1;
            Holder<Biome> b = g.getBiomeSource().getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y), QuartPos.fromBlock(z), sampler);
            sampled++;
            // Vanilla's own valley slice (weirdness 0) is river except on the flattest, wettest ground
            // (erosion 6), where it is swamp or mangrove swamp: a planet whose wide rivers cross such
            // lowland (seed 16: 21 of 37 samples) has swamp along them, as vanilla would. A pack may
            // put its own swamp in that slot (Biomes O' Plenty's wetland, under TerraBlender: 12 of 14
            // samples on the planet of 2026-10-02), so any biome of the common tag #c:is_swamp counts.
            if (b.is(BiomeTags.IS_RIVER))
                river++;
            else if (b.is(net.neoforged.neoforge.common.Tags.Biomes.IS_SWAMP) || b.is(net.minecraft.world.level.biome.Biomes.SWAMP)
                    || b.is(net.minecraft.world.level.biome.Biomes.MANGROVE_SWAMP))
                swamp++;
            else if (seen.length() < 400)
                seen.append(' ').append(b.getRegisteredName());
        }
        DeepTime.LOGGER.info("[gametest] riverBiomes sampled={} river={} swamp={} share={} others:{}", sampled, river, swamp,
                sampled == 0 ? "n/a" : String.format("%.3f", (river + swamp) / (double) sampled), seen);
        if (sampled > 0)
            helper.assertTrue(river + swamp >= sampled / 2, "only " + river + " river and " + swamp + " swamp of " + sampled
                    + " wide-channel samples: the valley slice is not reaching the channels");
        helper.succeed();
    }
}
