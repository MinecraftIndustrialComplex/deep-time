// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.gametest.framework.GameTestInfo;
import net.minecraft.gametest.framework.LogTestReporter;
import net.minecraft.gametest.framework.TestReporter;

import java.util.Locale;
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
 * gametest-planet.sh waits for.
 *
 * <p>Repeated tests ({@code attempts}/{@code requiredSuccesses} on {@code @GameTest}): a test run
 * with {@code /test run} has no retry options, so vanilla's {@code ReportGameListener} takes its
 * "flaky" path, which reruns the test itself and reports here exactly once: one PASS after
 * {@code requiredSuccesses} passing attempts, or one FAIL as soon as that can no longer happen.
 * With {@code attempts == requiredSuccesses}, one PASS therefore means every attempt passed. Test
 * bodies also report each attempt through {@link #attempt}, so the summary shows the per-attempt
 * counts ({@code attempts}, {@code attemptsOk}) next to the required number.
 */
public final class DeepTimeTestReporter implements TestReporter {

    public static final DeepTimeTestReporter INSTANCE = new DeepTimeTestReporter();

    private final LogTestReporter log = new LogTestReporter();
    private final Map<String, Entry> entries = new TreeMap<>();

    private static final class Entry {
        int pass, fail, attempts, attemptsOk, maxAttempts = 1, requiredSuccesses = 1;
        boolean required = true;
    }

    private DeepTimeTestReporter() {}

    private Entry entry(String testName) {
        return entries.computeIfAbsent(testName.toLowerCase(Locale.ROOT), k -> new Entry());
    }

    private Entry entry(GameTestInfo info) {
        Entry e = entry(info.getTestName());
        e.required = info.isRequired();
        e.maxAttempts = info.maxAttempts();
        e.requiredSuccesses = info.requiredSuccesses();
        return e;
    }

    @Override
    public synchronized void onTestSuccess(GameTestInfo info) {
        entry(info).pass++;
        DeepTime.LOGGER.info("[gametest-result] {} PASS required={} {}ms", info.getTestName(), info.isRequired(), info.getRunTime());
    }

    @Override
    public synchronized void onTestFailed(GameTestInfo info) {
        entry(info).fail++;
        DeepTime.LOGGER.info("[gametest-result] {} FAIL required={} {}ms {}", info.getTestName(), info.isRequired(),
                info.getRunTime(), info.getError() == null ? "" : info.getError().getMessage());
        log.onTestFailed(info);
    }

    /** One attempt of a repeated test, reported by the test body (see the class comment). */
    public synchronized void attempt(String testName, boolean ok, String detail) {
        Entry e = entry(testName);
        e.attempts++;
        if (ok)
            e.attemptsOk++;
        DeepTime.LOGGER.info("[gametest-attempt] {} #{} {} {}", testName, e.attempts, ok ? "ok" : "FAILED", detail);
    }

    public synchronized void reset() {
        entries.clear();
    }

    public synchronized JsonObject summary() {
        JsonObject o = new JsonObject();
        JsonObject tests = new JsonObject();
        int pass = 0, fail = 0;
        for (var e : entries.entrySet()) {
            Entry v = e.getValue();
            JsonObject t = new JsonObject();
            t.addProperty("pass", v.pass);
            t.addProperty("fail", v.fail);
            t.addProperty("required", v.required);
            if (v.maxAttempts > 1 || v.attempts > 0) {
                t.addProperty("maxAttempts", v.maxAttempts);
                t.addProperty("requiredSuccesses", v.requiredSuccesses);
                t.addProperty("attempts", v.attempts);
                t.addProperty("attemptsOk", v.attemptsOk);
            }
            tests.add(e.getKey(), t);
            pass += v.pass;
            fail += v.fail;
        }
        o.addProperty("passed", pass);
        o.addProperty("failed", fail);
        o.add("tests", tests);
        return o;
    }
}
