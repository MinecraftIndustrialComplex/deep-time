// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

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
 * <p>The scale k grows on steep ground ({@link #scale}): with the fixed k the router's cave band
 * breaks steep faces into steps between noise-cell corners.
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

    /**
     * The noise-cell spacing, blocks: vanilla evaluates the router at corners this far apart and
     * interpolates between them ({@code size_horizontal} 1).
     */
    static final int CORNER = 4;

    /**
     * Share of the steepest corner-to-corner drop the scale grows to (see {@link #scale});
     * {@code -Ddeeptime.steepScale=0} turns the growth off (the P1-08 router, for comparisons).
     */
    static final double STEEP_SCALE = Double.parseDouble(System.getProperty("deeptime.steepScale", "0.8"));

    /** Per-thread memo of the last column: vanilla sweeps a column's corners bottom to top. */
    private static final class Memo {
        /** Weak, so a stopped server's world data is not kept alive by idle worker threads. */
        WeakReference<TerrainSource> source = new WeakReference<>(null);
        double baseK = Double.NaN;
        int x = Integer.MIN_VALUE, z = Integer.MIN_VALUE;
        double height, scale;
    }

    private static final ThreadLocal<Memo> MEMO = ThreadLocal.withInitial(Memo::new);

    /**
     * The density scale at column (x, z): k where the terrain is gentle, and on steep ground
     * {@value #STEEP_SCALE} × the largest height difference to the eight neighbouring corners.
     *
     * <p>The final density takes the terrain value only within 1.5625·scale blocks of the surface
     * and the cave field below (the router's {@code range_choice}, as vanilla's around
     * {@code sloped_cheese}). Where two corners of a noise cell differ by more than that — any
     * slope over ~3 blocks per block at k = 8 — the lower corner's cell mixes the upper corner's
     * cave value (≈ 0.5, not the terrain's large positive value) into the interpolation, the zero
     * crossing moves up-slope and the face breaks into three-block benches and 20-block steps
     * every four blocks: the "striped snow" of the P1-08 renders (vertical stripes on an
     * east-facing flank; a column's heights read 308, 308, 307, 287, 284, 283, 282, 275, 256, …).
     * Growing the scale with the local drop keeps every corner of such a cell in the terrain band,
     * so the interpolated surface stays the bilinear H between corners. The surface at a corner is
     * H whatever the scale; on gentle ground (drops under 10 blocks) nothing changes.
     */
    double scale(TerrainSource src, int x, int z, double h) {
        double drop = 0.0;
        for (int dz = -CORNER; dz <= CORNER; dz += CORNER) {
            for (int dx = -CORNER; dx <= CORNER; dx += CORNER) {
                if (dx == 0 && dz == 0)
                    continue;
                drop = Math.max(drop, Math.abs(src.height(x + dx, z + dz) - h));
            }
        }
        return Math.max(k, STEEP_SCALE * drop);
    }

    @Override
    public double compute(FunctionContext ctx) {
        TerrainSource src = source();
        int x = ctx.blockX(), z = ctx.blockZ();
        Memo m = MEMO.get();
        if (m.x != x || m.z != z || m.source.get() != src || m.baseK != k) {
            double h = src.height(x, z);
            m.height = h;
            m.scale = scale(src, x, z, h);
            m.x = x;
            m.z = z;
            m.source = new WeakReference<>(src);
            m.baseK = k;
        }
        double d = (m.height - ctx.blockY()) / m.scale;
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
