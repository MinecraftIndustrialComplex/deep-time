// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * The {@code deeptime:rock_class} datapack registry (plan §5.4): {@code data/<ns>/deeptime/rock_class/}.
 * Registered by {@code DeepTimeRegistries} through NeoForge's {@code DataPackRegistryEvent}; not
 * synced to clients (only the server and the Create World screen, which loads the datapacks
 * itself, read it).
 */
public final class RockClassRegistry {

    public static final ResourceKey<Registry<RockClass>> KEY =
            ResourceKey.createRegistryKey(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "rock_class"));

    private RockClassRegistry() {}

    /**
     * Every class by name, ordered by entry id. An entry whose {@code class} differs from its id's
     * path is still used under its {@code class}; of two entries claiming one name, the first by id
     * wins and the other is logged.
     */
    public static Map<ResourceLocation, RockClass> entries(RegistryAccess access) {
        Map<ResourceLocation, RockClass> out = new LinkedHashMap<>();
        access.registry(KEY).ifPresent(reg -> reg.entrySet().stream()
                .sorted(Comparator.comparing(e -> e.getKey().location().toString()))
                .forEach(e -> out.put(e.getKey().location(), e.getValue())));
        return out;
    }

    /** {@link #entries} keyed by class name. */
    public static Map<String, RockClass> byName(RegistryAccess access) {
        Map<String, RockClass> out = new LinkedHashMap<>();
        entries(access).forEach((id, rc) -> {
            if (!id.getPath().equals(rc.name()))
                DeepTime.LOGGER.warn("Rock class {} is named '{}' (the name should equal the file name)", id, rc.name());
            RockClass prev = out.putIfAbsent(rc.name(), rc);
            if (prev != null)
                DeepTime.LOGGER.warn("Rock class {} redefines '{}'; the first definition wins", id, rc.name());
        });
        return out;
    }
}
