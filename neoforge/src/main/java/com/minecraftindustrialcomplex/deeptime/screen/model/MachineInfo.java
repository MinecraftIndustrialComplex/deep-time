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
 * operating-system bean for the total, else unknown (−1). The "free" figure is what a new process
 * could get, because the bean's own free figure leaves out memory the system hands back on demand:
 * <ul>
 *   <li>Linux: {@code MemAvailable} from {@code /proc/meminfo}; the bean reports {@code MemFree},
 *       which leaves out the page cache (0.7 GiB "free" of 15 GiB on the laptop with 7 GiB
 *       available, measured in P1-11's client check).</li>
 *   <li>macOS: free + speculative + inactive + purgeable pages from {@code vm_stat}
 *       ({@link MacMemory}); the bean reports free pages alone (1.0 GiB of 16 GiB on the M6 Mac
 *       mini with about 6 GiB available). {@code vm_stat} is a process spawn, so it is read on a
 *       background thread: {@link #prime()} starts a reading when a screen opens and
 *       {@link #current()} uses the last one, never waiting. With no reading yet (or no working
 *       {@code vm_stat}) the figure is unknown (−1), not the bean's wrong one.</li>
 *   <li>Elsewhere (Windows): the bean's figure.</li>
 * </ul>
 * The figure is a snapshot of the moment {@link #current()} ran.
 *
 * @param freeBytes   physical memory available now, −1 if unknown
 * @param totalBytes  physical memory in total, −1 if unknown
 * @param jvmMaxBytes the JVM's heap limit ({@code -Xmx})
 */
public record MachineInfo(long freeBytes, long totalBytes, long jvmMaxBytes, int processors) {

    /** Reads the running machine; never waits on a process or a disk read of any size (see the class comment). */
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
        else if (isMac())
            free = MacMemory.SYSTEM.availableBytes();
        if (total > 0 && free > total)
            free = total;
        return new MachineInfo(free, total, Runtime.getRuntime().maxMemory(), Runtime.getRuntime().availableProcessors());
    }

    /**
     * Starts reading the free memory in the background where that takes a process (macOS), so that
     * {@link #current()} has a fresh figure when the editor opens a moment later. Cheap and
     * rate-limited: a screen may call it on every init. Nothing to do elsewhere.
     */
    public static void prime() {
        if (isMac())
            MacMemory.SYSTEM.refreshIfStale();
    }

    private static boolean isMac() {
        return System.getProperty("os.name", "").startsWith("Mac");
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
