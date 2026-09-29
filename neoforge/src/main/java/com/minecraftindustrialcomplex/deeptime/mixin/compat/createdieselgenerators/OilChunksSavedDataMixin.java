// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.createdieselgenerators;

import com.jesz.createdieselgenerators.CDGConfig;
import com.jesz.createdieselgenerators.world.OilChunksSavedData;
import com.minecraftindustrialcomplex.deeptime.compat.cdg.CdgBridge;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.OptionalInt;

/**
 * Oil comes from petroleum traps in Deep Time levels (P1-16, plan §7.14; {@link CdgBridge}).
 * Target: CDG 1.3.15 {@code public static int OilChunksSavedData.getBaseOilAmount(ServerLevel,
 * ChunkPos)}, a chunk's oil before anything was pumped (the pumpjack and the oil scanner reach it
 * through {@code getChunkOilAmount}, which stores what is left once pumping starts). HEAD
 * (cancellable): in a Deep Time level, the trap's share under the chunk or 0; CDG's own first step,
 * the KubeJS override {@code CDGKubeJSPlugin.calculateOilChunks}, and its noise then do not run
 * there (MIC's KubeJS has no {@code calculateOilChunks}, re-checked 2026-09-28). Other levels keep
 * CDG's generator. Applied only when CDG is installed.
 */
@Mixin(value = OilChunksSavedData.class, remap = false)
public abstract class OilChunksSavedDataMixin {

    @Inject(method = "getBaseOilAmount", at = @At("HEAD"), cancellable = true)
    private static void deeptime$trapCharge(ServerLevel level, ChunkPos chunk, CallbackInfoReturnable<Integer> cir) {
        OptionalInt amount = CdgBridge.baseOilAmount(level, chunk, deeptime$infiniteThreshold());
        if (amount.isPresent())
            cir.setReturnValue(amount.getAsInt());
    }

    @Unique
    private static int deeptime$infiniteThreshold() {
        try {
            return CDGConfig.OIL_CHUNK_INFINITE_THRESHOLD.get();
        } catch (RuntimeException e) {
            return CdgBridge.DEFAULT_INFINITE_THRESHOLD;
        }
    }
}
