// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.worldgen.LavaFeatures;
import net.minecraft.core.BlockPos;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import net.minecraft.world.level.levelgen.placement.PlacementContext;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Keeps the biomes' random lava lakes and springs out of Deep Time levels ({@link LavaFeatures}):
 * every placed feature, top-level or nested, is placed through {@code placeWithContext}, which
 * knows the generator it is placed for.
 */
@Mixin(PlacedFeature.class)
abstract class PlacedFeatureLavaMixin {

    @Inject(method = "placeWithContext", at = @At("HEAD"), cancellable = true)
    private void deeptime$noRandomLava(PlacementContext context, RandomSource source, BlockPos pos, CallbackInfoReturnable<Boolean> cir) {
        if (LavaFeatures.removed((PlacedFeature) (Object) this, context))
            cir.setReturnValue(false);
    }
}
