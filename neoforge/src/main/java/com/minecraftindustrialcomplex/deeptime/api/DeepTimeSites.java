// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api;

import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.phys.AABB;

import java.util.List;
import java.util.Optional;

/**
 * The site registry of a Deep Time level (plan §5.6; decisions file "Unlimited-mining sites": the
 * engine exports a site registry and a Java API with location, extent, grade, type and forming
 * event), for the future unlimited-mining system, prospecting and compat. Server side; read-only;
 * thread-safe. Positions wrap east–west like the level (Toroidal World's cylinder): a query box
 * across the seam finds the sites on both sides.
 *
 * <pre>{@code
 * DeepTimeSites sites = DeepTimeSites.of(serverLevel);
 * if (sites.isDeepTimeLevel())
 *     sites.at(pos).ifPresent(s -> ...);   // the site whose body holds pos
 * }</pre>
 */
public interface DeepTimeSites {

    /**
     * The registry of {@code level}: its world data's sites for a Deep Time level, and an empty
     * implementation ({@link #isDeepTimeLevel()} false, every query empty) for any other level.
     */
    static DeepTimeSites of(ServerLevel level) {
        return com.minecraftindustrialcomplex.deeptime.worldgen.LevelSites.of(level);
    }

    /** True for a level whose generator is Deep Time's. */
    boolean isDeepTimeLevel();

    /** Every site of the level, by id. */
    List<Site> all();

    /** The site whose body contains the block, if any (the first by site id where bodies overlap). */
    Optional<Site> at(BlockPos pos);

    /** The sites whose bounds touch the chunk's column. */
    List<Site> inChunk(ChunkPos pos);

    /** The sites whose bounds intersect {@code box}. */
    List<Site> within(AABB box);

    Optional<Site> byId(int id);

    /** The event that formed {@code site}. */
    Optional<GeoEvent> formingEvent(Site site);

    /**
     * The causes of {@code event}, breadth first, up to {@code maxDepth} links away, each once
     * ("Why here?", decisions log Q13).
     */
    List<GeoEvent> causeChain(GeoEvent event, int maxDepth);

    /** The event with {@code id}, if the log has it. */
    Optional<GeoEvent> event(long id);
}
