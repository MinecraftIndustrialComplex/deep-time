// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngine;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockBehaviour;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * A neighbour update addressed to a band position runs on the stored block instead, with the source
 * position moved by the same motion (docs/design/wrap-engine.md §4.1): block logic always runs at
 * stored positions. Applied only with {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(BlockBehaviour.BlockStateBase.class)
abstract class BlockStateBaseWrapMixin {

    @Inject(method = "handleNeighborChanged", at = @At("HEAD"), cancellable = true)
    private void deeptime$foldNeighbourUpdate(Level level, BlockPos pos, Block block, BlockPos fromPos, boolean isMoving, CallbackInfo ci) {
        WrapTopology t = ((WrapLevel) level).deeptime$wrap();
        if (t == null || !t.mayFold(pos.getX(), pos.getZ()))
            return;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        BlockPos stored = new BlockPos(f.x(), pos.getY(), f.z());
        BlockState state = level.getBlockState(stored);
        state.handleNeighborChanged(level, stored, block, WrapEngine.apply(f.iso(), fromPos), isMoving);
        ci.cancel();
    }
}
