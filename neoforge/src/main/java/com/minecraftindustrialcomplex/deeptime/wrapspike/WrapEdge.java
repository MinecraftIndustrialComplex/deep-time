// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

/**
 * One edge of the fundamental domain and the motion that brings what lies beyond it back inside
 * (dev-only wrap-engine spike, docs/design/wrap-engine.md §3).
 *
 * <p>The edge is the line {@code normal = coord} (normal axis x or z). Beyond it is {@code v < coord}
 * for a minimum edge and {@code v >= coord} for a maximum edge, restricted to {@code [spanLo, spanHi)}
 * along the edge and to {@code depth} blocks out (the band; {@link Integer#MAX_VALUE} for a whole-world
 * topology). {@code iso} maps a block beyond the edge to the block inside it that it is.
 *
 * @param name     for logs
 * @param normalX  true when the edge's normal is x (an east or west edge), false for z (north, south)
 * @param minSide  true when beyond is the low side ({@code v < coord})
 * @param coord    the edge line, a block boundary
 * @param spanLo   first block along the edge
 * @param spanHi   one past the last block along the edge
 * @param depth    how far beyond the edge positions are folded
 * @param iso      beyond → inside
 */
public record WrapEdge(String name, boolean normalX, boolean minSide, int coord, int spanLo, int spanHi, int depth,
                       CellIsometry iso) {

    /** True when the block (x, z) lies beyond this edge, in its band. */
    public boolean beyond(int x, int z) {
        int v = normalX ? x : z;
        int u = normalX ? z : x;
        if (u < spanLo || u >= spanHi)
            return false;
        if (minSide)
            return v < coord && (depth == Integer.MAX_VALUE || v >= coord - depth);
        return v >= coord && (depth == Integer.MAX_VALUE || v < coord + depth);
    }

    /** True when the point (x, z) lies beyond this edge, in its band (entities). */
    public boolean beyondPoint(double x, double z) {
        double v = normalX ? x : z;
        double u = normalX ? z : x;
        if (u < spanLo || u >= spanHi)
            return false;
        if (minSide)
            return v < coord && (depth == Integer.MAX_VALUE || v >= (double) coord - depth);
        return v >= coord && (depth == Integer.MAX_VALUE || v < (double) coord + depth);
    }

    /** Distance in blocks from the block (x, z) to the edge line, measured along the normal (0 = touching). */
    public int distanceInside(int x, int z) {
        int v = normalX ? x : z;
        return minSide ? v - coord : coord - 1 - v;
    }
}
