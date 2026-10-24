// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.preset;

import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import com.google.gson.stream.JsonReader;
import com.google.gson.stream.JsonToken;

import java.io.IOException;
import java.io.InputStream;
import java.io.StringReader;
import java.io.UncheckedIOException;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalDouble;
import java.util.OptionalInt;
import java.util.regex.Pattern;

/**
 * Presets on the Java side (plan §5.1): the built-in presets shipped in the jar, preset arguments
 * (built-in name, file or share code), the header fields the game reads before running
 * (circumference, timeline, placeholder), and canonical JSON. The Rust core owns validation; this
 * class only reads what the game needs to decide before it calls the natives.
 *
 * <p>Built-in presets are {@code crates/deeptime-core/presets/*.json}, copied into the jar at
 * {@code deeptime/presets/<name>.json} by Gradle's {@code processResources}. Since P1-08 they
 * are simulated planets (the planet pipeline under its game names); a preset marked {@code placeholder}
 * (the dummy pipeline plus a synthetic terrain, P1-13's stand-ins) still runs, with a warning.
 */
public final class Presets {

    /** The built-in presets, in the editor's order (plan §5.1). */
    public static final List<String> BUILTIN = List.of(
            "earthlike_quick_16k", "earthlike_full_16k", "earthlike_quick_32k", "earthlike_full_32k");

    /** Where the built-in presets sit in the jar. */
    public static final String RESOURCE_DIR = "/deeptime/presets/";

    /** The timeline start configurations (plan §5.1; Rust {@code StartConfig}). */
    public static final List<String> START_CONFIGS = List.of("supercontinent", "scattered_continents", "random");

    /** The shortest and longest history, in millions of years (Rust {@code LENGTH_MYR_RANGE}). */
    public static final double MIN_LENGTH_MYR = 1.0;
    public static final double MAX_LENGTH_MYR = 4600.0;

    private static final Gson GSON = new Gson();
    private static final Pattern NAME = Pattern.compile("[a-z0-9_]+");
    private static final Pattern INTEGER = Pattern.compile("-?(0|[1-9][0-9]*)");
    private static final BigInteger U64_MAX = BigInteger.ONE.shiftLeft(64).subtract(BigInteger.ONE);
    private static final BigInteger I64_MIN = BigInteger.valueOf(Long.MIN_VALUE);

    private Presets() {}

    /** The player's timeline choice. */
    public record Timeline(String start, double lengthMyr) {

        /** Checks the values; returns the problems (empty when valid), each naming the field. */
        public List<String> problems(String path) {
            List<String> p = new ArrayList<>();
            if (start == null || !START_CONFIGS.contains(start))
                p.add(path + ".start: " + (start == null || start.isEmpty() ? "empty" : "'" + start + "'")
                        + "; choose one of " + String.join(", ", START_CONFIGS));
            if (!(lengthMyr >= MIN_LENGTH_MYR && lengthMyr <= MAX_LENGTH_MYR))
                p.add(path + ".length_myr: " + fmt(lengthMyr) + "; choose a number of million years from "
                        + fmt(MIN_LENGTH_MYR) + " to " + fmt(MAX_LENGTH_MYR));
            return p;
        }

        @Override
        public String toString() {
            return start + ", " + fmt(lengthMyr) + " Myr";
        }
    }

    /** What the game reads from a preset before running it. */
    public record Info(Optional<String> name, boolean placeholder, OptionalInt circumference,
                       Optional<Timeline> timeline, Optional<String> description, String projection) {

        /**
         * The world's x period in blocks, what the level type's {@code circumference_blocks} and Toroidal
         * World's loop hold: 4s on the cube (Rust {@code ScaleParams::world_circumference}), C on a
         * synthetic world laid out in the Lambert frame; empty when the preset names no circumference.
         */
        public OptionalInt worldCircumference() {
            return circumference.isEmpty() ? circumference : OptionalInt.of(Presets.worldCircumference(circumference.getAsInt(), projection));
        }
    }

    /**
     * A preset's projection: a planet is always the cube world (Ben, decisions log 2026-10-24: "Cube
     * only"; a planet preset has no {@code scale.projection}). Only a synthetic world says
     * {@code lambert} there (Rust {@code preset::header}).
     */
    public static final String DEFAULT_PROJECTION = "cube";

    /**
     * The world's x period for a simulation circumference C and a projection (see {@link Info#worldCircumference}):
     * 4s on the cube (s = {@code face_for(C)}, Rust {@code ScaleParams::world_circumference}), C on a
     * Lambert-frame synthetic world.
     */
    public static int worldCircumference(int c, String projection) {
        if ("lambert".equals(projection))
            return c;
        return 4 * com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout.faceFor(c);
    }

    /** A resolved preset argument: where it came from, and its JSON. */
    public record Resolved(String label, String json) {}

    // --------------------------------------------------------------------------------------------
    // Built-ins and arguments
    // --------------------------------------------------------------------------------------------

    /** The JSON of a built-in preset, read from the jar. */
    public static Optional<String> builtin(String name) {
        if (!BUILTIN.contains(name))
            return Optional.empty();
        try (InputStream in = Presets.class.getResourceAsStream(RESOURCE_DIR + name + ".json")) {
            if (in == null)
                throw new PresetException("the built-in preset " + name + " is missing from the jar ("
                        + RESOURCE_DIR + name + ".json); reinstall Deep Time");
            return Optional.of(new String(in.readAllBytes(), StandardCharsets.UTF_8));
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    /**
     * Resolves a preset argument: a share code ({@code DT1.…}), a built-in name, or a file (an
     * absolute path, or relative to {@code baseDirs} in order).
     */
    public static Resolved resolve(String arg, Path... baseDirs) {
        String a = arg == null ? "" : arg.strip();
        if (a.isEmpty())
            throw new PresetException("no preset given; name a built-in preset (" + String.join(", ", BUILTIN)
                    + "), a preset file or a share code (DT1.…)");
        if (ShareCode.isShareCode(a))
            return new Resolved("share code " + (a.length() > 16 ? a.substring(0, 16) + "…" : a), ShareCode.decode(a));
        Optional<String> b = builtin(a);
        if (b.isPresent())
            return new Resolved(a, b.get());
        List<Path> tried = new ArrayList<>();
        try {
            Path p = Path.of(a);
            if (p.isAbsolute())
                tried.add(p);
            else
                for (Path base : baseDirs)
                    tried.add(base.resolve(p).normalize());
        } catch (InvalidPathException e) {
            throw new PresetException("'" + a + "' is not a built-in preset, a share code or a valid path");
        }
        for (Path p : tried) {
            if (Files.isRegularFile(p)) {
                try {
                    return new Resolved(p.toString(), Files.readString(p, StandardCharsets.UTF_8));
                } catch (IOException e) {
                    throw new PresetException("cannot read the preset file " + p + ": " + e, e);
                }
            }
        }
        if (NAME.matcher(a).matches())
            throw new PresetException("'" + a + "' is not a built-in preset (" + String.join(", ", BUILTIN)
                    + "), a preset file or a share code (DT1.…)");
        throw new PresetException("the preset file " + a + " does not exist (looked at " + tried + ")");
    }

    // --------------------------------------------------------------------------------------------
    // Reading and editing
    // --------------------------------------------------------------------------------------------

    /** Parses strict JSON (RFC 8259: no comments, no trailing commas, one value). */
    public static JsonElement parseStrict(String text) {
        try {
            JsonReader r = new JsonReader(new StringReader(text));
            r.setLenient(false);
            JsonElement e = GSON.getAdapter(JsonElement.class).read(r);
            if (r.peek() != JsonToken.END_DOCUMENT)
                throw new PresetException("more than one JSON value");
            return e;
        } catch (IOException | RuntimeException e) {
            if (e instanceof PresetException pe)
                throw pe;
            throw new PresetException(e.getMessage() == null ? e.toString() : e.getMessage(), e);
        }
    }

    /** Parses a preset's JSON object. */
    public static JsonObject parseObject(String json) {
        JsonElement e = parseStrict(json);
        if (!e.isJsonObject())
            throw new PresetException("a preset is a JSON object");
        return e.getAsJsonObject();
    }

    /** The header fields of a preset (plan §5.1; Rust {@code PresetHeader}). */
    public static Info info(JsonObject preset) {
        Optional<String> name = string(preset, "name");
        boolean placeholder = preset.has("placeholder") && preset.get("placeholder").isJsonPrimitive()
                && preset.get("placeholder").getAsJsonPrimitive().isBoolean() && preset.get("placeholder").getAsBoolean();
        OptionalInt c = OptionalInt.empty();
        if (preset.has("scale") && preset.get("scale").isJsonObject()) {
            JsonElement ce = preset.getAsJsonObject("scale").get("circumference_blocks");
            if (ce != null && ce.isJsonPrimitive() && ce.getAsJsonPrimitive().isNumber()) {
                BigDecimal v = ce.getAsBigDecimal();
                if (v.signum() > 0 && v.stripTrailingZeros().scale() <= 0 && v.compareTo(BigDecimal.valueOf(1 << 24)) <= 0)
                    c = OptionalInt.of(v.intValueExact());
            }
        }
        Optional<Timeline> timeline = Optional.empty();
        if (preset.has("timeline") && preset.get("timeline").isJsonObject()) {
            JsonObject t = preset.getAsJsonObject("timeline");
            timeline = Optional.of(new Timeline(string(t, "start").orElse(""), number(t, "length_myr").orElse(Double.NaN)));
        }
        String projection = DEFAULT_PROJECTION;
        if (preset.has("scale") && preset.get("scale").isJsonObject())
            projection = string(preset.getAsJsonObject("scale"), "projection").orElse(DEFAULT_PROJECTION);
        return new Info(name, placeholder, c, timeline, string(preset, "description"), projection);
    }

    /** Sets {@code timeline} in a preset and returns its canonical JSON. */
    public static String withTimeline(JsonObject preset, Timeline t) {
        JsonObject copy = preset.deepCopy();
        JsonObject tl = new JsonObject();
        tl.addProperty("start", t.start());
        tl.addProperty("length_myr", t.lengthMyr()); // a double, "1000.0", as Rust's with_timeline writes it
        copy.add("timeline", tl);
        return canonical(copy);
    }

    /**
     * Sets {@code timeline.stop_myr} (Myr after the start at which the run stops: less than the
     * length stops it early, more continues it; docs/design/playability-steering.md) and returns
     * the canonical JSON. The preset must carry a timeline.
     */
    public static String withStop(String presetJson, double stopMyr) {
        JsonObject copy = parseObject(presetJson);
        if (!copy.has("timeline") || !copy.get("timeline").isJsonObject())
            throw new PresetException("a stop needs a timeline");
        copy.getAsJsonObject("timeline").addProperty("stop_myr", stopMyr);
        return canonical(copy);
    }

    /** Where a preset's run stops, Myr after its start: {@code timeline.stop_myr}, else its length. */
    public static OptionalDouble stopMyr(JsonObject preset) {
        if (!preset.has("timeline") || !preset.get("timeline").isJsonObject())
            return OptionalDouble.empty();
        JsonObject t = preset.getAsJsonObject("timeline");
        Optional<Double> stop = number(t, "stop_myr");
        if (stop.isPresent())
            return OptionalDouble.of(stop.get());
        return number(t, "length_myr").map(OptionalDouble::of).orElse(OptionalDouble.empty());
    }

    private static Optional<String> string(JsonObject o, String key) {
        JsonElement e = o.get(key);
        return e != null && e.isJsonPrimitive() && e.getAsJsonPrimitive().isString()
                ? Optional.of(e.getAsString()) : Optional.empty();
    }

    private static Optional<Double> number(JsonObject o, String key) {
        JsonElement e = o.get(key);
        return e != null && e.isJsonPrimitive() && e.getAsJsonPrimitive().isNumber()
                ? Optional.of(e.getAsDouble()) : Optional.empty();
    }

    // --------------------------------------------------------------------------------------------
    // Canonical JSON (serde_json's output, so both sides write one text per preset)
    // --------------------------------------------------------------------------------------------

    /** Canonical JSON of a text: see {@link #canonical(JsonElement)}. */
    public static String canonical(String json) {
        return canonical(parseStrict(json));
    }

    /**
     * Canonical JSON, byte for byte what Rust's {@code serde_json::Value::to_string} writes: no
     * whitespace; object keys sorted by code point (UTF-8 byte order); strings escaping only
     * {@code "}, {@code \} and control characters ({@code \b \t \n \f \r}, else backslash-u00xx);
     * integers that fit u64/i64 as written; every other number as the shortest round-trip double
     * in serde_json's layout (its float formatter zmij: {@code 1000.0}, {@code 0.001}, {@code 1e-7}, {@code 1.5e+300}).
     */
    public static String canonical(JsonElement e) {
        StringBuilder b = new StringBuilder();
        write(b, e);
        return b.toString();
    }

    private static void write(StringBuilder b, JsonElement e) {
        if (e == null || e.isJsonNull()) {
            b.append("null");
        } else if (e.isJsonObject()) {
            List<Map.Entry<String, JsonElement>> entries = new ArrayList<>(e.getAsJsonObject().entrySet());
            entries.sort((x, y) -> compareCodePoints(x.getKey(), y.getKey()));
            b.append('{');
            for (int i = 0; i < entries.size(); i++) {
                if (i > 0)
                    b.append(',');
                string(b, entries.get(i).getKey());
                b.append(':');
                write(b, entries.get(i).getValue());
            }
            b.append('}');
        } else if (e.isJsonArray()) {
            JsonArray a = e.getAsJsonArray();
            b.append('[');
            for (int i = 0; i < a.size(); i++) {
                if (i > 0)
                    b.append(',');
                write(b, a.get(i));
            }
            b.append(']');
        } else {
            JsonPrimitive p = e.getAsJsonPrimitive();
            if (p.isBoolean())
                b.append(p.getAsBoolean());
            else if (p.isNumber())
                b.append(canonicalNumber(p.getAsNumber().toString()));
            else
                string(b, p.getAsString());
        }
    }

    static int compareCodePoints(String x, String y) {
        int i = 0, j = 0;
        while (i < x.length() && j < y.length()) {
            int a = x.codePointAt(i), c = y.codePointAt(j);
            if (a != c)
                return Integer.compare(a, c);
            i += Character.charCount(a);
            j += Character.charCount(c);
        }
        return Integer.compare(x.length() - i, y.length() - j);
    }

    private static void string(StringBuilder b, String s) {
        b.append('"');
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '"' -> b.append("\\\"");
                case '\\' -> b.append("\\\\");
                case '\b' -> b.append("\\b");
                case '\t' -> b.append("\\t");
                case '\n' -> b.append("\\n");
                case '\f' -> b.append("\\f");
                case '\r' -> b.append("\\r");
                default -> {
                    if (c < 0x20)
                        b.append(String.format(Locale.ROOT, "\\u%04x", (int) c));
                    else
                        b.append(c);
                }
            }
        }
        b.append('"');
    }

    /** A JSON number literal as serde_json writes it back (see {@link #canonical(JsonElement)}). */
    static String canonicalNumber(String literal) {
        if (INTEGER.matcher(literal).matches()) {
            BigInteger v = new BigInteger(literal);
            if (v.signum() >= 0 ? v.compareTo(U64_MAX) <= 0 : v.compareTo(I64_MIN) >= 0)
                return v.toString();
        }
        double d = Double.parseDouble(literal);
        if (Double.isInfinite(d) || Double.isNaN(d))
            return "null"; // serde_json writes a non-finite f64 as null
        return shortestDouble(d);
    }

    /** serde_json's float layout (zmij, serde_json 1.0.151) over Java's shortest round-trip digits. */
    static String shortestDouble(double d) {
        if (d == 0.0)
            return (1 / d < 0) ? "-0.0" : "0.0";
        StringBuilder out = new StringBuilder();
        if (d < 0) {
            out.append('-');
            d = -d;
        }
        BigDecimal v = new BigDecimal(Double.toString(d)).stripTrailingZeros();
        String digits = v.unscaledValue().toString();
        int length = digits.length();
        int k = -v.scale();       // value = digits × 10^k
        int kk = length + k;      // position of the decimal point
        if (k >= 0 && kk <= 16) {
            out.append(digits).append("0".repeat(k)).append(".0");
        } else if (kk > 0 && kk <= 16) {
            out.append(digits, 0, kk).append('.').append(digits, kk, length);
        } else if (kk > -5 && kk <= 0) {
            out.append("0.").append("0".repeat(-kk)).append(digits);
        } else {
            out.append(digits, 0, 1);
            if (length > 1)
                out.append('.').append(digits, 1, length);
            int exp = kk - 1;
            out.append('e').append(exp >= 0 ? "+" : "").append(exp);
        }
        return out.toString();
    }

    static String fmt(double v) {
        if (Double.isNaN(v))
            return "missing";
        return v == Math.rint(v) && Math.abs(v) < 1e15 ? Long.toString((long) v) : Double.toString(v);
    }
}
