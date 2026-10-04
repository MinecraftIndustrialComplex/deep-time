// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.function.BiPredicate;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The wrap engine's gluing algebra (docs/design/wrap-engine.md §1): block motions compose and invert,
 * every band block folds to a stored block, every stored block's band images fold back to it, and the
 * fold is a local isometry: two neighbouring blocks anywhere in the bands stay neighbours, turned the
 * same way, after folding. Checked on the bounded test seams, Guyou (with and without the slides),
 * Peirce and the cube cross net, on seeded random blocks (properties, not pinned outputs). Plus the
 * fast path (no position it rejects folds) and the band manager's chunk-level view (every block of a
 * band chunk folds by the chunk's motion into the chunk's source).
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
            assertEquals(a.apply(x, z)[0], a.applyX(x, z));
            assertEquals(a.apply(x, z)[1], a.applyZ(x, z));
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
        // Chunk alignment: turns about chunk corners and centres map chunks to chunks; others do not.
        assertTrue(CellIsometry.halfTurnAbout(16, 32).mapsChunksToChunks());
        assertTrue(CellIsometry.halfTurnAbout(8, 24).mapsChunksToChunks(), "a half-turn about a chunk centre");
        assertTrue(CellIsometry.quarterTurnAbout(1, 32, -16).mapsChunksToChunks());
        assertTrue(CellIsometry.quarterTurnAbout(3, 8, 8).mapsChunksToChunks(), "a quarter-turn about a chunk centre");
        assertFalse(CellIsometry.halfTurnAbout(4, 0).mapsChunksToChunks());
        assertFalse(CellIsometry.translation(8, 0).mapsChunksToChunks());
    }

    @Test
    void edgesRefuseEmptyOrUnboundedBands() {
        assertThrows(IllegalArgumentException.class, () -> new WrapEdge("e", false, true, 0, 0, 16, 0, CellIsometry.IDENTITY));
        assertThrows(IllegalArgumentException.class,
                () -> new WrapEdge("e", false, true, 0, Integer.MIN_VALUE, Integer.MAX_VALUE, 16, CellIsometry.IDENTITY));
    }

    @Test
    void theHalfTurnSeamGluesAsTheDesignSays() {
        // Beyond the edge, (x, z) is (2cx − x − 1, 2z0 − z − 1): the block form of (−x, 2b − z).
        int cx = 4096, z0 = 4096;
        WrapTopology t = WrapLayouts.halfTurnSeam(cx, z0, 32, 16);
        assertNull(t.fold(cx, z0), "inside is not folded");
        WrapTopology.Fold f = t.fold(cx + 2, z0 - 1);
        assertNotNull(f);
        assertEquals(cx - 3, f.x());
        assertEquals(z0, f.z());
        assertEquals(2, f.iso().quarterTurns());
        assertNull(t.fold(cx + 32, z0 - 1), "outside the seam's span nothing folds");
        assertNull(t.fold(cx, z0 - 17), "beyond the band nothing folds");
        List<int[]> img = t.images(cx - 3, z0);
        assertEquals(1, img.size());
        assertArrayEquals(new int[]{cx + 2, z0 - 1}, img.get(0));
        assertTrue(t.images(cx - 3, z0 + 16).isEmpty(), "a block deeper inside than the band has no image");
        checkLocalIsometry(t, cx - 32, cx + 32, z0 - 16, z0 + 16, 16, (x, z) -> x >= cx - 32 && x < cx + 32);
    }

    @Test
    void guyouFoldsEveryBandBlockAndStaysSeamless() {
        int h = 512;
        WrapTopology t = WrapLayouts.guyou(h, 32, true);
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
        // A corner block slides, then turns: two steps, one stored block.
        WrapTopology.Fold corner = t.fold(h + 3, -h / 2 - 2);
        assertNotNull(corner);
        assertTrue(t.inside(corner.x(), corner.z()));
        checkLocalIsometry(t, -h - 31, h + 31, -h / 2 - 31, h / 2 + 31, 32);
        checkImages(t, -h, h, -h / 2, h / 2, 32);
    }

    @Test
    void guyouTurnsOnlyLeavesTheSlideToToroidalWorld() {
        int h = 512;
        WrapTopology t = WrapLayouts.guyou(h, 32, false);
        assertEquals(2, t.edges().size());
        assertNull(t.fold(h + 3, 0), "east of the rectangle is Toroidal World's, not the engine's");
        assertNotNull(t.fold(h - 3, -h / 2 - 1));
        checkLocalIsometry(t, -h, h, -h / 2 - 31, h / 2 + 31, 32, (x, z) -> x >= -h && x < h);
    }

    @Test
    void peirceFoldsEveryBandBlockAndStaysSeamless() {
        int a = 640;
        WrapTopology t = WrapLayouts.peirce(a, 32);
        checkLocalIsometry(t, -a / 2 - 31, a / 2 + 31, -a / 2 - 31, a / 2 + 31, 32);
        checkImages(t, -a / 2, a / 2, -a / 2, a / 2, 32);
    }

    @Test
    void quarterTurnsCloseToo() {
        WrapTopology t = WrapLayouts.quarterTurnCorner(0, 0, 256, 16);
        WrapTopology.Fold f = t.fold(5, -2);
        assertNotNull(f);
        assertEquals(1, f.x());
        assertEquals(5, f.z());
        assertEquals(1, f.iso().quarterTurns());
        // Each band is the other strip turned: the west band's fold is the inverse turn.
        WrapTopology.Fold g = t.fold(-2, 5);
        assertNotNull(g);
        assertEquals(3, g.iso().quarterTurns());
        assertNull(t.fold(-2, -2), "the missing quadrant of the cone point stays unglued");
        checkLocalIsometry(t, -15, 240, -15, 240, 16, (x, z) -> (x >= 0 || z >= 0) && x < 256 && z < 256);
    }

    @Test
    void theCubeCrossIsSeamlessAwayFromItsCorners() {
        int s = 256, d = 32;
        for (boolean slides : new boolean[]{true, false}) {
            WrapTopology t = WrapLayouts.cubeCross(s, d, slides);
            int[][] corners = WrapLayouts.cubeCrossCorners(s);
            BiPredicate<Integer, Integer> awayFromCorners = (x, z) -> {
                for (int[] c : corners)
                    if (Math.max(Math.abs(x - c[0]), Math.abs(z - c[1])) <= d + 2)
                        return false;
                // Without the slides, the band row's ends are Toroidal World's.
                return slides || (x >= 0 && x < 4 * s) || z < 0 || z >= s;
            };
            checkLocalIsometry(t, -d, 4 * s + d, -s - d, 2 * s + d, d, awayFromCorners);
            // The meridian walk of the survey: north out of F0 over the pole N comes out on F2 heading south.
            WrapTopology.PointFold over = t.foldPoint(s / 2.0 + 0.25, -s - 1.5);
            assertNotNull(over);
            assertTrue(over.x() >= 2 * s && over.x() < 3 * s && over.z() >= 0 && over.z() < s, "lands on F2: " + over);
            assertEquals(180f, over.iso().yawDegrees());
            // From F1 into N through a quarter-turn.
            WrapTopology.PointFold f1 = t.foldPoint(1.5 * s, -1.5);
            assertNotNull(f1);
            assertEquals(270f, f1.iso().yawDegrees());
        }
        WrapTopology t = WrapLayouts.cubeCross(s, d, true);
        // Images: stored blocks near the middle of each seam (away from corners) have one.
        Random r = new Random(11);
        int checked = 0;
        for (int i = 0; i < 20000; i++) {
            int x = -d + r.nextInt(4 * s + 2 * d), z = -s - d + r.nextInt(3 * s + 2 * d);
            if (t.inside(x, z) || !t.mayFold(x, z))
                continue;
            WrapTopology.Fold f = t.fold(x, z);
            if (f == null)
                continue;
            boolean found = t.images(f.x(), f.z()).stream().anyMatch(q -> q[0] == x && q[1] == z);
            assertTrue(found, "band block (" + x + ", " + z + ") is not listed as an image of its stored block");
            checked++;
        }
        assertTrue(checked > 500, "too few band blocks checked: " + checked);
    }

    @Test
    void theFastPathNeverRejectsABandBlock() {
        List<WrapTopology> all = layouts();
        Random r = new Random(5);
        for (WrapTopology t : all) {
            int rejected = 0;
            for (int i = 0; i < 200_000; i++) {
                int x = r.nextInt(8192) - 4096, z = r.nextInt(8192) - 4096;
                if (!t.mayFold(x, z)) {
                    rejected++;
                    for (WrapEdge e : t.edges())
                        assertFalse(e.beyond(x, z), t + ": fast path rejected (" + x + ", " + z + ") beyond " + e.name());
                }
            }
            assertTrue(rejected > 0, t + ": the fast path rejected nothing");
        }
        // On a whole-world layout every block deeper inside than the band (plus one grid cell) is fast.
        WrapTopology g = WrapLayouts.guyou(1024, 32, true);
        for (int i = 0; i < 100_000; i++) {
            int x = -1024 + 48 + r.nextInt(2048 - 96), z = -512 + 48 + r.nextInt(1024 - 96);
            assertFalse(g.mayFold(x, z), "interior block (" + x + ", " + z + ") takes the slow path");
        }
    }

    @Test
    void everyBlockOfABandChunkFoldsByTheChunksMotion() {
        Random r = new Random(9);
        for (WrapTopology t : layouts()) {
            assertTrue(t.isChunkAligned(), t + " is chunk-aligned");
            List<Long> band = new ArrayList<>();
            t.forEachBandChunk(band::add);
            assertFalse(band.isEmpty(), t + " has band chunks");
            for (long key : band) {
                int cx = (int) key, cz = (int) (key >> 32);
                WrapTopology.ChunkFold cf = t.chunkFold(cx, cz);
                assertNotNull(cf, t + ": band chunk " + cx + "," + cz + " does not fold");
                for (int i = 0; i < 12; i++) {
                    int x = (cx << 4) + r.nextInt(16), z = (cz << 4) + r.nextInt(16);
                    WrapTopology.Fold f = t.fold(x, z);
                    assertNotNull(f);
                    assertEquals(cf.iso(), f.iso(), t + ": block (" + x + ", " + z + ") folds by another motion than its chunk");
                    assertEquals(cf.sourceX(), f.x() >> 4);
                    assertEquals(cf.sourceZ(), f.z() >> 4);
                }
                // The source chunk lists this band chunk among its images.
                boolean listed = false;
                for (CellIsometry g : t.chunkImages(cf.sourceX(), cf.sourceZ())) {
                    int bx = Math.min(g.applyX(cf.sourceX() << 4, cf.sourceZ() << 4), g.applyX((cf.sourceX() << 4) + 15, (cf.sourceZ() << 4) + 15));
                    int bz = Math.min(g.applyZ(cf.sourceX() << 4, cf.sourceZ() << 4), g.applyZ((cf.sourceX() << 4) + 15, (cf.sourceZ() << 4) + 15));
                    if (bx >> 4 == cx && bz >> 4 == cz) {
                        listed = true;
                        assertEquals(cf.iso().inverse(), g);
                    }
                }
                assertTrue(listed, t + ": band chunk " + cx + "," + cz + " missing from its source's images");
            }
        }
        assertFalse(WrapLayouts.halfTurnSeam(4100, 4096, 32, 16).isChunkAligned(), "a pivot off the chunk grid is refused");
    }

    private static List<WrapTopology> layouts() {
        return List.of(WrapLayouts.halfTurnSeam(4096, 4096, 32, 16), WrapLayouts.quarterTurnCorner(512, -512, 256, 16),
                WrapLayouts.guyou(512, 32, true), WrapLayouts.guyou(512, 32, false), WrapLayouts.peirce(640, 32),
                WrapLayouts.cubeCross(256, 32, true), WrapLayouts.cubeCross(256, 32, false),
                WrapTopology.union("sites", List.of(WrapLayouts.halfTurnSeam(20480, 20480, 32, 16),
                        WrapLayouts.quarterTurnCorner(21504, 20480, 48, 16))));
    }

    /**
     * For random blocks q in the bands (or inside) and each horizontal step d with q + d also within
     * reach: folding q + d gives the same stored block as stepping from fold(q) in the turned direction
     * R(d) and folding again, with the same total turn. That is what makes the seam invisible to
     * fluids, redstone and collisions.
     */
    private static void checkLocalIsometry(WrapTopology t, int x0, int x1, int z0, int z1, int depth) {
        checkLocalIsometry(t, x0, x1, z0, z1, depth, (x, z) -> true);
    }

    /**
     * As above, leaving out the blocks {@code valid} rejects: past the ends of a bounded seam, the
     * missing quadrant of a cone point, a neighbourhood of each cube corner.
     */
    private static void checkLocalIsometry(WrapTopology t, int x0, int x1, int z0, int z1, int depth,
                                           BiPredicate<Integer, Integer> valid) {
        Random r = new Random(42);
        int checked = 0;
        for (int i = 0; i < 60000; i++) {
            int x = x0 + r.nextInt(x1 - x0), z = z0 + r.nextInt(z1 - z0);
            if (!withinReach(t, x, z, depth - 1) || !valid.test(x, z))
                continue;
            Canon q = canon(t, x, z);
            for (int[] d : DIRS) {
                if (!withinReach(t, x + d[0], z + d[1], depth - 1) || !valid.test(x + d[0], z + d[1]))
                    continue;
                Canon direct = canon(t, x + d[0], z + d[1]);
                double[] rd = q.iso.applyVector(d[0], d[1]);
                int sx = q.x + (int) Math.round(rd[0]), sz = q.z + (int) Math.round(rd[1]);
                if (!valid.test(sx, sz))
                    continue;
                Canon stepped = canon(t, sx, sz);
                assertEquals(direct.x, stepped.x, t + ": x at (" + x + ", " + z + ") + " + d[0] + "," + d[1]);
                assertEquals(direct.z, stepped.z, t + ": z at (" + x + ", " + z + ") + " + d[0] + "," + d[1]);
                assertEquals(direct.iso.quarterTurns(), stepped.iso.compose(q.iso).quarterTurns(),
                        t + ": turn at (" + x + ", " + z + ") + " + d[0] + "," + d[1]);
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
        boolean beyondAny = false;
        for (WrapEdge e : t.edges()) {
            if (e.beyond(x, z)) {
                beyondAny = true;
                if (-e.distanceInside(x, z) - 1 < reach)
                    return t.fold(x, z) != null;
            }
        }
        if (beyondAny)
            return false;
        // Not beyond any edge: a stored block, unless it lies past the band (outside the world).
        return isStored(t, x, z);
    }

    /** Inside the domain: a block some band image could fold to. Approximated as "every edge's coordinate on its inner side". */
    private static boolean isStored(WrapTopology t, int x, int z) {
        for (WrapEdge e : t.edges()) {
            int u = e.normalX() ? z : x;
            if (u < e.spanLo() || u >= e.spanHi())
                continue;
            int v = e.normalX() ? x : z;
            if (e.minSide() ? v < e.coord() : v >= e.coord())
                return false;
        }
        return true;
    }
}
