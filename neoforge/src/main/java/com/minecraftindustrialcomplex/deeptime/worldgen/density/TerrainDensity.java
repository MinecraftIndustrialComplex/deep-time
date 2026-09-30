// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainScale;
import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import com.mojang.serialization.Codec;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import java.lang.ref.WeakReference;
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
 * <p>The scale k grows on steep ground ({@link TerrainScale}): with a fixed k the router's cave
 * band breaks steep faces into steps between noise-cell corners.
 *
 * <p>{@code ratio} mode is k / scale (1 on gentle ground): the router multiplies the cave-entrance
 * term of its terrain band by it, so min(terrain, 5·entrances) carves exactly where it did with the
 * fixed k (both sides shrink by the same factor) and steep faces get no extra cave mouths.
 *
 * <p>{@code initial} mode adds 0.390625: {@code NoiseChunk} takes the preliminary surface as the
 * highest cell y where initial density exceeds 0.390625, so this puts it at H.
 */
public final class TerrainDensity extends DeepTimeDensity {

    public enum Mode implements StringRepresentable {
        INITIAL("initial"), FINAL("final"), RATIO("ratio");

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

    /** Per-thread memo of the last column: vanilla sweeps a column's corners bottom to top. */
    private static final class Memo {
        /** Weak, so a stopped server's world data is not kept alive by idle worker threads. */
        WeakReference<TerrainSource> source = new WeakReference<>(null);
        double baseK = Double.NaN;
        int x = Integer.MIN_VALUE, z = Integer.MIN_VALUE;
        double height, scale;
    }

    private static final ThreadLocal<Memo> MEMO = ThreadLocal.withInitial(Memo::new);

    @Override
    public double compute(FunctionContext ctx) {
        TerrainSource src = source();
        int x = ctx.blockX(), z = ctx.blockZ();
        Memo m = MEMO.get();
        if (m.x != x || m.z != z || m.source.get() != src || m.baseK != k) {
            double h = src.height(x, z);
            m.height = h;
            m.scale = TerrainScale.at(src, x, z, h, k);
            m.x = x;
            m.z = z;
            m.source = new WeakReference<>(src);
            m.baseK = k;
        }
        if (mode == Mode.RATIO)
            return k / m.scale;
        double d = (m.height - ctx.blockY()) / m.scale;
        return mode == Mode.INITIAL ? d + PRELIMINARY_SURFACE_THRESHOLD : d;
    }

    @Override
    public double minValue() {
        if (mode == Mode.RATIO)
            return 0.0;
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
