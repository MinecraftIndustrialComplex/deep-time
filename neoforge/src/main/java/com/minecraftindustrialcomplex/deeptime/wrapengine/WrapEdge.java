// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

/**
 * One edge of the fundamental domain and the motion that brings what lies beyond it back inside
 * (wrap engine, docs/design/wrap-engine.md §1).
 *
 * <p>The edge is the line {@code normal = coord} (normal axis x or z). Beyond it is {@code v < coord}
 * for a minimum edge and {@code v >= coord} for a maximum edge, restricted to {@code [spanLo, spanHi)}
 * along the edge and to {@code depth} blocks out: the band. {@code iso} maps a block beyond the edge to
 * the block inside it that it is. Spans and depth are finite: the engine's fast path is a grid over
 * the bands' bounding box.
 *
 * @param name     for logs
 * @param normalX  true when the edge's normal is x (an east or west edge), false for z (north, south)
 * @param minSide  true when beyond is the low side ({@code v < coord})
 * @param coord    the edge line, a block boundary
 * @param spanLo   first block along the edge
 * @param spanHi   one past the last block along the edge
 * @param depth    how far beyond the edge positions are folded (the band depth)
 * @param iso      beyond → inside
 */
public record WrapEdge(String name, boolean normalX, boolean minSide, int coord, int spanLo, int spanHi, int depth,
                       CellIsometry iso) {

    public WrapEdge {
        if (depth <= 0 || spanHi <= spanLo)
            throw new IllegalArgumentException("wrap edge " + name + ": empty band (depth " + depth + ", span " + spanLo + ".." + spanHi + ")");
        if ((long) spanHi - spanLo > 1L << 26 || depth > 1 << 20)
            throw new IllegalArgumentException("wrap edge " + name + ": spans and depth must be finite world sizes");
    }

    /** True when the block (x, z) lies beyond this edge, in its band. */
    public boolean beyond(int x, int z) {
        int v = normalX ? x : z;
        int u = normalX ? z : x;
        if (u < spanLo || u >= spanHi)
            return false;
        if (minSide)
            return v < coord && v >= coord - depth;
        return v >= coord && v < coord + depth;
    }

    /** True when the point (x, z) lies beyond this edge, in its band (entities). */
    public boolean beyondPoint(double x, double z) {
        double v = normalX ? x : z;
        double u = normalX ? z : x;
        if (u < spanLo || u >= spanHi)
            return false;
        if (minSide)
            return v < coord && v >= (double) coord - depth;
        return v >= coord && v < (double) coord + depth;
    }

    /** Distance in blocks from the block (x, z) to the edge line, measured along the normal (0 = touching). */
    public int distanceInside(int x, int z) {
        int v = normalX ? x : z;
        return minSide ? v - coord : coord - 1 - v;
    }

    /** The band's first x (inclusive). */
    public int bandMinX() {
        return normalX ? (minSide ? coord - depth : coord) : spanLo;
    }

    /** The band's last x (exclusive). */
    public int bandMaxX() {
        return normalX ? (minSide ? coord : coord + depth) : spanHi;
    }

    /** The band's first z (inclusive). */
    public int bandMinZ() {
        return normalX ? spanLo : (minSide ? coord - depth : coord);
    }

    /** The band's last z (exclusive). */
    public int bandMaxZ() {
        return normalX ? spanHi : (minSide ? coord : coord + depth);
    }

    /** True when the edge line, its span ends and its depth lie on chunk lines, and its motion maps chunks to chunks. */
    public boolean chunkAligned() {
        return (coord & 15) == 0 && (spanLo & 15) == 0 && (spanHi & 15) == 0 && (depth & 15) == 0 && iso.mapsChunksToChunks();
    }
}
