// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.terrablender;

import net.minecraft.core.Holder;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.Climate;
import terrablender.api.Region;
import terrablender.worldgen.IExtendedParameterList;

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
        IExtendedParameterList<?> ext = (IExtendedParameterList<?>) parameters;
        Climate.RTree<Holder<Biome>> tree;
        try {
            tree = tree(ext, region);
        } catch (RuntimeException noTree) {
            tree = tree(ext, 0);
        }
        Holder<Biome> biome = tree.search(target, Climate.RTree.Node::distance);
        if (biome.is(Region.DEFERRED_PLACEHOLDER))
            return tree(ext, 0).search(target, Climate.RTree.Node::distance);
        return biome;
    }
}
