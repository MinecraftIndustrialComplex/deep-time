// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The magma of active volcanoes ({@link Magma}; Ben, decisions log 2026-10-01, "Volcanoes is a go
 * ahead"): what is plain Java. Cones are built here in closed form, so nothing depends on a
 * planet. The placed blocks are the GameTest {@code magmaUnderActiveVolcanoes}'s.
 */
class MagmaTest {

    private static final int C = 4096, SEA = 63, LAVA_FLOOR = -502;
    private static final double MH = 300.0, MV = 20.0;

    /** A cone at (cx, cz): summit rim at y {@code top} twelve blocks out (3.6 km), a crater 6 blocks deep inside it. */
    private static Magma.Heights cone(int cx, int cz, double top, double base) {
        return (x, z) -> {
            int dx = Math.floorMod(x - cx + C / 2, C) - C / 2;
            double r = Math.hypot(dx, z - cz);
            double flank = Math.max(base, top - 3.0 * Math.max(0.0, r - 12.0));
            return r < 12.0 ? top - 6.0 * (1.0 - r / 12.0) : flank;
        };
    }

    private static Volcanoes.Volcano volcano(int id, double x, double z, double activity) {
        return new Volcanoes.Volcano(id, "stratovolcano", x, z, 14_000, 2_000, activity, false, false, 3_600, 120, false);
    }

    private static Magma one(Magma.Heights h, Magma.Wet wet, MagmaRules rules) {
        return Magma.build(List.of(volcano(1, 100, 50, 0.9)), rules, h, wet, C, SEA, LAVA_FLOOR, MH, MV, (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(C));
    }

    private static final Magma.Wet DRY = (x, z) -> false;

    @Test
    void aVolcanoOnLandGetsAChamberAConduitAndAPlug() {
        Magma m = one(cone(100, 50, 182, 70), DRY, new MagmaRules(3.0, 1.0, 0.0, true).withoutColumn());
        Magma.Body b = m.bodies().get(0);
        assertEquals(175, b.floorY(), "the crater floor: H 176 at the centre");
        // Radius: a tenth of 14 km = 1.4 km = 5 blocks across; half-height a fifth of that = 14 blocks up.
        assertEquals(5, b.chamberRh());
        assertEquals(14, b.chamberRv());
        // Its top 3 km (150 blocks) under the crater floor.
        assertEquals(175 - 150, b.chamberY() + b.chamberRv());
        assertFalse(b.lake());
        assertFalse(b.submerged());
        assertEquals(175 - Magma.PLUG, b.conduitTop());
        assertTrue(b.plugTop() >= b.floorY(), "on land the plug reaches the surface");
        assertEquals(Magma.Part.CHAMBER, b.at(0, 0, b.chamberY()));
        assertEquals(Magma.Part.CHAMBER, b.at(5, 0, b.chamberY()));
        assertEquals(Magma.Part.CHAMBER_SHELL, b.at(6, 0, b.chamberY()));
        assertEquals(Magma.Part.CHAMBER_SHELL, b.at(0, 0, b.chamberY() - b.chamberRv() - 1));
        assertEquals(Magma.Part.NONE, b.at(9, 0, b.chamberY()));
        assertEquals(Magma.Part.CONDUIT, b.at(0, 0, 100));
        assertEquals(Magma.Part.CONDUIT, b.at(1, 0, 100));
        assertEquals(Magma.Part.CONDUIT_SHELL, b.at(1, 1, 100));
        assertEquals(Magma.Part.CONDUIT_SHELL, b.at(2, 0, 100));
        assertEquals(Magma.Part.NONE, b.at(3, 0, 100));
        assertEquals(Magma.Part.PLUG, b.at(0, 0, b.conduitTop() + 1));
        assertEquals(Magma.Part.PLUG, b.at(0, 0, b.floorY()));
        assertEquals(Magma.Part.NONE, b.at(0, 0, b.plugTop() + 1));
        assertEquals(Magma.Part.CHAMBER, m.at(100, b.chamberY(), 50));
        assertEquals(Magma.Part.NONE, m.at(300, b.chamberY(), 50));
    }

    @Test
    void theMostActiveVolcanoesOnLandHoldACraterLake() {
        List<Volcanoes.Volcano> vs = new ArrayList<>();
        double[] activity = {0.2, 0.9, 0.5, 0.7};
        for (int i = 0; i < 4; i++)
            vs.add(volcano(i + 1, 200 + 400 * i, 0, activity[i]));
        Magma.Heights h = (x, z) -> {
            double best = 70;
            for (int i = 0; i < 4; i++)
                best = Math.max(best, cone(200 + 400 * i, 0, 182, 70).at(x, z));
            return best;
        };
        Magma m = Magma.build(vs, new MagmaRules(3.0, 1.0, 0.25, true), h, DRY, C, SEA, LAVA_FLOOR, MH, MV, (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(C));
        assertEquals(1, m.bodies().stream().filter(Magma.Body::lake).count(), "a quarter of four");
        Magma.Body b = m.bodies().stream().filter(Magma.Body::lake).findFirst().orElseThrow();
        assertEquals(2, b.id(), "the most active one");
        // Half the crater's 3.6 km radius: 6 blocks. The lava's surface is one block under the lowest
        // ground of the two-block ring around the lake.
        assertEquals(6, b.lakeRadius());
        assertTrue(b.lakeTop() >= b.floorY() && b.lakeTop() <= 180, "lake top " + b.lakeTop());
        for (int dx = -8; dx <= 8; dx++)
            for (int dz = -8; dz <= 8; dz++) {
                int d2 = dx * dx + dz * dz;
                if (d2 > 36 && d2 <= 64)
                    assertTrue(Magma.top(h.at(b.x() + dx, b.z() + dz)) > b.lakeTop(), "the rim holds the lake");
            }
        assertEquals(Magma.Part.LAKE, b.at(0, 0, b.lakeTop()));
        assertEquals(Magma.Part.LAKE, b.at(6, 0, b.lakeBottom()));
        assertEquals(Magma.Part.LAKE_AIR, b.at(0, 0, b.lakeTop() + 1));
        assertEquals(Magma.Part.LAKE_SHELL, b.at(7, 0, b.lakeTop()));
        assertEquals(Magma.Part.LAKE_SHELL, b.at(3, 3, b.lakeBottom() - 1));
        assertEquals(Magma.Part.CONDUIT, b.at(0, 0, b.lakeBottom() - 1), "the conduit opens into the lake's floor");
        assertEquals(b.lakeBottom() - 1, b.conduitTop());
        // Nobody reaches a volcano beyond the pole cap's line: no lake there, whatever its activity.
        Magma polar = Magma.build(vs, new MagmaRules(3.0, 1.0, 1.0, true), h, DRY, C, SEA, LAVA_FLOOR, MH, MV, -1);
        assertEquals(0, polar.bodies().stream().filter(Magma.Body::lake).count());
        assertEquals(4, polar.bodies().stream().filter(x -> x.chamberRh() > 0).count(), "they keep their chambers");
        // Every share from none to all.
        for (double share : new double[]{0.0, 0.5, 1.0}) {
            Magma n = Magma.build(vs, new MagmaRules(3.0, 1.0, share, true), h, DRY, C, SEA, LAVA_FLOOR, MH, MV, (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(C));
            assertEquals(Math.round(share * 4), n.bodies().stream().filter(Magma.Body::lake).count(), "share " + share);
        }
    }

    /** Every molten block's six neighbours belong to the body too: lava never meets what was there before. */
    @Test
    void magmaIsSealedOnEverySide() {
        for (double share : new double[]{0.0, 1.0}) {
            Magma.Body b = one(cone(100, 50, 182, 70), DRY, new MagmaRules(3.0, 1.5, share, true)).bodies().get(0);
            assertEquals(share > 0, b.lake());
            assertTrue(sealed(b) > b.chamberBlocks() / 2);
        }
    }

    /**
     * The same for the broad, flat lenses under a large active shield (the GameTest planet's volcano
     * 78 of 2026-10-03): a lens 50 to 90 blocks across and 5 blocks high. With the shell's radii
     * scaled up by 1.5 blocks, the shell above such a lens's flanks was thinner than a block and lava
     * faced the rock (and the caves) over it.
     */
    @Test
    void broadLensesAreSealedOnEverySide() {
        Magma.Body b = kindOf("shield", false, 45_000, 1.0, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0);
        int widest = 0;
        for (int[] l : b.deep().lenses())
            widest = Math.max(widest, l[1]);
        assertTrue(widest >= 51, "the case needs a lens 51 blocks or more in radius: " + widest);
        sealed(b);
    }

    /** Asserts every molten block of a body faces only the body on all six sides; returns the molten blocks. */
    private static long sealed(Magma.Body b) {
        long molten = 0;
        int[][] n6 = {{1, 0, 0}, {-1, 0, 0}, {0, 0, 1}, {0, 0, -1}, {0, 1, 0}, {0, -1, 0}};
        for (int dx = -b.reach() - 1; dx <= b.reach() + 1; dx++)
            for (int dz = -b.reach() - 1; dz <= b.reach() + 1; dz++)
                for (int y = b.minY() - 1; y <= b.maxY() + 1; y++) {
                    Magma.Part p = b.at(dx, dz, y);
                    if (Math.abs(dx) > b.reach() || Math.abs(dz) > b.reach() || y < b.minY() || y > b.maxY()) {
                        assertEquals(Magma.Part.NONE, p, "nothing outside the body's reach");
                        continue;
                    }
                    if (!p.molten())
                        continue;
                    molten++;
                    for (int[] d : n6) {
                        Magma.Part q = b.at(dx + d[0], dz + d[2], y + d[1]);
                        assertNotEquals(Magma.Part.NONE, q,
                                p + " at (" + dx + ", " + y + ", " + dz + ") is open towards " + d[0] + "," + d[1] + "," + d[2]);
                        if (q == Magma.Part.LAKE_AIR)
                            assertTrue(p == Magma.Part.LAKE && d[1] == 1, "only a lake's surface faces air");
                    }
                }
        return molten;
    }

    @Test
    void aVolcanoUnderOrBesideWaterGetsNoLakeAndABuriedPlug() {
        MagmaRules all = new MagmaRules(3.0, 1.0, 1.0, true);
        // Under the sea: the summit at y 40.
        Magma.Body sub = one(cone(100, 50, 46, -100), DRY, all).bodies().get(0);
        assertTrue(sub.submerged());
        assertFalse(sub.lake());
        assertEquals(sub.floorY() - Magma.BURIED, sub.plugTop());
        assertTrue(sub.chamberRh() > 0, "a submarine volcano still has its chamber");
        // On land, with a lake or river at the summit.
        Magma.Body wet = one(cone(100, 50, 182, 70), (x, z) -> x == 100 && z == 50, all).bodies().get(0);
        assertTrue(wet.submerged());
        assertFalse(wet.lake());
        // Water at the rim only: the plug reaches the surface, but no lava lake.
        Magma.Body rim = one(cone(100, 50, 182, 70), (x, z) -> x == 107 && z == 50, all).bodies().get(0);
        assertFalse(rim.submerged());
        assertFalse(rim.lake());
        // A peak too sharp to hold a lake without cutting more than LAKE_MAX_CUT: no lake.
        Magma.Heights spike = (x, z) -> 300 - 8.0 * Math.hypot(x - 100, z - 50);
        assertFalse(one(spike, DRY, all).bodies().get(0).lake());
    }

    @Test
    void aChamberStaysBetweenTheGroundAndTheLavaFloor() {
        MagmaRules rules = new MagmaRules(3.0, 1.0, 0.0, true);
        // A deep submarine summit: 3 km down would be below the lava floor, so the chamber is raised.
        Magma.Body deep = one(cone(100, 50, -200, -330), DRY, rules).bodies().get(0);
        assertTrue(deep.chamberRh() > 0);
        assertTrue(deep.chamberY() - deep.chamberRv() >= LAVA_FLOOR + Magma.FLOOR_GAP, "bottom " + (deep.chamberY() - deep.chamberRv()));
        assertTrue(deep.chamberY() + deep.chamberRv() <= deep.floorY() - 3 * Magma.COVER);
        // A summit at the lava floor itself: no room, no magma, and no part anywhere.
        Magma.Body none = one(cone(100, 50, LAVA_FLOOR + 20, LAVA_FLOOR), DRY, rules).bodies().get(0);
        assertEquals(0, none.chamberRh());
        assertEquals(Magma.Part.NONE, none.at(0, 0, none.floorY() - 10));
        // Size 0: no bodies at all.
        assertTrue(one(cone(100, 50, 182, 70), DRY, new MagmaRules(3.0, 0.0, 1.0, true)).bodies().isEmpty());
        // A bigger chamber and a shallower one are what the settings say.
        Magma.Body big = one(cone(100, 50, 182, 70), DRY, new MagmaRules(1.0, 2.0, 0.0, true)).bodies().get(0);
        assertEquals(9, big.chamberRh());
        assertEquals(28, big.chamberRv());
        assertEquals(175 - 50, big.chamberY() + big.chamberRv());
    }

    @Test
    void bodiesAreFoundAcrossTheSeam() {
        Magma m = Magma.build(List.of(volcano(7, -C / 2 + 2, 0, 0.5)), new MagmaRules(3.0, 1.0, 0.0, true),
                cone(-C / 2 + 2, 0, 182, 70), DRY, C, SEA, LAVA_FLOOR, MH, MV, (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(C));
        Magma.Body b = m.bodies().get(0);
        assertEquals(-C / 2 + 2, b.x());
        // The chunk just west of the seam (x up to C/2 − 1) holds part of the chamber.
        assertEquals(1, m.near(C / 2 - 16, -8, C / 2 - 1, 7).size());
        assertEquals(-3, m.dx(C / 2 - 1, b));
        assertEquals(Magma.Part.CHAMBER, m.at(C / 2 - 1, b.chamberY(), 0));
        assertTrue(m.near(0, 0, 15, 15).isEmpty());
    }

    @Test
    void theSettingsAndTheListAreRead() {
        JsonObject notes = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"magma_chamber_depth_km\": 5.5, "
                + "\"magma_chamber_size\": 2, \"crater_lake_share\": 0.1, \"remove_lava_features\": false}}}}").getAsJsonObject();
        assertEquals(new MagmaRules(5.5, 2.0, 0.1, false), MagmaRules.of(notes));
        assertEquals(MagmaRules.DEFAULT, MagmaRules.of(null));
        assertEquals(MagmaRules.DEFAULT, MagmaRules.of(JsonParser.parseString("{\"bake\": {}}").getAsJsonObject()));
        assertTrue(MagmaRules.DEFAULT.removeLavaFeatures());
        Volcanoes v = Volcanoes.parse(JsonParser.parseString("{\"version\": 1, \"t_myr\": 0, \"volcanoes\": [{\"id\": 3, "
                + "\"kind\": \"shield\", \"x\": -12.5, \"z\": 40.25, \"radius_m\": 60000, \"height_m\": 5200, \"activity\": 0.75, "
                + "\"oceanic\": true, \"silicic\": false, \"age_myr\": 1.5, \"crater\": {\"radius_m\": 2400, \"depth_m\": 150, "
                + "\"caldera\": true}}, {\"id\": 4, \"kind\": \"stratovolcano\", \"x\": 1, \"z\": 2, \"radius_m\": 9000, "
                + "\"height_m\": 1500, \"activity\": 0.3, \"oceanic\": false, \"silicic\": true, \"age_myr\": 0.2, \"crater\": null}]}")
                .getAsJsonObject(), "test");
        assertEquals(2, v.list().size());
        assertEquals(new Volcanoes.Volcano(3, "shield", -12.5, 40.25, 60000, 5200, 0.75, true, false, 2400, 150, true), v.list().get(0));
        assertEquals(0.0, v.list().get(1).craterRadiusM());
        assertThrows(RuntimeException.class, () -> Volcanoes.parse(JsonParser.parseString("{\"version\": 2, \"volcanoes\": []}")
                .getAsJsonObject(), "test"));
    }

    // ---- the crust-deep column (Ben, decisions log 2026-10-02) ------------------------------------------------

    private static final MagmaRules NO_COLUMN = new MagmaRules(3.0, 1.0, 0.0, true).withoutColumn();
    private static final MagmaRules COLUMN = new MagmaRules(3.0, 1.0, 0.0, true);

    private static Magma kindOf(String kind, boolean silicic, double radiusM, double activity, MagmaRules rules, int lavaFloor, int minY) {
        var v = new Volcanoes.Volcano(1, kind, 100, 50, radiusM, 2_000, activity, "shield".equals(kind), silicic, 3_600, 120, false);
        return Magma.build(List.of(v), rules, cone(100, 50, 182, 70), DRY, C, SEA, lavaFloor, minY, MH, MV, C / 4);
    }

    /** Molten blocks of a body, by enumerating its box (before the cover rule of the pass). */
    private static long molten(Magma.Body b) {
        long n = 0;
        for (int dx = -b.reach(); dx <= b.reach(); dx++)
            for (int dz = -b.reach(); dz <= b.reach(); dz++)
                for (int y = b.minY(); y <= b.maxY(); y++)
                    if (b.at(dx, dz, y).molten())
                        n++;
        return n;
    }

    @Test
    void aColumnOfStackedLensesRunsFromTheChamberToTheLavaFloor() {
        Magma.Body b = one(cone(100, 50, 182, 70), DRY, COLUMN).bodies().get(0);
        Magma.Deep d = b.deep();
        assertNotNull(d, "an active volcano with room under it gets the column");
        int[][] lenses = d.lenses();   // {y, radius, half-height, offset x, offset z}
        assertTrue(lenses.length >= 8, "lenses " + lenses.length);
        assertEquals(b.lensCount(), lenses.length);
        for (int k = 0; k < lenses.length; k++) {
            int[] l = lenses[k];
            assertTrue(l[1] >= 2 * l[2] + 1, "wider than tall: radius " + l[1] + ", half-height " + l[2]);
            assertTrue(2 * l[2] + 1 <= 11, "a few blocks thick");
            if (k > 0) {
                int gap = lenses[k - 1][0] - l[0];
                assertTrue(gap >= 12 && gap <= 45, "tens of blocks apart: " + gap);
            }
        }
        // Every lens lies under the chamber and the column grows wider with depth.
        assertTrue(lenses[0][0] + lenses[0][2] + 1 < b.chamberY() - b.chamberRv(), "the first lens is under the chamber");
        assertTrue(lenses[lenses.length - 1][1] > lenses[0][1] * 1.8, "the lowest lens is much wider than the highest");
        // The basal lens straddles the lava floor, above the bedrock.
        int[] basal = lenses[lenses.length - 1];
        assertTrue(basal[0] - basal[2] <= LAVA_FLOOR && basal[0] + basal[2] > LAVA_FLOOR, "the basal lens meets the lava floor");
        assertTrue(basal[0] - basal[2] >= LAVA_FLOOR - 10 + Magma.BEDROCK_GAP, "and stays off the bedrock");
        assertEquals(Magma.Part.LENS, b.at(basal[3], basal[4], basal[0]));
        assertTrue(b.minY() <= basal[0] - basal[2] - 2, "the body reaches the basal lens's shell");
    }

    @Test
    void theColumnIsOneConnectedBodyOfLava() {
        Magma.Body b = one(cone(100, 50, 182, 70), DRY, COLUMN).bodies().get(0);
        int r = b.reach(), w = 2 * r + 1, y0 = b.minY(), h = b.maxY() - y0 + 1;
        boolean[] lava = new boolean[w * w * h], seen = new boolean[w * w * h];
        long total = 0;
        for (int dx = -r; dx <= r; dx++)
            for (int dz = -r; dz <= r; dz++)
                for (int y = y0; y <= b.maxY(); y++)
                    if (b.at(dx, dz, y).molten()) {
                        lava[((dx + r) * w + dz + r) * h + y - y0] = true;
                        total++;
                    }
        java.util.ArrayDeque<int[]> q = new java.util.ArrayDeque<>();
        q.add(new int[]{r, r, b.chamberY() - y0});
        seen[(r * w + r) * h + b.chamberY() - y0] = true;
        long reached = 0;
        int[][] n6 = {{1, 0, 0}, {-1, 0, 0}, {0, 1, 0}, {0, -1, 0}, {0, 0, 1}, {0, 0, -1}};
        while (!q.isEmpty()) {
            int[] c = q.poll();
            reached++;
            for (int[] d : n6) {
                int a = c[0] + d[0], z = c[1] + d[1], y = c[2] + d[2];
                if (a < 0 || z < 0 || a >= w || z >= w || y < 0 || y >= h)
                    continue;
                int i = (a * w + z) * h + y;
                if (lava[i] && !seen[i]) {
                    seen[i] = true;
                    q.add(new int[]{a, z, y});
                }
            }
        }
        assertEquals(total, reached, "the chamber, the conduit, every lens and every segment are one body of lava");
    }

    @Test
    void theColumnGivesSeveralTimesTheLava() {
        for (double radiusM : new double[]{14_000, 28_000}) {
            long before = molten(kindOf("stratovolcano", false, radiusM, 0.9, NO_COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0));
            long after = molten(kindOf("stratovolcano", false, radiusM, 0.9, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0));
            assertTrue(after >= 3 * before, "a " + radiusM / 1000 + "-km arc volcano: " + before + " -> " + after);
        }
        long shieldBefore = molten(kindOf("shield", false, 60_000, 0.9, NO_COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0));
        long shieldAfter = molten(kindOf("shield", false, 60_000, 0.9, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0));
        assertTrue(shieldAfter >= 3 * shieldBefore, "the shield: " + shieldBefore + " -> " + shieldAfter);
    }

    @Test
    void theColumnFollowsTheWorldsFloorNotANumber() {
        // The floor moves from y -352 to y -512: the lava floor from -342 to -502. The column goes to it.
        Magma.Body high = kindOf("stratovolcano", false, 14_000, 0.9, COLUMN, -342, -352).bodies().get(0);
        Magma.Body low = kindOf("stratovolcano", false, 14_000, 0.9, COLUMN, -502, -512).bodies().get(0);
        int[][] a = high.deep().lenses(), z = low.deep().lenses();
        assertTrue(z.length > a.length, "a deeper world has more lenses: " + a.length + " -> " + z.length);
        assertTrue(a[a.length - 1][0] > -342 && a[a.length - 1][0] < -330, "basal lens at the -342 floor");
        assertTrue(z[z.length - 1][0] > -502 && z[z.length - 1][0] < -490, "basal lens at the -502 floor");
        assertTrue(low.minY() >= -512 + Magma.BEDROCK_GAP - 2, "nothing within the bedrock");
        // The same spacing: about as many lenses per block of depth.
        assertEquals((double) (a[0][0] - a[a.length - 1][0]) / (a.length - 1), (double) (z[0][0] - z[z.length - 1][0]) / (z.length - 1), 6.0);
        // The chunk-generation bounds: the whole body lies in minY..maxY.
        for (int[] l : z)
            assertTrue(l[0] - l[2] - 2 >= low.minY() && l[0] + l[2] + 2 <= low.maxY());
        // With no lava floor (lava_floor_blocks 0) the column still stops short of the bedrock.
        Magma.Body bare = kindOf("stratovolcano", false, 14_000, 0.9, COLUMN, -512, -512).bodies().get(0);
        assertTrue(bare.minY() >= -512 + Magma.BEDROCK_GAP - 2, "bottom " + bare.minY());
    }

    @Test
    void kindRadiusAndActivityScaleTheSystem() {
        Magma.Body arc = kindOf("stratovolcano", false, 14_000, 1.0, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0);
        Magma.Body quiet = kindOf("stratovolcano", false, 14_000, 0.0, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0);
        Magma.Body silicic = kindOf("stratovolcano", true, 14_000, 1.0, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0);
        Magma.Body shield = kindOf("shield", false, 14_000, 1.0, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0);
        Magma.Body big = kindOf("stratovolcano", false, 40_000, 1.0, COLUMN, LAVA_FLOOR, LAVA_FLOOR - 10).bodies().get(0);
        assertTrue(silicic.chamberRh() > arc.chamberRh() * 1.3, "silicic centres hold larger shallow chambers");
        assertTrue(shield.deep().lenses()[0][1] > arc.deep().lenses()[0][1] * 1.2, "shield lenses are broader");
        assertTrue(quiet.deep().lenses()[0][1] <= arc.deep().lenses()[0][1], "quiet volcanoes' lenses are not wider");
        assertTrue(big.deep().lenses()[0][1] > arc.deep().lenses()[0][1], "a bigger volcano has wider lenses");
        assertTrue(big.reach() > arc.reach());
    }

    @Test
    void theColumnSettingsAreSettings() {
        Magma.Body base = one(cone(100, 50, 182, 70), DRY, COLUMN).bodies().get(0);
        long baseLava = molten(base);
        MagmaRules few = new MagmaRules(3.0, 1.0, 0.0, true, 3, 28.0, 1.0, 1.0);
        Magma.Body fewer = one(cone(100, 50, 182, 70), DRY, few).bodies().get(0);
        assertEquals(3, fewer.lensCount());
        int[][] fl = fewer.deep().lenses();
        assertTrue(fl[fl.length - 1][0] < LAVA_FLOOR + 10, "a capped count widens the spacing: the column still reaches the floor");
        MagmaRules wide = new MagmaRules(3.0, 1.0, 0.0, true, 24, 28.0, 2.0, 1.0);
        assertTrue(molten(one(cone(100, 50, 182, 70), DRY, wide).bodies().get(0)) > 2 * baseLava, "twice the width, much more lava");
        MagmaRules sparse = new MagmaRules(3.0, 1.0, 0.0, true, 24, 56.0, 1.0, 1.0);
        Magma.Body sp = one(cone(100, 50, 182, 70), DRY, sparse).bodies().get(0);
        assertTrue(sp.lensCount() < base.lensCount() * 0.6 && sp.lensCount() >= 2, base.lensCount() + " -> " + sp.lensCount());
        assertNull(one(cone(100, 50, 182, 70), DRY, NO_COLUMN).bodies().get(0).deep());
        assertNull(one(cone(100, 50, 182, 70), DRY, new MagmaRules(3.0, 1.0, 0.0, true, 24, 28.0, 0.0, 1.0)).bodies().get(0).deep());
        // Read from the world's notes; absent keys keep the defaults.
        JsonObject notes = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"magma_lens_count\": 7, "
                + "\"magma_lens_spacing_blocks\": 40, \"magma_lens_width\": 2.5, \"magma_mush_width\": 0}}}}").getAsJsonObject();
        MagmaRules r = MagmaRules.of(notes);
        assertEquals(7, r.lensCount());
        assertEquals(40.0, r.lensSpacingBlocks());
        assertEquals(2.5, r.lensWidth());
        assertEquals(1.5, MagmaRules.DEFAULT.lensWidth());
        assertEquals(0.0, r.mushWidth());
        assertEquals(3.0, r.chamberDepthKm());
        assertEquals(24, MagmaRules.DEFAULT.lensCount());
        assertEquals(28.0, MagmaRules.DEFAULT.lensSpacingBlocks());
        assertTrue(MagmaRules.DEFAULT.hasColumn());
        assertFalse(NO_COLUMN.hasColumn());
    }

    @Test
    void theColumnSitsInABodyOfAlteredHotRock() {
        Magma.Body b = one(cone(100, 50, 182, 70), DRY, COLUMN).bodies().get(0);
        int[][] lenses = b.deep().lenses();
        for (int[] l : lenses) {
            // Every lens is inside the mush: its centre and its rim.
            assertTrue(b.mush(l[3], l[4], l[0]), "mush at a lens centre");
            assertTrue(b.mush(l[3] + l[1] - 1, l[4], l[0]), "mush at a lens's rim");
        }
        // Between two lenses the mush is there too; none above the chamber's equator.
        int mid = (lenses[3][0] + lenses[4][0]) / 2;
        assertTrue(b.mush(0, 0, mid));
        assertTrue(b.mush(lenses[3][1], 0, lenses[3][0]));
        assertFalse(b.mush(0, 0, b.chamberY() + 1));
        assertFalse(b.mush(b.reach() + 1, 0, lenses[3][0]), "beyond the reach");
        assertFalse(b.mush(0, 0, b.minY() - 1));
        // It wraps the lava: no molten block outside it.
        for (int dx = -b.reach(); dx <= b.reach(); dx += 1)
            for (int y = b.minY(); y <= b.chamberY() - b.chamberRv(); y++)
                if (b.at(dx, 0, y).molten())
                    assertTrue(b.mush(dx, 0, y), "lava at " + dx + ", " + y + " outside the mush");
        // The setting: wider mush, none.
        Magma.Body wide = one(cone(100, 50, 182, 70), DRY, new MagmaRules(3.0, 1.0, 0.0, true, 24, 28.0, 1.0, 2.0)).bodies().get(0);
        assertTrue(wide.reach() > b.reach());
        Magma.Body none = one(cone(100, 50, 182, 70), DRY, new MagmaRules(3.0, 1.0, 0.0, true, 24, 28.0, 1.0, 0.0)).bodies().get(0);
        assertFalse(none.mush(0, 0, lenses[3][0]));
        assertNotNull(none.deep());
        // Without the column there is no mush at all.
        assertFalse(one(cone(100, 50, 182, 70), DRY, NO_COLUMN).bodies().get(0).mush(0, 0, b.chamberY() - 5));
    }

    @Test
    void aChamberAtTheFloorHasNoColumnAndAThinOneHasFewLenses() {
        Magma.Body deepSea = one(cone(100, 50, -200, -330), DRY, COLUMN).bodies().get(0);
        assertTrue(deepSea.chamberRh() > 0);
        assertTrue(deepSea.deep() == null || deepSea.lensCount() <= 4, "a chamber near the floor: " + deepSea.lensCount());
        // Whatever it has stays between the chamber and the floor.
        assertTrue(deepSea.minY() >= LAVA_FLOOR - 10 + Magma.BEDROCK_GAP - 2);
        Magma.Body mid = one(cone(100, 50, -50, -120), DRY, COLUMN).bodies().get(0);
        Magma.Body tall = one(cone(100, 50, 182, 70), DRY, COLUMN).bodies().get(0);
        assertTrue(mid.lensCount() < tall.lensCount());
    }

    @Test
    void aLavaFeatureIsOneWhoseConfigurationPlacesLava() {
        // Vanilla's lake_lava: a lake whose fluid provider's state is lava.
        assertTrue(LavaFeatureRule.namesLavaState(JsonParser.parseString("{\"type\": \"minecraft:lake\", \"config\": {\"barrier\": "
                + "{\"type\": \"minecraft:simple_state_provider\", \"state\": {\"Name\": \"minecraft:stone\"}}, \"fluid\": "
                + "{\"type\": \"minecraft:simple_state_provider\", \"state\": {\"Name\": \"minecraft:lava\", \"Properties\": {\"level\": \"0\"}}}}}")));
        // Vanilla's spring_lava_overworld: a spring whose fluid state is lava.
        assertTrue(LavaFeatureRule.namesLavaState(JsonParser.parseString("{\"type\": \"minecraft:spring_feature\", \"config\": "
                + "{\"state\": {\"Name\": \"minecraft:lava\", \"Properties\": {\"falling\": \"true\"}}, \"valid_blocks\": [\"minecraft:stone\"]}}")));
        // Terralith's lava_remover (lava sources into deepslate): it would eat a magma chamber.
        assertTrue(LavaFeatureRule.namesLavaState(JsonParser.parseString("{\"type\": \"minecraft:netherrack_replace_blobs\", \"config\": "
                + "{\"radius\": 11, \"state\": {\"Name\": \"minecraft:deepslate\"}, \"target\": {\"Name\": \"minecraft:lava\", "
                + "\"Properties\": {\"level\": \"0\"}}}}")));
        // A water spring, and a feature that only tests for lava nearby, are not lava features.
        assertFalse(LavaFeatureRule.namesLavaState(JsonParser.parseString("{\"type\": \"minecraft:spring_feature\", \"config\": "
                + "{\"state\": {\"Name\": \"minecraft:water\"}, \"valid_blocks\": [\"minecraft:stone\"]}}")));
        assertFalse(LavaFeatureRule.namesLavaState(JsonParser.parseString("{\"type\": \"minecraft:simple_block\", \"config\": "
                + "{\"to_place\": {\"type\": \"minecraft:simple_state_provider\", \"state\": {\"Name\": \"minecraft:magma_block\"}}, "
                + "\"predicate\": {\"type\": \"minecraft:matching_blocks\", \"blocks\": \"minecraft:lava\"}}}")));
    }
}
