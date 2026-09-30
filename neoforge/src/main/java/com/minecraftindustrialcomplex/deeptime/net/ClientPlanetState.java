// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.Level;

/**
 * The last planet info and pole storm the server sent this client (plan §7.11, §7.15). Plain data,
 * no client classes: written by {@link ClientHandlers} and read by the client-only renderer and
 * debug line on the client's main thread; cleared when the client leaves the server.
 */
public final class ClientPlanetState {

    private static volatile WorldInfoPayload worldInfo;
    private static volatile PoleStormPayload storm = PoleStormPayload.CALM;
    /** The planet the info describes, resolved once so per-block readers do not parse or allocate. */
    private static volatile Planet planet;

    /** A planet level and its circumference, from a {@link WorldInfoPayload} with {@code planet} set. */
    private record Planet(ResourceKey<Level> dimension, int circumference) {}

    private ClientPlanetState() {}

    static void setWorldInfo(WorldInfoPayload info) {
        worldInfo = info;
        planet = planetOf(info);
        // A new level starts calm; the server sends the storm again if the player is in one.
        storm = PoleStormPayload.CALM;
    }

    static void setStorm(PoleStormPayload payload) {
        storm = payload;
    }

    /** The planet info for the current level, or null before the server sent one. */
    public static WorldInfoPayload worldInfo() {
        return worldInfo;
    }

    public static PoleStormPayload storm() {
        return storm;
    }

    /**
     * The circumference of the planet {@code level} is, as the server last said, or 0 when the info
     * is for another level, is not a planet, or has not arrived. Allocation-free and safe from any
     * thread (the public face is {@code api.climate.DeepTimeClimate.circumferenceBlocks}).
     */
    public static int circumference(Level level) {
        Planet p = planet;
        return p != null && level != null && p.dimension() == level.dimension() ? p.circumference() : 0;
    }

    private static Planet planetOf(WorldInfoPayload info) {
        if (info == null || !info.planet() || info.circumference() <= 0)
            return null;
        ResourceLocation id = ResourceLocation.tryParse(info.dimension());
        // ResourceKey.create interns, so the level's own key is this same instance.
        return id == null ? null : new Planet(ResourceKey.create(Registries.DIMENSION, id), info.circumference());
    }

    public static void reset() {
        planet = null;
        worldInfo = null;
        storm = PoleStormPayload.CALM;
    }
}
