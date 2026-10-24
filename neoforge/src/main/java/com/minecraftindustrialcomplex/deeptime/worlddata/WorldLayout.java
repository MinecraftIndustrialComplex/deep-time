// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.ArrayList;
import java.util.List;

/**
 * {@code manifest.json} {@code projection}: the world's projection and how its edges are glued
 * (FORMAT.md "The world's layout", format 0.5, layout version 1; "The cube world's layout", format
 * 0.7, layout version 2; Rust {@code deeptime_format::projection::ProjectionSpec}). The contract the
 * wrap engine reads to install its topology; the game reads the border from it.
 *
 * @param kind             the projection
 * @param layoutVersion    the block's schema version (1 on the polar worlds, 2 on a cube world)
 * @param hemisphereBlocks b (Petroff–Guyou; 0 otherwise)
 * @param lambdaCDeg       λc, the western square's centre longitude (Petroff–Guyou; 0 otherwise)
 * @param xMin             the fundamental domain: x in [xMin, xMax), z in [zMin, zMax]
 * @param edges            one entry per domain edge
 * @param conePoints       the layout's cone points (none on Lambert)
 * @param poles            each pole's images (none on Lambert)
 * @param border           the temporary border at edges nothing joins yet, or null
 * @param cube             a cube world's net, map and turn (layout version 2), else null
 */
public record WorldLayout(WorldProjection.Kind kind, int layoutVersion, int hemisphereBlocks, double lambdaCDeg,
                          double xMin, double xMax, double zMin, double zMax, List<Edge> edges,
                          List<ConePoint> conePoints, List<Pole> poles, Border border, CubeSpec cube) {

    /** The schema version of the polar layouts (Lambert, Petroff–Guyou). */
    public static final int LAYOUT_VERSION = 1;

    /** The schema version of the cube world's layout (Rust {@code CUBE_LAYOUT_VERSION}). */
    public static final int CUBE_LAYOUT_VERSION = 2;

    /** A polar layout (no cube block). */
    public WorldLayout(WorldProjection.Kind kind, int layoutVersion, int hemisphereBlocks, double lambdaCDeg,
                       double xMin, double xMax, double zMin, double zMax, List<Edge> edges,
                       List<ConePoint> conePoints, List<Pole> poles, Border border) {
        this(kind, layoutVersion, hemisphereBlocks, lambdaCDeg, xMin, xMax, zMin, zMax, edges, conePoints, poles, border, null);
    }

    /**
     * One edge (the wrap engine's {@code WrapEdge}): {@code axis} x (east/west, the line x = coord) or z;
     * {@code side} min (beyond is v &lt; coord) or max (beyond is v ≥ coord); {@code iso} the motion
     * beyond → inside as {@code [quarterTurns, tx, tz]} ({@code CellIsometry}); {@code pivot} the
     * turn's centre or null; {@code owner} {@code toroidal_world} or {@code wrap_engine}. On a cube
     * world {@code seam} names the cube edge ({@code east_west}, {@code ne}, ...) and {@code partner}
     * the other net edge of it; both null on the polar layouts.
     */
    public record Edge(String name, String kind, boolean axisX, boolean sideMin, long coord, long spanFrom, long spanTo,
                       int quarterTurns, long tx, long tz, long[] pivot, String owner, String seam, String partner) {

        /** A polar layout's edge (no seam, no partner). */
        public Edge(String name, String kind, boolean axisX, boolean sideMin, long coord, long spanFrom, long spanTo,
                    int quarterTurns, long tx, long tz, long[] pivot, String owner) {
            this(name, kind, axisX, sideMin, coord, spanFrom, spanTo, quarterTurns, tx, tz, pivot, owner, null, null);
        }

        public boolean halfTurn() {
            return "half_turn".equals(kind);
        }

        public boolean slide() {
            return "slide".equals(kind);
        }

        /** The motion beyond → inside. */
        public CubeLayout.Iso iso() {
            return new CubeLayout.Iso(quarterTurns, tx, tz);
        }
    }

    /**
     * A cone point: {@code images} are block corners {x, z}. On a cube world also the three
     * {@code faces} that meet there, the corner's cube-frame direction {@code cube}, the corner
     * search's {@code landDistanceDeg} (NaN when the manifest has none) and whether a player can stand
     * on it ({@code reachable}); null, NaN and false on the polar layouts.
     */
    public record ConePoint(String name, double coneDeg, double latDeg, double lonDeg, List<long[]> images,
                            List<String> faces, int[] cube, double landDistanceDeg, boolean reachable) {

        /** A polar layout's cone point. */
        public ConePoint(String name, double coneDeg, double latDeg, double lonDeg, List<long[]> images) {
            this(name, coneDeg, latDeg, lonDeg, images, null, null, Double.NaN, false);
        }
    }

    /**
     * A pole's images: {x, z} net points (block corners on the polar layouts, any point on a cube
     * world), and on a cube world the face and local (u, v) it lies at (null, NaN otherwise).
     */
    public record Pole(String name, List<double[]> images, String face, double u, double v) {}

    /**
     * The temporary border: {@code edge_stop} stops players, their vehicles and Sable ships
     * {@code insetBlocks} inside the named edges until the wrap engine joins them. On a cube world the
     * names are the 12 turning net edges ({@code segments} in the manifest).
     */
    public record Border(String kind, List<String> edges, double insetBlocks, boolean temporary) {

        public boolean stops(String edge) {
            return "edge_stop".equals(kind) && edges.contains(edge);
        }
    }

    /**
     * A cube world's own block (layout version 2): the face side, the map table by name and sha256,
     * the rotation planet → cube (3 rows) and the faces as written.
     */
    public record CubeSpec(int faceBlocks, String mapName, String mapSha256, double[][] rotation, List<FaceSpec> faces) {}

    /** One face as the manifest lists it ({@code faces[]}). */
    public record FaceSpec(String id, int index, long originX, long originZ, List<String> frame,
                           double centreLatDeg, double centreLonDeg) {}

    /** The edge named {@code name}, or null. */
    public Edge edge(String name) {
        for (Edge e : edges)
            if (e.name().equals(name))
                return e;
        return null;
    }

    /** The world projection this block describes, for a world of x period {@code c}. */
    public WorldProjection projection(int c) {
        return WorldProjection.of(this, c);
    }

    /** Parses the block (FORMAT.md "The world's layout", "The cube world's layout"). */
    public static WorldLayout fromJson(JsonObject o) {
        String k = o.get("kind").getAsString();
        WorldProjection.Kind kind = WorldProjection.Kind.parse(k);
        if (kind == null)
            throw new IllegalArgumentException("unknown projection kind '" + k + "'");
        int want = kind == WorldProjection.Kind.CUBE ? CUBE_LAYOUT_VERSION : LAYOUT_VERSION;
        int version = o.has("layout_version") ? o.get("layout_version").getAsInt() : want;
        if (version != want)
            throw new IllegalArgumentException("projection layout_version " + version + " on a " + k + " world, this Deep Time reads "
                    + want);
        JsonObject d = o.getAsJsonObject("domain");
        List<Edge> edges = new ArrayList<>();
        for (JsonElement el : o.getAsJsonArray("edges")) {
            JsonObject e = el.getAsJsonObject();
            JsonArray span = e.getAsJsonArray("span"), iso = e.getAsJsonArray("iso");
            long[] pivot = null;
            if (e.has("pivot") && e.get("pivot").isJsonArray()) {
                JsonArray p = e.getAsJsonArray("pivot");
                pivot = new long[]{p.get(0).getAsLong(), p.get(1).getAsLong()};
            }
            edges.add(new Edge(e.get("name").getAsString(), e.get("kind").getAsString(),
                    "x".equals(e.get("axis").getAsString()), "min".equals(e.get("side").getAsString()),
                    e.get("coord").getAsLong(), span.get(0).getAsLong(), span.get(1).getAsLong(),
                    iso.get(0).getAsInt(), iso.get(1).getAsLong(), iso.get(2).getAsLong(), pivot,
                    e.get("owner").getAsString(), string(e, "seam"), string(e, "partner")));
        }
        List<ConePoint> cones = new ArrayList<>();
        if (o.has("cone_points"))
            for (JsonElement el : o.getAsJsonArray("cone_points")) {
                JsonObject c = el.getAsJsonObject();
                List<String> faces = null;
                if (c.has("faces") && c.get("faces").isJsonArray()) {
                    faces = new ArrayList<>();
                    for (JsonElement f : c.getAsJsonArray("faces"))
                        faces.add(f.getAsString());
                    faces = List.copyOf(faces);
                }
                int[] cube = null;
                if (c.has("cube") && c.get("cube").isJsonArray()) {
                    JsonArray a = c.getAsJsonArray("cube");
                    cube = new int[]{a.get(0).getAsInt(), a.get(1).getAsInt(), a.get(2).getAsInt()};
                }
                cones.add(new ConePoint(c.get("name").getAsString(), c.get("cone_deg").getAsDouble(),
                        c.get("lat_deg").getAsDouble(), c.get("lon_deg").getAsDouble(), points(c.getAsJsonArray("images")),
                        faces, cube, number(c, "land_distance_deg"),
                        c.has("reachable") && !c.get("reachable").isJsonNull() && c.get("reachable").getAsBoolean()));
            }
        List<Pole> poles = new ArrayList<>();
        if (o.has("poles"))
            for (JsonElement el : o.getAsJsonArray("poles")) {
                JsonObject p = el.getAsJsonObject();
                List<double[]> images = new ArrayList<>();
                if (p.has("images"))
                    for (JsonElement im : p.getAsJsonArray("images")) {
                        JsonArray a = im.getAsJsonArray();
                        images.add(new double[]{a.get(0).getAsDouble(), a.get(1).getAsDouble()});
                    }
                poles.add(new Pole(p.get("name").getAsString(), List.copyOf(images), string(p, "face"), number(p, "u"), number(p, "v")));
            }
        Border border = null;
        if (o.has("border") && o.get("border").isJsonObject()) {
            JsonObject b = o.getAsJsonObject("border");
            List<String> names = new ArrayList<>();
            if (b.has("edges") && b.get("edges").isJsonArray())
                for (JsonElement el : b.getAsJsonArray("edges"))
                    names.add(el.getAsString());
            // Layout version 2 names its border by segments, the turning net edges.
            if (b.has("segments") && b.get("segments").isJsonArray())
                for (JsonElement el : b.getAsJsonArray("segments"))
                    if (!names.contains(el.getAsString()))
                        names.add(el.getAsString());
            border = new Border(b.get("kind").getAsString(), List.copyOf(names), b.get("inset_blocks").getAsDouble(),
                    !b.has("temporary") || b.get("temporary").getAsBoolean());
        }
        CubeSpec cube = kind == WorldProjection.Kind.CUBE ? cubeSpec(o) : null;
        return new WorldLayout(kind, version,
                o.has("hemisphere_blocks") && !o.get("hemisphere_blocks").isJsonNull() ? o.get("hemisphere_blocks").getAsInt() : 0,
                o.has("lambda_c_deg") && !o.get("lambda_c_deg").isJsonNull() ? o.get("lambda_c_deg").getAsDouble() : 0.0,
                d.get("x_min").getAsDouble(), d.get("x_max").getAsDouble(), d.get("z_min").getAsDouble(), d.get("z_max").getAsDouble(),
                List.copyOf(edges), List.copyOf(cones), List.copyOf(poles), border, cube);
    }

    /** The cube block: {@code face_blocks}, {@code map}, {@code rotation}, {@code faces} (each required). */
    private static CubeSpec cubeSpec(JsonObject o) {
        if (!o.has("face_blocks") || o.get("face_blocks").isJsonNull())
            throw new IllegalArgumentException("cube layout without face_blocks");
        int face = o.get("face_blocks").getAsInt();
        if (face < 2 || face % 2 != 0)
            throw new IllegalArgumentException("cube face_blocks " + face + " is not even");
        if (!o.has("map") || !o.get("map").isJsonObject())
            throw new IllegalArgumentException("cube layout without map");
        JsonObject map = o.getAsJsonObject("map");
        if (!o.has("rotation") || !o.get("rotation").isJsonArray())
            throw new IllegalArgumentException("cube layout without rotation");
        JsonArray rows = o.getAsJsonArray("rotation");
        double[][] rot = new double[rows.size()][];
        for (int i = 0; i < rows.size(); i++) {
            JsonArray r = rows.get(i).getAsJsonArray();
            rot[i] = new double[r.size()];
            for (int j = 0; j < r.size(); j++)
                rot[i][j] = r.get(j).getAsDouble();
        }
        List<FaceSpec> faces = new ArrayList<>();
        if (o.has("faces") && o.get("faces").isJsonArray())
            for (JsonElement el : o.getAsJsonArray("faces")) {
                JsonObject f = el.getAsJsonObject();
                JsonArray origin = f.getAsJsonArray("origin");
                List<String> frame = new ArrayList<>();
                if (f.has("frame"))
                    for (JsonElement fr : f.getAsJsonArray("frame"))
                        frame.add(fr.getAsString());
                faces.add(new FaceSpec(f.get("id").getAsString(), f.get("index").getAsInt(),
                        origin.get(0).getAsLong(), origin.get(1).getAsLong(), List.copyOf(frame),
                        number(f, "centre_lat_deg"), number(f, "centre_lon_deg")));
            }
        return new CubeSpec(face, map.get("name").getAsString(), map.get("sha256").getAsString(), rot, List.copyOf(faces));
    }

    private static String string(JsonObject o, String key) {
        return o.has(key) && !o.get(key).isJsonNull() ? o.get(key).getAsString() : null;
    }

    private static double number(JsonObject o, String key) {
        return o.has(key) && !o.get(key).isJsonNull() ? o.get(key).getAsDouble() : Double.NaN;
    }

    private static List<long[]> points(JsonArray a) {
        List<long[]> out = new ArrayList<>();
        if (a != null)
            for (JsonElement el : a) {
                JsonArray p = el.getAsJsonArray();
                out.add(new long[]{p.get(0).getAsLong(), p.get(1).getAsLong()});
            }
        return List.copyOf(out);
    }
}
