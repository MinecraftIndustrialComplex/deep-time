// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Arrays;

/**
 * A world's climate mapping (plan §7.2, P1-09): five monotone tables that take the biome inputs the
 * bake wrote (placed against the reference bands and vanilla's land distributions, recorded in
 * {@code notes.bake.climate.reference}) to the pack's: its band edges for temperature, vegetation
 * and continentalness, and its own router's land distribution for erosion and peaks-and-valleys.
 * Computed once per world by {@link ClimateMappingBuilder} and stored in the world data folder as
 * {@value #FILE}, so a later pack update never shifts biomes in unexplored chunks. Pure, immutable
 * and thread-safe: the terrain source applies it on every climate sample.
 */
public final class ClimateMapping {

    /** The file in the world data folder. */
    public static final String FILE = "climate_mapping.json";
    public static final int SCHEMA = 1;

    public static final String TEMPERATURE = "climate/temperature";
    public static final String VEGETATION = "climate/vegetation";
    public static final String CONTINENTS = "climate/continents";
    public static final String EROSION = "climate/erosion";
    public static final String RIDGES = "climate/ridges";
    public static final String[] LAYERS = {TEMPERATURE, VEGETATION, CONTINENTS, EROSION, RIDGES};

    /** Maps nothing: the synthetic worlds, and worlds baked before P1-09. */
    public static final ClimateMapping IDENTITY = new ClimateMapping(null, null, null, null, null, null);

    /**
     * Piecewise linear through ({@code from[i]}, {@code to[i]}), {@code from} ascending, clamped to
     * the end values; {@code odd}: applied to |v| with the sign kept (weirdness).
     */
    public record Table(double[] from, double[] to, boolean odd) {

        public Table {
            if (from.length != to.length || from.length < 2)
                throw new IllegalArgumentException("a table needs ≥ 2 matching points");
            for (int i = 1; i < from.length; i++)
                if (!(from[i] > from[i - 1]))
                    throw new IllegalArgumentException("table inputs must ascend: " + Arrays.toString(from));
        }

        public double apply(double v) {
            if (odd)
                return Math.copySign(interp(Math.abs(v)), v);
            return interp(v);
        }

        private double interp(double v) {
            if (Double.isNaN(v))
                return v;
            int n = from.length;
            if (v <= from[0])
                return to[0] + (v - from[0]);   // keep the offset below the table: rare, and continuous
            if (v >= from[n - 1])
                return to[n - 1] + (v - from[n - 1]);
            int lo = 0, hi = n - 1;
            while (hi - lo > 1) {
                int mid = (lo + hi) >>> 1;
                if (from[mid] <= v)
                    lo = mid;
                else
                    hi = mid;
            }
            double f = (v - from[lo]) / (from[hi] - from[lo]);
            return to[lo] + f * (to[hi] - to[lo]);
        }

        /** Largest |to − from| over the points: how far the table moves values. */
        public double maxShift() {
            double m = 0;
            for (int i = 0; i < from.length; i++)
                m = Math.max(m, Math.abs(to[i] - from[i]));
            return m;
        }

        JsonObject toJson() {
            JsonObject o = new JsonObject();
            o.add("from", OverworldBands.json(from));
            o.add("to", OverworldBands.json(to));
            if (odd)
                o.addProperty("odd", true);
            return o;
        }

        static Table fromJson(JsonObject o) {
            return new Table(OverworldBands.array(o.getAsJsonArray("from")), OverworldBands.array(o.getAsJsonArray("to")),
                    o.has("odd") && o.get("odd").getAsBoolean());
        }
    }

    /**
     * A quantile function from sorted samples at u = i / (n − 1), optionally conditioned on a lower
     * or upper limit (the Rust {@code bake::biomes::Quantile}, which the bake used).
     */
    public static final class Quantile {
        private final double[] q;
        private final double lo, hi;

        public Quantile(double[] samples) {
            this(sorted(samples), 0.0, 1.0);
        }

        private Quantile(double[] q, double lo, double hi) {
            if (q.length < 1)
                throw new IllegalArgumentException("no samples");
            this.q = q;
            this.lo = lo;
            this.hi = hi;
        }

        private static double[] sorted(double[] v) {
            double[] c = v.clone();
            Arrays.sort(c);
            return c;
        }

        private double raw(double u) {
            int n = q.length;
            if (n == 1)
                return q[0];
            double x = Math.max(0, Math.min(1, u)) * (n - 1);
            int i = Math.min(n - 2, (int) Math.floor(x));
            return q[i] + (x - i) * (q[i + 1] - q[i]);
        }

        double cdf(double v) {
            int n = q.length;
            if (n == 1 || v <= q[0])
                return 0.0;
            for (int i = 0; i < n - 1; i++) {
                if (v <= q[i + 1]) {
                    double d = q[i + 1] - q[i];
                    double f = d > 0 ? (v - q[i]) / d : 1.0;
                    return (i + f) / (n - 1);
                }
            }
            return 1.0;
        }

        /** Conditioned on values ≤ {@code v}. */
        public Quantile below(double v) {
            return new Quantile(q, lo, lo + (hi - lo) * Math.max(1e-3, cdf(v)));
        }

        /** Conditioned on values ≥ {@code v}. */
        public Quantile above(double v) {
            return new Quantile(q, lo + (hi - lo) * Math.min(1.0 - 1e-3, cdf(v)), hi);
        }

        public double at(double u) {
            return raw(lo + Math.max(0, Math.min(1, u)) * (hi - lo));
        }
    }

    private final Table temperature, vegetation, continents, erosion, ridges;
    private final JsonObject meta;

    public ClimateMapping(Table temperature, Table vegetation, Table continents, Table erosion, Table ridges, JsonObject meta) {
        this.temperature = temperature;
        this.vegetation = vegetation;
        this.continents = continents;
        this.erosion = erosion;
        this.ridges = ridges;
        this.meta = meta == null ? new JsonObject() : meta;
    }

    public boolean isIdentity() {
        return temperature == null && vegetation == null && continents == null && erosion == null && ridges == null;
    }

    /** The table of a layer, or null (identity). */
    public Table table(String layer) {
        return switch (layer) {
            case TEMPERATURE -> temperature;
            case VEGETATION -> vegetation;
            case CONTINENTS -> continents;
            case EROSION -> erosion;
            case RIDGES -> ridges;
            default -> null;
        };
    }

    /** The pack's value of a baked biome input; other layers pass through. */
    public double apply(String layer, double v) {
        Table t = table(layer);
        return t == null ? v : t.apply(v);
    }

    /** What the builder recorded (source, pack distribution, reference, why). */
    public JsonObject meta() {
        return meta;
    }

    /** The table from one set of band edges to another (edge i → edge i). */
    public static Table bandTable(double[] from, double[] to) {
        if (from.length != to.length)
            throw new IllegalArgumentException("band counts differ: " + from.length + " vs " + to.length);
        return new Table(from.clone(), to.clone(), false);
    }

    /**
     * Erosion: the reference's land distribution below its swamp band (E6) onto the pack's below
     * its own, then E6 edge to edge.
     */
    public static Table erosionTable(double[] refQ, double refE6, double[] packQ, double packE6, int points) {
        Quantile r = new Quantile(refQ).below(refE6 - 1e-3), p = new Quantile(packQ).below(packE6 - 1e-3);
        double[] from = new double[points + 2], to = new double[points + 2];
        for (int i = 0; i < points; i++) {
            double u = i / (double) (points - 1);
            from[i] = r.at(u);
            to[i] = p.at(u);
        }
        from[points] = refE6;
        to[points] = packE6;
        from[points + 1] = 1.0;
        to[points + 1] = 1.0;
        return strictly(from, to, false);
    }

    /**
     * Weirdness: |w| on the rising branch, through the reference's and the pack's land
     * peaks-and-valleys distributions above the valley slice (PV = 3|w| − 1).
     */
    public static Table ridgesTable(double[] refPv, double refValley, double[] packPv, double packValley, int points) {
        double refMin = 3 * (refValley + 1.0 / 60.0) - 1, packMin = 3 * (packValley + 1.0 / 60.0) - 1;
        Quantile r = new Quantile(refPv).above(refMin), p = new Quantile(packPv).above(packMin);
        double[] from = new double[points + 2], to = new double[points + 2];
        from[0] = 0;
        to[0] = 0;
        for (int i = 0; i < points; i++) {
            double u = i / (double) (points - 1);
            from[i + 1] = (Math.max(refMin, r.at(u)) + 1) / 3;
            to[i + 1] = (Math.max(packMin, p.at(u)) + 1) / 3;
        }
        from[points + 1] = 1.0;
        to[points + 1] = 1.0;
        return strictly(from, to, true);
    }

    /** Drops points whose input does not rise (flat stretches of a quantile function). */
    static Table strictly(double[] from, double[] to, boolean odd) {
        double[] f = new double[from.length], t = new double[to.length];
        int n = 0;
        for (int i = 0; i < from.length; i++) {
            if (n > 0 && !(from[i] > f[n - 1] + 1e-9)) {
                t[n - 1] = Math.max(t[n - 1], to[i]);
                continue;
            }
            f[n] = from[i];
            t[n] = n > 0 ? Math.max(t[n - 1], to[i]) : to[i];
            n++;
        }
        return new Table(Arrays.copyOf(f, n), Arrays.copyOf(t, n), odd);
    }

    /**
     * The mapping from the reference the bake used onto the pack's bands and distributions.
     *
     * @param refBands   {@code notes.bake.climate.reference.bands}
     * @param refErosion {@code notes.bake.climate.reference.reference.erosion}
     * @param refPv      {@code …peaks_valleys}
     * @param packErosion the pack router's land erosion quantiles, or null to keep the reference's
     * @param packPv      its land peaks-and-valleys quantiles, or null
     */
    public static ClimateMapping build(OverworldBands refBands, double[] refErosion, double[] refPv, OverworldBands packBands,
                                       double[] packErosion, double[] packPv, JsonObject meta) {
        int points = 65;
        Table e = erosionTable(refErosion, refBands.erosion()[6], packErosion == null ? refErosion : packErosion,
                packBands.erosion()[6], points);
        Table r = ridgesTable(refPv, refBands.weirdness()[1], packPv == null ? refPv : packPv, packBands.weirdness()[1], points);
        return new ClimateMapping(bandTable(refBands.temperature(), packBands.temperature()),
                bandTable(refBands.vegetation(), packBands.vegetation()),
                bandTable(refBands.continents(), packBands.continents()), e, r, meta);
    }

    public JsonObject toJson() {
        JsonObject o = new JsonObject();
        o.addProperty("schema", SCHEMA);
        JsonObject tables = new JsonObject();
        for (String l : LAYERS) {
            Table t = table(l);
            if (t != null)
                tables.add(l, t.toJson());
        }
        o.add("tables", tables);
        for (var e : meta.entrySet())
            if (!o.has(e.getKey()))
                o.add(e.getKey(), e.getValue());
        return o;
    }

    public static ClimateMapping fromJson(JsonObject o) {
        int schema = o.has("schema") ? o.get("schema").getAsInt() : 0;
        if (schema != SCHEMA)
            throw new IllegalArgumentException(FILE + ": schema " + schema + ", expected " + SCHEMA);
        JsonObject tables = o.getAsJsonObject("tables");
        Table[] t = new Table[LAYERS.length];
        for (int i = 0; i < LAYERS.length; i++)
            if (tables != null && tables.has(LAYERS[i]))
                t[i] = Table.fromJson(tables.getAsJsonObject(LAYERS[i]));
        JsonObject meta = o.deepCopy();
        meta.remove("tables");
        meta.remove("schema");
        return new ClimateMapping(t[0], t[1], t[2], t[3], t[4], meta);
    }

    public static ClimateMapping read(Path file) throws IOException {
        return fromJson(JsonParser.parseString(Files.readString(file, StandardCharsets.UTF_8)).getAsJsonObject());
    }

    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create();

    /** Writes {@link #toJson} to {@code file} (tmp, then an atomic rename). */
    public void write(Path file) throws IOException {
        Path tmp = file.resolveSibling(file.getFileName() + ".tmp");
        Files.writeString(tmp, GSON.toJson(toJson()) + "\n", StandardCharsets.UTF_8);
        Files.move(tmp, file, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
    }

    /** A one-line summary for the log: how far each table moves values. */
    public String summary() {
        if (isIdentity())
            return "identity";
        StringBuilder b = new StringBuilder();
        for (String l : LAYERS) {
            Table t = table(l);
            b.append(l.substring("climate/".length())).append(' ')
                    .append(t == null ? "=" : String.format(java.util.Locale.ROOT, "±%.3f", t.maxShift())).append(' ');
        }
        return b.toString().trim();
    }

    static JsonArray arr(double[] v) {
        return OverworldBands.json(v);
    }
}
