// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * {@link Craters}: {@code impacts/craters.json} as the core's bake writes it (P2-08,
 * {@code surface::impacts::present_craters}; FORMAT.md, "Impact craters").
 */
class CratersTest {

    private static final String TEXT = """
            {"version": 1, "t_myr": 0.0, "craters": [
              {"id": 73, "x": 2410.5, "z": -1033.0, "diameter_m": 6300.0, "morphology": "complex", "age_myr": 12.4,
               "depth_m": 255.0, "rim_m": 91.0, "relief_m": 180.0, "fate": "degraded", "breccia_m": 255.0, "melt_m": 0.0,
               "cover_m": 12.0, "in_surface": false},
              {"id": 90, "x": -100.0, "z": 20.0, "diameter_m": 42000.0, "morphology": "peak_ring", "age_myr": 140.0,
               "depth_m": 430.0, "rim_m": 220.0, "relief_m": 0.0, "fate": "buried", "breccia_m": 430.0, "melt_m": 55.0,
               "cover_m": 900.0, "in_surface": true}]}
            """;

    private static JsonObject json(String text) {
        return JsonParser.parseString(text).getAsJsonObject();
    }

    @Test
    void theListIsRead() {
        Craters c = Craters.parse(json(TEXT), "test");
        assertEquals(2, c.list().size());
        Craters.Crater a = c.list().get(0), b = c.list().get(1);
        assertEquals(73, a.id());
        assertEquals(2410.5, a.x());
        assertEquals(-1033.0, a.z());
        assertEquals(6300.0, a.diameterM());
        assertEquals("complex", a.morphology());
        assertEquals(12.4, a.ageMyr());
        assertEquals(255.0, a.depthM());
        assertEquals(91.0, a.rimM());
        assertEquals(180.0, a.reliefM());
        assertEquals(255.0, a.brecciaM());
        assertEquals(12.0, a.coverM());
        assertFalse(a.inSurface());
        assertTrue(a.shows(), "a degraded crater still shows");
        assertEquals("buried", b.fate());
        assertEquals(55.0, b.meltM());
        assertTrue(b.inSurface());
        assertFalse(b.shows(), "a buried crater does not");
    }

    @Test
    void aWorldWithoutTheFileHasNoCraters(@TempDir Path folder) throws IOException {
        assertSame(Craters.NONE, Craters.read(folder));
        assertTrue(Craters.NONE.list().isEmpty());
        Files.createDirectories(folder.resolve("impacts"));
        Files.writeString(folder.resolve(Craters.FILE), TEXT, StandardCharsets.UTF_8);
        assertEquals(2, Craters.read(folder).list().size());
    }

    @Test
    void anotherVersionOrABrokenFileIsRefused(@TempDir Path folder) throws IOException {
        WorldDataException version = assertThrows(WorldDataException.class,
                () -> Craters.parse(json(TEXT.replace("\"version\": 1", "\"version\": 2")), "test"));
        assertTrue(version.getMessage().contains("version 2"), version.getMessage());
        assertThrows(WorldDataException.class, () -> Craters.parse(json("{\"version\": 1}"), "test"));
        assertThrows(WorldDataException.class,
                () -> Craters.parse(json(TEXT.replace("\"fate\": \"degraded\", ", "")), "test"));
        Files.createDirectories(folder.resolve("impacts"));
        Files.writeString(folder.resolve(Craters.FILE), "not json", StandardCharsets.UTF_8);
        assertThrows(WorldDataException.class, () -> Craters.read(folder));
    }
}
