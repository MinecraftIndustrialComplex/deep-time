// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.sable;

import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
import dev.ryanhcode.sable.api.physics.handle.RigidBodyHandle;
import dev.ryanhcode.sable.api.sublevel.ServerSubLevelContainer;
import dev.ryanhcode.sable.api.sublevel.SubLevelContainer;
import dev.ryanhcode.sable.companion.math.BoundingBox3dc;
import dev.ryanhcode.sable.sublevel.ServerSubLevel;
import net.minecraft.server.level.ServerLevel;
import org.joml.Vector3d;
import org.joml.Vector3dc;

/**
 * The pole cap for Sable ships (plan §7.11): wind toward the equator on every sub-level poleward of
 * the cap line, and a hard stop at the stop line. Called by {@code wrap/PoleCap} after each tick of
 * a Deep Time level, only when Sable is loaded; like {@link SableTestOps} it names Sable types only
 * inside its bodies, and its signature holds Deep Time and vanilla types, so nothing loads it (or
 * Sable) without Sable.
 *
 * <p><b>Wind</b>: {@link PoleCapCurve#wind} (up to {@value PoleCapCurve#WIND_MAX} blocks/s added per
 * tick at the stop line) through {@code RigidBodyHandle.addLinearAndAngularVelocity}, taken at the
 * sub-level's pose (its centre). <b>Hard stop</b>: when the hull's poleward edge (its global bounds)
 * is past the stop line, the poleward part of its velocity is cancelled and it gets the full wind,
 * every tick until it is back inside. Velocity only, never a teleport: a teleport would leave the
 * players standing on the deck behind.
 */
public final class SablePoleCap {

    private SablePoleCap() {}

    /** One tick; returns how many sub-levels the wind or the stop acted on. */
    public static int tick(ServerLevel level, PoleCapCurve cap) {
        ServerSubLevelContainer container = SubLevelContainer.getContainer(level);
        if (container == null)
            return 0;
        int acted = 0;
        for (ServerSubLevel s : container.getAllSubLevels()) {
            if (s.isRemoved())
                continue;
            double z = s.logicalPose().position().z();
            double edge = polewardEdge(s, z);
            double wind = cap.wind(z);
            boolean stop = cap.pastStop(edge);
            if (wind == 0 && !stop)
                continue;
            RigidBodyHandle body = RigidBodyHandle.of(s);
            if (body == null || !body.isValid())
                continue;
            double dir = PoleCapCurve.equatorward(z);
            double dvz = dir * wind;
            if (stop) {
                Vector3dc v = body.getLinearVelocity();
                if (v.z() * dir < 0)
                    dvz -= v.z();           // no poleward velocity left
                dvz += dir * (PoleCapCurve.WIND_MAX - wind);   // the full wind, whatever the centre's latitude
            }
            body.addLinearAndAngularVelocity(new Vector3d(0, 0, dvz), new Vector3d());
            acted++;
        }
        return acted;
    }

    /**
     * One tick of the temporary edge border of a Petroff–Guyou world ({@code wrap/EdgeBorder}): a hull
     * whose outer edge is past |z| = {@code stopZ} loses its outward velocity and is blown back at
     * {@link PoleCapCurve#WIND_MAX} per tick, like the pole cap's hard stop. Returns how many it acted on.
     */
    public static int tickEdge(ServerLevel level, double stopZ) {
        ServerSubLevelContainer container = SubLevelContainer.getContainer(level);
        if (container == null)
            return 0;
        int acted = 0;
        for (ServerSubLevel s : container.getAllSubLevels()) {
            if (s.isRemoved())
                continue;
            double z = s.logicalPose().position().z();
            if (Math.abs(polewardEdge(s, z)) <= stopZ)
                continue;
            RigidBodyHandle body = RigidBodyHandle.of(s);
            if (body == null || !body.isValid())
                continue;
            double dir = PoleCapCurve.equatorward(z);
            double dvz = dir * PoleCapCurve.WIND_MAX;
            Vector3dc v = body.getLinearVelocity();
            if (v.z() * dir < 0)
                dvz -= v.z();
            body.addLinearAndAngularVelocity(new Vector3d(0, 0, dvz), new Vector3d());
            acted++;
        }
        return acted;
    }

    /**
     * The hull's z furthest toward its pole: the global bounds' edge on the pose's side of the
     * equator, or the pose itself when the bounds look unset (Sable fills them after the first
     * physics step).
     */
    static double polewardEdge(ServerSubLevel s, double z) {
        BoundingBox3dc b = s.boundingBox();
        if (b == null || !(b.maxZ() >= b.minZ()) || b.maxZ() - b.minZ() > 4096 || z < b.minZ() - 1 || z > b.maxZ() + 1)
            return z;
        return z >= 0 ? Math.max(z, b.maxZ()) : Math.min(z, b.minZ());
    }
}
