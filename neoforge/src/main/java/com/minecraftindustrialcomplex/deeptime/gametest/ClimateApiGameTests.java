// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.api.climate.ClimateSample;
import com.minecraftindustrialcomplex.deeptime.api.climate.DeepTimeClimate;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Deep Time's public climate API (P1-25) in a running server: on the real baked planet
 * ({@code deeptime:test_planet_real}) {@link DeepTimeClimate#of} answers every block of Rust's
 * climate fixture ({@code parity/climate.json}, written by {@code deeptime parity} in gametest.sh)
 * exactly, the tropics are warmer than the high latitudes and the lapse rate applies with height;
 * on a synthetic world-data level it reports no climate, and on a level Deep Time did not generate
 * it reports nothing at all.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class ClimateApiGameTests {

    private ClimateApiGameTests() {}

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void climateApiOnTheRealPlanet(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        WorldDataTerrain t = RealPlanetGameTests.terrain(helper, level);
        DeepTimeClimate climate = DeepTimeClimate.of(level);
        helper.assertTrue(climate.isDeepTimeLevel(), "the real planet is a Deep Time level");
        helper.assertTrue(climate.hasClimate(), "the real planet has the simulated climate");
        helper.assertTrue(DeepTimeClimate.of(level) == climate, "one object per world");

        // Rust's evaluator, block for block.
        Path fixture = t.data().folder().resolve("parity/climate.json");
        JsonObject doc;
        try {
            doc = JsonParser.parseString(Files.readString(fixture, StandardCharsets.UTF_8)).getAsJsonObject();
        } catch (IOException e) {
            helper.fail("cannot read " + fixture + ": " + e.getMessage());
            return;
        }
        JsonArray samples = doc.getAsJsonArray("samples");
        int equal = 0;
        String firstBad = null;
        for (JsonElement e : samples) {
            JsonObject r = e.getAsJsonObject();
            int x = r.get("x").getAsInt(), y = r.get("y").getAsInt(), z = r.get("z").getAsInt();
            ClimateSample s = climate.at(new BlockPos(x, y, z)).orElse(null);
            if (s == null) {
                helper.fail("no climate at " + x + " " + y + " " + z);
                return;
            }
            JsonArray tm = r.getAsJsonArray("t_month"), pm = r.getAsJsonArray("p_month");
            boolean same = s.meanC() == r.get("t_mean").getAsDouble()
                    && s.warmestMonthC() == r.get("t_warm").getAsDouble()
                    && s.coldestMonthC() == r.get("t_cold").getAsDouble()
                    && s.annualPrecipitationMm() == r.get("p_annual").getAsDouble()
                    && s.koppen().equals(r.get("koppen_code").getAsString());
            for (int m = 0; m < 12 && same; m++)
                same = s.monthC(m) == tm.get(m).getAsDouble() && s.monthPrecipitationMm(m) == pm.get(m).getAsDouble();
            if (same)
                equal++;
            else if (firstBad == null)
                firstBad = s + " vs " + r;
        }

        // Physical sense: the tropics (land and sea, at the surface) are warmer than the high
        // latitudes, and 50 blocks up is colder everywhere (at sea level the lapse starts at 0, so
        // this holds over the sea too). The small test planet may have no tropical land at all.
        int c = t.circumference(), sea = level.getSeaLevel();
        double tropics = 0, high = 0;
        int nt = 0, nh = 0, land = 0, lapsed = 0, n = 400;
        for (int i = 0; i < n; i++) {
            int x = -c / 2 + (i * 97) % c;
            int zp = (int) t.zHalf();   // the poles (Lambert) or the turning edges (Petroff–Guyou)
            int z = ((i * 61) % (2 * zp)) - zp;
            double lat = DeepTimeClimate.latitudeDeg(level, x, z);
            int y = Math.max((int) Math.ceil(t.height(x, z)), sea);
            ClimateSample s = climate.at(x, y, z).orElseThrow();
            if (!s.isSea())
                land++;
            if (Math.abs(lat) < 15) {
                tropics += s.meanC();
                nt++;
            } else if (Math.abs(lat) > 55) {
                high += s.meanC();
                nh++;
            }
            if (climate.at(x, y + 50, z).orElseThrow().meanC() < s.meanC())
                lapsed++;
        }
        DeepTime.LOGGER.info("[gametest] climateApiOnTheRealPlanet fixture={} equal={} tropics={} meanC={} "
                        + "highLat={} meanC={} land={} colderHigher={} of {} firstBad={}",
                samples.size(), equal, nt, nt == 0 ? Double.NaN : tropics / nt, nh, nh == 0 ? Double.NaN : high / nh,
                land, lapsed, n, firstBad);
        helper.assertTrue(samples.size() == 500, "the fixture has " + samples.size() + " samples");
        helper.assertTrue(equal == samples.size(), (samples.size() - equal) + " samples differ from Rust's: " + firstBad);
        helper.assertTrue(nt > 5 && nh > 5, "samples: tropics " + nt + ", high latitudes " + nh);
        helper.assertTrue(tropics / nt > high / nh + 10, "tropics " + tropics / nt + " vs high latitudes " + high / nh);
        helper.assertTrue(lapsed == n, "50 blocks up is colder at " + lapsed + " of " + n + " samples");
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 200)
    public static void climateApiIsEmptyElsewhere(GameTestHelper helper) {
        ServerLevel synthetic = WorldDataGameTests.level(helper);
        DeepTimeClimate s = DeepTimeClimate.of(synthetic);
        helper.assertTrue(s.isDeepTimeLevel(), "the synthetic level is a Deep Time level");
        helper.assertTrue(!s.hasClimate() && s.at(BlockPos.ZERO).isEmpty(), "a synthetic world has no simulated climate");
        ServerLevel overworld = helper.getLevel().getServer().overworld();
        DeepTimeClimate o = DeepTimeClimate.of(overworld);
        boolean deep = overworld.getChunkSource().getGenerator() instanceof GeologicChunkGenerator;
        helper.assertTrue(o.isDeepTimeLevel() == deep, "the overworld is " + (deep ? "" : "not ") + "Deep Time's");
        if (!deep)
            helper.assertTrue(o.at(BlockPos.ZERO).isEmpty() && Double.isNaN(o.lapseRateCPerKm()), "nothing outside Deep Time");
        // API 2: the planet's geometry, on the server from the generator (the synthetic level is a planet
        // too; a level Deep Time did not generate is not).
        int c = DeepTimeClimate.circumferenceBlocks(synthetic);
        int expected = synthetic.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g ? g.dataRef().circumferenceBlocks() : -1;
        helper.assertTrue(c > 0 && c == expected, "the synthetic planet's circumference " + c + " vs its generator's " + expected);
        helper.assertTrue(deep || DeepTimeClimate.circumferenceBlocks(overworld) == 0, "a level Deep Time did not generate has no circumference");
        helper.assertTrue(DeepTimeClimate.circumferenceBlocks(null) == 0, "no level, no circumference");
        double pole = com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c);
        helper.assertTrue(DeepTimeClimate.latitudeDeg(-pole, c) == 90.0 && Math.abs(DeepTimeClimate.latitudeDeg(pole / 2, c) + 30.0) < 1e-9
                        && DeepTimeClimate.latitudeDeg(-c / 4.0, c) == 90.0
                        && DeepTimeClimate.latitudeDeg(0, c) == 0.0 && Double.isNaN(DeepTimeClimate.latitudeDeg(100, 0)),
                "latitude: north is -z, sin φ = -z/R (Lambert), the poles at |z| = R = C/2π");
        // The year convention: month 0 starts at solar longitude 280°.
        helper.assertTrue(Math.abs(DeepTimeClimate.yearFractionAtSolarLongitude(0) - DeepTimeClimate.NORTHERN_SPRING_EQUINOX) < 1e-12
                        && Math.abs(DeepTimeClimate.yearFractionAtSolarLongitude(90) - DeepTimeClimate.NORTHERN_SUMMER_SOLSTICE) < 1e-12
                        && DeepTimeClimate.yearFractionAtSolarLongitude(280) == 0.0
                        && DeepTimeClimate.yearFractionAtSolarLongitude(279.9) > 0.99,
                "year fractions of solar longitudes");
        DeepTime.LOGGER.info("[gametest] climateApiIsEmptyElsewhere syntheticHasClimate={} overworldDeepTime={} api={} syntheticC={}",
                s.hasClimate(), deep, DeepTimeClimate.API_VERSION, c);
        helper.succeed();
    }
}
