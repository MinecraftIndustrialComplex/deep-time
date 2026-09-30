// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.climate;

import com.minecraftindustrialcomplex.deeptime.worlddata.ClimateLayers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;

import java.util.Collections;
import java.util.Map;
import java.util.Optional;
import java.util.WeakHashMap;

/**
 * {@link DeepTimeClimate} over a level's world data: one per world data, kept while the world data
 * lives, answering through {@link ClimateColumns}.
 */
final class LevelClimate implements DeepTimeClimate {

    private static final Map<WorldData, LevelClimate> BY_DATA = Collections.synchronizedMap(new WeakHashMap<>());
    private static final LevelClimate NONE = new LevelClimate(false, null, 63);

    private final boolean deepTime;
    private final ClimateColumns columns;
    private final int seaLevelY;

    private LevelClimate(boolean deepTime, ClimateColumns columns, int seaLevelY) {
        this.deepTime = deepTime;
        this.columns = columns;
        this.seaLevelY = seaLevelY;
    }

    static DeepTimeClimate of(Level level) {
        if (!(level instanceof ServerLevel server)
                || !(server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g))
            return NONE;
        if (!(g.terrain() instanceof WorldDataTerrain t))
            return new LevelClimate(true, null, g.getSeaLevel());
        return BY_DATA.computeIfAbsent(t.data(), d -> {
            ClimateLayers l = ClimateLayers.of(d);
            return new LevelClimate(true, l == null ? null : new ClimateColumns(l), d.manifest().seaLevelY());
        });
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
    public double lapseRateCPerKm() {
        return columns == null ? Double.NaN : columns.layers().lapseCPerKm();
    }

    @Override
    public int seaLevelY() {
        return seaLevelY;
    }
}
