// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.LakeIce;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.LevelReader;
import net.minecraft.world.level.WorldGenLevel;

/**
 * Keeps a Deep Time world's open lakes open (Ben, decisions log 2026-10-01: lakes are iced by the
 * climate's rule, "regardless of biome"). Vanilla freezes water wherever the biome is cold: the
 * {@code freeze_top_layer} feature at generation, and snowfall afterwards, both through
 * {@code Biome.shouldFreeze}. {@code mixin/BiomeFreezeMixin} asks here first: the surface block of
 * a lake that {@link LakeIce} says is not frozen never freezes. Frozen lakes need no help, because
 * {@link WaterBodies} places their ice, and vanilla ice melts only by block light. Rivers, the sea
 * and every other world are left to the biome.
 */
public final class LakeFreeze {

    private LakeFreeze() {}

    /** Whether {@code pos} is the surface block of a Deep Time lake that the climate keeps open. */
    public static boolean keepsOpen(LevelReader level, BlockPos pos) {
        ServerLevel server = level instanceof ServerLevel s ? s : level instanceof WorldGenLevel w ? w.getLevel() : null;
        if (server == null || !(server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)
                || !(g.terrain() instanceof WorldDataTerrain t))
            return false;
        return LakeIce.keepsOpen(t.data(), g.waterReach(), pos.getX(), pos.getY(), pos.getZ());
    }
}
