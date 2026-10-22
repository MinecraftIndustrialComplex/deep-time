// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Random;
import java.util.stream.IntStream;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The lava flows ({@link LavaFlows}; review fix 8, 2026-10-22): the paths and the paint, on cones built here
 * in closed form so nothing depends on a planet. The chunk pass and far terrain only read {@link LavaFlows#code}.
 */
class LavaFlowsTest {

    private static final int C = 4096;
    private static final double MH = 300.0, MV = 20.0, Z_MIN = -1500.0, Z_MAX = 1500.0, BASE = 64.0;
    private static final double[] ISO = {MH * MH, 0.0, MH * MH};

    private static Volcanoes.Volcano shield(int id, double x, double z, double activity) {
        // 150 blocks across its base, a 12-block crater 6 blocks deep, 100 blocks high.
        return new Volcanoes.Volcano(id, "shield", x, z, 45_000, 2_000, activity, true, false, 3_600, 120, false);
    }

    private static Volcanoes.Volcano strato(int id, double x, double z, double activity) {
        // 46.7 blocks across its base, the same crater, 100 blocks high.
        return new Volcanoes.Volcano(id, "stratovolcano", x, z, 14_000, 2_000, activity, false, false, 3_600, 120, false);
    }

    /** x - x0 folded into [−C/2, C/2). */
    private static double delta(double x, double x0) {
        double d = (x - x0 + C / 2.0) % C;
        if (d < 0)
            d += C;
        return d - C / 2.0;
    }

    /**
     * Closed-form cones on a plain that falls gently away from them: each cone's flank falls linearly from its
     * crater's rim (y {@code BASE + height}) to its base radius, its crater drops inside the rim, and the plain
     * beyond falls 0.05 blocks a block (0.0033 m/m). The ground is the highest cone's.
     */
    private static class Cones implements LavaFlows.Env {
        final List<Volcanoes.Volcano> vs;
        double precip = 1_000.0;
        double wetFromX = Double.POSITIVE_INFINITY;
        double[] metric = ISO;

        Cones(List<Volcanoes.Volcano> vs) {
            this.vs = vs;
        }

        @Override
        public double height(double x, double z) {
            double h = Double.NEGATIVE_INFINITY;
            for (Volcanoes.Volcano v : vs) {
                double d = Math.hypot(delta(x, v.x()), z - v.z());
                double rB = v.radiusM() / MH, cB = v.craterRadiusM() / MH, top = BASE + v.heightM() / MV;
                double here;
                if (d < cB)
                    here = top - v.craterDepthM() / MV * (1.0 - d / cB);
                else
                    here = Math.max(BASE + (top - BASE) * (rB - d) / (rB - cB), BASE - 0.05 * (d - rB));
                h = Math.max(h, here);
            }
            return h;
        }

        @Override
        public boolean wet(int x, int z) {
            return delta(x, 0.0) >= wetFromX;
        }

        @Override
        public double precipMm(double x, double z) {
            return precip;
        }

        @Override
        public double[] metric(double x, double z) {
            return metric.clone();
        }
    }

    private static Cones cones(List<Volcanoes.Volcano> vs, double precipMm) {
        Cones c = new Cones(vs);
        c.precip = precipMm;
        return c;
    }

    private static LavaFlows flows(LavaFlows.Env env, List<Volcanoes.Volcano> vs) {
        return new LavaFlows(vs, env, C, Z_MIN, Z_MAX, MH, MV, null, true);
    }

    private static LavaFlows flows(Cones env) {
        return flows(env, env.vs);
    }

    private static int[] box(LavaFlows f, int x0, int z0, int x1, int z1) {
        int w = x1 - x0 + 1;
        int[] out = new int[w * (z1 - z0 + 1)];
        for (int z = z0; z <= z1; z++)
            for (int x = x0; x <= x1; x++)
                out[(z - z0) * w + (x - x0)] = f.code(x, z);
        return out;
    }

    private static double thicknessBlocks(LavaFlows.Group g) {
        return (g == LavaFlows.Group.BASALTIC ? LavaFlows.BASALT_THICK_M : LavaFlows.VISCOUS_THICK_M) / MV;
    }

    // ---- paths ----

    @Test
    void pathsOnAConeOnlyGoDown() {
        Cones env = new Cones(List.of(shield(1, 0, 0, 1.0), strato(2, 1_200, 0, 1.0)));   // apart: no valley between
        LavaFlows f = flows(env);
        int checked = 0;
        for (int i = 0; i < f.size(); i++) {
            assertFalse(f.flows(i).isEmpty(), "volcano " + i + " shows flows");
            for (LavaFlows.Flow fl : f.flows(i)) {
                List<double[]> p = fl.path();
                assertEquals(fl.steps() + 1, p.size());
                for (int k = 1; k < p.size(); k++) {
                    assertTrue(p.get(k)[2] <= p.get(k - 1)[2] + 1e-9, "a step climbs on a cone: " + fl);
                    checked++;
                }
                if (p.size() > 2)
                    assertTrue(p.get(p.size() - 1)[2] < p.get(0)[2]);
            }
        }
        assertTrue(checked > 1_000, "steps checked: " + checked);
    }

    @Test
    void aHollowFillsOnlyToTheFlowsThickness() {
        // A ramp falling east (0.2 blocks a block) into a trough at x = 20, rising 0.5 a block beyond it.
        LavaFlows.Env env = new LavaFlows.Env() {
            public double height(double x, double z) {
                double dx = delta(x, 0.0);
                return dx < 20.0 ? 100.0 - 0.2 * dx : 96.0 + 0.5 * (dx - 20.0);
            }

            public boolean wet(int x, int z) {
                return false;
            }

            public double precipMm(double x, double z) {
                return 300.0;
            }

            public double[] metric(double x, double z) {
                return ISO.clone();
            }
        };
        // A viscous vent (no flank vents) with no crater: its flows fill the trough 2 blocks deep (40 m).
        Volcanoes.Volcano v = new Volcanoes.Volcano(1, "stratovolcano", 0, 0, 14_000, 200, 1.0, false, false, 0, 0, false);
        LavaFlows f = flows(env, List.of(v));
        double thick = thicknessBlocks(LavaFlows.Group.VISCOUS);
        assertTrue(f.flows(0).size() > 20, f.summary(0).describe());
        assertTrue(f.flows(0).stream().anyMatch(fl -> !fl.stop().equals("length")), "some reach the trough: " + f.summary(0).describe());
        for (LavaFlows.Flow fl : f.flows(0)) {
            double low = Double.POSITIVE_INFINITY;
            for (double[] p : fl.path()) {
                // a flow may overshoot the floor by one step before it sees the way back
                assertTrue(p[2] <= low + thick + 0.5 * LavaFlows.STEP + 1e-9, "climbed out of the hollow: " + fl);
                low = Math.min(low, p[2]);
            }
        }
        // 2 blocks up a 0.5 slope is 4 blocks past the floor; then a step, the side column and the toe lobe
        for (int z = -250; z <= 250; z++)
            for (int x = 29; x <= 250; x++)
                assertEquals(-1, f.code(x, z), "painted beyond the trough at " + x + ", " + z);
    }

    @Test
    void aFlowThatStartsOnLevelGroundRunsOutFromTheVent() {
        // A shield with no crater whose top is a level plateau 9 blocks in radius (as where the elevation raster
        // holds a level patch over a gentle summit): its summit flows start on level ground.
        Volcanoes.Volcano v = new Volcanoes.Volcano(1, "shield", 0, 0, 45_000, 2_000, 1.0, true, false, 0, 0, false);
        Cones env = new Cones(List.of(v)) {
            @Override
            public double height(double x, double z) {
                return Math.min(super.height(x, z), BASE + 2_000 / MV - 6.0);
            }
        };
        LavaFlows f = flows(env);
        int out = 0, summit = 0;
        for (LavaFlows.Flow fl : f.flows(0)) {
            if (fl.flank())
                continue;
            summit++;
            double r = Math.hypot(fl.endX(), fl.endZ());
            assertTrue(fl.stop().equals("length") || r > 9.0, "stopped on the plateau: " + fl);
            if (r > 9.5)
                out++;
            List<double[]> p = fl.path();
            for (int k = 1; k < p.size(); k++)
                assertTrue(p.get(k)[2] <= p.get(k - 1)[2] + 1e-9, "a step climbs: " + fl);
        }
        assertTrue(summit > 20 && out > summit / 2, "summit flows " + summit + ", off the plateau " + out);
    }

    @Test
    void summitFlowsStartPastTheCratersRim() {
        Cones env = new Cones(List.of(shield(1, 0, 0, 1.0), strato(2, 1_200, 0, 1.0)));   // apart: no valley between
        LavaFlows f = flows(env);
        for (int i = 0; i < f.size(); i++) {
            Volcanoes.Volcano v = env.vs.get(i);
            double rim = v.craterRadiusM() / MH;
            for (LavaFlows.Flow fl : f.flows(i)) {
                double r = Math.hypot(fl.startX() - v.x(), fl.startZ() - v.z());
                assertTrue(r >= rim + LavaFlows.RIM_PAST - 1e-6, "starts inside the rim: " + r);
                if (fl.flank())
                    assertTrue(r <= LavaFlows.FLANK_MAX_SHARE * v.radiusM() / MH + 1e-6, "a flank vent too far out: " + r);
                else
                    assertEquals(rim + LavaFlows.RIM_PAST, r, 1e-6);
                assertTrue(env.height(fl.startX(), fl.startZ()) < env.height(v.x() + rim, v.z()) + 1e-9);
            }
        }
    }

    @Test
    void aFlowUsesItsLengthAndNoMore() {
        Cones env = new Cones(List.of(shield(1, 0, 0, 1.0), strato(2, 1_200, 0, 1.0)));   // apart: no valley between
        LavaFlows f = flows(env);
        int full = 0;
        for (int i = 0; i < f.size(); i++) {
            boolean basaltic = LavaFlows.group(env.vs.get(i)) == LavaFlows.Group.BASALTIC;
            double lo = basaltic ? LavaFlows.BASALT_MIN_M : LavaFlows.VISCOUS_MIN_M;
            double hi = basaltic ? LavaFlows.BASALT_MAX_M : LavaFlows.VISCOUS_MAX_M;
            for (LavaFlows.Flow fl : f.flows(i)) {
                assertTrue(fl.lengthM() >= lo && fl.lengthM() <= hi, "length out of bounds: " + fl.lengthM());
                assertTrue(fl.travelledM() <= fl.lengthM() + LavaFlows.STEP * MH + 1e-6, fl.toString());
                if (fl.stop().equals("length")) {
                    assertTrue(fl.travelledM() >= fl.lengthM());
                    full++;
                }
                assertEquals(fl.aaFromVent(), basaltic && fl.lengthM() > Math.sqrt(lo * hi));
            }
        }
        assertTrue(full > 10, "flows that ran their full length: " + full);
    }

    @Test
    void flowsAreYoungestFirstAndInsideTheirWindow() {
        Cones env = new Cones(List.of(shield(1, 0, 0, 1.0)));
        env.precip = 2_000.0;
        LavaFlows f = flows(env);
        LavaFlows.Summary s = f.summary(0);
        assertEquals(LavaFlows.interpolate(LavaFlows.VISIBLE_YR, 2_000.0), s.windowYr(), 1e-9);
        double last = 0.0;
        for (LavaFlows.Flow fl : f.flows(0)) {
            assertTrue(fl.ageYr() > last);
            assertTrue(fl.ageYr() < s.windowYr());
            last = fl.ageYr();
        }
        assertEquals(s.flows(), f.flows(0).size());
    }

    // ---- the metric ----

    @Test
    void theMetricsDescentAndLengths() {
        LavaFlows.Metric m = new LavaFlows.Metric(new double[]{9 * MH * MH, 2 * MH * MH, 4 * MH * MH});
        double gx = 0.3, gz = -0.7;
        double[] d = m.descent(gx, gz);
        // G d = −∇H
        assertEquals(-gx, m.gxx * d[0] + m.gxz * d[1], 1e-12);
        assertEquals(-gz, m.gxz * d[0] + m.gzz * d[1], 1e-12);
        assertEquals(3 * MH, m.length(1, 0), 1e-9);
        assertEquals(2 * MH, m.length(0, 1), 1e-9);
        assertEquals(Math.sqrt(32) * MH * MH, m.area, 1e-6);
        double lMin = MH * MH * (6.5 - Math.sqrt(6.25 + 4));
        assertEquals(Math.sqrt(lMin), m.mMin, 1e-9);
        for (double az = 0; az < 2 * Math.PI; az += 0.37) {
            double[] b = m.perMetre(az);
            assertEquals(1.0, m.length(b[0], b[1]), 1e-12, "one metre of ground at " + az);
        }
        double[] e = m.perMetre(0.0), n = m.perMetre(Math.PI / 2.0);
        assertEquals(0.0, e[0] * (m.gxx * n[0] + m.gxz * n[1]) + e[1] * (m.gxz * n[0] + m.gzz * n[1]), 1e-12, "orthogonal on the ground");
    }

    @Test
    void anAnisotropicMetricShortensFlowsAlongItsLongBlocks() {
        Cones env = new Cones(List.of(shield(1, 0, 0, 1.0)));
        env.precip = 200.0;
        env.metric = new double[]{4 * MH * MH, 0.0, MH * MH};   // an x block is 600 m of ground, a z block 300 m
        LavaFlows f = flows(env);
        LavaFlows.Metric m = new LavaFlows.Metric(env.metric);
        for (LavaFlows.Flow fl : f.flows(0)) {
            double sum = 0.0;
            List<double[]> p = fl.path();
            for (int k = 1; k < p.size(); k++)
                sum += m.length(p.get(k)[0] - p.get(k - 1)[0], p.get(k)[1] - p.get(k - 1)[1]);
            assertEquals(fl.travelledM(), sum, 1e-6 * Math.max(1.0, sum), "metres travelled are the ground's");
        }
        int maxX = 0, maxZ = 0;
        for (int z = -260; z <= 260; z++)
            for (int x = -260; x <= 260; x++)
                if (f.code(x, z) >= 0) {
                    maxX = Math.max(maxX, Math.abs(x));
                    maxZ = Math.max(maxZ, Math.abs(z));
                }
        assertTrue(maxZ > 1.3 * maxX, "reach in z " + maxZ + " against x " + maxX);
    }

    @Test
    void sanitizeKeepsTheMetricNearTheNominalBlock() {
        double h2 = MH * MH;
        assertArrayEquals(ISO, LavaFlows.sanitize(ISO, MH), 0.0);
        assertArrayEquals(ISO, LavaFlows.sanitize(null, MH), 0.0);
        assertArrayEquals(ISO, LavaFlows.sanitize(new double[]{Double.NaN, 0, h2}, MH), 0.0);
        assertArrayEquals(ISO, LavaFlows.sanitize(new double[]{h2, 2 * h2, h2}, MH), 0.0, "not positive definite");
        assertArrayEquals(ISO, LavaFlows.sanitize(new double[]{h2, 0, 0}, MH), 0.0, "degenerate");
        double[] ok = {3 * h2, 0.5 * h2, 2 * h2};
        assertArrayEquals(ok, LavaFlows.sanitize(ok, MH), 0.0);
        double[] wide = LavaFlows.sanitize(new double[]{100 * h2, 0, 0.001 * h2}, MH);
        assertArrayEquals(new double[]{16 * h2, 0, h2 / 16}, wide, 1e-9 * h2);
        // A rotated stretch keeps its axes: R diag(100, 1) Rᵀ becomes R diag(16, 1) Rᵀ.
        double a = 0.6, cs = Math.cos(a), sn = Math.sin(a);
        double[] g = {(100 * cs * cs + sn * sn) * h2, 99 * cs * sn * h2, (100 * sn * sn + cs * cs) * h2};
        double[] want = {(16 * cs * cs + sn * sn) * h2, 15 * cs * sn * h2, (16 * sn * sn + cs * cs) * h2};
        assertArrayEquals(want, LavaFlows.sanitize(g, MH), 1e-9 * h2);
    }

    // ---- the climate's clock ----

    @Test
    void timesAgainstRainInterpolateLogLogBetweenTheAnchors() {
        double[] p = LavaFlows.ANCHOR_P_MM;
        for (int i = 0; i < p.length; i++) {
            assertEquals(LavaFlows.FRESH_YR[i], LavaFlows.interpolate(LavaFlows.FRESH_YR, p[i]), 1e-9 * LavaFlows.FRESH_YR[i]);
            assertEquals(LavaFlows.VISIBLE_YR[i], LavaFlows.interpolate(LavaFlows.VISIBLE_YR, p[i]), 1e-9 * LavaFlows.VISIBLE_YR[i]);
        }
        assertEquals(LavaFlows.FRESH_YR[0], LavaFlows.interpolate(LavaFlows.FRESH_YR, 10.0));
        assertEquals(LavaFlows.FRESH_YR[0], LavaFlows.interpolate(LavaFlows.FRESH_YR, Double.NaN));
        assertEquals(LavaFlows.FRESH_YR[2], LavaFlows.interpolate(LavaFlows.FRESH_YR, 20_000.0));
        double mid = Math.sqrt(p[0] * p[1]);
        assertEquals(Math.sqrt(LavaFlows.VISIBLE_YR[0] * LavaFlows.VISIBLE_YR[1]), LavaFlows.interpolate(LavaFlows.VISIBLE_YR, mid), 1e-6);
        double prev = Double.POSITIVE_INFINITY;
        for (double mm = 100; mm < 5_000; mm *= 1.1) {
            double fresh = LavaFlows.interpolate(LavaFlows.FRESH_YR, mm), vis = LavaFlows.interpolate(LavaFlows.VISIBLE_YR, mm);
            assertTrue(fresh <= prev, "wetter is never slower to weather");
            assertTrue(fresh < vis);
            prev = fresh;
        }
    }

    @Test
    void aWetClimateShowsFewerFlowsThanADryOne() {
        Cones dry = new Cones(List.of(shield(1, 0, 0, 1.0)));
        dry.precip = 200.0;
        Cones wet = new Cones(List.of(shield(1, 0, 0, 1.0)));
        wet.precip = 3_000.0;
        LavaFlows.Summary d = flows(dry).summary(0), w = flows(wet).summary(0);
        assertTrue(w.windowYr() < d.windowYr());
        assertTrue(w.visible() < d.visible(), "wet " + w.describe() + " / dry " + d.describe());
        assertTrue(d.cover() <= LavaFlows.COVER_CAP + 0.05, "the cap holds: " + d.describe());
        // in the wet, most of what flowed is grey, brown or covered; in the dry most is still black
        int wetFresh = w.cellsByCode()[LavaFlows.Paint.AA.code()] + w.cellsByCode()[LavaFlows.Paint.PAHOEHOE.code()];
        int dryFresh = d.cellsByCode()[LavaFlows.Paint.AA.code()] + d.cellsByCode()[LavaFlows.Paint.PAHOEHOE.code()];
        assertTrue(wetFresh < w.visible() / 2, w.describe());
        assertTrue(dryFresh > d.visible() / 2, d.describe());
    }

    @Test
    void aQuieterVolcanoShowsFewerFlows() {
        Cones loud = new Cones(List.of(shield(1, 0, 0, 1.0)));
        loud.precip = 3_000.0;
        Cones quiet = new Cones(List.of(shield(1, 0, 0, 0.05)));
        quiet.precip = 3_000.0;
        LavaFlows.Summary l = flows(loud).summary(0), q = flows(quiet).summary(0);
        assertTrue(q.flows() < l.flows(), q.describe() + " / " + l.describe());
        assertTrue(q.visible() < l.visible());
        Cones dormant = new Cones(List.of(shield(1, 0, 0, 0.0)));
        LavaFlows f = flows(dormant);
        assertEquals(0, f.summary(0).flows());
        for (int z = -200; z <= 200; z += 7)
            for (int x = -200; x <= 200; x += 3)
                assertEquals(-1, f.code(x, z));
    }

    // ---- the paint ----

    @Test
    void viscousLavaIsBlockyAndALongBasalticFlowIsAa() {
        Cones s = new Cones(List.of(strato(1, 0, 0, 1.0)));
        s.precip = 150.0;
        LavaFlows.Summary vs = flows(s).summary(0);
        assertEquals(0, vs.cellsByCode()[LavaFlows.Paint.AA.code()], vs.describe());
        assertEquals(0, vs.cellsByCode()[LavaFlows.Paint.PAHOEHOE.code()], vs.describe());
        assertTrue(vs.cellsByCode()[LavaFlows.Paint.BLOCKY.code()] > 0, vs.describe());

        Cones b = new Cones(List.of(shield(1, 0, 0, 1.0)));
        b.precip = 150.0;
        LavaFlows f = flows(b);
        LavaFlows.Summary bs = f.summary(0);
        assertEquals(0, bs.cellsByCode()[LavaFlows.Paint.BLOCKY.code()], bs.describe());
        assertTrue(bs.cellsByCode()[LavaFlows.Paint.AA.code()] > 0, bs.describe());
        assertTrue(bs.cellsByCode()[LavaFlows.Paint.PAHOEHOE.code()] > 0, bs.describe());
        // The youngest flow lies over every other: its vent column shows its own fresh paint.
        LavaFlows.Flow young = f.flows(0).get(0);
        LavaFlows.Paint want = young.aaFromVent() ? LavaFlows.Paint.AA : LavaFlows.Paint.PAHOEHOE;
        if (!young.aaFromVent()) {
            // a pahoehoe flow turns to aa on steep ground: on this shield (2.6 degrees) it never does
            assertTrue(Math.toDegrees(Math.atan(2_000.0 / (45_000.0 - 3_600.0))) < LavaFlows.PAHOEHOE_TO_AA_DEG);
        }
        assertEquals(want, f.at((int) Math.floor(young.startX()), (int) Math.floor(young.startZ())));
    }

    @Test
    void paintCodesRoundTrip() {
        for (LavaFlows.Paint p : LavaFlows.Paint.values())
            assertEquals(p, LavaFlows.Paint.of(p.code()));
        assertNull(LavaFlows.Paint.of(0));
        assertNull(LavaFlows.Paint.of(-1));
        assertNull(LavaFlows.Paint.of(LavaFlows.Paint.values().length + 1));
    }

    // ---- water, the seam, determinism ----

    @Test
    void waterStopsAFlow() {
        Cones env = new Cones(List.of(shield(1, 0, 0, 1.0)));
        env.precip = 200.0;
        env.wetFromX = 40.0;
        LavaFlows f = flows(env);
        assertTrue(f.flows(0).stream().anyMatch(fl -> fl.stop().equals("water")), f.summary(0).describe());
        for (int z = -260; z <= 260; z++)
            for (int x = 40; x <= 260; x++)
                assertEquals(-1, f.code(x, z), "painted in the water at " + x + ", " + z);
        int dryCells = 0;
        for (int z = -260; z <= 260; z++)
            for (int x = -260; x < 40; x++)
                if (f.code(x, z) > 0)
                    dryCells++;
        assertTrue(dryCells > 1_000, "the dry side still shows its flows: " + dryCells);
    }

    @Test
    void aVolcanoOnTheSeamPaintsBothSidesAlike() {
        double vx = C / 2.0 - 5.0;
        Cones env = new Cones(List.of(shield(1, vx, 30, 1.0)));
        env.precip = 300.0;
        LavaFlows f = flows(env);
        int east = 0, west = 0;
        for (int z = 30 - 260; z <= 30 + 260; z++)
            for (int x = (int) vx - 260; x <= (int) vx + 260; x++) {
                int c = f.code(x, z);
                assertEquals(c, f.code(x - C, z), "a lap west at " + x);
                assertEquals(c, f.code(x + C, z), "a lap east at " + x);
                if (c > 0) {
                    if (x >= C / 2)
                        east++;
                    else
                        west++;
                }
            }
        assertTrue(east > 100 && west > 100, "flows on both sides of the seam: " + east + " / " + west);
        assertTrue(f.touches(C / 2 + 50, 20, C / 2 + 65, 35));
        assertTrue(f.touches(-C / 2 + 50, 20, -C / 2 + 65, 35));
    }

    @Test
    void theAnswerIsTheSameWhateverAsksAndInWhateverOrder() {
        List<Volcanoes.Volcano> vs = List.of(shield(1, 0, 0, 1.0), strato(2, 150, 40, 0.8));
        int x0 = -280, z0 = -280, x1 = 320, z1 = 300, w = x1 - x0 + 1, n = w * (z1 - z0 + 1);
        int[] rows = box(flows(cones(vs, 300.0)), x0, z0, x1, z1);

        List<Integer> order = new ArrayList<>();
        for (int i = 0; i < n; i++)
            order.add(i);
        Collections.shuffle(order, new Random(7));
        LavaFlows shuffled = flows(cones(vs, 300.0));
        int[] s = new int[n];
        for (int i : order)
            s[i] = shuffled.code(x0 + i % w, z0 + i / w);
        assertArrayEquals(rows, s);

        LavaFlows parallel = flows(cones(vs, 300.0));
        int[] p = new int[n];
        IntStream.range(0, n).parallel().forEach(i -> p[i] = parallel.code(x0 + i % w, z0 + i / w));
        assertArrayEquals(rows, p);

        // Where both volcanoes' flows lie, the answer is one of theirs.
        int[] a = box(flows(cones(vs, 300.0), List.of(vs.get(0))), x0, z0, x1, z1);
        int[] b = box(flows(cones(vs, 300.0), List.of(vs.get(1))), x0, z0, x1, z1);
        int both = 0;
        for (int i = 0; i < n; i++) {
            if (a[i] < 0 && b[i] < 0)
                assertEquals(-1, rows[i]);
            else if (a[i] < 0)
                assertEquals(b[i], rows[i]);
            else if (b[i] < 0)
                assertEquals(a[i], rows[i]);
            else {
                assertTrue(rows[i] == a[i] || rows[i] == b[i]);
                both++;
            }
        }
        assertTrue(both > 0, "the two volcanoes' flows meet somewhere");
    }

    @Test
    void touchesCoversEveryPaintedChunk() {
        Cones env = new Cones(List.of(shield(1, 0, 0, 1.0), strato(2, 600, -300, 1.0)));
        LavaFlows f = flows(env);
        for (int cz = -40; cz < 40; cz++)
            for (int cx = -40; cx < 60; cx++) {
                boolean any = false;
                for (int z = cz * 16; z < cz * 16 + 16 && !any; z++)
                    for (int x = cx * 16; x < cx * 16 + 16 && !any; x++)
                        any = f.code(x, z) >= 0;
                if (any)
                    assertTrue(f.touches(cx * 16, cz * 16, cx * 16 + 15, cz * 16 + 15), "chunk " + cx + ", " + cz);
            }
        assertFalse(f.touches(-2_000, 1_000, -1_985, 1_015));
    }

    @Test
    void theWanderIsSmoothAndBounded() {
        long seed = 0x1234_5678_9ABCL;
        for (double z = -20; z < 20; z += 0.37)
            for (double x = -20; x < 20; x += 0.29) {
                double w0 = LavaFlows.wander(seed, x, z);
                assertTrue(Math.abs(w0) <= 1.0);
                assertTrue(Math.abs(LavaFlows.wander(seed, x + 0.01, z) - w0) < 0.03);
                assertTrue(Math.abs(LavaFlows.wander(seed, x, z + 0.01) - w0) < 0.03);
                assertEquals(w0, LavaFlows.wander(seed, x, z));
            }
    }
}
