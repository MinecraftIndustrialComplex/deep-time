// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.net;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.ImageCodec;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * What the server's dev map sent this client: the level's {@link MapInfoPayload}, the base image
 * once all its parts arrived (decoded off the main thread) and the players. Plain data with no
 * client classes, like Deep Time's {@code ClientPlanetState}: the handlers write it on the client's
 * main thread, the screen and the HUD read it. When the server has no dev map nothing arrives and
 * the map falls back to Deep Time's own planet info (projection, no image, no other players).
 */
public final class DevMapState {

    /** A decoded base image. */
    public record Image(String dimension, int id, MapFrame frame, int[] abgr) {}

    private static volatile MapInfoPayload info;
    private static volatile Image image;
    private static volatile PlayersPayload players;
    private static byte[][] pending;
    private static int pendingId;
    private static final AtomicInteger VERSION = new AtomicInteger();

    private DevMapState() {}

    static void onInfo(MapInfoPayload p) {
        info = p;
        Image im = image;
        if (im != null && (im.id() != p.imageId() || !im.dimension().equals(p.dimension()))) {
            image = null;
            im = null;
        }
        // The server sends the parts right after an info that names an image: collect them unless
        // that image is already here or already being collected.
        if (p.imageId() != 0 && im == null && !(pending != null && pendingId == p.imageId())) {
            pendingId = p.imageId();
            pending = new byte[Math.max(1, p.parts())][];
        }
        VERSION.incrementAndGet();
    }

    static void onImagePart(MapImagePayload p) {
        MapInfoPayload in = info;
        if (in == null || p.imageId() != pendingId || pending == null || p.count() != pending.length
                || p.index() < 0 || p.index() >= pending.length)
            return;
        pending[p.index()] = p.data();
        for (byte[] b : pending)
            if (b == null)
                return;
        List<byte[]> parts = new ArrayList<>(Arrays.asList(pending));
        pending = null;
        int id = pendingId;
        MapFrame frame = in.frame();
        String dim = in.dimension();
        if (frame == null)
            return;
        Thread t = new Thread(() -> {
            try {
                int[] px = ImageCodec.decode(parts, frame.w(), frame.h());
                MapInfoPayload now = info;
                if (now != null && now.imageId() == id) {
                    image = new Image(dim, id, frame, px);
                    VERSION.incrementAndGet();
                    DevMap.LOGGER.info("[devmap] base image {} ({}x{}, {} blocks a pixel) for {}", id, frame.w(), frame.h(),
                            frame.blocksPerPx(), dim);
                }
            } catch (RuntimeException e) {
                DevMap.LOGGER.error("[devmap] cannot decode base image {}", id, e);
            }
        }, "deeptime_devmap-decode");
        t.setDaemon(true);
        t.start();
    }

    static void onPlayers(PlayersPayload p) {
        players = p;
    }

    /** The server's info for the level the client is in, or null (no dev map on the server, or it has not arrived). */
    public static MapInfoPayload info(String dimension) {
        MapInfoPayload p = info;
        return p != null && p.dimension().equals(dimension) ? p : null;
    }

    /** The base image of {@code dimension}, or null. */
    public static Image image(String dimension) {
        Image im = image;
        return im != null && im.dimension().equals(dimension) ? im : null;
    }

    /** The players the server listed for {@code dimension}, or none. */
    public static List<PlayersPayload.Entry> players(String dimension) {
        PlayersPayload p = players;
        return p != null && p.dimension().equals(dimension) ? p.players() : List.of();
    }

    /** Bumped whenever the info or the image changes. */
    public static int version() {
        return VERSION.get();
    }

    public static void reset() {
        info = null;
        image = null;
        players = null;
        pending = null;
        pendingId = 0;
        VERSION.incrementAndGet();
    }
}
