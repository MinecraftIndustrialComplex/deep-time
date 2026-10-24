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

    /**
     * A planet level, its x period, projection and nominal metres per block, from a {@link
     * WorldInfoPayload} with {@code planet} set.
     */
    private record Planet(ResourceKey<Level> dimension, int circumference,
                          com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection projection,
                          double mPerBlockH) {}

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

    /**
     * The world projection of the planet {@code level} is (Lambert, Petroff–Guyou or the cube), or null
     * when the info is for another level, is not a planet, or has not arrived. Safe from any thread.
     */
    public static com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection projection(Level level) {
        Planet p = planet;
        return p != null && level != null && p.dimension() == level.dimension() ? p.projection() : null;
    }

    /**
     * The nominal horizontal metres per block of the planet {@code level} is (the manifest's {@code
     * m_per_block_h}), or NaN as {@link #projection} is null. Safe from any thread.
     */
    public static double mPerBlockH(Level level) {
        Planet p = planet;
        return p != null && level != null && p.dimension() == level.dimension() ? p.mPerBlockH() : Double.NaN;
    }

    /**
     * The circumference of the sphere the planet's blocks stand for (2πR, R the equal-area radius): C on
     * a Lambert world, √(2π)·b on a Petroff–Guyou one; 0 as {@link #circumference}. For Distant Horizons'
     * curvature.
     */
    public static int sphereCircumference(Level level) {
        Planet p = planet;
        if (p == null || level == null || p.dimension() != level.dimension())
            return 0;
        return (int) Math.round(2 * Math.PI * p.projection().radius());
    }

    private static Planet planetOf(WorldInfoPayload info) {
        if (info == null || !info.planet() || info.circumference() <= 0)
            return null;
        ResourceLocation id = ResourceLocation.tryParse(info.dimension());
        // ResourceKey.create interns, so the level's own key is this same instance.
        return id == null ? null : new Planet(ResourceKey.create(Registries.DIMENSION, id), info.circumference(), info.worldProjection(),
                info.mPerBlockH());
    }

    public static void reset() {
        planet = null;
        worldInfo = null;
        storm = PoleStormPayload.CALM;
    }
}
