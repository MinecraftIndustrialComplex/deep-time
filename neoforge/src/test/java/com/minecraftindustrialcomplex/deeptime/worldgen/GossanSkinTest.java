// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import org.junit.jupiter.api.Test;

import java.util.List;

/** Which gossans crop out and which blocks of a column their cap repaints (decisions log 2026-10-17; no Minecraft types needed). */
class GossanSkinTest {

    private static final int C = 1024;

    /** A blanket (shape 7) 10 blocks across, {@code 2 * semiY} blocks thick, centred at y 100, its top clipped at {@code clipTopY}. */
    private static Body gossan(float semiY, int clipTopY) {
        return new Body(7, 1, 7, GossanSkin.ROLE, new double[]{0, 100, 0}, new float[]{10, semiY, 10}, 0f, 0f, clipTopY,
                new int[]{-10, 97, -10}, new int[]{10, 103, 10}, "ironstone", List.of());
    }

    @Test
    void aGossanAtTheGroundCropsOut() {
        // The generated ground at the centre within the slack above the top, or below it.
        assertTrue(GossanSkin.exposed(101, 101));
        assertTrue(GossanSkin.exposed(101 + GossanSkin.EXPOSURE_SLACK, 101));
        assertTrue(GossanSkin.exposed(90, 101), "the ground below the top: the body stands out of a lowered surface");
    }

    @Test
    void aGossanUnderCoverIsLeftAlone() {
        // The exit run's buried gossans lay 14, 24 and 37 blocks under the ground; the exposed ones within 1 to 3.
        for (double cover : new double[]{14, 24, 37})
            assertFalse(GossanSkin.exposed(101 + cover, 101), "cover " + cover);
        assertFalse(GossanSkin.exposed(101 + GossanSkin.EXPOSURE_SLACK + 0.5, 101));
        assertTrue(GossanSkin.EXPOSURE_SLACK > 3 && GossanSkin.EXPOSURE_SLACK < 14);
        // A body that was never clipped at a surface is not an outcrop.
        assertFalse(GossanSkin.exposed(101, Body.NO_CLIP));
    }

    @Test
    void theCapIsTheBodysThicknessAndAtLeastTwoBlocks() {
        assertEquals(4, GossanSkin.thickness(gossan(2f, 101)));
        assertEquals(9, GossanSkin.thickness(gossan(4.5f, 101)));
        // DEP-210: the deposits side never makes an exposed cap under two blocks; the skin keeps the floor too.
        assertEquals(2, GossanSkin.thickness(gossan(0.3f, 101)));
    }

    @Test
    void theCapFollowsTheGroundWhereverItStands() {
        // Ground above the flat body, in it and below it: the cap is always the top blocks of the ground.
        for (int ground : new int[]{60, 98, 101, 115}) {
            GossanSkin.Span s = GossanSkin.cap(ground, 4);
            assertEquals(ground, s.from());
            assertEquals(ground - 3, s.to());
        }
    }

    @Test
    void theFootprintIsTheBlanketsExtentAtItsMidPlane() {
        Body b = gossan(2f, 101);
        assertTrue(GossanSkin.inFootprint(b, C, 0, 0));
        assertTrue(GossanSkin.inFootprint(b, C, 6, -6));
        assertFalse(GossanSkin.inFootprint(b, C, 40, 0));
        assertFalse(GossanSkin.inFootprint(b, C, 0, -40));
    }
}
