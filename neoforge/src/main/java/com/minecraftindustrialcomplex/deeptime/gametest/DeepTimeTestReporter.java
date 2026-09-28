// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.gametest.framework.GameTestInfo;
import net.minecraft.gametest.framework.LogTestReporter;
import net.minecraft.gametest.framework.TestReporter;

import java.util.Map;
import java.util.TreeMap;

/**
 * Logs every GameTest result on the dev planet server (cold-read review 2, gap 1, the blocker).
 *
 * <p>In 1.21.1 a dedicated server with no players logs no passes and no summary: {@code /test}
 * reports through chat to players only, and vanilla's {@code LogTestReporter} ignores successes.
 * This reporter logs {@code [gametest-result] <test> PASS|FAIL required=<b> <ms>ms} for every
 * result, delegates failures to {@code LogTestReporter}, and keeps per-test counts that
 * {@code /deeptime gametest summary} prints as one {@code [gametest-summary]} JSON line, which
 * gametest-planet.sh waits for. With retries ({@code attempts}/{@code requiredSuccesses}) vanilla
 * reports each passing attempt but not failing ones, so passes are what the harness judges.
 */
public final class DeepTimeTestReporter implements TestReporter {

    public static final DeepTimeTestReporter INSTANCE = new DeepTimeTestReporter();

    private final LogTestReporter log = new LogTestReporter();
    private final Map<String, int[]> counts = new TreeMap<>();

    private DeepTimeTestReporter() {}

    @Override
    public synchronized void onTestSuccess(GameTestInfo info) {
        counts.computeIfAbsent(info.getTestName(), k -> new int[2])[0]++;
        DeepTime.LOGGER.info("[gametest-result] {} PASS required={} {}ms", info.getTestName(), info.isRequired(), info.getRunTime());
    }

    @Override
    public synchronized void onTestFailed(GameTestInfo info) {
        counts.computeIfAbsent(info.getTestName(), k -> new int[2])[1]++;
        DeepTime.LOGGER.info("[gametest-result] {} FAIL required={} {}ms {}", info.getTestName(), info.isRequired(),
                info.getRunTime(), info.getError() == null ? "" : info.getError().getMessage());
        log.onTestFailed(info);
    }

    public synchronized void reset() {
        counts.clear();
    }

    public synchronized JsonObject summary() {
        JsonObject o = new JsonObject();
        JsonObject tests = new JsonObject();
        int pass = 0, fail = 0;
        for (var e : counts.entrySet()) {
            JsonObject t = new JsonObject();
            t.addProperty("pass", e.getValue()[0]);
            t.addProperty("fail", e.getValue()[1]);
            tests.add(e.getKey(), t);
            pass += e.getValue()[0];
            fail += e.getValue()[1];
        }
        o.addProperty("passed", pass);
        o.addProperty("failed", fail);
        o.add("tests", tests);
        return o;
    }
}
