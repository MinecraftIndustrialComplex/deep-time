// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import net.minecraft.world.level.biome.Climate;
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
 * {@link Peatland} (BIO-106): cold wetland is taiga unless the pack's own biome source answers with a
 * peatland biome at the swamp row of the cold band, region by region.
 */
class PeatlandTest {

    static final OverworldBands REF = OverworldBands.defaults();

    static ClimateMapping identity() {
        return ClimateMapping.build(REF, ClimateMappingTest.refErosion(), ClimateMappingTest.refPv(), REF, null, null, new JsonObject());
    }

    static double t(Climate.TargetPoint p) {
        return Climate.unquantizeCoord(p.temperature());
    }

    static double e(Climate.TargetPoint p) {
        return Climate.unquantizeCoord(p.erosion());
    }

    /** A pack whose region "b" has a bog (cold, swamp row E6) and region "a" only a swamp that is not peatland. */
    static final Peatland.Probe PACK = (region, p) -> {
        boolean swampRow = e(p) >= 0.55;
        double temp = t(p);
        if (region.equals("b") && swampRow && temp >= -0.45 && temp < -0.15)
            return "mod:bog";
        return null;   // vanilla's swamp and the like: not peatland
    };

    @Test
    void aPackWithABogInOneRegionHasPeatlandThere() {
        Peatland p = Peatland.decide(identity(), REF, List.of("a", "b"), 2.0, PACK);
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
        Peatland p = Peatland.decide(identity(), REF, List.of(Peatland.NO_TABLE), 2.0, (r, pt) -> null);
        assertTrue(p.regions().isEmpty());
        assertTrue(p.biomes().isEmpty());
    }

    @Test
    void aBogOnlyInTheFrozenOrTemperateBandIsNotColdWetland() {
        Peatland.Probe frozen = (r, p) -> e(p) >= 0.55 && t(p) < -0.45 ? "mod:muskeg" : null;
        Peatland.Probe temperate = (r, p) -> e(p) >= 0.55 && t(p) >= -0.15 ? "mod:fen" : null;
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), 2.0, frozen).regions().isEmpty());
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), 2.0, temperate).regions().isEmpty());
    }

    @Test
    void aBogThatAnswersAtTooFewPointsDoesNotCount() {
        // Peatland at the driest humidities of the wet range only (under a fifth of the points).
        Peatland.Probe few = (r, p) -> e(p) >= 0.55 && Climate.unquantizeCoord(p.humidity()) < 0.05 ? "mod:bog" : null;
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), 2.0, few).regions().isEmpty());
        // Under half of the points (one weirdness sign of two) is not a peatland region either.
        Peatland.Probe half = (r, p) -> e(p) >= 0.55 && Climate.unquantizeCoord(p.weirdness()) > 0.3 ? "mod:bog" : null;
        assertTrue(Peatland.decide(identity(), REF, List.of("b"), 2.0, half).regions().isEmpty());
    }

    @Test
    void theKnobThatTurnsColdWetlandOffTurnsPeatlandOff() {
        Peatland p = Peatland.decide(identity(), REF, List.of("b"), 1.0, PACK);
        assertTrue(p.regions().isEmpty(), "no cold band, no peatland");
    }

    @Test
    void theProbePointsAreColdWetClimatesAtTheSwampRow() {
        List<Climate.TargetPoint> seen = new ArrayList<>();
        Peatland.decide(identity(), REF, List.of("b"), 2.0, (r, p) -> {
            seen.add(p);
            return null;
        });
        assertFalse(seen.isEmpty());
        for (Climate.TargetPoint p : seen) {
            assertTrue(t(p) >= -0.45 && t(p) < -0.15, "temperature in the cold band: " + t(p));
            assertTrue(e(p) >= 0.55 && e(p) <= 1.0, "erosion in E6: " + e(p));
            assertTrue(Climate.unquantizeCoord(p.humidity()) >= -0.1, "wet: " + Climate.unquantizeCoord(p.humidity()));
            assertTrue(Climate.unquantizeCoord(p.continentalness()) > -0.11, "inland");
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
}
