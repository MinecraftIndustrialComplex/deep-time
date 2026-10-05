// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.preset.PresetException;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * {@code config/deeptime/server.json}: what a dedicated server generates on its first start (plan
 * §7.13, P1-13). Plain JSON; keys starting with {@code _} are explanations and ignored, any other
 * unknown key is an error (a typo must not silently fall back to a default).
 *
 * <pre>{@code
 * {"preset": "earthlike_quick_16k" | "<file>" | "DT1.<share code>",
 *  "timeline": {"start": "supercontinent" | "scattered_continents" | "random", "length_myr": 1000},
 *  "memory_cap_gib": 4, "threads": 0}
 * }</pre>
 *
 * <p>The timeline has no default (decisions file, "Timeline"; review 2, gap 21): {@code timeline}
 * is required unless the preset itself names one (a share code exported from the editor), and
 * when both do, this file wins.
 *
 * @param preset       a built-in preset name, a preset file (relative to {@code config/deeptime/},
 *                     then the server folder) or a share code
 * @param timeline     the timeline to run, if this file names one
 * @param memoryCapGib the native memory cap in GiB (plan §5.7.5; default 4)
 * @param threads      simulation threads, 0 = all processors (plan §6.2)
 */
public record ServerConfig(String preset, Optional<Presets.Timeline> timeline, double memoryCapGib, int threads) {

    public static final String FILE = "server.json";
    public static final double DEFAULT_MEMORY_CAP_GIB = 4.0;

    /** A config that cannot be used; {@link #problems()} names each field. */
    public static final class Invalid extends Exception {
        private static final long serialVersionUID = 1L;
        private final List<String> problems;

        Invalid(List<String> problems) {
            super(String.join("; ", problems));
            this.problems = List.copyOf(problems);
        }

        public List<String> problems() {
            return problems;
        }
    }

    /** The native memory cap in bytes. */
    public long memoryCapBytes() {
        return (long) (memoryCapGib * (1L << 30));
    }

    /** Parses and checks the file's text. */
    public static ServerConfig parse(String text) throws Invalid {
        JsonObject o;
        try {
            JsonElement e = Presets.parseStrict(text);
            if (!e.isJsonObject())
                throw new Invalid(List.of("the file must hold one JSON object"));
            o = e.getAsJsonObject();
        } catch (PresetException e) {
            throw new Invalid(List.of("not valid JSON: " + e.getMessage()));
        }
        List<String> problems = new ArrayList<>();
        for (Map.Entry<String, JsonElement> e : o.entrySet()) {
            String k = e.getKey();
            if (!k.startsWith("_") && !List.of("preset", "timeline", "memory_cap_gib", "threads").contains(k))
                problems.add(k + ": unknown key (keys starting with _ are comments)");
        }

        String preset = "";
        JsonElement p = o.get("preset");
        if (p == null)
            problems.add("preset: missing");
        else if (!p.isJsonPrimitive() || !p.getAsJsonPrimitive().isString())
            problems.add("preset: expected a string");
        else if ((preset = p.getAsString().strip()).isEmpty())
            problems.add("preset: empty; name a built-in preset (" + String.join(", ", Presets.BUILTIN)
                    + "), a preset file or a share code (DT1.…)");

        Optional<Presets.Timeline> timeline = Optional.empty();
        JsonElement t = o.get("timeline");
        if (t != null && !t.isJsonNull()) {
            if (!t.isJsonObject()) {
                problems.add("timeline: expected {\"start\": …, \"length_myr\": …}");
            } else {
                JsonObject to = t.getAsJsonObject();
                for (String k : to.keySet())
                    if (!k.startsWith("_") && !k.equals("start") && !k.equals("length_myr"))
                        problems.add("timeline." + k + ": unknown key");
                JsonElement s = to.get("start");
                String start = s != null && s.isJsonPrimitive() && s.getAsJsonPrimitive().isString() ? s.getAsString().strip() : "";
                JsonElement l = to.get("length_myr");
                double length = l != null && l.isJsonPrimitive() && l.getAsJsonPrimitive().isNumber() ? l.getAsDouble() : Double.NaN;
                Presets.Timeline tl = new Presets.Timeline(start, length);
                List<String> tp = tl.problems("timeline");
                problems.addAll(tp);
                if (tp.isEmpty())
                    timeline = Optional.of(tl);
            }
        }

        double cap = DEFAULT_MEMORY_CAP_GIB;
        JsonElement c = o.get("memory_cap_gib");
        if (c != null) {
            if (!c.isJsonPrimitive() || !c.getAsJsonPrimitive().isNumber() || !(c.getAsDouble() >= 0.25) || c.getAsDouble() > 1 << 20)
                problems.add("memory_cap_gib: expected a number of GiB, at least 0.25 (default 4)");
            else
                cap = c.getAsDouble();
        }

        int threads = 0;
        JsonElement th = o.get("threads");
        if (th != null) {
            if (!th.isJsonPrimitive() || !th.getAsJsonPrimitive().isNumber() || th.getAsDouble() != Math.rint(th.getAsDouble())
                    || th.getAsDouble() < 0 || th.getAsDouble() > 4096)
                problems.add("threads: expected a whole number ≥ 0 (0 = all processors)");
            else
                threads = th.getAsInt();
        }

        if (!problems.isEmpty())
            throw new Invalid(problems);
        return new ServerConfig(preset, timeline, cap, threads);
    }

    /**
     * The file written when there is none: valid JSON (review 2, gap 21), with the explanations in
     * {@code _} keys. As written it does not start a world: {@code preset} and {@code timeline}
     * are empty on purpose, because there is no default story.
     */
    public static String template() {
        return """
                {
                  "_about": "Deep Time generates this world's geologic history once, on the server's first start, before the world is created. Fill in preset and timeline, then start the server again. Keys that start with _ are explanations and are ignored.",
                  "preset": "",
                  "_preset": "A built-in preset: earthlike_quick_16k or earthlike_full_16k for level-type=deeptime:planet (C = 16384 blocks, drawn as a 13312 x 6656 Petroff-Guyou world), earthlike_quick_32k or earthlike_full_32k for level-type=deeptime:planet_32k (C = 32768, a 26112 x 13056 world). A preset with scale.projection lambert needs deeptime:planet_lambert or deeptime:planet_lambert_32k. Or a preset file (a path relative to this folder or to the server folder), or a share code from the Create World screen (DT1.…). The preset's size and projection must match the level-type in server.properties.",
                  "timeline": {
                    "start": "",
                    "length_myr": 0
                  },
                  "_timeline": "There is no default story. start: supercontinent, scattered_continents or random. length_myr: how much history to simulate, in millions of years (1 to 4600). A share code that already names a timeline may leave this out; when both do, this one wins.",
                  "memory_cap_gib": 4,
                  "_memory_cap_gib": "Native memory the simulation may use, in GiB (default 4). Raise it freely on a big machine; it is separate from the server's -Xmx.",
                  "threads": 0,
                  "_threads": "Simulation threads; 0 uses every processor (the server is not running yet)."
                }
                """;
    }
}
