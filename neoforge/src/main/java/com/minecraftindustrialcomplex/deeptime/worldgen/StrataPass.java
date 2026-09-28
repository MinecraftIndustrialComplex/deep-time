// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
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
 * Paints rock strata into the unclassified crust (plan §7.5), v0: horizon stacks only, blocks from
 * {@link RockPalette}. Runs after the pack's surface rules, which only repaint the default block,
 * and before carvers and features. Only {@code minecraft:stone} and {@code minecraft:deepslate} are
 * replaced, so soil, sand, water, ore and anything a surface rule placed stay. Each column takes the
 * nearest strata cell's stack: horizon k fills from its top down to horizon k+1's top, the last one
 * down to the floor. Bodies (intrusions, deposits) come with P1-12/P1-15.
 */
public final class StrataPass {

    public static final String LAYER = "strata/horizons";
    private static final Map<WorldData, List<String>> CLASSES = new ConcurrentHashMap<>();

    private StrataPass() {}

    public static void apply(ChunkAccess chunk, WorldData wd) {
        if (!wd.has(LAYER))
            return;
        RasterSampler strata = wd.layer(LAYER);
        var grid = strata.desc().grid();
        List<String> classes = classes(wd);
        int k = strata.stackDepth();
        BlockState[] blocks = new BlockState[k];
        double[] tops = new double[k];
        int minY = chunk.getMinBuildHeight();
        int c = wd.circumference();
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        for (int lx = 0; lx < 16; lx++) {
            for (int lz = 0; lz < 16; lz++) {
                int x = x0 + lx, z = z0 + lz;
                int wx = Math.floorMod(x + c / 2, c) - c / 2;
                long i = Math.round((wx - grid.originBlockX()) / (double) grid.r());
                long j = Math.round((z - grid.originBlockZ()) / (double) grid.r());
                int used = 0;
                for (int h = 0; h < k; h++) {
                    int cls = strata.horizonClass(i, j, h);
                    double top = strata.horizonTop(i, j, h);
                    if (cls == 255 || Double.isNaN(top) || cls >= classes.size())
                        break;
                    tops[used] = top;
                    blocks[used] = RockPalette.resolve(classes.get(cls)).orElse(null);
                    used++;
                }
                if (used == 0)
                    continue;
                int yTop = (int) Math.floor(tops[0]);
                for (int y = Math.min(yTop, chunk.getMaxBuildHeight() - 1); y >= minY; y--) {
                    int h = 0;
                    while (h + 1 < used && y < tops[h + 1])
                        h++;
                    BlockState rock = blocks[h];
                    if (rock == null)
                        continue;
                    LevelChunkSection section = chunk.getSection(chunk.getSectionIndex(y));
                    BlockState here = section.getBlockState(lx, y & 15, lz);
                    if (here.is(Blocks.STONE) || here.is(Blocks.DEEPSLATE))
                        section.setBlockState(lx, y & 15, lz, rock, false);
                }
            }
        }
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

    /** The class at (x, y, z) per the stack, for tests and probes; empty above the top horizon. */
    public static Optional<String> classAt(WorldData wd, int x, int y, int z) {
        RasterSampler strata = wd.layer(LAYER);
        var grid = strata.desc().grid();
        int c = wd.circumference();
        int wx = Math.floorMod(x + c / 2, c) - c / 2;
        long i = Math.round((wx - grid.originBlockX()) / (double) grid.r());
        long j = Math.round((z - grid.originBlockZ()) / (double) grid.r());
        List<String> classes = classes(wd);
        String found = null;
        for (int h = 0; h < strata.stackDepth(); h++) {
            int cls = strata.horizonClass(i, j, h);
            double top = strata.horizonTop(i, j, h);
            if (cls == 255 || Double.isNaN(top))
                break;
            if (y < Math.floor(top) + 1)
                found = classes.get(cls);
        }
        return Optional.ofNullable(found);
    }
}
