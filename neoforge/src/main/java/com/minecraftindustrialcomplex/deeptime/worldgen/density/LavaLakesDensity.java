// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.LavaRules;
import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.levelgen.DensityFunction;

/**
 * {@code deeptime:lava_lakes {min_y}}: 1 where a cave lake may be lava, else 0. The router's
 * {@code lava} function is this gate times vanilla's {@code aquifer_lava} noise.
 *
 * <p>Vanilla's aquifer makes a cave lake lava when its surface is at y −10 or lower and the
 * router's {@code lava} function exceeds 0.3 in magnitude, and y −10 is hard-coded
 * ({@code Aquifer.computeFluidType}): right 54 blocks above a floor at y −64, but in a world whose
 * floor is y −352 it would allow lava lakes through 342 blocks of rock. The aquifer samples the
 * function at <b>cell</b> coordinates, not block coordinates: (⌊x/64⌋, ⌊y/40⌋, ⌊z/64⌋). So
 * {@link DensityFunction.FunctionContext#blockY} here is the lake's 40-block level cell, and the
 * gate is {@link LavaRules#lakeCell} for the level's rules: zero in every cell centred more than
 * {@code underground.lava_lake_blocks} above the floor, which no noise value turns into lava.
 *
 * <p>It is a density function rather than a mixin on the aquifer so that it holds whatever
 * replaces the aquifer's code (C2ME rewrites it) and so a datapack can see and change it.
 */
public final class LavaLakesDensity extends DeepTimeDensity {

    public static final MapCodec<LavaLakesDensity> MAP_CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
            Codec.INT.fieldOf("min_y").forGetter(d -> d.minY)
    ).apply(i, minY -> new LavaLakesDensity(null, minY)));
    public static final KeyDispatchDataCodec<LavaLakesDensity> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    private final int minY;

    public LavaLakesDensity(TerrainSource source, int minY) {
        super(source);
        this.minY = minY;
    }

    @Override
    public LavaLakesDensity bind(TerrainSource source) {
        return new LavaLakesDensity(source, minY);
    }

    @Override
    public double compute(FunctionContext ctx) {
        return source().lava().lakeCell(minY, ctx.blockY()) ? 1.0 : 0.0;
    }

    @Override
    public double minValue() {
        return 0.0;
    }

    @Override
    public double maxValue() {
        return 1.0;
    }

    @Override
    public KeyDispatchDataCodec<? extends DensityFunction> codec() {
        return CODEC;
    }
}
