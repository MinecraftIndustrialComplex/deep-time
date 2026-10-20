// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.server;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.BaseImage;
import com.minecraftindustrialcomplex.deeptime.devmap.model.ImageCodec;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;
import com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometry;
import com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapNetwork;
import com.minecraftindustrialcomplex.deeptime.devmap.net.MapImagePayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.MapInfoPayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.PlayersPayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.TeleportPayload;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.worlddata.Projection;
import com.minecraftindustrialcomplex.deeptime.worlddata.TileCache;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.minecraftindustrialcomplex.deeptime.worldgen.ActiveWorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.entity.player.PlayerEvent;
import net.neoforged.neoforge.event.server.ServerStartedEvent;
import net.neoforged.neoforge.event.server.ServerStoppedEvent;
import net.neoforged.neoforge.event.tick.ServerTickEvent;
import net.neoforged.neoforge.network.handling.IPayloadContext;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * The server half of the dev map: the base image of each Deep Time level (rendered once per server
 * run on a background thread from the level's world data, with its own small tile cache so the
 * generator's is not disturbed), the {@link MapInfoPayload} and image parts for every dev-map client
 * on login, dimension change and respawn, the player list once a second, and the operator teleport.
 * Players whose client lacks the dev map get nothing.
 */
public final class ServerMaps {

    /** The base image of a level: its frame and deflated parts, or a note why there is none. */
    record Built(int id, MapFrame frame, List<byte[]> parts, String note) {}

    private static final Map<ResourceKey<Level>, CompletableFuture<Built>> IMAGES = new ConcurrentHashMap<>();
    private static final AtomicInteger IDS = new AtomicInteger(1);
    private static final ExecutorService WORKER = Executors.newSingleThreadExecutor(r -> {
        Thread t = new Thread(r, "deeptime_devmap-image");
        t.setDaemon(true);
        t.setPriority(Thread.MIN_PRIORITY);
        return t;
    });

    /** Edge-border margin a teleport keeps, blocks (inside the border's own inset). */
    static final double EDGE_MARGIN = 4.0;

    private static int tick;

    private ServerMaps() {}

    public static void init() {
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerLoggedInEvent.class, e -> sendInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerChangedDimensionEvent.class, e -> sendInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerRespawnEvent.class, e -> sendInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(ServerStartedEvent.class, e -> e.getServer().getAllLevels().forEach(ServerMaps::image));
        NeoForge.EVENT_BUS.addListener(ServerStoppedEvent.class, e -> IMAGES.clear());
        NeoForge.EVENT_BUS.addListener(ServerTickEvent.Post.class, e -> {
            if (++tick % 20 == 0)
                sendPlayers(e.getServer());
        });
    }

    /** The level's Deep Time generator, or null when it is not a planet. */
    static GeologicChunkGenerator generator(ServerLevel level) {
        return level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g ? g : null;
    }

    /** The base image of {@code level}, rendered on first use (null future value never: a note instead). */
    static CompletableFuture<Built> image(ServerLevel level) {
        GeologicChunkGenerator g = generator(level);
        if (g == null)
            return null;
        return IMAGES.computeIfAbsent(level.dimension(), k -> {
            WorldDataRef ref = g.dataRef();
            if (ref.effectiveTestTerrain().isPresent())
                return CompletableFuture.completedFuture(new Built(0, null, List.of(),
                        "test terrain " + ref.effectiveTestTerrain().get() + ": no world data, graticule only"));
            if (!ActiveWorldData.has(ref.folder()))
                return CompletableFuture.completedFuture(new Built(0, null, List.of(),
                        "world data '" + ref.folder() + "' is not bound: graticule only"));
            WorldData active = ActiveWorldData.get(ref.folder());
            String dim = k.location().toString();
            return CompletableFuture.supplyAsync(() -> render(active, dim), WORKER);
        });
    }

    private static Built render(WorldData active, String dim) {
        long t0 = System.nanoTime();
        try {
            // Its own world-data reader with a small cache: the whole planet once, without evicting
            // the tiles the generator is using.
            WorldData wd = WorldData.open(active.folder(), new TileCache(64L << 20));
            WorldProjection p = wd.projection();
            MapFrame f = MapFrame.of(p, MapFrame.DEFAULT_WIDTH);
            Ramps ramps = Ramps.builtin();
            int[] px = BaseImage.render(f, p, wd::height, wd::waterLevel, wd.manifest().seaLevelY(),
                    ramps.forLayer(WorldData.ELEVATION), ramps.forLayer(WorldData.WATER_LEVEL));
            List<byte[]> parts = ImageCodec.encode(px);
            long bytes = parts.stream().mapToLong(b -> b.length).sum();
            int id = IDS.getAndIncrement();
            DevMap.LOGGER.info("[devmap] base image {} of {}: {}x{}, {} blocks a pixel, {} KiB in {} parts, {} ms",
                    id, dim, f.w(), f.h(), f.blocksPerPx(), bytes / 1024, parts.size(), (System.nanoTime() - t0) / 1_000_000);
            return new Built(id, f, parts, String.format(Locale.ROOT, "base image: world data elevation, %dx%d, %.1f blocks/px",
                    f.w(), f.h(), f.blocksPerPx()));
        } catch (RuntimeException e) {
            DevMap.LOGGER.error("[devmap] cannot render the base image of {}", dim, e);
            return new Built(0, null, List.of(), "base image failed: " + e.getMessage());
        }
    }

    /** Sends {@code player} the info of its level now, and the image once it is ready. */
    public static void sendInfo(Player player) {
        if (!(player instanceof ServerPlayer sp) || !DevMapNetwork.hasDevMap(sp))
            return;
        ServerLevel level = sp.serverLevel();
        String dim = level.dimension().location().toString();
        GeologicChunkGenerator g = generator(level);
        if (g == null) {
            DevMapNetwork.send(sp, MapInfoPayload.none(dim, "no Deep Time planet in this dimension"));
            return;
        }
        CompletableFuture<Built> f = image(level);
        Built ready = f.getNow(null);
        if (ready != null) {
            sendBuilt(sp, level, g, ready);
            return;
        }
        sendBuilt(sp, level, g, new Built(0, null, List.of(), "base image: rendering on the server..."));
        MinecraftServer server = level.getServer();
        ResourceKey<Level> key = level.dimension();
        f.thenAccept(b -> server.execute(() -> {
            if (!sp.hasDisconnected() && sp.level().dimension() == key)
                sendBuilt(sp, sp.serverLevel(), g, b);
        }));
    }

    private static void sendBuilt(ServerPlayer sp, ServerLevel level, GeologicChunkGenerator g, Built b) {
        WorldDataRef ref = g.dataRef();
        int sea = level.getSeaLevel();
        MapFrame f = b.frame();
        MapInfoPayload info = new MapInfoPayload(level.dimension().location().toString(), true, b.note(), ref.projection(),
                ref.circumferenceBlocks(), f == null ? 0 : b.id(), f == null ? 0 : f.w(), f == null ? 0 : f.h(),
                f == null ? 0 : f.x0(), f == null ? 0 : f.z0(), f == null ? 1 : f.blocksPerPx(), b.parts().size(),
                g.borderInset(), sea, sp.hasPermissions(2));
        DevMapNetwork.send(sp, info);
        if (f != null)
            for (int i = 0; i < b.parts().size(); i++)
                DevMapNetwork.send(sp, new MapImagePayload(b.id(), i, b.parts().size(), b.parts().get(i)));
    }

    private static void sendPlayers(MinecraftServer server) {
        for (ServerLevel level : server.getAllLevels()) {
            if (generator(level) == null || level.players().isEmpty())
                continue;
            List<PlayersPayload.Entry> list = new ArrayList<>();
            for (ServerPlayer p : level.players())
                list.add(new PlayersPayload.Entry(p.getGameProfile().getName(), p.getX(), p.getY(), p.getZ(), p.getYRot()));
            PlayersPayload payload = new PlayersPayload(level.dimension().location().toString(), List.copyOf(list));
            for (ServerPlayer p : level.players())
                DevMapNetwork.send(p, payload);
        }
    }

    /**
     * A map teleport: operators only (level 2). The point is brought into the world's domain (x
     * wrapped, z folded), kept {@link #EDGE_MARGIN} blocks inside the edge border (Petroff–Guyou)
     * or inside the pole stop (Lambert), its chunk loaded and the player put on top of the highest
     * motion-blocking block there (leaves skipped).
     */
    public static void onTeleport(TeleportPayload req, IPayloadContext ctx) {
        if (!(ctx.player() instanceof ServerPlayer sp))
            return;
        if (!sp.hasPermissions(2)) {
            sp.sendSystemMessage(Component.literal("Dev map: teleporting needs operator permission"));
            return;
        }
        ServerLevel level = sp.serverLevel();
        GeologicChunkGenerator g = generator(level);
        if (g == null || !level.dimension().location().toString().equals(req.dimension()) || !Double.isFinite(req.x())
                || !Double.isFinite(req.z()))
            return;
        WorldProjection p = WorldInfoPayload.forLevel(level).worldProjection();
        if (p == null)
            return;
        double[] target = target(p, g.dataRef(), g.borderInset(), req.x(), req.z());
        int bx = (int) Math.floor(target[0]), bz = (int) Math.floor(target[1]);
        level.getChunk(bx >> 4, bz >> 4);
        int y = level.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, bx, bz);
        sp.teleportTo(level, bx + 0.5, y, bz + 0.5, sp.getYRot(), sp.getXRot());
        sp.resetFallDistance();
        double[] ll = PlanetGeometry.displayLatLon(p, bx + 0.5, bz + 0.5);
        String where = String.format(Locale.ROOT, "%d %d %d (%s)", bx, y, bz, PlanetGeometry.formatLatLon(ll[0], ll[1]));
        DevMap.LOGGER.info("[devmap] teleported {} to {} (asked {}, {})", sp.getGameProfile().getName(), where, req.x(), req.z());
        sp.sendSystemMessage(Component.literal("Dev map: teleported to " + where));
    }

    /** Where a teleport to (x, z) lands, {x, z}: canonical, and inside the edge border or the pole stop. */
    static double[] target(WorldProjection p, WorldDataRef ref, double borderInset, double x, double z) {
        double[] c = p.canonical(x, z);
        double lo, hi;
        if (PlanetGeometry.petroffGuyou(p)) {
            double inset = Double.isNaN(borderInset) ? 0.0 : borderInset;
            lo = p.zMin() + inset + EDGE_MARGIN;
            hi = p.zMax() - inset - EDGE_MARGIN;
        } else {
            double stop = Projection.zOfLatitude(p.period(), Math.max(0.0, ref.poleStopDeg() - 0.5));
            lo = Math.max(p.zMin(), stop) + EDGE_MARGIN;
            hi = Math.min(p.zMax(), -stop) - EDGE_MARGIN;
        }
        return new double[]{c[0], Math.max(lo, Math.min(hi, c[1]))};
    }
}
