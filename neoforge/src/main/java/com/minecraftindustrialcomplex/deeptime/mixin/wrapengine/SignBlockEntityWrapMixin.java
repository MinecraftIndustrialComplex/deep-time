// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.BandBlockEntities;
import net.minecraft.core.BlockPos;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.SignBlock;
import net.minecraft.world.level.block.entity.SignBlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.Vec3;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Which side of a sign a player edits is judged from where the player stands as the sign sees it
 * (docs/design/wrap-engine.md §12.8): through the band image nearest the player, so a player in front
 * of a sign's image across a seam edits its front. Vanilla's own test, on the moved position. Applied
 * only with {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(SignBlockEntity.class)
abstract class SignBlockEntityWrapMixin {

    @Inject(method = "isFacingFrontText", at = @At("HEAD"), cancellable = true)
    private void deeptime$facingThroughImage(Player player, CallbackInfoReturnable<Boolean> cir) {
        SignBlockEntity self = (SignBlockEntity) (Object) this;
        Level level = self.getLevel();
        if (level == null)
            return;
        BlockPos pos = self.getBlockPos();
        Vec3 seen = BandBlockEntities.seenFrom(level, pos, player.position()); // ledger: SCL-186
        BlockState state = self.getBlockState();
        if (seen == null || !(state.getBlock() instanceof SignBlock sign))
            return;
        Vec3 c = sign.getSignHitboxCenterPosition(state);
        double dx = seen.x - (pos.getX() + c.x), dz = seen.z - (pos.getZ() + c.z);
        float facing = sign.getYRotationDegrees(state);
        float toPlayer = (float) (Mth.atan2(dz, dx) * 180.0F / (float) Math.PI) - 90.0F;
        cir.setReturnValue(Mth.degreesDifferenceAbs(facing, toPlayer) <= 90.0F);
    }
}
