// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeStatus;
import com.minecraftindustrialcomplex.deeptime.nativebridge.PlatformKey;

import java.util.Optional;

/**
 * The guidance panel's content (plan §6.6): shown instead of the setup page when the natives did
 * not load. Existing worlds keep playing; creating a Deep Time world needs the CLI route.
 *
 * @param failureClass the {@code FailureClass} name, or {@code NOT_LOADED} when the loader never ran
 * @param platform     the detected platform key
 * @param library      the library path that was tried, or empty
 * @param guidance     what to do (the loader's guidance, which ends with the CLI route)
 * @param details      the full block the log has, for "Copy details"
 */
public record GuidanceModel(String failureClass, String platform, String library, String guidance, String details) {

    public static GuidanceModel of(Optional<NativeStatus> status) {
        if (status.isEmpty())
            return new GuidanceModel("NOT_LOADED", detectPlatform(), "",
                    "The Deep Time native library was not loaded (see the log). "
                            + com.minecraftindustrialcomplex.deeptime.nativebridge.FailureClass.CLI_ROUTE,
                    "Deep Time natives were not loaded.");
        NativeStatus s = status.get();
        if (s.isOk())
            throw new IllegalArgumentException("the natives loaded: " + s.summary());
        return new GuidanceModel(String.valueOf(s.failureClass()), s.platformKey() == null ? detectPlatform() : s.platformKey(),
                s.attemptedPath() == null ? "" : s.attemptedPath().toString(), s.guidance(), s.describe());
    }

    private static String detectPlatform() {
        try {
            return PlatformKey.detect().key();
        } catch (RuntimeException | LinkageError e) {
            return System.getProperty("os.name") + " " + System.getProperty("os.arch");
        }
    }
}
