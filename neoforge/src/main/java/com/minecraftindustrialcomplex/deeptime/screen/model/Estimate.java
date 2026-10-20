// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.util.ArrayList;
import java.util.List;

/**
 * A preset's memory and time estimate (plan §5.7.5, §6.1 {@code estimate}):
 * {@code {"peak_bytes":P,"base_bytes":B,"history_bytes":H,"spill_bytes":S,"mem_cap":C,
 * "stages":[{"bytes","name","seconds",…},…]}}.
 *
 * <p>The recorded history ({@code H}, the frames the deposit engine replays) stays in native memory
 * while the cap allows and goes to a spill file in the run's folder after that (P1-17; decisions
 * log 2026-09-29): a run only needs its base ({@code B}) under the cap, and over it just runs
 * slower. The native estimate's peak and spill are for the default cap; {@link #peakUnder} and
 * {@link #spillUnder} give them for any cap. An estimate without {@code base_bytes} (a library
 * from before the spill) counts its whole peak as base.
 *
 * <p>The stages' seconds also weight the generation page's progress bar and its time left
 * ({@link #fraction}, {@link #timeLeftMs}), with the bake after them ({@link #bakeSeconds}).
 */
public record Estimate(long peakBytes, long baseBytes, long historyBytes, List<Stage> stages) {

    /**
     * One stage: its peak bytes, its seconds on an unloaded machine, and the stage it steps with
     * ({@code coupled_to}: a running session reports a group of coupled stages under its lead's
     * index), or null.
     */
    public record Stage(String name, long bytes, double seconds, String coupledTo) {
        public Stage(String name, long bytes, double seconds) {
            this(name, bytes, seconds, null);
        }
    }

    public Estimate {
        stages = List.copyOf(stages);
    }

    /** An estimate whose peak is all base (no spillable history). */
    public Estimate(long peakBytes, List<Stage> stages) {
        this(peakBytes, peakBytes, 0, stages);
    }

    public static Estimate parse(String json) {
        JsonObject o = JsonParser.parseString(json).getAsJsonObject();
        List<Stage> stages = new ArrayList<>();
        if (o.has("stages"))
            for (JsonElement e : o.getAsJsonArray("stages")) {
                JsonObject s = e.getAsJsonObject();
                JsonElement to = s.get("coupled_to");
                stages.add(new Stage(s.get("name").getAsString(), s.get("bytes").getAsLong(), s.get("seconds").getAsDouble(),
                        to == null || to.isJsonNull() ? null : to.getAsString()));
            }
        long peak = o.get("peak_bytes").getAsLong();
        long base = o.has("base_bytes") ? o.get("base_bytes").getAsLong() : peak;
        long history = o.has("history_bytes") ? o.get("history_bytes").getAsLong() : 0;
        return new Estimate(peak, base, history, stages);
    }

    /**
     * The bake (the world data written once the stages are done) in the stages' seconds, as a
     * share of the amplify stage's: both walk the last level's cells. Measured on the VM's client
     * walks of 2026-10-20 (earthlike 16k, 300 Myr, three runs): amplify 29, 28 and 30 s, bake 21,
     * 30 and 20 s, so 0.67 to 1.07 of it (game-choice: a progress-bar weight calibrated on those
     * runs, the coordinator's call, not a physical value).
     */
    static final double BAKE_PER_AMPLIFY = 0.8;

    /**
     * How far the bake's share of the bar fills by time alone (game-choice: the bake reports no
     * progress, so the bar never claims the end before it is there).
     */
    static final double BAKE_HOLD = 0.95;

    /**
     * The bake's estimated seconds: {@link #BAKE_PER_AMPLIFY} of amplify's; 0 without amplify
     * (that bake writes the coarse levels only, about a second).
     */
    public double bakeSeconds() {
        for (Stage st : stages)
            if ("amplify".equals(st.name()))
                return BAKE_PER_AMPLIFY * Math.max(0, st.seconds());
        return 0;
    }

    /** The stages' estimated seconds, one after another. */
    public double stageSeconds() {
        double s = 0;
        for (Stage st : stages)
            s += Math.max(0, st.seconds());
        return s;
    }

    /** The run's estimated seconds: the stages one after another, then the bake. */
    public double seconds() {
        return stageSeconds() + bakeSeconds();
    }

    /**
     * The estimated seconds done when the group led by {@code stageIndex} (its coupled stages
     * after it report under its index) is {@code stagePermille} done; -1 when the stage is unknown.
     */
    private double doneSeconds(int stageIndex, int stagePermille) {
        if (stageIndex < 0 || stageIndex >= stages.size())
            return -1;
        int end = stageIndex + 1;
        while (end < stages.size() && stages.get(end).coupledTo() != null)
            end++;
        double before = 0, group = 0;
        for (int i = 0; i < end; i++) {
            double s = Math.max(0, stages.get(i).seconds());
            if (i < stageIndex)
                before += s;
            else
                group += s;
        }
        return before + group * Math.min(1000, Math.max(0, stagePermille)) / 1000.0;
    }

    /**
     * The run's progress, 0 to 1, while its stages step: the estimated seconds done over the whole
     * run's, bake included; -1 when the stage is unknown. The session's own total weights every
     * group alike, so the coupled plates group (three quarters of the run's time) counts as one
     * ninth: a third of the way through the plates the bar read 3 % (VM walk 2026-10-20), where
     * this reads 21 % and the run was 18 % of the way through its wall time.
     */
    public double fraction(int stageIndex, int stagePermille) {
        double done = doneSeconds(stageIndex, stagePermille);
        double total = seconds();
        if (done < 0 || total <= 0)
            return -1;
        return Math.min(1, done / total);
    }

    /**
     * Milliseconds a running session has left, from the stages' estimated seconds and the bake
     * after them: the stages before the running group are done, the group (its lead at
     * {@code stageIndex} and the coupled stages after it, reported together) is
     * {@code stagePermille} done. The estimate is for an unloaded machine, so once 2 % of the run
     * is done the rest is scaled by the run's own pace ({@code elapsedMs} against the estimated
     * seconds done). -1 when the stage is unknown. The session's own figure weights every group
     * alike, a coupled group of three stages as one ninth of a nine-stage run: on the VM that read
     * "~55 min left" for a 5-min run.
     */
    public long timeLeftMs(int stageIndex, int stagePermille, long elapsedMs) {
        double done = doneSeconds(stageIndex, stagePermille);
        double total = seconds();
        if (done < 0 || total <= 0)
            return -1;
        double left = total - done;
        if (done >= 0.02 * total && elapsedMs > 0)
            left *= elapsedMs / 1000.0 / done;
        return Math.round(left * 1000);
    }

    /**
     * The bake's wall milliseconds: its estimated seconds at the pace the stages ran
     * ({@code stagesMs} for all of them), or at the estimate's own pace when that is unknown.
     */
    public long bakeMs(long stagesMs) {
        double stagesS = stageSeconds();
        double pace = stagesS > 0 && stagesMs > 0 ? stagesMs / 1000.0 / stagesS : 1;
        return Math.round(bakeSeconds() * pace * 1000);
    }

    /**
     * The run's progress while it bakes, {@code bakeElapsedMs} into the bake after stages that
     * took {@code stagesMs}: the stages' share, and the bake's by the time it is expected to take,
     * held at {@link #BAKE_HOLD} of it until the run is done.
     */
    public double bakeFraction(long stagesMs, long bakeElapsedMs) {
        double total = seconds();
        if (total <= 0)
            return 1;
        long expect = bakeMs(stagesMs);
        double part = expect <= 0 ? BAKE_HOLD : Math.min(BAKE_HOLD, Math.max(0, bakeElapsedMs) / (double) expect);
        return (stageSeconds() + bakeSeconds() * part) / total;
    }

    /** Milliseconds the bake has left, {@code bakeElapsedMs} into it after stages that took {@code stagesMs}; 0 once overdue. */
    public long bakeLeftMs(long stagesMs, long bakeElapsedMs) {
        return Math.max(0, bakeMs(stagesMs) - Math.max(0, bakeElapsedMs));
    }

    /** True when the run cannot fit {@code capBytes}: its base alone is above it. */
    public boolean exceeds(long capBytes) {
        return baseBytes > capBytes;
    }

    /** The native memory the run peaks at under {@code capBytes}: the base and what of the history fits. */
    public long peakUnder(long capBytes) {
        return baseBytes + Math.min(historyBytes, Math.max(0, capBytes - baseBytes));
    }

    /** The history that goes to disk under {@code capBytes} (0 when it all fits). */
    public long spillUnder(long capBytes) {
        return historyBytes - (peakUnder(capBytes) - baseBytes);
    }
}
