// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worldgen.AnalyticTestTerrain;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * P0-09: the Java reader returns exactly what the Rust writer wrote. {@code deeptime synth
 * --dump-samples} lists the raw value of every cell of the first and last tile plus 1,000 random
 * cells of {@code elevation}, {@code water_level} and {@code climate/temperature}, bilinear samples
 * at random fractional block positions (across the seam and past the poles), and random horizon
 * stacks; this test reads the same world data through {@link WorldData} and compares.
 */
class TilePackParityTest {

    private static final int MAX_REPORTED = 10;

    @ParameterizedTest
    @ValueSource(strings = {"wrap1024", "lake1024"})
    void cellsMatchTheWriter(String fixture) {
        WorldData wd = WorldData.open(Fixtures.world(fixture));
        JsonObject dump = Fixtures.samples(fixture);
        assertEquals(FormatVersion.CURRENT, wd.manifest().formatVersion());
        assertEquals(dump.get("circumference").getAsInt(), wd.circumference());
        wd.verify();

        List<String> bad = new ArrayList<>();
        int checked = 0;
        for (Map.Entry<String, JsonElement> e : dump.getAsJsonObject("layers").entrySet()) {
            RasterSampler s = wd.layer(e.getKey());
            for (JsonElement c : e.getValue().getAsJsonObject().getAsJsonArray("cells")) {
                JsonArray a = c.getAsJsonArray();
                int i = a.get(0).getAsInt(), j = a.get(1).getAsInt();
                long raw = a.get(2).getAsLong();
                double got = s.rawCell(i, j);
                double phys = s.get(i, j);
                double wantPhys = s.desc().physical(raw);
                boolean ok = got == raw && (Double.isNaN(wantPhys) ? Double.isNaN(phys) : phys == wantPhys);
                if (!ok && bad.size() < MAX_REPORTED)
                    bad.add(e.getKey() + " (" + i + ", " + j + "): rust " + raw + ", java " + got);
                checked++;
            }
        }
        assertTrue(bad.isEmpty(), fixture + ": cell mismatches " + bad);
        // Every cell of the first and last tile plus 1,000 random cells, for three layers.
        assertTrue(checked > 3 * 1000, "checked only " + checked + " cells");

        for (Map.Entry<String, JsonElement> e : dump.getAsJsonObject("bilinear").entrySet()) {
            RasterSampler s = wd.layer(e.getKey());
            for (JsonElement p : e.getValue().getAsJsonArray()) {
                JsonArray a = p.getAsJsonArray();
                double bx = a.get(0).getAsDouble(), bz = a.get(1).getAsDouble(), want = a.get(2).getAsDouble();
                double got = s.bilinear(bx, bz);
                if (got != want && bad.size() < MAX_REPORTED)
                    bad.add(e.getKey() + " bilinear(" + bx + ", " + bz + "): rust " + want + ", java " + got);
            }
        }
        assertTrue(bad.isEmpty(), fixture + ": bilinear mismatches " + bad);

        JsonObject hz = dump.getAsJsonObject("horizons");
        RasterSampler h = wd.layer(WorldData.HORIZONS);
        int k = hz.get("stack_depth").getAsInt();
        assertEquals(k, h.stackDepth());
        for (JsonElement c : hz.getAsJsonArray("cells")) {
            JsonArray a = c.getAsJsonArray();
            int i = a.get(0).getAsInt(), j = a.get(1).getAsInt();
            for (int n = 0; n < k; n++) {
                short top = a.get(2).getAsJsonArray().get(n).getAsShort();
                int cls = a.get(3).getAsJsonArray().get(n).getAsInt();
                if ((h.horizonTopRaw(i, j, n) != top || h.horizonClass(i, j, n) != cls) && bad.size() < MAX_REPORTED)
                    bad.add("horizon " + n + " at (" + i + ", " + j + ")");
            }
        }
        assertTrue(bad.isEmpty(), fixture + ": horizon mismatches " + bad);
    }

    /**
     * The wrap-test world data is the analytic terrain written as layers: at every L3 sample point
     * the elevation is H rounded to 1/16 block, and the L2 climate layers are the analytic rasters
     * rounded to 1/10,000. This is the Rust transcription of {@link AnalyticTestTerrain} checked
     * against the Java original, and the baseline P0-10 compares generated terrain against.
     */
    @Test
    void wrapTestIsTheAnalyticTerrain() {
        WorldData wd = WorldData.open(Fixtures.world("wrap1024"));
        AnalyticTestTerrain t = new AnalyticTestTerrain(wd.circumference());
        RasterSampler e = wd.layer(WorldData.ELEVATION);
        GridDesc g = e.desc().grid();
        double worst = 0;
        for (int j = 0; j < g.h(); j++)
            for (int i = 0; i < g.w(); i++) {
                int x = (int) g.blockX(i), z = (int) g.blockZ(j);
                worst = Math.max(worst, Math.abs(wd.height(x, z) - t.height(x, z)));
            }
        assertTrue(worst <= 1.0 / 32 + 1e-9, "elevation differs from AnalyticTestTerrain by " + worst);

        for (String layer : List.of("climate/temperature", "climate/vegetation", "climate/continents",
                "climate/erosion", "climate/ridges")) {
            RasterSampler s = wd.layer(layer);
            GridDesc cg = s.desc().grid();
            double w = 0;
            for (int j = 0; j < cg.h(); j++)
                for (int i = 0; i < cg.w(); i++) {
                    int x = (int) cg.blockX(i), z = (int) cg.blockZ(j);
                    w = Math.max(w, Math.abs(wd.raster(layer, x, z) - t.raster(layer, x, z)));
                }
            assertTrue(w <= 0.5e-4 + 1e-9, layer + " differs from AnalyticTestTerrain by " + w);
        }
    }

    @Test
    void lakeTestHoldsItsBasin() {
        WorldData wd = WorldData.open(Fixtures.world("lake1024"));
        assertEquals(110.0, wd.height(0, 0));
        assertEquals(120.0, wd.waterLevel(0, 0));
        assertEquals(110.0, wd.height(100, 0));   // the channel
        assertEquals(120.0, wd.waterLevel(100, 0));
        assertEquals(125.0, wd.height(0, 48));    // the rim crest
        assertTrue(Double.isNaN(wd.waterLevel(0, 48)));
        assertEquals(100.0, wd.height(0, 200));   // the plain
        JsonObject lake = wd.manifest().notes().getAsJsonObject("lake");
        assertEquals(120.0, lake.get("water_level_y").getAsDouble());
    }
}
