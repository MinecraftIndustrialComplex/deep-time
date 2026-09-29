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

    public record Stage(String name, long bytes, double seconds) {}

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
                stages.add(new Stage(s.get("name").getAsString(), s.get("bytes").getAsLong(), s.get("seconds").getAsDouble()));
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
