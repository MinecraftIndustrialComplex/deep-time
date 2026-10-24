// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.sable;

import com.minecraftindustrialcomplex.deeptime.wrap.CubeBorder;
import com.minecraftindustrialcomplex.deeptime.wrap.EdgeBorder;
import dev.ryanhcode.sable.api.physics.handle.RigidBodyHandle;
import dev.ryanhcode.sable.api.sublevel.ServerSubLevelContainer;
import dev.ryanhcode.sable.api.sublevel.SubLevelContainer;
import dev.ryanhcode.sable.companion.math.BoundingBox3dc;
import dev.ryanhcode.sable.sublevel.ServerSubLevel;
import net.minecraft.server.level.ServerLevel;
import org.joml.Vector3d;
import org.joml.Vector3dc;

/**
 * The cube world's temporary border for Sable ships (cube plan W3; {@code wrap/EdgeBorder}): a hull
 * whose bounds leave the planet loses its outward velocity and is blown back, every tick until it is
 * back inside. Called by {@code EdgeBorder} after each tick of a cube level, only when Sable is
 * loaded; like {@link SableTestOps} it names Sable types only inside its bodies, and its signature
 * holds Deep Time and vanilla types, so nothing loads it (or Sable) without Sable. Velocity only,
 * never a teleport: a teleport would leave the players standing on the deck behind.
 *
 * <p>Named {@code SablePoleCap} until the pole cap went with the Lambert world (cube plan §7.4,
 * decisions log 2026-10-24).
 */
public final class SableBorder {

    private SableBorder() {}

    /**
     * One tick of a cube world's temporary border ({@code wrap/EdgeBorder}, cube plan W3): a hull whose
     * footprint (its global bounds' four corners) leaves A({@code inset}), the planet the inset inside
     * every turning edge ({@link CubeBorder}), loses the part of its velocity that points out of the
     * planet and is blown back at {@link EdgeBorder#SABLE_PUSH} per tick ({@link CubeBorder#stopVelocity}).
     * The way out is the one {@link CubeBorder#outward} gives: across the segment, or diagonally out of
     * a void square by F0's corners. The slide is open (the band runs on through it). Returns how many
     * it acted on.
     */
    public static int tickCube(ServerLevel level, CubeBorder border, double inset) {
        ServerSubLevelContainer container = SubLevelContainer.getContainer(level);
        if (container == null)
            return 0;
        int acted = 0;
        for (ServerSubLevel s : container.getAllSubLevels()) {
            if (s.isRemoved())
                continue;
            double[] out = border.outward(footprint(s), inset);
            if (out == null)
                continue;
            RigidBodyHandle body = RigidBodyHandle.of(s);
            if (body == null || !body.isValid())
                continue;
            Vector3dc v = body.getLinearVelocity();
            double[] dv = CubeBorder.stopVelocity(v.x(), v.z(), out, EdgeBorder.SABLE_PUSH);
            body.addLinearAndAngularVelocity(new Vector3d(dv[0], 0, dv[1]), new Vector3d());
            acted++;
        }
        return acted;
    }

    /**
     * The hull's footprint in x and z: its global bounds' four corners {x0, z0, x1, z1, ...}, or its
     * pose alone when the bounds look unset (Sable fills them after the first physics step).
     */
    static double[] footprint(ServerSubLevel s) {
        Vector3dc p = s.logicalPose().position();
        BoundingBox3dc b = s.boundingBox();
        if (b == null || !(b.maxX() >= b.minX()) || !(b.maxZ() >= b.minZ()) || b.maxX() - b.minX() > 4096
                || b.maxZ() - b.minZ() > 4096 || p.x() < b.minX() - 1 || p.x() > b.maxX() + 1
                || p.z() < b.minZ() - 1 || p.z() > b.maxZ() + 1)
            return new double[]{p.x(), p.z()};
        return new double[]{b.minX(), b.minZ(), b.maxX(), b.minZ(), b.minX(), b.maxZ(), b.maxX(), b.maxZ()};
    }
}
