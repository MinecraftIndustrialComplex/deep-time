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
 * Spawn on a Deep Time planet (plan §7.8): of the world data's ranked spawn candidates
 * ({@code spawn/candidates.json}), the first whose generated surface is above sea level among those
 * with the fewest essentials out of reach ({@code unmet}, written by the core's spawn ranking,
 * P1-15; a file without it counts every candidate as 0). Replaces vanilla's spawn search through the
 * cancellable {@code LevelEvent.CreateSpawnPosition}, which fires once when a new overworld is
 * created.
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
        choose(level, w.data()).ifPresent(choice -> {
            level.setDefaultSpawnPos(choice.pos(), 0.0F);
            event.setCanceled(true);
            DeepTime.LOGGER.info("Deep Time spawn at {} (from the world data's spawn candidates; essentials {})", choice.pos(),
                    choice.candidate().has("essentials") ? choice.candidate().get("essentials") : "not ranked");
        });
    }

    /** The chosen spawn and the candidate record it came from. */
    public record Choice(BlockPos pos, com.google.gson.JsonObject candidate) {}

    /** The candidate that is dry land once generated, fewest unmet essentials first, then file order. */
    public static Optional<BlockPos> chooseSpawn(ServerLevel level, WorldData wd) {
        return choose(level, wd).map(Choice::pos);
    }

    public static Optional<Choice> choose(ServerLevel level, WorldData wd) {
        try {
            var root = JsonParser.parseString(Files.readString(wd.folder().resolve("spawn/candidates.json"))).getAsJsonObject();
            var generator = level.getChunkSource().getGenerator();
            int sea = generator.getSeaLevel();
            Choice best = null;
            int bestUnmet = Integer.MAX_VALUE;
            for (var e : root.getAsJsonArray("candidates")) {
                var c = e.getAsJsonObject();
                int unmet = c.has("unmet") ? c.get("unmet").getAsInt() : 0;
                if (unmet >= bestUnmet)
                    continue;
                int x = c.get("x").getAsInt(), z = c.get("z").getAsInt();
                int y = generator.getBaseHeight(x, z, Heightmap.Types.OCEAN_FLOOR_WG, level, level.getChunkSource().randomState());
                if (y > sea) {
                    best = new Choice(new BlockPos(x, y, z), c);
                    bestUnmet = unmet;
                    if (unmet == 0)
                        break;
                }
            }
            return Optional.ofNullable(best);
        } catch (Exception e) {
            DeepTime.LOGGER.error("Deep Time: no usable spawn candidates in {}", wd.folder(), e);
        }
        return Optional.empty();
    }
}
