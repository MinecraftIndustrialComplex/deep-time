// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.server.ServerMaps;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.server.level.ServerPlayer;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.neoforge.common.util.FakePlayer;
import net.neoforged.neoforge.network.PacketDistributor;
import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;

/**
 * The dev map's channels. All of them are optional (unlike Deep Time's own, which are required): a
 * client with the dev map joins a server without it, and a client without it joins a server with
 * it; neither side sends a payload the other has not registered ({@link #send} checks the channel).
 * The client handlers only store into {@link DevMapState}, which names no client class, so the
 * registration loads nothing a dedicated server lacks.
 */
public final class DevMapNetwork {

    /** The channel version: bump it whenever a codec or a handler's meaning changes. */
    public static final String VERSION = "2"; // 2: the info names a satellite and an elevation picture

    private DevMapNetwork() {}

    public static void init(IEventBus modBus) {
        modBus.addListener(RegisterPayloadHandlersEvent.class, DevMapNetwork::register);
    }

    private static void register(RegisterPayloadHandlersEvent event) {
        event.registrar(VERSION)
                .optional()
                .playToClient(MapInfoPayload.TYPE, MapInfoPayload.STREAM_CODEC, (p, ctx) -> DevMapState.onInfo(p))
                .playToClient(MapImagePayload.TYPE, MapImagePayload.STREAM_CODEC, (p, ctx) -> DevMapState.onImagePart(p))
                .playToClient(PlayersPayload.TYPE, PlayersPayload.STREAM_CODEC, (p, ctx) -> DevMapState.onPlayers(p))
                .playToServer(TeleportPayload.TYPE, TeleportPayload.STREAM_CODEC, ServerMaps::onTeleport);
    }

    /** Whether {@code player}'s client has the dev map (a real player whose connection has its channel). */
    public static boolean hasDevMap(ServerPlayer player) {
        return !(player instanceof FakePlayer) && player.connection != null && player.connection.hasChannel(MapInfoPayload.TYPE);
    }

    /** Sends {@code payload} to {@code player} when its client has the channel; otherwise nothing. */
    public static void send(ServerPlayer player, CustomPacketPayload payload) {
        if (player instanceof FakePlayer || player.connection == null || !player.connection.hasChannel(payload.type()))
            return;
        PacketDistributor.sendToPlayer(player, payload);
    }
}
