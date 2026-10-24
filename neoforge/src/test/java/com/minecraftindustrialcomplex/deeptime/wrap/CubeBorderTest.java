// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import com.minecraftindustrialcomplex.deeptime.server.SpawnHandler;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** The cube world's temporary border (cube plan W3 to W5, docs/design/cube-game.md section 5). */
class CubeBorderTest {

    private static final long S = 3840;   // the 16k cube
    private static final double H = S / 2.0, D = 1.0;
    private static final CubeBorder B = CubeBorder.of(S);

    @Test
    void theBorderHasTheTwelveTurningEdges() {
        assertEquals(12, B.segmentCount());
    }

    @Test
    void theFacesAreInsideAndTheVoidIsNot() {
        // Every face's centre, the band's ends at the slide, and F0's four corners.
        double[][] in = {{0, 0}, {S, 0}, {-S, 0}, {2 * S - 0.5, 0}, {-2 * S + 0.5, 0}, {0, -S}, {0, S},
                {H, -H}, {-H, -H}, {H, H}, {-H, H}, {H - 0.5, -H + 0.5}};
        for (double[] p : in)
            assertTrue(B.inside(p[0], p[1], D), "inside: " + p[0] + ", " + p[1]);
        // The void squares by F0's corners, past the polar faces, past the band's top and bottom.
        double[][] out = {{H + 5, -H - 5}, {-H - 5, H + 5}, {0, -3 * H - 0.5}, {S, -H - 0.5}, {-S, H + 0.5},
                {H - 0.5, -2 * H}, {2 * S - 10, -H}, {0, 3 * H}, {H - 0.5, -H - 0.5}};
        for (double[] p : out)
            assertFalse(B.inside(p[0], p[1], D), "outside: " + p[0] + ", " + p[1]);
        // The stop line is the inset inside the segment, the slide not bordered at all.
        assertTrue(B.inside(H - 1.0, -2 * H, D));
        assertFalse(B.inside(H - 0.99, -2 * H, D));
        assertTrue(B.inside(S, -H + 1.0, D));
        assertFalse(B.inside(S, -H + 0.99, D));
        assertTrue(B.inside(2 * S + 7, 0, D) && B.inside(-2 * S - 7, 0, D), "the slide is open (x wraps)");
    }

    @Test
    void theNearestPointIsInsideAndNearest() {
        double[][] pts = {{H + 5, -H - 5}, {H + 0.3, -H + 0.2}, {0, -3 * H - 40}, {S, -H - 30}, {-S - 7, 2 * H},
                {2 * S - 3, -H - 2}, {-2 * S + 3, H + 2}, {5 * S, -H - 0.5}};
        for (double[] p : pts) {
            double[] q = B.nearest(p[0], p[1], D);
            assertTrue(B.inside(q[0], q[1], D - 1e-9), "nearest of " + p[0] + ", " + p[1] + " is inside: " + q[0] + ", " + q[1]);
            double dist = Math.hypot(q[0] - p[0], q[1] - p[1]);
            // No point of a fine ring around p that is inside is nearer.
            for (int a = 0; a < 360; a += 3)
                for (double r = 0.05; r < dist - 1e-6; r += 0.05) {
                    double x = p[0] + r * Math.cos(Math.toRadians(a)), z = p[1] + r * Math.sin(Math.toRadians(a));
                    assertFalse(B.inside(x, z, D) && Math.hypot(x - p[0], z - p[1]) < dist - 1e-6,
                            "a nearer inside point " + x + ", " + z + " for " + p[0] + ", " + p[1]);
                }
        }
        double[] same = B.nearest(17, 23, D);
        assertEquals(17, same[0]);
        assertEquals(23, same[1]);
        // A point past the window's east end comes back in its own period.
        double[] east = B.nearest(4 * S + S, -H - 30, D);
        assertEquals(4 * S + S, east[0], 1e-9);
        assertEquals(-H + D, east[1], 1e-9);
    }

    @Test
    void zonesWarnNearTheSegmentsOnly() {
        assertEquals(0, B.zone(0, 0, D, EdgeBorder.WARN_BLOCKS));
        assertEquals(1, B.zone(S, -H + 10, D, EdgeBorder.WARN_BLOCKS));
        assertEquals(2, B.zone(S, -H + 0.5, D, EdgeBorder.WARN_BLOCKS));
        // The direct seams (F0 with N, F1, F3 and S) are open: no warning in the middle of one.
        assertEquals(0, B.zone(0, -H, D, EdgeBorder.WARN_BLOCKS));
        assertEquals(0, B.zone(H, 0, D, EdgeBorder.WARN_BLOCKS));
        assertEquals(Math.sqrt(2) * H, B.distanceToSegment(0, 0), 1e-9);
    }

    @Test
    void anArrivalDeepOutsideFoldsOntoThePlanet() {
        CubeLayout cube = B.cube();
        // Beyond N's east edge: the fill there is F1's top band, a quarter turn away.
        double x = H + 40, z = -2 * H;
        assertTrue(B.depthOutside(x, z) > EdgeBorder.FOLD_DEPTH);
        var p = cube.foldPoint(x, z);
        assertNotNull(p);
        assertTrue(p.iso().k() == 1 || p.iso().k() == 3, "a quarter turn: " + p.iso());
        double[] q = B.nearest(B.frameX(p.x()), p.z(), D);
        assertTrue(B.inside(q[0], q[1], D - 1e-9));
        assertEquals(0.0, Math.hypot(q[0] - p.x(), q[1] - p.z()), 1e-9, "the partner point is well inside F1");
        // Beyond F2's top edge (a half turn onto N's top edge).
        var t = cube.foldPoint(2 * S, -H - 50);
        assertNotNull(t);
        assertEquals(2, t.iso().k());
    }

    @Test
    void headingsTurnWithTheMotion() {
        CubeLayout.Iso id = CubeLayout.Iso.IDENTITY;
        assertEquals(37.0f, CubeBorder.turnYaw(37.0f, id), 1e-4);
        assertEquals(-143.0f, CubeBorder.turnYaw(37.0f, CubeLayout.Iso.halfTurnAbout(0, 0)), 1e-4);
        // A quarter turn takes a heading through 90 degrees, and four bring it back.
        CubeLayout.Iso q = CubeLayout.Iso.quarterTurnAbout(1, 0, 0);
        float y = CubeBorder.turnYaw(10.0f, q);
        double turned = Math.abs(y - 10.0f);
        assertTrue(Math.abs(turned - 90.0) < 1e-3 || Math.abs(turned - 270.0) < 1e-3, "a quarter turn: " + y);
        float back = CubeBorder.turnYaw(CubeBorder.turnYaw(CubeBorder.turnYaw(y, q), q), q);
        assertEquals(10.0f, back, 1e-3);
        // The heading follows the motion's vector: facing +z (yaw 0) is the direction (0, 1).
        double[] v = q.vector(0, 1);
        double want = Math.toDegrees(Math.atan2(-v[0], v[1]));
        assertEquals(want, CubeBorder.turnYaw(0.0f, q), 1e-4);
    }

    @Test
    void spawnKeepsAwayFromSegmentsAndCorners() {
        // SpawnHandler's margins are compile-time constants (inlined: the test loads no Minecraft class).
        double edge = SpawnHandler.EDGE_MARGIN, corner = SpawnHandler.CORNER_MARGIN;
        assertTrue(B.spawnOk(0, 0, edge, corner));
        assertTrue(B.spawnOk((int) S, 0, edge, corner), "F1's centre");
        assertFalse(B.spawnOk((int) (H - 100), (int) (-H + 100), edge, corner), "within 256 of F0's corner");
        assertFalse(B.spawnOk((int) (H - 30), (int) (-2 * H), edge, corner), "within 64 of N's east edge");
        assertFalse(B.spawnOk((int) (H + 30), (int) (-2 * H), edge, corner), "off the planet");
        assertTrue(B.spawnOk((int) (H - 100), (int) (-2 * H), edge, corner), "100 blocks inside N's east edge");
    }

    /**
     * Cube plan G11, the walk under the border: straight wherever the segment stays on the planet
     * (in the band, through the slide, from N to S through F0, through an inner corner exactly), bent
     * round the inner corners (±h, ±h) otherwise; symmetric and never shorter than the straight line.
     */
    @Test
    void theWalkGoesStraightOrRoundTheInnerCorners() {
        assertEquals(0.0, B.walkDistance(17, 23, 17, 23), 1e-9);
        assertEquals(Math.hypot(3000, 350), B.walkDistance(-1000, 50, 2000, -300), 1e-9, "in the band");
        assertEquals(200.0, B.walkDistance(2 * S - 100, 0, -2 * S + 100, 0), 1e-9, "through the slide");
        assertEquals(4 * H, B.walkDistance(0, -2 * H, 0, 2 * H), 1e-9, "N to S through F0");
        assertEquals(Math.hypot(400, 4.7 * H), B.walkDistance(100, -2.5 * H, -300, 2.2 * H), 1e-9, "down the column");
        assertEquals(2 * Math.sqrt(2) * H, B.walkDistance(0, -2 * H, 2 * H, 0), 1e-6, "N to F1 through the corner exactly");
        assertEquals((Math.sqrt(2) + Math.hypot(1, 0.5)) * H, B.walkDistance(0, -2 * H, 2 * H, -H / 2), 1e-6,
                "N to F1 round the corner");
        assertEquals((Math.sqrt(2) + Math.sqrt(10)) * H, B.walkDistance(0, -2 * H, 4 * H, 0), 1e-6,
                "N to F2's centre round either corner");
        assertEquals((Math.sqrt(2) + Math.sqrt(10)) * H, B.walkDistance(0, 2 * H, -4 * H, 0), 1e-6,
                "S to F2's centre, the other way round");
        // The straight line leaves N's column at z = -h/2, inside the band, so it cuts inside the corner.
        assertEquals(Math.hypot(2 * H, 3 * H), B.walkDistance(0, -2 * H, 2 * H, H), 1e-6, "N to F1's lower half, straight");
        assertTrue(Double.isNaN(B.walkDistance(H + 5, -H - 5, 0, 0)), "from off the planet");
        java.util.SplittableRandom rnd = new java.util.SplittableRandom(11);
        int pairs = 0;
        while (pairs < 300) {
            double ax = rnd.nextDouble(-2 * S, 2 * S), az = rnd.nextDouble(-3 * H, 3 * H);
            double bx = rnd.nextDouble(-2 * S, 2 * S), bz = rnd.nextDouble(-3 * H, 3 * H);
            if (!B.inside(ax, az, 0) || !B.inside(bx, bz, 0))
                continue;
            double w = B.walkDistance(ax, az, bx, bz), dx = Math.abs(B.frameX(bx) - B.frameX(ax));
            double straight = Math.hypot(Math.min(dx, 4 * S - dx), bz - az);
            assertTrue(w >= straight - 1e-6 && w < 4 * S + 6 * H, "the walk " + w + " against the straight " + straight);
            assertEquals(w, B.walkDistance(bx, bz, ax, az), 1e-6, "symmetric");
            pairs++;
        }
    }

    /**
     * Cube plan G15, reachability: a flood fill of the net at chunk resolution, through the chunks
     * whose centre is inside the stop line (x wrapping through Toroidal World's frame), reaches all
     * six faces from spawn at (0, 0), and every chunk of every face whose centre is inside.
     */
    @Test
    void everyFaceIsReachableFromSpawnUnderTheBorder() {
        for (long s : new long[]{1024, 3840}) {
            CubeBorder b = CubeBorder.of(s);
            CubeLayout cube = b.cube();
            long period = cube.period(), h = s / 2;
            int nx = (int) (period / 16), z0 = (int) (-(3 * h + 64) / 16), nz = (int) ((6 * h + 128) / 16);
            boolean[][] open = new boolean[nx][nz], seen = new boolean[nx][nz];
            for (int i = 0; i < nx; i++)
                for (int j = 0; j < nz; j++)
                    open[i][j] = b.inside(-period / 2.0 + 16 * i + 8, 16.0 * (z0 + j) + 8, D);
            int si = (int) (period / 2 / 16), sj = -z0;
            assertTrue(open[si][sj], "spawn's chunk is open");
            java.util.ArrayDeque<int[]> queue = new java.util.ArrayDeque<>();
            queue.add(new int[]{si, sj});
            seen[si][sj] = true;
            java.util.EnumSet<CubeLayout.Face> faces = java.util.EnumSet.noneOf(CubeLayout.Face.class);
            int reached = 0;
            while (!queue.isEmpty()) {
                int[] c = queue.poll();
                reached++;
                var fc = cube.faceOfBlock(-period / 2 + 16L * c[0] + 8, 16L * (z0 + c[1]) + 8);
                if (fc != null)
                    faces.add(fc.face());
                for (int[] d : new int[][]{{1, 0}, {-1, 0}, {0, 1}, {0, -1}}) {
                    int i = Math.floorMod(c[0] + d[0], nx), j = c[1] + d[1];
                    if (j >= 0 && j < nz && open[i][j] && !seen[i][j]) {
                        seen[i][j] = true;
                        queue.add(new int[]{i, j});
                    }
                }
            }
            int openCount = 0;
            for (boolean[] col : open)
                for (boolean o : col)
                    if (o)
                        openCount++;
            assertEquals(6, faces.size(), "s " + s + ": faces reached " + faces);
            assertEquals(openCount, reached, "s " + s + ": every open chunk is reached");
            assertEquals(6 * (s / 16) * (s / 16), reached, "s " + s + ": the open chunks are the six faces' chunks");
        }
    }
}
