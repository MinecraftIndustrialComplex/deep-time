// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.BandBlockEntities;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngine;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import net.minecraft.core.BlockPos;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * A block is in reach when an image of it is (docs/design/wrap-engine.md §12.8): a container menu's
 * {@code stillValid} and a sign's editor ask {@code canInteractWithBlock} with the stored position,
 * which can be the far side of the world while the player stands at its image across a seam. Only
 * when vanilla's answer is no and the level has a topology. Applied only with
 * {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(Player.class)
abstract class PlayerWrapMixin {

    @Inject(method = "canInteractWithBlock", at = @At("RETURN"), cancellable = true)
    private void deeptime$reachThroughImage(BlockPos pos, double distance, CallbackInfoReturnable<Boolean> cir) {
        Player self = (Player) (Object) this;
        if (cir.getReturnValueZ() || ((WrapLevel) self.level()).deeptime$wrap() == null)
            return;
        Vec3 eye = self.getEyePosition();
        BandBlockEntities.Image im = BandBlockEntities.nearestBandImage(self.level(), WrapEngine.canonical(self.level(), pos), eye);
        if (im == null || im.pos().equals(pos))
            return;
        double r = self.blockInteractionRange() + distance;
        if (new AABB(im.pos()).distanceToSqr(eye) < r * r)
            cir.setReturnValue(true);
    }
}
