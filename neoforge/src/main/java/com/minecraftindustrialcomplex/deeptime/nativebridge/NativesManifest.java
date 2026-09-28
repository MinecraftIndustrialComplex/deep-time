// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;

/**
 * {@code natives/deeptime/natives.json} in the mod jar, in the schema of plan §6.5 (schema 1):
 *
 * <pre>{@code
 * { "schema": 1, "version": "0.0.1", "format_version": "0.0", "git_sha": "ab12cd3",
 *   "libraries": [ { "platform": "linux-x86_64-gnu", "file": "libdeeptime_jni.so",
 *                    "rust_target": "x86_64-unknown-linux-gnu", "glibc_floor": "2.17",
 *                    "sha256": "<64 lowercase hex>", "size": 1234567 } ] }
 * }</pre>
 *
 * Each library sits in the jar at {@code natives/deeptime/<platform>/<file>}.
 */
record NativesManifest(int schema, String version, String formatVersion, String gitSha, List<Library> libraries) {

    static final int SCHEMA = 1;

    record Library(String platform, String file, String rustTarget, String glibcFloor, String sha256, long size) {

        /** Resource path inside the jar, relative to {@code natives/deeptime/}. */
        String resource() {
            return platform + "/" + file;
        }
    }

    Optional<Library> library(String platform) {
        return libraries.stream().filter(l -> l.platform().equals(platform)).findFirst();
    }

    @SuppressWarnings("unchecked")
    static NativesManifest parse(String json) {
        Object root = MiniJson.parse(json);
        if (!(root instanceof Map<?, ?> m))
            throw new IllegalArgumentException("natives.json: top level is not an object");
        Map<String, Object> o = (Map<String, Object>) m;
        int schema = (int) number(o, "schema");
        if (schema != SCHEMA)
            throw new IllegalArgumentException("natives.json: schema " + schema + ", this loader reads " + SCHEMA);
        List<Library> libs = new ArrayList<>();
        Object list = o.get("libraries");
        if (!(list instanceof List<?> l))
            throw new IllegalArgumentException("natives.json: libraries is not an array");
        for (Object e : l) {
            if (!(e instanceof Map<?, ?> lm))
                throw new IllegalArgumentException("natives.json: a library is not an object");
            Map<String, Object> lo = (Map<String, Object>) lm;
            String sha = string(lo, "sha256").toLowerCase(Locale.ROOT);
            if (!sha.matches("[0-9a-f]{64}"))
                throw new IllegalArgumentException("natives.json: bad sha256 " + sha);
            libs.add(new Library(string(lo, "platform"), string(lo, "file"), string(lo, "rust_target"),
                    optString(lo, "glibc_floor"), sha, number(lo, "size")));
        }
        return new NativesManifest(schema, string(o, "version"), string(o, "format_version"),
                optString(o, "git_sha"), List.copyOf(libs));
    }

    private static String string(Map<String, Object> o, String key) {
        if (o.get(key) instanceof String s)
            return s;
        throw new IllegalArgumentException("natives.json: " + key + " is missing or not a string");
    }

    private static String optString(Map<String, Object> o, String key) {
        return o.get(key) instanceof String s ? s : null;
    }

    private static long number(Map<String, Object> o, String key) {
        if (o.get(key) instanceof Long n)
            return n;
        throw new IllegalArgumentException("natives.json: " + key + " is missing or not an integer");
    }
}
