// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import org.junit.jupiter.api.Test;

import java.util.List;

/** Which blocks of a column an exposed gossan repaints (decisions log 2026-10-17; no Minecraft types needed). */
class GossanSkinTest {

    private static final int C = 1024;

    /** A blanket (shape 7) 10 blocks across, 4 blocks thick (y 98 to 101), its top clipped at the surface y 101. */
    private static Body gossan(int clipTopY) {
        return new Body(7, 1, 7, GossanSkin.ROLE, new double[]{0, 100, 0}, new float[]{10, 2, 10}, 0f, 0f, clipTopY,
                new int[]{-10, 97, -10}, new int[]{10, 103, 10}, "ironstone", List.of());
    }

    @Test
    void theExtentIsTheBlocksOfTheBodyInTheColumn() {
        assertArrayEquals(new int[]{101, 98}, GossanSkin.extent(gossan(101), C, 0, 0, 90, 110));
        assertArrayEquals(new int[]{101, 98}, GossanSkin.extent(gossan(101), C, 5, -4, 90, 110));
        // A column beside the body misses it.
        assertNull(GossanSkin.extent(gossan(101), C, 40, 0, 90, 110));
        // The clip cuts the top: the body is a sheet below the surface that cut it.
        assertArrayEquals(new int[]{99, 98}, GossanSkin.extent(gossan(99), C, 0, 0, 90, 110));
    }

    @Test
    void theGroundWithinTheSlackAboveTheTopIsTheOutcrop() {
        int top = 101, bottom = 98;
        // Ground at the top, inside the body, and the soil's worth above it (the simulation's surface
        // and the generated terrain a block or three apart): the column is repainted to the bottom.
        for (int ground = bottom; ground <= top + GossanSkin.EXPOSURE_SLACK; ground++) {
            GossanSkin.Span s = GossanSkin.span(ground, top, bottom);
            assertNotNull(s, "ground " + ground);
            assertEquals(ground, s.from());
            assertEquals(bottom, s.to());
        }
    }

    @Test
    void aGossanUnderCoverOrBelowTheGroundIsLeftAlone() {
        int top = 101, bottom = 98;
        // The ground more than the slack above the top: a palaeo-gossan under younger cover.
        assertNull(GossanSkin.span(top + GossanSkin.EXPOSURE_SLACK + 1, top, bottom));
        assertNull(GossanSkin.span(top + 30, top, bottom));
        // The ground under the whole body: nothing of it is left in the column.
        assertNull(GossanSkin.span(bottom - 1, top, bottom));
    }

    @Test
    void theSkinIsAtLeastAsThickAsTheBodyWhateverTheSoil() {
        // The two-block floor of the deposits side (DEP-210) makes every column span at least two blocks.
        GossanSkin.Span s = GossanSkin.span(101, 101, 100);
        assertNotNull(s);
        assertEquals(2, s.from() - s.to() + 1);
    }
}
