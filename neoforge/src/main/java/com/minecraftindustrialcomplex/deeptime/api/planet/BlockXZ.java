// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.planet;

/**
 * x and z in a level's block frame, in blocks: a position (block (bx, bz) has its centre at
 * (bx + 0.5, bz + 0.5)) or, from {@link LocalFrame#step}, a step between two positions.
 */
public record BlockXZ(double x, double z) {

    /** The block x this position lies in. */
    public int blockX() {
        return (int) Math.floor(x);
    }

    /** The block z this position lies in. */
    public int blockZ() {
        return (int) Math.floor(z);
    }
}
