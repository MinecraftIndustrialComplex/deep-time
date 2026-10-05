// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** P1-13: {@code config/deeptime/server.json}, its template, and the first-start progress line. */
class ServerConfigTest {

    private static List<String> problems(String json) {
        return assertThrows(ServerConfig.Invalid.class, () -> ServerConfig.parse(json)).problems();
    }

    @Test
    void templateIsValidJsonThatStartsNothing() {
        // Valid JSON (review 2, gap 21), but no default story: preset and timeline are empty.
        Presets.parseObject(ServerConfig.template());
        List<String> p = problems(ServerConfig.template());
        assertEquals(3, p.size(), p.toString());
        assertTrue(p.get(0).startsWith("preset: empty"), p.toString());
        assertTrue(p.get(1).startsWith("timeline.start: empty"), p.toString());
        assertTrue(p.get(2).startsWith("timeline.length_myr: 0"), p.toString());
    }

    @Test
    void aFilledTemplateParses() throws ServerConfig.Invalid {
        String filled = ServerConfig.template()
                .replace("\"preset\": \"\"", "\"preset\": \"earthlike_quick_16k\"")
                .replace("\"start\": \"\"", "\"start\": \"supercontinent\"")
                .replace("\"length_myr\": 0", "\"length_myr\": 1000");
        ServerConfig c = ServerConfig.parse(filled);
        assertEquals("earthlike_quick_16k", c.preset());
        assertEquals(Optional.of(new Presets.Timeline("supercontinent", 1000)), c.timeline());
        assertEquals(4L << 30, c.memoryCapBytes());
        assertEquals(0, c.threads());
    }

    @Test
    void minimalConfigAndItsProblems() throws ServerConfig.Invalid {
        ServerConfig c = ServerConfig.parse("{\"preset\": \"DT1.abc\", \"memory_cap_gib\": 12.5, \"threads\": 6}");
        assertEquals(Optional.empty(), c.timeline(), "a share code may carry the timeline");
        assertEquals((long) (12.5 * (1L << 30)), c.memoryCapBytes());
        assertEquals(6, c.threads());

        assertEquals(List.of("preset: missing"), problems("{}"));
        assertTrue(problems("[1]").get(0).contains("one JSON object"));
        assertTrue(problems("{\"preset\": \"x\",}").get(0).startsWith("not valid JSON"));
        assertTrue(problems("{\"preset\": \"x\", \"memory_cap\": 8}").get(0).startsWith("memory_cap: unknown key"));
        assertTrue(problems("{\"preset\": \"x\", \"memory_cap_gib\": 0}").get(0).startsWith("memory_cap_gib"));
        assertTrue(problems("{\"preset\": \"x\", \"threads\": 1.5}").get(0).startsWith("threads"));
        assertTrue(problems("{\"preset\": \"x\", \"timeline\": {\"start\": \"random\"}}").get(0)
                .startsWith("timeline.length_myr: missing"));
        assertTrue(problems("{\"preset\": \"x\", \"timeline\": {\"start\": \"random\", \"length_myr\": 5, \"seed\": 1}}")
                .get(0).startsWith("timeline.seed: unknown key"));
    }

    @Test
    void progressLineFollowsThePlan() {
        var p = new SimSession.Progress(SimSession.Progress.RUNNING, 0, 2, 370, 185, 312_000_000L, 72_000, 160_000,
                (long) (1.3 * (1L << 30)), 0, 3, 40);
        assertEquals("dummy_drift 37% (−312 Ma) · total 18% · 1m12s elapsed · ~2m40s left · native 1.3/4.0 GiB",
                FirstStartGenerator.progressLine(p, List.of("dummy_drift", "dummy_erode"), 4L << 30));
        var done = new SimSession.Progress(SimSession.Progress.DONE, 2, 2, 1000, 1000, 0, 3_723_000, 0, 0, 0, 9, 40);
        assertEquals("writing world data · total 100% · 1h02m elapsed · native 0.0/4.0 GiB",
                FirstStartGenerator.progressLine(done, List.of(), 4L << 30));
        assertEquals("0 Ma", FirstStartGenerator.ma(0));
        assertEquals("deeptime:planet_32k", FirstStartGenerator.levelTypeFor(26_112, "petroff_guyou"));
        assertEquals("deeptime:planet", FirstStartGenerator.levelTypeFor(13_312, "petroff_guyou"));
        assertEquals("deeptime:planet_lambert_32k", FirstStartGenerator.levelTypeFor(32_768, "lambert"));
        assertEquals("13,312-block Petroff–Guyou world", FirstStartGenerator.describe(13_312, "petroff_guyou"));
    }
}
