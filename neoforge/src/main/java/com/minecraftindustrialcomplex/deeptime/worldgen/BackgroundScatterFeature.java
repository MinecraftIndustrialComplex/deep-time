// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.ore.OreBlockResolver;
import com.minecraftindustrialcomplex.deeptime.ore.OreData;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.mojang.serialization.Codec;
import net.minecraft.core.BlockPos;
import net.minecraft.core.RegistryAccess;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.feature.Feature;
import net.minecraft.world.level.levelgen.feature.FeaturePlaceContext;
import net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration;

import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.WeakHashMap;

/**
 * Background scatter (plan §7.8; {@code deeptime:background_scatter}, step
 * {@code underground_ores}, after the deposits): small clusters of common materials anywhere, so
 * nobody searches for hours and finds nothing (IE #2229, COE #85). The rules are the
 * {@code deeptime:scatter} registry ({@link OreData.ScatterRule}).
 *
 * <p>Per chunk and rule, the number of clusters is ⌊clusters_per_chunk⌋ plus one with the
 * fractional chance; each cluster starts at a random column of the chunk at a depth drawn from
 * {@code y_window} (a fraction of the column from the world data's surface down to the bottom of
 * the world) and random-walks {@code size} steps inside the chunk. A visited block becomes ore when
 * it is bedrock host ({@link RockPalette.Host#ROCK}; loose rock never hosts scatter) and a draw
 * passes the host's {@code host_bias}. Every draw is a hash of the chunk, the material and the
 * position ({@link #plan}), so the scatter is the same on every regeneration and a test can
 * recompute it.
 */
public final class BackgroundScatterFeature extends Feature<NoneFeatureConfiguration> {

    private static final Map<RegistryAccess, List<OreData.ScatterRule>> RULES =
            Collections.synchronizedMap(new WeakHashMap<>());

    public BackgroundScatterFeature(Codec<NoneFeatureConfiguration> codec) {
        super(codec);
    }

    /** The scatter rules of a server's registries, ordered by id. */
    public static List<OreData.ScatterRule> rules(RegistryAccess access) {
        return RULES.computeIfAbsent(access, a -> OreData.entries(a, OreData.SCATTER).stream().map(Map.Entry::getValue).toList());
    }

    @Override
    public boolean place(FeaturePlaceContext<NoneFeatureConfiguration> ctx) {
        WorldData wd = DepositFeature.worldData(ctx.chunkGenerator());
        if (wd == null)
            return false;
        WorldGenLevel level = ctx.level();
        int cx = ctx.origin().getX() >> 4, cz = ctx.origin().getZ() >> 4;
        return placeInChunk(level, wd, rules(level.registryAccess()), cx, cz) > 0;
    }

    /** Places the scatter of chunk (cx, cz); returns the number of ore blocks written. */
    public static int placeInChunk(WorldGenLevel level, WorldData wd, List<OreData.ScatterRule> rules, int cx, int cz) {
        RockPalette palette = RockPalette.get();
        OreBlockResolver ores = OreBlockResolver.get();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int placed = 0;
        for (OreData.ScatterRule rule : rules) {
            if (ores.blocks(rule.material()).isEmpty())
                continue;
            for (Candidate p : plan(wd, rule, cx, cz, level.getMinBuildHeight())) {
                pos.set(p.x(), p.y(), p.z());
                BlockState host = level.getBlockState(pos);
                if (palette.host(host) != RockPalette.Host.ROCK)
                    continue;
                if (!(p.draw() < rule.bias(palette.classesOf(host))))
                    continue;
                BlockState ore = ores.resolve(rule.material(), host, p.y());
                if (ore == null)
                    continue;
                level.setBlock(pos, ore, 2);
                placed++;
            }
        }
        return placed;
    }

    /** A block a cluster visits, with the draw its host bias is tested against. */
    public record Candidate(int x, int y, int z, double draw) {}

    /**
     * Every block the clusters of {@code rule} visit in chunk (cx, cz), in order, each once. A pure
     * function of the world data's surface, the rule and the chunk.
     */
    public static List<Candidate> plan(WorldData wd, OreData.ScatterRule rule, int cx, int cz, int minY) {
        int salt = rule.material().toString().hashCode();
        int c = wd.circumference();
        int kx = Math.floorMod(cx, c >> 4);
        double count = rule.clustersPerChunk();
        int n = (int) Math.floor(count);
        if (DepositFeature.unit(kx, cz, -1, salt) < count - n)
            n++;
        double f0 = rule.yWindow().get(0), f1 = rule.yWindow().get(1);
        LinkedHashSet<Long> seen = new LinkedHashSet<>();
        List<Candidate> out = new ArrayList<>();
        int x0 = cx << 4, z0 = cz << 4;
        for (int k = 0; k < n; k++) {
            int ks = salt ^ (k * 0x9E3779B9);
            int lx = (int) (DepositFeature.unit(kx, cz, 1, ks) * 16), lz = (int) (DepositFeature.unit(kx, cz, 2, ks) * 16);
            int surface = (int) Math.floor(wd.height(Math.floorMod(x0 + lx + c / 2, c) - c / 2, z0 + lz));
            double f = f0 + DepositFeature.unit(kx, cz, 3, ks) * (f1 - f0);
            int y = surface - (int) Math.round(f * (surface - minY));
            for (int step = 0; step < rule.size(); step++) {
                if (y >= minY && y <= surface) {
                    int x = x0 + lx, z = z0 + lz;
                    long key = ((long) (lx * 16 + lz) << 32) | (y & 0xFFFFFFFFL);
                    if (seen.add(key))
                        out.add(new Candidate(x, y, z, DepositFeature.unit(Math.floorMod(x + c / 2, c) - c / 2, y, z, ks)));
                }
                int dir = (int) (DepositFeature.unit(kx, cz, 16 + step, ks) * 6);
                switch (dir) {
                    case 0 -> lx = Math.min(15, lx + 1);
                    case 1 -> lx = Math.max(0, lx - 1);
                    case 2 -> lz = Math.min(15, lz + 1);
                    case 3 -> lz = Math.max(0, lz - 1);
                    case 4 -> y++;
                    default -> y--;
                }
            }
        }
        return out;
    }
}
