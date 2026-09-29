// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.cdg;

import com.jesz.createdieselgenerators.CDGConfig;
import com.jesz.createdieselgenerators.world.OilChunksSavedData;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;

/**
 * {@link CdgAccess} over Create: Diesel Generators 1.3.15 (MIT; compiled against it as
 * {@code compileOnly}, loaded only when CDG is installed). Signatures checked with javap on the MIC
 * pack's jar: {@code OilChunksSavedData.getBaseOilAmount(ServerLevel, ChunkPos)} and
 * {@code getChunkOilAmount(ServerLevel, ChunkPos)}, both {@code public static int}.
 */
final class CdgAccessImpl implements CdgAccess {

    CdgAccessImpl() {}

    @Override
    public int base(ServerLevel level, ChunkPos pos) {
        return OilChunksSavedData.getBaseOilAmount(level, pos);
    }

    @Override
    public int amount(ServerLevel level, ChunkPos pos) {
        return OilChunksSavedData.getChunkOilAmount(level, pos);
    }

    @Override
    public int infiniteThreshold() {
        return infiniteThresholdOf();
    }

    /** CDG's infinite threshold, or its default when the server config is not loaded. */
    static int infiniteThresholdOf() {
        try {
            return CDGConfig.OIL_CHUNK_INFINITE_THRESHOLD.get();
        } catch (RuntimeException e) {
            return CdgBridge.DEFAULT_INFINITE_THRESHOLD;
        }
    }

    @Override
    public String config() {
        try {
            return "infinite>" + CDGConfig.OIL_CHUNK_INFINITE_THRESHOLD.get() + " threshold=" + CDGConfig.OIL_CHUNK_THRESHOLD.get()
                    + " normal×" + CDGConfig.OIL_MULTIPLIER.get() + " high×" + CDGConfig.HIGH_OIL_MULTIPLIER.get()
                    + " normalDisabled=" + CDGConfig.DISABLE_NORMAL_OIL_CHUNKS.get() + " highDisabled=" + CDGConfig.DISABLE_HIGH_OIL_CHUNKS.get();
        } catch (RuntimeException e) {
            return "config not loaded (" + e.getMessage() + ")";
        }
    }
}
