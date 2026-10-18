// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
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
    void aPlacersGravelKeepsItsPlaceUnderTheCap() {
        // A placer's cells are not repainted: the cap would leave its gold no loose block to sit in.
        Body placer = new Body(7, 2, 9, "placer", new double[]{0, 100, 0}, new float[]{10, 3, 10}, 0f, 0f, 103,
                new int[]{-10, 97, -10}, new int[]{10, 103, 10}, "gravel", List.of());
        List<Body> placers = List.of(placer);
        assertTrue(GossanSkin.covered(placers, C, 0, 100, 0));
        assertFalse(GossanSkin.covered(placers, C, 40, 100, 0), "outside the placer");
        assertFalse(GossanSkin.covered(placers, C, 0, 130, 0), "above the placer");
        assertFalse(GossanSkin.covered(List.of(), C, 0, 100, 0));
    }

    @Test
    void theFootprintIsTheBlanketsExtentAtItsMidPlane() {
        Body b = gossan(2f, 101);
        assertTrue(GossanSkin.inFootprint(b, C, 0, 0));
        assertTrue(GossanSkin.inFootprint(b, C, 6, -6));
        assertFalse(GossanSkin.inFootprint(b, C, 40, 0));
        assertFalse(GossanSkin.inFootprint(b, C, 0, -40));
    }

    // ---- DEP-212, DEP-213: a leached cap holds no ore -------------------------------------------

    private static final GossanSkin.Caps NO_CAPS = new GossanSkin.Caps(C, List.of(), (x, z) -> 100);

    /** The caps of {@code gossan(2f, ...)} over ground at y 100 everywhere. */
    private static GossanSkin.Caps capsOver() {
        return new GossanSkin.Caps(C, List.of(gossan(2f, 101)), (x, z) -> 100);
    }

    @Test
    void theWindowIsTheCapMovedByTheMarginEitherWay() {
        int m = GossanSkin.GROUND_MARGIN;
        GossanSkin.Span w = GossanSkin.window(100, 4);
        assertEquals(100 + m, w.from(), "the highest block a cap hangs from");
        assertEquals(100 - 3 - m, w.to(), "the lowest block of a cap hanging from the lowest");
    }

    @Test
    void theCapNeverLeavesItsWindow() {
        // Whatever the scanned top is (a ravine floor, a hill the model did not see), the blocks the skin
        // repaints are inside the window the ore feature leaves ore-free: the rule holds by construction.
        for (int thick : new int[]{2, 4, 9})
            for (int ground : new int[]{40, 100, 180})
                for (int top = ground - 40; top <= ground + 40; top++) {
                    GossanSkin.Span cap = GossanSkin.cap(GossanSkin.anchor(top, ground), thick);
                    GossanSkin.Span w = GossanSkin.window(ground, thick);
                    assertTrue(cap.from() <= w.from() && cap.to() >= w.to(), "top " + top + " ground " + ground + " thick " + thick);
                }
    }

    @Test
    void aTopWithinTheMarginIsTheCapsAnchorAndOneBeyondIsHeld() {
        int m = GossanSkin.GROUND_MARGIN;
        assertEquals(100, GossanSkin.anchor(100, 100));
        assertEquals(100 + m, GossanSkin.anchor(100 + m, 100));
        assertEquals(100 - m, GossanSkin.anchor(100 - m, 100));
        assertEquals(100 + m, GossanSkin.anchor(100 + m + 1, 100), "a hill above the model: the cap stays in the window");
        assertEquals(100 - m, GossanSkin.anchor(100 - m - 1, 100), "a hollow below the model");
    }

    @Test
    void aBlockInTheWindowOfAnExposedGossanHoldsNoOre() {
        GossanSkin.Caps caps = capsOver();
        GossanSkin.Span w = caps.window(0, 0);
        assertEquals(GossanSkin.window(100, 4), w);
        assertTrue(caps.holdsNoOre(w, 100), "the ground block");
        assertTrue(caps.holdsNoOre(w, w.from()), "the window's top");
        assertTrue(caps.holdsNoOre(w, w.to()), "the window's bottom");
        assertFalse(caps.holdsNoOre(w, w.from() + 1), "above the window");
        assertFalse(caps.holdsNoOre(w, w.to() - 1), "below the window: the rock under the cap keeps its ore");
    }

    @Test
    void aColumnOutsideTheFootprintHasNoWindow() {
        GossanSkin.Caps caps = capsOver();
        assertNull(caps.window(40, 0));
        assertNull(caps.window(0, -40));
        assertFalse(caps.holdsNoOre(caps.window(40, 0), 100));
        assertNull(NO_CAPS.window(0, 0));
        assertTrue(NO_CAPS.isEmpty());
        assertFalse(capsOver().isEmpty());
    }

    @Test
    void theWindowFollowsEachColumnsOwnGround() {
        // Ground 100 at x < 5 and 112 beyond: the window rises with it (the cap follows the ground).
        GossanSkin.Caps caps = new GossanSkin.Caps(C, List.of(gossan(2f, 101)), (x, z) -> x < 5 ? 100 : 112);
        assertEquals(GossanSkin.window(100, 4), caps.window(0, 0));
        assertEquals(GossanSkin.window(112, 4), caps.window(7, 0));
    }

    @Test
    void twoCapsOverOneColumnGiveTheirHull() {
        // A thin cap and a thick one over the same column: the window reaches the lowest bottom and the highest top.
        GossanSkin.Caps caps = new GossanSkin.Caps(C, List.of(gossan(1f, 101), gossan(4.5f, 101)), (x, z) -> 100);
        GossanSkin.Span thin = GossanSkin.window(100, 2), thick = GossanSkin.window(100, 9);
        GossanSkin.Span w = caps.window(0, 0);
        assertEquals(Math.max(thin.from(), thick.from()), w.from());
        assertEquals(Math.min(thin.to(), thick.to()), w.to());
        assertEquals(thick, w);
    }
}
