// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * Platform keys, libc detection from a synthetic ELF {@code PT_INTERP} (so the Linux path is
 * checked on any build host), and the {@code natives.json} reader. No native library is linked.
 */
class PlatformKeyTest {

    @TempDir
    Path dir;

    /** A minimal ELF file: header, one PT_LOAD and one PT_INTERP program header, the path string. */
    private static byte[] elf(boolean is64, boolean bigEndian, String interp) {
        ByteBuffer b = ByteBuffer.allocate(512).order(bigEndian ? ByteOrder.BIG_ENDIAN : ByteOrder.LITTLE_ENDIAN);
        b.put(new byte[] {0x7f, 'E', 'L', 'F', (byte) (is64 ? 2 : 1), (byte) (bigEndian ? 2 : 1), 1});
        int phoff = is64 ? 64 : 52;
        int phentsize = is64 ? 56 : 32;
        int strOff = phoff + 2 * phentsize;
        byte[] s = (interp + "\0").getBytes(StandardCharsets.UTF_8);
        if (is64) {
            b.putLong(0x20, phoff);
            b.putShort(0x36, (short) phentsize);
            b.putShort(0x38, (short) 2);
            b.putInt(phoff, 1); // PT_LOAD
            b.putInt(phoff + phentsize, 3); // PT_INTERP
            b.putLong(phoff + phentsize + 8, strOff);
            b.putLong(phoff + phentsize + 32, s.length);
        } else {
            b.putInt(0x1C, phoff);
            b.putShort(0x2A, (short) phentsize);
            b.putShort(0x2C, (short) 2);
            b.putInt(phoff, 1);
            b.putInt(phoff + phentsize, 3);
            b.putInt(phoff + phentsize + 4, strOff);
            b.putInt(phoff + phentsize + 16, s.length);
        }
        b.position(strOff);
        b.put(s);
        byte[] out = new byte[strOff + s.length];
        b.rewind();
        b.get(out);
        return out;
    }

    @Test
    void readsPtInterpOfElf64And32() throws Exception {
        Path musl = dir.resolve("musl");
        Files.write(musl, elf(true, false, "/lib/ld-musl-x86_64.so.1"));
        assertEquals(Optional.of("/lib/ld-musl-x86_64.so.1"), ElfInterp.read(musl));
        Path gnuBe = dir.resolve("gnu32be");
        Files.write(gnuBe, elf(false, true, "/lib/ld-linux.so.2"));
        assertEquals(Optional.of("/lib/ld-linux.so.2"), ElfInterp.read(gnuBe));
        Path notElf = dir.resolve("text");
        Files.writeString(notElf, "#!/bin/sh\n");
        assertEquals(Optional.empty(), ElfInterp.read(notElf));
        assertEquals(Optional.empty(), ElfInterp.read(dir.resolve("missing")));
    }

    @Test
    void libcAndCandidateOrder() {
        assertEquals(Optional.of("musl"), PlatformKey.libcOf(Optional.of("/lib/ld-musl-aarch64.so.1")));
        assertEquals(Optional.of("gnu"),
                PlatformKey.libcOf(Optional.of("/nix/store/abc-glibc-2.40/lib/ld-linux-x86-64.so.2")));
        assertEquals(Optional.empty(), PlatformKey.libcOf(Optional.of("/system/bin/linker64")));

        PlatformKey musl = new PlatformKey("linux", "aarch64", "musl");
        assertEquals("linux-aarch64-musl", musl.key());
        assertEquals(List.of("linux-aarch64-musl", "linux-aarch64-gnu"), musl.candidateKeys());
        PlatformKey unknown = new PlatformKey("linux", "x86_64", null);
        assertEquals("linux-x86_64-gnu", unknown.key());
        assertEquals(List.of("linux-x86_64-gnu", "linux-x86_64-musl"), unknown.candidateKeys());
        assertEquals(List.of("macos-aarch64"), new PlatformKey("macos", "aarch64", null).candidateKeys());

        assertEquals("macos", PlatformKey.normaliseOs("Mac OS X"));
        assertEquals("windows", PlatformKey.normaliseOs("Windows 11"));
        assertEquals("linux", PlatformKey.normaliseOs("Linux"));
        assertEquals("x86_64", PlatformKey.normaliseArch("amd64"));
        assertEquals("aarch64", PlatformKey.normaliseArch("arm64"));

        PlatformKey self = PlatformKey.detect();
        assertTrue(List.of("linux", "macos", "windows").contains(self.os()), self.toString());
        if (self.isLinux() && Files.exists(Path.of("/proc/self/exe")))
            assertTrue(self.libc() != null, "the JVM's own PT_INTERP should tell gnu from musl: " + self);
    }

    @Test
    void readsTheNativesJsonSchema() {
        String json = """
                {"schema": 1, "version": "0.0.1", "format_version": "0.0", "git_sha": "ab12cd3",
                 "libraries": [{"platform": "linux-x86_64-gnu", "file": "libdeeptime_jni.so",
                                "rust_target": "x86_64-unknown-linux-gnu", "glibc_floor": "2.17",
                                "sha256": "%s", "size": 42},
                               {"platform": "windows-x86_64", "file": "deeptime_jni.dll",
                                "rust_target": "x86_64-pc-windows-msvc", "glibc_floor": null,
                                "sha256": "%s", "size": 7}]}
                """.formatted("a".repeat(64), "B".repeat(64));
        NativesManifest m = NativesManifest.parse(json);
        assertEquals("0.0.1", m.version());
        assertEquals("0.0", m.formatVersion());
        assertEquals(2, m.libraries().size());
        NativesManifest.Library lib = m.library("linux-x86_64-gnu").orElseThrow();
        assertEquals("linux-x86_64-gnu/libdeeptime_jni.so", lib.resource());
        assertEquals("2.17", lib.glibcFloor());
        assertEquals(42, lib.size());
        assertEquals("b".repeat(64), m.library("windows-x86_64").orElseThrow().sha256());
        assertEquals(null, m.library("windows-x86_64").orElseThrow().glibcFloor());
        assertEquals(Map.of("k", List.of(1L, 2.5, true)), MiniJson.parse("{\"k\": [1, 2.5, true]}"));
        assertEquals("a\"é\n", MiniJson.parse("\"a\\\"\\u00e9\\n\""));
    }

    @Test
    void versionStringParses() {
        NativeLoader.VersionInfo v = NativeLoader.VersionInfo
                .parse("0.0.1+ab12cd3;format=0.0;target=x86_64-unknown-linux-gnu").orElseThrow();
        assertEquals("0.0.1", v.semver());
        assertEquals("ab12cd3", v.gitSha());
        assertEquals("0.0", v.format());
        assertEquals("x86_64-unknown-linux-gnu", v.target());
        assertTrue(NativeLoader.VersionInfo.parse("garbage").isEmpty());
        assertTrue(NativeLoader.isLibcError(new UnsatisfiedLinkError("version `GLIBC_2.34' not found")));
        assertTrue(NativeLoader.isLibcError(new UnsatisfiedLinkError("Error loading shared library ld-linux-x86-64.so.2")));
        assertTrue(!NativeLoader.isLibcError(new UnsatisfiedLinkError("wrong ELF class: ELFCLASS32")));
    }
}
