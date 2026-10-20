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
import java.util.SplittableRandom;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
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
        assertNull(WorldProjection.Kind.parse("cube"));
        assertEquals(6656, PetroffGuyou.hemisphereFor(16384));
        assertEquals(13056, PetroffGuyou.hemisphereFor(32768));
        assertEquals(1536, PetroffGuyou.hemisphereFor(4096));
    }
}
