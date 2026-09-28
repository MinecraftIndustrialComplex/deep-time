// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.TagsUpdatedEvent;

import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Rock class → block (plan §7.5), v0: the first member of {@code #deeptime:rock/<class>}, else
 * nothing (the class stays unclassified crust). Deep Time ships vanilla member tags; packs add
 * their own blocks with ordinary tags. Fallback tags, {@code rock_states} and the per-class
 * {@code vanilla_fallback} arrive with P1-12. Rebuilt when tags reload.
 */
public final class RockPalette {

    private static final Map<String, Optional<BlockState>> CACHE = new ConcurrentHashMap<>();

    private RockPalette() {}

    public static void init() {
        NeoForge.EVENT_BUS.addListener(TagsUpdatedEvent.class, e -> CACHE.clear());
    }

    public static Optional<BlockState> resolve(String rockClass) {
        return CACHE.computeIfAbsent(rockClass, RockPalette::lookup);
    }

    private static Optional<BlockState> lookup(String rockClass) {
        TagKey<Block> tag = TagKey.create(Registries.BLOCK, ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "rock/" + rockClass));
        var members = BuiltInRegistries.BLOCK.getTag(tag);
        Optional<BlockState> state = members.flatMap(set -> set.stream().findFirst()).map(h -> h.value().defaultBlockState());
        if (state.isEmpty())
            DeepTime.LOGGER.warn("Rock class '{}' has no block (#{} is empty); it stays unclassified crust", rockClass, tag.location());
        return state;
    }
}
