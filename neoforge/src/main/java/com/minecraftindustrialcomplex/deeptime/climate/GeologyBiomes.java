// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.Role;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.tags.TagKey;
import net.minecraft.world.level.biome.Biome;

import java.util.HashMap;
import java.util.Map;

/**
 * The geology gate (Ben, decisions log 2026-10-04, "Deep Time-side fixes (Recommended)"; research note
 * biome-mods-survey.md, fix 3): a pack's rock-named biomes go only on the ground they are named for. The gate is a
 * biome rule file, {@code deeptime:rock_named} (FORMAT.md "Biome rules"): per {@link Role} of {@link GeologyGround} a
 * rule whose candidates are the tag {@code #deeptime:geology/<role>} (Terralith's Shield on shield provinces, Yosemite
 * on exposed granite, White, Granite and Basalt Cliffs on coasts of carbonate, granite and basalt, the red-bed canyons
 * on dry sedimentary rock, Stony Spires on humid carbonate), {@code exclusive} (kept out of the climate search) and
 * {@code in_slot} with a {@code ground} condition (given back their own slot where the ground has the role). Off its
 * ground the pack's next-nearest biome answers; the gate takes biomes away, it never adds them. This class reads which
 * biomes a level's bound rules keep to which ground, for the census and the GameTests.
 */
public final class GeologyBiomes {

    private GeologyBiomes() {}

    /** Every biome of every geology role (the rule file's {@code requires}). */
    public static final TagKey<Biome> GATED = TagKey.create(Registries.BIOME,
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "geology"));

    /** The tag of a role's biomes. */
    public static TagKey<Biome> tag(Role role) {
        return TagKey.create(Registries.BIOME, ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, "geology/" + role.id));
    }

    /**
     * The ground role bits each biome kept to its ground by {@code rules} may stand on: the candidates of every
     * exclusive rule with a {@code ground} condition (a biome of several such rules may stand on the ground of any of
     * them). Empty when no such rule is active.
     */
    public static Map<Holder<Biome>, Integer> gated(BiomeRules.Set_ rules) {
        Map<Holder<Biome>, Integer> out = new HashMap<>();
        for (BiomeRules.Bound b : rules.ordinary())
            if (b.rule().ground() != 0 && b.rule().exclusive())
                for (Holder<Biome> h : b.candidates())
                    out.merge(h, b.rule().ground(), (a, c) -> a | c);
        return out;
    }

    /**
     * Whether a biome query's decision ({@link VolcanicBiomes#lastDecision()}) was a rule of {@code rules} with a
     * {@code ground} condition: a rock-named biome given its slot on its own ground.
     */
    public static boolean byGroundRule(String decision, BiomeRules.Set_ rules) {
        if (decision == null || !decision.startsWith("rule:"))
            return false;
        for (BiomeRules.Bound b : rules.ordinary())
            if (b.rule().ground() != 0 && decision.equals("rule:" + b.file() + "/" + b.rule().id()))
                return true;
        return false;
    }

    /** The role ids of a bit set, for logs and censuses. */
    public static String names(int bits) {
        return Role.names(bits);
    }
}
