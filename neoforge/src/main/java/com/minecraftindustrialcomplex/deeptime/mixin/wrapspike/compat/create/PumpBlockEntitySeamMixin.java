// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.llamalad7.mixinextras.sugar.Local;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.simibubi.create.content.fluids.FluidTransportBehaviour;
import com.simibubi.create.content.fluids.pump.PumpBlockEntity;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.LevelAccessor;
import net.neoforged.neoforge.capabilities.BlockCapability;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

/**
 * A pump's pressure search across a turning seam: it walks in the pump's frame, so the face of each
 * pipe it pressurises, and of a pump or fluid handler it ends on, is turned into that block entity's
 * frame (docs/design/wrap-engine.md, "Create across turning seams"). Applied only with
 * {@code -Ddeeptime.wrapspike=true} and Create installed.
 */
@Mixin(value = PumpBlockEntity.class, remap = false)
abstract class PumpBlockEntitySeamMixin {

    @WrapOperation(method = "distributePressureTo", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/fluids/FluidTransportBehaviour;addPressure(Lnet/minecraft/core/Direction;ZF)V"))
    private void deeptime$pressureSide(FluidTransportBehaviour pipe, Direction side, boolean inbound, float pressure, Operation<Void> original,
                                       @Local(name = "pipePos") BlockPos pipePos) {
        original.call(pipe, SeamFrames.toStoredFrame(((PumpBlockEntity) (Object) this).getLevel(), pipePos, side), inbound, pressure);
    }

    @WrapOperation(method = "hasReachedValidEndpoint", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/fluids/pump/PumpBlockEntity;isFront(Lnet/minecraft/core/Direction;)Z"))
    private boolean deeptime$otherPumpFront(PumpBlockEntity pump, Direction side, Operation<Boolean> original,
                                            @Local(argsOnly = true) LevelAccessor world, @Local(name = "connectedPos") BlockPos connectedPos) {
        return original.call(pump, SeamFrames.toStoredFrame(world, connectedPos, side));
    }

    @WrapOperation(method = "hasReachedValidEndpoint", at = @At(value = "INVOKE",
            target = "Lnet/minecraft/world/level/Level;getCapability(Lnet/neoforged/neoforge/capabilities/BlockCapability;Lnet/minecraft/core/BlockPos;Ljava/lang/Object;)Ljava/lang/Object;"))
    private Object deeptime$endpointSide(Level level, BlockCapability<?, ?> cap, BlockPos storedPos, Object side, Operation<Object> original,
                                         @Local(argsOnly = true) LevelAccessor world, @Local(name = "connectedPos") BlockPos connectedPos) {
        return original.call(level, cap, storedPos, side instanceof Direction d ? SeamFrames.toStoredFrame(world, connectedPos, d) : side);
    }
}
