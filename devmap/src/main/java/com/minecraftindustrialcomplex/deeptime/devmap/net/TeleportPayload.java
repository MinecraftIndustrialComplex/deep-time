// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

/**
 * A request to be put at block (x, z) of the level {@code dimension} (client to server, from a
 * right-click on the map). The server checks operator permission, brings the point into the world's
 * domain, keeps it inside the edge border and chooses the height.
 */
public record TeleportPayload(String dimension, double x, double z) implements CustomPacketPayload {

    public static final Type<TeleportPayload> TYPE = new Type<>(ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "teleport"));

    public static final StreamCodec<FriendlyByteBuf, TeleportPayload> STREAM_CODEC =
            StreamCodec.ofMember(TeleportPayload::write, TeleportPayload::read);

    private void write(FriendlyByteBuf buf) {
        buf.writeUtf(dimension, MapInfoPayload.MAX_STRING);
        buf.writeDouble(x);
        buf.writeDouble(z);
    }

    private static TeleportPayload read(FriendlyByteBuf buf) {
        return new TeleportPayload(buf.readUtf(MapInfoPayload.MAX_STRING), buf.readDouble(), buf.readDouble());
    }

    @Override
    public Type<TeleportPayload> type() {
        return TYPE;
    }
}
