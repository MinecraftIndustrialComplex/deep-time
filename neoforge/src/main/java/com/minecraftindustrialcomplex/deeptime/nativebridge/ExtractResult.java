// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.file.Path;
import java.util.List;
import java.util.Optional;

/**
 * What {@link NativeLoader#extract} produced: the verified library files to try, in order, the
 * build it chose for this CPU and why, and the version and format {@code natives.json} expects the
 * library to report; or the failure that stopped extraction. Extraction is separate from linking so
 * tests can extract many times in one JVM while a library is linked at most once per JVM (plan
 * §6.5).
 */
public final class ExtractResult {

    /**
     * One library on disk.
     *
     * @param platform   its platform key
     * @param variant    its build: {@code baseline} or {@code x86-64-v3}; {@code null} for a
     *                   {@code -Ddeeptime.natives.path} override
     * @param file       absolute path of the library file
     * @param sha256     its verified sha256 (lowercase hex); {@code null} for a
     *                   {@code -Ddeeptime.natives.path} override, which is not verified
     * @param rustTarget the Rust target triple natives.json records for it
     * @param written    true if this extraction wrote the file, false if it reused a verified copy
     */
    public record Extracted(String platform, String variant, Path file, String sha256, String rustTarget,
            boolean written) {

        /** True for the x86-64-v3 build, which falls back to the baseline build when it cannot load. */
        public boolean isV3() {
            return NativesManifest.X86_64_V3.equals(variant);
        }
    }

    private final String platformKey;
    private final List<Extracted> candidates;
    private final String expectedVersion;
    private final String expectedFormat;
    private final boolean override;
    private final String variant;
    private final String variantReason;
    private final Path probeFile;
    private final NativeStatus failure;

    private ExtractResult(String platformKey, List<Extracted> candidates, String expectedVersion,
            String expectedFormat, boolean override, String variant, String variantReason, Path probeFile,
            NativeStatus failure) {
        this.platformKey = platformKey;
        this.candidates = List.copyOf(candidates);
        this.expectedVersion = expectedVersion;
        this.expectedFormat = expectedFormat;
        this.override = override;
        this.variant = variant;
        this.variantReason = variantReason;
        this.probeFile = probeFile;
        this.failure = failure;
    }

    static ExtractResult ok(String platformKey, List<Extracted> candidates, String expectedVersion,
            String expectedFormat, boolean override, String variant, String variantReason, Path probeFile) {
        return new ExtractResult(platformKey, candidates, expectedVersion, expectedFormat, override, variant,
                variantReason, probeFile, null);
    }

    static ExtractResult failed(NativeStatus failure) {
        return new ExtractResult(failure.platformKey(), List.of(), null, null, false, null, null, null, failure);
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

    /**
     * Libraries to try in order: per libc (the detected one first on Linux, then the other), the
     * x86-64-v3 build when it was chosen and could be extracted, then the baseline build.
     */
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

    /** The build chosen for this CPU, {@code baseline} or {@code x86-64-v3}; {@code null} for an override. */
    public String variant() {
        return variant;
    }

    /** Why {@link #variant} was chosen (the CPU probe's answer, a forced choice, a fallback). */
    public String variantReason() {
        return variantReason;
    }

    /** The CPU probe library this JVM loaded, when it was extracted under this game directory. */
    public Path probeFile() {
        return probeFile;
    }

    /** The directory of the first candidate, or {@code null}. */
    public Path directory() {
        return candidates.isEmpty() ? null : candidates.get(0).file().getParent();
    }
}
