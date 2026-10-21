// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import com.minecraftindustrialcomplex.deeptime.worlddata.Playa;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.Geothermal;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.Heightmap;

/** TEMPORARY investigation helper (thermal_calcite on a playa, TerraBlender). */
final class DIAG {
    private DIAG() {}

    static String why(ServerLevel level, GeologicChunkGenerator g, WorldDataTerrain w, WorldData wd, Water.Spec spec, int x, int y, int z) {
        StringBuilder b = new StringBuilder();
        BlockPos at = new BlockPos(x, y + 1, z);
        b.append("at=").append(x).append(',').append(y).append(',').append(z);
        b.append(" storedBiome=").append(level.getBiome(at).unwrapKey().map(k -> k.location().toString()).orElse("?"));
        var sampler = level.getChunkSource().randomState().sampler();
        var biome = g.getBiomeSource().getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y + 1), QuartPos.fromBlock(z), sampler);
        b.append(" query=").append(biome.unwrapKey().map(k -> k.location().toString()).orElse("?")).append(" why=").append(VolcanicBiomes.lastDecision());
        if (sampler.spawnTarget() instanceof RegionSpawnTarget m) {
            int bits = m.volcanic().zones().rolesAt(x, z);
            b.append(" roles=");
            for (VolcanicZones.Role r : VolcanicZones.Role.values())
                if ((bits & (1 << r.ordinal())) != 0)
                    b.append(r.id).append('+');
        }
        b.append(" playa=").append(wd.layer(Playa.LAYER).nearest(x, z));
        var settings = g.generatorSettings().value();
        Geothermal geo = w.geothermal(g.waterReach(), settings.noiseSettings().minY(), settings.seaLevel());
        Geothermal.Site best = null;
        double bd = Double.MAX_VALUE;
        for (Geothermal.Site s : geo.sites()) {
            double dx = geo.dx(x, s), dz = z - s.z();
            double d = Math.sqrt(dx * dx + dz * dz);
            if (d < bd) {
                bd = d;
                best = s;
            }
        }
        if (best != null)
            b.append(String.format(" nearestSite=%s@%d,%d d=%.1f r=%d apron=%d level=%d groundY=%d volcano=%d part=%s", best.kind(), best.x(), best.z(), bd,
                    best.r(), best.apron(), best.level(), best.groundY(), best.volcano(), geo.partAt(x, z)));
        // Water within 6 blocks, between y-2 and y+3.
        int nw = 0;
        String nearestW = "none";
        double nwd = Double.MAX_VALUE;
        BlockPos.MutableBlockPos p = new BlockPos.MutableBlockPos();
        for (int dx = -6; dx <= 6; dx++)
            for (int dz = -6; dz <= 6; dz++)
                for (int dy = -2; dy <= 3; dy++) {
                    BlockState s = level.getBlockState(p.set(x + dx, y + dy, z + dz));
                    if (s.getFluidState().is(FluidTags.WATER)) {
                        nw++;
                        double d = Math.sqrt(dx * dx + dz * dz);
                        if (d < nwd) {
                            nwd = d;
                            Water.Column col = Water.column(wd, spec, x + dx, z + dz);
                            nearestW = (x + dx) + "," + (y + dy) + "," + (z + dz) + " " + BuiltInRegistries.BLOCK.getKey(s.getBlock()) + " dtWater=" + col.kind()
                                    + " geoPart=" + geo.partAt(x + dx, z + dz);
                        }
                    }
                }
        b.append(" waterBlocks=").append(nw).append(" nearestWater=").append(nearestW);
        // The top blocks of the 13 x 13 around the column, as a map of letters.
        StringBuilder map = new StringBuilder();
        java.util.Map<String, Character> legend = new java.util.LinkedHashMap<>();
        for (int dz = -6; dz <= 6; dz++) {
            map.append(" |");
            for (int dx = -6; dx <= 6; dx++) {
                var chunk = level.getChunk((x + dx) >> 4, (z + dz) >> 4);
                int ty = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR, (x + dx) & 15, (z + dz) & 15);
                BlockState top = chunk.getBlockState(p.set(x + dx, ty, z + dz));
                BlockState above = chunk.getBlockState(p.set(x + dx, ty + 1, z + dz));
                String key = BuiltInRegistries.BLOCK.getKey(top.getBlock()) + (above.getFluidState().is(FluidTags.WATER) ? "~" : "");
                char c = legend.computeIfAbsent(key, k -> (char) ('a' + legend.size()));
                map.append(c);
            }
            map.append('|');
        }
        b.append(" map(OCEAN_FLOOR top, ~=water above):").append(map).append(" legend=").append(legend);
        StringBuilder hs = new StringBuilder();
        for (int dx = -6; dx <= 6; dx++) {
            var chunk = level.getChunk((x + dx) >> 4, z >> 4);
            hs.append(chunk.getHeight(Heightmap.Types.OCEAN_FLOOR, (x + dx) & 15, z & 15)).append(' ');
        }
        b.append(" floorRow=").append(hs);
        return b.toString();
    }
}
