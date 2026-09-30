// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.climate;

import com.minecraftindustrialcomplex.deeptime.worlddata.ClimateLayers;

import java.util.concurrent.ConcurrentHashMap;

/**
 * The Minecraft-free core of {@link LevelClimate}: a world's {@link ClimateLayers} with the
 * height-independent part of each block column (24 bilinear samples and the Köppen class) cached, so
 * a repeated query only applies the lapse rate. Thread-safe.
 */
final class ClimateColumns {

    /** Cached columns before the cache is dropped and refilled (~0.3 KB each). */
    static final int MAX_COLUMNS = 16_384;

    private final ClimateLayers layers;
    private final ConcurrentHashMap<Long, ClimateLayers.Column> columns = new ConcurrentHashMap<>();

    ClimateColumns(ClimateLayers layers) {
        this.layers = layers;
    }

    ClimateLayers layers() {
        return layers;
    }

    ClimateSample sample(int x, int y, int z) {
        long wx = layers.wrapX(x);
        long key = (wx << 32) ^ (z & 0xffffffffL);
        ClimateLayers.Column col = columns.get(key);
        if (col == null) {
            col = layers.column(wx, z);
            if (columns.size() >= MAX_COLUMNS)
                columns.clear();
            columns.put(key, col);
        }
        ClimateLayers.Sample s = col.at(x, y, z);
        return new ClimateSample(x, y, z, s.elevationM(), layers.lapseCPerKm(), s.tMonth(), s.tMean(), s.tWarm(),
                s.tCold(), s.warmest(), s.coldest(), s.pAnnual(), s.pMonth(), s.koppenCode());
    }
}
