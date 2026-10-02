// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.climate;

import java.util.Locale;

/**
 * The simulated climate at one block of a Deep Time level ({@link DeepTimeClimate#at}). Immutable.
 * Temperatures are °C at the block's height; months are 0–11 (0 ≈ January, see
 * {@link DeepTimeClimate}); precipitation is millimetres of water.
 */
public final class ClimateSample {

    private final int x, y, z;
    private final double elevationM;
    private final double lapseCPerKm;
    private final double[] tMonth;
    private final double tMean, tWarm, tCold;
    private final int warmest, coldest;
    private final double pAnnual;
    private final double[] pMonth;
    private final String koppen;

    /** Built by Deep Time; arrays are copied. */
    public ClimateSample(int x, int y, int z, double elevationM, double lapseCPerKm, double[] tMonth, double tMean,
                         double tWarm, double tCold, int warmest, int coldest, double pAnnual, double[] pMonth,
                         String koppen) {
        if (tMonth.length != DeepTimeClimate.MONTHS || pMonth.length != DeepTimeClimate.MONTHS)
            throw new IllegalArgumentException("twelve months expected");
        this.x = x;
        this.y = y;
        this.z = z;
        this.elevationM = elevationM;
        this.lapseCPerKm = lapseCPerKm;
        this.tMonth = tMonth.clone();
        this.tMean = tMean;
        this.tWarm = tWarm;
        this.tCold = tCold;
        this.warmest = warmest;
        this.coldest = coldest;
        this.pAnnual = pAnnual;
        this.pMonth = pMonth.clone();
        this.koppen = koppen;
    }

    public int x() {
        return x;
    }

    public int y() {
        return y;
    }

    public int z() {
        return z;
    }

    /** Height above the sea the lapse rate was applied over, metres (0 at and below sea level). */
    public double elevationMetres() {
        return elevationM;
    }

    /** The simulation's lapse rate, °C per km. */
    public double lapseRateCPerKm() {
        return lapseCPerKm;
    }

    /** Annual mean temperature, °C. */
    public double meanC() {
        return tMean;
    }

    /** Mean temperature of the warmest month, °C. */
    public double warmestMonthC() {
        return tWarm;
    }

    /** Mean temperature of the coldest month, °C. */
    public double coldestMonthC() {
        return tCold;
    }

    /** The warmest month, 0–11. */
    public int warmestMonth() {
        return warmest;
    }

    /** The coldest month, 0–11. */
    public int coldestMonth() {
        return coldest;
    }

    /** Mean temperature of month {@code m} (0–11), °C. */
    public double monthC(int m) {
        return tMonth[Math.floorMod(m, DeepTimeClimate.MONTHS)];
    }

    /** The twelve monthly mean temperatures, °C (a copy). */
    public double[] monthlyC() {
        return tMonth.clone();
    }

    /**
     * Mean temperature at year fraction {@code f} (any real; its fractional part counts), °C: linear
     * between the months' midpoints, periodic over the year. Its mean over the year is
     * {@link #meanC()}.
     */
    public double temperatureC(double f) {
        return interpolate(tMonth, f);
    }

    /** Annual precipitation, mm. */
    public double annualPrecipitationMm() {
        return pAnnual;
    }

    /** Precipitation in month {@code m} (0–11), mm (the twelve sum to the annual). */
    public double monthPrecipitationMm(int m) {
        return pMonth[Math.floorMod(m, DeepTimeClimate.MONTHS)];
    }

    /** The twelve monthly precipitations, mm (a copy). */
    public double[] monthlyPrecipitationMm() {
        return pMonth.clone();
    }

    /** Precipitation rate at year fraction {@code f}, mm per year (the months' totals × 12, interpolated). */
    public double precipitationRateMmPerYear(double f) {
        return 12.0 * interpolate(pMonth, f);
    }

    /** The wettest month, 0–11. */
    public int wettestMonth() {
        int best = 0;
        for (int m = 1; m < DeepTimeClimate.MONTHS; m++)
            if (pMonth[m] > pMonth[best])
                best = m;
        return best;
    }

    /** The driest month, 0–11. */
    public int driestMonth() {
        int best = 0;
        for (int m = 1; m < DeepTimeClimate.MONTHS; m++)
            if (pMonth[m] < pMonth[best])
                best = m;
        return best;
    }

    /**
     * Precipitation seasonality: the coefficient of variation of the monthly totals, percent
     * (0 = even all year, above ~100 = a strong dry season). This is the standard deviation
     * (n - 1) of the twelve totals over their mean. WorldClim's BIO15 as written up by O'Donnell and
     * Ignizio (2012, eq. 16) divides by the mean plus 1 mm instead, to avoid huge values where the
     * mean is under 1 mm; the two differ by under 2 % where the mean is above 50 mm. Ledger BIO-095.
     */
    public double precipitationSeasonality() {
        double mean = pAnnual / DeepTimeClimate.MONTHS;
        if (mean <= 0.0)
            return 0.0;
        double ss = 0.0;
        for (double p : pMonth)
            ss += (p - mean) * (p - mean);
        return 100.0 * Math.sqrt(ss / (DeepTimeClimate.MONTHS - 1)) / mean;
    }

    /** The Köppen class code (e.g. {@code Cfb}, {@code BWh}, {@code EF}), or {@code sea}. */
    public String koppen() {
        return koppen;
    }

    /** True at sea (no Köppen class). */
    public boolean isSea() {
        return "sea".equals(koppen);
    }

    /** The Köppen group, {@code A}–{@code E}, or {@code 0} at sea. */
    public char koppenGroup() {
        return isSea() || koppen.isEmpty() ? 0 : koppen.charAt(0);
    }

    /** Mean temperature of month {@code m} at sea level, °C (the lapse rate taken back out). */
    public double seaLevelMonthC(int m) {
        return monthC(m) + lapseCPerKm * elevationM / 1000.0;
    }

    private static double interpolate(double[] v, double f) {
        double u = (f - Math.floor(f)) * DeepTimeClimate.MONTHS - 0.5;
        double fu = Math.floor(u);
        double frac = u - fu;
        int m0 = Math.floorMod((int) fu, DeepTimeClimate.MONTHS);
        int m1 = (m0 + 1) % DeepTimeClimate.MONTHS;
        return v[m0] + (v[m1] - v[m0]) * frac;
    }

    @Override
    public String toString() {
        return String.format(Locale.ROOT, "ClimateSample[%d %d %d, %s, mean %.2f C (warmest %.2f, coldest %.2f), %.0f mm/yr]",
                x, y, z, koppen, tMean, tWarm, tCold, pAnnual);
    }
}
