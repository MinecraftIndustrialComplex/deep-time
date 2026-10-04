// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.function.LongConsumer;

/**
 * A fundamental domain and the motions that glue its edges (wrap engine, docs/design/wrap-engine.md
 * §1): any table of edges whose motions are translations, half-turns and quarter-turns. Layouts are in
 * {@link WrapLayouts}: the Guyou rectangle, Peirce's square, the cube cross net, and the bounded test
 * seams.
 *
 * <p>{@link #fold} takes a block anywhere in the bands to the one stored block it is, with the motion
 * that took it there (so a block state read there is turned back by its inverse). {@link #images} is
 * the reverse: every band block that shows a given stored block, which the engine keeps as turned
 * replicas so that the client, collisions and light see the far side without knowing about the seam.
 *
 * <p><b>Fast path</b> (design §5). {@link #mayFold} is a lookup in a coarse grid over the bands'
 * bounding box (one flag per cell of 16 to 1,024 blocks): outside the box, or in a cell no band
 * touches, no position folds. {@link #fold} does that first, so a position away from every seam costs
 * a subtraction, two shifts, four compares and one array load, and allocates nothing.
 *
 * <p>Immutable and thread-safe. Pure integer code, unit-tested in WrapTopologyTest.
 */
public final class WrapTopology {

    /** Bound on folding steps: a corner needs two; more means the edges do not close up. */
    static final int MAX_STEPS = 8;
    /** Bound on the length of the edge words {@link #images} tries (corners: 2, a turn point region: 3). */
    static final int MAX_WORD = 3;
    /** Largest fast-path grid, in cells (a byte each). */
    static final int MAX_GRID_CELLS = 1 << 21;

    private final String name;
    private final List<WrapEdge> edges;
    private final WrapEdge[] edgeArray;
    // Fast-path grid over the bands' bounding box.
    private final int gridX0, gridZ0, gridW, gridH, gridShift;
    private final byte[] grid;

    public WrapTopology(List<WrapEdge> edges) {
        this("custom", edges);
    }

    public WrapTopology(String name, List<WrapEdge> edges) {
        this.name = name;
        this.edges = List.copyOf(edges);
        this.edgeArray = this.edges.toArray(new WrapEdge[0]);
        if (edgeArray.length == 0) {
            gridX0 = gridZ0 = gridW = gridH = 0;
            gridShift = 4;
            grid = new byte[0];
            return;
        }
        long x0 = Long.MAX_VALUE, x1 = Long.MIN_VALUE, z0 = Long.MAX_VALUE, z1 = Long.MIN_VALUE;
        for (WrapEdge e : edgeArray) {
            x0 = Math.min(x0, e.bandMinX());
            x1 = Math.max(x1, e.bandMaxX());
            z0 = Math.min(z0, e.bandMinZ());
            z1 = Math.max(z1, e.bandMaxZ());
        }
        int shift = 4;
        while (shift < 10 && cells(x0, x1, shift) * cells(z0, z1, shift) > MAX_GRID_CELLS)
            shift++;
        gridShift = shift;
        gridX0 = (int) x0;
        gridZ0 = (int) z0;
        gridW = (int) cells(x0, x1, shift);
        gridH = (int) cells(z0, z1, shift);
        grid = new byte[gridW * gridH];
        for (WrapEdge e : edgeArray) {
            int cx0 = (e.bandMinX() - gridX0) >> shift, cx1 = (e.bandMaxX() - 1 - gridX0) >> shift;
            int cz0 = (e.bandMinZ() - gridZ0) >> shift, cz1 = (e.bandMaxZ() - 1 - gridZ0) >> shift;
            for (int gz = cz0; gz <= cz1; gz++)
                for (int gx = cx0; gx <= cx1; gx++)
                    grid[gz * gridW + gx] = 1;
        }
    }

    private static long cells(long lo, long hi, int shift) {
        return ((hi - lo - 1) >> shift) + 1;
    }

    public String name() {
        return name;
    }

    public List<WrapEdge> edges() {
        return edges;
    }

    /** A topology with the edges of all of these (the GameTests glue several test sites in one level). */
    public static WrapTopology union(String name, List<WrapTopology> parts) {
        List<WrapEdge> all = new ArrayList<>();
        for (WrapTopology t : parts)
            all.addAll(t.edges);
        return new WrapTopology(name, all);
    }

    /**
     * False when no position in the block column (x, z) can fold: the fast path, allocation-free.
     * True means "maybe": the cell touches a band.
     */
    public boolean mayFold(int x, int z) {
        int gx = (x - gridX0) >> gridShift;
        int gz = (z - gridZ0) >> gridShift;
        if (gx < 0 || gz < 0 || gx >= gridW || gz >= gridH)
            return false;
        return grid[gz * gridW + gx] != 0;
    }

    /**
     * Where a block beyond an edge is stored.
     *
     * @param x   stored block x
     * @param z   stored block z
     * @param iso the motion from the queried block to the stored one
     */
    public record Fold(int x, int z, CellIsometry iso) {}

    /** The stored block for (x, z), or {@code null} when (x, z) is not beyond any edge (inside, or past every band). */
    public Fold fold(int x, int z) {
        if (!mayFold(x, z))
            return null;
        CellIsometry acc = null;
        int cx = x, cz = z;
        for (int step = 0; step < MAX_STEPS; step++) {
            WrapEdge hit = firstBeyond(cx, cz);
            if (hit == null)
                return acc == null ? null : new Fold(cx, cz, acc);
            CellIsometry g = hit.iso();
            int nx = g.applyX(cx, cz), nz = g.applyZ(cx, cz);
            cx = nx;
            cz = nz;
            acc = acc == null ? g : g.compose(acc);
        }
        throw new IllegalStateException("wrap topology " + name + " does not close: (" + x + ", " + z + ") still beyond an edge after "
                + MAX_STEPS + " steps");
    }

    private WrapEdge firstBeyond(int x, int z) {
        for (WrapEdge e : edgeArray)
            if (e.beyond(x, z))
                return e;
        return null;
    }

    /** A point beyond an edge, folded (entities): {x, z} and the motion. */
    public record PointFold(double x, double z, CellIsometry iso) {}

    /** The point (x, z) folded, or {@code null} when it is not beyond any edge. */
    public PointFold foldPoint(double x, double z) {
        if (!mayFold((int) Math.floor(x), (int) Math.floor(z)))
            return null;
        CellIsometry acc = null;
        double cx = x, cz = z;
        for (int step = 0; step < MAX_STEPS; step++) {
            WrapEdge hit = null;
            for (WrapEdge e : edgeArray) {
                if (e.beyondPoint(cx, cz)) {
                    hit = e;
                    break;
                }
            }
            if (hit == null)
                return acc == null ? null : new PointFold(cx, cz, acc);
            double[] p = hit.iso().applyPoint(cx, cz);
            cx = p[0];
            cz = p[1];
            acc = acc == null ? hit.iso() : hit.iso().compose(acc);
        }
        throw new IllegalStateException("wrap topology " + name + " does not close at point (" + x + ", " + z + ")");
    }

    /** True when (x, z) is not beyond any edge (a stored block, or a block past every band). */
    public boolean inside(int x, int z) {
        return !mayFold(x, z) || firstBeyond(x, z) == null;
    }

    /**
     * Every band block that shows the stored block (x, z): the blocks q beyond some edge whose fold is
     * (x, z). Found by walking edge words backwards (inverse motions) up to {@value #MAX_WORD} long,
     * which covers corners and the regions round turn points.
     */
    public List<int[]> images(int x, int z) {
        Set<Long> seen = new LinkedHashSet<>();
        List<int[]> out = new ArrayList<>();
        List<int[]> frontier = List.of(new int[]{x, z});
        seen.add(key(x, z));
        for (int len = 1; len <= MAX_WORD && !frontier.isEmpty(); len++) {
            List<int[]> next = new ArrayList<>();
            for (int[] s : frontier) {
                for (WrapEdge e : edgeArray) {
                    CellIsometry inv = e.iso().inverse();
                    int qx = inv.applyX(s[0], s[1]), qz = inv.applyZ(s[0], s[1]);
                    if (!seen.add(key(qx, qz)))
                        continue;
                    if (inside(qx, qz))
                        continue;
                    Fold f = fold(qx, qz);
                    if (f != null && f.x() == x && f.z() == z) {
                        int[] q = {qx, qz};
                        out.add(q);
                        next.add(q);
                    }
                }
            }
            frontier = next;
        }
        return out;
    }

    private static long key(int x, int z) {
        return ((long) x << 32) ^ (z & 0xffffffffL);
    }

    // --- Chunks (the band manager) -------------------------------------------------------------

    /** True when every edge lies on chunk lines and every motion maps chunks to chunks (see {@link WrapEdge#chunkAligned}). */
    public boolean isChunkAligned() {
        for (WrapEdge e : edgeArray)
            if (!e.chunkAligned())
                return false;
        return true;
    }

    /** A band chunk: the stored chunk it shows and the motion from it to that chunk. */
    public record ChunkFold(int sourceX, int sourceZ, CellIsometry iso) {}

    /**
     * The chunk (cx, cz) as a band chunk: its source chunk and the motion, or {@code null} when it is not
     * a band chunk. On a chunk-aligned topology every block of a band chunk folds by the same motion.
     */
    public ChunkFold chunkFold(int cx, int cz) {
        Fold f = fold(cx << 4, cz << 4);
        if (f == null)
            return null;
        int sx = Math.min(f.x(), f.iso().applyX((cx << 4) + 15, (cz << 4) + 15));
        int sz = Math.min(f.z(), f.iso().applyZ((cx << 4) + 15, (cz << 4) + 15));
        return new ChunkFold(sx >> 4, sz >> 4, f.iso());
    }

    /**
     * The motions from the stored chunk (cx, cz) to each band chunk that shows it (empty when none does).
     * On a chunk-aligned topology every block of the chunk has its images in exactly these band chunks.
     */
    public CellIsometry[] chunkImages(int cx, int cz) {
        int x = cx << 4, z = cz << 4;
        if (!inside(x, z))
            return new CellIsometry[0];
        List<CellIsometry> out = new ArrayList<>();
        for (int[] q : images(x, z)) {
            Fold f = fold(q[0], q[1]);
            out.add(f.iso().inverse());
        }
        return out.toArray(new CellIsometry[0]);
    }

    /** Calls {@code action} with {@code ChunkPos.asLong(cx, cz)} for every band chunk. */
    public void forEachBandChunk(LongConsumer action) {
        for (WrapEdge e : edgeArray) {
            for (int cz = e.bandMinZ() >> 4; cz <= (e.bandMaxZ() - 1) >> 4; cz++)
                for (int cx = e.bandMinX() >> 4; cx <= (e.bandMaxX() - 1) >> 4; cx++)
                    // Report each chunk once: under the first edge it is beyond (the one fold uses).
                    if (firstBeyond(cx << 4, cz << 4) == e)
                        action.accept((long) cx & 0xffffffffL | ((long) cz & 0xffffffffL) << 32);
        }
    }

    @Override
    public String toString() {
        return "WrapTopology[" + name + ", " + edgeArray.length + " edges]";
    }
}
