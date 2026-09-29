// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.createoreexcavation;

import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeBridge;
import com.tom.createores.OreVeinGenerator;
import com.tom.createores.recipe.VeinRecipe;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.item.crafting.RecipeHolder;
import net.minecraft.world.level.chunk.LevelChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Create Ore Excavation's vein choice in Deep Time levels (P1-16, plan §7.14; {@link CoeBridge}).
 * Target: COE 1.6.8 {@code public static RecipeHolder<VeinRecipe> OreVeinGenerator.pick(LevelChunk)}
 * (null = no vein), which {@code OreData.populate} calls the first time a chunk's vein is read.
 *
 * <ul>
 *   <li>HEAD: a chunk a site's ore reaches is that site's vein (the recipe from the level's
 *       {@code RecipeManager}).</li>
 *   <li>RETURN: COE's noise grid does not hand out a claimed vein (one a Deep Time deposit type
 *       names) anywhere else.</li>
 * </ul>
 * Outside Deep Time levels both do nothing. Applied only when COE is installed
 * ({@code DeepTimeMixinPlugin}, the {@code compat.createoreexcavation} package).
 */
@Mixin(value = OreVeinGenerator.class, remap = false)
public abstract class OreVeinGeneratorMixin {

    @Inject(method = "pick", at = @At("HEAD"), cancellable = true)
    private static void deeptime$siteVein(LevelChunk chunk, CallbackInfoReturnable<RecipeHolder<VeinRecipe>> cir) {
        if (!(chunk.getLevel() instanceof ServerLevel level))
            return;
        CoeBridge.siteVein(level, chunk.getPos().x, chunk.getPos().z).ifPresent(v -> {
            RecipeHolder<VeinRecipe> r = deeptime$recipe(level, v.vein());
            if (r != null)
                cir.setReturnValue(r);
            else
                CoeBridge.missingRecipe(v);
        });
    }

    @Inject(method = "pick", at = @At("RETURN"), cancellable = true)
    private static void deeptime$dropClaimedNoise(LevelChunk chunk, CallbackInfoReturnable<RecipeHolder<VeinRecipe>> cir) {
        RecipeHolder<VeinRecipe> r = cir.getReturnValue();
        if (r != null && chunk.getLevel() instanceof ServerLevel level
                && CoeBridge.dropNoiseVein(level, chunk.getPos().x, chunk.getPos().z, r.id()))
            cir.setReturnValue(null);
    }

    @Unique
    @SuppressWarnings("unchecked")
    private static RecipeHolder<VeinRecipe> deeptime$recipe(ServerLevel level, ResourceLocation id) {
        return level.getRecipeManager().byKey(id)
                .filter(h -> h.value() instanceof VeinRecipe)
                .map(h -> (RecipeHolder<VeinRecipe>) h)
                .orElse(null);
    }
}
