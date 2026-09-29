// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import net.neoforged.neoforge.network.handling.IPayloadContext;

/**
 * The client's handlers for Deep Time's payloads (plan §7.15). They only store what arrived in
 * {@link ClientPlanetState}; the client-only code ({@code client/DeepTimeClient}) reads it. This
 * class names no client class, so registering its handlers on a dedicated server (NeoForge
 * registers both sides' handlers everywhere) loads nothing that is missing there. NeoForge runs
 * these handlers on the client's main thread (the registrar's default).
 */
public final class ClientHandlers {

    private ClientHandlers() {}

    public static void onWorldInfo(WorldInfoPayload payload, IPayloadContext context) {
        ClientPlanetState.setWorldInfo(payload);
    }

    public static void onPoleStorm(PoleStormPayload payload, IPayloadContext context) {
        ClientPlanetState.setStorm(payload);
    }
}
