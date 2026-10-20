// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

/**
 * Whether a share counted on a sample of columns is significantly below a floor (2026-10-19). A
 * census counts {@code n} columns of a family and {@code hits} of them on a matching biome; the
 * share's spread from where the sample points fell alone is binomial, about
 * {@code sqrt(p(1-p)/n)}: 0.042 at 0.63 on 132 columns. A floor compared with the bare share fails
 * at that noise; this compares it with the share's one-sided upper confidence bound instead
 * (Wilson's score interval, Wilson 1927, JASA 22: 209), so a family fails only when its whole
 * plausible range sits under the floor. A broken mapping (shares of 0.1 to 0.3, what a random biome
 * gives) still fails on 40 columns.
 */
public final class ShareBound {

    /** One-sided 99 %: the standard normal's 0.99 quantile. Thirteen families on five sets make
     *  about 65 comparisons a gate batch, so 95 % would fail about three of them at their floor. */
    public static final double Z_99 = 2.326; // ledger: BIO-072

    private ShareBound() {
    }

    /** Wilson's score upper bound of {@code hits / n} at the normal quantile {@code z} (1 when {@code n} is 0). */
    public static double wilsonUpper(int hits, int n, double z) {
        if (n <= 0)
            return 1.0;
        double p = hits / (double) n, z2 = z * z;
        double centre = p + z2 / (2.0 * n);
        double half = z * Math.sqrt(p * (1.0 - p) / n + z2 / (4.0 * n * (double) n));
        return Math.min(1.0, (centre + half) / (1.0 + z2 / n));
    }

    /** Whether {@code hits} of {@code n} is significantly (one-sided, {@code z}) below {@code floor}. */
    public static boolean below(int hits, int n, double floor, double z) {
        return wilsonUpper(hits, n, z) < floor;
    }

    /** Wilson's score lower bound of {@code hits / n} at the normal quantile {@code z} (0 when {@code n} is 0). */
    public static double wilsonLower(int hits, int n, double z) {
        if (n <= 0)
            return 0.0;
        double p = hits / (double) n, z2 = z * z;
        double centre = p + z2 / (2.0 * n);
        double half = z * Math.sqrt(p * (1.0 - p) / n + z2 / (4.0 * n * (double) n));
        return Math.max(0.0, (centre - half) / (1.0 + z2 / n));
    }

    /**
     * Whether {@code hits} of {@code n} is significantly (one-sided, {@code z}) above {@code ceiling}: the
     * mirror of {@link #below}, for a share that is bounded rather than banned (the ice cap's ice spikes,
     * 2026-10-20), whose expected value is not zero and so moves with the sample like a floor's.
     */
    public static boolean above(int hits, int n, double ceiling, double z) {
        return wilsonLower(hits, n, z) > ceiling;
    }
}
