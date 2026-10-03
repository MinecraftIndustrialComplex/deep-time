// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import com.minecraftindustrialcomplex.deeptime.compat.terrablender.RegionCatalog;
import net.minecraft.world.level.biome.Climate;

import java.util.AbstractList;
import java.util.List;
import java.util.Optional;
import java.util.RandomAccess;
import java.util.concurrent.atomic.AtomicReference;

/**
 * The noise settings' spawn-target list, carrying the level's {@link RegionSource} and its volcanic
 * biome state ({@link VolcanicBiomes.Level}).
 *
 * <p>TerraBlender's regions are chosen where a {@code MultiNoiseBiomeSource} answers a biome
 * query, and the only things that query is given are the position and the level's
 * {@code Climate.Sampler}, which {@code RandomState} builds from the level's noise settings. Those
 * settings are per generator (see {@link GeologicChunkGenerator}) and pass their {@code spawnTarget}
 * list to the sampler unchanged, so a list that wraps the real one and remembers the level's
 * regions is how the biome source tells a Deep Time level's sampler from any other's, and finds
 * that level's province raster, without a global lookup by level.
 *
 * <p>Every level on world data is wrapped, with or without TerraBlender: the volcanic biomes (decisions
 * log 2026-10-03) are placed by {@code mixin.VolcanicBiomeSourceMixin} on any multi-noise biome source,
 * and that mixin finds a Deep Time level by this list too.
 */
public final class RegionSpawnTarget extends AbstractList<Climate.ParameterPoint> implements RandomAccess {

    private final List<Climate.ParameterPoint> delegate;
    private final WorldDataTerrain terrain;
    private final VolcanicBiomes.Level volcanic;
    private final AtomicReference<Optional<RegionSource>> source = new AtomicReference<>();

    private RegionSpawnTarget(List<Climate.ParameterPoint> delegate, WorldDataTerrain terrain, VolcanicBiomes.Level volcanic) {
        this.delegate = delegate;
        this.terrain = terrain;
        this.volcanic = volcanic;
    }

    /**
     * {@code spawnTarget}, wrapped when the level is on world data; otherwise as it is.
     *
     * @param cell     the noise cell width, {@code minY} the floor, {@code seaLevel} the sea level of the
     *                 level's settings (the volcanic zones are built from the same magma and geothermal sites
     *                 as the surface step's)
     */
    public static List<Climate.ParameterPoint> wrap(List<Climate.ParameterPoint> spawnTarget, TerrainSource terrain, int cell, int minY,
                                                    int seaLevel) {
        if (!(terrain instanceof WorldDataTerrain t))
            return spawnTarget;
        return new RegionSpawnTarget(spawnTarget, t, new VolcanicBiomes.Level(t, cell, minY, seaLevel));
    }

    /** The level's volcanic biome state. */
    public VolcanicBiomes.Level volcanic() {
        return volcanic;
    }

    /**
     * The level's regions, resolved on first use (the world data and TerraBlender's regions exist
     * by the first biome query, not when the settings are bound); empty when the world has no
     * province → region table.
     */
    public Optional<RegionSource> source() {
        Optional<RegionSource> s = source.get();
        if (s == null && !net.neoforged.fml.ModList.get().isLoaded(RegionCatalog.MOD_ID)) {
            source.compareAndSet(null, Optional.empty());
            return Optional.empty();
        }
        if (s == null) {
            s = RegionSource.of(terrain, RegionCatalog.load());
            if (!source.compareAndSet(null, s))
                s = source.get();
            else
                com.minecraftindustrialcomplex.deeptime.DeepTime.LOGGER.info("Deep Time: TerraBlender regions {} for this level",
                        s.map(r -> "follow its " + r.provinceCount() + " geologic provinces").orElse("are TerraBlender's own (no province table)"));
        }
        return s;
    }

    @Override
    public Climate.ParameterPoint get(int index) {
        return delegate.get(index);
    }

    @Override
    public int size() {
        return delegate.size();
    }
}
