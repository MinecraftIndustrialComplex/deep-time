// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.SectionPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunkSection;

import java.io.IOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Paints rock strata into the unclassified crust (plan §7.5), v1. Runs after the pack's surface
 * rules, which only repaint the default block, and before carvers and features. Only
 * {@code minecraft:stone} and {@code minecraft:deepslate} are replaced, so soil, sand, water, ore and
 * anything a surface rule placed stay.
 *
 * <p><b>Columns.</b> Each column reads the horizon stack: horizon tops interpolated bilinearly
 * between the four surrounding strata cells (the nearest cell's top where a corner lacks that
 * horizon), classes from the nearest cell. With integer tops T_h = ⌊top_h⌋, horizon h fills the
 * blocks T_{h+1} &lt; y ≤ T_h, the last one down to the bottom of the world.
 *
 * <p><b>Blocks.</b> {@link RockPalette} resolves each class; a class with several members uses one
 * member per horizon, picked by a hash of the horizon's key ({@link #horizonKey}), so a layer is one
 * block type. A class that resolves to nothing leaves the crust as it is.
 *
 * <p><b>Writes.</b> Whole runs go straight into the section palettes (sections holding no stone or
 * deepslate are skipped). Bodies (intrusions, alteration halos, gossans; {@code bodies/}, P1-14)
 * then override horizons inside their SDF where they name a rock class ({@link #paintBodies},
 * P1-15).
 */
public final class StrataPass {

    public static final String LAYER = "strata/horizons";
    private static final Map<WorldData, List<String>> CLASSES = new ConcurrentHashMap<>();

    private StrataPass() {}

    /**
     * A horizon's key for the member choice. World data v0 has no horizon ids, so a horizon is
     * named by its stack index and class; P1-08's format can give real unit ids.
     */
    public static long horizonKey(int h, int classIndex) {
        return ((long) h << 8) | classIndex;
    }

    /** One column's horizons, top-down: integer tops and class indexes. */
    public static final class Column {
        public final int[] tops;
        public final int[] classes;
        public int count;

        public Column(int k) {
            tops = new int[k];
            classes = new int[k];
        }

        /** The index of the horizon holding block y, or −1 above the first horizon's top. */
        public int horizonAt(int y) {
            if (count == 0 || y > tops[0])
                return -1;
            int h = 0;
            while (h + 1 < count && y <= tops[h + 1])
                h++;
            return h;
        }
    }

    /** Fills {@code out} with the stack at block (x, z). */
    public static void column(WorldData wd, RasterSampler strata, int x, int z, Column out) {
        var grid = strata.desc().grid();
        int c = wd.circumference();
        int k = Math.min(strata.stackDepth(), out.tops.length);
        double wx = Math.floorMod(x + c / 2, c) - c / 2;
        double u = (wx - grid.originBlockX()) / grid.r();
        double v = (z - grid.originBlockZ()) / (double) grid.r();
        long in = (long) Math.floor(u + 0.5), jn = (long) Math.floor(v + 0.5);
        long i0 = (long) Math.floor(u), j0 = (long) Math.floor(v);
        double fu = u - i0, fv = v - j0;
        int used = 0;
        int prev = Integer.MAX_VALUE;
        for (int h = 0; h < k; h++) {
            int cls = strata.horizonClass(in, jn, h);
            double nearest = strata.horizonTop(in, jn, h);
            if (cls == 255 || Double.isNaN(nearest))
                break;
            double t00 = strata.horizonTop(i0, j0, h), t10 = strata.horizonTop(i0 + 1, j0, h);
            double t01 = strata.horizonTop(i0, j0 + 1, h), t11 = strata.horizonTop(i0 + 1, j0 + 1, h);
            double top = Double.isNaN(t00) || Double.isNaN(t10) || Double.isNaN(t01) || Double.isNaN(t11) ? nearest
                    : (t00 + (t10 - t00) * fu) * (1 - fv) + (t01 + (t11 - t01) * fu) * fv;
            int t = Math.min((int) Math.floor(top), prev);   // tops never rise with depth
            out.tops[used] = t;
            out.classes[used] = cls;
            prev = t;
            used++;
        }
        out.count = used;
    }

    public static void apply(ChunkAccess chunk, WorldData wd) {
        if (!wd.has(LAYER))
            return;
        RasterSampler strata = wd.layer(LAYER);
        List<String> classIds = classes(wd);
        PaletteRules.Result<BlockState>[] byIndex = RockPalette.get().forClasses(classIds);
        int k = strata.stackDepth();
        int minY = chunk.getMinBuildHeight(), maxY = chunk.getMaxBuildHeight() - 1;
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();

        // Runs per column: [runTop, runBottom] → block, top-down.
        int[] runTop = new int[256 * k], runBottom = new int[256 * k], runs = new int[256];
        BlockState[] runBlock = new BlockState[256 * k];
        Column col = new Column(k);
        int highest = Integer.MIN_VALUE;
        for (int lx = 0; lx < 16; lx++) {
            for (int lz = 0; lz < 16; lz++) {
                column(wd, strata, x0 + lx, z0 + lz, col);
                int ci = lx * 16 + lz, n = 0;
                for (int h = 0; h < col.count; h++) {
                    int top = Math.min(col.tops[h], maxY);
                    int bottom = h + 1 < col.count ? col.tops[h + 1] + 1 : minY;
                    bottom = Math.max(bottom, minY);
                    int cls = col.classes[h];
                    var result = cls < byIndex.length ? byIndex[cls] : null;
                    BlockState rock = result == null ? null : result.pick(horizonKey(h, cls));
                    if (rock == null || top < bottom)
                        continue;   // unclassified crust stays
                    runTop[ci * k + n] = top;
                    runBottom[ci * k + n] = bottom;
                    runBlock[ci * k + n] = rock;
                    n++;
                    highest = Math.max(highest, top);
                }
                runs[ci] = n;
            }
        }
        if (highest != Integer.MIN_VALUE)
            writeRuns(chunk, highest, k, runTop, runBottom, runBlock, runs);
        paintBodies(chunk, wd, k, runTop, runBottom, runBlock, runs);
    }

    private static void writeRuns(ChunkAccess chunk, int highest, int k, int[] runTop, int[] runBottom,
                                  BlockState[] runBlock, int[] runs) {
        for (int si = chunk.getSectionIndex(highest); si >= 0; si--) {
            LevelChunkSection section = chunk.getSection(si);
            if (section.hasOnlyAir() || !section.maybeHas(StrataPass::isCrust))
                continue;
            int sy0 = SectionPos.sectionToBlockCoord(chunk.getSectionYFromSectionIndex(si)), sy1 = sy0 + 15;
            for (int ci = 0; ci < 256; ci++) {
                int lx = ci >> 4, lz = ci & 15;
                for (int r = 0; r < runs[ci]; r++) {
                    int top = Math.min(runTop[ci * k + r], sy1), bottom = Math.max(runBottom[ci * k + r], sy0);
                    BlockState rock = runBlock[ci * k + r];
                    for (int y = top; y >= bottom; y--) {
                        BlockState here = section.getBlockState(lx, y & 15, lz);
                        if (here != rock && isCrust(here))
                            section.setBlockState(lx, y & 15, lz, rock, false);
                    }
                }
            }
        }
    }

    /** A body's key for the member choice: distinct from every horizon key. */
    public static long bodyKey(int bodyId) {
        return (1L << 48) | Integer.toUnsignedLong(bodyId);
    }

    /**
     * Bodies override horizons inside their SDF (plan §7.5): where a body (or one of its zones) names
     * a rock class, the crust or horizon rock at that block becomes the class's block (one member
     * per body, {@link #bodyKey}). A zone's class wins over the body's; of several zones the last
     * that applies. Soil, sand, water and anything the surface rules placed stay, as for horizons.
     */
    static void paintBodies(ChunkAccess chunk, WorldData wd, int k, int[] runTop, int[] runBottom,
                            BlockState[] runBlock, int[] runs) {
        int minY = chunk.getMinBuildHeight(), maxY = chunk.getMaxBuildHeight() - 1;
        int cx = chunk.getPos().x, cz = chunk.getPos().z, x0 = cx << 4, z0 = cz << 4;
        var bodies = wd.deposits().bodiesInChunk(cx, cz, minY, maxY);
        if (bodies.isEmpty())
            return;
        int c = wd.circumference();
        RockPalette palette = RockPalette.get();
        for (var b : bodies) {
            boolean anyClass = b.rockClass() != null;
            for (var z : b.zones())
                anyClass |= z.rockClass() != null;
            if (!anyClass)
                continue;
            long key = bodyKey(b.id());
            int yLo = Math.max(minY, b.min()[1]), yHi = Math.min(maxY, b.max()[1]);
            for (int lx = 0; lx < 16; lx++) {
                for (int lz = 0; lz < 16; lz++) {
                    int x = x0 + lx, z = z0 + lz;
                    if (!b.aabbContains(c, x, yLo, z) && !b.aabbContains(c, x, yHi, z))
                        continue;
                    int ci = lx * 16 + lz;
                    for (int y = yLo; y <= yHi; y++) {
                        var at = b.locate(c, x, y, z);
                        if (at == null)
                            continue;
                        String cls = b.rockClass();
                        for (var zone : b.zones())
                            if (zone.rockClass() != null && zone.contains(at.d(), at.v()))
                                cls = zone.rockClass();
                        if (cls == null)
                            continue;
                        BlockState rock = palette.block(cls, key);
                        if (rock == null)
                            continue;
                        LevelChunkSection section = chunk.getSection(chunk.getSectionIndex(y));
                        BlockState here = section.getBlockState(lx, y & 15, lz);
                        if (here == rock)
                            continue;
                        if (isCrust(here) || here == horizonBlock(ci, y, k, runTop, runBottom, runBlock, runs))
                            section.setBlockState(lx, y & 15, lz, rock, false);
                    }
                }
            }
        }
    }

    /**
     * The rock the strata pass paints at (x, y, z) where the crust is: the horizon's block, or a
     * body's where one names a class there (as {@link #paintBodies} decides). Null above the
     * horizons or where nothing resolves (the crust stays). For tests and probes.
     */
    public static BlockState predictedRock(WorldData wd, int x, int y, int z) {
        RockPalette palette = RockPalette.get();
        BlockState rock = null;
        if (wd.has(LAYER)) {
            var h = horizonAt(wd, x, y, z);
            if (h.isPresent())
                rock = palette.block(h.get().rockClass(), h.get().key());
        }
        int c = wd.circumference();
        for (var b : wd.deposits().bodiesIn(x, y, z, x, y, z)) {
            var at = b.locate(c, x, y, z);
            if (at == null)
                continue;
            String cls = b.rockClass();
            for (var zone : b.zones())
                if (zone.rockClass() != null && zone.contains(at.d(), at.v()))
                    cls = zone.rockClass();
            if (cls == null)
                continue;
            BlockState r = palette.block(cls, bodyKey(b.id()));
            if (r != null)
                rock = r;
        }
        return rock;
    }

    /** The block the horizon runs put at (column ci, y), or null. */
    private static BlockState horizonBlock(int ci, int y, int k, int[] runTop, int[] runBottom, BlockState[] runBlock,
                                           int[] runs) {
        for (int r = 0; r < runs[ci]; r++)
            if (y <= runTop[ci * k + r] && y >= runBottom[ci * k + r])
                return runBlock[ci * k + r];
        return null;
    }

    /** The unclassified crust the router and surface rules leave behind. */
    public static boolean isCrust(BlockState s) {
        return s.is(Blocks.STONE) || s.is(Blocks.DEEPSLATE);
    }

    /** Class index → rock class id, from {@code strata/classes.json}. */
    public static List<String> classes(WorldData wd) {
        return CLASSES.computeIfAbsent(wd, d -> {
            try {
                var arr = JsonParser.parseString(Files.readString(d.folder().resolve("strata/classes.json")))
                        .getAsJsonObject().getAsJsonArray("classes");
                List<String> out = new ArrayList<>();
                arr.forEach(e -> out.add(e.getAsString()));
                return List.copyOf(out);
            } catch (IOException | RuntimeException e) {
                throw new IllegalStateException("cannot read strata/classes.json in " + d.folder(), e);
            }
        });
    }

    /** A horizon at a block, for tests and probes. */
    public record Horizon(int index, int classIndex, String rockClass, int top, int bottom) {
        public long key() {
            return horizonKey(index, classIndex);
        }
    }

    /** The horizon holding (x, y, z) per the stack; empty above the first horizon's top. */
    public static Optional<Horizon> horizonAt(WorldData wd, int x, int y, int z) {
        RasterSampler strata = wd.layer(LAYER);
        Column col = new Column(strata.stackDepth());
        column(wd, strata, x, z, col);
        int h = col.horizonAt(y);
        if (h < 0)
            return Optional.empty();
        int bottom = h + 1 < col.count ? col.tops[h + 1] + 1 : Integer.MIN_VALUE;
        return Optional.of(new Horizon(h, col.classes[h], classes(wd).get(col.classes[h]), col.tops[h], bottom));
    }

    /** The class at (x, y, z) per the stack; empty above the top horizon. */
    public static Optional<String> classAt(WorldData wd, int x, int y, int z) {
        return horizonAt(wd, x, y, z).map(Horizon::rockClass);
    }
}
