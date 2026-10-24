// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import com.minecraftindustrialcomplex.deeptime.compat.sable.SablePoleCap;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngine;
import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.vehicle.VehicleEntity;
import net.minecraft.world.phys.Vec3;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.common.util.FakePlayer;
import net.neoforged.neoforge.event.tick.LevelTickEvent;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/**
 * The <b>temporary</b> border at a Petroff–Guyou world's north and south edges (FORMAT.md "The world's
 * layout", {@code border}: {@code edge_stop}; Ben, decisions log 2026-10-04, "Engine only; pause IP").
 * Each of those edges is glued onto itself by a half-turn that only the wrap engine can make walkable;
 * until it installs the layout's topology on the level, players, the vehicles they ride and Sable ships
 * are stopped {@link #INSET} blocks inside the edge, like the pole cap's hard stop on a Lambert world.
 * The terrain past the edge is the folded copy of the glued side (the grids fold through the
 * half-turns), so the view across the border is the right one; only the crossing waits.
 *
 * <p>The border is off when the level's generator is not a Petroff–Guyou or cube Deep Time planet, when
 * the world's preset asked for none ({@code scale.edge_border = none}: the manifest's layout block has
 * no border), and as soon as {@link WrapEngine#topology} is set on the level. Creative players are
 * stopped too (as at the pole cap); spectators pass. Within {@link #WARN_BLOCKS} of the stop line the
 * action bar says the edge is closed for now.
 *
 * <p><b>The cube world</b> (cube plan W3 and W4, docs/design/cube-game.md section 5) stops at its 12
 * turning edges ({@link CubeBorder}): a player or vehicle past the stop line is put back at the nearest
 * point of the planet that is the inset inside every segment. A player who arrives more than
 * {@link #FOLD_DEPTH} blocks outside the planet (a teleport, a pearl, a portal) is folded onto the
 * partner block instead, the heading turned by the edge's motion: the terrain there is the same as
 * where they landed, turned. A Sable ship whose bounds leave that region loses its outward velocity
 * and is blown back ({@code SablePoleCap.tickCube}), never teleported.
 */
public final class EdgeBorder {

    /** How far inside the edge the stop line is, blocks (the layout block's {@code inset_blocks} default). */
    public static final double INSET = 1.0; // ledger: POL-022, SCL-088 (game-choice)
    /** The warning band inside the stop line, blocks. */
    public static final double WARN_BLOCKS = 24.0; // ledger: POL-023 (game-choice)
    /** How often the warning repeats while in the band, ticks. */
    public static final int BAR_EVERY = 40; // ledger: POL-023 (game-choice)
    /** How far inside the stop line a stopped player or vehicle is put, blocks. */
    static final double STOP_INSET = 0.05; // ledger: POL-022 (game-choice)

    public static final String KEY_NEAR = "deeptime.edgeborder.near";
    public static final String KEY_STOP = "deeptime.edgeborder.stop";
    public static final String KEY_CUBE_NEAR = "deeptime.edgeborder.cube.near";
    public static final String KEY_CUBE_STOP = "deeptime.edgeborder.cube.stop";

    /**
     * On a cube world, a player found more than this far outside the planet is folded onto the partner
     * block rather than stopped, blocks: further than one tick of any ordinary movement carries a
     * player past the stop line (an elytra dive tops out near 4 blocks a tick), so only an arrival (a
     * teleport, a pearl, a portal exit) lands that deep.
     */
    public static final double FOLD_DEPTH = 16.0; // ledger: POL-031 (game-choice)

    /** A cube world's border in force on a level: the geometry and the stop line's inset. */
    public record CubeStop(CubeBorder border, double inset) {}

    private static final Map<UUID, Long> LAST_BAR = new HashMap<>();
    private static boolean sable;

    private EdgeBorder() {}

    public static void init() {
        sable = ModList.get().isLoaded("sable");
        NeoForge.EVENT_BUS.addListener(LevelTickEvent.Post.class, EdgeBorder::onLevelTick);
    }

    /**
     * The |z| of the stop line on {@code level}, or NaN when the level has no edge border now (not a
     * Petroff–Guyou planet, a world without a border, or the wrap engine joins the edges).
     */
    public static double stopZ(ServerLevel level) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !g.dataRef().petroffGuyou())
            return Double.NaN;
        if (WrapEngine.topology(level) != null)
            return Double.NaN;
        double inset = g.borderInset();
        if (Double.isNaN(inset))
            return Double.NaN;
        return g.dataRef().hemisphereBlocks() / 2.0 - inset;
    }

    /**
     * The cube border on {@code level}, or null when it has none now (not a cube planet, a world
     * without a border, or the wrap engine joins the edges).
     */
    public static CubeStop cubeStop(ServerLevel level) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !g.dataRef().cube())
            return null;
        if (WrapEngine.topology(level) != null)
            return null;
        double inset = g.borderInset();
        if (Double.isNaN(inset))
            return null;
        return new CubeStop(CubeBorder.of(g.dataRef().faceBlocks()), inset);
    }

    private static void onLevelTick(LevelTickEvent.Post event) {
        if (event.getLevel() instanceof ServerLevel level) {
            double stop = stopZ(level);
            if (!Double.isNaN(stop)) {
                tick(level, stop);
                return;
            }
            CubeStop cube = cubeStop(level);
            if (cube != null)
                tickCube(level, cube);
        }
    }

    /** One tick of a cube world's border on {@code level}. */
    public static void tickCube(ServerLevel level, CubeStop c) {
        long now = level.getGameTime();
        Map<Entity, Boolean> vehicles = null;
        for (ServerPlayer player : List.copyOf(level.players())) {
            if (player instanceof FakePlayer || player.isRemoved() || player.isDeadOrDying() || player.isSpectator())
                continue;
            Entity root = player.getRootVehicle();
            int zone = c.border().zone(root.getX(), root.getZ(), c.inset(), WARN_BLOCKS);
            if (root != player) {
                if (vehicles == null)
                    vehicles = new IdentityHashMap<>();
                vehicles.put(root, Boolean.TRUE);
            } else if (zone == 2) {
                if (foldArrival(level, c, player))
                    continue;
                stopPlayerCube(level, c, player);
            }
            if (zone != 0) {
                Long last = LAST_BAR.get(player.getUUID());
                if (zone == 2 || last == null || now - last >= BAR_EVERY || now < last) {
                    player.displayClientMessage(cubeWarning(zone), true);
                    LAST_BAR.put(player.getUUID(), now);
                }
            }
        }
        if (vehicles != null)
            for (Entity v : vehicles.keySet())
                if (c.border().zone(v.getX(), v.getZ(), c.inset(), WARN_BLOCKS) == 2)
                    stopVehicleCube(level, c, v);
        if (sable)
            SablePoleCap.tickCube(level, c.border(), c.inset());
    }

    /** The cube world's action-bar line (plain text fallbacks match en_us.json). */
    static Component cubeWarning(int zone) {
        if (zone == 2)
            return Component.translatableWithFallback(KEY_CUBE_STOP,
                    "This edge of the world is closed for now: the seam that joins it is not built yet").withStyle(ChatFormatting.RED);
        return Component.translatableWithFallback(KEY_CUBE_NEAR,
                "Near a turning edge of the world: a temporary border stops you here until the seam is built").withStyle(ChatFormatting.GOLD);
    }

    /**
     * Cube plan W4: a player more than {@link #FOLD_DEPTH} outside the planet is moved to the planet
     * point the fill there shows (the fold through the nearest net edge), heading turned, and kept the
     * inset inside the border. False when the player is not that deep or is past the net's corners
     * (the ocean there is no copy of anything; the stop brings them back).
     */
    static boolean foldArrival(ServerLevel level, CubeStop c, ServerPlayer player) {
        CubeBorder b = c.border();
        if (b.depthOutside(player.getX(), player.getZ()) <= FOLD_DEPTH)
            return false;
        var p = b.cube().foldPoint(player.getX(), player.getZ());
        if (p == null)
            return false;
        double[] q = b.nearest(b.frameX(p.x()), p.z(), c.inset() + STOP_INSET);
        double y = PoleCap.freeY(level, player, q[0], player.getY(), q[1]);
        player.connection.teleport(q[0], y, q[1], CubeBorder.turnYaw(player.getYRot(), p.iso()), player.getXRot());
        return true;
    }

    static void stopPlayerCube(ServerLevel level, CubeStop c, ServerPlayer player) {
        double[] q = c.border().nearest(player.getX(), player.getZ(), c.inset() + STOP_INSET);
        double y = PoleCap.freeY(level, player, q[0], player.getY(), q[1]);
        player.connection.teleport(q[0], y, q[1], player.getYRot(), player.getXRot());
    }

    static void stopVehicleCube(ServerLevel level, CubeStop c, Entity vehicle) {
        if (!(vehicle instanceof VehicleEntity || vehicle instanceof LivingEntity)) {
            List<ServerPlayer> riders = new ArrayList<>();
            for (Entity e : vehicle.getIndirectPassengers())
                if (e instanceof ServerPlayer p && !(p instanceof FakePlayer) && !p.isSpectator())
                    riders.add(p);
            for (ServerPlayer p : riders) {
                p.stopRiding();
                stopPlayerCube(level, c, p);
            }
            return;
        }
        double[] q = c.border().nearest(vehicle.getX(), vehicle.getZ(), c.inset() + STOP_INSET);
        double y = PoleCap.freeY(level, vehicle, q[0], vehicle.getY(), q[1]);
        double nx = vehicle.getX() - q[0], nz = vehicle.getZ() - q[1];
        vehicle.teleportTo(q[0], y, q[1]);
        // Take away the part of the motion that points out of the planet.
        Vec3 v = vehicle.getDeltaMovement();
        double len = Math.hypot(nx, nz), out = len > 0 ? (v.x * nx + v.z * nz) / len : 0.0;
        if (out > 0)
            vehicle.setDeltaMovement(v.x - out * nx / len, v.y, v.z - out * nz / len);
        vehicle.hurtMarked = true;
        if (vehicle.getControllingPassenger() instanceof ServerPlayer driver && !(driver instanceof FakePlayer))
            driver.connection.send(new ClientboundMoveVehiclePacket(vehicle));
    }

    /** The zone of z: 0 open, 1 in the warning band, 2 past the stop line. */
    public static int zone(double stopZ, double z) {
        double a = Math.abs(z);
        if (a > stopZ)
            return 2;
        return a > stopZ - WARN_BLOCKS ? 1 : 0;
    }

    /** z put back on the stop line's inner side, or unchanged. */
    public static double clampZ(double stopZ, double z) {
        double limit = stopZ - STOP_INSET;
        return z > limit ? limit : (z < -limit ? -limit : z);
    }

    /** One tick of the border on {@code level}. */
    public static void tick(ServerLevel level, double stopZ) {
        long now = level.getGameTime();
        Map<Entity, Boolean> vehicles = null;
        for (ServerPlayer player : List.copyOf(level.players())) {
            if (player instanceof FakePlayer || player.isRemoved() || player.isDeadOrDying() || player.isSpectator())
                continue;
            Entity root = player.getRootVehicle();
            int zone = zone(stopZ, root.getZ());
            if (root != player) {
                if (vehicles == null)
                    vehicles = new IdentityHashMap<>();
                vehicles.put(root, Boolean.TRUE);
            } else if (zone == 2) {
                stopPlayer(level, stopZ, player);
            }
            if (zone != 0) {
                Long last = LAST_BAR.get(player.getUUID());
                if (zone == 2 || last == null || now - last >= BAR_EVERY || now < last) {
                    player.displayClientMessage(warning(root.getZ(), zone), true);
                    LAST_BAR.put(player.getUUID(), now);
                }
            }
        }
        if (vehicles != null)
            for (Entity v : vehicles.keySet())
                if (zone(stopZ, v.getZ()) == 2)
                    stopVehicle(level, stopZ, v);
        if (sable)
            SablePoleCap.tickEdge(level, stopZ);
    }

    /** The action-bar line (plain text fallbacks match en_us.json). */
    static Component warning(double z, int zone) {
        String edge = z < 0 ? "north" : "south";
        if (zone == 2)
            return Component.translatableWithFallback(KEY_STOP,
                    "The world's %s edge is closed for now: the seam that joins it is not built yet", edge).withStyle(ChatFormatting.RED);
        return Component.translatableWithFallback(KEY_NEAR,
                "Near the world's %s edge: a temporary border stops you here until the seam is built", edge).withStyle(ChatFormatting.GOLD);
    }

    static void stopPlayer(ServerLevel level, double stopZ, ServerPlayer player) {
        double z = clampZ(stopZ, player.getZ());
        double y = PoleCap.freeY(level, player, player.getX(), player.getY(), z);
        player.connection.teleport(player.getX(), y, z, player.getYRot(), player.getXRot());
    }

    static void stopVehicle(ServerLevel level, double stopZ, Entity vehicle) {
        if (!(vehicle instanceof VehicleEntity || vehicle instanceof LivingEntity)) {
            List<ServerPlayer> riders = new ArrayList<>();
            for (Entity e : vehicle.getIndirectPassengers())
                if (e instanceof ServerPlayer p && !(p instanceof FakePlayer) && !p.isSpectator())
                    riders.add(p);
            for (ServerPlayer p : riders) {
                p.stopRiding();
                stopPlayer(level, stopZ, p);
            }
            return;
        }
        double z = clampZ(stopZ, vehicle.getZ());
        double y = PoleCap.freeY(level, vehicle, vehicle.getX(), vehicle.getY(), z);
        vehicle.teleportTo(vehicle.getX(), y, z);
        Vec3 v = vehicle.getDeltaMovement();
        if (v.z * z > 0)
            vehicle.setDeltaMovement(v.x, v.y, 0);
        vehicle.hurtMarked = true;
        if (vehicle.getControllingPassenger() instanceof ServerPlayer driver && !(driver instanceof FakePlayer))
            driver.connection.send(new ClientboundMoveVehiclePacket(vehicle));
    }
}
