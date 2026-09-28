// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.mojang.serialization.MapCodec;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.core.Holder;
import net.minecraft.util.KeyDispatchDataCodec;
import net.minecraft.world.level.levelgen.NoiseGeneratorSettings;
import net.minecraft.world.level.levelgen.SurfaceRules;

/**
 * {@code deeptime:delegate {settings}}: the surface rule of another noise settings entry —
 * the pack's {@code minecraft:overworld} — resolved each time it is applied, not when Deep
 * Time's settings load (plan §7.1). Surface-rule edits that Terralith/Lithostitched or
 * TerraBlender make to the pack's overworld therefore still paint Deep Time's terrain.
 */
public record DelegatingRuleSource(Holder<NoiseGeneratorSettings> settings) implements SurfaceRules.RuleSource {

    public static final MapCodec<DelegatingRuleSource> MAP_CODEC = RecordCodecBuilder.mapCodec(i -> i.group(
            NoiseGeneratorSettings.CODEC.fieldOf("settings").forGetter(DelegatingRuleSource::settings)
    ).apply(i, DelegatingRuleSource::new));
    public static final KeyDispatchDataCodec<DelegatingRuleSource> CODEC = KeyDispatchDataCodec.of(MAP_CODEC);

    @Override
    public SurfaceRules.SurfaceRule apply(SurfaceRules.Context context) {
        SurfaceRules.RuleSource target = settings.value().surfaceRule();
        if (target instanceof DelegatingRuleSource)
            throw new IllegalStateException("deeptime:delegate points at another delegate: " + settings);
        return target.apply(context);
    }

    @Override
    public KeyDispatchDataCodec<? extends SurfaceRules.RuleSource> codec() {
        return CODEC;
    }
}
