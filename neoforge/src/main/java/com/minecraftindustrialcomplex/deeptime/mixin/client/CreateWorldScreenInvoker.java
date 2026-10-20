// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.client;

import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Invoker;

/**
 * The Create World screen's private {@code onCreate} (its Create button), for the generation
 * screen's Play: once the planet is simulated and bound, Play creates the world exactly as the
 * Create button does (the experimental-settings confirmation included).
 */
@Mixin(CreateWorldScreen.class)
public interface CreateWorldScreenInvoker {

    @Invoker("onCreate")
    void deeptime$onCreate();
}
