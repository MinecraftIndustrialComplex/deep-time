// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrap;

import com.minecraftindustrialcomplex.deeptime.gametest.GameTestSupport;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.entity.vehicle.VehicleEntity;
import net.minecraft.world.level.Level;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.tick.EntityTickEvent;

/**
 * PROTOTYPE of plan §7.11's pole soft cap (P0-06 spike item). Not the cap: that is P1-18's
 * {@code wrap/PoleCap}, with the action-bar warning, the storm state, Sable hulls and the hard
 * stop at 80°. This prototype only answers "does a server-side equatorward nudge that grows with
 * latitude move things back", and it is active only in the dev GameTest runtimes
 * ({@code -Ddeeptime.gametest=true}), never in a real world.
 *
 * <p>Latitude is equirectangular (plan §5.2): z ∈ [−C/4, C/4) spans the poles, 90° at |z| = C/4,
 * north (−z) positive. Poleward of {@link #CAP_DEG} an entity gets, after its own tick, a velocity
 * change toward z = 0 of {@link #MAX_NUDGE} × (|lat| − 75°) / 5°, clamped at 80°. It applies to
 * players, vehicles, and (for the GameTest) entities tagged {@link #PROBE_TAG}.
 */
public final class PoleCapPrototype {

    public static final double CAP_DEG = 75.0;
    public static final double HARD_DEG = 80.0;
    /** Velocity change per tick at 80°, in blocks per tick (a strong headwind, not a wall). */
    public static final double MAX_NUDGE = 0.08;
    public static final String PROBE_TAG = "deeptime_polecap_probe";

    private PoleCapPrototype() {}

    public static double latitudeDeg(int circumference, double z) {
        return -z / (circumference / 4.0) * 90.0;
    }

    public static double zOfLatitude(int circumference, double latDeg) {
        return -latDeg / 90.0 * (circumference / 4.0);
    }

    /** Equatorward velocity change per tick, blocks/tick: 0 up to 75°, linear to MAX at 80°. */
    public static double nudge(double absLatDeg) {
        if (absLatDeg <= CAP_DEG)
            return 0;
        return MAX_NUDGE * Math.min(1.0, (absLatDeg - CAP_DEG) / (HARD_DEG - CAP_DEG));
    }

    public static void init() {
        if (!GameTestSupport.enabled())
            return;
        NeoForge.EVENT_BUS.addListener(EntityTickEvent.Post.class, PoleCapPrototype::onEntityTick);
    }

    private static void onEntityTick(EntityTickEvent.Post event) {
        Entity entity = event.getEntity();
        if (!(entity.level() instanceof ServerLevel level) || level.dimension() != Level.OVERWORLD)
            return;
        if (!(entity instanceof Player || entity instanceof VehicleEntity || entity.getTags().contains(PROBE_TAG)))
            return;
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g))
            return;
        double dv = nudge(Math.abs(latitudeDeg(g.dataRef().circumferenceBlocks(), entity.getZ())));
        if (dv == 0)
            return;
        entity.push(0, 0, entity.getZ() > 0 ? -dv : dv);
        if (entity instanceof ServerPlayer)
            entity.hurtMarked = true;   // send the new velocity to the client
    }
}
