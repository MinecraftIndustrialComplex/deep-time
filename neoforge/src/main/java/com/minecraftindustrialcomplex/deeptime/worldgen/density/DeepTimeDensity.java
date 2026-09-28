// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen.density;

import com.minecraftindustrialcomplex.deeptime.worldgen.TerrainSource;
import net.minecraft.world.level.levelgen.DensityFunction;

/**
 * Base of Deep Time's density-function types.
 *
 * <p>The noise router is shared data ({@code deeptime:planet} noise settings), but every Deep
 * Time level needs its own world data and circumference. So the functions decoded from JSON are
 * <em>unbound</em>, and each {@code GeologicChunkGenerator} walks the router with
 * {@link DensityFunction#mapAll} and replaces them with copies bound to its level's
 * {@link TerrainSource} (plan §7.1; cold-read review 2, gap 4). An unbound function that is
 * evaluated throws instead of returning plausible garbage.
 */
public abstract class DeepTimeDensity implements DensityFunction.SimpleFunction {

    protected final TerrainSource source;

    protected DeepTimeDensity(TerrainSource source) {
        this.source = source;
    }

    public final boolean isBound() {
        return source != null;
    }

    /** A copy of this function reading {@code source}. */
    public abstract DeepTimeDensity bind(TerrainSource source);

    protected final TerrainSource source() {
        if (source == null)
            throw new IllegalStateException(codec().codec() + ": a Deep Time density function was evaluated outside a "
                    + "Deep Time generator (unbound). Only the deeptime:geologic generator binds them.");
        return source;
    }
}
