// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;

import static com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometryTest.LAMBERT;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

/**
 * The dev map's view transform, image codec, base image and graticule; on a cube (cube plan G14) the
 * view wraps 4s, the pictures leave the void clear (the synthetic cube world's own elevation picture
 * among them) and the graticule is cut only at the turning seams.
 */
class MapModelTest {

    /** x wrapped into [−period/2, period/2). */
    static double wrap(double period, double x) {
        double t = x - period * Math.floor(x / period + 0.5);
        return t >= period / 2 ? t - period : t;
    }

    @Test
    void viewTransformRoundTripAndZoomKeepsThePointUnderTheMouse() {
        ViewTransform v = new ViewTransform(13312, 100, -200, 0.25);
        v.viewport(4, 22, 800, 400);
        for (double sx : new double[]{4, 100.5, 404, 803})
            for (double sy : new double[]{22, 200.25, 421}) {
                assertEquals(sx, v.screenX(v.blockX(sx)), 1e-9);
                assertEquals(sy, v.screenY(v.blockZ(sy)), 1e-9);
            }
        assertEquals(404.0, v.screenX(100), 1e-9);
        assertEquals(222.0, v.screenY(-200), 1e-9);
        double bx = v.blockX(650), bz = v.blockZ(90);
        v.zoomAt(650, 90, 1.25, 0.01, 8);
        assertEquals(0.3125, v.scale(), 1e-12);
        // The centre wraps, so the block under the mouse is the same modulo the period.
        assertEquals(0.0, wrap(13312, v.blockX(650) - bx), 1e-6);
        assertEquals(bz, v.blockZ(90), 1e-6);
        v.zoomAt(650, 90, 1e9, 0.01, 8);
        assertEquals(8.0, v.scale());
        // Panning past the seam wraps the centre into [-P/2, P/2).
        v.centre(6650, 0);
        v.pan(-8 * 8, 0);
        assertEquals(-6654.0, v.cx(), 1e-9);
        assertEquals(0.0, wrap(13312, v.cx() - 6658), 1e-9);
        v.centre(0, 5000);
        v.clampZ(-3328, 3328);
        assertEquals(3328.0, v.cz());
    }

    @Test
    void viewTransformCopiesCoverTheView() {
        // Zoomed out so the view is three laps wide: the copies drawn must cover it.
        ViewTransform v = new ViewTransform(13312, 1000, 0, 1200.0 / (3 * 13312));
        v.viewport(0, 0, 1200, 300);
        int k0 = v.firstCopy(-6656, 6656), k1 = v.lastCopy(-6656, 6656);
        assertTrue(v.screenX(-6656 + k0 * 13312) <= 0, "left covered");
        assertTrue(v.screenX(6656 + k1 * 13312) >= 1200, "right covered");
        assertTrue(k1 - k0 <= 5);
    }

    @Test
    void imageCodecRoundTrip() {
        Random r = new Random(7);
        for (int[] wh : new int[][]{{1, 1}, {37, 11}, {2048, 1024}}) {
            int[] px = new int[wh[0] * wh[1]];
            // Smooth with noise: compresses like a map, still several parts at the full size.
            for (int i = 0; i < px.length; i++)
                px[i] = 0xff000000 | ((i / 97) & 0xff) << 16 | ((i * 31) & 0xff) << 8 | r.nextInt(256);
            List<byte[]> parts = ImageCodec.encode(px);
            for (byte[] p : parts)
                assertTrue(p.length <= ImageCodec.PART_BYTES);
            assertArrayEquals(px, ImageCodec.decode(parts, wh[0], wh[1]));
            if (wh[0] == 2048)
                assertTrue(parts.size() > 1, "a full image goes in several parts: " + parts.size());
        }
    }

    @Test
    void imageCodecRejectsWrongSizesAndBadData() {
        int[] px = new int[64 * 32];
        java.util.Arrays.fill(px, 0xff336699);
        List<byte[]> parts = ImageCodec.encode(px);
        assertThrows(IllegalArgumentException.class, () -> ImageCodec.decode(parts, 64, 31));
        assertThrows(IllegalArgumentException.class, () -> ImageCodec.decode(parts, 64, 33));
        assertThrows(IllegalArgumentException.class, () -> ImageCodec.decode(parts, 0, 10));
        assertThrows(IllegalArgumentException.class, () -> ImageCodec.decode(parts, 8192, 8192));
        assertThrows(IllegalArgumentException.class, () -> ImageCodec.decode(List.of(new byte[]{1, 2, 3, 4, 5}), 4, 4));
        List<byte[]> cut = new ArrayList<>(parts);
        byte[] last = cut.remove(cut.size() - 1);
        cut.add(java.util.Arrays.copyOf(last, last.length / 2));
        assertThrows(IllegalArgumentException.class, () -> ImageCodec.decode(cut, 64, 32));
    }

    @Test
    void baseImageSamplesTheCanonicalPointOfEachPixel() {
        // A synthetic planet: land (height 100) where latitude > 0, sea (height 20) south of it, a
        // lake (water at 150 over height 120) in a box: the image must follow the projection.
        MapFrame f = MapFrame.of(LAMBERT, 512);
        double sea = 63;
        Ramps.Ramp land = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x000000, 0xffffff});
        Ramps.Ramp lake = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x0000ff, 0x0000ff});
        int[] px = BaseImage.render(f, LAMBERT,
                (x, z) -> inLake(x, z) ? 120 : LAMBERT.latLon(x, z)[0] > 0 ? 100 : 20,
                (x, z) -> inLake(x, z) ? 150 : Double.NaN, sea, land, lake);
        int landC = land.abgr(100), seaC = land.abgr(20), lakeC = lake.abgr(150);
        int wrong = 0;
        for (int j = 0; j < f.h(); j++)
            for (int i = 0; i < f.w(); i++) {
                double x = f.blockX(i + 0.5), z = f.blockZ(j + 0.5);
                int want = inLake(x, z) ? lakeC : LAMBERT.latLon(x, z)[0] > 0 ? landC : seaC;
                if (px[j * f.w() + i] != want)
                    wrong++;
            }
        assertEquals(0, wrong);
        // sample() reads the pixel of a block, with x wrapping.
        double[] north = LAMBERT.blockOf(60, 146);
        assertEquals(landC, f.sample(px, north[0], north[1]));
        assertEquals(landC, f.sample(px, north[0] + 16384, north[1]));
        double[] south = LAMBERT.blockOf(-60, 146);
        assertEquals(seaC, f.sample(px, south[0], south[1]));
        assertEquals(lakeC, f.sample(px, 1050, 1050));
    }

    @Test
    void aClosedBasinBelowSeaLevelIsDrawnAsLandOrItsLake() {
        // Sea everywhere (height 20 under sea level 63) but a closed box (HYD-100): its dry ground
        // is drawn as the land ramp at sea level, its lake (water at 40 over 30) by the lake ramp.
        MapFrame f = MapFrame.of(LAMBERT, 512);
        double sea = 63;
        Ramps.Ramp land = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x000000, 0xffffff});
        Ramps.Ramp lake = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x0000ff, 0x00ffff});
        int[] px = BaseImage.render(f, LAMBERT,
                (x, z) -> inLake(x, z) ? 30 : 20,
                (x, z) -> inLake(x, z) && x < 1050 ? 40 : Double.NaN,
                (x, z) -> inLake(x, z) ? 1.0 : 0.0, sea, land, lake);
        int lakes = 0, dry = 0, wrong = 0;
        for (int j = 0; j < f.h(); j++)
            for (int i = 0; i < f.w(); i++) {
                double x = f.blockX(i + 0.5), z = f.blockZ(j + 0.5);
                int want = !inLake(x, z) ? land.abgr(20) : x < 1050 ? lake.abgr(40) : land.abgr(sea);
                lakes += want == lake.abgr(40) ? 1 : 0;
                dry += want == land.abgr(sea) ? 1 : 0;
                if (px[j * f.w() + i] != want)
                    wrong++;
            }
        assertEquals(0, wrong);
        assertTrue(lakes > 0 && dry > 0, lakes + " lake and " + dry + " dry pixels");
    }

    private static boolean inLake(double x, double z) {
        return x > 1000 && x < 1100 && z > 1000 && z < 1100;
    }

    @Test
    void latLonPictureAgreesWithTheBlockImage() {
        MapFrame f = MapFrame.of(LAMBERT, 512);
        int[] block = new int[f.w() * f.h()];
        for (int j = 0; j < f.h(); j++)
            for (int i = 0; i < f.w(); i++) {
                double[] ll = LAMBERT.latLon(f.blockX(i + 0.5), f.blockZ(j + 0.5));
                // Northern hemisphere east of the central meridian: red; the rest: green.
                double d = ll[1] - 146;
                d -= 360 * Math.round(d / 360);
                block[j * f.w() + i] = ll[0] > 0 && d > 0 ? 0xff0000ff : 0xff00ff00;
            }
        int w = 360, h = MapPicture.height(w);
        int[] pic = BaseImage.latLonPicture(block, f, LAMBERT, w, h);
        int wrong = 0, n = 0;
        for (int y = 0; y < h; y++) {
            double lat = MapPicture.latitudeDeg(y, h);
            for (int x = 0; x < w; x++) {
                double lon = MapPicture.longitudeDeg(x, w);
                double d = lon - 146;
                d -= 360 * Math.round(d / 360);
                // Skip the boundaries, where a 32-block pixel straddles the colour change.
                if (Math.abs(lat) < 1.5 || Math.abs(d) < 1.5 || Math.abs(Math.abs(d) - 180) < 1.5 || Math.abs(lat) > 88)
                    continue;
                int want = lat > 0 && d > 0 ? 0xff0000ff : 0xff00ff00;
                n++;
                if (pic[y * w + x] != want)
                    wrong++;
            }
        }
        assertTrue(n > 30000);
        assertEquals(0, wrong, wrong + " of " + n);
    }

    @Test
    void graticuleLiesOnItsLatitudeOrLongitude() {
        for (WorldProjection p : List.of(LAMBERT)) {
            List<Graticule.Line> lines = Graticule.lines(p, 15.0, 0.5);
            int meridians = 0, parallels = 0;
            for (Graticule.Line l : lines) {
                if (l.kind() == Graticule.Kind.MERIDIAN)
                    meridians++;
                else
                    parallels++;
                for (int i = 0; i < l.size(); i++) {
                    double[] ll = PlanetGeometry.displayLatLon(p, l.xs()[i], l.zs()[i]);
                    if (l.kind() == Graticule.Kind.PARALLEL) {
                        assertEquals(l.value(), ll[0], PlanetGeometry.FORWARD_TOLERANCE_DEG);
                    } else if (Math.abs(ll[0]) < 89.8) {
                        double d = ll[1] - l.value();
                        d -= 360 * Math.round(d / 360);
                        assertEquals(0.0, d, 1e-5, p.kind() + " meridian " + l.value() + " at lat " + ll[0]);
                    }
                    // No segment jumps: consecutive points are close in block space.
                    if (i > 0) {
                        double step = Math.hypot(l.xs()[i] - l.xs()[i - 1], l.zs()[i] - l.zs()[i - 1]);
                        assertTrue(step < 0.5 * p.period() / 4, p.kind() + ": a " + step + "-block step in " + l.kind() + " " + l.value());
                    }
                }
            }
            assertTrue(meridians >= 24, p.kind() + " meridians " + meridians);
            assertTrue(parallels >= 11, p.kind() + " parallels " + parallels);
            // Straight lines on the Lambert frame: one piece each, meridians vertical, parallels
            // horizontal (the cube's breaks at its turning seams have their own test below).
            assertEquals(24, meridians);
            assertEquals(11, parallels);
        }
    }

    // ---- the cube (cube plan G14) ----

    @Test
    void viewTransformWrapsACubesPeriod() {
        double s = PlanetGeometryTest.S, h = s / 2, period = 4 * s;
        ViewTransform v = new ViewTransform(period, -3 * h - 10, 0, 0.5);
        v.viewport(0, 0, 800, 600);
        // The centre is kept in Toroidal World's frame [-2s, 2s): -3h - 10 is in it, -4h - 10 is not.
        assertEquals(-3 * h - 10, v.cx(), 1e-9);
        // A drag of h + 10 blocks to the right moves the centre west to -4h - 20, which wraps to 4h - 20.
        v.pan(0.5 * (h + 10), 0);
        assertEquals(4 * h - 20, v.cx(), 1e-9, "wrapped past -2s");
        v.pan(-0.5 * 30, 0);
        assertEquals(-4 * h + 10, v.cx(), 1e-9, "wrapped past 2s");
        // The copies drawn from the window's west side cover the view, zoomed out to three laps.
        v.scale(800.0 / (3 * period));
        double w0 = PlanetGeometry.west(PlanetGeometryTest.CUBE);
        int k0 = v.firstCopy(w0, w0 + period), k1 = v.lastCopy(w0, w0 + period);
        assertTrue(v.screenX(w0 + k0 * period) <= 0, "left covered");
        assertTrue(v.screenX(w0 + period + k1 * period) >= 800, "right covered");
    }

    @Test
    void aCubesBaseImageLeavesTheVoidClear() {
        WorldProjection.Cube c = PlanetGeometryTest.CUBE;
        MapFrame f = MapFrame.of(c, 256);
        Ramps.Ramp land = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x000000, 0xffffff});
        Ramps.Ramp lake = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x0000ff, 0x0000ff});
        // Land north of the equator, sea south of it, read at the canonical point.
        int[] px = BaseImage.render(f, c, (x, z) -> c.latLon(x, z)[0] > 0 ? 100 : 20, (x, z) -> Double.NaN, 63, land, lake);
        int faces = 0, wrong = 0;
        for (int j = 0; j < f.h(); j++)
            for (int i = 0; i < f.w(); i++) {
                double x = f.blockX(i + 0.5), z = f.blockZ(j + 0.5);
                boolean on = PlanetGeometry.onPlanet(c, x, z);
                int want = !on ? 0 : land.abgr(c.latLon(x, z)[0] > 0 ? 100 : 20);
                faces += on ? 1 : 0;
                if (px[j * f.w() + i] != want)
                    wrong++;
            }
        assertEquals(0, wrong);
        assertEquals(f.w() * f.h() / 2, faces, "six faces of the twelve squares in the window");
        // The satellite's mask clears the same pixels and keeps the rest.
        int[] sat = new int[f.w() * f.h()];
        java.util.Arrays.fill(sat, 0xff123456);
        BaseImage.maskVoid(sat, f, c);
        for (int k = 0; k < sat.length; k++)
            assertEquals(px[k] == 0, sat[k] == 0, "pixel " + k);
        // Nothing to mask on a Lambert-frame level.
        MapFrame pf = MapFrame.of(LAMBERT, 64);
        int[] all = new int[pf.w() * pf.h()];
        java.util.Arrays.fill(all, 7);
        BaseImage.maskVoid(all, pf, LAMBERT);
        for (int v : all)
            assertEquals(7, v);
        // The latitude/longitude picture finds a face for every pixel.
        int w = 360, hh = MapPicture.height(w);
        int[] pic = BaseImage.latLonPicture(px, f, c, w, hh);
        int holes = 0;
        for (int v : pic)
            holes += v == 0 ? 1 : 0;
        assertEquals(0, holes, "transparent pixels in the globe's picture");
    }

    @Test
    void theCubeFixturesElevationPictureCoversItsFaces() {
        java.nio.file.Path dir = java.nio.file.Path.of(System.getProperty("deeptime.testWorlds")).resolve("cube1024");
        com.minecraftindustrialcomplex.deeptime.worlddata.WorldData wd = com.minecraftindustrialcomplex.deeptime.worlddata.WorldData.open(dir,
                new com.minecraftindustrialcomplex.deeptime.worlddata.TileCache(16L << 20));
        WorldProjection p = wd.projection();
        assertTrue(p instanceof WorldProjection.Cube, "cube1024 is " + p.kind());
        MapFrame f = MapFrame.of(p, 512);
        Ramps ramps = Ramps.builtin();
        int[] px = BaseImage.render(f, p, wd::height, wd::waterLevel, wd.manifest().seaLevelY(),
                ramps.forLayer(com.minecraftindustrialcomplex.deeptime.worlddata.WorldData.ELEVATION),
                ramps.forLayer(com.minecraftindustrialcomplex.deeptime.worlddata.WorldData.WATER_LEVEL));
        int clear = 0, opaque = 0, wrong = 0;
        for (int j = 0; j < f.h(); j++)
            for (int i = 0; i < f.w(); i++) {
                boolean on = PlanetGeometry.onPlanet(p, f.blockX(i + 0.5), f.blockZ(j + 0.5));
                int v = px[j * f.w() + i];
                if (on != ((v >>> 24) != 0))
                    wrong++;
                clear += v == 0 ? 1 : 0;
                opaque += (v >>> 24) == 0xff ? 1 : 0;
            }
        assertEquals(0, wrong, "void pixels drawn or face pixels clear");
        assertEquals(f.w() * f.h() / 2, clear);
        assertEquals(f.w() * f.h() / 2, opaque);
    }

    @Test
    void aCubesGraticuleIsCutOnlyAtTurningSeams() {
        for (WorldProjection.Cube c : PlanetGeometryTest.cubes()) {
            double h = c.side() / 2.0;
            List<Graticule.Line> lines = Graticule.lines(c, 15.0, 0.5);
            int pieces = 0;
            boolean acrossTheSlide = false;
            for (Graticule.Line l : lines) {
                pieces++;
                for (int i = 0; i < l.size(); i++) {
                    double x = l.xs()[i], z = l.zs()[i];
                    assertTrue(PlanetGeometry.onPlanet(c, x, z), "a graticule point off the faces at " + x + ", " + z);
                    double[] ll = PlanetGeometry.displayLatLon(c, x, z);
                    if (l.kind() == Graticule.Kind.PARALLEL)
                        assertEquals(l.value(), ll[0], PlanetGeometry.FORWARD_TOLERANCE_DEG);
                    if (i == 0)
                        continue;
                    // Never across the void: consecutive points on one face or on faces the net joins.
                    var a = c.cube().localOfPoint(l.xs()[i - 1], l.zs()[i - 1]);
                    var b = c.cube().localOfPoint(x, z);
                    assertTrue(Graticule.netJoined(a.face(), b.face()), l.kind() + " " + l.value() + ": " + a.face() + " to " + b.face());
                    double step = Math.hypot(x - l.xs()[i - 1], z - l.zs()[i - 1]);
                    assertTrue(step < 0.05 * c.side(), l.kind() + " " + l.value() + ": a " + step + "-block step");
                    // Onto or past the slide's line at x = 5h (or -3h): a sample can fall on the line itself.
                    acrossTheSlide |= (l.xs()[i - 1] < 5 * h) != (x < 5 * h) || (l.xs()[i - 1] > -3 * h) != (x > -3 * h);
                }
            }
            if (!acrossTheSlide) {
                StringBuilder why = new StringBuilder("no line runs on across the slide on the cube of side " + c.side() + ":");
                for (Graticule.Line l : lines)
                    if (l.kind() == Graticule.Kind.PARALLEL && Math.abs(l.value()) < 1e-9 || l.kind() == Graticule.Kind.MERIDIAN && l.value() == -180.0) {
                        double lo = Double.POSITIVE_INFINITY, hi = Double.NEGATIVE_INFINITY;
                        for (double x : l.xs()) {
                            lo = Math.min(lo, x);
                            hi = Math.max(hi, x);
                        }
                        why.append(String.format(java.util.Locale.ROOT, " [%s %.0f: %d points, x %.1f to %.1f, from (%.1f, %.1f) to (%.1f, %.1f)]",
                                l.kind(), l.value(), l.size(), lo, hi, l.xs()[0], l.zs()[0], l.xs()[l.size() - 1], l.zs()[l.size() - 1]));
                    }
                fail(why.toString());
            }
            // 24 meridians and 11 parallels, each cut where it crosses a turning seam.
            assertTrue(pieces > 35, "pieces " + pieces);
        }
        // The pairs the net joins.
        CubeLayout.Face f0 = CubeLayout.Face.F0, f1 = CubeLayout.Face.F1, f2 = CubeLayout.Face.F2, f3 = CubeLayout.Face.F3,
                n = CubeLayout.Face.N, s = CubeLayout.Face.S;
        for (CubeLayout.Face[] q : new CubeLayout.Face[][]{{f3, f0}, {f0, f1}, {f1, f2}, {f2, f3}, {n, f0}, {f0, s}, {f1, f1}})
            assertTrue(Graticule.netJoined(q[0], q[1]) && Graticule.netJoined(q[1], q[0]), q[0] + " " + q[1]);
        for (CubeLayout.Face[] q : new CubeLayout.Face[][]{{n, f1}, {n, f2}, {n, f3}, {s, f1}, {s, f2}, {s, f3}, {f0, f2}, {f1, f3}, {n, s}})
            assertFalse(Graticule.netJoined(q[0], q[1]) || Graticule.netJoined(q[1], q[0]), q[0] + " " + q[1]);
    }
}
