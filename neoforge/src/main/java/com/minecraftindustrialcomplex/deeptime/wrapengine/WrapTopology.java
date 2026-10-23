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
 * {@link WrapLayouts}: the Guyou rectangle, Peirce's square, the cube from the cube world's net, and
 * the bounded test seams.
 *
 * <p>{@link #fold} takes a block anywhere in the bands to the one stored block it is, with the motion
 * that took it there (so a block state read there is turned back by its inverse). {@link #images} is
 * the reverse: every band block that shows a given stored block, which the engine keeps as turned
 * replicas so that the client, collisions and light see the far side without knowing about the seam.
 *
 * <p><b>Overlapping bands</b> (design §12.2). Where two bands share a block (the cube's void squares
 * by a face's corner) the block folds by the edge it is least deep beyond, ties to the earlier edge in
 * the list, as {@code CubeLayout.fold} does (ledger: SCL-181). A band chunk on such a diagonal folds by
 * two motions: {@link ChunkFold#split()}, {@link #chunkColumns}. Topologies whose bands never overlap
 * (every phase-1 layout) fold by the one edge that holds the block, as before.
 *
 * <p><b>Period</b> (design §12.3). With {@code periodX} &gt; 0 the level loops east to west by itself
 * (Toroidal World): a block outside the window [windowLo, windowLo + periodX) is reduced into it
 * before the band test and the translation is part of its motion; every band lies in the window, and
 * band chunks and images exist only there.
 *
 * <p><b>Corners</b> (design §12.4). A topology may name its cone points ({@link WrapCorner});
 * {@link #nearCorner} is true within {@link #CORNER_KEEP_OUT} blocks of one, where seam links are
 * refused.
 *
 * <p><b>Fast path</b> (design §5). {@link #mayFold} is a lookup in a coarse grid over the bands'
 * bounding box (one flag per cell of 16 to 1,024 blocks): outside the box, or in a cell no band
 * touches, no position folds. {@link #fold} does that first, so a position away from every seam costs
 * a subtraction, two shifts, four compares and one array load, and allocates nothing.
 *
 * <p>Immutable and thread-safe. Pure integer code, unit-tested in WrapTopologyTest and
 * WrapTopologyCubeTest.
 */
public final class WrapTopology {

    /** Bound on folding steps: a corner needs two; more means the edges do not close up. */
    static final int MAX_STEPS = 8;
    /** Bound on the length of the edge words {@link #images} tries (corners: 2, a turn point region: 3). */
    static final int MAX_WORD = 3;
    /** Largest fast-path grid, in cells (a byte each). */
    static final int MAX_GRID_CELLS = 1 << 21;
    /**
     * Seam links within this many blocks of a cone point are refused: the 4 × 4 blocks round it, the
     * corner's own 2 × 2 and their diagonal neighbours (ledger: SCL-183, game-choice).
     */
    public static final int CORNER_KEEP_OUT = 2;

    private static final CellIsometry[] NO_ISOS = new CellIsometry[0];
    /** The column table of a band chunk with one source (shared; never written). */
    private static final byte[] ONE_SOURCE = new byte[256];

    private final String name;
    private final List<WrapEdge> edges;
    private final WrapEdge[] edgeArray;
    private final CellIsometry[] inverses;
    private final List<WrapCorner> corners;
    private final int periodX, windowLo;
    private final boolean overlapping;
    // Fast-path grid over the bands' bounding box.
    private final int gridX0, gridZ0, gridW, gridH, gridShift;
    private final byte[] grid;

    public WrapTopology(List<WrapEdge> edges) {
        this("custom", edges);
    }

    public WrapTopology(String name, List<WrapEdge> edges) {
        this(name, edges, List.of(), 0, 0);
    }

    /**
     * The general constructor (design §9.6): any edges and cone points.
     *
     * @param periodX  0, or the east-west period of a level that loops by itself (Toroidal World)
     * @param windowLo the first x of the window [windowLo, windowLo + periodX) every band lies in
     *                 (ignored when {@code periodX} is 0)
     */
    public WrapTopology(String name, List<WrapEdge> edges, List<WrapCorner> corners, int periodX, int windowLo) {
        if (periodX < 0)
            throw new IllegalArgumentException("wrap topology " + name + ": negative period " + periodX);
        this.name = name;
        this.edges = List.copyOf(edges);
        this.edgeArray = this.edges.toArray(new WrapEdge[0]);
        this.corners = List.copyOf(corners);
        this.periodX = periodX;
        this.windowLo = periodX == 0 ? 0 : windowLo;
        this.inverses = new CellIsometry[edgeArray.length];
        boolean ov = false;
        for (int i = 0; i < edgeArray.length; i++) {
            WrapEdge e = edgeArray[i];
            inverses[i] = e.iso().inverse();
            if (periodX > 0 && (e.bandMinX() < windowLo || (long) e.bandMaxX() > (long) windowLo + periodX))
                throw new IllegalArgumentException("wrap topology " + name + ": edge " + e.name() + "'s band [" + e.bandMinX() + ", " + e.bandMaxX()
                        + ") leaves the window [" + windowLo + ", " + ((long) windowLo + periodX) + ")");
            for (int j = 0; j < i; j++)
                ov |= e.bandOverlaps(edgeArray[j]);
        }
        this.overlapping = ov;
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

    /** The cone points (empty for a topology that names none). */
    public List<WrapCorner> corners() {
        return corners;
    }

    /** The east-west period the level loops by itself, or 0. */
    public int periodX() {
        return periodX;
    }

    /** The window's first x when {@link #periodX} &gt; 0 (else 0). */
    public int windowLo() {
        return windowLo;
    }

    /** A topology with the edges and corners of all of these (the GameTests glue several test sites in one level). */
    public static WrapTopology union(String name, List<WrapTopology> parts) {
        List<WrapEdge> all = new ArrayList<>();
        List<WrapCorner> cs = new ArrayList<>();
        for (WrapTopology t : parts) {
            if (t.periodX != 0)
                throw new IllegalArgumentException("wrap topology " + t.name + " has a period; a union of periodic topologies is not defined");
            all.addAll(t.edges);
            cs.addAll(t.corners);
        }
        return new WrapTopology(name, all, cs, 0, 0);
    }

    /** The same topology moved by (dx, dz): edges, motions (conjugated), corners and window (tests put a cube at a far site). */
    public WrapTopology translated(int dx, int dz) {
        List<WrapEdge> e = new ArrayList<>(edgeArray.length);
        for (WrapEdge w : edgeArray)
            e.add(w.translated(dx, dz));
        List<WrapCorner> c = new ArrayList<>(corners.size());
        for (WrapCorner k : corners)
            c.add(k.translated(dx, dz));
        return new WrapTopology(name + " at (" + dx + ", " + dz + ")", e, c, periodX, windowLo + dx);
    }

    // --- The period -----------------------------------------------------------------------------

    /** x reduced into the window (periodic topologies only). */
    private int reduceX(int x) {
        long d = (long) x - windowLo;
        if (d >= 0 && d < periodX)
            return x;
        return (int) (windowLo + Math.floorMod(d, (long) periodX));
    }

    private boolean inWindow(int x) {
        return periodX == 0 || reduceX(x) == x;
    }

    /**
     * The copy of x (a position in this topology's frame) nearest {@code toward} under the period: x
     * itself when the topology has none. Distances between a stored position and a band image are
     * measured with this.
     */
    public int nearX(int x, int toward) {
        if (periodX == 0)
            return x;
        long d = Math.floorMod((long) x - toward + periodX / 2, (long) periodX) - periodX / 2;
        return (int) (toward + d);
    }

    // --- Folding --------------------------------------------------------------------------------

    /**
     * False when no position in the block column (x, z) can fold: the fast path, allocation-free.
     * True means "maybe": the cell touches a band.
     */
    public boolean mayFold(int x, int z) {
        return gridHit(periodX != 0 ? reduceX(x) : x, z);
    }

    private boolean gridHit(int x, int z) {
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
        int cx = periodX != 0 ? reduceX(x) : x;
        if (!gridHit(cx, z))
            return null;
        CellIsometry acc = cx != x ? CellIsometry.translation(cx - x, 0) : null;
        boolean folded = false;
        int cz = z;
        for (int step = 0; step < MAX_STEPS; step++) {
            WrapEdge hit = beyondEdge(cx, cz);
            if (hit == null)
                return folded ? new Fold(cx, cz, acc) : null;
            CellIsometry g = hit.iso();
            int nx = g.applyX(cx, cz), nz = g.applyZ(cx, cz);
            cx = nx;
            cz = nz;
            acc = acc == null ? g : g.compose(acc);
            folded = true;
            if (periodX != 0) {
                int r = reduceX(cx);
                if (r != cx) {
                    acc = CellIsometry.translation(r - cx, 0).compose(acc);
                    cx = r;
                }
            }
        }
        throw new IllegalStateException("wrap topology " + name + " does not close: (" + x + ", " + z + ") still beyond an edge after "
                + MAX_STEPS + " steps");
    }

    /** The edge a block folds by: the only one that holds it, or with overlapping bands the nearest. */
    private WrapEdge beyondEdge(int x, int z) {
        return overlapping ? nearestBeyond(x, z) : firstBeyond(x, z);
    }

    private WrapEdge firstBeyond(int x, int z) {
        for (WrapEdge e : edgeArray)
            if (e.beyond(x, z))
                return e;
        return null;
    }

    /** The edge (x, z) is least deep beyond, ties to the earlier edge (ledger: SCL-181). */
    private WrapEdge nearestBeyond(int x, int z) {
        WrapEdge best = null;
        int bestD = Integer.MAX_VALUE;
        for (WrapEdge e : edgeArray) {
            int d = e.depthBeyond(x, z);
            if (d >= 0 && d < bestD) {
                best = e;
                bestD = d;
            }
        }
        return best;
    }

    /** A point beyond an edge, folded (entities): {x, z} and the motion. */
    public record PointFold(double x, double z, CellIsometry iso) {}

    /**
     * The point (x, z) folded, or {@code null} when it is not beyond any edge. A point takes the motion
     * of the block it is in (design §12.2), so an entity and the block under it never part.
     */
    public PointFold foldPoint(double x, double z) {
        Fold f = fold((int) Math.floor(x), (int) Math.floor(z));
        if (f == null)
            return null;
        double[] p = f.iso().applyPoint(x, z);
        return new PointFold(p[0], p[1], f.iso());
    }

    /** True when (x, z) is not beyond any edge (a stored block, or a block past every band). */
    public boolean inside(int x, int z) {
        int cx = periodX != 0 ? reduceX(x) : x;
        return !gridHit(cx, z) || beyondEdge(cx, z) == null;
    }

    /**
     * True when the block (x, z) lies within {@link #CORNER_KEEP_OUT} blocks of a cone point's image:
     * x in [cx − K, cx + K) and z in [cz − K, cz + K) (design §12.4).
     */
    public boolean nearCorner(int x, int z) {
        if (corners.isEmpty())
            return false;
        int cx = periodX != 0 ? reduceX(x) : x;
        for (WrapCorner c : corners)
            for (int[] p : c.images())
                if (cx >= p[0] - CORNER_KEEP_OUT && cx < p[0] + CORNER_KEEP_OUT && z >= p[1] - CORNER_KEEP_OUT && z < p[1] + CORNER_KEEP_OUT)
                    return true;
        return false;
    }

    /**
     * Every band block that shows the stored block (x, z): the blocks q beyond some edge whose fold is
     * (x, z). Found by walking edge words backwards (inverse motions) up to {@value #MAX_WORD} long,
     * which covers corners and the regions round turn points. On a periodic topology, only images in the
     * window (Toroidal World aliases the rest), of (x, z) reduced into it.
     */
    public List<int[]> images(int x, int z) {
        if (periodX != 0)
            x = reduceX(x);
        Set<Long> seen = new LinkedHashSet<>();
        List<int[]> out = new ArrayList<>();
        List<int[]> frontier = List.of(new int[]{x, z});
        seen.add(key(x, z));
        for (int len = 1; len <= MAX_WORD && !frontier.isEmpty(); len++) {
            List<int[]> next = new ArrayList<>();
            for (int[] s : frontier) {
                for (CellIsometry inv : inverses) {
                    int qx = inv.applyX(s[0], s[1]), qz = inv.applyZ(s[0], s[1]);
                    if (!inWindow(qx) || !seen.add(key(qx, qz)))
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

    /**
     * Of the stored block (x, z) and its images, the one nearest the point (px, pz) (block centres;
     * on a periodic topology the copy nearest the point): {x, z}.
     */
    public int[] nearestImage(int x, int z, double px, double pz) {
        int bx = nearX(x, (int) Math.floor(px)), bz = z;
        double best = dist2(bx, bz, px, pz);
        for (int[] q : images(x, z)) {
            int qx = nearX(q[0], (int) Math.floor(px));
            double d = dist2(qx, q[1], px, pz);
            if (d < best) {
                best = d;
                bx = qx;
                bz = q[1];
            }
        }
        return new int[]{bx, bz};
    }

    private static double dist2(int x, int z, double px, double pz) {
        double dx = x + 0.5 - px, dz = z + 0.5 - pz;
        return dx * dx + dz * dz;
    }

    // --- Chunks (the band manager) -------------------------------------------------------------

    /**
     * True when every edge lies on chunk lines and every motion maps chunks to chunks (see
     * {@link WrapEdge#chunkAligned}), and the window, if any, is whole chunks.
     */
    public boolean isChunkAligned() {
        for (WrapEdge e : edgeArray)
            if (!e.chunkAligned())
                return false;
        return periodX == 0 || ((periodX & 15) == 0 && (windowLo & 15) == 0);
    }

    /**
     * A band chunk: the stored chunk its first block (its north-west corner) shows and the motion from
     * it to that chunk. {@code split} when not every block of the chunk folds by that motion (a chunk on
     * the diagonal of two overlapping bands): {@link #chunkColumns} then says which block folds how.
     */
    public record ChunkFold(int sourceX, int sourceZ, CellIsometry iso, boolean split) {}

    /**
     * The chunk (cx, cz) as a band chunk, or {@code null} when it is not a band chunk (on a periodic
     * topology: not in the window either). On a chunk-aligned topology every block of a band chunk
     * folds, by this motion unless the chunk is split.
     */
    public ChunkFold chunkFold(int cx, int cz) {
        int x0 = cx << 4, z0 = cz << 4;
        if (!inWindow(x0))
            return null;
        Fold f = fold(x0, z0);
        if (f == null)
            return null;
        boolean split = differs(f, x0 + 15, z0) || differs(f, x0, z0 + 15) || differs(f, x0 + 15, z0 + 15);
        return new ChunkFold(sourceChunkX(f.iso(), x0, z0), sourceChunkZ(f.iso(), x0, z0), f.iso(), split);
    }

    /**
     * Whether (x, z) folds by another motion than {@code f}. Checking a chunk's four corners finds every
     * split chunk: two bands' diagonal is a half-plane boundary, and a half-plane that cuts a square
     * separates two of its corners.
     */
    private boolean differs(Fold f, int x, int z) {
        Fold g = fold(x, z);
        return g == null || !g.iso().equals(f.iso());
    }

    private static int sourceChunkX(CellIsometry g, int x0, int z0) {
        return Math.min(g.applyX(x0, z0), g.applyX(x0 + 15, z0 + 15)) >> 4;
    }

    private static int sourceChunkZ(CellIsometry g, int x0, int z0) {
        return Math.min(g.applyZ(x0, z0), g.applyZ(x0 + 15, z0 + 15)) >> 4;
    }

    /** One source of a band chunk: a stored chunk and the motion from the band chunk to it. */
    public record ChunkSource(int sourceX, int sourceZ, CellIsometry iso) {}

    /**
     * A band chunk's sources and which one each block column shows.
     *
     * @param sources the sources, the one of the chunk's first block first
     * @param columns per column {@code (lz << 4) | lx}: the index in {@code sources}, or -1 for a
     *                column that does not fold (none on a chunk-aligned topology); shared, never write it
     */
    public record ChunkColumns(List<ChunkSource> sources, byte[] columns) {
        /** The source of column (lx, lz), or null. */
        public ChunkSource at(int lx, int lz) {
            int i = columns[(lz << 4) | lx];
            return i < 0 ? null : sources.get(i);
        }
    }

    /** The band chunk (cx, cz)'s sources by column, or {@code null} when it is not a band chunk. */
    public ChunkColumns chunkColumns(int cx, int cz) {
        ChunkFold cf = chunkFold(cx, cz);
        if (cf == null)
            return null;
        if (!cf.split())
            return new ChunkColumns(List.of(new ChunkSource(cf.sourceX(), cf.sourceZ(), cf.iso())), ONE_SOURCE);
        int x0 = cx << 4, z0 = cz << 4;
        List<ChunkSource> sources = new ArrayList<>(2);
        byte[] columns = new byte[256];
        for (int lz = 0; lz < 16; lz++) {
            for (int lx = 0; lx < 16; lx++) {
                Fold f = fold(x0 + lx, z0 + lz);
                int i = -1;
                if (f != null) {
                    for (int k = 0; k < sources.size() && i < 0; k++)
                        if (sources.get(k).iso().equals(f.iso()))
                            i = k;
                    if (i < 0) {
                        i = sources.size();
                        sources.add(new ChunkSource(sourceChunkX(f.iso(), x0, z0), sourceChunkZ(f.iso(), x0, z0), f.iso()));
                    }
                }
                columns[(lz << 4) | lx] = (byte) i;
            }
        }
        return new ChunkColumns(List.copyOf(sources), columns);
    }

    /** Every (source chunk, motion) of the band chunk (cx, cz); empty when it is not a band chunk. */
    public List<ChunkSource> chunkSources(int cx, int cz) {
        ChunkColumns c = chunkColumns(cx, cz);
        return c == null ? List.of() : c.sources();
    }

    /**
     * The motions from the stored chunk (cx, cz) to each band chunk that shows some of its blocks (empty
     * when none does). On a chunk-aligned topology each block of the chunk has its images among these
     * motions' targets; near an overlap diagonal a motion shows only part of the chunk, so a writer
     * checks that the fold of a target is the stored block ({@link BandManager#replicate} does). Found
     * from the images of the chunk's four corners: the blocks a motion shows form a half-plane cut of
     * the chunk, which holds a corner.
     */
    public CellIsometry[] chunkImages(int cx, int cz) {
        int x0 = cx << 4, z0 = cz << 4;
        if (!inWindow(x0) || !inside(x0, z0))
            return NO_ISOS;
        List<CellIsometry> out = null;
        for (int c = 0; c < 4; c++) {
            int x = x0 + ((c & 1) != 0 ? 15 : 0), z = z0 + ((c & 2) != 0 ? 15 : 0);
            for (int[] q : images(x, z)) {
                CellIsometry g = fold(q[0], q[1]).iso().inverse();
                if (out == null)
                    out = new ArrayList<>(2);
                if (!out.contains(g))
                    out.add(g);
            }
        }
        return out == null ? NO_ISOS : out.toArray(NO_ISOS);
    }

    /** Calls {@code action} with {@code ChunkPos.asLong(cx, cz)} for every band chunk. */
    public void forEachBandChunk(LongConsumer action) {
        for (WrapEdge e : edgeArray) {
            for (int cz = e.bandMinZ() >> 4; cz <= (e.bandMaxZ() - 1) >> 4; cz++)
                for (int cx = e.bandMinX() >> 4; cx <= (e.bandMaxX() - 1) >> 4; cx++)
                    // Report each chunk once: under the edge its first block folds by.
                    if (beyondEdge(cx << 4, cz << 4) == e)
                        action.accept((long) cx & 0xffffffffL | ((long) cz & 0xffffffffL) << 32);
        }
    }

    @Override
    public String toString() {
        return "WrapTopology[" + name + ", " + edgeArray.length + " edges" + (corners.isEmpty() ? "" : ", " + corners.size() + " corners")
                + (periodX == 0 ? "" : ", period " + periodX + " from " + windowLo) + "]";
    }
}
