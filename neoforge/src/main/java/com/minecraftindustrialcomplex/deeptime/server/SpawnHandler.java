// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.level.LevelEvent;

import java.nio.file.Files;
import java.util.Optional;

/**
 * Spawn on a Deep Time planet (plan §7.8), v0: the first of the world data's ranked spawn
 * candidates ({@code spawn/candidates.json}) whose generated surface is above sea level. Replaces
 * vanilla's spawn search through the cancellable {@code LevelEvent.CreateSpawnPosition}, which
 * fires once when a new overworld is created. P1-15 adds the essentials rule.
 */
public final class SpawnHandler {

    private SpawnHandler() {}

    public static void init() {
        NeoForge.EVENT_BUS.addListener(LevelEvent.CreateSpawnPosition.class, SpawnHandler::onCreateSpawn);
    }

    private static void onCreateSpawn(LevelEvent.CreateSpawnPosition event) {
        if (!(event.getLevel() instanceof ServerLevel level) || level.dimension() != Level.OVERWORLD)
            return;
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain w))
            return;
        chooseSpawn(level, w.data()).ifPresent(pos -> {
            level.setDefaultSpawnPos(pos, 0.0F);
            event.setCanceled(true);
            DeepTime.LOGGER.info("Deep Time spawn at {} (from the world data's spawn candidates)", pos);
        });
    }

    /** The first candidate that is dry land once generated. */
    public static Optional<BlockPos> chooseSpawn(ServerLevel level, WorldData wd) {
        try {
            var root = JsonParser.parseString(Files.readString(wd.folder().resolve("spawn/candidates.json"))).getAsJsonObject();
            var generator = level.getChunkSource().getGenerator();
            int sea = generator.getSeaLevel();
            for (var e : root.getAsJsonArray("candidates")) {
                var c = e.getAsJsonObject();
                int x = c.get("x").getAsInt(), z = c.get("z").getAsInt();
                int y = generator.getBaseHeight(x, z, Heightmap.Types.OCEAN_FLOOR_WG, level, level.getChunkSource().randomState());
                if (y > sea)
                    return Optional.of(new BlockPos(x, y, z));
            }
        } catch (Exception e) {
            DeepTime.LOGGER.error("Deep Time: no usable spawn candidates in {}", wd.folder(), e);
        }
        return Optional.empty();
    }
}
