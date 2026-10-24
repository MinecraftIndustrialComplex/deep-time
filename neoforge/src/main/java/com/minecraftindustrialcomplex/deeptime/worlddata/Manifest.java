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
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * {@code manifest.json} (plan §5.7.1; FORMAT.md). The format version is checked before anything
 * else is parsed, so a newer layout still produces the version message.
 */
public record Manifest(String formatVersion, String source, String createdUtc, int circumference, int seaLevelY,
                       int minY, int maxY, Map<String, GridDesc> grids, List<LayerEntry> layers,
                       List<FileEntry> files, JsonObject notes, double mPerBlockH, double mPerBlockV, WorldLayout layout) {

    public static final String FILE = "manifest.json";

    /** Plan §5.1 defaults, for manifests that do not name the scales (the vertical one is 35 m since
     *  Ben's decision of 2026-10-02, as Rust's {@code deeptime_format::DEFAULT_M_PER_BLOCK_V}). */
    public static final double DEFAULT_M_PER_BLOCK_H = 300.0;
    public static final double DEFAULT_M_PER_BLOCK_V = 35.0; // ledger: SCL-002 (game choice, Ben)

    /** One tile pack of the folder. */
    public record LayerEntry(String name, String path, String grid, long bytes, long crc32) {}

    /** Any other listed file. */
    public record FileEntry(String path, long bytes, long crc32) {}

    /** Reads {@code <folder>/manifest.json}. */
    public static Manifest read(Path folder) {
        Path file = folder.resolve(FILE);
        String text;
        try {
            text = Files.readString(file, StandardCharsets.UTF_8);
        } catch (NoSuchFileException e) {
            throw new WorldDataException(file + ": no world-data manifest (missing or incompletely written)", e);
        } catch (IOException e) {
            throw new WorldDataException(file + ": " + e.getMessage(), e);
        }
        JsonObject o;
        try {
            o = JsonParser.parseString(text).getAsJsonObject();
        } catch (RuntimeException e) {
            throw new WorldDataException(file + ": not JSON: " + e.getMessage(), e);
        }
        JsonElement fv = o.get("format_version");
        if (fv == null || !fv.isJsonPrimitive())
            throw new WorldDataException(file + ": no format_version");
        FormatVersion.require(fv.getAsString());
        Manifest m;
        try {
            JsonObject scale = o.getAsJsonObject("scale");
            Map<String, GridDesc> grids = new LinkedHashMap<>();
            for (Map.Entry<String, JsonElement> g : o.getAsJsonObject("grids").entrySet())
                grids.put(g.getKey(), GridDesc.fromJson(g.getValue().getAsJsonObject()));
            List<LayerEntry> layers = new ArrayList<>();
            for (JsonElement e : o.getAsJsonArray("layers")) {
                JsonObject l = e.getAsJsonObject();
                layers.add(new LayerEntry(l.get("name").getAsString(), l.get("path").getAsString(),
                        l.get("grid").getAsString(), l.get("bytes").getAsLong(), l.get("crc32").getAsLong()));
            }
            List<FileEntry> files = new ArrayList<>();
            if (o.has("files"))
                for (JsonElement e : o.getAsJsonArray("files")) {
                    JsonObject f = e.getAsJsonObject();
                    files.add(new FileEntry(f.get("path").getAsString(), f.get("bytes").getAsLong(),
                            f.get("crc32").getAsLong()));
                }
            m = new Manifest(
                    fv.getAsString(),
                    o.get("source").getAsString(),
                    o.get("created_utc").getAsString(),
                    scale.get("circumference_blocks").getAsInt(),
                    scale.get("sea_level_y").getAsInt(),
                    scale.get("min_y").getAsInt(),
                    scale.get("max_y").getAsInt(),
                    Map.copyOf(grids),
                    List.copyOf(layers),
                    List.copyOf(files),
                    o.has("notes") ? o.getAsJsonObject("notes") : new JsonObject(),
                    scale.has("m_per_block_h") ? scale.get("m_per_block_h").getAsDouble() : DEFAULT_M_PER_BLOCK_H,
                    scale.has("m_per_block_v") ? scale.get("m_per_block_v").getAsDouble() : DEFAULT_M_PER_BLOCK_V,
                    o.has("projection") && o.get("projection").isJsonObject() ? WorldLayout.fromJson(o.getAsJsonObject("projection")) : null);
        } catch (RuntimeException e) {
            throw new WorldDataException(file + ": malformed manifest: " + e.getMessage(), e);
        }
        // A cube world is 0.7 and only a cube world is (FORMAT.md "Polar and cube worlds side by side"; Rust
        // Manifest::read).
        WorldProjection.Kind kind = m.layout() == null ? WorldProjection.Kind.LAMBERT : m.layout().kind();
        String want = FormatVersion.forKind(kind);
        if (!want.equals(m.formatVersion()))
            throw new WorldDataException(file + ": format_version " + m.formatVersion() + " on a " + kind.id()
                    + " world, which is " + want);
        // Its map table and rotation must be ones this jar can draw, refused before any layer is read (Rust
        // WorldProjection::try_from_spec in Manifest::read).
        if (kind == WorldProjection.Kind.CUBE)
            m.projection();
        return m;
    }

    /**
     * The planet's name for players (the F3 line, maps): the preset of a run ({@code run:<preset>}),
     * the kind of a synthetic world ({@code synth:<kind>}), else the source as written.
     */
    public String planetName() {
        int colon = source.indexOf(':');
        return colon >= 0 && colon + 1 < source.length() ? source.substring(colon + 1) : source;
    }

    /**
     * The world's projection ({@code projection}, FORMAT.md "The world's layout"): Petroff–Guyou,
     * Lambert or the cube ({@link WorldProjection.Cube}); a manifest without the block is a Lambert
     * world of {@link #circumference}. A new object per call: keep it ({@link WorldData#projection}
     * does).
     *
     * @throws WorldDataException on a cube world whose map table this jar does not ship (by name and
     *                            sha256) or whose rotation is not one
     */
    public WorldProjection projection() {
        return WorldProjection.of(layout, circumference);
    }

    /** Whether this is a cube world (format 0.7, {@code projection.kind} {@code cube}). */
    public boolean cube() {
        return layout != null && layout.kind() == WorldProjection.Kind.CUBE;
    }

    /** The entry of layer {@code name}, or null. */
    public LayerEntry layer(String name) {
        for (LayerEntry l : layers)
            if (l.name().equals(name)) return l;
        return null;
    }
}
