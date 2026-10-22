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
 * Lava above that is the magma of the world's active volcanoes ({@link Magma}, {@link MagmaPass}), and
 * around them lie their hot springs and fumaroles ({@link Geothermal}, {@link GeothermalPass}).
 *
 * <p><b>TerraBlender's regions</b> follow the world's geologic provinces (P1-10): the bound settings'
 * spawn-target list carries the level's {@link RegionSource} ({@link RegionSpawnTarget}), which
 * {@code mixin.compat.terrablender.RegionBiomeSourceMixin} finds on the level's climate sampler.
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
        // Vanilla's picker floods everything below y -54 with lava, which is 458 blocks of a world
        // whose floor is y -512 (LavaRules). The field is opened by the access transformer; it is
        // read on first use, when the level's world data can be loaded.
        this.globalFluidPicker = com.google.common.base.Suppliers.memoize(() -> LavaFluids.picker(settings.value(), terrain()));
    }

    public WorldDataRef dataRef() {
        return dataRef;
    }

    /**
     * The inset of the temporary edge border of a Petroff–Guyou world (its manifest's layout block,
     * {@code border.inset_blocks}), or NaN when the level has none: a Lambert world, a test terrain, or
     * a world whose preset asked for no border ({@code scale.edge_border = none}).
     */
    public double borderInset() {
        if (!dataRef.petroffGuyou() || !(terrain() instanceof WorldDataTerrain w))
            return Double.NaN;
        var layout = w.data().layout();
        if (layout == null || layout.border() == null || !layout.border().stops("north"))
            return Double.NaN;
        return layout.border().insetBlocks();
    }

    /** The level's terrain source, chosen on first use (plan §7.1, "Terrain source"). */
    public TerrainSource terrain() {
        TerrainSource t = terrain;
        if (t == null) {
            synchronized (this) {
                if (terrain == null) {
                    TerrainSource made = TerrainSources.forRef(dataRef);
                    if (made instanceof WorldDataTerrain w)
                        w.peatlandDecider(() -> com.minecraftindustrialcomplex.deeptime.climate.PeatlandProbe.decide(
                                getBiomeSource(), w.mapping(), w.data()));
                    terrain = made;
                }
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
                s.surfaceRule(), RegionSpawnTarget.wrap(s.spawnTarget(), terrain,
                        s.noiseSettings().getCellWidth(), s.noiseSettings().minY(), s.seaLevel()), s.seaLevel(), s.disableMobGeneration(), s.aquifersEnabled(),
                s.oreVeinsEnabled(), s.useLegacyRandomSource());
    }

    /** Chunks whose surface this process built (all levels; the {@code deeptime probe dh} line reports it: real chunks Distant Horizons or the player caused). */
    private static final java.util.concurrent.atomic.LongAdder CHUNKS_SURFACED = new java.util.concurrent.atomic.LongAdder();
    private static final java.util.concurrent.atomic.AtomicLong CHUNK_X_MIN = new java.util.concurrent.atomic.AtomicLong(Long.MAX_VALUE),
            CHUNK_X_MAX = new java.util.concurrent.atomic.AtomicLong(Long.MIN_VALUE);

    public static long chunksSurfaced() {
        return CHUNKS_SURFACED.sum();
    }

    /** "min..max" chunk x of the chunks built so far, or "none". */
    public static String chunkXExtent() {
        long lo = CHUNK_X_MIN.get(), hi = CHUNK_X_MAX.get();
        return lo > hi ? "none" : lo + ".." + hi;
    }

    @Override
    public void buildSurface(net.minecraft.server.level.WorldGenRegion region, net.minecraft.world.level.StructureManager structures,
                             net.minecraft.world.level.levelgen.RandomState random, net.minecraft.world.level.chunk.ChunkAccess chunk) {
        super.buildSurface(region, structures, random, chunk);   // the pack's surface rules repaint the default block
        CHUNKS_SURFACED.increment();
        CHUNK_X_MIN.accumulateAndGet(chunk.getPos().x, Math::min);
        CHUNK_X_MAX.accumulateAndGet(chunk.getPos().x, Math::max);
        if (terrain() instanceof WorldDataTerrain w) {
            // No cave of any kind reaches the underwater face of a sea cliff: a wall of rock is kept (before the strata).
            SeaCliffPass.apply(chunk, w::height, sourceSettings.value().seaLevel(), w.cliffRules());
            StrataPass.apply(chunk, w.data());
            WaterBodies.apply(chunk, w.data(), waterReach());  // lakes, rivers, banks (plan §7.9, P1-22)
            PlayaPass.apply(chunk, w.data());                  // salt on the floors of basins too dry for a lake
            // Magma under and in the active volcanoes, after the water so a river's cut counts as ground.
            var settings = sourceSettings.value();
            MagmaPass.apply(chunk, w.magma(waterReach(), settings.noiseSettings().minY(), settings.seaLevel()));
            // Hot springs, terraces, fumaroles and altered ground around the same volcanoes (P2-14).
            GeothermalPass.apply(chunk, w.geothermal(waterReach(), settings.noiseSettings().minY(), settings.seaLevel()));
            IcePass.apply(chunk, w.data(), waterReach());      // snow and ice over glaciated ground, whatever the biome
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
     * Whether this level leaves out the biomes' random lava lakes and springs
     * ({@code underground.remove_lava_features}; {@link LavaFeatures}). A level on a test terrain has
     * no settings and takes the default.
     */
    public boolean removesLavaFeatures() {
        return terrain() instanceof WorldDataTerrain w ? w.magmaRules().removeLavaFeatures() : MagmaRules.DEFAULT.removeLavaFeatures();
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
