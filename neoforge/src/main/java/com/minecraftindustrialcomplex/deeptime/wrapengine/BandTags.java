// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import java.util.List;

/**
 * What a block entity's client data needs when the band shows it (docs/design/wrap-engine.md §12.8):
 * the keys of an update tag that hold a position or a direction, by block entity type. Block states
 * turn by themselves; these do not. An absolute position moves by the motion stored → band, an offset
 * and a horizontal direction turn by its turn, so the band's copy points where the stored one points,
 * seen from the band.
 *
 * <p>Vanilla needs two rows. Create's keys join in E6 (and the table may become a data file then, as
 * Toroidal World's position rows are). This class touches no Minecraft type: the tag itself is read
 * and written by {@link BandBlockEntities#turnTag}.
 */
public final class BandTags {

    /** How a key's value changes. */
    public enum Kind {
        /** An absolute block position, an int array {x, y, z}: moved by the motion. */
        BLOCK_POS,
        /** An offset {dx, dy, dz}: turned by the motion's turn. */
        OFFSET,
        /** A {@code Direction} ordinal (down, up, north, south, west, east = 0 to 5): turned. */
        DIRECTION
    }

    /**
     * One position-bearing key.
     *
     * @param type the block entity type's id ({@code minecraft:end_gateway})
     * @param key  the key in its update tag
     */
    public record Row(String type, String key, Kind kind) {}

    /** The rows, vanilla's: an end gateway's exit and a brushable block's brushing side. */
    public static final List<Row> ROWS = List.of( // ledger: SCL-186 (vanilla needs these two rows; E6 adds Create's)
            new Row("minecraft:end_gateway", "exit_portal", Kind.BLOCK_POS),
            new Row("minecraft:brushable_block", "hit_direction", Kind.DIRECTION));

    /** North, south, west, east as (dx, dz), indexed by {@code Direction} ordinal − 2. */
    private static final int[][] HORIZONTAL = {{0, -1}, {0, 1}, {-1, 0}, {1, 0}};

    private BandTags() {}

    /** The rows for a block entity type id (empty for most). */
    public static List<Row> rowsFor(String type) {
        List<Row> out = null;
        for (Row r : ROWS)
            if (r.type().equals(type)) {
                if (out == null)
                    out = new java.util.ArrayList<>(1);
                out.add(r);
            }
        return out == null ? List.of() : out;
    }

    /** A position {x, y, z} moved by {@code g} (a new array; y unchanged). */
    public static int[] movePos(int[] p, CellIsometry g) {
        return new int[]{g.applyX(p[0], p[2]), p[1], g.applyZ(p[0], p[2])};
    }

    /** An offset {dx, dy, dz} turned by {@code g}'s turn (a new array). */
    public static int[] turnOffset(int[] d, CellIsometry g) {
        double[] v = g.applyVector(d[0], d[2]);
        return new int[]{(int) Math.round(v[0]), d[1], (int) Math.round(v[1])};
    }

    /** A {@code Direction} ordinal turned by {@code g}'s turn: up and down, and unknown values, unchanged. */
    public static int turnDirection(int ordinal, CellIsometry g) {
        if (ordinal < 2 || ordinal > 5)
            return ordinal;
        int[] d = HORIZONTAL[ordinal - 2];
        double[] v = g.applyVector(d[0], d[1]);
        int dx = (int) Math.round(v[0]), dz = (int) Math.round(v[1]);
        for (int i = 0; i < 4; i++)
            if (HORIZONTAL[i][0] == dx && HORIZONTAL[i][1] == dz)
                return i + 2;
        throw new IllegalStateException("a quarter turn of a horizontal direction is horizontal: " + dx + ", " + dz);
    }
}
