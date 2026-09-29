// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.cdg;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.neoforged.fml.ModList;

import java.util.Optional;

/**
 * Create: Diesel Generators' own answers, for the probes and the GameTests (P1-16). Only
 * {@link CdgAccessImpl} names CDG's classes; it is loaded reflectively and only when CDG is
 * installed. Signatures use vanilla types.
 */
public interface CdgAccess {

    /** {@code OilChunksSavedData.getBaseOilAmount(level, pos)}: the chunk's untouched oil (mB). */
    int base(ServerLevel level, ChunkPos pos);

    /** {@code OilChunksSavedData.getChunkOilAmount(level, pos)}: what a pumpjack reads (the stored amount once pumped). */
    int amount(ServerLevel level, ChunkPos pos);

    /** CDG's {@code "Infinite oil chunk threshold"} (mB). */
    int infiniteThreshold();

    /** CDG's oil settings, for the log. */
    String config();

    /** The access when CDG is installed and answers; empty otherwise. */
    static Optional<CdgAccess> load() {
        return Holder.ACCESS;
    }

    /** True when CDG is installed. */
    static boolean installed() {
        return ModList.get() != null && ModList.get().isLoaded(CdgBridge.MOD_ID);
    }

    final class Holder {
        private Holder() {}

        static final Optional<CdgAccess> ACCESS = create();

        private static Optional<CdgAccess> create() {
            if (!installed())
                return Optional.empty();
            try {
                Class<?> impl = Class.forName("com.minecraftindustrialcomplex.deeptime.compat.cdg.CdgAccessImpl");
                return Optional.of((CdgAccess) impl.getDeclaredConstructor().newInstance());
            } catch (Throwable t) {
                DeepTime.LOGGER.error("Create: Diesel Generators is installed but Deep Time cannot reach it", t);
                return Optional.empty();
            }
        }
    }
}
