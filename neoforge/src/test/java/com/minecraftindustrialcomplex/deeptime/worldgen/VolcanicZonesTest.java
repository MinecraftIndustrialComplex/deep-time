// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones.Role;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Where the volcanic biomes go ({@link VolcanicZones}; decisions log 2026-10-03): the roles' discs around
 * volcanoes, craters, calderas, pools and magma, built from closed-form inputs. Properties, not one
 * planet's numbers. The old placement had no zones at all (the biomes fell wherever their climate
 * parameters did), so every test here fails on it.
 */
class VolcanicZonesTest {

    private static final int C = 8192;
    private static final double MH = 300.0;

    /** A volcano of basal radius {@code rBlocks} blocks at (x, z). */
    private static Volcanoes.Volcano volcano(int id, String kind, double x, double z, double rBlocks, double activity, double craterBlocks,
                                             boolean caldera) {
        return new Volcanoes.Volcano(id, kind, x, z, rBlocks * MH, 2000, activity, false, false, craterBlocks * MH, 150, caldera);
    }

    private static Geothermal.Site pool(int id, int x, int z, int r) {
        return new Geothermal.Site(id, Geothermal.Kind.POOL, 1, x, z, r, r + 4, 100, 100, true, false, 7L, 0.0, 0.0);
    }

    private static Magma.Body body(int id, int x, int z, int rh) {
        return new Magma.Body(id, "stratovolcano", 0.5, x, z, 100, 40, rh, 6, 90, 94, false, false, 0, 0, 0, rh + 2, null);
    }

    private static boolean has(int bits, Role r) {
        return (bits & (1 << r.ordinal())) != 0;
    }

    /** The highest-priority surface role at a column, or null. */
    private static Role first(VolcanicZones z, int x, int zz) {
        int bits = z.rolesAt(x, zz);
        for (Role r : Role.values())
            if (!r.cave && has(bits, r))
                return r;
        return null;
    }

    @Test
    void anActiveConeHasItsCraterConeAndAshFieldAndNothingBeyond() {
        var v = volcano(1, "stratovolcano", 1000, 500, 80, 0.8, 6, false);
        var z = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CRATER, first(z, 1000, 500));
        assertEquals(Role.CRATER, first(z, 1000 + 9, 500));          // the crater's rim plus its margin (6 + 4), at least 12
        assertEquals(Role.CONE, first(z, 1000 + 30, 500));           // 0.45 R = 36
        assertEquals(Role.ASH_FIELD, first(z, 1000, 500 + 70));      // out to R = 80
        assertEquals(null, first(z, 1000 + 90, 500));
        assertTrue(z.outsideSurfaceZones(1000 + 90, 500) > 9.0 && z.outsideSurfaceZones(1000 + 90, 500) < 11.0);
        assertTrue(z.outsideSurfaceZones(1000, 500) <= 0.0);
    }

    @Test
    void aQuietConeHasNoAshField() {
        var v = volcano(1, "stratovolcano", 0, 0, 80, 0.2, 0, false);
        var z = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CONE, first(z, 0, 0));                     // no crater: the cone reaches the summit
        assertEquals(null, first(z, 60, 0));
    }

    @Test
    void aShieldHasALavaFieldAndACalderaComplexHasItsFloorAndAsh() {
        var shield = volcano(1, "shield", -2000, 0, 100, 0.9, 0, false);
        var centre = volcano(2, "caldera_complex", 2000, 0, 60, 0.9, 20, true);
        var z = VolcanicZones.build(List.of(shield, centre), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CONE, first(z, -2000 + 40, 0));
        assertEquals(Role.LAVA_FIELD, first(z, -2000 + 80, 0));
        assertEquals(Role.CALDERA, first(z, 2000, 15));              // 0.9 of a 20-block rim = 18
        assertEquals(Role.ASH_FIELD, first(z, 2000, 40));            // a centre has no cone: the ash field reaches in
        assertEquals(null, first(z, 2000, 70));
    }

    @Test
    void aHotSpringFieldWinsOverTheConeAndTheCaldera() {
        var v = volcano(1, "caldera_complex", 0, 0, 60, 0.9, 25, true);
        var z = VolcanicZones.build(List.of(v), List.of(pool(5, 10, 0, 3)), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        int bits = z.rolesAt(10, 0);
        assertTrue(has(bits, Role.CALDERA) && has(bits, Role.HOT_SPRING_FIELD));
        assertEquals(Role.HOT_SPRING_FIELD, first(z, 10, 0));
        // Every column of the pool's own reach is in its field.
        var s = pool(5, 10, 0, 3);
        for (int dx = -s.reach(); dx <= s.reach(); dx++)
            assertEquals(Role.HOT_SPRING_FIELD, first(z, 10 + dx, 0));
    }

    @Test
    void zonesAreNeverSmallerThanThreeBiomeCells() {
        var tiny = volcano(1, "stratovolcano", 0, 0, 6, 0.9, 1, false);
        var z = VolcanicZones.build(List.of(tiny), List.of(pool(2, 500, 500, 2)), List.of(body(3, -500, -500, 3)), VolcanicBiomeRules.DEFAULT,
                MH, C);
        for (VolcanicZones.Zone zn : z.zones())
            assertTrue(zn.radius() >= VolcanicZones.MIN_RADIUS, zn.toString());
        // A field ring narrower than MIN_RING beyond the cone is left out: R 6 gives no ash field.
        assertTrue(z.zones().stream().noneMatch(zn -> zn.role() == Role.ASH_FIELD));
    }

    @Test
    void zonesWrapAcrossTheSeam() {
        var v = volcano(1, "stratovolcano", C / 2.0 - 5, 0, 80, 0.8, 0, false);
        var z = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CONE, first(z, -C / 2 + 10, 0));
        assertEquals(Role.ASH_FIELD, first(z, -C / 2 + 60, 0));
        assertTrue(z.outsideSurfaceZones(-C / 2 + 10, 0) <= 0.0);
    }

    @Test
    void thermalCavesFollowTheMagma() {
        var z = VolcanicZones.build(List.of(), List.of(), List.of(body(1, 300, 300, 20), body(2, 900, 900, 0)), VolcanicBiomeRules.DEFAULT,
                MH, C);
        assertTrue(has(z.rolesAt(300 + 29, 300), Role.THERMAL_CAVES));     // 1.5 x 20
        assertEquals(0, z.rolesAt(300 + 32, 300));
        assertEquals(0, z.rolesAt(900, 900));                               // no chamber, no caves
        assertEquals(null, first(z, 300, 300));                             // a cave role is not a surface role
    }

    @Test
    void theReachSettingScalesTheFlanksAndZeroPlacesNothing() {
        var v = volcano(1, "stratovolcano", 0, 0, 80, 0.8, 0, false);
        var wide = VolcanicZones.build(List.of(v), List.of(), List.of(), new VolcanicBiomeRules(2.0), MH, C);
        assertEquals(Role.CONE, first(wide, 70, 0));                 // 0.45 x 80 x 2 = 72
        assertEquals(Role.ASH_FIELD, first(wide, 150, 0));
        var none = VolcanicZones.build(List.of(v), List.of(pool(2, 0, 0, 3)), List.of(body(3, 0, 0, 20)), new VolcanicBiomeRules(0.0), MH,
                C);
        assertTrue(none.zones().isEmpty());
        assertEquals(0, none.rolesAt(0, 0));
    }

    @Test
    void theSettingIsReadFromTheBakeNotes() {
        var notes = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"volcanic_biome_reach\": 1.5}}}}").getAsJsonObject();
        assertEquals(1.5, VolcanicBiomeRules.of(notes).reach());
        var high = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"volcanic_biome_reach\": 9}}}}").getAsJsonObject();
        assertEquals(3.0, VolcanicBiomeRules.of(high).reach());
        var none = JsonParser.parseString("{\"bake\": {}}").getAsJsonObject();
        assertEquals(VolcanicBiomeRules.DEFAULT, VolcanicBiomeRules.of(none));
    }
}
