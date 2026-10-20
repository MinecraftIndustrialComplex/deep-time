// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

/**
 * What the server's dev map tells a client about the level it is in (server to client, on login,
 * dimension change and respawn, and again when the base image is ready). The projection itself is
 * Deep Time's own ({@code ClientPlanetState.projection}); this adds what Deep Time does not send:
 * the base image's frame, the edge border's inset and whether the player may teleport.
 *
 * @param dimension    the level's id
 * @param planet       the level is a Deep Time planet
 * @param note         a line for the readout (why there is no image, or what the image is)
 * @param kind         the projection the server drew with ({@code lambert} or {@code petroff_guyou})
 * @param period       its x period, blocks (the client checks it against Deep Time's)
 * @param imageId      the base image that follows in {@link MapImagePayload}s, 0 for none
 * @param imageW       its width, pixels
 * @param imageH       its height
 * @param x0           block x of its west side
 * @param z0           block z of its north side
 * @param blocksPerPx  blocks a pixel
 * @param parts        how many {@link MapImagePayload}s carry it
 * @param borderInset  the temporary edge border's inset, blocks, NaN for none
 * @param seaLevel     the world's sea level, block y
 * @param canTeleport  the player may teleport from the map (operator level 2)
 */
public record MapInfoPayload(String dimension, boolean planet, String note, String kind, int period, int imageId,
                             int imageW, int imageH, double x0, double z0, double blocksPerPx, int parts,
                             double borderInset, int seaLevel, boolean canTeleport) implements CustomPacketPayload {

    public static final Type<MapInfoPayload> TYPE = new Type<>(ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "map_info"));

    public static final int MAX_STRING = 512;

    public static final StreamCodec<FriendlyByteBuf, MapInfoPayload> STREAM_CODEC =
            StreamCodec.ofMember(MapInfoPayload::write, MapInfoPayload::read);

    public MapInfoPayload {
        dimension = cut(dimension);
        note = cut(note);
        kind = cut(kind);
    }

    /** The info for a level with no planet. */
    public static MapInfoPayload none(String dimension, String note) {
        return new MapInfoPayload(dimension, false, note, "", 0, 0, 0, 0, 0, 0, 1, 0, Double.NaN, 63, false);
    }

    /** The base image's frame, or null when there is no image. */
    public MapFrame frame() {
        return imageId == 0 || imageW <= 0 || imageH <= 0 || !(blocksPerPx > 0) ? null : new MapFrame(x0, z0, blocksPerPx, imageW, imageH);
    }

    private static String cut(String s) {
        if (s == null)
            return "";
        return s.length() <= MAX_STRING ? s : s.substring(0, MAX_STRING);
    }

    private void write(FriendlyByteBuf buf) {
        buf.writeUtf(dimension, MAX_STRING);
        buf.writeBoolean(planet);
        buf.writeUtf(note, MAX_STRING);
        buf.writeUtf(kind, MAX_STRING);
        buf.writeVarInt(period);
        buf.writeVarInt(imageId);
        buf.writeVarInt(imageW);
        buf.writeVarInt(imageH);
        buf.writeDouble(x0);
        buf.writeDouble(z0);
        buf.writeDouble(blocksPerPx);
        buf.writeVarInt(parts);
        buf.writeDouble(borderInset);
        buf.writeVarInt(seaLevel);
        buf.writeBoolean(canTeleport);
    }

    private static MapInfoPayload read(FriendlyByteBuf buf) {
        return new MapInfoPayload(buf.readUtf(MAX_STRING), buf.readBoolean(), buf.readUtf(MAX_STRING), buf.readUtf(MAX_STRING),
                buf.readVarInt(), buf.readVarInt(), buf.readVarInt(), buf.readVarInt(), buf.readDouble(), buf.readDouble(),
                buf.readDouble(), buf.readVarInt(), buf.readDouble(), buf.readVarInt(), buf.readBoolean());
    }

    @Override
    public Type<MapInfoPayload> type() {
        return TYPE;
    }
}
