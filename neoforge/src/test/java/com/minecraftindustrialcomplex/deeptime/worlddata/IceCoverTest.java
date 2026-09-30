// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The ice cover (Ben, 2026-09-30, "Force snow/ice cover"): the layer the bake writes from the climate
 * stage's equilibrium line, the table recorded beside it, and the block mix of a cover.
 */
class IceCoverTest {

    private static final String RECORD = """
            {"bake": {"climate": {"ice_cover": {"step_m": 160.0, "max_blocks": 12, "snow_blocks": 3,
              "blue_ice_from_blocks": 9, "blue_ice_blocks": 2, "snow": "minecraft:snow_block",
              "ice": "minecraft:packed_ice", "deep_ice": "minecraft:blue_ice", "cells": 7}}}}""";

    private static IceCover.Spec spec() {
        return IceCover.Spec.read(JsonParser.parseString(RECORD).getAsJsonObject());
    }

    private static String mix(IceCover.Spec s, int total) {
        StringBuilder out = new StringBuilder();
        for (int i = 0; i < total; i++)
            out.append("SID".charAt(s.kind(i, total)));
        return out.toString();
    }

    @Test
    void theRecordIsReadAndAMissingOneIsNull() {
        IceCover.Spec s = spec();
        assertEquals(12, s.maxBlocks());
        assertEquals("minecraft:blue_ice", s.deepIce());
        assertEquals(160.0, s.stepM());
        assertNull(IceCover.Spec.read(new JsonObject()));
        assertNull(IceCover.Spec.read(JsonParser.parseString("{\"bake\": {\"climate\": {}}}").getAsJsonObject()));
        assertNull(IceCover.Spec.read(JsonParser.parseString("{\"bake\": {\"climate\": {\"ice_cover\": null}}}").getAsJsonObject()));
    }

    /** Snow on top, packed ice between, blue ice at the base of a thick cover. */
    @Test
    void theBlockMixOfACover() {
        IceCover.Spec s = spec();
        assertEquals("S", mix(s, 1));
        assertEquals("SSS", mix(s, 3));
        assertEquals("SSSI", mix(s, 4));
        assertEquals("SSSIIIII", mix(s, 8));
        assertEquals("SSSIIIIDD", mix(s, 9), "a cover of 9 blocks ends in blue ice");
        assertEquals("SSSIIIIIIIDD", mix(s, 12));
        assertEquals("minecraft:snow_block", s.blockId(IceCover.Spec.SNOW));
        assertEquals("minecraft:packed_ice", s.blockId(IceCover.Spec.ICE));
        assertEquals("minecraft:blue_ice", s.blockId(IceCover.Spec.DEEP_ICE));
        // Snow takes precedence where the blue ice would overlap it.
        IceCover.Spec thin = new IceCover.Spec(50, 5, 4, 2, 3, "a:s", "a:i", "a:d");
        assertEquals("SSSS", mix(thin, 4));
        assertEquals("SSSSD", mix(thin, 5));
    }

    /** The baked 4k planet has the layer, the record and a cover that reads the layer bilinearly. */
    @Test
    void theBakedPlanetHasItsCover() {
        WorldData wd = WorldData.open(Fixtures.world("planet4k"));
        Optional<IceCover> cover = wd.iceCover();
        assertTrue(cover.isPresent(), "a baked planet with a climate stage has an ice cover");
        assertEquals(12, cover.get().spec().maxBlocks());
        RasterSampler layer = wd.layer(IceCover.LAYER);
        int thickest = 0;
        var g = layer.desc().grid();
        for (int j = 0; j < g.h(); j++) {
            for (int i = 0; i < g.w(); i++) {
                int v = (int) layer.get(i, j);
                thickest = Math.max(thickest, v);
                if (v > 0) {
                    // At a cell's own block the bilinear read is the cell.
                    int x = (int) (g.originBlockX() + (long) i * g.r()), z = (int) (g.originBlockZ() + (long) j * g.r());
                    assertEquals(v, cover.get().blocks(x, z), "cell (" + i + ", " + j + ")");
                }
            }
        }
        assertTrue(thickest <= cover.get().spec().maxBlocks(), "no thicker than the ceiling: " + thickest);
        // (The 150 Myr test planet may have no ground above its equilibrium line: nothing to read then.)
    }

    @Test
    void aSyntheticWorldHasNone() {
        assertTrue(WorldData.open(Fixtures.world("wrap1024")).iceCover().isEmpty());
    }
}
