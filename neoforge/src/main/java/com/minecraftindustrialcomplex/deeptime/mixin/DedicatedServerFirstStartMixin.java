// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.server.ServerStop;
import net.minecraft.server.dedicated.DedicatedServer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Leaves {@code DedicatedServer.initServer} right after {@code ServerAboutToStartEvent} when
 * {@link ServerStop} was requested (P1-13): before "Preparing level", {@code loadLevel()}, RCON,
 * the watchdog and {@code ServerStartingEvent}. It returns {@code true} so {@code runServer} does
 * not turn the stop into a crash report; the server is already halted, so it goes straight to its
 * normal shutdown (see {@code MinecraftServerFirstStartMixin}).
 */
@Mixin(DedicatedServer.class)
abstract class DedicatedServerFirstStartMixin {

    @Inject(method = "initServer",
            at = @At(value = "INVOKE",
                    target = "Lnet/neoforged/neoforge/server/ServerLifecycleHooks;handleServerAboutToStart(Lnet/minecraft/server/MinecraftServer;)V",
                    shift = At.Shift.AFTER),
            cancellable = true)
    private void deeptime$stopBeforeLoadLevel(CallbackInfoReturnable<Boolean> cir) {
        if (ServerStop.requested())
            cir.setReturnValue(true);
    }
}
