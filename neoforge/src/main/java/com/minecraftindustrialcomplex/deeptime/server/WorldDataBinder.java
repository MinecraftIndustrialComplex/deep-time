// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.ActiveWorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.core.registries.Registries;
import net.minecraft.world.level.storage.LevelResource;
import net.neoforged.bus.api.EventPriority;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.server.ServerAboutToStartEvent;
import net.neoforged.neoforge.event.server.ServerStoppedEvent;

import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Binds each Deep Time level's world data before any level exists (plan §7.1, §7.13). At
 * {@code ServerAboutToStartEvent} — which dedicated, integrated and GameTest servers all post
 * before {@code loadLevel()} — every level stem whose generator is Deep Time's and has no test
 * terrain gets {@code <world>/<folder>/} opened into {@link ActiveWorldData}. A missing folder is
 * logged, not thrown: chunk generation then fails with a message naming it.
 *
 * <p>Staging from the Create World screen (P1-11) and first-start generation (P1-13) hook in here.
 */
public final class WorldDataBinder {

    private WorldDataBinder() {}

    public static void init() {
        NeoForge.EVENT_BUS.addListener(EventPriority.HIGHEST, ServerAboutToStartEvent.class, WorldDataBinder::bind);
        NeoForge.EVENT_BUS.addListener(ServerStoppedEvent.class, e -> ActiveWorldData.clear());
    }

    private static void bind(ServerAboutToStartEvent event) {
        var server = event.getServer();
        Path world = server.getWorldPath(LevelResource.ROOT).toAbsolutePath().normalize();
        server.registryAccess().registryOrThrow(Registries.LEVEL_STEM).entrySet().forEach(e -> {
            if (!(e.getValue().generator() instanceof GeologicChunkGenerator g) || g.dataRef().effectiveTestTerrain().isPresent())
                return;
            String folder = g.dataRef().folder();
            Path dir = world.resolve(folder);
            if (!Files.isRegularFile(dir.resolve("manifest.json"))) {
                DeepTime.LOGGER.error("Deep Time level {} has no world data at {} (manifest.json missing)", e.getKey().location(), dir);
                return;
            }
            WorldData wd = WorldData.open(dir);
            ActiveWorldData.put(folder, wd);
            DeepTime.LOGGER.info("Bound Deep Time world data {} for {} (format {}, C = {})", dir, e.getKey().location(),
                    wd.manifest().formatVersion(), wd.circumference());
        });
    }
}
