// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.sable.SableTestOps;
import com.minecraftindustrialcomplex.deeptime.net.PoleStormPayload;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCap;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.entity.vehicle.Minecart;
import net.minecraft.world.level.GameType;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;

/**
 * The pole soft cap (plan §7.11, P1-18) on the dev planet server, both hemispheres, in clear air
 * near the top of the world (a probe drifting at surface height can hit rising ground, P0-15).
 *
 * <p>Each test loads its own strips of chunks at the poles (forced chunks, released at the end;
 * TW's {@code /forceload} is not needed at x ≈ 0), so the harness only has to put the test itself
 * somewhere loaded: {@code --run entitypushedback,hardstopat80,sablehullpushedback@0,300,64}.
 *
 * <p>Players are {@link MockPlayers}: real server players on an in-memory connection, so the tests
 * read exactly what the server sends them (the storm payload with its wind, the action bar) and see
 * the server's hard stop. A player's own movement is its client's (see {@link PoleCap}), so the
 * physical pushback is shown on the vehicles the server moves itself (minecarts carrying a mock
 * player) and on Sable hulls.
 */
@GameTestHolder(GameTestSupport.NS_PLANET)
@PrefixGameTestTemplate(false)
public final class PoleCapGameTests {

    /** Latitudes of the players and carts that must feel the cap; the first is below it. */
    private static final double[] LATITUDES = {69.5, 76.0, 77.5, 79.0};
    private static final int RUN_TICKS = 40;
    /** How long a test may wait for its forced chunks. */
    private static final int LOAD_TICKS = 600;

    private PoleCapGameTests() {}

    // ---------------------------------------------------------------- shared set-up

    /** A test's position and cap, and the chunks it forced. */
    private static final class Rig {
        final GameTestHelper helper;
        final ServerLevel level;
        final PoleCapCurve cap;
        final int c, x;
        final double y;
        final List<long[]> forced = new ArrayList<>();
        final List<BlockPos> spots = new ArrayList<>();
        final List<MockPlayers.Mock> mocks = new ArrayList<>();
        final List<Entity> entities = new ArrayList<>();
        final List<UUID> hulls = new ArrayList<>();

        Rig(GameTestHelper helper) {
            this.helper = helper;
            this.level = helper.getLevel();
            this.c = SeamGameTests.circumference(helper);
            this.cap = PoleCap.capOf(level);
            BlockPos spot = SeamGameTests.surfaceSpot(helper);
            this.x = spot.getX();
            this.y = level.getMaxBuildHeight() - 20.5;
        }

        double z(double latDeg) {
            return PoleCapCurve.zOfLatitude(c, latDeg);
        }

        double lat(double z) {
            return cap.latitudeDeg(z);
        }

        /** Forces the chunks from latitude |a| to |b| (same sign) over x … x + width. */
        void force(double latA, double latB, int width) {
            double z0 = Math.min(z(latA), z(latB)) - 16, z1 = Math.max(z(latA), z(latB)) + 16;
            for (int cx = x >> 4; cx <= (x + width) >> 4; cx++)
                for (int cz = (int) Math.floor(z0) >> 4; cz <= (int) Math.floor(z1) >> 4; cz++) {
                    level.setChunkForced(cx, cz, true);
                    forced.add(new long[]{cx, cz});
                }
        }

        /** True once every registered spot is entity-ticking. */
        boolean ready() {
            for (BlockPos p : spots)
                if (!level.isPositionEntityTicking(p))
                    return false;
            return true;
        }

        BlockPos spot(double dx, double latDeg) {
            BlockPos p = BlockPos.containing(x + dx, y, z(latDeg));
            spots.add(p);
            return p;
        }

        MockPlayers.Mock player(GameType mode, double dx, double latDeg) {
            MockPlayers.Mock m = MockPlayers.create(level, mode);
            mocks.add(m);
            m.player.teleportTo(level, x + dx + 0.5, y, z(latDeg), 0, 0);
            return m;
        }

        /** A gravity-free minecart at rest with a survival mock player in it. */
        Minecart cart(double dx, double latDeg, MockPlayers.Mock[] riderOut) {
            Minecart cart = EntityType.MINECART.create(level);
            cart.setNoGravity(true);
            cart.moveTo(x + dx + 0.5, y, z(latDeg), 0, 0);
            cart.setDeltaMovement(Vec3.ZERO);
            level.addFreshEntity(cart);
            entities.add(cart);
            MockPlayers.Mock rider = player(GameType.SURVIVAL, dx, latDeg);
            rider.player.startRiding(cart, true);
            riderOut[0] = rider;
            return cart;
        }

        void cleanUp() {
            MockPlayers.removeAll(mocks);
            for (Entity e : entities)
                if (!e.isRemoved())
                    e.discard();
            if (!hulls.isEmpty() && ModList.get().isLoaded(SableTestOps.MOD_ID))
                for (UUID id : hulls)
                    SableTestOps.remove(level, id);
            for (long[] f : forced)
                level.setChunkForced((int) f[0], (int) f[1], false);
        }
    }

    private static String f(double v) {
        return String.format(Locale.ROOT, "%.4f", v);
    }

    // ---------------------------------------------------------------- entityPushedBack

    /**
     * No effect below the cap line (70° by default), then a pushback toward the equator that grows with latitude, in both
     * hemispheres:
     * <ul>
     *   <li>survival players at 69.5°, 76°, 77.5° and 79° N and S are sent the storm and the wind
     *       of {@link PoleCapCurve} for their latitude (none at 69.5°), increasing with latitude,
     *       and an action-bar warning inside the band;</li>
     *   <li>a creative player at 77.5° N gets the storm and the warning but no wind; a spectator at
     *       77.5° S the storm only;</li>
     *   <li>gravity-free minecarts carrying a survival player at 69.5°, 76°, 77.5° and 79° S and
     *       77.5° N move toward the equator in 40 ticks, further the higher they start (the one at
     *       69.5° does not move), and their riders get no wind of their own (the server moves the cart).</li>
     * </ul>
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void entityPushedBack(GameTestHelper helper) {
        Rig rig = new Rig(helper);
        rig.force(-69, -79.5, 48);
        rig.force(69, 79.5, 48);
        Map<String, MockPlayers.Mock> players = new LinkedHashMap<>();
        Map<String, Minecart> carts = new LinkedHashMap<>();
        Map<String, MockPlayers.Mock> riders = new LinkedHashMap<>();
        Map<String, Double> cartZ0 = new LinkedHashMap<>();
        for (double lat : LATITUDES) {
            rig.spot(0, lat);
            rig.spot(0, -lat);
        }
        long[] startedAt = {-1};
        boolean[] done = {false};
        helper.onEachTick(() -> {
            if (done[0])
                return;
            long t = helper.getTick();
            if (startedAt[0] < 0) {
                if (!rig.ready()) {
                    if (t > LOAD_TICKS) {
                        done[0] = true;
                        rig.cleanUp();
                        helper.fail("the pole strips never became entity-ticking");
                    }
                    return;
                }
                int dx = 0;
                for (double lat : LATITUDES)
                    for (int sign : new int[]{1, -1}) {
                        String name = PoleCap.formatLatitude(sign * lat);
                        players.put(name, rig.player(GameType.SURVIVAL, dx, sign * lat));
                        dx += 2;
                    }
                players.put("creative 77.5° N", rig.player(GameType.CREATIVE, dx, 77.5));
                players.put("spectator 77.5° S", rig.player(GameType.SPECTATOR, dx + 2, -77.5));
                int cx = 24;
                for (double lat : new double[]{-69.5, -76.0, -77.5, -79.0, 77.5}) {
                    String name = PoleCap.formatLatitude(lat);
                    MockPlayers.Mock[] rider = new MockPlayers.Mock[1];
                    Minecart cart = rig.cart(cx, lat, rider);
                    carts.put(name, cart);
                    riders.put(name, rider[0]);
                    cartZ0.put(name, cart.getZ());
                    cx += 4;
                }
                startedAt[0] = t;
                return;
            }
            if (t - startedAt[0] < RUN_TICKS)
                return;
            done[0] = true;
            try {
                check(helper, rig, players, carts, riders, cartZ0);
            } finally {
                rig.cleanUp();
            }
        });
    }

    private static void check(GameTestHelper helper, Rig rig, Map<String, MockPlayers.Mock> players,
                              Map<String, Minecart> carts, Map<String, MockPlayers.Mock> riders, Map<String, Double> cartZ0) {
        PoleCapCurve cap = rig.cap;
        StringBuilder d = new StringBuilder();
        List<String> problems = new ArrayList<>();
        // Players standing at each latitude (they do not move: no client).
        for (int sign : new int[]{1, -1}) {
            double lastPush = -1, lastStorm = -1;
            for (double lat : LATITUDES) {
                String name = PoleCap.formatLatitude(sign * lat);
                MockPlayers.Mock m = players.get(name);
                double z = m.player.getZ();
                PoleStormPayload s = m.last(PoleStormPayload.class);
                float storm = s == null ? 0f : s.storm(), push = s == null ? 0f : s.push();
                boolean bar = m.sawActionBar(PoleCap.KEY_STORM);
                d.append(String.format(Locale.ROOT, " %s:storm=%s,push=%s,bar=%s", name, f(storm), f(push), bar));
                if (lat < cap.capDeg()) {
                    if (storm != 0 || push != 0 || bar)
                        problems.add(name + " is below the cap but got storm=" + storm + " push=" + push + " bar=" + bar);
                    continue;
                }
                if (Math.abs(storm - cap.storm(z)) > 1e-4 || Math.abs(push - cap.push(z)) > 1e-5)
                    problems.add(name + ": sent storm/push " + storm + "/" + push + ", the curve says " + cap.storm(z) + "/" + cap.push(z));
                if (!(push > lastPush && storm > lastStorm))
                    problems.add(name + ": the wind/storm does not grow with latitude (" + push + " after " + lastPush + ")");
                if (!bar)
                    problems.add(name + ": no action-bar warning");
                lastPush = push;
                lastStorm = storm;
            }
        }
        MockPlayers.Mock creative = players.get("creative 77.5° N");
        PoleStormPayload cs = creative.last(PoleStormPayload.class);
        d.append(" creative:").append(cs).append(",bar=").append(creative.sawActionBar(PoleCap.KEY_STORM_NO_WIND));
        if (cs == null || cs.storm() <= 0 || cs.push() != 0 || !creative.sawActionBar(PoleCap.KEY_STORM_NO_WIND))
            problems.add("creative at 77.5° N: expected storm, no wind and the no-wind warning, got " + cs);
        MockPlayers.Mock spectator = players.get("spectator 77.5° S");
        PoleStormPayload ss = spectator.last(PoleStormPayload.class);
        boolean specBar = !spectator.actionBars().isEmpty();
        d.append(" spectator:").append(ss).append(",bar=").append(specBar);
        if (ss == null || ss.storm() <= 0 || ss.push() != 0 || specBar)
            problems.add("spectator at 77.5° S: expected the storm only, got " + ss + " bar=" + specBar);
        // Carts: toward the equator, more the further poleward.
        double lastMove = -1;
        for (String name : List.of("69.5° S", "76.0° S", "77.5° S", "79.0° S")) {
            Minecart cart = carts.get(name);
            double dz = cart.getZ() - cartZ0.get(name);
            PoleStormPayload rs = riders.get(name).last(PoleStormPayload.class);
            d.append(String.format(Locale.ROOT, " cart %s:dz=%s,riderPush=%s", name, f(dz), rs == null ? "-" : f(rs.push())));
            if (name.startsWith("69.5")) {
                if (Math.abs(dz) > 1e-3)
                    problems.add("the cart at 69.5° S moved: dz=" + dz);
                continue;
            }
            // South is +z: toward the equator is -z.
            if (!(-dz > lastMove && -dz > 0.1))
                problems.add("the cart at " + name + " did not move further toward the equator: dz=" + dz + " (previous " + lastMove + ")");
            if (rs != null && rs.push() != 0)
                problems.add("the rider of the cart at " + name + " was sent its own wind: " + rs.push());
            lastMove = -dz;
        }
        Minecart north = carts.get("77.5° N");
        double dzN = north.getZ() - cartZ0.get("77.5° N");
        d.append(" cart 77.5° N:dz=").append(f(dzN));
        if (!(dzN > 0.1))
            problems.add("the cart at 77.5° N did not move toward the equator (+z): dz=" + dzN);
        DeepTime.LOGGER.info("[gametest] entityPushedBack C={} cap={}..{} ticks={}{}", rig.c, cap.capDeg(), cap.stopDeg(), RUN_TICKS, d);
        if (!problems.isEmpty())
            helper.fail("entityPushedBack: " + String.join("; ", problems));
        helper.succeed();
    }

    // ---------------------------------------------------------------- hardStopAt80

    /**
     * Nothing the cap governs stays past 80°, in both hemispheres. A survival player put at 80.5° N
     * and one at 82° S (as /tp would), and a creative player at 80.5° S, are put back on the stop
     * line (and the survival one is told why); a spectator at 81° N is left alone. A minecart with a
     * player put at 80.5° N, and one at 79.995° S running poleward at 0.4 blocks/tick, never show a
     * position past the line in 60 ticks, and neither do their riders.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void hardStopAt80(GameTestHelper helper) {
        Rig rig = new Rig(helper);
        rig.force(-79.5, -82.5, 24);
        rig.force(79.5, 82.5, 24);
        rig.spot(0, 80.2);
        rig.spot(0, -80.2);
        Map<String, MockPlayers.Mock> players = new LinkedHashMap<>();
        Map<String, Double> z0 = new LinkedHashMap<>();
        Map<String, Entity> carts = new LinkedHashMap<>();
        Map<String, MockPlayers.Mock> riders = new LinkedHashMap<>();
        double[] maxCartLat = {0, 0};
        long[] startedAt = {-1};
        boolean[] done = {false};
        helper.onEachTick(() -> {
            if (done[0])
                return;
            long t = helper.getTick();
            if (startedAt[0] < 0) {
                if (!rig.ready()) {
                    if (t > LOAD_TICKS) {
                        done[0] = true;
                        rig.cleanUp();
                        helper.fail("the pole strips never became entity-ticking");
                    }
                    return;
                }
                players.put("survival 80.5° N", rig.player(GameType.SURVIVAL, 0, 80.5));
                players.put("survival 82.0° S", rig.player(GameType.SURVIVAL, 2, -82.0));
                players.put("creative 80.5° S", rig.player(GameType.CREATIVE, 4, -80.5));
                players.put("spectator 81.0° N", rig.player(GameType.SPECTATOR, 6, 81.0));
                players.forEach((k, m) -> z0.put(k, m.player.getZ()));
                MockPlayers.Mock[] r = new MockPlayers.Mock[1];
                carts.put("cart 80.5° N", rig.cart(10, 80.5, r));
                riders.put("cart 80.5° N", r[0]);
                // 0.23 blocks inside the line, running poleward at the off-rail limit (0.4 blocks/tick).
                Minecart fast = rig.cart(14, -79.995, r);
                fast.setDeltaMovement(0, 0, 0.4);   // south: poleward is +z
                carts.put("cart 79.995° S", fast);
                riders.put("cart 79.995° S", r[0]);
                startedAt[0] = t;
                return;
            }
            // Every tick after the first: nothing may show past the line.
            if (t - startedAt[0] >= 1) {
                maxCartLat[0] = Math.max(maxCartLat[0], Math.abs(rig.lat(carts.get("cart 80.5° N").getZ())));
                maxCartLat[1] = Math.max(maxCartLat[1], Math.abs(rig.lat(carts.get("cart 79.995° S").getZ())));
            }
            if (t - startedAt[0] < 60)
                return;
            done[0] = true;
            try {
                PoleCapCurve cap = rig.cap;
                double stop = cap.stopZ();
                List<String> problems = new ArrayList<>();
                StringBuilder d = new StringBuilder();
                players.forEach((name, m) -> {
                    double z = m.player.getZ();
                    d.append(String.format(Locale.ROOT, " %s:%s->%s(%s°)", name, f(z0.get(name)), f(z), f(rig.lat(z))));
                    if (name.startsWith("spectator")) {
                        if (z != z0.get(name))
                            problems.add(name + " was moved (spectators pass the stop): " + z0.get(name) + " -> " + z);
                        return;
                    }
                    if (Math.abs(z) > stop || Math.abs(z) < stop - 0.5)
                        problems.add(name + " is not held on the stop line: |z| = " + Math.abs(z) + ", line " + stop);
                });
                if (!players.get("survival 80.5° N").sawActionBar(PoleCap.KEY_STOP))
                    problems.add("the survival player at 80.5° N was not told why it stopped");
                d.append(String.format(Locale.ROOT, " maxCartLat N=%s S=%s", f(maxCartLat[0]), f(maxCartLat[1])));
                for (int i = 0; i < 2; i++)
                    if (maxCartLat[i] > cap.stopDeg())
                        problems.add("a cart showed " + maxCartLat[i] + "°, past the stop");
                carts.forEach((name, cart) -> {
                    double rz = riders.get(name).player.getZ();
                    d.append(String.format(Locale.ROOT, " %s:z=%s,rider=%s", name, f(cart.getZ()), f(rz)));
                    if (Math.abs(cart.getZ()) > stop || Math.abs(rz) > stop + 1)
                        problems.add(name + " (or its rider) ended past the stop: cart z=" + cart.getZ() + " rider z=" + rz);
                    if (!riders.get(name).player.isPassenger())
                        problems.add(name + ": the rider fell out");
                });
                DeepTime.LOGGER.info("[gametest] hardStopAt80 C={} stopZ={}{}", rig.c, f(stop), d);
                if (!problems.isEmpty())
                    helper.fail("hardStopAt80: " + String.join("; ", problems));
                helper.succeed();
            } finally {
                rig.cleanUp();
            }
        });
    }

    // ---------------------------------------------------------------- sableHullPushedBack

    /** Hull blocks: a 3 × 3 deck and a mast, 10 blocks (P0-06's jet). */
    private static List<BlockPos> hull(BlockPos anchor) {
        List<BlockPos> hull = new ArrayList<>();
        for (int dx = -1; dx <= 1; dx++)
            for (int dz = -1; dz <= 1; dz++)
                hull.add(anchor.offset(dx, 0, dz));
        hull.add(anchor.above());
        return hull;
    }

    /**
     * Sable ships (plan §7.11): hulls at rest at 69.5°, 76.5° and 78.5° S and 77.5° N. After 40
     * ticks of wind the one at 69.5° has not moved in z, the others have moved toward the equator,
     * further the higher they started. A fifth hull at 79.3° N launched poleward at 60 blocks/s
     * reaches the stop line and is held: its poleward edge is never more than one tick of travel
     * past the line, and it ends inside it, moving back toward the equator.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void sableHullPushedBack(GameTestHelper helper) {
        if (!ModList.get().isLoaded(SableTestOps.MOD_ID)) {
            helper.fail("Sable is not loaded on this server");
            return;
        }
        Rig rig = new Rig(helper);
        rig.force(-69, -79, 40);
        rig.force(77, 80.5, 40);
        double[] lats = {-69.5, -76.5, -78.5, 77.5, 79.3};
        List<BlockPos> anchors = new ArrayList<>();
        for (int i = 0; i < lats.length; i++)
            anchors.add(rig.spot(8 * i, lats[i]));
        List<Vec3> start = new ArrayList<>();
        double[] maxEdgeLat = {0};
        double speed = 60.0;
        int[] pushes = {1};
        long[] startedAt = {-1};
        boolean[] done = {false};
        helper.onEachTick(() -> {
            if (done[0])
                return;
            long t = helper.getTick();
            ServerLevel level = rig.level;
            if (startedAt[0] < 0) {
                if (!rig.ready()) {
                    if (t > LOAD_TICKS) {
                        done[0] = true;
                        rig.cleanUp();
                        helper.fail("the pole strips never became entity-ticking");
                    }
                    return;
                }
                for (BlockPos a : anchors) {
                    List<BlockPos> blocks = hull(a);
                    for (BlockPos p : blocks)
                        level.setBlock(p, Blocks.OAK_PLANKS.defaultBlockState(), 3);
                    UUID id = SableTestOps.assemble(level, a, blocks);
                    rig.hulls.add(id);
                    start.add(SableTestOps.position(level, id));
                }
                // North is -z: poleward for the last hull.
                SableTestOps.addVelocity(level, rig.hulls.get(4), new Vec3(0, 0, -speed));
                startedAt[0] = t;
                return;
            }
            long run = t - startedAt[0];
            UUID fast = rig.hulls.get(4);
            Vec3 v = SableTestOps.velocity(level, fast);
            // As in jetCrossesSeam: if the body joined the physics pipeline after the first push, push once more.
            if (run == 1 && pushes[0] < 2 && v != null && v.z > -speed / 2) {
                SableTestOps.addVelocity(level, fast, new Vec3(0, 0, -speed - v.z));
                pushes[0]++;
            }
            AABB box = SableTestOps.bounds(level, fast);
            Vec3 pos = SableTestOps.position(level, fast);
            if (box != null && pos != null && box.minZ <= pos.z + 1 && box.maxZ >= pos.z - 1)
                maxEdgeLat[0] = Math.max(maxEdgeLat[0], rig.lat(box.minZ));   // north: the poleward edge is min z
            else if (pos != null)
                maxEdgeLat[0] = Math.max(maxEdgeLat[0], rig.lat(pos.z));
            if (run < RUN_TICKS)
                return;
            done[0] = true;
            try {
                PoleCapCurve cap = rig.cap;
                List<String> problems = new ArrayList<>();
                StringBuilder d = new StringBuilder();
                double[] dz = new double[lats.length];
                for (int i = 0; i < lats.length; i++) {
                    Vec3 p = SableTestOps.position(level, rig.hulls.get(i));
                    if (p == null || start.get(i) == null) {
                        problems.add("hull at " + lats[i] + "° vanished");
                        continue;
                    }
                    dz[i] = p.z - start.get(i).z;
                    Vec3 vel = SableTestOps.velocity(level, rig.hulls.get(i));
                    d.append(String.format(Locale.ROOT, " %s:dz=%s,vz=%s,y=%s", PoleCap.formatLatitude(lats[i]), f(dz[i]),
                            vel == null ? "-" : f(vel.z), f(p.y)));
                }
                if (problems.isEmpty()) {
                    if (Math.abs(dz[0]) > 0.5)
                        problems.add("the hull at 69.5° S moved in z: " + dz[0]);
                    if (!(dz[1] < -0.5 && dz[2] < dz[1] - 0.5))
                        problems.add("the hulls at 76.5° and 78.5° S were not pushed toward the equator (-z), more at 78.5°: "
                                + dz[1] + ", " + dz[2]);
                    if (!(dz[3] > 0.5))
                        problems.add("the hull at 77.5° N was not pushed toward the equator (+z): " + dz[3]);
                    // The stop: one tick at 60 blocks/s is 3 blocks; allow a block more for Sable's substeps.
                    double allowed = Math.abs(rig.lat(rig.z(cap.stopDeg()) - 4.0));   // 4 blocks past the stop
                    Vec3 fp = SableTestOps.position(level, fast), fv = SableTestOps.velocity(level, fast);
                    d.append(String.format(Locale.ROOT, " stop hull: maxEdgeLat=%s (allowed %s) end lat=%s vz=%s pushes=%d",
                            f(maxEdgeLat[0]), f(allowed), f(rig.lat(fp.z)), fv == null ? "-" : f(fv.z), pushes[0]));
                    if (maxEdgeLat[0] < 79.9)
                        problems.add("the fast hull never reached the stop line (max edge " + maxEdgeLat[0] + "°): the stop was not tested");
                    if (maxEdgeLat[0] > allowed)
                        problems.add("the fast hull's edge reached " + maxEdgeLat[0] + "°, past " + allowed + "°");
                    if (rig.lat(fp.z) > cap.stopDeg() || fv == null || fv.z < 0)
                        problems.add("the fast hull is not held inside the stop, moving back: lat " + rig.lat(fp.z) + " vz " + (fv == null ? null : fv.z));
                }
                DeepTime.LOGGER.info("[gametest] sableHullPushedBack C={} ticks={} windMax={}{}", rig.c, RUN_TICKS, PoleCapCurve.WIND_MAX, d);
                if (!problems.isEmpty())
                    helper.fail("sableHullPushedBack: " + String.join("; ", problems));
                helper.succeed();
            } finally {
                rig.cleanUp();
            }
        });
    }
}
