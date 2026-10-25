// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.toroidalworld.api.v1.option.GenerationOptions;
import com.toroidalworld.api.v1.shape.LoopSpans;
import com.toroidalworld.api.v1.shape.ShapeDimensions;
import net.minecraft.core.Direction;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;

/**
 * Gives a Deep Time world its planet shape through Toroidal World (plan §7.10): the overworld
 * loops in X with period C (the generator's {@code circumference_blocks}) and not in Z; the
 * Nether loops at exactly 1:8; the End stays vanilla.
 *
 * <p>Toroidal World's own {@code ShapeModule} path only runs from its Create World button and
 * never on a dedicated server (P0-06 lookup of TW 0.19.3), so Deep Time shapes its worlds itself,
 * from {@code WorldDimensions.bake} ({@code WorldDimensionsShapeMixin}). That runs for a new
 * singleplayer world, a server's first start with {@code level-type=deeptime:planet_cube}, and every
 * reload. A world that already carries a shape (restored from level.dat by TW) keeps it.
 *
 * <p>Climate compression is left at TW's default: it pattern-matches vanilla's climate density
 * functions, and Deep Time's climate comes from {@code deeptime:raster}, so it never applies.
 */
public final class PlanetShape {

    /** The Nether loops at the portal scale, so portal coordinates stay consistent across the seam. */
    public static final int NETHER_SCALE = 8;

    private PlanetShape() {}

    public static WorldDimensions shape(WorldDimensions dims) {
        LevelStem overworld = dims.dimensions().get(LevelStem.OVERWORLD);
        if (overworld == null || !(overworld.generator() instanceof GeologicChunkGenerator g))
            return dims;
        int chunks = g.dataRef().circumferenceBlocks() / 16;
        LoopSpans existing = ShapeDimensions.spansOf(dims, LevelStem.OVERWORLD);
        if (existing != null) {
            if (!existing.loops(Direction.Axis.X) || existing.loops(Direction.Axis.Z)
                    || existing.chunkWidth(Direction.Axis.X) != chunks)
                DeepTime.LOGGER.warn("Deep Time overworld carries a Toroidal World shape {} that is not the planet's "
                        + "X-only loop of {} chunks; keeping the stored shape", existing, chunks);
            return dims;
        }
        LoopSpans spans = LoopSpans.ofWidth(Direction.Axis.X, chunks);
        DeepTime.LOGGER.info("Shaping the Deep Time overworld: X loops over {} chunks (C = {}), Z does not; "
                + "Nether 1:{}, End unlooped", chunks, chunks * 16, NETHER_SCALE);
        return ShapeDimensions.withSpans(dims, spans, spans.scaledDown(NETHER_SCALE), LoopSpans.NONE, GenerationOptions.DEFAULT);
    }
}
