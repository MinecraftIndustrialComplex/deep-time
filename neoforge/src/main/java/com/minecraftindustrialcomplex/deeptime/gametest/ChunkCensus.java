// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.server.ServerStartedEvent;
import net.neoforged.neoforge.event.server.ServerStoppingEvent;
import net.neoforged.neoforge.event.tick.ServerTickEvent;

import java.lang.management.ManagementFactory;
import java.lang.management.MemoryPoolMXBean;
import java.lang.management.MemoryType;
import java.lang.management.MemoryUsage;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Chunk and heap census for the GameTest runtimes. After every server tick it counts each level's
 * chunk holders ({@code getLoadedChunksCount}: every status, proto-chunks included, under vanilla's
 * chunk system; C2ME's rewrite reports only the ones it promotes, so under MIC the heap numbers are
 * the measure) and samples the heap. It logs a {@code [gametest-chunks]} line when the chunk total
 * reaches a new peak by 10 % or more and one after each passing test ({@link #afterTest}), and at
 * the end one {@code [gametest] chunks peak=} line: the peak chunk total, the peak heap in use
 * (garbage included) and the peak live heap (the old generation after a collection). The MIC run's 8 GB heap died with ~68,000 chunks resident (the lakeHolds stall,
 * see {@link ChunkSampling}); these lines show a run's footprint.
 */
final class ChunkCensus {

    private static MinecraftServer server;
    private static long peak;
    private static long logged;
    private static String peakLevels = "";
    private static long peakHeapMb;
    private static long peakLiveMb;

    private ChunkCensus() {}

    static void register() {
        NeoForge.EVENT_BUS.addListener(ServerStartedEvent.class, e -> server = e.getServer());
        NeoForge.EVENT_BUS.addListener(ServerTickEvent.Post.class, e -> tick(e.getServer()));
        NeoForge.EVENT_BUS.addListener(ServerStoppingEvent.class, e ->
                DeepTime.LOGGER.info("[gametest] chunks peak={} levelsAtPeak={} heapPeakMB={} liveHeapPeakMB={} maxHeapMB={}",
                        peak, peakLevels, peakHeapMb, peakLiveMb, Runtime.getRuntime().maxMemory() >> 20));
    }

    private static void tick(MinecraftServer s) {
        sampleHeap();
        Map<String, Integer> levels = new LinkedHashMap<>();
        long total = count(s, levels);
        if (total <= peak)
            return;
        peak = total;
        peakLevels = levels.toString();
        if (total >= logged + Math.max(1000, logged / 10)) {
            logged = total;
            DeepTime.LOGGER.info("[gametest-chunks] tick={} total={} heapUsedMB={} levels={}", s.getTickCount(), total, usedMb(), levels);
        }
    }

    /** Logs the chunks and heap right after a test passed. */
    static void afterTest(String test) {
        MinecraftServer s = server;
        if (s == null)
            return;
        sampleHeap();
        Map<String, Integer> levels = new LinkedHashMap<>();
        long total = count(s, levels);
        DeepTime.LOGGER.info("[gametest-chunks] after={} tick={} total={} heapUsedMB={} levels={}", test, s.getTickCount(), total,
                usedMb(), levels);
    }

    private static long count(MinecraftServer s, Map<String, Integer> levels) {
        long total = 0;
        for (ServerLevel level : s.getAllLevels()) {
            int n = level.getChunkSource().getLoadedChunksCount();
            total += n;
            if (n > 0)
                levels.put(level.dimension().location().getPath(), n);
        }
        return total;
    }

    private static long usedMb() {
        Runtime rt = Runtime.getRuntime();
        return (rt.totalMemory() - rt.freeMemory()) >> 20;
    }

    private static final List<MemoryPoolMXBean> OLD_GEN = ManagementFactory.getMemoryPoolMXBeans().stream()
            .filter(p -> p.getType() == MemoryType.HEAP && p.getName().contains("Old Gen") && p.isCollectionUsageThresholdSupported())
            .toList();

    private static void sampleHeap() {
        peakHeapMb = Math.max(peakHeapMb, usedMb());
        for (MemoryPoolMXBean p : OLD_GEN) {
            MemoryUsage after = p.getCollectionUsage();
            if (after != null)
                peakLiveMb = Math.max(peakLiveMb, after.getUsed() >> 20);
        }
    }
}
