// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.util.List;
import java.util.Locale;
import java.util.Optional;

/**
 * The platform this JVM runs on, as the keys of plan §6.4: {@code linux-x86_64-gnu},
 * {@code linux-aarch64-musl}, {@code windows-x86_64}, {@code macos-aarch64}, …
 *
 * @param os   {@code linux}, {@code windows}, {@code macos}, or the raw {@code os.name} otherwise
 * @param arch {@code x86_64}, {@code aarch64}, or the raw {@code os.arch} otherwise
 * @param libc {@code gnu} or {@code musl} on Linux when the JVM's loader tells; {@code null} when
 *             unknown or not Linux
 */
public record PlatformKey(String os, String arch, String libc) {

    public static final String GNU = "gnu";
    public static final String MUSL = "musl";

    /** Detects the running JVM's platform; libc from {@link ElfInterp#ofSelf()} on Linux. */
    public static PlatformKey detect() {
        String os = normaliseOs(System.getProperty("os.name", ""));
        String arch = normaliseArch(System.getProperty("os.arch", ""));
        String libc = null;
        if (os.equals("linux"))
            libc = libcOf(ElfInterp.ofSelf()).orElse(null);
        return new PlatformKey(os, arch, libc);
    }

    static String normaliseOs(String osName) {
        String n = osName.toLowerCase(Locale.ROOT);
        if (n.startsWith("linux"))
            return "linux";
        if (n.startsWith("windows"))
            return "windows";
        if (n.startsWith("mac") || n.contains("darwin") || n.contains("os x"))
            return "macos";
        return n.replace(' ', '_');
    }

    static String normaliseArch(String osArch) {
        String a = osArch.toLowerCase(Locale.ROOT);
        return switch (a) {
            case "amd64", "x86_64", "x86-64", "x64" -> "x86_64";
            case "aarch64", "arm64" -> "aarch64";
            default -> a;
        };
    }

    /** {@code gnu} for an {@code ld-linux*} loader, {@code musl} for {@code ld-musl*}. */
    static Optional<String> libcOf(Optional<String> interp) {
        return interp.map(i -> i.substring(i.lastIndexOf('/') + 1)).flatMap(name -> {
            if (name.startsWith("ld-musl"))
                return Optional.of(MUSL);
            if (name.startsWith("ld-linux") || name.startsWith("ld64.so") || name.startsWith("ld.so"))
                return Optional.of(GNU);
            return Optional.empty();
        });
    }

    public boolean isLinux() {
        return os.equals("linux");
    }

    /** The key of the preferred build: on Linux with unknown libc, the gnu build. */
    public String key() {
        return isLinux() ? keyFor(libc == null ? GNU : libc) : os + "-" + arch;
    }

    private String keyFor(String c) {
        return os + "-" + arch + "-" + c;
    }

    /**
     * Keys to try, in order (plan §6.5 step 1): on Linux the detected libc first, then the other
     * one (so {@link FailureClass#LIBC_MISMATCH} is reported only after both were tried); with
     * unknown libc, gnu then musl. Elsewhere just {@link #key()}.
     */
    public List<String> candidateKeys() {
        if (!isLinux())
            return List.of(key());
        return MUSL.equals(libc) ? List.of(keyFor(MUSL), keyFor(GNU)) : List.of(keyFor(GNU), keyFor(MUSL));
    }

    @Override
    public String toString() {
        return key() + (isLinux() && libc == null ? " (libc unknown)" : "");
    }
}
