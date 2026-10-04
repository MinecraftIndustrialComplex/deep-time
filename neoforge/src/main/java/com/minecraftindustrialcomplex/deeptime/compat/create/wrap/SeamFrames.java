// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.wrapspike.CellIsometry;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapEngine;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapTopology;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.block.state.BlockState;

/**
 * The one place the Create adapters touch the wrap engine (docs/design/wrap-engine.md, "Create across
 * turning seams"). Everything here is a question about frames: block logic runs at stored positions,
 * each block entity in its own stored frame, and code that walks from block to block (Create's rotation
 * propagator, pipe searches, pump pressure) walks in the frame of the block it started from, through
 * band positions that the folded access answers turned. A direction handed from the walker's frame to a
 * block entity it reached through a band position must be turned into that block entity's frame.
 *
 * <p>Built on the spike's fold today ({@link WrapEngine#topology}, {@link WrapTopology#fold},
 * {@link WrapTopology#images}). When Phase 1's folded-access API lands, only this class changes.
 */
public final class SeamFrames {

    private SeamFrames() {}

    /** The level's turning topology, or null (no engine, no topology, or a client level). */
    static WrapTopology topology(Level level) {
        if (level == null || level.isClientSide)
            return null;
        return WrapEngine.topology(level);
    }

    static WrapTopology topology(BlockGetter getter) {
        return getter instanceof Level level ? topology(level) : null;
    }

    /** True when the level has a wrap topology installed (server side). */
    public static boolean active(Level level) {
        return topology(level) != null;
    }

    /** The motion from a position beyond an edge to its stored block, or null for a stored position. */
    public static CellIsometry fold(BlockGetter getter, BlockPos pos) {
        WrapTopology t = topology(getter);
        if (t == null)
            return null;
        WrapTopology.Fold f = t.fold(pos.getX(), pos.getZ());
        return f == null ? null : f.iso();
    }

    /** The stored position a position shows (itself when stored). */
    public static BlockPos stored(BlockGetter getter, BlockPos pos) {
        CellIsometry iso = fold(getter, pos);
        return iso == null ? pos : WrapEngine.apply(iso, pos);
    }

    /**
     * A direction in the frame of the code that reached {@code fetchedAt} (a neighbour position, maybe a
     * band position), turned into the frame of the block entity stored there.
     */
    public static Direction toStoredFrame(BlockGetter getter, BlockPos fetchedAt, Direction d) {
        if (d == null)
            return null;
        CellIsometry iso = fold(getter, fetchedAt);
        return iso == null ? d : WrapEngine.rotation(iso).rotate(d);
    }

    /**
     * A stored block seen from another stored block across a seam: where it appears to the viewer
     * ({@code viewPos}, a band position next to the viewer) and the motion that takes that band
     * position to the stored block ({@code toStored}).
     */
    public record Link(BlockPos viewPos, CellIsometry toStored) {
        /** The rotation that turns the viewer's frame into the target's stored frame. */
        public Rotation intoTarget() {
            return WrapEngine.rotation(toStored);
        }

        /** The target's stored state as the viewer sees it (turned back). */
        public BlockState view(BlockState stored) {
            return stored.rotate(WrapEngine.rotation(toStored.inverse()));
        }
    }

    /**
     * The band image of the stored block {@code target} that lies within {@code reach} blocks of the
     * stored block {@code viewer} (Chebyshev distance horizontally, y equal within {@code reach}), the
     * nearest by Manhattan distance; null when there is none or no topology. Rotation propagation and
     * pipe searches only ever step to such neighbours.
     */
    public static Link link(Level level, BlockPos viewer, BlockPos target, int reach) {
        WrapTopology t = topology(level);
        if (t == null || Math.abs(target.getY() - viewer.getY()) > reach)
            return null;
        int[] q = SeamTurns.nearestImage(t, viewer.getX(), viewer.getZ(), target.getX(), target.getZ(), reach);
        if (q == null)
            return null;
        WrapTopology.Fold f = t.fold(q[0], q[1]);
        return f == null ? null : new Link(new BlockPos(q[0], target.getY(), q[1]), f.iso());
    }

    /** True when (x, z) lies within {@code margin} blocks of an edge, on either side (a cheap pre-check). */
    public static boolean nearEdge(Level level, BlockPos pos, int margin) {
        WrapTopology t = topology(level);
        if (t == null)
            return false;
        for (int dx = -margin; dx <= margin; dx += margin)
            for (int dz = -margin; dz <= margin; dz += margin)
                if (t.fold(pos.getX() + dx, pos.getZ() + dz) != null)
                    return true;
        return false;
    }

    /** True when any block of the box (inclusive corners) lies beyond an edge. */
    public static boolean anyBeyond(Level level, BlockPos a, BlockPos b) {
        WrapTopology t = topology(level);
        if (t == null)
            return false;
        int x0 = Math.min(a.getX(), b.getX()), x1 = Math.max(a.getX(), b.getX());
        int z0 = Math.min(a.getZ(), b.getZ()), z1 = Math.max(a.getZ(), b.getZ());
        if ((long) (x1 - x0 + 1) * (z1 - z0 + 1) > 1 << 16)
            return true; // conservative: a box this large on a wrapped level counts as crossing
        for (int x = x0; x <= x1; x++)
            for (int z = z0; z <= z1; z++)
                if (t.fold(x, z) != null)
                    return true;
        return false;
    }
}
