// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.client;

import com.minecraftindustrialcomplex.deeptime.net.ClientPlanetState;
import com.minecraftindustrialcomplex.deeptime.net.PoleStormPayload;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
import com.mojang.blaze3d.shaders.FogShape;
import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.particles.ParticleTypes;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.material.FogType;
import net.minecraft.world.phys.Vec3;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.client.event.ViewportEvent;

/**
 * The polar storm on the client (plan §7.11): the wind the server asks for, fog and driven snow.
 *
 * <ul>
 *   <li><b>Wind.</b> Every client tick, before the player moves, {@link PoleStormPayload#push()} is
 *       added toward the equator to the local player's velocity, or to the vehicle it steers. The
 *       server sends 0 when it does not want the client to push (creative, spectator, a passenger
 *       of a server-moved vehicle), and enforces the hard stop itself.</li>
 *   <li><b>Storm.</b> The drawn intensity eases toward the server's value by at most
 *       {@value #EASE} per tick (full in 2.5 s), so a teleport or a level change never snaps.</li>
 *   <li><b>Fog.</b> With the camera in air, the fog's far plane goes from the render distance down to
 *       {@value #STORM_FAR} blocks at full storm (a white-out), its near plane to 0, and its colour
 *       toward a pale grey.</li>
 *   <li><b>Snow.</b> Up to {@value #SNOW_PER_TICK} snowflake particles a tick in a 24-block box
 *       around the camera, blown toward the equator.</li>
 * </ul>
 */
public final class PolarStorm {

    static final float EASE = 0.02f;
    static final float STORM_FAR = 20f;
    static final float SNOW_PER_TICK = 16f;
    /** Fog colour at full storm (a pale blue-grey), and how far the colour goes toward it. */
    static final float FOG_R = 0.80f, FOG_G = 0.84f, FOG_B = 0.88f, FOG_MIX = 0.85f;

    private static final RandomSource RANDOM = RandomSource.create();
    private static float shown;

    private PolarStorm() {}

    /** The drawn storm intensity 0…1. */
    public static float shown() {
        return shown;
    }

    static void reset() {
        shown = 0f;
    }

    static void onClientTick(ClientTickEvent.Pre event) {
        Minecraft mc = Minecraft.getInstance();
        LocalPlayer player = mc.player;
        if (player == null || mc.level == null || mc.isPaused())
            return;
        boolean here = DeepTimeClient.onPlanet(mc, ClientPlanetState.worldInfo());
        PoleStormPayload storm = here ? ClientPlanetState.storm() : PoleStormPayload.CALM;
        shown += Mth.clamp(storm.storm() - shown, -EASE, EASE);
        if (shown < 1e-4f)
            shown = 0f;
        if (here && storm.push() > 0f)
            wind(player, storm.push());
        if (shown > 0.01f)
            snow(mc, mc.level, shown);
    }

    private static void wind(LocalPlayer player, float push) {
        Entity root = player.getRootVehicle();
        Entity target = root == player ? player : root.isControlledByLocalInstance() ? root : null;
        if (target == null)
            return;   // a passenger of something the server moves: the server pushes that
        double dir = PoleCapCurve.equatorward(target.getZ());
        target.setDeltaMovement(target.getDeltaMovement().add(0, 0, dir * push));
    }

    private static void snow(Minecraft mc, ClientLevel level, float k) {
        Vec3 c = mc.gameRenderer.getMainCamera().getPosition();
        double dir = PoleCapCurve.equatorward(c.z);
        float want = k * SNOW_PER_TICK;
        int n = (int) want + (RANDOM.nextFloat() < want - (int) want ? 1 : 0);
        for (int i = 0; i < n; i++) {
            double x = c.x + (RANDOM.nextDouble() - 0.5) * 24;
            double y = c.y + (RANDOM.nextDouble() - 0.3) * 12;
            double z = c.z + (RANDOM.nextDouble() - 0.5) * 24;
            level.addParticle(ParticleTypes.SNOWFLAKE, x, y, z,
                    (RANDOM.nextDouble() - 0.5) * 0.05, -0.05 - 0.1 * k, dir * (0.15 + 0.35 * k));
        }
    }

    static void onRenderFog(ViewportEvent.RenderFog event) {
        float k = shown;
        if (k <= 0.01f || event.getType() != FogType.NONE)
            return;
        float far = event.getFarPlaneDistance();
        float target = Mth.lerp(k, far, STORM_FAR);
        if (target >= far)
            return;
        event.setFarPlaneDistance(target);
        event.setNearPlaneDistance(Math.min(Mth.lerp(k, event.getNearPlaneDistance(), 0f), target * 0.9f));
        event.setFogShape(FogShape.SPHERE);
        event.setCanceled(true);   // NeoForge applies the changed planes only when the event is cancelled
    }

    static void onFogColor(ViewportEvent.ComputeFogColor event) {
        float k = shown * FOG_MIX;
        if (k <= 0.01f)
            return;
        event.setRed(Mth.lerp(k, event.getRed(), FOG_R));
        event.setGreen(Mth.lerp(k, event.getGreen(), FOG_G));
        event.setBlue(Mth.lerp(k, event.getBlue(), FOG_B));
    }
}
