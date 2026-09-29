// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.Locale;

/**
 * Number formats of the planet editor (plan §7.12): bytes, durations and simulated time. Plain
 * strings, the same on every locale, as the server's progress lines (P1-13) print them.
 */
public final class Formats {

    private static final double KIB = 1024.0, MIB = KIB * 1024, GIB = MIB * 1024;

    private Formats() {}

    /** {@code 512 B}, {@code 3.4 KiB}, {@code 12.0 MiB}, {@code 1.3 GiB}. */
    public static String bytes(long b) {
        if (b < 0)
            return "?";
        if (b < 1024)
            return b + " B";
        if (b < MIB)
            return String.format(Locale.ROOT, "%.1f KiB", b / KIB);
        if (b < GIB)
            return String.format(Locale.ROOT, "%.1f MiB", b / MIB);
        return String.format(Locale.ROOT, "%.1f GiB", b / GIB);
    }

    /** Bytes as GiB with one decimal, no unit ({@code 4.0}). */
    public static String gib(long b) {
        return String.format(Locale.ROOT, "%.1f", b / GIB);
    }

    /** GiB (a player's number) to bytes, rounded; clamps at {@link Long#MAX_VALUE}. */
    public static long gibToBytes(double gib) {
        double b = gib * GIB;
        return b >= Long.MAX_VALUE ? Long.MAX_VALUE : Math.round(b);
    }

    /** {@code 5s}, {@code 2m05s}, {@code 1h02m}. */
    public static String duration(long ms) {
        long s = Math.max(0, ms) / 1000;
        if (s < 60)
            return s + "s";
        if (s < 3600)
            return (s / 60) + "m" + String.format(Locale.ROOT, "%02d", s % 60) + "s";
        return (s / 3600) + "h" + String.format(Locale.ROOT, "%02d", (s / 60) % 60) + "m";
    }

    /** Seconds (an estimate) as a duration; under a second is {@code <1s}. */
    public static String seconds(double s) {
        if (!(s >= 0))
            return "?";
        if (s < 1)
            return "<1s";
        return duration(Math.round(s * 1000));
    }

    /**
     * Simulated time (years relative to world creation) as the viewer prints it:
     * {@code 312.0 Myr ago}, {@code 4.5 kyr ago}, {@code the present}, {@code +2.0 Myr}.
     */
    public static String time(double years) {
        if (!Double.isFinite(years))
            return "";
        if (years == 0)
            return "the present";
        double a = Math.abs(years);
        String s = a >= 1e6 ? String.format(Locale.ROOT, "%.1f Myr", a / 1e6)
                : a >= 1e3 ? String.format(Locale.ROOT, "%.1f kyr", a / 1e3)
                : String.format(Locale.ROOT, "%.0f yr", a);
        return years < 0 ? s + " ago" : "+" + s;
    }

    /** Simulated time in progress form (years × −1): {@code −312 Ma}, {@code 0 Ma}. */
    public static String ma(long yearsAgo) {
        long ma = Math.round(yearsAgo / 1e6);
        return ma == 0 ? "0 Ma" : (ma > 0 ? "−" + ma : "+" + (-ma)) + " Ma";
    }

    /** Permille as a whole percent: {@code 37%}. */
    public static String percent(int permille) {
        return Math.max(0, Math.min(1000, permille)) / 10 + "%";
    }

    /**
     * A number for an edit box: integers without a fraction, others with up to six significant
     * decimals and no trailing zeros ({@code 9.81}, {@code 300}, {@code 0.001}).
     */
    public static String number(double v) {
        if (!Double.isFinite(v))
            return "";
        if (v == Math.rint(v) && Math.abs(v) < 1e15)
            return Long.toString((long) v);
        String s = String.format(Locale.ROOT, "%.6f", v);
        s = s.replaceAll("0+$", "");
        return s.endsWith(".") ? s.substring(0, s.length() - 1) : s;
    }

    /** A number typed by the player, or NaN ({@code ,} accepted as the decimal point). */
    public static double parse(String text) {
        if (text == null)
            return Double.NaN;
        String t = text.strip().replace(',', '.');
        if (t.isEmpty())
            return Double.NaN;
        try {
            double v = Double.parseDouble(t);
            return Double.isFinite(v) ? v : Double.NaN;
        } catch (NumberFormatException e) {
            return Double.NaN;
        }
    }

    /** {@code rift_onset} or {@code deeptime:rift_onset} → {@code Rift onset}. */
    public static String humanize(String id) {
        String s = id == null ? "" : id.substring(id.indexOf(':') + 1);
        s = s.substring(s.lastIndexOf('/') + 1).replace('_', ' ');
        return s.isEmpty() ? s : Character.toUpperCase(s.charAt(0)) + s.substring(1);
    }
}
