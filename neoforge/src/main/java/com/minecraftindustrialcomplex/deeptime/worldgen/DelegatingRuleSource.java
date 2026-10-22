// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import com.mojang.serialization.JsonOps;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Holder;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.levelgen.SurfaceRules;
import net.minecraft.world.level.levelgen.VerticalAnchor;
import net.neoforged.neoforge.server.ServerLifecycleHooks;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Function;

/**
 * {@code deeptime:delegate {settings}}: the surface rule of another noise settings entry —
 * the pack's {@code minecraft:overworld} — resolved each time it is applied, not when Deep
 * Time's settings load (plan §7.1). Surface-rule edits that Terralith/Lithostitched or
 * TerraBlender make to the pack's overworld therefore still paint Deep Time's terrain.
 *
 * <p>Less the pack's <b>world-depth layers</b> (decisions log 2026-10-04, Expanded Ecosphere; ledger BIO-131): a
 * top-level rule of the pack's sequence that paints one block below a fixed height (a vertical gradient between
 * two absolute heights) other than stone, deepslate or bedrock is a layer drawn for vanilla's 384-block world.
 * In a Deep Time world the rock below the surface is the strata's, which paint only stone and deepslate
 * ({@link StrataPass#isCrust}), so such a layer would be left standing down to the world's floor: Expanded
 * Ecosphere's smooth basalt below y −55 replaced every rock class under that height. Vanilla's deepslate
 * gradient and the bedrock floor stay.
 *
 * <p>Less the pack's <b>sea-level trims</b> too (ledger BIO-156): a top-level rule that turns the ground above a
 * fixed height into air, in every biome or in some, and paints nothing else. Expanded Ecosphere's turns all stone
 * above y 63 into air in its beaches, rivers, salt lakes and several oceans, which in vanilla's terrain lie at the
 * sea anyway. A Deep Time level's ground stands where the world data puts it, and a beach or a river there may lie
 * above the sea: the trim cut it down to y 62, a step of air at the biome's edge that far terrain, drawing the
 * world data, does not have (on the 2026-10-17 test planet a black beach east of the seam, its ground at y 65 to
 * 67, was cut to 62).
 *
 * <p>Less the pack's <b>lava stripes</b> in its volcanic biomes (review fix 8, 2026-10-22; ledger VOL-163): under a
 * biome condition whose every biome is one of Deep Time's volcanic biomes ({@code #deeptime:volcanic}, the biomes that
 * stand only at real volcanoes), a rule that paints blackstone or smooth basalt where a noise lies within a band about a
 * level (both ends inside the noise's range, {@link #STRIPE_BAND_MAX}) is left out; a one-sided threshold, which paints
 * patches (Terralith's Yellowstone), stays. Terralith draws a volcano's dark lines so, as contours of its
 * {@code volcano_surface} noise; in a Deep Time
 * level the lines are the volcano's own lava flows, which run downhill from its vents ({@link LavaFlows},
 * {@link LavaFlowPass}). The rest of those biomes' rules stay (Terralith's basalt and gravel ground, its crater's magma
 * blocks), and so does the same noise's paint in any other biome (its tropical jungle's). Without a server (no
 * registry to read the tag in) nothing is left out.
 */
public record DelegatingRuleSource(Holder<NoiseGeneratorSettings> settings) implements SurfaceRules.RuleSource {

    public static final MapCodec<DelegatingRuleSource> MAP_CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
            NoiseGeneratorSettings.CODEC.fieldOf("settings").forGetter(DelegatingRuleSource::settings)
    ).apply(i, DelegatingRuleSource::new));
    public static final KeyDispatchDataCodec<DelegatingRuleSource> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    /**
     * The last pack rule seen and what Deep Time applies of it (one pair: the pack's rule object rarely changes),
     * with the tag generation its volcanic biomes were read at (−1: no server, none read).
     */
    private record Stripped(SurfaceRules.RuleSource from, int tags, SurfaceRules.RuleSource to) {}

    private static volatile Stripped last;

    @Override
    public SurfaceRules.SurfaceRule apply(SurfaceRules.Context context) {
        return levelRule(this).apply(context);
    }

    /**
     * The rule a level's chunks are painted by when its noise settings hold {@code rule}: a delegate's
     * pack rule, resolved now and less its world-depth layers and sea-level trims, any other rule
     * itself. {@link #apply} paints with it; far terrain runs part of it ({@code farterrain.PackSurfaceRule}).
     */
    public static SurfaceRules.RuleSource levelRule(SurfaceRules.RuleSource rule) {
        if (!(rule instanceof DelegatingRuleSource d))
            return rule;
        SurfaceRules.RuleSource target = d.settings.value().surfaceRule();
        if (target instanceof DelegatingRuleSource)
            throw new IllegalStateException("deeptime:delegate points at another delegate: " + d.settings);
        return planetRule(target);
    }

    /** {@code target} without its world-depth layers, sea-level trims and volcanic lava stripes (cached by identity and tag generation). */
    static SurfaceRules.RuleSource planetRule(SurfaceRules.RuleSource target) {
        MinecraftServer server = ServerLifecycleHooks.getCurrentServer();
        int tags = server == null ? -1 : VolcanicBiomes.tagGeneration();
        Stripped s = last;
        if (s != null && s.from == target && s.tags == tags)
            return s.to;
        SurfaceRules.RuleSource to = withoutVolcanoStripes(withoutDepthLayers(target), volcanicBiomes(server));
        last = new Stripped(target, tags, to);
        return to;
    }

    /**
     * Whether a biome is one of Deep Time's volcanic biomes, by the server's tags (null for a biome the registry
     * lacks); null without a server.
     */
    static Function<ResourceKey<Biome>, Boolean> volcanicBiomes(MinecraftServer server) {
        if (server == null)
            return null;
        Registry<Biome> biomes = server.registryAccess().registryOrThrow(Registries.BIOME);
        return key -> biomes.getHolder(key).map(h -> h.is(VolcanicBiomes.KEEP_OUT)).orElse(null);
    }

    /** {@code rule} less its lava stripes under volcanic biome conditions ({@code rule} itself when it has none or {@code volcanic} is null). */
    static SurfaceRules.RuleSource withoutVolcanoStripes(SurfaceRules.RuleSource rule, Function<ResourceKey<Biome>, Boolean> volcanic) {
        if (volcanic == null)
            return rule;
        List<String> dropped = new ArrayList<>();
        SurfaceRules.RuleSource out = stripStripes(rule, false, volcanic, dropped);
        if (!dropped.isEmpty())
            DeepTime.LOGGER.info("[deeptime] surface rule: the pack's lava stripes in its volcanic biomes are left out of Deep Time levels "
                    + "(the volcanoes' own lava flows paint those lines, LavaFlowPass): {}", dropped);
        return out;
    }

    /** {@code r} less the stripes under a volcanic biome condition ({@code inside}: under one already). */
    private static SurfaceRules.RuleSource stripStripes(SurfaceRules.RuleSource r, boolean inside, Function<ResourceKey<Biome>, Boolean> volcanic,
                                                        List<String> dropped) {
        if (r instanceof SurfaceRules.SequenceRuleSource seq) {
            List<SurfaceRules.RuleSource> kept = new ArrayList<>(seq.sequence().size());
            List<String> here = new ArrayList<>();
            boolean changed = false;
            for (SurfaceRules.RuleSource c : seq.sequence()) {
                String stripe = inside ? stripe(c) : null;
                if (stripe != null) {
                    here.add(stripe);
                    continue;
                }
                SurfaceRules.RuleSource n = stripStripes(c, inside, volcanic, dropped);
                changed |= n != c;
                kept.add(n);
            }
            if (kept.isEmpty())
                return r;   // a sequence of nothing but stripes stays: a sequence needs a rule
            dropped.addAll(here);
            return changed || !here.isEmpty() ? SurfaceRules.sequence(kept.toArray(SurfaceRules.RuleSource[]::new)) : r;
        }
        if (r instanceof SurfaceRules.TestRuleSource t) {
            boolean in = inside || volcanicCondition(t.ifTrue(), volcanic);
            SurfaceRules.RuleSource then = stripStripes(t.thenRun(), in, volcanic, dropped);
            return then == t.thenRun() ? r : SurfaceRules.ifTrue(t.ifTrue(), then);
        }
        return r;
    }

    /**
     * Both ends of a stripe's band lie within this of zero: inside a noise's range, so the band is a contour line. A
     * threshold with no upper end of its own (Terralith's Yellowstone puts 100 there) paints patches, not lines, and stays.
     */
    static final double STRIPE_BAND_MAX = 1.5;   // ledger: VOL-163 (game-choice)

    /** A description of {@code r} if it is a lava stripe (a noise band painting blackstone or smooth basalt), else null. */
    static String stripe(SurfaceRules.RuleSource r) {
        if (r instanceof SurfaceRules.TestRuleSource t && t.thenRun() instanceof SurfaceRules.BlockRuleSource b
                && (b.resultState().is(Blocks.BLACKSTONE) || b.resultState().is(Blocks.SMOOTH_BASALT))
                && isType(t.ifTrue(), "minecraft:noise_threshold")) {
            JsonObject c = encode(t.ifTrue());
            if (c != null && "minecraft:noise_threshold".equals(type(c)) && c.has("min_threshold") && c.has("max_threshold")) {
                double lo = c.get("min_threshold").getAsDouble(), hi = c.get("max_threshold").getAsDouble();
                if (Math.abs(lo) <= STRIPE_BAND_MAX && Math.abs(hi) <= STRIPE_BAND_MAX)
                    return BuiltInRegistries.BLOCK.getKey(b.resultState().getBlock()) + " where " + c.get("noise").getAsString()
                            + " is in [" + lo + ", " + hi + "]";
            }
        }
        return null;
    }

    /** Whether {@code c} is a biome condition whose every biome the registry holds is volcanic (and it holds one). */
    static boolean volcanicCondition(SurfaceRules.ConditionSource c, Function<ResourceKey<Biome>, Boolean> volcanic) {
        if (!isType(c, "minecraft:biome"))
            return false;
        JsonObject o = encode(c);
        if (o == null || !"minecraft:biome".equals(type(o)) || !o.has("biome_is") || !o.get("biome_is").isJsonArray())
            return false;
        int n = 0;
        for (JsonElement e : o.getAsJsonArray("biome_is")) {
            ResourceLocation id = ResourceLocation.tryParse(e.getAsString());
            Boolean v = id == null ? null : volcanic.apply(ResourceKey.create(Registries.BIOME, id));
            if (Boolean.FALSE.equals(v))
                return false;
            if (v != null)
                n++;
        }
        return n > 0;
    }

    /** Whether condition {@code c} is of the registered type {@code id} (read from the registry, no encoding). */
    private static boolean isType(SurfaceRules.ConditionSource c, String id) {
        ResourceLocation k = BuiltInRegistries.MATERIAL_CONDITION.getKey(c.codec().codec());
        return k != null && id.equals(k.toString());
    }

    /** A condition as its JSON (its type and fields), or null when it does not encode. */
    private static JsonObject encode(SurfaceRules.ConditionSource c) {
        try {
            return SurfaceRules.ConditionSource.CODEC.encodeStart(JsonOps.INSTANCE, c).result()
                    .filter(JsonElement::isJsonObject).map(JsonElement::getAsJsonObject).orElse(null);
        } catch (RuntimeException e) {
            return null;
        }
    }

    private static String type(JsonObject o) {
        return o.has("type") ? o.get("type").getAsString() : "";
    }

    /** The pack's top-level sequence less its world-depth layers and sea-level trims; {@code target} itself when it has none. */
    static SurfaceRules.RuleSource withoutDepthLayers(SurfaceRules.RuleSource target) {
        if (!(target instanceof SurfaceRules.SequenceRuleSource seq))
            return target;
        List<SurfaceRules.RuleSource> kept = new ArrayList<>(seq.sequence().size());
        List<String> dropped = new ArrayList<>(), trims = new ArrayList<>();
        for (SurfaceRules.RuleSource r : seq.sequence()) {
            BlockState layer = depthLayer(r);
            Integer trim = layer == null ? seaLevelTrim(r) : null;
            if (layer != null)
                dropped.add(net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(layer.getBlock()) + " below y "
                        + ((VerticalAnchor.Absolute) ((SurfaceRules.VerticalGradientConditionSource) ((SurfaceRules.TestRuleSource) r).ifTrue())
                        .trueAtAndBelow()).y());
            else if (trim != null)
                trims.add("air from y " + trim);
            else
                kept.add(r);
        }
        if (!dropped.isEmpty())
            DeepTime.LOGGER.info("[deeptime] surface rule: the pack's world-depth layers are left out of Deep Time levels (the strata are "
                    + "the rock there): {}", dropped);
        if (!trims.isEmpty())
            DeepTime.LOGGER.info("[deeptime] surface rule: the pack's sea-level trims are left out of Deep Time levels (the ground stands "
                    + "where the world data puts it): {}", trims);
        if (dropped.isEmpty() && trims.isEmpty())
            return target;
        return SurfaceRules.sequence(kept.toArray(SurfaceRules.RuleSource[]::new));
    }

    /**
     * The height of a sea-level trim, or null when {@code r} is not one: a rule on "at or above a fixed
     * height" (no surface or stone depth added) whose every outcome is air.
     */
    static Integer seaLevelTrim(SurfaceRules.RuleSource r) {
        if (r instanceof SurfaceRules.TestRuleSource t
                && t.ifTrue() instanceof SurfaceRules.YConditionSource y
                && y.anchor() instanceof VerticalAnchor.Absolute a && y.surfaceDepthMultiplier() == 0 && !y.addStoneDepth()
                && onlyAir(t.thenRun()))
            return a.y();
        return null;
    }

    /** Whether every block {@code r} can place is air (and it places one at all). */
    static boolean onlyAir(SurfaceRules.RuleSource r) {
        if (r instanceof SurfaceRules.BlockRuleSource b)
            return b.resultState().isAir();
        if (r instanceof SurfaceRules.TestRuleSource t)
            return onlyAir(t.thenRun());
        if (r instanceof SurfaceRules.SequenceRuleSource s)
            return !s.sequence().isEmpty() && s.sequence().stream().allMatch(DelegatingRuleSource::onlyAir);
        return false;
    }

    /** The block of a world-depth layer rule, or null when {@code r} is not one. */
    static BlockState depthLayer(SurfaceRules.RuleSource r) {
        if (r instanceof SurfaceRules.TestRuleSource t
                && t.ifTrue() instanceof SurfaceRules.VerticalGradientConditionSource g
                && g.trueAtAndBelow() instanceof VerticalAnchor.Absolute && g.falseAtAndAbove() instanceof VerticalAnchor.Absolute
                && t.thenRun() instanceof SurfaceRules.BlockRuleSource b) {
            BlockState s = b.resultState();
            if (s.is(Blocks.STONE) || s.is(Blocks.DEEPSLATE) || s.is(Blocks.BEDROCK))
                return null;
            return s;
        }
        return null;
    }

    @Override
    public KeyDispatchDataCodec<? extends SurfaceRules.RuleSource> codec() {
        return CODEC;
    }
}
