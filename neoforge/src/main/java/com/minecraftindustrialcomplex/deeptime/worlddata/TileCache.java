// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import com.google.common.util.concurrent.ExecutionError;
import com.google.common.util.concurrent.UncheckedExecutionException;

import java.util.concurrent.ExecutionException;

/**
 * The bounded, concurrent cache of decoded LZ4 tiles (plan §5.7.3): Guava's {@link CacheBuilder}
 * (on the game classpath), weighted by decoded bytes, default 256 MiB. One cache can serve every
 * tile pack of every open world.
 */
public final class TileCache {

    /** The plan's default budget. */
    public static final long DEFAULT_BYTES = 256L << 20;

    private record Key(TilePack pack, int tile) {
        // TilePack keeps identity equality, so two opens of one file never share entries.
    }

    private final Cache<Key, Tile> cache;
    private final long maxBytes;

    /**
     * The least a cache segment must hold: several of the largest tiles. A 256 × 256 tile of
     * {@code strata/horizons} with K = 32 horizons is 14.7 MiB decoded.
     */
    static final long MIN_SEGMENT_BYTES = 64L << 20;

    /**
     * A cache of {@code maxBytes} with as many segments as fit {@link #MIN_SEGMENT_BYTES} each, at
     * most one per processor (4 for the default budget). Guava never keeps an entry heavier than
     * its segment's share: with one segment per processor a 32-core machine had 8 MiB segments, the
     * strata tiles were never cached, and every sample that left a thread's last tile decoded
     * 14.7 MiB again (P3-04, measured on a C = 16,384 planet: a far section at a strata tile's edge
     * 566 ms against 34 ms). Reads take no lock, so fewer segments cost only concurrent decodes.
     */
    public TileCache(long maxBytes) {
        this(maxBytes, (int) Math.max(1, Math.min(Runtime.getRuntime().availableProcessors(), maxBytes / MIN_SEGMENT_BYTES)));
    }

    /**
     * @param concurrency Guava's concurrency level. Guava splits the budget across that many
     *                    segments, so each segment must hold several tiles (a 256 × 256 i16 tile
     *                    is 128 KiB).
     */
    public TileCache(long maxBytes, int concurrency) {
        this.maxBytes = maxBytes;
        this.cache = CacheBuilder.newBuilder()
                .maximumWeight(maxBytes)
                .weigher((Key k, Tile t) -> t.byteSize())
                .concurrencyLevel(concurrency)
                .build();
    }

    /** Decoded bytes currently held. */
    public long weightBytes() {
        long sum = 0;
        for (Tile t : cache.asMap().values()) sum += t.byteSize();
        return sum;
    }

    public TileCache() {
        this(DEFAULT_BYTES);
    }

    public long maxBytes() {
        return maxBytes;
    }

    /** Tile t of pack, decoding it on a miss. Concurrent misses on one tile decode it once. */
    Tile get(TilePack pack, int t) {
        Key key = new Key(pack, t);
        Tile hit = cache.getIfPresent(key);
        if (hit != null) return hit;
        try {
            return cache.get(key, () -> pack.decode(t));
        } catch (ExecutionException | UncheckedExecutionException | ExecutionError e) {
            Throwable c = e.getCause();
            if (c instanceof WorldDataException wde) throw wde;
            throw new WorldDataException(pack.path() + ": tile " + t + ": " + c, c);
        }
    }

    /** Decoded tiles currently held. */
    public long size() {
        return cache.size();
    }

    /** Drops every tile of {@code pack} (when a world is closed). */
    public void invalidate(TilePack pack) {
        cache.asMap().keySet().removeIf(k -> k.pack() == pack);
    }

    public void invalidateAll() {
        cache.invalidateAll();
    }
}
