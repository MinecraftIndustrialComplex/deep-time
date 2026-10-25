// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;
import java.util.zip.CRC32;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The ice cover (Ben, 2026-09-30, "Force snow/ice cover"): the layer the bake writes from the climate
 * stage's land ice, the table recorded beside it, the rule it is read by, and the block mix of a cover.
 */
class IceCoverTest {

    /** The cover rule's vectors: deeptime-format's {@code the_nearest_cell_decides_and_the_covered_cells_blend} holds the same ones. */
    @Test
    void theNearestCellDecidesAndTheCoveredCellsBlend() {
        assertEquals(8.0, RasterSampler.coverOf(8, new double[] {8, 0}, new double[] {0.5, 0.5}));
        assertEquals(7.0, RasterSampler.coverOf(4, new double[] {4, 8, 0}, new double[] {0.25, 0.75, 0}));
        assertEquals(0.0, RasterSampler.coverOf(0, new double[] {12, 0}, new double[] {0.75, 0.25}));
        assertEquals(12.0, RasterSampler.coverOf(12, new double[] {12, 0}, new double[] {0.25, 0.75}));
        assertEquals(0.0, RasterSampler.coverOf(Double.NaN, new double[] {6}, new double[] {1}));
        assertEquals(6.0, RasterSampler.coverOf(6, new double[] {Double.NaN, 6}, new double[] {0.75, 0.25}));
        assertEquals(5.0, RasterSampler.coverOf(5, new double[] {0}, new double[] {1}));
        assertEquals(5.0, RasterSampler.coverOf(5, new double[] {}, new double[] {}));
        assertEquals(0.0, RasterSampler.coverOf(0, new double[] {}, new double[] {}));
    }

    /**
     * A cover's edge lies half-way between a covered cell and its bare neighbour, on every side: a bilinear read rounded
     * to blocks put a 12-block cell's cover most of a cell past it (5 blocks five-eighths of the way to the bare cell's
     * centre, where the nearest cell is the bare one). Checked
     * on a layer made to hold the cases (a 4 × 4 sheet of 12, a lone cell, a ramp and a body across the x seam), so the
     * check does not depend on where a simulated planet happens to put its ice.
     */
    @Test
    void aCoversEdgeLiesHalfWayToItsBareNeighbour(@TempDir Path tmp) throws IOException {
        int w = 24, h = 12;
        int[] cells = new int[w * h];
        for (int j = 3; j <= 6; j++)
            for (int i = 4; i <= 7; i++)
                cells[j * w + i] = 12;
        cells[4 * w + 14] = 6;
        cells[8 * w + 18] = 3;
        cells[8 * w + 19] = 7;
        cells[8 * w + 20] = 11;
        cells[10 * w] = 9;
        cells[10 * w + w - 1] = 9;
        RasterSampler layer = rawU8Layer(tmp.resolve("ice_cover.dtr"), w, h, 8, cells);
        IceCover cover = IceCover.of(layer, spec());
        // 16 sides of the sheet's border cells, 4 of the lone cell, 8 of the ramp, 6 of the seam body.
        assertEquals(34, checkEdges(layer, cover));
        var g = layer.desc().grid();
        // Between the ramp's 3 and 7, half-way: covered (the nearest cell is the 7), at the blend of the two.
        assertEquals(5, cover.blocks((int) (g.originBlockX() + 18L * 8 + 4), (int) (g.originBlockZ() + 8L * 8)));
        // The seam body is one body: its two cells' blend at the seam, which the x wrap joins.
        assertEquals(9, cover.blocks((int) (g.originBlockX() + 23L * 8 + 4), (int) (g.originBlockZ() + 10L * 8)));
    }

    /**
     * The baked planet's cover follows the same rule at each of its edges, a cover has an edge, and the layer is bare
     * where the record counts no covered cell. (The test planet may carry no ice at all: since the crust fix of master
     * 25ae6723 its record counts 0 cells. The edge rule is then held by the constructed layer above, never skipped.)
     */
    @Test
    void thePlanetsCoverFollowsTheRuleAtEveryEdge() {
        WorldData wd = WorldData.open(Fixtures.world("planet4k"));
        IceCover cover = wd.iceCover().orElseThrow();
        RasterSampler layer = wd.layer(IceCover.LAYER);
        var g = layer.desc().grid();
        int covered = 0;
        for (int j = 0; j < g.h(); j++)
            for (int i = 0; i < g.w(); i++)
                if (layer.get(i, j) > 0)
                    covered++;
        int cells = wd.manifest().notes().getAsJsonObject("bake").getAsJsonObject("climate").getAsJsonObject("ice_cover")
                .get("cells").getAsInt();
        // (Not the converse: the record counts L2 cells, the layer is on the world grid, and a lone L2 cell need not be
        // the nearest of any world cell.)
        assertTrue(cells > 0 || covered == 0, "the record counts no covered cell, the layer has " + covered);
        int edges = checkEdges(layer, cover);
        assertTrue(covered == 0 || edges > 0, "a cover of " + covered + " cells has no edge");
    }

    /**
     * The block at the centre of {@code g}'s cell (i, j), {x, z}: on a Lambert-frame grid the cell's own block; on a cube
     * grid (the baked planet, cube plan §7.4) the block east and south of the cell's centre on the net (r is even), the
     * one {@link GridDesc#cubeSpotOfBlock} takes to the cell.
     */
    private static int[] centreBlock(GridDesc g, int i, int j) {
        if (!g.isCube())
            return new int[]{(int) (g.originBlockX() + (long) i * g.r()), (int) (g.originBlockZ() + (long) j * g.r())};
        double[] p = g.cubePoint(i, j);
        return new int[]{(int) Math.floor(p[0]), (int) Math.floor(p[1])};
    }

    /** Whether cell (i, j) + (di, dj) is a neighbour {@link #checkEdges} reads: on a cube grid only on the same face. */
    private static boolean neighbourRead(GridDesc g, int i, int j, int di, int dj) {
        int nj = j + dj;
        if (nj < 0 || nj >= g.h())
            return false;
        if (!g.isCube())
            return true;
        int ni = i + di;
        return ni >= 0 && ni < g.w() && nj / g.w() == j / g.w();
    }

    /**
     * Checks every edge of {@code layer}'s cover (a covered cell beside a bare one, east, west, north or south; x wraps,
     * the first and last rows have no neighbour beyond them; on a cube grid the neighbours on the cell's own face) and
     * returns how many there are: on the cell's row or column
     * a block short of the half-way line is covered at the cell's own depth, a block past it is bare, and over the cell
     * and its neighbours the nearest cell decides, as it decides the block's class: covered exactly where it is covered.
     */
    private static int checkEdges(RasterSampler layer, IceCover cover) {
        var g = layer.desc().grid();
        int r = g.r();
        assertTrue(r >= 4 && r % 2 == 0, "cells of " + r + " blocks");
        int[][] sides = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}};
        int edges = 0;
        for (int j = 0; j < g.h(); j++) {
            for (int i = 0; i < g.w(); i++) {
                int v = (int) layer.get(i, j);
                if (v == 0)
                    continue;
                int[] at0 = centreBlock(g, i, j);
                int x = at0[0], z = at0[1];
                for (int[] s : sides) {
                    if (!neighbourRead(g, i, j, s[0], s[1]) || layer.get(i + s[0], j + s[1]) != 0)
                        continue;
                    edges++;
                    String at = "cell (" + i + ", " + j + ") toward (" + s[0] + ", " + s[1] + ")";
                    int in = r / 2 - 1, out = r / 2 + 1;
                    assertEquals(v, cover.blocks(x + s[0] * in, z + s[1] * in), "inside the half-way line of " + at);
                    assertEquals(0, cover.blocks(x + s[0] * out, z + s[1] * out), "past the half-way line of " + at);
                    for (int d = -r; d <= r; d++)
                        for (int e = -r; e <= r; e++)
                            assertEquals(layer.nearest(x + d, z + e) > 0, cover.blocks(x + d, z + e) > 0,
                                    "the nearest cell decides at (" + (x + d) + ", " + (z + e) + ") by " + at);
                }
            }
        }
        return edges;
    }

    /**
     * Writes a one-tile raw u8 pack of {@code cells} (row-major, {@code w} × {@code h}, {@code r}-block cells, x wrapping,
     * centred on block 0) as FORMAT.md lays a tile pack out, and opens it. {@link PlayaTest} builds its layer with it too.
     */
    static RasterSampler rawU8Layer(Path path, int w, int h, int r, int[] cells) throws IOException {
        String desc = "{\"name\": \"" + IceCover.LAYER + "\", \"dtype\": \"u8\", \"scale\": 1.0, \"offset\": 0.0, \"units\": \"blocks\","
                + " \"grid\": {\"name\": \"L2\", \"w\": " + w + ", \"h\": " + h + ", \"r\": " + r + ", \"wraps_x\": true,"
                + " \"origin_block_x\": " + (-w * r / 2) + ", \"origin_block_z\": " + (-h * r / 2) + "},"
                + " \"tile\": " + Math.max(w, h) + ", \"codec\": \"raw\"}";
        byte[] json = desc.getBytes(StandardCharsets.UTF_8);
        int headerLen = 16 + json.length, indexAt = (headerLen + 7) & ~7, tileAt = indexAt + 16;
        byte[] tile = new byte[w * h];
        for (int k = 0; k < tile.length; k++)
            tile[k] = (byte) cells[k];
        ByteBuffer b = ByteBuffer.allocate(tileAt + ((tile.length + 7) & ~7)).order(ByteOrder.LITTLE_ENDIAN);
        b.put(new byte[] {'D', 'T', 'R', '1'}).putInt(headerLen).putShort((short) FormatVersion.MINOR).putShort((short) 0);
        b.put(16, json);
        CRC32 crc = new CRC32();
        crc.update(b.array(), 0, 12);
        crc.update(json);
        b.putInt(12, (int) crc.getValue());
        CRC32 tileCrc = new CRC32();
        tileCrc.update(tile);
        b.putLong(indexAt, tileAt).putInt(indexAt + 8, tile.length).putInt(indexAt + 12, (int) tileCrc.getValue());
        b.put(tileAt, tile);
        Files.write(path, b.array());
        return new RasterSampler(TilePack.open(path, new TileCache()));
    }

    private static final String RECORD = """
            {"bake": {"climate": {"ice_cover": {"step_m": 160.0, "max_blocks": 12, "snow_blocks": 3,
              "blue_ice_from_blocks": 9, "blue_ice_blocks": 2, "snow": "minecraft:snow_block",
              "ice": "minecraft:packed_ice", "deep_ice": "minecraft:blue_ice", "cells": 7}}}}""";

    private static IceCover.Spec spec() {
        return IceCover.Spec.read(JsonParser.parseString(RECORD).getAsJsonObject());
    }

    private static String mix(IceCover.Spec s, int total) {
        StringBuilder out = new StringBuilder();
        for (int i = 0; i < total; i++)
            out.append("SID".charAt(s.kind(i, total)));
        return out.toString();
    }

    @Test
    void theRecordIsReadAndAMissingOneIsNull() {
        IceCover.Spec s = spec();
        assertEquals(12, s.maxBlocks());
        assertEquals("minecraft:blue_ice", s.deepIce());
        assertEquals(160.0, s.stepM());
        assertNull(IceCover.Spec.read(new JsonObject()));
        assertNull(IceCover.Spec.read(JsonParser.parseString("{\"bake\": {\"climate\": {}}}").getAsJsonObject()));
        assertNull(IceCover.Spec.read(JsonParser.parseString("{\"bake\": {\"climate\": {\"ice_cover\": null}}}").getAsJsonObject()));
    }

    /** Snow on top, packed ice between, blue ice at the base of a thick cover. */
    @Test
    void theBlockMixOfACover() {
        IceCover.Spec s = spec();
        assertEquals("S", mix(s, 1));
        assertEquals("SSS", mix(s, 3));
        assertEquals("SSSI", mix(s, 4));
        assertEquals("SSSIIIII", mix(s, 8));
        assertEquals("SSSIIIIDD", mix(s, 9), "a cover of 9 blocks ends in blue ice");
        assertEquals("SSSIIIIIIIDD", mix(s, 12));
        assertEquals("minecraft:snow_block", s.blockId(IceCover.Spec.SNOW));
        assertEquals("minecraft:packed_ice", s.blockId(IceCover.Spec.ICE));
        assertEquals("minecraft:blue_ice", s.blockId(IceCover.Spec.DEEP_ICE));
        // Snow takes precedence where the blue ice would overlap it.
        IceCover.Spec thin = new IceCover.Spec(50, 5, 4, 2, 3, "a:s", "a:i", "a:d");
        assertEquals("SSSS", mix(thin, 4));
        assertEquals("SSSSD", mix(thin, 5));
    }

    /** The baked 4k planet has the layer, the record and a cover that reads each cell's value at its own block. */
    @Test
    void theBakedPlanetHasItsCover() {
        WorldData wd = WorldData.open(Fixtures.world("planet4k"));
        Optional<IceCover> cover = wd.iceCover();
        assertTrue(cover.isPresent(), "a baked planet with a climate stage has an ice cover");
        assertEquals(12, cover.get().spec().maxBlocks());
        RasterSampler layer = wd.layer(IceCover.LAYER);
        int thickest = 0;
        var g = layer.desc().grid();
        for (int j = 0; j < g.h(); j++) {
            for (int i = 0; i < g.w(); i++) {
                int v = (int) layer.get(i, j);
                thickest = Math.max(thickest, v);
                if (v > 0) {
                    // At a cell's own block the cover rule reads the cell.
                    int[] at = centreBlock(g, i, j);
                    assertEquals(v, cover.get().blocks(at[0], at[1]), "cell (" + i + ", " + j + ")");
                }
            }
        }
        assertTrue(thickest <= cover.get().spec().maxBlocks(), "no thicker than the ceiling: " + thickest);
        // (The 150 Myr test planet may have no ground above its equilibrium line: nothing to read then.)
    }

    @Test
    void aSyntheticWorldHasNone() {
        assertTrue(WorldData.open(Fixtures.world("wrap1024")).iceCover().isEmpty());
    }
}
