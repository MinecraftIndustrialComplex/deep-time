// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.OptionalLong;
import java.util.concurrent.Callable;
import java.util.concurrent.Executor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.LongSupplier;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Memory a new process could get on macOS, from {@code vm_stat}, read off the calling thread.
 *
 * <p>Why not the JDK: {@code OperatingSystemMXBean.getFreeMemorySize()} is the kernel's free-page
 * count alone, which on a busy Mac is a few hundred MiB to 1 GiB of 16 (the Create World page read
 * "1.0 GiB free of 16.0 GiB" on the M6 Mac mini while about 6 GiB was there for the asking).
 * macOS keeps otherwise idle RAM as inactive pages (recently unused; the system compresses or drops
 * them when something needs the space) and purgeable pages (caches an app has marked as
 * disposable), so the figure that means "room for a new run" is
 * <b>free + speculative + inactive + purgeable</b>. That is the sum the rest of the tool chain uses
 * (graphide's {@code m6lock.sh avail_gb}: free + inactive + speculative; psutil and Zabbix:
 * free + inactive, where their "free" includes the speculative pages) plus purgeable. It is not
 * Activity Monitor's "physical memory minus memory used", which counts inactive anonymous pages
 * as used (App Memory, Compressed) and so reads lower whenever a lot of memory is idle but not
 * file cache. vm_stat's "Pages free" already leaves out the speculative pages
 * ({@code host_statistics64}'s free count is free + speculative), so the two do not overlap; a
 * purgeable page that also sits in the inactive queue is counted twice, a few dozen MiB at most.
 *
 * <p>Reading it means running {@code /usr/bin/vm_stat}, about 12 ms of process spawn on the M6. So
 * nothing here waits for it: {@link #availableBytes()} returns the last reading (−1 until the
 * first one lands) and, when that reading is older than the TTL, starts the next one on a daemon
 * thread. {@link #refreshIfStale()} is the same without the return value, for a screen to call
 * when it opens so the value is there by the time the editor asks.
 */
final class MacMemory {

    /** How long a reading is used before the next one starts (resize storms of the Create World screen call in). */
    static final long TTL_NANOS = TimeUnit.SECONDS.toNanos(5);

    /** The production instance: {@code /usr/bin/vm_stat} on a daemon thread per reading. */
    static final MacMemory SYSTEM = new MacMemory(MacMemory::runVmStat, MacMemory::daemonThread, System::nanoTime, TTL_NANOS);

    private static final Pattern PAGE_SIZE = Pattern.compile("page size of (\\d+) bytes");
    private static final Pattern COUNT_LINE = Pattern.compile("^([^:]+):\\s+(\\d+)\\.?\\s*$");

    private final Callable<String> source;
    private final Executor executor;
    private final LongSupplier clock;
    private final long ttlNanos;
    private final AtomicBoolean reading = new AtomicBoolean();
    private volatile long availableBytes = -1;
    private volatile boolean attempted;
    /** {@link #clock} at the start of the last attempt (successful or not: a failing vm_stat is not retried every frame). */
    private volatile long attemptedAt;

    MacMemory(Callable<String> source, Executor executor, LongSupplier clock, long ttlNanos) {
        this.source = source;
        this.executor = executor;
        this.clock = clock;
        this.ttlNanos = ttlNanos;
    }

    /** The last reading in bytes, −1 if none has landed (or vm_stat never worked); never blocks. */
    long availableBytes() {
        refreshIfStale();
        return availableBytes;
    }

    /** Starts a background reading when there has been none, or the last is older than the TTL. Never blocks. */
    void refreshIfStale() {
        if (!attempted || clock.getAsLong() - attemptedAt >= ttlNanos)
            refresh();
    }

    /** Starts a background reading unless one is already running. Never blocks. */
    void refresh() {
        if (!reading.compareAndSet(false, true))
            return;
        attempted = true;
        attemptedAt = clock.getAsLong();
        try {
            executor.execute(() -> {
                try {
                    parse(source.call()).ifPresent(v -> availableBytes = v);
                } catch (Exception | LinkageError e) {
                    // vm_stat missing, slow or garbled: keep the last reading (or −1, "unknown")
                } finally {
                    reading.set(false);
                }
            });
        } catch (RuntimeException e) {
            reading.set(false);
        }
    }

    /**
     * free + speculative + inactive + purgeable pages of {@code vm_stat} output, in bytes; empty when
     * the page size, the free count or the inactive count is missing.
     */
    static OptionalLong parse(String vmStat) {
        if (vmStat == null)
            return OptionalLong.empty();
        long pageSize = -1, free = -1, inactive = -1, speculative = 0, purgeable = 0;
        for (String line : vmStat.split("\\R")) {
            try {
                Matcher size = PAGE_SIZE.matcher(line);
                if (size.find()) {
                    pageSize = Long.parseLong(size.group(1));
                    continue;
                }
                Matcher m = COUNT_LINE.matcher(line);
                if (!m.matches())
                    continue;
                long pages = Long.parseLong(m.group(2));
                switch (m.group(1)) {
                    case "Pages free" -> free = pages;
                    case "Pages inactive" -> inactive = pages;
                    case "Pages speculative" -> speculative = pages;
                    case "Pages purgeable" -> purgeable = pages;
                    default -> {
                    }
                }
            } catch (NumberFormatException e) {
                // a counter too large for a long is none of the four that matter
            }
        }
        if (pageSize <= 0 || free < 0 || inactive < 0)
            return OptionalLong.empty();
        return OptionalLong.of((free + speculative + inactive + purgeable) * pageSize);
    }

    private static String runVmStat() throws IOException, InterruptedException {
        Process p = new ProcessBuilder("/usr/bin/vm_stat").redirectErrorStream(true).start();
        try {
            // The output (about 2 kB) fits the pipe, so waiting for the exit first cannot deadlock;
            // a vm_stat that hangs is abandoned after 2 s rather than holding the reader forever.
            if (!p.waitFor(2, TimeUnit.SECONDS))
                throw new IOException("vm_stat did not finish in 2 s");
            return new String(p.getInputStream().readAllBytes(), StandardCharsets.US_ASCII);
        } finally {
            p.destroyForcibly();
        }
    }

    private static void daemonThread(Runnable r) {
        Thread t = new Thread(r, "deeptime-vmstat");
        t.setDaemon(true);
        t.start();
    }
}
