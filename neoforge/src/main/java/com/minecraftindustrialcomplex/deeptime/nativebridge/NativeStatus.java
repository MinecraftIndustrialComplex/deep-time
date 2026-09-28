// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.file.Path;
import java.util.Objects;

/**
 * The outcome of loading the native library (plan §6.5, §6.6): either OK, with what the library
 * reported, or a {@link FailureClass} with its cause and guidance. Immutable; JDK types only.
 */
public final class NativeStatus {

    private final FailureClass failure;
    private final String platformKey;
    private final Path attemptedPath;
    private final Throwable cause;
    private final String guidance;
    private final String version;
    private final String format;
    private final String target;
    private final String gitSha;
    private final long selfTest;

    private NativeStatus(FailureClass failure, String platformKey, Path attemptedPath, Throwable cause,
            String guidance, String version, String format, String target, String gitSha, long selfTest) {
        this.failure = failure;
        this.platformKey = platformKey;
        this.attemptedPath = attemptedPath;
        this.cause = cause;
        this.guidance = guidance;
        this.version = version;
        this.format = format;
        this.target = target;
        this.gitSha = gitSha;
        this.selfTest = selfTest;
    }

    /** A linked, verified library. */
    public static NativeStatus ok(String platformKey, Path path, String version, String format, String target,
            String gitSha, long selfTest) {
        return new NativeStatus(null, platformKey, path, null, "", version, format, target, gitSha, selfTest);
    }

    /** A failure; the guidance comes from {@link FailureClass#guidance}. */
    public static NativeStatus failure(FailureClass failureClass, String platformKey, Path attemptedPath,
            Throwable cause, String guidance) {
        Objects.requireNonNull(failureClass, "failureClass");
        return new NativeStatus(failureClass, platformKey, attemptedPath, cause, guidance, null, null, null, null, 0);
    }

    static NativeStatus failure(FailureClass failureClass, String platformKey, Path attemptedPath, Throwable cause) {
        return failure(failureClass, platformKey, attemptedPath, cause,
                failureClass.guidance(platformKey, attemptedPath, cause));
    }

    public boolean isOk() {
        return failure == null;
    }

    /** The failure class, or {@code null} when OK. */
    public FailureClass failureClass() {
        return failure;
    }

    public String platformKey() {
        return platformKey;
    }

    /** The library file that was (or would have been) loaded; may be {@code null}. */
    public Path attemptedPath() {
        return attemptedPath;
    }

    /** The exception behind a failure; may be {@code null}. */
    public Throwable cause() {
        return cause;
    }

    /** What to do about a failure; empty when OK. */
    public String guidance() {
        return guidance;
    }

    /** The library's semver (without the git sha), e.g. {@code 0.0.1}; {@code null} on failure. */
    public String version() {
        return version;
    }

    /** The world-data format version the library writes, e.g. {@code 0.0}; {@code null} on failure. */
    public String format() {
        return format;
    }

    /** The Rust target triple the library was built for; {@code null} on failure. */
    public String target() {
        return target;
    }

    /** The git commit the library was built from (informational); {@code null} on failure. */
    public String gitSha() {
        return gitSha;
    }

    /** The self-test checksum the library returned; 0 on failure. */
    public long selfTest() {
        return selfTest;
    }

    /** One line for logs and {@code NativeSmoke}: {@code status=OK ...} or {@code status=FAILED class=...}. */
    public String summary() {
        return isOk()
                ? "status=OK version=" + version + " format=" + format + " selftest=" + Long.toUnsignedString(selfTest)
                : "status=FAILED class=" + failure;
    }

    /** The multi-line block logged once at ERROR on failure: platform, class, path, cause, guidance. */
    public String describe() {
        if (isOk()) {
            return "Deep Time natives: " + summary() + " target=" + target + " platform=" + platformKey
                    + " path=" + attemptedPath;
        }
        StringBuilder b = new StringBuilder();
        b.append("Deep Time natives failed to load: ").append(failure).append('\n');
        b.append("  platform: ").append(platformKey).append('\n');
        if (attemptedPath != null)
            b.append("  library:  ").append(attemptedPath).append('\n');
        if (cause != null)
            b.append("  cause:    ").append(cause).append('\n');
        b.append("  what to do: ").append(guidance);
        return b.toString();
    }

    @Override
    public String toString() {
        return summary();
    }
}
