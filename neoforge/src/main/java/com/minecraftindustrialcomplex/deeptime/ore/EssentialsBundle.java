// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.ore;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;

import java.util.List;
import java.util.function.Predicate;

/**
 * The essentials the content bundle passes to the core ({@link OreData#contentJson}): only those
 * whose material has an ore block in the running game. An essential of a material with none (zinc
 * without Create, lead without a mod that adds a lead ore) is left out, so the core does not force
 * a deposit for a material it could never place (ledger ORE-273). Plain Java, no Minecraft class,
 * so a unit test can call it.
 */
public final class EssentialsBundle {

    private EssentialsBundle() {
    }

    /**
     * The essentials of {@code essentials} (objects with a {@code material}) whose material passes
     * {@code hasOreBlock}, in order; the materials left out are added to {@code skipped}.
     */
    public static JsonArray placeable(JsonArray essentials, Predicate<String> hasOreBlock, List<String> skipped) {
        JsonArray out = new JsonArray();
        for (JsonElement e : essentials) {
            String material = e.getAsJsonObject().get("material").getAsString();
            if (hasOreBlock.test(material))
                out.add(e);
            else
                skipped.add(material);
        }
        return out;
    }
}
