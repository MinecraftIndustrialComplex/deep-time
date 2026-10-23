// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.ImageCodec;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapLayer;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.EnumMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * What the server's dev map sent this client: the level's {@link MapInfoPayload}, its pictures
 * (satellite and elevation, {@link MapLayer}) once all of a picture's parts arrived (decoded off
 * the main thread) and the players. Plain data with no client classes, like Deep Time's
 * {@code ClientPlanetState}: the handlers write it on the client's main thread, the screen and the
 * HUD read it. When the server has no dev map nothing arrives and the map falls back to Deep Time's
 * own planet info (projection, no picture, no other players).
 */
public final class DevMapState {

    /** A decoded picture. */
    public record Image(String dimension, MapLayer layer, int id, MapFrame frame, int[] abgr) {}

    /** The parts of one picture on their way. */
    private static final class Pending {
        final MapLayer layer;
        final byte[][] parts;

        Pending(MapLayer layer, int count) {
            this.layer = layer;
            this.parts = new byte[Math.max(1, count)][];
        }
    }

    private static volatile MapInfoPayload info;
    /** Copied on write: readers take the map whole. */
    private static volatile Map<MapLayer, Image> images = Map.of();
    private static volatile PlayersPayload players;
    private static final Map<Integer, Pending> PENDING = new java.util.HashMap<>();
    private static final AtomicInteger VERSION = new AtomicInteger();

    private DevMapState() {}

    static synchronized void onInfo(MapInfoPayload p) {
        info = p;
        // Keep a picture only while the info still names it (same level, same image).
        Map<MapLayer, Image> kept = new EnumMap<>(MapLayer.class);
        for (Image im : images.values())
            if (im.dimension().equals(p.dimension()) && p.imageId(im.layer()) == im.id())
                kept.put(im.layer(), im);
        images = kept.isEmpty() ? Map.of() : Map.copyOf(kept);
        PENDING.entrySet().removeIf(e -> p.imageId(e.getValue().layer) != e.getKey());
        // The server sends a picture's parts right after an info that names it: collect them unless
        // that picture is already here or already being collected.
        for (MapInfoPayload.Picture pic : p.pictures())
            if (pic.id() != 0 && !kept.containsKey(pic.layer()) && !PENDING.containsKey(pic.id()))
                PENDING.put(pic.id(), new Pending(pic.layer(), pic.parts()));
        VERSION.incrementAndGet();
    }

    static synchronized void onImagePart(MapImagePayload p) {
        MapInfoPayload in = info;
        Pending pending = PENDING.get(p.imageId());
        if (in == null || pending == null || p.count() != pending.parts.length || p.index() < 0
                || p.index() >= pending.parts.length)
            return;
        pending.parts[p.index()] = p.data();
        for (byte[] b : pending.parts)
            if (b == null)
                return;
        PENDING.remove(p.imageId());
        List<byte[]> parts = new ArrayList<>(Arrays.asList(pending.parts));
        int id = p.imageId();
        MapLayer layer = pending.layer;
        MapFrame frame = in.frame();
        String dim = in.dimension();
        if (frame == null)
            return;
        Thread t = new Thread(() -> {
            try {
                int[] px = ImageCodec.decode(parts, frame.w(), frame.h());
                decoded(new Image(dim, layer, id, frame, px));
            } catch (RuntimeException e) {
                DevMap.LOGGER.error("[devmap] cannot decode {} picture {}", layer.id(), id, e);
            }
        }, "deeptime_devmap-decode");
        t.setDaemon(true);
        t.start();
    }

    private static synchronized void decoded(Image im) {
        MapInfoPayload now = info;
        if (now == null || !now.dimension().equals(im.dimension()) || now.imageId(im.layer()) != im.id())
            return;
        Map<MapLayer, Image> next = new EnumMap<>(MapLayer.class);
        next.putAll(images);
        next.put(im.layer(), im);
        images = Map.copyOf(next);
        VERSION.incrementAndGet();
        DevMap.LOGGER.info("[devmap] {} picture {} ({}x{}, {} blocks a pixel) for {}", im.layer().id(), im.id(),
                im.frame().w(), im.frame().h(), im.frame().blocksPerPx(), im.dimension());
    }

    static void onPlayers(PlayersPayload p) {
        players = p;
    }

    /** The server's info for the level the client is in, or null (no dev map on the server, or it has not arrived). */
    public static MapInfoPayload info(String dimension) {
        MapInfoPayload p = info;
        return p != null && p.dimension().equals(dimension) ? p : null;
    }

    /** The picture of {@code layer} for {@code dimension}, or null. */
    public static Image image(String dimension, MapLayer layer) {
        Image im = images.get(layer);
        return im != null && im.dimension().equals(dimension) ? im : null;
    }

    /** Whether {@code layer}'s picture of the info's level is still arriving (named, not decoded yet). */
    public static boolean receiving(MapInfoPayload p, MapLayer layer) {
        return p != null && p.imageId(layer) != 0 && image(p.dimension(), layer) == null;
    }

    /** The players the server listed for {@code dimension}, or none. */
    public static List<PlayersPayload.Entry> players(String dimension) {
        PlayersPayload p = players;
        return p != null && p.dimension().equals(dimension) ? p.players() : List.of();
    }

    /** Bumped whenever the info or a picture changes. */
    public static int version() {
        return VERSION.get();
    }

    public static synchronized void reset() {
        info = null;
        images = Map.of();
        players = null;
        PENDING.clear();
        VERSION.incrementAndGet();
    }
}
