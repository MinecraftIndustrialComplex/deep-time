// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.worlddata.MapPicture;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

import java.util.function.DoubleBinaryOperator;

/**
 * The map's elevation picture ({@code MapLayer.ELEVATION}; its satellite sibling is drawn by Deep
 * Time's natives, {@code nativebridge.NativeWorldMap}, and needs them, so this one is also the
 * fallback), drawn the way Deep Time's own review map draws the world data
 * ({@code screen.model.ReviewModel.elevationMap}): the terrain height H = E + A·N
 * ({@code WorldData.height}) through the shared {@code elevation} colour ramp
 * ({@code deeptime/ramps.json}, the ramp the keyframes, the viewer and the Create World screen use),
 * and lakes above sea level through the {@code water} ramp of their surface. A closed basin below
 * sea level (the {@code closed_basin} layer, HYD-100) holds no sea: its lakes are drawn by their
 * surface like any lake, and its dry ground as the lowest land, the ramp's colour at sea level.
 * Only the sampling grid
 * differs: block space, one sample at each pixel's centre, rather than the review map's
 * latitude/longitude picture. On a cube the pixels off the six faces (the void of the net's window)
 * are transparent ({@link #maskVoid}).
 */
public final class BaseImage {

    private BaseImage() {}

    /**
     * The block-space image of the frame.
     *
     * @param height    terrain height at block (x, z), block y (NaN: nodata, drawn transparent)
     * @param water     water surface at block (x, z), block y, NaN where there is none
     * @param seaLevel  the world's sea level, block y (water at or below it is the sea, which the
     *                  elevation ramp already draws by depth)
     */
    public static int[] render(MapFrame f, WorldProjection p, DoubleBinaryOperator height, DoubleBinaryOperator water,
                               double seaLevel, Ramps.Ramp land, Ramps.Ramp lake) {
        return render(f, p, height, water, (x, z) -> 0.0, seaLevel, land, lake);
    }

    /**
     * As {@link #render(MapFrame, WorldProjection, DoubleBinaryOperator, DoubleBinaryOperator, double, Ramps.Ramp,
     * Ramps.Ramp)}, with {@code closed} 1 at block (x, z) where no sea stands on ground below sea level (HYD-100).
     */
    public static int[] render(MapFrame f, WorldProjection p, DoubleBinaryOperator height, DoubleBinaryOperator water,
                               DoubleBinaryOperator closed, double seaLevel, Ramps.Ramp land, Ramps.Ramp lake) {
        int[] px = new int[f.w() * f.h()];
        boolean cube = PlanetGeometry.cube(p) != null;
        for (int j = 0; j < f.h(); j++) {
            double z = f.blockZ(j + 0.5);
            for (int i = 0; i < f.w(); i++) {
                if (cube && !PlanetGeometry.onPlanet(p, f.blockX(i + 0.5), z))
                    continue;
                double[] c = p.canonical(f.blockX(i + 0.5), z);
                double h = height.applyAsDouble(c[0], c[1]);
                double wl = water.applyAsDouble(c[0], c[1]);
                boolean lakeHere = !Double.isNaN(wl) && wl > h;
                int colour;
                if (h < seaLevel && closed.applyAsDouble(c[0], c[1]) == 1.0)
                    colour = lakeHere ? lake.abgr(wl) : land.abgr(seaLevel);
                else
                    colour = lakeHere && wl > seaLevel + 0.5 ? lake.abgr(wl) : land.abgr(h);
                px[j * f.w() + i] = colour;
            }
        }
        return px;
    }

    /**
     * Clears, in place, the pixels of a block-space image of a cube world whose centres lie off the
     * six faces (the void of the net's window, which the satellite's natives draw as the copies the
     * fold brings in), so the map shows the net as it is played; nothing on a polar world. Returns
     * the image.
     */
    public static int[] maskVoid(int[] abgr, MapFrame f, WorldProjection p) {
        if (PlanetGeometry.cube(p) == null)
            return abgr;
        for (int j = 0; j < f.h(); j++) {
            double z = f.blockZ(j + 0.5);
            for (int i = 0; i < f.w(); i++)
                if (!PlanetGeometry.onPlanet(p, f.blockX(i + 0.5), z))
                    abgr[j * f.w() + i] = 0;
        }
        return abgr;
    }

    /**
     * The latitude/longitude picture of a block-space image, for Deep Time's globe
     * ({@code GlobeProjection.render}): {@code w × h} pixels, pixel (x, y) at longitude
     * {@link MapPicture#longitudeDeg}(x) and latitude {@link MapPicture#latitudeDeg}(y)
     * (the keyframes' layout), coloured by the block image at {@link PlanetGeometry#blockOf} (Deep Time's forward, checked).
     */
    public static int[] latLonPicture(int[] block, MapFrame f, WorldProjection p, int w, int h) {
        int[] out = new int[w * h];
        for (int y = 0; y < h; y++) {
            double lat = MapPicture.latitudeDeg(y, h);
            for (int x = 0; x < w; x++) {
                double[] b = PlanetGeometry.blockOf(p, lat, MapPicture.longitudeDeg(x, w));
                out[y * w + x] = f.sample(block, b[0], b[1]);
            }
        }
        return out;
    }
}
