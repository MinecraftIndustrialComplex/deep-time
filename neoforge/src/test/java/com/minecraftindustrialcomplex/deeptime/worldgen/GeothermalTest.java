// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The geothermal sites of active volcanoes ({@link Geothermal}; P2-14): what is plain Java.
 * Volcanoes and terrain are built here in closed form, so nothing depends on a planet; the tests
 * assert properties (a pool is sealed, nothing stands in water, the setting decides the kind), not
 * the numbers of one planet. The placed blocks are the GameTest {@code geothermalAroundActiveVolcanoes}'s.
 */
class GeothermalTest {

    private static final int C = 8192, SEA = 63, LAVA_FLOOR = -342;
    private static final double MH = 300.0, MV = 20.0;

    private static final Geothermal.Env WET_WARM = env(1200, 15, false, false);
    private static final Geothermal.Env DRY_WARM = env(120, 15, false, false);
    private static final Geothermal.Env WET_COLD = env(1200, -6, false, false);
    private static final Magma.Wet DRY = (x, z) -> false;

    private static Geothermal.Env env(double precip, double mean, boolean iced, boolean carbonate) {
        return new Geothermal.Env() {
            @Override
            public double precipMm(int x, int z) {
                return precip;
            }

            @Override
            public double meanC(int x, int z) {
                return mean;
            }

            @Override
            public boolean iced(int x, int z) {
                return iced;
            }

            @Override
            public boolean carbonate(int x, int z, int groundY) {
                return carbonate;
            }
        };
    }

    /** Active volcanoes 1,000 blocks apart along z = 0 (so their fields never meet), 18 km (60 blocks) wide. */
    private static List<Volcanoes.Volcano> volcanoes(int n, boolean silicic) {
        List<Volcanoes.Volcano> out = new ArrayList<>();
        for (int i = 0; i < n; i++)
            out.add(new Volcanoes.Volcano(i + 1, "stratovolcano", -C / 2.0 + 500 + 1000.0 * i, 20.0 * (i % 5), 18_000, 2_000,
                    0.5 + 0.5 * (i % 3) / 2.0, false, silicic, 600, 60, false));
        return out;
    }

    /** A broad, gentle cone (slope 0.12) over every volcano on a plain at y 75: ground a pool can sit level on. */
    private static Magma.Heights terrain(List<Volcanoes.Volcano> vs) {
        return terrain(vs, 0.12);
    }

    private static Magma.Heights terrain(List<Volcanoes.Volcano> vs, double slope) {
        return (x, z) -> {
            double best = 75.0;
            for (Volcanoes.Volcano v : vs) {
                int dx = Geothermal.fold(x - (int) Math.round(v.x()), C);
                double r = Math.hypot(dx, z - v.z());
                best = Math.max(best, 75.0 + slope * (120.0 - r));
            }
            return best;
        };
    }

    private static Geothermal build(List<Volcanoes.Volcano> vs, GeothermalRules rules, Geothermal.Env env, Magma.Wet wet) {
        Magma.Heights h = terrain(vs);
        Magma magma = Magma.build(vs, MagmaRules.DEFAULT, h, wet, C, SEA, LAVA_FLOOR, MH, MV, C / 4);
        return Geothermal.build(vs, rules, magma, h, wet, env, C, SEA, MH);
    }

    private static Geothermal build(List<Volcanoes.Volcano> vs, Geothermal.Env env) {
        return build(vs, GeothermalRules.DEFAULT, env, DRY);
    }

    private static long count(Geothermal g, Geothermal.Kind k) {
        return g.sites().stream().filter(s -> s.kind() == k).count();
    }

    @Test
    void aDensityOfZeroMeansNoFeaturesAndIsTheOnlySwitch() {
        var vs = volcanoes(20, false);
        Geothermal none = build(vs, new GeothermalRules(0.0, 1.0, 1.0, 400.0, 100.0, 1.0), WET_WARM, DRY);
        assertTrue(none.sites().isEmpty());
        assertEquals(0.0, none.warmthC(vs.get(0).x() > 0 ? 1 : -1, 80, 0));
        assertTrue(none.near(-C / 2, -4000, C / 2, 4000).isEmpty());
        assertFalse(build(vs, WET_WARM).sites().isEmpty(), "the default density has features");
        assertTrue(build(List.of(), WET_WARM).sites().isEmpty(), "a planet without an active volcano has none");
    }

    @Test
    void theSettingDecidesTheKind() {
        var vs = volcanoes(30, false);
        Geothermal wet = build(vs, WET_WARM), dry = build(vs, DRY_WARM), cold = build(vs, WET_COLD);
        assertTrue(count(wet, Geothermal.Kind.POOL) > 0, "groundwater and a thaw: hot springs");
        assertTrue(count(wet, Geothermal.Kind.VENT) > 0, "and fumaroles near the summits");
        assertEquals(0, count(dry, Geothermal.Kind.POOL), "dry ground: no standing water");
        assertTrue(count(dry, Geothermal.Kind.VENT) > 0 && count(dry, Geothermal.Kind.PATCH) > 0, "fumaroles and bare altered ground");
        assertEquals(0, count(cold, Geothermal.Kind.POOL), "frozen ground: no pools");
        assertTrue(count(cold, Geothermal.Kind.VENT) > 0);
        // The threshold is a setting: with none, the dry world gets springs too.
        Geothermal lowBar = build(vs, new GeothermalRules(1.0, 1.0, 1.0, 0.0, 100.0, 1.0), DRY_WARM, DRY);
        assertTrue(count(lowBar, Geothermal.Kind.POOL) > 0);
        // A world without a climate is taken as warm and wet.
        assertTrue(count(build(vs, Geothermal.Env.OPEN), Geothermal.Kind.POOL) > 0);
    }

    @Test
    void theTerraceIsSinterUnlessCarbonateLiesUnderneath() {
        var vs = volcanoes(30, false);
        Geothermal sinter = build(vs, env(1200, 15, false, false)), travertine = build(vs, env(1200, 15, false, true));
        assertTrue(sinter.sites().stream().filter(s -> s.kind() == Geothermal.Kind.POOL).allMatch(Geothermal.Site::sinter));
        assertTrue(travertine.sites().stream().filter(s -> s.kind() == Geothermal.Kind.POOL).noneMatch(Geothermal.Site::sinter));
    }

    /** Water is held on every side: a pool column's neighbours are pool or rim, the level is the lowest ground. */
    @Test
    void everyPoolIsSealedAndSitsLevel() {
        var vs = volcanoes(40, false);
        Magma.Heights h = terrain(vs);
        Geothermal g = build(vs, WET_WARM);
        int pools = 0;
        for (Geothermal.Site s : g.sites()) {
            if (s.kind() != Geothermal.Kind.POOL)
                continue;
            pools++;
            int lowest = Integer.MAX_VALUE, highest = Integer.MIN_VALUE, water = 0;
            for (int dx = -s.reach(); dx <= s.reach(); dx++)
                for (int dz = -s.reach(); dz <= s.reach(); dz++) {
                    Geothermal.Part p = s.part(dx, dz);
                    if (p.pool()) {
                        water++;
                        for (int[] d : new int[][]{{1, 0}, {-1, 0}, {0, 1}, {0, -1}}) {
                            Geothermal.Part q = s.part(dx + d[0], dz + d[1]);
                            assertTrue(q.pool() || q == Geothermal.Part.RIM, "pool column (" + dx + "," + dz + ") of site " + s.id()
                                    + " borders " + q);
                        }
                    }
                    if (p.pool() || p == Geothermal.Part.RIM) {
                        int top = Magma.top(h.at(s.x() + dx, s.z() + dz));
                        lowest = Math.min(lowest, top);
                        highest = Math.max(highest, top);
                    }
                }
            assertTrue(water >= 3, "a pool holds water");
            assertTrue(s.level() >= lowest && s.level() <= highest, "the water stands within the footprint's ground");
            assertTrue(highest - s.level() <= Geothermal.CUT, "no more than " + Geothermal.CUT + " blocks cut: " + (highest - s.level()));
            assertTrue(s.level() - lowest <= Geothermal.DAM, "no more than " + Geothermal.DAM + " blocks of dam: " + (s.level() - lowest));
            assertTrue(s.level() >= SEA + Geothermal.MIN_ABOVE_SEA, "above the coast");
        }
        assertTrue(pools >= 10, "pools: " + pools);
    }

    /** On a 35-degree-steep world (a block of rise per block across) a pool is dammed downhill and cut into the slope uphill. */
    @Test
    void poolsSitOnSlopesByCuttingAndDamming() {
        var vs = volcanoes(60, false);
        Magma.Heights h = terrain(vs, 1.0);
        Geothermal g = Geothermal.build(vs, new GeothermalRules(2.0, 1.0, 1.0, 400.0, 100.0, 1.0), Magma.NONE, h, DRY, WET_WARM, C, SEA, MH);
        int pools = 0;
        for (Geothermal.Site s : g.sites()) {
            if (s.kind() != Geothermal.Kind.POOL)
                continue;
            pools++;
            for (int dx = -s.reach(); dx <= s.reach(); dx++)
                for (int dz = -s.reach(); dz <= s.reach(); dz++) {
                    Geothermal.Part p = s.part(dx, dz);
                    if (p.pool() || p == Geothermal.Part.RIM) {
                        int top = Magma.top(h.at(s.x() + dx, s.z() + dz));
                        assertTrue(top - s.level() <= Geothermal.CUT && s.level() - top <= Geothermal.DAM, "site " + s.id() + " " + p + " " + (top - s.level()));
                    }
                }
        }
        assertTrue(pools > 0, "pools on the slopes");
    }

    @Test
    void nothingStandsInWaterUnderIceOrAtTheCoast() {
        var vs = volcanoes(60, false);
        // Water north of z = 12 (rivers, lakes, the sea) and ice east of x = 0.
        Magma.Wet wet = (x, z) -> z > 12;
        Geothermal.Env iced = new Geothermal.Env() {
            @Override
            public double precipMm(int x, int z) {
                return 1200;
            }

            @Override
            public double meanC(int x, int z) {
                return 15;
            }

            @Override
            public boolean iced(int x, int z) {
                return x > 0;
            }

            @Override
            public boolean carbonate(int x, int z, int groundY) {
                return false;
            }
        };
        Geothermal g = build(vs, GeothermalRules.DEFAULT, iced, wet);
        assertFalse(g.sites().isEmpty());
        for (Geothermal.Site s : g.sites())
            for (int dx = -s.reach(); dx <= s.reach(); dx++)
                for (int dz = -s.reach(); dz <= s.reach(); dz++) {
                    Geothermal.Part p = s.part(dx, dz);
                    boolean exact = p.pool() || p == Geothermal.Part.RIM || p == Geothermal.Part.VENT || p == Geothermal.Part.SULFUR;
                    if (!exact)
                        continue;
                    int x = Math.floorMod(s.x() + dx + C / 2, C) - C / 2, z = s.z() + dz;
                    assertFalse(wet.at(x, z), "site " + s.id() + " " + p + " in water at " + x + "," + z);
                    assertFalse(iced.iced(x, z), "site " + s.id() + " " + p + " under ice at " + x + "," + z);
                }
        // The sea: ground at or below the coast takes nothing.
        Magma.Heights low = (x, z) -> SEA + 1.5;
        Geothermal coast = Geothermal.build(vs, GeothermalRules.DEFAULT, Magma.build(vs, MagmaRules.DEFAULT, low, DRY, C, SEA, LAVA_FLOOR, MH, MV, C / 4),
                low, DRY, WET_WARM, C, SEA, MH);
        assertTrue(coast.sites().isEmpty());
    }

    @Test
    void sitesKeepClearOfOneAnotherAndOfOpenMagma() {
        var vs = volcanoes(40, false);
        Magma.Heights h = terrain(vs);
        Magma magma = Magma.build(vs, new MagmaRules(3.0, 1.0, 1.0, true), h, DRY, C, SEA, LAVA_FLOOR, MH, MV, C / 4);
        Geothermal g = Geothermal.build(vs, new GeothermalRules(3.0, 1.0, 1.0, 400.0, 100.0, 1.0), magma, h, DRY, WET_WARM, C, SEA, MH);
        assertTrue(magma.bodies().stream().anyMatch(Magma.Body::lake), "the test has crater lakes to keep clear of");
        List<Geothermal.Site> sites = g.sites();
        for (int i = 0; i < sites.size(); i++) {
            Geothermal.Site a = sites.get(i);
            for (int j = i + 1; j < sites.size(); j++) {
                Geothermal.Site b = sites.get(j);
                assertTrue(Math.hypot(Geothermal.fold(a.x() - b.x(), C), a.z() - b.z()) >= a.hard() + b.hard() + 1,
                        "sites " + a.id() + " and " + b.id() + " overlap");
            }
            for (Magma.Body m : magma.bodies()) {
                if (m.chamberRh() <= 0)
                    continue;
                double d = Math.hypot(Geothermal.fold(a.x() - m.x(), C), a.z() - m.z());
                assertTrue(d >= (m.lake() ? m.lakeRadius() + 2 : 2) + 3 + a.hard(), "site " + a.id() + " too near the magma of volcano " + m.id());
                // No hard part of a site reaches a lava lake's open air or the plug.
                for (int dx = -a.reach(); dx <= a.reach(); dx++)
                    for (int dz = -a.reach(); dz <= a.reach(); dz++) {
                        Geothermal.Part p = a.part(dx, dz);
                        if (p.pool() || p == Geothermal.Part.RIM || p == Geothermal.Part.VENT || p == Geothermal.Part.SULFUR) {
                            int x = Math.floorMod(a.x() + dx + C / 2, C) - C / 2;
                            for (int y = m.floorY() - 6; y <= m.floorY() + 4; y++)
                                assertTrue(magma.at(x, y, a.z() + dz) == Magma.Part.NONE, "site " + a.id() + " " + p + " on magma");
                        }
                    }
            }
        }
    }

    @Test
    void densityScalesTheNumberOfFeatures() {
        var vs = volcanoes(60, false);
        int half = build(vs, new GeothermalRules(0.5, 1.0, 1.0, 400.0, 100.0, 1.0), WET_WARM, DRY).sites().size();
        int one = build(vs, WET_WARM).sites().size();
        int twice = build(vs, new GeothermalRules(2.0, 1.0, 1.0, 400.0, 100.0, 1.0), WET_WARM, DRY).sites().size();
        assertTrue(half < one && one < twice, half + " < " + one + " < " + twice);
        // Every volcano can have some, the more active ones more.
        assertTrue(build(vs, new GeothermalRules(4.0, 1.0, 1.0, 400.0, 100.0, 1.0), WET_WARM, DRY).sites().stream().map(Geothermal.Site::volcano)
                .distinct().count() >= 55);
    }

    @Test
    void reachAndSizeAreSettings() {
        var vs = volcanoes(40, false);
        double near = meanDistance(build(vs, new GeothermalRules(1.0, 0.25, 1.0, 400.0, 100.0, 1.0), WET_WARM, DRY), vs);
        double far = meanDistance(build(vs, new GeothermalRules(1.0, 2.0, 1.0, 400.0, 100.0, 1.0), WET_WARM, DRY), vs);
        assertTrue(near < far, "reach 0.25: " + near + " blocks from the summit on average, reach 2: " + far);
        double small = meanApron(build(vs, new GeothermalRules(1.0, 1.0, 0.5, 400.0, 100.0, 1.0), WET_WARM, DRY));
        double large = meanApron(build(vs, new GeothermalRules(1.0, 1.0, 2.0, 400.0, 100.0, 1.0), WET_WARM, DRY));
        assertTrue(small < large, small + " < " + large);
    }

    private static double meanDistance(Geothermal g, List<Volcanoes.Volcano> vs) {
        double sum = 0;
        for (Geothermal.Site s : g.sites()) {
            Volcanoes.Volcano v = vs.get(s.volcano() - 1);
            sum += Math.hypot(Geothermal.fold(s.x() - (int) Math.round(v.x()), C), s.z() - v.z());
        }
        return sum / Math.max(1, g.sites().size());
    }

    private static double meanApron(Geothermal g) {
        return g.sites().stream().mapToInt(Geothermal.Site::apron).average().orElse(0);
    }

    @Test
    void siteListsAreDeterministicAndEachVolcanoIsItsOwn() {
        var vs = volcanoes(25, true);
        Geothermal a = build(vs, WET_WARM), b = build(vs, WET_WARM);
        assertEquals(a.sites(), b.sites());
        // The same volcanoes in another order (they do not touch): the same features.
        List<Volcanoes.Volcano> shuffled = new ArrayList<>(vs);
        Collections.reverse(shuffled);
        Geothermal c = Geothermal.build(shuffled, GeothermalRules.DEFAULT, Magma.NONE, terrain(vs), DRY, WET_WARM, C, SEA, MH);
        Geothermal d = Geothermal.build(vs, GeothermalRules.DEFAULT, Magma.NONE, terrain(vs), DRY, WET_WARM, C, SEA, MH);
        assertEquals(key(d), key(c));
        // Blocks are a function of the column: asking twice gives the same part.
        for (Geothermal.Site s : a.sites())
            for (int dx = -s.reach(); dx <= s.reach(); dx += 3)
                for (int dz = -s.reach(); dz <= s.reach(); dz += 3)
                    assertEquals(s.part(dx, dz), s.part(dx, dz));
    }

    private static Set<String> key(Geothermal g) {
        Set<String> out = new HashSet<>();
        for (Geothermal.Site s : g.sites())
            out.add(s.volcano() + ":" + s.kind() + "@" + s.x() + "," + s.z() + " r" + s.r() + " a" + s.apron() + " L" + s.level());
        return out;
    }

    @Test
    void silicicVolcanoesCarryCinnabarAndOthersDoNot() {
        long with = cinnabarColumns(build(volcanoes(40, true), WET_WARM));
        long without = cinnabarColumns(build(volcanoes(40, false), WET_WARM));
        assertTrue(with > 0, "cinnabar stains in the altered ground of silicic fields");
        assertEquals(0, without);
    }

    private static long cinnabarColumns(Geothermal g) {
        long n = 0;
        for (Geothermal.Site s : g.sites())
            for (int dx = -s.reach(); dx <= s.reach(); dx++)
                for (int dz = -s.reach(); dz <= s.reach(); dz++)
                    if (s.part(dx, dz) == Geothermal.Part.CINNABAR)
                        n++;
        return n;
    }

    @Test
    void ventsHaveACoreACrustAndAlteredGroundAndNoWater() {
        Geothermal g = build(volcanoes(40, false), DRY_WARM);
        for (Geothermal.Site s : g.sites()) {
            if (s.kind() != Geothermal.Kind.VENT)
                continue;
            assertEquals(Geothermal.Part.VENT, s.part(0, 0), "the vent's heart is hot ground");
            boolean crust = false, altered = false;
            for (int dx = -s.reach(); dx <= s.reach(); dx++)
                for (int dz = -s.reach(); dz <= s.reach(); dz++) {
                    Geothermal.Part p = s.part(dx, dz);
                    crust |= p == Geothermal.Part.SULFUR;
                    altered |= p == Geothermal.Part.ALTERED || p == Geothermal.Part.STAIN;
                    assertFalse(p.pool() || p == Geothermal.Part.RIM, "a fumarole holds no water");
                }
            assertTrue(crust && altered);
        }
    }

    @Test
    void theWarmGroundSignalIsWarmInsideAndGoneFarOffAndAcrossTheSeamToo() {
        var vs = new ArrayList<>(volcanoes(5, false));
        // A volcano on the seam.
        vs.add(new Volcanoes.Volcano(99, "stratovolcano", -C / 2.0 + 3, 40, 18_000, 2_000, 1.0, false, false, 600, 60, false));
        Geothermal g = build(vs, DRY_WARM);
        assertFalse(g.sites().isEmpty());
        for (Geothermal.Site s : g.sites()) {
            double heart = g.warmthC(s.x(), s.groundY(), s.z());
            assertTrue(heart > 0.0 && heart <= Geothermal.WARM_VENT, "heart of site " + s.id() + ": " + heart);
            assertEquals(0.0, g.warmthC(s.x(), s.groundY() + 40, s.z()), "far above the ground");
            assertEquals(0.0, g.warmthC(s.x(), s.groundY() - 60, s.z()), "far below it");
            // Warmth falls with distance.
            double prev = s.warmth(0, 0, s.groundY());
            for (int d = 1; d <= s.reach() + Geothermal.WARM_FADE + 4; d++) {
                double w = s.warmth(d, 0, s.groundY());
                assertTrue(w <= prev + 1e-9, "warmth does not rise with distance");
                prev = w;
            }
            assertEquals(0.0, prev, "and is gone beyond the fade");
        }
        assertEquals(0.0, g.warmthC(0, 200, 5000));
        // The seam: sites near it are found from either side, and the folded offsets agree.
        int seamSites = 0;
        for (Geothermal.Site s : g.sites()) {
            if (s.volcano() != 99)
                continue;
            seamSites++;
            int xa = s.x(), xb = s.x() < 0 ? s.x() + C : s.x() - C;
            assertTrue(g.near(xa - 2, s.z() - 2, xa + 2, s.z() + 2).contains(s));
            assertTrue(g.near(xb - 2, s.z() - 2, xb + 2, s.z() + 2).contains(s), "found from the other side of the seam");
            assertEquals(g.partAt(xa, s.z()), g.partAt(xb, s.z()), "one column, two spellings across the seam");
            assertEquals(g.warmthC(xa, s.groundY(), s.z()), g.warmthC(xb, s.groundY(), s.z()));
        }
        assertTrue(seamSites > 0, "the volcano on the seam has features");
    }

    /** An environment whose continental heat flow is {@code hot} mW/m2 inside the box x in [-2000, 2000), z in [-1000, 1000) and 62 elsewhere. */
    private static Geothermal.Env heatEnv(double hot, double precip, double mean, java.util.function.IntBinaryOperator icedAt) {
        return new Geothermal.Env() {
            @Override
            public double precipMm(int x, int z) {
                return precip;
            }

            @Override
            public double meanC(int x, int z) {
                return mean;
            }

            @Override
            public double heatFlowMwM2(int x, int z) {
                return x >= -2000 && x < 2000 && z >= -1000 && z < 1000 ? hot : 62.0;
            }

            @Override
            public boolean iced(int x, int z) {
                return icedAt.applyAsInt(x, z) != 0;
            }

            @Override
            public boolean carbonate(int x, int z, int groundY) {
                return false;
            }
        };
    }

    private static Geothermal heatBuild(GeothermalRules rules, Geothermal.Env env, Magma.Wet wet) {
        Magma.Heights flat = (x, z) -> 75.0;
        return Geothermal.build(List.of(), rules, Magma.NONE, flat, wet, env, C, SEA, MH);
    }

    @Test
    void hotContinentalGroundTakesFieldsWithoutAnyVolcano() {
        // Ben, decisions log 2026-10-02: fields also where the baked heat flow is high.
        Geothermal g = heatBuild(GeothermalRules.DEFAULT, heatEnv(120, 1200, 15, (x, z) -> 0), DRY);
        assertTrue(g.sites().size() > 40, "a box of 720,000 km2 at one field per 10,000: " + g.sites().size() + " sites");
        assertEquals(g.sites().size(), g.heatFlowSites(), "no volcano: every site is the ground's heat");
        assertTrue(count(g, Geothermal.Kind.POOL) > 0, "wet and warm: hot springs");
        for (Geothermal.Site s : g.sites()) {
            assertEquals(Geothermal.HEAT_FLOW_SOURCE, s.volcano());
            // A field is drawn on hot ground; its sites spread a few tens of blocks about it.
            assertTrue(s.x() > -2060 && s.x() < 2060 && s.z() > -1060 && s.z() < 1060, "site at " + s.x() + "," + s.z() + " off the hot ground");
        }
        // Cooler ground than the setting: nothing.
        assertTrue(heatBuild(GeothermalRules.DEFAULT, heatEnv(99, 1200, 15, (x, z) -> 0), DRY).sites().isEmpty(), "99 is under the 100 threshold");
        // The threshold is a setting: lowered under the ground's 62 the whole world is hot.
        Geothermal everywhere = heatBuild(new GeothermalRules(1.0, 1.0, 1.0, 400.0, 60.0, 1.0), heatEnv(99, 1200, 15, (x, z) -> 0), DRY);
        assertTrue(everywhere.sites().size() > 3 * g.sites().size(), everywhere.sites().size() + " against " + g.sites().size());
        // Density is a setting too, and 0 means none from the heat flow (the volcanoes' fields are separate).
        assertTrue(heatBuild(new GeothermalRules(1.0, 1.0, 1.0, 400.0, 100.0, 0.0), heatEnv(120, 1200, 15, (x, z) -> 0), DRY).sites().isEmpty());
        int twice = heatBuild(new GeothermalRules(1.0, 1.0, 1.0, 400.0, 100.0, 2.0), heatEnv(120, 1200, 15, (x, z) -> 0), DRY).sites().size();
        assertTrue(twice > 1.5 * g.sites().size(), twice + " against " + g.sites().size());
        // The one switch of the geothermal features turns these off too.
        assertTrue(heatBuild(new GeothermalRules(0.0, 1.0, 1.0, 400.0, 100.0, 1.0), heatEnv(120, 1200, 15, (x, z) -> 0), DRY).sites().isEmpty());
        // A world without the layer has none.
        assertTrue(heatBuild(GeothermalRules.DEFAULT, WET_WARM, DRY).sites().isEmpty());
    }

    @Test
    void heatFlowFieldsKeepEveryGeothermalRule() {
        // Dry ground: fumaroles and altered ground, no pools. Cold: the same (no pools under 3 degC).
        Geothermal dry = heatBuild(GeothermalRules.DEFAULT, heatEnv(120, 120, 15, (x, z) -> 0), DRY);
        Geothermal cold = heatBuild(GeothermalRules.DEFAULT, heatEnv(120, 1200, -6, (x, z) -> 0), DRY);
        assertFalse(dry.sites().isEmpty());
        assertFalse(cold.sites().isEmpty());
        assertEquals(0, count(dry, Geothermal.Kind.POOL));
        assertEquals(0, count(cold, Geothermal.Kind.POOL));
        // Under ice and in water: nothing in the footprint, as for a volcano's fields.
        Magma.Wet wet = (x, z) -> z > 200;
        Geothermal.Env iced = heatEnv(120, 1200, 15, (x, z) -> x > 0 ? 1 : 0);
        Geothermal g = heatBuild(GeothermalRules.DEFAULT, iced, wet);
        assertFalse(g.sites().isEmpty());
        for (Geothermal.Site s : g.sites())
            for (int dx = -s.reach(); dx <= s.reach(); dx++)
                for (int dz = -s.reach(); dz <= s.reach(); dz++) {
                    Geothermal.Part p = s.part(dx, dz);
                    boolean exact = p.pool() || p == Geothermal.Part.RIM || p == Geothermal.Part.VENT || p == Geothermal.Part.SULFUR;
                    if (!exact)
                        continue;
                    int x = Math.floorMod(s.x() + dx + C / 2, C) - C / 2, z = s.z() + dz;
                    assertFalse(wet.at(x, z), "site " + s.id() + " " + p + " in water at " + x + "," + z);
                    assertFalse(iced.iced(x, z), "site " + s.id() + " " + p + " under ice at " + x + "," + z);
                }
        // Every pool is sealed as for the volcanoes' (the same machinery): its level is the ground's.
        assertTrue(heatBuild(GeothermalRules.DEFAULT, heatEnv(120, 1200, 15, (x, z) -> 0), DRY).sites().stream()
                .filter(s -> s.kind() == Geothermal.Kind.POOL).allMatch(s -> s.level() == 74));
        // A coast: ground at the sea takes nothing.
        Magma.Heights low = (x, z) -> SEA + 1.0;
        assertTrue(Geothermal.build(List.of(), GeothermalRules.DEFAULT, Magma.NONE, low, DRY, heatEnv(120, 1200, 15, (x, z) -> 0), C, SEA, MH)
                .sites().isEmpty());
    }

    @Test
    void theSameWorldGivesTheSameHeatFlowFields() {
        Geothermal a = heatBuild(GeothermalRules.DEFAULT, heatEnv(120, 1200, 15, (x, z) -> 0), DRY);
        Geothermal b = heatBuild(GeothermalRules.DEFAULT, heatEnv(120, 1200, 15, (x, z) -> 0), DRY);
        assertEquals(a.sites(), b.sites());
    }

    @Test
    void rulesComeFromTheBakeNotesAndAreClamped() {
        JsonObject heat = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"geothermal_heat_flow_min_mw_m2\": 85, "
                + "\"geothermal_heat_density\": 2.5}}}}").getAsJsonObject();
        assertEquals(new GeothermalRules(1.0, 1.0, 1.0, 400.0, 85.0, 2.5), GeothermalRules.of(heat));
        JsonObject silly2 = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"geothermal_heat_flow_min_mw_m2\": 5, "
                + "\"geothermal_heat_density\": 90}}}}").getAsJsonObject();
        assertEquals(50.0, GeothermalRules.of(silly2).heatFlowMinMwM2());
        assertEquals(4.0, GeothermalRules.of(silly2).heatDensity());
        assertEquals(GeothermalRules.DEFAULT, GeothermalRules.of(new JsonObject()));
        JsonObject notes = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"geothermal_density\": 0.4, "
                + "\"geothermal_reach\": 2.5, \"geothermal_size\": 1.5, \"geothermal_spring_min_precip_mm\": 900}}}}").getAsJsonObject();
        assertEquals(new GeothermalRules(0.4, 2.5, 1.5, 900.0, 100.0, 1.0), GeothermalRules.of(notes));
        JsonObject silly = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"geothermal_density\": -3, "
                + "\"geothermal_reach\": 99, \"geothermal_size\": \"big\"}}}}").getAsJsonObject();
        GeothermalRules r = GeothermalRules.of(silly);
        assertEquals(0.0, r.density());
        assertEquals(3.0, r.reach());
        assertEquals(1.0, r.size(), "a value of the wrong type is the default");
    }
}
