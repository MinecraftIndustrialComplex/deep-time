// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologyGround.Role;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * The geology gate's ground rules (decisions log 2026-10-04, "Deep Time-side fixes"): which roles a column's province,
 * bedrock, coast and climate give it.
 */
class GeologyGroundTest {

    // Köppen indices: 0 Af, 4 BWh, 7 BSk, 14 Cfa, 15 Cfb, 29 ET.
    private static final Set<String> GRANITE = Set.of("deeptime:rock/intrusive", "deeptime:rock/igneous", "deeptime:rock/felsic");
    private static final Set<String> LIMESTONE = Set.of("deeptime:rock/chemical", "deeptime:rock/sedimentary", "deeptime:rock/carbonate");
    private static final Set<String> MARBLE = Set.of("deeptime:rock/metamorphic", "deeptime:rock/carbonate");
    private static final Set<String> BASALT = Set.of("deeptime:rock/volcanic", "deeptime:rock/igneous", "deeptime:rock/mafic");
    private static final Set<String> RED_BED = Set.of("deeptime:rock/clastic", "deeptime:rock/sedimentary");
    private static final Set<String> SAPROLITE = Set.of("deeptime:rock/unconsolidated");
    private static final Set<String> RHYOLITE = Set.of("deeptime:rock/volcanic", "deeptime:rock/igneous", "deeptime:rock/felsic");

    private static boolean has(int bits, Role r) {
        return (bits & r.bit()) != 0;
    }

    @Test
    void aShieldIsAProvinceAndGraniteIsExposedFelsicIntrusiveRock() {
        assertEquals(Role.SHIELD.bit(), GeologyGround.roles("Shield", Set.of(), false, Set.of(), 15));
        assertEquals(0, GeologyGround.roles("Platform", Set.of(), false, Set.of(), 15));
        assertEquals(Role.GRANITE.bit(), GeologyGround.roles("Orogen", GRANITE, false, Set.of(), 29));
        // Felsic lava is not granite; loose cover is no rock at all.
        assertEquals(0, GeologyGround.roles("Orogen", RHYOLITE, false, Set.of(), 29));
        assertEquals(0, GeologyGround.roles("Orogen", SAPROLITE, false, Set.of(), 29));
        assertEquals(Role.SHIELD.bit() | Role.GRANITE.bit(), GeologyGround.roles("Shield", GRANITE, false, Set.of(), 15));
    }

    @Test
    void coastsTakeTheirBedrocksCliffAndOnlyOnACoast() {
        assertEquals(Role.CARBONATE_COAST.bit(), GeologyGround.roles("Platform", SAPROLITE, true, LIMESTONE, 15));
        assertEquals(Role.CARBONATE_COAST.bit(), GeologyGround.roles("Platform", SAPROLITE, true, MARBLE, 29));
        assertEquals(Role.GRANITE_COAST.bit() | Role.GRANITE.bit(), GeologyGround.roles("Orogen", GRANITE, true, GRANITE, 29));
        assertEquals(Role.BASALT_COAST.bit(), GeologyGround.roles("Large igneous province", BASALT, true, BASALT, 0));
        assertEquals(0, GeologyGround.roles("Large igneous province", BASALT, false, Set.of(), 0));
        assertEquals(0, GeologyGround.roles("Orogen", SAPROLITE, true, Set.of(), 15));
    }

    @Test
    void redBedsWantADryClimateAndKarstAHumidOne() {
        int dry = GeologyGround.roles("Basin", RED_BED, false, Set.of(), 4);
        assertEquals(Role.DRY_SEDIMENTARY.bit(), dry);
        assertEquals(Role.DRY_SEDIMENTARY.bit(), GeologyGround.roles("Basin", RED_BED, false, Set.of(), 7));
        assertEquals(0, GeologyGround.roles("Basin", RED_BED, false, Set.of(), 15));
        // Limestone is sedimentary too: in a dry climate a red-bed canyon may stand on it, in a humid one tower karst.
        int karstDry = GeologyGround.roles("Platform", LIMESTONE, false, Set.of(), 4);
        assertEquals(true, has(karstDry, Role.DRY_SEDIMENTARY));
        assertEquals(false, has(karstDry, Role.HUMID_KARST));
        assertEquals(Role.HUMID_KARST.bit(), GeologyGround.roles("Platform", LIMESTONE, false, Set.of(), 14));
        assertEquals(Role.HUMID_KARST.bit(), GeologyGround.roles("Platform", MARBLE, false, Set.of(), 0));
        // Polar ground is neither dry nor humid here.
        assertEquals(0, GeologyGround.roles("Platform", LIMESTONE, false, Set.of(), 29));
    }

    @Test
    void theProvinceClassesComeFromTheBakeNotes() {
        var notes = JsonParser.parseString("{\"bake\":{\"meta\":{\"provinces\":{\"province\":[{\"id\":0,\"class\":\"Shield\"},"
                + "{\"id\":1,\"class\":\"Orogen\"}]}}}}").getAsJsonObject();
        assertArrayEquals(new String[]{"Shield", "Orogen"}, GeologyGround.provinceClasses(notes));
        assertArrayEquals(new String[0], GeologyGround.provinceClasses(new com.google.gson.JsonObject()));
    }
}
