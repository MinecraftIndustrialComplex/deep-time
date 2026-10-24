// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeMap;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeTurn;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import java.util.TreeSet;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The Create World screen on a cube world (cube plan G13): the globe's outline of the cube's edges
 * and corners ({@link CubeOutline}, {@link GlobeProjection#drawOutline}) and the review page's net
 * ({@link NetView}, {@link ReviewModel#hintsOverlay}), on cubes turned three ways.
 */
class CubeNetViewsTest {

    static final long S = 1024;

    static List<WorldProjection.Cube> cubes() {
        return List.of(
                new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.IDENTITY, 1.0)),
                new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.polar(30.0), 1.0)),
                new WorldProjection.Cube(new CubeWorld(CubeLayout.of(S), CubeMap.world(), CubeTurn.fromRows(rotation(1, 2, 3, 40.0)), 1.0)));
    }

    /**
     * Twelve edges: the six the border closes (N and S against F1, F2 and F3), the slide (F2 against
     * F3) and the five the net joins; eight corners, of which the four touching F0 are reachable.
     */
    @Test
    void theOutlineHasTheCubesEdgesAndCorners() {
        for (WorldProjection.Cube wp : cubes()) {
            CubeOutline o = CubeOutline.of(wp);
            Map<CubeOutline.Kind, Set<String>> byKind = new HashMap<>();
            for (CubeOutline.Edge e : o.edges())
                byKind.computeIfAbsent(e.kind(), k -> new TreeSet<>()).add(e.name());
            assertEquals(12, o.edges().size());
            assertEquals(new TreeSet<>(Set.of("F1|N", "F2|N", "F3|N", "F1|S", "F2|S", "F3|S")), byKind.get(CubeOutline.Kind.BORDER));
            assertEquals(Set.of("F2|F3"), byKind.get(CubeOutline.Kind.SLIDE));
            assertEquals(new TreeSet<>(Set.of("F0|F1", "F1|F2", "F0|F3", "F0|N", "F0|S")), byKind.get(CubeOutline.Kind.JOINED));
            Set<String> reachable = new TreeSet<>();
            for (CubeOutline.Corner c : o.corners())
                if (c.reachable())
                    reachable.add(c.name());
            assertEquals(8, o.corners().size());
            assertEquals(new TreeSet<>(Set.of("n0", "n3", "s0", "s3")), reachable);
        }
    }

    /** Each edge runs from one corner to another along the great circle through them (the faces' sides are arcs). */
    @Test
    void edgesRunBetweenCornersOnGreatCircles() {
        for (WorldProjection.Cube wp : cubes()) {
            CubeOutline o = CubeOutline.of(wp);
            for (CubeOutline.Edge e : o.edges()) {
                double[][] pts = e.latLon();
                assertEquals(CubeOutline.SAMPLES + 1, pts.length);
                double[] a = unit(pts[0]), b = unit(pts[pts.length - 1]);
                assertTrue(nearestCorner(o, a) < 1e-7, e.name() + " starts at a corner: " + nearestCorner(o, a));
                assertTrue(nearestCorner(o, b) < 1e-7, e.name() + " ends at a corner: " + nearestCorner(o, b));
                double[] n = cross(a, b);
                double len = Math.sqrt(dot(n, n));
                double last = -1.0;
                for (double[] p : pts) {
                    double[] u = unit(p);
                    assertTrue(Math.abs(dot(u, n)) / len < 1e-6, e.name() + " leaves the great circle by " + Math.abs(dot(u, n)) / len);
                    double along = Math.acos(Math.max(-1.0, Math.min(1.0, dot(a, u))));
                    assertTrue(along >= last - 1e-12, e.name() + " runs one way");
                    last = along;
                }
            }
        }
    }

    /** The globe draws a visible border edge in the border's colour and a reachable corner in white, on the near side only. */
    @Test
    void theGlobeDrawsTheOutline() {
        WorldProjection.Cube wp = cubes().get(1);
        CubeOutline o = CubeOutline.of(wp);
        int w = 360, h = 114, size = 256;
        int[] src = new int[w * h];
        java.util.Arrays.fill(src, 0xff336633);
        // Look at the middle of a border edge, then at a reachable corner.
        CubeOutline.Edge edge = o.edges().stream().filter(e -> e.kind() == CubeOutline.Kind.BORDER).findFirst().orElseThrow();
        double[] mid = edge.latLon()[CubeOutline.SAMPLES / 2];
        GlobeProjection g = new GlobeProjection(mid[0], mid[1]);
        int[] dst = new int[size * size];
        g.render(src, w, h, dst, size);
        int[] plain = dst.clone();
        g.drawOutline(o, dst, size);
        assertEquals(GlobeProjection.BORDER_ABGR, near(dst, size, size / 2, size / 2, GlobeProjection.BORDER_ABGR),
                "the border edge under the globe's centre is drawn");
        int changed = 0;
        for (int i = 0; i < dst.length; i++) {
            if (plain[i] == 0)
                assertEquals(0, dst[i], "nothing is drawn off the disk");
            if (dst[i] != plain[i])
                changed++;
        }
        assertTrue(changed > 200 && changed < dst.length / 10, "lines, not a fill: " + changed);

        CubeOutline.Corner corner = o.corners().stream().filter(CubeOutline.Corner::reachable).findFirst().orElseThrow();
        GlobeProjection gc = new GlobeProjection(corner.lat(), corner.lon());
        int[] dc = new int[size * size];
        gc.render(src, w, h, dc, size);
        gc.drawOutline(o, dc, size);
        assertEquals(GlobeProjection.CORNER_ABGR, near(dc, size, size / 2, size / 2, GlobeProjection.CORNER_ABGR), "a reachable corner in white");
        CubeOutline.Corner blocked = o.corners().stream().filter(c -> !c.reachable()).findFirst().orElseThrow();
        GlobeProjection gb = new GlobeProjection(blocked.lat(), blocked.lon());
        int[] db = new int[size * size];
        gb.render(src, w, h, db, size);
        gb.drawOutline(o, db, size);
        assertEquals(GlobeProjection.CORNER_BLOCKED_ABGR, db[(size / 2) * size + size / 2], "a corner behind the border in its colour");
    }

    /** The review's net: the window's fractions, the faces and the void, the lines, and the picture. */
    @Test
    void theNetViewIsTheWindow() {
        Random rnd = new Random(13);
        for (WorldProjection.Cube wp : cubes()) {
            NetView nv = new NetView(wp);
            CubeLayout cube = wp.cube();
            long h = S / 2;
            // The window's corners and F0's centre.
            assertArrayEquals2(new double[]{-3.0 * h, -3.0 * h}, nv.blockAt(0, 0));
            assertArrayEquals2(new double[]{5.0 * h, 3.0 * h}, nv.blockAt(1, 1));
            assertArrayEquals2(new double[]{0.375, 0.5}, nv.fractionOf(0, 0));
            // Faces and the void.
            for (CubeLayout.Face f : CubeLayout.Face.all()) {
                double[] c = nv.fractionOf(cube.faceCentreX(f), cube.faceCentreZ(f));
                assertEquals(f, nv.faceAt(c[0], c[1]));
                assertArrayEquals2(wp.latLon(cube.faceCentreX(f), cube.faceCentreZ(f)), nv.latLonAt(c[0], c[1]));
            }
            assertNull(nv.faceAt(0.05, 0.05));
            assertNull(nv.latLonAt(0.95, 0.95));
            // A point on a face goes to its latitude and longitude and back.
            for (int k = 0; k < 200; k++) {
                CubeLayout.Face f = CubeLayout.Face.all()[rnd.nextInt(6)];
                double[] p = cube.pointOfLocal(f, rnd.nextDouble(-0.98, 0.98), rnd.nextDouble(-0.98, 0.98));
                double[] fr = nv.fractionOf(p[0], p[1]);
                double[] ll = nv.latLonAt(fr[0], fr[1]);
                assertNotNull(ll);
                double[] back = nv.fractionOfLatLon(ll[0], ll[1]);
                assertTrue(Math.abs(back[0] - fr[0]) < 1e-6 && Math.abs(back[1] - fr[1]) < 1e-6,
                        f + " (" + fr[0] + ", " + fr[1] + ") came back at (" + back[0] + ", " + back[1] + ")");
            }
            // The lines: the border's twelve are the wrap engine's net edges, then the five joined sides.
            Set<String> border = new HashSet<>(), wanted = new HashSet<>();
            int joined = 0;
            for (NetView.Line l : nv.lines()) {
                if (l.kind() == NetView.LineKind.BORDER)
                    border.add(key(nv.blockAt(l.fx0(), l.fy0()), nv.blockAt(l.fx1(), l.fy1())));
                else
                    joined++;
            }
            for (CubeLayout.NetEdge e : cube.edges())
                if ("wrap_engine".equals(e.owner()))
                    wanted.add(e.axis() == CubeLayout.Axis.X
                            ? key(new double[]{e.coord(), e.spanFrom()}, new double[]{e.coord(), e.spanTo()})
                            : key(new double[]{e.spanFrom(), e.coord()}, new double[]{e.spanTo(), e.coord()}));
            assertEquals(12, border.size());
            assertEquals(wanted, border);
            assertEquals(5, joined);
            // The picture: the six faces painted, the void transparent (512 x 384 is 128 pixels a face side).
            int[] px = nv.picture(NetView.W, NetView.H, (x, z) -> 0xff112233);
            int painted = 0;
            for (int c : px)
                painted += c != 0 ? 1 : 0;
            assertEquals(6 * 128 * 128, painted);
        }
    }

    /** The province hints over the net colour the pixels whose latitude and longitude fall in a hinted cell, and only those. */
    @Test
    void theHintsOverlayFollowsTheCells() {
        WorldProjection.Cube wp = cubes().get(2);
        NetView nv = new NetView(wp);
        ProvinceHints.Cell cell = new ProvinceHints.Cell(3, 2, "deeptime:placer_au", 2.0);
        ReviewModel.MapImage o = ReviewModel.hintsOverlay(nv, List.of(cell), 128, 96);
        int rgb = ProvinceHints.colour("deeptime:placer_au");
        int abgr = 0x50 << 24 | (rgb & 0xff) << 16 | (rgb & 0xff00) | (rgb >> 16 & 0xff);
        int in = 0;
        for (int y = 0; y < 96; y++)
            for (int x = 0; x < 128; x++) {
                double[] ll = nv.latLonAt((x + 0.5) / 128, (y + 0.5) / 96);
                boolean inCell = ll != null && ll[1] >= cell.lonWest() && ll[1] < cell.lonEast() && ll[0] <= cell.latNorth() && ll[0] > cell.latSouth();
                assertEquals(inCell ? abgr : 0, o.abgr()[y * 128 + x], "pixel " + x + "," + y);
                in += inCell ? 1 : 0;
            }
        assertTrue(in > 50, "the cell covers part of the net: " + in);
    }

    // ------------------------------------------------------------------------------------------

    private static String key(double[] a, double[] b) {
        return Math.round(a[0]) + "," + Math.round(a[1]) + "-" + Math.round(b[0]) + "," + Math.round(b[1]);
    }

    private static void assertArrayEquals2(double[] want, double[] got) {
        assertTrue(Math.abs(want[0] - got[0]) < 1e-9 && Math.abs(want[1] - got[1]) < 1e-9,
                "(" + want[0] + ", " + want[1] + ") vs (" + got[0] + ", " + got[1] + ")");
    }

    /** {@code want} when a pixel within 2 of (x, y) has it, else the centre's colour. */
    private static int near(int[] dst, int size, int x, int y, int want) {
        for (int dy = -2; dy <= 2; dy++)
            for (int dx = -2; dx <= 2; dx++)
                if (dst[(y + dy) * size + x + dx] == want)
                    return want;
        return dst[y * size + x];
    }

    private static double nearestCorner(CubeOutline o, double[] u) {
        double best = Double.POSITIVE_INFINITY;
        for (CubeOutline.Corner c : o.corners()) {
            double[] cu = unit(new double[]{c.lat(), c.lon()});
            best = Math.min(best, Math.acos(Math.max(-1.0, Math.min(1.0, dot(cu, u)))));
        }
        return best;
    }

    private static double[] unit(double[] ll) {
        double p = Math.toRadians(ll[0]), l = Math.toRadians(ll[1]);
        return new double[]{Math.cos(p) * Math.cos(l), Math.cos(p) * Math.sin(l), Math.sin(p)};
    }

    private static double[] cross(double[] a, double[] b) {
        return new double[]{a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]};
    }

    private static double dot(double[] a, double[] b) {
        return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }

    /** The rotation by {@code deg} about the axis (a, b, c) (Rodrigues), as rows. */
    private static double[][] rotation(double a, double b, double c, double deg) {
        double n = Math.sqrt(a * a + b * b + c * c);
        double x = a / n, y = b / n, z = c / n, t = Math.toRadians(deg), co = Math.cos(t), si = Math.sin(t), k = 1 - co;
        return new double[][]{
                {co + x * x * k, x * y * k - z * si, x * z * k + y * si},
                {y * x * k + z * si, co + y * y * k, y * z * k - x * si},
                {z * x * k - y * si, z * y * k + x * si, co + z * z * k}};
    }
}
