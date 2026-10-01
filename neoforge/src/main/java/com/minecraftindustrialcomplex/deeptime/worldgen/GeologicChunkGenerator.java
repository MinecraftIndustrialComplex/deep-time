// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.density.DeepTimeDensity;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;

/**
 * Deep Time's chunk generator (plan §7.1): vanilla's noise generator with Deep Time's router.
 *
 * <p>It stays a {@link NoiseBasedChunkGenerator} so that everything keyed on that class keeps
 * working: TerraBlender's regions, Toroidal World's stamping, structures, {@code /locate},
 * carvers, features, and Distant Horizons' vanilla-chunk mode.
 *
 * <p><b>Per-level binding.</b> The noise settings ({@code deeptime:planet}) are shared registry
 * data, but each level has its own world data and circumference. {@code ChunkMap} builds the
 * level's {@code RandomState} — and with it the router every chunk samples — from
 * {@link #generatorSettings()}, so that getter returns a copy of the settings whose Deep Time
 * density functions are bound to this generator's {@link TerrainSource}. The binding is lazy:
 * registry holders are not bound yet while the codec decodes. Everything else in the superclass
 * reads the registry settings, which differ from the bound copy only in the router.
 *
 * <p><b>Lava.</b> Vanilla measures its two lava heights from y 0 (everything open below y −54 is
 * lava; cave lakes at y −10 or lower may be). This generator measures them from the world's floor:
 * the global fluid picker is {@link LavaFluids}'s, and the router's {@code lava} function carries
 * the {@code deeptime:lava_lakes} gate. {@link LavaRules} has the rules and their settings.
 */
public class GeologicChunkGenerator extends NoiseBasedChunkGenerator {

    public static final MapCodec<GeologicChunkGenerator> CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
            BiomeSource.CODEC.fieldOf("biome_source").forGetter(g -> g.biomeSource),
            NoiseGeneratorSettings.CODEC.fieldOf("settings").forGetter(g -> g.sourceSettings),
            WorldDataRef.CODEC.fieldOf("world_data").forGetter(g -> g.dataRef)
    ).apply(i, GeologicChunkGenerator::new));

    private final Holder<NoiseGeneratorSettings> sourceSettings;
    private final WorldDataRef dataRef;
    private volatile Holder<NoiseGeneratorSettings> bound;
    private volatile TerrainSource terrain;

    public GeologicChunkGenerator(BiomeSource biomeSource, Holder<NoiseGeneratorSettings> settings, WorldDataRef dataRef) {
        super(biomeSource, settings);
        this.sourceSettings = settings;
        this.dataRef = dataRef;
        // Vanilla's picker floods everything below y -54 with lava, which is 298 blocks of a world
        // whose floor is y -352 (LavaRules). The field is opened by the access transformer; it is
        // read on first use, when the level's world data can be loaded.
        this.globalFluidPicker = com.google.common.base.Suppliers.memoize(() -> LavaFluids.picker(settings.value(), terrain()));
    }

    public WorldDataRef dataRef() {
        return dataRef;
    }

    /** The level's terrain source, chosen on first use (plan §7.1, "Terrain source"). */
    public TerrainSource terrain() {
        TerrainSource t = terrain;
        if (t == null) {
            synchronized (this) {
                if (terrain == null)
                    terrain = TerrainSources.forRef(dataRef);
                t = terrain;
            }
        }
        return t;
    }

    @Override
    public Holder<NoiseGeneratorSettings> generatorSettings() {
        Holder<NoiseGeneratorSettings> b = bound;
        if (b == null) {
            synchronized (this) {
                if (bound == null)
                    bound = Holder.direct(bindSettings(sourceSettings.value(), terrain()));
                b = bound;
            }
        }
        return b;
    }

    /** The registry settings this generator was decoded with (serialised by reference). */
    public Holder<NoiseGeneratorSettings> sourceSettings() {
        return sourceSettings;
    }

    static NoiseGeneratorSettings bindSettings(NoiseGeneratorSettings s, TerrainSource terrain) {
        int[] count = {0};
        var router = s.noiseRouter().mapAll(f -> {
            if (f instanceof DeepTimeDensity d) {
                count[0]++;
                return d.bind(terrain);
            }
            return f;
        });
        DeepTimeDensityCheck.requireBound(router);
        DeepTime.LOGGER.info("Bound {} Deep Time density functions to {} (C = {})", count[0],
                terrain.getClass().getSimpleName(), terrain.circumference());
        return new NoiseGeneratorSettings(s.noiseSettings(), s.defaultBlock(), s.defaultFluid(), router,
                s.surfaceRule(), s.spawnTarget(), s.seaLevel(), s.disableMobGeneration(), s.aquifersEnabled(),
                s.oreVeinsEnabled(), s.useLegacyRandomSource());
    }

    @Override
    public void buildSurface(net.minecraft.server.level.WorldGenRegion region, net.minecraft.world.level.StructureManager structures,
                             net.minecraft.world.level.levelgen.RandomState random, net.minecraft.world.level.chunk.ChunkAccess chunk) {
        super.buildSurface(region, structures, random, chunk);   // the pack's surface rules repaint the default block
        if (terrain() instanceof WorldDataTerrain w) {
            StrataPass.apply(chunk, w.data());
            WaterBodies.apply(chunk, w.data(), waterReach());  // lakes, rivers, banks (plan §7.9, P1-22)
            PlayaPass.apply(chunk, w.data());                  // salt on the floors of basins too dry for a lake
            IcePass.apply(chunk, w.data());                    // snow and ice over glaciated ground, whatever the biome
        }
    }

    /**
     * After the air carvers, wakes the water that hangs over a cave they opened into a river or lake
     * bed ({@link WaterBodies#wake}); the carvers themselves never touch Deep Time's water, because
     * {@link WaterBodies} put its blocks in the chunk's carving mask before they ran.
     */
    @Override
    public void applyCarvers(net.minecraft.server.level.WorldGenRegion region, long seed, net.minecraft.world.level.levelgen.RandomState random,
                             net.minecraft.world.level.biome.BiomeManager biomes, net.minecraft.world.level.StructureManager structures,
                             net.minecraft.world.level.chunk.ChunkAccess chunk, net.minecraft.world.level.levelgen.GenerationStep.Carving step) {
        super.applyCarvers(region, seed, random, biomes, structures, chunk, step);
        if (step == net.minecraft.world.level.levelgen.GenerationStep.Carving.AIR && terrain() instanceof WorldDataTerrain w)
            WaterBodies.wake(chunk, w.data(), waterReach());
    }

    /**
     * The noise cell width: how far the shore rule carries a lake's level past its last wet cell,
     * and the cells {@link WaterBodies} interpolates the generated surface over.
     */
    public int waterReach() {
        return sourceSettings.value().noiseSettings().getCellWidth();
    }

    @Override
    protected MapCodec<? extends ChunkGenerator> codec() {
        return CODEC;
    }
}
