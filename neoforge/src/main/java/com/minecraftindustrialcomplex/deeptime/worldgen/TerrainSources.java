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
        // World data arrives in P0-10 (WorldDataTerrain). Until then a Deep Time level without a
        // test terrain has nothing to generate from: fail loudly rather than build flat garbage.
        throw new IllegalStateException("Deep Time level '" + ref.folder() + "' has no world data reader yet (P0-10) and no "
                + "test terrain; start the server with -Ddeeptime.testTerrain=" + AnalyticTestTerrain.WRAP_TEST);
    }
}
