// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamRefusals;
import com.simibubi.create.content.kinetics.chainConveyor.ChainConveyorBlockEntity;
import net.minecraft.core.BlockPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * A chain conveyor connection whose span touches a turning seam is refused (docs/design/wrap-engine.md,
 * "Create across turning seams"): connections are stored as offsets to the partner, which a turn does
 * not carry. Applied only with {@code -Ddeeptime.wrapspike=true} and Create installed.
 */
@Mixin(value = ChainConveyorBlockEntity.class, remap = false)
abstract class ChainConveyorSeamMixin {

    @Inject(method = "addConnectionTo", at = @At("HEAD"), cancellable = true)
    private void deeptime$refuseAcrossSeam(BlockPos target, CallbackInfoReturnable<Boolean> cir) {
        ChainConveyorBlockEntity self = (ChainConveyorBlockEntity) (Object) this;
        if (SeamFrames.anyBeyond(self.getLevel(), self.getBlockPos(), target)) {
            SeamRefusals.refuse(self.getLevel(), self.getBlockPos(), SeamRefusals.Kind.CHAIN_CONVEYOR, "to " + target.toShortString());
            cir.setReturnValue(false);
        }
    }
}
