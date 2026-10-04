// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import java.util.ArrayList;
import java.util.List;

/**
 * The wrap engine's layouts as edge tables (docs/design/wrap-engine.md §1). The production layout is
 * not chosen yet (Guyou in the plan, an equal-area cube cross net in the projection survey), so the
 * engine takes any table of translations, half-turns and quarter-turns, and this class builds the
 * candidates plus the bounded seams the GameTests use.
 *
 * <p>{@code slides}: whether the engine also owns the translation edges (the east–west wrap). Ben,
 * 2026-10-03: "TW slides, engine turns": in a real world Toroidal World keeps the slide and the engine
 * owns only the turning edges, so {@code slides = false} there; the unit tests and a dev world without
 * Toroidal World use {@code true}. A turning edge whose motion includes a translation (the cube's
 * polar faces against band face 3) is still the engine's.
 *
 * <p>Every layout is chunk-aligned when its size is a multiple of 32 (Guyou {@code h}, Peirce
 * {@code a}) or 16 (the cube's {@code s}, the bounded seams' pivots, spans and depths).
 */
public final class WrapLayouts {

    private WrapLayouts() {}

    /**
     * One half-turn seam (GameTests): the stored side is z ≥ {@code z0}; the band is
     * z ∈ [z0 − depth, z0) for x ∈ [cx − halfSpan, cx + halfSpan), glued by the half-turn about the
     * point (cx, z0): block (x, z) beyond is block (2cx − x − 1, 2z0 − z − 1).
     */
    public static WrapTopology halfTurnSeam(int cx, int z0, int halfSpan, int depth) {
        return new WrapTopology("half-turn seam at (" + cx + ", " + z0 + ")", List.of(
                new WrapEdge("north-half-turn", false, true, z0, cx - halfSpan, cx + halfSpan, depth, CellIsometry.halfTurnAbout(cx, z0))));
    }

    /**
     * A quarter-turn corner (GameTests), as two faces of a cube net meet at a shared vertex: the stored
     * side is the quadrant x ≥ x0, z ≥ z0; the band beyond the north edge (z &lt; z0, x ∈ [x0, x0 + span))
     * is the strip inside the west edge turned a quarter about the corner (x0, z0), and the band beyond
     * the west edge (x &lt; x0, z ∈ [z0, z0 + span)) the strip inside the north edge turned back. The
     * quadrant x &lt; x0, z &lt; z0 is the missing quarter of a cone point and stays unglued.
     */
    public static WrapTopology quarterTurnCorner(int x0, int z0, int span, int depth) {
        return new WrapTopology("quarter-turn corner at (" + x0 + ", " + z0 + ")", List.of(
                new WrapEdge("north-quarter-turn", false, true, z0, x0, x0 + span, depth, CellIsometry.quarterTurnAbout(1, x0, z0)),
                new WrapEdge("west-quarter-turn", true, true, x0, z0, z0 + span, depth, CellIsometry.quarterTurnAbout(3, x0, z0))));
    }

    /**
     * The Guyou layout of docs/design/seamless-wrapping.md §4: the rectangle x ∈ [−h, h), z ∈ [−h/2, h/2)
     * (width 2h, height h); the north and south edges turn half about their midpoints (0, ∓h/2); with
     * {@code slides} the engine also wraps east–west by 2h (otherwise Toroidal World does).
     */
    public static WrapTopology guyou(int h, int depth, boolean slides) {
        int w = 2 * h, hz = h / 2;
        List<WrapEdge> e = new ArrayList<>();
        if (slides) {
            // The slides' spans reach past the corners so a corner block slides first, then turns.
            e.add(new WrapEdge("west-slide", true, true, -h, -hz - depth, hz + depth, depth, CellIsometry.translation(w, 0)));
            e.add(new WrapEdge("east-slide", true, false, h, -hz - depth, hz + depth, depth, CellIsometry.translation(-w, 0)));
        }
        e.add(new WrapEdge("north-half-turn", false, true, -hz, -h, h, depth, CellIsometry.halfTurnAbout(0, -hz)));
        e.add(new WrapEdge("south-half-turn", false, false, hz, -h, h, depth, CellIsometry.halfTurnAbout(0, hz)));
        return new WrapTopology("guyou h=" + h + (slides ? "" : " (turns only)"), e);
    }

    /** Peirce's square, side a (even), centred on the origin: every edge turns half about its midpoint. */
    public static WrapTopology peirce(int a, int depth) {
        int s = a / 2;
        return new WrapTopology("peirce a=" + a, List.of(
                new WrapEdge("west-half-turn", true, true, -s, -s - depth, s + depth, depth, CellIsometry.halfTurnAbout(-s, 0)),
                new WrapEdge("east-half-turn", true, false, s, -s - depth, s + depth, depth, CellIsometry.halfTurnAbout(s, 0)),
                new WrapEdge("north-half-turn", false, true, -s, -s, s, depth, CellIsometry.halfTurnAbout(0, -s)),
                new WrapEdge("south-half-turn", false, false, s, -s, s, depth, CellIsometry.halfTurnAbout(0, s))));
    }

    /**
     * The cube in a cross-shaped net (docs/research/notes/seamless-projection-survey.md §3.3): four band
     * faces F0..F3 of side s along z ∈ [0, s), x ∈ [0, 4s) (the equator runs east–west and wraps by 4s),
     * the north polar face N at x ∈ [0, s), z ∈ [−s, 0) and the south polar face S at x ∈ [0, s),
     * z ∈ [s, 2s), both on F0. Seven seam pairs: the slide (with {@code slides}), four quarter-turns
     * (N and S against F1 about the net's concave corners; N and S against F3, a quarter-turn about the
     * corner at x = 0 followed by the 4s slide) and two half-turns (N's and S's far edges against F2).
     *
     * <p>The eight cube corners are cone points of 270°; near each the band shows one of the two faces
     * that meet there, as any flat net must, so blocks near a corner are not locally seamless (the unit
     * tests leave out a neighbourhood of each).
     */
    public static WrapTopology cubeCross(int s, int depth, boolean slides) {
        CellIsometry slideEast = CellIsometry.translation(4 * s, 0);
        CellIsometry nWest = slideEast.compose(CellIsometry.quarterTurnAbout(3, 0, 0));
        CellIsometry sWest = slideEast.compose(CellIsometry.quarterTurnAbout(1, 0, s));
        CellIsometry north = new CellIsometry(2, 3 * s, -s);      // (x, z) → (3s − x − 1, −s − z − 1)
        CellIsometry south = new CellIsometry(2, 3 * s, 3 * s);   // (x, z) → (3s − x − 1, 3s − z − 1)
        List<WrapEdge> e = new ArrayList<>();
        if (slides) {
            e.add(new WrapEdge("F0-west-slide", true, true, 0, 0, s, depth, slideEast));
            e.add(new WrapEdge("F3-east-slide", true, false, 4 * s, 0, s, depth, slideEast.inverse()));
        }
        e.add(new WrapEdge("N-east/F1", true, false, s, -s, 0, depth, CellIsometry.quarterTurnAbout(1, s, 0)));
        e.add(new WrapEdge("F1-top/N", false, true, 0, s, 2 * s, depth, CellIsometry.quarterTurnAbout(3, s, 0)));
        e.add(new WrapEdge("N-west/F3", true, true, 0, -s, 0, depth, nWest));
        e.add(new WrapEdge("F3-top/N", false, true, 0, 3 * s, 4 * s, depth, nWest.inverse()));
        e.add(new WrapEdge("N-north/F2", false, true, -s, 0, s, depth, north));
        e.add(new WrapEdge("F2-top/N", false, true, 0, 2 * s, 3 * s, depth, north));
        e.add(new WrapEdge("S-east/F1", true, false, s, s, 2 * s, depth, CellIsometry.quarterTurnAbout(3, s, s)));
        e.add(new WrapEdge("F1-bottom/S", false, false, s, s, 2 * s, depth, CellIsometry.quarterTurnAbout(1, s, s)));
        e.add(new WrapEdge("S-west/F3", true, true, 0, s, 2 * s, depth, sWest));
        e.add(new WrapEdge("F3-bottom/S", false, false, s, 3 * s, 4 * s, depth, sWest.inverse()));
        e.add(new WrapEdge("S-south/F2", false, false, 2 * s, 0, s, depth, south));
        e.add(new WrapEdge("F2-bottom/S", false, false, s, 2 * s, 3 * s, depth, south));
        return new WrapTopology("cube cross s=" + s + (slides ? "" : " (turns only)"), e);
    }

    /** The cube cross net's corners in the plane (the cone points' images), for tests and diagnostics. */
    public static int[][] cubeCrossCorners(int s) {
        return new int[][]{{0, 0}, {s, 0}, {2 * s, 0}, {3 * s, 0}, {4 * s, 0}, {0, s}, {s, s}, {2 * s, s}, {3 * s, s}, {4 * s, s},
                {0, -s}, {s, -s}, {0, 2 * s}, {s, 2 * s}};
    }
}
