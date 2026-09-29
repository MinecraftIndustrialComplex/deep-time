// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@link ClimateMapping} (P1-09): identity when the pack is the reference, edges onto edges for the
 * banded parameters, a quantile correspondence for erosion and weirdness, the file round trip.
 */
class ClimateMappingTest {

    static final OverworldBands REF = OverworldBands.defaults();

    static double[] refErosion() {
        return OverworldBands.array(OverworldBands.defaultDocument().getAsJsonObject("reference").getAsJsonArray("erosion"));
    }

    static double[] refPv() {
        return OverworldBands.array(OverworldBands.defaultDocument().getAsJsonObject("reference").getAsJsonArray("peaks_valleys"));
    }

    @Test
    void theJarCarriesTheBuilderSBands() {
        assertEquals(-0.45, REF.temperature()[1]);
        assertEquals(0.55, REF.temperature()[4]);
        assertEquals(0.3, REF.vegetation()[4]);
        assertEquals(-0.19, REF.continents()[3]);
        assertEquals(0.55, REF.erosion()[6]);
        assertEquals(0.05, REF.weirdness()[1]);
        assertEquals(65, refErosion().length);
        assertEquals(65, refPv().length);
    }

    @Test
    void thePackAsReferenceIsIdentity() {
        ClimateMapping m = ClimateMapping.build(REF, refErosion(), refPv(), REF, refErosion(), refPv(), new JsonObject());
        for (String layer : ClimateMapping.LAYERS)
            for (double v = -1.0; v <= 1.0; v += 0.0137)
                assertEquals(v, m.apply(layer, v), 1e-9, layer + " at " + v);
        assertEquals(0.123, m.apply("elevation", 0.123));
    }

    @Test
    void bandsMapEdgeToEdge() {
        double[] t = REF.temperature().clone();
        t[3] = 0.1;   // a pack whose warm band starts at 0.1
        OverworldBands pack = new OverworldBands(t, REF.vegetation(), REF.continents(), REF.erosion(), REF.weirdness());
        ClimateMapping m = ClimateMapping.build(REF, refErosion(), refPv(), pack, null, null, new JsonObject());
        assertEquals(0.1, m.apply(ClimateMapping.TEMPERATURE, 0.2), 1e-12);
        assertEquals(0.55, m.apply(ClimateMapping.TEMPERATURE, 0.55), 1e-12);
        // Mid-band stays mid-band.
        assertEquals(OverworldBands.coord(REF.temperature(), 0.375), OverworldBands.coord(t, m.apply(ClimateMapping.TEMPERATURE, 0.375)), 1e-9);
        assertEquals(-0.3, m.apply(ClimateMapping.TEMPERATURE, -0.3), 1e-12);
    }

    @Test
    void erosionFollowsThePackDistribution() {
        double[] ref = refErosion();
        double[] pack = new double[ref.length];
        for (int i = 0; i < pack.length; i++)
            pack[i] = Math.pow(i / (double) (pack.length - 1), 2) * 1.6 - 0.9;   // a skewed pack
        ClimateMapping m = ClimateMapping.build(REF, ref, refPv(), REF, pack, refPv(), new JsonObject());
        ClimateMapping.Quantile r = new ClimateMapping.Quantile(ref).below(0.55 - 1e-3);
        ClimateMapping.Quantile p = new ClimateMapping.Quantile(pack).below(0.55 - 1e-3);
        for (double u = 0.05; u < 1; u += 0.1)
            assertEquals(p.at(u), m.apply(ClimateMapping.EROSION, r.at(u)), 2e-3, "u " + u);
        // The swamp band keeps its edges; the table rises everywhere.
        assertEquals(0.55, m.apply(ClimateMapping.EROSION, 0.55), 1e-9);
        assertEquals(1.0, m.apply(ClimateMapping.EROSION, 1.0), 1e-9);
        double prev = -2;
        for (double v = -1; v <= 1; v += 0.01) {
            double o = m.apply(ClimateMapping.EROSION, v);
            assertTrue(o >= prev - 1e-12, "monotone at " + v);
            prev = o;
        }
    }

    @Test
    void weirdnessIsOddAndStaysOutOfTheValleySlice() {
        double[] pack = new double[65];
        for (int i = 0; i < pack.length; i++)
            pack[i] = -1 + 2 * Math.sqrt(i / 64.0);
        ClimateMapping m = ClimateMapping.build(REF, refErosion(), refPv(), REF, refErosion(), pack, new JsonObject());
        for (double w = 0.07; w < 0.66; w += 0.05) {
            assertEquals(-m.apply(ClimateMapping.RIDGES, w), m.apply(ClimateMapping.RIDGES, -w), 1e-12);
            assertTrue(Math.abs(m.apply(ClimateMapping.RIDGES, w)) > 0.05, "no valley slice at " + w);
        }
    }

    @Test
    void theFileRoundTrips() throws Exception {
        JsonObject meta = new JsonObject();
        meta.addProperty("why", "test");
        ClimateMapping m = ClimateMapping.build(REF, refErosion(), refPv(), REF, null, null, meta);
        Path dir = Files.createTempDirectory("climate-mapping");
        Path f = dir.resolve(ClimateMapping.FILE);
        m.write(f);
        ClimateMapping back = ClimateMapping.read(f);
        for (String layer : ClimateMapping.LAYERS)
            for (double v = -1; v <= 1; v += 0.1)
                assertEquals(m.apply(layer, v), back.apply(layer, v), 1e-12);
        assertEquals("test", back.meta().get("why").getAsString());
        Files.delete(f);
        Files.delete(dir);
    }
}
