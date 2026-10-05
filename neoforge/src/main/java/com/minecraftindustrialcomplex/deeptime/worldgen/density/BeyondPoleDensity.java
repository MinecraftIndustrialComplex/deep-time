// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import com.mojang.serialization.MapCodec;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.levelgen.DensityFunction;

/**
 * {@code deeptime:beyond_pole}: 1 where |z| ≥ R = C/2π (past the pole edge, {@link
 * com.minecraftindustrialcomplex.deeptime.worlddata.Projection#poleZ}), else 0. Nobody reaches
 * it through the soft cap, but chunks there are still generated at view distance; the router
 * uses this mask to turn them into an ice-sheet ocean (plan §7.1, §7.11). On a seamless
 * Petroff–Guyou world there is no "beyond the pole" (the poles lie on the turning edges and past an
 * edge is the folded inside), so the mask is 0 everywhere there.
 */
public final class BeyondPoleDensity extends DeepTimeDensity {

    public static final MapCodec<BeyondPoleDensity> MAP_CODEC = MapCodec.unit(() -> new BeyondPoleDensity(null));
    public static final KeyDispatchDataCodec<BeyondPoleDensity> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    public BeyondPoleDensity(TerrainSource source) {
        super(source);
    }

    @Override
    public BeyondPoleDensity bind(TerrainSource source) {
        return new BeyondPoleDensity(source);
    }

    @Override
    public double compute(FunctionContext ctx) {
        if (source().seamless())
            return 0.0;
        int pole = (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(source().circumference());
        int z = ctx.blockZ();
        return z >= pole || z < -pole ? 1.0 : 0.0;
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
