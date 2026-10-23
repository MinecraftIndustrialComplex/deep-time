// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.wrapengine;

import it.unimi.dsi.fastutil.shorts.ShortSet;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ChunkHolder;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;
import org.spongepowered.asm.mixin.gen.Invoker;

import java.util.List;

/**
 * For the wrap engine's GameTests (block entities in the band, design §12.8): a holder's marked
 * changes and its private block-entity send. Applied only with {@code -Ddeeptime.wrapengine=true}.
 */
@Mixin(ChunkHolder.class)
public interface ChunkHolderAccessor {
    @Accessor("changedBlocksPerSection")
    ShortSet[] deeptime$changedBlocksPerSection();

    @Invoker("broadcastBlockEntity")
    void deeptime$broadcastBlockEntity(List<ServerPlayer> players, Level level, BlockPos pos);
}
