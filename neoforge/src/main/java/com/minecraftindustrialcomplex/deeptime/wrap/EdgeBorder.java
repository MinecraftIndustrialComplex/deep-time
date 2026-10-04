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
 * <p>The border is off when the level's generator is not a Petroff–Guyou Deep Time planet, when the
 * world's preset asked for none ({@code scale.edge_border = none}: the manifest's layout block has no
 * border), and as soon as {@link WrapEngine#topology} is set on the level. Creative players are stopped
 * too (as at the pole cap); spectators pass. Within {@link #WARN_BLOCKS} of the stop line the action bar
 * says the edge is closed for now.
 */
public final class EdgeBorder {

    /** How far inside the edge the stop line is, blocks (the layout block's {@code inset_blocks} default). */
    public static final double INSET = 1.0;
    /** The warning band inside the stop line, blocks. */
    public static final double WARN_BLOCKS = 24.0;
    /** How often the warning repeats while in the band, ticks. */
    public static final int BAR_EVERY = 40;
    /** How far inside the stop line a stopped player or vehicle is put, blocks. */
    static final double STOP_INSET = 0.05;

    public static final String KEY_NEAR = "deeptime.edgeborder.near";
    public static final String KEY_STOP = "deeptime.edgeborder.stop";

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

    private static void onLevelTick(LevelTickEvent.Post event) {
        if (event.getLevel() instanceof ServerLevel level) {
            double stop = stopZ(level);
            if (!Double.isNaN(stop))
                tick(level, stop);
        }
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
