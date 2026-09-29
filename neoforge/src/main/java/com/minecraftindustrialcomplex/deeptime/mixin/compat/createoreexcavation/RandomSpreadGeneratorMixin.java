// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.createoreexcavation;

import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeBridge;
import com.tom.createores.recipe.VeinRecipe;
import com.tom.createores.util.RandomSpreadGenerator;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.item.crafting.RecipeHolder;
import net.minecraft.world.level.ChunkPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * COE's vein search keeps to the same rule as {@link OreVeinGeneratorMixin} (P1-16): the vein
 * finder's "found traces of … about N blocks away" and {@code /coe locate} walk COE's placement grid
 * through the private {@code RandomSpreadGenerator.pick(ServerLevel, ChunkPos, RecipeHolder)} and
 * confirm a hit through the chunk's stored vein only when the chunk is loaded, so without this they
 * would point at noise veins of claimed metals that a Deep Time level never holds. In a Deep Time
 * level a claimed vein is dropped here unless the grid chunk is that vein's site. Sites themselves
 * are not searched (prospecting is planned separately; decisions file). Applied only when COE is
 * installed.
 */
@Mixin(value = RandomSpreadGenerator.class, remap = false)
public abstract class RandomSpreadGeneratorMixin {

    @Inject(method = "pick(Lnet/minecraft/server/level/ServerLevel;Lnet/minecraft/world/level/ChunkPos;Lnet/minecraft/world/item/crafting/RecipeHolder;)Lnet/minecraft/world/item/crafting/RecipeHolder;",
            at = @At("RETURN"), cancellable = true)
    private void deeptime$dropClaimedNoise(ServerLevel level, ChunkPos chunk, RecipeHolder<VeinRecipe> last,
                                           CallbackInfoReturnable<RecipeHolder<VeinRecipe>> cir) {
        RecipeHolder<VeinRecipe> r = cir.getReturnValue();
        if (r != null && CoeBridge.dropNoiseVein(level, chunk.x, chunk.z, r.id()))
            cir.setReturnValue(null);
    }
}
