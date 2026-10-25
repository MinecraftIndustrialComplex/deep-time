// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.server;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.model.BaseImage;
import com.minecraftindustrialcomplex.deeptime.devmap.model.ImageCodec;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapFrame;
import com.minecraftindustrialcomplex.deeptime.devmap.model.MapLayer;
import com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometry;
import com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapNetwork;
import com.minecraftindustrialcomplex.deeptime.devmap.net.MapImagePayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.MapInfoPayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.PlayersPayload;
import com.minecraftindustrialcomplex.deeptime.devmap.net.TeleportPayload;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeWorldMap;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.worlddata.TileCache;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.minecraftindustrialcomplex.deeptime.worldgen.ActiveWorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef;
import com.minecraftindustrialcomplex.deeptime.wrap.CubeBorder;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.entity.player.PermissionsChangedEvent;
import net.neoforged.neoforge.event.entity.player.PlayerEvent;
import net.neoforged.neoforge.event.server.ServerStartedEvent;
import net.neoforged.neoforge.event.server.ServerStoppedEvent;
import net.neoforged.neoforge.event.tick.ServerTickEvent;
import net.neoforged.neoforge.network.handling.IPayloadContext;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.EnumMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * The server half of the dev map: the pictures of each Deep Time level, the {@link MapInfoPayload}
 * and picture parts for every dev-map client on login, dimension change and respawn, the player
 * list once a second, and the operator teleport. Players whose client lacks the dev map get nothing.
 *
 * <p>Each level has two pictures on one frame ({@link MapFrame#of}, 2048 pixels across the
 * period), drawn once per server run, one after the other on a background thread:
 * <ul>
 *   <li><b>elevation</b> ({@link BaseImage}): the world data's height and water through Deep Time's
 *       own ramps, drawn here in Java from a private world-data reader (a small tile cache, so the
 *       generator's is not disturbed). It needs no natives, so it is always there.</li>
 *   <li><b>satellite</b> ({@link NativeWorldMap}): the workbench's "Final world" look, drawn by
 *       Deep Time's natives from the bake's climate, ice, rock, lakes and rivers, the same Rust
 *       renderer as the workbench and the Create World screen's satellite view. Without the natives
 *       (or a folder they cannot read) its note says why and the client shows elevation.</li>
 * </ul>
 * A client gets the info at once (a picture still drawing is named {@link MapInfoPayload#RENDERING})
 * and then each picture's parts as it is ready, with a fresh info naming it.
 */
public final class ServerMaps {

    /** A level's pictures: their shared frame (null: none, {@code note} says why) and each picture as it is drawn. */
    record LevelMaps(MapFrame frame, String note, Map<MapLayer, CompletableFuture<Built>> pictures) {}

    /** A drawn picture: its image id and deflated parts, or id 0 and a note why there is none. */
    record Built(MapLayer layer, int id, List<byte[]> parts, String note) {
        MapInfoPayload.Picture picture() {
            return new MapInfoPayload.Picture(layer, id, parts.size(), note);
        }
    }

    private static final Map<ResourceKey<Level>, LevelMaps> MAPS = new ConcurrentHashMap<>();
    private static final AtomicInteger IDS = new AtomicInteger(1);
    private static final ExecutorService WORKER = Executors.newSingleThreadExecutor(r -> {
        Thread t = new Thread(r, "deeptime_devmap-image");
        t.setDaemon(true);
        t.setPriority(Thread.MIN_PRIORITY);
        return t;
    });

    /** The worker's order: elevation first (quick, no natives), so a client has a picture early. */
    private static final List<MapLayer> DRAW_ORDER = List.of(MapLayer.ELEVATION, MapLayer.SATELLITE);

    /** Edge-border margin a teleport keeps, blocks (inside the border's own inset). */
    static final double EDGE_MARGIN = 4.0;

    private static int tick;
    private static final Set<UUID> RESEND = ConcurrentHashMap.newKeySet();

    private ServerMaps() {}

    public static void init() {
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerLoggedInEvent.class, e -> sendInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerChangedDimensionEvent.class, e -> sendInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerRespawnEvent.class, e -> sendInfo(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(ServerStartedEvent.class, e -> e.getServer().getAllLevels().forEach(ServerMaps::maps));
        NeoForge.EVENT_BUS.addListener(ServerStoppedEvent.class, e -> MAPS.clear());
        // Fired before the change: the new permission (the teleport flag) goes out on the next tick.
        NeoForge.EVENT_BUS.addListener(PermissionsChangedEvent.class, e -> RESEND.add(e.getEntity().getUUID()));
        NeoForge.EVENT_BUS.addListener(ServerTickEvent.Post.class, e -> {
            if (!RESEND.isEmpty())
                for (UUID id : List.copyOf(RESEND)) {
                    RESEND.remove(id);
                    ServerPlayer sp = e.getServer().getPlayerList().getPlayer(id);
                    if (sp != null)
                        sendInfo(sp);
                }
            if (++tick % 20 == 0)
                sendPlayers(e.getServer());
        });
    }

    /** The level's Deep Time generator, or null when it is not a planet. */
    static GeologicChunkGenerator generator(ServerLevel level) {
        return level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g ? g : null;
    }

    /** The pictures of {@code level}, queued for drawing on first use; null when it is not a planet. */
    static LevelMaps maps(ServerLevel level) {
        GeologicChunkGenerator g = generator(level);
        if (g == null)
            return null;
        return MAPS.computeIfAbsent(level.dimension(), k -> {
            WorldDataRef ref = g.dataRef();
            if (ref.effectiveTestTerrain().isPresent())
                return new LevelMaps(null, "test terrain " + ref.effectiveTestTerrain().get()
                        + ": no world data, graticule only", Map.of());
            if (!ActiveWorldData.has(ref.folder()))
                return new LevelMaps(null, "world data '" + ref.folder() + "' is not bound: graticule only", Map.of());
            WorldData active = ActiveWorldData.get(ref.folder());
            String dim = k.location().toString();
            MapFrame f = MapFrame.of(active.projection(), MapFrame.DEFAULT_WIDTH);
            Map<MapLayer, CompletableFuture<Built>> pictures = new EnumMap<>(MapLayer.class);
            for (MapLayer layer : DRAW_ORDER)
                pictures.put(layer, CompletableFuture.supplyAsync(() -> draw(layer, active.folder(), active.projection(), f, dim), WORKER));
            return new LevelMaps(f, String.format(Locale.ROOT, "frame %dx%d, %.1f blocks/px", f.w(), f.h(),
                    f.blocksPerPx()), Collections.unmodifiableMap(pictures));
        });
    }

    /** Draws one picture of the frame and deflates it; never throws (a failure is the picture's note). */
    private static Built draw(MapLayer layer, Path folder, WorldProjection projection, MapFrame f, String dim) {
        long t0 = System.nanoTime();
        try {
            int[] px;
            String what;
            if (layer == MapLayer.SATELLITE) {
                // The natives draw on threads of their own: a few, beside a server making chunks.
                int threads = Math.max(1, Math.min(4, Runtime.getRuntime().availableProcessors() / 4));
                NativeWorldMap.Picture pic = NativeWorldMap.render(folder, NativeWorldMap.SATELLITE, f.x0(), f.z0(),
                        f.blocksPerPx(), f.w(), f.h(), threads);
                // A cube's void, where the natives draw the copies the fold brings in, is left clear.
                px = BaseImage.maskVoid(pic.abgr(), f, projection);
                what = "true colour from the bake, the workbench's look";
                DevMap.LOGGER.info("[devmap] satellite of {} from the natives: {}", dim, pic.info());
            } else {
                // Its own world-data reader with a small cache: the whole planet once, without
                // evicting the tiles the generator is using.
                WorldData wd = WorldData.open(folder, new TileCache(64L << 20));
                WorldProjection p = wd.projection();
                Ramps ramps = Ramps.builtin();
                px = BaseImage.render(f, p, wd::height, wd::waterLevel, (x, z) -> wd.closedBasin(x, z) ? 1.0 : 0.0,
                        wd.manifest().seaLevelY(), ramps.forLayer(WorldData.ELEVATION), ramps.forLayer(WorldData.WATER_LEVEL));
                what = "world data height and water through Deep Time's ramps";
            }
            List<byte[]> parts = ImageCodec.encode(px);
            long bytes = parts.stream().mapToLong(b -> b.length).sum();
            long ms = (System.nanoTime() - t0) / 1_000_000;
            int id = IDS.getAndIncrement();
            DevMap.LOGGER.info("[devmap] {} picture {} of {}: {}x{}, {} blocks a pixel, {} KiB in {} parts, {} ms",
                    layer.id(), id, dim, f.w(), f.h(), f.blocksPerPx(), bytes / 1024, parts.size(), ms);
            return new Built(layer, id, parts, String.format(Locale.ROOT, "%s, drawn in %.1f s", what, ms / 1000.0));
        } catch (RuntimeException | LinkageError e) {
            // No natives (IllegalStateException, UnsatisfiedLinkError), a folder they cannot read
            // (DeepTimeIoException) or anything else: the other picture still shows.
            DevMap.LOGGER.warn("[devmap] cannot draw the {} picture of {}: {}", layer.id(), dim, e.toString());
            return new Built(layer, 0, List.of(), "not drawn: " + e.getMessage());
        }
    }

    /** Sends {@code player} the info of its level now, the pictures that are ready, and each other one once it is. */
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
        LevelMaps m = maps(level);
        DevMapNetwork.send(sp, info(sp, level, g, m));
        MinecraftServer server = level.getServer();
        ResourceKey<Level> key = level.dimension();
        for (CompletableFuture<Built> f : m.pictures().values()) {
            Built ready = f.getNow(null);
            if (ready != null) {
                sendParts(sp, ready);
                continue;
            }
            f.thenAccept(b -> server.execute(() -> {
                if (sp.hasDisconnected() || sp.level().dimension() != key)
                    return;
                // A fresh info names the picture (and any other finished meanwhile), then its parts.
                DevMapNetwork.send(sp, info(sp, sp.serverLevel(), g, m));
                sendParts(sp, b);
            }));
        }
    }

    /** The info of {@code level} as it stands: each picture drawn, failed or still {@link MapInfoPayload#RENDERING}. */
    private static MapInfoPayload info(ServerPlayer sp, ServerLevel level, GeologicChunkGenerator g, LevelMaps m) {
        WorldDataRef ref = g.dataRef();
        MapFrame f = m.frame();
        List<MapInfoPayload.Picture> pictures = new ArrayList<>();
        for (Map.Entry<MapLayer, CompletableFuture<Built>> e : m.pictures().entrySet()) {
            Built b = e.getValue().getNow(null);
            pictures.add(b != null ? b.picture() : new MapInfoPayload.Picture(e.getKey(), 0, 0, MapInfoPayload.RENDERING + "..."));
        }
        return new MapInfoPayload(level.dimension().location().toString(), true, m.note(), ref.projection(),
                ref.circumferenceBlocks(), f == null ? 0 : f.w(), f == null ? 0 : f.h(), f == null ? 0 : f.x0(),
                f == null ? 0 : f.z0(), f == null ? 1 : f.blocksPerPx(), pictures, g.borderInset(), level.getSeaLevel(),
                sp.hasPermissions(2));
    }

    /** The parts of a drawn picture (none for one that failed). */
    private static void sendParts(ServerPlayer sp, Built b) {
        if (b.id() == 0)
            return;
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
     * wrapped, z folded), kept {@link #EDGE_MARGIN} blocks inside the cube's border (or inside the
     * poles of a Lambert-frame test level), its chunk loaded and the player put on top of the highest
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
        double[] target = target(p, g.borderInset(), req.x(), req.z());
        int bx = (int) Math.floor(target[0]), bz = (int) Math.floor(target[1]);
        level.getChunk(bx >> 4, bz >> 4);
        int y = level.getHeight(Heightmap.Types.MOTION_BLOCKING_NO_LEAVES, bx, bz);
        sp.teleportTo(level, bx + 0.5, y, bz + 0.5, sp.getYRot(), sp.getXRot());
        sp.resetFallDistance();
        // A player who may fly (creative, spectator) stays at the landing height instead of sinking
        // through deep water.
        if (sp.getAbilities().mayfly && !sp.getAbilities().flying) {
            sp.getAbilities().flying = true;
            sp.onUpdateAbilities();
        }
        double[] ll = PlanetGeometry.displayLatLon(p, bx + 0.5, bz + 0.5);
        String where = String.format(Locale.ROOT, "%d %d %d (%s)", bx, y, bz, PlanetGeometry.formatLatLon(ll[0], ll[1]));
        DevMap.LOGGER.info("[devmap] teleported {} to {} (asked {}, {}); {}", sp.getGameProfile().getName(), where, req.x(), req.z(),
                geography(level, g.dataRef(), bx, y, bz));
        sp.sendSystemMessage(Component.literal("Dev map: teleported to " + where));
    }

    /**
     * What the world data says about column (x, z) next to what the world has there: the base
     * image is drawn from H and the water level, so the two agreeing is the image agreeing with the
     * terrain the player sees.
     */
    static String geography(ServerLevel level, WorldDataRef ref, int x, int surfaceY, int z) {
        BlockPos top = new BlockPos(x, surfaceY - 1, z);
        String block = BuiltInRegistries.BLOCK.getKey(level.getBlockState(top).getBlock()).toString();
        boolean wet = !level.getFluidState(top).isEmpty();
        if (!ActiveWorldData.has(ref.folder()))
            return String.format(Locale.ROOT, "surface y %d (%s%s), no world data", surfaceY, block, wet ? ", water" : "");
        WorldData wd = ActiveWorldData.get(ref.folder());
        double h = wd.height(x + 0.5, z + 0.5), wl = wd.waterLevel(x + 0.5, z + 0.5);
        int sea = wd.manifest().seaLevelY();
        // The sea stands below sea level unless the column is in a closed basin (HYD-100).
        boolean dataWet = (h < sea && !wd.closedBasin(x + 0.5, z + 0.5)) || (!Double.isNaN(wl) && wl > h);
        return String.format(Locale.ROOT, "surface y %d (%s%s); world data H %.1f, water level %s, sea level %d (%s); %s",
                surfaceY, block, wet ? ", water" : "", h, Double.isNaN(wl) ? "none" : String.format(Locale.ROOT, "%.1f", wl), sea,
                dataWet ? "water" : "land", wet == dataWet ? "AGREE" : "DIFFER");
    }

    /**
     * Where a teleport to (x, z) lands, {x, z}: on a cube the nearest point at least the border's
     * inset and {@link #EDGE_MARGIN} inside its segments ({@code CubeBorder.nearest}: the nearest
     * point of the planet, never a void copy); on a Lambert-frame test level canonical and
     * {@link #EDGE_MARGIN} inside the poles.
     */
    static double[] target(WorldProjection p, double borderInset, double x, double z) {
        WorldProjection.Cube cube = PlanetGeometry.cube(p);
        if (cube != null) {
            double inset = Double.isNaN(borderInset) ? 0.0 : borderInset;
            double[] n = CubeBorder.of(cube.side()).nearest(cube.wrapX(x), z, inset + EDGE_MARGIN);
            return new double[]{cube.wrapX(n[0]), n[1]};
        }
        double[] c = p.canonical(x, z);
        double lo = p.zMin() + EDGE_MARGIN, hi = p.zMax() - EDGE_MARGIN;
        return new double[]{c[0], Math.max(lo, Math.min(hi, c[1]))};
    }
}
