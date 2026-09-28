// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

/**
 * Where the loader's messages and the drained native log lines go. The package is JDK-only
 * (plan §6.5: {@code NativeSmoke} runs it with only the mod jar on the classpath), so it logs
 * through this callback; the mod installs one that forwards to SLF4J, and the default is
 * {@link System.Logger}.
 */
@FunctionalInterface
public interface NativeLogSink {

    /** Logger name of the native log lines (plan §6.3). */
    String NATIVE = "deeptime.native";
    /** Logger name of the loader's own messages. */
    String LOADER = "deeptime.natives";

    /**
     * @param logger  {@link #NATIVE} or {@link #LOADER}
     * @param level   the level
     * @param message the message
     * @param thrown  an exception to log with it, or {@code null}
     */
    void log(String logger, System.Logger.Level level, String message, Throwable thrown);

    /** The default: {@link System#getLogger(String)}. */
    static NativeLogSink systemLogger() {
        return (logger, level, message, thrown) -> {
            System.Logger l = System.getLogger(logger);
            if (thrown == null)
                l.log(level, message);
            else
                l.log(level, message, thrown);
        };
    }
}
