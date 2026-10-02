// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * {@code impacts/craters.json} (FORMAT.md, "Impact craters"; P2-08): the impact craters the
 * simulation leaves at the present, those that still show, the buried ones and the eroded roots.
 * The game builds nothing from the list (the craters are in the terrain, their breccia and melt in
 * the strata and in bodies); it is here to find them. Plain Java; a world without the file (a
 * synthetic terrain, a preset without the impact stage) has none.
 */
public final class Craters {

    public static final String FILE = "impacts/craters.json";
    public static final int VERSION = 1;
    public static final Craters NONE = new Craters(List.of());

    /**
     * One crater.
     *
     * @param x          block position of its centre, x in [−C/2, C/2)
     * @param diameterM  rim to rim, m
     * @param morphology {@code simple}, {@code complex} or {@code peak_ring}
     * @param depthM     rim crest to floor when fresh, game metres
     * @param rimM       rim crest above the ground it struck when fresh, game metres
     * @param reliefM    what is left of {@code depthM} now
     * @param fate       {@code fresh}, {@code degraded}, {@code buried} or {@code eroded_to_roots}
     * @param brecciaM   thickness of its breccia lens, game metres
     * @param meltM      thickness of its melt sheet, game metres (0 without one)
     * @param coverM     sediment over its floor, m
     * @param inSurface  cut into the deep-time surface and laid in the strata log (a large crater),
     *                   rather than drawn by the amplifier and given a body
     */
    public record Crater(int id, double x, double z, double diameterM, String morphology, double ageMyr, double depthM,
                         double rimM, double reliefM, String fate, double brecciaM, double meltM, double coverM,
                         boolean inSurface) {

        /** Its bowl still shows in the terrain. */
        public boolean shows() {
            return fate.equals("fresh") || fate.equals("degraded");
        }
    }

    private final List<Crater> list;

    public Craters(List<Crater> list) {
        this.list = List.copyOf(list);
    }

    public List<Crater> list() {
        return list;
    }

    /** The world's craters; {@link #NONE} when the folder has no list. */
    public static Craters read(Path folder) {
        Path path = folder.resolve(FILE);
        if (!Files.isRegularFile(path))
            return NONE;
        JsonObject root;
        try {
            root = JsonParser.parseString(Files.readString(path, StandardCharsets.UTF_8)).getAsJsonObject();
        } catch (IOException | RuntimeException e) {
            throw new WorldDataException(path + ": cannot read the impact craters: " + e);
        }
        return parse(root, path.toString());
    }

    /** Parses the file's JSON; {@code what} names it in errors. */
    public static Craters parse(JsonObject root, String what) {
        try {
            int version = root.get("version").getAsInt();
            if (version != VERSION)
                throw new WorldDataException(what + ": version " + version + ", this build reads " + VERSION);
            List<Crater> out = new ArrayList<>();
            for (JsonElement e : root.getAsJsonArray("craters")) {
                JsonObject c = e.getAsJsonObject();
                out.add(new Crater(c.get("id").getAsInt(), c.get("x").getAsDouble(), c.get("z").getAsDouble(),
                        c.get("diameter_m").getAsDouble(), c.get("morphology").getAsString(), c.get("age_myr").getAsDouble(),
                        c.get("depth_m").getAsDouble(), c.get("rim_m").getAsDouble(), c.get("relief_m").getAsDouble(),
                        c.get("fate").getAsString(), c.get("breccia_m").getAsDouble(), c.get("melt_m").getAsDouble(),
                        c.get("cover_m").getAsDouble(), c.get("in_surface").getAsBoolean()));
            }
            return new Craters(out);
        } catch (WorldDataException e) {
            throw e;
        } catch (RuntimeException e) {
            throw new WorldDataException(what + ": not a list of impact craters: " + e);
        }
    }
}
