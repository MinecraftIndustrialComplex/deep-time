// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

/**
 * A cone point of a topology and its images in the topology's frame (docs/design/wrap-engine.md
 * §12.4): on the cube, a corner where three faces meet, 270° round (ledger: SCL-182). Near a cone point
 * no gluing is locally flat, so seam links within {@link WrapTopology#CORNER_KEEP_OUT} blocks of an
 * image are refused ({@link WrapTopology#nearCorner}).
 *
 * @param name   for logs ({@code n0} .. {@code s3} on the cube)
 * @param images every block corner (integer point) of the topology's frame that is this cone point:
 *               {x, z} each
 */
public record WrapCorner(String name, int[][] images) {

    public WrapCorner {
        for (int[] p : images)
            if (p.length != 2)
                throw new IllegalArgumentException("wrap corner " + name + ": an image is {x, z}");
    }

    /** This corner moved by (dx, dz). */
    public WrapCorner translated(int dx, int dz) {
        int[][] moved = new int[images.length][];
        for (int i = 0; i < images.length; i++)
            moved[i] = new int[]{images[i][0] + dx, images[i][1] + dz};
        return new WrapCorner(name, moved);
    }
}
