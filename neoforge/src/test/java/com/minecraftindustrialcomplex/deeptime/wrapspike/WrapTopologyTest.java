// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapspike;

import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The wrap-engine spike's gluing algebra (docs/design/wrap-engine.md §3): block motions compose and
 * invert, every band block folds to a stored block, every stored block's band images fold back to it,
 * and the fold is a local isometry: two neighbouring blocks anywhere in the bands stay neighbours,
 * turned the same way, after folding. Checked on the spike's one-seam site, Guyou, Peirce and a
 * quarter-turn corner, on seeded random blocks (properties, not pinned outputs).
 */
class WrapTopologyTest {

    private static final int[][] DIRS = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}};

    @Test
    void motionsComposeAndInvert() {
        Random r = new Random(1);
        for (int i = 0; i < 2000; i++) {
            CellIsometry a = new CellIsometry(r.nextInt(4), r.nextInt(2001) - 1000, r.nextInt(2001) - 1000);
            CellIsometry b = new CellIsometry(r.nextInt(4), r.nextInt(2001) - 1000, r.nextInt(2001) - 1000);
            int x = r.nextInt(4001) - 2000, z = r.nextInt(4001) - 2000;
            assertArrayEquals(new int[]{x, z}, a.inverse().apply(a.apply(x, z)[0], a.apply(x, z)[1]), a + " inverse");
            int[] ab = b.apply(x, z);
            assertArrayEquals(a.apply(ab[0], ab[1]), a.compose(b).apply(x, z), a + " ∘ " + b);
            // Cells move as their centres do.
            double[] c = a.applyPoint(x + 0.5, z + 0.5);
            int[] cell = a.apply(x, z);
            assertEquals(cell[0] + 0.5, c[0], 1e-9);
            assertEquals(cell[1] + 0.5, c[1], 1e-9);
        }
    }

    @Test
    void turnsAboutAPointFixItAndCloseUp() {
        CellIsometry h = CellIsometry.halfTurnAbout(7, -3);
        assertTrue(h.compose(h).isIdentity(), "a half-turn is an involution");
        double[] fixed = h.applyPoint(7, -3);
        assertEquals(7, fixed[0], 1e-12);
        assertEquals(-3, fixed[1], 1e-12);
        // The four blocks round the turn point are swapped in pairs.
        assertArrayEquals(new int[]{6, -4}, h.apply(7, -3));
        CellIsometry q = CellIsometry.quarterTurnAbout(1, 5, 9);
        assertTrue(q.compose(q).compose(q).compose(q).isIdentity(), "four quarter turns close up");
        assertEquals(CellIsometry.halfTurnAbout(5, 9), q.compose(q), "two quarter turns are the half-turn");
        double[] qf = q.applyPoint(5, 9);
        assertEquals(5, qf[0], 1e-12);
        assertEquals(9, qf[1], 1e-12);
        // One clockwise quarter turn takes north (−z) to east (+x), Minecraft's CLOCKWISE_90.
        assertArrayEquals(new double[]{1, 0}, q.applyVector(0, -1), 1e-12);
        assertEquals(90f, q.yawDegrees());
    }

    @Test
    void theSpikeSeamGluesAsTheTaskSays() {
        // Beyond the edge, (x, z) is (2cx − x − 1, 2z0 − z − 1): the block form of (−x, 2b − z).
        int cx = 4096, z0 = 4096;
        WrapTopology t = WrapTopology.halfTurnSeam(cx, z0, 8, 8);
        assertNull(t.fold(cx, z0), "inside is not folded");
        WrapTopology.Fold f = t.fold(cx + 2, z0 - 1);
        assertNotNull(f);
        assertEquals(cx - 3, f.x());
        assertEquals(z0, f.z());
        assertEquals(2, f.iso().quarterTurns());
        assertNull(t.fold(cx + 8, z0 - 1), "outside the seam's span nothing folds");
        assertNull(t.fold(cx, z0 - 9), "beyond the band nothing folds");
        List<int[]> img = t.images(cx - 3, z0);
        assertEquals(1, img.size());
        assertArrayEquals(new int[]{cx + 2, z0 - 1}, img.get(0));
        assertTrue(t.images(cx - 3, z0 + 8).isEmpty(), "a block deeper inside than the band has no image");
        checkLocalIsometry(t, cx - 8, cx + 8, z0 - 8, z0 + 8, 8);
    }

    @Test
    void guyouFoldsEveryBandBlockAndStaysSeamless() {
        int h = 512;
        WrapTopology t = WrapTopology.guyou(h, 32);
        // A point beyond the north edge is (−x, 2b − z) in point form, with b = −h/2.
        WrapTopology.PointFold pf = t.foldPoint(100.25, -h / 2.0 - 3.5);
        assertNotNull(pf);
        assertEquals(-100.25, pf.x(), 1e-9);
        assertEquals(-h / 2.0 + 3.5, pf.z(), 1e-9);
        assertEquals(180f, pf.iso().yawDegrees());
        // A slide keeps the heading.
        WrapTopology.PointFold east = t.foldPoint(h + 0.5, 10);
        assertEquals(-h + 0.5, east.x(), 1e-9);
        assertEquals(0f, east.iso().yawDegrees());
        checkLocalIsometry(t, -h - 31, h + 31, -h / 2 - 31, h / 2 + 31, 32);
        checkImages(t, -h, h, -h / 2, h / 2, 32);
    }

    @Test
    void peirceFoldsEveryBandBlockAndStaysSeamless() {
        int a = 640;
        WrapTopology t = WrapTopology.peirce(a, 32);
        checkLocalIsometry(t, -a / 2 - 31, a / 2 + 31, -a / 2 - 31, a / 2 + 31, 32);
        checkImages(t, -a / 2, a / 2, -a / 2, a / 2, 32);
    }

    @Test
    void quarterTurnsCloseToo() {
        WrapTopology t = WrapTopology.quarterTurnCorner(0, 0, 16);
        WrapTopology.Fold f = t.fold(5, -2);
        assertNotNull(f);
        assertEquals(1, f.x());
        assertEquals(5, f.z());
        assertEquals(1, f.iso().quarterTurns());
        // Each band is the other strip turned: the west band's fold is the inverse turn.
        WrapTopology.Fold g = t.fold(-2, 5);
        assertNotNull(g);
        assertEquals(3, g.iso().quarterTurns());
        checkLocalIsometry(t, -15, 200, -15, 200, 16);
    }

    /**
     * For random blocks q in the bands (or inside) and each horizontal step d with q + d also within
     * reach: folding q + d gives the same stored block as stepping from fold(q) in the turned direction
     * R(d) and folding again, with the same total turn. That is what makes the seam invisible to
     * fluids, redstone and collisions.
     */
    private static void checkLocalIsometry(WrapTopology t, int x0, int x1, int z0, int z1, int depth) {
        Random r = new Random(42);
        int checked = 0;
        for (int i = 0; i < 40000; i++) {
            int x = x0 + r.nextInt(x1 - x0), z = z0 + r.nextInt(z1 - z0);
            if (!withinReach(t, x, z, depth - 1))
                continue;
            Canon q = canon(t, x, z);
            for (int[] d : DIRS) {
                if (!withinReach(t, x + d[0], z + d[1], depth - 1))
                    continue;
                Canon direct = canon(t, x + d[0], z + d[1]);
                double[] rd = q.iso.applyVector(d[0], d[1]);
                Canon stepped = canon(t, q.x + (int) Math.round(rd[0]), q.z + (int) Math.round(rd[1]));
                assertEquals(direct.x, stepped.x, "x at (" + x + ", " + z + ") + " + d[0] + "," + d[1]);
                assertEquals(direct.z, stepped.z, "z at (" + x + ", " + z + ") + " + d[0] + "," + d[1]);
                assertEquals(direct.iso.quarterTurns(), stepped.iso.compose(q.iso).quarterTurns(),
                        "turn at (" + x + ", " + z + ") + " + d[0] + "," + d[1]);
                checked++;
            }
        }
        assertTrue(checked > 1000, "too few checks: " + checked);
    }

    /** Every band image of a stored block near an edge folds back to it, and every band block is someone's image. */
    private static void checkImages(WrapTopology t, int x0, int x1, int z0, int z1, int depth) {
        Random r = new Random(7);
        int withImages = 0;
        for (int i = 0; i < 4000; i++) {
            // Stored blocks within the band's width of an edge.
            int x = x0 + r.nextInt(x1 - x0), z = r.nextBoolean() ? z0 + r.nextInt(depth) : z1 - 1 - r.nextInt(depth);
            assertTrue(t.inside(x, z));
            List<int[]> images = t.images(x, z);
            if (!images.isEmpty())
                withImages++;
            for (int[] q : images) {
                assertFalse(t.inside(q[0], q[1]));
                WrapTopology.Fold f = t.fold(q[0], q[1]);
                assertEquals(x, f.x());
                assertEquals(z, f.z());
            }
        }
        assertEquals(4000, withImages, "every block within the band's width of an edge has an image");
        for (int i = 0; i < 4000; i++) {
            int x = x0 - depth + 1 + r.nextInt(x1 - x0 + 2 * depth - 2), z = z0 - depth + 1 + r.nextInt(z1 - z0 + 2 * depth - 2);
            if (t.inside(x, z))
                continue;
            WrapTopology.Fold f = t.fold(x, z);
            boolean found = t.images(f.x(), f.z()).stream().anyMatch(q -> q[0] == x && q[1] == z);
            assertTrue(found, "band block (" + x + ", " + z + ") is not listed as an image of its stored block");
        }
    }

    private record Canon(int x, int z, CellIsometry iso) {}

    private static Canon canon(WrapTopology t, int x, int z) {
        WrapTopology.Fold f = t.fold(x, z);
        return f == null ? new Canon(x, z, CellIsometry.IDENTITY) : new Canon(f.x(), f.z(), f.iso());
    }

    /** True when (x, z) is inside or within {@code reach} blocks beyond the edges (the fold is defined there). */
    private static boolean withinReach(WrapTopology t, int x, int z, int reach) {
        if (t.inside(x, z))
            return true;
        for (WrapEdge e : t.edges()) {
            if (e.beyond(x, z) && -e.distanceInside(x, z) - 1 < reach) {
                // Folding once must land inside or in another band within reach.
                WrapTopology.Fold f = t.fold(x, z);
                return f != null;
            }
        }
        return false;
    }
}
