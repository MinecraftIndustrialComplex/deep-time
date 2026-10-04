// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * The pack's volcanic biomes only at real volcanoes (decisions log 2026-10-03; {@link VolcanicBiomes}), on a
 * plain multi-noise biome source (vanilla, or Terralith's parameter list, as in MIC).
 *
 * <p>Target: {@code MultiNoiseBiomeSource.getNoiseBiome(int, int, int, Climate.Sampler)}, quart coordinates.
 * Only a Deep Time level's sampler is touched (its spawn-target list is a {@link RegionSpawnTarget}), and only
 * when its parameter list holds a biome of {@code #deeptime:volcanic} or a volcanic role's biome; otherwise the
 * method runs as vanilla's. Priority 1100 puts this after TerraBlender's own HEAD injector (1000), which answers
 * every query when TerraBlender is installed: with TerraBlender the same work is done by
 * {@code mixin.compat.terrablender.RegionBiomeSourceMixin} (priority 900), in the region's tree.
 */
@Mixin(value = MultiNoiseBiomeSource.class, priority = 1100)
public abstract class VolcanicBiomeSourceMixin {

    @Shadow
    public abstract Climate.ParameterList<Holder<Biome>> parameters();

    @Inject(method = "getNoiseBiome(IIILnet/minecraft/world/level/biome/Climate$Sampler;)Lnet/minecraft/core/Holder;",
            at = @At("HEAD"), cancellable = true)
    private void deeptime$volcanicBiomes(int x, int y, int z, Climate.Sampler sampler, CallbackInfoReturnable<Holder<Biome>> cir) {
        if (!(sampler.spawnTarget() instanceof RegionSpawnTarget marker))
            return;
        Holder<Biome> b = VolcanicBiomes.find(marker.volcanic(), new VolcanicBiomes.ListSource((BiomeSource) (Object) this, parameters()),
                x, y, z, sampler);
        if (b != null)
            cir.setReturnValue(b);
    }
}
