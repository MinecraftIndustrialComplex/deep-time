// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.wrapengine.CellIsometry;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapLayouts;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapTopology;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The Create adapters' frame algebra (docs/design/wrap-engine.md, "Create across turning seams"): what a
 * turn does to a rotation axis and to the sign of a speed about it, and how a stored block finds its
 * neighbour's image across a seam. Properties, not pinned outputs.
 */
class SeamTurnsTest {

    private static final int[] AXES = {SeamTurns.X, SeamTurns.Y, SeamTurns.Z};

    /** An angular velocity s·e_axis carried by a turn is s·R(e_axis); the sign rule must say exactly that. */
    @Test
    void axisSignIsTheTurnedPositiveDirection() {
        for (int k = 0; k < 4; k++)
            for (int a : AXES) {
                double[] v = a == SeamTurns.Y ? new double[]{0, 0}
                        : new CellIsometry(k, 0, 0).applyVector(a == SeamTurns.X ? 1 : 0, a == SeamTurns.Z ? 1 : 0);
                int turned = SeamTurns.turnedAxis(k, a);
                if (a == SeamTurns.Y) {
                    assertEquals(SeamTurns.Y, turned);
                    assertEquals(1, SeamTurns.axisSign(k, a), "a vertical axis keeps its speed under any turn");
                    continue;
                }
                // The turned unit vector lies along the turned axis, with the sign the rule gives.
                double along = turned == SeamTurns.X ? v[0] : v[1];
                double across = turned == SeamTurns.X ? v[1] : v[0];
                assertEquals(0, across, 1e-12, "turn " + k + " axis " + a + " lands on axis " + turned);
                assertEquals(SeamTurns.axisSign(k, a), (int) Math.signum(along), "turn " + k + " axis " + a);
            }
    }

    @Test
    void halfTurnFlipsHorizontalSpeedsOnly() {
        assertEquals(-1, SeamTurns.axisSign(2, SeamTurns.X));
        assertEquals(-1, SeamTurns.axisSign(2, SeamTurns.Z));
        assertEquals(1, SeamTurns.axisSign(2, SeamTurns.Y));
        for (int a : AXES)
            assertEquals(a, SeamTurns.turnedAxis(2, a), "a half-turn keeps every axis");
    }

    @Test
    void quarterTurnSwapsHorizontalAxesAndFlipsOne() {
        for (int k : new int[]{1, 3}) {
            assertEquals(SeamTurns.Z, SeamTurns.turnedAxis(k, SeamTurns.X));
            assertEquals(SeamTurns.X, SeamTurns.turnedAxis(k, SeamTurns.Z));
            assertEquals(-1, SeamTurns.axisSign(k, SeamTurns.X) * SeamTurns.axisSign(k, SeamTurns.Z),
                    "exactly one of x and z flips under turn " + k);
        }
    }

    /** Signs compose like the turns: carrying a speed through two seams equals carrying it through their composite. */
    @Test
    void signsComposeAndUndo() {
        for (int k1 = 0; k1 < 4; k1++)
            for (int k2 = 0; k2 < 4; k2++)
                for (int a : AXES) {
                    int viaTwo = SeamTurns.axisSign(k1, a) * SeamTurns.axisSign(k2, SeamTurns.turnedAxis(k1, a));
                    assertEquals(SeamTurns.axisSign(k1 + k2, a), viaTwo, k1 + "+" + k2 + " axis " + a);
                    assertEquals(SeamTurns.turnedAxis(k1 + k2, a), SeamTurns.turnedAxis(k2, SeamTurns.turnedAxis(k1, a)));
                }
    }

    /**
     * For every stored block next to a seam and every neighbour that lies beyond it: the link is accepted
     * from both ends or refused from both (refused only right at cube corners and turn points, under a
     * tenth of the samples); an accepted link is that neighbour, next to the first block; and the two
     * links' turns cancel, so a speed sent across and back returns with its own sign (no flicker in
     * Create's propagation).
     */
    @Test
    void linksAcrossSeamsAreFoundBothWaysAndCancel() {
        List<WrapTopology> tops = List.of(
                WrapLayouts.halfTurnSeam(64, 32, 16, 16),
                WrapLayouts.guyou(256, 16, true),
                WrapLayouts.peirce(256, 16),
                WrapLayouts.quarterTurnCorner(48, 48, 256, 16),
                WrapLayouts.cubeCross(128, 16, true));
        int[][] dirs = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}, {1, 1}, {1, -1}, {-1, 1}, {-1, -1}};
        Random r = new Random(7);
        for (WrapTopology t : tops) {
            int checked = 0, refused = 0;
            for (int i = 0; i < 40000 && checked < 400; i++) {
                // A block just inside a random edge, anywhere along it (corners and turn points included).
                var e = t.edges().get(r.nextInt(t.edges().size()));
                int v = e.minSide() ? e.coord() + r.nextInt(2) : e.coord() - 1 - r.nextInt(2);
                int lo = Math.max(e.spanLo(), -4000), hi = Math.min(e.spanHi(), 4000);
                if (hi <= lo)
                    continue;
                int u = lo + r.nextInt(hi - lo);
                int ax = e.normalX() ? v : u, az = e.normalX() ? u : v;
                if (!t.inside(ax, az))
                    continue;
                for (int[] d : dirs) {
                    int qx = ax + d[0], qz = az + d[1];
                    WrapTopology.Fold f = t.fold(qx, qz);
                    if (f == null)
                        continue;
                    // At a turn point (cone point) a block can be its own neighbour across the seam; the
                    // kinetic adapter refuses that self-connection (KineticSeams.modifier), so skip it here.
                    if (f.x() == ax && f.z() == az)
                        continue;
                    // Next to a turn point or cube corner the far block can also be a direct neighbour; the
                    // adapter then connects the two only directly (symmetric), so no seam link is needed.
                    if (Math.max(Math.abs(f.x() - ax), Math.abs(f.z() - az)) <= 1)
                        continue;
                    int[] img = SeamTurns.seamLink(t, ax, az, f.x(), f.z());
                    int[] img2 = SeamTurns.seamLink(t, f.x(), f.z(), ax, az);
                    String where = "topology " + t.name() + " a=(" + ax + ", " + az + ") q=(" + qx + ", " + qz + ") f=(" + f.x() + ", " + f.z() + ")";
                    assertEquals(img == null, img2 == null, "a link is accepted from both ends or refused from both: " + where);
                    if (img == null) {
                        refused++;
                        checked++;
                        continue;
                    }
                    WrapTopology.Fold back = t.fold(img[0], img[1]);
                    assertNotNull(back);
                    assertArrayEquals(new int[]{f.x(), f.z()}, new int[]{back.x(), back.z()}, "the image folds to the neighbour: " + where);
                    assertTrue(Math.max(Math.abs(img[0] - ax), Math.abs(img[1] - az)) <= 1, "the image is a neighbour: " + where);
                    WrapTopology.Fold back2 = t.fold(img2[0], img2[1]);
                    assertEquals(0, (back.iso().quarterTurns() + back2.iso().quarterTurns()) % 4,
                            "the turns of a link and its way back cancel: " + where);
                    checked++;
                }
            }
            assertTrue(checked >= 100, "enough seam neighbours sampled: " + checked);
            // Refusals happen only right at cube corners and turn points, a small share of an edge.
            assertTrue(refused * 10 <= checked, t.name() + ": " + refused + " of " + checked + " seam neighbours refused");
        }
    }

    private static String imgs(List<int[]> l) {
        StringBuilder sb = new StringBuilder();
        for (int[] q : l)
            sb.append('(').append(q[0]).append(", ").append(q[1]).append(')');
        return sb.toString();
    }
}
