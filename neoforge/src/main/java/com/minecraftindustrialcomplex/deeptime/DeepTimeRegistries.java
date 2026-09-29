// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime;

import com.minecraftindustrialcomplex.deeptime.ore.ClaimOresBiomeModifier;
import com.minecraftindustrialcomplex.deeptime.ore.OreData;
import com.minecraftindustrialcomplex.deeptime.rock.RockClass;
import com.minecraftindustrialcomplex.deeptime.rock.RockClassRegistry;
import com.minecraftindustrialcomplex.deeptime.rock.RockStates;
import com.minecraftindustrialcomplex.deeptime.rock.RockStatesRegistry;
import com.minecraftindustrialcomplex.deeptime.worldgen.BackgroundScatterFeature;
import com.minecraftindustrialcomplex.deeptime.worldgen.DelegatingRuleSource;
import com.minecraftindustrialcomplex.deeptime.worldgen.DepositFeature;
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
import net.minecraft.world.level.levelgen.feature.Feature;
import net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration;
import net.neoforged.neoforge.common.world.BiomeModifier;
import net.neoforged.neoforge.registries.NeoForgeRegistries;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.neoforge.registries.DataPackRegistryEvent;
import net.neoforged.neoforge.registries.DeferredRegister;

/** Codec registrations. Deep Time registers no blocks or items (decisions file). */
public final class DeepTimeRegistries {

    private static final DeferredRegister<MapCodec<? extends ChunkGenerator>> CHUNK_GENERATORS =
            DeferredRegister.create(Registries.CHUNK_GENERATOR, DeepTime.MODID);
    private static final DeferredRegister<MapCodec<? extends DensityFunction>> DENSITY_FUNCTIONS =
            DeferredRegister.create(Registries.DENSITY_FUNCTION_TYPE, DeepTime.MODID);
    private static final DeferredRegister<MapCodec<? extends SurfaceRules.RuleSource>> RULES =
            DeferredRegister.create(Registries.MATERIAL_RULE, DeepTime.MODID);
    private static final DeferredRegister<Feature<?>> FEATURES = DeferredRegister.create(Registries.FEATURE, DeepTime.MODID);
    private static final DeferredRegister<MapCodec<? extends BiomeModifier>> BIOME_MODIFIERS =
            DeferredRegister.create(NeoForgeRegistries.Keys.BIOME_MODIFIER_SERIALIZERS, DeepTime.MODID);

    /** {@code deeptime:deposits} (plan §7.6). */
    public static final net.neoforged.neoforge.registries.DeferredHolder<Feature<?>, DepositFeature> DEPOSITS =
            FEATURES.register("deposits", () -> new DepositFeature(NoneFeatureConfiguration.CODEC));
    /** {@code deeptime:background_scatter} (plan §7.8). */
    public static final net.neoforged.neoforge.registries.DeferredHolder<Feature<?>, BackgroundScatterFeature> BACKGROUND_SCATTER =
            FEATURES.register("background_scatter", () -> new BackgroundScatterFeature(NoneFeatureConfiguration.CODEC));

    static {
        CHUNK_GENERATORS.register("geologic", () -> GeologicChunkGenerator.CODEC);
        DENSITY_FUNCTIONS.register("terrain", () -> TerrainDensity.MAP_CODEC);
        DENSITY_FUNCTIONS.register("depth", () -> DepthDensity.MAP_CODEC);
        DENSITY_FUNCTIONS.register("raster", () -> RasterDensity.MAP_CODEC);
        DENSITY_FUNCTIONS.register("beyond_pole", () -> BeyondPoleDensity.MAP_CODEC);
        RULES.register("delegate", () -> DelegatingRuleSource.MAP_CODEC);
        BIOME_MODIFIERS.register("claim_ores", () -> ClaimOresBiomeModifier.CODEC);
    }

    private DeepTimeRegistries() {}

    public static void register(IEventBus modBus) {
        CHUNK_GENERATORS.register(modBus);
        DENSITY_FUNCTIONS.register(modBus);
        RULES.register(modBus);
        FEATURES.register(modBus);
        BIOME_MODIFIERS.register(modBus);
        // The datapack registries of plan §5.4: data/<ns>/deeptime/rock_class/, rock_states/,
        // deposit_type/ (Java reads only products and the ore-block policy), scatter/ and
        // essentials/ (P1-15). No network codec: clients never need them (the Create World screen
        // loads the datapacks itself). sim_field joins in Phase 3.
        modBus.addListener(DataPackRegistryEvent.NewRegistry.class, e -> {
            e.dataPackRegistry(RockClassRegistry.KEY, RockClass.CODEC);
            e.dataPackRegistry(RockStatesRegistry.KEY, RockStates.CODEC);
            e.dataPackRegistry(OreData.DEPOSIT_TYPE, OreData.DepositTypeInfo.CODEC);
            e.dataPackRegistry(OreData.SCATTER, OreData.ScatterRule.CODEC);
            e.dataPackRegistry(OreData.ESSENTIALS, OreData.Essential.CODEC);
        });
    }
}
