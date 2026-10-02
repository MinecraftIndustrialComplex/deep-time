// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.mojang.serialization.Codec;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.codecs.RecordCodecBuilder;
import net.minecraft.resources.ResourceLocation;

import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * One rock class of the {@code deeptime:rock_class} datapack registry (plan §5.4.1), read from
 * {@code data/<ns>/deeptime/rock_class/<name>.json}. Deep Time ships the 62 classes of the rock-tag
 * vocabulary; packs add or override classes with ordinary data files.
 *
 * <p>The numeric properties are relative multipliers the erosion and structure stages consume
 * (1.0 = generic crystalline basement), tuned in P1-07; {@code sources} records where each value
 * came from. {@code vanillaFallback} is the class's vanilla stand-in, the last resort of
 * {@link RockPalette} (plan §7.5), so a pack without any rock tags still resolves every class.
 *
 * @param name            the class name, as in {@code strata/classes.json} and the block tags
 *                        {@code #deeptime:rock/<name>} and {@code #deeptime:rock_fallback/<name>}
 * @param family          intrusive_igneous, volcanic, clastic_sedimentary, chemical_sedimentary,
 *                        metamorphic, special_host or unconsolidated
 * @param groups          the group tags that contain the class ({@code deeptime:rock/clastic}, …)
 * @param metamorphicPath the prograde sequence of class names
 * @param weathersTo      the class weathering produces
 * @param lithifiesTo     what a loose class becomes once buried (P1-07's structure stage)
 * @param looseEquivalent how the young, shallow part of a deposit of this class is shown (P1-07)
 * @param faciesRule      the name of the setting that forms the class (the settings the core forms
 *                        are listed in {@code tools/dev/gen_rock_classes.py})
 * @param tags            flags the deposit and structure stages key on
 */
public record RockClass(String name, String family, List<String> groups, double densityKgM3, double erodibility,
                        double diffusivity, List<String> metamorphicPath, Optional<String> weathersTo,
                        Optional<String> lithifiesTo, Optional<String> looseEquivalent, Optional<String> faciesRule, List<String> tags, Optional<ResourceLocation> vanillaFallback,
                        Map<String, String> sources, Optional<String> notes) {

    /** Class names are block-tag path segments. */
    public static final Codec<String> NAME = Codec.STRING.validate(s -> s.matches("[a-z0-9_]+")
            ? DataResult.success(s) : DataResult.error(() -> "rock class name '" + s + "' is not [a-z0-9_]+"));

    public static final Codec<RockClass> CODEC = RecordCodecBuilder.create(i -> i.group(
            NAME.fieldOf("class").forGetter(RockClass::name),
            Codec.STRING.fieldOf("family").forGetter(RockClass::family),
            Codec.STRING.listOf().optionalFieldOf("groups", List.of()).forGetter(RockClass::groups),
            Codec.doubleRange(1, 30_000).fieldOf("density_kg_m3").forGetter(RockClass::densityKgM3),
            Codec.doubleRange(1e-6, 1e6).fieldOf("erodibility").forGetter(RockClass::erodibility),
            Codec.doubleRange(1e-6, 1e6).fieldOf("diffusivity").forGetter(RockClass::diffusivity),
            NAME.listOf().optionalFieldOf("metamorphic_path", List.of()).forGetter(RockClass::metamorphicPath),
            NAME.optionalFieldOf("weathers_to").forGetter(RockClass::weathersTo),
            NAME.optionalFieldOf("lithifies_to").forGetter(RockClass::lithifiesTo),
            NAME.optionalFieldOf("loose_equivalent").forGetter(RockClass::looseEquivalent),
            Codec.STRING.optionalFieldOf("facies_rule").forGetter(RockClass::faciesRule),
            Codec.STRING.listOf().optionalFieldOf("tags", List.of()).forGetter(RockClass::tags),
            ResourceLocation.CODEC.optionalFieldOf("vanilla_fallback").forGetter(RockClass::vanillaFallback),
            Codec.unboundedMap(Codec.STRING, Codec.STRING).optionalFieldOf("sources", Map.of()).forGetter(RockClass::sources),
            Codec.STRING.optionalFieldOf("notes").forGetter(RockClass::notes)
    ).apply(i, RockClass::new));
}
