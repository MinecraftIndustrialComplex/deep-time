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
        SurfaceRules.RuleSource target = settings.value().surfaceRule();
        if (target instanceof DelegatingRuleSource)
            throw new IllegalStateException("deeptime:delegate points at another delegate: " + settings);
        return planetRule(target).apply(context);
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

    /** The pack's top-level sequence less its world-depth layers; {@code target} itself when it has none. */
    static SurfaceRules.RuleSource withoutDepthLayers(SurfaceRules.RuleSource target) {
        if (!(target instanceof SurfaceRules.SequenceRuleSource seq))
            return target;
        List<SurfaceRules.RuleSource> kept = new ArrayList<>(seq.sequence().size());
        List<String> dropped = new ArrayList<>();
        for (SurfaceRules.RuleSource r : seq.sequence()) {
            BlockState layer = depthLayer(r);
            if (layer != null)
                dropped.add(net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(layer.getBlock()) + " below y "
                        + ((VerticalAnchor.Absolute) ((SurfaceRules.VerticalGradientConditionSource) ((SurfaceRules.TestRuleSource) r).ifTrue())
                        .trueAtAndBelow()).y());
            else
                kept.add(r);
        }
        if (dropped.isEmpty())
            return target;
        DeepTime.LOGGER.info("[deeptime] surface rule: the pack's world-depth layers are left out of Deep Time levels (the strata are "
                + "the rock there): {}", dropped);
        return SurfaceRules.sequence(kept.toArray(SurfaceRules.RuleSource[]::new));
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
