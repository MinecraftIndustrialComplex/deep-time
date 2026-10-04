// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.compat.create;

import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamRefusals;
import com.simibubi.create.content.kinetics.belt.item.BeltConnectorItem;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * A belt whose span touches a turning seam is refused (docs/design/wrap-engine.md, "Create across
 * turning seams"): a belt's segments find their controller by absolute position, which a turn does not
 * carry. Applied only with {@code -Ddeeptime.wrapengine=true} and Create installed.
 */
@Mixin(value = BeltConnectorItem.class, remap = false)
abstract class BeltConnectorSeamMixin {

    @Inject(method = "canConnect", at = @At("HEAD"), cancellable = true)
    private static void deeptime$refuseAcrossSeam(Level world, BlockPos first, BlockPos second, CallbackInfoReturnable<Boolean> cir) {
        if (SeamFrames.anyBeyond(world, first, second)) {
            SeamRefusals.refuse(world, first, SeamRefusals.Kind.BELT, "belt " + first.toShortString() + " to " + second.toShortString());
            cir.setReturnValue(false);
        }
    }
}
