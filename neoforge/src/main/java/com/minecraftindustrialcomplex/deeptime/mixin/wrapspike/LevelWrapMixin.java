// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike;

import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapEngine;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapLevel;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapTopology;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.material.FluidState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * The spike's folded block access (docs/design/wrap-engine.md §3.2). A position beyond an edge is
 * answered from the stored block it is, turned: reads return the stored state turned back by the fold's
 * inverse, writes store the state turned by the fold, block entities come from the stored position.
 * After a stored block changes, its band copies are rewritten ({@link WrapEngine#replicate}).
 *
 * <p>Dev only: {@code DeepTimeMixinPlugin} applies it only with {@code -Ddeeptime.wrapspike=true}.
 * With no topology installed the cost is one field read per call.
 */
@Mixin(Level.class)
abstract class LevelWrapMixin implements WrapLevel {

    @Unique
    private volatile WrapTopology deeptime$wrapTopology;

    @Override
    public WrapTopology deeptime$wrap() {
        return deeptime$wrapTopology;
    }

    @Override
    public void deeptime$setWrap(WrapTopology topology) {
        deeptime$wrapTopology = topology;
    }

    @Inject(method = "getBlockState", at = @At("HEAD"), cancellable = true)
    private void deeptime$foldRead(BlockPos pos, CallbackInfoReturnable<BlockState> cir) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null)
            return;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        BlockState stored = ((Level) (Object) this).getBlockState(new BlockPos(f.x(), pos.getY(), f.z()));
        cir.setReturnValue(stored.rotate(WrapEngine.rotation(f.iso().inverse())));
    }

    @Inject(method = "getFluidState", at = @At("HEAD"), cancellable = true)
    private void deeptime$foldFluid(BlockPos pos, CallbackInfoReturnable<FluidState> cir) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null)
            return;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        cir.setReturnValue(((Level) (Object) this).getFluidState(new BlockPos(f.x(), pos.getY(), f.z())));
    }

    @Inject(method = "getBlockEntity", at = @At("HEAD"), cancellable = true)
    private void deeptime$foldBlockEntity(BlockPos pos, CallbackInfoReturnable<BlockEntity> cir) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null)
            return;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        cir.setReturnValue(((Level) (Object) this).getBlockEntity(new BlockPos(f.x(), pos.getY(), f.z())));
    }

    @Inject(method = "setBlock(Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/level/block/state/BlockState;II)Z",
            at = @At("HEAD"), cancellable = true)
    private void deeptime$foldWrite(BlockPos pos, BlockState state, int flags, int recursionLeft, CallbackInfoReturnable<Boolean> cir) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null)
            return;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        BlockPos stored = new BlockPos(f.x(), pos.getY(), f.z());
        cir.setReturnValue(((Level) (Object) this).setBlock(stored, state.rotate(WrapEngine.rotation(f.iso())), flags, recursionLeft));
    }

    @Inject(method = "setBlock(Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/level/block/state/BlockState;II)Z",
            at = @At("RETURN"))
    private void deeptime$replicate(BlockPos pos, BlockState state, int flags, int recursionLeft, CallbackInfoReturnable<Boolean> cir) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null || !cir.getReturnValueZ())
            return;
        if ((Object) this instanceof ServerLevel sl)
            WrapEngine.replicate(sl, t, pos);
    }
}
