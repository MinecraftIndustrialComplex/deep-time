// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;

/**
 * A raster in block space, laid out by the planet's {@link Projection} (plan §5.2, FORMAT.md "Grids"). Cell (i, j) is the
 * point sample at block (originBlockX + i·r, originBlockZ + j·r); x wraps with period w·r = C,
 * z clamps.
 */
public record GridDesc(String name, int w, int h, int r, boolean wrapsX, long originBlockX, long originBlockZ) {

    static GridDesc fromJson(JsonObject o) {
        return new GridDesc(
                o.get("name").getAsString(),
                o.get("w").getAsInt(),
                o.get("h").getAsInt(),
                o.get("r").getAsInt(),
                o.get("wraps_x").getAsBoolean(),
                o.get("origin_block_x").getAsLong(),
                o.get("origin_block_z").getAsLong());
    }

    /** C = w·r. */
    public long circumference() {
        return (long) w * r;
    }

    /** Wraps a column index into [0, w). */
    public int wrapI(long i) {
        return Projection.wrapColumn(i, w);
    }

    /** Clamps a row index into [0, h − 1]. */
    public int clampJ(long j) {
        return Projection.clampRow(j, h);
    }

    /** Block x of column i. */
    public long blockX(long i) {
        return originBlockX + i * r;
    }

    /** Block z of row j. */
    public long blockZ(long j) {
        return originBlockZ + j * r;
    }
}
