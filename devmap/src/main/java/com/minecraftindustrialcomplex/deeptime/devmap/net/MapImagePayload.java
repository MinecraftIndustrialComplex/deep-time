// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.ImageCodec;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

/**
 * One part of the base image (server to client): {@link ImageCodec}'s deflated bytes, part
 * {@code index} of {@code count} of image {@code imageId} ({@link MapInfoPayload#imageId}).
 */
public record MapImagePayload(int imageId, int index, int count, byte[] data) implements CustomPacketPayload {

    public static final Type<MapImagePayload> TYPE = new Type<>(ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "map_image"));

    public static final StreamCodec<FriendlyByteBuf, MapImagePayload> STREAM_CODEC =
            StreamCodec.ofMember(MapImagePayload::write, MapImagePayload::read);

    private void write(FriendlyByteBuf buf) {
        buf.writeVarInt(imageId);
        buf.writeVarInt(index);
        buf.writeVarInt(count);
        buf.writeByteArray(data);
    }

    private static MapImagePayload read(FriendlyByteBuf buf) {
        return new MapImagePayload(buf.readVarInt(), buf.readVarInt(), buf.readVarInt(), buf.readByteArray(ImageCodec.PART_BYTES));
    }

    @Override
    public Type<MapImagePayload> type() {
        return TYPE;
    }
}
