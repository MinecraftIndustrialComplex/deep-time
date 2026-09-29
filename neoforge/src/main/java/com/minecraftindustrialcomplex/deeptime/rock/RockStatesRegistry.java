// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * The {@code deeptime:rock_states} datapack registry (plan §5.4, §7.5):
 * {@code data/<ns>/deeptime/rock_states/}. Not synced to clients.
 */
public final class RockStatesRegistry {

    public static final ResourceKey<Registry<RockStates>> KEY =
            ResourceKey.createRegistryKey(ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "rock_states"));

    /** One file with its id, for messages. */
    public record Named(ResourceLocation id, RockStates states) {}

    private RockStatesRegistry() {}

    /** Every file, ordered by id (the order files for one class merge in). */
    public static List<Named> entries(RegistryAccess access) {
        List<Named> out = new ArrayList<>();
        access.registry(KEY).ifPresent(reg -> reg.entrySet().stream()
                .sorted(Comparator.comparing(e -> e.getKey().location().toString()))
                .forEach(e -> out.add(new Named(e.getKey().location(), e.getValue()))));
        return out;
    }
}
