// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.createoreexcavation;

import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeBridge;
import com.tom.createores.OreData;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.chunk.LevelChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * A site vein's size follows the deposit's (P1-16, plan §7.14; {@link CoeBridge#quantile}). Target:
 * COE 1.6.8 {@code public void OreData.populate(LevelChunk)}, which sets the chunk's vein from
 * {@code OreVeinGenerator.pick} and {@code randomMul} from a chunk-seeded random float; COE's amount
 * is {@code round(((max − min)·randomMul + min)·finiteAmountBase)} ({@code getResourcesRemaining}).
 * At RETURN, when the chunk's vein is its site's, {@code randomMul} becomes the site's tonnage
 * quantile, so the recipe's own multiplier range scales the yield by the deposit's size. The state
 * is stored in COE's chunk attachment {@code createoreexcavation:ore_vein} as before. Applied only
 * when COE is installed.
 */
@Mixin(value = OreData.class, remap = false)
public abstract class OreDataMixin {

    @Inject(method = "populate", at = @At("RETURN"))
    private void deeptime$siteQuantile(LevelChunk chunk, CallbackInfo ci) {
        OreData self = (OreData) (Object) this;
        ResourceLocation vein = self.getRecipeId();
        if (vein == null || !(chunk.getLevel() instanceof ServerLevel level))
            return;
        CoeBridge.siteVein(level, chunk.getPos().x, chunk.getPos().z)
                .filter(v -> v.vein().equals(vein))
                .ifPresent(v -> self.setRandomMul(v.randomMul()));
    }
}
