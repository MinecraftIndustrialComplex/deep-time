// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;

/** Chooses a level's terrain source from its {@link WorldDataRef} (plan §7.1). */
public final class TerrainSources {

    private TerrainSources() {}

    public static TerrainSource forRef(WorldDataRef ref) {
        var test = ref.effectiveTestTerrain();
        if (test.isPresent()) {
            if (ref.testTerrain().isEmpty())
                DeepTime.LOGGER.warn("Using the analytic test terrain '{}' from -Ddeeptime.testTerrain for world data '{}' "
                        + "(development only)", test.get(), ref.folder());
            return AnalyticTestTerrain.of(test.get(), ref.circumferenceBlocks());
        }
        return new WorldDataTerrain(ref);
    }
}
