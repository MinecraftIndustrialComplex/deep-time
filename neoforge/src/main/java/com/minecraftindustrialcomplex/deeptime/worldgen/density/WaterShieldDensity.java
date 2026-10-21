// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import com.minecraftindustrialcomplex.deeptime.worldgen.WaterCaveShield;
import com.mojang.serialization.MapCodec;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.levelgen.DensityFunction;

import java.lang.ref.WeakReference;

/**
 * {@code deeptime:water_shield}: how near column (x, z) lies to Deep Time's water, 1 at and beside
 * it, 0 from {@link WaterCaveShield#FADE_BLOCKS} away ({@link TerrainSource#waterShield}). The
 * {@code deeptime:planet} router lifts its near-surface cave entrances and noodle caves by
 * {@value WaterCaveShield#LIFT} times it, so no cave mouth opens a pit beside a river, a lake or a
 * shallow sea (HYD-078). A 2D function: wrap it in {@code minecraft:flat_cache}.
 */
public final class WaterShieldDensity extends DeepTimeDensity {

    public static final MapCodec<WaterShieldDensity> MAP_CODEC = MapCodec.unit(() -> new WaterShieldDensity(null));
    public static final KeyDispatchDataCodec<WaterShieldDensity> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    public WaterShieldDensity(TerrainSource source) {
        super(source);
    }

    @Override
    public WaterShieldDensity bind(TerrainSource source) {
        return new WaterShieldDensity(source);
    }

    /** The last column asked on this thread: outside a flat cache the router asks it once per y. */
    private static final class Memo {
        /** Weak, so a stopped server's world data is not kept alive by idle worker threads. */
        WeakReference<TerrainSource> source = new WeakReference<>(null);
        int x = Integer.MIN_VALUE, z = Integer.MIN_VALUE;
        double value;
    }

    private static final ThreadLocal<Memo> MEMO = ThreadLocal.withInitial(Memo::new);

    @Override
    public double compute(FunctionContext ctx) {
        TerrainSource src = source();
        int x = ctx.blockX(), z = ctx.blockZ();
        Memo m = MEMO.get();
        if (m.x != x || m.z != z || m.source.get() != src) {
            m.value = src.waterShield(x, z);
            m.x = x;
            m.z = z;
            if (m.source.get() != src)
                m.source = new WeakReference<>(src);
        }
        return m.value;
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
