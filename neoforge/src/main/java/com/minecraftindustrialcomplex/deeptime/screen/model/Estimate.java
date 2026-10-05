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

    /** The run's estimated seconds: the stages run one after another. */
    public double seconds() {
        double s = 0;
        for (Stage st : stages)
            s += st.seconds();
        return s;
    }

    /**
     * Milliseconds a running session has left, from the stages' estimated seconds: the stages
     * before the running group are done, the group (its lead at {@code stageIndex} and the
     * coupled stages after it, reported together) is {@code stagePermille} done. The estimate is
     * for an unloaded machine, so once 2 % of the run is done the rest is scaled by the run's own
     * pace ({@code elapsedMs} against the estimated seconds done). -1 when the stage is unknown.
     * The session's own figure weights every group alike, a coupled group of three stages as one
     * ninth of a nine-stage run: on the VM that read "~55 min left" for a 5-min run.
     */
    public long timeLeftMs(int stageIndex, int stagePermille, long elapsedMs) {
        if (stageIndex < 0 || stageIndex >= stages.size())
            return -1;
        int end = stageIndex + 1;
        while (end < stages.size() && stages.get(end).coupledTo() != null)
            end++;
        double before = 0, group = 0, total = 0;
        for (int i = 0; i < stages.size(); i++) {
            double s = Math.max(0, stages.get(i).seconds());
            total += s;
            if (i < stageIndex)
                before += s;
            else if (i < end)
                group += s;
        }
        if (total <= 0)
            return -1;
        double done = before + group * Math.min(1000, Math.max(0, stagePermille)) / 1000.0;
        double left = total - done;
        if (done >= 0.02 * total && elapsedMs > 0)
            left *= elapsedMs / 1000.0 / done;
        return Math.round(left * 1000);
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
