// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

/**
 * The last planet info and pole storm the server sent this client (plan §7.11, §7.15). Plain data,
 * no client classes: written by {@link ClientHandlers} and read by the client-only renderer and
 * debug line on the client's main thread; cleared when the client leaves the server.
 */
public final class ClientPlanetState {

    private static volatile WorldInfoPayload worldInfo;
    private static volatile PoleStormPayload storm = PoleStormPayload.CALM;

    private ClientPlanetState() {}

    static void setWorldInfo(WorldInfoPayload info) {
        worldInfo = info;
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

    public static void reset() {
        worldInfo = null;
        storm = PoleStormPayload.CALM;
    }
}
