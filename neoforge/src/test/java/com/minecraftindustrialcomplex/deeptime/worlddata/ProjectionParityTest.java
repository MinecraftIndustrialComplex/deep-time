// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.List;
import java.util.SplittableRandom;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

/**
 * The seamless world's projection in Java against the Rust original, bit for bit: every output in
 * {@code crates/deeptime-format/tests/data/projection_parity.json} (written by deeptime-format's
 * {@code parity_fixture} test) comes out of {@link PortableMath}, {@link Petroff} and
 * {@link PetroffGuyou} with the same bits. Plus the properties the Rust tests assert (round trips,
 * equal area, seam closure ≤ 10⁻⁸ rad), the cell fold and the layout block's parsing.
 */
class ProjectionParityTest {

    private static final double[][] WORLDS = {{6656, 31.25}, {13056, 200.0}, {1536, 0.0}};

    private static JsonObject fixture() throws IOException {
        return JsonParser.parseString(Files.readString(
                Fixtures.repoRoot().resolve("crates/deeptime-format/tests/data/projection_parity.json"), StandardCharsets.UTF_8))
                .getAsJsonObject();
    }

    private static double f(JsonElement hex) {
        return Double.longBitsToDouble(Long.parseUnsignedLong(hex.getAsString(), 16));
    }

    private static void same(String what, double got, JsonElement want) {
        long w = Long.parseUnsignedLong(want.getAsString(), 16);
        if (Double.doubleToRawLongBits(got) != w)
            fail(what + ": got " + got + " (" + Long.toHexString(Double.doubleToRawLongBits(got)) + "), want "
                    + Double.longBitsToDouble(w) + " (" + want.getAsString() + ")");
    }

    @Test
    void everyFixtureValueHasRustsBits() throws IOException {
        JsonObject fx = fixture();
        int checked = 0;
        int k = 0;
        for (JsonElement el : fx.getAsJsonArray("math")) {
            JsonObject r = el.getAsJsonObject();
            double x = f(r.get("x")), y = f(r.get("y")), u = f(r.get("u"));
            same("math[" + k + "].sin", PortableMath.sin(x), r.get("sin"));
            same("math[" + k + "].cos", PortableMath.cos(x), r.get("cos"));
            same("math[" + k + "].tan", PortableMath.tan(x), r.get("tan"));
            same("math[" + k + "].atan", PortableMath.atan(x), r.get("atan"));
            same("math[" + k + "].atan2", PortableMath.atan2(y, x), r.get("atan2"));
            same("math[" + k + "].asin", PortableMath.asin(u), r.get("asin"));
            same("math[" + k + "].acos", PortableMath.acos(u), r.get("acos"));
            checked += 7;
            k++;
        }
        Petroff pet = new Petroff();
        k = 0;
        for (JsonElement el : fx.getAsJsonArray("petroff")) {
            JsonObject r = el.getAsJsonObject();
            double[] ll = pet.inverse(f(r.get("x")), f(r.get("y")));
            same("petroff[" + k + "].lambda", ll[0], r.get("lambda"));
            same("petroff[" + k + "].phi", ll[1], r.get("phi"));
            double[] xy = pet.forward(f(r.get("lam_in")), f(r.get("phi_in")));
            same("petroff[" + k + "].x_out", xy[0], r.get("x_out"));
            same("petroff[" + k + "].y_out", xy[1], r.get("y_out"));
            checked += 4;
            k++;
        }
        for (JsonElement we : fx.getAsJsonArray("worlds")) {
            JsonObject w = we.getAsJsonObject();
            long b = w.get("b").getAsLong();
            PetroffGuyou g = new PetroffGuyou(b, f(w.get("lambda_c_deg")));
            k = 0;
            for (JsonElement el : w.getAsJsonArray("points")) {
                JsonObject r = el.getAsJsonObject();
                double x = f(r.get("x")), z = f(r.get("z"));
                double[] v = g.unit(x, z);
                JsonArray rv = r.getAsJsonArray("v");
                for (int i = 0; i < 3; i++)
                    same("b " + b + " points[" + k + "].v[" + i + "]", v[i], rv.get(i));
                double[] ll = g.latLon(x, z);
                same("b " + b + " points[" + k + "].lat", ll[0], r.get("lat"));
                same("b " + b + " points[" + k + "].lon", ll[1], r.get("lon"));
                checked += 5;
                k++;
            }
            k = 0;
            for (JsonElement el : w.getAsJsonArray("inverse")) {
                JsonObject r = el.getAsJsonObject();
                double[] p = g.blockOf(f(r.get("lat")), f(r.get("lon")));
                same("b " + b + " inverse[" + k + "].x", p[0], r.get("x"));
                same("b " + b + " inverse[" + k + "].z", p[1], r.get("z"));
                checked += 2;
                k++;
            }
            // Exact inputs on the forward's degenerate lines (the equator, the tiles' midlines and
            // edges, the cone points, the 67.5° vertices; ledger: SCL-083).
            k = 0;
            for (JsonElement el : w.getAsJsonArray("exact")) {
                JsonObject r = el.getAsJsonObject();
                double[] p = g.blockOf(f(r.get("lat")), f(r.get("lon")));
                same("b " + b + " exact[" + k + "].x", p[0], r.get("x"));
                same("b " + b + " exact[" + k + "].z", p[1], r.get("z"));
                checked += 2;
                k++;
            }
            assertTrue(k > 400, "b " + b + ": " + k + " exact rows");
        }
        assertTrue(checked > 2000, "the fixture holds " + checked + " values");
    }

    /**
     * Sphere to block to sphere on exact inputs where Petroff's forward is degenerate (ledger:
     * SCL-083): the equator at every 0.1° of longitude, the meridians λc, λc ± 90° and λc + 180° at
     * every 0.1° of latitude, the cone points, the 67.5° vertices, the tile centres and the poles.
     * Before 2026-10-20 these landed up to 10.5° off (the play world's cone points at (616, −3328)).
     */
    @Test
    void exactInputsOnTheDiagonalsRoundTrip() {
        double[][] worlds = {{6656, 31.25}, {13056, 200.0}, {1536, 0.0}, {6656, 146.0}};
        for (double[] wc : worlds) {
            PetroffGuyou g = new PetroffGuyou((long) wc[0], wc[1]);
            double b = wc[0], h = b / 2, lc = wc[1];
            java.util.List<double[]> pts = new java.util.ArrayList<>();
            for (int k = 0; k < 3600; k++)
                pts.add(new double[]{0.0, k * 0.1});
            for (int k = -899; k <= 899; k++)
                for (double d : new double[]{0, 90, 180, 270})
                    pts.add(new double[]{k * 0.1, PortableMath.wrapDeg(lc + d)});
            for (double lat : new double[]{45, -45})
                for (double d : new double[]{90, 270})
                    pts.add(new double[]{lat, PortableMath.wrapDeg(lc + d)});
            for (double d : new double[]{0, 180}) {
                for (double o : new double[]{22.5, -22.5}) {
                    pts.add(new double[]{0.0, PortableMath.wrapDeg(lc + d + o)});
                    pts.add(new double[]{o, PortableMath.wrapDeg(lc + d)});
                }
                pts.add(new double[]{0.0, PortableMath.wrapDeg(lc + d)});
            }
            pts.add(new double[]{90, 0});
            pts.add(new double[]{-90, 0});
            double worst = 0;
            for (double[] ll : pts) {
                double[] p = g.blockOf(ll[0], ll[1]);
                String at = "b " + (long) b + " λc " + lc + ": (" + ll[0] + ", " + ll[1] + ") -> (" + p[0] + ", " + p[1] + ")";
                assertTrue(p[0] >= -b && p[0] < b && Math.abs(p[1]) <= h, at);
                double d = Math.toDegrees(angle(g.unit(p[0], p[1]), unitOf(ll[0], ll[1])));
                worst = Math.max(worst, d);
                assertTrue(d < 1e-5, at + " is " + d + "° off");
                if (ll[0] == 0.0)
                    assertTrue(Math.abs(p[1]) < 1e-6, at + ": the equator is z = 0");
            }
            System.out.println("exact diagonal inputs, b " + (long) b + " λc " + lc + ": " + pts.size()
                    + " points, worst round trip " + worst + "°");
        }
    }

    private static double angle(double[] a, double[] b) {
        double cx = a[1] * b[2] - a[2] * b[1], cy = a[2] * b[0] - a[0] * b[2], cz = a[0] * b[1] - a[1] * b[0];
        return Math.atan2(Math.sqrt(cx * cx + cy * cy + cz * cz), a[0] * b[0] + a[1] * b[1] + a[2] * b[2]);
    }

    private static double[] unitOf(double latDeg, double lonDeg) {
        double la = Math.toRadians(latDeg), lo = Math.toRadians(lonDeg);
        return new double[]{Math.cos(la) * Math.cos(lo), Math.cos(la) * Math.sin(lo), Math.sin(la)};
    }

    @Test
    void roundTripsToTheSphereAndBack() {
        for (double[] wc : WORLDS) {
            PetroffGuyou g = new PetroffGuyou((long) wc[0], wc[1]);
            double b = wc[0], h = b / 2;
            SplittableRandom rng = new SplittableRandom(0xC0DE ^ (long) b);
            for (int n = 0; n < 20_000; n++) {
                double lat = Math.toDegrees(Math.asin(2 * rng.nextDouble() - 1)), lon = 360 * rng.nextDouble();
                double[] p = g.blockOf(lat, lon);
                assertTrue(p[0] >= -b && p[0] < b && Math.abs(p[1]) <= h, "(" + lat + ", " + lon + ") -> " + p[0] + ", " + p[1]);
                double d = angle(g.unit(p[0], p[1]), unitOf(lat, lon));
                assertTrue(d < 1e-8, "(" + lat + ", " + lon + "): " + d + " rad");
                double x = (rng.nextDouble() - 0.5) * 2 * b, z = (rng.nextDouble() - 0.5) * b;
                double[] q = g.pointOfUnit(g.unit(x, z));
                double e = Math.max(Math.abs(g.wrapX(q[0] - x)), Math.abs(q[1] - z));
                assertTrue(e < 1e-6, "(" + x + ", " + z + ") -> (" + q[0] + ", " + q[1] + ")");
            }
        }
    }

    @Test
    void everyGluedEdgeMapsTheSameSpherePoints() {
        for (double[] wc : WORLDS) {
            PetroffGuyou g = new PetroffGuyou((long) wc[0], wc[1]);
            double b = wc[0], h = b / 2, worst = 0;
            for (int k = 0; k <= 4000; k++) {
                double t = k / 4000.0;
                double z = -h + b * t;
                worst = Math.max(worst, angle(g.unitUnfolded(-b, z), g.unitUnfolded(b, z)));
                double x = -b + 2 * b * t;
                worst = Math.max(worst, angle(g.unitUnfolded(x, -h), g.unitUnfolded(-x, -h)));
                worst = Math.max(worst, angle(g.unitUnfolded(x, h), g.unitUnfolded(-x, h)));
                worst = Math.max(worst, angle(g.unitUnfolded(-1e-12, z), g.unitUnfolded(1e-12, z)));
            }
            assertTrue(worst <= 1e-8, "b " + b + ": seam closure " + worst + " rad");
            // Past an edge is the glued inside.
            for (int k = 0; k < 200; k++) {
                double x = -b + 2 * b * k / 200.0 + 0.37;
                for (double d : new double[]{0.5, 7.0, 40.0}) {
                    assertTrue(angle(g.unit(x, -h - d), g.unit(-x, -h + d)) < 1e-12);
                    assertTrue(angle(g.unit(x, h + d), g.unit(-x, h - d)) < 1e-12);
                }
            }
        }
    }

    @Test
    void everyBlockCoversTheSameArea() {
        for (double[] wc : WORLDS) {
            PetroffGuyou g = new PetroffGuyou((long) wc[0], wc[1]);
            double b = wc[0], h = b / 2, rr = g.radius(), worst = 0;
            SplittableRandom rng = new SplittableRandom(0xA4EA ^ (long) b);
            double[][] cones = {{0, -h}, {0, h}, {-b, -h}, {-b, h}, {b, -h}, {b, h}};
            int n = 0;
            while (n < 5000) {
                double x = (rng.nextDouble() - 0.5) * 2 * b, z = (rng.nextDouble() - 0.5) * (b - 1);
                boolean near = false;
                for (double[] c : cones)
                    near |= Math.hypot(x - c[0], z - c[1]) < 0.01 * b;
                if (near)
                    continue;
                double s = 0.05;
                double[] v0 = g.unit(x, z);
                double best = Double.MAX_VALUE;
                for (int sx : new int[]{1, -1})
                    for (int sz : new int[]{1, -1}) {
                        double[] vx = oneSided(g, v0, x, z, sx, 0, s), vz = oneSided(g, v0, x, z, 0, sz, s);
                        double cx = vx[1] * vz[2] - vx[2] * vz[1], cy = vx[2] * vz[0] - vx[0] * vz[2], cz = vx[0] * vz[1] - vx[1] * vz[0];
                        best = Math.min(best, Math.abs(Math.sqrt(cx * cx + cy * cy + cz * cz) * rr * rr - 1.0));
                    }
                worst = Math.max(worst, best);
                n++;
            }
            assertTrue(worst < 1e-5, "b " + b + ": a block's area off by " + worst);
            assertEquals(2 * b * b, 4 * Math.PI * rr * rr, 1e-6 * b * b, "2b² = 4πR²");
        }
    }

    private static double[] oneSided(PetroffGuyou g, double[] v0, double x, double z, double dx, double dz, double s) {
        double[] v1 = g.unit(x + dx * s, z + dz * s), v2 = g.unit(x + 2 * dx * s, z + 2 * dz * s);
        double k = (dx + dz) / (2 * s);
        return new double[]{(-3 * v0[0] + 4 * v1[0] - v2[0]) * k, (-3 * v0[1] + 4 * v1[1] - v2[1]) * k, (-3 * v0[2] + 4 * v1[2] - v2[2]) * k};
    }

    @Test
    void cellsAndGridRowsFoldThroughTheHalfTurns() {
        PetroffGuyou g = new PetroffGuyou(6656, 31.25);
        long b = 6656, h = b / 2;
        // A cell north of the edge is the half-turned cell inside; twice is the identity.
        for (long x = -b; x < b; x += 97)
            for (long d = 0; d < 40; d += 3) {
                long[] c = g.foldCell(x, -h - 1 - d);
                assertEquals(Math.floorMod(-x - 1 + b, 2 * b) - b, c[0], "x of " + x);
                assertEquals(-h + d, c[1]);
                assertEquals(1, c[2]);
                long[] s = g.foldCell(x, h + d);
                assertEquals(h - 1 - d, s[1]);
            }
        assertEquals(0, g.foldCell(5, 7)[2]);
        // The grid: row −k is row k and row H − 1 + k is row H − 1 − k, both at column W − i.
        GridDesc grid = g.grid("L3", 4);
        assertEquals(2 * b / 4, grid.w());
        assertEquals(b / 4 + 1, grid.h());
        assertEquals(GridDesc.ZEdges.HALF_TURN, grid.zEdges());
        for (long i = -3; i < grid.w() + 3; i += 17)
            for (long k = 1; k < 10; k++) {
                long c = grid.cell(i, -k);
                assertEquals(k, GridDesc.cellJ(c));
                assertEquals(grid.wrapI(grid.w() - i), GridDesc.cellI(c));
                long s = grid.cell(i, grid.h() - 1 + k);
                assertEquals(grid.h() - 1 - k, GridDesc.cellJ(s));
            }
        // The point of a folded cell is the glued planet point.
        long pivotCol = grid.w() / 2;
        assertEquals(0, grid.blockX(pivotCol));
    }

    @Test
    void theLayoutBlockParsesAndBuildsTheWorld() {
        String json = """
                {"kind": "petroff_guyou", "layout_version": 1, "hemisphere_blocks": 6656, "lambda_c_deg": 31.25,
                 "domain": {"x_min": -6656.0, "x_max": 6656.0, "z_min": -3328.0, "z_max": 3328.0},
                 "edges": [
                  {"name": "west", "kind": "slide", "axis": "x", "side": "min", "coord": -6656, "span": [-3328, 3328], "iso": [0, 13312, 0], "owner": "toroidal_world"},
                  {"name": "east", "kind": "slide", "axis": "x", "side": "max", "coord": 6656, "span": [-3328, 3328], "iso": [0, -13312, 0], "owner": "toroidal_world"},
                  {"name": "north", "kind": "half_turn", "axis": "z", "side": "min", "coord": -3328, "span": [-6656, 6656], "iso": [2, 0, -6656], "pivot": [0, -3328], "owner": "wrap_engine"},
                  {"name": "south", "kind": "half_turn", "axis": "z", "side": "max", "coord": 3328, "span": [-6656, 6656], "iso": [2, 0, 6656], "pivot": [0, 3328], "owner": "wrap_engine"}],
                 "cone_points": [{"name": "north-midpoint", "cone_deg": 180.0, "lat_deg": 45.0, "lon_deg": 121.25, "images": [[0, -3328]]}],
                 "poles": [{"name": "north", "images": [[-3328, -3328], [3328, -3328]]}],
                 "border": {"kind": "edge_stop", "edges": ["north", "south"], "inset_blocks": 1.0, "temporary": true}}""";
        WorldLayout l = WorldLayout.fromJson(JsonParser.parseString(json).getAsJsonObject());
        assertEquals(WorldProjection.Kind.PETROFF_GUYOU, l.kind());
        assertEquals(4, l.edges().size());
        WorldLayout.Edge north = l.edge("north");
        assertNotNull(north);
        assertTrue(north.halfTurn() && !north.axisX() && north.sideMin());
        assertEquals(2, north.quarterTurns());
        assertEquals(-6656, north.tz());
        assertEquals("wrap_engine", north.owner());
        assertTrue(l.border().stops("north") && l.border().stops("south") && !l.border().stops("west"));
        WorldProjection wp = l.projection(13312);
        assertTrue(wp instanceof PetroffGuyou);
        assertEquals(13312, wp.period());
        // The poles the block lists are the poles of the world it builds.
        assertEquals(90.0, wp.latLon(-3328, -3328)[0], 1e-6);
        assertEquals(90.0, wp.latLon(3328, -3328)[0], 1e-6);
        assertEquals(-90.0, wp.latLon(-3328, 3328)[0], 1e-6);
        // The equator is z = 0, north is −z along it.
        assertEquals(0.0, wp.latLon(-1000, 0)[0], 1e-9);
        assertTrue(wp.latLon(-1000, -100)[0] > 0);
        double[] n = wp.north(-3328 - 1500, 0);
        assertTrue(n[1] < -0.99, "north is −z on the western square's equator: " + n[0] + ", " + n[1]);
        // A manifest without the block is Lambert.
        assertFalse(WorldProjection.of(null, 16384) instanceof PetroffGuyou);
        assertEquals(6656, PetroffGuyou.hemisphereFor(16384));
        assertEquals(13056, PetroffGuyou.hemisphereFor(32768));
        assertEquals(1536, PetroffGuyou.hemisphereFor(4096));
    }

    /**
     * The kind "cube" is known (W1) but is not a WorldProjection yet (phase 2): the two factories
     * that read a manifest's kind refuse it with a message, never fall back to Lambert.
     */
    @Test
    void theCubeKindIsKnownButRefusedByTheFactoriesUntilPhase2() {
        assertEquals(WorldProjection.Kind.CUBE, WorldProjection.Kind.parse("cube"));
        assertEquals("cube", WorldProjection.Kind.CUBE.id());
        // Every kind still parses from its own id.
        for (WorldProjection.Kind k : WorldProjection.Kind.values())
            assertEquals(k, WorldProjection.Kind.parse(k.id()));
        String json = """
                {"kind": "cube", "layout_version": 1,
                 "domain": {"x_min": -1536.0, "x_max": 2560.0, "z_min": -1536.0, "z_max": 1536.0}, "edges": []}""";
        IllegalArgumentException fromJson = org.junit.jupiter.api.Assertions.assertThrows(IllegalArgumentException.class,
                () -> WorldLayout.fromJson(JsonParser.parseString(json).getAsJsonObject()));
        assertTrue(fromJson.getMessage().contains("cube"), fromJson.getMessage());
        WorldLayout cube = new WorldLayout(WorldProjection.Kind.CUBE, 1, 0, 0.0, -1536, 2560, -1536, 1536,
                java.util.List.of(), java.util.List.of(), java.util.List.of(), null);
        IllegalArgumentException of = org.junit.jupiter.api.Assertions.assertThrows(IllegalArgumentException.class,
                () -> WorldProjection.of(cube, 4096));
        assertTrue(of.getMessage().contains("cube"), of.getMessage());
        // The unknown kinds stay unknown.
        assertNull(WorldProjection.Kind.parse("cubes"));
    }

    // ------------------------------------------------------------------------------------------
    // The cube section: CubeLayout (W1, W2), CubeTurn, CubeMap and CubeWorld (M3) against Rust
    // ------------------------------------------------------------------------------------------

    private int cubeChecked;

    private void bits(String what, double got, JsonElement want) {
        same(what, got, want);
        cubeChecked++;
    }

    private void bits3(String what, double[] got, JsonElement want) {
        JsonArray a = want.getAsJsonArray();
        assertEquals(3, a.size(), what);
        assertEquals(3, got.length, what);
        for (int i = 0; i < 3; i++)
            bits(what + "[" + i + "]", got[i], a.get(i));
    }

    private void num(String what, long got, JsonElement want) {
        assertEquals(want.getAsLong(), got, what);
        cubeChecked++;
    }

    private void text(String what, String got, JsonElement want) {
        assertEquals(want.getAsString(), got, what);
        cubeChecked++;
    }

    private void iso(String what, CubeLayout.Iso got, JsonElement want) {
        JsonArray a = want.getAsJsonArray();
        num(what + ".k", got.k(), a.get(0));
        num(what + ".tx", got.tx(), a.get(1));
        num(what + ".tz", got.tz(), a.get(2));
    }

    private static CubeLayout.Face face(JsonElement index) {
        CubeLayout.Face f = CubeLayout.Face.fromIndex(index.getAsInt());
        assertNotNull(f, "face index " + index);
        return f;
    }

    @Test
    void everyCubeFixtureValueHasRustsBits() throws IOException {
        JsonObject cube = fixture().getAsJsonObject("cube");
        assertNotNull(cube, "the fixture has no cube section");
        cubeChecked = 0;

        // W1: the face side of a planet size.
        int k = 0;
        for (JsonElement el : cube.getAsJsonArray("face_for")) {
            JsonObject r = el.getAsJsonObject();
            num("face_for[" + k + "] c " + r.get("c"), CubeLayout.faceFor(r.get("c").getAsInt()), r.get("s"));
            k++;
        }
        assertEquals(15, k);
        assertEquals(256, CubeLayout.FACE_QUANTUM);

        // W2: the net.
        JsonObject net = cube.getAsJsonObject("net");
        int s = net.get("s").getAsInt();
        CubeLayout layout = CubeLayout.of(s);
        num("net.s", layout.side(), net.get("s"));
        JsonArray edges = net.getAsJsonArray("edges");
        assertEquals(edges.size(), layout.edges().size());
        k = 0;
        for (JsonElement el : edges) {
            JsonObject r = el.getAsJsonObject();
            CubeLayout.NetEdge e = layout.edges().get(k);
            String w = "net.edges[" + k + "] " + e.name();
            text(w + ".name", e.name(), r.get("name"));
            text(w + ".seam", e.seam().id(), r.get("seam"));
            text(w + ".motion", e.motion().id(), r.get("motion"));
            num(w + ".face", e.face().index(), r.get("face"));
            num(w + ".face_side", e.faceSide().index(), r.get("face_side"));
            text(w + ".axis", e.axis() == CubeLayout.Axis.X ? "x" : "z", r.get("axis"));
            text(w + ".beyond", e.beyond() == CubeLayout.Side.MIN ? "min" : "max", r.get("beyond"));
            num(w + ".coord", e.coord(), r.get("coord"));
            num(w + ".span[0]", e.spanFrom(), r.getAsJsonArray("span").get(0));
            num(w + ".span[1]", e.spanTo(), r.getAsJsonArray("span").get(1));
            iso(w + ".iso", e.iso(), r.get("iso"));
            if (r.get("pivot").isJsonNull()) {
                assertNull(e.pivot(), w + ".pivot");
                cubeChecked++;
            } else {
                num(w + ".pivot[0]", e.pivot()[0], r.getAsJsonArray("pivot").get(0));
                num(w + ".pivot[1]", e.pivot()[1], r.getAsJsonArray("pivot").get(1));
            }
            num(w + ".partner", e.partner(), r.get("partner"));
            text(w + ".owner", e.owner(), r.get("owner"));
            k++;
        }
        JsonArray corners = net.getAsJsonArray("corners");
        List<CubeLayout.Corner> mine = layout.corners();
        assertEquals(corners.size(), mine.size());
        k = 0;
        for (JsonElement el : corners) {
            JsonObject r = el.getAsJsonObject();
            CubeLayout.Corner c = mine.get(k);
            String w = "net.corners[" + k + "]";
            text(w + ".name", c.name(), r.get("name"));
            for (int i = 0; i < 3; i++) {
                num(w + ".faces[" + i + "]", c.faces()[i].index(), r.getAsJsonArray("faces").get(i));
                num(w + ".cube[" + i + "]", c.cube()[i], r.getAsJsonArray("cube").get(i));
            }
            JsonArray images = r.getAsJsonArray("images");
            num(w + ".images.length", c.images().length, new com.google.gson.JsonPrimitive(images.size()));
            for (int i = 0; i < images.size(); i++) {
                num(w + ".images[" + i + "][0]", c.images()[i][0], images.get(i).getAsJsonArray().get(0));
                num(w + ".images[" + i + "][1]", c.images()[i][1], images.get(i).getAsJsonArray().get(1));
            }
            k++;
        }
        JsonArray adjacency = net.getAsJsonArray("adjacency");
        assertEquals(6, adjacency.size());
        for (CubeLayout.Face f : CubeLayout.Face.all()) {
            JsonArray row = adjacency.get(f.index()).getAsJsonArray();
            assertEquals(4, row.size());
            for (CubeLayout.FaceSide side : CubeLayout.FaceSide.all()) {
                JsonArray a = row.get(side.index()).getAsJsonArray();
                CubeLayout.Across got = f.across(side);
                String w = "net.adjacency[" + f + "][" + side.id() + "]";
                num(w + ".face", got.face().index(), a.get(0));
                num(w + ".side", got.side().index(), a.get(1));
                num(w + ".quarter_turns", got.quarterTurns(), a.get(2));
                num(w + ".shift[0]", got.shiftU(), a.get(3));
                num(w + ".shift[1]", got.shiftV(), a.get(4));
            }
        }
        k = 0;
        for (JsonElement el : net.getAsJsonArray("blocks")) {
            JsonObject r = el.getAsJsonObject();
            long x = r.get("x").getAsLong(), z = r.get("z").getAsLong();
            String w = "net.blocks[" + k + "] (" + x + ", " + z + ")";
            CubeLayout.Fold fold = layout.fold(x, z);
            String kind = r.get("kind").getAsString();
            switch (kind) {
                case "face" -> {
                    assertTrue(fold instanceof CubeLayout.OnFace, w + ": " + fold);
                    CubeLayout.OnFace on = (CubeLayout.OnFace) fold;
                    num(w + ".face", on.face().index(), r.get("face"));
                    num(w + ".i", on.i(), r.get("i"));
                    num(w + ".j", on.j(), r.get("j"));
                }
                case "outside" -> {
                    assertTrue(fold instanceof CubeLayout.Outside, w + ": " + fold);
                    CubeLayout.Outside out = (CubeLayout.Outside) fold;
                    num(w + ".edge", out.edge(), r.get("edge"));
                    num(w + ".px", out.x(), r.get("px"));
                    num(w + ".pz", out.z(), r.get("pz"));
                    iso(w + ".iso", out.iso(), r.get("iso"));
                }
                case "ocean" -> {
                    assertTrue(fold instanceof CubeLayout.Ocean, w + ": " + fold);
                    cubeChecked++;
                }
                default -> fail(w + ": kind " + kind);
            }
            k++;
        }
        assertTrue(k > 300, "net.blocks: " + k);
        k = 0;
        for (JsonElement el : net.getAsJsonArray("points")) {
            JsonObject r = el.getAsJsonObject();
            double x = f(r.get("x")), z = f(r.get("z"));
            String w = "net.points[" + k + "]";
            CubeLayout.PointOnFace p = layout.foldPoint(x, z);
            if (r.get("face").isJsonNull()) {
                assertNull(p, w + " should be past the net's corners");
                cubeChecked++;
            } else {
                assertNotNull(p, w + " has no fold");
                num(w + ".face", p.face().index(), r.get("face"));
                bits(w + ".u", p.u(), r.get("u"));
                bits(w + ".v", p.v(), r.get("v"));
                bits(w + ".px", p.x(), r.get("px"));
                bits(w + ".pz", p.z(), r.get("pz"));
                iso(w + ".iso", p.iso(), r.get("iso"));
                if (r.get("edge").isJsonNull()) {
                    assertEquals(-1, p.edge(), w + ".edge");
                    cubeChecked++;
                } else {
                    num(w + ".edge", p.edge(), r.get("edge"));
                }
            }
            k++;
        }
        assertTrue(k > 150, "net.points: " + k);

        // M3: the tables.
        JsonArray maps = cube.getAsJsonArray("maps");
        assertEquals(2, maps.size());
        for (JsonElement me : maps) {
            JsonObject mo = me.getAsJsonObject();
            String name = mo.get("name").getAsString();
            CubeMap map = CubeMap.builtin(name);
            assertNotNull(map, "the jar has no table " + name);
            text(name + ".name", map.name(), mo.get("name"));
            text(name + ".sha256", map.sha256Hex(), mo.get("sha256"));
            k = 0;
            for (JsonElement el : mo.getAsJsonArray("points")) {
                JsonObject r = el.getAsJsonObject();
                CubeLayout.Face fc = face(r.get("face"));
                double u = f(r.get("u")), v = f(r.get("v"));
                String w = name + ".points[" + k + "]";
                double[] p = map.forward(fc, u, v);
                bits3(w + ".p", p, r.get("p"));
                CubeMap.FaceJacobian j = map.jacobian(fc, u, v);
                bits3(w + ".jacobian.p", j.p(), r.get("p"));
                bits3(w + ".du", j.du(), r.get("du"));
                bits3(w + ".dv", j.dv(), r.get("dv"));
                k++;
            }
            assertTrue(k > 300, name + ".points: " + k);
            k = 0;
            for (JsonElement el : mo.getAsJsonArray("inverse")) {
                JsonObject r = el.getAsJsonObject();
                JsonArray pa = r.getAsJsonArray("p");
                double[] p = {f(pa.get(0)), f(pa.get(1)), f(pa.get(2))};
                CubeLayout.LocalPoint lp = map.inverse(p);
                String w = name + ".inverse[" + k + "]";
                num(w + ".face", lp.face().index(), r.get("face"));
                bits(w + ".u", lp.u(), r.get("u"));
                bits(w + ".v", lp.v(), r.get("v"));
                k++;
            }
            assertTrue(k > 300, name + ".inverse: " + k);
        }

        // M3: the worlds (turn, map, net together).
        JsonArray worlds = cube.getAsJsonArray("worlds");
        assertEquals(2, worlds.size());
        for (JsonElement we : worlds) {
            JsonObject wo = we.getAsJsonObject();
            int ws = wo.get("s").getAsInt();
            String mapName = wo.get("map").getAsString();
            JsonArray rot = wo.getAsJsonArray("rotation");
            assertEquals(9, rot.size());
            double[][] rows = new double[3][3];
            for (int i = 0; i < 9; i++)
                rows[i / 3][i % 3] = f(rot.get(i));
            CubeTurn turn = CubeTurn.fromRows(rows);
            double[][] back = turn.rows();
            for (int i = 0; i < 9; i++)
                bits("worlds[" + ws + "].rotation[" + i + "]", back[i / 3][i % 3], rot.get(i));
            double mpb = f(wo.get("m_per_block_h"));
            bits("worlds[" + ws + "].m_per_block_h", mpb, wo.get("m_per_block_h"));
            CubeWorld w = new CubeWorld(CubeLayout.of(ws), CubeMap.builtin(mapName), turn, mpb);
            String tag = "worlds[s " + ws + " " + mapName + "]";
            bits(tag + ".radius_m", w.radiusM(), wo.get("radius_m"));
            for (String pole : new String[]{"north_pole", "south_pole"}) {
                CubeLayout.LocalPoint lp = pole.equals("north_pole") ? w.northPole() : w.southPole();
                JsonObject po = wo.getAsJsonObject(pole);
                num(tag + "." + pole + ".face", lp.face().index(), po.get("face"));
                bits(tag + "." + pole + ".u", lp.u(), po.get("u"));
                bits(tag + "." + pole + ".v", lp.v(), po.get("v"));
                double[] xz = w.cube().pointOfLocal(lp.face(), lp.u(), lp.v());
                bits(tag + "." + pole + ".x", xz[0], po.get("x"));
                bits(tag + "." + pole + ".z", xz[1], po.get("z"));
            }
            JsonArray wc = wo.getAsJsonArray("corners");
            List<CubeLayout.Corner> cs = w.cube().corners();
            assertEquals(cs.size(), wc.size());
            for (int i = 0; i < wc.size(); i++) {
                JsonObject r = wc.get(i).getAsJsonObject();
                CubeLayout.Corner c = cs.get(i);
                String cw = tag + ".corners[" + i + "]";
                text(cw + ".name", c.name(), r.get("name"));
                bits3(cw + ".unit", w.cornerUnit(c.cube()), r.get("unit"));
                double[] ll = w.cornerLatLon(c.cube());
                bits(cw + ".lat", ll[0], r.get("lat"));
                bits(cw + ".lon", ll[1], r.get("lon"));
            }
            k = 0;
            int none = 0;
            for (JsonElement el : wo.getAsJsonArray("points")) {
                JsonObject r = el.getAsJsonObject();
                double x = f(r.get("x")), z = f(r.get("z"));
                String pw = tag + ".points[" + k + "]";
                double[] v = w.unit(x, z);
                double[] ll = w.latLon(x, z);
                CubeWorld.WorldJacobian jac = w.jacobian(x, z);
                if (r.get("v").isJsonNull()) {
                    assertNull(v, pw + ".v");
                    assertNull(ll, pw + ".lat_lon");
                    assertNull(jac, pw + ".jacobian");
                    cubeChecked++;
                    none++;
                } else {
                    assertNotNull(v, pw + " has no unit vector");
                    assertNotNull(ll, pw + " has no lat/lon");
                    assertNotNull(jac, pw + " has no Jacobian");
                    bits3(pw + ".v", v, r.get("v"));
                    bits(pw + ".lat", ll[0], r.get("lat"));
                    bits(pw + ".lon", ll[1], r.get("lon"));
                    JsonArray m = r.getAsJsonArray("m");
                    bits(pw + ".m[0]", jac.m()[0][0], m.get(0));
                    bits(pw + ".m[1]", jac.m()[0][1], m.get(1));
                    bits(pw + ".m[2]", jac.m()[1][0], m.get(2));
                    bits(pw + ".m[3]", jac.m()[1][1], m.get(3));
                    bits(pw + ".sigma", jac.sigma(), r.get("sigma"));
                    bits(pw + ".angle", jac.angle(), r.get("angle"));
                }
                k++;
            }
            assertTrue(k > 150, tag + ".points: " + k);
            assertTrue(none > 0, tag + ": no point past the net's corners");
            k = 0;
            for (JsonElement el : wo.getAsJsonArray("inverse")) {
                JsonObject r = el.getAsJsonObject();
                double[] xz = w.pointOfLatLon(f(r.get("lat")), f(r.get("lon")));
                bits(tag + ".inverse[" + k + "].x", xz[0], r.get("x"));
                bits(tag + ".inverse[" + k + "].z", xz[1], r.get("z"));
                k++;
            }
            assertTrue(k > 60, tag + ".inverse: " + k);
        }
        System.out.println("ProjectionParityTest cube section: " + cubeChecked + " fixture values checked bit for bit");
        assertTrue(cubeChecked > 10_000, "the cube section holds " + cubeChecked + " values");
    }

    /** The table loader's checks: a wrong hash, a truncated file and a bad magic are refused; the shipped tables load and are cached. */
    @Test
    void theMapTableLoaderChecksItsInput() throws IOException {
        for (String name : CubeMap.builtinNames()) {
            CubeMap map = CubeMap.builtin(name);
            assertNotNull(map);
            assertSame(map, CubeMap.builtin(name), "builtin tables are loaded once");
            assertEquals(name, map.name());
        }
        assertEquals(CubeMap.WORLD_MAP, CubeMap.world().name());
        assertEquals(CubeMap.SIMULATION_MAP, CubeMap.simulation().name());
        assertEquals("conformal-n513", CubeMap.WORLD_MAP);
        assertEquals("moser-t1-n513", CubeMap.SIMULATION_MAP);
        assertEquals(4, CubeMap.NEWTON_STEPS);
        assertNull(CubeMap.builtin("no-such-table"));
        assertNull(CubeMap.builtin("../conformal-n513"));

        java.nio.file.Path data = Fixtures.repoRoot().resolve("crates/deeptime-format/data");
        byte[] bytes = Files.readAllBytes(data.resolve("cube-map-conformal-n513.bin"));
        String sum = Files.readString(data.resolve("cube-map-conformal-n513.bin.sha256")).strip().split("\\s+")[0];
        CubeMap loaded = CubeMap.loadChecked(bytes, sum.toUpperCase(java.util.Locale.ROOT) + "\n");
        assertEquals(sum, loaded.sha256Hex());
        assertTrue(loaded.cornerPatch() != null, "the conformal table has a corner patch");
        assertEquals(0.0, loaded.tau());
        String wrong = (sum.charAt(0) == '0' ? "1" : "0") + sum.substring(1);
        assertTrue(org.junit.jupiter.api.Assertions.assertThrows(WorldDataException.class,
                () -> CubeMap.loadChecked(bytes, wrong)).getMessage().contains("sha256"));
        assertThrows(WorldDataException.class, () -> CubeMap.load(java.util.Arrays.copyOf(bytes, bytes.length - 16)));
        assertThrows(WorldDataException.class, () -> CubeMap.load(java.util.Arrays.copyOf(bytes, bytes.length + 1)));
        assertThrows(WorldDataException.class, () -> CubeMap.load(new byte[10]));
        byte[] badMagic = bytes.clone();
        badMagic[0] = 'X';
        assertThrows(WorldDataException.class, () -> CubeMap.load(badMagic));
        byte[] badVersion = bytes.clone();
        badVersion[4] = 2;
        assertThrows(WorldDataException.class, () -> CubeMap.load(badVersion));
        // The simulation's table is the equal-area one, without a corner patch.
        CubeMap moser = CubeMap.simulation();
        assertEquals(1.0, moser.tau());
        assertNull(moser.cornerPatch());
    }

    /** The turn's rows: copied in and out, checked (the rotation tests are CubeLayoutTest's). */
    @Test
    void cubeTurnRowsAreCopiedAndChecked() {
        double[][] id = CubeTurn.IDENTITY.rows();
        for (int i = 0; i < 3; i++)
            for (int j = 0; j < 3; j++)
                assertEquals(i == j ? 1.0 : 0.0, id[i][j]);
        assertThrows(WorldDataException.class, () -> CubeTurn.fromRows(new double[][]{{1, 0, 0}, {0, 1, 0}, {0, 0, -1}}));
        assertThrows(WorldDataException.class, () -> CubeTurn.fromRows(new double[][]{{1, 0, 0}, {0, 1, 0}, {0, 0.1, 1}}));
        assertThrows(WorldDataException.class, () -> CubeTurn.fromRows(new double[][]{{1, 0, 0}, {0, 1, 0}}));
        assertThrows(WorldDataException.class, () -> CubeTurn.fromRows(new double[][]{{1, 0, 0}, {0, 1, 0}, {0, Double.NaN, 1}}));
        // The rows are copied in and out.
        double[][] rows = {{1, 0, 0}, {0, 1, 0}, {0, 0, 1}};
        CubeTurn t = CubeTurn.fromRows(rows);
        rows[0][0] = 5.0;
        t.rows()[1][1] = 7.0;
        assertEquals(1.0, t.rows()[0][0]);
        assertEquals(1.0, t.rows()[1][1]);
    }
}
