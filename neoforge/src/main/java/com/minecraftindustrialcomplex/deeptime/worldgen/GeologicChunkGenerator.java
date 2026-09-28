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
            WaterPass.apply(chunk, w.data());                  // lakes above sea level (plan §7.9)
        }
    }

    @Override
    protected MapCodec<? extends ChunkGenerator> codec() {
        return CODEC;
    }
}
