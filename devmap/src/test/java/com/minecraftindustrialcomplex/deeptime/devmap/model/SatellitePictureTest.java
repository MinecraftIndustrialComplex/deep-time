// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeStatus;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeWorldMap;
import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.worlddata.TileCache;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Path;
import java.util.Locale;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The satellite picture the server sends ({@code ServerMaps}: {@link NativeWorldMap} over the dev
 * map's {@link MapFrame}) on the real test planet (the cube world of period 4096, {@code planet4k} from
 * {@code ./gradlew testWorlds}): it covers the frame, and it lies where the world is. Deep sea in the
 * world data (the elevation picture's source) is the satellite palette's sea blue, dry land is not,
 * and the same check a quarter of the world east fails, so the agreement is the picture's
 * registration and not the planet's sea share.
 */
class SatellitePictureTest {

    @TempDir
    static Path gameDir;

    static Path world;

    @BeforeAll
    static void load() {
        NativeStatus status = NativeLoader.load(gameDir);
        assertTrue(status.isOk(), status::describe);
        world = Path.of(System.getProperty("deeptime.testWorlds")).resolve("planet4k");
    }

    /**
     * Sea in the satellite palette (shallow #3fa5b5, shelf #25649a, ocean #113a78, deep #071a42):
     * blue well above red. Land cover, snow, glacier and sea ice all have blue within some 16 of red.
     */
    static boolean seaBlue(int abgr) {
        int r = abgr & 0xff, b = (abgr >>> 16) & 0xff;
        return b - r > 35;
    }

    @Test
    void theSatellitePictureLiesWhereTheWorldDataSaysSeaAndLandAre() {
        WorldData wd = WorldData.open(world, new TileCache(64L << 20));
        WorldProjection p = wd.projection();
        MapFrame f = MapFrame.of(p, 512);
        long t0 = System.nanoTime();
        NativeWorldMap.Picture sat = NativeWorldMap.render(world, NativeWorldMap.SATELLITE, f.x0(), f.z0(), f.blocksPerPx(),
                f.w(), f.h(), 4);
        long ms = (System.nanoTime() - t0) / 1_000_000;
        assertEquals(f.w() * f.h(), sat.abgr().length);
        for (int i = 0; i < sat.abgr().length; i++)
            assertEquals(0xff, sat.abgr()[i] >>> 24, "pixel " + i + " is opaque");

        int[] here = agreement(wd, p, f, sat.abgr(), 0);
        int[] east = agreement(wd, p, f, sat.abgr(), f.w() / 4);
        String line = String.format(Locale.ROOT,
                "frame %dx%d at %.1f blocks/px drawn in %d ms; deep sea %d of %d sea blue, dry land %d of %d not;"
                        + " a quarter east: %d of %d, %d of %d; %s",
                f.w(), f.h(), f.blocksPerPx(), ms, here[1], here[0], here[3], here[2], east[1], east[0], east[3], east[2],
                sat.info());
        System.out.println("[junit] SatellitePictureTest " + line);
        // A sample-size floor, not a property of the planet: the test planet is mostly sea, and since
        // Law 1 (PLT-655) its dry land below 50 degrees is some 1,400 pixels of this frame.
        assertTrue(here[0] > 1000 && here[2] > 1000, "enough deep sea and dry land to judge: " + line);
        assertTrue(here[1] >= 0.95 * here[0], "deep sea is sea blue: " + line);
        assertTrue(here[3] >= 0.90 * here[2], "dry land is not sea blue: " + line);
        double hereScore = (double) here[1] / here[0] + (double) here[3] / here[2];
        double eastScore = (double) east[1] / east[0] + (double) east[3] / east[2];
        assertTrue(eastScore < hereScore - 0.2, "a picture a quarter of the world off agrees much less: " + line);
    }

    /**
     * Pixels of the frame below 50° of latitude (no sea ice, few glaciers) as {deep sea, of them sea
     * blue, dry land, of them not sea blue}, the picture read {@code shift} columns east.
     */
    private static int[] agreement(WorldData wd, WorldProjection p, MapFrame f, int[] abgr, int shift) {
        int sea = wd.manifest().seaLevelY();
        int[] n = new int[4];
        for (int j = 0; j < f.h(); j++) {
            double z = f.blockZ(j + 0.5);
            for (int i = 0; i < f.w(); i++) {
                double x = f.blockX(i + 0.5);
                double[] c = p.canonical(x, z);
                if (Math.abs(p.latLon(c[0], c[1])[0]) >= 50)
                    continue;
                double h = wd.height(c[0], c[1]);
                if (Double.isNaN(h))
                    continue;
                boolean blue = seaBlue(abgr[j * f.w() + Math.floorMod(i + shift, f.w())]);
                if (h < sea - 15 && !wd.closedBasin(c[0], c[1])) {
                    n[0]++;
                    if (blue)
                        n[1]++;
                } else if (h > sea + 5) {
                    double wl = wd.waterLevel(c[0], c[1]);
                    if (!Double.isNaN(wl) && wl > h)
                        continue;
                    n[2]++;
                    if (!blue)
                        n[3]++;
                }
            }
        }
        return n;
    }

    @Test
    void bothPicturesShareTheFrame() {
        // The elevation picture the server draws in Java beside it: the same frame, every pixel set.
        WorldData wd = WorldData.open(world, new TileCache(64L << 20));
        WorldProjection p = wd.projection();
        MapFrame f = MapFrame.of(p, 256);
        Ramps ramps = Ramps.builtin();
        int[] elev = BaseImage.render(f, p, wd::height, wd::waterLevel, (x, z) -> wd.closedBasin(x, z) ? 1.0 : 0.0,
                wd.manifest().seaLevelY(), ramps.forLayer(WorldData.ELEVATION), ramps.forLayer(WorldData.WATER_LEVEL));
        NativeWorldMap.Picture sat = NativeWorldMap.render(world, NativeWorldMap.SATELLITE, f.x0(), f.z0(), f.blocksPerPx(),
                f.w(), f.h(), 2);
        assertEquals(elev.length, sat.abgr().length);
        assertEquals(f.w(), sat.w());
        assertEquals(f.h(), sat.h());
        int differ = 0;
        for (int i = 0; i < elev.length; i++)
            if (elev[i] != sat.abgr()[i])
                differ++;
        assertTrue(differ > elev.length / 2, "two different looks: " + differ + " of " + elev.length + " pixels differ");
    }
}
