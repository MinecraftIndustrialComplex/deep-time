// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.ActiveWorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef;
import net.minecraft.core.registries.Registries;
import net.minecraft.server.MinecraftServer;
import net.minecraft.world.level.storage.LevelResource;
import net.neoforged.bus.api.EventPriority;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.server.ServerAboutToStartEvent;
import net.neoforged.neoforge.event.server.ServerStoppedEvent;

import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Clock;
import java.util.HashSet;
import java.util.Optional;
import java.util.Set;

/**
 * Binds each Deep Time level's world data before any level exists (plan §7.1, §7.13). At
 * {@code ServerAboutToStartEvent} — which dedicated, integrated and GameTest servers all post
 * before {@code loadLevel()} — every level stem whose generator is Deep Time's and has no test
 * terrain gets {@code <world>/<folder>/} opened into {@link ActiveWorldData}. A missing folder is
 * logged, not thrown: chunk generation then fails with a message naming it.
 *
 * <p>The one HIGHEST listener runs, in this order, so the order never depends on how the event bus
 * orders two listeners of one priority:
 * <ol>
 *   <li>{@link #moveStaged}: a generator bound to a Create World staging folder (P1-11,
 *       {@code WorldDataRef.stagingId}) gets that folder moved into {@code <world>/<folder>/}
 *       ({@link StagingMover}); abandoned staging folders are cleaned up in the background.</li>
 *   <li>{@link FirstStartGenerator#prepare} (P1-13): a dedicated server's first start generates
 *       the world data; when it stops the server, nothing is bound.</li>
 *   <li>{@link #bind}.</li>
 * </ol>
 */
public final class WorldDataBinder {

    private WorldDataBinder() {}

    public static void init() {
        NeoForge.EVENT_BUS.addListener(EventPriority.HIGHEST, ServerAboutToStartEvent.class, WorldDataBinder::onAboutToStart);
        NeoForge.EVENT_BUS.addListener(ServerStoppedEvent.class, e -> ActiveWorldData.clear());
    }

    private static void onAboutToStart(ServerAboutToStartEvent event) {
        MinecraftServer server = event.getServer();
        if (!moveStaged(server))
            return; // a dedicated server stops before loadLevel()
        if (!FirstStartGenerator.prepare(server))
            return; // the server stops before loadLevel(): nothing to bind
        bind(server);
    }

    /**
     * Moves every staging-bound level's staged world data into the world. Returns {@code false}
     * when a dedicated server was asked to stop; on an integrated server a failure throws, which
     * ends the world's start with the message instead of a worldgen crash later.
     */
    private static boolean moveStaged(MinecraftServer server) {
        Path gameDir = FMLPaths.GAMEDIR.get();
        Path world = server.getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
        Set<String> referenced = new HashSet<>();
        boolean[] ok = {true};
        server.registryAccess().registryOrThrow(Registries.LEVEL_STEM).entrySet().forEach(e -> {
            if (!ok[0] || !(e.getValue().generator() instanceof GeologicChunkGenerator g))
                return;
            g.dataRef().stagingId().ifPresent(referenced::add);
            try {
                moveStaged(gameDir, world, g.dataRef(), false);
            } catch (StagingMover.StagingException ex) {
                String headline = "cannot move the staged Deep Time world data into " + world.resolve(g.dataRef().folder());
                if (server.isDedicatedServer()) {
                    ServerStop.request(server, headline, ex.getMessage() + "\nCopy complete world data (a folder with its manifest.json) into "
                            + world.resolve(g.dataRef().folder()) + " and start again.", true);
                    ok[0] = false;
                } else {
                    throw new IllegalStateException(headline + ": " + ex.getMessage(), ex);
                }
            }
        });
        cleanupInBackground(gameDir, referenced);
        return ok[0];
    }

    /**
     * Moves the staging folder {@code ref} names into {@code <world>/<folder>/}. Empty when the ref
     * names no staging folder or has a test terrain.
     *
     * @param forceCopy skip the atomic rename (GameTests)
     */
    public static Optional<StagingMover.Result> moveStaged(Path gameDir, Path world, WorldDataRef ref, boolean forceCopy)
            throws StagingMover.StagingException {
        if (ref.stagingId().isEmpty() || ref.effectiveTestTerrain().isPresent())
            return Optional.empty();
        String id = ref.stagingId().get();
        if (!Staging.isId(id))
            throw new StagingMover.StagingException("malformed staging id '" + id + "' in the world's generator settings");
        Path target = world.resolve(ref.folder());
        StagingMover.Result r = StagingMover.move(Staging.dir(gameDir, id), target, forceCopy);
        switch (r.kind()) {
            case ALREADY_PRESENT -> DeepTime.LOGGER.debug("Deep Time world data already in {} (staging {})", target, id);
            case RENAMED -> DeepTime.LOGGER.info("Moved staged Deep Time world data {} into {} (rename)", r.from(), target);
            case COPIED -> DeepTime.LOGGER.info("Copied staged Deep Time world data {} into {} ({} files, {} bytes, CRC32 verified)",
                    r.from(), target, r.files(), r.bytes());
        }
        return Optional.of(r);
    }

    private static void cleanupInBackground(Path gameDir, Set<String> keep) {
        if (!Files.isDirectory(Staging.root(gameDir)))
            return;
        Thread t = new Thread(() -> {
            var removed = Staging.cleanupAbandoned(gameDir, Staging.ABANDONED_AFTER, Set.copyOf(keep), Clock.systemUTC());
            if (!removed.isEmpty())
                DeepTime.LOGGER.info("Removed {} abandoned Deep Time staging folder(s): {}", removed.size(), removed);
        }, "deeptime-staging-cleanup");
        t.setDaemon(true);
        t.start();
    }

    private static void bind(MinecraftServer server) {
        Path world = server.getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
        server.registryAccess().registryOrThrow(Registries.LEVEL_STEM).entrySet().forEach(e -> {
            if (!(e.getValue().generator() instanceof GeologicChunkGenerator g))
                return;
            if (!bindLevel(world, g.dataRef()))
                DeepTime.LOGGER.error("Deep Time level {} has no world data at {} (manifest.json missing)", e.getKey().location(),
                        world.resolve(g.dataRef().folder()));
        });
    }

    /**
     * Opens {@code <world>/<folder>/} into {@link ActiveWorldData}. Returns {@code false} when it
     * holds no world data; {@code true} when bound, or when the ref has a test terrain (nothing to bind).
     */
    public static boolean bindLevel(Path world, WorldDataRef ref) {
        if (ref.effectiveTestTerrain().isPresent())
            return true;
        String folder = ref.folder();
        Path dir = world.resolve(folder);
        if (!Files.isRegularFile(dir.resolve("manifest.json")))
            return false;
        WorldData wd = WorldData.open(dir);
        ActiveWorldData.put(folder, wd);
        DeepTime.LOGGER.info("Bound Deep Time world data {} (format {}, C = {}{})", dir, wd.manifest().formatVersion(),
                wd.circumference(), ref.stagingId().map(s -> ", staged as " + s).orElse(""));
        return true;
    }
}
