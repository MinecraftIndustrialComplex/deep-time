// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapChunk;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.material.Fluid;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Band chunks hold pictures, not blocks (docs/design/wrap-engine.md §4.2), so nothing ticks in them:
 * no random ticks, lightning, snow or ice ({@code tickChunk}; through the fold they would act on the
 * stored block a second time), and no scheduled tick fires at a band position (ticks are scheduled
 * at stored positions; one a band chunk brought from its generation or its save is dropped). Applied
 * only with {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(ServerLevel.class)
abstract class ServerLevelWrapMixin {

    @Inject(method = "tickChunk", at = @At("HEAD"), cancellable = true)
    private void deeptime$noBandChunkTicks(LevelChunk chunk, int randomTickSpeed, CallbackInfo ci) {
        if (((WrapChunk) chunk).deeptime$bandIso() != null)
            ci.cancel();
    }

    @Inject(method = "tickBlock", at = @At("HEAD"), cancellable = true)
    private void deeptime$noBandBlockTicks(BlockPos pos, Block block, CallbackInfo ci) {
        if (deeptime$inBand(pos))
            ci.cancel();
    }

    @Inject(method = "tickFluid", at = @At("HEAD"), cancellable = true)
    private void deeptime$noBandFluidTicks(BlockPos pos, Fluid fluid, CallbackInfo ci) {
        if (deeptime$inBand(pos))
            ci.cancel();
    }

    @Unique
    private boolean deeptime$inBand(BlockPos pos) {
        WrapTopology t = ((WrapLevel) (Object) this).deeptime$wrap();
        return t != null && t.fold(pos.getX(), pos.getZ()) != null;
    }
}
