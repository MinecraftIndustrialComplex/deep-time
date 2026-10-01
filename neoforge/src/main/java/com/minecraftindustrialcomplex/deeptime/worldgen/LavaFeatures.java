// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.levelgen.feature.ConfiguredFeature;
import net.minecraft.world.level.levelgen.placement.PlacedFeature;
import net.minecraft.world.level.levelgen.placement.PlacementContext;

import java.util.Map;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The biomes' random lava, kept out of Deep Time levels ({@code underground.remove_lava_features},
 * on by default; Ben, decisions log 2026-10-01: lava follows the volcanoes). A placed feature is a
 * <b>lava feature</b> when its configured feature's own data places lava: a block or fluid state
 * {@code {"Name": "minecraft:lava"}} anywhere in its configuration. In the 168-mod MIC set that is
 * vanilla's {@code lake_lava_surface}, {@code lake_lava_underground}, {@code spring_lava} and
 * {@code spring_lava_frozen}, and Terralith's {@code volcano/lava_lakes}, {@code volcano/lava_flow},
 * {@code volcano/spring_lava} and {@code cave/mantle/lava_drip}; any other mod's lava lake or
 * spring is caught the same way, with no list to maintain. A feature that only names lava as
 * something to replace ({@code netherrack_replace_blobs}, Terralith's {@code lava_remover}) is not
 * one.
 *
 * <p>It is decided where features are placed ({@code PlacedFeature.placeWithContext}, through
 * {@code mixin.PlacedFeatureLavaMixin}), not with a biome modifier: biome modifiers are applied
 * once per server, before a new world's data is in place, and to every dimension alike. Here the
 * generator placing the feature answers, so the setting is the level's own and other dimensions
 * keep their lava. The verdict per feature is cached; the check is one map lookup.
 */
public final class LavaFeatures {

    private LavaFeatures() {}

    /** Feature types whose configuration names lava without placing it. */
    private static final Set<ResourceLocation> NOT_PLACING = Set.of(ResourceLocation.withDefaultNamespace("netherrack_replace_blobs"));

    private static final Map<ConfiguredFeature<?, ?>, Boolean> VERDICTS = new ConcurrentHashMap<>();
    private static final Set<String> LOGGED = ConcurrentHashMap.newKeySet();

    /** Whether {@code feature} must not be placed: a lava feature in a Deep Time level that removes them. */
    public static boolean removed(PlacedFeature feature, PlacementContext context) {
        if (!(context.generator() instanceof GeologicChunkGenerator g) || !g.removesLavaFeatures())
            return false;
        ConfiguredFeature<?, ?> cf = feature.feature().value();
        Boolean lava = VERDICTS.get(cf);
        if (lava == null) {
            lava = placesLava(cf, context);
            VERDICTS.put(cf, lava);
            if (lava) {
                String id = feature.feature().unwrapKey().map(k -> k.location().toString()).orElse("(inline)");
                if (LOGGED.add(id))
                    DeepTime.LOGGER.info("[deeptime] lava feature {} ({}) is not placed in Deep Time levels "
                            + "(underground.remove_lava_features)", id, BuiltInRegistries.FEATURE.getKey(cf.feature()));
            }
        }
        return lava;
    }

    /** The configured features found to place lava so far (ids, for probes and tests). */
    public static Set<String> seen() {
        return Set.copyOf(LOGGED);
    }

    private static boolean placesLava(ConfiguredFeature<?, ?> cf, PlacementContext context) {
        ResourceLocation type = BuiltInRegistries.FEATURE.getKey(cf.feature());
        if (type != null && NOT_PLACING.contains(type))
            return false;
        try {
            var ops = context.getLevel().registryAccess().createSerializationContext(JsonOps.INSTANCE);
            return ConfiguredFeature.DIRECT_CODEC.encodeStart(ops, cf).result().map(LavaFeatureRule::namesLavaState).orElse(false);
        } catch (RuntimeException e) {
            return false;   // a configuration that cannot be written out is left alone
        }
    }
}
