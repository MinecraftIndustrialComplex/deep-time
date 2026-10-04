// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/**
 * A fundamental domain and the motions that glue its edges (dev-only wrap-engine spike,
 * docs/design/wrap-engine.md §3). Any set of edges whose motions are translations, half-turns and
 * quarter-turns: Toroidal World's slide, Guyou's two half-turn edges, Peirce's four.
 *
 * <p>{@link #fold} takes a block anywhere in the bands to the one stored block it is, with the motion
 * that took it there (so a block state read there is turned back by its inverse). {@link #images} is
 * the reverse: every band block that shows a given stored block, which the engine keeps as rotated
 * replicas so that the client, collisions and light see the far side without knowing about the seam.
 * Pure integer code, unit-tested in WrapTopologyTest.
 */
public final class WrapTopology {

    /** Bound on folding steps: a corner needs two; more means the edges do not close up. */
    static final int MAX_STEPS = 8;
    /** Bound on the length of the edge words {@link #images} tries (corners: 2, a turn point region: 3). */
    static final int MAX_WORD = 3;

    private final List<WrapEdge> edges;

    public WrapTopology(List<WrapEdge> edges) {
        this.edges = List.copyOf(edges);
    }

    public List<WrapEdge> edges() {
        return edges;
    }

    /**
     * Where a block beyond an edge is stored.
     *
     * @param x   stored block x
     * @param z   stored block z
     * @param iso the motion from the queried block to the stored one
     */
    public record Fold(int x, int z, CellIsometry iso) {}

    /** The stored block for (x, z), or {@code null} when (x, z) is inside (the fast path). */
    public Fold fold(int x, int z) {
        CellIsometry acc = null;
        int cx = x, cz = z;
        for (int step = 0; step < MAX_STEPS; step++) {
            WrapEdge hit = null;
            for (WrapEdge e : edges) {
                if (e.beyond(cx, cz)) {
                    hit = e;
                    break;
                }
            }
            if (hit == null)
                return acc == null ? null : new Fold(cx, cz, acc);
            int[] p = hit.iso().apply(cx, cz);
            cx = p[0];
            cz = p[1];
            acc = acc == null ? hit.iso() : hit.iso().compose(acc);
        }
        throw new IllegalStateException("wrap topology does not close: (" + x + ", " + z + ") still beyond an edge after "
                + MAX_STEPS + " steps");
    }

    /** A point beyond an edge, folded (entities): {x, z} and the motion, or {@code null} when inside. */
    public record PointFold(double x, double z, CellIsometry iso) {}

    public PointFold foldPoint(double x, double z) {
        CellIsometry acc = null;
        double cx = x, cz = z;
        for (int step = 0; step < MAX_STEPS; step++) {
            WrapEdge hit = null;
            for (WrapEdge e : edges) {
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
        throw new IllegalStateException("wrap topology does not close at point (" + x + ", " + z + ")");
    }

    /** True when (x, z) is a stored block (not beyond any edge). */
    public boolean inside(int x, int z) {
        for (WrapEdge e : edges)
            if (e.beyond(x, z))
                return false;
        return true;
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
                for (WrapEdge e : edges) {
                    int[] q = e.iso().inverse().apply(s[0], s[1]);
                    if (!seen.add(key(q[0], q[1])))
                        continue;
                    if (inside(q[0], q[1]))
                        continue;
                    Fold f = fold(q[0], q[1]);
                    if (f != null && f.x() == x && f.z() == z) {
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

    // --- Layouts -------------------------------------------------------------------------------

    /**
     * One half-turn seam, for the spike's test sites: the stored side is z ≥ {@code z0}; the band is
     * z ∈ [z0 − depth, z0) for x ∈ [cx − halfSpan, cx + halfSpan), glued by the half-turn about the
     * point (cx, z0): block (x, z) beyond is block (2cx − x − 1, 2z0 − z − 1).
     */
    public static WrapTopology halfTurnSeam(int cx, int z0, int halfSpan, int depth) {
        return new WrapTopology(List.of(new WrapEdge("north-half-turn", false, true, z0, cx - halfSpan, cx + halfSpan, depth,
                CellIsometry.halfTurnAbout(cx, z0))));
    }

    /**
     * The Guyou layout of docs/design/seamless-wrapping.md §4: the rectangle x ∈ [−h, h), z ∈ [−h/2, h/2)
     * (width w = 2h, height h); the east and west edges slide by w, the north and south edges turn half
     * about their midpoints (0, ∓h/2). {@code depth}: the band folded beyond each edge.
     */
    public static WrapTopology guyou(int h, int depth) {
        int w = 2 * h, hx = h, hz = h / 2;
        int lo = Integer.MIN_VALUE, hi = Integer.MAX_VALUE;
        return new WrapTopology(List.of(
                new WrapEdge("west-slide", true, true, -hx, lo, hi, depth, CellIsometry.translation(w, 0)),
                new WrapEdge("east-slide", true, false, hx, lo, hi, depth, CellIsometry.translation(-w, 0)),
                new WrapEdge("north-half-turn", false, true, -hz, lo, hi, depth, CellIsometry.halfTurnAbout(0, -hz)),
                new WrapEdge("south-half-turn", false, false, hz, lo, hi, depth, CellIsometry.halfTurnAbout(0, hz))));
    }

    /** Peirce's square, side a (even), centred on the origin: every edge turns half about its midpoint. */
    public static WrapTopology peirce(int a, int depth) {
        int s = a / 2;
        int lo = Integer.MIN_VALUE, hi = Integer.MAX_VALUE;
        return new WrapTopology(List.of(
                new WrapEdge("west-half-turn", true, true, -s, lo, hi, depth, CellIsometry.halfTurnAbout(-s, 0)),
                new WrapEdge("east-half-turn", true, false, s, lo, hi, depth, CellIsometry.halfTurnAbout(s, 0)),
                new WrapEdge("north-half-turn", false, true, -s, lo, hi, depth, CellIsometry.halfTurnAbout(0, -s)),
                new WrapEdge("south-half-turn", false, false, s, lo, hi, depth, CellIsometry.halfTurnAbout(0, s))));
    }

    /**
     * A quarter-turn gluing, as two faces of a cube net meet at a shared vertex: the stored side is the
     * quadrant x ≥ x0, z ≥ z0; the band beyond the north edge (z &lt; z0, x ≥ x0) is the strip inside the
     * west edge turned a quarter about the corner (x0, z0), and the band beyond the west edge (x &lt; x0,
     * z ≥ z0) the strip inside the north edge turned back. Used by the unit tests, to show the algebra
     * closes for quarter turns as well as half-turns.
     */
    public static WrapTopology quarterTurnCorner(int x0, int z0, int depth) {
        int hi = Integer.MAX_VALUE;
        return new WrapTopology(List.of(
                new WrapEdge("north-quarter-turn", false, true, z0, x0, hi, depth, CellIsometry.quarterTurnAbout(1, x0, z0)),
                new WrapEdge("west-quarter-turn", true, true, x0, z0, hi, depth, CellIsometry.quarterTurnAbout(3, x0, z0))));
    }
}
