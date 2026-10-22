// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.farterrain;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.DelegatingRuleSource;
import it.unimi.dsi.fastutil.longs.Long2IntOpenHashMap;
import it.unimi.dsi.fastutil.longs.Long2ObjectOpenHashMap;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.Mth;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.levelgen.DensityFunction;
import net.minecraft.world.level.levelgen.NoiseSettings;
import net.minecraft.world.level.levelgen.RandomState;
import net.minecraft.world.level.levelgen.SurfaceRules;
import net.minecraft.world.level.levelgen.VerticalAnchor;
import net.minecraft.world.level.levelgen.WorldGenerationContext;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

/**
 * The part of a level's surface rule far terrain runs itself (ledger BAK-048): the leading rules of
 * the top-level sequence that the level's chunks are painted by ({@link DelegatingRuleSource#levelRule}),
 * up to the first that asks the chunk something. Vanilla's second rule already does
 * ({@code above_preliminary_surface}, which reads the chunk's noise), so a vanilla pack runs only its
 * bedrock floor here and far terrain's top block stays the surface table's ({@link FarSurface}). A
 * pack that paints by height and biome before that, as Expanded Ecosphere does (dead brain coral
 * on every warm ocean floor from y 58 to 62, sand on lukewarm ones), gets those blocks far as near.
 *
 * <p>The rules run in the game's own code: a {@link SurfaceRules.Context} like the one
 * {@code SurfaceSystem.buildSurface} makes for a chunk, with the level's random state and surface
 * system and no chunk, updated for the column and its top block as that loop does (one block of
 * stone above it, water standing over it or not, the solid column under it: far terrain has no
 * caves, and a Deep Time chunk has none before its carvers). Where the table stands in, one more
 * piece of vanilla's rule is run here on its own ({@link #shallowFloor}, BAK-050): under shallow
 * water vanilla paints the sea bed as it paints under a dry floor's top (dirt in a lukewarm ocean
 * up to 6 + surface depth blocks down, sand only deeper), so far terrain draws the table's lake bed
 * there and its sea bed below. The biome is the one the chunk's
 * biome manager gives a block, the quart cells blended by the seed's zoom, x folded into the lap.
 * A rule is run here only when everything in it reads the column, the block's height, its biome,
 * noise or the random state: {@code steep} (the chunk's heightmap), {@code above_preliminary_surface}
 * (the chunk's noise) and any rule or condition a mod adds end the part run here.
 *
 * <p>A sea bed's top block runs the whole rule ({@link #top} with water over the block, ledger
 * BAK-065), the two chunk reads answered as the chunk answers them there: {@code steep} reads the
 * chunk's {@code WORLD_SURFACE_WG} heightmap, which over the sea is the water's top, so it is false;
 * {@code above_preliminary_surface} compares the block with the noise's preliminary surface at the
 * corners of its 16-block cell, which is computed here as {@code NoiseChunk} computes it, from the
 * level's {@code initial_density_without_jaggedness}. Terralith 2.6.2 paints every warm ocean's floor
 * below the shallow band sandstone that way, where the surface table has sand. A condition or rule a
 * mod adds still ends the run where it is reached, and the table stands in.
 *
 * <p>On dry ground and a lake's bed, where the table stands in, the whole rule is asked one thing
 * (BAK-067): whether it paints water in place of the top block, as vanilla's rule paints a swamp's
 * floor at y 62 where the swamp noise is positive (Expanded Ecosphere's flooded forests and fens
 * too). {@code above_preliminary_surface} is answered as on a sea bed; {@code steep}, which on dry
 * ground reads the chunk's heightmap round the column, counts only where it does not matter: the
 * water is drawn where the rule paints it with the column steep and not steep alike. A probe of the
 * rule's paths to water ({@link #waterProbe}) leaves out every column none of them can reach, so the
 * whole rule runs on few columns. Thread-safe: one context per thread.
 */
public final class PackSurfaceRule {

    /** Conditions that read nothing of a chunk (SurfaceRules.ConditionSource.bootstrap; {@code not} is followed). */
    private static final Set<ResourceLocation> CHUNK_FREE_CONDITIONS = Set.of(   // ledger: BAK-048
            ResourceLocation.withDefaultNamespace("biome"), ResourceLocation.withDefaultNamespace("noise_threshold"),
            ResourceLocation.withDefaultNamespace("vertical_gradient"), ResourceLocation.withDefaultNamespace("y_above"),
            ResourceLocation.withDefaultNamespace("water"), ResourceLocation.withDefaultNamespace("temperature"),
            ResourceLocation.withDefaultNamespace("hole"), ResourceLocation.withDefaultNamespace("stone_depth"));
    /** Rules besides sequence, condition and block that read nothing of a chunk (badlands' bands read the surface system). */
    private static final Set<ResourceLocation> CHUNK_FREE_RULES = Set.of(ResourceLocation.withDefaultNamespace("bandlands"));

    private static final BlockState STONE = Blocks.STONE.defaultBlockState();
    private static final BlockState WATER = Blocks.WATER.defaultBlockState();

    /** What the sea-bed run gives where it reaches a rule or condition it cannot answer without a chunk (never placed). */
    private static final BlockState UNDECIDED = Blocks.STRUCTURE_VOID.defaultBlockState();
    /** A condition every block in the world meets (its y at or over the world's bottom), and its negation. */
    private static final SurfaceRules.ConditionSource ALWAYS = SurfaceRules.yBlockCheck(VerticalAnchor.bottom(), 0);
    private static final SurfaceRules.ConditionSource NEVER = SurfaceRules.not(ALWAYS);
    private static final ResourceLocation ABOVE_PRELIMINARY_SURFACE = ResourceLocation.withDefaultNamespace("above_preliminary_surface");
    private static final ResourceLocation STEEP = ResourceLocation.withDefaultNamespace("steep");
    /** {@code NoiseChunk}'s preliminary surface: the highest cell boundary whose initial density exceeds this (vanilla's constant). */
    private static final double PRELIMINARY_DENSITY = 0.390625;   // ledger: BAK-065

    /**
     * Vanilla's shallow band (ledger BAK-050): the overworld rule ({@code SurfaceRuleData.overworldLike})
     * paints a floor whose water starts at most 6 + surface depth blocks above it
     * ({@code waterStartCheck(-6, -1)}) and lies within the surface depth ({@code UNDER_FLOOR}) with
     * the rule it paints under a dry floor's top (dirt in an ocean or a lukewarm ocean, sand on a beach),
     * before its sea-bed rule (sand in a lukewarm ocean, gravel in most others) is reached. The block
     * placed here only marks that the band holds.
     */
    private static final SurfaceRules.RuleSource SHALLOW_BAND = SurfaceRules.ifTrue(SurfaceRules.waterStartCheck(-6, -1),
            SurfaceRules.ifTrue(SurfaceRules.UNDER_FLOOR, SurfaceRules.state(STONE)));

    /** The leading chunk-free rules, or null when not even the first can run without a chunk. */
    private final SurfaceRules.RuleSource rule;
    private final boolean complete;
    /** The whole rule for a sea bed's top block, the block above the preliminary surface or not (BAK-065); null with no rule. */
    private final SurfaceRules.RuleSource seaAbove, seaBelow;
    /** The whole rule with {@code steep} true, above the preliminary surface or not (BAK-067); null with no rule. */
    private final SurfaceRules.RuleSource steepAbove, steepBelow;
    /** The rule's paths to water with what a chunk answers left out ({@link #waterProbe}); null where no path ends in water. */
    private final SurfaceRules.RuleSource probe;
    /** The level's noise, for its preliminary surface ({@code NoiseChunk.computePreliminarySurfaceLevel}). */
    private final DensityFunction initial;
    private final int noiseMinY, noiseTop, cellHeight;
    private final RandomState random;
    private final ChunkGenerator generator;
    private final ServerLevel level;
    private final BiomeManager biomes;
    private final int minY;
    private final ThreadLocal<Runner> runners;

    /** One thread's context and the rules applied to it (a SurfaceRule is bound to its context; {@code rule} null with no rule run here). */
    private record Runner(SurfaceRules.Context context, SurfaceRules.SurfaceRule rule, SurfaceRules.SurfaceRule band,
                          SurfaceRules.SurfaceRule seaAbove, SurfaceRules.SurfaceRule seaBelow,
                          SurfaceRules.SurfaceRule steepAbove, SurfaceRules.SurfaceRule steepBelow, SurfaceRules.SurfaceRule probe,
                          Long2IntOpenHashMap preliminary) {}

    /** The quart cells a thread read last (at most 4,096, then it starts again). */
    private static final class QuartCache implements BiomeManager.NoiseBiomeSource {
        private final BiomeManager.NoiseBiomeSource source;
        private final ThreadLocal<Long2ObjectOpenHashMap<Holder<Biome>>> cells = ThreadLocal.withInitial(Long2ObjectOpenHashMap::new);

        QuartCache(BiomeManager.NoiseBiomeSource source) {
            this.source = source;
        }

        @Override
        public Holder<Biome> getNoiseBiome(int x, int y, int z) {
            Long2ObjectOpenHashMap<Holder<Biome>> m = cells.get();
            long key = BlockPos.asLong(x, y, z);
            Holder<Biome> b = m.get(key);
            if (b == null) {
                if (m.size() >= 4096)
                    m.clear();
                b = source.getNoiseBiome(x, y, z);
                m.put(key, b);
            }
            return b;
        }
    }

    private PackSurfaceRule(SurfaceRules.RuleSource rule, boolean complete, SurfaceRules.RuleSource whole, NoiseSettings noise,
                            ServerLevel level, ChunkGenerator generator, BiomeManager.NoiseBiomeSource quarts) {
        this.rule = rule;
        this.complete = complete;
        this.seaAbove = whole == null ? null : seaBed(whole, true, false);
        this.seaBelow = whole == null ? null : seaBed(whole, false, false);
        this.steepAbove = whole == null ? null : seaBed(whole, true, true);
        this.steepBelow = whole == null ? null : seaBed(whole, false, true);
        this.probe = whole == null ? null : waterProbe(whole);
        this.initial = level.getChunkSource().randomState().router().initialDensityWithoutJaggedness();
        this.noiseMinY = noise.minY();
        this.noiseTop = noise.minY() + noise.height();
        this.cellHeight = noise.getCellHeight();
        this.level = level;
        this.generator = generator;
        this.random = level.getChunkSource().randomState();
        // A chunk's surface rules see the biome through its region's biome manager: the stored
        // quart cells blended by the zoom of the level's seed (WorldGenRegion). The zoom reads eight
        // cells a block and neighbouring columns share them, so each thread keeps the cells it read.
        this.biomes = new BiomeManager(new QuartCache(quarts), BiomeManager.obfuscateSeed(level.getSeed()));
        this.minY = level.getMinBuildHeight();
        this.runners = ThreadLocal.withInitial(this::runner);
    }

    private Runner runner() {
        SurfaceRules.Context context = new SurfaceRules.Context(random.surfaceSystem(), random, null, null, biomes::getBiome,
                level.registryAccess().registryOrThrow(Registries.BIOME), new WorldGenerationContext(generator, level));
        return new Runner(context, rule == null ? null : rule.apply(context), SHALLOW_BAND.apply(context),
                seaAbove == null ? null : seaAbove.apply(context), seaBelow == null ? null : seaBelow.apply(context),
                steepAbove == null ? null : steepAbove.apply(context), steepBelow == null ? null : steepBelow.apply(context),
                probe == null ? null : probe.apply(context), new Long2IntOpenHashMap());
    }

    /**
     * The part of {@code levelRule} far terrain runs (none when not even its first rule can run
     * without a chunk: {@link #top} is then null, and {@link #shallowFloor} still answers).
     * {@code quarts} gives the biome a chunk stores at a quart cell.
     */
    public static PackSurfaceRule of(ServerLevel level, ChunkGenerator generator, NoiseSettings noise, SurfaceRules.RuleSource levelRule,
                                     BiomeManager.NoiseBiomeSource quarts) {
        List<SurfaceRules.RuleSource> all = levelRule instanceof SurfaceRules.SequenceRuleSource seq ? seq.sequence() : List.of(levelRule);
        List<SurfaceRules.RuleSource> run = new ArrayList<>();
        for (SurfaceRules.RuleSource r : all) {
            if (!chunkFree(r))
                break;
            run.add(r);
        }
        boolean complete = run.size() == all.size();
        DeepTime.LOGGER.info("[deeptime] far terrain: runs {} of the surface rule's {} top-level rules itself{}", run.size(), all.size(),
                complete ? " (all of them)" : "; the surface table stands for the rest");
        SurfaceRules.RuleSource leading = run.isEmpty() ? null : SurfaceRules.sequence(run.toArray(SurfaceRules.RuleSource[]::new));
        return new PackSurfaceRule(leading, complete && leading != null, levelRule, noise.clampToHeightAccessor(level), level, generator, quarts);
    }

    /**
     * The block the level's surface rule gives the top block (x, y, z) of a column, as far as the
     * part run here decides it: a block; stone where the rule leaves the stone (the strata pass
     * paints it, so the rock shows), which only a rule run here in full can say; water where the
     * rule paints water in place of the block (BAK-067: the block is water, the ground under it one
     * lower); or null where the rest of the rule decides (the surface table stands for it), or where
     * the block the rule gives is air or another fluid (a cut far terrain does not draw). Under the
     * sea the whole rule runs (BAK-065), and only a rule or condition a mod adds leaves the block to
     * the table; on dry ground and a lake's bed the whole rule is asked only whether it paints water
     * ({@link #paintsWater}).
     *
     * @param waterHeight the y just above the sea standing over the block, or {@link Integer#MIN_VALUE} for none
     *                    (SurfaceSystem's own convention; a lake's bed is run dry, as the chunk's rule saw it)
     */
    public BlockState top(int x, int y, int z, int waterHeight) {
        Runner r = runners.get();
        r.context.updateXZ(x, z);
        r.context.updateY(1, y - minY + 1, waterHeight, x, y, z);
        BlockState s = r.rule == null ? null : r.rule.tryApply(x, y, z);
        if (s == null && !complete && r.seaAbove != null) {
            if (waterHeight == Integer.MIN_VALUE)
                return paintsWater(r, x, y, z) ? WATER : null;   // dry ground or a lake's bed: the table stands in (BAK-067)
            // A sea bed: the whole rule, with the chunk's two answers there (BAK-065).
            s = (abovePreliminarySurface(r, x, y, z) ? r.seaAbove : r.seaBelow).tryApply(x, y, z);
            if (s == UNDECIDED)
                return null;
            if (s == null)
                return STONE;
        }
        if (s == null)
            return complete ? STONE : null;
        if (s.is(Blocks.WATER))
            return WATER;   // ledger: BAK-067
        if (s.isAir() || !s.getFluidState().isEmpty())
            return null;
        return s;
    }

    /**
     * Whether the level's whole rule paints water in place of the top block of a dry column or a
     * lake's bed (ledger BAK-067), the context updated for it: {@code above_preliminary_surface}
     * answered as the chunk answers it (BAK-065), and the same water whether the column is steep or
     * not ({@code steep} reads the chunk's heightmap round the column, which far terrain does not
     * build). The probe first: a column none of the rule's paths to water can reach is never run.
     */
    private boolean paintsWater(Runner r, int x, int y, int z) {
        if (r.probe == null || r.probe.tryApply(x, y, z) == null)
            return false;
        boolean above = abovePreliminarySurface(r, x, y, z);
        BlockState flat = (above ? r.seaAbove : r.seaBelow).tryApply(x, y, z);
        if (flat == null || !flat.is(Blocks.WATER))
            return false;
        BlockState steep = (above ? r.steepAbove : r.steepBelow).tryApply(x, y, z);
        return steep != null && steep.is(Blocks.WATER);
    }

    /**
     * The rule's paths to a water block as one rule ({@link #paintsWater}'s probe): each path the
     * conditions on it that read no chunk, ending in water. It gives water wherever the rule could
     * paint water (a path's left-out conditions and the rules before it can only stop it there), so
     * a column it leaves is one the rule paints no water on. Null where no path ends in water.
     */
    static SurfaceRules.RuleSource waterProbe(SurfaceRules.RuleSource r) {
        List<SurfaceRules.RuleSource> paths = new ArrayList<>();
        waterPaths(r, new ArrayList<>(), paths);
        return paths.isEmpty() ? null : SurfaceRules.sequence(paths.toArray(SurfaceRules.RuleSource[]::new));
    }

    private static void waterPaths(SurfaceRules.RuleSource r, List<SurfaceRules.ConditionSource> on, List<SurfaceRules.RuleSource> out) {
        if (r instanceof SurfaceRules.SequenceRuleSource s) {
            for (SurfaceRules.RuleSource e : s.sequence())
                waterPaths(e, on, out);
        } else if (r instanceof SurfaceRules.TestRuleSource t) {
            boolean kept = conditionChunkFree(t.ifTrue());
            if (kept)
                on.add(t.ifTrue());
            waterPaths(t.thenRun(), on, out);
            if (kept)
                on.remove(on.size() - 1);
        } else if (r instanceof SurfaceRules.BlockRuleSource b && b.resultState().is(Blocks.WATER)) {
            SurfaceRules.RuleSource path = SurfaceRules.state(WATER);
            for (int i = on.size() - 1; i >= 0; i--)
                path = SurfaceRules.ifTrue(on.get(i), path);
            out.add(path);
        }
    }

    /**
     * Whether block (x, y, z) lies at or over the minimum surface a chunk's surface rules compare it
     * with ({@code SurfaceRules.Context.getMinSurfaceLevel}, BAK-065): the preliminary surface at the
     * four corners of its 16-block cell, blended to the column and floored, plus the column's
     * surface depth, less 8. The context must be updated for the column.
     */
    private boolean abovePreliminarySurface(Runner r, int x, int y, int z) {
        int cx = x >> 4 << 4, cz = z >> 4 << 4;
        int p00 = preliminarySurface(r, cx, cz), p10 = preliminarySurface(r, cx + 16, cz);
        int p01 = preliminarySurface(r, cx, cz + 16), p11 = preliminarySurface(r, cx + 16, cz + 16);
        int level = Mth.floor(Mth.lerp2((double) ((float) (x & 15) / 16.0F), (double) ((float) (z & 15) / 16.0F), p00, p10, p01, p11));
        return y >= level + r.context.surfaceDepth - 8;
    }

    /**
     * {@code NoiseChunk.computePreliminarySurfaceLevel} at a cell corner: the highest cell boundary,
     * counted down from the noise's top, whose initial density exceeds {@link #PRELIMINARY_DENSITY}.
     */
    private int preliminarySurface(Runner r, int x, int z) {
        long key = ChunkPos.asLong(x, z);
        if (r.preliminary.containsKey(key))
            return r.preliminary.get(key);
        int level = Integer.MAX_VALUE;
        for (int y = noiseTop; y >= noiseMinY; y -= cellHeight) {
            if (initial.compute(new DensityFunction.SinglePointContext(x, y, z)) > PRELIMINARY_DENSITY) {
                level = y;
                break;
            }
        }
        if (r.preliminary.size() >= 4096)
            r.preliminary.clear();
        r.preliminary.put(key, level);
        return level;
    }

    /**
     * The level's whole rule with the two conditions that read a chunk given (BAK-065):
     * {@code above_preliminary_surface} as {@code above} and {@code steep} as {@code steep} (on a sea
     * bed false: it reads the {@code WORLD_SURFACE_WG} heightmap, the water's top over the sea; on dry
     * ground both are run, BAK-067), and a rule or condition that reads the chunk otherwise (any a mod
     * adds) replaced by {@link #UNDECIDED} where it would be reached.
     */
    static SurfaceRules.RuleSource seaBed(SurfaceRules.RuleSource r, boolean above, boolean steep) {
        if (r instanceof SurfaceRules.SequenceRuleSource s) {
            if (s.sequence().isEmpty())
                return r;
            return SurfaceRules.sequence(s.sequence().stream().map(e -> seaBed(e, above, steep)).toArray(SurfaceRules.RuleSource[]::new));
        }
        if (r instanceof SurfaceRules.TestRuleSource t) {
            SurfaceRules.ConditionSource c = seaBed(t.ifTrue(), above, steep);
            return c == null ? SurfaceRules.state(UNDECIDED) : SurfaceRules.ifTrue(c, seaBed(t.thenRun(), above, steep));
        }
        return chunkFree(r) ? r : SurfaceRules.state(UNDECIDED);
    }

    /** The stand-in for condition {@code c} (see {@link #seaBed(SurfaceRules.RuleSource, boolean, boolean)}), or null where only the chunk can answer it. */
    static SurfaceRules.ConditionSource seaBed(SurfaceRules.ConditionSource c, boolean above, boolean steep) {
        if (c instanceof SurfaceRules.NotConditionSource n) {
            SurfaceRules.ConditionSource inner = seaBed(n.target(), above, steep);
            return inner == null ? null : SurfaceRules.not(inner);
        }
        ResourceLocation key = BuiltInRegistries.MATERIAL_CONDITION.getKey(c.codec().codec());
        if (ABOVE_PRELIMINARY_SURFACE.equals(key))
            return above ? ALWAYS : NEVER;
        if (STEEP.equals(key))
            return steep ? ALWAYS : NEVER;
        return key != null && CHUNK_FREE_CONDITIONS.contains(key) ? c : null;
    }

    /**
     * Whether the top block (x, y, z) of a column under water to {@code waterHeight} (the y just
     * above the water) lies in vanilla's shallow band ({@link #SHALLOW_BAND}, ledger BAK-050), with
     * the surface depth the chunk's surface system gives the column. There the rule paints the sea
     * bed as it paints under a dry floor's top, so far terrain draws a lake's bed, not the surface
     * table's sea bed.
     */
    public boolean shallowFloor(int x, int y, int z, int waterHeight) {
        Runner r = runners.get();
        r.context.updateXZ(x, z);
        r.context.updateY(1, y - minY + 1, waterHeight, x, y, z);
        return r.band.tryApply(x, y, z) != null;
    }

    /** Whether the rule reads nothing of a chunk (every condition and rule in it is one of the known chunk-free ones). */
    static boolean chunkFree(SurfaceRules.RuleSource r) {
        if (r instanceof SurfaceRules.SequenceRuleSource s) {
            for (SurfaceRules.RuleSource e : s.sequence())
                if (!chunkFree(e))
                    return false;
            return true;
        }
        if (r instanceof SurfaceRules.TestRuleSource t)
            return conditionChunkFree(t.ifTrue()) && chunkFree(t.thenRun());
        if (r instanceof SurfaceRules.BlockRuleSource)
            return true;
        ResourceLocation key = BuiltInRegistries.MATERIAL_RULE.getKey(r.codec().codec());
        return key != null && CHUNK_FREE_RULES.contains(key);
    }

    static boolean conditionChunkFree(SurfaceRules.ConditionSource c) {
        if (c instanceof SurfaceRules.NotConditionSource n)
            return conditionChunkFree(n.target());
        ResourceLocation key = BuiltInRegistries.MATERIAL_CONDITION.getKey(c.codec().codec());
        return key != null && CHUNK_FREE_CONDITIONS.contains(key);
    }
}
