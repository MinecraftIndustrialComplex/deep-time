// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.minecraftindustrialcomplex.deeptime.worlddata.ClimateLayers;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;

import java.nio.file.Path;
import java.util.Map;
import java.util.TreeMap;

/**
 * A measuring tool, not a check: the cold deserts of a baked world (BWk land whose baked temperature parameter is below
 * the cold band's top, {@link BiomeCensus#COLD_BAND_TOP}, the columns {@code BWk:cold_not_sand} counts) against the
 * mean temperature a biome rule reads there (the column's months lapsed to its own height, {@link ClimateLayers#sample},
 * as {@code BiomeRules.Column} does). Runs only when {@code DEEPTIME_CENSUS_WORLD} names a world-data folder; prints
 * to stdout.
 *
 * <pre>DEEPTIME_CENSUS_WORLD=&lt;folder&gt; ./gradlew test --tests '*ColdDesertCensusTest' -i</pre>
 */
class ColdDesertCensusTest {

    @Test
    @EnabledIfEnvironmentVariable(named = "DEEPTIME_CENSUS_WORLD", matches = ".+")
    void census() {
        WorldData wd = WorldData.open(Path.of(System.getenv("DEEPTIME_CENSUS_WORLD")));
        int c = wd.circumference(), sea = wd.manifest().seaLevelY(), step = Integer.getInteger("deeptime.step", 4);
        double mV = wd.manifest().mPerBlockV();
        ClimateLayers cl = ClimateLayers.of(wd);
        RasterSampler kp = wd.layer(wd.has("climate/koppen_l2") ? "climate/koppen_l2" : "climate/koppen");
        RasterSampler temp = wd.layer("climate/temperature");
        RasterSampler mean = wd.has("climate/t_mean_c") ? wd.layer("climate/t_mean_c") : null;
        int pole = (int) wd.projection().zMax();
        System.out.println("CENSUS cold deserts C=" + c + " zHalf=" + pole + " step=" + step + " koppen layer="
                + (wd.has("climate/koppen_l2") ? "koppen_l2" : "koppen") + " lapse=" + cl.lapseCPerKm() + " mV=" + mV);
        long bwk = 0, cold = 0, coldWarmRule = 0, coldWarmFlat = 0, coldNearestWarm = 0, warm = 0, warmColdRule = 0;
        Map<Integer, Integer> byExcess = new TreeMap<>();
        Map<Integer, Integer> byDetail = new TreeMap<>();
        double[] sumDetail = new double[2];
        long[] nDetail = new long[2];
        int printed = 0;
        for (int z = -pole + step / 2; z < pole; z += step)
            for (int x = -c / 2 + step / 2; x < c / 2; x += step) {
                double kv = kp.nearest(x, z);
                if (Double.isNaN(kv) || Math.round(kv) != 5)
                    continue;
                double y = wd.height(x, z);
                if (!(y > sea))
                    continue;
                bwk++;
                double tParam = temp.bilinear(x, z);
                double tRule = cl.sample(x, (int) Math.floor(y), z).tMean();
                // The same months at the elevation layer's height: the ground without the detail term.
                double tFlat = cl.sample(x, (int) Math.floor(wd.elevation(x, z)), z).tMean();
                double detail = y - wd.elevation(x, z);
                if (tParam < BiomeCensus.COLD_BAND_TOP) {
                    cold++;
                    if (temp.nearest(x, z) >= BiomeCensus.COLD_BAND_TOP)
                        coldNearestWarm++;
                    if (tRule >= 10.0) {
                        coldWarmRule++;
                        byExcess.merge((int) Math.floor(tRule - 10.0), 1, Integer::sum);
                        sumDetail[0] += detail;
                        nDetail[0]++;
                        byDetail.merge((int) Math.floor(detail / 4.0) * 4, 1, Integer::sum);
                        if (printed++ < 40)
                            System.out.println(String.format(
                                    "CENSUS cold-warm x=%d z=%d y=%.1f elev=%.1f tParam=%.3f tParamNearest=%.3f tRule=%.2f tFlat=%.2f tMeanLayer=%.2f",
                                    x, z, y, wd.elevation(x, z), tParam, temp.nearest(x, z), tRule, tFlat,
                                    mean == null ? Double.NaN : mean.bilinear(x, z)));
                    } else {
                        sumDetail[1] += detail;
                        nDetail[1]++;
                    }
                    if (tFlat >= 10.0)
                        coldWarmFlat++;
                } else {
                    warm++;
                    if (tRule < 10.0)
                        warmColdRule++;
                }
            }
        System.out.println("CENSUS BWk land=" + bwk + " cold(tParam<" + BiomeCensus.COLD_BAND_TOP + ")=" + cold + " of them ruleMean>=10: "
                + coldWarmRule + " flatMean>=10: " + coldWarmFlat + " nearestParamWarm: " + coldNearestWarm
                + " | warm=" + warm + " of them ruleMean<10: " + warmColdRule);
        System.out.println("CENSUS cold with ruleMean>=10 by floor(ruleMean-10): " + byExcess);
        System.out.println("CENSUS cold with ruleMean>=10 by detail (height-elevation, 4-block bins): " + byDetail);
        System.out.println(String.format("CENSUS mean detail: cold-warm %.2f blocks (n=%d), cold-cold %.2f blocks (n=%d)",
                nDetail[0] == 0 ? Double.NaN : sumDetail[0] / nDetail[0], nDetail[0],
                nDetail[1] == 0 ? Double.NaN : sumDetail[1] / nDetail[1], nDetail[1]));
    }
}
