// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.client;

import com.minecraftindustrialcomplex.deeptime.screen.CreateWorldFlow;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Create World starts on Deep Time and its Create button simulates first (Ben, 2026-10-19;
 * docs/design/create-world-ux.md). NeoForge has no hook for the default world type, so:
 * <ul>
 *   <li>{@code openFresh} (Singleplayer, Create New World; not {@code createFromExisting}, which
 *       recreates a world with its own type) hands its new screen to
 *       {@link CreateWorldFlow#fresh}, which selects {@code deeptime:planet_cube} the way the World tab's
 *       button does ({@code WorldCreationUiState.setWorldType}, which also sets the dimensions);</li>
 *   <li>{@code onCreate} (the Create button and Enter) asks {@link CreateWorldFlow#interceptCreate}
 *       first: a Deep Time world without world data opens the generation screen instead.</li>
 * </ul>
 * Client only (the "client" section of deeptime.mixins.json).
 */
@Mixin(CreateWorldScreen.class)
abstract class CreateWorldScreenMixin {

    @ModifyArg(method = "openFresh",
            at = @At(value = "INVOKE", target = "Lnet/minecraft/client/Minecraft;setScreen(Lnet/minecraft/client/gui/screens/Screen;)V"))
    private static Screen deeptime$defaultWorldType(Screen screen) {
        if (screen instanceof CreateWorldScreen cws)
            CreateWorldFlow.fresh(cws);
        return screen;
    }

    @Inject(method = "onCreate", at = @At("HEAD"), cancellable = true)
    private void deeptime$simulateFirst(CallbackInfo ci) {
        if (CreateWorldFlow.interceptCreate((CreateWorldScreen) (Object) this))
            ci.cancel();
    }
}
