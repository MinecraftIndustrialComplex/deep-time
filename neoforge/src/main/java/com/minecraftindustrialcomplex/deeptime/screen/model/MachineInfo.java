// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.lang.management.ManagementFactory;

/**
 * What the machine offers, shown beside the estimate (plan §5.7.5: the native heap is separate
 * from the JVM's, so a player needs RAM for both). JDK only: {@code com.sun.management}'s
 * operating-system bean where present, else unknown (−1).
 *
 * @param freeBytes  physical memory free now, −1 if unknown
 * @param totalBytes physical memory in total, −1 if unknown
 * @param jvmMaxBytes the JVM's heap limit ({@code -Xmx})
 */
public record MachineInfo(long freeBytes, long totalBytes, long jvmMaxBytes, int processors) {

    /** Reads the running machine. */
    public static MachineInfo current() {
        long free = -1, total = -1;
        try {
            if (ManagementFactory.getOperatingSystemMXBean() instanceof com.sun.management.OperatingSystemMXBean os) {
                free = os.getFreeMemorySize();
                total = os.getTotalMemorySize();
            }
        } catch (RuntimeException | LinkageError e) {
            // unknown
        }
        return new MachineInfo(free, total, Runtime.getRuntime().maxMemory(), Runtime.getRuntime().availableProcessors());
    }

    /** Rayon threads for a run on a client: all processors but one, so the render thread stays responsive (plan §6.2). */
    public int clientThreads() {
        return Math.max(1, processors - 1);
    }
}
