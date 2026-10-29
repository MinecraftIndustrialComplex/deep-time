// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.worldgen.PlayaPass;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.WorldGenRegion;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * No feature lays soil or a tree on a salt flat's salt or replaces it ({@link PlayaPass#refusesOnSalt}, ledger HYD-180): features,
 * top-level or nested, and structures write through the region they are placed in.
 */
@Mixin(WorldGenRegion.class)
abstract class WorldGenRegionSaltMixin {

    @Inject(method = "setBlock(Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/level/block/state/BlockState;II)Z",
            at = @At("HEAD"), cancellable = true)
    private void deeptime$noSoilOnSalt(BlockPos pos, BlockState state, int flags, int recursionLeft, CallbackInfoReturnable<Boolean> cir) {
        if (PlayaPass.refusesOnSalt((WorldGenRegion) (Object) this, pos, state))
            cir.setReturnValue(false);
    }
}
