// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.ore.OreBlockResolver;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.mojang.serialization.Codec;
import net.minecraft.core.BlockPos;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.levelgen.feature.Feature;
import net.minecraft.world.level.levelgen.feature.FeaturePlaceContext;
import net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration;

import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The deposit feature (plan §7.6; {@code deeptime:deposits}, step {@code underground_ores}): for
 * every body whose AABB touches the chunk, each host-rock block inside the body's SDF becomes ore
 * with the probability its zones give. The draw is a hash of the block position and the body id
 * ({@link #unit}), so regenerating a chunk places exactly the same blocks; zones that overlap stack
 * their densities in zone order (a draw below the first zone's density takes its material, below the
 * sum of the first two the second's, …, capped at 1); of several bodies at one block the lowest id
 * places first.
 *
 * <p>Host rock is what the strata pass leaves as rock: stone, deepslate and every rock class's
 * blocks ({@link RockPalette#host}); unconsolidated rock (sand, gravel, soil, …) hosts ore only in a
 * body whose own class is unconsolidated (a placer's gravel). The ore block is
 * {@link OreBlockResolver}'s for the host. Only blocks of the chunk being decorated are written.
 * Does nothing unless the chunk's generator is Deep Time's and reads world data.
 */
public final class DepositFeature extends Feature<NoneFeatureConfiguration> {

    public DepositFeature(Codec<NoneFeatureConfiguration> codec) {
        super(codec);
    }

    private static final Map<String, ResourceLocation> MATERIALS = new ConcurrentHashMap<>();

    static ResourceLocation material(String id) {
        return MATERIALS.computeIfAbsent(id, ResourceLocation::parse);
    }

    /** The world data of a Deep Time generator, or null (another generator, or a test terrain). */
    static WorldData worldData(ChunkGenerator generator) {
        if (generator instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)
            return w.data();
        return null;
    }

    @Override
    public boolean place(FeaturePlaceContext<NoneFeatureConfiguration> ctx) {
        WorldData wd = worldData(ctx.chunkGenerator());
        if (wd == null)
            return false;
        WorldGenLevel level = ctx.level();
        int cx = ctx.origin().getX() >> 4, cz = ctx.origin().getZ() >> 4;
        return placeInChunk(level, wd, cx, cz) > 0;
    }

    /** Places the deposits of chunk (cx, cz); returns the number of ore blocks written. */
    public static int placeInChunk(WorldGenLevel level, WorldData wd, int cx, int cz) {
        int minY = level.getMinBuildHeight(), maxY = level.getMaxBuildHeight() - 1;
        List<Body> bodies = wd.deposits().bodiesInChunk(cx, cz, minY, maxY);
        if (bodies.isEmpty())
            return 0;
        int c = wd.circumference();
        RockPalette palette = RockPalette.get();
        OreBlockResolver ores = OreBlockResolver.get();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int x0 = cx << 4, z0 = cz << 4, placed = 0;
        for (Body b : bodies) {
            if (!placesOre(b))
                continue;
            boolean loose = b.rockClass() != null && palette.isLoose(b.rockClass());
            int yLo = Math.max(minY, b.min()[1]), yHi = Math.min(maxY, b.max()[1]);
            for (int lx = 0; lx < 16; lx++) {
                for (int lz = 0; lz < 16; lz++) {
                    int x = x0 + lx, z = z0 + lz;
                    if (yLo > yHi || !b.aabbContains(c, x, yLo, z))
                        continue;
                    for (int y = yLo; y <= yHi; y++) {
                        String material = materialAt(b, c, x, y, z);
                        if (material == null)
                            continue;
                        pos.set(x, y, z);
                        BlockState host = level.getBlockState(pos);
                        RockPalette.Host kind = palette.host(host);
                        if (kind == RockPalette.Host.NONE || (kind == RockPalette.Host.LOOSE && !loose))
                            continue;
                        BlockState ore = ores.resolve(material(material), host, y);
                        if (ore == null)
                            continue;
                        level.setBlock(pos, ore, 2);
                        placed++;
                    }
                }
            }
        }
        return placed;
    }

    /** True when some zone of the body can place ore. */
    public static boolean placesOre(Body b) {
        for (Body.Zone z : b.zones())
            if (z.material() != null && z.density() > 0)
                return true;
        return false;
    }

    /**
     * The material the deposit places at block (x, y, z) of body {@code b} on a planet of
     * circumference c before the host check, or null: the block's draw against the densities of the
     * zones that apply there (plan §7.6). A pure function of the position and the body.
     */
    public static String materialAt(Body b, int c, int x, int y, int z) {
        Body.Location at = b.locate(c, x, y, z);
        if (at == null)
            return null;
        double u = unit(Math.floorMod(x + c / 2, c) - c / 2, y, z, b.id());
        double acc = 0;
        for (Body.Zone zone : b.zones()) {
            if (zone.material() == null || !(zone.density() > 0) || !zone.contains(at.d(), at.v()))
                continue;
            acc += zone.density();
            if (u < acc)
                return zone.material();
        }
        return null;
    }

    /** A uniform draw in [0, 1) from a block position and a salt (SplitMix64 finaliser). */
    public static double unit(int x, int y, int z, int salt) {
        long h = mix(0x9E3779B97F4A7C15L * (x + 0x632BE59BD9B4E019L));
        h = mix(h ^ (0xC2B2AE3D27D4EB4FL * (y + 0x2545F4914F6CDD1DL)));
        h = mix(h ^ (0x165667B19E3779F9L * (z + 0x1B873593L)));
        h = mix(h ^ Integer.toUnsignedLong(salt));
        return (h >>> 11) * 0x1.0p-53;
    }

    static long mix(long z) {
        z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
        z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
        return z ^ (z >>> 31);
    }
}
