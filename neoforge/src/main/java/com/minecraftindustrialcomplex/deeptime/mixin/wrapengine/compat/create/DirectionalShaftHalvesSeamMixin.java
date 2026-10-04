// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.compat.create;

import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.KineticSeams;
import com.simibubi.create.content.kinetics.base.DirectionalShaftHalvesBlockEntity;
import com.simibubi.create.content.kinetics.base.KineticBlockEntity;
import net.minecraft.core.Direction;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * A gearbox, clutch or gearshift whose rotation source is across a turning seam: Create stores the
 * source's (stored, far) position and takes the face as the nearest direction towards it, which is
 * meaningless across a seam; the face it is seen through is right (compat/create/wrap/KineticSeams).
 * Applied only with {@code -Ddeeptime.wrapengine=true} and Create installed.
 */
@Mixin(value = DirectionalShaftHalvesBlockEntity.class, remap = false)
abstract class DirectionalShaftHalvesSeamMixin {

    @Inject(method = "getSourceFacing", at = @At("HEAD"), cancellable = true)
    private void deeptime$seamSourceFacing(CallbackInfoReturnable<Direction> cir) {
        Direction d = KineticSeams.sourceFacing((KineticBlockEntity) (Object) this);
        if (d != null)
            cir.setReturnValue(d);
    }
}
