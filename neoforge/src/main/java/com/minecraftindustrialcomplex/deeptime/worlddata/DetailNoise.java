// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;

/**
 * The value noise N of the terrain's sub-cell detail term A·N (FORMAT.md, "The detail layer" and
 * "The terrain height"; Rust {@code deeptime_format::detail::Noise}, operation for operation):
 * IEEE-754 double arithmetic with {@code + − × /}, {@code floor} and {@code round} only, and integers
 * hashed with splitmix64's finaliser, so both sides agree bit for bit ({@code DetailNoiseTest} pins
 * values Rust's test pins too; {@code ColumnParityTest} compares whole columns).
 *
 * <p>Value noise of wavelength λ blocks: n = max(1, round(C/λ)) lattice cells around the planet,
 * spacing s = C/n; u = (x + C/2)/s, v = z/s; the corners (⌊u⌋ mod n, ⌊v⌋) … (+1, +1) get lattice
 * values in [−1, 1] and are blended bilinearly with the fade f²(3 − 2f). {@link #fbm} sums
 * {@code octaves} of them at λ, λ/2, … with weights 1, ½, … (salt + octave), normalised. Periodic
 * in x with period C, which keeps the wrap seamless. Immutable and thread-safe.
 *
 * @param seed          the noise seed (a u64 in two's complement)
 * @param circumference C, blocks
 * @param salt          the salt of the first octave (a u32)
 * @param octaves       octaves of {@link #fbm}
 * @param wavelength    the first octave's wavelength, blocks
 * @param halfTurnB     b of a Petroff–Guyou world (format 0.5), whose north and south edges are glued
 *                      onto themselves by half-turns and {@link #at} folds and blends there; 0 on a
 *                      Lambert or synthetic world
 */
public record DetailNoise(long seed, int circumference, int salt, int octaves, double wavelength, long halfTurnB) {

    /** The width of the band inside a Petroff–Guyou world's turning edges where the noise is blended
     *  towards its half-turn-symmetric mean, blocks (Rust {@code EDGE_BLEND_BLOCKS}; ledger: SCL-089, game-choice). */
    public static final double EDGE_BLEND_BLOCKS = 32.0; // ledger: SCL-089 (game-choice)

    /** A world without turning edges. */
    public DetailNoise(long seed, int circumference, int salt, int octaves, double wavelength) {
        this(seed, circumference, salt, octaves, wavelength, 0L);
    }

    /** The layer the amplitude comes from ({@code notes.bake.detail.layer}). */
    public static final String DEFAULT_LAYER = "detail/amplitude";

    /**
     * The detail term a manifest declares ({@code notes.bake.detail}), or null for a world without
     * one (the synthetic worlds: their terrain is the {@code elevation} layer alone).
     */
    public static DetailNoise of(Manifest m) {
        JsonObject bake = m.notes().has("bake") && m.notes().get("bake").isJsonObject()
                ? m.notes().getAsJsonObject("bake") : null;
        if (bake == null || !bake.has("detail") || !bake.get("detail").isJsonObject())
            return null;
        JsonObject d = bake.getAsJsonObject("detail");
        try {
            long seed = Long.parseUnsignedLong(d.get("seed").getAsString());
            int salt = (int) d.get("salt").getAsLong();
            int octaves = d.get("octaves").getAsInt();
            double wavelength = d.get("wavelength_blocks").getAsDouble();
            if (octaves < 1 || octaves > 16 || !(wavelength > 0) || Double.isInfinite(wavelength))
                throw new WorldDataException("notes.bake.detail: octaves " + octaves + ", wavelength " + wavelength);
            WorldProjection wp = m.projection();
            long b = wp instanceof PetroffGuyou g ? g.hemisphere() : 0L;
            return new DetailNoise(seed, m.circumference(), salt, octaves, wavelength, b);
        } catch (RuntimeException e) {
            if (e instanceof WorldDataException w)
                throw w;
            throw new WorldDataException("notes.bake.detail is malformed: " + e.getMessage(), e);
        }
    }

    /** The amplitude layer's name in {@code notes.bake.detail} (the default when absent). */
    public static String layerOf(Manifest m) {
        try {
            JsonObject d = m.notes().getAsJsonObject("bake").getAsJsonObject("detail");
            return d.has("layer") ? d.get("layer").getAsString() : DEFAULT_LAYER;
        } catch (RuntimeException e) {
            return DEFAULT_LAYER;
        }
    }

    /** splitmix64's finaliser. */
    static long mix(long x) {
        x = (x ^ (x >>> 30)) * 0xBF58476D1CE4E5B9L;
        x = (x ^ (x >>> 27)) * 0x94D049BB133111EBL;
        return x ^ (x >>> 31);
    }

    /** The lattice value at (ix, jz) for {@code salt} (a u32), in [−1, 1]. */
    public double lattice(int salt, long ix, long jz) {
        long a = mix(seed ^ (Integer.toUnsignedLong(salt) * 0x9E3779B97F4A7C15L));
        long b = mix(a ^ (ix * 0xD6E8FEB86659FD93L));
        long h = mix(b ^ (jz * 0xA0761D6478BD642FL));
        return (double) (h >>> 11) * (2.0 / 9_007_199_254_740_992.0) - 1.0;
    }

    /** Value noise of wavelength {@code wl} at block (x, z), in [−1, 1]. */
    public double value(double x, double z, double wl, int salt) {
        double c = circumference;
        double n = Math.max((double) Math.round(c / wl), 1.0);
        double s = c / n;
        double u = (x + c / 2.0) / s;
        double v = z / s;
        double fu = Math.floor(u), fv = Math.floor(v);
        double fx = u - fu, fz = v - fv;
        long ni = (long) n;
        long ix0 = Math.floorMod((long) fu, ni);
        long ix1 = (ix0 + 1) % ni;
        long jz0 = (long) fv;
        long jz1 = jz0 + 1;
        double tx = fx * fx * (3.0 - 2.0 * fx);
        double tz = fz * fz * (3.0 - 2.0 * fz);
        double h00 = lattice(salt, ix0, jz0);
        double h10 = lattice(salt, ix1, jz0);
        double h01 = lattice(salt, ix0, jz1);
        double h11 = lattice(salt, ix1, jz1);
        double a = h00 + (h10 - h00) * tx;
        double b = h01 + (h11 - h01) * tx;
        return a + (b - a) * tz;
    }

    /** {@code octaves} of {@link #value} at λ, λ/2, … (salt + octave), normalised to [−1, 1]. */
    public double fbm(double x, double z, double wl, int octaves, int salt) {
        double sum = 0.0, norm = 0.0, weight = 1.0, w = wl;
        for (int o = 0; o < octaves; o++) {
            sum += weight * value(x, z, w, salt + o);
            norm += weight;
            weight *= 0.5;
            w *= 0.5;
        }
        return sum / norm;
    }

    /**
     * N at block (x, z) with this world's octaves, wavelength and salt (Rust {@code DetailSpec::noise}).
     * On a Petroff–Guyou world a point past the north or south edge is first folded through the edge's
     * half-turn, and within {@link #EDGE_BLEND_BLOCKS} of an edge the noise is blended towards its
     * half-turn-symmetric mean S = ½(N + N(mirror)), so the detail term is continuous through the seam.
     */
    public double at(double x, double z) {
        if (halfTurnB == 0L)
            return fbm(x, z, wavelength, octaves, salt);
        double b = (double) halfTurnB;
        double h = 0.5 * b;
        for (int k = 0; k < 64; k++) {
            if (z < -h) {
                x = -x;
                z = -b - z;
            } else if (z > h) {
                x = -x;
                z = b - z;
            } else {
                break;
            }
        }
        double n = fbm(x, z, wavelength, octaves, salt);
        double d, mz;
        if (z < 0.0) {
            d = z + h;
            mz = -b - z;
        } else {
            d = h - z;
            mz = b - z;
        }
        if (d >= EDGE_BLEND_BLOCKS)
            return n;
        double m = fbm(-x, mz, wavelength, octaves, salt);
        double s = 0.5 * (n + m);
        double t = d / EDGE_BLEND_BLOCKS;
        double w = t * t * (3.0 - 2.0 * t);
        return s + (n - s) * w;
    }
}
