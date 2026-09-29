// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.preset;

import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Base64;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.zip.Deflater;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * P1-13: share codes in Java ({@code java.util.zip}, no natives) against the Rust encoder. The
 * fixture {@code crates/deeptime-core/tests/share-codes.json} holds canonical JSON with the code
 * Rust writes and the code Java writes; both suites decode both. This test also writes Java's
 * codes to {@code build/share-codes-java.json} ({@code tools/dev/share_code_fixture.py} copies
 * them into the fixture).
 */
class ShareCodeTest {

    private static JsonObject fixture() throws IOException {
        Path f = Path.of(System.getProperty("deeptime.repoRoot"), "crates", "deeptime-core", "tests", "share-codes.json");
        return Presets.parseObject(Files.readString(f, StandardCharsets.UTF_8));
    }

    @Test
    void fixtureCodesDecodeInJava() throws IOException {
        Map<String, String> mine = new LinkedHashMap<>();
        var cases = fixture().getAsJsonArray("cases");
        for (JsonElement e : cases) {
            JsonObject c = e.getAsJsonObject();
            mine.put(c.get("name").getAsString(), ShareCode.encode(c.get("json").getAsString()));
        }
        Files.createDirectories(Path.of("build"));
        Files.writeString(Path.of("build", "share-codes-java.json"),
                new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create().toJson(mine) + "\n");

        assertTrue(cases.size() >= 3);
        for (JsonElement e : cases) {
            JsonObject c = e.getAsJsonObject();
            String name = c.get("name").getAsString();
            String json = c.get("json").getAsString();
            assertEquals(json, Presets.canonical(json), name + ": Java's canonical JSON is Rust's");
            assertEquals(json, ShareCode.decode(c.get("rust").getAsString()), name + ": Rust's code decodes in Java");
            assertEquals(json, ShareCode.decode(c.get("java").getAsString()), name + ": the fixture's Java code decodes");
            assertEquals(json, ShareCode.decode(mine.get(name)), name + ": Java round trip");
        }
    }

    @Test
    void codesAreCanonicalAndRoundTrip() {
        String a = ShareCode.encode("{ \"b\": 1, \"a\": {\"y\": [1, 2.5], \"x\": \"é—\" } }");
        String b = ShareCode.encode("{\"a\":{\"x\":\"é—\",\"y\":[1,2.5]},\"b\":1}");
        assertEquals(a, b);
        assertTrue(a.startsWith(ShareCode.PREFIX));
        assertFalse(a.contains("=") || a.contains("+") || a.contains("/"));
        assertEquals("{\"a\":{\"x\":\"é—\",\"y\":[1,2.5]},\"b\":1}", ShareCode.decode(a));
        // Pasted across lines.
        assertEquals(ShareCode.decode(a), ShareCode.decode("  " + a.substring(0, 10) + "\n  " + a.substring(10) + "\n"));
    }

    @Test
    void badCodesSayWhy() {
        assertTrue(message("hello").contains("starts with DT1."));
        assertTrue(message("DT2.abc").contains("DT2 codes are not supported"));
        assertTrue(message("DT1.!!").contains("base64url"));
        assertTrue(message("DT1.Zm9vYmFy").contains("deflate"));
        assertTrue(message(code("not json".getBytes(StandardCharsets.UTF_8))).contains("does not hold JSON"));
        assertTrue(message(code(" ".repeat(2 << 20).getBytes(StandardCharsets.UTF_8))).contains("more than"));
        assertTrue(ShareCode.isShareCode("DT1.x") && ShareCode.isShareCode(" DT12.x"));
        assertFalse(ShareCode.isShareCode("DT.x") || ShareCode.isShareCode("earthlike_quick_16k") || ShareCode.isShareCode(null));
    }

    private static String message(String code) {
        PresetException e = assertThrows(PresetException.class, () -> ShareCode.decode(code));
        assertTrue(e.getMessage().startsWith("share code: "), e.getMessage());
        return e.getMessage();
    }

    private static String code(byte[] plain) {
        Deflater d = new Deflater(9, true);
        d.setInput(plain);
        d.finish();
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        byte[] buf = new byte[8192];
        while (!d.finished())
            out.write(buf, 0, d.deflate(buf));
        d.end();
        return ShareCode.PREFIX + Base64.getUrlEncoder().withoutPadding().encodeToString(out.toByteArray());
    }
}
