// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

/**
 * The density scale of {@code deeptime:terrain} at a column (plan §7.1): the terrain density is
 * (H − y) / scale, with scale = k where the terrain is gentle and, on steep ground,
 * {@link #STEEP_SCALE} × the largest height difference to the eight neighbouring noise-cell corners.
 *
 * <p><b>Why.</b> The final density takes the terrain value only within 1.5625·scale blocks of the
 * surface and the cave field below (the router's {@code range_choice}, as vanilla's around
 * {@code sloped_cheese}), and vanilla interpolates the router between corners four blocks apart.
 * Where two corners of a noise cell differ by more than 1.5625·scale — any slope over about three
 * blocks per block at k = 8 — the lower corner's cell mixes the upper corner's cave value (≈ 0.5,
 * not the terrain's large positive value) into the interpolation, the zero crossing moves
 * up-slope, and the face breaks into three-block benches and 20-block steps every four blocks.
 * That was the "striped snow" of the P1-08 renders: an east-facing flank whose columns read 308,
 * 308, 307, 287, 284, 283, 282, 275, 256, … — on 43 % of steep noise cells one step took 70 % or
 * more of the drop. Growing the scale with the local drop keeps every corner of such a cell in the
 * terrain band, so the drawn surface stays the bilinear H between corners (4 % to 11 % of steep
 * cells, the rest real cliffs). The surface at a corner is H whatever the scale; on gentle ground
 * (drops under 10 blocks at k = 8) nothing changes.
 *
 * <p>Pure and thread-safe; {@code -Ddeeptime.steepScale=0} turns the growth off (the P1-08 router,
 * for comparisons).
 */
public final class TerrainScale {

    private TerrainScale() {}

    /**
     * The noise-cell spacing, blocks: vanilla evaluates the router at corners this far apart and
     * interpolates between them ({@code size_horizontal} 1).
     */
    public static final int CORNER = 4;

    /** Share of the steepest corner-to-corner drop the scale grows to. */
    public static final double STEEP_SCALE =
            Double.parseDouble(System.getProperty("deeptime.steepScale", "0.8"));

    /** The scale at column (x, z) whose height is {@code h}, for the base scale {@code k}. */
    public static double at(TerrainSource src, int x, int z, double h, double k) {
        if (STEEP_SCALE <= 0.0)
            return k;
        double drop = 0.0;
        for (int dz = -CORNER; dz <= CORNER; dz += CORNER) {
            for (int dx = -CORNER; dx <= CORNER; dx += CORNER) {
                if (dx == 0 && dz == 0)
                    continue;
                drop = Math.max(drop, Math.abs(src.height(x + dx, z + dz) - h));
            }
        }
        return Math.max(k, STEEP_SCALE * drop);
    }
}
