// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSource;
import com.minecraftindustrialcomplex.deeptime.worldgen.RegionSpawnTarget;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import com.mojang.datafixers.util.Pair;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;
import net.minecraft.world.level.biome.MultiNoiseBiomeSource;
import terrablender.api.Region;
import terrablender.api.RegionType;
import terrablender.api.Regions;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

/**
 * The TerraBlender half of {@code TerraBlenderGameTests.regionsFollowProvinces}: it names
 * TerraBlender's classes, so the holder calls it only when TerraBlender is installed.
 */
final class TerraBlenderChecks {

    private TerraBlenderChecks() {}

    /** What one run over a level found. */
    record Result(int samples, int provincesSeen, int regionsSeen, int outsideItsRegion, int perProvinceMismatch,
                  int landSamplesInModRegions, Map<String, Integer> landByNamespace, Map<Integer, Integer> provincesByRegion,
                  String firstProblem, int chunkSamples, int chunkOutside, int chunkModBiomes) {}

    /**
     * Samples a {@code cols × rows} grid of surface columns. For each: the province, the region its
     * table gives it, and the biome the level's biome source places there; the biome must be one the
     * region (or, for TerraBlender's deferred placeholder and its region-less trees, vanilla's
     * region 0) offers. Also per province: one region, equal to the one named in the world's table.
     */
    static Result run(ServerLevel level, int cols, int rows) {
        var g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        var t = (WorldDataTerrain) g.terrain();
        var sampler = level.getChunkSource().randomState().sampler();
        RegionSource src = ((RegionSpawnTarget) sampler.spawnTarget()).source().orElseThrow();
        MultiNoiseBiomeSource biomeSource = (MultiNoiseBiomeSource) g.getBiomeSource();
        Registry<Biome> biomes = level.registryAccess().registryOrThrow(Registries.BIOME);

        Map<Integer, Set<ResourceKey<Biome>>> offered = new HashMap<>();
        Set<ResourceKey<Biome>> vanilla = new HashSet<>();
        for (Pair<Climate.ParameterPoint, Holder<Biome>> p : biomeSource.parameters().values())
            p.getSecond().unwrapKey().ifPresent(vanilla::add);
        offered.put(0, vanilla);
        for (Region r : Regions.get(RegionType.OVERWORLD)) {
            int idx = Regions.getIndex(RegionType.OVERWORLD, r.getName());
            if (idx == 0)
                continue;
            Set<ResourceKey<Biome>> keys = new HashSet<>();
            r.addBiomes(biomes, pair -> keys.add(pair.getSecond()));
            offered.put(idx, keys);
        }

        Map<Integer, Integer> regionOfProvince = new HashMap<>();
        Map<String, Integer> byNamespace = new TreeMap<>();
        Map<Integer, Integer> provincesByRegion = new TreeMap<>();
        int samples = 0, outside = 0, mismatch = 0, land = 0;
        String first = null;
        int c = t.circumference();
        for (int j = 0; j < rows; j++) {
            for (int i = 0; i < cols; i++) {
                int x = -c / 2 + (int) ((i + 0.5) * c / cols);
                int z = -c / 4 + (int) ((j + 0.5) * (c / 2.0) / rows);
                int province = src.provinceAt(x, z);
                int region = src.regionAt(x, z);
                double y = t.height(x, z);
                Holder<Biome> b = biomeSource.getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock((int) Math.floor(y)),
                        QuartPos.fromBlock(z), sampler);
                samples++;
                Integer seen = regionOfProvince.putIfAbsent(province, region);
                if (seen != null && seen != region) {
                    mismatch++;
                    if (first == null)
                        first = "province " + province + " has regions " + seen + " and " + region + " at (" + x + "," + z + ")";
                }
                var key = b.unwrapKey().orElseThrow();
                Set<ResourceKey<Biome>> ok = offered.getOrDefault(region, vanilla);
                if (!ok.contains(key) && !vanilla.contains(key)) {
                    outside++;
                    if (first == null)
                        first = "biome " + key.location() + " at (" + x + "," + z + ") is not offered by region " + region
                                + " of province " + province;
                }
                if (y > g.getSeaLevel() + 1) {
                    byNamespace.merge(key.location().getNamespace(), 1, Integer::sum);
                    if (region != 0)
                        land++;
                }
            }
        }
        regionOfProvince.values().forEach(r -> provincesByRegion.merge(r, 1, Integer::sum));
        // The world's table, read back from the file's names: every province's region index is the
        // one TerraBlender has for the name the table holds.
        var table = t.mapping().regions();
        for (var e : regionOfProvince.entrySet()) {
            String name = table == null ? null : table.regionOf(e.getKey());
            int want = name == null ? 0 : Regions.getIndex(RegionType.OVERWORLD,
                    net.minecraft.resources.ResourceLocation.parse(name));
            if (want != e.getValue()) {
                mismatch++;
                if (first == null)
                    first = "province " + e.getKey() + " names region " + name + " (index " + want + ") but the source says " + e.getValue();
            }
        }
        // The biomes a chunk stores when it is generated (its own climate sampler, built from the registry
        // settings unless RegionChunkBiomesMixin hands it the level's) are offered by their column's region.
        int chunkSamples = 0, chunkOutside = 0, chunkMod = 0;
        int minY = level.getMinBuildHeight(), maxY = level.getMaxBuildHeight() - 1;
        for (int k = 0; k < 64; k++) {
            int x = -c / 2 + (int) ((k + 0.5) * c / 64.0);
            int z = -c / 4 + (int) ((((k * 29) % 64) + 0.5) * (c / 2.0) / 64.0);
            int y = (int) Math.max(minY, Math.min(maxY, Math.floor(t.height(x, z))));
            var chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, net.minecraft.world.level.chunk.status.ChunkStatus.BIOMES, true);
            var key = chunk.getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(y), QuartPos.fromBlock(z)).unwrapKey().orElseThrow();
            int region = src.regionAt(x, z);
            chunkSamples++;
            if (!key.location().getNamespace().equals("minecraft"))
                chunkMod++;
            if (!offered.getOrDefault(region, vanilla).contains(key) && !vanilla.contains(key)) {
                chunkOutside++;
                if (first == null)
                    first = "generated chunk biome " + key.location() + " at (" + x + "," + z + ") is not offered by region " + region;
            }
        }
        return new Result(samples, regionOfProvince.size(), provincesByRegion.size(), outside, mismatch, land, byNamespace,
                provincesByRegion, first, chunkSamples, chunkOutside, chunkMod);
    }
}
