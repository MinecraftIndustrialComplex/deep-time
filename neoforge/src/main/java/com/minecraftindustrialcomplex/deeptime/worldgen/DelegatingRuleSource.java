// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Holder;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.levelgen.SurfaceRules;
import net.minecraft.world.level.levelgen.VerticalAnchor;

import java.util.ArrayList;
import java.util.List;

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
 */
public record DelegatingRuleSource(Holder<NoiseGeneratorSettings> settings) implements SurfaceRules.RuleSource {

    public static final MapCodec<DelegatingRuleSource> MAP_CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
            NoiseGeneratorSettings.CODEC.fieldOf("settings").forGetter(DelegatingRuleSource::settings)
    ).apply(i, DelegatingRuleSource::new));
    public static final KeyDispatchDataCodec<DelegatingRuleSource> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    /** The last pack rule seen and what Deep Time applies of it (one pair: the pack's rule object rarely changes). */
    private record Stripped(SurfaceRules.RuleSource from, SurfaceRules.RuleSource to) {}

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

    /** {@code target} without its world-depth layers (cached by identity). */
    static SurfaceRules.RuleSource planetRule(SurfaceRules.RuleSource target) {
        Stripped s = last;
        if (s != null && s.from == target)
            return s.to;
        SurfaceRules.RuleSource to = withoutDepthLayers(target);
        last = new Stripped(target, to);
        return to;
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
