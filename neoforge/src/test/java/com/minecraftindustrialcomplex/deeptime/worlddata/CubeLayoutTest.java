// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.Axis;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.EdgeMotion;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.Face;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.FaceCell;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.FaceSide;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.Fold;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.Iso;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.NetEdge;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.OnFace;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.Outside;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.PointOnFace;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.Side;
import com.minecraftindustrialcomplex.deeptime.wrapengine.CellIsometry;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEdge;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLayouts;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

/**
 * The cube world's net ({@link CubeLayout}, plan item W1 and W2) against its own invariants, the
 * Rust tests of {@code deeptime_format::cube} ported one for one, and the wrap engine's
 * {@link WrapLayouts#cubeCross} (the same net in the engine's frame). The fixture rows are checked
 * bit for bit by {@code ProjectionParityTest}.
 *
 * <p>Float comparisons use {@code ==}, as the Rust tests' {@code assert_eq!} does, never
 * {@code assertEquals(double, double)}: 0.0 and −0.0 are the same number here.
 */
class CubeLayoutTest {

    private static final int[] SIDES = {1_024, 3_840, 64};

    /** SplitMix64, as the parity fixture's. */
    private static final class Rng {
        private long state;

        Rng(long seed) {
            state = seed;
        }

        long next() {
            state += 0x9E37_79B9_7F4A_7C15L;
            long z = state;
            z = (z ^ (z >>> 30)) * 0xBF58_476D_1CE4_E5B9L;
            z = (z ^ (z >>> 27)) * 0x94D0_49BB_1331_11EBL;
            return z ^ (z >>> 31);
        }

        /** lo + next() % (hi − lo), the remainder unsigned as Rust's u64. */
        long range(long lo, long hi) {
            return lo + Long.remainderUnsigned(next(), hi - lo);
        }
    }

    private static void assertSameDouble(double want, double got, String what) {
        assertTrue(want == got, what + ": " + got + " != " + want);
    }

    /** Component-wise {@code ==} (Rust's PartialEq on arrays of f64): 0.0 equals -0.0. */
    private static boolean sameValues(double[] a, double[] b) {
        if (a.length != b.length)
            return false;
        for (int i = 0; i < a.length; i++)
            if (a[i] != b[i])
                return false;
        return true;
    }

    private static void assertSameDoubles(double[] want, double[] got, String what) {
        assertEquals(want.length, got.length, what);
        for (int i = 0; i < want.length; i++)
            assertSameDouble(want[i], got[i], what + "[" + i + "]");
    }

    // ------------------------------------------------------------------------------------------
    // W1
    // ------------------------------------------------------------------------------------------

    @Test
    void theFaceSideOfThePlanetSizes() {
        assertEquals(256, CubeLayout.FACE_QUANTUM);
        assertEquals(1_024, CubeLayout.faceFor(4_096));
        assertEquals(3_840, CubeLayout.faceFor(16_384));
        assertEquals(7_424, CubeLayout.faceFor(32_768));
        // The smallest planet still has a face, and the figure is a multiple of the quantum.
        assertEquals(256, CubeLayout.faceFor(1));
        assertEquals(256, CubeLayout.faceFor(0));
        for (int c : new int[]{512, 1_000, 2_735, 4_961, 9_000, 20_000, 65_536, 110_772, 131_072, 147_808, 172_981}) {
            int s = CubeLayout.faceFor(c);
            assertEquals(0, s % CubeLayout.FACE_QUANTUM, "c " + c);
            // Within half a quantum of the exact figure, or at the floor of one quantum.
            double exact = c / Math.sqrt(6.0 * Math.PI);
            assertTrue(s == CubeLayout.FACE_QUANTUM || Math.abs(s - exact) <= CubeLayout.FACE_QUANTUM / 2.0 + 1e-9, "c " + c + " s " + s);
        }
        // WorldDataRef.FACE_QUANTUM is a compile-time constant (inlined, so reading it does not load
        // WorldDataRef, whose codec needs DataFixerUpper, absent from the plain unit-test classpath);
        // WorldDataRef.faceFor forwards to CubeLayout.faceFor and is not called here for that reason.
        assertEquals(CubeLayout.FACE_QUANTUM, com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataRef.FACE_QUANTUM);
        // An unsigned 32-bit count, as Rust's u32: -1 is 4,294,967,295.
        assertTrue(CubeLayout.faceFor(-1) > CubeLayout.faceFor(Integer.MAX_VALUE));
        // The net of a planet size, and the sides it takes.
        assertEquals(3_840, CubeLayout.forCircumference(16_384).side());
        assertTrue(CubeLayout.of(3_840).quantised());
        assertFalse(CubeLayout.of(64).quantised());
        assertThrows(IllegalArgumentException.class, () -> CubeLayout.of(63));
        assertThrows(IllegalArgumentException.class, () -> CubeLayout.of(0));
        assertEquals(4 * 3_840L, CubeLayout.of(3_840).period());
        assertArrayEquals(new double[]{-1_536.0, 2_560.0, -1_536.0, 1_536.0}, CubeLayout.of(1_024).window());
    }

    // ------------------------------------------------------------------------------------------
    // The port of cube.rs's tests
    // ------------------------------------------------------------------------------------------

    @Test
    void isoAlgebraMatchesTheWrapEngine() {
        // CellIsometry's examples: a half turn about (a, b) on cells is (2a − x − 1, 2b − z − 1).
        Iso h = Iso.halfTurnAbout(3, -5);
        assertArrayEquals(new long[]{2 * 3 - 1 - 1, -10 - 2 - 1}, h.cell(1, 2));
        assertEquals(CellIsometry.halfTurnAbout(3, -5), h.toCellIsometry());
        long[][] pivots = {{0, 0}, {7, -3}, {-128, 64}};
        long[][] cells = {{0, 0}, {5, -9}, {-200, 31}};
        for (int k = 0; k < 4; k++) {
            for (long[] pv : pivots) {
                Iso q = Iso.quarterTurnAbout(k, pv[0], pv[1]);
                CellIsometry c = CellIsometry.quarterTurnAbout(k, (int) pv[0], (int) pv[1]);
                assertEquals(c, q.toCellIsometry(), "k " + k + " about " + Arrays.toString(pv));
                // The pivot is fixed as a point.
                double[] fixed = q.point((double) pv[0], (double) pv[1]);
                assertSameDouble((double) pv[0], fixed[0], "pivot x");
                assertSameDouble((double) pv[1], fixed[1], "pivot z");
                Iso inv = q.inverse();
                assertEquals(c.inverse(), inv.toCellIsometry());
                for (long[] xz : cells) {
                    long[] moved = q.cell(xz[0], xz[1]);
                    assertArrayEquals(xz, inv.cell(moved[0], moved[1]));
                    assertArrayEquals(xz, q.compose(inv).cell(xz[0], xz[1]));
                    assertArrayEquals(new long[]{c.applyX((int) xz[0], (int) xz[1]), c.applyZ((int) xz[0], (int) xz[1])}, moved);
                    // The point and vector of the wrap engine, bit for bit.
                    double[] pt = q.point(xz[0] + 0.25, xz[1] - 0.5);
                    double[] wp = c.applyPoint(xz[0] + 0.25, xz[1] - 0.5);
                    assertSameDoubles(wp, pt, "point");
                    assertSameDoubles(c.applyVector(2.0, -3.0), q.vector(2.0, -3.0), "vector");
                }
                // compose against the wrap engine's.
                Iso other = Iso.quarterTurnAbout(3, 11, -2);
                assertEquals(c.compose(CellIsometry.quarterTurnAbout(3, 11, -2)), q.compose(other).toCellIsometry());
            }
        }
        // Translations fitting an int cross to the wrap engine; others refuse.
        assertEquals(CellIsometry.translation(4_096, 0), Iso.translation(4_096, 0).toCellIsometry());
        assertThrows(ArithmeticException.class, () -> Iso.translation(1L << 40, 0).toCellIsometry());
        // k is taken mod 4; the array form is [k, tx, tz].
        assertEquals(new Iso(1, 2, 3), new Iso(5, 2, 3));
        assertEquals(new Iso(3, 0, 0), new Iso(-1, 0, 0));
        assertArrayEquals(new long[]{2, 6, -10}, Iso.halfTurnAbout(3, -5).asArray());
        assertEquals(Iso.IDENTITY, new Iso(0, 0, 0));
    }

    @Test
    void facesTileTheNet() {
        for (int si : SIDES) {
            CubeLayout cube = CubeLayout.of(si);
            long s = si;
            long[] count = new long[6];
            long step = Math.max(s / 64, 1);
            for (long z = -3 * s / 2; z < 3 * s / 2; z += step) {
                for (long x = -3 * s / 2; x < 5 * s / 2; x += step) {
                    FaceCell fc = cube.faceOfBlock(x, z);
                    if (fc == null)
                        continue;
                    assertTrue(fc.i() >= 0 && fc.i() < s && fc.j() >= 0 && fc.j() < s);
                    assertEquals(x, cube.blockX(fc.face(), fc.i()));
                    assertEquals(z, cube.blockZ(fc.face(), fc.j()));
                    count[fc.face().index()]++;
                    // The slide: the same block a period away.
                    assertEquals(fc, cube.faceOfBlock(x + 4 * s, z));
                    assertEquals(fc, cube.faceOfBlock(x - 8 * s, z));
                }
            }
            long per = (s / step) * (s / step);
            long[] want = new long[6];
            Arrays.fill(want, per);
            assertArrayEquals(want, count, "s " + s);
        }
    }

    @Test
    void framesAreRightHandedTheSameWay() {
        for (Face f : Face.all()) {
            double[][] fr = f.frame();
            double[] c = fr[0], eu = fr[1], ev = fr[2];
            double[] cross = {
                    eu[1] * ev[2] - eu[2] * ev[1],
                    eu[2] * ev[0] - eu[0] * ev[2],
                    eu[0] * ev[1] - eu[1] * ev[0]};
            assertSameDoubles(new double[]{-c[0], -c[1], -c[2]}, cross, f.name());
            assertEquals(f, Face.ofVector(c));
            // The frame accessor hands out copies.
            fr[0][0] = 99.0;
            assertTrue(f.frame()[0][0] != 99.0);
        }
        // Ties go to x, then y, then z; a zero component counts as positive.
        assertEquals(Face.F0, Face.ofVector(new double[]{1.0, 1.0, 1.0}));
        assertEquals(Face.F1, Face.ofVector(new double[]{0.5, 1.0, 1.0}));
        assertEquals(Face.N, Face.ofVector(new double[]{0.5, 0.5, 1.0}));
        assertEquals(Face.S, Face.ofVector(new double[]{0.5, 0.5, -1.0}));
        assertEquals(Face.F0, Face.ofVector(new double[]{0.0, 0.0, 0.0}));
        assertEquals(Face.F2, Face.ofVector(new double[]{-1.0, 0.0, 0.0}));
        assertEquals(Face.F3, Face.ofVector(new double[]{0.0, -1.0, 0.0}));
    }

    /** The cube-frame point of a gnomonic face point (A, B) (unnormalised). */
    private static double[] gnomonic(Face f, double a, double b) {
        double[][] fr = f.frame();
        double[] c = fr[0], eu = fr[1], ev = fr[2];
        double[] p = new double[3];
        for (int k = 0; k < 3; k++)
            p[k] = c[k] + a * eu[k] + b * ev[k];
        return p;
    }

    private static double[] gnomonicOf(Face f, double[] p) {
        double[][] fr = f.frame();
        double dc = dot(p, fr[0]), du = dot(p, fr[1]), dv = dot(p, fr[2]);
        return new double[]{du / dc, dv / dc};
    }

    private static double dot(double[] p, double[] q) {
        return p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
    }

    @Test
    void adjacencyMatchesTheCubeFrames() {
        // On an edge the map's local coordinate along the edge is the gnomonic one up to a fixed
        // odd function (the table's eightfold symmetry), so the gnomonic cube checks the edge
        // correspondence; the cell motion's direction across (into the neighbour) checks the rest.
        for (Face f : Face.all()) {
            for (FaceSide side : FaceSide.all()) {
                CubeLayout.Across a = f.across(side);
                assertSame(CubeLayout.adjacency(f, side), a);
                for (double t : new double[]{-0.75, -0.2, 0.0, 0.4, 1.0}) {
                    double u, v;
                    switch (side) {
                        case EAST -> { u = 1.0; v = t; }
                        case WEST -> { u = -1.0; v = t; }
                        case SOUTH -> { u = t; v = 1.0; }
                        default -> { u = t; v = -1.0; }
                    }
                    double[] p = gnomonic(f, u, v);
                    double[] uv2 = a.local(u, v);
                    double[] q = gnomonic(a.face(), uv2[0], uv2[1]);
                    for (int k = 0; k < 3; k++)
                        assertTrue(Math.abs(p[k] - q[k]) < 1e-12, f + " " + side + " t " + t + ": " + Arrays.toString(p) + " vs " + Arrays.toString(q));
                    double[] g = gnomonicOf(a.face(), p);
                    assertTrue(Math.abs(g[0] - uv2[0]) < 1e-12 && Math.abs(g[1] - uv2[1]) < 1e-12);
                }
                // A step out of f lands just inside the neighbour, on the side it names.
                double u, v;
                switch (side) {
                    case EAST -> { u = 1.01; v = 0.3; }
                    case WEST -> { u = -1.01; v = 0.3; }
                    case SOUTH -> { u = 0.3; v = 1.01; }
                    default -> { u = 0.3; v = -1.01; }
                }
                double[] uv2 = a.local(u, v);
                assertTrue(Math.abs(uv2[0]) < 1.0 && Math.abs(uv2[1]) < 1.0, f + " " + side + ": " + Arrays.toString(uv2));
                boolean near = switch (a.side()) {
                    case EAST -> uv2[0] > 0.98;
                    case WEST -> uv2[0] < -0.98;
                    case SOUTH -> uv2[1] > 0.98;
                    case NORTH -> uv2[1] < -0.98;
                };
                assertTrue(near, f + " " + side + " enters " + a.side() + " at " + Arrays.toString(uv2));
                // And back.
                CubeLayout.Across back = a.face().across(a.side());
                assertEquals(f, back.face());
                assertEquals(side, back.side());
                double[] uv3 = back.local(uv2[0], uv2[1]);
                assertTrue(Math.abs(uv3[0] - u) < 1e-12 && Math.abs(uv3[1] - v) < 1e-12);
                // The direction map is the quarter turns of the cell map.
                assertSameDoubles(new Iso(a.quarterTurns(), 0, 0).vector(1.0, 2.0), a.vector(1.0, 2.0), "vector");
            }
        }
    }

    @Test
    void adjacencyMatchesTheFold() {
        // With n = s the face grid is the net's blocks: a cell one step (or up to a face width)
        // beyond a side is the block the fold gives.
        for (int si : SIDES) {
            CubeLayout cube = CubeLayout.of(si);
            long n = si;
            for (Face f : Face.all()) {
                for (FaceSide side : FaceSide.all()) {
                    for (long depth : new long[]{0, 1, n / 3, n - 1}) {
                        for (long t : new long[]{0, 1, n / 2, n - 2, n - 1}) {
                            long i, j;
                            switch (side) {
                                case EAST -> { i = n + depth; j = t; }
                                case WEST -> { i = -1 - depth; j = t; }
                                case SOUTH -> { i = t; j = n + depth; }
                                default -> { i = t; j = -1 - depth; }
                            }
                            CubeLayout.CellOn on = CubeLayout.cellOn(f, n, i, j);
                            assertNotNull(on, f + " " + side + " (" + i + ", " + j + ")");
                            long x = cube.blockX(f, i), z = cube.blockZ(f, j);
                            long fx, fz;
                            int fk;
                            switch (cube.fold(x, z)) {
                                case OnFace of -> {
                                    fx = cube.blockX(of.face(), of.i());
                                    fz = cube.blockZ(of.face(), of.j());
                                    fk = 0;
                                }
                                case Outside out -> {
                                    fx = out.x();
                                    fz = out.z();
                                    fk = out.iso().k();
                                }
                                case CubeLayout.Ocean ocean -> {
                                    fail(f + " " + side + " (" + i + ", " + j + ") is ocean");
                                    return;
                                }
                            }
                            // A block in a void square by F0's corner may belong to the other
                            // edge's half (the fold splits the square on its diagonal); the
                            // adjacency's answer is then the other half's.
                            boolean diag = depth >= Math.min(t, n - 1 - t);
                            if (!diag) {
                                String what = "s " + si + " " + f + " " + side + " depth " + depth + " t " + t;
                                assertEquals(cube.blockX(on.face(), on.i()), fx, what);
                                assertEquals(cube.blockZ(on.face(), on.j()), fz, what);
                                assertEquals(on.quarterTurns(), fk, what);
                            }
                        }
                    }
                }
            }
        }
        // cellOn: on the face it is the face; in a corner square it is null; further than a face
        // width out it is null.
        assertEquals(new CubeLayout.CellOn(Face.F0, 3, 4, 0), CubeLayout.cellOn(Face.F0, 16, 3, 4));
        assertNull(CubeLayout.cellOn(Face.F0, 16, -1, -1));
        assertNull(CubeLayout.cellOn(Face.F0, 16, 16, 16));
        assertNull(CubeLayout.cellOn(Face.F0, 16, 32, 3));
        assertNull(CubeLayout.cellOn(Face.F0, 16, 3, -17));
        assertNotNull(CubeLayout.cellOn(Face.F0, 16, 31, 3));
        assertNotNull(CubeLayout.cellOn(Face.F0, 16, 3, -16));
    }

    @Test
    void seamsMapEdgeOntoEdgeAndClose() {
        for (int si : SIDES) {
            CubeLayout cube = CubeLayout.of(si);
            long s = si;
            List<NetEdge> edges = cube.edges();
            assertEquals(14, edges.size());
            for (int k = 0; k < edges.size(); k++) {
                NetEdge e = edges.get(k);
                NetEdge p = edges.get(e.partner());
                assertEquals(k, p.partner());
                assertEquals(e.seam(), p.seam());
                assertEquals(s, e.spanTo() - e.spanFrom());
                // Every block of the band's first row maps onto the partner edge's inner row, and
                // every block of the partner's band onto this edge's inner row: the motion takes
                // beyond-this-edge onto inside-the-partner.
                for (long t = e.spanFrom(); t < e.spanTo(); t++) {
                    long x, z;
                    if (e.axis() == Axis.X) {
                        x = e.beyond() == Side.MAX ? e.coord() : e.coord() - 1;
                        z = t;
                    } else {
                        x = t;
                        z = e.beyond() == Side.MAX ? e.coord() : e.coord() - 1;
                    }
                    long px = e.iso().cellX(x, z), pz = e.iso().cellZ(x, z);
                    boolean inner;
                    if (p.axis() == Axis.X)
                        inner = p.beyond() == Side.MAX ? px == p.coord() - 1 : px == p.coord();
                    else
                        inner = p.beyond() == Side.MAX ? pz == p.coord() - 1 : pz == p.coord();
                    assertTrue(inner, e.name() + ": (" + x + ", " + z + ") -> (" + px + ", " + pz + ") not on " + p.name());
                    long along = p.axis() == Axis.X ? pz : px;
                    assertTrue(along >= p.spanFrom() && along < p.spanTo());
                    FaceCell fc = cube.faceOfBlock(px, pz);
                    assertTrue(fc != null && fc.face() == p.face());
                }
                // The partner's motion is this one's inverse; turns close up.
                assertEquals(e.iso().inverse(), p.iso(), e.name() + " / " + p.name());
                int order = switch (e.motion()) {
                    case SLIDE -> 1;
                    case HALF_TURN -> 2;
                    case QUARTER_TURN -> 4;
                };
                Iso m = Iso.IDENTITY;
                for (int i = 0; i < order; i++)
                    m = e.iso().compose(m);
                if (e.motion() != EdgeMotion.SLIDE) {
                    assertEquals(Iso.IDENTITY, m, e.name() + " to the power " + order);
                    long[] pivot = e.pivot();
                    assertNotNull(pivot);
                    double[] fixed = e.iso().point((double) pivot[0], (double) pivot[1]);
                    assertSameDouble((double) pivot[0], fixed[0], e.name() + " pivot x");
                    assertSameDouble((double) pivot[1], fixed[1], e.name() + " pivot z");
                    assertEquals(0, Math.floorMod(pivot[0], 16L));
                    assertEquals(0, Math.floorMod(pivot[1], 16L));
                } else {
                    assertNull(e.pivot());
                    assertEquals("toroidal_world", e.owner());
                    assertEquals(0, e.iso().k());
                    assertEquals(0, e.iso().tz());
                    assertEquals(4 * s, Math.abs(e.iso().tx()));
                }
            }
            // The half turns do not move their pivot's blocks twice: a half turn is an involution
            // on cells too, and the order-2 closure above covers points; check one cell.
            NetEdge nn = edges.get(6);
            long x0 = 3, z0 = -s - 2;
            long[] once = nn.iso().cell(x0, z0);
            assertArrayEquals(new long[]{x0, z0}, nn.iso().cell(once[0], once[1]));
        }
    }

    /** Whether the block is within s (Chebyshev, the slide counted) of a planet block. */
    private static boolean withinS(CubeLayout cube, long x, long z) {
        long s = cube.side();
        long xw = cube.wrapX(x);
        for (Face f : Face.all()) {
            long ox = cube.faceOriginX(f), oz = cube.faceOriginZ(f);
            for (long o : new long[]{-4 * s, 0, 4 * s}) {
                long dx = Math.max(Math.max(ox + o - xw, xw - (ox + o + s - 1)), 0);
                long dz = Math.max(Math.max(oz - z, z - (oz + s - 1)), 0);
                if (Math.max(dx, dz) <= s)
                    return true;
            }
        }
        return false;
    }

    @Test
    void foldRoundTripsOutsideBlocks() {
        // Plan W2: fold(partner) is the identity on 10,000 random outside blocks within s of the
        // planet: the partner is a planet block, and the motion's inverse brings it back.
        for (int si : SIDES) {
            CubeLayout cube = CubeLayout.of(si);
            long s = si;
            Rng rng = new Rng(0x0DE5_7001L ^ s);
            int outside = 0, ocean = 0, tries = 0;
            while (outside < 10_000) {
                tries++;
                assertTrue(tries < 1_000_000);
                long x = rng.range(-5 * s / 2 - s, 7 * s / 2 + s);
                long z = rng.range(-5 * s / 2, 5 * s / 2);
                if (cube.faceOfBlock(x, z) != null || !withinS(cube, x, z))
                    continue;
                switch (cube.fold(x, z)) {
                    case Outside out -> {
                        outside++;
                        assertArrayEquals(new long[]{out.x(), out.z()}, out.iso().cell(x, z));
                        assertTrue(cube.fold(out.x(), out.z()) instanceof OnFace);
                        assertArrayEquals(new long[]{x, z}, out.iso().inverse().cell(out.x(), out.z()));
                        NetEdge e = cube.edges().get(out.edge());
                        assertTrue(e.depth(cube.wrapX(x), z) >= 0);
                        assertEquals(cube.edges().get(e.partner()).face(), cube.faceOfBlock(out.x(), out.z()).face());
                    }
                    case CubeLayout.Ocean o -> ocean++;
                    case OnFace of -> fail("(" + x + ", " + z + ") is off the planet");
                }
            }
            System.out.println("CubeLayoutTest s " + s + ": 10,000 outside blocks round-trip, " + ocean + " ocean (past the net's corners)");
        }
    }

    @Test
    void cornersAndTheirImagesAgree() {
        for (int si : SIDES) {
            CubeLayout cube = CubeLayout.of(si);
            long s = si;
            List<CubeLayout.Corner> corners = cube.corners();
            assertEquals(8, corners.size());
            double h = (double) s / 2.0;
            for (CubeLayout.Corner c : corners) {
                double[] want = {c.cube()[0], c.cube()[1], c.cube()[2]};
                for (long[] img : c.images()) {
                    // The image is a corner of each of its faces that the net draws there; seen
                    // from every face square that touches it, it is that face's cube corner.
                    int seen = 0;
                    for (Face f : Face.all()) {
                        long cx = cube.faceCentreX(f), cz = cube.faceCentreZ(f);
                        for (double off : new double[]{0.0, 4.0 * (double) s, -4.0 * (double) s}) {
                            double u = ((double) img[0] + off - (double) cx) / h;
                            double v = ((double) img[1] - (double) cz) / h;
                            if (Math.abs(u) == 1.0 && Math.abs(v) == 1.0) {
                                assertTrue(Arrays.asList(c.faces()).contains(f), c.name() + " image " + Arrays.toString(img) + " on " + f);
                                assertSameDoubles(want, gnomonic(f, u, v), c.name() + " via " + f);
                                seen++;
                            }
                        }
                    }
                    assertTrue(seen >= 1, c.name() + " image " + Arrays.toString(img) + " touches no face");
                }
                // The three faces' corners at this cube corner, all accounted for in the images.
                for (Face f : c.faces()) {
                    boolean found = false;
                    for (long[] img : c.images()) {
                        long cx = cube.faceCentreX(f), cz = cube.faceCentreZ(f);
                        for (double off : new double[]{0.0, 4.0 * (double) s, -4.0 * (double) s}) {
                            double u = ((double) img[0] + off - (double) cx) / h;
                            double v = ((double) img[1] - (double) cz) / h;
                            if (Math.abs(u) == 1.0 && Math.abs(v) == 1.0 && sameValues(gnomonic(f, u, v), want))
                                found = true;
                        }
                    }
                    assertTrue(found, c.name() + ": face " + f + " has no image");
                }
            }
            // The F0 corners are the quarter-turn pivots.
            List<long[]> pivots = new ArrayList<>();
            for (NetEdge e : cube.edges())
                if (e.pivot() != null)
                    pivots.add(e.pivot());
            for (CubeLayout.Corner c : corners) {
                if (!Arrays.asList(c.faces()).contains(Face.F0))
                    continue;
                assertEquals(1, c.images().length);
                assertTrue(pivots.stream().anyMatch(pv -> Arrays.equals(pv, c.images()[0])), c.name());
            }
        }
    }

    @Test
    void theVoidSquaresSplitOnTheDiagonal() {
        // Plan §1.4: the void square above F1 splits on the diagonal through F0's corner, each half
        // folding through its own edge.
        CubeLayout cube = CubeLayout.of(1_024);
        long h = 512;
        for (long a : new long[]{0, 3, 100, 500}) {
            for (long b : new long[]{0, 3, 100, 500}) {
                long x = h + a, z = -h - 1 - b;
                Fold f = cube.fold(x, z);
                assertTrue(f instanceof Outside, "(" + x + ", " + z + ") " + f);
                String name = cube.edges().get(((Outside) f).edge()).name();
                assertEquals(a <= b ? "N-east" : "F1-top", name, "a " + a + " b " + b);
            }
        }
        // The void above F2 is N turned half way round.
        Fold f = cube.fold(2 * 1_024, -1_024);
        assertTrue(f instanceof Outside);
        Outside out = (Outside) f;
        assertEquals(2, out.iso().k());
        Fold target = cube.fold(out.x(), out.z());
        assertTrue(target instanceof OnFace on && on.face() == Face.N);
    }

    @Test
    void pointsFoldLikeBlocks() {
        CubeLayout cube = CubeLayout.of(1_024);
        Rng rng = new Rng(7);
        for (int n = 0; n < 20_000; n++) {
            long x = rng.range(-4_000, 4_000);
            long z = rng.range(-2_000, 2_000);
            Fold bf = cube.fold(x, z);
            PointOnFace pf = cube.foldPoint((double) x + 0.5, (double) z + 0.5);
            switch (bf) {
                case OnFace of -> {
                    assertNotNull(pf);
                    assertEquals(of.face(), pf.face());
                    assertEquals(-1, pf.edge());
                    assertSameDouble((double) cube.blockX(of.face(), of.i()) + 0.5, pf.x(), "x");
                    assertSameDouble((double) cube.blockZ(of.face(), of.j()) + 0.5, pf.z(), "z");
                }
                case Outside out -> {
                    assertNotNull(pf, "(" + x + ", " + z + ")");
                    assertEquals(out.iso(), pf.iso());
                    assertEquals(out.edge(), pf.edge());
                    assertSameDouble((double) out.x() + 0.5, pf.x(), "x");
                    assertSameDouble((double) out.z() + 0.5, pf.z(), "z");
                }
                case CubeLayout.Ocean o -> assertNull(pf);
            }
        }
    }

    @Test
    void turnsAreRotationsAndInvert() {
        CubeTurn polar = CubeTurn.polar(30.0);
        // the planet point (0 deg, 30 deg) at F0's centre, the north pole at N's
        double[] p = CubeWorld.unitOfLatLon(0.0, 30.0);
        double[] c = polar.toCube(p);
        assertTrue(Math.abs(c[0] - 1.0) < 1e-15 && Math.abs(c[1]) < 1e-15 && Math.abs(c[2]) < 1e-15, Arrays.toString(c));
        assertSameDoubles(new double[]{0.0, 0.0, 1.0}, polar.toCube(new double[]{0.0, 0.0, 1.0}), "pole");
        CubeTurn.fromRows(polar.rows());
        // a free turn: the inverse is the transpose
        double r = 1.0 / Math.sqrt(3.0);
        double s = 1.0 / Math.sqrt(2.0);
        double t = 1.0 / Math.sqrt(6.0);
        CubeTurn free = CubeTurn.fromRows(new double[][]{{r, r, r}, {s, -s, 0.0}, {t, t, -2.0 * t}});
        Rng rng = new Rng(3);
        for (int n = 0; n < 1_000; n++) {
            double[] q = {(double) rng.range(-1000, 1000), (double) rng.range(-1000, 1000), (double) rng.range(-1000, 1000)};
            double[] back = free.toPlanet(free.toCube(q));
            for (int k = 0; k < 3; k++)
                assertTrue(Math.abs(back[k] - q[k]) < 1e-12, Arrays.toString(q) + " " + Arrays.toString(back));
        }
        // reflections and non-orthonormal rows are refused
        assertThrows(WorldDataException.class, () -> CubeTurn.fromRows(new double[][]{{1, 0, 0}, {0, 1, 0}, {0, 0, -1}}));
        assertThrows(WorldDataException.class, () -> CubeTurn.fromRows(new double[][]{{1, 0, 0}, {0, 1, 0}, {0, 0.1, 1}}));
    }

    // ------------------------------------------------------------------------------------------
    // The wrap engine's cube cross is the same net
    // ------------------------------------------------------------------------------------------

    /**
     * {@link WrapLayouts#cubeCross}(s, s, true) has F0 at [0, s)² and F3 on the right, at
     * [3s, 4s) × [0, s); this layout's F0 is centred on the origin with F3 on the left, in the
     * window x in [-3s/2, 5s/2). The frames differ by the translation (s/2, s/2): X = x + s/2 and
     * Z = z + s/2 (X in [-s, 3s) for a window x). Edges 2 to 13 are the same twelve edges in the same
     * order; the two engine bands over the void squares above and below F3 (F3-top, F3-bottom) sit one
     * period to the right of this layout's (X + 4s), the engine having no wrap before its fold, so each
     * edge carries its copy offset. The slides (edges 0 and 1) are not compared. Where two bands meet,
     * in the void squares by F0's corners, the engine takes the first edge in its list and chains, this
     * layout the nearer edge: those blocks are compared against the engine's candidate set, not
     * against its fold.
     */
    @Test
    void theLayoutIsTheWrapEnginesCubeCross() {
        for (int si : SIDES) {
            CubeLayout cube = CubeLayout.of(si);
            int s = si;
            int h = s / 2;
            WrapTopology topo = WrapLayouts.cubeCross(s, s, true);
            List<WrapEdge> we = topo.edges();
            assertEquals(14, we.size());
            int[] offset = new int[14];
            // Geometry and motions of the twelve turning edges.
            for (int k = 2; k < 14; k++) {
                NetEdge e = cube.edges().get(k);
                WrapEdge w = we.get(k);
                String what = "s " + s + " edge " + k + " " + e.name() + " / " + w.name();
                assertEquals(e.axis() == Axis.X, w.normalX(), what);
                assertEquals(e.beyond() == Side.MIN, w.minSide(), what);
                assertEquals(s, w.depth(), what);
                assertEquals(e.coord() + h, w.coord(), what);
                long lo = e.spanFrom() + h;
                if (e.axis() == Axis.Z) {
                    long off = w.spanLo() - lo;
                    assertTrue(off == 0 || off == 4L * s, what + ": span offset " + off);
                    offset[k] = (int) off;
                } else {
                    assertEquals(lo, w.spanLo(), what);
                }
                assertEquals(lo + offset[k], w.spanLo(), what);
                assertEquals(lo + offset[k] + s, w.spanHi(), what);
                // The motion, on the first and the last row of the band: the same image (x modulo
                // the period) and the same quarter turns.
                for (int row : new int[]{0, s - 1}) {
                    for (long t = e.spanFrom(); t < e.spanTo(); t++) {
                        long n = e.beyond() == Side.MAX ? e.coord() + row : e.coord() - 1 - row;
                        long x = e.axis() == Axis.X ? n : t;
                        long z = e.axis() == Axis.X ? t : n;
                        int bigX = (int) (x + h + offset[k]);
                        int bigZ = (int) (z + h);
                        String at = what + " row " + row + " (" + x + ", " + z + ")";
                        assertTrue(w.beyond(bigX, bigZ), at);
                        long[] mine = e.iso().cell(x, z);
                        CellIsometry g = w.iso();
                        assertEquals(Math.floorMod(mine[0] + h, 4L * s), Math.floorMod((long) g.applyX(bigX, bigZ), 4L * s), at);
                        assertEquals(mine[1] + h, g.applyZ(bigX, bigZ), at);
                        assertEquals(e.iso().k(), g.quarterTurns(), at);
                    }
                }
            }
            // Every block of the window, against the engine's candidates.
            boolean dense = s <= 64;
            Rng rng = new Rng(0xC0B3_C055L ^ s);
            long xLo = -7 * (long) s / 2, xHi = 9 * (long) s / 2, zLo = -5 * (long) s / 2, zHi = 5 * (long) s / 2;
            long total = dense ? (xHi - xLo) * (zHi - zLo) : 60_000;
            long[] counts = new long[3];
            for (long n = 0; n < total; n++) {
                long x, z;
                if (dense) {
                    x = xLo + n % (xHi - xLo);
                    z = zLo + n / (xHi - xLo);
                } else {
                    x = rng.range(xLo, xHi);
                    z = rng.range(zLo, zHi);
                }
                compareWithTheEngine(cube, topo, offset, x, z, counts);
            }
            System.out.println("CubeLayoutTest s " + s + " vs cubeCross: " + counts[0] + " on the planet, " + counts[1]
                    + " outside, " + counts[2] + " ocean, all checked");
            assertTrue(counts[0] > 0 && counts[1] > 0 && counts[2] > 0);
        }
    }

    private static void compareWithTheEngine(CubeLayout cube, WrapTopology topo, int[] offset, long x, long z, long[] counts) {
        int s = (int) cube.side();
        int h = s / 2;
        long xw = cube.wrapX(x);
        int bigX = (int) (xw + h);
        int bigZ = (int) (z + h);
        List<WrapEdge> we = topo.edges();
        List<Integer> candidates = new ArrayList<>();
        for (int k = 2; k < 14; k++)
            if (we.get(k).beyond(bigX + offset[k], bigZ))
                candidates.add(k);
        // The nearest candidate by this layout's depth, ties to the earlier edge.
        long bestD = Long.MAX_VALUE;
        int best = -1;
        for (int k : candidates) {
            long d = cube.edges().get(k).depth(xw, z);
            assertTrue(d >= 0 && d < s, "depth " + d);
            if (d < bestD) {
                bestD = d;
                best = k;
            }
        }
        String what = "s " + s + " (" + x + ", " + z + ")";
        switch (cube.fold(x, z)) {
            case OnFace of -> {
                counts[0]++;
                assertTrue(candidates.isEmpty(), what + ": on a face, yet beyond " + candidates);
                assertNull(topo.fold((int) Math.floorMod((long) bigX, 4L * s), bigZ), what);
            }
            case Outside out -> {
                counts[1]++;
                assertEquals(best, out.edge(), what + ": candidates " + candidates);
                CellIsometry g = we.get(out.edge()).iso();
                int gx = bigX + offset[out.edge()];
                assertEquals(Math.floorMod(out.x() + h, 4L * s), Math.floorMod((long) g.applyX(gx, bigZ), 4L * s), what);
                assertEquals(out.z() + h, g.applyZ(gx, bigZ), what);
                assertEquals(out.iso().k(), g.quarterTurns(), what);
                // The planet block it shows is a block of a face.
                assertNotNull(cube.faceOfBlock(out.x(), out.z()), what + ": (" + out.x() + ", " + out.z() + ") is not a block of a face");
                if (candidates.size() == 1) {
                    WrapTopology.Fold tf = topo.fold(gx, bigZ);
                    assertNotNull(tf, what);
                    assertEquals(Math.floorMod(out.x() + h, 4L * s), Math.floorMod((long) tf.x(), 4L * s), what);
                    assertEquals(out.z() + h, tf.z(), what);
                    assertEquals(out.iso().k(), tf.iso().quarterTurns(), what);
                }
            }
            case CubeLayout.Ocean ocean -> {
                counts[2]++;
                // Either no band holds the block, or the nearest band's image is not a block of the
                // planet (the missing quarter round an outer cube corner).
                if (best >= 0) {
                    CellIsometry g = we.get(best).iso();
                    int gx = bigX + offset[best];
                    long px = cube.wrapX(Math.floorMod((long) g.applyX(gx, bigZ) - h, 4L * s));
                    long pz = g.applyZ(gx, bigZ) - h;
                    assertNull(cube.faceOfBlock(px, pz), what + ": ocean, yet edge " + best + " shows (" + px + ", " + pz + ")");
                }
            }
        }
    }
}
