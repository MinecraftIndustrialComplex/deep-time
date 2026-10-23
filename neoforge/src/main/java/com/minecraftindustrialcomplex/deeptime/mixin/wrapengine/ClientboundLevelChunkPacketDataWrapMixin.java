// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import com.minecraftindustrialcomplex.deeptime.wrapengine.BandBlockEntities;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapChunk;
import net.minecraft.core.SectionPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.chunk.LevelChunk;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.List;

/**
 * A band chunk's packet lists the stored block entities the chunk shows, at their band positions,
 * their update tags turned (docs/design/wrap-engine.md §12.8): the client then draws chests, signs and
 * banners in the band as anywhere. The band chunk itself holds none. Applied only with
 * {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(ClientboundLevelChunkPacketData.class)
abstract class ClientboundLevelChunkPacketDataWrapMixin {

    @Shadow
    @Final
    private List<ClientboundLevelChunkPacketData.BlockEntityInfo> blockEntitiesData;

    @Inject(method = "<init>(Lnet/minecraft/world/level/chunk/LevelChunk;)V", at = @At("RETURN"))
    private void deeptime$bandBlockEntities(LevelChunk chunk, CallbackInfo ci) {
        if (((WrapChunk) chunk).deeptime$bandIso() == null)
            return;
        Level level = chunk.getLevel();
        BandBlockEntities.forEachShown(level, chunk, (pos, be, toBand) -> {
            CompoundTag tag = BandBlockEntities.bandTag(be, toBand, level.registryAccess());
            int packedXZ = SectionPos.sectionRelative(pos.getX()) << 4 | SectionPos.sectionRelative(pos.getZ());
            blockEntitiesData.add(new ClientboundLevelChunkPacketData.BlockEntityInfo(packedXZ, pos.getY(), be.getType(), tag.isEmpty() ? null : tag));
        });
    }
}
