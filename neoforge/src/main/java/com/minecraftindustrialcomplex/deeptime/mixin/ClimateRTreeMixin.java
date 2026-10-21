// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.climate.ColdBiomeSearch;
import net.minecraft.world.level.biome.Climate;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * A climate search inside {@link ColdBiomeSearch} starts cold: the tree forgets the thread's last
 * answer first, so a tie between equally near entries does not go to whatever the thread found
 * before. Every other search runs as vanilla's, warm start and all.
 */
@Mixin(Climate.RTree.class)
abstract class ClimateRTreeMixin {

    @Shadow
    @Final
    private ThreadLocal<?> lastResult;

    @Inject(method = "search", at = @At("HEAD"))
    private void deeptime$coldStart(Climate.TargetPoint target, Climate.DistanceMetric<?> metric, CallbackInfoReturnable<?> cir) {
        if (ColdBiomeSearch.active())
            lastResult.remove();
    }
}
