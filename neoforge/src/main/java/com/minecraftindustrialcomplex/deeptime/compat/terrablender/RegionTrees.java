// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.terrablender;

import com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes;
import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeSource;
import net.minecraft.world.level.biome.Climate;
import terrablender.api.Region;
import terrablender.worldgen.IExtendedParameterList;

import java.util.Set;
import java.util.function.Consumer;

/**
 * The biome of a climate point inside one TerraBlender region's search tree (the body of
 * TerraBlender 4.1's {@code findValuePositional}, with the region index given instead of its own
 * layered noise's). Used by {@code mixin.compat.terrablender.RegionBiomeSourceMixin}.
 */
public final class RegionTrees {

    private RegionTrees() {}

    /** Whether TerraBlender has initialised this parameter list (its regions' trees are built). */
    public static boolean ready(Climate.ParameterList<Holder<Biome>> parameters) {
        return parameters instanceof IExtendedParameterList<?> ext && ext.isInitialized();
    }

    /** TerraBlender's own region at quart (x, y, z): its layered noise (where the level has no province table). */
    public static int ownRegion(Climate.ParameterList<Holder<Biome>> parameters, int x, int y, int z) {
        return ((IExtendedParameterList<?>) parameters).getUniqueness(x, y, z);
    }

    @SuppressWarnings("unchecked")
    private static Climate.RTree<Holder<Biome>> tree(IExtendedParameterList<?> ext, int region) {
        return (Climate.RTree<Holder<Biome>>) ext.getTree(region);
    }

    /**
     * The biome of {@code target} in the tree of region {@code region}. A region without a tree
     * (it adds no biomes) and a deferred placeholder both answer from the vanilla region 0, as
     * TerraBlender's own lookup does.
     */
    public static Holder<Biome> find(Climate.ParameterList<Holder<Biome>> parameters, int region, Climate.TargetPoint target) {
        return find(parameters, region, target, VolcanicBiomes.NEAREST);
    }

    /** As {@link #find(Climate.ParameterList, int, Climate.TargetPoint)}, nearest under {@code metric}. */
    public static Holder<Biome> find(Climate.ParameterList<Holder<Biome>> parameters, int region, Climate.TargetPoint target,
                                     Climate.DistanceMetric<Holder<Biome>> metric) {
        IExtendedParameterList<?> ext = (IExtendedParameterList<?>) parameters;
        Climate.RTree<Holder<Biome>> tree;
        try {
            tree = tree(ext, region);
        } catch (RuntimeException noTree) {
            tree = tree(ext, 0);
        }
        Holder<Biome> biome = tree.search(target, metric);
        if (biome.is(Region.DEFERRED_PLACEHOLDER))
            return tree(ext, 0).search(target, metric);
        return biome;
    }

    /**
     * One region's search as the volcanic biomes see it ({@link VolcanicBiomes.Source}): its tree, the biome
     * source's biomes, and the leaves of every region's tree (TerraBlender's regions hold biomes the vanilla
     * list does not).
     */
    public record RegionSearch(BiomeSource biomes, Climate.ParameterList<Holder<Biome>> parameters, int region)
            implements VolcanicBiomes.Source {
        @Override
        public Object key() {
            return parameters;
        }

        @Override
        public Holder<Biome> find(Climate.TargetPoint target, Climate.DistanceMetric<Holder<Biome>> metric) {
            return RegionTrees.find(parameters, region, target, metric);
        }

        @Override
        public Set<Holder<Biome>> possibleBiomes() {
            return biomes.possibleBiomes();
        }

        @Override
        public java.util.Optional<VolcanicBiomes.Source> base() {
            return region == 0 ? java.util.Optional.empty() : java.util.Optional.of(new RegionSearch(biomes, parameters, 0));
        }

        @Override
        public void entries(Consumer<VolcanicBiomes.Entry> sink) {
            IExtendedParameterList<?> ext = (IExtendedParameterList<?>) parameters;
            for (int i = 0; i < ext.getTreeCount(); i++) {
                Climate.RTree<Holder<Biome>> t;
                try {
                    t = tree(ext, i);
                } catch (RuntimeException noTree) {
                    continue;
                }
                if (t != null)
                    VolcanicBiomes.leaves(t, sink);
            }
        }
    }
}
