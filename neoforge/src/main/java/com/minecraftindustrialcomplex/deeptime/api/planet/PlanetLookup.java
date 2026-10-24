// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.planet;

import com.minecraftindustrialcomplex.deeptime.net.ClientPlanetState;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;

import java.util.Optional;

/**
 * {@link DeepTimePlanet#of}: a level's projection and nominal metres per block, from its generator on
 * a server and from the planet info on a client, and the planet over them, kept for the few worlds
 * last asked about so per-block callers get the same object without allocating.
 */
final class PlanetLookup {

    /** How many (projection, scale) pairs are kept: a server's planet dimensions and a client's. */
    private static final int KEEP = 8;

    private record Entry(WorldProjection projection, double mPerBlockH, Optional<DeepTimePlanet> planet) {}

    /** Newest first; replaced whole (copy on write), so readers on any thread never lock. */
    private static volatile Entry[] recent = new Entry[0];

    private PlanetLookup() {}

    static Optional<DeepTimePlanet> of(Level level) {
        WorldProjection p = projection(level);
        if (p == null)
            return Optional.empty();
        double m = mPerBlockH(level);
        if (!(m > 0.0) || !Double.isFinite(m) || !(p.radius() > 0.0))
            return Optional.empty();
        return planet(p, m);
    }

    /** The planet over {@code p} at {@code m} metres per block, the kept one when it was asked for lately. */
    static Optional<DeepTimePlanet> planet(WorldProjection p, double m) {
        Entry[] r = recent;
        for (Entry e : r)
            if ((e.projection() == p || e.projection().equals(p)) && e.mPerBlockH() == m)
                return e.planet();
        Optional<DeepTimePlanet> made = Optional.of(new ProjectionPlanet(p, m));
        Entry[] n = new Entry[Math.min(r.length + 1, KEEP)];
        n[0] = new Entry(p, m, made);
        System.arraycopy(r, 0, n, 1, n.length - 1);
        recent = n;
        return made;
    }

    /**
     * The world projection of {@code level}: the world data's on a server (Lambert of the generator's
     * C on an analytic test terrain), the planet info's on a client; null for anything else.
     */
    static WorldProjection projection(Level level) {
        if (level == null)
            return null;
        if (level.isClientSide())
            return ClientPlanetState.projection(level);
        if (level instanceof ServerLevel server && server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) {
            if (g.terrain() instanceof WorldDataTerrain t)
                return t.data().projection();
            int c = g.dataRef().circumferenceBlocks();
            return c > 0 ? new WorldProjection.Lambert(c) : null;
        }
        return null;
    }

    /**
     * The nominal horizontal metres per block of {@code level}: its manifest's {@code m_per_block_h} on
     * a server (the default on a test terrain, as the planet info sends), the planet info's on a
     * client; NaN for anything else.
     */
    static double mPerBlockH(Level level) {
        if (level == null)
            return Double.NaN;
        if (level.isClientSide())
            return ClientPlanetState.mPerBlockH(level);
        if (level instanceof ServerLevel server && server.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)
            return g.terrain() instanceof WorldDataTerrain t ? t.data().manifest().mPerBlockH() : Manifest.DEFAULT_M_PER_BLOCK_H;
        return Double.NaN;
    }
}
