// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.compat.create;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.llamalad7.mixinextras.sugar.Local;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.simibubi.create.content.fluids.FluidNetwork;
import com.simibubi.create.content.fluids.FluidTransportBehaviour;
import com.simibubi.create.content.fluids.PipeConnection;
import net.createmod.catnip.math.BlockFace;
import net.minecraft.core.Direction;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;

/**
 * Create's fluid network (the transfer layer) walks from its source in the source pipe's frame; a pipe
 * it reaches through a turning seam is asked for the connection on the face turned into its frame
 * (docs/design/wrap-engine.md, "Create across turning seams"). Applied only with
 * {@code -Ddeeptime.wrapengine=true} and Create installed.
 */
@Mixin(value = FluidNetwork.class, remap = false)
abstract class FluidNetworkSeamMixin {

    @Shadow
    Level world;

    @WrapOperation(method = "get", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/fluids/FluidTransportBehaviour;getConnection(Lnet/minecraft/core/Direction;)Lcom/simibubi/create/content/fluids/PipeConnection;"))
    private PipeConnection deeptime$connectionFace(FluidTransportBehaviour pipe, Direction face, Operation<PipeConnection> original,
                                                   @Local(argsOnly = true) BlockFace location) {
        return original.call(pipe, SeamFrames.toStoredFrame(world, location.getPos(), face));
    }
}
