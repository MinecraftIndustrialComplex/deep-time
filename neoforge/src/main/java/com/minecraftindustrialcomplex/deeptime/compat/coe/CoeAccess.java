// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.coe;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.BlockPos;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.chunk.LevelChunk;
import net.neoforged.fml.ModList;

import java.util.List;
import java.util.Optional;

/**
 * Create Ore Excavation's own answers, for the probes and the GameTests (P1-16): what COE holds for
 * a chunk, what its generator picks, its vein recipes. Only {@link CoeAccessImpl} names COE's
 * classes; it is loaded reflectively and only when COE is installed. Signatures use vanilla types.
 */
public interface CoeAccess {

    /**
     * COE's state of one chunk.
     *
     * @param vein      the chunk's vein recipe, if any
     * @param randomMul COE's {@code randomMul} (0–1)
     * @param amount    the vein's size as COE computes it for a finite vein,
     *                  {@code round(((max − min)·randomMul + min)·finiteAmountBase)} (also for an
     *                  infinite one, to show the scale); 0 without a vein
     * @param remaining what is left ({@code OreData.getResourcesRemaining}: 0 infinite, −1 exhausted)
     * @param infinite  whether COE treats the vein as infinite (the recipe's {@code finite} and
     *                  COE's {@code defaultInfinite})
     */
    record ChunkState(Optional<ResourceLocation> vein, float randomMul, long amount, long remaining, boolean infinite) {}

    /**
     * One vein recipe.
     *
     * @param id        recipe id
     * @param min       {@code amountMultiplierMin}
     * @param max       {@code amountMultiplierMax}
     * @param finite    {@code always}, {@code default} or {@code never}
     * @param spacing   the placement grid's spacing in chunks
     */
    record VeinInfo(ResourceLocation id, float min, float max, String finite, int spacing) {}

    /** COE's stored state of a (loaded or generated) chunk, populating it on first access like the drills do. */
    ChunkState chunkState(LevelChunk chunk);

    /** A fresh, unattached {@code OreData} populated from {@code chunk} (COE's {@code populate}, hooks included). */
    ChunkState populate(LevelChunk chunk);

    /** {@code OreVeinGenerator.pick(chunk)}. */
    Optional<ResourceLocation> pick(LevelChunk chunk);

    /** Every vein recipe of the level's recipe manager, by id. */
    List<VeinInfo> veins(ServerLevel level);

    /** The chunk vein {@code id}'s placement grid picks in the grid cell of chunk (cx, cz). */
    Optional<ChunkPos> gridChunk(ServerLevel level, ResourceLocation id, int cx, int cz);

    /** COE's {@code locate} (the vein finder's far search and {@code /coe locate}): the nearest vein {@code id} within {@code radius} grid rings. */
    Optional<BlockPos> locate(ServerLevel level, ResourceLocation id, BlockPos from, int radius);

    /** COE's server config: {@code finiteAmountBase}, {@code defaultInfinite}. */
    String config();

    /** The access when COE is installed and answers; empty otherwise. */
    static Optional<CoeAccess> load() {
        return Holder.ACCESS;
    }

    /** True when COE is installed. */
    static boolean installed() {
        return ModList.get() != null && ModList.get().isLoaded(CoeBridge.MOD_ID);
    }

    final class Holder {
        private Holder() {}

        static final Optional<CoeAccess> ACCESS = create();

        private static Optional<CoeAccess> create() {
            if (!installed())
                return Optional.empty();
            try {
                Class<?> impl = Class.forName("com.minecraftindustrialcomplex.deeptime.compat.coe.CoeAccessImpl");
                return Optional.of((CoeAccess) impl.getDeclaredConstructor().newInstance());
            } catch (Throwable t) {
                DeepTime.LOGGER.error("Create Ore Excavation is installed but Deep Time cannot reach it", t);
                return Optional.empty();
            }
        }
    }
}
