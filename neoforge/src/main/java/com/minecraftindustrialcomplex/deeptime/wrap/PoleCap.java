// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import com.minecraftindustrialcomplex.deeptime.compat.sable.SablePoleCap;
import com.minecraftindustrialcomplex.deeptime.net.DeepTimeNetwork;
import com.minecraftindustrialcomplex.deeptime.net.PoleStormPayload;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve.Zone;
import net.minecraft.ChatFormatting;
import net.minecraft.network.chat.Component;
import net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.entity.vehicle.VehicleEntity;
import net.minecraft.world.level.Level;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.common.util.FakePlayer;
import net.neoforged.neoforge.event.entity.player.PlayerEvent;
import net.neoforged.neoforge.event.server.ServerStoppedEvent;
import net.neoforged.neoforge.event.tick.LevelTickEvent;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.UUID;

/**
 * The pole soft cap (plan §7.11; decisions file, "Poles"): a storm and a pushback toward the equator
 * from {@code pole_cap_deg} (75°), a hard stop at {@code pole_stop_deg} (80°), per Deep Time level
 * (both from the generator's {@code world_data}; {@link PoleCapCurve} holds the geometry and the
 * curves). Runs on the server after every tick of a Deep Time level.
 *
 * <p><b>Who it governs.</b>
 * <ul>
 *   <li><b>Players</b>, by game mode ({@link Policy}): survival and adventure get everything;
 *       creative gets the storm, the warning and the hard stop but no wind (the vanilla world border
 *       stops creative players too, but builders at 76° should not be blown about); spectators are
 *       exempt from wind and stop (as they pass the vanilla border) and only see the storm. Fake
 *       players (machines) are ignored.</li>
 *   <li><b>Vehicles carrying a player</b> (the player's root vehicle), under the strongest policy
 *       among its players. Empty vehicles and mobs are not capped: nothing is gained by pushing them.</li>
 *   <li><b>Sable ships</b> (sub-levels), when Sable is loaded: {@link SablePoleCap}.</li>
 * </ul>
 *
 * <p><b>Who moves whom.</b> A player's movement, and that of a vehicle the player steers (boat,
 * horse), is simulated by the player's client; the server cannot add to that velocity without
 * overwriting it (a motion packet replaces the client's velocity). So the server sends the wind as
 * {@link PoleStormPayload#push()} and Deep Time's client adds it every tick. Vehicles the server
 * simulates (minecarts, anything a player rides but does not steer) are pushed here. The hard stop is
 * always the server's: a player past the stop line is teleported back onto it (an absolute teleport,
 * which also zeroes the client's velocity), a vehicle is moved back with its riders (plus a
 * {@code ClientboundMoveVehiclePacket} when a client steers it), and a rider of anything else (Create
 * contraptions, modded seats) is dismounted and stopped alone.
 *
 * <p><b>What the player sees.</b> The storm intensity goes to the client in {@link PoleStormPayload}
 * when it changes (fog and snow are drawn there); an action-bar line warns while in the storm band
 * (every {@value #BAR_EVERY} ticks, and at once when the stop fires).
 */
public final class PoleCap {

    /** How often the storm warning is repeated while in the band, ticks (the action bar fades in ~3 s). */
    public static final int BAR_EVERY = 40;
    /** How far inside the stop line a stopped player or vehicle is put, blocks. */
    public static final double STOP_INSET = 0.05;
    /** A new storm payload goes out when the storm changes by this much... */
    static final float STORM_STEP = 0.02f;
    /** ...or the push by this much (2 % of the maximum). */
    static final float PUSH_STEP = (float) (PoleCapCurve.PUSH_MAX * 0.02);

    /** Translation keys (assets/deeptime/lang/en_us.json). */
    public static final String KEY_STORM = "deeptime.polecap.storm";
    public static final String KEY_STORM_NO_WIND = "deeptime.polecap.storm.no_wind";
    public static final String KEY_STOP = "deeptime.polecap.stop";

    /** What the cap does to a player. */
    public enum Policy {
        /** Wind, warning and hard stop (survival, adventure). */
        FULL,
        /** Warning and hard stop, no wind (creative). */
        STOP_ONLY,
        /** Only the storm is drawn (spectator). */
        NONE;

        Policy stronger(Policy other) {
            return ordinal() <= other.ordinal() ? this : other;
        }
    }

    /** What the server last told one player. */
    private static final class PlayerState {
        ResourceKey<Level> level;
        float storm, push;
        long lastBar = Long.MIN_VALUE / 2;
        Zone zone = Zone.OPEN;
    }

    private static final Map<UUID, PlayerState> STATES = new HashMap<>();
    private static boolean sable;

    private PoleCap() {}

    public static void init() {
        sable = ModList.get().isLoaded("sable");
        NeoForge.EVENT_BUS.addListener(LevelTickEvent.Post.class, PoleCap::onLevelTick);
        // Whenever the client resets its state (a new level, a respawn, a new login), forget ours so
        // the storm goes out again.
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerLoggedInEvent.class, e -> forget(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerLoggedOutEvent.class, e -> forget(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerChangedDimensionEvent.class, e -> forget(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(PlayerEvent.PlayerRespawnEvent.class, e -> forget(e.getEntity()));
        NeoForge.EVENT_BUS.addListener(ServerStoppedEvent.class, e -> STATES.clear());
    }

    private static void forget(Player player) {
        STATES.remove(player.getUUID());
    }

    public static Policy policy(ServerPlayer player) {
        if (player.isSpectator())
            return Policy.NONE;
        if (player.isCreative())
            return Policy.STOP_ONLY;
        return Policy.FULL;
    }

    /** The cap of {@code level}, or null when it is not a Deep Time planet. */
    public static PoleCapCurve capOf(ServerLevel level) {
        return level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g ? g.dataRef().poleCap() : null;
    }

    private static void onLevelTick(LevelTickEvent.Post event) {
        if (event.getLevel() instanceof ServerLevel level) {
            PoleCapCurve cap = capOf(level);
            if (cap != null)
                tick(level, cap);
        }
    }

    /** One tick of the cap on {@code level}. */
    public static void tick(ServerLevel level, PoleCapCurve cap) {
        long now = level.getGameTime();
        Map<Entity, Policy> vehicles = null;
        // A copy: a hard stop can dismount a player, which must not disturb the iteration.
        for (ServerPlayer player : List.copyOf(level.players())) {
            if (player instanceof FakePlayer || player.isRemoved() || player.isDeadOrDying())
                continue;
            Policy policy = policy(player);
            Entity root = player.getRootVehicle();
            if (root != player && policy != Policy.NONE) {
                if (vehicles == null)
                    vehicles = new IdentityHashMap<>();
                vehicles.merge(root, policy, Policy::stronger);
            }
            tickPlayer(level, cap, player, policy, root, now);
        }
        if (vehicles != null)
            vehicles.forEach((vehicle, policy) -> tickVehicle(level, cap, vehicle, policy));
        if (sable)
            SablePoleCap.tick(level, cap);
    }

    private static void tickPlayer(ServerLevel level, PoleCapCurve cap, ServerPlayer player, Policy policy, Entity root, long now) {
        // A rider is where its vehicle is (the vehicle is what moves and what gets stopped).
        double z = root.getZ();
        Zone zone = cap.zone(z);
        boolean stopped = false;
        if (zone == Zone.STOP && policy != Policy.NONE && root == player) {
            stopPlayer(level, cap, player);
            stopped = true;
        }
        // The wind the client applies: to the player, or to the vehicle it steers.
        boolean clientMoves = root == player || root.getControllingPassenger() == player;
        float storm = (float) cap.storm(z);
        float push = policy == Policy.FULL && clientMoves ? (float) cap.push(z) : 0f;

        PlayerState state = STATES.computeIfAbsent(player.getUUID(), u -> new PlayerState());
        if (state.level != level.dimension()) {
            state.level = level.dimension();
            state.storm = state.push = 0f;
            state.zone = Zone.OPEN;
        }
        if (changed(storm, state.storm, STORM_STEP) || changed(push, state.push, PUSH_STEP)) {
            DeepTimeNetwork.send(player, new PoleStormPayload(storm, push));
            state.storm = storm;
            state.push = push;
        }
        if (policy != Policy.NONE && zone != Zone.OPEN && (stopped || zone != state.zone || now - state.lastBar >= BAR_EVERY)) {
            player.displayClientMessage(warning(cap, zone, policy, z), true);
            state.lastBar = now;
        }
        state.zone = zone;
    }

    /** Sends on any change to or from zero, and on steps of at least {@code step} otherwise. */
    private static boolean changed(float now, float last, float step) {
        if ((now == 0f) != (last == 0f))
            return true;
        return Math.abs(now - last) >= step;
    }

    /** The action-bar line (plain text fallbacks match en_us.json). */
    static Component warning(PoleCapCurve cap, Zone zone, Policy policy, double z) {
        String lat = formatLatitude(cap.latitudeDeg(z));
        String stop = formatLatitude(Math.copySign(cap.stopDeg(), cap.latitudeDeg(z)));
        if (zone == Zone.STOP)
            return Component.translatableWithFallback(KEY_STOP,
                    "You cannot go any further: %s is the edge of the world", stop).withStyle(ChatFormatting.RED);
        if (policy == Policy.FULL)
            return Component.translatableWithFallback(KEY_STORM,
                    "Polar storm (%s): the wind is pushing you back toward the equator", lat).withStyle(ChatFormatting.GOLD);
        return Component.translatableWithFallback(KEY_STORM_NO_WIND,
                "Polar storm (%s): the world ends at %s", lat, stop).withStyle(ChatFormatting.GOLD);
    }

    /** "77.4° N" / "80.0° S". */
    public static String formatLatitude(double latDeg) {
        return String.format(Locale.ROOT, "%.1f° %s", Math.abs(latDeg), latDeg >= 0 ? "N" : "S");
    }

    /** Puts a player past the stop line back onto it: an absolute teleport, which zeroes its velocity. */
    static void stopPlayer(ServerLevel level, PoleCapCurve cap, ServerPlayer player) {
        double z = cap.clampZ(player.getZ(), STOP_INSET);
        double y = freeY(level, player, player.getX(), player.getY(), z);
        player.connection.teleport(player.getX(), y, z, player.getYRot(), player.getXRot());
    }

    private static void tickVehicle(ServerLevel level, PoleCapCurve cap, Entity vehicle, Policy policy) {
        double z = vehicle.getZ();
        Zone zone = cap.zone(z);
        if (zone == Zone.OPEN)
            return;
        boolean clientSteers = vehicle.getControllingPassenger() instanceof Player;
        if (zone == Zone.STOP) {
            stopVehicle(level, cap, vehicle);
            return;
        }
        // Wind for vehicles the server simulates; a steered one gets it from its client.
        if (policy == Policy.FULL && !clientSteers)
            vehicle.push(0, 0, PoleCapCurve.equatorward(z) * cap.push(z));
    }

    /** Moves a vehicle past the stop line back onto it, riders included. */
    static void stopVehicle(ServerLevel level, PoleCapCurve cap, Entity vehicle) {
        if (!(vehicle instanceof VehicleEntity || vehicle instanceof LivingEntity)) {
            // A contraption or modded seat may not move when told to (a Create train follows its
            // track graph): its riders leave it and are stopped on their own.
            List<ServerPlayer> riders = new ArrayList<>();
            for (Entity e : vehicle.getIndirectPassengers())
                if (e instanceof ServerPlayer p && !(p instanceof FakePlayer) && policy(p) != Policy.NONE)
                    riders.add(p);
            for (ServerPlayer p : riders) {
                p.stopRiding();
                stopPlayer(level, cap, p);
            }
            return;
        }
        double z = cap.clampZ(vehicle.getZ(), STOP_INSET);
        double y = freeY(level, vehicle, vehicle.getX(), vehicle.getY(), z);
        vehicle.teleportTo(vehicle.getX(), y, z);
        Vec3 v = vehicle.getDeltaMovement();
        if (v.z * PoleCapCurve.equatorward(z) < 0)
            vehicle.setDeltaMovement(v.x, v.y, 0);
        vehicle.hurtMarked = true;
        if (vehicle.getControllingPassenger() instanceof ServerPlayer driver && !(driver instanceof FakePlayer))
            driver.connection.send(new ClientboundMoveVehiclePacket(vehicle));
    }

    /**
     * The lowest y from {@code y} up at which {@code entity} fits at (x, z): the stop line can run
     * through rising ground. The destination is loaded first (it usually is: it is next to the entity).
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
