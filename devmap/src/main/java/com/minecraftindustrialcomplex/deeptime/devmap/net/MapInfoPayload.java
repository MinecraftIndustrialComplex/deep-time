// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapLayer;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.ResourceLocation;

import java.util.ArrayList;
import java.util.List;

/**
 * What the server's dev map tells a client about the level it is in (server to client, on login,
 * dimension change and respawn, and again as each picture is ready). The projection itself is
 * Deep Time's own ({@code ClientPlanetState.projection}); this adds what Deep Time does not send:
 * the frame of the map's pictures, the pictures themselves ({@link MapLayer}: satellite and
 * elevation, each following in {@link MapImagePayload}s), the edge border's inset and whether the
 * player may teleport.
 *
 * @param dimension    the level's id
 * @param planet       the level is a Deep Time planet
 * @param note         a line for the readout (why there are no pictures, or the frame they share)
 * @param kind         the projection the server drew with ({@code cube}, or {@code lambert} on a test level)
 * @param period       its x period, blocks (the client checks it against Deep Time's)
 * @param imageW       the frame's width, pixels (0: no frame)
 * @param imageH       its height
 * @param x0           block x of its west side
 * @param z0           block z of its north side
 * @param blocksPerPx  blocks a pixel
 * @param pictures     the pictures on that frame, at most one per layer
 * @param borderInset  the temporary edge border's inset, blocks, NaN for none
 * @param seaLevel     the world's sea level, block y
 * @param canTeleport  the player may teleport from the map (operator level 2)
 */
public record MapInfoPayload(String dimension, boolean planet, String note, String kind, int period, int imageW,
                             int imageH, double x0, double z0, double blocksPerPx, List<Picture> pictures,
                             double borderInset, int seaLevel, boolean canTeleport) implements CustomPacketPayload {

    /**
     * One picture of the frame.
     *
     * @param id     the image that follows in {@link MapImagePayload}s, 0 for none (yet)
     * @param parts  how many {@link MapImagePayload}s carry it
     * @param note   what it is, or why it is not here (still rendering, or what failed)
     */
    public record Picture(MapLayer layer, int id, int parts, String note) {
        public Picture {
            note = cut(note);
        }

        /** Whether the server is still drawing it (a later info names it). */
        public boolean rendering() {
            return id == 0 && note.startsWith(RENDERING);
        }
    }

    public static final Type<MapInfoPayload> TYPE = new Type<>(ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "map_info"));

    public static final int MAX_STRING = 512;
    /** The note of a picture the server is still drawing. */
    public static final String RENDERING = "rendering on the server";

    public static final StreamCodec<FriendlyByteBuf, MapInfoPayload> STREAM_CODEC =
            StreamCodec.ofMember(MapInfoPayload::write, MapInfoPayload::read);

    public MapInfoPayload {
        dimension = cut(dimension);
        note = cut(note);
        kind = cut(kind);
        pictures = pictures == null ? List.of() : List.copyOf(pictures);
    }

    /** The info for a level with no planet. */
    public static MapInfoPayload none(String dimension, String note) {
        return new MapInfoPayload(dimension, false, note, "", 0, 0, 0, 0, 0, 1, List.of(), Double.NaN, 63, false);
    }

    /** The pictures' frame, or null when there is none. */
    public MapFrame frame() {
        return imageW <= 0 || imageH <= 0 || !(blocksPerPx > 0) ? null : new MapFrame(x0, z0, blocksPerPx, imageW, imageH);
    }

    /** The picture of {@code layer}, or null when the server sent none. */
    public Picture picture(MapLayer layer) {
        for (Picture p : pictures)
            if (p.layer() == layer)
                return p;
        return null;
    }

    /** The image id of {@code layer}'s picture, 0 for none. */
    public int imageId(MapLayer layer) {
        Picture p = picture(layer);
        return p == null ? 0 : p.id();
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
        buf.writeVarInt(imageW);
        buf.writeVarInt(imageH);
        buf.writeDouble(x0);
        buf.writeDouble(z0);
        buf.writeDouble(blocksPerPx);
        buf.writeVarInt(pictures.size());
        for (Picture p : pictures) {
            buf.writeUtf(p.layer().id(), MAX_STRING);
            buf.writeVarInt(p.id());
            buf.writeVarInt(p.parts());
            buf.writeUtf(p.note(), MAX_STRING);
        }
        buf.writeDouble(borderInset);
        buf.writeVarInt(seaLevel);
        buf.writeBoolean(canTeleport);
    }

    private static MapInfoPayload read(FriendlyByteBuf buf) {
        String dimension = buf.readUtf(MAX_STRING);
        boolean planet = buf.readBoolean();
        String note = buf.readUtf(MAX_STRING);
        String kind = buf.readUtf(MAX_STRING);
        int period = buf.readVarInt(), w = buf.readVarInt(), h = buf.readVarInt();
        double x0 = buf.readDouble(), z0 = buf.readDouble(), s = buf.readDouble();
        int n = buf.readVarInt();
        if (n < 0 || n > MapLayer.values().length)
            throw new IllegalArgumentException("dev map info with " + n + " pictures");
        List<Picture> pictures = new ArrayList<>(n);
        for (int i = 0; i < n; i++)
            pictures.add(new Picture(MapLayer.parse(buf.readUtf(MAX_STRING)), buf.readVarInt(), buf.readVarInt(),
                    buf.readUtf(MAX_STRING)));
        return new MapInfoPayload(dimension, planet, note, kind, period, w, h, x0, z0, s, pictures, buf.readDouble(),
                buf.readVarInt(), buf.readBoolean());
    }

    @Override
    public Type<MapInfoPayload> type() {
        return TYPE;
    }
}
