// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.net;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.network.ConnectionProtocol;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerPlayer;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.common.util.FakePlayer;
import net.neoforged.neoforge.event.entity.player.PlayerEvent;
import net.neoforged.neoforge.network.PacketDistributor;
import net.neoforged.neoforge.network.event.RegisterPayloadHandlersEvent;
import net.neoforged.neoforge.network.registration.NetworkRegistry;
import net.neoforged.neoforge.network.registration.PayloadRegistration;

import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Deep Time's network channels (plan §7.15). "Required on both client and server" (decisions file,
 * "Sides") is enforced here: the payloads are registered without {@code .optional()}, so NeoForge's
 * negotiation ({@code NetworkComponentNegotiator}) refuses a NeoForge client that lacks them, and
 * {@code NetworkRegistry.initializeOtherConnection} refuses a vanilla or other client outright.
 * {@code NetworkGameTests.worldInfoChannelRequired} checks the registration and the negotiation;
 * {@code gametest-planet.sh --scenario refuse_client} checks it over a real connection.
 */
public final class DeepTimeNetwork {

    /** The channel version: bump it whenever a codec or a handler's meaning changes. */
    public static final String VERSION = "2"; // 2: the pole storm and the pole cap's fields went (cube plan §7.4)

    /** Deep Time's payload types, in registration order. */
    public static final List<CustomPacketPayload.Type<?>> TYPES = List.of(WorldInfoPayload.TYPE);

    private DeepTimeNetwork() {}

    public static void init(IEventBus modBus) {
        modBus.addListener(RegisterPayloadHandlersEvent.class, DeepTimeNetwork::register);
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerLoggedInEvent.class, e -> sendWorldInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerChangedDimensionEvent.class, e -> sendWorldInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerRespawnEvent.class, e -> sendWorldInfo(e.getEntity()));
    }

    private static void register(RegisterPayloadHandlersEvent event) {
        // Required (no .optional()): a client without Deep Time cannot join.
        event.registrar(VERSION)
                .playToClient(WorldInfoPayload.TYPE, WorldInfoPayload.STREAM_CODEC, ClientHandlers::onWorldInfo);
    }

    private static void sendWorldInfo(net.minecraft.world.entity.player.Player player) {
        if (player instanceof ServerPlayer sp)
            send(sp, WorldInfoPayload.forLevel(sp.serverLevel()));
    }

    /**
     * Sends {@code payload} to a real player. Fake players (machines) have no client; a connection
     * without the channel cannot exist for a negotiated client, but a mock or foreign one is skipped
     * instead of throwing.
     */
    public static void send(ServerPlayer player, CustomPacketPayload payload) {
        if (player instanceof FakePlayer || player.connection == null || !player.connection.hasChannel(payload.type()))
            return;
        PacketDistributor.sendToPlayer(player, payload);
    }

    /** One payload registration as NeoForge holds it. */
    public record Channel(ConnectionProtocol protocol, ResourceLocation id, String version, Optional<PacketFlow> flow,
                          boolean optional) {}

    /**
     * Every payload registration NeoForge holds, sorted by protocol and id. NeoForge exposes no
     * public view of its registrations, so this reads {@code NetworkRegistry.PAYLOAD_REGISTRATIONS}
     * (a private static field, NeoForge 21.1) by reflection; the probe and the tests use it.
     *
     * @throws IllegalStateException when the field is gone or unreadable (a NeoForge change)
     */
    @SuppressWarnings("unchecked")
    public static List<Channel> registrations() {
        Map<ConnectionProtocol, Map<ResourceLocation, PayloadRegistration<?>>> all;
        try {
            Field f = NetworkRegistry.class.getDeclaredField("PAYLOAD_REGISTRATIONS");
            f.setAccessible(true);
            all = (Map<ConnectionProtocol, Map<ResourceLocation, PayloadRegistration<?>>>) f.get(null);
        } catch (ReflectiveOperationException | RuntimeException e) {
            throw new IllegalStateException("cannot read NeoForge's payload registrations (NetworkRegistry.PAYLOAD_REGISTRATIONS): " + e, e);
        }
        List<Channel> out = new ArrayList<>();
        all.forEach((protocol, regs) -> regs.values().forEach(r ->
                out.add(new Channel(protocol, r.id(), r.version(), r.flow(), r.optional()))));
        out.sort(Comparator.comparing((Channel c) -> c.protocol().ordinal()).thenComparing(c -> c.id().toString()));
        return out;
    }

    /** True for Deep Time's own channels. */
    public static boolean ours(ResourceLocation id) {
        return DeepTime.MODID.equals(id.getNamespace());
    }
}
