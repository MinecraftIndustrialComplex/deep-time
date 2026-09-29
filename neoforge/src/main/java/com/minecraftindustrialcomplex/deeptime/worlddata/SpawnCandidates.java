// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * {@code spawn/candidates.json} ({@code FORMAT.md}): the world data's ranked land cells for the
 * spawn. The server's {@code SpawnHandler} takes the first whose generated surface is above sea
 * level; before any chunk exists (the Create World screen's review page) the recorded
 * {@code surface_y} stands in for the generated one.
 */
public final class SpawnCandidates {

    public static final String FILE = "spawn/candidates.json";

    /** One candidate: block coordinates and the recorded surface height. */
    public record Candidate(int x, int z, double surfaceY) {}

    private SpawnCandidates() {}

    /** Every candidate in rank order; empty when the folder has no candidates file. */
    public static List<Candidate> read(Path folder) {
        Path file = folder.resolve(FILE);
        String text;
        try {
            text = Files.readString(file, StandardCharsets.UTF_8);
        } catch (NoSuchFileException e) {
            return List.of();
        } catch (IOException e) {
            throw new WorldDataException(file + ": " + e.getMessage(), e);
        }
        try {
            JsonObject o = JsonParser.parseString(text).getAsJsonObject();
            List<Candidate> out = new ArrayList<>();
            for (JsonElement e : o.getAsJsonArray("candidates")) {
                JsonObject c = e.getAsJsonObject();
                out.add(new Candidate(c.get("x").getAsInt(), c.get("z").getAsInt(),
                        c.has("surface_y") ? c.get("surface_y").getAsDouble() : Double.NaN));
            }
            return List.copyOf(out);
        } catch (RuntimeException e) {
            throw new WorldDataException(file + ": malformed: " + e.getMessage(), e);
        }
    }

    /** The first candidate whose recorded surface is above {@code seaLevel} (the spawn the server will most likely pick). */
    public static Optional<Candidate> firstDry(Path folder, int seaLevel) {
        for (Candidate c : read(folder))
            if (c.surfaceY() > seaLevel)
                return Optional.of(c);
        return Optional.empty();
    }
}
