// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.Manifest;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.minecraftindustrialcomplex.deeptime.wrap.CubeBorder;
import com.minecraftindustrialcomplex.deeptime.wrap.EdgeBorder;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * A cube world's twelve edges and eight corners on the planet (cube plan G13, "the globe adds the
 * cube's edges and corners"), in latitude and longitude, for the timelapse's globe. The cube's turn
 * on the planet is chosen when the run's world data is baked (on the final land), so a run has an
 * outline only once it is done and its manifest is a cube's ({@link #read}).
 *
 * <p>An edge is one of the cube's twelve, met by two faces. Its kind says how the game joins it:
 * {@link Kind#BORDER} for the six whose two net edges turn (the wrap engine's, where the temporary
 * border stops a player, docs/design/cube-game.md section 5), {@link Kind#SLIDE} for F2 against F3
 * (Toroidal World's slide), {@link Kind#JOINED} for the five the net keeps side by side. Each edge
 * is sampled at {@link #SAMPLES} + 1 points along its face side, through the world's own map
 * ({@code CubeWorld.unitOnFace}), so the line is where the world's faces meet. A corner is
 * reachable when a player can stand on it under the border: the four that touch F0.
 */
public record CubeOutline(List<Edge> edges, List<Corner> corners) {

    /** Segments per edge (an edge spans about 70° of arc: under 1.5° a segment). */
    public static final int SAMPLES = 48;

    /** How the game joins an edge. */
    public enum Kind {
        BORDER, SLIDE, JOINED
    }

    /**
     * One cube edge.
     *
     * @param name   the two faces, in the format's order ({@code F0|N})
     * @param latLon {@link #SAMPLES} + 1 points {lat, lon} in degrees, lon in [0, 360)
     */
    public record Edge(String name, Kind kind, double[][] latLon) {}

    /** A cube corner ({@code n0} .. {@code s3}) at (lat, lon) in degrees. */
    public record Corner(String name, double lat, double lon, boolean reachable) {}

    /** The outline of a cube world's projection. */
    public static CubeOutline of(WorldProjection.Cube wp) {
        CubeLayout cube = wp.cube();
        Map<String, String> owner = new HashMap<>();
        for (CubeLayout.NetEdge e : cube.edges())
            owner.put(e.face().name() + "/" + e.faceSide().name(), e.owner());
        List<Edge> edges = new ArrayList<>();
        java.util.Set<String> seen = new java.util.HashSet<>();
        for (CubeLayout.Face f : CubeLayout.Face.all())
            for (CubeLayout.FaceSide side : CubeLayout.FaceSide.all()) {
                CubeLayout.Face g = f.across(side).face();
                CubeLayout.Face a = f.index() < g.index() ? f : g, b = a == f ? g : f;
                String name = a.name() + "|" + b.name();
                if (!seen.add(name))
                    continue;
                String o = owner.get(f.name() + "/" + side.name());
                Kind kind = o == null ? Kind.JOINED : "wrap_engine".equals(o) ? Kind.BORDER : Kind.SLIDE;
                double[][] pts = new double[SAMPLES + 1][];
                for (int k = 0; k <= SAMPLES; k++) {
                    double t = -1.0 + 2.0 * k / SAMPLES;
                    double u = switch (side) {
                        case EAST -> 1.0;
                        case WEST -> -1.0;
                        default -> t;
                    };
                    double v = switch (side) {
                        case SOUTH -> 1.0;
                        case NORTH -> -1.0;
                        default -> t;
                    };
                    pts[k] = WorldProjection.latLonOfUnit(wp.world().unitOnFace(f, u, v));
                }
                edges.add(new Edge(name, kind, pts));
            }
        CubeBorder border = CubeBorder.of(cube.side());
        List<Corner> corners = new ArrayList<>();
        for (CubeLayout.Corner c : cube.corners()) {
            boolean reachable = false;
            for (long[] img : c.images())
                reachable |= border.inside(img[0], img[1], EdgeBorder.INSET);
            double[] ll = wp.world().cornerLatLon(c.cube());
            corners.add(new Corner(c.name(), ll[0], ll[1], reachable));
        }
        return new CubeOutline(List.copyOf(edges), List.copyOf(corners));
    }

    /** The outline of the world data in {@code dir} when it is a cube world's; empty otherwise or when it cannot be read. */
    public static Optional<CubeOutline> read(Path dir) {
        try {
            Manifest m = Manifest.read(dir);
            return m.projection() instanceof WorldProjection.Cube c ? Optional.of(of(c)) : Optional.empty();
        } catch (RuntimeException e) {
            return Optional.empty();
        }
    }
}
