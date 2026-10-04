// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.llamalad7.mixinextras.injector.ModifyReturnValue;
import com.llamalad7.mixinextras.sugar.Local;
import com.minecraftindustrialcomplex.deeptime.wrapengine.BandManager;
import com.minecraftindustrialcomplex.deeptime.wrapengine.FoldedPos;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngine;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Folded block access at the {@code Level} boundary (docs/design/wrap-engine.md §4.1 and §5). A
 * position beyond an edge is answered from the stored block it is, turned: reads return the stored
 * state turned back into the band's frame, writes store the state turned into the stored frame, fluids
 * and block entities come from the stored position.
 *
 * <p>The read path is the busiest call in the game, so it allocates nothing unless the position is in
 * a band: {@code getBlockState}'s position is replaced at the head ({@code @ModifyVariable}, no
 * callback object) after the topology's grid test, and the result is turned at the return only when
 * the position was replaced by a {@link FoldedPos}. With no topology the cost is a field read and a
 * null test, plus an {@code instanceof} at the return.
 *
 * <p>Applied only with {@code -Ddeeptime.wrapengine=true} ({@code DeepTimeMixinPlugin}).
 */
@Mixin(Level.class)
abstract class LevelWrapMixin implements WrapLevel {

    @Unique
    private WrapTopology deeptime$wrapTopology;
    @Unique
    private BandManager deeptime$wrapBands;

    @Override
    public WrapTopology deeptime$wrap() {
        return deeptime$wrapTopology;
    }

    @Override
    public void deeptime$setWrap(WrapTopology topology) {
        deeptime$wrapTopology = topology;
    }

    @Override
    public BandManager deeptime$bands() {
        return deeptime$wrapBands;
    }

    @Override
    public void deeptime$setBands(BandManager bands) {
        deeptime$wrapBands = bands;
    }

    @ModifyVariable(method = "getBlockState", at = @At("HEAD"), argsOnly = true)
    private BlockPos deeptime$foldRead(BlockPos pos) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null || !t.mayFold(pos.getX(), pos.getZ()))
            return pos;
        return FoldedPos.forRead(t, pos);
    }

    @ModifyReturnValue(method = "getBlockState", at = @At("RETURN"))
    private BlockState deeptime$turnRead(BlockState state, @Local(argsOnly = true) BlockPos pos) {
        return pos instanceof FoldedPos f ? state.rotate(f.back) : state;
    }

    @ModifyVariable(method = "getFluidState", at = @At("HEAD"), argsOnly = true)
    private BlockPos deeptime$foldFluid(BlockPos pos) {
        return deeptime$fold(pos);
    }

    @ModifyVariable(method = "getBlockEntity", at = @At("HEAD"), argsOnly = true)
    private BlockPos deeptime$foldBlockEntity(BlockPos pos) {
        return deeptime$fold(pos);
    }

    @ModifyVariable(method = "removeBlockEntity", at = @At("HEAD"), argsOnly = true)
    private BlockPos deeptime$foldRemoveBlockEntity(BlockPos pos) {
        return deeptime$fold(pos);
    }

    @Unique
    private BlockPos deeptime$fold(BlockPos pos) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null || !t.mayFold(pos.getX(), pos.getZ()))
            return pos;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        return f == null ? pos : new BlockPos(f.x(), pos.getY(), f.z());
    }

    @Inject(method = "setBlock(Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/level/block/state/BlockState;II)Z",
            at = @At("HEAD"), cancellable = true)
    private void deeptime$foldWrite(BlockPos pos, BlockState state, int flags, int recursionLeft, CallbackInfoReturnable<Boolean> cir) {
        WrapTopology t = deeptime$wrapTopology;
        if (t == null || !t.mayFold(pos.getX(), pos.getZ()))
            return;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        if (f == null)
            return;
        BlockPos stored = new BlockPos(f.x(), pos.getY(), f.z());
        cir.setReturnValue(((Level) (Object) this).setBlock(stored, state.rotate(WrapEngine.rotation(f.iso())), flags, recursionLeft));
    }
}
