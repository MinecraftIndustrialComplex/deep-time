// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.terrablender;

import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionTrees;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSource;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.Optional;

/**
 * TerraBlender's regions follow Deep Time's geologic provinces (plan §7.3, P1-10).
 *
 * <p>Target: {@code MultiNoiseBiomeSource.getNoiseBiome(int, int, int, Climate.Sampler)}, quart
 * coordinates. TerraBlender 4.1.0.8 injects at HEAD, cancellable, and answers with
 * {@code findValuePositional}, which picks the region from a layered noise of (x, z) alone
 * ({@code IExtendedParameterList.getUniqueness}) and searches that region's tree. This injector
 * runs first (priority 900 against TerraBlender's default 1000, so its handler is the first at
 * HEAD) and, <em>only for a Deep Time level's sampler</em> (its spawn-target list is a
 * {@link RegionSpawnTarget}, which carries the level's province → region table), answers with the
 * same search in the region of the column's province. Anything else, and a parameter list
 * TerraBlender has not initialised, falls through to TerraBlender or vanilla unchanged.
 *
 * <p>Plan §7.3 ported ReTerraForged's four mixins (a {@code uniqueness} field on
 * {@code Climate.TargetPoint}, set by {@code Climate.Sampler.sample} from a {@code terrablender:
 * uniqueness} density function, and a redirect of {@code getUniqueness}). TerraBlender 4.1.0.8 has
 * no such density function (its uniqueness is not part of the sampler), and the parameter list it
 * indexes is shared by every level on the overworld preset, so none of that was needed; no
 * ReTerraForged code is used. Applied only when TerraBlender is installed.
 */
@Mixin(value = MultiNoiseBiomeSource.class, priority = 900)
public abstract class RegionBiomeSourceMixin {

    @Shadow
    public abstract Climate.ParameterList<Holder<Biome>> parameters();

    @Inject(method = "getNoiseBiome(IIILnet/minecraft/world/level/biome/Climate$Sampler;)Lnet/minecraft/core/Holder;",
            at = @At("HEAD"), cancellable = true)
    private void deeptime$provinceRegion(int x, int y, int z, Climate.Sampler sampler, CallbackInfoReturnable<Holder<Biome>> cir) {
        if (!(sampler.spawnTarget() instanceof RegionSpawnTarget marker))
            return;
        Optional<RegionSource> regions = marker.source();
        Climate.ParameterList<Holder<Biome>> parameters = parameters();
        if (regions.isEmpty() || !RegionTrees.ready(parameters))
            return;
        int region = regions.get().regionAt(QuartPos.toBlock(x), QuartPos.toBlock(z));
        cir.setReturnValue(RegionTrees.find(parameters, region, sampler.sample(x, y, z)));
    }
}
