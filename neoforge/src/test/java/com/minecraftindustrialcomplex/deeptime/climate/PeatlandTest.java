// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@link Peatland} (BIO-135): cold wetland is taiga unless the pack's own biome source answers with a
 * peatland biome at the swamp row of the cold band, region by region.
 */
class PeatlandTest {

    static final OverworldBands REF = OverworldBands.defaults();

    static ClimateMapping identity() {
        return ClimateMapping.build(REF, ClimateMappingTest.refErosion(), ClimateMappingTest.refPv(), REF, null, null, new JsonObject());
    }

    /** A pack whose region "b" has a bog (cold, swamp row E6) and region "a" only a swamp that is not peatland. */
    static final Peatland.Probe PACK = (region, temp, hum, cont, ero, weird) -> {
        boolean swampRow = ero >= 0.55;
        if (region.equals("b") && swampRow && temp >= -0.45 && temp < -0.15)
            return "mod:bog";
        return null;   // vanilla's swamp and the like: not peatland
    };

    @Test
    void aPackWithABogInOneRegionHasPeatlandThere() {
        Peatland p = Peatland.decide(identity(), REF, List.of("a", "b"), PACK);
        assertEquals(List.of("b"), p.regions());
        assertEquals(List.of("mod:bog"), p.biomes());
        assertTrue(p.appliesTo("b"));
        assertFalse(p.appliesTo("a"));
        assertFalse(p.appliesTo(null), "a province without a region is taiga");
        // The erosion given to the flagged ground is inside the swamp band E6.
        assertTrue(p.erosion() >= REF.erosion()[6] && p.erosion() <= 1.0, "erosion " + p.erosion());
    }

    @Test
    void aPackWithoutPeatlandGivesTaigaEverywhere() {
        // Vanilla and Terralith: the swamp row of the cold band holds swamp, which is not cold.
        Peatland p = Peatland.decide(identity(), REF, List.of(Peatland.NO_TABLE), (r, t, h, c, e, w) -> null);
        assertTrue(p.regions().isEmpty());
        assertTrue(p.biomes().isEmpty());
    }

    @Test
    void aBogOnlyInTheFrozenOrTemperateBandIsNotColdWetland() {
        Peatland.Probe frozen = (r, t, h, c, e, w) -> e >= 0.55 && t < -0.45 ? "mod:muskeg" : null;
        Peatland.Probe temperate = (r, t, h, c, e, w) -> e >= 0.55 && t >= -0.15 ? "mod:fen" : null;
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), frozen).regions().isEmpty());
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), temperate).regions().isEmpty());
    }

    @Test
    void aBogThatAnswersAtTooFewPointsDoesNotCount() {
        // Peatland at the driest humidities of the wet range only (under a fifth of the points).
        Peatland.Probe few = (r, t, h, c, e, w) -> e >= 0.55 && h < 0.05 ? "mod:bog" : null;
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), few).regions().isEmpty());
        // Under half of the points (one weirdness sign of two) is not a peatland region either.
        Peatland.Probe half = (r, t, h, c, e, w) -> e >= 0.55 && w > 0.3 ? "mod:bog" : null;
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), half).regions().isEmpty());
    }

    @Test
    void theProbePointsAreColdWetClimatesAtTheSwampRow() {
        List<double[]> seen = new ArrayList<>();
        Peatland.decide(identity(), REF, List.of("b"), (r, t, h, c, e, w) -> {
            seen.add(new double[] {t, h, c, e, w});
            return null;
        });
        assertEquals(5 * 5 * 3 * 6, seen.size());
        for (double[] p : seen) {
            assertTrue(p[0] >= -0.45 && p[0] < -0.15, "temperature in the cold band: " + p[0]);
            assertTrue(p[1] >= -0.1, "wet: " + p[1]);
            assertTrue(p[2] > -0.11, "inland: " + p[2]);
            assertTrue(p[3] >= 0.55 && p[3] <= 1.0, "erosion in E6: " + p[3]);
        }
    }

    @Test
    void theVerdictRoundTripsInTheMappingFile() throws Exception {
        Peatland p = new Peatland(0.78, List.of("b", Peatland.NO_TABLE), List.of("mod:bog"));
        ClimateMapping m = identity().withPeatland(p);
        Path dir = Files.createTempDirectory("peatland");
        Path f = dir.resolve(ClimateMapping.FILE);
        m.write(f);
        ClimateMapping back = ClimateMapping.read(f);
        assertNotNull(back.peatland());
        assertEquals(p, back.peatland());
        assertNull(identity().peatland());
        assertFalse(back.meta().has(Peatland.KEY), "the verdict is not left in the meta");
        Files.delete(f);
        Files.delete(dir);
    }

    /** The shared tag lists the pack bogs the task names, each optional (a missing mod must not break the tag). */
    @Test
    void theSharedPeatlandTagListsTheKnownBogsAsOptional() throws Exception {
        try (var in = PeatlandTest.class.getResourceAsStream("/data/deeptime/tags/worldgen/biome/peatland.json")) {
            assertNotNull(in, "#deeptime:peatland is shipped");
            JsonObject tag = com.google.gson.JsonParser.parseReader(new java.io.InputStreamReader(in)).getAsJsonObject();
            assertFalse(tag.get("replace").getAsBoolean(), "other packs' support adds to it");
            List<String> ids = new ArrayList<>();
            for (var e : tag.getAsJsonArray("values")) {
                JsonObject v = e.getAsJsonObject();
                assertFalse(v.get("required").getAsBoolean(), v + " must be optional");
                ids.add(v.get("id").getAsString());
            }
            for (String want : List.of("biomesoplenty:bog", "biomesoplenty:muskeg", "expanded_ecosphere:fen", "expanded_ecosphere:snowy_bog"))
                assertTrue(ids.contains(want), want);
        }
    }
}
