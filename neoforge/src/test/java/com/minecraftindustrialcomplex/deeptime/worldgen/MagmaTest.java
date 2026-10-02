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
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The magma of active volcanoes ({@link Magma}; Ben, decisions log 2026-10-01, "Volcanoes is a go
 * ahead"): what is plain Java. Cones are built here in closed form, so nothing depends on a
 * planet. The placed blocks are the GameTest {@code magmaUnderActiveVolcanoes}'s.
 */
class MagmaTest {

    private static final int C = 4096, SEA = 63, LAVA_FLOOR = -342;
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
        Magma m = one(cone(100, 50, 182, 70), DRY, new MagmaRules(3.0, 1.0, 0.0, true));
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
            assertTrue(molten > b.chamberBlocks() / 2, "molten " + molten);
        }
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
