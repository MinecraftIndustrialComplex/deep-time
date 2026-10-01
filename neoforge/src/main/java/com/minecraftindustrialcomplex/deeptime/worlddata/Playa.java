// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * The playas of a world (FORMAT.md "Playas"): the baked layer of their salt and the rock class it
 * is. {@code worldgen/PlayaPass} lays it. Plain Java, no Minecraft.
 */
public final class Playa {

    /** The baked layer: blocks of salt per cell, 0 for none. */
    public static final String LAYER = "playa";
    /** The rock class of the salt when the bake's note names none. */
    public static final String DEFAULT_CLASS = "halite";

    private Playa() {}

    /** {@code notes.bake.water.playa.rock_class}, else {@link #DEFAULT_CLASS}. */
    public static String rockClass(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "water", "playa", "rock_class"}) {
            if (e == null || !e.isJsonObject())
                return DEFAULT_CLASS;
            e = e.getAsJsonObject().get(key);
        }
        return e != null && e.isJsonPrimitive() && e.getAsJsonPrimitive().isString() ? e.getAsString() : DEFAULT_CLASS;
    }
}
