// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.BandBlockEntities;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.block.entity.SignBlockEntity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * The sign editor opens on the sign the player sees (docs/design/wrap-engine.md §12.8): when a band
 * image of the sign is nearer the player than the sign itself, the editor and its block update go to
 * that image. The edit comes back addressed to it, and folded access finds the sign. Applied only with
 * {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(ServerPlayer.class)
abstract class ServerPlayerWrapMixin {

    @Inject(method = "openTextEdit", at = @At("HEAD"), cancellable = true)
    private void deeptime$editAtImage(SignBlockEntity sign, boolean isFrontText, CallbackInfo ci) {
        ServerPlayer self = (ServerPlayer) (Object) this;
        if (((WrapLevel) self.level()).deeptime$wrap() == null)
            return;
        BandBlockEntities.Image im = BandBlockEntities.nearestBandImage(self.level(), sign.getBlockPos(), self.position());
        if (im == null)
            return;
        self.connection.send(new ClientboundBlockUpdatePacket(self.level(), im.pos()));
        self.connection.send(new ClientboundOpenSignEditorPacket(im.pos(), isFrontText));
        ci.cancel();
    }
}
