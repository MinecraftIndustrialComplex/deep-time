// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.mojang.authlib.GameProfile;
import io.netty.channel.embedded.EmbeddedChannel;
import net.minecraft.network.Connection;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.network.protocol.game.ClientboundSystemChatPacket;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.server.network.CommonListenerCookie;
import net.minecraft.world.level.GameType;
import net.neoforged.neoforge.network.connection.ConnectionType;
import net.neoforged.neoforge.network.registration.NetworkRegistry;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * Players for server-side tests (P1-18). A {@link net.neoforged.neoforge.common.util.FakePlayer}
 * is not in the level's player list and drops every packet, so it cannot show what the pole cap
 * does to a player. These are real {@link ServerPlayer}s placed through
 * {@code PlayerList.placeNewPlayer} (as vanilla's {@code GameTestHelper.makeMockServerPlayerInLevel}
 * does) on an in-memory connection that NeoForge treats as a fully compatible NeoForge client
 * ({@code NetworkRegistry.configureMockConnection}): they are in {@code level.players()}, and every
 * packet the server sends them lands, as a packet object, in the connection's
 * {@link EmbeddedChannel}, where the test reads it. No client moves them: a mock player stays
 * where the test or the server puts it (no physics runs for it), which is exactly what a test of
 * the server's own decisions needs. Dev GameTest runtimes only.
 */
final class MockPlayers {

    /** One mock player and what the server sent it. */
    static final class Mock {
        final ServerPlayer player;
        final EmbeddedChannel channel;
        private final List<Object> sent = new ArrayList<>();

        Mock(ServerPlayer player, EmbeddedChannel channel) {
            this.player = player;
            this.channel = channel;
        }

        /** Everything sent so far (drains the channel into the list). */
        List<Object> sent() {
            channel.flushOutbound();
            Object m;
            while ((m = channel.readOutbound()) != null)
                sent.add(m);
            return sent;
        }

        /** Every custom payload of {@code type} sent so far, oldest first. */
        <T extends CustomPacketPayload> List<T> payloads(Class<T> type) {
            List<T> out = new ArrayList<>();
            for (Object o : sent())
                if (o instanceof ClientboundCustomPayloadPacket p && type.isInstance(p.payload()))
                    out.add(type.cast(p.payload()));
            return out;
        }

        <T extends CustomPacketPayload> T last(Class<T> type) {
            List<T> all = payloads(type);
            return all.isEmpty() ? null : all.get(all.size() - 1);
        }

        /** The action-bar lines sent so far. */
        List<Component> actionBars() {
            List<Component> out = new ArrayList<>();
            // ServerPlayer.displayClientMessage(c, true) is a system chat packet with overlay = true.
            for (Object o : sent())
                if (o instanceof ClientboundSystemChatPacket p && p.overlay())
                    out.add(p.content());
            return out;
        }

        /** True when an action-bar line with translation key {@code key} was sent. */
        boolean sawActionBar(String key) {
            for (Component c : actionBars())
                if (c.getContents() instanceof TranslatableContents t && t.getKey().equals(key))
                    return true;
            return false;
        }

        void forgetSent() {
            sent();
            sent.clear();
        }

        /** Takes the player out of the game, as a disconnect would. */
        void remove() {
            if (!player.hasDisconnected())
                player.server.getPlayerList().remove(player);
            channel.close();
            sent.clear();
        }
    }

    private static int counter;

    private MockPlayers() {}

    /** A new mock player in {@code mode}, logged in (it starts where a new player would, in the overworld). */
    static Mock create(ServerLevel level, GameType mode) {
        var server = level.getServer();
        GameProfile profile = new GameProfile(UUID.randomUUID(), "dt-mock-" + (counter++));
        var cookie = new CommonListenerCookie(profile, 0, ClientInformation.createDefault(), false, ConnectionType.NEOFORGE);
        ServerPlayer player = new ServerPlayer(server, level, profile, cookie.clientInformation());
        Connection connection = new Connection(PacketFlow.SERVERBOUND);
        EmbeddedChannel channel = new EmbeddedChannel(connection);
        NetworkRegistry.configureMockConnection(connection);
        server.getPlayerList().placeNewPlayer(connection, player, cookie);
        player.setGameMode(mode);
        return new Mock(player, channel);
    }

    /** Removes every mock in {@code mocks}, ignoring failures (test cleanup). */
    static void removeAll(List<Mock> mocks) {
        for (Mock m : mocks) {
            try {
                m.remove();
            } catch (RuntimeException ignored) {
                // cleanup only
            }
        }
        mocks.clear();
    }
}
