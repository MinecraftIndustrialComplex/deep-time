// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * The world's settings for geothermal features ({@code notes.bake.underground.params}, preset
 * section {@code underground}; plain Java). {@link Geothermal} builds the sites from them.
 *
 * @param density             multiplies the number of fields each active volcano gets; 0: no
 *                            geothermal features ({@code geothermal_density})
 * @param reach               how far from the summit fields may lie, in basal radii
 *                            ({@code geothermal_reach})
 * @param size                multiplies every feature's size ({@code geothermal_size})
 * @param springMinPrecipMm   a field is hot springs only where the year's rain reaches this much
 *                            ({@code geothermal_spring_min_precip_mm}); drier ground gets fumaroles
 */
public record GeothermalRules(double density, double reach, double size, double springMinPrecipMm) {

    public static final GeothermalRules DEFAULT = new GeothermalRules(1.0, 1.0, 1.0, 400.0);

    public GeothermalRules {
        density = Math.clamp(density, 0.0, 4.0);
        reach = Math.clamp(reach, 0.25, 3.0);
        size = Math.clamp(size, 0.5, 3.0);
        springMinPrecipMm = Math.clamp(springMinPrecipMm, 0.0, 4000.0);
    }

    /** The world's rules; {@link #DEFAULT} for every key that is absent or of the wrong type. */
    public static GeothermalRules of(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "underground", "params"}) {
            if (e == null || !e.isJsonObject())
                return DEFAULT;
            e = e.getAsJsonObject().get(key);
        }
        if (e == null || !e.isJsonObject())
            return DEFAULT;
        JsonObject p = e.getAsJsonObject();
        return new GeothermalRules(number(p, "geothermal_density", DEFAULT.density),
                number(p, "geothermal_reach", DEFAULT.reach),
                number(p, "geothermal_size", DEFAULT.size),
                number(p, "geothermal_spring_min_precip_mm", DEFAULT.springMinPrecipMm));
    }

    private static double number(JsonObject p, String key, double fallback) {
        JsonElement v = p.get(key);
        if (v == null || !v.isJsonPrimitive() || !v.getAsJsonPrimitive().isNumber())
            return fallback;
        double d = v.getAsDouble();
        return Double.isFinite(d) ? d : fallback;
    }
}
