// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;

/**
 * A raster in block space, laid out by the world's projection (plan §5.2, FORMAT.md "Grids"). Cell
 * (i, j) is the point sample at block (originBlockX + i·r, originBlockZ + j·r); x wraps with period
 * w·r. Rows past the first or the last clamp ({@link ZEdges#CLAMP}: the simulation's grids and Lambert
 * worlds) or fold through the north and south edges' half-turns ({@link ZEdges#HALF_TURN}: a
 * Petroff–Guyou world, where row −k is row k and row h − 1 + k is row h − 1 − k, both at column
 * w − i). Every reader samples through {@link #cellI}/{@link #cellJ} (Rust {@code GridDesc::cell}).
 */
public record GridDesc(String name, int w, int h, int r, boolean wrapsX, long originBlockX, long originBlockZ, ZEdges zEdges) {

    /** {@code z_edges}: what a row past the first or the last means. */
    public enum ZEdges {
        CLAMP,
        HALF_TURN
    }

    /** A clamping grid (the simulation's grids, Lambert worlds, synthetic worlds). */
    public GridDesc(String name, int w, int h, int r, boolean wrapsX, long originBlockX, long originBlockZ) {
        this(name, w, h, r, wrapsX, originBlockX, originBlockZ, ZEdges.CLAMP);
    }

    static GridDesc fromJson(JsonObject o) {
        ZEdges z = ZEdges.CLAMP;
        if (o.has("z_edges")) {
            String s = o.get("z_edges").getAsString();
            z = switch (s) {
                case "clamp" -> ZEdges.CLAMP;
                case "half_turn" -> ZEdges.HALF_TURN;
                default -> throw new IllegalArgumentException("unknown z_edges '" + s + "'");
            };
        }
        return new GridDesc(
                o.get("name").getAsString(),
                o.get("w").getAsInt(),
                o.get("h").getAsInt(),
                o.get("r").getAsInt(),
                o.get("wraps_x").getAsBoolean(),
                o.get("origin_block_x").getAsLong(),
                o.get("origin_block_z").getAsLong(),
                z);
    }

    /** The x period w·r (C on a Lambert world, 2b on a Petroff–Guyou one). */
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

    /**
     * The stored cell of any index pair, packed: {@code (long) row << 32 | column}. Columns wrap; rows
     * clamp, or fold through the half-turns. Use {@link #cellI}/{@link #cellJ} to unpack.
     */
    public long cell(long i, long j) {
        if (zEdges == ZEdges.CLAMP)
            return ((long) clampJ(j) << 32) | wrapI(i);
        long last = h - 1L;
        if (last <= 0)
            return wrapI(i);
        for (int k = 0; k < 64; k++) {
            if (j < 0) {
                j = -j;
                i = w - i;
            } else if (j > last) {
                j = 2 * last - j;
                i = w - i;
            } else {
                break;
            }
        }
        return ((long) (int) Math.max(0, Math.min(last, j)) << 32) | wrapI(i);
    }

    /** The column of a packed {@link #cell}. */
    public static int cellI(long cell) {
        return (int) (cell & 0xFFFF_FFFFL);
    }

    /** The row of a packed {@link #cell}. */
    public static int cellJ(long cell) {
        return (int) (cell >>> 32);
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
