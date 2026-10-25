// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Playa;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.WorldGenRegion;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;

import java.util.List;

/**
 * Salt flats (Ben, decisions log 2026-10-01: "Playas / salt flats as a real surface in the game").
 * A basin too dry to hold a lake keeps a playa: the floor where the little water that reaches it
 * dries. The bake writes it as the layer {@code playa} (u8, blocks of salt, on the
 * {@code water_level} grid; FORMAT.md "Playas"). In every column whose nearest cell holds salt,
 * this pass replaces that many of the ground's top blocks with the block of the rock class the
 * bake names ({@link Playa#rockClass}: {@code notes.bake.water.playa.rock_class}, halite), resolved like any stratum
 * ({@link RockPalette}: the pack's rock tags, else the class's own block), so the terrain's height
 * does not change.
 *
 * <p>Runs in {@link GeologicChunkGenerator#buildSurface} after {@link WaterBodies} and before
 * {@link IcePass}. Columns whose surface is fluid, and air, fluid and bedrock blocks, are left as
 * they are. Only the column's own blocks are read.
 *
 * <p>No feature lays soil over the salt afterwards ({@link #refusesSoil}, ledger HYD-180).
 */
public final class PlayaPass {

    private PlayaPass() {}

    /** Whether any cell of {@code layer} in (or next to) block box [x0, x1] × [z0, z1] holds salt. */
    static boolean anySalt(RasterSampler layer, int x0, int z0, int x1, int z1) {
        return layer.anyInBox(x0, z0, x1, z1, v -> v > 0);
    }

    /** Lays the chunk's salt; returns the number of blocks replaced. */
    public static int apply(ChunkAccess chunk, WorldData wd) {
        if (!wd.has(Playa.LAYER))
            return 0;
        RasterSampler layer = wd.layer(Playa.LAYER);
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        if (!anySalt(layer, x0, z0, x0 + 15, z0 + 15))
            return 0;
        BlockState salt = salt(wd);
        if (salt == null)
            return 0;
        int minY = chunk.getMinBuildHeight();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int placed = 0;
        for (int lx = 0; lx < 16; lx++) {
            for (int lz = 0; lz < 16; lz++) {
                int x = x0 + lx, z = z0 + lz;
                double v = layer.nearest(x, z);
                int n = Double.isNaN(v) ? 0 : (int) Math.round(v);
                if (n <= 0)
                    continue;
                int top = IcePass.surfaceY(chunk, x, z, minY);
                if (top < minY)
                    continue;   // a fluid or empty column
                for (int i = 0; i < n && top - i >= minY; i++) {
                    BlockState here = chunk.getBlockState(pos.set(x, top - i, z));
                    if (here.isAir() || !here.getFluidState().isEmpty() || here.is(Blocks.BEDROCK))
                        break;
                    if (here != salt) {
                        chunk.setBlockState(pos, salt, false);
                        placed++;
                    }
                }
            }
        }
        return placed;
    }

    /**
     * Whether a feature's write of {@code next} at {@code pos} is refused: soil ({@code #minecraft:dirt}:
     * grass, dirt, podzol, moss, mud, ...) over a salt flat's salt in a Deep Time level (ledger HYD-180). A salt
     * crust grows no soil; Expanded Ecosphere's {@code cliff_grass} turns any calcite under air, halite's vanilla
     * stand-in, into grass. Asked by every {@code WorldGenRegion.setBlock} ({@code mixin.WorldGenRegionSaltMixin});
     * a write that is no soil costs one tag test, and the plants a patch would set on that soil then find none.
     */
    public static boolean refusesSoil(WorldGenRegion region, BlockPos pos, BlockState next) {
        if (!next.is(BlockTags.DIRT))
            return false;
        if (!(region.getLevel().getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)
                || !(g.terrain() instanceof WorldDataTerrain w) || !w.data().has(Playa.LAYER))
            return false;
        BlockState salt = salt(w.data());
        if (salt == null || !region.hasChunk(pos.getX() >> 4, pos.getZ() >> 4) || region.getBlockState(pos) != salt)
            return false;
        double v = w.data().layer(Playa.LAYER).nearest(pos.getX(), pos.getZ());
        return !Double.isNaN(v) && Math.round(v) > 0;
    }

    private record Salt(WorldData wd, RockPalette palette, BlockState state) {}

    private static volatile Salt saltCache;

    /** The salt's block (the bake's rock class through the palette), kept per world data and palette. */
    static BlockState salt(WorldData wd) {
        RockPalette palette = RockPalette.get();
        Salt s = saltCache;
        if (s == null || s.wd() != wd || s.palette() != palette) {
            PaletteRules.Result<BlockState>[] found = palette.forClasses(List.of(Playa.rockClass(wd.manifest().notes())));
            s = new Salt(wd, palette, found.length == 0 || found[0] == null ? null : found[0].pick(0L));
            saltCache = s;
        }
        return s.state();
    }
}
