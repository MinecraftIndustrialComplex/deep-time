// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapChunk;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.NaturalSpawner;
import net.minecraft.world.level.chunk.LevelChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * No natural spawning in band chunks (docs/design/wrap-engine.md §4.2): a mob spawned there would be
 * carried to the stored side at once, doubling the spawn rate near a seam. Applied only with
 * {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(NaturalSpawner.class)
abstract class NaturalSpawnerWrapMixin {

    @Inject(method = "spawnForChunk", at = @At("HEAD"), cancellable = true)
    private static void deeptime$noBandSpawns(ServerLevel level, LevelChunk chunk, NaturalSpawner.SpawnState spawnState, boolean spawnFriendlies,
                                              boolean spawnMonsters, boolean forcedDespawn, CallbackInfo ci) {
        if (((WrapChunk) chunk).deeptime$bandIso() != null)
            ci.cancel();
    }
}
