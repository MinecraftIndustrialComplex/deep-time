// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.List;

/**
 * A message of the planet editor's models: a translation key under {@code deeptime.} (the screen
 * turns it into a translatable component; {@code LangKeysTest} checks every key the models name is
 * in {@code en_us.json}) and its arguments, already formatted as strings.
 *
 * @param blocking true when it keeps Run (or Accept) disabled; false for a warning
 */
public record Problem(String key, List<String> args, boolean blocking) {

    public Problem {
        args = List.copyOf(args);
    }

    /** A problem that blocks Run. */
    public static Problem of(String key, Object... args) {
        return new Problem(key, strings(args), true);
    }

    /** A warning. */
    public static Problem warning(String key, Object... args) {
        return new Problem(key, strings(args), false);
    }

    private static List<String> strings(Object[] args) {
        return java.util.Arrays.stream(args).map(String::valueOf).toList();
    }

    /** The full translation key. */
    public String translationKey() {
        return "deeptime." + key;
    }

    @Override
    public String toString() {
        return (blocking ? "" : "warning ") + key + args;
    }
}
