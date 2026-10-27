// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * A one-block seam between two river channels is channel ({@link Water#finish}; ledger: HYD-182), as in Rust's
 * {@code a_one_block_seam_between_two_channels_is_channel}.
 */
class WaterSeamTest {

    private static Water.Core river(int top, int bed) {
        return new Water.Core(Water.Kind.RIVER, 64.4, 64, Double.NaN, top, bed, Rivers.Sample.NONE);
    }

    private static final Water.Core NONE = new Water.Core(Water.Kind.NONE, 64.4, 64, Double.NaN, Integer.MIN_VALUE,
            Integer.MIN_VALUE, Rivers.Sample.NONE);

    @Test
    void aOneBlockSeamBetweenTwoChannelsIsChannel() {
        // East and west are river columns of two segments whose corridors left this column out: it takes the
        // lower water (62) on the shallower bed (60), not a bank's ground.
        Water.Column seam = Water.finish(NONE, new Water.Core[]{river(63, 57), river(62, 60), NONE, NONE},
                new int[]{63, 63, Integer.MIN_VALUE, Integer.MIN_VALUE});
        assertEquals(Water.Kind.RIVER, seam.kind());
        assertEquals(62, seam.top());
        assertEquals(60, seam.bed());
        // North and south count the same; one river side alone stays a bank.
        Water.Column ns = Water.finish(NONE, new Water.Core[]{NONE, NONE, river(63, 58), river(63, 59)},
                new int[]{Integer.MIN_VALUE, Integer.MIN_VALUE, 63, 63});
        assertEquals(Water.Kind.RIVER, ns.kind());
        assertEquals(63, ns.top());
        assertEquals(59, ns.bed());
        Water.Column bank = Water.finish(NONE, new Water.Core[]{river(63, 57), NONE, NONE, NONE},
                new int[]{63, Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE});
        assertEquals(Water.Kind.BANK, bank.kind());
    }
}
