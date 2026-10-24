// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.server.Staging;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;
import net.minecraft.world.level.levelgen.presets.WorldPreset;

import java.nio.file.Path;
import java.util.Optional;

/**
 * The Deep Time overworld generator of a world being created (plan §7.12), as plain functions of
 * the world's dimensions so the Create World screen's Accept and the GameTests share them:
 *
 * <ul>
 *   <li>{@link #withStaging}: Accept's {@code updateDimensions} — the overworld becomes a
 *       {@link GeologicChunkGenerator} bound to a staging folder and the preset's circumference. The
 *       Toroidal World shape follows from that circumference when the dimensions are baked
 *       ({@code PlanetShape}, {@code WorldDimensionsShapeMixin}).</li>
 *   <li>{@link #createBlock}: whether a world with these dimensions may be created yet. A Deep Time
 *       world needs world data, and in singleplayer that means a finished, accepted run: there is no
 *       default story (decisions file, "Timeline").</li>
 * </ul>
 */
public final class PlanetGenerators {

    /**
     * The Deep Time world types, whose Customize button opens the planet editor: the cube world (format
     * 0.7, docs/plans/cube-world-plan.md), 4s = 15,360 at 16k and 29,696 at 32k. The only two since the
     * Lambert and Petroff–Guyou types went (cube plan §7.4, decisions log 2026-10-24: "Cube only"); the
     * ids keep their {@code _cube} names.
     */
    public static final ResourceKey<WorldPreset> PLANET_CUBE = ResourceKey.create(Registries.WORLD_PRESET,
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "planet_cube"));
    public static final ResourceKey<WorldPreset> PLANET_CUBE_32K = ResourceKey.create(Registries.WORLD_PRESET,
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "planet_cube_32k"));

    private PlanetGenerators() {}

    /** The Deep Time generator of the overworld, if it is one. */
    public static Optional<GeologicChunkGenerator> overworld(WorldDimensions dims) {
        LevelStem stem = dims.dimensions().get(LevelStem.OVERWORLD);
        return stem != null && stem.generator() instanceof GeologicChunkGenerator g ? Optional.of(g) : Optional.empty();
    }

    /**
     * The dimensions with the overworld bound to staging folder {@code stagingId}, the world's x period
     * {@code circumference} and its {@code projection} ({@code cube}, or {@code lambert} for a synthetic
     * test world: the accepted run's, {@link com.minecraftindustrialcomplex.deeptime.preset.Presets.Info#worldCircumference}).
     * The biome source and noise settings are the current Deep Time overworld's, else those of the
     * {@code deeptime:planet_cube} world preset.
     */
    public static WorldDimensions withStaging(RegistryAccess registries, WorldDimensions dims, String stagingId, int circumference,
                                              String projection) {
        if (!Staging.isId(stagingId))
            throw new IllegalArgumentException("not a staging id: " + stagingId);
        GeologicChunkGenerator base = overworld(dims).orElseGet(() -> presetOverworld(registries));
        WorldDataRef old = base.dataRef();
        // The folder stays the world preset's.
        WorldDataRef ref = new WorldDataRef(old.folder(), circumference, Optional.empty(), Optional.of(stagingId), projection);
        ChunkGenerator generator = new GeologicChunkGenerator(base.getBiomeSource(), base.sourceSettings(), ref);
        return dims.replaceOverworldGenerator(registries, generator);
    }

    private static GeologicChunkGenerator presetOverworld(RegistryAccess registries) {
        WorldPreset preset = registries.registryOrThrow(Registries.WORLD_PRESET).getOrThrow(PLANET_CUBE);
        return preset.overworld().map(LevelStem::generator).filter(GeologicChunkGenerator.class::isInstance)
                .map(GeologicChunkGenerator.class::cast)
                .orElseThrow(() -> new IllegalStateException(PLANET_CUBE.location() + " has no Deep Time overworld"));
    }

    /** Why a world cannot be created yet. */
    public enum CreateBlock {
        /** Not a Deep Time world, a test terrain, or staged world data is ready. */
        NONE,
        /** The natives did not load and nothing is staged (plan §6.6: Create is disabled). */
        NATIVES,
        /** No run was accepted in the planet editor. */
        NOT_SIMULATED,
        /** The accepted run's staging folder is gone (cleaned up, or a re-created world's old id). */
        STAGING_MISSING
    }

    /** Whether the Create button may create a world with {@code dims} (see {@link CreateBlock}). */
    public static CreateBlock createBlock(WorldDimensions dims, Path gameDir, boolean nativesLinked) {
        Optional<GeologicChunkGenerator> g = overworld(dims);
        if (g.isEmpty() || g.get().dataRef().testTerrain().isPresent())
            return CreateBlock.NONE;
        Optional<String> id = g.get().dataRef().stagingId();
        if (id.isPresent() && Staging.isId(id.get()) && Staging.complete(Staging.dir(gameDir, id.get())))
            return CreateBlock.NONE;
        if (!nativesLinked)
            return CreateBlock.NATIVES;
        return id.isPresent() ? CreateBlock.STAGING_MISSING : CreateBlock.NOT_SIMULATED;
    }
}
