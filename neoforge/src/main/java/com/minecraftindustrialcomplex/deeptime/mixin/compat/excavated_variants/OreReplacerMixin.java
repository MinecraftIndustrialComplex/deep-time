// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.excavated_variants;

import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.world.level.levelgen.feature.FeaturePlaceContext;
import net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Pseudo;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Excavated Variants' own worldgen replacement ({@code excavated_variants:ore_replacer}, added to
 * every biome in {@code TOP_LAYER_MODIFICATION} after every other modifier, gated by
 * {@code attempt_worldgen_replacement} in {@code config/excavated_variants.json}, default on) stays
 * off in Deep Time levels (P1-15, plan §7.6): it re-picks each ore's stone from its neighbours with
 * chunk-edge bookkeeping, so the result depends on the order chunks generate in (a regenerated
 * deposit differed in 200 of 1,406 ore blocks, {@code DepositGameTests.depositsDeterministic}
 * under {@code -PwithEv}), and it overrides the host the strata pass chose where two rocks meet.
 * Deep Time already places EV's variant for the host ({@code OreBlockResolver}). Other levels keep
 * EV's replacement. Applied only when EV is installed ({@code DeepTimeMixinPlugin}, the
 * {@code compat.excavated_variants} package).
 */
@Pseudo
@Mixin(targets = "dev.lukebemish.excavatedvariants.impl.worldgen.OreReplacer", remap = false)
public abstract class OreReplacerMixin {

    @Inject(method = "place", at = @At("HEAD"), cancellable = true)
    private void deeptime$notInDeepTimeLevels(FeaturePlaceContext<NoneFeatureConfiguration> ctx,
                                              CallbackInfoReturnable<Boolean> cir) {
        if (ctx.chunkGenerator() instanceof GeologicChunkGenerator)
            cir.setReturnValue(false);
    }
}
