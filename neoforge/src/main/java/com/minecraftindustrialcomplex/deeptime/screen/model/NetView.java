// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/**
 * The review page's final world on a cube world (cube plan G13: "the final-world view draws the
 * net with the border"): the window of the net, x ∈ [−3s/2, 5s/2) and z ∈ [−3s/2, 3s/2], drawn
 * as it is played, 4 : 3, north (−z) up. The six faces carry the world; the void around N and S is
 * left transparent; the lines are the twelve turning net edges where the temporary border stops a
 * player ({@link LineKind#BORDER}) and the five face sides the net joins ({@link LineKind#JOINED}).
 * Toroidal World's slide joins the window's two ends and is not drawn.
 *
 * <p>Positions are fractions of the window: (0, 0) its north-west corner, (1, 1) its south-east.
 */
public final class NetView {

    /** The review's picture of the net, pixels (4 : 3). */
    public static final int W = 512, H = 384;

    /** A line's kind. */
    public enum LineKind {
        BORDER, JOINED
    }

    /** An axis-aligned line of the net, as fractions of the window. */
    public record Line(double fx0, double fy0, double fx1, double fy1, LineKind kind) {}

    /** A colour for the net point (x, z) of a pixel on a face, ABGR. */
    @FunctionalInterface
    public interface Paint {
        int abgr(double x, double z);
    }

    private final WorldProjection.Cube wp;
    private final CubeLayout cube;
    private final double x0, z0, spanX, spanZ;
    private final List<Line> lines;

    public NetView(WorldProjection.Cube wp) {
        this.wp = wp;
        this.cube = wp.cube();
        double[] win = cube.window();
        this.x0 = win[0];
        this.z0 = win[2];
        this.spanX = win[1] - win[0];
        this.spanZ = win[3] - win[2];
        this.lines = buildLines();
    }

    public WorldProjection.Cube projection() {
        return wp;
    }

    /** The net point at window fractions (fx, fy): {x, z}. */
    public double[] blockAt(double fx, double fy) {
        return new double[]{x0 + fx * spanX, z0 + fy * spanZ};
    }

    /** The window fractions of the net point (x, z), x wrapped into the window: {fx, fy}. */
    public double[] fractionOf(double x, double z) {
        return new double[]{(cube.wrapXPoint(x) - x0) / spanX, (z - z0) / spanZ};
    }

    /** The window fractions of (lat, lon) in degrees: its point on the faces. */
    public double[] fractionOfLatLon(double latDeg, double lonDeg) {
        double[] p = wp.blockOf(latDeg, lonDeg);
        return fractionOf(p[0], p[1]);
    }

    /** The face at window fractions (fx, fy), or null in the void. */
    public CubeLayout.Face faceAt(double fx, double fy) {
        double[] p = blockAt(fx, fy);
        CubeLayout.LocalPoint l = cube.localOfPoint(p[0], p[1]);
        return l == null ? null : l.face();
    }

    /** {lat, lon} in degrees at window fractions (fx, fy), or null in the void. */
    public double[] latLonAt(double fx, double fy) {
        double[] p = blockAt(fx, fy);
        if (cube.localOfPoint(p[0], p[1]) == null)
            return null;
        return wp.latLon(p[0], p[1]);
    }

    /** The border's twelve lines, then the five joined face sides. */
    public List<Line> lines() {
        return lines;
    }

    /**
     * A {@code w × h} picture of the net (ABGR, row by row from the north): each pixel centre on a
     * face takes {@code paint} at its net point, the void is transparent (0).
     */
    public int[] picture(int w, int h, Paint paint) {
        int[] px = new int[w * h];
        for (int y = 0; y < h; y++) {
            double fy = (y + 0.5) / h;
            for (int x = 0; x < w; x++) {
                double[] p = blockAt((x + 0.5) / w, fy);
                if (cube.localOfPoint(p[0], p[1]) != null)
                    px[y * w + x] = paint.abgr(p[0], p[1]);
            }
        }
        return px;
    }

    private List<Line> buildLines() {
        List<Line> out = new ArrayList<>();
        Set<String> owned = new java.util.HashSet<>();
        for (CubeLayout.NetEdge e : cube.edges()) {
            owned.add(e.face().name() + "/" + e.faceSide().name());
            if ("wrap_engine".equals(e.owner()))
                out.add(line(e.axis() == CubeLayout.Axis.X, e.coord(), e.spanFrom(), e.spanTo(), LineKind.BORDER));
        }
        // The face sides no net edge owns are the ones the net joins; each is met from both faces.
        Set<String> seen = new LinkedHashSet<>();
        long s = cube.side();
        for (CubeLayout.Face f : CubeLayout.Face.all())
            for (CubeLayout.FaceSide side : CubeLayout.FaceSide.all()) {
                if (owned.contains(f.name() + "/" + side.name()))
                    continue;
                long ox = cube.faceOriginX(f), oz = cube.faceOriginZ(f);
                boolean vertical = side == CubeLayout.FaceSide.EAST || side == CubeLayout.FaceSide.WEST;
                long coord = switch (side) {
                    case EAST -> ox + s;
                    case WEST -> ox;
                    case SOUTH -> oz + s;
                    case NORTH -> oz;
                };
                long from = vertical ? oz : ox;
                if (seen.add((vertical ? "x" : "z") + coord + "@" + from))
                    out.add(line(vertical, coord, from, from + s, LineKind.JOINED));
            }
        return List.copyOf(out);
    }

    /** The line x = coord (vertical) or z = coord, from {@code from} to {@code to} along it, as fractions. */
    private Line line(boolean vertical, long coord, long from, long to, LineKind kind) {
        if (vertical) {
            double fx = (coord - x0) / spanX;
            return new Line(fx, (from - z0) / spanZ, fx, (to - z0) / spanZ, kind);
        }
        double fy = (coord - z0) / spanZ;
        return new Line((from - x0) / spanX, fy, (to - x0) / spanX, fy, kind);
    }
}
