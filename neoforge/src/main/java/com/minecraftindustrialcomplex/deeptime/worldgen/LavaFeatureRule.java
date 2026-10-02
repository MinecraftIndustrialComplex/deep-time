// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * What makes a configured feature a lava feature, on its JSON (plain Java, so JUnit can test it;
 * {@link LavaFeatures} applies it to the game's features).
 */
public final class LavaFeatureRule {

    private LavaFeatureRule() {}

    /**
     * Whether the JSON holds a block or fluid state of lava: an object whose {@code Name} is
     * {@code minecraft:lava} (or the flowing fluid), whether the feature places it or replaces it. A
     * block predicate or a tag that merely mentions lava ({@code "blocks": "minecraft:lava"}) is
     * not a state.
     */
    public static boolean namesLavaState(JsonElement e) {
        if (e.isJsonObject()) {
            JsonObject o = e.getAsJsonObject();
            JsonElement name = o.get("Name");
            if (name != null && name.isJsonPrimitive()) {
                String n = name.getAsString();
                if (n.equals("minecraft:lava") || n.equals("minecraft:flowing_lava"))
                    return true;
            }
            for (var entry : o.entrySet())
                if (namesLavaState(entry.getValue()))
                    return true;
        } else if (e.isJsonArray()) {
            for (JsonElement c : e.getAsJsonArray())
                if (namesLavaState(c))
                    return true;
        }
        return false;
    }
}
