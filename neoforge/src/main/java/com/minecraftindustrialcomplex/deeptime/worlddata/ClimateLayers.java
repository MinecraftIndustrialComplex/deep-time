// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.ArrayList;
import java.util.List;

/**
 * The climate at a block (FORMAT.md "The climate at a block", P1-25), read from a baked world's L1
 * climate layers: what the public climate API ({@code api.climate.DeepTimeClimate}) answers. The
 * arithmetic is Rust's {@code deeptime_format::ClimateWorld::sample}, operation for operation, so
 * both agree bit for bit ({@code ClimateParityTest} against {@code parity/climate.json}).
 *
 * <p>Split in two so a caller can cache the part that does not depend on the height:
 * {@link #column} samples the rasters at (x, z) (the twelve sea-level months, the annual
 * precipitation, the normalised monthly shares, the Köppen class) and {@link Column#at} applies the
 * lapse rate at a block height. Thread-safe.
 */
public final class ClimateLayers {

    /** Köppen index of the sea ({@code notes.bake.climate.koppen_codes[31]}). */
    public static final int SEA = 31;

    private final int circumference;
    private final int seaLevelY;
    private final double mPerBlockV;
    private final double lapseCPerKm;
    private final RasterSampler[] tSl = new RasterSampler[12];
    private final RasterSampler[] pShare = new RasterSampler[12];
    private final RasterSampler pMm;
    private final RasterSampler koppen;
    private final List<String> codes;

    private ClimateLayers(WorldData wd, double lapseCPerKm, List<String> codes) {
        this.circumference = wd.circumference();
        this.seaLevelY = wd.manifest().seaLevelY();
        this.mPerBlockV = wd.manifest().mPerBlockV();
        this.lapseCPerKm = lapseCPerKm;
        for (int m = 0; m < 12; m++) {
            tSl[m] = wd.layer(tSlLayer(m));
            pShare[m] = wd.layer(pShareLayer(m));
        }
        this.pMm = wd.layer("climate/p_mm");
        this.koppen = wd.layer("climate/koppen");
        this.codes = codes;
    }

    /** Layer name of month {@code m}'s (0–11) sea-level temperature. */
    public static String tSlLayer(int m) {
        return String.format("climate/month/t_sl_c_%02d", m + 1);
    }

    /** Layer name of month {@code m}'s (0–11) precipitation share. */
    public static String pShareLayer(int m) {
        return String.format("climate/month/p_share_%02d", m + 1);
    }

    /**
     * The climate layers of {@code wd}, or null when it has no twelve months (a synthetic world,
     * or a run without a climate section).
     */
    public static ClimateLayers of(WorldData wd) {
        JsonObject notes = wd.manifest().notes();
        JsonObject climate = obj(obj(notes, "bake"), "climate");
        JsonObject monthly = obj(climate, "monthly");
        if (monthly == null)
            return null;
        JsonElement lapse = monthly.get("lapse_c_per_km");
        if (lapse == null || !lapse.isJsonPrimitive() || !lapse.getAsJsonPrimitive().isNumber())
            throw new WorldDataException(wd.folder() + ": notes.bake.climate.monthly.lapse_c_per_km is not a number");
        List<String> codes = new ArrayList<>();
        JsonElement arr = climate.get("koppen_codes");
        if (arr instanceof JsonArray a)
            a.forEach(e -> codes.add(e.isJsonPrimitive() ? e.getAsString() : "?"));
        for (int m = 0; m < 12; m++)
            for (String name : new String[] {tSlLayer(m), pShareLayer(m)})
                if (!wd.has(name))
                    throw new WorldDataException(wd.folder() + ": notes.bake.climate.monthly is set but the layer "
                            + name + " is missing");
        return new ClimateLayers(wd, lapse.getAsDouble(), List.copyOf(codes));
    }

    private static JsonObject obj(JsonObject o, String key) {
        if (o == null)
            return null;
        JsonElement e = o.get(key);
        return e instanceof JsonObject j ? j : null;
    }

    public double lapseCPerKm() {
        return lapseCPerKm;
    }

    public int seaLevelY() {
        return seaLevelY;
    }

    public double mPerBlockV() {
        return mPerBlockV;
    }

    /** Block x folded into the canonical span [−C/2, C/2). */
    public long wrapX(long x) {
        return Math.floorMod(x + circumference / 2, (long) circumference) - circumference / 2;
    }

    /** The code of Köppen index {@code k} ({@code sea} for 31, {@code ?} if unknown). */
    public String code(int k) {
        if (k == SEA)
            return "sea";
        return k >= 0 && k < codes.size() ? codes.get(k) : "?";
    }

    /** The height-independent part at block column (x, z), x folded here. */
    public Column column(long x, long z) {
        long wx = wrapX(x);
        double bx = (double) wx, bz = (double) z;
        double[] t = new double[12];
        double[] share = new double[12];
        double s = 0.0;
        for (int m = 0; m < 12; m++) {
            t[m] = tSl[m].bilinear(bx, bz);
            share[m] = pShare[m].bilinear(bx, bz);
            s += share[m];
        }
        double pAnnual = Math.max(pMm.bilinear(bx, bz), 0.0);
        double[] p = new double[12];
        for (int m = 0; m < 12; m++) {
            double sm = s > 0.0 ? 12.0 * share[m] / s : 1.0;
            p[m] = pAnnual * sm / 12.0;
        }
        double k = koppen.nearest(bx, bz);
        int ki = Double.isNaN(k) ? SEA : (int) Math.max(0, Math.min(SEA, Math.round(k)));
        return new Column(this, t, pAnnual, p, ki);
    }

    /** The climate at block (x, y, z). */
    public Sample sample(long x, int y, long z) {
        return column(x, z).at(x, y, z);
    }

    /**
     * What (x, z) holds regardless of height: the twelve sea-level months (°C), the annual and
     * monthly precipitation (mm) and the Köppen index.
     */
    public record Column(ClimateLayers layers, double[] tSl, double pAnnual, double[] pMonth, int koppen) {

        /** The climate at height {@code y} of this column, reported as block (x, y, z). */
        public Sample at(long x, int y, long z) {
            double elevationM = (double) Math.max(y - layers.seaLevelY, 0) * layers.mPerBlockV;
            double lapse = layers.lapseCPerKm * elevationM / 1000.0;
            double[] t = new double[12];
            for (int m = 0; m < 12; m++)
                t[m] = tSl[m] - lapse;
            double sum = 0.0;
            int warmest = 0, coldest = 0;
            for (int m = 0; m < 12; m++) {
                sum += t[m];
                if (t[m] > t[warmest])
                    warmest = m;
                if (t[m] < t[coldest])
                    coldest = m;
            }
            return new Sample(x, y, z, elevationM, t, sum / 12.0, t[warmest], t[coldest], warmest, coldest,
                    pAnnual, pMonth.clone(), koppen, layers.code(koppen));
        }
    }

    /** The climate at one block: Rust's {@code ClimateSample}, field for field. */
    public record Sample(long x, int y, long z, double elevationM, double[] tMonth, double tMean, double tWarm,
                         double tCold, int warmest, int coldest, double pAnnual, double[] pMonth, int koppen,
                         String koppenCode) {}
}
