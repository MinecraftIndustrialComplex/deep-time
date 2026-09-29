// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.SiteRecords;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * Ore-province hints for the review page (plan §7.12 page 3: "ore-province hints (not exact
 * deposits)"): the sites are binned into a coarse latitude–longitude grid and blurred over the
 * neighbouring cells, and each cell reports only its dominant deposit type. The map shows where a
 * kind of ore is common, never where a deposit is.
 */
public final class ProvinceHints {

    /** Grid columns (longitude, wrapping) and rows (latitude): 30° cells. */
    public static final int COLS = 12, ROWS = 6;

    /** A cell's score must reach this (one site in it, or two next to it) to show a hint. */
    static final double MIN_SCORE = 1.0;

    /** Weight a site gives each of the eight cells around its own. */
    static final double NEIGHBOUR_WEIGHT = 0.5;

    /**
     * One hinted cell.
     *
     * @param col   0…{@link #COLS}−1 from the seam eastwards
     * @param row   0…{@link #ROWS}−1 from the north
     * @param type  the dominant deposit type
     * @param score its blurred count
     */
    public record Cell(int col, int row, String type, double score) {

        /** The cell's bounds in degrees: west, east longitude; north, south latitude. */
        public double lonWest() {
            return col * 360.0 / COLS;
        }

        public double lonEast() {
            return (col + 1) * 360.0 / COLS;
        }

        public double latNorth() {
            return 90.0 - row * 180.0 / ROWS;
        }

        public double latSouth() {
            return 90.0 - (row + 1) * 180.0 / ROWS;
        }
    }

    private ProvinceHints() {}

    public static List<Cell> compute(List<SiteRecords.Site> sites) {
        return compute(sites, COLS, ROWS);
    }

    public static List<Cell> compute(List<SiteRecords.Site> sites, int cols, int rows) {
        // type -> score per cell
        Map<String, double[]> scores = new TreeMap<>();
        for (SiteRecords.Site s : sites) {
            if (!Double.isFinite(s.lat()) || !Double.isFinite(s.lon()))
                continue;
            double[] grid = scores.computeIfAbsent(s.depositType(), k -> new double[cols * rows]);
            int c = Math.floorMod((int) Math.floor(GlobeProjection.wrapLon(s.lon()) / 360.0 * cols), cols);
            int r = Math.max(0, Math.min(rows - 1, (int) Math.floor((90.0 - s.lat()) / 180.0 * rows)));
            for (int dr = -1; dr <= 1; dr++) {
                int rr = r + dr;
                if (rr < 0 || rr >= rows)
                    continue;
                for (int dc = -1; dc <= 1; dc++) {
                    int cc = Math.floorMod(c + dc, cols);
                    grid[rr * cols + cc] += dr == 0 && dc == 0 ? 1.0 : NEIGHBOUR_WEIGHT;
                }
            }
        }
        List<Cell> out = new ArrayList<>();
        for (int r = 0; r < rows; r++) {
            for (int c = 0; c < cols; c++) {
                String best = null;
                double bestScore = 0;
                for (Map.Entry<String, double[]> e : scores.entrySet()) {
                    double v = e.getValue()[r * cols + c];
                    if (v > bestScore) {
                        bestScore = v;
                        best = e.getKey();
                    }
                }
                if (best != null && bestScore >= MIN_SCORE)
                    out.add(new Cell(c, r, best, bestScore));
            }
        }
        return out;
    }

    /** The viewer's colour of a deposit type ({@code viewer.js} {@code TYPE_COLOURS}), as {@code 0xRRGGBB}. */
    public static int colour(String type) {
        String t = type.contains(":") ? type : "deeptime:" + type;
        return switch (t) {
            case "deeptime:porphyry_cu_mo_au" -> 0xd7301f;
            case "deeptime:supergene_cu_blanket" -> 0xfdae61;
            case "deeptime:placer_au" -> 0xffd700;
            case "deeptime:bif_fe" -> 0xa6611a;
            case "deeptime:coal_seam" -> 0x9e9e9e;
            case "deeptime:mvt_zn_pb" -> 0x1a9850;
            case "deeptime:sedex_zn_pb" -> 0x66c2a5;
            case "deeptime:petroleum_trap" -> 0x8073ac;
            default -> 0xffb000;
        };
    }
}
