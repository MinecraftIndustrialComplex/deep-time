// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;

/**
 * {@code natives/deeptime/natives.json} in the mod jar, in the schema of plan §6.5 (schema 2):
 *
 * <pre>{@code
 * { "schema": 2, "version": "0.0.1", "format_version": "0.0", "git_sha": "ab12cd3",
 *   "libraries": [ { "platform": "linux-x86_64-gnu", "variant": "baseline", "file": "libdeeptime_jni.so",
 *                    "rust_target": "x86_64-unknown-linux-gnu", "glibc_floor": "2.17",
 *                    "sha256": "<64 lowercase hex>", "size": 1234567 },
 *                  { "platform": "linux-x86_64-gnu", "variant": "x86-64-v3", ... } ],
 *   "probes":    [ { "platform": "linux-x86_64-gnu", "file": "libdeeptime_cpuprobe.so",
 *                    "rust_target": "x86_64-unknown-linux-gnu", "glibc_floor": "2.17",
 *                    "sha256": "<64 lowercase hex>", "size": 12345 } ] }
 * }</pre>
 *
 * Every platform has a {@code baseline} library; each x86-64 platform also an {@code x86-64-v3} one
 * and a CPU probe ({@link CpuProbe}) that tells the loader whether the CPU can run it (decisions log
 * 2026-10-18). A library sits in the jar at {@code natives/deeptime/<platform>/<variant>/<file>}, a
 * probe at {@code natives/deeptime/<platform>/probe/<file>}.
 */
record NativesManifest(int schema, String version, String formatVersion, String gitSha, List<Library> libraries,
        List<Library> probes) {

    static final int SCHEMA = 2;
    /** The build for the platform's baseline CPU; every platform has one. */
    static final String BASELINE = "baseline";
    /** The build for the x86-64-v3 level ({@code -C target-cpu=x86-64-v3}); x86-64 platforms only. */
    static final String X86_64_V3 = "x86-64-v3";
    /** The {@link Library#variant} of a CPU probe. */
    static final String PROBE = "probe";

    record Library(String platform, String variant, String file, String rustTarget, String glibcFloor, String sha256,
            long size) {

        /** Resource path inside the jar, relative to {@code natives/deeptime/}. */
        String resource() {
            return platform + "/" + variant + "/" + file;
        }

        boolean isV3() {
            return X86_64_V3.equals(variant);
        }
    }

    Optional<Library> library(String platform, String variant) {
        return libraries.stream().filter(l -> l.platform().equals(platform) && l.variant().equals(variant)).findFirst();
    }

    Optional<Library> probe(String platform) {
        return probes.stream().filter(l -> l.platform().equals(platform)).findFirst();
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
        List<Library> libs = libraries(o.get("libraries"), "libraries", false);
        List<Library> probes = o.containsKey("probes") ? libraries(o.get("probes"), "probes", true) : List.of();
        return new NativesManifest(schema, string(o, "version"), string(o, "format_version"),
                optString(o, "git_sha"), libs, probes);
    }

    @SuppressWarnings("unchecked")
    private static List<Library> libraries(Object list, String name, boolean probes) {
        if (!(list instanceof List<?> l))
            throw new IllegalArgumentException("natives.json: " + name + " is not an array");
        List<Library> out = new ArrayList<>();
        for (Object e : l) {
            if (!(e instanceof Map<?, ?> lm))
                throw new IllegalArgumentException("natives.json: an entry of " + name + " is not an object");
            Map<String, Object> lo = (Map<String, Object>) lm;
            String sha = string(lo, "sha256").toLowerCase(Locale.ROOT);
            if (!sha.matches("[0-9a-f]{64}"))
                throw new IllegalArgumentException("natives.json: bad sha256 " + sha);
            String variant = probes ? PROBE : string(lo, "variant");
            if (!probes && !variant.equals(BASELINE) && !variant.equals(X86_64_V3))
                throw new IllegalArgumentException("natives.json: unknown variant " + variant);
            out.add(new Library(string(lo, "platform"), variant, string(lo, "file"), string(lo, "rust_target"),
                    optString(lo, "glibc_floor"), sha, number(lo, "size")));
        }
        return List.copyOf(out);
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
