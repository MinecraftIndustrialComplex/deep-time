// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat;

import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.Deposits;
import com.minecraftindustrialcomplex.deeptime.worldgen.LevelSites;
import net.minecraft.world.level.ChunkPos;

import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.WeakHashMap;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Predicate;

/**
 * Which chunks a site's ore reaches, and how much of it lies in each (P1-16, plan §7.14): the COE
 * and CDG hooks work per chunk, Deep Time's sites are bodies. A site's <b>footprint</b> is the
 * volume (blocks) of its {@code ore} bodies ({@link Body#locate}, the deposit feature's own test,
 * erosion clip included) in every chunk column it reaches; lithocaps and gossans do not count.
 * Chunk x is taken in the canonical span, so a body across the seam fills the chunks on both
 * sides. On a cube world a chunk wholly off the faces counts nothing: it places no ore of its own
 * ({@code CubeFill}, ledger SCL-210; it holds a turned copy of its partner's, whose own sites count
 * there), so the part of a body past a turning edge is not in the game. Computed once per site and world data (every block of the bodies' bounds; a body bigger
 * than {@link #MAX_SAMPLES} evaluations is sampled on a coarser lattice, each sample standing for
 * stride³ blocks) and kept while the world data is loaded.
 */
public final class SiteChunks {

    private SiteChunks() {}

    /** Body evaluations per site before the lattice coarsens. */
    public static final long MAX_SAMPLES = 2_000_000L;

    /**
     * The body role whose blocks count (FORMAT.md "Bodies"): every site's first body, and all of a stacked
     * deposit's seams (consecutive bodies of this role).
     */
    public static final String ORE_ROLE = "ore";

    private static final Map<Deposits, Map<Integer, Footprint>> CACHE = Collections.synchronizedMap(new WeakHashMap<>());

    /**
     * A site's ore-body volume per chunk.
     *
     * @param siteId  the site
     * @param byChunk {@link ChunkPos#asLong} of a canonical chunk → ore-body blocks in its column
     * @param total   the sum over every chunk
     * @param stride  the lattice step used (1 = every block evaluated)
     */
    public record Footprint(int siteId, Map<Long, Double> byChunk, double total, int stride) {

        /** Ore-body blocks in chunk (cx, cz) (canonical chunk coordinates). */
        public double in(int cx, int cz) {
            return byChunk.getOrDefault(ChunkPos.asLong(cx, cz), 0.0);
        }

        /** The chunk's fraction of the site's ore-body volume. */
        public double share(int cx, int cz) {
            return total > 0 ? in(cx, cz) / total : 0;
        }

        /** The chunk holding the most of the body (ties: the smallest key), or null for an empty footprint. */
        public ChunkPos best() {
            long bestKey = 0;
            double bestV = 0;
            for (var e : byChunk.entrySet())
                if (e.getValue() > bestV || (e.getValue() == bestV && bestV > 0 && e.getKey() < bestKey)) {
                    bestV = e.getValue();
                    bestKey = e.getKey();
                }
            return bestV > 0 ? new ChunkPos(bestKey) : null;
        }
    }

    /** Chunk x of a chunk moved into the canonical span [−C/2, C/2) of a planet of circumference c. */
    public static int canonicalChunkX(int c, int cx) {
        int half = c / 2;
        return (Math.floorMod((cx << 4) + half, c) - half) >> 4;
    }

    /** The x extent of {@code b}'s box sampled: its own, at most one period {@code c}. */
    private static long spanX(int c, Body b) {
        return Math.min((long) b.max()[0] - b.min()[0] + 1, c);
    }

    /** The footprint of {@code site} in the level's world data. */
    public static Footprint footprint(LevelSites sites, Site site) {
        Deposits d = sites.deposits();
        Map<Integer, Footprint> perSite = CACHE.computeIfAbsent(d, k -> new ConcurrentHashMap<>());
        return perSite.computeIfAbsent(site.id(), id -> compute(d, id, sites.cube()));
    }

    private static Footprint compute(Deposits d, int siteId, CubeLayout cube) {
        return compute(d.bodies().ofSite(siteId), d.circumference(), cube, siteId);
    }

    /**
     * The footprint of a site's bodies on a planet of circumference {@code c} ({@code cube}: a cube
     * world's net, or null). Public for the GameTests ({@code CompatGameTests.siteFootprintsStayOnTheFaces}).
     */
    public static Footprint compute(List<Body> bodies, int c, CubeLayout cube, int siteId) {
        List<Body> ore = bodies.stream().filter(b -> ORE_ROLE.equals(b.role())).toList();
        if (ore.isEmpty())
            return new Footprint(siteId, Map.of(), 0, 1);
        // A site's ore bodies are disjoint (one per seam of a stacked deposit, FORMAT.md "Bodies"),
        // so each is sampled over its own bounds and the counts add. The lattice step is set by
        // the bounds' total volume: a stack of thin seams is not one tall box.
        // The loops run on long and over at most one period in x (the world wraps there): a body's box can
        // reach the int range's ends (2026-10-24: a placer piece past the pole made a box of ±2^31, and the
        // int loop overflowed and never ended on the server thread).
        double box = 0;
        for (Body b : ore)
            box += (double) spanX(c, b) * ((long) b.max()[1] - b.min()[1] + 1) * ((long) b.max()[2] - b.min()[2] + 1);
        int stride = (int) Math.max(1, Math.ceil(Math.cbrt(box / MAX_SAMPLES)));
        double weight = (double) stride * stride * stride;
        Map<Long, Double> out = new HashMap<>();
        double total = 0;
        for (Body b : ore) {
            int[] min = b.min(), max = b.max();
            long xEnd = (long) min[0] + spanX(c, b) - 1;
            for (long xl = min[0]; xl <= xEnd; xl += stride) {
                int x = xl <= Integer.MAX_VALUE ? (int) xl : (int) Math.floorMod(xl, (long) c);  // Body wraps x itself
                int cx = canonicalChunkX(c, x >> 4);
                for (long zl = min[2]; zl <= max[2]; zl += stride) {
                    int z = (int) zl;
                    if (cube != null && cube.faceOfBlock(x, z) == null)
                        continue;   // a void chunk: none of this body is placed there
                    long key = ChunkPos.asLong(cx, z >> 4);
                    double column = 0;
                    for (long yl = min[1]; yl <= max[1]; yl += stride)
                        if (b.aabbContains(c, x, (int) yl, z) && b.locate(c, x, (int) yl, z) != null)
                            column += weight;
                    if (column > 0) {
                        out.merge(key, column, Double::sum);
                        total += column;
                    }
                }
            }
        }
        return new Footprint(siteId, Map.copyOf(out), total, stride);
    }

    /**
     * Among the sites whose bounds touch chunk (cx, cz) and pass {@code filter}, the one whose ore
     * bodies fill the chunk the most (ties: the lower site id); empty outside Deep Time levels and
     * where no such site's ore reaches the chunk.
     */
    public static Optional<Site> covering(LevelSites sites, int cx, int cz, Predicate<Site> filter) {
        if (!sites.isDeepTimeLevel())
            return Optional.empty();
        int ccx = canonicalChunkX(sites.deposits().circumference(), cx);
        Site best = null;
        double bestV = 0;
        for (Site s : sites.inChunk(new ChunkPos(ccx, cz))) {
            if (!filter.test(s))
                continue;
            double v = footprint(sites, s).in(ccx, cz);
            if (v > bestV || (v == bestV && v > 0 && best != null && Integer.compareUnsigned(s.id(), best.id()) < 0)) {
                best = s;
                bestV = v;
            }
        }
        return Optional.ofNullable(best);
    }
}
