// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import com.minecraftindustrialcomplex.deeptime.compat.sable.SableBorder;
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
import net.minecraft.world.phys.AABB;
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
 * The <b>temporary</b> border at the cube world's 12 turning edges (FORMAT.md "The cube world's
 * layout", {@code border}: {@code edge_stop}; cube plan W3 and W4, docs/design/cube-game.md section 5;
 * Ben, decisions log 2026-10-04, "Engine only; pause IP"). Each of those edges is glued to another by
 * a turn that only the wrap engine can make walkable; until it installs the layout's topology on the
 * level, players, the vehicles they ride and Sable ships are stopped {@link #INSET} blocks inside the
 * edge ({@link CubeBorder}). The terrain past the edge is the folded copy of the glued side (the grids
 * fold through the turns), so the view across the border is the right one; only the crossing waits.
 *
 * <p>A player or vehicle past the stop line is put back at the nearest point of the planet that is the
 * inset inside every segment. A player who arrives more than {@link #FOLD_DEPTH} blocks outside the
 * planet (a teleport, a pearl, a portal) is folded onto the partner block instead, the heading turned
 * by the edge's motion: the terrain there is the same as where they landed, turned. A Sable ship whose
 * bounds leave that region loses its outward velocity and is blown back ({@code SableBorder.tickCube}),
 * never teleported.
 *
 * <p>The border is off when the level's generator is not a cube Deep Time planet, when the world's
 * preset asked for none ({@code scale.edge_border = none}: the manifest's layout block has no border),
 * and as soon as {@link WrapEngine#topology} is set on the level. Creative players are stopped too;
 * spectators pass. Within {@link #WARN_BLOCKS} of the stop line the action bar says the edge is closed
 * for now. (The Petroff–Guyou world's north and south edge stop went with that world, cube plan §7.4,
 * decisions log 2026-10-24.)
 */
public final class EdgeBorder {

    /** How far inside the edge the stop line is, blocks (the layout block's {@code inset_blocks} default). */
    public static final double INSET = 1.0; // ledger: POL-031, SCL-088 (game-choice)
    /** The warning band inside the stop line, blocks. */
    public static final double WARN_BLOCKS = 24.0; // ledger: POL-023 (game-choice)
    /** How often the warning repeats while in the band, ticks. */
    public static final int BAR_EVERY = 40; // ledger: POL-023 (game-choice)
    /** How far inside the stop line a stopped player or vehicle is put, blocks. */
    static final double STOP_INSET = 0.05; // ledger: POL-031 (game-choice)

    /**
     * The push that turns a Sable ship back at the border, blocks per second added per tick (20 blocks/s²):
     * the pole cap's Sable wind at its stop line, which the border kept when the cap went.
     */
    public static final double SABLE_PUSH = 1.0; // ledger: POL-031 (game-choice)

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
            SableBorder.tickCube(level, c.border(), c.inset());
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
        double y = freeY(level, player, q[0], player.getY(), q[1]);
        player.connection.teleport(q[0], y, q[1], CubeBorder.turnYaw(player.getYRot(), p.iso()), player.getXRot());
        return true;
    }

    static void stopPlayerCube(ServerLevel level, CubeStop c, ServerPlayer player) {
        double[] q = c.border().nearest(player.getX(), player.getZ(), c.inset() + STOP_INSET);
        double y = freeY(level, player, q[0], player.getY(), q[1]);
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
        double y = freeY(level, vehicle, q[0], vehicle.getY(), q[1]);
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

    /**
     * The lowest y from {@code y} up at which {@code entity} fits at (x, z): the stop line can run
     * through rising ground (ledger: POL-031). The destination is loaded first (it usually is: it is
     * next to the entity).
     */
    static double freeY(ServerLevel level, Entity entity, double x, double y, double z) {
        level.getChunk((int) Math.floor(x) >> 4, (int) Math.floor(z) >> 4);
        AABB box = entity.getBoundingBox().move(x - entity.getX(), y - entity.getY(), z - entity.getZ());
        for (int dy = 0; dy <= 64 && y + dy < level.getMaxBuildHeight(); dy++) {
            if (level.noCollision(entity, box.move(0, dy, 0)))
                return y + dy;
        }
        return y;
    }
}
