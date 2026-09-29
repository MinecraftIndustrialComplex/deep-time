// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.minecraftindustrialcomplex.deeptime.server.ServerStop;
import net.minecraft.server.MinecraftServer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * The rest of a stop before {@code loadLevel()} (P1-13, {@link ServerStop}): a server that never
 * loaded a level must not announce that it started or is stopping, and has nothing to save.
 * Every hook is a no-op unless {@link ServerStop#requested()}, which only a dedicated server's
 * first-start generator sets.
 */
@Mixin(MinecraftServer.class)
abstract class MinecraftServerFirstStartMixin {

    @WrapOperation(method = "runServer", at = @At(value = "INVOKE",
            target = "Lnet/neoforged/neoforge/server/ServerLifecycleHooks;handleServerStarted(Lnet/minecraft/server/MinecraftServer;)V"))
    private void deeptime$noStartedEvent(MinecraftServer server, Operation<Void> original) {
        if (!ServerStop.requested())
            original.call(server);
    }

    @WrapOperation(method = "runServer", at = @At(value = "INVOKE",
            target = "Lnet/neoforged/neoforge/server/ServerLifecycleHooks;handleServerStopping(Lnet/minecraft/server/MinecraftServer;)V"))
    private void deeptime$noStoppingEvent(MinecraftServer server, Operation<Void> original) {
        if (!ServerStop.requested())
            original.call(server);
    }

    /** Nothing to save without a level: vanilla would throw on the missing overworld here. */
    @Inject(method = "saveAllChunks", at = @At("HEAD"), cancellable = true)
    private void deeptime$nothingToSave(boolean suppressLog, boolean flush, boolean forced,
            CallbackInfoReturnable<Boolean> cir) {
        if (ServerStop.requested())
            cir.setReturnValue(false);
    }
}
