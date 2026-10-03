// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.levelgen.Heightmap;

import java.util.Optional;

/**
 * Snow and ice over glaciated ground, whatever the pack's biome source drew there (Ben, 2026-09-30,
 * "Force snow/ice cover"). Terralith has no flat inland ice biome: without this an ice cap's flats
 * come out as gravel desert or cold shrubland, and a biome tag cannot be read as ice from the
 * ground.
 *
 * <p>Runs last in {@link GeologicChunkGenerator#buildSurface}: after the pack's surface rules, the
 * {@link StrataPass} and the {@link WaterPass}. In every column the world's {@link IceCover} gives a
 * thickness for (the bake's {@code climate/ice_cover}: blocks of cover, 1 at the foot of an ice
 * field, more the higher the ground lies above its equilibrium line), the cover <b>replaces</b> the
 * top {@code n} solid blocks of the ground, so the terrain's height does not change. The blocks are
 * the table's: the top {@code snow_blocks} snow, a base of blue ice where the cover is thick, packed
 * ice between ({@link IceCover.Spec#kind}). Columns whose surface is fluid (a lake, the sea) and
 * air, bedrock and fluid blocks are left as they are, and so is the ice of a frozen lake (the water
 * pass's top block, {@link #surfaceY}); the ice pass is the only thing here that
 * reads the climate, and it reads it through {@link IceCover} alone.
 *
 * <p>Vanilla's {@code freeze_top_layer} feature, which runs later in every cold biome, then lays its
 * snow layer on the snow block as it would on stone.
 */
public final class IcePass {

    private static volatile boolean warned;

    private IcePass() {}

    /** The blocks of a cover spec, resolved against the registry (an unknown id falls back to the default). */
    record CoverBlocks(BlockState snow, BlockState ice, BlockState deepIce) {

        static CoverBlocks of(IceCover.Spec spec) {
            return new CoverBlocks(block(spec.snow(), Blocks.SNOW_BLOCK), block(spec.ice(), Blocks.PACKED_ICE),
                    block(spec.deepIce(), Blocks.BLUE_ICE));
        }

        private static BlockState block(String id, Block fallback) {
            ResourceLocation rl = ResourceLocation.tryParse(id);
            Optional<Block> b = rl == null ? Optional.empty() : BuiltInRegistries.BLOCK.getOptional(rl);
            if (b.isEmpty() && !warned) {
                warned = true;
                DeepTime.LOGGER.error("ice_cover names the block '{}', which does not exist; using {}", id,
                        BuiltInRegistries.BLOCK.getKey(fallback));
            }
            return b.orElse(fallback).defaultBlockState();
        }

        BlockState of(int kind) {
            return switch (kind) {
                case IceCover.Spec.SNOW -> snow;
                case IceCover.Spec.DEEP_ICE -> deepIce;
                default -> ice;
            };
        }
    }

    /** Lays the chunk's ice cover; returns the number of blocks replaced. */
    public static int apply(ChunkAccess chunk, WorldData wd) {
        return wd.iceCover().map(cover -> apply(chunk, cover)).orElse(0);
    }

    /** As {@link #apply(ChunkAccess, WorldData)}, for any source of cover thickness. */
    public static int apply(ChunkAccess chunk, IceCover ice) {
        CoverBlocks blocks = CoverBlocks.of(ice.spec());
        int minY = chunk.getMinBuildHeight();
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int placed = 0;
        for (int lx = 0; lx < 16; lx++) {
            for (int lz = 0; lz < 16; lz++) {
                int x = x0 + lx, z = z0 + lz;
                int n = ice.blocks(x, z);
                if (n <= 0)
                    continue;
                int top = surfaceY(chunk, x, z, minY);
                if (top < minY)
                    continue;   // a fluid or empty column
                for (int i = 0; i < n && top - i >= minY; i++) {
                    BlockState here = chunk.getBlockState(pos.set(x, top - i, z));
                    if (here.isAir() || !here.getFluidState().isEmpty() || here.is(Blocks.BEDROCK))
                        break;
                    BlockState block = blocks.of(ice.spec().kind(i, n));
                    if (here != block) {
                        chunk.setBlockState(pos, block, false);
                        placed++;
                    }
                }
            }
        }
        return placed;
    }

    /**
     * The y of the highest block of the column that is ground: the worldgen surface heightmap's,
     * walked down over air; {@code minY - 1} when that block is a fluid (a lake or the sea covers
     * the ground there), the ice of a frozen lake (the water pass's {@code minecraft:ice} top: it is
     * the lake's surface, not ground, and the cover must not replace it with snow, which left a
     * frozen lake a block of snow over its dirt bed and no water: "dry shore" in 96 of 898 water
     * columns of a test planet) or the column is empty.
     */
    static int surfaceY(ChunkAccess chunk, int x, int z, int minY) {
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int y = chunk.getHeight(Heightmap.Types.WORLD_SURFACE_WG, x, z);
        for (; y >= minY; y--) {
            BlockState s = chunk.getBlockState(pos.set(x, y, z));
            if (s.isAir())
                continue;
            return s.getFluidState().isEmpty() && !s.is(Blocks.ICE) ? y : minY - 1;
        }
        return minY - 1;
    }
}
