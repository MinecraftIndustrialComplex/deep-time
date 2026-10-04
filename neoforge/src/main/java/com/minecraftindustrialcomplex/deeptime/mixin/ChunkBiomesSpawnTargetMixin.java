// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.ModifyArg;

import java.util.List;

/**
 * A generated chunk's biomes see the level's spawn-target list, so they follow the province regions (P1-10)
 * and the volcanic biomes (decisions log 2026-10-03) like every other biome query of the level.
 *
 * <p>Target: {@code NoiseBasedChunkGenerator.doCreateBiomes}, which builds the chunk's climate sampler
 * with {@code NoiseChunk.cachedClimateSampler(router, this.settings.value().spawnTarget())}. The
 * {@code settings} field is the registry's settings, not the per-level copy {@link GeologicChunkGenerator}
 * binds (its {@code generatorSettings()}), so the sampler of a chunk being generated did not carry the
 * level's {@code RegionSpawnTarget}, and the biome-source mixins that recognise a Deep Time level by it let
 * the chunk's biomes through unchanged, while {@code RandomState}'s sampler (biome queries, {@code /locate},
 * far terrain) followed them. This hands the bound list to the chunk's sampler. Applied always (it was
 * TerraBlender-only while the regions were its only reader).
 */
@Mixin(NoiseBasedChunkGenerator.class)
public abstract class ChunkBiomesSpawnTargetMixin {

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
