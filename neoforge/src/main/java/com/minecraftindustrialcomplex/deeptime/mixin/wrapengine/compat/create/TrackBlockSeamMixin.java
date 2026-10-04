// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine.compat.create;

import com.llamalad7.mixinextras.sugar.Local;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamFrames;
import com.minecraftindustrialcomplex.deeptime.compat.create.wrap.SeamRefusals;
import com.simibubi.create.content.trains.track.TrackBlock;
import com.simibubi.create.content.trains.track.TrackPlacement;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.state.BlockState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Train track is refused at turning seams (docs/design/wrap-engine.md, "Create across turning seams"):
 * Create's track graph keys nodes by absolute position, and under folded access a straight run across a
 * seam would grow a second copy of the graph in band coordinates. Applied only with
 * {@code -Ddeeptime.wrapengine=true} and Create installed.
 *
 * <p>Any straight track block (placed by hand, by a schematic or by the track tool) within one block
 * of a seam is broken and dropped one tick after it is placed, before it joins the graph.
 */
@Mixin(value = TrackBlock.class, remap = false)
abstract class TrackBlockSeamMixin {
    @Inject(method = "tick", at = @At("HEAD"), cancellable = true)
    private void deeptime$refuseAtSeam(BlockState state, ServerLevel level, BlockPos pos, RandomSource random, CallbackInfo ci) {
        if (!SeamFrames.anyBeyond(level, pos.offset(-1, 0, -1), pos.offset(1, 0, 1)))
            return;
        SeamRefusals.refuse(level, pos, SeamRefusals.Kind.TRACK, "track block next to a seam");
        level.destroyBlock(pos, true);
        ci.cancel();
    }
}
