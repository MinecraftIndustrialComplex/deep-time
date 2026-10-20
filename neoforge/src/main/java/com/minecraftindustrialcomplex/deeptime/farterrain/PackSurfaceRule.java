// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.farterrain;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.DelegatingRuleSource;
import it.unimi.dsi.fastutil.longs.Long2ObjectOpenHashMap;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.levelgen.RandomState;
import net.minecraft.world.level.levelgen.SurfaceRules;
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
 * (the chunk's noise) and any rule or condition a mod adds end the part run here. Thread-safe: one
 * context per thread.
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
    private final RandomState random;
    private final ChunkGenerator generator;
    private final ServerLevel level;
    private final BiomeManager biomes;
    private final int minY;
    private final ThreadLocal<Runner> runners;

    /** One thread's context and the rules applied to it (a SurfaceRule is bound to its context; {@code rule} null with no rule run here). */
    private record Runner(SurfaceRules.Context context, SurfaceRules.SurfaceRule rule, SurfaceRules.SurfaceRule band) {}

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

    private PackSurfaceRule(SurfaceRules.RuleSource rule, boolean complete, ServerLevel level, ChunkGenerator generator,
                            BiomeManager.NoiseBiomeSource quarts) {
        this.rule = rule;
        this.complete = complete;
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
        return new Runner(context, rule == null ? null : rule.apply(context), SHALLOW_BAND.apply(context));
    }

    /**
     * The part of {@code levelRule} far terrain runs (none when not even its first rule can run
     * without a chunk: {@link #top} is then null, and {@link #shallowFloor} still answers).
     * {@code quarts} gives the biome a chunk stores at a quart cell.
     */
    public static PackSurfaceRule of(ServerLevel level, ChunkGenerator generator, SurfaceRules.RuleSource levelRule,
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
        return new PackSurfaceRule(leading, complete && leading != null, level, generator, quarts);
    }

    /**
     * The block the level's surface rule gives the top block (x, y, z) of a column, as far as the
     * part run here decides it: a block; stone where the rule leaves the stone (the strata pass
     * paints it, so the rock shows), which only a rule run here in full can say; or null where
     * the rest of the rule decides (the surface table stands for it), or where the block the rule
     * gives is air or a fluid (a cut far terrain does not draw).
     *
     * @param waterHeight the y just above the water standing over the block, or {@link Integer#MIN_VALUE} for none
     *                    (SurfaceSystem's own convention)
     */
    public BlockState top(int x, int y, int z, int waterHeight) {
        Runner r = runners.get();
        if (r.rule == null)
            return null;
        r.context.updateXZ(x, z);
        r.context.updateY(1, y - minY + 1, waterHeight, x, y, z);
        BlockState s = r.rule.tryApply(x, y, z);
        if (s == null)
            return complete ? STONE : null;
        if (s.isAir() || !s.getFluidState().isEmpty())
            return null;
        return s;
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
