// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.mojang.serialization.Lifecycle;
import net.minecraft.core.MappedRegistry;
import net.minecraft.core.RegistrationInfo;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import net.minecraft.world.level.biome.MultiNoiseBiomeSourceParameterLists;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;

import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Deep Time's generator wins in a Deep Time world when a pack overrides the overworld dimension (Ben,
 * decisions log 2026-10-04: "Build out support for both so if I install EE it works or if I go w/ terralith it
 * also works").
 *
 * <p>Vanilla's {@code WorldDimensions.bake} takes each dimension from the datapacks' {@code dimension} registry
 * first and from the world preset (or level.dat) only where no datapack defines it. Expanded Ecosphere's
 * built-in datapack {@code ee_default} defines {@code minecraft:overworld} (a vanilla noise generator with its
 * own 4,589-entry multi-noise biome list), so as packaged it replaces Deep Time's generator in a world created
 * with {@code deeptime:planet}: the world is an ordinary Expanded Ecosphere world under a Deep Time name. Here,
 * before that bake ({@code mixin.WorldDimensionsShapeMixin}):
 *
 * <ul>
 *   <li>If the world's own overworld (preset or level.dat) is a {@link GeologicChunkGenerator} and a datapack's
 *       overworld is not, the datapack's overworld is set aside: the world keeps Deep Time's generator, noise
 *       settings and dimension type.</li>
 *   <li>Every Deep Time generator whose biome source is vanilla's overworld parameter list (the preset's
 *       {@code "preset": "minecraft:overworld"}: "the pack's overworld biomes") takes the overriding dimension's
 *       biome source instead, so the pack's biomes reach the Deep Time world, placed by Deep Time's climate and
 *       its biome rules ({@code climate.BiomeRules}). That holds for the overworld and for any other Deep Time
 *       level the datapacks define (the GameTest levels).</li>
 *   <li>A world whose overworld is not Deep Time's (a vanilla preset) keeps the pack's override as the pack
 *       intends.</li>
 * </ul>
 *
 * <p>The adopted biome list is stored in level.dat with the generator, so the world keeps it across restarts;
 * removing the pack then leaves level.dat naming biomes that no longer exist (no compatibility is kept before
 * a public release).
 */
public final class OverworldOverride {

    private OverworldOverride() {}

    /** The dimensions and the datapack dimensions to bake, after the override is dealt with. */
    public record Result(WorldDimensions dims, Registry<LevelStem> stems) {}

    /**
     * What to bake instead of ({@code dims}, {@code stems}); the same objects when no datapack overrides the
     * overworld with a generator other than Deep Time's, or when nothing changes. Idempotent.
     */
    public static Result apply(WorldDimensions dims, Registry<LevelStem> stems) {
        LevelStem override = stems.get(LevelStem.OVERWORLD);
        if (override == null || override.generator() instanceof GeologicChunkGenerator)
            return new Result(dims, stems);
        BiomeSource biomes = override.generator().getBiomeSource();
        LevelStem own = dims.dimensions().get(LevelStem.OVERWORLD);
        boolean deepTimeWorld = own != null && own.generator() instanceof GeologicChunkGenerator;

        WorldDimensions outDims = dims;
        if (deepTimeWorld) {
            LevelStem adopted = adopt(own, biomes);
            if (adopted != own) {
                Map<ResourceKey<LevelStem>, LevelStem> m = new LinkedHashMap<>(dims.dimensions());
                m.put(LevelStem.OVERWORLD, adopted);
                outDims = new WorldDimensions(m);
            }
        }
        boolean changed = deepTimeWorld;
        for (var e : stems.entrySet())
            if (!e.getKey().equals(LevelStem.OVERWORLD) && adopt(e.getValue(), biomes) != e.getValue())
                changed = true;
        if (!changed)
            return new Result(dims, stems);

        MappedRegistry<LevelStem> out = new MappedRegistry<>(Registries.LEVEL_STEM, Lifecycle.stable());
        int adoptedLevels = 0;
        for (var e : stems.entrySet()) {
            ResourceKey<LevelStem> key = e.getKey();
            if (deepTimeWorld && key.equals(LevelStem.OVERWORLD))
                continue;   // the world's own (Deep Time) overworld is baked from dims
            LevelStem stem = key.equals(LevelStem.OVERWORLD) ? e.getValue() : adopt(e.getValue(), biomes);
            if (stem != e.getValue())
                adoptedLevels++;
            out.register(key, stem, stems.registrationInfo(key).orElse(RegistrationInfo.BUILT_IN));
        }
        out.freeze();
        String what = override.generator().getClass().getSimpleName() + " with " + describe(biomes);
        if (deepTimeWorld)
            DeepTime.LOGGER.info("[deeptime] overworld: a datapack overrides minecraft:overworld ({}); this is a Deep Time world, so "
                    + "Deep Time's generator stays and takes that biome source{}", what, adoptedLevels > 0
                    ? " (as do " + adoptedLevels + " other Deep Time levels)" : "");
        else
            DeepTime.LOGGER.info("[deeptime] overworld: a datapack overrides minecraft:overworld ({}); not a Deep Time world, so the "
                    + "override stays; {} Deep Time levels take that biome source", what, adoptedLevels);
        return new Result(outDims, out);
    }

    /**
     * {@code stem} with the biome source of {@code override} (a datapack's overworld) when that is not Deep Time's own
     * and {@code stem} is a Deep Time level on vanilla's overworld parameter list; else {@code stem} itself. For the
     * GameTest server, which bakes the datapacks' Deep Time levels alone ({@code mixin.gametest.GameTestServerMixin}).
     */
    public static LevelStem adopt(LevelStem stem, LevelStem override) {
        if (override == null || override.generator() instanceof GeologicChunkGenerator)
            return stem;
        return adopt(stem, override.generator().getBiomeSource());
    }

    /** {@code stem} with {@code biomes} when it is a Deep Time level on vanilla's overworld parameter list; else itself. */
    static LevelStem adopt(LevelStem stem, BiomeSource biomes) {
        if (!(stem.generator() instanceof GeologicChunkGenerator g) || !onOverworldPreset(g.getBiomeSource()))
            return stem;
        ChunkGenerator generator = new GeologicChunkGenerator(biomes, g.sourceSettings(), g.dataRef());
        return new LevelStem(stem.type(), generator);
    }

    /** Whether a biome source is vanilla's multi-noise overworld preset ({@code "preset": "minecraft:overworld"}). */
    static boolean onOverworldPreset(BiomeSource source) {
        return source instanceof MultiNoiseBiomeSource m && m.stable(MultiNoiseBiomeSourceParameterLists.OVERWORLD);
    }

    private static String describe(BiomeSource s) {
        if (s instanceof MultiNoiseBiomeSource m)
            return "a multi-noise biome source of " + m.possibleBiomes().size() + " biomes";
        return s.getClass().getSimpleName();
    }
}
