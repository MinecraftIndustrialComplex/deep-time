// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeLayout;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeMap;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeTurn;
import com.minecraftindustrialcomplex.deeptime.worlddata.CubeWorld;
import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;
import com.minecraftindustrialcomplex.deeptime.worldgen.VolcanicZones.Role;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Where the volcanic biomes go ({@link VolcanicZones}; decisions log 2026-10-03): the roles' discs around
 * volcanoes, craters, calderas, pools and magma, built from closed-form inputs. Properties, not one
 * planet's numbers. The old placement had no zones at all (the biomes fell wherever their climate
 * parameters did), so every test here fails on it.
 */
class VolcanicZonesTest {

    private static final int C = 8192;
    private static final double MH = 300.0;

    /** A volcano of basal radius {@code rBlocks} blocks at (x, z). */
    private static Volcanoes.Volcano volcano(int id, String kind, double x, double z, double rBlocks, double activity, double craterBlocks,
                                             boolean caldera) {
        return new Volcanoes.Volcano(id, kind, x, z, rBlocks * MH, 2000, activity, false, false, craterBlocks * MH, 150, caldera);
    }

    private static Geothermal.Site pool(int id, int x, int z, int r) {
        return new Geothermal.Site(id, Geothermal.Kind.POOL, 1, x, z, r, r + 4, 100, 100, true, false, 7L, 0.0, 0.0);
    }

    private static Magma.Body body(int id, int x, int z, int rh) {
        return new Magma.Body(id, "stratovolcano", 0.5, x, z, 100, 40, rh, 6, 90, 94, false, false, 0, 0, 0, rh + 2, null, null);
    }

    private static boolean has(int bits, Role r) {
        return (bits & (1 << r.ordinal())) != 0;
    }

    /** The highest-priority surface role at a column, or null. */
    private static Role first(VolcanicZones z, int x, int zz) {
        int bits = z.rolesAt(x, zz);
        for (Role r : Role.values())
            if (!r.cave && has(bits, r))
                return r;
        return null;
    }

    @Test
    void anActiveConeHasItsCraterConeAndAshFieldAndNothingBeyond() {
        var v = volcano(1, "stratovolcano", 1000, 500, 80, 0.8, 6, false);
        var z = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CRATER, first(z, 1000, 500));
        assertEquals(Role.CRATER, first(z, 1000 + 9, 500));          // the crater's rim plus its margin (6 + 4)
        assertEquals(Role.CONE, first(z, 1000 + 30, 500));           // 0.45 R = 36
        assertEquals(Role.ASH_FIELD, first(z, 1000, 500 + 70));      // out to R = 80
        assertEquals(null, first(z, 1000 + 90, 500));
        assertTrue(z.outsideSurfaceZones(1000 + 90, 500) > 9.0 && z.outsideSurfaceZones(1000 + 90, 500) < 11.0);
        assertTrue(z.outsideSurfaceZones(1000, 500) <= 0.0);
    }

    @Test
    void aQuietConeHasNoAshField() {
        var v = volcano(1, "stratovolcano", 0, 0, 80, 0.2, 0, false);
        var z = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CONE, first(z, 0, 0));                     // no crater: the cone reaches the summit
        assertEquals(null, first(z, 60, 0));
    }

    @Test
    void aShieldHasALavaFieldAndACalderaComplexHasItsFloorAndAsh() {
        var shield = volcano(1, "shield", -2000, 0, 100, 0.9, 0, false);
        var centre = volcano(2, "caldera_complex", 2000, 0, 60, 0.9, 20, true);
        var z = VolcanicZones.build(List.of(shield, centre), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CONE, first(z, -2000 + 40, 0));
        assertEquals(Role.LAVA_FIELD, first(z, -2000 + 80, 0));
        assertEquals(Role.CALDERA, first(z, 2000, 15));              // 0.9 of a 20-block rim = 18
        assertEquals(Role.ASH_FIELD, first(z, 2000, 40));            // a centre has no cone: the ash field reaches in
        assertEquals(null, first(z, 2000, 70));
    }

    @Test
    void aHotSpringFieldWinsOverTheConeAndTheCaldera() {
        var v = volcano(1, "caldera_complex", 0, 0, 60, 0.9, 25, true);
        var z = VolcanicZones.build(List.of(v), List.of(pool(5, 10, 0, 3)), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        int bits = z.rolesAt(10, 0);
        assertTrue(has(bits, Role.CALDERA) && has(bits, Role.HOT_SPRING_FIELD));
        assertEquals(Role.HOT_SPRING_FIELD, first(z, 10, 0));
        // Every column of the pool's own reach is in its field.
        var s = pool(5, 10, 0, 3);
        for (int dx = -s.reach(); dx <= s.reach(); dx++)
            assertEquals(Role.HOT_SPRING_FIELD, first(z, 10 + dx, 0));
    }

    @Test
    void zonesAreNeverSmallerThanTwoBiomeCells() {
        var tiny = volcano(1, "stratovolcano", 0, 0, 6, 0.9, 1, false);
        var z = VolcanicZones.build(List.of(tiny), List.of(pool(2, 500, 500, 2)), List.of(body(3, -500, -500, 3)), VolcanicBiomeRules.DEFAULT,
                MH, C);
        for (VolcanicZones.Zone zn : z.zones())
            assertTrue(zn.radius() >= VolcanicZones.MIN_RADIUS, zn.toString());
        // A field ring narrower than MIN_RING beyond the cone is left out: R 6 gives no ash field.
        assertTrue(z.zones().stream().noneMatch(zn -> zn.role() == Role.ASH_FIELD));
    }

    @Test
    void zonesWrapAcrossTheSeam() {
        var v = volcano(1, "stratovolcano", C / 2.0 - 5, 0, 80, 0.8, 0, false);
        var z = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(Role.CONE, first(z, -C / 2 + 10, 0));
        assertEquals(Role.ASH_FIELD, first(z, -C / 2 + 60, 0));
        assertTrue(z.outsideSurfaceZones(-C / 2 + 10, 0) <= 0.0);
    }

    @Test
    void thermalCavesFollowTheMagma() {
        var z = VolcanicZones.build(List.of(), List.of(), List.of(body(1, 300, 300, 20), body(2, 900, 900, 0)), VolcanicBiomeRules.DEFAULT,
                MH, C);
        assertTrue(has(z.rolesAt(300 + 29, 300), Role.THERMAL_CAVES));     // 1.5 x 20
        assertEquals(0, z.rolesAt(300 + 32, 300));
        assertEquals(0, z.rolesAt(900, 900));                               // no chamber, no caves
        assertEquals(null, first(z, 300, 300));                             // a cave role is not a surface role
    }

    @Test
    void theReachSettingScalesTheFlanksAndZeroPlacesNothing() {
        var v = volcano(1, "stratovolcano", 0, 0, 80, 0.8, 0, false);
        var wide = VolcanicZones.build(List.of(v), List.of(), List.of(), new VolcanicBiomeRules(2.0), MH, C);
        assertEquals(Role.CONE, first(wide, 70, 0));                 // 0.45 x 80 x 2 = 72
        assertEquals(Role.ASH_FIELD, first(wide, 150, 0));
        var none = VolcanicZones.build(List.of(v), List.of(pool(2, 0, 0, 3)), List.of(body(3, 0, 0, 20)), new VolcanicBiomeRules(0.0), MH,
                C);
        assertTrue(none.zones().isEmpty());
        assertEquals(0, none.rolesAt(0, 0));
    }

    /**
     * A cone's zones on world {@code p} at (x, z) are its circle on the planet (ledger SCL-230, BIO-182): measured
     * along 72 rays by the great-circle distance from the vent in planet blocks (the planet's radius in
     * {@code radius()} blocks times the angle between the unit vectors), the crater and cone hold 0.45 R, the ash
     * field reaches R and nothing lies beyond. Returns the cone zone.
     */
    private static VolcanicZones.Zone assertRoundOnThePlanet(WorldProjection p, double x, double z) {
        var v = volcano(1, "stratovolcano", x, z, 80, 0.8, 6, false);
        var zs = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, (int) p.period(), p);
        VolcanicZones.Zone cone = zs.zones().stream().filter(zn -> zn.role() == Role.CONE).findFirst().orElseThrow();
        int vx = (int) Math.round(x), vz = (int) Math.round(z);
        double rr = p.radius();
        double[] u0 = p.unit(vx, vz);
        int[] seen = new int[3];
        for (int k = 0; k < 72; k++) {
            double th = k * Math.PI / 36.0;
            for (int d = 1; d < 5000; d++) {
                int bx = vx + (int) Math.round(d * Math.cos(th)), bz = vz + (int) Math.round(d * Math.sin(th));
                double[] u = p.unit(bx, bz);
                double cx = u[1] * u0[2] - u[2] * u0[1], cy = u[2] * u0[0] - u[0] * u0[2], cz = u[0] * u0[1] - u[1] * u0[0];
                double f = rr * Math.atan2(Math.sqrt(cx * cx + cy * cy + cz * cz), u[0] * u0[0] + u[1] * u0[1] + u[2] * u0[2]);
                Role r = first(zs, bx, bz);
                String where = "ray " + k + ", " + d + " blocks out, " + f + " planet blocks; " + cone;
                if (f < 0.9 * 36) {                                    // 0.45 R = 36
                    assertTrue(r == Role.CRATER || r == Role.CONE, where + ": " + r);
                    seen[0]++;
                } else if (f > 1.1 * 36 && f < 0.9 * 80) {             // out to R = 80
                    assertEquals(Role.ASH_FIELD, r, where);
                    seen[1]++;
                } else if (f > 1.1 * 80) {
                    assertEquals(null, r, where);
                    assertTrue(zs.outsideSurfaceZones(bx, bz) > 0.0, where);
                    seen[2]++;
                    break;
                }
            }
        }
        assertTrue(seen[0] > 100 && seen[1] > 100 && seen[2] == 72, java.util.Arrays.toString(seen));
        return cone;
    }

    /**
     * On the cube world (a turned 16k cube, a vent well inside a band face) a cone's zones are its circle on the
     * planet, which the conformal map draws round: the cone zone's long over short axis is the map's stretch at
     * the vent, about 1. (Until cube plan §7.4 this was the Petroff–Guyou world's test, whose equal-area map drew
     * the circle 1.78 : 1 at the median.)
     */
    @Test
    void onTheCubeWorldTheZonesAreTheConesCircleOnThePlanet() {
        var cube = new WorldProjection.Cube(new CubeWorld(CubeLayout.of(3840), CubeMap.world(), CubeTurn.polar(30.0), 1.0));
        double[] at = cube.cube().pointOfLocal(CubeLayout.Face.F1, 0.3, -0.2);
        VolcanicZones.Zone cone = assertRoundOnThePlanet(cube, at[0], at[1]);
        VolcanicZones.Stretch st = VolcanicZones.stretch(cube, at[0], at[1]);
        double want = st.major() / st.minor();
        assertEquals(want, cone.radius() / cone.minor(), 0.02 * want, cone + " " + st);
        assertEquals(1.0, want, 0.02, "the conformal cube draws a circle round: " + st);
    }

    /**
     * Off a cube world's faces a query reads the partner column the void shows ({@code CubeFill.foldedBiome} reads
     * the biome there), never the net-plane part of a disc past a turning edge: a cone 8 blocks inside N's west
     * edge reaches into the void on the net, but the void there shows F3's top, which holds no zone; a second cone
     * put where a void column's partner lies is read at that void column. Before the fold the void held the first
     * cone's roles.
     */
    @Test
    void offACubesFacesAQueryReadsThePartnerColumn() {
        CubeLayout net = CubeLayout.of(1024);
        int c = (int) net.period();
        var proj = new WorldProjection.Cube(new CubeWorld(net, CubeMap.world(), CubeTurn.IDENTITY, 1.0));
        int west = -512, zv = -1000;
        var nearEdge = volcano(1, "stratovolcano", west + 8, zv, 80, 0.8, 6, false);
        var one = VolcanicZones.build(List.of(nearEdge), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, c, proj);
        assertTrue(net.faceOfBlock(west + 8, zv) != null && net.faceOfBlock(west - 4, zv) == null, "the cone sits by N's west edge");
        assertTrue(has(one.rolesAt(west + 8, zv), Role.CRATER), "its own face holds it");
        var flat = VolcanicZones.build(List.of(nearEdge), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, c);
        assertTrue(has(flat.rolesAt(west - 4, zv), Role.CONE), "the net-plane disc reaches past the edge");
        for (int dx = 1; dx <= 40; dx++)
            assertEquals(0, one.rolesAt(west - dx, zv), "the void " + dx + " blocks past the edge shows F3's top, no zone");
        assertTrue(one.outsideSurfaceZones(west - 4, zv) > 40, "and lies far from every zone on the planet");
        CubeLayout.Fold f = net.fold(west - 10, zv);
        assertTrue(f instanceof CubeLayout.Outside, "a band column: " + f);
        var o = (CubeLayout.Outside) f;
        var partner = volcano(2, "stratovolcano", o.x(), o.z(), 80, 0.8, 6, false);
        var two = VolcanicZones.build(List.of(nearEdge, partner), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, c, proj);
        assertTrue(has(two.rolesAt(west - 10, zv), Role.CRATER), "the void reads the partner's crater");
        for (int dz = -30; dz <= 30; dz += 3)
            for (int dx = 1; dx <= 30; dx += 3)
                if (net.fold(west - dx, zv + dz) instanceof CubeLayout.Outside p)
                    assertEquals(two.rolesAt((int) p.x(), (int) p.z()), two.rolesAt(west - dx, zv + dz));
    }

    /**
     * On a Lambert world the zones are the planet's circles as Lambert draws them: at 55° wide east–west by
     * 1/cos φ and narrow north–south by cos φ (the cone's 36 blocks are 62.8 by 20.6); at the equator discs, as a
     * world without a projection has. The old zones (discs on Lambert) fail it.
     */
    @Test
    void onALambertWorldTheZonesAreThePlanetsCirclesWideEastWest() {
        WorldProjection lambert = WorldProjection.of(null, C);
        double z55 = -lambert.radius() * Math.sin(Math.toRadians(55.0));
        VolcanicZones.Zone cone = assertRoundOnThePlanet(lambert, 1000, z55);
        double cos = Math.cos(Math.toRadians(55.0));
        assertEquals(36.0 / cos, cone.radius(), 0.5, cone.toString());
        assertEquals(36.0 * cos, cone.minor(), 0.5, cone.toString());
        assertEquals(1.0, Math.abs(cone.ax()), 1e-6, cone.toString());
        var v = volcano(1, "stratovolcano", 1000, 0, 80, 0.8, 6, false);
        var c = volcano(2, "caldera_complex", -1000, 0, 60, 0.9, 20, true);
        var plain = VolcanicZones.build(List.of(v, c), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        var equator = VolcanicZones.build(List.of(v, c), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C, lambert);
        assertEquals(plain.zones().size(), equator.zones().size());
        for (int i = 0; i < plain.zones().size(); i++) {
            VolcanicZones.Zone a = plain.zones().get(i), b = equator.zones().get(i);
            assertEquals(a.role(), b.role());
            assertEquals(a.radius(), b.radius(), 1e-3, b.toString());
            assertEquals(a.minor(), b.minor(), 1e-3, b.toString());
        }
    }

    /**
     * An elongated zone's {@code outside} is the distance to its nearest edge point: checked against the edge sampled
     * at 20,000 points, all round a 3:1 ellipse at 30 degrees, outside and inside.
     */
    @Test
    void anEllipsesOutsideIsTheDistanceToItsNearestEdgePoint() {
        double a = 60, b = 20, ax = Math.cos(Math.PI / 6), az = Math.sin(Math.PI / 6);
        var zn = new VolcanicZones.Zone(Role.CONE, 1, 0, 0, a, b, ax, az);
        int n = 20000;
        double[][] edge = new double[n][];
        for (int i = 0; i < n; i++) {
            double th = 2 * Math.PI * i / n, p = a * Math.cos(th), q = b * Math.sin(th);
            edge[i] = new double[]{p * ax - q * az, p * az + q * ax};
        }
        int outsideChecked = 0;
        for (int k = 0; k < 48; k++) {
            double th = k * Math.PI / 24;
            for (double r : new double[]{5, 15, 25, 40, 70, 90, 150}) {
                double dx = r * Math.cos(th), dz = r * Math.sin(th);
                double d = zn.outside(dx, dz);
                if (!zn.holds(dx, dz)) {
                    double best = Double.POSITIVE_INFINITY;
                    for (double[] e : edge)
                        best = Math.min(best, Math.hypot(dx - e[0], dz - e[1]));
                    assertEquals(best, d, 0.05, "offset " + dx + ", " + dz);
                    assertTrue(d >= Math.hypot(dx, dz) - a - 1e-9);
                    outsideChecked++;
                } else {
                    assertTrue(d <= 0.0, "inside at " + dx + ", " + dz + ": " + d);
                }
            }
        }
        assertTrue(outsideChecked > 150, "points outside: " + outsideChecked);
        // On the axes the old measure (along the line from the centre) was already exact.
        assertEquals(10.0, zn.outside((a + 10) * ax, (a + 10) * az), 1e-6);
        assertEquals(10.0, zn.outside(-(b + 10) * az, (b + 10) * ax), 1e-6);
    }

    private static final double MV = 35.0;
    private static final int SEA = 63;

    /** A shield of basal radius {@code rBlocks} blocks and height {@code hM} metres, on oceanic crust or not. */
    private static Volcanoes.Volcano shield(int id, double x, double z, double rBlocks, double hM, boolean oceanic) {
        return new Volcanoes.Volcano(id, "shield", x, z, rBlocks * MH, hM, 0.9, oceanic, false, 0, 0, false);
    }

    /** The ground: {@code ventAbove} blocks over the sea at (x, z) (the vent), the sea floor elsewhere. */
    private static VolcanicZones.Ground ground(double x, double z, double ventAbove) {
        return new VolcanicZones.Ground((bx, bz) -> bx == (int) Math.round(x) && bz == (int) Math.round(z) ? SEA + ventAbove : SEA - 100,
                SEA, MV);
    }

    private static VolcanicZones.Zone zone(VolcanicZones z, Role r) {
        return z.zones().stream().filter(zn -> zn.role() == r).findFirst().orElseThrow();
    }

    /**
     * An oceanic shield's zones are sized on the part of the edifice above the sea (BIO-198), not on its basal radius
     * on the sea floor: a shield standing 10.8 blocks above the sea at its vent, 3,000 m over a 220-block foot, has its
     * coast where its profile (1 - rho^2)^1.5 falls to the sea, 64 blocks out, and its cone zone (0.45 of that) lies
     * inside it. Sized on the foot the cone zone reached 99 blocks, past the coast.
     */
    @Test
    void anOceanicShieldsConeZoneLiesInsideItsCoast() {
        var v = shield(228, -531, 536, 220, 3000, true);
        var z = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C, null, ground(-531, 536, 10.8));
        // The coast, from the profile itself: the radius at which the edifice (3,000 m = h blocks, the vent 10.8 above the sea)
        // stands at the sea.
        double h = 3000.0 / MV, f = (h - 10.8) / h;
        double coast = 220 * Math.sqrt(1.0 - Math.pow(f, 2.0 / 3.0));
        assertEquals(f, Math.pow(1.0 - Math.pow(coast / 220, 2), 1.5), 1e-9);
        assertEquals(64.0, coast, 1.0);
        assertEquals(0.45 * coast, zone(z, Role.CONE).radius(), 1e-9);
        assertTrue(zone(z, Role.CONE).radius() < coast);
        assertEquals(coast, zone(z, Role.LAVA_FIELD).radius(), 1e-9);
        assertEquals(Role.CONE, first(z, -531 + 25, 536));
        assertEquals(Role.LAVA_FIELD, first(z, -531 + 50, 536));
        assertEquals(null, first(z, -531 + 70, 536));                // beyond the subaerial radius: the climate's own biome
        // Without the ground: its basal radius, as before.
        var old = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        assertEquals(0.45 * 220, zone(old, Role.CONE).radius(), 1e-9);
    }

    /** A continental volcano stands on land: its zones are the same with or without the ground, whatever the ground says. */
    @Test
    void aContinentalVolcanoKeepsItsBasalZones() {
        var v = shield(7, 100, 100, 150, 3000, false);
        var plain = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C);
        var withGround = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C, null, ground(100, 100, 10.8));
        assertEquals(plain.zones(), withGround.zones());
        assertEquals(0.45 * 150, zone(withGround, Role.CONE).radius(), 1e-9);
    }

    /** A vent above the whole edifice gives the foot, and a vent at or under the sea has no subaerial part. */
    @Test
    void theSubaerialRadiusIsBetweenNothingAndTheFoot() {
        var v = shield(1, 0, 0, 100, 1000, true);
        var tall = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C, null, ground(0, 0, 500));
        assertEquals(0.45 * 100, zone(tall, Role.CONE).radius(), 1e-9);
        var drowned = VolcanicZones.build(List.of(v), List.of(), List.of(), VolcanicBiomeRules.DEFAULT, MH, C, null, ground(0, 0, -2));
        assertEquals(VolcanicZones.MIN_RADIUS, zone(drowned, Role.CONE).radius(), 1e-9);
        assertTrue(drowned.zones().stream().noneMatch(zn -> zn.role() == Role.LAVA_FIELD));
    }

    @Test
    void theSettingIsReadFromTheBakeNotes() {
        var notes = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"volcanic_biome_reach\": 1.5}}}}").getAsJsonObject();
        assertEquals(1.5, VolcanicBiomeRules.of(notes).reach());
        var high = JsonParser.parseString("{\"bake\": {\"underground\": {\"params\": {\"volcanic_biome_reach\": 9}}}}").getAsJsonObject();
        assertEquals(3.0, VolcanicBiomeRules.of(high).reach());
        var none = JsonParser.parseString("{\"bake\": {}}").getAsJsonObject();
        assertEquals(VolcanicBiomeRules.DEFAULT, VolcanicBiomeRules.of(none));
    }
}
