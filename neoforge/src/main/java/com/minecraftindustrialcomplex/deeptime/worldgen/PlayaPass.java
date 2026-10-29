// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Playa;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.level.WorldGenRegion;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.BlockGetter;
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
 * <p>No feature lays soil or a tree on the salt or replaces it afterwards ({@link #refusesOnSalt}, ledger HYD-180).
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
     * Whether a feature's write of {@code next} at {@code pos} is refused on a salt flat in a Deep Time level, where
     * the playa layer holds salt (ledger HYD-180): soil ({@code #minecraft:dirt}: grass, dirt, podzol, moss, mud, ...)
     * over the salt; a log ({@code #minecraft:logs}) set on the salt; or another solid block in place of the salt's top
     * block (the one under air or plants). A salt crust grows no soil and no tree, and no feature's patch of its own
     * rock covers it. Expanded Ecosphere's {@code cliff_grass} turns any calcite under air, halite's vanilla stand-in,
     * into grass; on the test planet of master 649baf54 Biomes O' Plenty's redwoods set their trunks' wood on a playa
     * by a spring and Create Metallurgy's slag replaced its salt, the flat lying under a forest's biome. Expanded
     * Ecosphere's own salt (white concrete powder) may still replace it: salt over salt. Asked by every
     * {@code WorldGenRegion.setBlock} ({@code mixin.WorldGenRegionSaltMixin}); a write elsewhere costs a tag test or
     * a block read before the playa layer is sampled. Deep Time's own passes write through the chunk, not here.
     */
    public static boolean refusesOnSalt(WorldGenRegion region, BlockPos pos, BlockState next) {
        boolean soil = next.is(BlockTags.DIRT);
        boolean log = !soil && next.is(BlockTags.LOGS);
        boolean solid = !soil && !log && !next.isAir() && next.getFluidState().isEmpty()
                && !next.is(Blocks.WHITE_CONCRETE_POWDER) && !next.is(Blocks.WHITE_CONCRETE);
        if (!soil && !log && !solid)
            return false;
        if (!(region.getLevel().getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)
                || !(g.terrain() instanceof WorldDataTerrain w) || !w.data().has(Playa.LAYER))
            return false;
        BlockState salt = salt(w.data());
        if (salt == null || next == salt)
            return false;
        BlockPos on = log ? pos.below() : pos;
        if (!region.hasChunk(on.getX() >> 4, on.getZ() >> 4) || region.getBlockState(on) != salt)
            return false;
        if (solid) {
            BlockPos up = pos.above();
            if (!region.hasChunk(up.getX() >> 4, up.getZ() >> 4) || region.getBlockState(up).isSolid())
                return false;
        }
        double v = w.data().layer(Playa.LAYER).nearest(pos.getX(), pos.getZ());
        return !Double.isNaN(v) && Math.round(v) > 0;
    }

    /**
     * The salt crust of chunk (cx, cz), for the deposit feature (ledger HYD-181): an ore body of the bedrock
     * does not reach into a salt flat's salt, which dried out of the basin's water on top of that bedrock
     * (the younger bed lies on the older), so no deposit turns it into ore. On master 3ce1a373's test planet a
     * coal body under a playa turned the top salt block of 7 columns into coal ore. Null where the chunk has
     * no salt.
     */
    public static Crust crust(WorldData wd, int cx, int cz) {
        if (!wd.has(Playa.LAYER))
            return null;
        RasterSampler layer = wd.layer(Playa.LAYER);
        int x0 = cx << 4, z0 = cz << 4;
        if (!anySalt(layer, x0, z0, x0 + 15, z0 + 15))
            return null;
        BlockState salt = salt(wd);
        return salt == null ? null : new Crust(layer, salt);
    }

    /** {@link #crust}'s answer: the playa layer and the salt's block. */
    public record Crust(RasterSampler layer, BlockState salt) {
        /**
         * Whether the block at {@code pos}, {@code here}, is the salt this pass laid: the salt's block within the
         * top n blocks of the column's ground, n the salt the column's nearest cell holds, counted down from the
         * first block under air or fluid as {@link #apply} lays it.
         */
        public boolean holds(BlockGetter level, BlockPos pos, BlockState here) {
            if (here != salt)
                return false;
            double v = layer.nearest(pos.getX(), pos.getZ());
            int n = Double.isNaN(v) ? 0 : (int) Math.round(v);
            BlockPos.MutableBlockPos up = pos.mutable();
            for (int i = 1; i <= n; i++) {
                BlockState above = level.getBlockState(up.move(Direction.UP));
                if (above.isAir() || !above.getFluidState().isEmpty())
                    return true;
                if (above != salt)
                    return false;
            }
            return false;
        }
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
