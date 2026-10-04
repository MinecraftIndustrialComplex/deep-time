// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.compat.create;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.llamalad7.mixinextras.sugar.Local;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.simibubi.create.content.fluids.FluidPropagator;
import com.simibubi.create.content.fluids.FluidTransportBehaviour;
import com.simibubi.create.content.fluids.PipeConnection;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.LevelAccessor;
import net.neoforged.neoforge.capabilities.BlockCapability;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

/**
 * Create's pipe searches across a turning seam (docs/design/wrap-engine.md, "Create across turning
 * seams"). The searches walk in the frame of the pipe they started from, through band positions that
 * folded access answers with the stored, turned neighbour; every face handed to a block entity reached
 * that way is turned into its frame ({@link SeamFrames#toStoredFrame}). Applied only with
 * {@code -Ddeeptime.wrapengine=true} and Create installed.
 */
@Mixin(value = FluidPropagator.class, remap = false)
abstract class FluidPropagatorSeamMixin {

    /** A pump found across the seam is told which of its sides to update in its own frame. */
    @ModifyArg(method = "propagateChangedPipe", at = @At(value = "INVOKE",
            target = "Lnet/createmod/catnip/data/Pair;of(Ljava/lang/Object;Ljava/lang/Object;)Lnet/createmod/catnip/data/Pair;",
            ordinal = 1), index = 1)
    private static Object deeptime$pumpSide(Object side, @Local(argsOnly = true) LevelAccessor world, @Local(name = "target") BlockPos target) {
        return side instanceof Direction d ? SeamFrames.toStoredFrame(world, target, d) : side;
    }

    @WrapOperation(method = "resetAffectedFluidNetworks", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/fluids/FluidTransportBehaviour;getConnection(Lnet/minecraft/core/Direction;)Lcom/simibubi/create/content/fluids/PipeConnection;"))
    private static PipeConnection deeptime$resetConnection(FluidTransportBehaviour pipe, Direction d, Operation<PipeConnection> original,
                                                           @Local(argsOnly = true) Level world, @Local(name = "pos") BlockPos pos) {
        return original.call(pipe, SeamFrames.toStoredFrame(world, pos, d));
    }

    @WrapOperation(method = "hasFluidCapability", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/world/level/Level;getCapability(Lnet/neoforged/neoforge/capabilities/BlockCapability;Lnet/minecraft/core/BlockPos;Ljava/lang/Object;)Ljava/lang/Object;"))
    private static Object deeptime$capabilitySide(Level level, BlockCapability<?, ?> cap, BlockPos storedPos, Object side, Operation<Object> original,
                                                  @Local(argsOnly = true) BlockGetter world, @Local(argsOnly = true) BlockPos pos) {
        return original.call(level, cap, storedPos, side instanceof Direction d ? SeamFrames.toStoredFrame(world, pos, d) : side);
    }
}
