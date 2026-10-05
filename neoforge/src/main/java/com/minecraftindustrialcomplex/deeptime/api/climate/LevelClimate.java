// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.climate;

import com.minecraftindustrialcomplex.deeptime.net.ClientPlanetState;
import com.minecraftindustrialcomplex.deeptime.worlddata.ClimateLayers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.Geothermal;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;

import java.util.Collections;
import java.util.Map;
import java.util.Optional;
import java.util.WeakHashMap;
import java.util.function.Supplier;

/**
 * {@link DeepTimeClimate} over a level's world data: one per world data, kept while the world data
 * lives, answering through {@link ClimateColumns}.
 */
final class LevelClimate implements DeepTimeClimate {

    private static final Map<WorldData, LevelClimate> BY_DATA = Collections.synchronizedMap(new WeakHashMap<>());
    private static final LevelClimate NONE = new LevelClimate(false, null, 63, null);

    private final boolean deepTime;
    private final ClimateColumns columns;
    private final int seaLevelY;
    private final Supplier<Geothermal> geothermal;

    private LevelClimate(boolean deepTime, ClimateColumns columns, int seaLevelY, Supplier<Geothermal> geothermal) {
        this.deepTime = deepTime;
        this.columns = columns;
        this.seaLevelY = seaLevelY;
        this.geothermal = geothermal;
    }

    static DeepTimeClimate of(Level level) {
        if (!(level instanceof ServerLevel server)
                || !(server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g))
            return NONE;
        if (!(g.terrain() instanceof WorldDataTerrain t))
            return new LevelClimate(true, null, g.getSeaLevel(), null);
        return BY_DATA.computeIfAbsent(t.data(), d -> {
            ClimateLayers l = ClimateLayers.of(d);
            return new LevelClimate(true, l == null ? null : new ClimateColumns(l), d.manifest().seaLevelY(),
                    () -> t.geothermal(g.waterReach(), g.generatorSettings().value().noiseSettings().minY(), g.getSeaLevel()));
        });
    }

    /**
     * {@link DeepTimeClimate#circumferenceBlocks}: the generator's on a server level, the planet info's
     * on a client level (both allocation-free), 0 for anything else.
     */
    static int circumferenceBlocks(Level level) {
        if (level == null)
            return 0;
        if (level.isClientSide())
            return ClientPlanetState.circumference(level);
        if (level instanceof ServerLevel server && server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)
            return g.dataRef().circumferenceBlocks();
        return 0;
    }

    /** {@link DeepTimeClimate#projectionKind}. */
    static String projectionKind(Level level) {
        if (level == null)
            return "";
        if (level.isClientSide()) {
            var p = ClientPlanetState.projection(level);
            return p == null ? "" : p.kind().id();
        }
        if (level instanceof ServerLevel server && server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)
            return g.dataRef().projection();
        return "";
    }

    /**
     * The world projection of {@code level}: the world data's on a server (Lambert of the generator's C
     * on a test terrain), the planet info's on a client; null for anything else.
     */
    static com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection projection(Level level) {
        if (level == null)
            return null;
        if (level.isClientSide())
            return ClientPlanetState.projection(level);
        if (level instanceof ServerLevel server && server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) {
            if (g.terrain() instanceof WorldDataTerrain t)
                return t.data().projection();
            return new com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Lambert(g.dataRef().circumferenceBlocks());
        }
        return null;
    }

    @Override
    public boolean isDeepTimeLevel() {
        return deepTime;
    }

    @Override
    public boolean hasClimate() {
        return columns != null;
    }

    @Override
    public Optional<ClimateSample> at(BlockPos pos) {
        return at(pos.getX(), pos.getY(), pos.getZ());
    }

    @Override
    public Optional<ClimateSample> at(int x, int y, int z) {
        return columns == null ? Optional.empty() : Optional.of(columns.sample(x, y, z));
    }

    @Override
    public double geothermalWarmthC(int x, int y, int z) {
        return geothermal == null ? 0.0 : geothermal.get().warmthC(x, y, z);   // folds x across the seam itself
    }

    @Override
    public double lapseRateCPerKm() {
        return columns == null ? Double.NaN : columns.layers().lapseCPerKm();
    }

    @Override
    public int seaLevelY() {
        return seaLevelY;
    }
}
