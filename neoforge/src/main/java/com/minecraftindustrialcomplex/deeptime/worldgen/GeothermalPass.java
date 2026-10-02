// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeothermalPalette.Role;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.CarvingMask;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.levelgen.GenerationStep;
import net.minecraft.world.level.levelgen.Heightmap;

import java.util.List;
import java.util.concurrent.atomic.LongAdder;

/**
 * Places the geothermal features of the world's active volcanoes in a chunk ({@link Geothermal}
 * decides what every column is; P2-14). Runs in the surface step after the {@link MagmaPass} and
 * before the ice, carvers and features.
 *
 * <ul>
 *   <li><b>Pools.</b> Water at the site's {@code level} over a bed (sinter or travertine in the
 *       middle, yellow then orange mats towards the edge), the ground above the level cut away; the
 *       rim ring is solid on the level and one block under it, so every side of the water is a
 *       block. Bed, water and rim go into the chunk's carving masks, which vanilla's carvers skip
 *       (as {@link MagmaPass} does for magma): a cave may run under a pool, never through it.</li>
 *   <li><b>Terraces and altered ground.</b> The top one or two ground blocks of a column become the
 *       terrace's sinter or travertine, a vent's native sulfur and magma block, or bleached and
 *       stained altered ground (cinnabar stains at silicic volcanoes). Columns with a fluid on top
 *       are left as they are.</li>
 * </ul>
 *
 * <p>Everything is decided from the column's position and the level's {@link Geothermal}; only the
 * chunk's own blocks are read and written, so chunks generate in any order on any thread. Where two
 * sites claim a column the higher {@link Geothermal.Part#priority} wins, so a pool's rim is never
 * overwritten by a neighbour's terrace. No fluid tick is scheduled: the water rests in its basin.
 */
public final class GeothermalPass {

    private GeothermalPass() {}

    private static final BlockState WATER = Blocks.WATER.defaultBlockState();
    private static final BlockState AIR = Blocks.AIR.defaultBlockState();

    private static final LongAdder POOL_COLUMNS = new LongAdder(), VENT_COLUMNS = new LongAdder(), GROUND_COLUMNS = new LongAdder(),
            CHUNKS = new LongAdder();

    /** Pool columns, vent columns, other columns placed and chunks touched since the JVM started (logs, tests). */
    public static long[] totals() {
        return new long[]{POOL_COLUMNS.sum(), VENT_COLUMNS.sum(), GROUND_COLUMNS.sum(), CHUNKS.sum()};
    }

    /** Whether a ground block may be repainted: solid terrain, not a fluid, lava, magma or bedrock. */
    static boolean paintable(BlockState s) {
        return !s.isAir() && s.getFluidState().isEmpty() && !s.is(Blocks.BEDROCK) && !s.is(Blocks.MAGMA_BLOCK);
    }

    private static long key(Geothermal.Site s, int dx, int dz, int salt) {
        return PaletteRules.mix(s.seed() ^ ((long) dx << 32) ^ (dz & 0xFFFFFFFFL) ^ ((long) salt << 56));
    }

    /** Applies the pass; returns the number of columns it changed. */
    public static int apply(ChunkAccess chunk, Geothermal geothermal) {
        int x0 = chunk.getPos().getMinBlockX(), z0 = chunk.getPos().getMinBlockZ();
        List<Geothermal.Site> sites = geothermal.near(x0, z0, x0 + 15, z0 + 15);
        if (sites.isEmpty())
            return 0;
        GeothermalPalette palette = GeothermalPalette.get();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        CarvingMask[] masks = chunk instanceof ProtoChunk proto
                ? new CarvingMask[]{proto.getOrCreateCarvingMask(GenerationStep.Carving.AIR),
                        proto.getOrCreateCarvingMask(GenerationStep.Carving.LIQUID)}
                : new CarvingMask[0];
        int minY = chunk.getMinBuildHeight(), maxY = chunk.getMaxBuildHeight() - 1;
        int changed = 0;
        long pools = 0, vents = 0, ground = 0;
        for (int lz = 0; lz < 16; lz++)
            for (int lx = 0; lx < 16; lx++) {
                int x = x0 + lx, z = z0 + lz;
                Geothermal.Site site = null;
                Geothermal.Part part = Geothermal.Part.NONE;
                int sdx = 0, sdz = 0;
                for (Geothermal.Site s : sites) {
                    int dx = geothermal.dx(x, s), dz = z - s.z();
                    if (Math.abs(dx) > s.reach() || Math.abs(dz) > s.reach())
                        continue;
                    Geothermal.Part p = s.part(dx, dz);
                    if (p.priority > part.priority) {
                        site = s;
                        part = p;
                        sdx = dx;
                        sdz = dz;
                    }
                }
                if (site == null)
                    continue;
                int top = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, lx, lz) - 1;      // the highest solid block
                int surface = chunk.getHeight(Heightmap.Types.WORLD_SURFACE_WG, lx, lz) - 1;  // the highest block that is not air
                if (part.pool() || part == Geothermal.Part.RIM) {
                    placePool(chunk, masks, pos, palette, site, part, sdx, sdz, lx, lz, x, z, top, minY, maxY);
                    pools++;
                } else {
                    if (surface > top || top < minY || !paintable(chunk.getBlockState(pos.set(x, top, z))))
                        continue;   // water or something else stands on the ground here
                    placeGround(chunk, pos, palette, site, part, sdx, sdz, x, z, top, minY);
                    if (part == Geothermal.Part.VENT)
                        vents++;
                    else
                        ground++;
                }
                changed++;
            }
        if (changed > 0) {
            POOL_COLUMNS.add(pools);
            VENT_COLUMNS.add(vents);
            GROUND_COLUMNS.add(ground);
            CHUNKS.increment();
        }
        return changed;
    }

    private static void placePool(ChunkAccess chunk, CarvingMask[] masks, BlockPos.MutableBlockPos pos, GeothermalPalette palette,
                                  Geothermal.Site site, Geothermal.Part part, int dx, int dz, int lx, int lz, int x, int z, int top,
                                  int minY, int maxY) {
        int level = site.level();
        if (level < minY + 2 || level + Geothermal.CUT + 6 > maxY)
            return;
        BlockState flavour = palette.block(site.sinter() ? Role.SINTER : Role.TRAVERTINE, key(site, dx, dz, 5));
        if (part == Geothermal.Part.RIM) {
            // Solid on the level and under it: nothing in the pool can run out; down to the ground if the ground is lower.
            BlockState rim = palette.block(site.sinter() ? Role.SINTER : Role.TRAVERTINE, key(site, dx, dz, 6));
            for (int y = level; y >= level - 1; y--)
                set(chunk, masks, pos, lx, y, lz, x, z, rim);
            support(chunk, masks, pos, lx, lz, x, z, level - 2, minY, rim);
            // The ground above the level wears the terrace too.
            if (top > level && top <= level + Geothermal.CUT + 2 && paintable(chunk.getBlockState(pos.set(x, top, z))))
                chunk.setBlockState(pos.set(x, top, z), rim, false);
            return;
        }
        int depth = part.depth();
        for (int y = level + 1; y <= level + Geothermal.CUT + 6; y++) {
            BlockState s = chunk.getBlockState(pos.set(x, y, z));
            if (!s.isAir() && !s.is(Blocks.LAVA))
                chunk.setBlockState(pos, AIR, false);
        }
        for (int y = level; y > level - depth; y--)
            set(chunk, masks, pos, lx, y, lz, x, z, WATER);
        int bedY = level - depth;
        BlockState bed = switch (part) {
            case POOL_CORE -> flavour;
            case POOL_MAT -> palette.block(Role.STAIN, key(site, dx, dz, 7));
            default -> palette.block(Role.MAT, key(site, dx, dz, 8));
        };
        set(chunk, masks, pos, lx, bedY, lz, x, z, bed);
        support(chunk, masks, pos, lx, lz, x, z, bedY - 1, minY, bed);
    }

    /** Fills the air or fluid under a block downwards (at most the dam and a little more) so a bed or rim does not float. */
    private static void support(ChunkAccess chunk, CarvingMask[] masks, BlockPos.MutableBlockPos pos, int lx, int lz, int x, int z,
                                int from, int minY, BlockState fill) {
        for (int y = from; y >= Math.max(minY, from - (Geothermal.DAM + 3)); y--) {
            BlockState s = chunk.getBlockState(pos.set(x, y, z));
            if (!s.isAir() && s.getFluidState().isEmpty())
                break;
            set(chunk, masks, pos, lx, y, lz, x, z, fill);
        }
    }

    private static void set(ChunkAccess chunk, CarvingMask[] masks, BlockPos.MutableBlockPos pos, int lx, int y, int lz, int x, int z,
                            BlockState state) {
        chunk.setBlockState(pos.set(x, y, z), state, false);
        for (CarvingMask m : masks)
            m.set(lx, y, lz);
    }

    private static void placeGround(ChunkAccess chunk, BlockPos.MutableBlockPos pos, GeothermalPalette palette, Geothermal.Site site,
                                    Geothermal.Part part, int dx, int dz, int x, int z, int top, int minY) {
        Role terrace = site.sinter() ? Role.SINTER : Role.TRAVERTINE;
        switch (part) {
            case VENT -> {
                BlockState vent = palette.block(Role.VENT, key(site, dx, dz, 9));
                chunk.setBlockState(pos.set(x, top, z), vent, false);
                if (top - 1 >= minY && paintable(chunk.getBlockState(pos.set(x, top - 1, z))))
                    chunk.setBlockState(pos, vent, false);
            }
            case SULFUR -> chunk.setBlockState(pos.set(x, top, z), palette.block(Role.SULFUR, key(site, dx, dz, 10)), false);
            case APRON -> {
                // Mostly the site's own terrace rock, a fifth of the other (travertine bands in sinter and back).
                Role role = Geothermal.unit(site.seed(), dx, dz, 11) < 0.2 ? (site.sinter() ? Role.TRAVERTINE : Role.SINTER) : terrace;
                BlockState b = palette.block(role, key(site, dx, dz, 12));
                chunk.setBlockState(pos.set(x, top, z), b, false);
                if (Geothermal.unit(site.seed(), dx, dz, 13) < 0.4 && top - 1 >= minY && paintable(chunk.getBlockState(pos.set(x, top - 1, z))))
                    chunk.setBlockState(pos, b, false);
            }
            case APRON_STAIN, STAIN -> chunk.setBlockState(pos.set(x, top, z), palette.block(Role.STAIN, key(site, dx, dz, 14)), false);
            case CINNABAR -> chunk.setBlockState(pos.set(x, top, z), palette.block(Role.CINNABAR, key(site, dx, dz, 15)), false);
            case ALTERED -> {
                BlockState b = palette.block(Role.ALTERED, key(site, dx, dz, 16));
                chunk.setBlockState(pos.set(x, top, z), b, false);
                if (Geothermal.unit(site.seed(), dx, dz, 17) < 0.5 && top - 1 >= minY && paintable(chunk.getBlockState(pos.set(x, top - 1, z))))
                    chunk.setBlockState(pos, b, false);
            }
            default -> { }
        }
    }
}
