// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.Map;
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
    /** Sites, bodies and events, read on first use (P1-15). */
    private volatile Deposits deposits;

    private WorldData(Path folder, Manifest manifest, TileCache cache, Map<String, RasterSampler> layers) {
        this.folder = folder;
        this.manifest = manifest;
        this.cache = cache;
        this.layers = layers;
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

    /** Terrain surface height at block (x, z): bilinear on the L3 {@code elevation} layer. */
    public double height(double x, double z) {
        return layer(ELEVATION).bilinear(x, z);
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
