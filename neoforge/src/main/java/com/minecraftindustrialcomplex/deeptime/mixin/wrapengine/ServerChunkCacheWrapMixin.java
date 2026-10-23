// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.BandBlockEntities;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLevel;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerChunkCache;
import net.minecraft.server.level.ServerLevel;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * A change at a stored position is marked at each band position that shows it too
 * (docs/design/wrap-engine.md §12.8), so the band's holders send it: a block entity's data change
 * ({@code sendBlockUpdated}, a sign's new text) leaves the band's copy of the state as it was, and
 * would reach only the stored chunk's viewers. Applied only with {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(ServerChunkCache.class)
abstract class ServerChunkCacheWrapMixin {

    @Shadow
    @Final
    public ServerLevel level;

    @Inject(method = "blockChanged", at = @At("TAIL"))
    private void deeptime$markImages(BlockPos pos, CallbackInfo ci) {
        if (((WrapLevel) level).deeptime$wrap() != null)
            BandBlockEntities.markImages(level, pos);
    }
}
