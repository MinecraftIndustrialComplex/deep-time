// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worldgen.density.DeepTimeDensity;
import net.minecraft.world.level.levelgen.DensityFunction;
import net.minecraft.world.level.levelgen.NoiseRouter;

/** Asserts that binding reached every Deep Time function of a router. */
final class DeepTimeDensityCheck {

    private DeepTimeDensityCheck() {}

    static void requireBound(NoiseRouter router) {
        router.mapAll(new DensityFunction.Visitor() {
            @Override
            public DensityFunction apply(DensityFunction f) {
                if (f instanceof DeepTimeDensity d && !d.isBound())
                    throw new IllegalStateException("unbound Deep Time density function survived binding: " + f);
                return f;
            }
        });
    }
}
