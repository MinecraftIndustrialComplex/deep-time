// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.llamalad7.mixinextras.injector.wrapmethod.WrapMethod;
import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.minecraftindustrialcomplex.deeptime.worldgen.OverworldOverride;
import com.minecraftindustrialcomplex.deeptime.wrap.PlanetShape;
import net.minecraft.core.Registry;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;
import org.spongepowered.asm.mixin.Mixin;

/**
 * Prepares a world's dimensions before they are baked into levels: a datapack that overrides the overworld
 * does not replace Deep Time's generator in a Deep Time world, and Deep Time levels take that datapack's biome
 * source ({@link OverworldOverride}); then a Deep Time world gets its planet shape ({@link PlanetShape}).
 * Toroidal World's own mixin on {@code bake} runs inside the call made here, on the shaped dimensions, so its
 * restore and derive steps see the planet shape.
 */
@Mixin(WorldDimensions.class)
abstract class WorldDimensionsShapeMixin {

    @WrapMethod(method = "bake")
    private WorldDimensions.Complete deeptime$shapePlanet(Registry<LevelStem> stems, Operation<WorldDimensions.Complete> original) {
        WorldDimensions self = (WorldDimensions) (Object) this;
        OverworldOverride.Result o = OverworldOverride.apply(self, stems);
        WorldDimensions shaped = PlanetShape.shape(o.dims());
        if (shaped == self && o.stems() == stems)
            return original.call(stems);
        return shaped.bake(o.stems()); // re-enters this wrapper, which finds the override dealt with and the shape in place
    }
}
