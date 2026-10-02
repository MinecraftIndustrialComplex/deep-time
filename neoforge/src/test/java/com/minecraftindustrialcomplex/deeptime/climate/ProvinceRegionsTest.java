// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The province → TerraBlender region table (P1-10, plan §7.3): dealt out by weight with stratified
 * ranks, deterministic, kept by region name, stored in {@code climate_mapping.json}.
 */
class ProvinceRegionsTest {

    static final List<ProvinceRegions.Entry> REGIONS = List.of(
            new ProvinceRegions.Entry("minecraft:overworld", 10),
            new ProvinceRegions.Entry("biomesoplenty:overworld_primary", 5),
            new ProvinceRegions.Entry("biomesoplenty:overworld_rare", 2),
            new ProvinceRegions.Entry("somemod:zero", 0));

    @Test
    void everyProvinceGetsARegionInProportionToTheWeights() {
        ProvinceRegions r = ProvinceRegions.assign(170, REGIONS, 7);
        assertEquals(170, r.provinces());
        Map<String, Integer> n = r.counts();
        // 10 : 5 : 2 of 170 is 100 : 50 : 20; stratified ranks are exact to a province.
        assertEquals(100, n.get("minecraft:overworld"), 1);
        assertEquals(50, n.get("biomesoplenty:overworld_primary"), 1);
        assertEquals(20, n.get("biomesoplenty:overworld_rare"), 1);
        assertNull(n.get("somemod:zero"), "a region of weight 0 takes no province");
        for (int p = 0; p < 170; p++)
            assertNotNull(r.regionOf(p));
        assertNull(r.regionOf(170));
        assertNull(r.regionOf(-1));
    }

    @Test
    void aSmallPlanetStillReachesEveryHeavyEnoughRegion() {
        // Independent hashing could leave a light region without a province; stratified ranks cannot
        // while the province count is at least the weight total over the lightest weight.
        for (long salt = 0; salt < 50; salt++) {
            Map<String, Integer> n = ProvinceRegions.assign(17, REGIONS, salt).counts();
            assertEquals(3, n.size(), "salt " + salt + ": " + n);
        }
    }

    @Test
    void theAssignmentIsDeterministicAndTheSaltMovesIt() {
        ProvinceRegions a = ProvinceRegions.assign(60, REGIONS, 1), b = ProvinceRegions.assign(60, REGIONS, 1);
        assertArrayEquals(a.byProvince(), b.byProvince());
        assertEquals(a.names(), b.names());
        ProvinceRegions c = ProvinceRegions.assign(60, REGIONS, 2);
        assertTrue(!java.util.Arrays.equals(a.byProvince(), c.byProvince()), "another world, another deal");
        // The same shares either way.
        assertEquals(a.counts(), c.counts());
    }

    @Test
    void neighbouringIdsAreNotDealtInBlocks() {
        // Province ids run in the order of each province's first cell, so adjacent ids are often
        // neighbours; the hash must not hand them the same region in long runs.
        ProvinceRegions r = ProvinceRegions.assign(200, REGIONS, 3);
        int longest = 1, run = 1;
        for (int p = 1; p < 200; p++) {
            run = r.byProvince()[p] == r.byProvince()[p - 1] ? run + 1 : 1;
            longest = Math.max(longest, run);
        }
        assertTrue(longest <= 20, "longest run of one region " + longest);
    }

    @Test
    void oneRegionTakesAll() {
        ProvinceRegions r = ProvinceRegions.assign(9, List.of(new ProvinceRegions.Entry("minecraft:overworld", 10)), 0);
        assertEquals(Map.of("minecraft:overworld", 9), r.counts());
        assertThrows(IllegalArgumentException.class,
                () -> ProvinceRegions.assign(3, List.of(new ProvinceRegions.Entry("a:b", 0)), 0));
        assertEquals(0, ProvinceRegions.assign(0, REGIONS, 0).provinces());
    }

    @Test
    void theTableSurvivesTheMappingFile() throws Exception {
        ProvinceRegions r = ProvinceRegions.assign(40, REGIONS, 5);
        ClimateMapping m = ClimateMapping.IDENTITY.withRegions(r);
        JsonObject json = m.toJson();
        assertTrue(json.has("regions"));
        ClimateMapping back = ClimateMapping.fromJson(json);
        assertEquals(r.names(), back.regions().names());
        assertArrayEquals(r.byProvince(), back.regions().byProvince());
        assertTrue(!back.meta().has("regions"), "the table is not left in the free-form notes");
        Path file = Files.createTempFile("climate_mapping", ".json");
        try {
            m.write(file);
            assertArrayEquals(r.byProvince(), ClimateMapping.read(file).regions().byProvince());
        } finally {
            Files.deleteIfExists(file);
        }
        // A mapping without regions has none, and writes no key.
        assertNull(ClimateMapping.IDENTITY.regions());
        assertTrue(!ClimateMapping.IDENTITY.toJson().has("regions"));
    }

    @Test
    void anIndexOutsideTheNamesIsRefused() {
        assertThrows(IllegalArgumentException.class, () -> new ProvinceRegions(List.of("a:b"), new int[]{0, 1}));
    }
}
