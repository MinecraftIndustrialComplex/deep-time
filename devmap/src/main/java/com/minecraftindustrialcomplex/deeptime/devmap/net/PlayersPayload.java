// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

import java.util.ArrayList;
import java.util.List;

/** Every player in the receiver's level, about once a second (server to client), for the map's markers. */
public record PlayersPayload(String dimension, List<Entry> players) implements CustomPacketPayload {

    /** One player: name, position and facing (yaw, degrees, Minecraft's: 0 faces +z). */
    public record Entry(String name, double x, double y, double z, float yaw) {}

    public static final Type<PlayersPayload> TYPE = new Type<>(ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "players"));

    /** Most players one payload lists. */
    public static final int MAX_PLAYERS = 256;

    public static final StreamCodec<FriendlyByteBuf, PlayersPayload> STREAM_CODEC =
            StreamCodec.ofMember(PlayersPayload::write, PlayersPayload::read);

    private void write(FriendlyByteBuf buf) {
        buf.writeUtf(dimension, MapInfoPayload.MAX_STRING);
        int n = Math.min(MAX_PLAYERS, players.size());
        buf.writeVarInt(n);
        for (int i = 0; i < n; i++) {
            Entry e = players.get(i);
            buf.writeUtf(e.name(), 64);
            buf.writeDouble(e.x());
            buf.writeDouble(e.y());
            buf.writeDouble(e.z());
            buf.writeFloat(e.yaw());
        }
    }

    private static PlayersPayload read(FriendlyByteBuf buf) {
        String dim = buf.readUtf(MapInfoPayload.MAX_STRING);
        int n = Math.min(MAX_PLAYERS, buf.readVarInt());
        List<Entry> list = new ArrayList<>(n);
        for (int i = 0; i < n; i++)
            list.add(new Entry(buf.readUtf(64), buf.readDouble(), buf.readDouble(), buf.readDouble(), buf.readFloat()));
        return new PlayersPayload(dim, List.copyOf(list));
    }

    @Override
    public Type<PlayersPayload> type() {
        return TYPE;
    }
}
