// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.minecraftindustrialcomplex.deeptime.worldgen.StructureBurialPass;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.levelgen.RandomState;
import net.minecraft.world.level.levelgen.structure.Structure;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.function.Predicate;

/**
 * Buries the structures that vanilla places at an absolute height under Deep Time's ground
 * ({@link StructureBurialPass}, ledger SCL-200): a trial chamber at y -40 to -20 stood in open sea water over a
 * trench, wrapped in a blob of deepslate. {@code Structure.generate} is where every start is made, for world
 * generation and for {@code /place structure}; the start it returns has not been stored or read yet, so the
 * move happens before anything depends on its box.
 */
@Mixin(Structure.class)
abstract class StructureBurialMixin {

    @Inject(method = "generate", at = @At("RETURN"), cancellable = true)
    private void deeptime$buryStart(RegistryAccess registryAccess, ChunkGenerator chunkGenerator, BiomeSource biomeSource,
                                    RandomState randomState, StructureTemplateManager structureTemplateManager, long seed,
                                    ChunkPos chunkPos, int references, LevelHeightAccessor heightAccessor,
                                    Predicate<Holder<Biome>> validBiome, CallbackInfoReturnable<StructureStart> cir) {
        StructureStart start = cir.getReturnValue();
        StructureStart buried = StructureBurialPass.apply((Structure) (Object) this, start, registryAccess, chunkGenerator, heightAccessor);
        if (buried != start)
            cir.setReturnValue(buried);
    }
}
