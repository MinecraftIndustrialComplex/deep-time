// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.worldgen.LakeFreeze;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.LevelReader;
import net.minecraft.world.level.biome.Biome;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * An open Deep Time lake does not freeze in a cold biome (see {@link LakeFreeze}): the climate's
 * rule decides which lakes are iced, not the biome. Only the answer "does not freeze" is ever
 * forced, and only for a lake's surface block; everything else is vanilla's.
 */
@Mixin(Biome.class)
abstract class BiomeFreezeMixin {

    @Inject(method = "shouldFreeze(Lnet/minecraft/world/level/LevelReader;Lnet/minecraft/core/BlockPos;Z)Z", at = @At("HEAD"), cancellable = true)
    private void deeptime$openLakesStayOpen(LevelReader level, BlockPos pos, boolean mustBeAtEdge, CallbackInfoReturnable<Boolean> cir) {
        if (LakeFreeze.keepsOpen(level, pos))
            cir.setReturnValue(false);
    }
}
