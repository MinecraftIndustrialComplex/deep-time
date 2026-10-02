// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * The world's settings for lava at its volcanoes ({@code notes.bake.underground.params}, preset
 * section {@code underground}; plain Java). {@link Magma} builds the bodies from them.
 *
 * @param chamberDepthKm     depth of a magma chamber's top below the crater floor, km
 *                           ({@code magma_chamber_depth_km})
 * @param chamberSize        multiplies every chamber; 0: no magma at all ({@code magma_chamber_size})
 * @param craterLakeShare    share of the active volcanoes on land that hold a crater lava lake, the
 *                           most active first ({@code crater_lake_share})
 * @param removeLavaFeatures whether the biomes' random lava lakes and springs are removed
 *                           ({@code remove_lava_features})
 */
public record MagmaRules(double chamberDepthKm, double chamberSize, double craterLakeShare, boolean removeLavaFeatures) {

    public static final MagmaRules DEFAULT = new MagmaRules(3.0, 1.0, 0.25, true);

    public MagmaRules {
        chamberDepthKm = Math.clamp(chamberDepthKm, 0.5, 15.0);
        chamberSize = Math.clamp(chamberSize, 0.0, 4.0);
        craterLakeShare = Math.clamp(craterLakeShare, 0.0, 1.0);
    }

    /** The world's rules; {@link #DEFAULT} for every key that is absent or of the wrong type. */
    public static MagmaRules of(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "underground", "params"}) {
            if (e == null || !e.isJsonObject())
                return DEFAULT;
            e = e.getAsJsonObject().get(key);
        }
        if (e == null || !e.isJsonObject())
            return DEFAULT;
        JsonObject p = e.getAsJsonObject();
        JsonElement remove = p.get("remove_lava_features");
        return new MagmaRules(number(p, "magma_chamber_depth_km", DEFAULT.chamberDepthKm),
                number(p, "magma_chamber_size", DEFAULT.chamberSize),
                number(p, "crater_lake_share", DEFAULT.craterLakeShare),
                remove != null && remove.isJsonPrimitive() && remove.getAsJsonPrimitive().isBoolean()
                        ? remove.getAsBoolean() : DEFAULT.removeLavaFeatures);
    }

    private static double number(JsonObject p, String key, double fallback) {
        JsonElement v = p.get(key);
        if (v == null || !v.isJsonPrimitive() || !v.getAsJsonPrimitive().isNumber())
            return fallback;
        double d = v.getAsDouble();
        return Double.isFinite(d) ? d : fallback;
    }
}
