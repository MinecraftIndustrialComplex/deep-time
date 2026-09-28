// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.mixin;

import com.llamalad7.mixinextras.injector.wrapmethod.WrapMethod;
import com.llamalad7.mixinextras.injector.wrapoperation.Operation;
import com.minecraftindustrialcomplex.deeptime.wrap.PlanetShape;
import net.minecraft.core.Registry;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.WorldDimensions;
import org.spongepowered.asm.mixin.Mixin;

/**
 * Shapes a Deep Time world's dimensions before they are baked into levels (see {@link PlanetShape}).
 * Toroidal World's own mixin on {@code bake} runs inside the call made here, on the shaped
 * dimensions, so its restore and derive steps see the planet shape.
 */
@Mixin(WorldDimensions.class)
abstract class WorldDimensionsShapeMixin {

    @WrapMethod(method = "bake")
    private WorldDimensions.Complete deeptime$shapePlanet(Registry<LevelStem> stems, Operation<WorldDimensions.Complete> original) {
        WorldDimensions self = (WorldDimensions) (Object) this;
        WorldDimensions shaped = PlanetShape.shape(self);
        if (shaped == self)
            return original.call(stems);
        return shaped.bake(stems); // re-enters this wrapper, which then finds the shape in place
    }
}
