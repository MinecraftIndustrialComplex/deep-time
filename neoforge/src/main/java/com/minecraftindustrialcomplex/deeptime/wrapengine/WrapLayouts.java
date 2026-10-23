// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;

import java.util.ArrayList;
import java.util.List;

/**
 * The wrap engine's layouts as edge tables (docs/design/wrap-engine.md §1). The engine takes any table
 * of translations, half-turns and quarter-turns; this class builds the cube world's (§12, from
 * {@link CubeLayout}), the older candidates (Guyou, Peirce) and the bounded seams the GameTests use.
 *
 * <p>{@code slides}: whether the engine also owns the translation edges (the east–west wrap). Ben,
 * 2026-10-03: "TW slides, engine turns": in a real world Toroidal World keeps the slide and the engine
 * owns only the turning edges, so {@code slides = false} there; the unit tests and a dev world without
 * Toroidal World use {@code true}. A turning edge whose motion includes a translation (the cube's
 * pieces cut at Toroidal World's seam) is still the engine's.
 *
 * <p>Every layout is chunk-aligned when its size is a multiple of 32 (Guyou {@code h}, Peirce
 * {@code a}, the cube's {@code s}) and its depth a multiple of 16, as are the bounded seams' pivots,
 * spans and depths.
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

    /** Suffix of the edge pieces the cut at Toroidal World's seam makes (design §12.3). */
    public static final String TW_PIECE = "@tw";
    /** Suffix of the engine's own slide edges (a cube without Toroidal World's loop). */
    public static final String SLIDE = "-slide";

    /**
     * The cube world's net ({@link CubeLayout}, phase 0's fold W2: the one geometry, pinned against
     * Rust by the parity fixture) as the engine's topology, in Toroidal World's frame (design §12):
     * <ul>
     *   <li>the 12 turning net edges of {@code CubeLayout.edges()} in its order, D = {@code depth}
     *       deep (a multiple of 16, at most s/2; ledger: SCL-185);</li>
     *   <li>N-east and N-west (S-east, S-west) run D past N's (S's) outer corners and N-top (S-bottom)
     *       D past both, so the convex quadrants there are in bands (SCL-181);</li>
     *   <li>Toroidal World's window is x ∈ [−2s, 2s), so F2-top and F2-bottom are cut at x = 2s, their
     *       east pieces moved to [−2s, −3s/2) with the translation 4s composed in, and N-top and
     *       S-bottom are cut at x = 0, where their images leave the window (pieces named
     *       {@code <edge>@tw}, each next to its parent so ties break as in {@code CubeLayout});</li>
     *   <li>with {@code slides} (a level without Toroidal World's loop: GameTests, a dev world) the
     *       engine's own slide edges at x = ±2s over F2's rows and bands come first; without it the
     *       topology has period 4s from −2s and leaves the slide to Toroidal World;</li>
     *   <li>the eight cube corners ({@code CubeLayout.corners()}) as cone points (SCL-182).</li>
     * </ul>
     * The motions are {@code CubeLayout}'s; the unit tests check that the fold equals
     * {@code CubeLayout.fold} (x modulo 4s, the same turn) in every band.
     */
    public static WrapTopology cube(CubeLayout layout, int depth, boolean slides) {
        int s = Math.toIntExact(layout.side());
        int h = s / 2, d = depth, p = 4 * s;
        if (d <= 0 || d > h)
            throw new IllegalArgumentException("cube band depth " + d + " must be in [1, s/2 = " + h + "]");
        CellIsometry toEast = CellIsometry.translation(p, 0), toWest = CellIsometry.translation(-p, 0);
        List<WrapEdge> e = new ArrayList<>();
        if (slides) {
            e.add(new WrapEdge("west" + SLIDE, true, true, -2 * s, -h - d, h + d, d, toEast));
            e.add(new WrapEdge("east" + SLIDE, true, false, 2 * s, -h - d, h + d, d, toWest));
        }
        for (CubeLayout.NetEdge n : layout.edges()) {
            if (n.motion() == CubeLayout.EdgeMotion.SLIDE)
                continue;
            int from = Math.toIntExact(n.spanFrom()), to = Math.toIntExact(n.spanTo());
            CubeLayout.Face f = n.face();
            CubeLayout.FaceSide side = n.faceSide();
            boolean polarSide = side == CubeLayout.FaceSide.EAST || side == CubeLayout.FaceSide.WEST;
            // ledger: SCL-181 (the convex quadrants at N's and S's outer corners, in bands)
            if (f == CubeLayout.Face.N && polarSide)
                from -= d;
            else if (f == CubeLayout.Face.S && polarSide)
                to += d;
            else if (f == CubeLayout.Face.N || f == CubeLayout.Face.S) {
                from -= d;
                to += d;
            }
            boolean normalX = n.axis() == CubeLayout.Axis.X;
            boolean minSide = n.beyond() == CubeLayout.Side.MIN;
            int coord = Math.toIntExact(n.coord());
            CellIsometry iso = n.iso().toCellIsometry();
            if (f == CubeLayout.Face.F2) {
                // ledger: SCL-180 (the cut at Toroidal World's seam x = 2s: the east piece lives at [−2s, −3s/2))
                e.add(new WrapEdge(n.name(), normalX, minSide, coord, from, 2 * s, d, iso));
                e.add(new WrapEdge(n.name() + TW_PIECE, normalX, minSide, coord, 2 * s - p, to - p, d, iso.compose(toEast)));
            } else if ((f == CubeLayout.Face.N || f == CubeLayout.Face.S) && !polarSide) {
                // Cut at x = 0: the west half shows F2's east half, which lives at [−2s, −3s/2).
                e.add(new WrapEdge(n.name(), normalX, minSide, coord, 0, to, d, iso));
                e.add(new WrapEdge(n.name() + TW_PIECE, normalX, minSide, coord, from, 0, d, toWest.compose(iso)));
            } else {
                e.add(new WrapEdge(n.name(), normalX, minSide, coord, from, to, d, iso));
            }
        }
        List<WrapCorner> corners = cubeCorners(layout, slides ? d : -1); // ledger: SCL-182 (eight cone points of 270°)
        String name = "cube s=" + s + " depth " + d + (slides ? "" : " (turns only: Toroidal World slides)");
        return slides ? new WrapTopology(name, e, corners, 0, 0) : new WrapTopology(name, e, corners, p, -2 * s);
    }

    /** {@link #cube}({@code CubeLayout.of(s)}, depth, slides). */
    public static WrapTopology cubeCross(int s, int depth, boolean slides) {
        return cube(CubeLayout.of(s), depth, slides);
    }

    /** The cube's corner images in Toroidal World's frame, {x, z} each (for tests and diagnostics). */
    public static int[][] cubeCrossCorners(int s) {
        List<int[]> out = new ArrayList<>();
        for (WrapCorner c : cubeCorners(CubeLayout.of(s), -1))
            out.addAll(List.of(c.images()));
        return out.toArray(new int[0][]);
    }

    /**
     * {@code CubeLayout.corners()} in Toroidal World's frame: images at x ≥ 2s move by −4s (the third
     * image of the F2/F3 corners then coincides with the second). With {@code slideDepth} ≥ 0 (the
     * engine slides) the images one period away that lie within the slide bands are added.
     */
    private static List<WrapCorner> cubeCorners(CubeLayout layout, int slideDepth) {
        long s = layout.side(), p = 4 * s;
        List<WrapCorner> out = new ArrayList<>();
        for (CubeLayout.Corner c : layout.corners()) {
            List<int[]> imgs = new ArrayList<>();
            for (long[] q : c.images()) {
                long x = q[0] >= 2 * s ? q[0] - p : q[0];
                addImage(imgs, x, q[1]);
                if (slideDepth >= 0) {
                    long reach = 2 * s + slideDepth + WrapTopology.CORNER_KEEP_OUT;
                    for (long y : new long[]{x - p, x + p})
                        if (y >= -reach && y <= reach)
                            addImage(imgs, y, q[1]);
                }
            }
            out.add(new WrapCorner(c.name(), imgs.toArray(new int[0][])));
        }
        return out;
    }

    private static void addImage(List<int[]> imgs, long x, long z) {
        for (int[] q : imgs)
            if (q[0] == x && q[1] == z)
                return;
        imgs.add(new int[]{Math.toIntExact(x), Math.toIntExact(z)});
    }
}
