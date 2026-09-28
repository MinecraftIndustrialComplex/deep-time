// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.util.StringRepresentable;

/**
 * {@code deeptime:terrain {mode, k}}: the terrain as a density, (H(x, z) − y) / k.
 *
 * <p>Linear in y, so vanilla's interpolation over 8-block cells is exact and the surface lands at
 * y = H at every cell-corner column (plan §7.1). k = 8 gives about vanilla's density gradient near
 * the surface (sloped_cheese ≈ 4·factor/128 per block), which keeps the cave-entrance band of the
 * final density (terrain < 1.5625) about as deep as vanilla's.
 *
 * <p>{@code initial} mode adds 0.390625: {@code NoiseChunk} takes the preliminary surface as the
 * highest cell y where initial density exceeds 0.390625, so this puts it at H.
 */
public final class TerrainDensity extends DeepTimeDensity {

    public enum Mode implements StringRepresentable {
        INITIAL("initial"), FINAL("final");

        public static final Codec<Mode> CODEC = StringRepresentable.fromEnum(Mode::values);
        private final String name;

        Mode(String name) {
            this.name = name;
        }

        @Override
        public String getSerializedName() {
            return name;
        }
    }

    /** The preliminary-surface threshold of {@code NoiseChunk.computePreliminarySurfaceLevel}. */
    static final double PRELIMINARY_SURFACE_THRESHOLD = 0.390625;

    public static final MapCodec<TerrainDensity> MAP_CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
            Mode.CODEC.fieldOf("mode").forGetter(d -> d.mode),
            Codec.doubleRange(0.5, 1024).optionalFieldOf("k", 8.0).forGetter(d -> d.k)
    ).apply(i, (mode, k) -> new TerrainDensity(null, mode, k)));
    public static final KeyDispatchDataCodec<TerrainDensity> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    private final Mode mode;
    private final double k;

    public TerrainDensity(TerrainSource source, Mode mode, double k) {
        super(source);
        this.mode = mode;
        this.k = k;
    }

    @Override
    public TerrainDensity bind(TerrainSource source) {
        return new TerrainDensity(source, mode, k);
    }

    @Override
    public double compute(FunctionContext ctx) {
        double d = (source().height(ctx.blockX(), ctx.blockZ()) - ctx.blockY()) / k;
        return mode == Mode.INITIAL ? d + PRELIMINARY_SURFACE_THRESHOLD : d;
    }

    @Override
    public double minValue() {
        return -2048.0 / k;
    }

    @Override
    public double maxValue() {
        return 2048.0 / k + PRELIMINARY_SURFACE_THRESHOLD;
    }

    @Override
    public KeyDispatchDataCodec<? extends net.minecraft.world.level.levelgen.DensityFunction> codec() {
        return CODEC;
    }
}
