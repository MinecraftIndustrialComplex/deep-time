// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.BandBlockEntities;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import net.minecraft.core.BlockPos;
import net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket;
import net.minecraft.server.level.ChunkHolder;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.List;

/**
 * When a holder sends the block entity at a band position, it sends the stored entity's data addressed
 * to the band position, its tag turned (docs/design/wrap-engine.md §12.8). Vanilla's packet, from the
 * folded {@code getBlockEntity}, carries the stored position, which the band's viewers may not even
 * have loaded. Applied only with {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(ChunkHolder.class)
abstract class ChunkHolderWrapMixin {

    @Inject(method = "broadcastBlockEntity", at = @At("HEAD"), cancellable = true)
    private void deeptime$bandBlockEntity(List<ServerPlayer> players, Level level, BlockPos pos, CallbackInfo ci) {
        if (((WrapLevel) level).deeptime$wrap() == null)
            return;
        ClientboundBlockEntityDataPacket packet = BandBlockEntities.packetFor(level, pos);
        if (packet == null)
            return;
        // What ChunkHolder.broadcast does.
        for (ServerPlayer p : players)
            p.connection.send(packet);
        ci.cancel();
    }
}
