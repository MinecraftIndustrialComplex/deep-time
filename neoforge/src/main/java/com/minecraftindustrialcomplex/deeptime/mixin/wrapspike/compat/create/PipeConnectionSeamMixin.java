// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.simibubi.create.content.fluids.PipeConnection;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * A pipe connection's side is in its owner's stored frame, so it must find its source from the owner's
 * stored position. Create's fluid network calls it with the position it reached the owner at, which is
 * a band position when it walked across a turning seam; that position is folded first
 * (docs/design/wrap-engine.md, "Create across turning seams"). Applied only with
 * {@code -Ddeeptime.wrapspike=true} and Create installed.
 */
@Mixin(value = PipeConnection.class, remap = false)
abstract class PipeConnectionSeamMixin {

    @Inject(method = "determineSource", at = @At("HEAD"), cancellable = true)
    private void deeptime$fromStoredPos(Level world, BlockPos pos, CallbackInfoReturnable<Boolean> cir) {
        BlockPos stored = SeamFrames.stored(world, pos);
        if (stored != pos)
            cir.setReturnValue(((PipeConnection) (Object) this).determineSource(world, stored));
    }
}
