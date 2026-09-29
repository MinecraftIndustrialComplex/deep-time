// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.util.ArrayList;
import java.util.List;

/**
 * A preset's memory and time estimate (plan §5.7.5, §6.1 {@code estimate}):
 * {@code {"peak_bytes":P,"stages":[{"bytes":B,"name":N,"seconds":S},…]}}.
 */
public record Estimate(long peakBytes, List<Stage> stages) {

    public record Stage(String name, long bytes, double seconds) {}

    public Estimate {
        stages = List.copyOf(stages);
    }

    public static Estimate parse(String json) {
        JsonObject o = JsonParser.parseString(json).getAsJsonObject();
        List<Stage> stages = new ArrayList<>();
        if (o.has("stages"))
            for (JsonElement e : o.getAsJsonArray("stages")) {
                JsonObject s = e.getAsJsonObject();
                stages.add(new Stage(s.get("name").getAsString(), s.get("bytes").getAsLong(), s.get("seconds").getAsDouble()));
            }
        return new Estimate(o.get("peak_bytes").getAsLong(), stages);
    }

    /** The run's estimated seconds: the stages run one after another. */
    public double seconds() {
        double s = 0;
        for (Stage st : stages)
            s += st.seconds();
        return s;
    }

    /** True when the peak is above {@code capBytes}. */
    public boolean exceeds(long capBytes) {
        return peakBytes > capBytes;
    }
}
