// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin.compat.terrablender;

import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

import java.util.List;

/**
 * A generated chunk's biomes come from the province regions too (P1-10).
 *
 * <p>Target: {@code NoiseBasedChunkGenerator.doCreateBiomes}, which builds the chunk's climate sampler
 * with {@code NoiseChunk.cachedClimateSampler(router, this.settings.value().spawnTarget())}. The
 * {@code settings} field is the registry's settings, not the per-level copy {@link GeologicChunkGenerator}
 * binds (its {@code generatorSettings()}), so the sampler of a chunk being generated did not carry the
 * level's {@code RegionSpawnTarget} and {@link RegionBiomeSourceMixin} let TerraBlender's own regions
 * through, while {@code RandomState}'s sampler (biome queries, {@code /locate}, far terrain) followed the
 * provinces. This hands the bound list to the chunk's sampler. Applied only when TerraBlender is installed.
 */
@Mixin(NoiseBasedChunkGenerator.class)
public abstract class RegionChunkBiomesMixin {

    @ModifyArg(method = "doCreateBiomes",
            at = @At(value = "INVOKE",
                    target = "Lnet/minecraft/world/level/levelgen/NoiseChunk;cachedClimateSampler(Lnet/minecraft/world/level/levelgen/NoiseRouter;Ljava/util/List;)Lnet/minecraft/world/level/biome/Climate$Sampler;"),
            index = 1)
    private List<Climate.ParameterPoint> deeptime$levelSpawnTarget(List<Climate.ParameterPoint> registrySpawnTarget) {
        if ((Object) this instanceof GeologicChunkGenerator g)
            return g.generatorSettings().value().spawnTarget();
        return registrySpawnTarget;
    }
}
