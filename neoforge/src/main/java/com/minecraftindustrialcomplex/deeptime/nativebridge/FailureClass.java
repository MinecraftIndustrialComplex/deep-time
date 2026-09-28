// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.file.Path;

/**
 * Why the native library could not be used (plan §6.6). Each class has its own guidance; none of
 * them crashes the game: existing worlds play on, because chunk generation reads world data in
 * pure Java.
 */
public enum FailureClass {

    /** No library in {@code natives.json} for the platform key (or the library cannot run here). */
    UNSUPPORTED_PLATFORM,
    /** {@code UnsatisfiedLinkError} naming {@code GLIBC_}, {@code ld-linux} or {@code ld-musl}, after both libc builds were tried. */
    LIBC_MISMATCH,
    /** The extracted file vanished or changed between write and load. */
    ANTIVIRUS_SUSPECTED,
    /** An {@code IOException} while creating, locking, writing or moving. */
    IO_ERROR,
    /** The library's semver/format differs from {@code natives.json}, or its self-test sum differs from Java's. */
    VERSION_MISMATCH;

    /** The CLI route every "cannot simulate here" message ends with (plan §6.6). */
    public static final String CLI_ROUTE =
            "Existing Deep Time worlds still play. To create a new one, run `deeptime run` (the Deep Time CLI) on a "
                    + "supported machine and copy its deeptime/ folder into the world folder.";

    /** What the player or server owner should do, for {@link NativeStatus#guidance()}. */
    public String guidance(String platformKey, Path attemptedPath, Throwable cause) {
        String where = attemptedPath == null ? "the Deep Time natives directory"
                : String.valueOf(attemptedPath.getParent() == null ? attemptedPath : attemptedPath.getParent());
        String causeText = cause == null ? "" : cause.getClass().getSimpleName() + ": " + cause.getMessage();
        return switch (this) {
            case UNSUPPORTED_PLATFORM -> "Deep Time has no native library that runs on this platform ("
                    + platformKey + "). " + CLI_ROUTE;
            case LIBC_MISMATCH -> "The native library needs a C library this system does not provide ("
                    + causeText + "). Install the musl build, or update glibc (the floor is 2.17). " + CLI_ROUTE;
            case ANTIVIRUS_SUSPECTED -> "The extracted native library vanished or changed between writing and "
                    + "loading, which is what antivirus quarantine looks like. Add an exclusion for " + where
                    + ", or start the game with -Ddeeptime.natives.path=<a copy of the library>.";
            case IO_ERROR -> "Could not create, lock, write or move files in " + where + " (" + causeText
                    + "). Check that the folder is writable and the disk has space.";
            case VERSION_MISMATCH -> "The native library does not match this Deep Time (" + causeText
                    + "). Reinstall the mod; a stray -Ddeeptime.natives.path library is the usual cause.";
        };
    }
}
