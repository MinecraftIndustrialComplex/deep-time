// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.worlddata.Projection;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;

import static com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometryTest.LAMBERT;
import static com.minecraftindustrialcomplex.deeptime.devmap.model.PlanetGeometryTest.PG;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** The dev map's view transform, image codec, base image and graticule. */
class MapModelTest {

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
        assertEquals(0.0, PG.wrapX(v.blockX(650) - bx), 1e-6);
        assertEquals(bz, v.blockZ(90), 1e-6);
        v.zoomAt(650, 90, 1e9, 0.01, 8);
        assertEquals(8.0, v.scale());
        // Panning past the seam wraps the centre into [-P/2, P/2).
        v.centre(6650, 0);
        v.pan(-8 * 8, 0);
        assertEquals(-6654.0, v.cx(), 1e-9);
        assertEquals(0.0, PG.wrapX(v.cx() - 6658), 1e-9);
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
        MapFrame f = MapFrame.of(PG, 256);
        double sea = 63;
        Ramps.Ramp land = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x000000, 0xffffff});
        Ramps.Ramp lake = new Ramps.Ramp(new double[]{0, 200}, new int[]{0x0000ff, 0x0000ff});
        int[] px = BaseImage.render(f, PG,
                (x, z) -> inLake(x, z) ? 120 : PG.latLon(x, z)[0] > 0 ? 100 : 20,
                (x, z) -> inLake(x, z) ? 150 : Double.NaN, sea, land, lake);
        int landC = land.abgr(100), seaC = land.abgr(20), lakeC = lake.abgr(150);
        int wrong = 0;
        for (int j = 0; j < f.h(); j++)
            for (int i = 0; i < f.w(); i++) {
                double x = f.blockX(i + 0.5), z = f.blockZ(j + 0.5);
                int want = inLake(x, z) ? lakeC : PG.latLon(x, z)[0] > 0 ? landC : seaC;
                if (px[j * f.w() + i] != want)
                    wrong++;
            }
        assertEquals(0, wrong);
        // sample() reads the pixel of a block, with x wrapping.
        double[] north = PG.blockOf(60, 146);
        assertEquals(landC, f.sample(px, north[0], north[1]));
        assertEquals(landC, f.sample(px, north[0] + 13312, north[1]));
        double[] south = PG.blockOf(-60, 146);
        assertEquals(seaC, f.sample(px, south[0], south[1]));
        assertEquals(lakeC, f.sample(px, 1050, 1050));
    }

    private static boolean inLake(double x, double z) {
        return x > 1000 && x < 1100 && z > 1000 && z < 1100;
    }

    @Test
    void latLonPictureAgreesWithTheBlockImage() {
        MapFrame f = MapFrame.of(PG, 512);
        int[] block = new int[f.w() * f.h()];
        for (int j = 0; j < f.h(); j++)
            for (int i = 0; i < f.w(); i++) {
                double[] ll = PG.latLon(f.blockX(i + 0.5), f.blockZ(j + 0.5));
                // Northern hemisphere east of the central meridian: red; the rest: green.
                double d = ll[1] - 146;
                d -= 360 * Math.round(d / 360);
                block[j * f.w() + i] = ll[0] > 0 && d > 0 ? 0xff0000ff : 0xff00ff00;
            }
        int w = 360, h = Projection.pictureHeight(w);
        int[] pic = BaseImage.latLonPicture(block, f, PG, w, h);
        int wrong = 0, n = 0;
        for (int y = 0; y < h; y++) {
            double lat = Projection.pictureLatitudeDeg(y, h);
            for (int x = 0; x < w; x++) {
                double lon = Projection.pictureLongitudeDeg(x, w);
                double d = lon - 146;
                d -= 360 * Math.round(d / 360);
                // Skip the boundaries, where a 13-block pixel straddles the colour change.
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
    void graticuleLiesOnItsLatitudeOrLongitudeAndBreaksAtTheEdges() {
        for (WorldProjection p : List.of(PG, LAMBERT)) {
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
                        assertEquals(l.value(), ll[0], 1e-6);
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
            if (p == LAMBERT) {
                // Straight lines: one piece each, meridians vertical, parallels horizontal.
                assertEquals(24, meridians);
                assertEquals(11, parallels);
            } else {
                // Petroff-Guyou: the lines break where they cross an edge (the half-turn).
                assertTrue(meridians + parallels > 35, "pieces " + (meridians + parallels));
            }
        }
    }
}
