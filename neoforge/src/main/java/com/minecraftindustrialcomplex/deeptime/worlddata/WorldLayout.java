// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.ArrayList;
import java.util.List;

/**
 * {@code manifest.json} {@code projection}: the world's projection and how its edges are glued
 * (FORMAT.md "The world's layout", format 0.5; Rust {@code deeptime_format::projection::ProjectionSpec}).
 * The contract the wrap engine reads to install its topology; the game reads the border from it.
 *
 * @param kind             the projection
 * @param layoutVersion    the block's schema version (1)
 * @param hemisphereBlocks b (Petroff–Guyou; 0 on Lambert)
 * @param lambdaCDeg       λc, the western square's centre longitude (Petroff–Guyou; 0 on Lambert)
 * @param xMin             the fundamental domain: x in [xMin, xMax), z in [zMin, zMax]
 * @param edges            one entry per domain edge
 * @param conePoints       the layout's cone points (none on Lambert)
 * @param poles            each pole's images (none on Lambert)
 * @param border           the temporary border at edges nothing joins yet, or null
 */
public record WorldLayout(WorldProjection.Kind kind, int layoutVersion, int hemisphereBlocks, double lambdaCDeg,
                          double xMin, double xMax, double zMin, double zMax, List<Edge> edges,
                          List<ConePoint> conePoints, List<Pole> poles, Border border) {

    /** The schema version this reader knows. */
    public static final int LAYOUT_VERSION = 1;

    /**
     * One edge (the wrap engine's {@code WrapEdge}): {@code axis} x (east/west, the line x = coord) or z;
     * {@code side} min (beyond is v &lt; coord) or max (beyond is v ≥ coord); {@code iso} the motion
     * beyond → inside as {@code [quarterTurns, tx, tz]} ({@code CellIsometry}); {@code pivot} the
     * half-turn's centre or null; {@code owner} {@code toroidal_world} or {@code wrap_engine}.
     */
    public record Edge(String name, String kind, boolean axisX, boolean sideMin, long coord, long spanFrom, long spanTo,
                       int quarterTurns, long tx, long tz, long[] pivot, String owner) {

        public boolean halfTurn() {
            return "half_turn".equals(kind);
        }
    }

    /** A cone point: {@code images} are block corners {x, z}. */
    public record ConePoint(String name, double coneDeg, double latDeg, double lonDeg, List<long[]> images) {}

    /** A pole's images (block corners {x, z}). */
    public record Pole(String name, List<long[]> images) {}

    /**
     * The temporary border: {@code edge_stop} stops players, their vehicles and Sable ships
     * {@code insetBlocks} inside the named edges until the wrap engine joins them.
     */
    public record Border(String kind, List<String> edges, double insetBlocks, boolean temporary) {

        public boolean stops(String edge) {
            return "edge_stop".equals(kind) && edges.contains(edge);
        }
    }

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

    /** Parses the block (FORMAT.md "The world's layout"). */
    public static WorldLayout fromJson(JsonObject o) {
        String k = o.get("kind").getAsString();
        WorldProjection.Kind kind = WorldProjection.Kind.parse(k);
        if (kind == null)
            throw new IllegalArgumentException("unknown projection kind '" + k + "'");
        int version = o.has("layout_version") ? o.get("layout_version").getAsInt() : LAYOUT_VERSION;
        if (version != LAYOUT_VERSION)
            throw new IllegalArgumentException("projection layout_version " + version + ", this Deep Time reads " + LAYOUT_VERSION);
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
                    e.get("owner").getAsString()));
        }
        List<ConePoint> cones = new ArrayList<>();
        if (o.has("cone_points"))
            for (JsonElement el : o.getAsJsonArray("cone_points")) {
                JsonObject c = el.getAsJsonObject();
                cones.add(new ConePoint(c.get("name").getAsString(), c.get("cone_deg").getAsDouble(),
                        c.get("lat_deg").getAsDouble(), c.get("lon_deg").getAsDouble(), points(c.getAsJsonArray("images"))));
            }
        List<Pole> poles = new ArrayList<>();
        if (o.has("poles"))
            for (JsonElement el : o.getAsJsonArray("poles")) {
                JsonObject p = el.getAsJsonObject();
                poles.add(new Pole(p.get("name").getAsString(), points(p.getAsJsonArray("images"))));
            }
        Border border = null;
        if (o.has("border") && o.get("border").isJsonObject()) {
            JsonObject b = o.getAsJsonObject("border");
            List<String> names = new ArrayList<>();
            for (JsonElement el : b.getAsJsonArray("edges"))
                names.add(el.getAsString());
            border = new Border(b.get("kind").getAsString(), List.copyOf(names), b.get("inset_blocks").getAsDouble(),
                    !b.has("temporary") || b.get("temporary").getAsBoolean());
        }
        return new WorldLayout(kind, version,
                o.has("hemisphere_blocks") ? o.get("hemisphere_blocks").getAsInt() : 0,
                o.has("lambda_c_deg") ? o.get("lambda_c_deg").getAsDouble() : 0.0,
                d.get("x_min").getAsDouble(), d.get("x_max").getAsDouble(), d.get("z_min").getAsDouble(), d.get("z_max").getAsDouble(),
                List.copyOf(edges), List.copyOf(cones), List.copyOf(poles), border);
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
