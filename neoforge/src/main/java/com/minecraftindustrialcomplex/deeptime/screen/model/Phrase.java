// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.Arrays;
import java.util.List;
import java.util.Locale;

/**
 * A sentence the screens show, as a translation key under {@code deeptime.} and its arguments,
 * client-free so JUnit checks the wording logic (docs/design/create-world-ux.md: one convention for
 * time, plain words). An argument is a string or another phrase (the screen translates it first).
 * The screen side is {@code Lang.phrase}.
 *
 * @param key  the key without {@code deeptime.}
 * @param args strings and phrases
 */
public record Phrase(String key, List<Object> args) {

    public Phrase {
        args = List.copyOf(args);
    }

    public static Phrase of(String key, Object... args) {
        return new Phrase(key, Arrays.stream(args).map(a -> a instanceof Phrase ? a : (Object) String.valueOf(a)).toList());
    }

    /** Text shown as it is (a class name from ramps.json, a preset's own name): the empty key and the text as the one argument. */
    public static Phrase literal(String text) {
        return new Phrase("", List.of(text));
    }

    public boolean isLiteral() {
        return key.isEmpty();
    }

    /** The full translation key. */
    public String translationKey() {
        return "deeptime." + key;
    }

    // ------------------------------------------------------------------------------------------
    // Time: one convention everywhere, "N million years ago"
    // ------------------------------------------------------------------------------------------

    /**
     * Simulated time (years relative to world creation, negative in the past) in words:
     * {@code 640 million years ago}, {@code 1.2 billion years ago}, {@code 3.5 million years ago},
     * {@code 800 thousand years ago}, {@code today}, {@code 50 million years from now} (a continued
     * run goes past the present).
     */
    public static Phrase yearsAgo(double years) {
        if (!Double.isFinite(years))
            return of("time.unknown");
        double a = Math.abs(years);
        boolean future = years > 0;
        if (a < 500)
            return of("time.today");
        String unit;
        String n;
        if (a >= 999.5e6) {
            unit = "billion";
            n = oneDecimal(a / 1e9);
        } else if (a >= 0.9995e6) {
            unit = "million";
            n = a >= 9.95e6 ? whole(a / 1e6) : oneDecimal(a / 1e6);
        } else {
            unit = "thousand";
            n = whole(a / 1e3);
        }
        return of("time." + unit + (future ? ".ahead" : ".ago"), n);
    }

    /** Millions of years ago as a bare number for a column headed "Million years ago": {@code 640}, {@code 3.5}, {@code 0}. */
    public static String myrAgo(double years) {
        double a = -years / 1e6;
        if (!Double.isFinite(a))
            return "";
        if (Math.abs(a) < 0.05)
            return "0";
        return Math.abs(a) >= 9.95 ? whole(a) : oneDecimal(a);
    }

    /** Milliseconds left in words: {@code about 4 min left}, {@code less than a minute left}, {@code about 1 h 20 min left}. */
    public static Phrase timeLeft(long ms) {
        if (ms < 0)
            return of("time.left.unknown");
        long s = (ms + 999) / 1000;
        if (s < 60)
            return of("time.left.under_minute");
        long min = (s + 59) / 60;
        if (min < 90)
            return of("time.left.minutes", min);
        return of("time.left.hours", min / 60, min % 60);
    }

    /** A duration in words, for estimates: {@code about 4 min}, {@code under a minute}, {@code about 1 h 20 min}. */
    public static Phrase duration(double seconds) {
        if (!(seconds >= 0))
            return of("time.unknown");
        if (seconds < 60)
            return of("time.duration.under_minute");
        long min = Math.round(Math.ceil(seconds / 60));
        if (min < 90)
            return of("time.duration.minutes", min);
        return of("time.duration.hours", min / 60, min % 60);
    }

    private static String whole(double v) {
        return String.format(Locale.ROOT, "%,d", Math.round(v));
    }

    private static String oneDecimal(double v) {
        String s = String.format(Locale.ROOT, "%.1f", v);
        return s.endsWith(".0") ? s.substring(0, s.length() - 2) : s;
    }
}
