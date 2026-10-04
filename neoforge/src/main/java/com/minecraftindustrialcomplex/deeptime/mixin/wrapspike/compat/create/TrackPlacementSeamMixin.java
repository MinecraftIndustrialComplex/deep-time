// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapspike.compat.create;

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
 * {@code -Ddeeptime.wrapspike=true} and Create installed.
 *
 * <p>The track tool (curves and the straight runs it lays): refused before anything is placed or
 * paid for, with Create's own message path (the action bar and the deny sound; key
 * {@code create.track.deeptime_seam}).
 */
@Mixin(value = TrackPlacement.class, remap = false)
abstract class TrackPlacementSeamMixin {
    @Inject(method = "tryConnect", cancellable = true, at = @At(value = "INVOKE", ordinal = 0,
            target = "Lcom/simibubi/create/content/trains/track/TrackPlacement;placeTracks(Lnet/minecraft/world/level/Level;Lcom/simibubi/create/content/trains/track/TrackPlacement$PlacementInfo;Lnet/minecraft/world/level/block/state/BlockState;Lnet/minecraft/world/level/block/state/BlockState;Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;Z)Lcom/simibubi/create/content/trains/track/TrackPlacement$PlacementInfo;"))
    private static void deeptime$refuseAcrossSeam(Level level, Player player, BlockPos pos2, BlockState state2, ItemStack stack,
                                                  boolean girder, boolean maximiseTurn,
                                                  CallbackInfoReturnable<TrackPlacement.PlacementInfo> cir,
                                                  @Local TrackPlacement.PlacementInfo info) {
        PlacementInfoAccessor a = (PlacementInfoAccessor) info;
        BlockPos p1 = a.deeptime$pos1(), p2 = a.deeptime$pos2();
        if (p1 == null || p2 == null)
            return;
        BlockPos lo = new BlockPos(Math.min(p1.getX(), p2.getX()) - 1, p1.getY(), Math.min(p1.getZ(), p2.getZ()) - 1);
        BlockPos hi = new BlockPos(Math.max(p1.getX(), p2.getX()) + 1, p1.getY(), Math.max(p1.getZ(), p2.getZ()) + 1);
        if (!SeamFrames.anyBeyond(level, lo, hi))
            return;
        SeamRefusals.refuse(level, p1, SeamRefusals.Kind.TRACK, "track tool " + p1.toShortString() + " to " + p2.toShortString());
        a.deeptime$setValid(false);
        cir.setReturnValue(info.withMessage("deeptime_seam").tooJumbly());
    }
}
