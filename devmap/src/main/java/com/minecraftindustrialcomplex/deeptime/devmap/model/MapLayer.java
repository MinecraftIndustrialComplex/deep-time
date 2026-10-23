// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import java.util.Locale;
import java.util.function.Predicate;

/**
 * The dev map's two pictures of the planet, on the same frame ({@link MapFrame}) under the same
 * marks. The server draws both once per level; the client shows the one the player chose (the S key
 * on the map screen, its button, the client config's {@code mapLayer}; satellite by default).
 */
public enum MapLayer {

    /**
     * True colour, as the workbench's "Final world" and the Create World screen's satellite view
     * draw it: land cover from the biomes and climate, water, ice and relief shading. Drawn by Deep
     * Time's natives ({@code NativeWorldMap}); a server without them has none.
     */
    SATELLITE("Satellite"),
    /** The terrain height through the shared elevation ramp, lakes by their surface: drawn in Java ({@link BaseImage}). */
    ELEVATION("Elevation");

    /** The picture the map shows until the player picks one. */
    public static final MapLayer DEFAULT = SATELLITE;

    private final String label;

    MapLayer(String label) {
        this.label = label;
    }

    /** The name the buttons and the readout use. */
    public String label() {
        return label;
    }

    /** The id on the wire and in the config ({@code satellite}, {@code elevation}). */
    public String id() {
        return name().toLowerCase(Locale.ROOT);
    }

    /** The other picture (the S key's switch). */
    public MapLayer other() {
        return this == SATELLITE ? ELEVATION : SATELLITE;
    }

    /** The layer of id {@code id}, or {@link #DEFAULT} for anything else. */
    public static MapLayer parse(String id) {
        for (MapLayer l : values())
            if (l.id().equals(id))
                return l;
        return DEFAULT;
    }

    /**
     * The picture to show when the player chose {@code wanted}: it when {@code has} says it is here,
     * otherwise the other one when that is here (a map is better than none while the chosen picture
     * is on its way, or when the server could not draw it), otherwise null.
     */
    public static MapLayer shown(MapLayer wanted, Predicate<MapLayer> has) {
        if (has.test(wanted))
            return wanted;
        return has.test(wanted.other()) ? wanted.other() : null;
    }
}
