// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.compat.create;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.simibubi.create.content.fluids.FlowSource;
import com.simibubi.create.content.fluids.FluidTransportBehaviour;
import net.createmod.catnip.math.BlockFace;
import net.minecraft.core.Direction;
import net.neoforged.neoforge.fluids.FluidStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

/**
 * A pipe whose neighbour is across a turning seam asks that neighbour for its outward fluid on the face
 * turned into the neighbour's frame (docs/design/wrap-engine.md, "Create across turning seams").
 * Applied only with {@code -Ddeeptime.wrapengine=true} and Create installed.
 */
@Mixin(value = FlowSource.OtherPipe.class, remap = false)
abstract class OtherPipeSeamMixin {

    @WrapOperation(method = "provideFluid", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/fluids/FluidTransportBehaviour;getProvidedOutwardFluid(Lnet/minecraft/core/Direction;)Lnet/neoforged/neoforge/fluids/FluidStack;"))
    private FluidStack deeptime$outwardFace(FluidTransportBehaviour neighbour, Direction face, Operation<FluidStack> original) {
        BlockFace location = ((FlowSourceAccessor) (Object) this).deeptime$location();
        return original.call(neighbour, SeamFrames.toStoredFrame(neighbour.getWorld(), location.getConnectedPos(), face));
    }
}
