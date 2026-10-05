// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.Map;

/**
 * Event colours by kind, the browser workbench's ({@code workbench.js}, {@code KIND_COLOURS} and
 * {@code kindColour}): the listed kinds have fixed colours, any other kind a hue from its name
 * (HSL with 70 % saturation and 60 % lightness), so the in-game ticker and map read like the
 * workbench's. Colours are opaque ARGB.
 */
public final class EventStyle {

    private EventStyle() {}

    private static final Map<String, Integer> KIND_COLOURS = Map.ofEntries(
            Map.entry("collision", 0xe6194b), Map.entry("orogeny_phase", 0xb5651d), Map.entry("subduction_onset", 0xb04adf),
            Map.entry("ridge_birth", 0x4363d8), Map.entry("rift_onset", 0xf58231), Map.entry("terrane_accretion", 0xbfef45),
            Map.entry("hotspot_track", 0xff5a1f), Map.entry("large_igneous_province", 0xff1f1f), Map.entry("plate_birth", 0x3cb44b),
            Map.entry("plate_death", 0xa9a9a9), Map.entry("volcano_arc", 0xe8552d), Map.entry("volcano_hotspot_chain", 0xf0a020),
            Map.entry("volcano_stratovolcano", 0xff3b1f), Map.entry("volcano_shield", 0xffc02e),
            Map.entry("volcano_caldera_complex", 0xd45cff), Map.entry("volcano_caldera", 0xff79c6),
            Map.entry("volcano_flood_basalt", 0x9a8a7a), Map.entry("impact_crater", 0x7df9ff), Map.entry("impact_ocean", 0x5aa9c9),
            Map.entry("erosion_pulse", 0xffe119), Map.entry("basin_subsidence", 0x42d4f4), Map.entry("marine_transgression", 0x1e90ff),
            Map.entry("marine_regression", 0x9ecae1), Map.entry("glaciation", 0xe8f4ff), Map.entry("deposit_formation", 0xffd700));

    /** The kind without its namespace. */
    public static String shortKind(String kind) {
        return kind.substring(kind.indexOf(':') + 1);
    }

    /** The colour of a kind, opaque ARGB. */
    public static int colour(String kind) {
        String k = shortKind(kind);
        Integer c = KIND_COLOURS.get(k);
        if (c != null)
            return 0xff000000 | c;
        long h = 0;
        for (int i = 0; i < k.length(); i++)
            h = (h * 31 + k.charAt(i)) & 0xffffffffL;
        return 0xff000000 | hsl(h % 360, 0.70, 0.60);
    }

    /** CSS's HSL to RGB (h in degrees, s and l in 0…1). */
    static int hsl(double h, double s, double l) {
        double a = s * Math.min(l, 1 - l);
        int r = channel(0, h, a, l), g = channel(8, h, a, l), b = channel(4, h, a, l);
        return r << 16 | g << 8 | b;
    }

    private static int channel(int n, double h, double a, double l) {
        double k = (n + h / 30) % 12;
        double v = l - a * Math.max(-1, Math.min(Math.min(k - 3, 9 - k), 1));
        return (int) Math.round(v * 255);
    }
}
