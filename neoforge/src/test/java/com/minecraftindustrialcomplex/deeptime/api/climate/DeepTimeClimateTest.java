// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.climate;

import com.minecraftindustrialcomplex.deeptime.worlddata.ClimateLayers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import org.junit.jupiter.api.Test;

import java.nio.file.Path;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The public climate API (P1-25) on the real test planet ({@code planet4k}, Gradle
 * {@code testWorldPlanet4k}): it answers what {@link ClimateLayers} does (through its cache too), the
 * year curve passes through the months and averages to the annual mean, the lapse rate applies at the
 * block's height, precipitation adds up, and the seasons are opposite across the equator.
 */
class DeepTimeClimateTest {

    private static Path world() {
        String d = System.getProperty("deeptime.testWorlds");
        if (d == null)
            throw new IllegalStateException("system property deeptime.testWorlds is unset: run through Gradle's test task");
        return Path.of(d).resolve("planet4k");
    }

    @Test
    void theApiAnswersTheLayersThroughItsCache() {
        WorldData wd = WorldData.open(world());
        ClimateLayers cl = ClimateLayers.of(wd);
        ClimateColumns api = new ClimateColumns(cl);
        int c = wd.circumference();
        Random rng = new Random(25);
        for (int i = 0; i < 300; i++) {
            int zp = (int) com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Lambert.radius(c);
            int x = rng.nextInt(2 * c) - c, z = rng.nextInt(2 * zp) - zp, y = rng.nextInt(400) - 60;
            for (int pass = 0; pass < 2; pass++) {
                ClimateSample s = api.sample(x, y, z);
                ClimateLayers.Sample r = cl.sample(x, y, z);
                for (int m = 0; m < 12; m++) {
                    assertEquals(r.tMonth()[m], s.monthC(m), 0.0);
                    assertEquals(r.pMonth()[m], s.monthPrecipitationMm(m), 0.0);
                }
                assertEquals(r.tMean(), s.meanC(), 0.0);
                assertEquals(r.koppenCode(), s.koppen());
            }
        }
    }

    @Test
    void theYearCurveLapseAndPrecipitationAreConsistent() {
        ClimateLayers cl = ClimateLayers.of(WorldData.open(world()));
        ClimateColumns api = new ClimateColumns(cl);
        int sea = cl.seaLevelY();
        Random rng = new Random(7);
        int northWarmJuly = 0, southWarmJanuary = 0, hemi = 0;
        for (int i = 0; i < 200; i++) {
            int x = rng.nextInt(4096) - 2048, z = rng.nextInt(2048) - 1024;
            ClimateSample s = api.sample(x, sea + 40, z);
            // Through the months' midpoints, averaging to the mean.
            double sum = 0;
            for (int m = 0; m < 12; m++) {
                assertEquals(s.monthC(m), s.temperatureC((m + 0.5) / 12.0), 1e-9);
                sum += s.temperatureC((m + 0.5) / 12.0);
            }
            assertEquals(s.meanC(), sum / 12.0, 1e-9);
            assertTrue(s.coldestMonthC() <= s.meanC() && s.meanC() <= s.warmestMonthC());
            assertEquals(s.warmestMonthC(), s.monthC(s.warmestMonth()), 0.0);
            // Periodic.
            assertEquals(s.temperatureC(0.2), s.temperatureC(1.2), 1e-9);
            assertEquals(s.temperatureC(0.2), s.temperatureC(-0.8), 1e-9);
            // Lapse: 800 m higher (40 blocks × 20 m), 6.5 °C/km colder.
            ClimateSample low = api.sample(x, sea, z);
            assertEquals(0.0, low.elevationMetres());
            assertEquals(low.meanC() - s.meanC(), cl.lapseCPerKm() * s.elevationMetres() / 1000.0, 1e-9);
            assertEquals(low.monthC(3), s.seaLevelMonthC(3), 1e-9);
            // Below the sea: no lapse, no warming.
            assertEquals(low.meanC(), api.sample(x, sea - 50, z).meanC(), 0.0);
            // Precipitation adds up.
            double p = 0;
            for (int m = 0; m < 12; m++)
                p += s.monthPrecipitationMm(m);
            assertEquals(s.annualPrecipitationMm(), p, 1e-6 * Math.max(1, p));
            assertTrue(s.precipitationSeasonality() >= 0);
            // Seasons follow the hemisphere (away from the tropics).
            if (Math.abs(z) > 400 && s.warmestMonthC() - s.coldestMonthC() > 3) {
                hemi++;
                boolean julyish = s.warmestMonth() >= 5 && s.warmestMonth() <= 8;
                boolean januaryish = s.warmestMonth() <= 2 || s.warmestMonth() == 11;
                // z grows southward in Minecraft: negative z is the north.
                if (z < 0 && julyish)
                    northWarmJuly++;
                if (z > 0 && januaryish)
                    southWarmJanuary++;
            }
        }
        System.out.println("[junit] DeepTimeClimateTest hemisphere samples=" + hemi + " northJuly=" + northWarmJuly
                + " southJanuary=" + southWarmJanuary);
        assertTrue(hemi > 20, "enough seasonal samples: " + hemi);
        assertTrue(northWarmJuly + southWarmJanuary >= 0.9 * hemi,
                "warmest month in the local summer: " + (northWarmJuly + southWarmJanuary) + " of " + hemi);
    }
}
