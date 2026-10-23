// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The wrap engine on the cube (docs/design/wrap-engine.md §12; ledger: SCL-180 to SCL-183):
 * {@code WrapLayouts.cube} against phase 0's net ({@link CubeLayout}, the geometry the parity fixture
 * pins), in Toroidal World's frame, with and without the engine's own slides.
 * <ul>
 *   <li>In every band the engine's fold is {@code CubeLayout.fold}'s: the same planet block (x modulo
 *       4s) and the same turn, for s = 256, 1024 and 3840; only the convex quadrants differ, where the
 *       net has ocean and the engine lands on a face.</li>
 *   <li>A loop round each corner image on the strip's lines closes after three legs and turns by an
 *       odd number of quarter turns (the 270° cone point's holonomy).</li>
 *   <li>The fold is a local isometry away from the corners; split chunks are copied by column; the
 *       turns-only cube repeats every 4s; a moved cube folds as the cube moved; the install follows
 *       Toroidal World's shape.</li>
 * </ul>
 */
class WrapTopologyCubeTest {

    private static final int[][] DIRS = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}};

    @Test
    void theFoldEqualsTheNetsFoldInEveryBand() {
        for (int s : new int[]{256, 1024, 3840})
            for (int d : s == 256 ? new int[]{16, 32, 128} : new int[]{32})
                for (boolean slides : new boolean[]{true, false})
                    compareWithTheNet(s, d, slides);
    }

    private static void compareWithTheNet(int s, int d, boolean slides) {
        CubeLayout net = CubeLayout.of(s);
        WrapTopology t = WrapLayouts.cube(net, d, slides);
        int h = s / 2;
        assertTrue(t.isChunkAligned(), t + " is chunk-aligned");
        assertEquals(slides ? 0 : 4 * s, t.periodX());
        assertEquals(slides ? 18 : 16, t.edges().size(), t + ": 12 turning edges, 4 pieces, and 2 slides with slides");
        int outside = 0, slid = 0, quadrant = 0;
        for (WrapEdge e : t.edges()) {
            for (int z = e.bandMinZ(); z < e.bandMaxZ(); z++) {
                for (int x = e.bandMinX(); x < e.bandMaxX(); x++) {
                    WrapTopology.Fold f = t.fold(x, z);
                    String where = t.name() + " (" + x + ", " + z + ") in " + e.name();
                    assertNotNull(f, "a band block folds: " + where);
                    assertTrue(f.x() >= -2 * s && f.x() < 2 * s, "the fold lands in Toroidal World's window: " + where);
                    assertNotNull(net.faceOfBlock(f.x(), f.z()), "the fold lands on a face: " + where);
                    switch (compare(net, s, h, d, x, z, f, where)) {
                        case 0 -> outside++;
                        case 1 -> slid++;
                        default -> quadrant++;
                    }
                }
            }
        }
        assertTrue(outside > 12 * (s - 2 * d) * d, t + ": band blocks the net folds too: " + outside);
        assertEquals(slides, slid > 0, t + ": only the slides fold blocks the net has on a face");
        assertTrue(quadrant > 0, t + ": the convex quadrants are in bands");
        // Anywhere else the engine leaves a block alone only where the net has it on a face, in ocean
        // or deeper out than the band.
        Random r = new Random(31L * s + d);
        int lo = slides ? -2 * s - d : -5 * s, hi = slides ? 2 * s + d : 5 * s;
        for (int i = 0; i < 200_000; i++) {
            int x = lo + r.nextInt(hi - lo), z = -3 * h - s + r.nextInt(6 * h + 2 * s);
            WrapTopology.Fold f = t.fold(x, z);
            String where = t.name() + " sample (" + x + ", " + z + ")";
            if (f != null) {
                compare(net, s, h, d, x, z, f, where);
                continue;
            }
            if (net.fold(x, z) instanceof CubeLayout.Outside o) {
                long depth = net.edges().get(o.edge()).depth(net.wrapX(x), z);
                assertTrue(depth >= d, "the net folds a block within the band depth that the engine leaves: " + where + " depth " + depth);
            }
        }
    }

    /** One folded block against the net: 0 the net folds it too, 1 the net has it on a face (a slide), 2 a convex quadrant. */
    private static int compare(CubeLayout net, int s, int h, int d, int x, int z, WrapTopology.Fold f, String where) {
        long p = 4L * s;
        CubeLayout.Fold g = net.fold(x, z);
        if (g instanceof CubeLayout.Outside o) {
            assertEquals(0, Math.floorMod(f.x() - o.x(), p), "x modulo 4s: " + where);
            assertEquals(o.z(), f.z(), "z: " + where);
            assertEquals(o.iso().k(), f.iso().quarterTurns(), "the turn: " + where);
            return 0;
        }
        if (g instanceof CubeLayout.OnFace on) {
            assertEquals(0, f.iso().quarterTurns(), "only the slide folds a block the net has on a face: " + where);
            assertEquals(0, Math.floorMod(f.x() - net.blockX(on.face(), on.i()), p), "x modulo 4s: " + where);
            assertEquals(net.blockZ(on.face(), on.j()), f.z(), "z: " + where);
            return 1;
        }
        int xr = Math.floorMod(x + 2 * s, 4 * s) - 2 * s;
        boolean qx = (xr >= h && xr < h + d) || (xr >= -h - d && xr < -h);
        boolean qz = (z >= -3 * h - d && z < -3 * h) || (z >= 3 * h && z < 3 * h + d);
        assertTrue(qx && qz, "the net has ocean in a band only in the convex quadrants: " + where);
        return 2;
    }

    @Test
    void theCubeIsTheNetsMeridianWalk() {
        int s = 256, h = s / 2;
        for (boolean slides : new boolean[]{true, false}) {
            WrapTopology t = WrapLayouts.cubeCross(s, 32, slides);
            // North out of F0 over the pole N comes out on F2 heading south, on either side of x = 0.
            WrapTopology.PointFold east = t.foldPoint(0.25, -3 * h - 1.5);
            assertNotNull(east);
            assertEquals(4 * h - 0.25, east.x(), 1e-9);
            assertEquals(-h + 1.5, east.z(), 1e-9);
            assertEquals(180f, east.iso().yawDegrees());
            WrapTopology.PointFold west = t.foldPoint(-0.25, -3 * h - 1.5);
            assertNotNull(west);
            assertEquals(-4 * h + 0.25, west.x(), 1e-9, "F2's east half lives at the window's west end");
            assertEquals(180f, west.iso().yawDegrees());
            // From F1 into N through a quarter turn.
            WrapTopology.PointFold f1 = t.foldPoint(2 * h + 0.5, -h - 1.5);
            assertNotNull(f1);
            assertEquals(270f, f1.iso().yawDegrees());
            assertEquals(CubeLayout.Face.N, CubeLayout.of(s).faceOfBlock((long) Math.floor(f1.x()), (long) Math.floor(f1.z())).face());
        }
        // With the slides, a block beyond F2-top's east end slides, then turns: two steps, a half turn.
        WrapTopology t = WrapLayouts.cubeCross(s, 16, true);
        WrapTopology.Fold two = t.fold(4 * h + 5, -h - 3);
        assertNotNull(two);
        assertEquals(2, two.iso().quarterTurns());
        CubeLayout.Outside o = (CubeLayout.Outside) CubeLayout.of(s).fold(4 * h + 5, -h - 3);
        assertEquals(o.x(), two.x());
        assertEquals(o.z(), two.z());
    }

    @Test
    void aLoopRoundAStripCornerTurnsByAQuarter() {
        for (int s : new int[]{256, 1024}) {
            int h = s / 2;
            for (boolean slides : new boolean[]{true, false}) {
                WrapTopology t = WrapLayouts.cubeCross(s, 32, slides);
                int[][] top = {{h, -h}, {-h, -h}, {3 * h, -h}, {-3 * h, -h}};
                for (int[] c : top) {
                    walk(t, c[0], c[1], true);
                    walk(t, c[0], -c[1], false);
                }
            }
        }
    }

    /**
     * A turtle's walk round the corner image (px, pz) on the planet: three legs of 2r − 1 blocks with a
     * left turn between (a right turn below the strip), each step folded and the heading turned with
     * it. Round a 270° cone point the walk closes after three legs, and the seams it crossed turn it by
     * an odd number of quarter turns (ledger: SCL-182).
     */
    private static void walk(WrapTopology t, int px, int pz, boolean topLine) {
        int r = 6, len = 2 * r - 1;
        int x = px - r, z = topLine ? pz + r - 1 : pz - r;
        int x0 = x, z0 = z, hx = 1, hz = 0, k = 0;
        String where = t.name() + " corner (" + px + ", " + pz + ")";
        assertTrue(t.inside(x, z), "the walk starts on the planet: " + where);
        for (int leg = 0; leg < 3; leg++) {
            if (leg > 0) {
                int nx = topLine ? hz : -hz, nz = topLine ? -hx : hx;
                hx = nx;
                hz = nz;
            }
            for (int i = 0; i < len; i++) {
                int qx = x + hx, qz = z + hz;
                WrapTopology.Fold f = t.fold(qx, qz);
                if (f == null) {
                    x = qx;
                    z = qz;
                    continue;
                }
                x = f.x();
                z = f.z();
                double[] v = f.iso().applyVector(hx, hz);
                hx = (int) Math.round(v[0]);
                hz = (int) Math.round(v[1]);
                k += f.iso().quarterTurns();
            }
        }
        assertEquals(x0, x, "the walk closes after three legs: " + where);
        assertEquals(z0, z, "the walk closes after three legs: " + where);
        assertEquals(1, Math.floorMod(k, 2), "the seams turn the walk by an odd number of quarter turns: " + where + " k " + k);
    }

    @Test
    void theCornersHaveTheirKeepOut() {
        int s = 256, p = 4 * s;
        for (boolean slides : new boolean[]{true, false}) {
            WrapTopology t = WrapLayouts.cubeCross(s, 32, slides);
            assertEquals(8, t.corners().size());
            int[][] images = WrapLayouts.cubeCrossCorners(s);
            assertEquals(12, images.length, "n0, n3, s0, s3 once; n1, n2, s1, s2 twice in Toroidal World's frame");
            int k = WrapTopology.CORNER_KEEP_OUT;
            for (int[] c : images) {
                for (int dx = -k; dx < k; dx++)
                    for (int dz = -k; dz < k; dz++)
                        assertTrue(t.nearCorner(c[0] + dx, c[1] + dz), "near (" + c[0] + ", " + c[1] + ")");
                assertFalse(t.nearCorner(c[0] + k, c[1]));
                assertFalse(t.nearCorner(c[0] - k - 1, c[1]));
                assertFalse(t.nearCorner(c[0], c[1] + k));
                assertFalse(t.nearCorner(c[0] + 3, c[1] - 3));
                if (!slides)
                    assertTrue(t.nearCorner(c[0] + p, c[1]), "a period away (Toroidal World's alias)");
            }
        }
        assertFalse(WrapLayouts.guyou(512, 32, true).nearCorner(512, 256), "a layout without cone points refuses nothing");
    }

    @Test
    void splitChunksAreCopiedColumnByColumn() {
        for (boolean slides : new boolean[]{true, false}) {
            WrapTopology t = WrapLayouts.cubeCross(256, 32, slides);
            List<Long> band = new ArrayList<>();
            t.forEachBandChunk(band::add);
            assertEquals(band.size(), new HashSet<>(band).size(), "each band chunk once");
            int split = 0;
            for (long key : band) {
                int cx = (int) key, cz = (int) (key >> 32);
                WrapTopology.ChunkFold cf = t.chunkFold(cx, cz);
                WrapTopology.ChunkColumns cc = t.chunkColumns(cx, cz);
                assertNotNull(cf);
                assertNotNull(cc);
                boolean uniform = true;
                for (int lz = 0; lz < 16; lz++) {
                    for (int lx = 0; lx < 16; lx++) {
                        WrapTopology.Fold f = t.fold((cx << 4) + lx, (cz << 4) + lz);
                        assertNotNull(f);
                        WrapTopology.ChunkSource src = cc.at(lx, lz);
                        assertNotNull(src);
                        assertEquals(src.iso(), f.iso());
                        assertEquals(src.sourceX(), f.x() >> 4);
                        assertEquals(src.sourceZ(), f.z() >> 4);
                        uniform &= f.iso().equals(cf.iso());
                    }
                }
                assertEquals(!uniform, cf.split(), "chunk " + cx + "," + cz + ": split exactly when its columns fold by two motions");
                if (cf.split()) {
                    split++;
                    assertTrue(cc.sources().size() >= 2);
                }
                for (WrapTopology.ChunkSource src : cc.sources())
                    assertTrue(Arrays.asList(t.chunkImages(src.sourceX(), src.sourceZ())).contains(src.iso().inverse()),
                            "source " + src + " lists band chunk " + cx + "," + cz);
            }
            // Two per corner square and per convex quadrant on each diagonal: 2 chunks of 16 on a band 32 deep.
            assertEquals(8 * 2, split, t + ": the split chunks are the diagonals' (8 squares, 2 chunks each)");
        }
    }

    @Test
    void everyBandBlockIsAnImageOfItsStoredBlock() {
        Random r = new Random(13);
        for (boolean slides : new boolean[]{true, false}) {
            WrapTopology t = WrapLayouts.cubeCross(256, 32, slides);
            for (WrapEdge e : t.edges()) {
                for (int i = 0; i < 300; i++) {
                    int x = e.bandMinX() + r.nextInt(e.bandMaxX() - e.bandMinX()), z = e.bandMinZ() + r.nextInt(e.bandMaxZ() - e.bandMinZ());
                    WrapTopology.Fold f = t.fold(x, z);
                    assertNotNull(f);
                    List<int[]> images = t.images(f.x(), f.z());
                    assertTrue(images.stream().anyMatch(q -> q[0] == x && q[1] == z),
                            t + ": band block (" + x + ", " + z + ") is not an image of (" + f.x() + ", " + f.z() + ")");
                    for (int[] q : images) {
                        WrapTopology.Fold back = t.fold(q[0], q[1]);
                        assertEquals(f.x(), back.x());
                        assertEquals(f.z(), back.z());
                    }
                }
            }
        }
    }

    @Test
    void theCubeIsSeamlessAwayFromItsCorners() {
        int s = 256, d = 32, h = s / 2;
        CubeLayout net = CubeLayout.of(s);
        int[][] corners = WrapLayouts.cubeCrossCorners(s);
        for (boolean slides : new boolean[]{true, false}) {
            WrapTopology t = WrapLayouts.cubeCross(s, d, slides);
            int xlo = slides ? -4 * h - d : -4 * h - s, xhi = slides ? 4 * h + d : 4 * h + s;
            Random r = new Random(42);
            int checked = 0, across = 0;
            for (int i = 0; i < 80_000; i++) {
                int x = xlo + r.nextInt(xhi - xlo), z = -3 * h - d + r.nextInt(6 * h + 2 * d);
                if (!valid(t, net, s, x, z, slides) || nearCorner(corners, s, x, z, d + 2))
                    continue;
                int[] q = canon(t, s, x, z);
                CellIsometry qi = isoOf(t, x, z);
                for (int[] dir : DIRS) {
                    int nx = x + dir[0], nz = z + dir[1];
                    if (!valid(t, net, s, nx, nz, slides) || nearCorner(corners, s, nx, nz, d + 2))
                        continue;
                    int[] direct = canon(t, s, nx, nz);
                    double[] rd = qi.applyVector(dir[0], dir[1]);
                    int sx = q[0] + (int) Math.round(rd[0]), sz = q[1] + (int) Math.round(rd[1]);
                    if (!valid(t, net, s, sx, sz, slides))
                        continue;
                    int[] stepped = canon(t, s, sx, sz);
                    String where = t.name() + " (" + x + ", " + z + ") + " + dir[0] + "," + dir[1];
                    assertEquals(direct[0], stepped[0], "x: " + where);
                    assertEquals(direct[1], stepped[1], "z: " + where);
                    assertEquals(direct[2], Math.floorMod(stepped[2] + q[2], 4), "turn: " + where);
                    if (direct[2] != q[2])
                        across++;
                    checked++;
                }
            }
            assertTrue(checked > 20_000, "enough steps checked: " + checked);
            assertTrue(across > 200, "enough steps across a turning seam: " + across);
        }
    }

    /** On the planet (in the window, with the slides) or in a band. */
    private static boolean valid(WrapTopology t, CubeLayout net, int s, int x, int z, boolean slides) {
        if (t.fold(x, z) != null)
            return true;
        return net.faceOfBlock(x, z) != null && (!slides || (x >= -2 * s && x < 2 * s));
    }

    /** {stored x reduced into [−2s, 2s), stored z, quarter turns}. */
    private static int[] canon(WrapTopology t, int s, int x, int z) {
        WrapTopology.Fold f = t.fold(x, z);
        int fx = f == null ? x : f.x(), fz = f == null ? z : f.z(), k = f == null ? 0 : f.iso().quarterTurns();
        return new int[]{Math.floorMod(fx + 2 * s, 4 * s) - 2 * s, fz, k};
    }

    private static CellIsometry isoOf(WrapTopology t, int x, int z) {
        WrapTopology.Fold f = t.fold(x, z);
        return f == null ? CellIsometry.IDENTITY : f.iso();
    }

    private static boolean nearCorner(int[][] corners, int s, int x, int z, int dist) {
        for (int[] c : corners) {
            int dx = Math.floorMod(x - c[0] + 2 * s, 4 * s) - 2 * s;
            if (Math.max(Math.abs(dx), Math.abs(z - c[1])) <= dist)
                return true;
        }
        return false;
    }

    @Test
    void theTurnsOnlyCubeRepeatsEveryPeriod() {
        int s = 256, p = 4 * s, h = s / 2;
        WrapTopology t = WrapLayouts.cubeCross(s, 32, false);
        assertEquals(p, t.periodX());
        assertEquals(-2 * s, t.windowLo());
        Random r = new Random(3);
        int folded = 0;
        for (int i = 0; i < 40_000; i++) {
            int x = -2 * s + r.nextInt(p), z = -3 * h - 32 + r.nextInt(6 * h + 64);
            WrapTopology.Fold f = t.fold(x, z);
            for (int m : new int[]{-3, -1, 1, 2}) {
                WrapTopology.Fold g = t.fold(x + m * p, z);
                if (f == null) {
                    assertNull(g);
                    assertEquals(t.inside(x, z), t.inside(x + m * p, z));
                    continue;
                }
                assertNotNull(g);
                assertEquals(f.x(), g.x());
                assertEquals(f.z(), g.z());
                assertEquals(f.iso().compose(CellIsometry.translation(-m * p, 0)), g.iso());
            }
            if (f != null) {
                folded++;
                for (int[] q : t.images(f.x(), f.z()))
                    assertTrue(q[0] >= -2 * s && q[0] < 2 * s, "images are in the window");
                assertTrue(t.images(f.x() + p, f.z()).stream().anyMatch(q -> q[0] == x && q[1] == z), "a stored block one period away has the same images");
            }
        }
        assertTrue(folded > 1000);
        assertEquals(2 * s + 3, t.nearX(-2 * s + 3, 2 * s - 2));
        assertEquals(-5, t.nearX(-5 + 2 * p, 0));
        assertEquals(17, t.nearX(17, 0));
        // The nearest image of a stored block, across Toroidal World's seam.
        WrapTopology.Fold f = t.fold(-2 * s + 5, -h - 3);
        int[] near = t.nearestImage(f.x(), f.z(), 2 * s + 5.5, -h - 2.5);
        assertEquals(2 * s + 5, near[0]);
        assertEquals(-h - 3, near[1]);
    }

    @Test
    void aMovedCubeFoldsAsTheCubeMoved() {
        int s = 256, h = s / 2, dx = 49152, dz = 20480;
        WrapTopology t = WrapLayouts.cubeCross(s, 16, true), m = t.translated(dx, dz);
        assertTrue(m.isChunkAligned());
        Random r = new Random(17);
        for (int i = 0; i < 40_000; i++) {
            int x = -4 * h - 16 + r.nextInt(8 * h + 32), z = -3 * h - 16 + r.nextInt(6 * h + 32);
            WrapTopology.Fold f = t.fold(x, z), g = m.fold(x + dx, z + dz);
            if (f == null) {
                assertNull(g);
                continue;
            }
            assertNotNull(g);
            assertEquals(f.x() + dx, g.x());
            assertEquals(f.z() + dz, g.z());
            assertEquals(f.iso().quarterTurns(), g.iso().quarterTurns());
        }
        for (int[] c : WrapLayouts.cubeCrossCorners(s)) {
            assertTrue(m.nearCorner(c[0] + dx, c[1] + dz));
            assertFalse(m.nearCorner(c[0], c[1]));
        }
        assertThrows(IllegalArgumentException.class, () -> WrapTopology.union("periodic", List.of(WrapLayouts.cubeCross(s, 16, false))));
    }

    @Test
    void theCubeRefusesABandItCannotAlign() {
        assertThrows(IllegalArgumentException.class, () -> WrapLayouts.cubeCross(256, 0, true));
        assertThrows(IllegalArgumentException.class, () -> WrapLayouts.cubeCross(256, 129, true), "deeper than s/2");
        assertFalse(WrapLayouts.cubeCross(256, 24, true).isChunkAligned(), "a depth off the chunk grid");
        assertFalse(WrapLayouts.cubeCross(272, 16, true).isChunkAligned(), "a face whose half is off the chunk grid");
    }

    @Test
    void theInstallFollowsToroidalWorldsShape() {
        int s = 1024;
        assertEquals(CubeInstall.Mode.WITH_SLIDES, CubeInstall.decide(false, false, 0, 0, false, s).mode());
        assertEquals(CubeInstall.Mode.WITH_SLIDES, CubeInstall.decide(true, false, 0, 0, false, s).mode());
        assertEquals(CubeInstall.Mode.TURNS_ONLY, CubeInstall.decide(true, true, 4096, -2048, false, s).mode());
        assertEquals(CubeInstall.Mode.REFUSE, CubeInstall.decide(true, true, 4096, 0, false, s).mode(), "another window");
        assertEquals(CubeInstall.Mode.REFUSE, CubeInstall.decide(true, true, 8192, -4096, false, s).mode(), "another width");
        assertEquals(CubeInstall.Mode.REFUSE, CubeInstall.decide(true, true, 4096, -2048, true, s).mode(), "a z loop");
        assertEquals(CubeInstall.Mode.REFUSE, CubeInstall.decide(true, false, 0, 0, true, s).mode(), "a z loop alone");
        assertEquals(32, CubeInstall.DEFAULT_DEPTH);
    }
}
