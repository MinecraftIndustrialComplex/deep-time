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
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

/**
 * The world's projection in Java against the Rust original, bit for bit: every output in
 * {@code crates/deeptime-format/tests/data/projection_parity.json} (written by deeptime-format's
 * {@code parity_fixture} test) comes out of {@link PortableMath} and the cube world
 * ({@link WorldProjection.Cube}, {@link CubeMap}) with the same bits. Plus the layout block's
 * parsing and the map table loader's checks. (The Petroff–Guyou world and its sections went with
 * cube plan §7.4.)
 */
class ProjectionParityTest {

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
        assertEquals(240 * 7, checked, "the fixture's portable trigonometry");
    }

    /**
     * A manifest without the layout block is a Lambert-frame world (the synthetic worlds), and the
     * Petroff–Guyou kind is not read any more (cube plan §7.4).
     */
    @Test
    void noLayoutIsTheLambertFrameAndPetroffGuyouIsRefused() {
        WorldProjection.Lambert l = assertInstanceOf(WorldProjection.Lambert.class, WorldProjection.of(null, 16384));
        assertEquals(16384, l.period());
        assertNull(WorldProjection.Kind.parse("petroff_guyou"));
        String json = """
                {"kind": "petroff_guyou", "layout_version": 1, "hemisphere_blocks": 6656, "lambda_c_deg": 31.25,
                 "domain": {"x_min": -6656.0, "x_max": 6656.0, "z_min": -3328.0, "z_max": 3328.0}, "edges": []}""";
        assertThrows(IllegalArgumentException.class, () -> WorldLayout.fromJson(JsonParser.parseString(json).getAsJsonObject()));
    }

    /** A cube layout block of face side 1,024 (FORMAT.md "The cube world's layout"), map and version as given. */
    private static String cubeLayoutJson(int layoutVersion, String mapName, String sha256) {
        return """
                {"kind": "cube", "layout_version": %d, "face_blocks": 1024,
                 "map": {"name": "%s", "sha256": "%s"},
                 "rotation": [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                 "domain": {"x_min": -1536.0, "x_max": 2560.0, "z_min": -1536.0, "z_max": 1536.0, "period_x": 4096.0},
                 "edges": [{"name": "N-east", "seam": "ne", "kind": "quarter_turn", "axis": "x", "side": "max", "coord": 512,
                            "span": [-1536, -512], "iso": [1, 0, -1024], "pivot": [512, -512], "partner": "F1-top",
                            "owner": "wrap_engine"}],
                 "cone_points": [{"name": "n0", "cone_deg": 270.0, "faces": ["F0", "F1", "N"], "cube": [1, 1, 1],
                                  "lat_deg": 35.26, "lon_deg": 45.0, "reachable": true, "images": [[512, -512]]}],
                 "poles": [{"name": "north_pole", "face": "N", "u": 0.0, "v": 0.0, "images": [[0.0, -1024.0]]}],
                 "border": {"kind": "edge_stop", "inset_blocks": 8.0, "temporary": true, "segments": ["N-east"]}}"""
                .formatted(layoutVersion, mapName, sha256);
    }

    /**
     * The kind "cube" (format 0.7, layout version 2) is a WorldProjection from phase 2: the layout block
     * parses with its seams, cone points, poles and border segments, and the factory builds the cube
     * world the bake wrote, refusing a map this jar does not ship (Rust's message) and a polar layout
     * version.
     */
    @Test
    void theCubeKindParsesAndBuildsTheCubeWorld() {
        assertEquals(WorldProjection.Kind.CUBE, WorldProjection.Kind.parse("cube"));
        assertEquals("cube", WorldProjection.Kind.CUBE.id());
        // Every kind still parses from its own id.
        for (WorldProjection.Kind k : WorldProjection.Kind.values())
            assertEquals(k, WorldProjection.Kind.parse(k.id()));
        String sha = CubeMap.world().sha256Hex();
        WorldLayout layout = WorldLayout.fromJson(JsonParser.parseString(cubeLayoutJson(2, CubeMap.WORLD_MAP, sha)).getAsJsonObject());
        assertEquals(WorldProjection.Kind.CUBE, layout.kind());
        assertEquals(2, layout.layoutVersion());
        assertEquals(1024, layout.cube().faceBlocks());
        assertEquals(CubeMap.WORLD_MAP, layout.cube().mapName());
        WorldLayout.Edge ne = layout.edge("N-east");
        assertEquals("ne", ne.seam());
        assertEquals("F1-top", ne.partner());
        assertEquals(1, ne.quarterTurns());
        assertTrue(layout.border().stops("N-east"));
        assertFalse(layout.border().stops("F3-west"));
        WorldLayout.ConePoint n0 = layout.conePoints().get(0);
        assertTrue(n0.reachable());
        assertEquals(List.of("F0", "F1", "N"), n0.faces());
        assertTrue(Double.isNaN(n0.landDistanceDeg()));
        assertEquals("N", layout.poles().get(0).face());
        assertEquals(-1024.0, layout.poles().get(0).images().get(0)[1]);

        WorldProjection p = WorldProjection.of(layout, 4096);
        WorldProjection.Cube cube = assertInstanceOf(WorldProjection.Cube.class, p);
        assertEquals(1024, cube.side());
        assertEquals(4096.0, p.period());
        assertEquals(-1536.0, p.zMin());
        assertEquals(1536.0, p.zMax());
        assertEquals(-1536, p.indexOriginBlockX());
        assertEquals(3072, p.indexSpanBlockZ());
        assertEquals(1024.0 * Math.sqrt(1.5 / Math.PI), p.radius());
        // x wraps into the window, not into [−C/2, C/2).
        assertEquals(2000.0, p.wrapX(2000.0));
        assertEquals(2000.0, p.wrapX(2000.0 - 4096.0));
        GridDesc g = p.grid("L3", 4);
        assertTrue(g.isCube());
        assertEquals(256, g.w());
        assertEquals(6 * 256, g.h());
        assertEquals(4096, g.circumference());
        // Past the net's corners (beyond F1 and N both, more than a face out) there is no planet.
        assertTrue(Double.isNaN(p.latLon(1536.0 + 1100.0, -1536.0 - 1100.0)[0]));
        // The identity turn puts F0's centre at latitude 0, longitude 0.
        double[] ll = p.latLon(0.0, 0.0);
        assertEquals(0.0, ll[0], 1e-9);

        // A layout version 1 cube is a polar layout's number, refused.
        IllegalArgumentException v1 = assertThrows(IllegalArgumentException.class,
                () -> WorldLayout.fromJson(JsonParser.parseString(cubeLayoutJson(1, CubeMap.WORLD_MAP, sha)).getAsJsonObject()));
        assertTrue(v1.getMessage().contains("layout_version 1"), v1.getMessage());
        // A map this jar does not ship, by name or by sha256: Rust's refusal.
        for (String[] bad : new String[][]{{"conformal-n1025", sha}, {CubeMap.WORLD_MAP, "00" + sha.substring(2)}}) {
            WorldLayout other = WorldLayout.fromJson(JsonParser.parseString(cubeLayoutJson(2, bad[0], bad[1])).getAsJsonObject());
            WorldDataException e = assertThrows(WorldDataException.class, () -> WorldProjection.of(other, 4096));
            assertTrue(e.getMessage().startsWith("world data needs cube map " + bad[0] + " (sha256 " + bad[1]
                    + "); this Deep Time has "), e.getMessage());
            assertTrue(e.getMessage().contains(CubeMap.WORLD_MAP + " (sha256 " + sha + ")"), e.getMessage());
        }
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
