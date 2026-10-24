// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.api.climate.ClimateSample;
import com.minecraftindustrialcomplex.deeptime.api.climate.DeepTimeClimate;
import com.minecraftindustrialcomplex.deeptime.api.planet.DeepTimePlanet;
import com.minecraftindustrialcomplex.deeptime.api.planet.LocalFrame;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
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
 * it reports nothing at all. The planet's geometry (API 2, 4 and 5) on a Lambert-frame level and
 * the cube world: latitude, longitude, north's bearing and the metres per block.
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
            int zp = (int) t.zHalf();   // the net's outer edges on the cube (3s/2), the poles on a Lambert-frame world
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
        // API 5 on an equal-area world: north is -z along the equator, the scale is the nominal one.
        String kind = t.data().projection().kind().id();
        double nominal = t.data().manifest().mPerBlockH(), worstNorth = 0;
        int scaled = 0;
        for (int i = 0; i < 40; i++) {
            double x = -c / 2.0 + i * c / 40.0 + 13.5, z = ((i * 61) % (2 * (int) t.zHalf())) - t.zHalf();
            double b = DeepTimeClimate.northBearingDeg(level, x, 0.0);
            worstNorth = Math.max(worstNorth, Math.min(b, 360.0 - b));
            if (DeepTimeClimate.metresPerBlock(level, x, z) == nominal)
                scaled++;
        }
        helper.assertTrue(kind.equals("petroff_guyou") || kind.equals("lambert"), "the real planet's projection: " + kind);
        // The planet API on the real planet: one object per world, the equal-area sphere's radius, and
        // the climate API's geometry its answers.
        DeepTimePlanet planet = DeepTimePlanet.of(level).orElse(null);
        double radius = t.data().projection().radius() * nominal;
        helper.assertTrue(planet != null && DeepTimePlanet.of(level).orElseThrow() == planet
                        && Math.abs(planet.radiusMetres() - radius) <= 1e-9 * radius,
                "the real planet's sphere: " + planet + " vs R " + radius + " m");
        int same = 0;
        for (int i = 0; i < 40; i++) {
            double x = -c / 2.0 + i * c / 40.0 + 7.5, z = ((i * 37) % (2 * (int) t.zHalf())) - t.zHalf() + 0.5;
            LocalFrame f = planet.frame(x, z).orElseThrow();
            if (planet.latitudeDeg(x, z) == DeepTimeClimate.latitudeDeg(level, x, z) && planet.longitudeDeg(x, z) == DeepTimeClimate.longitudeDeg(level, x, z)
                    && f.northBearingDeg() == DeepTimeClimate.northBearingDeg(level, x, z) && f.metresPerBlock() == DeepTimeClimate.metresPerBlock(level, x, z))
                same++;
        }
        helper.assertTrue(same == 40, "the climate API's geometry is the planet API's at " + same + " of 40");
        helper.assertTrue(worstNorth < 0.1, "north on the equator is -z within " + worstNorth + "°");
        helper.assertTrue(scaled == 40 && nominal > 0, "the equal-area scale is the nominal " + nominal + " at " + scaled + " of 40");
        DeepTime.LOGGER.info("[gametest] climateApiOnTheRealPlanet fixture={} equal={} tropics={} meanC={} "
                        + "highLat={} meanC={} land={} colderHigher={} of {} firstBad={} kind={} worstNorthOnTheEquator={} metresPerBlock={}",
                samples.size(), equal, nt, nt == 0 ? Double.NaN : tropics / nt, nh, nh == 0 ? Double.NaN : high / nh,
                land, lapsed, n, firstBad, kind, worstNorth, nominal);
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
        // The planet API and the climate API's geometry through it, on the server from the generator (the
        // synthetic level is a planet too; a level Deep Time did not generate is not).
        GeologicChunkGenerator sg = synthetic.getChunkSource().getGenerator() instanceof GeologicChunkGenerator gen ? gen : null;
        int c = sg == null ? -1 : sg.dataRef().circumferenceBlocks();
        double nominal = sg != null && sg.terrain() instanceof WorldDataTerrain wt ? wt.data().manifest().mPerBlockH() : Double.NaN;
        DeepTimePlanet planet = DeepTimePlanet.of(synthetic).orElse(null);
        helper.assertTrue(c > 0 && planet != null && DeepTimePlanet.of(synthetic).orElseThrow() == planet
                        && Math.abs(planet.radiusMetres() - c / (2 * Math.PI) * nominal) <= 1e-9 * planet.radiusMetres(),
                "the synthetic planet: C " + c + ", " + planet + " at " + nominal + " m per block");
        helper.assertTrue(deep || DeepTimePlanet.of(overworld).isEmpty(), "a level Deep Time did not generate is not a planet");
        helper.assertTrue(DeepTimePlanet.of(null).isEmpty(), "no level, no planet");
        // The synthetic planet is a Lambert world: latitude from z, north -z, the nominal scale (API 5),
        // nothing past the poles.
        String kind = sg == null ? "" : sg.dataRef().projection();
        helper.assertTrue(kind.equals("lambert"), "the synthetic planet is a Lambert world: " + kind);
        double pole = com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Lambert.radius(c);
        helper.assertTrue(DeepTimeClimate.latitudeDeg(synthetic, 17, -pole) == 90.0
                        && Math.abs(DeepTimeClimate.latitudeDeg(synthetic, -300, pole / 2) + 30.0) < 1e-9
                        && Double.isNaN(DeepTimeClimate.latitudeDeg(synthetic, 5, -c / 4.0)) && !planet.isOnPlanet(5, -c / 4.0)
                        && DeepTimeClimate.latitudeDeg(synthetic, 0, 0) == 0.0
                        && Double.isNaN(DeepTimeClimate.latitudeDeg(overworld, 0, 100)) == !deep,
                "latitude: north is -z, sin φ = -z/R (Lambert), the poles at |z| = R = C/2π, off the planet past them");
        for (double[] q : new double[][]{{0, 0}, {-400, 100}, {300, -150}}) {
            helper.assertTrue(DeepTimeClimate.northBearingDeg(synthetic, q[0], q[1]) == 0.0,
                    "north is -z on a Lambert world: " + DeepTimeClimate.northBearingDeg(synthetic, q[0], q[1]));
            helper.assertTrue(DeepTimeClimate.metresPerBlock(synthetic, q[0], q[1]) == nominal && nominal > 0,
                    "a Lambert world's scale is the nominal one: " + DeepTimeClimate.metresPerBlock(synthetic, q[0], q[1]) + " vs " + nominal);
        }
        helper.assertTrue(deep || Double.isNaN(DeepTimeClimate.northBearingDeg(overworld, 0, 0))
                        && Double.isNaN(DeepTimeClimate.metresPerBlock(overworld, 0, 0)) && Double.isNaN(DeepTimeClimate.metresPerBlock(null, 0, 0)),
                "no bearing and no scale off a planet");
        // The year convention: month 0 starts at solar longitude 280°.
        helper.assertTrue(Math.abs(DeepTimeClimate.yearFractionAtSolarLongitude(0) - DeepTimeClimate.NORTHERN_SPRING_EQUINOX) < 1e-12
                        && Math.abs(DeepTimeClimate.yearFractionAtSolarLongitude(90) - DeepTimeClimate.NORTHERN_SUMMER_SOLSTICE) < 1e-12
                        && DeepTimeClimate.yearFractionAtSolarLongitude(280) == 0.0
                        && DeepTimeClimate.yearFractionAtSolarLongitude(279.9) > 0.99,
                "year fractions of solar longitudes");
        DeepTime.LOGGER.info("[gametest] climateApiIsEmptyElsewhere syntheticHasClimate={} overworldDeepTime={} api={} syntheticC={} metresPerBlock={}",
                s.hasClimate(), deep, DeepTimeClimate.API_VERSION, c, nominal);
        helper.succeed();
    }

    private static double dot(double[] a, double[] b) {
        return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    }

    private static double angle(double[] a, double[] b) {
        return Math.acos(Math.max(-1.0, Math.min(1.0, dot(a, b))));
    }

    private static double turnDiff(double a, double b) {
        double d = Math.abs(a - b) % 360.0;
        return Math.min(d, 360.0 - d);
    }

    /** The planet unit vector of the API's latitude and longitude at (x, z). */
    private static double[] apiUnit(ServerLevel level, double x, double z) {
        return CubeWorld.unitOfLatLon(DeepTimeClimate.latitudeDeg(level, x, z), DeepTimeClimate.longitudeDeg(level, x, z));
    }

    /**
     * Climate API 5 on the 4k test planet baked as a cube world (cube plan G9; s = 1,024, the turn the
     * bake chose, so the poles are off the face centres): the kind and the period; latitude and
     * longitude at the six face centres (the turned face normals), the two poles and the 8 corners
     * (the turned cube corners, at ±35.26° in the cube's frame); north's bearing at each face centre
     * against the face frame, and on each face against latitude stepped along and across it; the
     * metres per block against the great-circle distance a block spans; every answer running on across
     * all 14 net edges into the copy beyond; NaN past the corners; and the planet info the client
     * builds its answers from saying the same.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 600)
    public static void climateApiOnTheCube(GameTestHelper helper) {
        ServerLevel level = CubeWorldGameTests.level(helper, GameTestSupport.TEST_PLANET_CUBE);
        GeologicChunkGenerator g = CubeWorldGameTests.generator(helper, level);
        if (!(g.terrain() instanceof WorldDataTerrain t) || !(t.data().projection() instanceof WorldProjection.Cube wp)) {
            helper.fail("test_planet_cube is not a cube world from world data");
            return;
        }
        CubeLayout cube = wp.cube();
        CubeWorld w = wp.world();
        double r = wp.radius(), nominal = t.data().manifest().mPerBlockH();
        long s = cube.side();
        DeepTimePlanet planet = DeepTimePlanet.of(level).orElse(null);
        helper.assertTrue(planet != null && Math.abs(planet.radiusMetres() - r * nominal) <= 1e-9 * r * nominal,
                "the cube planet's sphere: " + planet + " vs R " + r * nominal + " m");

        // The face centres: the turned face normals, and north there from the face frame.
        WorldInfoPayload info = WorldInfoPayload.forLevel(level);
        WorldProjection cp = info.worldProjection();
        helper.assertTrue(info.mPerBlockH() == nominal && cp != null, "the planet info's scale " + info.mPerBlockH() + " vs " + nominal);
        StringBuilder centres = new StringBuilder();
        for (CubeLayout.Face f : CubeLayout.Face.all()) {
            double[][] fr = f.frame();
            double[] c = w.turn().toPlanet(fr[0]);
            double[] at = cube.pointOfLocal(f, 0.0, 0.0);
            double lat = DeepTimeClimate.latitudeDeg(level, at[0], at[1]), lon = DeepTimeClimate.longitudeDeg(level, at[0], at[1]);
            double off = angle(CubeWorld.unitOfLatLon(lat, lon), c) * r;
            helper.assertTrue(off < 1e-4, "face " + f + "'s centre is " + off + " blocks from its turned normal");
            double b = DeepTimeClimate.northBearingDeg(level, at[0], at[1]), m = DeepTimeClimate.metresPerBlock(level, at[0], at[1]);
            if (Math.abs(lat) < 89.0) {
                double[] n = w.turn().toCube(WorldProjection.localFrame(c)[1]);
                double want = WorldProjection.bearingDeg(dot(n, fr[1]), dot(n, fr[2]));
                helper.assertTrue(turnDiff(b, want) < 1e-4, "north at face " + f + "'s centre " + b + "° vs the frame's " + want + "°");
            }
            helper.assertTrue(cp.northBearingDeg(at[0], at[1]) == wp.northBearingDeg(at[0], at[1])
                            && cp.scale(at[0], at[1]) == wp.scale(at[0], at[1]),
                    "the client's projection answers as the server's at face " + f);
            centres.append(String.format(java.util.Locale.ROOT, " %s(%.2f, %.2f; north %.1f°, %.1f m)", f, lat, lon, b, m));
        }

        // The poles, where the turn put them.
        CubeLayout.LocalPoint np = w.northPole(), sp = w.southPole();
        for (CubeLayout.LocalPoint pole : new CubeLayout.LocalPoint[]{np, sp}) {
            double[] at = cube.pointOfLocal(pole.face(), pole.u(), pole.v());
            double lat = DeepTimeClimate.latitudeDeg(level, at[0], at[1]);
            double want = pole == np ? 90.0 : -90.0;
            helper.assertTrue(Math.abs(lat - want) < 1e-5, "the pole on " + pole.face() + " at (" + at[0] + ", " + at[1] + ") reads " + lat);
            centres.append(String.format(java.util.Locale.ROOT, " %s pole on %s (%.1f, %.1f)", pole == np ? "north" : "south", pole.face(), at[0], at[1]));
        }

        // The corners: just inside each image on a face that holds it.
        int corners = 0;
        double worstCorner = 0, worstCubeLat = 0, cornerLat = Math.toDegrees(Math.asin(1.0 / Math.sqrt(3.0)));
        for (CubeLayout.Corner k : cube.corners()) {
            double[] want = w.cornerUnit(k.cube());
            for (long[] img : k.images()) {
                for (CubeLayout.Face f : CubeLayout.Face.all()) {
                    long ox = cube.faceOriginX(f), oz = cube.faceOriginZ(f);
                    if (img[0] < ox || img[0] > ox + s || img[1] < oz || img[1] > oz + s)
                        continue;
                    double cx = ox + s / 2.0 - img[0], cz = oz + s / 2.0 - img[1], d = Math.hypot(cx, cz);
                    double x = img[0] + 0.01 * cx / d, z = img[1] + 0.01 * cz / d;
                    double[] u = apiUnit(level, x, z);
                    worstCorner = Math.max(worstCorner, angle(u, want) * r);
                    double inCube = Math.toDegrees(Math.asin(Math.min(1.0, Math.abs(w.turn().toCube(u)[2]))));
                    worstCubeLat = Math.max(worstCubeLat, Math.abs(inCube - cornerLat));
                    corners++;
                }
            }
        }
        helper.assertTrue(corners >= 24 && worstCorner < 0.05 && worstCubeLat < 0.01,
                corners + " corner images, worst " + worstCorner + " blocks from the turned corner, " + worstCubeLat + "° off ±35.26° in the cube frame");

        // Every face: north against latitude stepped along and across it, the scale against the
        // great-circle distance a block spans in x and in z.
        int checked = 0;
        double worstAlong = 0, worstAcross = 0, worstScale = 0;
        for (CubeLayout.Face f : CubeLayout.Face.all()) {
            for (int i = 0; i < 5; i++) {
                for (int j = 0; j < 5; j++) {
                    double[] at = cube.pointOfLocal(f, -0.7 + 0.35 * i, -0.7 + 0.35 * j);
                    double x = at[0] + 0.25, z = at[1] + 0.25;
                    double m = DeepTimeClimate.metresPerBlock(level, x, z);
                    double gx = angle(apiUnit(level, x - 1, z), apiUnit(level, x + 1, z)) * r / 2.0 * nominal;
                    double gz = angle(apiUnit(level, x, z - 1), apiUnit(level, x, z + 1)) * r / 2.0 * nominal;
                    worstScale = Math.max(worstScale, Math.max(Math.abs(gx / m - 1.0), Math.abs(gz / m - 1.0)));
                    if (Math.abs(DeepTimeClimate.latitudeDeg(level, x, z)) > 80.0)
                        continue;
                    double b = Math.toRadians(DeepTimeClimate.northBearingDeg(level, x, z)), h = 2.0;
                    double dx = Math.sin(b), dz = -Math.cos(b);
                    double along = DeepTimeClimate.latitudeDeg(level, x + h * dx, z + h * dz) - DeepTimeClimate.latitudeDeg(level, x - h * dx, z - h * dz);
                    double across = DeepTimeClimate.latitudeDeg(level, x - h * dz, z + h * dx) - DeepTimeClimate.latitudeDeg(level, x + h * dz, z - h * dx);
                    double want = Math.toDegrees(2.0 * h * m / (nominal * r));
                    worstAlong = Math.max(worstAlong, Math.abs(along / want - 1.0));
                    worstAcross = Math.max(worstAcross, Math.abs(across / along));
                    checked++;
                }
            }
        }
        helper.assertTrue(checked > 100 && worstAlong < 0.01 && worstAcross < 0.01 && worstScale < 0.005,
                checked + " points: latitude along north off by " + worstAlong + ", across it " + worstAcross + " of along; the scale off the great circles by " + worstScale);

        // Across every net edge into the copy beyond: about a block apart on the sphere, north and the
        // scale running on.
        double worstGap = 0, worstTurn = 0, worstStep = 0;
        for (CubeLayout.NetEdge e : cube.edges()) {
            for (int q = 1; q <= 3; q++) {
                double along = e.spanFrom() + q * (e.spanTo() - e.spanFrom()) / 4.0 + 0.5, n = e.coord();
                double in = e.beyond() == CubeLayout.Side.MAX ? n - 0.5 : n + 0.5, out = e.beyond() == CubeLayout.Side.MAX ? n + 0.5 : n - 0.5;
                boolean ax = e.axis() == CubeLayout.Axis.X;
                double xi = ax ? in : along, zi = ax ? along : in, xo = ax ? out : along, zo = ax ? along : out;
                double mi = DeepTimeClimate.metresPerBlock(level, xi, zi), mo = DeepTimeClimate.metresPerBlock(level, xo, zo);
                double gap = angle(apiUnit(level, xi, zi), apiUnit(level, xo, zo)) * r * nominal / mi;
                worstGap = Math.max(worstGap, Math.abs(gap - 1.0));
                worstStep = Math.max(worstStep, Math.abs(mo / mi - 1.0));
                if (Math.abs(DeepTimeClimate.latitudeDeg(level, xi, zi)) < 85.0)
                    worstTurn = Math.max(worstTurn, turnDiff(DeepTimeClimate.northBearingDeg(level, xi, zi), DeepTimeClimate.northBearingDeg(level, xo, zo)));
            }
        }
        helper.assertTrue(worstGap < 0.02 && worstTurn < 1.0 && worstStep < 0.01,
                "across the 14 edges: a block is " + worstGap + " off one block at the scale, north turns " + worstTurn + "°, the scale steps " + worstStep);

        // Past the net's corners there is no planet.
        net.minecraft.world.level.ChunkPos ocean = CubeWorldGameTests.oceanChunk(cube);
        double ox = ocean.getMiddleBlockX() + 0.5, oz = ocean.getMiddleBlockZ() + 0.5;
        helper.assertTrue(Double.isNaN(DeepTimeClimate.latitudeDeg(level, ox, oz)) && Double.isNaN(DeepTimeClimate.northBearingDeg(level, ox, oz))
                        && Double.isNaN(DeepTimeClimate.metresPerBlock(level, ox, oz)),
                "past the corners at (" + ox + ", " + oz + "): " + DeepTimeClimate.latitudeDeg(level, ox, oz) + ", "
                        + DeepTimeClimate.northBearingDeg(level, ox, oz) + ", " + DeepTimeClimate.metresPerBlock(level, ox, oz));
        DeepTime.LOGGER.info("[gametest] climateApiOnTheCube m_per_block_h={} R={} centres:{}; corners={} worst {} blocks, {}° in the cube frame; "
                        + "faces {} points, along {} across {} scale {}; edges gap {} north {}° scale step {}",
                nominal, r, centres, corners, worstCorner, worstCubeLat, checked, worstAlong, worstAcross, worstScale, worstGap, worstTurn, worstStep);
        helper.succeed();
    }
}
