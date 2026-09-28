// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import com.mojang.serialization.MapCodec;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.levelgen.DensityFunction;

/**
 * {@code deeptime:depth}: the biome {@code depth} input, (H(x, z) − y) / 128 — zero at the
 * surface and falling 1/128 per block like vanilla's {@code overworld/depth} (3.0 over 384
 * blocks), so surface and cave biome bands keep their meaning (plan §7.1).
 */
public final class DepthDensity extends DeepTimeDensity {

    public static final MapCodec<DepthDensity> MAP_CODEC = MapCodec.unit(() -> new DepthDensity(null));
    public static final KeyDispatchDataCodec<DepthDensity> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    public DepthDensity(TerrainSource source) {
        super(source);
    }

    @Override
    public DepthDensity bind(TerrainSource source) {
        return new DepthDensity(source);
    }

    @Override
    public double compute(FunctionContext ctx) {
        return (source().height(ctx.blockX(), ctx.blockZ()) - ctx.blockY()) / 128.0;
    }

    @Override
    public double minValue() {
        return -16.0;
    }

    @Override
    public double maxValue() {
        return 16.0;
    }

    @Override
    public KeyDispatchDataCodec<? extends DensityFunction> codec() {
        return CODEC;
    }
}
