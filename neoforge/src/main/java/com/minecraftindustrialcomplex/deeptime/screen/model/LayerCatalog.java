// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * How the timelapse groups and names keyframe layers, the same way the browser workbench does
 * ({@code crates/deeptime-viewer/assets/workbench.js}: {@code DEFAULT_LAYERS}, {@code LAYER_GROUPS},
 * {@code layerGroup}, {@code layerLabel}). A real planet's frames carry some sixty layers in all
 * (24 on every step of the coupled stages, then one frame each from deposits, young and amplify),
 * too many for one list on a Minecraft screen, so the run page shows one group at a time.
 *
 * <p>A layer's group is the part of its name before the first {@code /} ({@code climate/koppen} →
 * {@code climate}); names without one are the planet group ({@code ""}). Labels are language keys:
 * {@code deeptime.layer.<name with / as .>} first, then the word for the last part
 * ({@code deeptime.layer.word.<base>}, the workbench's {@code LAYER_WORDS}, with a unit suffix such
 * as {@code _m_myr} taken off), then the humanised name.
 */
public final class LayerCatalog {

    private LayerCatalog() {}

    /** The layer shown until the player picks another: the first of these a run has (workbench DEFAULT_LAYERS). */
    public static final List<String> DEFAULT_LAYERS = List.of("satellite", "elevation");

    /** Group order (workbench LAYER_GROUPS, then the end-of-run stages); unknown groups follow in first-seen order. */
    public static final List<String> GROUP_ORDER = List.of("", "climate", "surface", "structure", "strata", "volcanoes",
            "impacts", "deposits", "input", "young", "amplify");

    /** Unit suffixes the workbench strips from a layer's last part before naming it. */
    private static final String[] UNIT_SUFFIXES = {"_m_myr", "_mm", "_km", "_m", "_c"};

    /** The layer's group: the part before the first {@code /}, or {@code ""}. */
    public static String group(String layer) {
        int i = layer.indexOf('/');
        return i < 0 ? "" : layer.substring(0, i);
    }

    /** The groups of these layers, in {@link #GROUP_ORDER} (unknown ones after, in first-seen order). */
    public static List<String> groups(Collection<String> layers) {
        List<String> seen = new ArrayList<>();
        for (String l : layers) {
            String g = group(l);
            if (!seen.contains(g))
                seen.add(g);
        }
        List<String> out = new ArrayList<>();
        for (String g : GROUP_ORDER)
            if (seen.contains(g))
                out.add(g);
        for (String g : seen)
            if (!out.contains(g))
                out.add(g);
        return out;
    }

    /** The layers of one group, in the order given. */
    public static List<String> inGroup(Collection<String> layers, String group) {
        List<String> out = new ArrayList<>();
        for (String l : layers)
            if (group(l).equals(group))
                out.add(l);
        return out;
    }

    /** Layers by group, groups in {@link #groups} order. */
    public static Map<String, List<String>> byGroup(Collection<String> layers) {
        Map<String, List<String>> out = new LinkedHashMap<>();
        for (String g : groups(layers))
            out.put(g, inGroup(layers, g));
        return out;
    }

    /** The default layer of these: the first of {@link #DEFAULT_LAYERS} present, else the first; null for none. */
    public static String defaultLayer(List<String> layers) {
        for (String d : DEFAULT_LAYERS)
            if (layers.contains(d))
                return d;
        return layers.isEmpty() ? null : layers.get(0);
    }

    /** The full language key of a layer: {@code deeptime.layer.climate.koppen}. */
    public static String key(String layer) {
        return "deeptime.layer." + layer.replace('/', '.');
    }

    /** The word key for the layer's last part, unit suffix removed: {@code deeptime.layer.word.erosion}. */
    public static String wordKey(String layer) {
        String base = layer.substring(layer.lastIndexOf('/') + 1);
        for (String s : UNIT_SUFFIXES)
            if (base.endsWith(s) && base.length() > s.length()) {
                base = base.substring(0, base.length() - s.length());
                break;
            }
        return "deeptime.layer.word." + base;
    }

    /** The language key of a group's name: {@code deeptime.layer_group.planet} for {@code ""}. */
    public static String groupKey(String group) {
        return "deeptime.layer_group." + (group.isEmpty() ? "planet" : group);
    }
}
