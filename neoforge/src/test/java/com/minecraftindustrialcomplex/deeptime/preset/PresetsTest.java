// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.preset;

import com.google.gson.JsonObject;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** P1-13: built-in presets in the jar, preset arguments, header fields and canonical JSON. */
class PresetsTest {

    @Test
    void builtinsShipInTheJarAndMatchTheCore() throws IOException {
        Path src = Path.of(System.getProperty("deeptime.repoRoot"), "crates", "deeptime-core", "presets");
        List<String> onDisk;
        try (Stream<Path> s = Files.list(src)) {
            onDisk = s.map(p -> p.getFileName().toString()).filter(n -> n.endsWith(".json"))
                    .map(n -> n.substring(0, n.length() - 5)).sorted().toList();
        }
        assertEquals(Presets.BUILTIN.stream().sorted().toList(), onDisk, "Presets.BUILTIN lists every core preset");
        for (String name : Presets.BUILTIN) {
            String json = Presets.builtin(name).orElseThrow();
            assertEquals(Files.readString(src.resolve(name + ".json")), json, name + " is the core's file");
            Presets.Info info = Presets.info(Presets.parseObject(json));
            assertEquals(name, info.name().orElseThrow());
            assertFalse(info.placeholder(), name + " is a simulated planet since P1-08");
            assertTrue(info.timeline().isEmpty(), name + " names no timeline (no default story)");
            assertEquals(name.endsWith("_32k") ? 32_768 : 16_384, info.circumference().getAsInt(), name);
            // Every planet is the cube world (decisions log 2026-10-24): the level type's x period is 4s.
            assertEquals("cube", info.projection(), name);
            assertEquals(name.endsWith("_32k") ? 29_696 : 15_360, info.worldCircumference().getAsInt(), name);
        }
        assertTrue(Presets.builtin("earthlike_huge_64k").isEmpty());
    }

    @Test
    void argumentsResolveToCodesNamesAndFiles(@org.junit.jupiter.api.io.TempDir Path dir) throws IOException {
        String quick = Presets.builtin("earthlike_quick_16k").orElseThrow();
        assertEquals(quick, Presets.resolve(" earthlike_quick_16k ", dir).json());
        String code = ShareCode.encode(quick);
        Presets.Resolved r = Presets.resolve(code, dir);
        assertTrue(r.label().startsWith("share code DT1."), r.label());
        assertEquals(Presets.canonical(quick), r.json());
        Files.writeString(dir.resolve("mine.json"), "{\"dummy\":{\"steps\":1,\"sleep_ms\":0}}");
        Path other = Files.createDirectories(dir.resolve("other"));
        assertEquals("{\"dummy\":{\"steps\":1,\"sleep_ms\":0}}", Presets.resolve("mine.json", other, dir).json());
        assertTrue(Presets.resolve("mine.json", other, dir).label().endsWith("mine.json"));
        PresetException bad = assertThrows(PresetException.class, () -> Presets.resolve("earthlike_huge_64k", dir));
        assertTrue(bad.getMessage().contains("earthlike_quick_16k"), bad.getMessage());
        assertTrue(assertThrows(PresetException.class, () -> Presets.resolve("nope/missing.json", dir)).getMessage()
                .contains("does not exist"));
        assertThrows(PresetException.class, () -> Presets.resolve("", dir));
    }

    @Test
    void timelineIsWrittenAsRustWritesIt() {
        JsonObject quick = Presets.parseObject(Presets.builtin("earthlike_quick_16k").orElseThrow());
        String json = Presets.withTimeline(quick, new Presets.Timeline("scattered_continents", 800));
        assertTrue(json.contains("\"timeline\":{\"length_myr\":800.0,\"start\":\"scattered_continents\"}"), json);
        Presets.Info info = Presets.info(Presets.parseObject(json));
        assertEquals(new Presets.Timeline("scattered_continents", 800.0), info.timeline().orElseThrow());
        assertEquals(List.of(), info.timeline().get().problems("timeline"));
        assertEquals(2, new Presets.Timeline("pangaea", 0).problems("timeline").size());
        assertEquals(1, new Presets.Timeline("random", 5000).problems("timeline").size());
        assertFalse(quick.has("timeline"), "withTimeline copies");
    }

    /** The same text Rust's `canonical_numbers_follow_serde` pins. */
    @Test
    void canonicalJsonIsSerdes() {
        assertEquals("{\"a\":1000.0,\"b\":1e-7,\"c\":0.001,\"d\":1.5e+300,\"e\":123.456,\"f\":18446744073709551615,"
                        + "\"g\":-9223372036854775808,\"h\":1.8446744073709552e+19,\"i\":1e+16,\"j\":1.2345678901234568e+17,"
                        + "\"k\":0.00001,\"l\":1e-6,\"m\":-2.5,\"n\":0.0,\"o\":[true,false,null],\"p\":1000000000000000.0}",
                Presets.canonical("{\"o\":[true,false,null],\"b\":1.0E-7,\"a\":1e3,\"c\":0.001,\"d\":1.5e300,\"e\":123.456,"
                        + "\"f\":18446744073709551615,\"g\":-9223372036854775808,\"h\":18446744073709551616,\"i\":1e16,"
                        + "\"j\":123456789012345680.0,\"k\":0.00001,\"l\":0.000001,\"m\":-2.50,\"n\":0e5,\"p\":1e15}"));
        assertEquals("{\"s\":\"q\\\" b\\\\ \\b\\t\\n\\f\\r\\u0001 é — \u2028\"}",
                Presets.canonical("{\"s\":\"q\\\" b\\\\ \\b\\t\\n\\f\\r\\u0001 \\u00e9 — \\u2028\"}"));
        assertTrue(Presets.compareCodePoints("\uFFFF", "\uD83D\uDE00") < 0, "code point order, as UTF-8 bytes sort");
        assertThrows(PresetException.class, () -> Presets.parseStrict("{\"a\":1,}"));
        assertThrows(PresetException.class, () -> Presets.parseStrict("{a:1}"));
        assertThrows(PresetException.class, () -> Presets.parseStrict("{} {}"));
    }
}
