// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

/**
 * What {@link NativeLoader#extract} produced: the verified library files to try, in order, and the
 * version and format {@code natives.json} expects them to report; or the failure that stopped
 * extraction. Extraction is separate from linking so tests can extract many times in one JVM while
 * a library is linked at most once per JVM (plan §6.5).
 */
public final class ExtractResult {

    /**
     * One library on disk.
     *
     * @param platform   its platform key
     * @param file       absolute path of the library file
     * @param sha256     its verified sha256 (lowercase hex); {@code null} for a
     *                   {@code -Ddeeptime.natives.path} override, which is not verified
     * @param rustTarget the Rust target triple natives.json records for it
     * @param written    true if this extraction wrote the file, false if it reused a verified copy
     */
    public record Extracted(String platform, Path file, String sha256, String rustTarget, boolean written) {}

    private final String platformKey;
    private final List<Extracted> candidates;
    private final String expectedVersion;
    private final String expectedFormat;
    private final boolean override;
    private final NativeStatus failure;

    private ExtractResult(String platformKey, List<Extracted> candidates, String expectedVersion,
            String expectedFormat, boolean override, NativeStatus failure) {
        this.platformKey = platformKey;
        this.candidates = List.copyOf(candidates);
        this.expectedVersion = expectedVersion;
        this.expectedFormat = expectedFormat;
        this.override = override;
        this.failure = failure;
    }

    static ExtractResult ok(String platformKey, List<Extracted> candidates, String expectedVersion,
            String expectedFormat, boolean override) {
        return new ExtractResult(platformKey, candidates, expectedVersion, expectedFormat, override, null);
    }

    static ExtractResult failed(NativeStatus failure) {
        return new ExtractResult(failure.platformKey(), List.of(), null, null, false, failure);
    }

    public boolean isOk() {
        return failure == null;
    }

    /** The failure that stopped extraction; empty when {@link #isOk()}. */
    public Optional<NativeStatus> failure() {
        return Optional.ofNullable(failure);
    }

    /** The detected platform key. */
    public String platformKey() {
        return platformKey;
    }

    /** Libraries to try in order: the detected libc's build first on Linux, then the other. */
    public List<Extracted> candidates() {
        return candidates;
    }

    /** The semver the library must report (natives.json {@code version}). */
    public String expectedVersion() {
        return expectedVersion;
    }

    /** The format version the library must report (natives.json {@code format_version}). */
    public String expectedFormat() {
        return expectedFormat;
    }

    /** True when {@code -Ddeeptime.natives.path} supplied the library and nothing was extracted. */
    public boolean isOverride() {
        return override;
    }

    /** The directory of the first candidate, or {@code null}. */
    public Path directory() {
        return candidates.isEmpty() ? null : candidates.get(0).file().getParent();
    }
}
