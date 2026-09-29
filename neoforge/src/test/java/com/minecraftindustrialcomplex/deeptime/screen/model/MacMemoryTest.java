// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.lang.management.ManagementFactory;
import java.util.ArrayDeque;
import java.util.Queue;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledOnOs;
import org.junit.jupiter.api.condition.OS;

/** The macOS free-memory reading: the {@code vm_stat} parser and the never-blocking cache around it. */
class MacMemoryTest {

    private static final long GIB = 1L << 30;

    /** {@code vm_stat} on the 16 GB M6 Mac mini (macOS 27.0, aarch64), 2026-09-29, as printed. */
    static final String M6_VM_STAT = """
            Mach Virtual Memory Statistics: (page size of 16384 bytes)
            Pages free:                                   240653.
            Pages active:                                 272824.
            Pages inactive:                               271108.
            Pages speculative:                              1642.
            Pages throttled:                                   0.
            Pages wired down:                             156972.
            Pages purgeable:                                1947.
            "Translation faults":                     9135524895.
            Pages copy-on-write:                       701210430.
            Pages zero filled:                        3920117964.
            Pages reactivated:                         220118218.
            Pages purged:                                8035398.
            File-backed pages:                            224392.
            Anonymous pages:                              321182.
            Pages stored in compressor:                   186899.
            Pages occupied by compressor:                  67868.
            Decompressions:                            143413266.
            Compressions:                              164168621.
            Pageins:                                   168465572.
            Pageouts:                                     515474.
            Swapins:                                       86945.
            Swapouts:                                     149932.
            Pages tagged:                                 139258.
            Pages tagged resident:                        104523.
            Pages tagged compressed:                       34735.
            Pages tag-storage:                             32768.
            Pages tag-storage holding tags:                 5317.
            Pages tag-storage free:                         3722.
            Pages tag-storage non-tag pageable:            23612.
            Pages tag-storage non-tag wired:                   0.
            Bytes of compressed tags:                    5942336.
            Tagged compressions:                         1711832.
            Tagged decompressions:                       1629494.
            """;

    @Test
    void parsesTheM6Capture() {
        // (free 240,653 + speculative 1,642 + inactive 271,108 + purgeable 1,947) pages of 16 KiB. The strictly
        // free pages alone (what the JDK reports) are 240,653 x 16 KiB = 3.67 GiB here, and "Pages tag-storage
        // free" and the other tag/File-backed/Anonymous lines must not be mistaken for the four that count.
        long expected = (240_653L + 1_642 + 271_108 + 1_947) * 16_384;
        assertEquals(8_443_494_400L, expected);
        assertEquals(expected, MacMemory.parse(M6_VM_STAT).orElseThrow());
        assertTrue(expected > 7 * GIB && expected < 8 * GIB);
    }

    @Test
    void parsesTheFourKilobytePageLayout() {
        // The layout of Intel Macs and older macOS: 4 KiB pages, no File-backed / Anonymous / tag lines
        // (written from vm_stat's documented format).
        String intel = """
                Mach Virtual Memory Statistics: (page size of 4096 bytes)
                Pages free:                               65432.
                Pages active:                           1000000.
                Pages inactive:                          800000.
                Pages speculative:                        20000.
                Pages throttled:                              0.
                Pages wired down:                        500000.
                Pages purgeable:                          10000.
                "Translation faults":                 123456789.
                Pageins:                                 111111.
                Pageouts:                                     0.
                Swapins:                                      0.
                Swapouts:                                     0.
                """;
        assertEquals((65_432L + 20_000 + 800_000 + 10_000) * 4096, MacMemory.parse(intel).orElseThrow());
    }

    @Test
    void speculativeAndPurgeableAreOptional() {
        String minimal = "Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free: 100.\nPages inactive: 50.\n";
        assertEquals(150L * 16_384, MacMemory.parse(minimal).orElseThrow());
        // Windows line endings do not matter either.
        assertEquals(150L * 16_384, MacMemory.parse(minimal.replace("\n", "\r\n")).orElseThrow());
    }

    @Test
    void anythingIncompleteIsUnknown() {
        assertTrue(MacMemory.parse(null).isEmpty());
        assertTrue(MacMemory.parse("").isEmpty());
        assertTrue(MacMemory.parse("vm_stat: command not found\n").isEmpty());
        // No page size: the counts mean nothing.
        assertTrue(MacMemory.parse("Pages free: 100.\nPages inactive: 50.\n").isEmpty());
        // No inactive count: a sum of the rest would look like the strict figure it is meant to replace.
        assertTrue(MacMemory.parse("Mach Virtual Memory Statistics: (page size of 4096 bytes)\nPages free: 100.\n").isEmpty());
        assertTrue(MacMemory.parse("Mach Virtual Memory Statistics: (page size of 0 bytes)\nPages free: 1.\nPages inactive: 1.\n").isEmpty());
    }

    @Test
    void aCounterTooLargeForALongIsSkippedNotFatal() {
        String s = """
                Mach Virtual Memory Statistics: (page size of 16384 bytes)
                Pages free:      100.
                Pages inactive:   50.
                Pages purgeable: 99999999999999999999999999.
                """;
        assertEquals(150L * 16_384, MacMemory.parse(s).orElseThrow());
    }

    /** A cache with a queue for an executor (nothing runs until the test says) and a clock the test moves. */
    private static final class Rig {
        final Queue<Runnable> queue = new ArrayDeque<>();
        final AtomicLong now = new AtomicLong(1_000);
        final AtomicInteger calls = new AtomicInteger();
        volatile String output = "Mach Virtual Memory Statistics: (page size of 1024 bytes)\nPages free: 1.\nPages inactive: 1.\n";
        volatile boolean fail;
        final MacMemory memory = new MacMemory(() -> {
            calls.incrementAndGet();
            if (fail)
                throw new java.io.IOException("no vm_stat");
            return output;
        }, queue::add, now::get, 100);

        void runQueued() {
            Runnable r;
            while ((r = queue.poll()) != null)
                r.run();
        }
    }

    @Test
    void neverWaitsForTheReading() {
        Rig rig = new Rig();
        // The first call only starts the reading: unknown now, the source has not run on this thread.
        assertEquals(-1, rig.memory.availableBytes());
        assertEquals(0, rig.calls.get());
        assertEquals(1, rig.queue.size());
        // Calls while it runs start nothing more.
        assertEquals(-1, rig.memory.availableBytes());
        rig.memory.refresh();
        rig.memory.refreshIfStale();
        assertEquals(1, rig.queue.size());
        rig.runQueued();
        assertEquals(1, rig.calls.get());
        assertEquals(2 * 1024, rig.memory.availableBytes());
        assertEquals(0, rig.queue.size(), "fresh: no new reading");
    }

    @Test
    void staleReadingIsServedWhileTheNextOneRuns() {
        Rig rig = new Rig();
        rig.memory.refresh();
        rig.runQueued();
        assertEquals(2 * 1024, rig.memory.availableBytes());
        rig.output = "Mach Virtual Memory Statistics: (page size of 1024 bytes)\nPages free: 10.\nPages inactive: 10.\n";
        rig.now.addAndGet(99);
        assertEquals(2 * 1024, rig.memory.availableBytes());
        assertEquals(0, rig.queue.size(), "99 of 100 ticks: still fresh");
        rig.now.addAndGet(1);
        assertEquals(2 * 1024, rig.memory.availableBytes(), "the old figure is served at once");
        assertEquals(1, rig.queue.size(), "and the next reading is under way");
        rig.runQueued();
        assertEquals(20 * 1024, rig.memory.availableBytes());
        assertEquals(2, rig.calls.get());
    }

    @Test
    void aFailingVmStatKeepsTheLastFigureAndIsNotRetriedEveryFrame() {
        Rig rig = new Rig();
        rig.fail = true;
        assertEquals(-1, rig.memory.availableBytes());
        rig.runQueued();
        assertEquals(-1, rig.memory.availableBytes(), "no figure ever: unknown, never a made-up one");
        assertEquals(0, rig.queue.size(), "the failed attempt counts for the TTL");
        rig.fail = false;
        rig.now.addAndGet(100);
        assertEquals(-1, rig.memory.availableBytes());
        rig.runQueued();
        assertEquals(2 * 1024, rig.memory.availableBytes(), "it recovers on the next attempt");
        rig.fail = true;
        rig.now.addAndGet(100);
        rig.memory.availableBytes();
        rig.runQueued();
        assertEquals(2 * 1024, rig.memory.availableBytes(), "a later failure keeps the last good figure");
        assertEquals(3, rig.calls.get());
    }

    @Test
    void garbledOutputKeepsTheLastFigure() {
        Rig rig = new Rig();
        rig.memory.refresh();
        rig.runQueued();
        rig.output = "garbage";
        rig.now.addAndGet(100);
        rig.memory.availableBytes();
        rig.runQueued();
        assertEquals(2 * 1024, rig.memory.availableBytes());
    }

    @Test
    void anExecutorThatRefusesDoesNotWedgeTheCache() {
        AtomicInteger refusals = new AtomicInteger();
        Rig rig = new Rig();
        MacMemory refusing = new MacMemory(() -> rig.output, r -> {
            if (refusals.getAndIncrement() == 0)
                throw new java.util.concurrent.RejectedExecutionException("shutting down");
            rig.queue.add(r);
        }, rig.now::get, 100);
        assertEquals(-1, refusing.availableBytes());
        rig.now.addAndGet(100);
        assertEquals(-1, refusing.availableBytes());
        rig.runQueued();
        assertEquals(2 * 1024, refusing.availableBytes());
    }

    /** The real thing, only where vm_stat exists: the daemon thread, the process and the numbers. */
    @Test
    @EnabledOnOs(OS.MAC)
    void liveVmStatOnThisMac() throws Exception {
        long total = ((com.sun.management.OperatingSystemMXBean) ManagementFactory.getOperatingSystemMXBean()).getTotalMemorySize();
        long beanFree = ((com.sun.management.OperatingSystemMXBean) ManagementFactory.getOperatingSystemMXBean()).getFreeMemorySize();
        long t0 = System.nanoTime();
        MachineInfo.prime();
        long primeMicros = (System.nanoTime() - t0) / 1000;
        long deadline = System.currentTimeMillis() + 5000;
        MachineInfo m = MachineInfo.current();
        while (m.freeBytes() < 0 && System.currentTimeMillis() < deadline) {
            Thread.sleep(20);
            m = MachineInfo.current();
        }
        assertTrue(m.freeBytes() > 0, "vm_stat was read");
        assertTrue(m.freeBytes() <= m.totalBytes(), "at most all of the memory");
        assertEquals(total, m.totalBytes());
        System.out.printf("[junit] macAvailable: prime() took %d us on the calling thread; MachineInfo.freeBytes=%d (%.2f GiB) "
                + "of %.2f GiB; the JDK bean's free figure a moment before: %.2f GiB%n",
                primeMicros, m.freeBytes(), m.freeBytes() / (double) GIB, total / (double) GIB, beanFree / (double) GIB);
        // The schedule-only proof is neverWaitsForTheReading (the source does not run on the caller's thread);
        // this is a sanity bound, far under a vm_stat spawn plus the load of a shared Mac.
        assertTrue(primeMicros < 250_000, "prime() took " + primeMicros + " us");
    }
}
