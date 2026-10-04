// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.Role;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

/**
 * The geology gate (Ben, decisions log 2026-10-04, "Deep Time-side fixes (Recommended)"; research note
 * biome-mods-survey.md, fix 3): a pack's rock-named biomes go only on the ground they are named for. Each
 * {@link Role} of {@link GeologyGround} has a tag {@code #deeptime:geology/<role>} of the biomes that belong on its
 * ground (Terralith's Shield on shield provinces, Yosemite on exposed granite, White, Granite and Basalt Cliffs on
 * coasts of carbonate, granite and basalt, the red-bed canyons on dry sedimentary rock, Stony Spires on humid
 * carbonate); {@code #deeptime:geology} holds them all. The climate search of a Deep Time level
 * ({@link VolcanicBiomes#find}) keeps such a biome out of every column whose ground has none of its roles; the next
 * nearest biome of the pack's own tree answers there. On its ground the biome answers where the climate search puts
 * it: the gate takes biomes away, it never adds them. A biome in a geology role and in {@code #deeptime:volcanic}
 * (Terralith's Basalt Cliffs) goes at real volcanoes and on its own ground.
 *
 * <p>Both lists are datapack tags, so another pack's rock-named biomes can join (Expanded Ecosphere's would go in the
 * same role tags or new ones).
 */
public final class GeologyBiomes {

    private GeologyBiomes() {}

    /** Every biome of every geology role. */
    public static final TagKey<Biome> GATED = TagKey.create(Registries.BIOME,
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "geology"));

    /** The tag of a role's biomes. */
    public static TagKey<Biome> tag(Role role) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "geology/" + role.id));
    }

    /**
     * The role bits each gated biome of {@code possible} may stand on (a biome in several role tags may stand on the
     * ground of any of them); empty when the pack has none.
     */
    public static Map<Holder<Biome>, Integer> gated(Optional<? extends Registry<Biome>> registry, Set<Holder<Biome>> possible) {
        Map<Holder<Biome>, Integer> out = new HashMap<>();
        if (registry.isEmpty())
            return out;
        for (Role role : Role.values()) {
            Optional<HolderSet.Named<Biome>> set = registry.get().getTag(tag(role));
            if (set.isEmpty())
                continue;
            for (Holder<Biome> b : set.get())
                if (possible.contains(b))
                    out.merge(b, role.bit(), (a, c) -> a | c);
        }
        return out;
    }

    /** The role ids of a bit set, for logs and censuses. */
    public static String names(int bits) {
        StringBuilder s = new StringBuilder();
        for (Role r : Role.values())
            if ((bits & r.bit()) != 0)
                s.append(s.isEmpty() ? "" : "+").append(r.id);
        return s.isEmpty() ? "-" : s.toString();
    }
}
