// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.io.IOException;
import java.lang.management.ManagementFactory;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.OptionalLong;

/**
 * What the machine offers, shown beside the estimate (plan §5.7.5: the native heap is separate
 * from the JVM's, so a player needs RAM for both). JDK only: {@code com.sun.management}'s
 * operating-system bean where present, else unknown (−1). On Linux the "free" figure is
 * {@code MemAvailable} from {@code /proc/meminfo}: the bean reports {@code MemFree}, which leaves
 * out the page cache the kernel gives back on demand (0.7 GiB "free" of 15 GiB on the laptop
 * with 7 GiB available, measured in P1-11's client check).
 *
 * @param freeBytes   physical memory available now, −1 if unknown
 * @param totalBytes  physical memory in total, −1 if unknown
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
        OptionalLong available = memAvailable(Path.of("/proc/meminfo"));
        if (available.isPresent())
            free = available.getAsLong();
        return new MachineInfo(free, total, Runtime.getRuntime().maxMemory(), Runtime.getRuntime().availableProcessors());
    }

    /** {@code MemAvailable} of a Linux {@code /proc/meminfo} in bytes; empty when absent or unreadable. */
    static OptionalLong memAvailable(Path meminfo) {
        try {
            for (String line : Files.readAllLines(meminfo, StandardCharsets.US_ASCII)) {
                if (!line.startsWith("MemAvailable:"))
                    continue;
                String[] parts = line.substring("MemAvailable:".length()).trim().split("\\s+");
                long v = Long.parseLong(parts[0]);
                return OptionalLong.of(parts.length > 1 && parts[1].equalsIgnoreCase("kB") ? v * 1024 : v);
            }
        } catch (IOException | RuntimeException e) {
            // not Linux, or unreadable
        }
        return OptionalLong.empty();
    }

    /** Rayon threads for a run on a client: all processors but one, so the render thread stays responsive (plan §6.2). */
    public int clientThreads() {
        return Math.max(1, processors - 1);
    }
}
