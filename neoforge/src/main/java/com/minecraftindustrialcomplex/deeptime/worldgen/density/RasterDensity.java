// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.levelgen.DensityFunction;

/**
 * {@code deeptime:raster {layer}}: a 2D layer of the level's terrain source, sampled at (x, z)
 * and wrapping in x. In Phase 0 the analytic test terrain answers; from P0-10 the world data's
 * tile packs do (plan §7.1).
 */
public final class RasterDensity extends DeepTimeDensity {

    public static final MapCodec<RasterDensity> MAP_CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
            Codec.STRING.fieldOf("layer").forGetter(d -> d.layer)
    ).apply(i, layer -> new RasterDensity(null, layer)));
    public static final KeyDispatchDataCodec<RasterDensity> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    private final String layer;

    public RasterDensity(TerrainSource source, String layer) {
        super(source);
        this.layer = layer;
    }

    @Override
    public RasterDensity bind(TerrainSource source) {
        return new RasterDensity(source, layer);
    }

    @Override
    public double compute(FunctionContext ctx) {
        return source().raster(layer, ctx.blockX(), ctx.blockZ());
    }

    @Override
    public double minValue() {
        return -2.0;
    }

    @Override
    public double maxValue() {
        return 2.0;
    }

    @Override
    public KeyDispatchDataCodec<? extends DensityFunction> codec() {
        return CODEC;
    }
}
