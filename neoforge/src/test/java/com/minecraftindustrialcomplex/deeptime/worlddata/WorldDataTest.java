// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** The reader's refusals and bookkeeping. */
class WorldDataTest {

    /** The workspace Cargo.toml is the one source of the format version (cold-read review 2, gap 14). */
    @Test
    void formatVersionMatchesCargoWorkspace() throws IOException {
        String toml = Files.readString(Fixtures.repoRoot().resolve("Cargo.toml"), StandardCharsets.UTF_8);
        String section = toml.substring(toml.indexOf("[workspace.metadata.deeptime]"));
        Matcher m = Pattern.compile("(?m)^format_version\\s*=\\s*\"([^\"]+)\"").matcher(section);
        assertTrue(m.find(), "format_version in [workspace.metadata.deeptime]");
        assertEquals(m.group(1), FormatVersion.CURRENT);
        assertEquals("0." + FormatVersion.MINOR, FormatVersion.CURRENT);
    }

    private static Path copyWorld(Path to) throws IOException {
        Path from = Fixtures.world("wrap1024");
        try (Stream<Path> files = Files.walk(from)) {
            for (Path p : (Iterable<Path>) files::iterator) {
                Path q = to.resolve(from.relativize(p).toString());
                if (Files.isDirectory(p)) Files.createDirectories(q);
                else Files.copy(p, q);
            }
        }
        return to;
    }

    @Test
    void refusesAnotherFormatVersionNamingBoth(@TempDir Path tmp) throws IOException {
        Path w = copyWorld(tmp.resolve("w"));
        Path manifest = w.resolve(Manifest.FILE);
        String text = Files.readString(manifest)
                .replace("\"format_version\": \"" + FormatVersion.CURRENT + "\"", "\"format_version\": \"0.9\"");
        Files.writeString(manifest, text);
        WorldDataException e = assertThrows(WorldDataException.class, () -> WorldData.open(w));
        assertEquals("world data 0.9, this Deep Time reads 0.6 (Lambert-frame worlds) and 0.7 (cube worlds)"
                + "; before 1.0 worlds are not migrated", e.getMessage());
    }

    /** A polar world that says 0.7 is refused: only a cube world is 0.7 (Rust {@code Manifest::read}). */
    @Test
    void refusesAPolarWorldThatSaysTheCubesVersion(@TempDir Path tmp) throws IOException {
        Path w = copyWorld(tmp.resolve("w"));
        Path manifest = w.resolve(Manifest.FILE);
        String text = Files.readString(manifest)
                .replace("\"format_version\": \"" + FormatVersion.CURRENT + "\"", "\"format_version\": \"" + FormatVersion.CUBE + "\"");
        Files.writeString(manifest, text);
        WorldDataException e = assertThrows(WorldDataException.class, () -> WorldData.open(w));
        assertTrue(e.getMessage().contains("format_version 0.7 on a lambert world, which is 0.6"), e.getMessage());
    }

    @Test
    void refusesAMissingManifest(@TempDir Path tmp) throws IOException {
        Path w = copyWorld(tmp.resolve("w"));
        Files.delete(w.resolve(Manifest.FILE));
        WorldDataException e = assertThrows(WorldDataException.class, () -> WorldData.open(w));
        assertTrue(e.getMessage().contains("no world-data manifest"), e.getMessage());
    }

    @Test
    void detectsACorruptHeaderAndTile(@TempDir Path tmp) throws IOException {
        Path w = copyWorld(tmp.resolve("w"));
        Path elev = w.resolve("rasters/elevation.dtr");
        byte[] b = Files.readAllBytes(elev);
        byte[] h = b.clone();
        h[20] ^= 0x01; // inside the JSON descriptor
        Files.write(elev, h);
        assertThrows(WorldDataException.class, () -> WorldData.open(w));

        // Flip a byte in the middle of the last tile's stored bytes: the header still checks out,
        // the tile's CRC32 does not.
        TilePack good = TilePack.open(Fixtures.world("wrap1024").resolve("rasters/elevation.dtr"), new TileCache());
        int last = good.tileCount() - 1;
        int storedLength = good.storedBytes(last).capacity();
        byte[] c = b.clone();
        c[(int) offsetOf(good, last) + storedLength / 2] ^= (byte) 0xFF;
        Files.write(elev, c);
        TilePack bad = TilePack.open(elev, new TileCache());
        WorldDataException e = assertThrows(WorldDataException.class, () -> bad.tile(bad.tileCount() - 1));
        assertTrue(e.getMessage().contains("CRC32"), e.getMessage());
    }

    /** The tile index offset of tile t, read from the file the way FORMAT.md lays it out. */
    private static long offsetOf(TilePack p, int t) throws IOException {
        byte[] b = Files.readAllBytes(p.path());
        java.nio.ByteBuffer bb = java.nio.ByteBuffer.wrap(b).order(java.nio.ByteOrder.LITTLE_ENDIAN);
        long headerLen = bb.getInt(4) & 0xFFFFFFFFL;
        long indexAt = (headerLen + 7) & ~7L;
        return bb.getLong((int) (indexAt + 16L * t));
    }

    @Test
    void cacheStaysWithinItsBudget() {
        // One segment with room for two 128 KiB elevation tiles.
        long budget = 300L << 10;
        TileCache cache = new TileCache(budget, 1);
        WorldData wd = WorldData.open(Fixtures.world("wrap1024"), cache);
        RasterSampler e = wd.layer(WorldData.ELEVATION);
        for (int round = 0; round < 3; round++)
            for (int t = 0; t < e.pack().tileCount(); t++)
                e.pack().tile(t);
        for (String l : new String[]{"climate/temperature", "climate/erosion", "water_level"})
            wd.layer(l).pack().tile(0);
        assertTrue(cache.size() >= 1, "cache holds nothing");
        assertTrue(cache.weightBytes() <= budget, "cache holds " + cache.weightBytes() + " bytes");
        // Evicted tiles decode again on demand, with the same values.
        assertEquals(e.raw(3, 3), e.raw(3 + e.desc().grid().w(), 3));
        cache.invalidateAll();
        assertEquals(0, cache.size());
    }

    @Test
    void wrapsInXAndClampsInZ() {
        WorldData wd = WorldData.open(Fixtures.world("wrap1024"));
        RasterSampler e = wd.layer(WorldData.ELEVATION);
        GridDesc g = e.desc().grid();
        assertEquals(e.raw(0, 5), e.raw(g.w(), 5));
        assertEquals(e.raw(g.w() - 1, 5), e.raw(-1, 5));
        assertEquals(e.raw(7, 0), e.raw(7, -40));
        assertEquals(e.raw(7, g.h() - 1), e.raw(7, g.h() + 40));
        // A block and the same block one circumference east.
        assertEquals(wd.height(10.25, -3.5), wd.height(10.25 + wd.circumference(), -3.5));
    }
}
