// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.worlddata.Projection;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

import java.util.function.DoubleBinaryOperator;

/**
 * The map's base image, drawn the way Deep Time's own review map draws the world data
 * ({@code screen.model.ReviewModel.elevationMap}): the terrain height H = E + A·N
 * ({@code WorldData.height}) through the shared {@code elevation} colour ramp
 * ({@code deeptime/ramps.json}, the ramp the keyframes, the viewer and the Create World screen use),
 * and lakes above sea level through the {@code water} ramp of their surface. Only the sampling grid
 * differs: block space, one sample at each pixel's centre, rather than the review map's
 * latitude/longitude picture.
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
        int[] px = new int[f.w() * f.h()];
        for (int j = 0; j < f.h(); j++) {
            double z = f.blockZ(j + 0.5);
            for (int i = 0; i < f.w(); i++) {
                double[] c = p.canonical(f.blockX(i + 0.5), z);
                double h = height.applyAsDouble(c[0], c[1]);
                double wl = water.applyAsDouble(c[0], c[1]);
                px[j * f.w() + i] = !Double.isNaN(wl) && wl > h && wl > seaLevel + 0.5 ? lake.abgr(wl) : land.abgr(h);
            }
        }
        return px;
    }

    /**
     * The latitude/longitude picture of a block-space image, for Deep Time's globe
     * ({@code GlobeProjection.render}): {@code w × h} pixels, pixel (x, y) at longitude
     * {@link Projection#pictureLongitudeDeg}(x) and latitude {@link Projection#pictureLatitudeDeg}(y)
     * (the keyframes' layout), coloured by the block image at {@link PlanetGeometry#blockOf} (Deep Time's forward, checked).
     */
    public static int[] latLonPicture(int[] block, MapFrame f, WorldProjection p, int w, int h) {
        int[] out = new int[w * h];
        for (int y = 0; y < h; y++) {
            double lat = Projection.pictureLatitudeDeg(y, h);
            for (int x = 0; x < w; x++) {
                double[] b = PlanetGeometry.blockOf(p, lat, Projection.pictureLongitudeDeg(x, w));
                out[y * w + x] = f.sample(block, b[0], b[1]);
            }
        }
        return out;
    }
}
