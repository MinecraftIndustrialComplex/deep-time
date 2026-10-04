// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

import com.llamalad7.mixinextras.injector.wrapmethod.WrapMethod;
import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.llamalad7.mixinextras.injector.wrapoperation.WrapOperation;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.KineticSeams;
import com.simibubi.create.content.kinetics.RotationPropagator;
import com.simibubi.create.content.kinetics.base.KineticBlockEntity;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;

/**
 * Create's rotation propagation across a turning seam (compat/create/wrap/KineticSeams;
 * docs/design/wrap-engine.md, "Create across turning seams"). Applied only with
 * {@code -Ddeeptime.wrapspike=true} and Create installed (DeepTimeMixinPlugin).
 */
@Mixin(value = RotationPropagator.class, remap = false)
abstract class RotationPropagatorSeamMixin {

    @WrapMethod(method = "getRotationSpeedModifier")
    private static float deeptime$seamModifier(KineticBlockEntity from, KineticBlockEntity to, Operation<Float> original) {
        return KineticSeams.modifier(from, to, original);
    }

    @WrapOperation(method = "getRotationSpeedModifier", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/kinetics/base/KineticBlockEntity;getBlockState()Lnet/minecraft/world/level/block/state/BlockState;"))
    private static BlockState deeptime$seenState(KineticBlockEntity be, Operation<BlockState> original) {
        return KineticSeams.state(be, original);
    }

    @WrapOperation(method = "getRotationSpeedModifier", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/kinetics/base/KineticBlockEntity;getBlockPos()Lnet/minecraft/core/BlockPos;"))
    private static BlockPos deeptime$seenPos(KineticBlockEntity be, Operation<BlockPos> original) {
        return KineticSeams.pos(be, original);
    }

    @WrapOperation(method = "getRotationSpeedModifier", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/kinetics/RotationPropagator;getAxisModifier(Lcom/simibubi/create/content/kinetics/base/KineticBlockEntity;Lnet/minecraft/core/Direction;)F"))
    private static float deeptime$turnedAxisModifier(KineticBlockEntity be, Direction direction, Operation<Float> original) {
        return KineticSeams.axisModifier(be, direction, original);
    }

    @WrapOperation(method = "getRotationSpeedModifier", at = @At(value = "INVOKE",
            target = "Lcom/simibubi/create/content/kinetics/base/KineticBlockEntity;propagateRotationTo(Lcom/simibubi/create/content/kinetics/base/KineticBlockEntity;Lnet/minecraft/world/level/block/state/BlockState;Lnet/minecraft/world/level/block/state/BlockState;Lnet/minecraft/core/BlockPos;ZZ)F"))
    private static float deeptime$refuseCustom(KineticBlockEntity self, KineticBlockEntity target, BlockState stateFrom, BlockState stateTo,
                                               BlockPos diff, boolean axes, boolean cogs, Operation<Float> original) {
        return KineticSeams.custom(self, target, stateFrom, stateTo, diff, axes, cogs, original);
    }
}
