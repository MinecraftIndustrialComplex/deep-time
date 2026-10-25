// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * Trigonometry that gives the <b>same bits</b> in Rust, Java and JS: the cube world's projection
 * ({@link CubeWorld}) runs on it in all three, so the bake, the game and the workbench agree on every
 * block's latitude and longitude bit for bit. The twin of
 * {@code deeptime_format::portable_math} (whose documentation gives the method) and of the pages'
 * {@code DTMath}: IEEE double {@code + − × /}, {@link Math#sqrt} (correctly rounded) and comparisons
 * only, in the same order. Java never fuses {@code a·b + c} on its own (strict floating point since
 * Java 17). {@code ProjectionParityTest} checks every output against the Rust fixture.
 */
public final class PortableMath {

    /** π/2's high part: 33 significant bits (fdlibm {@code pio2_1}). */
    private static final double P1 = 1.57079632673412561417;
    /** π/2 − P1. */
    private static final double P1T = 6.07710050650619224932e-11;
    /** π/2 rounded to double. */
    public static final double HALF_PI = 1.570796326794896558;
    private static final double HALF_PI_LO = 6.123233995736766036e-17;
    /** π rounded to double. */
    public static final double PI = 3.141592653589793116;
    private static final double PI_LO = 1.2246467991473532072e-16;

    private PortableMath() {}

    private static double kernelSin(double r) {
        double z = r * r;
        double p = -1.0 / 6.0
                + z * (1.0 / 120.0
                + z * (-1.0 / 5040.0
                + z * (1.0 / 362880.0
                + z * (-1.0 / 39916800.0
                + z * (1.0 / 6227020800.0
                + z * (-1.0 / 1307674368000.0
                + z * (1.0 / 355687428096000.0
                + z * (-1.0 / 121645100408832000.0
                + z * (1.0 / 51090942171709440000.0
                + z * (-1.0 / 25852016738884976640000.0))))))))));
        return r + r * (z * p);
    }

    private static double kernelCos(double r) {
        double z = r * r;
        double q = 1.0 / 24.0
                + z * (-1.0 / 720.0
                + z * (1.0 / 40320.0
                + z * (-1.0 / 3628800.0
                + z * (1.0 / 479001600.0
                + z * (-1.0 / 87178291200.0
                + z * (1.0 / 20922789888000.0
                + z * (-1.0 / 6402373705728000.0
                + z * (1.0 / 2432902008176640000.0
                + z * (-1.0 / 1124000727777607680000.0
                + z * (1.0 / 620448401733239439360000.0))))))))));
        double hz = 0.5 * z;
        double w = 1.0 - hz;
        return w + (((1.0 - w) - hz) + (z * z) * q);
    }

    /** Rounds half away from zero (no library round, whose ties differ between languages). */
    private static double roundHalfAway(double x) {
        return x >= 0.0 ? Math.floor(x + 0.5) : -Math.floor(-x + 0.5);
    }

    public static double sin(double x) {
        if (!Double.isFinite(x))
            return Double.NaN;
        // x = k·π/2 + r, |r| ≤ π/4 (about).
        double k = roundHalfAway(x * (2.0 / PI));
        double r = (x - k * P1) - k * P1T;
        return switch ((int) Math.floorMod((long) k, 4L)) {
            case 0 -> kernelSin(r);
            case 1 -> kernelCos(r);
            case 2 -> -kernelSin(r);
            default -> -kernelCos(r);
        };
    }

    public static double cos(double x) {
        if (!Double.isFinite(x))
            return Double.NaN;
        double k = roundHalfAway(x * (2.0 / PI));
        double r = (x - k * P1) - k * P1T;
        return switch ((int) Math.floorMod((long) k, 4L)) {
            case 0 -> kernelCos(r);
            case 1 -> -kernelSin(r);
            case 2 -> -kernelCos(r);
            default -> kernelSin(r);
        };
    }

    public static double tan(double x) {
        return sin(x) / cos(x);
    }

    private static double atanUnit(double t) {
        double s = t;
        s = s / (1.0 + Math.sqrt(1.0 + s * s));
        s = s / (1.0 + Math.sqrt(1.0 + s * s));
        s = s / (1.0 + Math.sqrt(1.0 + s * s));
        double z = s * s;
        double p = -1.0 / 3.0
                + z * (1.0 / 5.0
                + z * (-1.0 / 7.0
                + z * (1.0 / 9.0
                + z * (-1.0 / 11.0
                + z * (1.0 / 13.0
                + z * (-1.0 / 15.0
                + z * (1.0 / 17.0
                + z * (-1.0 / 19.0
                + z * (1.0 / 21.0
                + z * (-1.0 / 23.0
                + z * (1.0 / 25.0
                + z * (-1.0 / 27.0
                + z * (1.0 / 29.0
                + z * (-1.0 / 31.0))))))))))))));
        return 8.0 * (s + s * (z * p));
    }

    private static boolean signNegative(double x) {
        return Double.doubleToRawLongBits(x) < 0;
    }

    public static double atan(double x) {
        if (Double.isNaN(x))
            return Double.NaN;
        double a = Math.abs(x);
        double r;
        if (a <= 1.0)
            r = atanUnit(a);
        else if (Double.isInfinite(a))
            r = HALF_PI;
        else
            r = HALF_PI - (atanUnit(1.0 / a) - HALF_PI_LO);
        return x < 0.0 ? -r : r;
    }

    public static double atan2(double y, double x) {
        if (Double.isNaN(x) || Double.isNaN(y))
            return Double.NaN;
        double ax = Math.abs(x), ay = Math.abs(y);
        if (ax == 0.0 && ay == 0.0) {
            double a = signNegative(x) ? PI : 0.0;
            return signNegative(y) ? -a : a;
        }
        double a;
        if (ay <= ax) {
            if (Double.isInfinite(ax))
                a = Double.isInfinite(ay) ? atanUnit(1.0) : 0.0;
            else
                a = atanUnit(ay / ax);
        } else if (Double.isInfinite(ay)) {
            a = HALF_PI;
        } else {
            a = HALF_PI - (atanUnit(ax / ay) - HALF_PI_LO);
        }
        if (x < 0.0)
            a = PI - (a - PI_LO);
        return (y < 0.0 || (y == 0.0 && signNegative(y))) ? -a : a;
    }

    private static double clamp1(double x) {
        return x < -1.0 ? -1.0 : (x > 1.0 ? 1.0 : x);
    }

    public static double asin(double x) {
        if (Double.isNaN(x))
            return Double.NaN;
        x = clamp1(x);
        return atan2(x, Math.sqrt((1.0 - x) * (1.0 + x)));
    }

    public static double acos(double x) {
        if (Double.isNaN(x))
            return Double.NaN;
        x = clamp1(x);
        return atan2(Math.sqrt((1.0 - x) * (1.0 + x)), x);
    }

    public static double toRadians(double deg) {
        return deg * (PI / 180.0);
    }

    public static double toDegrees(double rad) {
        return rad * (180.0 / PI);
    }

    /** Rust's {@code f64::rem_euclid}: the remainder in [0, |m|). */
    public static double remEuclid(double a, double m) {
        double r = a % m;
        return r < 0.0 ? r + Math.abs(m) : r;
    }

    /** Wraps degrees into [0, 360). */
    public static double wrapDeg(double a) {
        double w = remEuclid(a, 360.0);
        return w >= 360.0 ? 0.0 : w;
    }
}
