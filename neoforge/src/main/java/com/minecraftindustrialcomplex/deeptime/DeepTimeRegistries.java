// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime;

import com.minecraftindustrialcomplex.deeptime.worldgen.DelegatingRuleSource;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.density.BeyondPoleDensity;
import com.minecraftindustrialcomplex.deeptime.worldgen.density.DepthDensity;
import com.minecraftindustrialcomplex.deeptime.worldgen.density.RasterDensity;
import com.minecraftindustrialcomplex.deeptime.worldgen.density.TerrainDensity;
import com.mojang.serialization.MapCodec;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.levelgen.DensityFunction;
import net.minecraft.world.level.levelgen.SurfaceRules;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.neoforge.registries.DeferredRegister;

/** Codec registrations. Deep Time registers no blocks or items (decisions file). */
public final class DeepTimeRegistries {

    private static final DeferredRegister<MapCodec<? extends ChunkGenerator>> CHUNK_GENERATORS =
            DeferredRegister.create(Registries.CHUNK_GENERATOR, DeepTime.MODID);
    private static final DeferredRegister<MapCodec<? extends DensityFunction>> DENSITY_FUNCTIONS =
            DeferredRegister.create(Registries.DENSITY_FUNCTION_TYPE, DeepTime.MODID);
    private static final DeferredRegister<MapCodec<? extends SurfaceRules.RuleSource>> RULES =
            DeferredRegister.create(Registries.MATERIAL_RULE, DeepTime.MODID);

    static {
        CHUNK_GENERATORS.register("geologic", () -> GeologicChunkGenerator.CODEC);
        DENSITY_FUNCTIONS.register("terrain", () -> TerrainDensity.MAP_CODEC);
        DENSITY_FUNCTIONS.register("depth", () -> DepthDensity.MAP_CODEC);
        DENSITY_FUNCTIONS.register("raster", () -> RasterDensity.MAP_CODEC);
        DENSITY_FUNCTIONS.register("beyond_pole", () -> BeyondPoleDensity.MAP_CODEC);
        RULES.register("delegate", () -> DelegatingRuleSource.MAP_CODEC);
    }

    private DeepTimeRegistries() {}

    public static void register(IEventBus modBus) {
        CHUNK_GENERATORS.register(modBus);
        DENSITY_FUNCTIONS.register(modBus);
        RULES.register(modBus);
    }
}
