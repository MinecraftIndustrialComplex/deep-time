// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * The world's setting for the volcanic biomes ({@code notes.bake.underground.params}, preset section
 * {@code underground}; plain Java). {@link VolcanicZones} draws the volcanic biomes' places from it.
 *
 * @param reach how far a volcano's own biomes reach down its flanks, as a multiple of the built-in
 *              shares of its basal radius (the cone zone 0.45, the ash and lava fields 1.0); 0: no
 *              volcanic biome is placed anywhere (they stay kept out of the rest of the world)
 *              ({@code volcanic_biome_reach})
 */
public record VolcanicBiomeRules(double reach) {

    // ledger: BIO-104 (game-choice; the default and the range repeat the Rust side's)
    public static final VolcanicBiomeRules DEFAULT = new VolcanicBiomeRules(1.0);

    public VolcanicBiomeRules {
        reach = Double.isFinite(reach) ? Math.clamp(reach, 0.0, 3.0) : 1.0;
    }

    /** No volcanic biome is placed. */
    public boolean off() {
        return reach <= 0.0;
    }

    /** The world's rules; {@link #DEFAULT} when the key is absent or not a number. */
    public static VolcanicBiomeRules of(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "underground", "params"}) {
            if (e == null || !e.isJsonObject())
                return DEFAULT;
            e = e.getAsJsonObject().get(key);
        }
        if (e == null || !e.isJsonObject())
            return DEFAULT;
        JsonElement v = e.getAsJsonObject().get("volcanic_biome_reach");
        if (v == null || !v.isJsonPrimitive() || !v.getAsJsonPrimitive().isNumber())
            return DEFAULT;
        return new VolcanicBiomeRules(v.getAsDouble());
    }
}
