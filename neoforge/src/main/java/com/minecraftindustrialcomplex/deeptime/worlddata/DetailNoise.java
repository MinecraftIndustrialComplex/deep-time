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
 * @param cube          a cube world's net (format 0.7), on which {@link #at} reads the noise on the
 *                      cube's surface ({@link #cubeAt}); null on a Lambert-frame world
 */
public record DetailNoise(long seed, int circumference, int salt, int octaves, double wavelength, CubeLayout cube) {

    /** A world without turning edges (the Lambert frame). */
    public DetailNoise(long seed, int circumference, int salt, int octaves, double wavelength) {
        this(seed, circumference, salt, octaves, wavelength, null);
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
            CubeLayout cube = wp instanceof WorldProjection.Cube c ? c.cube() : null;
            return new DetailNoise(seed, m.circumference(), salt, octaves, wavelength, cube);
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
     * N at block (x, z) with this world's octaves, wavelength and salt (Rust {@code DetailSpec::noise}):
     * the {@link #fbm}, or on a cube world {@link #cubeAt}.
     */
    public double at(double x, double z) {
        if (cube != null)
            return cubeAt(x, z);
        return fbm(x, z, wavelength, octaves, salt);
    }

    /** The lattice value at the point (ix, iy, iz) of a cube's surface lattice for {@code salt} (a u32), in [−1, 1]. */
    public double lattice3(int salt, long ix, long iy, long iz) {
        long a = mix(seed ^ (Integer.toUnsignedLong(salt) * 0x9E3779B97F4A7C15L));
        long b = mix(a ^ (ix * 0xD6E8FEB86659FD93L));
        long c = mix(b ^ (iy * 0xA0761D6478BD642FL));
        long h = mix(c ^ (iz * 0xE7037ED1A0B428DBL));
        return (double) (h >>> 11) * (2.0 / 9_007_199_254_740_992.0) - 1.0;
    }

    /**
     * Value noise of wavelength {@code wl} blocks on the surface of a cube of side {@code s} blocks
     * (Rust {@code Noise::surface_value}; ledger: SCL-215, game-choice), at the surface point
     * (p0, p1, p2), blocks from the cube's centre in the cube frame: its face's axis is {@code fixed}
     * (0 x, 1 y, 2 z), where the point is +s/2 when {@code positive} and −s/2 otherwise. The lattice has
     * m = max(1, round(s/λ)) cells along every cube edge, index l = (p + s/2)·m/s on each axis, so two
     * faces share the lattice points of their common edge; a face blends its own square bilinearly with
     * the fade f²(3 − 2f) along its two free axes, the lower axis first.
     */
    public double surfaceValue(double s, double p0, double p1, double p2, int fixed, boolean positive, double wl, int salt) {
        double half = 0.5 * s;
        double m = Math.max((double) Math.round(s / wl), 1.0);
        double q1 = fixed == 0 ? p1 : p0;
        double q2 = fixed == 2 ? p1 : p2;
        double l1 = (q1 + half) * m / s;
        double l2 = (q2 + half) * m / s;
        double fl1 = Math.floor(l1), fl2 = Math.floor(l2);
        double f1 = l1 - fl1, f2 = l2 - fl2;
        double t1 = f1 * f1 * (3.0 - 2.0 * f1);
        double t2 = f2 * f2 * (3.0 - 2.0 * f2);
        long i1 = (long) fl1, i2 = (long) fl2;
        long k = positive ? (long) m : 0L;
        double h00 = corner(salt, fixed, k, i1, i2);
        double h10 = corner(salt, fixed, k, i1 + 1, i2);
        double h01 = corner(salt, fixed, k, i1, i2 + 1);
        double h11 = corner(salt, fixed, k, i1 + 1, i2 + 1);
        double a = h00 + (h10 - h00) * t1;
        double b = h01 + (h11 - h01) * t1;
        return a + (b - a) * t2;
    }

    /** The surface lattice point with index {@code k} on the axis {@code fixed} and (a, b) on the other two, in order. */
    private double corner(int salt, int fixed, long k, long a, long b) {
        return switch (fixed) {
            case 0 -> lattice3(salt, k, a, b);
            case 1 -> lattice3(salt, a, k, b);
            default -> lattice3(salt, a, b, k);
        };
    }

    /** {@code octaves} of {@link #surfaceValue} at λ, λ/2, … (salt + octave), normalised to [−1, 1], as {@link #fbm}. */
    public double surfaceFbm(double s, double p0, double p1, double p2, int fixed, boolean positive, double wl, int octaves, int salt) {
        double sum = 0.0, norm = 0.0, weight = 1.0, w = wl;
        for (int o = 0; o < octaves; o++) {
            sum += weight * surfaceValue(s, p0, p1, p2, fixed, positive, w, salt + o);
            norm += weight;
            weight *= 0.5;
            w *= 0.5;
        }
        return sum / norm;
    }

    /**
     * N at block (x, z) of a cube world (Rust {@code DetailSpec::cube_noise}; cube plan G3; ledger:
     * SCL-215, game-choice): the noise on the cube's surface ({@link #surfaceFbm}) at the block's centre
     * (x + ½, z + ½), brought onto the planet by {@link CubeLayout#fold} of its block (the slide, and past
     * a net edge the edge's motion, which takes block centres to block centres). A block off the faces
     * reads exactly its fold partner's noise, and the noise runs on through every seam and corner with
     * no blend. Past the net's corners it is 0.
     */
    public double cubeAt(double x, double z) {
        double px = x + 0.5, pz = z + 0.5;
        double bx = Math.floor(px), bz = Math.floor(pz);
        CubeLayout.Face face;
        double u, v;
        CubeLayout.Fold f = cube.fold((long) bx, (long) bz);
        if (f instanceof CubeLayout.OnFace on) {
            face = on.face();
            u = (double) on.i() + (px - bx);
            v = (double) on.j() + (pz - bz);
        } else if (f instanceof CubeLayout.Outside out) {
            CubeLayout.FaceCell fc = cube.faceOfBlock(out.x(), out.z());
            if (fc == null)
                return 0.0;
            double[] r = out.iso().point(px, pz);
            face = fc.face();
            u = (double) fc.i() + (r[0] - (double) out.x());
            v = (double) fc.j() + (r[1] - (double) out.z());
        } else {
            return 0.0;
        }
        double s = (double) cube.side();
        double half = 0.5 * s;
        u = u - half;
        v = v - half;
        double[][] fr = face.frameRef();
        double[] c = fr[0], eu = fr[1], ev = fr[2];
        double p0 = half * c[0] + u * eu[0] + v * ev[0];
        double p1 = half * c[1] + u * eu[1] + v * ev[1];
        double p2 = half * c[2] + u * eu[2] + v * ev[2];
        int fixed = c[0] != 0.0 ? 0 : c[1] != 0.0 ? 1 : 2;
        return surfaceFbm(s, p0, p1, p2, fixed, c[fixed] > 0.0, wavelength, octaves, salt);
    }
}
