// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.compat.create;

import com.llamalad7.mixinextras.sugar.Local;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.simibubi.create.content.fluids.FlowSource;
import net.createmod.catnip.math.BlockFace;
import net.minecraft.core.Direction;
import net.minecraft.server.level.ServerLevel;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

/**
 * A pipe ending on a fluid handler (a tank) across a turning seam asks for the handler's capability on
 * the face turned into the handler's frame, which matters for sided handlers
 * (docs/design/wrap-engine.md, "Create across turning seams"). The capability cache is built in a
 * lambda of {@code manageSource} ({@code lambda$manageSource$2} in Create 6.0.10; the mixin fails at load
 * if Create renames it). Applied only with {@code -Ddeeptime.wrapengine=true} and Create installed.
 */
@Mixin(value = FlowSource.FluidHandler.class, remap = false)
abstract class FluidHandlerSourceSeamMixin {

    @ModifyArg(method = "lambda$manageSource$2", at = @At(value = "INVOKE",
            target = "Lnet/neoforged/neoforge/capabilities/BlockCapabilityCache;create(Lnet/neoforged/neoforge/capabilities/BlockCapability;Lnet/minecraft/server/level/ServerLevel;Lnet/minecraft/core/BlockPos;Ljava/lang/Object;Ljava/util/function/BooleanSupplier;Ljava/lang/Runnable;)Lnet/neoforged/neoforge/capabilities/BlockCapabilityCache;"),
            index = 3)
    private Object deeptime$handlerFace(Object side, @Local(argsOnly = true) ServerLevel level) {
        BlockFace location = ((FlowSourceAccessor) (Object) this).deeptime$location();
        return side instanceof Direction d ? SeamFrames.toStoredFrame(level, location.getConnectedPos(), d) : side;
    }
}
