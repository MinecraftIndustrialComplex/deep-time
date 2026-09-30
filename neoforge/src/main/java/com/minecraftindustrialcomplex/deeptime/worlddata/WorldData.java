// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.zip.CRC32;

/**
 * One world-data folder (plan §5.7): the manifest and a {@link RasterSampler} per layer. Opening
 * reads the manifest (refusing another format version with a message naming both), maps every
 * listed tile pack and checks their headers; tiles decode lazily. Immutable after open and
 * thread-safe. No JNI: this is plain Java over memory-mapped files.
 *
 * <p>P0-10's {@code WorldDataTerrain} implements {@code TerrainSource} over {@link #height} and
 * {@link #raster}.
 */
public final class WorldData {

    public static final String ELEVATION = "elevation";
    public static final String WATER_LEVEL = "water_level";
    public static final String HORIZONS = "strata/horizons";

    private final Path folder;
    private final Manifest manifest;
    private final TileCache cache;
    private final Map<String, RasterSampler> layers;
    /** The sub-cell detail term's noise and amplitude layer (format 0.1), or null. */
    private final DetailNoise detail;
    private final RasterSampler amplitude;
    /** Sites, bodies and events, read on first use (P1-15). */
    private volatile Deposits deposits;
    /** {@code strata/classes.json}, read on first use. */
    private volatile List<String> classes;
    /** The ice cover, resolved on first use. */
    private volatile Optional<IceCover> iceCover;

    private WorldData(Path folder, Manifest manifest, TileCache cache, Map<String, RasterSampler> layers) {
        this.folder = folder;
        this.manifest = manifest;
        this.cache = cache;
        this.layers = layers;
        DetailNoise d = DetailNoise.of(manifest);
        if (d != null) {
            String name = DetailNoise.layerOf(manifest);
            this.amplitude = layers.get(name);
            if (amplitude == null)
                throw new WorldDataException(folder + ": notes.bake.detail names the layer " + name
                        + ", which the manifest lacks");
        } else {
            this.amplitude = null;
        }
        this.detail = d;
    }

    /** Opens {@code folder} with its own default-sized tile cache. */
    public static WorldData open(Path folder) {
        return open(folder, new TileCache());
    }

    /** Opens {@code folder}, decoding LZ4 tiles into {@code cache}. */
    public static WorldData open(Path folder, TileCache cache) {
        Manifest m = Manifest.read(folder);
        Map<String, RasterSampler> layers = new LinkedHashMap<>();
        for (Manifest.LayerEntry e : m.layers()) {
            Path p = folder.resolve(e.path());
            TilePack pack = TilePack.open(p, cache);
            if (pack.fileSize() != e.bytes())
                throw new WorldDataException(p + ": " + pack.fileSize() + " bytes, manifest says " + e.bytes());
            if (!pack.desc().name().equals(e.name()))
                throw new WorldDataException(p + ": holds layer " + pack.desc().name() + ", manifest says " + e.name());
            if (pack.desc().grid().circumference() != m.circumference())
                throw new WorldDataException(p + ": grid circumference " + pack.desc().grid().circumference()
                        + " != manifest circumference " + m.circumference());
            layers.put(e.name(), new RasterSampler(pack));
        }
        return new WorldData(folder, m, cache, Map.copyOf(layers));
    }

    public Path folder() {
        return folder;
    }

    public Manifest manifest() {
        return manifest;
    }

    public TileCache cache() {
        return cache;
    }

    /** C, the east-west period in blocks. */
    public int circumference() {
        return manifest.circumference();
    }

    /**
     * The world's sites, bodies and events ({@code sites/}, {@code bodies/}, {@code events/}), read
     * and checked on first use; empty for a folder without them (the synthetic terrains).
     */
    public Deposits deposits() {
        Deposits d = deposits;
        if (d == null) {
            synchronized (this) {
                if (deposits == null)
                    deposits = Deposits.read(folder, circumference());
                d = deposits;
            }
        }
        return d;
    }

    /** {@code strata/classes.json}: class index → rock class id (read once). */
    public List<String> classes() {
        List<String> c = classes;
        if (c == null) {
            synchronized (this) {
                if (classes == null) {
                    Path p = folder.resolve("strata/classes.json");
                    try {
                        var arr = JsonParser.parseString(Files.readString(p)).getAsJsonObject().getAsJsonArray("classes");
                        List<String> out = new ArrayList<>();
                        arr.forEach(e -> out.add(e.getAsString()));
                        classes = List.copyOf(out);
                    } catch (IOException | RuntimeException e) {
                        throw new WorldDataException("cannot read " + p + ": " + e.getMessage(), e);
                    }
                }
                c = classes;
            }
        }
        return c;
    }

    /** The world's ice cover ({@link IceCover#of}), resolved once; empty for a world without one. */
    public Optional<IceCover> iceCover() {
        Optional<IceCover> c = iceCover;
        if (c == null) {
            synchronized (this) {
                if (iceCover == null)
                    iceCover = IceCover.of(this);
                c = iceCover;
            }
        }
        return c;
    }

    public boolean has(String layer) {
        return layers.containsKey(layer);
    }

    /** The sampler of {@code layer}. */
    public RasterSampler layer(String layer) {
        RasterSampler s = layers.get(layer);
        if (s == null)
            throw new WorldDataException(folder + ": no layer '" + layer + "' (has " + layers.keySet() + ")");
        return s;
    }

    /**
     * The terrain height H at block (x, z) (FORMAT.md, "The terrain height"): the L3
     * {@code elevation} layer bilinear, plus, where the world has one (format 0.1 bakes), the
     * detail term A·N — the {@code detail/amplitude} layer bilinear times {@link DetailNoise}.
     * {@code x} is expected folded into [−C/2, C/2) ({@code TerrainSource.wrapX}); the noise is
     * evaluated at the x given, as Rust's column evaluator does.
     */
    public double height(double x, double z) {
        double e = layer(ELEVATION).bilinear(x, z);
        return detail == null ? e : e + amplitude.bilinear(x, z) * detail.at(x, z);
    }

    /** The {@code elevation} layer alone at block (x, z) (E of H = E + A·N). */
    public double elevation(double x, double z) {
        return layer(ELEVATION).bilinear(x, z);
    }

    /** The detail term A·N at block (x, z); 0 in a world without one. */
    public double detail(double x, double z) {
        return detail == null ? 0.0 : amplitude.bilinear(x, z) * detail.at(x, z);
    }

    /** The world's detail noise, or null. */
    public DetailNoise detailNoise() {
        return detail;
    }

    /**
     * A 2D biome input at block (x, z) in vanilla parameter units: bilinear on the L2
     * {@code climate/<name>} layer.
     */
    public double raster(String layer, double x, double z) {
        return layer(layer).bilinear(x, z);
    }

    /** Water surface above the terrain at block (x, z), NaN where there is none (nearest cell). */
    public double waterLevel(double x, double z) {
        return has(WATER_LEVEL) ? layer(WATER_LEVEL).nearest(x, z) : Double.NaN;
    }

    /** Checks every listed file's size and CRC32 (reads everything: a tool check, not a hot path). */
    public void verify() {
        for (Manifest.LayerEntry e : manifest.layers()) {
            long crc = layers.get(e.name()).pack().fileCrc32();
            if (crc != e.crc32())
                throw new WorldDataException(String.format("%s: file CRC32 %08x, manifest says %08x",
                        folder.resolve(e.path()), crc, e.crc32()));
        }
        for (Manifest.FileEntry f : manifest.files()) {
            Path p = folder.resolve(f.path());
            byte[] b;
            try {
                b = Files.readAllBytes(p);
            } catch (IOException ex) {
                throw new WorldDataException(p + ": " + ex.getMessage(), ex);
            }
            CRC32 crc = new CRC32();
            crc.update(b);
            if (b.length != f.bytes() || crc.getValue() != f.crc32())
                throw new WorldDataException(p + ": size or CRC32 differs from the manifest");
        }
    }
}
